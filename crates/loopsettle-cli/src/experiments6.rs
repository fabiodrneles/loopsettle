//! `liquidity-frontier` (specs/06): rejection and delay as a function of the liquidity
//! participants hold, for each settlement design, and the "equivalent liquidity" of each design
//! at equal service. All other experiments measure the liquidity a design needs with unlimited
//! balances; this one fixes the balances and measures what happens.

use crate::experiments::{f64_list, scenario_json};
use crate::out::{f, table, Results};
use crate::stats::{mean_ci95, par_map};
use crate::svg::{Chart, Series};
use loopsettle_core::json::Json;
use loopsettle_core::Cents;
use loopsettle_sim::toml_lite::Doc;
use loopsettle_sim::{run as run_engines, EngineSpec, Opening, ScenarioConfig};

/// What is kept of one engine at one liquidity level, for one seed.
#[derive(Clone, Copy)]
struct Point {
    rejected_value_pct: f64,
    rejected_count_pct: f64,
    delay_p50_ms: f64,
    delay_p95_ms: f64,
    within_2800: f64,
    cycle_value_share: f64,
}

/// Opening balances for one rule and level.
fn opening(rule: &str, alpha: f64, ubs: &[Cents], out: &[Cents]) -> Vec<Cents> {
    match rule {
        "ub" => ubs.iter().map(|&u| (u as f64 * alpha).floor() as Cents).collect(),
        _ => {
            let ub_total: f64 = ubs.iter().map(|&u| u as f64).sum();
            let out_total: f64 = out.iter().map(|&o| o as f64).sum::<f64>().max(1.0);
            out.iter().map(|&o| (alpha * ub_total * o as f64 / out_total).floor() as Cents).collect()
        }
    }
}

/// Smallest liquidity level at which `rej` (rejected value, %) is at most `target`, by linear
/// interpolation between the two grid points around the crossing. `None` if the target is not
/// reached on the grid. If it is already met at the lowest level, that level is returned (the
/// true value is at or below it).
fn equivalent_alpha(alphas: &[f64], rej: &[f64], target: f64) -> Option<f64> {
    let j = rej.iter().position(|&r| r <= target)?;
    if j == 0 {
        return Some(alphas[0]);
    }
    let (r0, r1) = (rej[j - 1], rej[j]);
    Some(alphas[j - 1] + (r0 - target) / (r0 - r1) * (alphas[j] - alphas[j - 1]))
}

