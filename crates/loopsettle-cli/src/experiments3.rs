//! `robustness`: does the central result survive the threats listed in docs/ASSESSMENT.md?
//!
//! R1 seeds   — many seeds x several receiver-bias values, with 95% confidence intervals
//! R2 scale   — 1M ... 30M payments per day (scale factor 239 ... 8), same scenario
//! R3 cycles  — maximum micro-cycle length 2 ... 10
//! R4 bound   — measured saving vs the analytical delay bound (crates/loopsettle-sim/src/bound.rs)
//! R5 banks   — per-participant ratio instant / deferred (the Finnish study's heterogeneity)

use crate::experiments::{f64_list, scenario_json};
use crate::out::{f, table, Results};
use crate::svg::{Chart, Series};
use loopsettle_core::json::Json;
use loopsettle_sim::bound::{delay_bounds, liquidity_floors};
use loopsettle_sim::toml_lite::Doc;
use loopsettle_sim::{run as run_engines, EngineSpec, Opening, ScenarioConfig};

/// Engines compared in R1/R2, each with its maximum settlement delay (for the bound).
const ENGINES: &[(&str, u64)] = &[
    ("micro:all:500:2000", 2_000),
    ("micro:all:500:10000", 10_000),
    ("deferred:8640", 10_000),
    ("deferred:96", 900_000),
    ("deferred:1", 86_400_000),
];

struct Measure {
    saving_pct: Vec<f64>,
    bound_pct: Vec<f64>,
    /// tight bound: UB minus the sum of liquidity floors, % of UB (METHODOLOGY section 9b)
    tight_pct: Vec<f64>,
    ub: i128,
    lb: i128,
    gross: i128,
}

/// One scenario through gross + ENGINES, plus the delay bound for each engine.
fn measure(cfg: &ScenarioConfig) -> Measure {
    let mut specs = vec![EngineSpec::Gross];
    specs.extend(ENGINES.iter().map(|(s, _)| EngineSpec::parse(s).unwrap()));
    let o = run_engines(cfg, &specs, &Opening::Unlimited, false, None);
    let ub = o.ub();
    let delays: Vec<u64> = ENGINES.iter().map(|&(_, d)| d).collect();
    let bounds = delay_bounds(cfg, &delays);
    // floors for the distinct delays only (each one costs a pass over the stream)
    let mut uniq = delays.clone();
    uniq.sort_unstable();
    uniq.dedup();
    let floors = liquidity_floors(cfg, &uniq);
    let floor_of = |d: u64| -> i128 { floors[uniq.iter().position(|&x| x == d).unwrap()].iter().map(|&v| v as i128).sum() };
    let pct = |x: i128| if ub == 0 { 0.0 } else { 100.0 * x as f64 / ub as f64 };
    Measure {
        saving_pct: o.results[1..].iter().map(|r| 100.0 * r.saving_vs(ub)).collect(),
        bound_pct: bounds.iter().map(|b| pct(b.iter().map(|&v| v as i128).sum())).collect(),
        tight_pct: delays.iter().map(|&d| pct(ub - floor_of(d))).collect(),
        ub,
        lb: o.lb(),
        gross: o.flow.gross_value,
    }
}

fn mean_sd(v: &[f64]) -> (f64, f64) {
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    let var = if v.len() > 1 { v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0) } else { 0.0 };
    (m, var.sqrt())
}

/// Two-sided 95% Student t critical value (small tables are enough here).
fn t95(df: usize) -> f64 {
    const T: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074,
        2.069, 2.064, 2.060, 2.056, 2.052, 2.048, 2.045, 2.042,
    ];
    if df == 0 {
        f64::NAN
    } else if df <= 30 {
        T[df - 1]
    } else {
        1.96
    }
}

fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let pos = q * (sorted.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
}

