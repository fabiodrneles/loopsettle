//! # loopsettle-sim
//!
//! Synthetic, calibrated payment generator and streaming simulation driver.

pub mod bound;
pub mod calibrate;
pub mod csvfile;
pub mod runner;
pub mod scenario;
pub mod stream;
pub mod toml_lite;

pub use scenario::{AmountModel, Class, Population, ScenarioConfig};
pub use runner::{run, run_payments, EngineResult, EngineSpec, Opening, OutageSpec, RunOutput};
pub use stream::{Outage, PaymentStream};

#[cfg(test)]
mod tests {
    use super::*;
    use loopsettle_core::rng::Rng;

    fn small() -> ScenarioConfig {
        ScenarioConfig { participants: 20, payments_per_day: 20_000.0, n_big: 2, n_mid: 3, ..Default::default() }
    }

    #[test]
    fn same_seed_same_stream() {
        let a: Vec<_> = PaymentStream::new(&small()).collect();
        let b: Vec<_> = PaymentStream::new(&small()).collect();
        assert_eq!(a, b);
        let c: Vec<_> = PaymentStream::new(&ScenarioConfig { seed: 2, ..small() }).take(100).collect();
        assert_ne!(a[..100], c[..]);
    }

    #[test]
    fn stream_is_sorted_valid_and_count_close_to_expected() {
        let v: Vec<_> = PaymentStream::new(&small()).collect();
        assert!(v.windows(2).all(|w| w[0].t_arrival <= w[1].t_arrival));
        assert!(v.iter().all(|p| p.amount > 0 && p.payer != p.payee && p.t_arrival < 86_400_000));
        // Poisson: sd = sqrt(20000) ~ 141; allow 5 sd.
        assert!((v.len() as f64 - 20_000.0).abs() < 710.0, "count {}", v.len());
    }

    #[test]
    fn amount_mean_matches_analytic() {
        let m = AmountModel::default();
        let mut r = Rng::new(3);
        let n = 400_000;
        let s: f64 = (0..n).map(|_| m.sample_cents(&mut r) as f64 / 100.0).sum();
        let emp = s / n as f64;
        // Heavy tail: allow 10 %.
        assert!((emp / m.mean() - 1.0).abs() < 0.10, "empirical {emp} analytic {}", m.mean());
    }

    #[test]
    fn outage_defers_and_keeps_order() {
        let base: Vec<_> = PaymentStream::new(&small()).collect();
        let mut o = Outage::new(base.clone().into_iter(), 0, 36_000_000, 1_800_000);
        let out: Vec<_> = (&mut o).collect();
        assert_eq!(out.len(), base.len());
        assert!(out.windows(2).all(|w| w[0].t_arrival <= w[1].t_arrival));
        assert!(o.deferred_count > 0);
        assert!(out.iter().all(|p| !((p.payer == 0 || p.payee == 0) && p.t_arrival >= 36_000_000 && p.t_arrival < 37_800_000)));
    }

    /// W2: with full class profiles, payment institutions send mostly at night and receive
    /// mostly in the day, while each class keeps roughly its daily share of payments sent.
    #[test]
    fn class_profiles_shift_timing_but_keep_daily_volume() {
        let base = ScenarioConfig { participants: 40, payments_per_day: 200_000.0, n_big: 3, n_mid: 6, ..Default::default() };
        let prof = ScenarioConfig { profile_strength: 1.0, ..base.clone() };
        let pop = Population::build(&base);
        let pi = Class::PaymentInstitution;
        let tally = |cfg: &ScenarioConfig| {
            let (mut sent, mut sent_night, mut recv, mut recv_day, mut total) = (0u64, 0u64, 0u64, 0u64, 0u64);
            for p in PaymentStream::new(cfg) {
                let h = (p.t_arrival / 3_600_000) % 24;
                total += 1;
                if pop.class[p.payer as usize] == pi {
                    sent += 1;
                    sent_night += (h < 7) as u64;
                }
                if pop.class[p.payee as usize] == pi {
                    recv += 1;
                    recv_day += (9..21).contains(&h) as u64;
                }
            }
            (sent as f64 / total as f64, sent_night as f64 / sent as f64, recv_day as f64 / recv as f64)
        };
        let (s0, n0, d0) = tally(&base);
        let (s1, n1, d1) = tally(&prof);
        assert!(n1 > 3.0 * n0, "night share of PI outflows {n0} -> {n1}");
        assert!(d1 > d0, "daytime share of PI inflows {d0} -> {d1}");
        assert!((s1 / s0 - 1.0).abs() < 0.15, "PI daily share of payments sent {s0} -> {s1}");
    }

    #[test]
    fn population_classes_and_shares() {
        let p = Population::build(&ScenarioConfig::default());
        assert_eq!(p.len(), 200);
        assert!((p.activity.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        assert_eq!(p.class[0], Class::BigBank);
        assert!(p.class.contains(&Class::Fintech) && p.class.contains(&Class::PaymentInstitution));
        assert!(p.top_share(5) > 0.3);
    }
    /// Echo payments: about `echo_share` of the stream, same expected volume, still sorted,
    /// valid and deterministic; each echo repeats the amount of a payment its payer received
    /// shortly before.
    #[test]
    fn echo_payments_keep_volume_and_order() {
        let base = ScenarioConfig { seed: 11, participants: 30, payments_per_day: 60_000.0, n_big: 3, n_mid: 5, ..Default::default() };
        let plain: Vec<_> = PaymentStream::new(&base).collect();
        let cfg = ScenarioConfig { echo_share: 0.3, echo_delay_ms: 4_000.0, echo_return_prob: 0.5, ..base.clone() };
        let a: Vec<_> = PaymentStream::new(&cfg).collect();
        let b: Vec<_> = PaymentStream::new(&cfg).collect();
        assert_eq!(a, b, "deterministic");
        let ratio = a.len() as f64 / plain.len() as f64;
        assert!((ratio - 1.0).abs() < 0.03, "expected volume is unchanged: {} vs {}", a.len(), plain.len());
        let mut last = 0;
        for (k, p) in a.iter().enumerate() {
            assert!(p.t_arrival >= last && p.payer != p.payee && p.amount > 0 && p.id == k as u64);
            last = p.t_arrival;
        }
        // An echo = a payment whose payer received the same amount in the previous 60 s.
        let mut echoes = 0;
        for (k, p) in a.iter().enumerate() {
            let from = a[..k].iter().rev().take_while(|q| p.t_arrival - q.t_arrival <= 60_000);
            if from.into_iter().any(|q| q.payee == p.payer && q.amount == p.amount) {
                echoes += 1;
            }
        }
        let share = echoes as f64 / a.len() as f64;
        assert!((share - 0.3).abs() < 0.03, "echo share {share}");
    }
}

/// The examples in this crate's README are compiled and run as tests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
