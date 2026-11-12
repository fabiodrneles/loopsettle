//! Several seeds for any experiment (specs/09, E1).
//!
//! The tables an experiment already writes stay as they are: they describe one seed and serve as
//! illustrations. Next to them, `seeds_summary.csv` gives, for each headline metric, its mean,
//! standard deviation, 95% confidence interval, minimum and maximum over the seeds, and
//! `seeds.csv` the value of every metric for every seed. Seeds come from `[experiment] seeds` in
//! the experiment file (default: the base seed and 1 to 9) and run in parallel; the output does
//! not depend on the number of threads.

use crate::out::{f, table, Results};
use crate::stats::{mean_ci95, mean_sd, par_map};
use loopsettle_core::json::Json;
use loopsettle_sim::toml_lite::Doc;
use loopsettle_sim::ScenarioConfig;

/// The seeds to run: `[experiment] seeds`, or the base seed followed by 1..9 (duplicates removed).
pub fn seed_list(doc: &Doc, base_seed: u64) -> Vec<u64> {
    let raw: Vec<u64> = match doc.f64_array("experiment.seeds") {
        Some(v) => v.into_iter().map(|x| x as u64).collect(),
        None => std::iter::once(base_seed).chain(1..=9).collect(),
    };
    let mut out = Vec::new();
    for s in raw {
        if !out.contains(&s) {
            out.push(s);
        }
    }
    out
}

/// Run `metrics` for every seed (in parallel) and write `seeds.csv` and `seeds_summary.csv`.
/// `metrics` returns one value per name in `names`; NaN means "not defined for this seed" and is
/// left out of that metric's statistics.
pub fn run_and_write(
    res: &mut Results,
    doc: &Doc,
    base: &ScenarioConfig,
    names: &[String],
    metrics: impl Fn(&ScenarioConfig) -> Vec<f64> + Sync,
) -> Vec<Vec<f64>> {
    let seeds = seed_list(doc, base.seed);
    let t0 = std::time::Instant::now();
    let per_seed: Vec<Vec<f64>> = par_map(&seeds, |&seed| {
        let v = metrics(&ScenarioConfig {
            seed,
            ..base.clone()
        });
        assert_eq!(v.len(), names.len(), "one value per metric name");
        v
    });
    eprintln!(
        "  {} seeds in {:.0} s",
        seeds.len(),
        t0.elapsed().as_secs_f64()
    );
    res.measure("seeds_seconds", t0.elapsed().as_secs_f64());

    let mut head = vec!["seed".to_string()];
    head.extend(names.iter().cloned());
    let head_ref: Vec<&str> = head.iter().map(String::as_str).collect();
    let rows: Vec<Vec<String>> = seeds
        .iter()
        .zip(&per_seed)
        .map(|(s, v)| {
            std::iter::once(s.to_string())
                .chain(v.iter().map(|&x| f(x)))
                .collect()
        })
        .collect();
    res.csv("seeds.csv", &table(&head_ref, &rows));

    let mut srows = Vec::new();
    let mut js = Vec::new();
    for (k, name) in names.iter().enumerate() {
        let vals: Vec<f64> = per_seed
            .iter()
            .map(|v| v[k])
            .filter(|x| x.is_finite())
            .collect();
        if vals.is_empty() {
            continue;
        }
        let (m, lo, hi) = mean_ci95(&vals);
        let (_, sd) = mean_sd(&vals);
        let (mn, mx) = vals
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |a, &x| {
                (a.0.min(x), a.1.max(x))
            });
        srows.push(vec![
            name.clone(),
            vals.len().to_string(),
            f(m),
            f(sd),
            f(lo),
            f(hi),
            f(mn),
            f(mx),
        ]);
        js.push(
            Json::obj()
                .set("metric", name.as_str())
                .set("seeds", vals.len() as u64)
                .set("mean", m)
                .set("ci95_low", lo)
                .set("ci95_high", hi),
        );
    }
    res.csv(
        "seeds_summary.csv",
        &table(
            &[
                "metric",
                "seeds",
                "mean",
                "sd",
                "ci95_low",
                "ci95_high",
                "min",
                "max",
            ],
            &srows,
        ),
    );
    res.set(
        "seeds",
        Json::Arr(seeds.iter().map(|&s| Json::from(s)).collect()),
    );
    res.set("seeds_summary", Json::Arr(js));
    per_seed
}
