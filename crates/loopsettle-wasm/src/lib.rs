//! # loopsettle-wasm
//!
//! Browser bindings: one function, [`simulate`], runs a small calibrated scenario and
//! returns everything the web UI draws as a JSON string. The heavy lifting is plain Rust
//! ([`simulate_json`]), so it is unit-tested natively; `#[wasm_bindgen]` only exports it.

use loopsettle_core::json::Json;
use loopsettle_core::QueueMode;
use loopsettle_sim::runner::MicroCycleSpec;
use loopsettle_sim::{calibrate, run, toml_lite, EngineSpec, Opening, PaymentStream, Population, ScenarioConfig};
use loopsettle_topology::{analyze, cdfd_max, FlowGraph, HodgeConfig};
use wasm_bindgen::prelude::*;

/// The public-data calibration, embedded at compile time.
const CALIBRATION: &str = include_str!("../../../data/calibration/pix.toml");

pub const MAX_PARTICIPANTS: u32 = 50;
pub const MAX_PAYMENTS: u32 = 100_000;

#[derive(Clone, Debug)]
pub struct Params {
    pub seed: u64,
    pub participants: u32,
    pub payments: u32,
    pub window_ms: u64,
    pub max_delay_ms: u64,
    pub receiver_bias_sigma: f64,
    /// opening balance as a fraction of each participant's UB, for the constrained run
    pub liquidity_fraction: f64,
}

fn scenario(p: &Params) -> ScenarioConfig {
    let n = p.participants.clamp(3, MAX_PARTICIPANTS) as usize;
    let mut cfg = ScenarioConfig {
        seed: p.seed,
        participants: n,
        payments_per_day: p.payments.clamp(1_000, MAX_PAYMENTS) as f64,
        n_big: (n / 10).max(1),
        n_mid: (n * 15 / 100).max(1),
        receiver_bias_sigma: p.receiver_bias_sigma.clamp(0.0, 2.0),
        ..Default::default()
    };
    if let Ok(doc) = toml_lite::parse(CALIBRATION) {
        if let Ok(t) = calibrate::Targets::from_doc(&doc) {
            calibrate::apply(&mut cfg, &t);
        }
    }
    cfg
}

fn micro(mode: QueueMode, w: u64, d: u64) -> EngineSpec {
    EngineSpec::Micro(MicroCycleSpec { mode, window_ms: w.max(1), max_delay_ms: d, max_cycle_len: 6 })
}

