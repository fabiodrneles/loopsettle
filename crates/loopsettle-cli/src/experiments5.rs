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
use loopsettle_sim::bound::{delay_bounds, window_stats};
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
    for &ppd in &scales {
        for &sigma in &sigmas {
            for &lam in &lambdas {
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
                let ws: Vec<(u64, u64)> = windows.iter().map(|w| (w.2, w.3)).collect();
                let st = window_stats(cfg.participants, || PaymentStream::new(&cfg), &ws, &delays);
                eprintln!("  {ppd:.0}/Thursday sigma {sigma} lambda {lam}: {:.1} s", t0.elapsed().as_secs_f64());
                for (k, (name, kind, w_start, w_end)) in windows.iter().enumerate() {
                    let need: i128 = st[k].need.iter().map(|&v| v as i128).sum();
                    let net: i128 = st[k].net.iter().map(|&v| v as i128).sum();
                    let pct = |x: i128| if need == 0 { 0.0 } else { 100.0 * x as f64 / need as f64 };
                    let b: Vec<f64> = st[k].bound.iter().map(|v| pct(v.iter().map(|&x| x as i128).sum())).collect();
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
                            .set("bound_15min_pct", b[1]),
                    );
                }
            }
        }
    }
    let header = [
        "payments_per_thursday", "receiver_bias_sigma", "profile_strength", "window", "kind", "hours", "prefunding_need_brl", "end_of_window_netting_need_brl", "netting_saving_pct",
        "bound_10s_pct", "bound_15min_pct", "bound_1h_pct",
    ];
    res.csv("windows.csv", &table(&header, &rows));
    res.set("scenario", scenario_json(base));
    res.set("source", "STR hours and day volumes: BCB, Relatório Anual do SPI 2024, pp. 12-15; hourly profile: Gráfico 12, p. 13");
    res.set("windows", Json::Arr(js));
    eprintln!("{}", table(&header, &rows));
    res.finish();
    Ok(())
}

fn b_hours(a: u64, b: u64) -> f64 {
    (b - a) as f64 / H as f64
}

pub fn regimes(path: &str, doc: &Doc, base: &ScenarioConfig) -> Result<(), String> {
    let mut res = Results::new("regimes", base.seed, path);
    let delays: Vec<u64> = vec![2_000, 5_000, 10_000];
    // ASSUMPTION: a stylised large-value system — 20 participants, payments with a median of
    // R$ 1 million and a heavy tail, a few thousand to a few hundred thousand payments a day.
    let wholesale = |ppd: f64| ScenarioConfig {
        participants: 20,
        n_big: 5,
        n_mid: 5,
        shares: Vec::new(),
        payments_per_day: ppd,
        amounts: AmountModel { body_median: 1.0e6, body_sigma: 2.0, tail_prob: 0.05, tail_xm: 2.0e7, tail_alpha: 1.3, max_amount: 5.0e9 },
        ..base.clone()
    };
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
        let pct = |b: &Vec<i64>| 100.0 * b.iter().map(|&v| v as i128).sum::<i128>() as f64 / ub.max(1) as f64;
        // average payment relative to the average participant's liquidity need
        let avg_payment = o.flow.gross_value as f64 / o.flow.count.max(1) as f64;
        let size_ratio = avg_payment / (ub as f64 / cfg.participants as f64);
        for (k, &d) in delays.iter().enumerate() {
            rows.push(vec![name.clone(), f(cfg.participants as f64), f(o.flow.count as f64), f(size_ratio), d.to_string(), f(100.0 * o.results[k + 1].saving_vs(ub)), f(pct(&bounds[k]))]);
        }
        js.push(Json::obj().set("case", name.as_str()).set("avg_payment_over_avg_ub", size_ratio).set("bound_5s_pct", pct(&bounds[1])).set("micro_5s_saving_pct", 100.0 * o.results[2].saving_vs(ub)));
        eprintln!("  {name}: done");
    }
    let header = ["case", "participants", "payments", "avg_payment_over_avg_participant_ub", "max_delay_ms", "micro_cycle_saving_pct", "delay_bound_pct"];
    res.csv("regimes.csv", &table(&header, &rows));
    // W5: realised saving of deferred netting with interval d vs the delay bound for the same d,
    // retail scenario, d from 10 s to one day.
    let ds: [(u32, u64); 6] = [(8640, 10_000), (1440, 60_000), (96, 900_000), (24, 3_600_000), (4, 21_600_000), (1, 86_400_000)];
    let mut dspecs = vec![EngineSpec::Gross];
    dspecs.extend(ds.iter().map(|&(c, _)| EngineSpec::Deferred { cycles_per_day: c }));
    let o = run_engines(base, &dspecs, &Opening::Unlimited, false, None);
    let ub = o.ub();
    let bnd = delay_bounds(base, &ds.iter().map(|&(_, d)| d).collect::<Vec<_>>());
    let mut crow = Vec::new();
    let (mut real_pts, mut bound_pts) = (Vec::new(), Vec::new());
    for (k, &(c, d)) in ds.iter().enumerate() {
        let saving = 100.0 * o.results[k + 1].saving_vs(ub);
        let bp = 100.0 * bnd[k].iter().map(|&v| v as i128).sum::<i128>() as f64 / ub.max(1) as f64;
        crow.push(vec![f(d as f64 / 1000.0), c.to_string(), f(saving), f(bp.min(100.0)), f(if bp > 0.0 { saving / bp } else { 0.0 })]);
        real_pts.push((d as f64 / 1000.0, saving));
        bound_pts.push((d as f64 / 1000.0, bp.min(100.0)));
    }
    res.csv("bound_curve.csv", &table(&["max_delay_s", "deferred_cycles_per_day", "realised_saving_pct", "delay_bound_pct_capped_100", "realised_over_bound"], &crow));
    let chart = crate::svg::Chart {
        title: "Realised saving vs delay bound (retail, Pix calibrated)".into(),
        x_label: "maximum settlement delay d (s, log)".into(),
        y_label: "% of UB".into(),
        series: vec![
            crate::svg::Series { label: "delay bound (any mechanism)".into(), points: bound_pts, line: true },
            crate::svg::Series { label: "deferred netting every d (realised)".into(), points: real_pts, line: true },
        ],
        log_x: true,
    };
    res.csv("fig_bound.svg", &chart.render());
    res.set("scenario", scenario_json(base));
    res.set("cases", Json::Arr(js));
    res.set("note", "McMahon et al. (2024) report 14-30% savings from payment reordering with delays under 5 s in T2 (large-value); this experiment asks whether the delay bound separates the two regimes");
    eprintln!("{}", table(&header, &rows));
    res.finish();
    Ok(())
}
