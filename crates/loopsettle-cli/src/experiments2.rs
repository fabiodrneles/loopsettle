//! Phase 6 experiments: baseline, potential-vs-realizable, stress, finland-replication.

use crate::experiments::{engine_row, f64_list, flow_json, scenario_json, ENGINE_HEADER};
use crate::out::{f, table, Results};
use crate::svg::{bars, Chart, Series};
use loopsettle_core::json::Json;
use loopsettle_sim::toml_lite::Doc;
use loopsettle_sim::{run as run_engines, EngineSpec, Opening, OutageSpec, Population, RunOutput, ScenarioConfig};
use loopsettle_topology::{analyze, FlowGraph, HodgeConfig, TopologyReport};
use std::time::Instant;

fn specs_from(doc: &Doc, key: &str, default: &[&str]) -> Result<Vec<EngineSpec>, String> {
    let v = doc.str_array(key).unwrap_or_else(|| default.iter().map(|s| s.to_string()).collect());
    v.iter().map(|s| EngineSpec::parse(s)).collect()
}

pub(crate) fn topology(o: &RunOutput, doc: &Doc) -> (TopologyReport, f64) {
    let g = FlowGraph::gross_from(&o.flow);
    let cfg = HodgeConfig { max_triangles: doc.i64("topology.max_triangles").unwrap_or(500_000) as usize, ..Default::default() };
    let t = Instant::now();
    let r = analyze(&g, &cfg);
    (r, t.elapsed().as_secs_f64())
}

const LADDER_HEADER: &[&str] =
    &["graph", "edges", "total_flow_brl", "hodge_l1_cyclic", "hodge_l2_curl", "hodge_l2_cyclic", "cdfd_max", "cycle_netting", "hodge_triangles", "triangles_capped", "cdfd_bff"];

fn ladder_rows(t: &TopologyReport) -> Vec<Vec<String>> {
    [("gross", &t.gross), ("net", &t.net)]
        .iter()
        .map(|(n, l)| {
            vec![
                n.to_string(),
                l.edges.to_string(),
                f(l.total_flow as f64 / 100.0),
                f(l.hodge_l1),
                f(l.hodge_curl_l2),
                f(l.hodge_cyclic_l2),
                f(l.cdfd_max),
                f(l.cycles),
                l.hodge_triangles.to_string(),
                l.hodge_triangles_capped.to_string(),
                f(l.cdfd_bff),
            ]
        })
        .collect()
}

fn quoted(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "'"))
}

// ---------------------------------------------------------------------------------------
// baseline
// ---------------------------------------------------------------------------------------

