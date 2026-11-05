//! Scenario configuration and the synthetic participant population.
//!
//! Every parameter is listed in `docs/PARAMETERS.md` with its status
//! (`SOURCED`, `CALIBRATED` or `ASSUMPTION`).

use loopsettle_core::rng::Rng;
use loopsettle_core::{TimeMs, MS_PER_DAY};

/// Institution class. Used for size ranking, homophily and receiver bias.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    BigBank,
    MidBank,
    Fintech,
    PaymentInstitution,
}

impl Class {
    pub fn name(self) -> &'static str {
        match self {
            Class::BigBank => "big_bank",
            Class::MidBank => "mid_bank",
            Class::Fintech => "fintech",
            Class::PaymentInstitution => "payment_institution",
        }
    }
}

/// Amount distribution: lognormal body mixed with a truncated Pareto tail (in reais).
#[derive(Clone, Debug)]
pub struct AmountModel {
    /// median of the lognormal body (R$)
    pub body_median: f64,
    /// sigma of log(amount) in the body
    pub body_sigma: f64,
    /// probability that an amount comes from the tail
    pub tail_prob: f64,
    /// Pareto scale (R$) and shape
    pub tail_xm: f64,
    pub tail_alpha: f64,
    /// truncation of the tail (R$)
    pub max_amount: f64,
}

impl Default for AmountModel {
    fn default() -> Self {
        AmountModel { body_median: 40.0, body_sigma: 1.4, tail_prob: 0.04, tail_xm: 1_000.0, tail_alpha: 1.3, max_amount: 5_000_000.0 }
    }
}

impl AmountModel {
    /// Analytic mean in reais (used by calibration).
    pub fn mean(&self) -> f64 {
        let body = self.body_median * (self.body_sigma * self.body_sigma / 2.0).exp();
        let (xm, a, m) = (self.tail_xm, self.tail_alpha, self.max_amount);
        // mean of a Pareto(xm, a) truncated at m
        let tail = if (a - 1.0).abs() < 1e-12 {
            xm * (m / xm).ln() / (1.0 - xm / m)
        } else {
            a * xm.powf(a) / (a - 1.0) * (xm.powf(1.0 - a) - m.powf(1.0 - a)) / (1.0 - (xm / m).powf(a))
        };
        (1.0 - self.tail_prob) * body + self.tail_prob * tail
    }

    /// Draw one amount, in cents (>= 1).
    pub fn sample_cents(&self, rng: &mut Rng) -> i64 {
        let reais = if rng.f64() < self.tail_prob {
            // inverse CDF of a Pareto truncated at max_amount
            let (xm, a, m) = (self.tail_xm, self.tail_alpha, self.max_amount);
            let u = rng.f64();
            let tail_mass = 1.0 - (xm / m).powf(a);
            xm / (1.0 - u * tail_mass).powf(1.0 / a)
        } else {
            (self.body_median.ln() + self.body_sigma * rng.normal()).exp().min(self.max_amount)
        };
        ((reais * 100.0).round() as i64).max(1)
    }
}

/// Default intraday profile (24 hourly weights). ASSUMPTION: shape inspired by public
/// descriptions of Pix usage (quiet nights, daytime plateau, evening peak); replace with a
/// sourced hourly profile when available.
pub const DEFAULT_HOURLY: [f64; 24] = [
    0.9, 0.5, 0.3, 0.25, 0.25, 0.4, 0.9, 1.8, 3.0, 4.0, 4.6, 5.0, 5.2, 4.8, 4.7, 4.8, 5.0, 5.3, 5.5, 5.2, 4.6, 3.8, 2.8, 1.8,
];

#[derive(Clone, Debug)]
pub struct ScenarioConfig {
    pub name: String,
    pub seed: u64,
    pub participants: usize,
    pub days: u32,
    /// expected number of simulated payments per day
    pub payments_per_day: f64,
    /// Zipf exponent of participant size when no calibrated shares are given
    pub zipf_s: f64,
    /// calibrated activity shares (overrides `zipf_s` if non-empty; length >= participants)
    pub shares: Vec<f64>,
    pub n_big: usize,
    pub n_mid: usize,
    /// fraction of the remaining participants that are fintechs (rest: payment institutions)
    pub fintech_frac: f64,
    /// exponent beta in payee weight size_j^beta
    pub payee_size_exp: f64,
    /// extra weight for payees in the payer's own class
    pub homophily: f64,
    /// sigma of the lognormal per-participant receiver bias
    pub receiver_bias_sigma: f64,
    /// receiver-bias multiplier of payment institutions (acquirers, P2B receivers)
    pub pi_receiver_mult: f64,
    pub hourly: Vec<f64>,
    pub amounts: AmountModel,
    /// how many real payments one simulated payment stands for (documentation only)
    pub scale_factor: f64,
}

impl Default for ScenarioConfig {
    fn default() -> Self {
        ScenarioConfig {
            name: "default".into(),
            seed: 1,
            participants: 200,
            days: 1,
            payments_per_day: 1_000_000.0,
            zipf_s: 1.1,
            shares: Vec::new(),
            n_big: 5,
            n_mid: 15,
            fintech_frac: 0.5,
            payee_size_exp: 1.0,
            homophily: 0.3,
            receiver_bias_sigma: 0.4,
            pi_receiver_mult: 1.3,
            hourly: DEFAULT_HOURLY.to_vec(),
            amounts: AmountModel::default(),
            scale_factor: 1.0,
        }
    }
}

