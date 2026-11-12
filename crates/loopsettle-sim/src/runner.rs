//! Running a scenario through several engines in one streaming pass, and summarising.

use crate::scenario::ScenarioConfig;
use crate::stream::{Outage, PaymentStream};
use loopsettle_core::json::Json;
use loopsettle_core::*;

/// Which engine to build. Parsed from strings such as `gross`, `deferred:10`,
/// `micro:all:1000:5000` (mode, window_ms, max_delay_ms), optionally followed by the maximum
/// cycle length and, for `shortfall`, the queue discipline: `micro:shortfall:500:10000:6:fifo`.
#[derive(Clone, Debug, PartialEq)]
pub enum EngineSpec {
    Gross,
    Deferred { cycles_per_day: u32 },
    Micro(MicroCycleSpec),
    /// `hold:<delay_ms>[:<participant>]`: hold every payment (or only one participant's) for
    /// exactly the delay, then settle gross. No netting.
    Hold { delay_ms: TimeMs, only: Option<u32> },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MicroCycleSpec {
    pub mode: QueueMode,
    pub window_ms: TimeMs,
    pub max_delay_ms: TimeMs,
    pub max_cycle_len: usize,
    /// gross-retry order in `shortfall` mode (`bypass` by default, or `fifo`)
    pub discipline: QueueDiscipline,
    /// `false` for the plain queue (`queue:fifo|bypass:<max_delay_ms>[:<retry_ms>]`)
    pub search_cycles: bool,
    /// multilateral gridlock resolution at every window (`gridlock:fifo|bypass:<window>:<delay>`)
    pub gridlock: bool,
}

impl EngineSpec {
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split(':').collect();
        let num = |i: usize| -> Result<u64, String> { parts.get(i).ok_or(format!("engine '{s}': missing field {i}"))?.parse().map_err(|e| format!("engine '{s}': {e}")) };
        match parts[0] {
            "gross" => Ok(EngineSpec::Gross),
            "deferred" => Ok(EngineSpec::Deferred { cycles_per_day: num(1)? as u32 }),
            "hold" => {
                if parts.len() > 3 {
                    return Err(format!("engine '{s}': expected hold:<delay_ms>[:<participant>]"));
                }
                Ok(EngineSpec::Hold { delay_ms: num(1)?, only: if parts.len() > 2 { Some(num(2)? as u32) } else { None } })
            }
            // Plain queue: settle gross when funds allow, otherwise wait (retried every
            // `retry_ms`, 100 ms by default) and reject at the deadline. No netting.
            "queue" => {
                let discipline = match parts.get(1).copied() {
                    Some("fifo") => QueueDiscipline::StrictFifoPerPayer,
                    Some("bypass") => QueueDiscipline::BypassFifo,
                    _ => return Err(format!("engine '{s}': discipline must be fifo|bypass")),
                };
                let window_ms = if parts.len() > 3 { num(3)? } else { 100 };
                if parts.len() > 4 || window_ms == 0 {
                    return Err(format!("engine '{s}': expected queue:fifo|bypass:<max_delay_ms>[:<retry_ms>]"));
                }
                Ok(EngineSpec::Micro(MicroCycleSpec { mode: QueueMode::QueueOnShortfall, window_ms, max_delay_ms: num(2)?, max_cycle_len: 2, discipline, search_cycles: false, gridlock: false }))
            }
            // Queue + multilateral gridlock resolution every window (Bech-Soramäki style).
            "gridlock" => {
                let discipline = match parts.get(1).copied() {
                    Some("fifo") => QueueDiscipline::StrictFifoPerPayer,
                    Some("bypass") => QueueDiscipline::BypassFifo,
                    _ => return Err(format!("engine '{s}': expected gridlock:fifo|bypass:<window_ms>:<max_delay_ms>")),
                };
                if parts.len() != 4 || num(2)? == 0 {
                    return Err(format!("engine '{s}': expected gridlock:fifo|bypass:<window_ms>:<max_delay_ms>"));
                }
                Ok(EngineSpec::Micro(MicroCycleSpec { mode: QueueMode::QueueOnShortfall, window_ms: num(2)?, max_delay_ms: num(3)?, max_cycle_len: 2, discipline, search_cycles: false, gridlock: true }))
            }
            // Bilateral offsetting only: the micro-cycle engine restricted to cycles of two.
            "bilateral" => {
                if parts.len() != 4 {
                    return Err(format!("engine '{s}': expected bilateral:all|shortfall:<window_ms>:<max_delay_ms>"));
                }
                Self::parse(&format!("micro:{}:{}:{}:2", parts[1], parts[2], parts[3]))
            }
            "micro" => {
                let mode = match parts.get(1).copied() {
                    Some("all") => QueueMode::QueueAll,
                    Some("shortfall") => QueueMode::QueueOnShortfall,
                    _ => return Err(format!("engine '{s}': mode must be all|shortfall")),
                };
                let max_cycle_len = if parts.len() > 4 { num(4)? as usize } else { 6 };
                let discipline = match parts.get(5).copied() {
                    None | Some("bypass") => QueueDiscipline::BypassFifo,
                    Some("fifo") => QueueDiscipline::StrictFifoPerPayer,
                    Some(x) => return Err(format!("engine '{s}': discipline must be fifo|bypass, got '{x}'")),
                };
                if parts.len() > 6 {
                    return Err(format!("engine '{s}': too many fields"));
                }
                Ok(EngineSpec::Micro(MicroCycleSpec { mode, window_ms: num(2)?, max_delay_ms: num(3)?, max_cycle_len, discipline, search_cycles: true, gridlock: false }))
            }
            _ => Err(format!("unknown engine '{s}'")),
        }
    }

