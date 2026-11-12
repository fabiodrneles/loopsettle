# loopsettle-core

Settlement engines, the cents ledger and streaming metrics of
[LoopSettle](../../README.md). No dependencies.

Run gross settlement and atomic micro-cycles on the same payments and compare the liquidity
each one needs (this block is compiled and run by `cargo test`):

```rust
use loopsettle_core::*;

let pay = |id, t, a, b, v| Payment { id, t_arrival: t, payer: a, payee: b, amount: v };
// A pays B 100 and, two seconds later, B pays A 100 (amounts in cents).
let payments = [pay(0, 0, 0, 1, 100), pay(1, 2_000, 1, 0, 100)];
let micro = MicroCycleConfig::new(1_000, 5_000, QueueMode::QueueAll);
let mut runs = vec![
    EngineRun::new(Box::new(GrossRealTime), Ledger::unlimited(2)),
    EngineRun::new(Box::new(AtomicMicroCycle::new(micro, 2)), Ledger::unlimited(2)),
];
let mut flow = FlowStats::new(2, false);
drive(payments, &mut runs, &mut flow, 10_000);

let need = |r: &EngineRun| r.ledger.peak_drawdowns().iter().sum::<i64>();
assert_eq!(need(&runs[0]), 100); // gross: A must hold 100
assert_eq!(need(&runs[1]), 0); // the two payments settle together as a cycle
assert_eq!(flow.ub_total(), 100);
```

To test a mechanism of your own, implement `SettlementEngine` (four methods); a complete example
is `crates/loopsettle-sim/examples/custom_engine.rs`. License: Apache-2.0.