impl ScenarioConfig {
    pub fn horizon_ms(&self) -> TimeMs {
        self.days as TimeMs * MS_PER_DAY
    }
}

/// The generated population.
#[derive(Clone, Debug)]
pub struct Population {
    pub class: Vec<Class>,
    /// activity share (sums to 1): probability of being the payer
    pub activity: Vec<f64>,
    /// receiver bias multiplier
    pub receiver_bias: Vec<f64>,
}

impl Population {
    pub fn build(cfg: &ScenarioConfig) -> Self {
        let n = cfg.participants;
        assert!(n >= 2, "need at least 2 participants");
        let mut raw: Vec<f64> = if cfg.shares.len() >= n {
            cfg.shares[..n].to_vec()
        } else {
            (0..n).map(|i| 1.0 / ((i + 1) as f64).powf(cfg.zipf_s)).collect()
        };
        // sort descending so rank -> class mapping works for calibrated shares too
        raw.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let total: f64 = raw.iter().sum();
        let activity: Vec<f64> = raw.iter().map(|x| x / total).collect();
        let rest = n.saturating_sub(cfg.n_big + cfg.n_mid);
        let n_fintech = (rest as f64 * cfg.fintech_frac).round() as usize;
        // Classes by rank, fintechs and PIs interleaved deterministically in the tail.
        let mut rng = Rng::derive(cfg.seed, 0xC1A55);
        let class: Vec<Class> = (0..n)
            .map(|i| {
                if i < cfg.n_big {
                    Class::BigBank
                } else if i < cfg.n_big + cfg.n_mid {
                    Class::MidBank
                } else {
                    let k = i - cfg.n_big - cfg.n_mid;
                    if (k * n_fintech) / rest.max(1) != ((k + 1) * n_fintech) / rest.max(1) { Class::Fintech } else { Class::PaymentInstitution }
                }
            })
            .collect();
        let receiver_bias: Vec<f64> = class
            .iter()
            .map(|c| {
                let b = (cfg.receiver_bias_sigma * rng.normal()).exp();
                if *c == Class::PaymentInstitution { b * cfg.pi_receiver_mult } else { b }
            })
            .collect();
        Population { class, activity, receiver_bias }
    }

    pub fn len(&self) -> usize {
        self.class.len()
    }
    pub fn is_empty(&self) -> bool {
        self.class.is_empty()
    }

    /// Herfindahl-Hirschman index of activity shares (0..1).
    pub fn hhi(&self) -> f64 {
        self.activity.iter().map(|s| s * s).sum()
    }
    /// Share of activity held by the top `k` participants.
    pub fn top_share(&self, k: usize) -> f64 {
        self.activity.iter().take(k).sum()
    }
}

impl ScenarioConfig {
    /// Read `[scenario]` keys from a parsed TOML document; missing keys keep defaults.
    /// If `scenario.calibration` names a file, the public-data calibration is applied
    /// (paths are relative to the current directory, i.e. the repository root).
    pub fn from_doc(d: &crate::toml_lite::Doc) -> Result<Self, String> {
        let mut c = ScenarioConfig::default();
        let g = |k: &str| d.get(&format!("scenario.{k}"));
        if let Some(v) = d.str("experiment.name") {
            c.name = v.to_string();
        }
        macro_rules! num {
            ($field:expr, $key:expr, $conv:ident, $t:ty) => {
                if let Some(v) = g($key).and_then(|v| v.$conv()) {
                    $field = v as $t;
                }
            };
        }
        num!(c.seed, "seed", as_i64, u64);
        num!(c.participants, "participants", as_i64, usize);
        num!(c.days, "days", as_i64, u32);
        num!(c.payments_per_day, "payments_per_day", as_f64, f64);
        num!(c.zipf_s, "zipf_s", as_f64, f64);
        num!(c.n_big, "n_big", as_i64, usize);
        num!(c.n_mid, "n_mid", as_i64, usize);
        num!(c.fintech_frac, "fintech_frac", as_f64, f64);
        num!(c.payee_size_exp, "payee_size_exp", as_f64, f64);
        num!(c.homophily, "homophily", as_f64, f64);
        num!(c.receiver_bias_sigma, "receiver_bias_sigma", as_f64, f64);
        num!(c.pi_receiver_mult, "pi_receiver_mult", as_f64, f64);
        num!(c.amounts.body_sigma, "body_sigma", as_f64, f64);
        num!(c.amounts.tail_alpha, "tail_alpha", as_f64, f64);
        num!(c.amounts.max_amount, "max_amount_brl", as_f64, f64);
        if let Some(h) = d.f64_array("scenario.hourly") {
            c.hourly = h;
        }
        if let Some(path) = d.str("scenario.calibration") {
            let t = crate::calibrate::Targets::load(path)?;
            crate::calibrate::apply(&mut c, &t);
        }
        // Explicit amount overrides win over calibration.
        num!(c.amounts.body_median, "body_median_brl", as_f64, f64);
        num!(c.amounts.tail_prob, "tail_prob", as_f64, f64);
        num!(c.amounts.tail_xm, "tail_xm_brl", as_f64, f64);
        Ok(c)
    }
}
