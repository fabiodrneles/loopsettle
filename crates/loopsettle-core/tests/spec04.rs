//! Regression tests for the engine and metric fixes of `specs/04-correcoes-motores.md`.
//! Each test failed before its fix.

use loopsettle_core::*;
use std::collections::HashMap;

#[derive(Default)]
struct Rec(HashMap<u64, (TimeMs, Option<SettleKind>)>);
impl Sink for Rec {
    fn settled(&mut self, p: &Payment, t: TimeMs, k: SettleKind) {
        self.0.insert(p.id, (t, Some(k)));
    }
    fn rejected(&mut self, p: &Payment, t: TimeMs) {
        self.0.insert(p.id, (t, None));
    }
}

fn pay(id: u64, t: TimeMs, a: u32, b: u32, v: Cents) -> Payment {
    Payment {
        id,
        t_arrival: t,
        payer: a,
        payee: b,
        amount: v,
    }
}

fn run(eng: &mut dyn SettlementEngine, ledger: &mut Ledger, ps: &[Payment]) -> Rec {
    let mut rec = Rec::default();
    for p in ps {
        eng.advance_to(p.t_arrival, ledger, &mut rec);
        eng.submit(*p, ledger, &mut rec);
    }
    let end = ps.last().unwrap().t_arrival;
    eng.advance_to(end, ledger, &mut rec);
    eng.finish(end, ledger, &mut rec);
    rec
}

fn shortfall(window: TimeMs, delay: TimeMs) -> AtomicMicroCycle {
    AtomicMicroCycle::new(
        MicroCycleConfig::new(window, delay, QueueMode::QueueOnShortfall),
        3,
    )
}

/// C1a. A (no funds) queues three payments to B on the same edge, then receives 30 from C.
/// All three must be released in the next window, not one per window.
#[test]
fn c1_same_edge_queue_is_released_in_one_window() {
    let ps = [
        pay(0, 0, 0, 1, 10),
        pay(1, 1, 0, 1, 10),
        pay(2, 2, 0, 1, 10),
        pay(3, 100, 2, 0, 30),
    ];
    let mut ledger = Ledger::new(vec![0, 0, 30]);
    let rec = run(&mut shortfall(1_000, 10_000), &mut ledger, &ps);
    assert_eq!(rec.0[&3], (100, Some(SettleKind::Gross)));
    for id in 0..3 {
        assert_eq!(rec.0[&id], (1_000, Some(SettleKind::Gross)), "payment {id}");
    }
}

/// C1b. B->C is queued before A->B; once A->B settles, B has the funds, so B->C must settle
/// in the same window (the retry runs to a fixed point).
#[test]
fn c1_chain_is_released_in_one_window() {
    let ps = [
        pay(0, 0, 1, 2, 10),
        pay(1, 1, 0, 1, 10),
        pay(2, 2, 2, 0, 10),
    ];
    let mut ledger = Ledger::new(vec![0, 0, 10]);
    let rec = run(&mut shortfall(1_000, 10_000), &mut ledger, &ps);
    assert_eq!(rec.0[&2], (2, Some(SettleKind::Gross)));
    assert_eq!(rec.0[&1], (1_000, Some(SettleKind::Gross)));
    assert_eq!(rec.0[&0], (1_000, Some(SettleKind::Gross)));
}

/// C2. Bypass (default): a new small payment settles on arrival although an older, larger one
/// of the same payer is queued. Strict FIFO per payer: it waits behind the older one.
#[test]
fn c2_strict_fifo_does_not_let_new_payments_jump_the_queue() {
    // A has 5. A->B 10 queues (shortfall). A->C 3 arrives later. Nothing else happens, so the
    // 10 expires and is rejected at t = 2000; then the 3 can go.
    let ps = [pay(0, 0, 0, 1, 10), pay(1, 10, 0, 2, 3)];
    let mut ledger = Ledger::new(vec![5, 0, 0]);
    let rec = run(&mut shortfall(500, 2_000), &mut ledger, &ps);
    assert_eq!(
        rec.0[&1],
        (10, Some(SettleKind::Gross)),
        "bypass settles the small payment on arrival"
    );

    let mut cfg = MicroCycleConfig::new(500, 2_000, QueueMode::QueueOnShortfall);
    cfg.discipline = QueueDiscipline::StrictFifoPerPayer;
    let mut ledger = Ledger::new(vec![5, 0, 0]);
    let rec = run(&mut AtomicMicroCycle::new(cfg, 3), &mut ledger, &ps);
    assert_eq!(
        rec.0[&0],
        (2_000, None),
        "the older payment expires unfunded"
    );
    let (t, kind) = rec.0[&1];
    assert_eq!(kind, Some(SettleKind::Gross));
    assert!(
        t >= 2_000,
        "strict FIFO: the newer payment cannot settle before the older one leaves (settled at {t})"
    );
}