pub fn liquidity_frontier(path: &str, doc: &Doc, base: &ScenarioConfig) -> Result<(), String> {
    let mut res = Results::new("liquidity-frontier", base.seed, path);
    let seeds: Vec<u64> = f64_list(doc, "frontier.seeds", &[20260930.0, 1.0, 2.0]).iter().map(|&s| s as u64).collect();
    let sigmas = f64_list(doc, "frontier.receiver_bias_sigma", &[0.4, 1.0]);
    let wholesale_ppd = doc.f64("frontier.wholesale_payments_per_day").unwrap_or(20_000.0);
    let rules = doc.str_array("frontier.rules").unwrap_or_else(|| vec!["ub".into(), "outflow".into()]);
    let alphas = f64_list(doc, "frontier.alphas", &[0.5, 0.75, 1.0, 1.5]);
    let targets = f64_list(doc, "frontier.targets_pct", &[0.01, 0.1, 1.0]);
    let engine_names = doc.str_array("frontier.engines").unwrap_or_else(|| vec!["gross".into(), "queue:fifo:10000".into(), "micro:shortfall:500:10000".into()]);
    if rules.iter().any(|r| r != "ub" && r != "outflow") {
        return Err("frontier.rules: each rule must be \"ub\" or \"outflow\"".into());
    }
    if !alphas.windows(2).all(|w| w[0] < w[1]) {
        return Err("frontier.alphas must be strictly increasing".into());
    }
    let specs: Vec<EngineSpec> = engine_names.iter().map(|s| EngineSpec::parse(s)).collect::<Result<_, _>>()?;
    if specs.first() != Some(&EngineSpec::Gross) {
        return Err("frontier.engines must start with \"gross\" (the reference design)".into());
    }

    let mut scenarios: Vec<(String, ScenarioConfig)> = sigmas.iter().map(|&s| (format!("retail sigma {}", f(s)), ScenarioConfig { receiver_bias_sigma: s, ..base.clone() })).collect();
    if wholesale_ppd > 0.0 {
        scenarios.push((format!("large-value {wholesale_ppd:.0}/day"), crate::experiments5::wholesale_config(base, wholesale_ppd)));
    }

    // One (scenario, seed) per thread. Result: [rule][alpha][engine] -> Point, plus the number
    // of grid steps where gross rejection went *up* when liquidity went up.
    let mut jobs = Vec::new();
    for s in 0..scenarios.len() {
        for &seed in &seeds {
            jobs.push((s, seed));
        }
    }
    let t0 = std::time::Instant::now();
    let done = par_map(&jobs, |&(s, seed)| {
        let cfg = ScenarioConfig { seed, ..scenarios[s].1.clone() };
        let free = run_engines(&cfg, &specs[..1], &Opening::Unlimited, false, None);
        let (ubs, out, gross) = (free.flow.ubs(), free.flow.out_value.clone(), free.flow.gross_value.max(1) as f64);
        let count = free.flow.count.max(1) as f64;
        let mut by_rule = Vec::new();
        for rule in &rules {
            let mut by_alpha = Vec::new();
            for &a in &alphas {
                let o = run_engines(&cfg, &specs, &Opening::Given(opening(rule, a, &ubs, &out)), false, None);
                let pts: Vec<Point> = o
                    .results
                    .iter()
                    .map(|r| {
                        let m = &r.metrics;
                        Point {
                            rejected_value_pct: 100.0 * m.rejected_value as f64 / gross,
                            rejected_count_pct: 100.0 * m.rejected_count as f64 / count,
                            delay_p50_ms: r.delay_q(0.5) as f64,
                            delay_p95_ms: r.delay_q(0.95) as f64,
                            within_2800: m.share_within_ms(2_800).unwrap_or(f64::NAN),
                            cycle_value_share: m.value_by_kind[1] as f64 / gross,
                        }
                    })
                    .collect();
                by_alpha.push(pts);
            }
            by_rule.push(by_alpha);
        }
        by_rule
    });
    eprintln!("  {} runs done in {:.0} s", jobs.len(), t0.elapsed().as_secs_f64());
    res.measure("simulation_seconds", t0.elapsed().as_secs_f64());

    let mut rows = Vec::new();
    let mut eq_rows = Vec::new();
    let mut js = Vec::new();
    let mut non_monotone = 0u64;
    let mut fig: Vec<Series> = Vec::new();
    for (s, (sname, _)) in scenarios.iter().enumerate() {
        // results of this scenario, one per seed
        let per_seed: Vec<&Vec<Vec<Vec<Point>>>> = jobs.iter().zip(&done).filter(|(j, _)| j.0 == s).map(|(_, d)| d).collect();
        for (ri, rule) in rules.iter().enumerate() {
            for (e, ename) in engine_names.iter().enumerate() {
                let label = specs[e].build(1).name();
                let _ = ename;
                let mut curve = Vec::new();
                for (ai, &a) in alphas.iter().enumerate() {
                    let col = |g: &dyn Fn(&Point) -> f64| -> Vec<f64> { per_seed.iter().map(|d| g(&d[ri][ai][e])).collect() };
                    let (rv, lo, hi) = mean_ci95(&col(&|p| p.rejected_value_pct));
                    let mean = |g: &dyn Fn(&Point) -> f64| mean_ci95(&col(g)).0;
                    rows.push(vec![
                        sname.clone(),
                        rule.clone(),
                        f(a),
                        label.clone(),
                        per_seed.len().to_string(),
                        f(rv),
                        f(lo),
                        f(hi),
                        f(mean(&|p| p.rejected_count_pct)),
                        f(mean(&|p| p.delay_p50_ms)),
                        f(mean(&|p| p.delay_p95_ms)),
                        f(mean(&|p| p.within_2800)),
                        f(mean(&|p| p.cycle_value_share)),
                    ]);
                    curve.push((a, rv));
                }
                if rule == "ub" && s == scenarios.len().min(sigmas.len()) - 1 {
                    fig.push(Series { label: label.clone(), points: curve.clone(), line: true });
                }
                // equivalent liquidity, per seed, then mean and CI over the seeds that reach the target
                for &t in &targets {
                    let per: Vec<Option<f64>> = per_seed.iter().map(|d| equivalent_alpha(&alphas, &(0..alphas.len()).map(|ai| d[ri][ai][e].rejected_value_pct).collect::<Vec<_>>(), t)).collect();
                    let gross_per: Vec<Option<f64>> = per_seed.iter().map(|d| equivalent_alpha(&alphas, &(0..alphas.len()).map(|ai| d[ri][ai][0].rejected_value_pct).collect::<Vec<_>>(), t)).collect();
                    let reached: Vec<f64> = per.iter().flatten().copied().collect();
                    // paired difference with gross, on the seeds where both reach the target
                    let diff: Vec<f64> = per.iter().zip(&gross_per).filter_map(|(a, g)| Some(100.0 * (g.as_ref()? - a.as_ref()?) / g.as_ref()?)).collect();
                    let (m, lo, hi) = if reached.is_empty() { (f64::NAN, f64::NAN, f64::NAN) } else { mean_ci95(&reached) };
                    let (dm, dlo, dhi) = if diff.is_empty() { (f64::NAN, f64::NAN, f64::NAN) } else { mean_ci95(&diff) };
                    eq_rows.push(vec![sname.clone(), rule.clone(), f(t), label.clone(), format!("{}/{}", reached.len(), per.len()), f(m), f(lo), f(hi), f(dm), f(dlo), f(dhi)]);
                    js.push(
                        Json::obj()
                            .set("scenario", sname.as_str())
                            .set("rule", rule.as_str())
                            .set("target_rejected_value_pct", t)
                            .set("engine", label.as_str())
                            .set("seeds_reaching_target", reached.len() as u64)
                            .set("equivalent_alpha_mean", m)
                            .set("liquidity_saving_vs_gross_pct_mean", dm)
                            .set("liquidity_saving_vs_gross_pct_ci95_low", dlo)
                            .set("liquidity_saving_vs_gross_pct_ci95_high", dhi),
                    );
                }
            }
            // coherence: does gross rejection ever go up when liquidity goes up?
            for d in &per_seed {
                non_monotone += (1..alphas.len()).filter(|&ai| d[ri][ai][0].rejected_value_pct > d[ri][ai - 1][0].rejected_value_pct + 1e-12).count() as u64;
            }
        }
    }
    let header = [
        "scenario", "rule", "alpha", "engine", "seeds", "rejected_value_pct_mean", "rejected_value_pct_ci95_low", "rejected_value_pct_ci95_high", "rejected_count_pct_mean", "delay_p50_ms_mean",
        "delay_p95_ms_mean", "share_within_2800ms_mean", "cycle_value_share_mean",
    ];
    res.csv("frontier.csv", &table(&header, &rows));
    let eq_header = [
        "scenario", "rule", "target_rejected_value_pct", "engine", "seeds_reaching_target", "equivalent_alpha_mean", "equivalent_alpha_ci95_low", "equivalent_alpha_ci95_high",
        "liquidity_saving_vs_gross_pct_mean", "liquidity_saving_vs_gross_pct_ci95_low", "liquidity_saving_vs_gross_pct_ci95_high",
    ];
    res.csv("equivalent_liquidity.csv", &table(&eq_header, &eq_rows));
    let chart = Chart {
        title: "Rejected value vs liquidity held (retail, calibrated bias; balances = alpha x UB)".into(),
        x_label: "liquidity held, as a multiple of each participant's UB".into(),
        y_label: "rejected value (% of gross value)".into(),
        series: fig,
        log_x: false,
    };
    res.csv("fig_frontier.svg", &chart.render());
    res.set("scenario", scenario_json(base));
    res.set("seeds", Json::Arr(seeds.iter().map(|&s| Json::from(s)).collect()));
    res.set("engines", Json::Arr(specs.iter().map(|s| s.to_json()).collect()));
    res.set("gross_rejection_rose_with_liquidity_grid_steps", non_monotone);
    res.set("equivalent_liquidity", Json::Arr(js));
    res.set(
        "note",
        "rejected payments are not retried (ASSUMPTION); rule ub uses the day's own peak (hindsight); equivalent liquidity is interpolated linearly on the alpha grid, per seed, and the saving vs gross is a paired difference over the seeds where both designs reach the target",
    );
    eprintln!("{}", table(&eq_header, &eq_rows));
    res.finish();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equivalent_alpha_interpolates_and_reports_unreached() {
        let a = [0.5, 1.0, 2.0];
        assert_eq!(equivalent_alpha(&a, &[4.0, 2.0, 0.0], 1.0), Some(1.5));
        assert_eq!(equivalent_alpha(&a, &[4.0, 2.0, 0.0], 3.0), Some(0.75));
        assert_eq!(equivalent_alpha(&a, &[0.5, 0.2, 0.0], 1.0), Some(0.5), "already met at the lowest level");
        assert_eq!(equivalent_alpha(&a, &[4.0, 3.0, 2.0], 1.0), None);
    }

    #[test]
    fn opening_rules() {
        assert_eq!(opening("ub", 0.5, &[10, 0, 4], &[1, 1, 2]), vec![5, 0, 2]);
        // total UB 14 shared by outflow 1:1:2 -> 3.5, 3.5, 7
        assert_eq!(opening("outflow", 1.0, &[10, 0, 4], &[1, 1, 2]), vec![3, 3, 7]);
    }
}
