# loopsettle-sim

Synthetic payment generator, scenario runner and liquidity bounds of
[LoopSettle](../../README.md).

The liquidity floor: the least liquidity *any* mechanism that settles every payment within `d`
of its arrival can need (docs/METHODOLOGY.md, section 9b). This block is compiled and run by
`cargo test`:

```rust
use loopsettle_core::Payment;
use loopsettle_sim::bound::liquidity_floors_from;

let pay = |id, t, a, b, v| Payment { id, t_arrival: t, payer: a, payee: b, amount: v };
// A pays 10 at t = 0 and 10 at t = 5; B pays A 8 at t = 7 (times in ms, amounts in cents).
let ps = vec![pay(0, 0, 0, 1, 10), pay(1, 5, 0, 1, 10), pay(2, 7, 1, 0, 8)];
let floors = liquidity_floors_from(2, || ps.iter().copied(), &[0, 1, 2]);
assert_eq!(floors[0][0], 20); // no delay: A needs everything it pays (the gross need)
assert_eq!(floors[1][0], 20); // 1 ms: the receipt of 8 comes too late to help
assert_eq!(floors[2][0], 12); // 2 ms: no mechanism can make A need less than 12
```

The same bound on a payment file, with only aggregates written out:
`cargo run --release -p loopsettle-cli -- bound <payments.csv> --report`. License: Apache-2.0.
