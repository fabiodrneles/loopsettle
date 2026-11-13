//! # loopsettle-wasm
//!
//! Browser bindings: one function, [`simulate`], runs a small calibrated scenario and
//! returns everything the web UI draws as a JSON string. The heavy lifting is plain Rust
//! ([`simulate_json`]), so it is unit-tested natively; `#[wasm_bindgen]` only exports it.

use loopsettle_core::json::Json;
use loopsettle_core::QueueMode;
use loopsettle_core::{Payment, TimeMs, MS_PER_DAY};
use loopsettle_sim::bound::{delay_bounds_from, liquidity_floors_from};
use loopsettle_sim::runner::MicroCycleSpec;
use loopsettle_sim::{
    calibrate, run, run_payments, toml_lite, EngineSpec, Opening, PaymentStream, Population,
    ScenarioConfig,
};
use loopsettle_topology::{analyze, cdfd_max, FlowGraph, HodgeConfig};
use wasm_bindgen::prelude::*;

/// The public-data calibration, embedded at compile time.
const CALIBRATION: &str = include_str!("../../../data/calibration/pix.toml");

pub const MAX_PARTICIPANTS: u32 = 50;
pub const MAX_PAYMENTS: u32 = 100_000;
/// Largest file `analyze_file` accepts, in payments (about 32 MB of memory for the payments).
pub const MAX_FILE_PAYMENTS: usize = 2_000_000;
/// Deadlines of the liquidity-floor curve, in ms: 1 s to 1 hour.
const FLOOR_DELAYS_MS: [TimeMs; 7] = [1_000, 2_000, 5_000, 10_000, 60_000, 900_000, 3_600_000];

/// The liquidity floor curve for a payment list (docs/METHODOLOGY.md, sections 9 and 9b): for
/// each deadline d, the most any mechanism with that deadline can save (tight bound, and the
/// looser window form) and what netting every d realises, all in % of UB.
fn floor_curve(n: usize, ps: &[Payment], t_end: TimeMs) -> (Json, i128, i128, Vec<Json>) {
    let dd = FLOOR_DELAYS_MS;
    let mut specs = vec![EngineSpec::Gross];
    specs.extend(dd.iter().map(|&d| EngineSpec::Deferred {
        cycles_per_day: (MS_PER_DAY / d) as u32,
    }));
    specs.push(EngineSpec::parse("micro:all:500:10000").expect("valid spec"));
    let o = run_payments(n, ps.iter().copied(), &specs, &Opening::Unlimited, t_end);
    let (ub, lb) = (o.ub(), o.lb());
    let floors = liquidity_floors_from(n, || ps.iter().copied(), &dd);
    let windows = delay_bounds_from(n, || ps.iter().copied(), &dd);
    let pct = |x: i128| {
        if ub == 0 {
            0.0
        } else {
            100.0 * x as f64 / ub as f64
        }
    };
    let points: Vec<Json> = dd
        .iter()
        .enumerate()
        .map(|(k, &d)| {
            let floor: i128 = floors[k].iter().map(|&v| v as i128).sum();
            let window: i128 = windows[k].iter().map(|&v| v as i128).sum();
            Json::obj()
                .set("delay_s", d as f64 / 1000.0)
                .set("tight_bound_pct", pct(ub - floor))
                .set("window_bound_pct", pct(window).min(100.0))
                .set("netting_saving_pct", 100.0 * o.results[k + 1].saving_vs(ub))
        })
        .collect();
    let engines: Vec<Json> = [0, dd.len() + 1]
        .iter()
        .map(|&k| o.results[k].to_json(ub, lb))
        .collect();
    (Json::Arr(points), ub, lb, engines)
}

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
    EngineSpec::Micro(MicroCycleSpec {
        mode,
        window_ms: w.max(1),
        max_delay_ms: d,
        max_cycle_len: 6,
        discipline: Default::default(),
        search_cycles: true,
        gridlock: false,
    })
}

