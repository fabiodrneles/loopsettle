//! Funding layer (specs/05, M5): external top-ups, prefunding of closed windows, credit lines.

use loopsettle_core::*;
use proptest::prelude::*;

fn pay(id: u64, t: TimeMs, a: u32, b: u32, v: Cents) -> Payment {
    Payment {
        id,
        t_arrival: t,
        payer: a,
        payee: b,
        amount: v,
    }
}

fn run_one(
    engine: Box<dyn SettlementEngine>,
    ledger: Ledger,
    funding: Option<Box<dyn FundingPolicy>>,
    ps: &[Payment],
) -> EngineRun {
    let n = ledger.len();
    let mut run = EngineRun::new(engine, ledger);
    if let Some(f) = funding {
        run = run.with_funding(f);
    }
    let mut runs = vec![run];
    let mut flow = FlowStats::new(n, false);
    let end = ps.last().map(|p| p.t_arrival).unwrap_or(0);
    drive(ps.iter().copied(), &mut runs, &mut flow, end);
    runs.pop().unwrap()
}

/// Prefunding by hand. Closed window [100, 200). A is prefunded with 12.
/// Before the window liquidity is unlimited: A pays 1,000 at t=50 without trouble.
/// Inside: A pays 5 (t=110), 7 (t=120): balance 0. A pays 1 at t=130: rejected. A receives 4 at
/// t=140 and pays 3 at t=150: fine. After the window (t=250) A pays 1,000 again: fine.
#[test]
fn prefunding_by_hand() {
    let ps = [
        pay(0, 50, 0, 1, 1_000),
        pay(1, 110, 0, 1, 5),
        pay(2, 120, 0, 1, 7),
        pay(3, 130, 0, 1, 1),
        pay(4, 140, 1, 0, 4),
        pay(5, 150, 0, 1, 3),
        pay(6, 250, 0, 1, 1_000),
    ];
    let policy = PrefundClosedWindows::new(vec![(100, 200)], vec![12, 12]);
    let r = run_one(
        Box::new(GrossRealTime),
        Ledger::unlimited(2),
        Some(Box::new(policy)),
        &ps,
    );
    assert_eq!(r.metrics.rejected_count, 1);
    assert_eq!(r.metrics.rejected_value, 1);
    assert_eq!(r.metrics.rejected_value_by_payer, vec![1, 0]);
    // accounts plus reserves are conserved: 2 x UNLIMITED at the start
    assert_eq!(r.ledger.total_with_reserves(), 2 * UNLIMITED as i128);
}

/// A funding instant falls between two engine events: the micro-cycle window at t=1,000 runs
/// *before* the top-up at t=1,500, so the queued payment is not released by the window (no
/// funds yet) but by the next one, after the top-up.
#[test]
fn funding_instants_keep_their_place_in_time() {
    let ps = [pay(0, 10, 0, 1, 50), pay(1, 2_600, 1, 0, 1)];
    let policy = TopUpToTarget::new(vec![(1_500, 1_600)], 1_000, vec![50, 0]);
    let engine = AtomicMicroCycle::new(
        MicroCycleConfig::new(1_000, 10_000, QueueMode::QueueOnShortfall),
        2,
    );
    let r = run_one(
        Box::new(engine),
        Ledger::new(vec![0, 0]),
        Some(Box::new(policy)),
        &ps,
    );
    assert_eq!(r.metrics.rejected_count, 0);
    // payment 0 arrived at 10 and settled at the window of t=2,000: delay 1,990 ms
    assert_eq!(r.metrics.delay.max(), 1_990);
    assert_eq!(r.ledger.external(0), 50);
}

/// Intraday credit: the balance may go down to minus the limit and no further.
#[test]
fn credit_line_by_hand() {
    let ps = [
        pay(0, 1, 0, 1, 30),
        pay(1, 2, 0, 1, 30),
        pay(2, 3, 0, 1, 30),
    ];
    let ledger = Ledger::new(vec![10, 0]).with_credit_limits(vec![50, 0]);
    let r = run_one(Box::new(GrossRealTime), ledger, None, &ps);
    assert_eq!(
        r.metrics.settled_count, 2,
        "10 + 50 of credit covers two payments of 30"
    );
    assert_eq!(r.metrics.rejected_count, 1);
    assert_eq!(r.ledger.balance(0), -50);
    assert_eq!(r.ledger.max_credit_used(0), 50);
    assert_eq!(r.ledger.max_credit_used(1), 0);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// With top-ups, credit lines and any engine: accounts plus reserves are conserved, no
    /// balance goes below minus its credit limit, and every payment has one outcome.
    #[test]
    fn funding_invariants(
        raw in prop::collection::vec((0u32..5, 1u32..5, 1i64..4_000, 0u64..900), 1..150),
        opening in prop::collection::vec(0i64..5_000, 5),
        limits in prop::collection::vec(0i64..3_000, 5),
        target in prop::collection::vec(0i64..6_000, 5),
        period in 50u64..5_000,
        kind in 0usize..3,
    ) {
        let mut t = 0;
        let ps: Vec<Payment> = raw.iter().enumerate().map(|(i, &(a, off, v, dt))| { t += dt; pay(i as u64, t, a, (a + off) % 5, v) }).collect();
        let end = ps.last().unwrap().t_arrival;
        let engine: Box<dyn SettlementEngine> = match kind {
            0 => Box::new(GrossRealTime),
            1 => Box::new(AtomicMicroCycle::new(MicroCycleConfig::new(700, 4_000, QueueMode::QueueOnShortfall), 5)),
            _ => Box::new(AtomicMicroCycle::new(MicroCycleConfig::new(700, 4_000, QueueMode::QueueAll), 5)),
        };
        let ledger = Ledger::new(opening.clone()).with_credit_limits(limits.clone());
        let before = ledger.total_with_reserves();
        // open in the first half of the stream only
        let policy = TopUpToTarget::new(vec![(0, end / 2 + 1)], period, target);
        let r = run_one(engine, ledger, Some(Box::new(policy)), &ps);
        prop_assert_eq!(r.ledger.total_with_reserves(), before);
        for p in 0..5u32 {
            prop_assert!(r.ledger.balance(p) >= -limits[p as usize]);
            prop_assert!(r.ledger.max_credit_used(p) <= limits[p as usize]);
            prop_assert!(r.ledger.external(p) >= 0);
        }
        prop_assert_eq!(r.metrics.settled_count + r.metrics.rejected_count, ps.len() as u64);
    }
}
