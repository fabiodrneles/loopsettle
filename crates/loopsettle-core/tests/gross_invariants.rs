//! Phase 1 invariants for `GrossRealTime`:
//! (1) conservation of total balance, (3) no negative balance, (7) determinism.
//! Plus the hand-worked UB/LB example from docs/METHODOLOGY.md.

use loopsettle_core::*;
use proptest::prelude::*;

fn pay(id: u64, t: u64, a: u32, b: u32, v: i64) -> Payment {
    Payment { id, t_arrival: t, payer: a, payee: b, amount: v }
}

/// Strategy: `n` participants and an arrival-sorted list of random payments.
fn payments(n: u32, max_len: usize) -> impl Strategy<Value = Vec<Payment>> {
    prop::collection::vec((0..n, 1..n, 1i64..10_000, 0u64..50), 0..max_len).prop_map(move |v| {
        let mut t = 0;
        v.into_iter()
            .enumerate()
            .map(|(i, (a, off, amt, dt))| {
                t += dt;
                pay(i as u64, t, a, (a + off) % n, amt)
            })
            .collect()
    })
}

fn run_gross(ps: &[Payment], opening: Vec<i64>) -> (EngineRun, FlowStats) {
    let n = opening.len();
    let mut runs = vec![EngineRun::new(Box::new(GrossRealTime), Ledger::new(opening))];
    let mut flow = FlowStats::new(n, true);
    drive(ps.iter().copied(), &mut runs, &mut flow, 0);
    (runs.pop().unwrap(), flow)
}

#[test]
fn hand_example_ub_lb() {
    // A=0, B=1, C=2. See docs/METHODOLOGY.md, "Exemplo à mão".
    let ps = [pay(0, 1, 0, 1, 100), pay(1, 2, 1, 2, 50), pay(2, 3, 2, 0, 70), pay(3, 4, 1, 0, 30)];
    let (run, flow) = run_gross(&ps, vec![UNLIMITED; 3]);
    assert_eq!(flow.ubs(), vec![100, 0, 20]);
    assert_eq!((0..3).map(|p| flow.lb(p)).collect::<Vec<_>>(), vec![0, 0, 20]);
    assert_eq!(flow.ub_total(), 120);
    assert_eq!(flow.lb_total(), 20);
    // Measured liquidity of the unconstrained gross engine equals UB.
    assert_eq!(run.ledger.peak_drawdowns(), vec![100, 0, 20]);
    // With exactly UB as opening balance nothing is rejected ...
    let (run, _) = run_gross(&ps, vec![100, 0, 20]);
    assert_eq!(run.metrics.rejected_count, 0);
    // ... and one cent less for A rejects A's first payment.
    let (run, _) = run_gross(&ps, vec![99, 0, 20]);
    assert!(run.metrics.rejected_count >= 1);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Invariants 1 and 3, checked after every single payment.
    #[test]
    fn conservation_and_no_negative(ps in payments(6, 200), opening in prop::collection::vec(0i64..20_000, 6)) {
        let mut ledger = Ledger::new(opening.clone());
        let total0 = ledger.total();
        let mut eng = GrossRealTime;
        let mut m = RunMetrics::new(6);
        for p in &ps {
            eng.submit(*p, &mut ledger, &mut m);
            prop_assert_eq!(ledger.total(), total0);
            prop_assert!(ledger.balances().iter().all(|&b| b >= 0));
        }
        prop_assert_eq!(m.settled_count + m.rejected_count, ps.len() as u64);
    }

    /// UB is exact: opening = UB gives zero rejections; unconstrained drawdown = UB.
    #[test]
    fn ub_is_tight(ps in payments(5, 150)) {
        let (unl, flow) = run_gross(&ps, vec![UNLIMITED; 5]);
        prop_assert_eq!(unl.ledger.peak_drawdowns(), flow.ubs());
        let (run, _) = run_gross(&ps, flow.ubs());
        prop_assert_eq!(run.metrics.rejected_count, 0);
        // LB <= UB and net positions sum to zero.
        let net: i64 = (0..5).map(|p| flow.net_position(p)).sum();
        prop_assert_eq!(net, 0);
        for p in 0..5 { prop_assert!(flow.lb(p) <= flow.ub(p)); }
    }

    /// Invariant 7: same input -> same output, bit for bit.
    #[test]
    fn deterministic(ps in payments(5, 150), opening in prop::collection::vec(0i64..20_000, 5)) {
        let (a, fa) = run_gross(&ps, opening.clone());
        let (b, fb) = run_gross(&ps, opening);
        prop_assert_eq!(a.ledger.balances(), b.ledger.balances());
        prop_assert_eq!(a.metrics.rejected_value, b.metrics.rejected_value);
        prop_assert_eq!(fa.ubs(), fb.ubs());
    }
}

#[test]
fn rng_determinism_across_runs() {
    // Invariant 7 at the source: the PRNG stream for a seed is fixed.
    let a: Vec<u64> = { let mut r = rng::Rng::new(2024); (0..64).map(|_| r.next_u64()).collect() };
    let b: Vec<u64> = { let mut r = rng::Rng::new(2024); (0..64).map(|_| r.next_u64()).collect() };
    assert_eq!(a, b);
}
