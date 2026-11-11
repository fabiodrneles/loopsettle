//! The sandwich of specs/03: liquidity floors <= offline optimum (LP) <= every engine.

mod common {
    pub mod optimum;
}
use common::optimum::lp_min_liquidity;
use loopsettle_core::rng::Rng;
use loopsettle_core::{Cents, Payment, TimeMs};
use loopsettle_sim::bound::liquidity_floors_from;
use loopsettle_sim::{run_payments, EngineSpec, Opening};

fn pay(id: u64, t: TimeMs, a: u32, b: u32, v: Cents) -> Payment {
    Payment { id, t_arrival: t, payer: a, payee: b, amount: v }
}

fn floors_total(n: usize, ps: &[Payment], d: TimeMs) -> f64 {
    liquidity_floors_from(n, || ps.iter().copied(), &[d])[0].iter().map(|&v| v as f64).sum()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-6 * (1.0 + a.abs().max(b.abs()))
}

/// Two offsetting payments 5 ms apart. With d = 5 both can settle together: no liquidity.
/// With d = 4 they cannot meet: the first payer needs the full 10.
#[test]
fn bilateral_pair_by_hand() {
    let ps = [pay(0, 0, 0, 1, 10), pay(1, 5, 1, 0, 10)];
    assert!(close(lp_min_liquidity(2, &ps, 5).unwrap(), 0.0));
    assert!(close(lp_min_liquidity(2, &ps, 4).unwrap(), 10.0));
    assert!(close(lp_min_liquidity(2, &ps, 0).unwrap(), 10.0));
}

/// The triangle of the topology tests, in time: A->B 10 at t=0, B->C 5 at t=1, C->A 5 at t=2.
/// Gross: A needs 10, B and C nothing (each receives before it pays): UB = 10.
/// d = 2: all three settle together at t=2; nets are A -5, B +5, C 0: optimum 5.
/// Floors: A paid 10 by t=0 and received 5 by t=2: 5; B and C: 0. Sum 5 = optimum.
#[test]
fn triangle_by_hand() {
    let ps = [pay(0, 0, 0, 1, 10), pay(1, 1, 1, 2, 5), pay(2, 2, 2, 0, 5)];
    assert!(close(lp_min_liquidity(3, &ps, 0).unwrap(), 10.0));
    assert!(close(lp_min_liquidity(3, &ps, 2).unwrap(), 5.0));
    assert!(close(floors_total(3, &ps, 2), 5.0));
    // d = 1: C->A cannot meet A->B; the 5 A receives comes too late: 10.
    assert!(close(lp_min_liquidity(3, &ps, 1).unwrap(), 10.0));
}

/// On random small instances: floors <= LP <= every engine with the same maximum delay;
/// LP(0) = UB; LP(very long) = LB.
#[test]
fn sandwich_on_random_instances() {
    let mut rng = Rng::new(20261111);
    let mut strict_gap = 0;
    let cases = 300;
    for case in 0..cases {
        let n = 3 + rng.below(4) as usize;
        let m = 5 + rng.below(36) as usize;
        let mut t = 0;
        let ps: Vec<Payment> = (0..m as u64)
            .map(|id| {
                t += rng.below(3_000);
                let a = rng.below(n as u64) as u32;
                let b = (a + 1 + rng.below(n as u64 - 1) as u32) % n as u32;
                pay(id, t, a, b, 1 + rng.below(5_000) as i64)
            })
            .collect();
        let t_end = ps.last().unwrap().t_arrival;
        let gross = run_payments(n, ps.iter().copied(), &[EngineSpec::Gross], &Opening::Unlimited, t_end);
        let (ub, lb) = (gross.ub() as f64, gross.lb() as f64);
        assert!(close(lp_min_liquidity(n, &ps, 0).unwrap(), ub), "case {case}: LP(0) must be UB");
        assert!(close(lp_min_liquidity(n, &ps, t_end + 1).unwrap(), lb), "case {case}: LP(long) must be LB");
        for d in [500u64, 2_000, 10_000] {
            let lp = lp_min_liquidity(n, &ps, d).unwrap();
            let fl = floors_total(n, &ps, d);
            assert!(fl <= lp + 1e-6 * (1.0 + lp), "case {case}, d {d}: floors {fl} > LP {lp}");
            assert!(lp <= ub + 1e-6 * (1.0 + ub));
            if fl < lp - 1e-6 * (1.0 + lp) {
                strict_gap += 1;
            }
            // engines whose maximum delay is d (deferred: 86,400,000 / d cycles a day)
            let specs = [
                EngineSpec::parse(&format!("micro:all:100:{d}")).unwrap(),
                EngineSpec::parse(&format!("micro:all:{d}:{d}")).unwrap(),
                EngineSpec::Deferred { cycles_per_day: (86_400_000 / d) as u32 },
            ];
            let o = run_payments(n, ps.iter().copied(), &specs, &Opening::Unlimited, t_end);
            for r in &o.results {
                let need = r.liquidity as f64;
                assert!(lp <= need + 1e-6 * (1.0 + need), "case {case}, d {d}: {} needs {need} < LP {lp}", r.name);
            }
        }
    }
    // The floors are not always attainable together: the LP is strictly above them sometimes.
    assert!(strict_gap > 0, "expected at least one instance with LP > sum of floors");
    eprintln!("LP strictly above the floors in {strict_gap} of {} (instance, d) pairs", cases * 3);
}