pub fn simulate_json(p: &Params) -> Json {
    let cfg = scenario(p);
    let n = cfg.participants;
    let (w, d) = (p.window_ms.max(1), p.max_delay_ms);

    // Pass 1: unlimited liquidity: what each design needs.
    let sweep_windows = [500u64, 1_000, 2_000, 5_000, 10_000];
    let mut specs = vec![EngineSpec::Gross, EngineSpec::Deferred { cycles_per_day: 1 }, EngineSpec::Deferred { cycles_per_day: 96 }, micro(QueueMode::QueueAll, w, d)];
    specs.extend(sweep_windows.iter().map(|&sw| micro(QueueMode::QueueAll, sw, d)));
    let o = run(&cfg, &specs, &Opening::Unlimited, true, None);
    let (ub, lb) = (o.ub(), o.lb());

    // Pass 2: constrained liquidity (fraction of own UB).
    let opening: Vec<i64> = o.flow.ubs().iter().map(|&u| (u as f64 * p.liquidity_fraction.clamp(0.0, 2.0)).floor() as i64).collect();
    let cspecs = [EngineSpec::Gross, micro(QueueMode::QueueOnShortfall, w, d), micro(QueueMode::QueueAll, w, d)];
    let oc = run(&cfg, &cspecs, &Opening::Given(opening), false, None);

    // Topology and the circular component of every edge.
    let g = FlowGraph::gross_from(&o.flow);
    let topo = analyze(&g, &HodgeConfig::default());
    let c = cdfd_max(&g);
    let mut idx: Vec<usize> = (0..g.edges.len()).collect();
    idx.sort_by_key(|&i| std::cmp::Reverse(g.edges[i].flow));
    idx.truncate(250);
    let edges: Vec<Json> = idx
        .iter()
        .map(|&i| {
            let e = g.edges[i];
            Json::Arr(vec![Json::Int(e.from as i128), Json::Int(e.to as i128), Json::Num(e.flow as f64 / 100.0), Json::Num(c.circular[i] as f64 / 100.0)])
        })
        .collect();
    let pop = Population::build(&cfg);
    let nodes: Vec<Json> = (0..n)
        .map(|i| {
            Json::obj()
                .set("id", i)
                .set("class", pop.class[i].name())
                .set("activity", pop.activity[i])
                .set("net_brl", o.flow.net_position(i as u32) as f64 / 100.0)
                .set("ub_brl", o.flow.ub(i as u32) as f64 / 100.0)
                .set("lb_brl", o.flow.lb(i as u32) as f64 / 100.0)
        })
        .collect();

    // Intraday series (15-minute buckets): value and system net debit under gross.
    let buckets = (cfg.horizon_ms() / 900_000) as usize;
    let mut value = vec![0f64; buckets];
    let mut debit = vec![0f64; buckets];
    let mut x = vec![0i64; n];
    let mut b_cur = 0usize;
    let sum_debit = |x: &[i64]| x.iter().map(|&v| (-v).max(0) as f64).sum::<f64>() / 100.0;
    for pmt in PaymentStream::new(&cfg) {
        let b = ((pmt.t_arrival / 900_000) as usize).min(buckets - 1);
        while b_cur < b {
            debit[b_cur] = sum_debit(&x);
            b_cur += 1;
        }
        value[b] += pmt.amount as f64 / 100.0;
        x[pmt.payer as usize] -= pmt.amount;
        x[pmt.payee as usize] += pmt.amount;
    }
    while b_cur < buckets {
        debit[b_cur] = sum_debit(&x);
        b_cur += 1;
    }

    let engines: Vec<Json> = o.results[..4].iter().map(|r| r.to_json(ub, lb)).collect();
    let sweep: Vec<Json> = o.results[4..]
        .iter()
        .map(|r| Json::obj().set("name", r.name.clone()).set("saving_pct", 100.0 * r.saving_vs(ub)).set("p95_ms", r.delay_q(0.95)).set("cycle_value_share", r.metrics.value_by_kind[1] as f64 / r.metrics.settled_value.max(1) as f64))
        .collect();
    let constrained: Vec<Json> = oc.results.iter().map(|r| r.to_json(ub, lb)).collect();

    Json::obj()
        .set("params", Json::obj().set("seed", p.seed).set("participants", n).set("payments_per_day", cfg.payments_per_day).set("window_ms", w).set("max_delay_ms", d).set("receiver_bias_sigma", cfg.receiver_bias_sigma).set("liquidity_fraction", p.liquidity_fraction))
        .set("scale_factor", cfg.scale_factor)
        .set("payments", o.flow.count)
        .set("gross_value_brl", o.flow.gross_value as f64 / 100.0)
        .set("ub_brl", ub as f64 / 100.0)
        .set("lb_brl", lb as f64 / 100.0)
        .set("potential_offset", o.flow.potential_offset_fraction())
        .set("potential_liquidity_saving", if ub == 0 { 0.0 } else { (ub - lb) as f64 / ub as f64 })
        .set("topology", topo.to_json())
        .set("nodes", Json::Arr(nodes))
        .set("edges", Json::Arr(edges))
        .set("engines", Json::Arr(engines))
        .set("sweep", Json::Arr(sweep))
        .set("constrained", Json::Arr(constrained))
        .set("intraday_value_brl", Json::Arr(value.into_iter().map(Json::Num).collect()))
        .set("intraday_net_debit_brl", Json::Arr(debit.into_iter().map(Json::Num).collect()))
}

/// Run a small calibrated simulation and return the results as JSON.
/// `seed` is a JS number (integer part used) to avoid BigInt on the JS side.
#[wasm_bindgen]
pub fn simulate(seed: f64, participants: u32, payments: u32, window_ms: u32, max_delay_ms: u32, receiver_bias_sigma: f64, liquidity_fraction: f64) -> String {
    let p = Params {
        seed: seed.max(0.0) as u64,
        participants,
        payments,
        window_ms: window_ms as u64,
        max_delay_ms: max_delay_ms as u64,
        receiver_bias_sigma,
        liquidity_fraction,
    };
    simulate_json(&p).to_string_compact()
}

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_run_produces_consistent_json() {
        let p = Params { seed: 7, participants: 20, payments: 20_000, window_ms: 1000, max_delay_ms: 5000, receiver_bias_sigma: 0.4, liquidity_fraction: 0.5 };
        let a = simulate_json(&p).to_string_compact();
        let b = simulate_json(&p).to_string_compact();
        assert_eq!(a, b, "deterministic");
        assert!(a.contains("\"engines\"") && a.contains("\"edges\"") && a.contains("\"sweep\""));
    }
}
