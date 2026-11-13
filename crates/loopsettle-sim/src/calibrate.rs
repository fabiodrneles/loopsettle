//! Calibration against public Pix aggregates (`data/calibration/pix.toml`).
//!
//! Targets and how each is matched (see docs/PARAMETERS.md):
//! * participant sizes: key shares per participant (SOURCED, proxy for activity);
//! * mean ticket: monthly value / monthly count (SOURCED);
//! * tail = "large payments": it carries the B2B value share (SOURCED proxy), via its mean;
//! * shape: with `[value_percentiles]` in the file (SPI annual report), the body dispersion and
//!   the tail's count share are fitted to the public p50, p90 and p99 (CALIBRATED; D30);
//!   without them, or with `scenario.amounts_fit = "mean"`, the tail's count share is the B2B
//!   count share and the dispersion stays as configured (the method before D30);
//! * body mean = remaining value / remaining count (CALIBRATED via `body_median`);
//! * scale factor = real daily count / simulated daily count.

use crate::scenario::{AmountModel, Population, ScenarioConfig};
use crate::stream::PaymentStream;
use crate::toml_lite::{self, Doc};
use loopsettle_core::json::Json;
use loopsettle_core::rng::Rng;

#[derive(Clone, Debug)]
pub struct Targets {
    pub month: i64,
    pub daily_count: f64,
    pub daily_value_brl: f64,
    pub ticket_mean_brl: f64,
    pub b2b_count_share: f64,
    pub b2b_value_share: f64,
    pub shares: Vec<f64>,
    pub hhi: f64,
    pub top5: f64,
    pub top10: f64,
    pub top20: f64,
    pub n_with_keys: i64,
    /// public p50, p90, p99 of the value of one payment, R$ (`[value_percentiles]`)
    pub percentiles: Option<[f64; 3]>,
}

impl Targets {
    pub fn from_doc(d: &Doc) -> Result<Self, String> {
        let f = |k: &str| {
            d.f64(k)
                .ok_or_else(|| format!("missing {k} in calibration file"))
        };
        Ok(Targets {
            month: d.i64("source.month").ok_or("missing source.month")?,
            daily_count: f("transactions.daily_count")?,
            daily_value_brl: f("transactions.daily_value_brl")?,
            ticket_mean_brl: f("transactions.ticket_mean_brl")?,
            b2b_count_share: f("transactions.share_count_B2B")?,
            b2b_value_share: f("transactions.share_value_B2B")?,
            shares: d
                .f64_array("participants.shares")
                .ok_or("missing participants.shares")?,
            hhi: f("participants.hhi")?,
            top5: f("participants.top5_share")?,
            top10: f("participants.top10_share")?,
            top20: f("participants.top20_share")?,
            n_with_keys: d.i64("participants.n_with_keys").unwrap_or(0),
            percentiles: match (
                d.f64("value_percentiles.p50_brl"),
                d.f64("value_percentiles.p90_brl"),
                d.f64("value_percentiles.p99_brl"),
            ) {
                (Some(a), Some(b), Some(c)) => Some([a, b, c]),
                _ => None,
            },
        })
    }
    pub fn load(path: &str) -> Result<Self, String> {
        let src = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        Self::from_doc(&toml_lite::parse(&src)?)
    }
}

/// Mean of a Pareto(xm, alpha) truncated at m.
fn trunc_pareto_mean(xm: f64, a: f64, m: f64) -> f64 {
    AmountModel {
        body_median: 1.0,
        body_sigma: 0.0,
        tail_prob: 1.0,
        tail_xm: xm,
        tail_alpha: a,
        max_amount: m,
    }
    .mean()
}

/// Solve `xm` so that the truncated Pareto has the given mean (bisection; mean is
/// increasing in xm).
fn solve_xm(mean: f64, a: f64, m: f64) -> f64 {
    let (mut lo, mut hi) = (1e-6, m * 0.999);
    for _ in 0..200 {
        let mid = (lo * hi).sqrt();
        if trunc_pareto_mean(mid, a, m) < mean {
            lo = mid
        } else {
            hi = mid
        }
    }
    (lo * hi).sqrt()
}