    pub fn build(&self, n: usize) -> Box<dyn SettlementEngine> {
        match *self {
            EngineSpec::Gross => Box::new(GrossRealTime),
            EngineSpec::Deferred { cycles_per_day } => Box::new(DeferredNet::new(cycles_per_day, n)),
            EngineSpec::Hold { delay_ms, only } => Box::new(HoldToDeadline::new(delay_ms, only)),
            EngineSpec::Micro(m) => {
                let mut c = MicroCycleConfig::new(m.window_ms, m.max_delay_ms, m.mode);
                c.max_cycle_len = m.max_cycle_len;
                c.discipline = m.discipline;
                c.search_cycles = m.search_cycles;
                c.gridlock = m.gridlock;
                Box::new(AtomicMicroCycle::new(c, n))
            }
        }
    }

    pub fn to_json(&self) -> Json {
        match *self {
            EngineSpec::Gross => Json::obj().set("kind", "gross"),
            EngineSpec::Deferred { cycles_per_day } => Json::obj().set("kind", "deferred").set("cycles_per_day", cycles_per_day),
            EngineSpec::Hold { delay_ms, only } => {
                let j = Json::obj().set("kind", "hold").set("delay_ms", delay_ms);
                match only {
                    Some(i) => j.set("only_participant", i),
                    None => j,
                }
            }
            EngineSpec::Micro(m) => Json::obj()
                .set("kind", if m.gridlock { "gridlock" } else if m.search_cycles { "micro" } else { "queue" })
                .set("mode", if m.mode == QueueMode::QueueAll { "all" } else { "shortfall" })
                .set("window_ms", m.window_ms)
                .set("max_delay_ms", m.max_delay_ms)
                .set("max_cycle_len", m.max_cycle_len)
                .set("discipline", if m.discipline == QueueDiscipline::StrictFifoPerPayer { "fifo" } else { "bypass" }),
        }
    }
}

/// Opening balances.
#[derive(Clone, Debug)]
pub enum Opening {
    /// `UNLIMITED` for everybody: measure the liquidity each engine needs.
    Unlimited,
    /// Explicit per-participant balances in cents (constrained liquidity).
    Given(Vec<Cents>),
}

#[derive(Clone, Copy, Debug)]
pub struct OutageSpec {
    pub participant: u32,
    pub start_ms: TimeMs,
    pub duration_ms: TimeMs,
}

