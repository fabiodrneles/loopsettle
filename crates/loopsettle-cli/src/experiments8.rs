//! `circularity-horizon` (specs/08, T3): the bridge between the static circularity ladder and
//! what engines recover in time.
//!
//! For a window width `w`, the day is cut into windows of that width; inside each window the
//! payments are aggregated into a flow graph and its maximum circulation (CDFD) is computed.
//! `C(w)` = the sum over windows, as a share of gross value: the value that *could be cancelled*
//! by netting inside windows of width `w`. Next to it, for the same `w` as a deadline: the tight
//! bound (UB minus the liquidity floors) and what netting every `w` realises, as shares of UB.
//! Cancelled value is not saved liquidity; the experiment puts the two side by side.

use crate::experiments::{f64_list, scenario_json};
use crate::out::{f, table, Results};
use crate::stats::{mean_ci95, par_map};
use crate::svg::{Chart, Series};
use loopsettle_core::json::Json;
use loopsettle_core::MS_PER_DAY;
use loopsettle_sim::bound::liquidity_floors;
use loopsettle_sim::toml_lite::Doc;
use loopsettle_sim::{run as run_engines, EngineSpec, Opening, PaymentStream, ScenarioConfig};
use loopsettle_topology::{cdfd_max, FlowGraph};
use std::collections::BTreeMap;

/// One width's numbers for one seed: (circulating share of gross, bilateral share of gross,
/// tight bound % of UB, realised saving % of UB).
type Row = (f64, f64, f64, f64);

/// Maximum circulation and bilateral offset of one window's payments, in cents.
/// Participants are renumbered to the ones present, so tiny windows build tiny graphs.
fn window_circulation(pairs: &BTreeMap<(u32, u32), i64>) -> (i128, i128) {
    if pairs.is_empty() {
        return (0, 0);
    }
    let mut id: BTreeMap<u32, u32> = BTreeMap::new();
    for &(a, b) in pairs.keys() {
        for x in [a, b] {
            let next = id.len() as u32;
            id.entry(x).or_insert(next);
        }
    }
    let triples: Vec<(u32, u32, i64)> = pairs.iter().map(|(&(a, b), &v)| (id[&a], id[&b], v)).collect();
    let g = FlowGraph::from_triples(id.len(), &triples);
    let bilateral = g.total_flow() - g.net().total_flow();
    (cdfd_max(&g).circular_total, bilateral)
}

