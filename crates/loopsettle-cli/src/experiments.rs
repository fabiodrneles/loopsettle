//! Experiment implementations. Each one reads its `experiments/<name>.toml`.

use crate::out::{f, table, Results};
use loopsettle_sim::calibrate;
use loopsettle_sim::toml_lite::Doc;
use loopsettle_sim::ScenarioConfig;

pub const ALL: &[(&str, &str)] = &[("calibration", "real vs simulated public Pix aggregates")];

pub fn run(name: &str, path: &str, doc: &Doc, quick: bool) -> Result<(), String> {
    let mut cfg = ScenarioConfig::from_doc(doc)?;
    if quick {
        cfg.payments_per_day /= 10.0;
        cfg.scale_factor *= 10.0;
    }
    match name {
        "calibration" => calibration(path, doc, &cfg),
        _ => Err(format!("unknown experiment '{name}' (see `loopsettle list`)")),
    }
}

fn scenario_json(cfg: &ScenarioConfig) -> loopsettle_core::json::Json {
    use loopsettle_core::json::Json;
    Json::obj()
        .set("participants", cfg.participants)
        .set("days", cfg.days as u64)
        .set("payments_per_day", cfg.payments_per_day)
        .set("scale_factor", cfg.scale_factor)
        .set("body_median_brl", cfg.amounts.body_median)
        .set("body_sigma", cfg.amounts.body_sigma)
        .set("tail_prob", cfg.amounts.tail_prob)
        .set("tail_xm_brl", cfg.amounts.tail_xm)
        .set("tail_alpha", cfg.amounts.tail_alpha)
        .set("max_amount_brl", cfg.amounts.max_amount)
        .set("analytic_mean_ticket_brl", cfg.amounts.mean())
        .set("homophily", cfg.homophily)
        .set("receiver_bias_sigma", cfg.receiver_bias_sigma)
        .set("calibrated_shares", !cfg.shares.is_empty())
}

fn calibration(path: &str, doc: &Doc, cfg: &ScenarioConfig) -> Result<(), String> {
    let cal_path = doc.str("scenario.calibration").ok_or("calibration experiment needs scenario.calibration")?;
    let t = calibrate::Targets::load(cal_path)?;
    let rows = calibrate::report(cfg, &t);
    let mut r = Results::new("calibration", cfg.seed, path);
    r.csv("calibration.csv", &calibrate::rows_to_csv(&rows));
    r.set("reference_month", t.month);
    r.set("scenario", scenario_json(cfg));
    r.set("rows", calibrate::rows_to_json(&rows));
    let md: Vec<Vec<String>> = rows.iter().map(|x| vec![x.name.to_string(), f(x.real), f(x.sim), format!("{:+.2}%", 100.0 * x.rel_err())]).collect();
    eprintln!("{}", table(&["aggregate", "real", "simulated", "rel_err"], &md));
    r.finish();
    Ok(())
}
