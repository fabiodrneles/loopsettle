//! `echo-sweep` (specs/07, G2): does the central result survive short-horizon circularity?
//!
//! With stationary Poisson arrivals a participant's receipts in the seconds after it pays are
//! there by chance only. Echo payments make them systematic: a share of all payments is a
//! response to a receipt, a few seconds later, onwards or straight back. For every echo
//! setting the experiment reports, over several seeds, the tight bound (UB minus the liquidity
//! floors) and what the engines realise.

use crate::experiments::{f64_list, scenario_json};
use crate::experiments3::{measure, Measure, ENGINES};
use crate::out::{f, table, Results};
use crate::stats::{mean_ci95, par_map};
use loopsettle_core::json::Json;
use loopsettle_sim::toml_lite::Doc;
use loopsettle_sim::{run as run_engines, EngineSpec, Opening, ScenarioConfig};

/// Kill criterion (docs/ASSESSMENT.md, update 5): the claim "second-scale mechanisms recover a
/// negligible share" stops being unconditional if the 10 s tight bound passes this share of UB
/// for an echo share of at most 20%.
const KILL_PCT: f64 = 2.0;

pub fn echo_sweep(path: &str, doc: &Doc, base: &ScenarioConfig) -> Result<(), String> {
    let mut res = Results::new("echo-sweep", base.seed, path);
    let seeds: Vec<u64> = f64_list(doc, "echo.seeds", &[20260930.0, 1.0, 2.0]).iter().map(|&s| s as u64).collect();
    let shares = f64_list(doc, "echo.share", &[0.05, 0.2, 0.5]);
    let delays = f64_list(doc, "echo.delay_ms", &[1_000.0, 10_000.0, 60_000.0]);
    let returns = f64_list(doc, "echo.return_prob", &[0.0, 1.0]);
    // (share, delay, return probability); the first case is "no echo"
    let mut cases = vec![(0.0, 0.0, 0.0)];
    for &s in &shares {
        for &d in &delays {
            for &r in &returns {
                cases.push((s, d, r));
            }
        }
    }
    let mut jobs = Vec::new();
    for c in 0..cases.len() {
        for &seed in &seeds {
            jobs.push((c, seed));
        }
    }
    let t0 = std::time::Instant::now();
    let done: Vec<Measure> = par_map(&jobs, |&(c, seed)| {
        let (share, delay, ret) = cases[c];
        measure(&ScenarioConfig { seed, echo_share: share, echo_delay_ms: delay.max(1.0), echo_return_prob: ret, ..base.clone() })
    });
    eprintln!("  {} runs done in {:.0} s", jobs.len(), t0.elapsed().as_secs_f64());
    res.measure("simulation_seconds", t0.elapsed().as_secs_f64());

    let mut rows = Vec::new();
    let mut js = Vec::new();
    let mut worst_kill: f64 = 0.0;
    for (c, &(share, delay, ret)) in cases.iter().enumerate() {
        let ms: Vec<&Measure> = jobs.iter().zip(&done).filter(|(j, _)| j.0 == c).map(|(_, m)| m).collect();
        let lb_ub = mean_ci95(&ms.iter().map(|m| m.lb as f64 / m.ub.max(1) as f64).collect::<Vec<_>>()).0;
        for (k, (name, d)) in ENGINES.iter().enumerate() {
            let col = |g: &dyn Fn(&Measure) -> f64| -> Vec<f64> { ms.iter().map(|m| g(m)).collect() };
            let (sv, slo, shi) = mean_ci95(&col(&|m| m.saving_pct[k]));
            let (tb, tlo, thi) = mean_ci95(&col(&|m| m.tight_pct[k]));
            let cyc = mean_ci95(&col(&|m| m.cycle_share[k])).0;
            rows.push(vec![f(share), f(delay), f(ret), name.to_string(), d.to_string(), ms.len().to_string(), f(sv), f(slo), f(shi), f(tb), f(tlo), f(thi), f(if tb > 0.0 { sv / tb } else { 0.0 }), f(cyc), f(lb_ub)]);
            if *d == 10_000 && name.starts_with("deferred") {
                if share <= 0.2 + 1e-9 {
                    worst_kill = worst_kill.max(thi);
                }
                js.push(
                    Json::obj()
                        .set("echo_share", share)
                        .set("echo_delay_ms", delay)
                        .set("echo_return_prob", ret)
                        .set("tight_bound_10s_pct_mean", tb)
                        .set("tight_bound_10s_pct_ci95_high", thi)
                        .set("netting_every_10s_saving_pct_mean", sv)
                        .set("lb_over_ub_mean", lb_ub),
                );
            }
        }
    }
    let header = [
        "echo_share", "echo_delay_ms", "echo_return_prob", "engine", "max_delay_ms", "seeds", "saving_pct_mean", "saving_pct_ci95_low", "saving_pct_ci95_high", "tight_bound_pct_mean",
        "tight_bound_pct_ci95_low", "tight_bound_pct_ci95_high", "saving_over_tight_bound", "cycle_value_share_mean", "lb_over_ub_mean",
    ];
    res.csv("echo.csv", &table(&header, &rows));

    // ---- Sensitivity of the case that fired the criterion (20% share, 1 s, straight back):
    // one thing changed at a time. Does the conditional result depend on echoes repeating the
    // exact amount, on the receiver bias, or on the scale?
    let fired = ScenarioConfig { echo_share: 0.2, echo_delay_ms: 1_000.0, echo_return_prob: 1.0, ..base.clone() };
    let variants: Vec<(&str, ScenarioConfig)> = vec![
        ("as in the sweep (same amount, base bias and scale)", fired.clone()),
        ("echo passes on half of the amount", ScenarioConfig { echo_amount_fraction: 0.5, ..fired.clone() }),
        ("echo passes on a quarter of the amount", ScenarioConfig { echo_amount_fraction: 0.25, ..fired.clone() }),
        ("calibrated receiver bias (sigma 1.0)", ScenarioConfig { receiver_bias_sigma: 1.0, ..fired.clone() }),
        ("3x the payments", ScenarioConfig { payments_per_day: 3.0 * base.payments_per_day, scale_factor: base.scale_factor / 3.0, ..fired.clone() }),
        ("10x the payments", ScenarioConfig { payments_per_day: 10.0 * base.payments_per_day, scale_factor: base.scale_factor / 10.0, ..fired.clone() }),
    ];
    let mut vjobs = Vec::new();
    for v in 0..variants.len() {
        for &seed in &seeds {
            vjobs.push((v, seed));
        }
    }
    let vdone: Vec<Measure> = par_map(&vjobs, |&(v, seed)| measure(&ScenarioConfig { seed, ..variants[v].1.clone() }));
    let mut vrows = Vec::new();
    for (v, (label, _)) in variants.iter().enumerate() {
        let ms: Vec<&Measure> = vjobs.iter().zip(&vdone).filter(|(j, _)| j.0 == v).map(|(_, m)| m).collect();
        for (k, (name, d)) in ENGINES.iter().enumerate() {
            if *d != 10_000 {
                continue;
            }
            let (sv, slo, shi) = mean_ci95(&ms.iter().map(|m| m.saving_pct[k]).collect::<Vec<_>>());
            let (tb, tlo, thi) = mean_ci95(&ms.iter().map(|m| m.tight_pct[k]).collect::<Vec<_>>());
            vrows.push(vec![label.replace(',', ";"), name.to_string(), ms.len().to_string(), f(sv), f(slo), f(shi), f(tb), f(tlo), f(thi)]);
        }
    }
    let vheader = ["variant", "engine", "seeds", "saving_pct_mean", "saving_pct_ci95_low", "saving_pct_ci95_high", "tight_bound_pct_mean", "tight_bound_pct_ci95_low", "tight_bound_pct_ci95_high"];
    res.csv("sensitivity.csv", &table(&vheader, &vrows));

    // ---- Diagnostic: why do micro-cycles need *more* liquidity than gross with onward echoes?
    // Compare, on the same payments, an engine that only delays (hold: every payment exactly
    // 10 s, arrival order kept) with engines that settle some payments early and others late.
    let dspecs: Vec<EngineSpec> = ["gross", "hold:10000", "deferred:8640", "micro:all:500:10000", "micro:all:10000:10000"].iter().map(|s| EngineSpec::parse(s)).collect::<Result<_, _>>()?;
    let dcases: Vec<(&str, ScenarioConfig)> = vec![
        ("no echo", base.clone()),
        ("20% onward echoes at 1 s", ScenarioConfig { echo_share: 0.2, echo_delay_ms: 1_000.0, ..base.clone() }),
        ("50% onward echoes at 1 s", ScenarioConfig { echo_share: 0.5, echo_delay_ms: 1_000.0, ..base.clone() }),
    ];
    let mut djobs = Vec::new();
    for c in 0..dcases.len() {
        for &seed in &seeds {
            djobs.push((c, seed));
        }
    }
    let ddone: Vec<Vec<f64>> = par_map(&djobs, |&(c, seed)| {
        let o = run_engines(&ScenarioConfig { seed, ..dcases[c].1.clone() }, &dspecs, &Opening::Unlimited, false, None);
        let ub = o.ub();
        o.results[1..].iter().map(|r| 100.0 * r.saving_vs(ub)).collect()
    });
    let mut drows = Vec::new();
    for (c, (label, _)) in dcases.iter().enumerate() {
        let per: Vec<&Vec<f64>> = djobs.iter().zip(&ddone).filter(|(j, _)| j.0 == c).map(|(_, v)| v).collect();
        for (k, spec) in dspecs[1..].iter().enumerate() {
            let (m, lo, hi) = mean_ci95(&per.iter().map(|v| v[k]).collect::<Vec<_>>());
            drows.push(vec![label.to_string(), spec.build(1).name(), per.len().to_string(), f(m), f(lo), f(hi)]);
        }
    }
    let dheader = ["case", "engine", "seeds", "saving_pct_mean", "saving_pct_ci95_low", "saving_pct_ci95_high"];
    res.csv("order_diagnostic.csv", &table(&dheader, &drows));
    eprintln!("{}\n{}", table(&vheader, &vrows), table(&dheader, &drows));
    res.set("scenario", scenario_json(base));
    res.set("seeds", Json::Arr(seeds.iter().map(|&s| Json::from(s)).collect()));
    res.set("cases_10s", Json::Arr(js));
    res.set(
        "kill_criterion",
        Json::obj()
            .set("rule", format!("the claim is conditional if the 10 s tight bound (upper end of its 95% CI) exceeds {KILL_PCT}% of UB for an echo share of at most 20%"))
            .set("worst_ci95_high_pct", worst_kill)
            .set("fires", worst_kill > KILL_PCT),
    );
    eprintln!("{}", table(&header, &rows));
    res.finish();
    Ok(())
}

