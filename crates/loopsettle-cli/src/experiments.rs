//! Experiment implementations. Each one reads its `experiments/<name>.toml`.

use crate::out::{f, table, Results};
use crate::svg::{Chart, Series};
use loopsettle_core::json::Json;
use loopsettle_core::QueueMode;
use loopsettle_sim::runner::MicroCycleSpec;
use loopsettle_sim::toml_lite::Doc;
use loopsettle_sim::{calibrate, run as run_engines, EngineResult, EngineSpec, Opening, RunOutput, ScenarioConfig};

pub const ALL: &[(&str, &str)] = &[
    ("calibration", "real vs simulated public Pix aggregates"),
    ("microcycle-sweep", "liquidity saving x p95 delay of atomic micro-cycles"),
    ("baseline", "UB, LB, circularity ladder and sensitivity"),
    ("potential-vs-realizable", "potential vs realizable liquidity savings (central comparison)"),
    ("stress", "largest participant offline for 30 minutes"),
    ("finland-replication", "fixed clearing cycles vs instant settlement"),
];

pub fn run(name: &str, path: &str, doc: &Doc, quick: bool) -> Result<(), String> {
    let mut cfg = ScenarioConfig::from_doc(doc)?;
    if quick {
        cfg.payments_per_day /= 10.0;
        cfg.scale_factor *= 10.0;
    }
    use crate::experiments2 as e2;
    match name {
        "calibration" => calibration(path, doc, &cfg),
        "microcycle-sweep" => microcycle_sweep(path, doc, &cfg),
        "baseline" => e2::baseline(path, doc, &cfg),
        "potential-vs-realizable" => e2::potential_vs_realizable(path, doc, &cfg),
        "stress" => e2::stress(path, doc, &cfg),
        "finland-replication" => e2::finland(path, doc, &cfg),
        _ => Err(format!("unknown experiment '{name}' (see `loopsettle list`)")),
    }
}

pub(crate) fn scenario_json(cfg: &ScenarioConfig) -> loopsettle_core::json::Json {
    use loopsettle_core::json::Json;
    Json::obj()
        .set("participants", cfg.participants)
        .set("days", cfg.days as u64)
        .set("payments_per_day", cfg.payments_per_day)
        .set("scale_factor", cfg.scale_factor)
        .set("body_median_brl", cfg.amounts.body_median)
        .set("body_sigma", cfg.amounts.body_sigma)
        .set("tail_prob", cfg.amounts.tail_prob)
        .set("tail_xm_brl", cfg.amounts.tail_xm)
        .set("tail_alpha", cfg.amounts.tail_alpha)
        .set("max_amount_brl", cfg.amounts.max_amount)
        .set("analytic_mean_ticket_brl", cfg.amounts.mean())
        .set("homophily", cfg.homophily)
        .set("receiver_bias_sigma", cfg.receiver_bias_sigma)
        .set("calibrated_shares", !cfg.shares.is_empty())
}

fn calibration(path: &str, doc: &Doc, cfg: &ScenarioConfig) -> Result<(), String> {
    let cal_path = doc.str("scenario.calibration").ok_or("calibration experiment needs scenario.calibration")?;
    let t = calibrate::Targets::load(cal_path)?;
    let rows = calibrate::report(cfg, &t);
    let mut r = Results::new("calibration", cfg.seed, path);
    r.csv("calibration.csv", &calibrate::rows_to_csv(&rows));
    r.set("reference_month", t.month);
    r.set("scenario", scenario_json(cfg));
    r.set("rows", calibrate::rows_to_json(&rows));
    let md: Vec<Vec<String>> = rows.iter().map(|x| vec![x.name.to_string(), f(x.real), f(x.sim), format!("{:+.2}%", 100.0 * x.rel_err())]).collect();
    eprintln!("{}", table(&["aggregate", "real", "simulated", "rel_err"], &md));
    r.finish();
    Ok(())
}

// ---------------------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------------------

pub(crate) const ENGINE_HEADER: &[&str] = &[
    "engine",
    "liquidity_brl",
    "saving_vs_ub_pct",
    "recovered_share_of_ub_minus_lb",
    "delay_p50_ms",
    "delay_p95_ms",
    "delay_p99_ms",
    "share_within_2800ms",
    "cycle_value_share",
    "rejected_value_share",
];

pub(crate) fn engine_row(r: &EngineResult, ub: i128, lb: i128) -> Vec<String> {
    let m = &r.metrics;
    let total = (m.settled_value + m.rejected_value).max(1) as f64;
    vec![
        r.name.clone(),
        f(r.liquidity as f64 / 100.0),
        f(100.0 * r.saving_vs(ub)),
        f(if ub > lb { (ub - r.liquidity) as f64 / (ub - lb) as f64 } else { 0.0 }),
        r.delay_q(0.5).to_string(),
        r.delay_q(0.95).to_string(),
        r.delay_q(0.99).to_string(),
        f(m.delay.fraction_at_most(2_800)),
        f(m.value_by_kind[1] as f64 / total),
        f(m.rejected_value as f64 / total),
    ]
}

