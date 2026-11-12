//! W2 and W2b (docs/PAPER_READINESS.md).
//!
//! `bias-calibration` — fit the receiver bias to the only real-data distribution available:
//!   BCB WP 619, eq. (3.2), per-bank nettable ratio 2·min(out, in)/(out + in), quartiles
//!   p25 18–45%, median 57–80%, p75 73–90% on real Pix data (PDF p. 16).
//! `heterogeneity` — class-specific intraday profiles (strength λ) x scale: does any
//!   second-scale mechanism become worth it (kill criterion K1: 10 s bound > 2% of UB at 30M)?

use crate::experiments::{f64_list, scenario_json};
use crate::out::{f, table, Results};
use loopsettle_core::json::Json;
use loopsettle_sim::bound::{delay_bounds, liquidity_floors};
use loopsettle_sim::toml_lite::Doc;
use crate::stats::{par_map, quantile};
use loopsettle_sim::{run as run_engines, EngineSpec, Opening, RunOutput, ScenarioConfig};

/// WP 619 target ranges (p25, median, p75), as fractions.
const WP619: [(f64, f64); 3] = [(0.18, 0.45), (0.57, 0.80), (0.73, 0.90)];

/// Per-bank nettable ratio quartiles (WP 619 eq. 3.2) of one simulated day.
fn wp619_quartiles(o: &RunOutput) -> [f64; 3] {
    let mut v: Vec<f64> = (0..o.flow.n)
        .filter_map(|i| {
            let (out, inn) = (o.flow.out_value[i] as f64, o.flow.in_value[i] as f64);
            (out + inn > 0.0).then(|| 2.0 * out.min(inn) / (out + inn))
        })
        .collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    [quantile(&v, 0.25), quantile(&v, 0.5), quantile(&v, 0.75)]
}

/// Squared distance to the midpoints of the WP 619 ranges, in percentage points.
fn loss(q: &[f64; 3]) -> f64 {
    q.iter().zip(WP619).map(|(x, (lo, hi))| (100.0 * (x - (lo + hi) / 2.0)).powi(2)).sum()
}

fn inside(q: &[f64; 3]) -> usize {
    q.iter().zip(WP619).filter(|(x, (lo, hi))| **x >= *lo && **x <= *hi).count()
}

pub fn bias_calibration(path: &str, doc: &Doc, base: &ScenarioConfig) -> Result<(), String> {
    let mut res = Results::new("bias-calibration", base.seed, path);
    let sigmas = f64_list(doc, "calibration.sigmas", &[0.4, 0.6, 0.8, 1.0, 1.2, 1.5, 2.0, 2.5]);
    let mults = f64_list(doc, "calibration.pi_receiver_mult", &[1.3, 2.0, 3.0]);
    let seeds: Vec<u64> = f64_list(doc, "calibration.seeds", &[20260930.0, 1.0, 2.0]).iter().map(|&s| s as u64).collect();
    let mut rows = Vec::new();
    let mut best: Option<(f64, f64, f64, [f64; 3], f64)> = None;
    for &sigma in &sigmas {
        for &mult in &mults {
            let mut acc = [0.0; 3];
            let mut lbub = 0.0;
            for &seed in &seeds {
                let cfg = ScenarioConfig { seed, receiver_bias_sigma: sigma, pi_receiver_mult: mult, ..base.clone() };
                let o = run_engines(&cfg, &[EngineSpec::Gross], &Opening::Unlimited, false, None);
                let q = wp619_quartiles(&o);
                for k in 0..3 {
                    acc[k] += q[k] / seeds.len() as f64;
                }
                lbub += o.lb() as f64 / o.ub().max(1) as f64 / seeds.len() as f64;
            }
            let l = loss(&acc);
            rows.push(vec![f(sigma), f(mult), f(acc[0]), f(acc[1]), f(acc[2]), inside(&acc).to_string(), f(l), f(lbub)]);
            if best.map_or(true, |b| l < b.2) {
                best = Some((sigma, mult, l, acc, lbub));
            }
        }
        eprintln!("  sigma {sigma} done");
    }
    let header = ["receiver_bias_sigma", "pi_receiver_mult", "ratio_p25", "ratio_p50", "ratio_p75", "quartiles_inside_wp619_ranges", "loss_pp2", "lb_over_ub"];
    res.csv("grid.csv", &table(&header, &rows));
    let (bs, bm, bl, bq, blb) = best.unwrap();
    res.set("scenario", scenario_json(base));
    res.set("target", Json::obj().set("source", "BCB WP 619 eq. (3.2), PDF p. 16").set("p25", "18-45%").set("median", "57-80%").set("p75", "73-90%"));
    res.set(
        "best",
        Json::obj()
            .set("receiver_bias_sigma", bs)
            .set("pi_receiver_mult", bm)
            .set("loss_pp2", bl)
            .set("ratio_p25", bq[0])
            .set("ratio_p50", bq[1])
            .set("ratio_p75", bq[2])
            .set("quartiles_inside_wp619_ranges", inside(&bq) as u64)
            .set("lb_over_ub", blb),
    );
    eprintln!("{}", table(&header, &rows));
    eprintln!("best: sigma {bs}, pi_receiver_mult {bm}, quartiles {bq:?}, loss {bl:.1}");
    res.finish();
    Ok(())
}

const ENGINES: &[(&str, u64)] = &[("micro:all:500:10000", 10_000), ("deferred:8640", 10_000), ("deferred:96", 900_000), ("deferred:1", 86_400_000)];

