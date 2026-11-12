//! `funding-windows` (W3) and `regimes` (reconciling with McMahon et al. 2024).
//!
//! funding-windows: the Conta PI can be topped up from reserves only while the STR is open
//! (6:30–18:30 on business days, plus a 30-minute top-up window: BCB, Relatório Anual do SPI
//! 2024, p. 15). Outside that window — nights and the whole weekend — a participant must have
//! prefunded what it will need. This experiment simulates Thursday to Monday with the sourced
//! hourly profile and day volumes, and reports, per window, the prefunding need, what one
//! netting at the end of the window would need, and the delay bound.
//!
//! regimes: the delay bound is (value paid near the peak) / (liquidity need). In retail instant
//! systems payments are tiny relative to liquidity needs; in large-value systems a few payments
//! can be a large share of them. This experiment computes the bound and the realised micro-cycle
//! saving in both regimes.

use crate::experiments::{f64_list, scenario_json};
use crate::out::{f, table, Results};
use loopsettle_core::json::Json;
use crate::stats::par_map;
use loopsettle_sim::bound::{delay_bounds, liquidity_floors, window_floors, window_stats};
use loopsettle_sim::scenario::{AmountModel, SPI_2024_HOURLY, SPI_2024_WEEKDAY};
use loopsettle_sim::toml_lite::Doc;
use loopsettle_sim::{run as run_engines, EngineSpec, Opening, PaymentStream, ScenarioConfig};

const H: u64 = 3_600_000;

