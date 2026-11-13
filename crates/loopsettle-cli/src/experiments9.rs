//! `bof-replication`: the design of Koponen & Soramaki (1998, Bank of Finland Studies E:14) run
//! on the calibrated Pix scenario.
//!
//! The 1998 study gives every bank a liquidity level between its two theoretical bounds,
//! `LA_i = LB_i + LL (UB_i - LB_i)` with `LL` in 0, 10%, ..., 100% (equation 4), lets payments
//! queue for the rest of the day, and compares designs by the settlement delay indicator rho
//! (equation 5) and the liquidity usage indicator pi (equation 6). Its finding on optimisation
//! was that netting the queues every 20, 5 or 1 minutes reduces delay "only with minor effect"
//! (section 7.4.3). This experiment measures the same curve for an instant-payment flow, with
//! the cycle search of LoopSettle and the Bech-Soramaki gridlock resolution beside the 1998
//! complete netting.

use crate::experiments::{f64_list, scenario_json};
use crate::out::{f, table, Results};
use crate::stats::{mean_ci95, par_map};
use crate::svg::{Chart, Series};
use loopsettle_core::json::Json;
use loopsettle_core::{pi, Cents};
use loopsettle_sim::toml_lite::Doc;
use loopsettle_sim::{run as run_engines, EngineSpec, Opening, ScenarioConfig};

#[derive(Clone, Copy)]
struct Point {
    rho: f64,
    pi: f64,
    rejected_value_pct: f64,
}

/// Opening balances at liquidity level `ll` (equation 4 of the 1998 study).
pub(crate) fn level_opening(ll: f64, lbs: &[Cents], ubs: &[Cents]) -> Vec<Cents> {
    lbs.iter()
        .zip(ubs)
        .map(|(&lb, &ub)| lb + (ll * (ub - lb) as f64).floor() as Cents)
        .collect()
}