pub fn simulate_json(p: &Params) -> Json {
    let cfg = scenario(p);
    let n = cfg.participants;
    let (w, d) = (p.window_ms.max(1), p.max_delay_ms);

    // Pass 1: unlimited liquidity: what each design needs.
    let sweep_windows = [500u64, 1_000, 2_000, 5_000, 10_000];
    let mut specs = vec![
        EngineSpec::Gross,
        EngineSpec::Deferred { cycles_per_day: 1 },
        EngineSpec::Deferred { cycles_per_day: 96 },
        micro(QueueMode::QueueAll, w, d),
    ];
    specs.extend(
        sweep_windows
            .iter()
            .map(|&sw| micro(QueueMode::QueueAll, sw, d)),
    );
    let o = run(&cfg, &specs, &Opening::Unlimited, true, None);
    let (ub, lb) = (o.ub(), o.lb());

    // Pass 2: constrained liquidity (fraction of own UB).
    let opening: Vec<i64> = o
        .flow
        .ubs()
        .iter()
        .map(|&u| (u as f64 * p.liquidity_fraction.clamp(0.0, 2.0)).floor() as i64)
        .collect();
    let cspecs = [
        EngineSpec::Gross,
        micro(QueueMode::QueueOnShortfall, w, d),
        micro(QueueMode::QueueAll, w, d),
    ];
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
            Json::Arr(vec![
                Json::Int(e.from as i128),
                Json::Int(e.to as i128),
                Json::Num(e.flow as f64 / 100.0),
                Json::Num(c.circular[i] as f64 / 100.0),
            ])
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
    let stream: Vec<Payment> = PaymentStream::new(&cfg).collect();
    let (floor, _, _, _) = floor_curve(n, &stream, cfg.horizon_ms());
    let sweep: Vec<Json> = o.results[4..]
        .iter()
        .map(|r| {
            Json::obj()
                .set("name", r.name.clone())
                .set("saving_pct", 100.0 * r.saving_vs(ub))
                .set("p95_ms", r.delay_q(0.95))
                .set(
                    "cycle_value_share",
                    r.metrics.value_by_kind[1] as f64 / r.metrics.settled_value.max(1) as f64,
                )
        })
        .collect();
    let constrained: Vec<Json> = oc.results.iter().map(|r| r.to_json(ub, lb)).collect();

    Json::obj()
        .set(
            "params",
            Json::obj()
                .set("seed", p.seed)
                .set("participants", n)
                .set("payments_per_day", cfg.payments_per_day)
                .set("window_ms", w)
                .set("max_delay_ms", d)
                .set("receiver_bias_sigma", cfg.receiver_bias_sigma)
                .set("liquidity_fraction", p.liquidity_fraction),
        )
        .set("scale_factor", cfg.scale_factor)
        .set("payments", o.flow.count)
        .set("gross_value_brl", o.flow.gross_value as f64 / 100.0)
        .set("ub_brl", ub as f64 / 100.0)
        .set("lb_brl", lb as f64 / 100.0)
        .set("potential_offset", o.flow.potential_offset_fraction())
        .set(
            "potential_liquidity_saving",
            if ub == 0 {
                0.0
            } else {
                (ub - lb) as f64 / ub as f64
            },
        )
        .set("topology", topo.to_json())
        .set("nodes", Json::Arr(nodes))
        .set("edges", Json::Arr(edges))
        .set("engines", Json::Arr(engines))
        .set("sweep", Json::Arr(sweep))
        .set("constrained", Json::Arr(constrained))
        .set("floor", floor)
        .set(
            "intraday_value_brl",
            Json::Arr(value.into_iter().map(Json::Num).collect()),
        )
        .set(
            "intraday_net_debit_brl",
            Json::Arr(debit.into_iter().map(Json::Num).collect()),
        )
}