/// C3. With a cycle count that does not divide the day, the last cycle of each day must still
/// fall exactly at the end of the day.
#[test]
fn c3_deferred_cycles_end_exactly_at_the_end_of_each_day() {
    for day in 0..3u64 {
        let t = (day + 1) * MS_PER_DAY - 1;
        let mut ledger = Ledger::unlimited(2);
        let rec = run(
            &mut DeferredNet::new(7, 2),
            &mut ledger,
            &[pay(0, t, 0, 1, 10)],
        );
        assert_eq!(rec.0[&0].0, (day + 1) * MS_PER_DAY, "day {day}");
    }
    // first cycle of the day: floor(1 * day / 7)
    let mut ledger = Ledger::unlimited(2);
    let rec = run(
        &mut DeferredNet::new(7, 2),
        &mut ledger,
        &[pay(0, 0, 0, 1, 10)],
    );
    assert_eq!(rec.0[&0].0, MS_PER_DAY / 7);
}

/// C4. The 2.8 s target is counted exactly: a delay of 2,810 ms is not "within 2,800 ms".
#[test]
fn c4_sla_share_is_exact() {
    let mut m = RunMetrics::new(2);
    for id in 0..100 {
        m.settled(&pay(id, 0, 0, 1, 10), 2_810, SettleKind::Gross);
    }
    assert_eq!(m.share_within_ms(2_800), Some(0.0));
    m.settled(&pay(100, 0, 0, 1, 10), 2_800, SettleKind::Gross);
    assert_eq!(m.share_within_ms(2_800), Some(1.0 / 101.0));
    assert_eq!(
        m.share_within_ms(1_234),
        None,
        "only configured thresholds are exact"
    );
    let (lo, hi) = m.delay.quantile_bounds(0.5);
    assert!(lo <= 2_810 && 2_810 <= hi && hi - lo <= 2_810 / 64 + 1);
}

fn gridlock(window: TimeMs, delay: TimeMs, n: usize) -> AtomicMicroCycle {
    let mut cfg = MicroCycleConfig::new(window, delay, QueueMode::QueueOnShortfall);
    cfg.gridlock = true;
    cfg.discipline = QueueDiscipline::StrictFifoPerPayer;
    AtomicMicroCycle::new(cfg, n)
}

/// Gridlock by hand. Nobody has funds. A->B 10, B->C 10, C->A 6 queue up: a chain that is not
/// a cycle of equal amounts. All three together would leave C at +4, B at 0 and A at -4, so the
/// last payment of A's queue (its only one) is dropped; then B is short, then C: nothing can
/// settle. Give A 4 and the whole batch settles at once in the next window.
#[test]
fn m3_gridlock_resolution_by_hand() {
    let ps = [pay(0, 0, 0, 1, 10), pay(1, 1, 1, 2, 10), pay(2, 2, 2, 0, 6)];
    let mut ledger = Ledger::new(vec![0, 0, 0]);
    let rec = run(&mut gridlock(1_000, 3_000, 3), &mut ledger, &ps);
    for id in 0..3 {
        assert_eq!(
            rec.0[&id].1, None,
            "payment {id} cannot settle without liquidity"
        );
    }
    let mut ledger = Ledger::new(vec![4, 0, 0]);
    let rec = run(&mut gridlock(1_000, 3_000, 3), &mut ledger, &ps);
    for id in 0..3 {
        assert_eq!(rec.0[&id], (1_000, Some(SettleKind::Cycle)), "payment {id}");
    }
    assert_eq!(ledger.balances(), &[0, 0, 4]);
}

/// The batch respects each payer's queue order: A's second payment (small, it would fit) is
/// not settled while its first (too large) is stuck; the cycle search with bypass would take it.
#[test]
fn m3_gridlock_keeps_the_payer_queue_order() {
    // A has 5: A->B 50 (stuck), then A->C 3. B and C have nothing queued.
    let ps = [pay(0, 0, 0, 1, 50), pay(1, 1, 0, 2, 3)];
    let mut ledger = Ledger::new(vec![5, 0, 0]);
    let rec = run(&mut gridlock(500, 2_000, 3), &mut ledger, &ps);
    assert_eq!(rec.0[&0], (2_000, None));
    assert!(
        rec.0[&1].0 >= 2_000,
        "the second payment waits for the first to leave the queue"
    );
}