/// Summary of one engine run.
#[derive(Clone, Debug)]
pub struct EngineResult {
    pub spec: EngineSpec,
    pub name: String,
    /// liquidity needed (sum over participants of peak drawdown), cents
    pub liquidity: i128,
    pub liquidity_by_participant: Vec<Cents>,
    pub metrics: RunMetrics,
    pub stats: Vec<(&'static str, f64)>,
}

impl EngineResult {
    pub fn delay_q(&self, q: f64) -> u64 {
        self.metrics.delay.quantile(q)
    }
    pub fn saving_vs(&self, ub: i128) -> f64 {
        if ub == 0 { 0.0 } else { 1.0 - self.liquidity as f64 / ub as f64 }
    }
    pub fn to_json(&self, ub: i128, lb: i128) -> Json {
        let m = &self.metrics;
        let mut j = Json::obj()
            .set("name", self.name.clone())
            .set("spec", self.spec.to_json())
            .set("liquidity_needed_brl", self.liquidity as f64 / 100.0)
            .set("liquidity_saving_vs_ub_brl", (ub - self.liquidity) as f64 / 100.0)
            .set("liquidity_saving_vs_ub_pct", 100.0 * self.saving_vs(ub))
            .set("share_of_ub_minus_lb_recovered", if ub > lb { (ub - self.liquidity) as f64 / (ub - lb) as f64 } else { 0.0 })
            .set("settled_count", m.settled_count)
            .set("settled_value_brl", m.settled_value as f64 / 100.0)
            .set("settled_gross_value_brl", m.value_by_kind[0] as f64 / 100.0)
            .set("settled_cycle_value_brl", m.value_by_kind[1] as f64 / 100.0)
            .set("settled_deferred_value_brl", m.value_by_kind[2] as f64 / 100.0)
            .set("settled_cycle_count", m.count_by_kind[1])
            .set("rejected_count", m.rejected_count)
            .set("rejected_value_brl", m.rejected_value as f64 / 100.0)
            .set("delay_p50_ms", self.delay_q(0.50))
            .set("delay_p95_ms", self.delay_q(0.95))
            .set("delay_p99_ms", self.delay_q(0.99))
            .set("delay_max_ms", m.delay.max())
            .set("delay_mean_ms", m.delay.mean())
            .set("delay_value_weighted_mean_ms", m.value_weighted_mean_delay_ms())
            .set("share_settled_within_2800ms", m.share_within_ms(SLA_PIX_MS).unwrap_or_else(|| m.delay.fraction_at_most(SLA_PIX_MS)));
        for (k, v) in &self.stats {
            j.push(k, *v);
        }
        j
    }
}

pub struct RunOutput {
    pub flow: FlowStats,
    pub results: Vec<EngineResult>,
    pub outage_deferred: Option<(u64, i128, u128)>,
}

impl RunOutput {
    pub fn ub(&self) -> i128 {
        self.flow.ub_total()
    }
    pub fn lb(&self) -> i128 {
        self.flow.lb_total()
    }
}

/// Settle any payment stream (sorted by arrival time) in every engine, in one pass.
pub fn run_payments<I: Iterator<Item = Payment>>(n: usize, payments: I, specs: &[EngineSpec], opening: &Opening, t_end: TimeMs) -> RunOutput {
    let mut runs: Vec<EngineRun> = specs
        .iter()
        .map(|s| {
            let ledger = match opening {
                Opening::Unlimited => Ledger::unlimited(n),
                Opening::Given(v) => Ledger::new(v.clone()),
            };
            EngineRun::new(s.build(n), ledger)
        })
        .collect();
    let mut flow = FlowStats::new(n, false);
    drive(payments, &mut runs, &mut flow, t_end);
    let results = specs
        .iter()
        .zip(runs)
        .map(|(spec, r)| {
            let dd = r.ledger.peak_drawdowns();
            EngineResult { spec: spec.clone(), name: r.engine.name(), liquidity: dd.iter().map(|&x| x as i128).sum(), liquidity_by_participant: dd, stats: r.engine.stats(), metrics: r.metrics }
        })
        .collect();
    RunOutput { flow, results, outage_deferred: None }
}

/// Generate the scenario's payments and settle them in every engine, in one pass.
pub fn run(cfg: &ScenarioConfig, specs: &[EngineSpec], opening: &Opening, with_pairs: bool, outage: Option<OutageSpec>) -> RunOutput {
    let n = cfg.participants;
    let mut runs: Vec<EngineRun> = specs
        .iter()
        .map(|s| {
            let ledger = match opening {
                Opening::Unlimited => Ledger::unlimited(n),
                Opening::Given(v) => Ledger::new(v.clone()),
            };
            EngineRun::new(s.build(n), ledger)
        })
        .collect();
    let mut flow = FlowStats::new(n, with_pairs);
    let stream = PaymentStream::new(cfg);
    let outage_deferred = match outage {
        None => {
            drive(stream, &mut runs, &mut flow, cfg.horizon_ms());
            None
        }
        Some(o) => {
            let mut s = Outage::new(stream, o.participant, o.start_ms, o.duration_ms);
            drive(&mut s, &mut runs, &mut flow, cfg.horizon_ms());
            Some((s.deferred_count, s.deferred_value, s.deferral_ms_sum))
        }
    };
    let results = specs
        .iter()
        .zip(runs)
        .map(|(spec, r)| {
            let dd = r.ledger.peak_drawdowns();
            EngineResult {
                spec: spec.clone(),
                name: r.engine.name(),
                liquidity: dd.iter().map(|&x| x as i128).sum(),
                liquidity_by_participant: dd,
                stats: r.engine.stats(),
                metrics: r.metrics,
            }
        })
        .collect();
    RunOutput { flow, results, outage_deferred }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_specs() {
        assert_eq!(EngineSpec::parse("gross").unwrap(), EngineSpec::Gross);
        assert_eq!(EngineSpec::parse("deferred:10").unwrap(), EngineSpec::Deferred { cycles_per_day: 10 });
        match EngineSpec::parse("micro:shortfall:500:2000").unwrap() {
            EngineSpec::Micro(m) => assert!(m.mode == QueueMode::QueueOnShortfall && m.window_ms == 500 && m.max_delay_ms == 2000),
            _ => panic!(),
        }
        assert!(EngineSpec::parse("micro:x:1:1").is_err());
        match EngineSpec::parse("micro:shortfall:500:2000:4:fifo").unwrap() {
            EngineSpec::Micro(m) => assert!(m.max_cycle_len == 4 && m.discipline == QueueDiscipline::StrictFifoPerPayer),
            _ => panic!(),
        }
        assert!(EngineSpec::parse("micro:shortfall:500:2000:4:lifo").is_err());
        match EngineSpec::parse("queue:fifo:10000").unwrap() {
            EngineSpec::Micro(m) => assert!(!m.search_cycles && m.window_ms == 100 && m.max_delay_ms == 10_000 && m.discipline == QueueDiscipline::StrictFifoPerPayer),
            _ => panic!(),
        }
        assert!(EngineSpec::parse("queue:lifo:10").is_err() && EngineSpec::parse("queue:fifo:10:0").is_err());
    }

    /// The plain queue is gross settlement when nobody is short of funds: with unlimited
    /// liquidity, and with opening balances equal to each participant's UB.
    #[test]
    fn queue_equals_gross_when_liquidity_suffices() {
        let cfg = ScenarioConfig { participants: 25, payments_per_day: 25_000.0, n_big: 3, n_mid: 5, ..Default::default() };
        let specs = [EngineSpec::Gross, EngineSpec::parse("queue:fifo:10000").unwrap(), EngineSpec::parse("queue:bypass:10000").unwrap()];
        let a = run(&cfg, &specs, &Opening::Unlimited, false, None);
        let b = run(&cfg, &specs, &Opening::Given(a.flow.ubs()), false, None);
        for o in [&a, &b] {
            for r in &o.results[1..] {
                assert_eq!(r.liquidity_by_participant, o.results[0].liquidity_by_participant, "{}", r.name);
                assert_eq!((r.metrics.rejected_count, r.metrics.delay.max(), r.metrics.count_by_kind[1]), (0, 0, 0), "{}", r.name);
            }
        }
    }

    /// With too little liquidity the queue never settles in cycles and gives every payment
    /// exactly one outcome. It does **not** have to reject less than gross settlement: in this
    /// scenario it rejects slightly more (159.72 M against 159.59 M of 623 M), because the
    /// shortfall lasts hours, not seconds, and a payment released late uses funds that gross
    /// settlement would have given to another payment. So nothing is asserted about the order
    /// of the two rejection totals; the liquidity-frontier experiment measures it.
    #[test]
    fn queue_under_constrained_liquidity_is_consistent() {
        let cfg = ScenarioConfig { participants: 25, payments_per_day: 25_000.0, n_big: 3, n_mid: 5, ..Default::default() };
        let specs = [EngineSpec::Gross, EngineSpec::parse("queue:fifo:10000").unwrap(), EngineSpec::parse("queue:bypass:10000").unwrap()];
        let ubs = run(&cfg, &specs[..1], &Opening::Unlimited, false, None).flow.ubs();
        let half: Vec<Cents> = ubs.iter().map(|u| u / 2).collect();
        let o = run(&cfg, &specs, &Opening::Given(half), false, None);
        assert!(o.results[0].metrics.rejected_value > 0);
        for r in &o.results {
            let m = &r.metrics;
            assert_eq!(m.settled_count + m.rejected_count, o.flow.count, "{}", r.name);
            assert_eq!(m.settled_value + m.rejected_value, o.flow.gross_value, "{}", r.name);
            assert_eq!(m.count_by_kind[1], 0, "{}", r.name);
            assert!(m.delay.max() <= 10_000, "{}", r.name);
            assert!(r.liquidity_by_participant.iter().zip(&ubs).all(|(l, u)| *l <= u / 2), "{}: no account goes below zero", r.name);
        }
        assert!(o.results[1].metrics.delay.max() > 0, "some payment waited in the queue");
    }

    #[test]
    fn gross_unlimited_equals_ub_and_runs_are_identical() {
        let cfg = ScenarioConfig { participants: 30, payments_per_day: 30_000.0, n_big: 3, n_mid: 5, ..Default::default() };
        let specs = [EngineSpec::Gross, EngineSpec::Deferred { cycles_per_day: 1 }, EngineSpec::parse("micro:all:1000:5000").unwrap()];
        let a = run(&cfg, &specs, &Opening::Unlimited, false, None);
        assert_eq!(a.results[0].liquidity, a.ub());
        assert_eq!(a.results[1].liquidity, a.lb());
        let b = run(&cfg, &specs, &Opening::Unlimited, false, None);
        for (x, y) in a.results.iter().zip(&b.results) {
            assert_eq!(x.liquidity_by_participant, y.liquidity_by_participant);
            assert_eq!(x.metrics.delay.quantile(0.95), y.metrics.delay.quantile(0.95));
        }
    }
}