pub fn robustness(path: &str, doc: &Doc, base: &ScenarioConfig) -> Result<(), String> {
    let mut res = Results::new("robustness", base.seed, path);
    let t_all = std::time::Instant::now();

    // ---------------- R1: seeds x receiver bias ----------------
    let seeds: Vec<u64> = f64_list(doc, "robustness.seeds", &[20260930.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]).iter().map(|&s| s as u64).collect();
    let sigmas = f64_list(doc, "robustness.sigmas", &[0.0, 0.4, 0.8]);
    let mut seed_rows = Vec::new();
    let mut sum_rows = Vec::new();
    let mut sum_json = Vec::new();
    for &sigma in &sigmas {
        let mut per: Vec<Vec<f64>> = vec![Vec::new(); ENGINES.len()];
        let mut perb: Vec<Vec<f64>> = vec![Vec::new(); ENGINES.len()];
        let mut pert: Vec<Vec<f64>> = vec![Vec::new(); ENGINES.len()];
        let mut lbub = Vec::new();
        for &seed in &seeds {
            let cfg = ScenarioConfig { seed, receiver_bias_sigma: sigma, ..base.clone() };
            let m = measure(&cfg);
            lbub.push(m.lb as f64 / m.ub.max(1) as f64);
            for (k, (name, _)) in ENGINES.iter().enumerate() {
                per[k].push(m.saving_pct[k]);
                perb[k].push(m.bound_pct[k]);
                pert[k].push(m.tight_pct[k]);
                seed_rows.push(vec![f(sigma), seed.to_string(), name.to_string(), f(m.saving_pct[k]), f(m.bound_pct[k]), f(m.ub as f64 / 100.0), f(m.tight_pct[k])]);
            }
        }
        eprintln!("  R1 sigma {sigma}: {} seeds done ({:.0} s)", seeds.len(), t_all.elapsed().as_secs_f64());
        let (lm, _) = mean_sd(&lbub);
        for (k, (name, d)) in ENGINES.iter().enumerate() {
            let (m, sd) = mean_sd(&per[k]);
            let (bm, _) = mean_sd(&perb[k]);
            let half = t95(per[k].len() - 1) * sd / (per[k].len() as f64).sqrt();
            let (tm, tsd) = mean_sd(&pert[k]);
            let thalf = t95(pert[k].len() - 1) * tsd / (pert[k].len() as f64).sqrt();
            let positive = per[k].iter().filter(|&&x| x > 0.0).count();
            sum_rows.push(vec![f(sigma), name.to_string(), d.to_string(), seeds.len().to_string(), f(m), f(sd), f(m - half), f(m + half), positive.to_string(), f(bm), f(lm), f(tm), f(tm - thalf), f(tm + thalf), f(if tm > 0.0 { m / tm } else { 0.0 })]);
            sum_json.push(
                Json::obj()
                    .set("receiver_bias_sigma", sigma)
                    .set("engine", *name)
                    .set("max_delay_ms", *d)
                    .set("seeds", seeds.len() as u64)
                    .set("saving_pct_mean", m)
                    .set("saving_pct_sd", sd)
                    .set("saving_pct_ci95_low", m - half)
                    .set("saving_pct_ci95_high", m + half)
                    .set("bound_pct_mean", bm)
                    .set("tight_bound_pct_mean", tm)
                    .set("tight_bound_pct_ci95_low", tm - thalf)
                    .set("tight_bound_pct_ci95_high", tm + thalf),
            );
        }
    }
    res.csv("seeds.csv", &table(&["receiver_bias_sigma", "seed", "engine", "saving_vs_ub_pct", "delay_bound_pct", "ub_brl", "tight_bound_pct"], &seed_rows));
    let sum_header = [
        "receiver_bias_sigma", "engine", "max_delay_ms", "seeds", "saving_pct_mean", "saving_pct_sd", "saving_pct_ci95_low", "saving_pct_ci95_high", "seeds_with_positive_saving",
        "delay_bound_pct_mean", "lb_over_ub_mean", "tight_bound_pct_mean", "tight_bound_pct_ci95_low", "tight_bound_pct_ci95_high", "saving_over_tight_bound",
    ];
    res.csv("seeds_summary.csv", &table(&sum_header, &sum_rows));

    // ---------------- R2: scale ----------------
    let scales = f64_list(doc, "robustness.payments_per_day", &[1e6, 3e6, 10e6, 30e6]);
    let mut scale_rows = Vec::new();
    let mut scale_json = Vec::new();
    let mut series: Vec<Series> = ENGINES.iter().take(3).map(|(n, _)| Series { label: n.to_string(), points: Vec::new(), line: true }).collect();
    let mut bound_series = Series { label: "window bound, 10 s".into(), points: Vec::new(), line: true };
    let mut tight_series = Series { label: "tight bound, 10 s".into(), points: Vec::new(), line: true };
    for &ppd in &scales {
        let cfg = ScenarioConfig { payments_per_day: ppd, scale_factor: base.scale_factor * base.payments_per_day / ppd, ..base.clone() };
        let t0 = std::time::Instant::now();
        let m = measure(&cfg);
        let secs = t0.elapsed().as_secs_f64();
        eprintln!("  R2 {ppd:.0} payments/day: {secs:.1} s");
        res.measure(&format!("scale_{ppd:.0}_seconds"), secs);
        for (k, (name, d)) in ENGINES.iter().enumerate() {
            scale_rows.push(vec![f(ppd), f(cfg.scale_factor), name.to_string(), d.to_string(), f(m.saving_pct[k]), f(m.bound_pct[k]), f(m.ub as f64 / m.gross.max(1) as f64), f(m.tight_pct[k])]);
            if k < 3 {
                series[k].points.push((ppd / 1e6, m.saving_pct[k]));
            }
        }
        bound_series.points.push((ppd / 1e6, m.bound_pct[1]));
        tight_series.points.push((ppd / 1e6, m.tight_pct[1]));
        scale_json.push(
            Json::obj()
                .set("payments_per_day", ppd)
                .set("scale_factor", cfg.scale_factor)
                .set("ub_over_gross", m.ub as f64 / m.gross.max(1) as f64)
                .set("saving_pct", Json::Arr(m.saving_pct.iter().map(|&x| Json::from(x)).collect()))
                .set("bound_pct", Json::Arr(m.bound_pct.iter().map(|&x| Json::from(x)).collect()))
                .set("tight_bound_pct", Json::Arr(m.tight_pct.iter().map(|&x| Json::from(x)).collect())),
        );
    }
    res.csv("scale.csv", &table(&["payments_per_day", "scale_factor", "engine", "max_delay_ms", "saving_vs_ub_pct", "delay_bound_pct", "ub_over_gross", "tight_bound_pct"], &scale_rows));
    series.push(bound_series);
    series.push(tight_series);
    let chart = Chart {
        title: "Scale: liquidity saving of second-scale netting as volume grows".into(),
        x_label: "simulated payments per day (millions, log)".into(),
        y_label: "liquidity saving vs gross UB (%)".into(),
        series,
        log_x: true,
    };
    res.csv("fig_scale.svg", &chart.render());

    // ---------------- R3: maximum cycle length ----------------
    let lens = f64_list(doc, "robustness.max_cycle_len", &[2.0, 3.0, 6.0, 10.0]);
    let mut specs = vec![EngineSpec::Gross];
    for &l in &lens {
        specs.push(EngineSpec::parse(&format!("micro:all:500:10000:{}", l as usize))?);
    }
    let o = run_engines(base, &specs, &Opening::Unlimited, false, None);
    let ub = o.ub();
    let mut len_rows = Vec::new();
    for (k, &l) in lens.iter().enumerate() {
        let r = &o.results[k + 1];
        let m = &r.metrics;
        let settled = m.settled_value.max(1) as f64;
        len_rows.push(vec![f(l), f(100.0 * r.saving_vs(ub)), f(m.value_by_kind[1] as f64 / settled)]);
    }
    res.csv("cycle_len.csv", &table(&["max_cycle_len", "saving_vs_ub_pct", "value_share_settled_in_cycles"], &len_rows));

    // ---------------- R5: per-participant instant / deferred ratio ----------------
    let specs = [EngineSpec::Gross, EngineSpec::Deferred { cycles_per_day: 1 }, EngineSpec::Deferred { cycles_per_day: 10 }];
    let o = run_engines(base, &specs, &Opening::Unlimited, false, None);
    let mut bank_rows = Vec::new();
    let mut bank_json = Vec::new();
    for (k, cycles) in [(1usize, 1u32), (2, 10)] {
        let mut ratios = Vec::new();
        let (mut zero_den, mut both_zero) = (0u64, 0u64);
        for i in 0..base.participants {
            let instant = o.results[0].liquidity_by_participant[i] as f64;
            let def = o.results[k].liquidity_by_participant[i] as f64;
            if def > 0.0 {
                ratios.push(instant / def);
            } else if instant > 0.0 {
                zero_den += 1;
            } else {
                both_zero += 1;
            }
        }
        ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let qs = [0.1, 0.25, 0.5, 0.75, 0.9];
        let mut row = vec![cycles.to_string(), ratios.len().to_string(), zero_den.to_string(), both_zero.to_string()];
        row.extend(qs.iter().map(|&q| f(quantile(&ratios, q))));
        row.push(f(*ratios.last().unwrap_or(&f64::NAN)));
        bank_rows.push(row);
        bank_json.push(
            Json::obj()
                .set("cycles_per_day", cycles)
                .set("participants_with_ratio", ratios.len() as u64)
                .set("needs_liquidity_only_if_instant", zero_den)
                .set("needs_no_liquidity", both_zero)
                .set("ratio_p50", quantile(&ratios, 0.5))
                .set("ratio_p90", quantile(&ratios, 0.9)),
        );
    }
    res.csv(
        "participants_ratio.csv",
        &table(&["cycles_per_day", "participants_with_ratio", "needs_liquidity_only_if_instant", "needs_no_liquidity", "ratio_p10", "ratio_p25", "ratio_p50", "ratio_p75", "ratio_p90", "ratio_max"], &bank_rows),
    );

    res.measure("total_seconds", t_all.elapsed().as_secs_f64());
    res.set("scenario", scenario_json(base));
    res.set("seeds", Json::Arr(seeds.iter().map(|&s| Json::from(s)).collect()));
    res.set("seeds_summary", Json::Arr(sum_json));
    res.set("scale", Json::Arr(scale_json));
    res.set("participants_ratio", Json::Arr(bank_json));
    eprintln!("{}", table(&sum_header, &sum_rows));
    eprintln!("{}", table(&["payments_per_day", "scale_factor", "engine", "max_delay_ms", "saving_vs_ub_pct", "delay_bound_pct", "ub_over_gross"], &scale_rows));
    res.finish();
    Ok(())
}