pub fn circularity_horizon(path: &str, doc: &Doc, base: &ScenarioConfig) -> Result<(), String> {
    let mut res = Results::new("circularity-horizon", base.seed, path);
    let seeds: Vec<u64> = f64_list(doc, "horizon.seeds", &[20260930.0, 1.0, 2.0]).iter().map(|&s| s as u64).collect();
    let widths_ms: Vec<u64> = f64_list(doc, "horizon.window_s", &[10.0, 60.0, 900.0, 3_600.0, 86_400.0]).iter().map(|&s| (s * 1000.0) as u64).collect();
    if widths_ms.iter().any(|&w| w == 0 || MS_PER_DAY % w != 0) {
        return Err("horizon.window_s: every width must divide a day".into());
    }
    let t0 = std::time::Instant::now();
    // per seed: per width (circulating share of gross, bilateral share of gross, tight bound %
    // of UB, realised saving % of UB), plus UB / gross
    let done: Vec<(Vec<Row>, f64)> = par_map(&seeds, |&seed| {
        let cfg = ScenarioConfig { seed, ..base.clone() };
        let k = widths_ms.len();
        let mut acc: Vec<BTreeMap<(u32, u32), i64>> = vec![BTreeMap::new(); k];
        let mut end: Vec<u64> = widths_ms.clone();
        let (mut circ, mut bil) = (vec![0i128; k], vec![0i128; k]);
        let mut gross: i128 = 0;
        for p in PaymentStream::new(&cfg) {
            gross += p.amount as i128;
            for j in 0..k {
                if p.t_arrival >= end[j] {
                    let (c, b) = window_circulation(&acc[j]);
                    circ[j] += c;
                    bil[j] += b;
                    acc[j].clear();
                    end[j] = (p.t_arrival / widths_ms[j] + 1) * widths_ms[j];
                }
                *acc[j].entry((p.payer, p.payee)).or_insert(0) += p.amount;
            }
        }
        for j in 0..k {
            let (c, b) = window_circulation(&acc[j]);
            circ[j] += c;
            bil[j] += b;
        }
        let mut specs = vec![EngineSpec::Gross];
        specs.extend(widths_ms.iter().map(|&w| EngineSpec::Deferred { cycles_per_day: (MS_PER_DAY / w) as u32 }));
        let o = run_engines(&cfg, &specs, &Opening::Unlimited, false, None);
        let ub = o.ub();
        let floors = liquidity_floors(&cfg, &widths_ms);
        let g = gross.max(1) as f64;
        let rows = (0..k)
            .map(|j| {
                let floor: i128 = floors[j].iter().map(|&v| v as i128).sum();
                (100.0 * circ[j] as f64 / g, 100.0 * bil[j] as f64 / g, 100.0 * (ub - floor) as f64 / ub.max(1) as f64, 100.0 * o.results[j + 1].saving_vs(ub))
            })
            .collect();
        (rows, ub as f64 / g)
    });
    res.measure("simulation_seconds", t0.elapsed().as_secs_f64());
    let ub_gross = mean_ci95(&done.iter().map(|d| d.1).collect::<Vec<_>>()).0;
    let mut rows = Vec::new();
    let (mut s_circ, mut s_tight, mut s_real) = (Vec::new(), Vec::new(), Vec::new());
    for (j, &w) in widths_ms.iter().enumerate() {
        let col = |g: &dyn Fn(&Row) -> f64| -> Vec<f64> { done.iter().map(|d| g(&d.0[j])).collect() };
        let (c, clo, chi) = mean_ci95(&col(&|x| x.0));
        let b = mean_ci95(&col(&|x| x.1)).0;
        let (t, tlo, thi) = mean_ci95(&col(&|x| x.2));
        let (r, rlo, rhi) = mean_ci95(&col(&|x| x.3));
        let ws = w as f64 / 1000.0;
        // the tight bound as a share of gross value, to compare with the circulating share
        let t_gross = t * ub_gross;
        rows.push(vec![f(ws), seeds.len().to_string(), f(c), f(clo), f(chi), f(b), f(t), f(tlo), f(thi), f(r), f(rlo), f(rhi), f(t_gross), f(if t_gross > 0.0 { c / t_gross } else { 0.0 })]);
        s_circ.push((ws, c));
        s_tight.push((ws, t_gross));
        s_real.push((ws, r * ub_gross));
    }
    let header = [
        "window_s", "seeds", "circulating_value_pct_of_gross", "circulating_ci95_low", "circulating_ci95_high", "bilateral_offset_pct_of_gross", "tight_bound_pct_of_ub", "tight_ci95_low", "tight_ci95_high",
        "netting_every_w_saving_pct_of_ub", "saving_ci95_low", "saving_ci95_high", "tight_bound_pct_of_gross", "circulating_over_recoverable",
    ];
    res.csv("horizon.csv", &table(&header, &rows));
    let chart = Chart {
        title: "Value that circulates within a window vs liquidity recoverable with that deadline".into(),
        x_label: "window / deadline (s, log)".into(),
        y_label: "% of gross value".into(),
        series: vec![
            Series { label: "circulating value (max circulation per window)".into(), points: s_circ, line: true },
            Series { label: "recoverable liquidity: tight bound".into(), points: s_tight, line: true },
            Series { label: "recovered by netting every window".into(), points: s_real, line: true },
        ],
        log_x: true,
    };
    res.csv("fig_horizon.svg", &chart.render());
    res.set("scenario", scenario_json(base));
    res.set("seeds", Json::Arr(seeds.iter().map(|&s| Json::from(s)).collect()));
    res.set("ub_over_gross_mean", ub_gross);
    res.set("note", "circulating value = sum over windows of the maximum circulation (CDFD) of the window's aggregated flows; it counts value that could be cancelled, not liquidity saved");
    eprintln!("{}", table(&header, &rows));
    res.finish();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The triangle of the topology tests: A->B 10, B->C 5, C->A 5 circulates 5 on each of the
    /// three edges (15 in all); a reciprocal pair of 7 and 4 offsets 4 each way (8).
    /// Nested windows never circulate less: a circulation of each half is a circulation of the
    /// union (capacities add), so the maximum circulation is superadditive and C(w) cannot fall
    /// when windows are merged. Checked on random payments and windows that divide each other.
    #[test]
    fn circulating_value_never_falls_when_windows_merge() {
        let mut rng = loopsettle_core::rng::Rng::new(77);
        for _ in 0..200 {
            let k = 2 + rng.below(6) as u32;
            let pays: Vec<(u64, u32, u32, i64)> = (0..rng.below(120)).map(|_| {
                let a = rng.below(k as u64) as u32;
                (rng.below(8_000), a, (a + 1 + rng.below(k as u64 - 1) as u32) % k, 1 + rng.below(500) as i64)
            }).collect();
            let c = |w: u64| -> i128 {
                let mut by: BTreeMap<u64, BTreeMap<(u32, u32), i64>> = BTreeMap::new();
                for &(t, a, b, v) in &pays {
                    *by.entry(t / w).or_default().entry((a, b)).or_insert(0) += v;
                }
                by.values().map(|m| window_circulation(m).0).sum()
            };
            let (c1, c2, c4, c8) = (c(1_000), c(2_000), c(4_000), c(8_000));
            assert!(c1 <= c2 && c2 <= c4 && c4 <= c8, "{c1} {c2} {c4} {c8}");
        }
    }

    #[test]
    fn window_circulation_by_hand() {
        let mut m = BTreeMap::new();
        m.insert((7u32, 3u32), 10i64);
        m.insert((3, 9), 5);
        m.insert((9, 7), 5);
        assert_eq!(window_circulation(&m), (15, 0));
        let mut m = BTreeMap::new();
        m.insert((1u32, 2u32), 7i64);
        m.insert((2, 1), 4);
        assert_eq!(window_circulation(&m), (8, 8));
        assert_eq!(window_circulation(&BTreeMap::new()), (0, 0));
    }
}