pub fn baseline(path: &str, doc: &Doc, cfg: &ScenarioConfig) -> Result<(), String> {
    let specs = [EngineSpec::Gross, EngineSpec::Deferred { cycles_per_day: 1 }, EngineSpec::Deferred { cycles_per_day: 10 }];
    let o = run_engines(cfg, &specs, &Opening::Unlimited, true, None);
    let (ub, lb) = (o.ub(), o.lb());
    let (topo, topo_s) = topology(&o, doc);
    eprintln!("  topology: {} gross edges in {:.1} s", topo.gross.edges, topo_s);
    let mut res = Results::new("baseline", cfg.seed, path);

    res.csv("ladder.csv", &table(LADDER_HEADER, &ladder_rows(&topo)));
    {
        let shuffles = doc.i64("topology.null_shuffles").unwrap_or(20).max(0) as usize;
        let (nrows, njson) = topology_nulls(&FlowGraph::gross_from(&o.flow), &topo, shuffles);
        res.csv("nulls.csv", &table(&["measure", "observed", "null_mean", "null_sd", "observed_over_null", "null"], &nrows));
        res.set("nulls", njson);
    }
    let erows: Vec<Vec<String>> = o.results.iter().map(|r| engine_row(r, ub, lb)).collect();
    res.csv("engines.csv", &table(ENGINE_HEADER, &erows));

    // Per-participant table (also used by the Phase 8 qualitative Hodge check).
    let pop = Population::build(cfg);
    let prow: Vec<Vec<String>> = (0..cfg.participants)
        .map(|i| {
            let p = i as u32;
            vec![
                i.to_string(),
                pop.class[i].name().to_string(),
                f(pop.activity[i]),
                f(o.flow.out_value[i] as f64 / 100.0),
                f(o.flow.in_value[i] as f64 / 100.0),
                f(o.flow.net_position(p) as f64 / 100.0),
                f(o.flow.ub(p) as f64 / 100.0),
                f(o.flow.lb(p) as f64 / 100.0),
                f(topo.potential[i] / 100.0),
            ]
        })
        .collect();
    res.csv(
        "participants.csv",
        &table(&["participant", "class", "activity_share", "out_brl", "in_brl", "net_brl", "ub_brl", "lb_brl", "hodge_potential_brl"], &prow),
    );

    let fig = bars(
        "Circularity ladder (gross graph, share of gross flow)",
        "fraction of total flow (Hodge L1 can exceed 1)",
        &[
            ("Hodge L1 (diagnostic)".into(), topo.gross.hodge_l1),
            ("Potential offset (BCB style)".into(), o.flow.potential_offset_fraction()),
            ("CDFD maximum (static ceiling)".into(), topo.gross.cdfd_max),
            ("Cycle netting (greedy)".into(), topo.gross.cycles),
            ("Bilateral netting only".into(), topo.bilateral_share),
        ],
    );
    res.csv("fig_ladder.svg", &fig);

    // Sensitivity to the receiver bias (main bilateral-structure ASSUMPTION).
    let sigmas = f64_list(doc, "sensitivity.receiver_bias_sigma", &[0.0, 0.2, 0.4, 0.8]);
    let micro = EngineSpec::parse(doc.str("sensitivity.micro_engine").unwrap_or("micro:all:1000:10000"))?;
    let mut srows = Vec::new();
    let mut sjson = Vec::new();
    for &sg in &sigmas {
        let c = ScenarioConfig { receiver_bias_sigma: sg, ..cfg.clone() };
        let os = run_engines(&c, &[EngineSpec::Gross, micro.clone()], &Opening::Unlimited, true, None);
        let (ts, _) = topology(&os, doc);
        let (u, l) = (os.ub(), os.lb());
        srows.push(vec![
            f(sg),
            f(u as f64 / 100.0),
            f(l as f64 / 100.0),
            f(l as f64 / u.max(1) as f64),
            f(os.flow.potential_offset_fraction()),
            f(ts.gross.hodge_l1),
            f(ts.gross.cdfd_max),
            f(ts.gross.cycles),
            f(ts.net.cdfd_max),
            f(100.0 * os.results[1].saving_vs(u)),
            os.results[1].delay_q(0.95).to_string(),
        ]);
        sjson.push(Json::obj().set("receiver_bias_sigma", sg).set("flow", flow_json(&os)).set("topology", ts.to_json()).set("micro", os.results[1].to_json(u, l)));
    }
    let sh = [
        "receiver_bias_sigma",
        "ub_brl",
        "lb_brl",
        "lb_over_ub",
        "potential_offset",
        "hodge_l1_gross",
        "cdfd_max_gross",
        "cycles_gross",
        "cdfd_max_net",
        "micro_saving_pct",
        "micro_p95_ms",
    ];
    res.csv("sensitivity.csv", &table(&sh, &srows));

    res.set("scenario", scenario_json(cfg));
    res.set("flow", flow_json(&o));
    res.set("topology", topo.to_json());
    res.measure("topology_seconds", topo_s);
    res.set("engines", Json::Arr(o.results.iter().map(|r| r.to_json(ub, lb)).collect()));
    res.set("sensitivity", Json::Arr(sjson));
    eprintln!("{}", table(LADDER_HEADER, &ladder_rows(&topo)));
    eprintln!("{}", table(&sh, &srows));
    // Several seeds (specs/09, E1): the headline numbers with confidence intervals.
    {
        let names: Vec<String> = [
            "ub_brl", "lb_over_ub", "potential_offset_fraction", "saving_deferred_1x_pct", "saving_deferred_10x_pct", "gross_cdfd_max", "gross_cdfd_bff", "gross_cycle_netting",
            "net_cdfd_max", "net_cdfd_bff", "net_cycle_netting", "net_hodge_l2_curl", "bilateral_share",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        crate::seeds::run_and_write(&mut res, doc, cfg, &names, |c| {
            let o = run_engines(c, &specs, &Opening::Unlimited, true, None);
            let (t, _) = topology(&o, doc);
            let ub = o.ub();
            vec![
                ub as f64 / 100.0,
                o.lb() as f64 / ub.max(1) as f64,
                o.flow.potential_offset_fraction(),
                100.0 * o.results[1].saving_vs(ub),
                100.0 * o.results[2].saving_vs(ub),
                t.gross.cdfd_max,
                t.gross.cdfd_bff,
                t.gross.cycles,
                t.net.cdfd_max,
                t.net.cdfd_bff,
                t.net.cycles,
                t.net.hodge_curl_l2,
                t.bilateral_share,
            ]
        });
    }
    res.finish();
    Ok(())
}

// ---------------------------------------------------------------------------------------
// potential-vs-realizable
// ---------------------------------------------------------------------------------------

pub fn potential_vs_realizable(path: &str, doc: &Doc, cfg: &ScenarioConfig) -> Result<(), String> {
    let cpd = doc.i64_array("compare.deferred_cycles_per_day").unwrap_or_else(|| vec![1, 10, 96, 1440, 8640]);
    let micro = specs_from(doc, "compare.micro_engines", &["micro:all:1000:10000"])?;
    let mut specs = vec![EngineSpec::Gross];
    specs.extend(cpd.iter().map(|&c| EngineSpec::Deferred { cycles_per_day: c as u32 }));
    specs.extend(micro.iter().cloned());
    let o = run_engines(cfg, &specs, &Opening::Unlimited, true, None);
    let (ub, lb) = (o.ub(), o.lb());
    let (topo, _) = topology(&o, doc);
    let mut res = Results::new("potential-vs-realizable", cfg.seed, path);

    // Engine table with the maximum wait each design imposes.
    let mut header = vec!["max_wait_s"];
    header.extend_from_slice(ENGINE_HEADER);
    let mut rows = Vec::new();
    let (mut dpts, mut mpts) = (Vec::new(), Vec::new());
    for r in &o.results {
        let wait = match r.spec {
            EngineSpec::Gross => 0.0,
            EngineSpec::Deferred { cycles_per_day } => 86_400.0 / cycles_per_day as f64,
            EngineSpec::Micro(m) => m.max_delay_ms as f64 / 1000.0,
            EngineSpec::Hold { delay_ms, .. } => delay_ms as f64 / 1000.0,
        };
        let mut row = vec![f(wait)];
        row.extend(engine_row(r, ub, lb));
        rows.push(row);
        match r.spec {
            EngineSpec::Deferred { .. } => dpts.push((wait, 100.0 * r.saving_vs(ub))),
            EngineSpec::Micro(_) => mpts.push((wait, 100.0 * r.saving_vs(ub))),
            _ => {}
        }
    }
    res.csv("engines.csv", &table(&header, &rows));

    // The central table: potential (static, order ignored) vs realizable (in time).
    let best_micro = o
        .results
        .iter()
        .filter(|r| matches!(r.spec, EngineSpec::Micro(_)))
        .max_by(|a, b| a.saving_vs(ub).partial_cmp(&b.saving_vs(ub)).unwrap());
    let potential_liq = if ub == 0 { 0.0 } else { (ub - lb) as f64 / ub as f64 };
    let mut central: Vec<(String, f64, &str, &str)> = vec![
        ("potential_offset_fraction_bcb_style".into(), o.flow.potential_offset_fraction(), "share of gross outflows fundable by same-day inflows, order ignored", "measured, no time"),
        ("potential_liquidity_saving_end_of_day".into(), potential_liq, "(UB - LB) / UB: saving of one end-of-day net settlement", "measured"),
        ("static_ceiling_cdfd_max_gross".into(), topo.gross.cdfd_max, "max circulation respecting edge bounds / gross flow", "measured, aggregated, no time"),
        ("static_greedy_cycle_netting_gross".into(), topo.gross.cycles, "greedy cycle cancellation / gross flow", "measured, aggregated, no time"),
        ("hodge_l1_cyclic_gross".into(), topo.gross.hodge_l1, "diagnostic; can exceed 1", "measured, aggregated, no time"),
    ];
    if let Some(b) = best_micro {
        central.push((format!("realizable_liquidity_saving_{}", b.name), b.saving_vs(ub), "best atomic micro-cycle config: 1 - liquidity / UB", "measured in time"));
        let cyc_share = b.metrics.value_by_kind[1] as f64 / b.metrics.settled_value.max(1) as f64;
        central.push((format!("realizable_value_in_cycles_{}", b.name), cyc_share, "value settled inside atomic cycles / settled value", "measured in time"));
    }
    for r in &o.results {
        if let EngineSpec::Deferred { cycles_per_day } = r.spec {
            if cycles_per_day == 8640 || cycles_per_day == 96 {
                central.push((format!("realizable_liquidity_saving_{}", r.name), r.saving_vs(ub), "deferred net settlement at this frequency", "measured in time"));
            }
        }
    }
    let chead = ["measure", "value", "meaning", "status"];
    let crow: Vec<Vec<String>> = central.iter().map(|(k, v, m, s)| vec![k.clone(), f(*v), quoted(m), quoted(s)]).collect();
    res.csv("central.csv", &table(&chead, &crow));

    let chart = Chart {
        title: "Liquidity saving vs maximum settlement wait".into(),
        x_label: "maximum wait (s, log): deferred cycle interval or micro-cycle max_delay".into(),
        y_label: "liquidity saving vs gross UB (%)".into(),
        series: vec![
            Series { label: "Deferred net cycles".into(), points: dpts, line: true },
            Series { label: "Atomic micro-cycles".into(), points: mpts, line: false },
            Series { label: "Potential (end of day)".into(), points: vec![(86_400.0, 100.0 * potential_liq)], line: false },
        ],
        log_x: true,
    };
    res.csv("fig_potential_vs_realizable.svg", &chart.render());

    res.set("scenario", scenario_json(cfg));
    res.set("flow", flow_json(&o));
    res.set("topology", topo.to_json());
    res.set(
        "central",
        Json::Arr(central.iter().map(|(k, v, m, s)| Json::obj().set("measure", k.clone()).set("value", *v).set("meaning", *m).set("status", *s)).collect()),
    );
    res.set("engines", Json::Arr(o.results.iter().map(|r| r.to_json(ub, lb)).collect()));
    eprintln!("{}", table(&chead, &crow));
    eprintln!("{}", table(&header, &rows));
    // Several seeds (specs/09, E1): the saving of every design, with confidence intervals.
    {
        let mut names = vec!["potential_offset_fraction".to_string(), "eod_saving_pct".to_string()];
        names.extend(o.results[1..].iter().map(|r| format!("saving_pct_{}", r.name)));
        names.extend(o.results[1..].iter().map(|r| format!("p95_delay_ms_{}", r.name)));
        crate::seeds::run_and_write(&mut res, doc, cfg, &names, |c| {
            let o = run_engines(c, &specs, &Opening::Unlimited, false, None);
            let ub = o.ub();
            let mut v = vec![o.flow.potential_offset_fraction(), if ub == 0 { 0.0 } else { 100.0 * (ub - o.lb()) as f64 / ub as f64 }];
            v.extend(o.results[1..].iter().map(|r| 100.0 * r.saving_vs(ub)));
            v.extend(o.results[1..].iter().map(|r| r.delay_q(0.95) as f64));
            v
        });
    }
    res.finish();
    Ok(())
}

// ---------------------------------------------------------------------------------------
// stress
// ---------------------------------------------------------------------------------------

pub fn stress(path: &str, doc: &Doc, cfg: &ScenarioConfig) -> Result<(), String> {
    let who = doc.i64("stress.participant").unwrap_or(0) as u32;
    let o_spec = OutageSpec {
        participant: who,
        start_ms: doc.i64("stress.start_ms").unwrap_or(36_000_000) as u64,
        duration_ms: doc.i64("stress.duration_ms").unwrap_or(1_800_000) as u64,
    };
    let specs = specs_from(doc, "stress.engines", &["gross", "micro:all:1000:10000"])?;
    let base = run_engines(cfg, &specs, &Opening::Unlimited, false, None);
    let hit = run_engines(cfg, &specs, &Opening::Unlimited, false, Some(o_spec));
    let mut res = Results::new("stress", cfg.seed, path);
    let mut header = vec!["scenario", "ub_total_brl", "ub_of_offline_participant_brl"];
    header.extend_from_slice(ENGINE_HEADER);
    let mut rows = Vec::new();
    for (label, o) in [("baseline", &base), ("outage", &hit)] {
        for r in &o.results {
            let mut row = vec![label.to_string(), f(o.ub() as f64 / 100.0), f(o.flow.ub(who) as f64 / 100.0)];
            row.extend(engine_row(r, o.ub(), o.lb()));
            rows.push(row);
        }
    }
    // Constrained: everyone opens with fraction x its *baseline* UB; the outage then causes
    // rejections under gross settlement. Does queueing absorb them?
    let fr = doc.f64("stress.constrained_fraction_of_ub").unwrap_or(1.0);
    let opening: Vec<i64> = base.flow.ubs().iter().map(|&u| (u as f64 * fr).floor() as i64).collect();
    let cons = run_engines(cfg, &specs, &Opening::Given(opening), false, Some(o_spec));
    for r in &cons.results {
        let mut row = vec![format!("outage_constrained_{}xUB", f(fr)), f(cons.ub() as f64 / 100.0), f(cons.flow.ub(who) as f64 / 100.0)];
        row.extend(engine_row(r, base.ub(), base.lb()));
        rows.push(row);
    }
    res.csv("stress.csv", &table(&header, &rows));
    let (dc, dv, ds) = hit.outage_deferred.unwrap_or((0, 0, 0));
    res.set("scenario", scenario_json(cfg));
    res.set(
        "outage",
        Json::obj()
            .set("participant", who)
            .set("start_ms", o_spec.start_ms)
            .set("duration_ms", o_spec.duration_ms)
            .set("deferred_payments", dc)
            .set("deferred_value_brl", dv as f64 / 100.0)
            .set("mean_deferral_ms", if dc == 0 { 0.0 } else { ds as f64 / dc as f64 }),
    );
    res.set("baseline_flow", flow_json(&base));
    res.set("outage_flow", flow_json(&hit));
    res.set("baseline", Json::Arr(base.results.iter().map(|r| r.to_json(base.ub(), base.lb())).collect()));
    res.set("outage_unlimited", Json::Arr(hit.results.iter().map(|r| r.to_json(hit.ub(), hit.lb())).collect()));
    res.set("outage_constrained", Json::Arr(cons.results.iter().map(|r| r.to_json(base.ub(), base.lb())).collect()));
    eprintln!("{}", table(&header, &rows));

    // Redesigned stress (REVIEW 39): who goes offline and when matters more than "the largest
    // participant at 10:00". Cases = targets x start times; for each we measure the spillover
    // on everyone else's liquidity need and, with openings fixed at the baseline UB, how much
    // value each engine rejects.
    let (crows, cjson) = stress_cases(doc, cfg, &base, o_spec.duration_ms)?;
    res.csv("cases.csv", &table(CASE_HEADER, &crows));
    res.set("cases", Json::Arr(cjson));
    eprintln!("{}", table(CASE_HEADER, &crows));
    // Several seeds (specs/09, E1) for the redesigned cases: spillover on the others' liquidity
    // need and rejected value, per case. A case is named by its target and the position of its
    // start time in the list (the "before its own peak" time differs from seed to seed).
    {
        let probe = stress_cases(doc, cfg, &base, o_spec.duration_ms)?.0;
        let mut names = Vec::new();
        let case_key = |r: &Vec<String>, k: usize| format!("{}_t{}", r[0], k);
        let mut seen: Vec<(String, String)> = Vec::new();
        for r in &probe {
            if !seen.iter().any(|(t, st)| *t == r[0] && *st == r[3]) {
                seen.push((r[0].clone(), r[3].clone()));
            }
        }
        let pos = |r: &Vec<String>, seen: &Vec<(String, String)>| seen.iter().filter(|(t, _)| *t == r[0]).position(|(_, st)| *st == r[3]).unwrap_or(0);
        let mut last = String::new();
        for r in &probe {
            let key = case_key(r, pos(r, &seen));
            if key != last {
                names.push(format!("others_ub_net_change_pct_{key}"));
                last = key.clone();
            }
            names.push(format!("rejected_value_share_{key}_{}", r[14]));
        }
        let n_names = names.len();
        crate::seeds::run_and_write(&mut res, doc, cfg, &names, |c| {
            let b = run_engines(c, &specs, &Opening::Unlimited, false, None);
            let rows = match stress_cases(doc, c, &b, o_spec.duration_ms) {
                Ok((rows, _)) => rows,
                Err(_) => return vec![f64::NAN; n_names],
            };
            let mut seen_c: Vec<(String, String)> = Vec::new();
            for r in &rows {
                if !seen_c.iter().any(|(t, st)| *t == r[0] && *st == r[3]) {
                    seen_c.push((r[0].clone(), r[3].clone()));
                }
            }
            let mut v = Vec::with_capacity(n_names);
            let mut last = String::new();
            for r in &rows {
                let key = format!("{}_t{}", r[0], seen_c.iter().filter(|(t, _)| *t == r[0]).position(|(_, st)| *st == r[3]).unwrap_or(0));
                if key != last {
                    v.push(r[10].parse().unwrap_or(f64::NAN));
                    last = key;
                }
                v.push(r[15].parse().unwrap_or(f64::NAN));
            }
            v.resize(n_names, f64::NAN);
            v
        });
    }
    res.finish();
    Ok(())
}

const CASE_HEADER: &[&str] = &[
    "target",
    "participant",
    "participant_net_position_brl",
    "start",
    "deferred_payments",
    "deferred_value_brl",
    "ub_total_change_pct",
    "target_ub_change_brl",
    "others_ub_increase_brl",
    "others_ub_decrease_brl",
    "others_ub_net_change_pct",
    "others_with_higher_ub",
    "largest_other_increase_brl",
    "largest_other_increase_pct_of_its_ub",
    "engine",
    "rejected_value_share_at_baseline_ub",
    "rejected_payments_at_baseline_ub",
    "payers_with_rejections",
];

/// Time of the first minimum of `who`'s cumulative gross position (its debit peak), if any.
fn first_min_time(cfg: &ScenarioConfig, who: u32) -> Option<u64> {
    let (mut x, mut min, mut t) = (0i64, 0i64, None);
    for p in loopsettle_sim::PaymentStream::new(cfg) {
        if p.payer == who {
            x -= p.amount;
            if x < min {
                min = x;
                t = Some(p.t_arrival);
            }
        } else if p.payee == who {
            x += p.amount;
        }
    }
    t
}

fn hhmm(ms: u64) -> String {
    format!("{:02}:{:02}", ms / 3_600_000, (ms / 60_000) % 60)
}

fn stress_cases(doc: &Doc, cfg: &ScenarioConfig, base: &RunOutput, duration_ms: u64) -> Result<(Vec<Vec<String>>, Vec<Json>), String> {
    let n = cfg.participants;
    let targets = doc.str_array("stress.case_targets").unwrap_or_else(|| vec!["largest".into(), "top_net_payer".into(), "top_net_receiver".into()]);
    let starts: Vec<u64> = doc.i64_array("stress.case_start_ms").unwrap_or_else(|| vec![36_000_000, 64_800_000]).into_iter().map(|v| v as u64).collect();
    let before_peak = doc.str("stress.case_before_own_peak").map(|s| s == "true").unwrap_or(true);
    let specs = specs_from(doc, "stress.case_engines", &["gross", "micro:shortfall:1000:10000", "micro:all:1000:10000"])?;
    let net: Vec<i64> = (0..n as u32).map(|i| base.flow.net_position(i)).collect();
    let base_ubs = base.flow.ubs();
    let opening = Opening::Given(base_ubs.clone());
    let (mut rows, mut out) = (Vec::new(), Vec::new());
    for t in &targets {
        let who = match t.as_str() {
            "largest" => 0u32,
            "top_net_payer" => (0..n).min_by_key(|&i| net[i]).unwrap_or(0) as u32,
            "top_net_receiver" => (0..n).max_by_key(|&i| net[i]).unwrap_or(0) as u32,
            other => other.parse::<u32>().map_err(|_| format!("stress.case_targets: unknown target {other:?}"))?,
        };
        let mut when: Vec<(String, u64)> = starts.iter().map(|&s| (hhmm(s), s)).collect();
        if before_peak {
            if let Some(ts) = first_min_time(cfg, who) {
                let s = ts.saturating_sub(duration_ms);
                when.push((format!("before_own_peak({})", hhmm(s)), s));
            }
        }
        for (label, start) in when {
            let o_spec = OutageSpec { participant: who, start_ms: start, duration_ms };
            let hit = run_engines(cfg, &[EngineSpec::Gross], &Opening::Unlimited, false, Some(o_spec));
            let cons = run_engines(cfg, &specs, &opening, false, Some(o_spec));
            let (dc, dv, _) = hit.outage_deferred.unwrap_or((0, 0, 0));
            let hit_ubs = hit.flow.ubs();
            let (mut up, mut down, mut n_up, mut worst, mut worst_rel) = (0i128, 0i128, 0u32, 0i64, 0.0f64);
            let mut others_base = 0i128;
            for i in (0..n).filter(|&i| i as u32 != who) {
                let d = hit_ubs[i] - base_ubs[i];
                others_base += base_ubs[i] as i128;
                if d > 0 {
                    up += d as i128;
                    n_up += 1;
                    if d > worst {
                        worst = d;
                        worst_rel = if base_ubs[i] > 0 { d as f64 / base_ubs[i] as f64 } else { f64::INFINITY };
                    }
                } else {
                    down += (-d) as i128;
                }
            }
            let total_change = 100.0 * (hit.ub() - base.ub()) as f64 / base.ub().max(1) as f64;
            let others_net = 100.0 * (up - down) as f64 / others_base.max(1) as f64;
            let mut cj = Json::obj()
                .set("target", t.clone())
                .set("participant", who)
                .set("start", label.clone())
                .set("start_ms", start)
                .set("deferred_payments", dc)
                .set("deferred_value_brl", dv as f64 / 100.0)
                .set("ub_total_change_pct", total_change)
                .set("others_ub_increase_brl", up as f64 / 100.0)
                .set("others_ub_decrease_brl", down as f64 / 100.0)
                .set("others_with_higher_ub", n_up);
            let mut ej = Vec::new();
            for r in &cons.results {
                let m = &r.metrics;
                let total = (m.settled_value + m.rejected_value).max(1) as f64;
                let payers = m.rejected_value_by_payer.iter().filter(|&&v| v > 0).count();
                rows.push(vec![
                    t.clone(),
                    who.to_string(),
                    f(net[who as usize] as f64 / 100.0),
                    label.clone(),
                    dc.to_string(),
                    f(dv as f64 / 100.0),
                    f(total_change),
                    f((hit_ubs[who as usize] - base_ubs[who as usize]) as f64 / 100.0),
                    f(up as f64 / 100.0),
                    f(down as f64 / 100.0),
                    f(others_net),
                    n_up.to_string(),
                    f(worst as f64 / 100.0),
                    f(100.0 * worst_rel),
                    r.name.clone(),
                    f(m.rejected_value as f64 / total),
                    m.rejected_count.to_string(),
                    payers.to_string(),
                ]);
                ej.push(Json::obj().set("engine", r.name.clone()).set("rejected_value_share", m.rejected_value as f64 / total).set("rejected_payments", m.rejected_count).set("payers_with_rejections", payers));
            }
            cj.push("constrained_at_baseline_ub", Json::Arr(ej));
            out.push(cj);
        }
    }
    Ok((rows, out))
}

// ---------------------------------------------------------------------------------------
// finland-replication
// ---------------------------------------------------------------------------------------

pub fn finland(path: &str, doc: &Doc, cfg: &ScenarioConfig) -> Result<(), String> {
    let cpd = doc.i64_array("finland.cycles_per_day").unwrap_or_else(|| vec![1, 2, 4, 6, 10, 24, 48]);
    let seeds = doc.i64_array("finland.seeds").unwrap_or_else(|| vec![cfg.seed as i64]);
    let mut specs = vec![EngineSpec::Gross];
    specs.extend(cpd.iter().map(|&c| EngineSpec::Deferred { cycles_per_day: c as u32 }));
    let sigmas = f64_list(doc, "finland.receiver_bias_sigma", &[cfg.receiver_bias_sigma]);
    let mut rows = Vec::new();
    let mut series = Vec::new();
    let mut per_seed = Vec::new();
    for &sg in &sigmas {
        // ratios[k][s] = instant liquidity / deferred(k) liquidity for seed s
        let mut ratios = vec![Vec::new(); cpd.len()];
        for &sd in &seeds {
            let c = ScenarioConfig { seed: sd as u64, receiver_bias_sigma: sg, ..cfg.clone() };
            let o = run_engines(&c, &specs, &Opening::Unlimited, false, None);
            let inst = o.results[0].liquidity as f64;
            let mut js = Json::obj().set("receiver_bias_sigma", sg).set("seed", sd).set("instant_liquidity_brl", inst / 100.0).set("lb_brl", o.lb() as f64 / 100.0);
            for (k, r) in o.results[1..].iter().enumerate() {
                ratios[k].push(inst / (r.liquidity as f64).max(1.0));
                js.push(&format!("deferred_{}x_liquidity_brl", cpd[k]), r.liquidity as f64 / 100.0);
            }
            per_seed.push(js);
        }
        let mut pts = Vec::new();
        for (k, &c) in cpd.iter().enumerate() {
            let v = &ratios[k];
            let mean = v.iter().sum::<f64>() / v.len() as f64;
            let sd = (v.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (v.len().max(2) - 1) as f64).sqrt();
            let (lo, hi) = v.iter().fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
            rows.push(vec![f(sg), c.to_string(), f(mean), f(sd), f(lo), f(hi), v.len().to_string()]);
            pts.push((c as f64, mean));
        }
        series.push(Series { label: format!("receiver bias sigma {}", f(sg)), points: pts, line: true });
    }
    let header = ["receiver_bias_sigma", "cycles_per_day", "instant_over_deferred_liquidity_mean", "sd", "min", "max", "seeds"];
    let mut res = Results::new("finland-replication", cfg.seed, path);
    res.csv("ratios.csv", &table(&header, &rows));
    let chart = Chart {
        title: "Instant (gross) vs fixed-cycle liquidity need".into(),
        x_label: "clearing cycles per day (log scale)".into(),
        y_label: "liquidity instant / liquidity cycles (x)".into(),
        series,
        log_x: true,
    };
    res.csv("fig_finland.svg", &chart.render());
    res.set("scenario", scenario_json(cfg));
    res.set("seeds", Json::Arr(seeds.iter().map(|&s| Json::Int(s as i128)).collect()));
    res.set("per_seed", Json::Arr(per_seed));
    eprintln!("{}", table(&header, &rows));
    res.finish();
    Ok(())
}

// ---------------------------------------------------------------------------------------
// topology <edges.csv>  (Phase 8: external validation on any public transfer list)
// ---------------------------------------------------------------------------------------

/// Read `from,to,amount` rows (header required; ids are arbitrary strings, amount in
/// currency units), aggregate them and run the circularity ladder. Streaming read: only the
/// aggregated pair map is kept in memory.
pub fn topology_csv(path: &str, shuffles: usize) -> Result<(), String> {
    use std::collections::{BTreeMap, HashMap};
    use std::io::BufRead;
    let file = std::fs::File::open(path).map_err(|e| format!("{path}: {e}"))?;
    let mut ids: HashMap<String, u32> = HashMap::new();
    let mut pairs: BTreeMap<(u32, u32), i64> = BTreeMap::new();
    let (mut rows, mut skipped) = (0u64, 0u64);
    for (i, line) in std::io::BufReader::new(file).lines().enumerate() {
        let line = line.map_err(|e| e.to_string())?;
        if i == 0 || line.trim().is_empty() {
            continue;
        }
        let c: Vec<&str> = line.split(',').map(str::trim).collect();
        let amount: f64 = match c.get(2).and_then(|s| s.parse().ok()) {
            Some(a) if a > 0.0 => a,
            _ => {
                skipped += 1;
                continue;
            }
        };
        if c[0] == c[1] {
            skipped += 1;
            continue;
        }
        let next = ids.len() as u32;
        let a = *ids.entry(c[0].to_string()).or_insert(next);
        let next = ids.len() as u32;
        let b = *ids.entry(c[1].to_string()).or_insert(next);
        *pairs.entry((a, b)).or_insert(0) += (amount * 100.0).round() as i64;
        rows += 1;
    }
    let triples: Vec<(u32, u32, i64)> = pairs.into_iter().map(|((a, b), v)| (a, b, v)).collect();
    let g = FlowGraph::from_triples(ids.len(), &triples);
    let t = Instant::now();
    let r = analyze(&g, &HodgeConfig::default());
    let stem = std::path::Path::new(path).file_stem().and_then(|s| s.to_str()).unwrap_or("input");
    let name = format!("topology-{stem}");
    let mut res = Results::new(&name, 0, path);
    res.set_command(&format!("cargo run --release -p loopsettle-cli -- topology {path}"));
    res.set("rows", rows);
    res.set("rows_skipped", skipped);
    res.set("nodes", ids.len());
    res.set("topology", r.to_json());
    res.measure("seconds", t.elapsed().as_secs_f64());
    res.csv("ladder.csv", &table(LADDER_HEADER, &ladder_rows(&r)));
    eprintln!("{}", table(LADDER_HEADER, &ladder_rows(&r)));
    let (nrows, njson) = topology_nulls(&g, &r, shuffles);
    res.csv("nulls.csv", &table(&["measure", "observed", "null_mean", "null_sd", "observed_over_null", "null"], &nrows));
    res.set("nulls", njson);
    eprintln!("{}", table(&["measure", "observed", "null_mean", "null_sd", "observed_over_null", "null"], &nrows));
    res.finish();
    Ok(())
}

/// Null values for the circularity measures of a flow graph (specs/08, T4).
///
/// * Hodge, analytic: for random flows on a graph with `m` edges, `n` nodes and `k` connected
///   components, the expected share of the squared norm outside the gradient space is
///   `(m - n + k) / m`; it is compared with the observed L2 cyclic share of the net graph.
/// * Payee shuffle: the destination of every aggregated pair is permuted at random among the
///   pairs (payer and value kept; self-pairs dropped), `shuffles` times, with a fixed seed. The
///   measures of the shuffled graphs give the level expected from the payers' volumes and the
///   payees' popularity alone, without the actual bilateral structure.
fn topology_nulls(g: &FlowGraph, r: &TopologyReport, shuffles: usize) -> (Vec<Vec<String>>, loopsettle_core::json::Json) {
    use loopsettle_core::json::Json;
    let net = g.net();
    // connected components of the net graph (undirected)
    let mut parent: Vec<usize> = (0..net.n).collect();
    fn find(p: &mut [usize], x: usize) -> usize {
        let mut x = x;
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }
    let mut touched = vec![false; net.n];
    for e in &net.edges {
        let (a, b) = (find(&mut parent, e.from as usize), find(&mut parent, e.to as usize));
        parent[a] = b;
        touched[e.from as usize] = true;
        touched[e.to as usize] = true;
    }
    let nodes = touched.iter().filter(|&&t| t).count();
    let comps = (0..net.n).filter(|&v| touched[v] && find(&mut parent, v) == v).count();
    let m = net.edges.len();
    let hodge_null = if m == 0 { 0.0 } else { (m + comps - nodes) as f64 / m as f64 };
    let mut rows = vec![vec![
        "hodge_l2_cyclic_net".into(),
        f(r.net.hodge_cyclic_l2),
        f(hodge_null),
        f(0.0),
        f(if hodge_null > 0.0 { r.net.hodge_cyclic_l2 / hodge_null } else { 0.0 }),
        "analytic (m - n + k)/m".into(),
    ]];
    let mut js = Json::obj().set("hodge_analytic_null", hodge_null);
    if shuffles > 0 && !g.edges.is_empty() {
        let mut rng = loopsettle_core::rng::Rng::new(0x5EED_2026);
        let measures: [(&str, f64); 3] = [("cdfd_max_gross", r.gross.cdfd_max), ("cdfd_bff_gross", r.gross.cdfd_bff), ("cycle_netting_gross", r.gross.cycles)];
        let mut samples: Vec<Vec<f64>> = vec![Vec::new(); measures.len()];
        for _ in 0..shuffles {
            let mut to: Vec<u32> = g.edges.iter().map(|e| e.to).collect();
            for i in (1..to.len()).rev() {
                let j = rng.below(i as u64 + 1) as usize;
                to.swap(i, j);
            }
            let t: Vec<(u32, u32, i64)> = g.edges.iter().zip(&to).filter(|(e, &b)| e.from != b).map(|(e, &b)| (e.from, b, e.flow)).collect();
            let h = FlowGraph::from_triples(g.n, &t);
            let tot = h.total_flow().max(1) as f64;
            samples[0].push(loopsettle_topology::cdfd_max(&h).circular_total as f64 / tot);
            samples[1].push(loopsettle_topology::cdfd_bff(&h).circular_total / tot);
            samples[2].push(loopsettle_topology::cycle_netting(&h, false).cancelled_total as f64 / tot);
        }
        let mut nj = Vec::new();
        for ((name, obs), s) in measures.iter().zip(&samples) {
            let (mean, sd) = crate::stats::mean_sd(s);
            rows.push(vec![name.to_string(), f(*obs), f(mean), f(sd), f(if mean > 0.0 { obs / mean } else { 0.0 }), format!("payee shuffle x{shuffles}")]);
            nj.push(Json::obj().set("measure", *name).set("observed", *obs).set("null_mean", mean).set("null_sd", sd));
        }
        js = js.set("payee_shuffle", Json::Arr(nj)).set("shuffles", shuffles as u64);
    }
    (rows, js)
}