pub fn bof_replication(path: &str, doc: &Doc, base: &ScenarioConfig) -> Result<(), String> {
    let mut res = Results::new("bof-replication", base.seed, path);
    let seeds: Vec<u64> = f64_list(doc, "bof.seeds", &[20260930.0, 1.0, 2.0])
        .iter()
        .map(|&s| s as u64)
        .collect();
    let levels = f64_list(
        doc,
        "bof.levels",
        &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
    );
    if !levels.windows(2).all(|w| w[0] < w[1]) || levels.iter().any(|&l| !(0.0..=1.0).contains(&l))
    {
        return Err("bof.levels must be strictly increasing, within [0, 1]".into());
    }
    let engine_names = doc.str_array("bof.engines").unwrap_or_else(|| {
        vec![
            "queue:fifo:86400000:1000".into(),
            "netq:fifo:1200000:86400000:1000".into(),
            "netq:fifo:300000:86400000:1000".into(),
            "netq:fifo:60000:86400000:1000".into(),
        ]
    });
    let specs: Vec<EngineSpec> = engine_names
        .iter()
        .map(|s| EngineSpec::parse(s))
        .collect::<Result<_, _>>()?;
    let t_end = base.horizon_ms();
    // ASSUMPTION: the large-value scenario is the stylised one of `regimes` (20 participants,
    // median payment R$ 1 million) at a volume of the order of the 1998/2001 data (about 1,000
    // payments a day): the regime where those studies found gridlocks.
    let wholesale_ppd = doc.f64("bof.wholesale_payments_per_day").unwrap_or(1_500.0);
    let mut scenarios = vec![("retail (Pix)".to_string(), base.clone())];
    if wholesale_ppd > 0.0 {
        scenarios.push((
            format!("large-value {wholesale_ppd:.0}/day"),
            crate::experiments5::wholesale_config(base, wholesale_ppd),
        ));
    }
    let mut jobs = Vec::new();
    for sc in 0..scenarios.len() {
        for &seed in &seeds {
            jobs.push((sc, seed));
        }
    }

    let t0 = std::time::Instant::now();
    // per (scenario, seed): [level][engine] -> Point
    let done = par_map(&jobs, |&(sc, seed)| {
        let cfg = ScenarioConfig {
            seed,
            ..scenarios[sc].1.clone()
        };
        let free = run_engines(&cfg, &[EngineSpec::Gross], &Opening::Unlimited, false, None);
        let n = cfg.participants as u32;
        let ubs = free.flow.ubs();
        let lbs: Vec<Cents> = (0..n).map(|p| free.flow.lb(p)).collect();
        let gross = free.flow.gross_value.max(1);
        levels
            .iter()
            .map(|&ll| {
                let o = run_engines(
                    &cfg,
                    &specs,
                    &Opening::Given(level_opening(ll, &lbs, &ubs)),
                    false,
                    None,
                );
                o.results
                    .iter()
                    .map(|r| Point {
                        rho: r.metrics.rho(t_end),
                        pi: pi(r.liquidity, gross),
                        rejected_value_pct: 100.0 * r.metrics.rejected_value as f64 / gross as f64,
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    });
    eprintln!(
        "  {} scenarios x {} seeds x {} levels done in {:.0} s",
        scenarios.len(),
        seeds.len(),
        levels.len(),
        t0.elapsed().as_secs_f64()
    );
    res.measure("simulation_seconds", t0.elapsed().as_secs_f64());

    let labels: Vec<String> = specs.iter().map(|s| s.build(1).name()).collect();
    let mut rows = Vec::new();
    let mut heads = Vec::new();
    for (sc, (sname, _)) in scenarios.iter().enumerate() {
        let done: Vec<&Vec<Vec<Point>>> = jobs
            .iter()
            .zip(&done)
            .filter(|(j, _)| j.0 == sc)
            .map(|(_, d)| d)
            .collect();
        let mut fig = Vec::new();
        for (e, label) in labels.iter().enumerate() {
            let mut curve = Vec::new();
            for (li, &ll) in levels.iter().enumerate() {
                let col = |g: &dyn Fn(&Point) -> f64| -> Vec<f64> {
                    done.iter().map(|d| g(&d[li][e])).collect()
                };
                let (rho, rlo, rhi) = mean_ci95(&col(&|p| p.rho));
                let (pim, plo, phi) = mean_ci95(&col(&|p| p.pi));
                let (rej, _, _) = mean_ci95(&col(&|p| p.rejected_value_pct));
                // paired difference in rho with the first engine (the plain queue), per seed
                let diff: Vec<f64> = done.iter().map(|d| d[li][e].rho - d[li][0].rho).collect();
                let (dm, dlo, dhi) = mean_ci95(&diff);
                rows.push(vec![
                    sname.clone(),
                    f(ll),
                    label.clone(),
                    seeds.len().to_string(),
                    f(rho),
                    f(rlo),
                    f(rhi),
                    f(pim),
                    f(plo),
                    f(phi),
                    f(rej),
                    f(dm),
                    f(dlo),
                    f(dhi),
                ]);
                curve.push((100.0 * pim, 100.0 * rho));
            }
            fig.push(Series {
                label: label.clone(),
                points: curve,
                line: true,
            });
        }
        let chart = Chart {
            title: format!(
                "Liquidity usage vs settlement delay, {sname} (Koponen & Soramaki 1998 design)"
            ),
            x_label: "liquidity usage pi (% of gross value)".into(),
            y_label: "settlement delay rho (%)".into(),
            series: fig,
            log_x: false,
        };
        res.csv(
            &format!(
                "fig_bof_{}.svg",
                if sc == 0 { "retail" } else { "large_value" }
            ),
            &chart.render(),
        );

        // headline: the largest reduction of rho, relative to the plain queue, that any other
        // engine achieves across levels (mean over seeds)
        let mut best = (0.0f64, 0.0f64, String::new(), 0.0f64);
        for (li, &ll) in levels.iter().enumerate() {
            for (e, label) in labels.iter().enumerate().skip(1) {
                let k = done.len() as f64;
                let q: f64 = done.iter().map(|s| s[li][0].rho).sum::<f64>() / k;
                let d: f64 = done
                    .iter()
                    .map(|s| s[li][0].rho - s[li][e].rho)
                    .sum::<f64>()
                    / k;
                if d > best.0 {
                    best = (d, if q > 0.0 { d / q } else { 0.0 }, label.clone(), ll);
                }
            }
        }
        heads.push(
            Json::obj()
                .set("scenario", sname.as_str())
                .set("largest_rho_reduction_vs_queue", best.0)
                .set("largest_rho_reduction_relative", best.1)
                .set("engine", best.2.as_str())
                .set("liquidity_level", best.3),
        );
    }
    res.csv(
        "bof.csv",
        &table(
            &[
                "scenario",
                "liquidity_level",
                "engine",
                "seeds",
                "rho_mean",
                "rho_ci95_low",
                "rho_ci95_high",
                "pi_mean",
                "pi_ci95_low",
                "pi_ci95_high",
                "rejected_value_pct_mean",
                "rho_minus_first_engine_mean",
                "rho_minus_first_engine_ci95_low",
                "rho_minus_first_engine_ci95_high",
            ],
            &rows,
        ),
    );
    res.set("scenario", scenario_json(base));
    res.set(
        "engines",
        Json::Arr(labels.iter().map(|l| Json::from(l.as_str())).collect()),
    );
    res.set(
        "seeds",
        Json::Arr(seeds.iter().map(|&s| Json::from(s)).collect()),
    );
    res.set("headline", Json::Arr(heads));
    res.set(
        "note",
        "rho and pi as in Koponen & Soramaki (1998), equations 5 and 6; levels LB + LL (UB - LB), equation 4; \
         rejected payments count as waiting until the end of the day",
    );
    res.finish();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::level_opening;

    #[test]
    fn levels_interpolate_between_the_bounds() {
        assert_eq!(level_opening(0.0, &[3, 0], &[10, 7]), vec![3, 0]);
        assert_eq!(level_opening(1.0, &[3, 0], &[10, 7]), vec![10, 7]);
        assert_eq!(level_opening(0.5, &[3, 0], &[10, 7]), vec![6, 3]);
    }
}