/// Complementary error function (Numerical Recipes `erfcc`, relative error < 1.2e-7).
/// Written out so that the fit does not depend on a platform's special functions.
fn erfc(x: f64) -> f64 {
    let z = x.abs();
    let t = 1.0 / (1.0 + 0.5 * z);
    let r = t
        * (-z * z - 1.265_512_23
            + t * (1.000_023_68
                + t * (0.374_091_96
                    + t * (0.096_784_18
                        + t * (-0.186_288_06
                            + t * (0.278_868_07
                                + t * (-1.135_203_98
                                    + t * (1.488_515_87
                                        + t * (-0.822_152_23 + t * 0.170_872_77)))))))))
            .exp();
    if x >= 0.0 {
        r
    } else {
        2.0 - r
    }
}

/// CDF of the amount model (R$), ignoring the rounding to cents and the cap of the body.
fn amount_cdf(am: &AmountModel, x: f64) -> f64 {
    let body = if am.body_sigma > 0.0 {
        0.5 * erfc(-(x / am.body_median).ln() / (am.body_sigma * std::f64::consts::SQRT_2))
    } else if x >= am.body_median {
        1.0
    } else {
        0.0
    };
    let tail = if x < am.tail_xm {
        0.0
    } else {
        ((1.0 - (am.tail_xm / x).powf(am.tail_alpha))
            / (1.0 - (am.tail_xm / am.max_amount).powf(am.tail_alpha)))
        .min(1.0)
    };
    (1.0 - am.tail_prob) * body + am.tail_prob * tail
}

/// Quantile of the amount model (R$), by bisection on a log scale.
pub fn amount_quantile(am: &AmountModel, u: f64) -> f64 {
    let (mut lo, mut hi) = (0.01f64, am.max_amount);
    for _ in 0..100 {
        let mid = (lo * hi).sqrt();
        if amount_cdf(am, mid) < u {
            lo = mid
        } else {
            hi = mid
        }
    }
    (lo * hi).sqrt()
}

/// The amount model with tail count share `p` and body dispersion `sigma`, holding the mean
/// ticket and the share of value carried by the tail.
fn amounts_with(base: &AmountModel, t: &Targets, p: f64, sigma: f64, xm: f64) -> AmountModel {
    let body_mean = (1.0 - t.b2b_value_share) * t.ticket_mean_brl / (1.0 - p);
    AmountModel {
        body_sigma: sigma,
        body_median: body_mean / (sigma * sigma / 2.0).exp(),
        tail_prob: p,
        tail_xm: xm,
        ..base.clone()
    }
}

/// Grids of the percentile fit: tail count share 0.001..0.100 by 0.001, dispersion 0.80..3.00
/// by 0.02. Grid points are exact decimals, so the chosen pair is reproducible.
const FIT_P: (u32, u32) = (1, 100); // in thousandths
const FIT_SIGMA: (u32, u32) = (40, 150); // in fiftieths

/// Fit `(tail_prob, body_sigma)` to the public p50, p90, p99: least squares of the log ratios
/// over the grids above, every candidate holding the mean ticket and the tail's value share.
pub fn fit_percentiles(base: &AmountModel, t: &Targets, q: [f64; 3]) -> (f64, f64) {
    let mut best = (f64::INFINITY, 0.0, 0.0);
    for kp in FIT_P.0..=FIT_P.1 {
        let p = kp as f64 / 1000.0;
        let xm = solve_xm(
            t.b2b_value_share * t.ticket_mean_brl / p,
            base.tail_alpha,
            base.max_amount,
        );
        for ks in FIT_SIGMA.0..=FIT_SIGMA.1 {
            let sigma = ks as f64 / 50.0;
            let am = amounts_with(base, t, p, sigma, xm);
            let err: f64 = [0.5, 0.9, 0.99]
                .iter()
                .zip(q)
                .map(|(&u, target)| (amount_quantile(&am, u) / target).ln().powi(2))
                .sum();
            if err < best.0 {
                best = (err, p, sigma);
            }
        }
    }
    (best.1, best.2)
}