pub(crate) fn flow_json(o: &RunOutput) -> Json {
    Json::obj()
        .set("payments", o.flow.count)
        .set("gross_value_brl", o.flow.gross_value as f64 / 100.0)
        .set("ub_brl", o.ub() as f64 / 100.0)
        .set("lb_brl", o.lb() as f64 / 100.0)
        .set("lb_over_ub", if o.ub() == 0 { 0.0 } else { o.lb() as f64 / o.ub() as f64 })
        .set("ub_over_gross", o.ub() as f64 / o.flow.gross_value.max(1) as f64)
        .set("potential_offset_fraction", o.flow.potential_offset_fraction())
}

pub(crate) fn f64_list(doc: &Doc, key: &str, default: &[f64]) -> Vec<f64> {
    doc.f64_array(key).unwrap_or_else(|| default.to_vec())
}

// ---------------------------------------------------------------------------------------
// microcycle-sweep
// ---------------------------------------------------------------------------------------

fn microcycle_sweep(path: &str, doc: &Doc, cfg: &ScenarioConfig) -> Result<(), String> {
    let windows = f64_list(doc, "sweep.window_ms", &[500.0, 1000.0, 2000.0, 5000.0, 10000.0]);
    let delays = f64_list(doc, "sweep.max_delay_ms", &[2000.0, 5000.0, 10000.0]);
    let max_len = doc.i64("sweep.max_cycle_len").unwrap_or(6) as usize;

    // Pass 1: unlimited liquidity, gross + every QueueAll configuration.
    let mut specs = vec![EngineSpec::Gross];
    for &d in &delays {
        for &w in &windows {
            specs.push(EngineSpec::Micro(MicroCycleSpec { mode: QueueMode::QueueAll, window_ms: w as u64, max_delay_ms: d as u64, max_cycle_len: max_len }));
        }
    }
    let t0 = std::time::Instant::now();
    let o = run_engines(cfg, &specs, &Opening::Unlimited, false, None);
    eprintln!("  sweep pass: {} engines in {:.1} s", specs.len(), t0.elapsed().as_secs_f64());
    let (ub, lb) = (o.ub(), o.lb());
    let mut res = Results::new("microcycle-sweep", cfg.seed, path);
    let mut header = vec!["window_ms", "max_delay_ms"];
    header.extend_from_slice(ENGINE_HEADER);
    let mut rows = Vec::new();
    let mut series: Vec<Series> = Vec::new();
    for &d in &delays {
        let mut pts = Vec::new();
        for r in &o.results {
            if let EngineSpec::Micro(m) = r.spec {
                if m.max_delay_ms == d as u64 {
                    let mut row = vec![m.window_ms.to_string(), m.max_delay_ms.to_string()];
                    row.extend(engine_row(r, ub, lb));
                    rows.push(row);
                    pts.push((r.delay_q(0.95) as f64 / 1000.0, 100.0 * r.saving_vs(ub)));
                }
            }
        }
        series.push(Series { label: format!("max_delay {} s", d / 1000.0), points: pts, line: true });
    }
    res.csv("sweep.csv", &table(&header, &rows));
    let chart = Chart {
        title: "Atomic micro-cycles: liquidity saving vs p95 delay".into(),
        x_label: "p95 settlement delay (s); points = window 0.5, 1, 2, 5, 10 s".into(),
        y_label: "liquidity saving vs gross UB (%)".into(),
        series,
        log_x: false,
    };
    res.csv("fig_tradeoff.svg", &chart.render());

    // Pass 2..: constrained liquidity, gross vs QueueOnShortfall.
    let fracs = f64_list(doc, "sweep.shortfall_liquidity_fraction", &[0.25, 0.5, 0.75]);
    let sw = doc.i64("sweep.shortfall_window_ms").unwrap_or(1000) as u64;
    let sd = doc.i64("sweep.shortfall_max_delay_ms").unwrap_or(10_000) as u64;
    let ubs = o.flow.ubs();
    let mut srows = Vec::new();
    let mut sjson = Vec::new();
    let cspecs = [
        EngineSpec::Gross,
        EngineSpec::Micro(MicroCycleSpec { mode: QueueMode::QueueOnShortfall, window_ms: sw, max_delay_ms: sd, max_cycle_len: max_len }),
        EngineSpec::Micro(MicroCycleSpec { mode: QueueMode::QueueAll, window_ms: sw, max_delay_ms: sd, max_cycle_len: max_len }),
    ];
    for &fr in &fracs {
        let opening: Vec<i64> = ubs.iter().map(|&u| (u as f64 * fr).floor() as i64).collect();
        let oc = run_engines(cfg, &cspecs, &Opening::Given(opening), false, None);
        for r in &oc.results {
            let mut row = vec![f(fr)];
            row.extend(engine_row(r, ub, lb));
            srows.push(row);
            sjson.push(Json::obj().set("liquidity_fraction_of_ub", fr).set("result", r.to_json(ub, lb)));
        }
    }
    let mut sh = vec!["liquidity_fraction_of_ub"];
    sh.extend_from_slice(ENGINE_HEADER);
    res.csv("shortfall.csv", &table(&sh, &srows));

    res.set("scenario", scenario_json(cfg));
    res.set("flow", flow_json(&o));
    res.set("unlimited", Json::Arr(o.results.iter().map(|r| r.to_json(ub, lb)).collect()));
    res.set("constrained", Json::Arr(sjson));
    eprintln!("{}", table(&header, &rows));
    eprintln!("{}", table(&sh, &srows));
    res.finish();
    Ok(())
}
