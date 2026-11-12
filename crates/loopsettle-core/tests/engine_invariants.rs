//! Phase 5 invariants for `DeferredNet` and `AtomicMicroCycle`:
//! 1 conservation, 3 no negative balance (micro-cycle), 4 atomicity, 5 offset = minimum
//! payment of the cycle, 2 net positions equal gross settlement, 7 determinism, plus
//! delay bounds and the DeferredNet(1/day) = LB identity.

use loopsettle_core::*;
use proptest::prelude::*;
use std::collections::HashMap;

#[derive(Default)]
struct Recorder {
    ev: HashMap<u64, (TimeMs, Option<SettleKind>, Payment)>,
    dup: bool,
}
impl Sink for Recorder {
    fn settled(&mut self, p: &Payment, t: TimeMs, k: SettleKind) {
        self.dup |= self.ev.insert(p.id, (t, Some(k), *p)).is_some();
    }
    fn rejected(&mut self, p: &Payment, t: TimeMs) {
        self.dup |= self.ev.insert(p.id, (t, None, *p)).is_some();
    }
}

fn payments(n: u32, max_len: usize) -> impl Strategy<Value = Vec<Payment>> {
    prop::collection::vec((0..n, 1..n, 1i64..5_000, 0u64..700), 1..max_len).prop_map(move |v| {
        let mut t = 0;
        v.into_iter()
            .enumerate()
            .map(|(i, (a, off, amt, dt))| {
                t += dt;
                Payment {
                    id: i as u64,
                    t_arrival: t,
                    payer: a,
                    payee: (a + off) % n,
                    amount: amt,
                }
            })
            .collect()
    })
}

/// Drive one engine step by step, checking invariants 1 and 3 after every call.
fn drive_checked(
    eng: &mut dyn SettlementEngine,
    ps: &[Payment],
    ledger: &mut Ledger,
    rec: &mut Recorder,
    nonneg: bool,
) -> Result<(), TestCaseError> {
    let total = ledger.total();
    let check = |l: &Ledger| -> Result<(), TestCaseError> {
        prop_assert_eq!(l.total(), total);
        if nonneg {
            prop_assert!(l.balances().iter().all(|&b| b >= 0));
        }
        Ok(())
    };
    for p in ps {
        eng.advance_to(p.t_arrival, ledger, rec);
        check(ledger)?;
        eng.submit(*p, ledger, rec);
        check(ledger)?;
    }
    let end = ps.last().map(|p| p.t_arrival).unwrap_or(0);
    eng.advance_to(end, ledger, rec);
    eng.finish(end, ledger, rec);
    check(ledger)?;
    prop_assert_eq!(eng.queued(), 0);
    Ok(())
}

