//! # loopsettle-sim
//!
//! Synthetic, calibrated payment generator and streaming simulation driver.

pub mod scenario;
pub mod stream;
pub mod toml_lite;

pub use scenario::{AmountModel, Class, Population, ScenarioConfig};
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

    #[test]
    fn population_classes_and_shares() {
        let p = Population::build(&ScenarioConfig::default());
        assert_eq!(p.len(), 200);
        assert!((p.activity.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        assert_eq!(p.class[0], Class::BigBank);
        assert!(p.class.contains(&Class::Fintech) && p.class.contains(&Class::PaymentInstitution));
        assert!(p.top_share(5) > 0.3);
    }
}