/// Apply targets to a scenario. Returns the scale factor.
pub fn apply(cfg: &mut ScenarioConfig, t: &Targets) -> f64 {
    cfg.shares = t.shares.clone();
    let (p, sigma) = match t.percentiles {
        Some(q) => fit_percentiles(&cfg.amounts, t, q),
        None => (t.b2b_count_share, cfg.amounts.body_sigma),
    };
    let xm = solve_xm(
        t.b2b_value_share * t.ticket_mean_brl / p,
        cfg.amounts.tail_alpha,
        cfg.amounts.max_amount,
    );
    cfg.amounts = amounts_with(&cfg.amounts, t, p, sigma, xm);
    cfg.scale_factor = t.daily_count / cfg.payments_per_day;
    cfg.scale_factor
}

#[derive(Clone, Debug)]
pub struct Row {
    pub name: &'static str,
    pub real: f64,
    pub sim: f64,
    pub note: &'static str,
}

impl Row {
    pub fn rel_err(&self) -> f64 {
        if self.real == 0.0 {
            0.0
        } else {
            (self.sim - self.real) / self.real
        }
    }
}

/// Simulate the calibrated scenario and compare every aggregate with its target.
pub fn report(cfg: &ScenarioConfig, t: &Targets) -> Vec<Row> {
    let pop = Population::build(cfg);
    let n = cfg.participants;
    let mut count_by_payer = vec![0u64; n];
    let (mut count, mut value) = (0u64, 0f64);
    for p in PaymentStream::with_population(cfg, &pop) {
        count += 1;
        value += p.amount as f64 / 100.0;
        count_by_payer[p.payer as usize] += 1;
    }
    let days = cfg.days as f64;
    let mut sh: Vec<f64> = count_by_payer
        .iter()
        .map(|&c| c as f64 / count as f64)
        .collect();
    sh.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let top = |k: usize| sh.iter().take(k).sum::<f64>();
    // Tail share of value: Monte Carlo of the amount model with a tag per draw.
    let mut rng = Rng::derive(cfg.seed, 0x7A11);
    let (mut tail_n, mut tail_v, mut all_v) = (0u64, 0f64, 0f64);
    let m = &cfg.amounts;
    let draws = 1_000_000;
    let mut values: Vec<f64> = Vec::with_capacity(draws); // 8 MB, for the percentiles
    for _ in 0..draws {
        let is_tail = rng.f64() < m.tail_prob;
        let am = AmountModel {
            tail_prob: if is_tail { 1.0 } else { 0.0 },
            ..m.clone()
        };
        let v = am.sample_cents(&mut rng) as f64;
        values.push(v / 100.0);
        all_v += v;
        if is_tail {
            tail_n += 1;
            tail_v += v;
        }
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let sim_q = |u: f64| values[((u * draws as f64) as usize).min(draws - 1)];
    // Renormalised real top-k over the participants we simulate.
    let kept: f64 = t.shares.iter().take(n).sum();
    let real_top = |k: usize| t.shares.iter().take(k.min(n)).sum::<f64>() / kept;
    let real_hhi: f64 = t
        .shares
        .iter()
        .take(n)
        .map(|s| (s / kept) * (s / kept))
        .sum();
    let mut rows = vec![
        Row {
            name: "daily_count (scaled)",
            real: t.daily_count,
            sim: count as f64 / days * cfg.scale_factor,
            note: "SOURCED; sim count x scale factor",
        },
        Row {
            name: "daily_value_brl (scaled)",
            real: t.daily_value_brl,
            sim: value / days * cfg.scale_factor,
            note: "SOURCED; sim value x scale factor",
        },
        Row {
            name: "ticket_mean_brl",
            real: t.ticket_mean_brl,
            sim: value / count as f64,
            note: "SOURCED target, matched via body/tail means",
        },
        Row {
            name: "tail_count_share",
            real: if t.percentiles.is_some() {
                cfg.amounts.tail_prob
            } else {
                t.b2b_count_share
            },
            sim: tail_n as f64 / draws as f64,
            note: if t.percentiles.is_some() {
                "CALIBRATED (percentile fit); real = fitted value"
            } else {
                "SOURCED proxy (B2B count share)"
            },
        },
        Row {
            name: "tail_value_share (B2B proxy)",
            real: t.b2b_value_share,
            sim: tail_v / all_v,
            note: "CALIBRATED via tail_xm",
        },
        Row {
            name: "top5_share",
            real: real_top(5),
            sim: top(5),
            note: "SOURCED (key shares, renormalised to simulated participants)",
        },
        Row {
            name: "top10_share",
            real: real_top(10),
            sim: top(10),
            note: "SOURCED (key shares)",
        },
        Row {
            name: "top20_share",
            real: real_top(20),
            sim: top(20),
            note: "SOURCED (key shares)",
        },
        Row {
            name: "hhi",
            real: real_hhi,
            sim: sh.iter().map(|s| s * s).sum(),
            note: "SOURCED (key shares)",
        },
    ];
    if let Some(q) = t.percentiles {
        for ((name, u), real) in [
            ("value_p50_brl", 0.5),
            ("value_p90_brl", 0.9),
            ("value_p99_brl", 0.99),
        ]
        .into_iter()
        .zip(q)
        {
            rows.push(Row {
                name,
                real,
                sim: sim_q(u),
                note: "SOURCED (SPI annual report 2025, Dec 2025); CALIBRATED via body_sigma and tail_prob",
            });
        }
    }
    rows
}

pub fn rows_to_csv(rows: &[Row]) -> String {
    let mut s = String::from("aggregate,real,simulated,relative_error,note\n");
    for r in rows {
        s.push_str(&format!(
            "{},{},{},{:.6},\"{}\"\n",
            r.name,
            r.real,
            r.sim,
            r.rel_err(),
            r.note
        ));
    }
    s
}

pub fn rows_to_json(rows: &[Row]) -> Json {
    Json::Arr(
        rows.iter()
            .map(|r| {
                Json::obj()
                    .set("aggregate", r.name)
                    .set("real", r.real)
                    .set("simulated", r.sim)
                    .set("relative_error", r.rel_err())
                    .set("note", r.note)
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solve_xm_hits_mean() {
        let xm = solve_xm(5_000.0, 1.3, 5_000_000.0);
        assert!((trunc_pareto_mean(xm, 1.3, 5_000_000.0) / 5_000.0 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn apply_matches_ticket_analytically() {
        let t = Targets {
            month: 202609,
            daily_count: 2.4e8,
            daily_value_brl: 1.08e11,
            ticket_mean_brl: 449.83,
            b2b_count_share: 0.043,
            b2b_value_share: 0.52,
            shares: vec![0.5, 0.3, 0.2],
            hhi: 0.38,
            top5: 1.0,
            top10: 1.0,
            top20: 1.0,
            n_with_keys: 3,
            percentiles: None,
        };
        let mut cfg = ScenarioConfig::default();
        apply(&mut cfg, &t);
        assert!((cfg.amounts.mean() / 449.83 - 1.0).abs() < 1e-9);
        assert!((cfg.scale_factor - 240.0).abs() < 1e-9);
    }

    #[test]
    fn erfc_matches_known_values() {
        assert!((erfc(0.0) - 1.0).abs() < 1e-7);
        assert!((erfc(1.0) - 0.157_299_207).abs() < 1e-7);
        assert!((erfc(-1.0) - 1.842_700_793).abs() < 1e-7);
        assert!((erfc(2.0) - 0.004_677_735).abs() < 1e-8);
    }

    /// The percentile fit (D30) on the public targets: the mean ticket and the tail's value share
    /// stay exact, and p50, p90, p99 land within 10% of the SPI annual report 2025.
    #[test]
    fn percentile_fit_keeps_the_mean_and_matches_the_shape() {
        let t = Targets::load(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../data/calibration/pix.toml"
        ))
        .unwrap();
        let q = t.percentiles.expect("pix.toml has [value_percentiles]");
        let mut cfg = ScenarioConfig::default();
        apply(&mut cfg, &t);
        let am = &cfg.amounts;
        assert!((am.mean() / t.ticket_mean_brl - 1.0).abs() < 1e-9);
        let tail_share =
            am.tail_prob * trunc_pareto_mean(am.tail_xm, am.tail_alpha, am.max_amount) / am.mean();
        assert!((tail_share / t.b2b_value_share - 1.0).abs() < 1e-9);
        for (u, target) in [0.5, 0.9, 0.99].into_iter().zip(q) {
            let got = amount_quantile(am, u);
            assert!(
                (got / target - 1.0).abs() < 0.10,
                "quantile {u}: {got} vs {target}"
            );
        }
    }
}