fn mode_strategy() -> impl Strategy<Value = QueueMode> {
    prop_oneof![Just(QueueMode::QueueAll), Just(QueueMode::QueueOnShortfall)]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Invariants 1, 3, 4, 5 and delay bounds under *constrained* liquidity.
    #[test]
    fn micro_cycle_invariants(
        ps in payments(6, 150),
        opening in prop::collection::vec(0i64..8_000, 6),
        window in 1u64..2_000,
        max_delay in 1u64..6_000,
        mode in mode_strategy(),
        max_len in 2usize..6,
        strict in any::<bool>(),
        search in prop::bool::weighted(0.8),
    ) {
        let mut cfg = MicroCycleConfig::new(window, max_delay, mode);
        cfg.max_cycle_len = max_len;
        cfg.search_cycles = search;
        cfg.discipline = if strict { QueueDiscipline::StrictFifoPerPayer } else { QueueDiscipline::BypassFifo };
        let mut eng = AtomicMicroCycle::new(cfg, 6).with_log();
        let mut ledger = Ledger::new(opening);
        let mut rec = Recorder::default();
        drive_checked(&mut eng, &ps, &mut ledger, &mut rec, true)?;

        // Every payment has exactly one outcome, never before arrival nor after the deadline.
        prop_assert!(!rec.dup);
        prop_assert_eq!(rec.ev.len(), ps.len());
        for p in &ps {
            let (t, _, _) = rec.ev[&p.id];
            prop_assert!(t >= p.t_arrival && t <= p.t_arrival + max_delay, "t={} arrival={}", t, p.t_arrival);
        }

        // Invariant 4: each settled cycle is a real directed cycle whose payments were ALL
        // settled, together, at the cycle's time, as Cycle settlements; and nothing else was.
        let log = eng.log.as_ref().unwrap();
        let mut in_cycles = 0;
        for (t, ids, offset) in log {
            prop_assert!(ids.len() >= 2 && ids.len() <= max_len);
            let legs: Vec<Payment> = ids.iter().map(|id| rec.ev[id].2).collect();
            for (i, p) in legs.iter().enumerate() {
                let (tt, k, _) = rec.ev[&p.id];
                prop_assert_eq!(tt, *t);
                prop_assert_eq!(k, Some(SettleKind::Cycle));
                prop_assert_eq!(p.payee, legs[(i + 1) % legs.len()].payer);
            }
            // Invariant 5: the offset is exactly the smallest payment in the cycle.
            prop_assert_eq!(*offset, legs.iter().map(|p| p.amount).min().unwrap());
            in_cycles += ids.len();
        }
        let cycle_events = rec.ev.values().filter(|e| e.1 == Some(SettleKind::Cycle)).count();
        prop_assert_eq!(cycle_events, in_cycles);
    }

    /// Invariant 2 at engine level: with unlimited liquidity every payment settles and the
    /// final balances equal those of gross settlement; same for DeferredNet.
    #[test]
    fn net_positions_preserved(ps in payments(5, 150), window in 1u64..3_000, max_delay in 1u64..8_000, mode in mode_strategy(), cpd in 1u32..48) {
        let mut g = Ledger::unlimited(5);
        let mut rg = Recorder::default();
        drive_checked(&mut GrossRealTime, &ps, &mut g, &mut rg, true)?;

        let mut l = Ledger::unlimited(5);
        let mut r = Recorder::default();
        drive_checked(&mut AtomicMicroCycle::new(MicroCycleConfig::new(window, max_delay, mode), 5), &ps, &mut l, &mut r, true)?;
        prop_assert!(r.ev.values().all(|e| e.1.is_some()), "nothing rejected with unlimited liquidity");
        prop_assert_eq!(l.balances(), g.balances());

        let mut d = Ledger::unlimited(5);
        let mut rd = Recorder::default();
        drive_checked(&mut DeferredNet::new(cpd, 5), &ps, &mut d, &mut rd, false)?;
        prop_assert_eq!(d.balances(), g.balances());
        prop_assert_eq!(rd.ev.len(), ps.len());
    }

    /// DeferredNet liquidity lies between LB and UB; with 1 cycle per day it equals LB.
    #[test]
    fn deferred_net_between_lb_and_ub(ps in payments(5, 150), cpd in 1u32..24) {
        let mut flow = FlowStats::new(5, false);
        ps.iter().for_each(|p| flow.observe(p));
        let mut d = Ledger::unlimited(5);
        let mut rd = Recorder::default();
        drive_checked(&mut DeferredNet::new(cpd, 5), &ps, &mut d, &mut rd, false)?;
        for p in 0..5u32 {
            prop_assert!(d.peak_drawdown(p) >= flow.lb(p) && d.peak_drawdown(p) <= flow.ub(p));
        }
        let mut one = Ledger::unlimited(5);
        drive_checked(&mut DeferredNet::new(1, 5), &ps, &mut one, &mut Recorder::default(), false)?;
        for p in 0..5u32 {
            prop_assert_eq!(one.peak_drawdown(p), flow.lb(p));
        }
    }

    /// Invariant 7 for the micro-cycle engine.
    #[test]
    fn micro_cycle_deterministic(ps in payments(6, 120), opening in prop::collection::vec(0i64..8_000, 6), mode in mode_strategy()) {
        let run = || {
            let mut eng = AtomicMicroCycle::new(MicroCycleConfig::new(500, 2_000, mode), 6).with_log();
            let mut l = Ledger::new(opening.clone());
            let mut rec = Recorder::default();
            drive_checked(&mut eng, &ps, &mut l, &mut rec, true).unwrap();
            let mut ev: Vec<_> = rec.ev.into_iter().map(|(k, v)| (k, v.0, v.1)).collect();
            ev.sort_by_key(|e| e.0);
            (l.balances().to_vec(), ev, eng.log.unwrap())
        };
        prop_assert_eq!(run(), run());
    }
}