/// Run a small calibrated simulation and return the results as JSON.
/// `seed` is a JS number (integer part used) to avoid BigInt on the JS side.
#[wasm_bindgen]
pub fn simulate(
    seed: f64,
    participants: u32,
    payments: u32,
    window_ms: u32,
    max_delay_ms: u32,
    receiver_bias_sigma: f64,
    liquidity_fraction: f64,
) -> String {
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

/// Analyse a payment file's text (CSV with `time`, `from`, `to`, `amount`; see
/// `crates/loopsettle-sim/src/csvfile.rs`) entirely in memory: UB, LB and the liquidity floor
/// curve. Used by the browser, so the file never leaves the user's machine. Errors are returned
/// as JSON `{"error": "..."}`.
pub fn analyze_file_json(text: &str) -> Json {
    let t = match loopsettle_sim::csvfile::payments_from_text(text) {
        Ok(t) => t,
        Err(e) => return Json::obj().set("error", e),
    };
    if t.payments.len() > MAX_FILE_PAYMENTS {
        return Json::obj().set("error", format!("{} payments; the browser handles at most {MAX_FILE_PAYMENTS}; use `loopsettle bound` on the command line", t.payments.len()));
    }
    let n = t.names.len();
    let (floor, ub, lb, engines) = floor_curve(n, &t.payments, t.span_ms);
    let gross: i128 = t.payments.iter().map(|p| p.amount as i128).sum();
    Json::obj()
        .set("payments", t.payments.len())
        .set("participants", n)
        .set("rows_skipped", t.skipped)
        .set("rows_skipped_status", t.skipped_status)
        .set("span_hours", t.span_ms as f64 / 3.6e6)
        .set("gross_value", gross as f64 / 100.0)
        .set("ub", ub as f64 / 100.0)
        .set("lb", lb as f64 / 100.0)
        .set("floor", floor)
        .set("engines", Json::Arr(engines))
}

#[wasm_bindgen]
pub fn analyze_file(text: &str) -> String {
    analyze_file_json(text).to_string_compact()
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
        let p = Params {
            seed: 7,
            participants: 20,
            payments: 20_000,
            window_ms: 1000,
            max_delay_ms: 5000,
            receiver_bias_sigma: 0.4,
            liquidity_fraction: 0.5,
        };
        let a = simulate_json(&p).to_string_compact();
        let b = simulate_json(&p).to_string_compact();
        assert_eq!(a, b, "deterministic");
        assert!(
            a.contains("\"engines\"")
                && a.contains("\"edges\"")
                && a.contains("\"sweep\"")
                && a.contains("\"floor\"")
        );
    }

    /// The file analysis agrees with the command-line bound on the same payments: the tight
    /// bound is never above the window bound, and realised netting never above the tight bound.
    #[test]
    fn analyze_file_is_consistent() {
        let cfg = ScenarioConfig {
            seed: 3,
            participants: 15,
            payments_per_day: 20_000.0,
            n_big: 2,
            n_mid: 3,
            ..Default::default()
        };
        let mut txt = String::from("t_ms,from,to,amount\n");
        for p in PaymentStream::new(&cfg) {
            txt.push_str(&format!(
                "{},b{},b{},{}.{:02}\n",
                p.t_arrival,
                p.payer,
                p.payee,
                p.amount / 100,
                p.amount % 100
            ));
        }
        let j = analyze_file_json(&txt).to_string_compact();
        assert!(!j.contains("\"error\""), "{j}");
        let r = analyze_file_json(&txt);
        let Json::Obj(kv) = r else { panic!() };
        let floor = kv
            .iter()
            .find(|(k, _)| k == "floor")
            .map(|(_, v)| v.clone())
            .unwrap();
        let Json::Arr(points) = floor else { panic!() };
        assert_eq!(points.len(), FLOOR_DELAYS_MS.len());
        let num = |o: &Json, key: &str| -> f64 {
            let Json::Obj(kv) = o else { panic!() };
            match kv.iter().find(|(k, _)| k == key).map(|(_, v)| v) {
                Some(Json::Num(x)) => *x,
                Some(Json::Int(x)) => *x as f64,
                other => panic!("{key}: {other:?}"),
            }
        };
        for p in &points {
            let (t, w, r) = (
                num(p, "tight_bound_pct"),
                num(p, "window_bound_pct"),
                num(p, "netting_saving_pct"),
            );
            assert!(
                r <= t + 1e-9 && t <= w + 1e-9,
                "realised {r} <= tight {t} <= window {w}"
            );
        }
        assert!(analyze_file_json("nonsense")
            .to_string_compact()
            .contains("error"));
    }
}