pub fn funding_windows(path: &str, doc: &Doc, base: &ScenarioConfig) -> Result<(), String> {
    let mut res = Results::new("funding-windows", base.seed, path);
    // Thursday 00:00 to Monday 24:00. Day weights relative to Thursday (SPI 2024, p. 12).
    let thu = SPI_2024_WEEKDAY[3];
    let day_weights: Vec<f64> = [3usize, 4, 5, 6, 0].iter().map(|&d| SPI_2024_WEEKDAY[d] / thu).collect();
    let open_h = 6.5;
    let close_h = 19.0; // 18:30 regular close + 30-minute top-up window
    let at = |day: u64, hour: f64| day * 24 * H + (hour * H as f64) as u64;
    let windows: Vec<(&str, &str, u64, u64)> = vec![
        ("thu_day", "open", at(0, open_h), at(0, close_h)),
        ("thu_night", "closed", at(0, close_h), at(1, open_h)),
        ("fri_day", "open", at(1, open_h), at(1, close_h)),
        ("weekend", "closed", at(1, close_h), at(4, open_h)),
        ("mon_day", "open", at(4, open_h), at(4, close_h)),
    ];
    let delays: Vec<u64> = vec![10_000, 900_000, 3_600_000];
    let sigmas = f64_list(doc, "windows.receiver_bias_sigma", &[0.4, 1.0]);
    let lambdas = f64_list(doc, "windows.profile_strength", &[0.0, 1.0]);
    let scales = f64_list(doc, "windows.payments_per_thursday", &[1e6, 10e6]);
    let mut rows = Vec::new();
    let mut js = Vec::new();
    let mut cases = Vec::new();
    for &ppd in &scales {
        for &sigma in &sigmas {
            for &lam in &lambdas {
                cases.push((ppd, sigma, lam));
            }
        }
    }
    let ws: Vec<(u64, u64)> = windows.iter().map(|w| (w.2, w.3)).collect();
    // One case per thread; `par_map` returns the cases in the order above.
    let measured = par_map(&cases, |&(ppd, sigma, lam)| {
        let cfg = ScenarioConfig {
            days: 5,
            day_weights: day_weights.clone(),
            hourly: SPI_2024_HOURLY.to_vec(),
            payments_per_day: ppd,
            scale_factor: base.scale_factor * base.payments_per_day / ppd,
            receiver_bias_sigma: sigma,
            profile_strength: lam,
            ..base.clone()
        };
        let t0 = std::time::Instant::now();
        let st = window_stats(cfg.participants, || PaymentStream::new(&cfg), &ws, &delays);
        let fl = window_floors(cfg.participants, || PaymentStream::new(&cfg), &ws, &delays);
        eprintln!("  {ppd:.0}/Thursday sigma {sigma} lambda {lam}: {:.1} s", t0.elapsed().as_secs_f64());
        (st, fl)
    });
    {
        {
            for (&(ppd, sigma, lam), (st, fl)) in cases.iter().zip(&measured) {
                for (k, (name, kind, w_start, w_end)) in windows.iter().enumerate() {
                    let need: i128 = st[k].need.iter().map(|&v| v as i128).sum();
                    let net: i128 = st[k].net.iter().map(|&v| v as i128).sum();
                    let pct = |x: i128| if need == 0 { 0.0 } else { 100.0 * x as f64 / need as f64 };
                    let b: Vec<f64> = st[k].bound.iter().map(|v| pct(v.iter().map(|&x| x as i128).sum())).collect();
                    // tight bound = need - sum of window floors (METHODOLOGY section 10)
                    let tb: Vec<f64> = fl[k].iter().map(|v| pct(need - v.iter().map(|&x| x as i128).sum::<i128>())).collect();
                    rows.push(vec![
                        f(ppd),
                        f(sigma),
                        f(lam),
                        name.to_string(),
                        kind.to_string(),
                        f((b_hours(*w_start, *w_end) * 10.0).round() / 10.0),
                        f(need as f64 / 100.0),
                        f(net as f64 / 100.0),
                        f(100.0 - pct(net)),
                        f(b[0]),
                        f(b[1]),
                        f(b[2]),
                        f(tb[0]),
                        f(tb[1]),
                        f(tb[2]),
                    ]);
                    js.push(
                        Json::obj()
                            .set("payments_per_thursday", ppd)
                            .set("receiver_bias_sigma", sigma)
                            .set("profile_strength", lam)
                            .set("window", *name)
                            .set("kind", *kind)
                            .set("need_brl", need as f64 / 100.0)
                            .set("netting_saving_pct", 100.0 - pct(net))
                            .set("bound_10s_pct", b[0])
                            .set("bound_15min_pct", b[1])
                            .set("tight_bound_10s_pct", tb[0])
                            .set("tight_bound_15min_pct", tb[1]),
                    );
                }
            }
        }
    }
    let header = [
        "payments_per_thursday", "receiver_bias_sigma", "profile_strength", "window", "kind", "hours", "prefunding_need_brl", "end_of_window_netting_need_brl", "netting_saving_pct",
        "bound_10s_pct", "bound_15min_pct", "bound_1h_pct", "tight_bound_10s_pct", "tight_bound_15min_pct", "tight_bound_1h_pct",
    ];
    res.csv("windows.csv", &table(&header, &rows));

    // ---- Prefunding below the need (specs/05, M5). Each participant holds, when STR closes,
    // `alpha` times its largest need over the closed windows; top-ups are unlimited while STR is
    // open. Rejected value for lack of prefunding, under gross settlement and under designs that
    // let a payment wait (up to one hour) for receipts or for the reopening. Smallest scale only.
    let alphas = f64_list(doc, "windows.prefund_alphas", &[0.8, 0.9, 0.95, 1.0]);
    let pnames: Vec<String> = doc.str_array("windows.prefund_engines").unwrap_or_else(|| vec!["gross".into(), "queue:fifo:3600000:1000".into(), "micro:shortfall:1000:3600000:6:fifo".into()]);
    let pspecs: Vec<EngineSpec> = pnames.iter().map(|s| EngineSpec::parse(s)).collect::<Result<_, _>>()?;
    let closed: Vec<(u64, u64)> = windows.iter().filter(|w| w.1 == "closed").map(|w| (w.2, w.3)).collect();
    let small = scales.first().copied().unwrap_or(1e6);
    let pcases: Vec<(f64, f64)> = cases.iter().filter(|c| c.0 == small).map(|c| (c.1, c.2)).collect();
    let pdone: Vec<Vec<(i128, i128, i128)>> = par_map(&pcases, |&(sigma, lam)| {
        let cfg = ScenarioConfig {
            days: 5,
            day_weights: day_weights.clone(),
            hourly: SPI_2024_HOURLY.to_vec(),
            payments_per_day: small,
            scale_factor: base.scale_factor * base.payments_per_day / small,
            receiver_bias_sigma: sigma,
            profile_strength: lam,
            ..base.clone()
        };
        let n = cfg.participants;
        let st = window_stats(n, || PaymentStream::new(&cfg), &closed, &[]);
        let need: Vec<i64> = (0..n).map(|i| st.iter().map(|w| w.need[i]).max().unwrap_or(0)).collect();
        let total_need: i128 = need.iter().map(|&v| v as i128).sum();
        let mut out = Vec::new();
        for &a in &alphas {
            let prefund: Vec<i64> = need.iter().map(|&v| (v as f64 * a).floor() as i64).collect();
            let mut runs: Vec<loopsettle_core::EngineRun> = pspecs
                .iter()
                .map(|sp| loopsettle_core::EngineRun::new(sp.build(n), loopsettle_core::Ledger::unlimited(n)).with_funding(Box::new(loopsettle_core::PrefundClosedWindows::new(closed.clone(), prefund.clone()))))
                .collect();
            let mut flow = loopsettle_core::FlowStats::new(n, false);
            loopsettle_core::drive(PaymentStream::new(&cfg), &mut runs, &mut flow, cfg.horizon_ms());
            for r in &runs {
                out.push((r.metrics.rejected_value, flow.gross_value, total_need));
            }
        }
        out
    });
    let mut prow = Vec::new();
    for (c, &(sigma, lam)) in pcases.iter().enumerate() {
        for (ai, &a) in alphas.iter().enumerate() {
            for (e, name) in pnames.iter().enumerate() {
                let (rej, gross, need) = pdone[c][ai * pnames.len() + e];
                prow.push(vec![f(small), f(sigma), f(lam), f(a), name.clone(), f(need as f64 / 100.0), f(100.0 * rej as f64 / gross.max(1) as f64), f(rej as f64 / 100.0)]);
            }
        }
    }
    res.csv(
        "prefunding.csv",
        &table(&["payments_per_thursday", "receiver_bias_sigma", "profile_strength", "prefund_fraction_of_need", "engine", "total_need_brl", "rejected_value_pct_of_gross", "rejected_value_brl"], &prow),
    );
    res.set("scenario", scenario_json(base));
    res.set("source", "STR hours and day volumes: BCB, Relatório Anual do SPI 2024, pp. 12-15; hourly profile: Gráfico 12, p. 13");
    res.set("windows", Json::Arr(js));
    eprintln!("{}", table(&header, &rows));
    // Several seeds (specs/09, E1), smallest scale: per case and closed window, the prefunding
    // need (R$), the saving of one netting at the end of the window and the 10 s tight bound.
    {
        let mut names = Vec::new();
        for &(sigma, lam) in &pcases {
            for w in windows.iter().filter(|w| w.1 == "closed") {
                let k = format!("sigma{}_lambda{}_{}", f(sigma), f(lam), w.0);
                for m in ["prefunding_need_brl", "netting_saving_pct", "tight_bound_10s_pct"] {
                    names.push(format!("{m}_{k}"));
                }
            }
        }
        let closed_idx: Vec<usize> = windows.iter().enumerate().filter(|(_, w)| w.1 == "closed").map(|(i, _)| i).collect();
        crate::seeds::run_and_write(&mut res, doc, base, &names, |c| {
            let mut v = Vec::new();
            for &(sigma, lam) in &pcases {
                let cfg = ScenarioConfig {
                    seed: c.seed,
                    days: 5,
                    day_weights: day_weights.clone(),
                    hourly: SPI_2024_HOURLY.to_vec(),
                    payments_per_day: small,
                    scale_factor: base.scale_factor * base.payments_per_day / small,
                    receiver_bias_sigma: sigma,
                    profile_strength: lam,
                    ..base.clone()
                };
                let st = window_stats(cfg.participants, || PaymentStream::new(&cfg), &ws, &[]);
                let fl = window_floors(cfg.participants, || PaymentStream::new(&cfg), &ws, &[10_000]);
                for &k in &closed_idx {
                    let need: i128 = st[k].need.iter().map(|&x| x as i128).sum();
                    let net: i128 = st[k].net.iter().map(|&x| x as i128).sum();
                    let floor: i128 = fl[k][0].iter().map(|&x| x as i128).sum();
                    let pct = |x: i128| if need == 0 { 0.0 } else { 100.0 * x as f64 / need as f64 };
                    v.extend([need as f64 / 100.0, 100.0 - pct(net), pct(need - floor)]);
                }
            }
            v
        });
    }
    res.finish();
    Ok(())
}