/// `echo-scale`: the case that fired the kill criterion of `echo-sweep`, at growing volume, with
/// and without the echoes, and the fitted exponent of the bound in the number of payments.
pub fn echo_scale(path: &str, doc: &Doc, base: &ScenarioConfig) -> Result<(), String> {
    let mut res = Results::new("echo-scale", base.seed, path);
    let seeds: Vec<u64> = f64_list(doc, "echo_scale.seeds", &[20260930.0, 1.0]).iter().map(|&s| s as u64).collect();
    let scales = f64_list(doc, "echo_scale.payments_per_day", &[1e6, 3e6]);
    let share = doc.f64("echo_scale.share").unwrap_or(0.2);
    let delay = doc.f64("echo_scale.delay_ms").unwrap_or(1_000.0);
    let ret = doc.f64("echo_scale.return_prob").unwrap_or(1.0);
    // (payments a day, with echoes?)
    let mut cases = Vec::new();
    for &ppd in &scales {
        // only with echoes: the same scales without them are in `robustness/scale.csv`
        for echo in [true] {
            cases.push((ppd, echo));
        }
    }
    let mut jobs = Vec::new();
    for c in 0..cases.len() {
        for &seed in &seeds {
            jobs.push((c, seed));
        }
    }
    let t0 = std::time::Instant::now();
    let done: Vec<Measure> = par_map(&jobs, |&(c, seed)| {
        let (ppd, echo) = cases[c];
        measure(&ScenarioConfig {
            seed,
            payments_per_day: ppd,
            scale_factor: base.scale_factor * base.payments_per_day / ppd,
            echo_share: if echo { share } else { 0.0 },
            echo_delay_ms: delay,
            echo_return_prob: ret,
            ..base.clone()
        })
    });
    res.measure("simulation_seconds", t0.elapsed().as_secs_f64());
    // 10 s engines of ENGINES: index of "deferred:8640"
    let k = ENGINES.iter().position(|(n, _)| *n == "deferred:8640").expect("deferred:8640 is in ENGINES");
    let mut rows = Vec::new();
    let mut pts = Vec::new(); // (ln payments, ln mean tight bound) with echoes
    for (c, &(ppd, echo)) in cases.iter().enumerate() {
        let ms: Vec<&Measure> = jobs.iter().zip(&done).filter(|(j, _)| j.0 == c).map(|(_, m)| m).collect();
        let (sv, slo, shi) = mean_ci95(&ms.iter().map(|m| m.saving_pct[k]).collect::<Vec<_>>());
        let (tb, tlo, thi) = mean_ci95(&ms.iter().map(|m| m.tight_pct[k]).collect::<Vec<_>>());
        // the same bound in money per payment: does the *absolute* recoverable amount grow?
        let (ab, _, _) = mean_ci95(&ms.iter().map(|m| m.tight_pct[k] / 100.0 * m.ub as f64 / 100.0).collect::<Vec<_>>());
        let (ubg, _, _) = mean_ci95(&ms.iter().map(|m| m.ub as f64 / m.gross.max(1) as f64).collect::<Vec<_>>());
        rows.push(vec![f(ppd), if echo { f(share) } else { f(0.0) }, ms.len().to_string(), f(sv), f(slo), f(shi), f(tb), f(tlo), f(thi), f(ab), f(ubg)]);
        if echo && tb > 0.0 {
            pts.push((ppd.ln(), tb.ln(), ab.max(1e-9).ln(), ubg.ln()));
        }
    }
    // least-squares slopes in log-log
    let slope = |ys: &dyn Fn(&(f64, f64, f64, f64)) -> f64| -> f64 {
        let n = pts.len() as f64;
        let (mx, my) = (pts.iter().map(|p| p.0).sum::<f64>() / n, pts.iter().map(ys).sum::<f64>() / n);
        let sxy: f64 = pts.iter().map(|p| (p.0 - mx) * (ys(p) - my)).sum();
        let sxx: f64 = pts.iter().map(|p| (p.0 - mx).powi(2)).sum();
        if sxx > 0.0 { sxy / sxx } else { f64::NAN }
    };
    let header = [
        "payments_per_day", "echo_share", "seeds", "netting_10s_saving_pct_mean", "saving_ci95_low", "saving_ci95_high", "tight_bound_10s_pct_mean", "tight_ci95_low", "tight_ci95_high",
        "tight_bound_10s_brl_mean", "ub_over_gross_mean",
    ];
    res.csv("scale.csv", &table(&header, &rows));
    res.set("scenario", scenario_json(base));
    res.set("echo", Json::obj().set("share", share).set("delay_ms", delay).set("return_prob", ret));
    res.set(
        "loglog_slopes_with_echo",
        Json::obj()
            .set("tight_bound_pct_of_ub", slope(&|p| p.1))
            .set("tight_bound_in_money", slope(&|p| p.2))
            .set("ub_over_gross", slope(&|p| p.3))
            .set("note", "exponent b in y ~ payments^b, least squares over the scales; one value per scale (the mean over seeds)"),
    );
    eprintln!("{}", table(&header, &rows));
    res.finish();
    Ok(())
}