pub fn heterogeneity(path: &str, doc: &Doc, base: &ScenarioConfig) -> Result<(), String> {
    let mut res = Results::new("heterogeneity", base.seed, path);
    let lambdas = f64_list(doc, "heterogeneity.profile_strength", &[0.0, 0.5, 1.0]);
    let scales = f64_list(doc, "heterogeneity.payments_per_day", &[1e6, 10e6, 30e6]);
    let sigmas = f64_list(doc, "heterogeneity.receiver_bias_sigma", &[base.receiver_bias_sigma]);
    let mut rows = Vec::new();
    let mut k1 = Vec::new();
    let mut cases = Vec::new();
    for &sigma in &sigmas {
        for &lam in &lambdas {
            for &ppd in &scales {
                cases.push((sigma, lam, ppd));
            }
        }
    }
    let delays: Vec<u64> = ENGINES.iter().map(|&(_, d)| d).collect();
    let mut uniq = delays.clone();
    uniq.sort_unstable();
    uniq.dedup();
    // One case per thread; `par_map` returns the cases in the order above.
    let measured = par_map(&cases, |&(sigma, lam, ppd)| {
        let cfg = ScenarioConfig { profile_strength: lam, receiver_bias_sigma: sigma, payments_per_day: ppd, scale_factor: base.scale_factor * base.payments_per_day / ppd, ..base.clone() };
        let t0 = std::time::Instant::now();
        let mut specs = vec![EngineSpec::Gross];
        specs.extend(ENGINES.iter().map(|(s, _)| EngineSpec::parse(s).unwrap()));
        let o = run_engines(&cfg, &specs, &Opening::Unlimited, false, None);
        let bounds = delay_bounds(&cfg, &delays);
        let floors = liquidity_floors(&cfg, &uniq);
        eprintln!("  sigma {sigma} lambda {lam} {ppd:.0}/day: {:.1} s", t0.elapsed().as_secs_f64());
        (o, bounds, floors)
    });
    for (&(sigma, lam, ppd), (o, bounds, floors)) in cases.iter().zip(&measured) {
        let ub = o.ub();
        let pct = |b: &Vec<i64>| 100.0 * b.iter().map(|&v| v as i128).sum::<i128>() as f64 / ub.max(1) as f64;
        // tight bound = UB - sum of liquidity floors (METHODOLOGY section 9b)
        let tight = |d: u64| 100.0 * (ub - floors[uniq.iter().position(|&x| x == d).unwrap()].iter().map(|&v| v as i128).sum::<i128>()) as f64 / ub.max(1) as f64;
        let q = wp619_quartiles(o);
        for (k, (name, d)) in ENGINES.iter().enumerate() {
            rows.push(vec![
                f(sigma),
                f(lam),
                f(ppd),
                name.to_string(),
                d.to_string(),
                f(100.0 * o.results[k + 1].saving_vs(ub)),
                f(pct(&bounds[k])),
                f(o.lb() as f64 / ub.max(1) as f64),
                f(q[0]),
                f(q[1]),
                f(tight(*d)),
            ]);
        }
        if (ppd - 30e6).abs() < 1.0 {
            let b10 = pct(&bounds[0]);
            k1.push(
                Json::obj()
                    .set("receiver_bias_sigma", sigma)
                    .set("profile_strength", lam)
                    .set("bound_10s_pct_of_ub", b10)
                    .set("tight_bound_10s_pct_of_ub", tight(10_000))
                    .set("k1_fires", b10 > 2.0),
            );
        }
    }
    let header = ["receiver_bias_sigma", "profile_strength", "payments_per_day", "engine", "max_delay_ms", "saving_vs_ub_pct", "delay_bound_pct", "lb_over_ub", "wp619_ratio_p25", "wp619_ratio_p50", "tight_bound_pct"];
    res.csv("heterogeneity.csv", &table(&header, &rows));
    res.set("scenario", scenario_json(base));
    res.set("kill_criterion_k1", Json::obj().set("rule", "stop if the 10 s delay bound exceeds 2% of UB at 30M payments/day").set("cases", Json::Arr(k1)));
    eprintln!("{}", table(&header, &rows));
    // Several seeds (specs/09, E1), at the smallest scale only (each 30-million run takes about a
    // minute): for each receiver bias and profile strength, the 10 s tight bound, what netting
    // every 10 s and once a day save, and LB/UB.
    {
        let small = scales.first().copied().unwrap_or(1e6);
        let mut names = Vec::new();
        for &sigma in &sigmas {
            for &lam in &lambdas {
                let k = format!("sigma{}_lambda{}", f(sigma), f(lam));
                for m in ["tight_bound_10s_pct", "netting_10s_saving_pct", "netting_daily_saving_pct", "lb_over_ub"] {
                    names.push(format!("{m}_{k}"));
                }
            }
        }
        let base_small = ScenarioConfig { payments_per_day: small, scale_factor: base.scale_factor * base.payments_per_day / small, ..base.clone() };
        let k10 = crate::experiments3::ENGINES.iter().position(|(n, _)| *n == "deferred:8640").expect("in ENGINES");
        let k1 = crate::experiments3::ENGINES.iter().position(|(n, _)| *n == "deferred:1").expect("in ENGINES");
        crate::seeds::run_and_write(&mut res, doc, &base_small, &names, |c| {
            let mut v = Vec::new();
            for &sigma in &sigmas {
                for &lam in &lambdas {
                    let m = crate::experiments3::measure(&ScenarioConfig { receiver_bias_sigma: sigma, profile_strength: lam, ..c.clone() });
                    v.extend([m.tight_pct[k10], m.saving_pct[k10], m.saving_pct[k1], m.lb as f64 / m.ub.max(1) as f64]);
                }
            }
            v
        });
    }
    res.finish();
    Ok(())
}