#[test]
fn failed_batch_leaves_ledger_untouched() {
    // Invariant 4 at the ledger level.
    let mut l = Ledger::new(vec![0, 100, 0]);
    let mut scratch = vec![0; 3];
    // A pays B 50, B pays C 120, C pays A 50: B nets -70 and has 100 -> ok.
    assert!(l.try_batch(&[(0, 1, 50), (1, 2, 120), (2, 0, 50)], &mut scratch));
    assert_eq!(l.balances(), &[0, 30, 70]);
    // A pays B 10, B pays A 5: A nets -5 with balance 0 -> refused, nothing moves.
    assert!(!l.try_batch(&[(0, 1, 10), (1, 0, 5)], &mut scratch));
    assert_eq!(l.balances(), &[0, 30, 70]);
    assert!(scratch.iter().all(|&x| x == 0));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(600))]

    /// The event-driven gross retry is the literal definition, payment by payment: same
    /// outcome, same time and same kind for every payment, and the same final balances.
    /// Tight liquidity and long deadlines, so queues are deep and receipts release chains.
    #[test]
    fn event_driven_retry_equals_the_reference(
        ps in payments(6, 250),
        opening in prop::collection::vec(0i64..6_000, 6),
        window in 1u64..3_000,
        max_delay in 1u64..60_000,
        strict in any::<bool>(),
        search in any::<bool>(),
        max_len in 2usize..5,
    ) {
        let mut out = Vec::new();
        for reference in [false, true] {
            let mut cfg = MicroCycleConfig::new(window, max_delay, QueueMode::QueueOnShortfall);
            cfg.max_cycle_len = max_len;
            cfg.search_cycles = search;
            cfg.reference_retry = reference;
            cfg.discipline = if strict { QueueDiscipline::StrictFifoPerPayer } else { QueueDiscipline::BypassFifo };
            let mut eng = AtomicMicroCycle::new(cfg, 6);
            let mut ledger = Ledger::new(opening.clone());
            let mut rec = Recorder::default();
            drive_checked(&mut eng, &ps, &mut ledger, &mut rec, true)?;
            prop_assert!(!rec.dup);
            let mut ev: Vec<(u64, TimeMs, Option<SettleKind>)> = rec.ev.iter().map(|(id, e)| (*id, e.0, e.1)).collect();
            ev.sort_by_key(|e| e.0);
            out.push((ev, ledger.balances().to_vec(), eng.cycles_settled));
        }
        prop_assert_eq!(&out[0], &out[1]);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// Gridlock resolution: conservation, no negative balance, and one outcome per payment
    /// within its deadline. (No dominance over the plain queue is claimed: only invariants.)
    #[test]
    fn gridlock_invariants(
        ps in payments(6, 200),
        opening in prop::collection::vec(0i64..6_000, 6),
        window in 1u64..3_000,
        max_delay in 1u64..30_000,
        strict in any::<bool>(),
    ) {
        let mut cfg = MicroCycleConfig::new(window, max_delay, QueueMode::QueueOnShortfall);
        cfg.gridlock = true;
        cfg.discipline = if strict { QueueDiscipline::StrictFifoPerPayer } else { QueueDiscipline::BypassFifo };
        let mut eng = AtomicMicroCycle::new(cfg, 6);
        let mut ledger = Ledger::new(opening);
        let mut rec = Recorder::default();
        drive_checked(&mut eng, &ps, &mut ledger, &mut rec, true)?;
        prop_assert!(!rec.dup);
        prop_assert_eq!(rec.ev.len(), ps.len());
        for p in &ps {
            let (t, _, _) = rec.ev[&p.id];
            prop_assert!(t >= p.t_arrival && t <= p.t_arrival + max_delay);
        }
    }
}

#[test]
fn bilateral_pair_offsets_without_liquidity() {
    // A->B 100 at t=0 and B->A 100 at t=10, zero balances: gross rejects both; the
    // micro-cycle engine settles both atomically at the first window after t=10.
    let ps = [
        Payment {
            id: 0,
            t_arrival: 0,
            payer: 0,
            payee: 1,
            amount: 100,
        },
        Payment {
            id: 1,
            t_arrival: 10,
            payer: 1,
            payee: 0,
            amount: 100,
        },
    ];
    let mut runs = vec![
        EngineRun::new(Box::new(GrossRealTime), Ledger::new(vec![0, 0])),
        EngineRun::new(
            Box::new(AtomicMicroCycle::new(
                MicroCycleConfig::new(100, 1_000, QueueMode::QueueAll),
                2,
            )),
            Ledger::new(vec![0, 0]),
        ),
    ];
    let mut flow = FlowStats::new(2, false);
    drive(ps, &mut runs, &mut flow, 0);
    assert_eq!(runs[0].metrics.rejected_count, 2);
    assert_eq!(runs[1].metrics.rejected_count, 0);
    assert_eq!(runs[1].metrics.count_by_kind[1], 2);
    assert_eq!(runs[1].metrics.delay.max(), 100);
}