fn b_hours(a: u64, b: u64) -> f64 {
    (b - a) as f64 / H as f64
}

/// ASSUMPTION: a stylised large-value system — 20 participants, payments with a median of
/// R$ 1 million and a heavy tail. Not calibrated to any real system.
pub(crate) fn wholesale_config(base: &ScenarioConfig, ppd: f64) -> ScenarioConfig {
    ScenarioConfig {
        participants: 20,
        n_big: 5,
        n_mid: 5,
        shares: Vec::new(),
        payments_per_day: ppd,
        amounts: AmountModel { body_median: 1.0e6, body_sigma: 2.0, tail_prob: 0.05, tail_xm: 2.0e7, tail_alpha: 1.3, max_amount: 5.0e9 },
        ..base.clone()
    }
}

pub fn regimes(path: &str, doc: &Doc, base: &ScenarioConfig) -> Result<(), String> {
    let mut res = Results::new("regimes", base.seed, path);
    let delays: Vec<u64> = vec![2_000, 5_000, 10_000];
    // ASSUMPTION: a stylised large-value system — 20 participants, payments with a median of
    // R$ 1 million and a heavy tail, a few thousand to a few hundred thousand payments a day.
    let wholesale = |ppd: f64| wholesale_config(base, ppd);
    let wholesale_ppd = f64_list(doc, "regimes.wholesale_payments_per_day", &[2e3, 2e4, 2e5]);
    let mut cases: Vec<(String, ScenarioConfig)> = vec![("retail (Pix calibrated)".into(), base.clone())];
    for &p in &wholesale_ppd {
        cases.push((format!("large-value {p:.0}/day"), wholesale(p)));
    }
    let specs = [EngineSpec::Gross, EngineSpec::parse("micro:all:500:2000").unwrap(), EngineSpec::parse("micro:all:500:5000").unwrap(), EngineSpec::parse("micro:all:500:10000").unwrap()];
    let mut rows = Vec::new();
    let mut js = Vec::new();
    for (name, cfg) in &cases {
        let o = run_engines(cfg, &specs, &Opening::Unlimited, false, None);
        let ub = o.ub();
        let bounds = delay_bounds(cfg, &delays);
        let floors = liquidity_floors(cfg, &delays);
        let pct = |b: &Vec<i64>| 100.0 * b.iter().map(|&v| v as i128).sum::<i128>() as f64 / ub.max(1) as f64;
        // tight bound = UB - sum of liquidity floors (METHODOLOGY section 9b)
        let tight = |fl: &Vec<i64>| 100.0 * (ub - fl.iter().map(|&v| v as i128).sum::<i128>()) as f64 / ub.max(1) as f64;
        // average payment relative to the average participant's liquidity need
        let avg_payment = o.flow.gross_value as f64 / o.flow.count.max(1) as f64;
        let size_ratio = avg_payment / (ub as f64 / cfg.participants as f64);
        for (k, &d) in delays.iter().enumerate() {
            rows.push(vec![name.clone(), f(cfg.participants as f64), f(o.flow.count as f64), f(size_ratio), d.to_string(), f(100.0 * o.results[k + 1].saving_vs(ub)), f(pct(&bounds[k])), f(tight(&floors[k]))]);
        }
        js.push(Json::obj().set("case", name.as_str()).set("avg_payment_over_avg_ub", size_ratio).set("bound_5s_pct", pct(&bounds[1])).set("tight_bound_5s_pct", tight(&floors[1])).set("micro_5s_saving_pct", 100.0 * o.results[2].saving_vs(ub)));
        eprintln!("  {name}: done");
    }
    let header = ["case", "participants", "payments", "avg_payment_over_avg_participant_ub", "max_delay_ms", "micro_cycle_saving_pct", "delay_bound_pct", "tight_bound_pct"];
    res.csv("regimes.csv", &table(&header, &rows));
    // W5: realised saving of deferred netting with interval d vs the delay bound for the same d,
    // retail scenario, d from 10 s to one day.
    let ds: [(u32, u64); 6] = [(8640, 10_000), (1440, 60_000), (96, 900_000), (24, 3_600_000), (4, 21_600_000), (1, 86_400_000)];
    let mut dspecs = vec![EngineSpec::Gross];
    dspecs.extend(ds.iter().map(|&(c, _)| EngineSpec::Deferred { cycles_per_day: c }));
    let o = run_engines(base, &dspecs, &Opening::Unlimited, false, None);
    let dd: Vec<u64> = ds.iter().map(|&(_, d)| d).collect();
    // Several seeds (the base seed first), in parallel: per seed, per delay, the realised
    // saving, the window bound and the tight bound, all in % of that seed's UB.
    let mut curve_seeds: Vec<u64> = vec![base.seed];
    curve_seeds.extend(f64_list(doc, "regimes.curve_seeds", &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]).iter().map(|&x| x as u64).filter(|&x| x != base.seed));
    let per_seed: Vec<Vec<(f64, f64, f64)>> = par_map(&curve_seeds, |&seed| {
        let cfg = ScenarioConfig { seed, ..base.clone() };
        let o = run_engines(&cfg, &dspecs, &Opening::Unlimited, false, None);
        let ub = o.ub();
        let bnd = delay_bounds(&cfg, &dd);
        let flo = liquidity_floors(&cfg, &dd);
        (0..dd.len())
            .map(|k| {
                let pct = |x: i128| 100.0 * x as f64 / ub.max(1) as f64;
                (100.0 * o.results[k + 1].saving_vs(ub), pct(bnd[k].iter().map(|&v| v as i128).sum()), pct(ub - flo[k].iter().map(|&v| v as i128).sum::<i128>()))
            })
            .collect()
    });
    let mut crow = Vec::new();
    let (mut real_pts, mut bound_pts, mut tight_pts) = (Vec::new(), Vec::new(), Vec::new());
    for (k, &(c, d)) in ds.iter().enumerate() {
        let col = |g: &dyn Fn(&(f64, f64, f64)) -> f64| -> Vec<f64> { per_seed.iter().map(|v| g(&v[k])).collect() };
        let (saving, slo, shi) = crate::stats::mean_ci95(&col(&|x| x.0));
        let bp = crate::stats::mean_ci95(&col(&|x| x.1)).0;
        let (tp, tlo, thi) = crate::stats::mean_ci95(&col(&|x| x.2));
        // realised / tight, seed by seed (a ratio of means would hide the spread)
        let (rt, rlo, rhi) = crate::stats::mean_ci95(&col(&|x| if x.2 > 0.0 { x.0 / x.2 } else { 0.0 }));
        crow.push(vec![
            f(d as f64 / 1000.0),
            c.to_string(),
            f(saving),
            f(bp.min(100.0)),
            f(if bp > 0.0 { saving / bp } else { 0.0 }),
            f(tp),
            f(rt),
            curve_seeds.len().to_string(),
            f(slo),
            f(shi),
            f(tlo),
            f(thi),
            f(rlo),
            f(rhi),
        ]);
        tight_pts.push((d as f64 / 1000.0, tp));
        real_pts.push((d as f64 / 1000.0, saving));
        bound_pts.push((d as f64 / 1000.0, bp.min(100.0)));
    }
    res.csv(
        "bound_curve.csv",
        &table(
            &[
                "max_delay_s", "deferred_cycles_per_day", "realised_saving_pct", "delay_bound_pct_capped_100", "realised_over_bound", "tight_bound_pct", "realised_over_tight_bound", "seeds",
                "realised_ci95_low", "realised_ci95_high", "tight_ci95_low", "tight_ci95_high", "realised_over_tight_ci95_low", "realised_over_tight_ci95_high",
            ],
            &crow,
        ),
    );
    let chart = crate::svg::Chart {
        title: "Realised saving vs delay bound (retail, Pix calibrated)".into(),
        x_label: "maximum settlement delay d (s, log)".into(),
        y_label: "% of UB".into(),
        series: vec![
            crate::svg::Series { label: "window bound (any mechanism)".into(), points: bound_pts, line: true },
            crate::svg::Series { label: "tight bound: UB - liquidity floor".into(), points: tight_pts, line: true },
            crate::svg::Series { label: "deferred netting every d (realised)".into(), points: real_pts, line: true },
        ],
        log_x: true,
    };
    res.csv("fig_bound.svg", &chart.render());
    // Per-participant view (specs/02, FR5): the aggregate can hide a participant for whom a
    // short deadline would matter. Distribution of (UB_i - floor_i) / UB_i over participants
    // that need liquidity, and the share of the total possible saving held by the top five.
    let pd: [u64; 4] = [10_000, 60_000, 900_000, 3_600_000];
    let pf = liquidity_floors(base, &pd);
    let mut prow = Vec::new();
    for (k, &d) in pd.iter().enumerate() {
        let mut rel: Vec<f64> = Vec::new();
        let mut abs: Vec<i64> = Vec::new();
        for i in 0..base.participants {
            let u = o.flow.ub(i as u32);
            if u > 0 {
                rel.push(100.0 * (u - pf[k][i]) as f64 / u as f64);
                abs.push(u - pf[k][i]);
            }
        }
        rel.sort_by(|a, b| a.partial_cmp(b).unwrap());
        abs.sort_unstable_by(|a, b| b.cmp(a));
        let total: i128 = abs.iter().map(|&v| v as i128).sum();
        let top5: i128 = abs.iter().take(5).map(|&v| v as i128).sum();
        let q = |x: f64| crate::stats::quantile(&rel, x);
        prow.push(vec![f(d as f64 / 1000.0), rel.len().to_string(), f(q(0.1)), f(q(0.5)), f(q(0.9)), f(q(1.0)), f(if total > 0 { top5 as f64 / total as f64 } else { 0.0 })]);
    }
    res.csv(
        "floor_by_participant.csv",
        &table(&["max_delay_s", "participants_with_ub", "tight_bound_pct_of_own_ub_p10", "p50", "p90", "max", "share_of_total_bound_in_top5_participants"], &prow),
    );
    res.set("scenario", scenario_json(base));
    res.set("cases", Json::Arr(js));
    res.set("note", "McMahon et al. (2024) report 14-30% savings from payment reordering with delays under 5 s in T2 (large-value); this experiment asks whether the delay bound separates the two regimes");
    eprintln!("{}", table(&header, &rows));
    res.finish();
    Ok(())
}
