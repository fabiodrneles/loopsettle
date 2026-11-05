# Methodology

> 🇧🇷 [Versão em português](../METHODOLOGY.md)
>
> Each metric in plain language and in formal terms. Numbers: [`results/SUMMARY.en.md`](../../results/SUMMARY.en.md).

## 1. Liquidity needed: upper bound (UB) and lower bound (LB)

Let `x_i(t)` be participant `i`'s cumulative position up to time `t`:
`x_i(t) = inflows_i(t) − outflows_i(t)`, summing payments in arrival order.

- **UB (upper bound)**: `UB_i = max(0, −min_t x_i(t))`. It is the smallest opening balance
  that avoids any rejection in the `GrossRealTime` engine (real-time gross settlement, as in
  Pix). Proof: the balance at `t` is `opening_balance + x_i(t)`, and a payment is rejected only
  if the balance would become negative. The test `ub_is_tight` checks that with opening
  balance = UB nothing is rejected, and that the liquidity consumed in unlimited mode is
  exactly UB.
- **LB (lower bound)**: `LB_i = max(0, −x_i(T))`, the net debit position at the end of the
  period `T`. It is what a single multilateral netting at the end of the period would require.

Always `LB_i ≤ UB_i`, and `Σ_i x_i(T) = 0`.

### Worked example (test `hand_example_ub_lb`)

Participants A, B, C, unlimited balances:

| t | payment | x_A | x_B | x_C |
|---|---|---|---|---|
| 1 | A→B 100 | −100 | +100 | 0 |
| 2 | B→C 50 | −100 | +50 | +50 |
| 3 | C→A 70 | −30 | +50 | −20 |
| 4 | B→A 30 | 0 | +20 | −20 |

- UB = (100, 0, 20), total **120**.
- LB = (0, 0, 20), total **20**.
- With opening balances (100, 0, 20) nothing is rejected; with (99, 0, 20) A's first payment
  is rejected.

The difference UB − LB = 100 is liquidity that exists only because of the **time order** of
payments: a system able to net everything at the end of the period would save it.

## 2. Measuring an engine's liquidity

Each engine runs with an "unlimited" opening balance (`UNLIMITED = 2^50` cents). The liquidity
the engine needed is the **peak drawdown of the balance** (`opening_balance − lowest balance
reached`). For `GrossRealTime` this is exactly UB. See [`DECISIONS.en.md`](../../DECISIONS.en.md), D3.

## 3. Three lenses on circularity (the "ladder")

Let `F` be the aggregated flow graph (gross: `i→j` and `j→i` may coexist).

1. **Hodge (diagnostic).** On the net flow `f`, `f = B1ᵀφ + B2ψ + h`: gradient (flow running
   "downhill" from a potential φ per participant), curl (circulation around triangles) and
   harmonic (larger cycles). `L0 φ = B1 f` and `(B2ᵀB2) ψ = B2ᵀ(f − B1ᵀφ)`, both solved by
   conjugate gradient.
2. **Maximum CDFD (static ceiling).** The largest circulation `c` with `0 ≤ c_e ≤ F_e` and
   conservation at every node. It respects each edge's bound.
3. **Cycle netting (executable).** Cancels, cycle by cycle, the smallest flow in the cycle.

**Triangle (fixed test, invariant 8):** A→B 10, B→C 5, C→A 5.

| lens | per edge | total | fraction |
|---|---|---|---|
| Hodge (curl) | 6.67 | 20 | 100% (L1) |
| Maximum CDFD | 5 | 15 | 75% |
| Cycle netting | 5 | 15 | 75% |

Hodge projects `f` onto the cycle direction `(1,1,1)` and assigns the mean 20/3 = 6.67 to each
edge, **above** the flow of 5 on B→C and C→A: it does not respect edge bounds. That is why it is
a diagnostic (how much "circular shape" the flow has), not a nettable amount.

In general: CDFD ≥ greedy netting (property test) and, in the 200-node / 5,000-edge benchmark,
Hodge L1 ≫ CDFD > netting (see `CHECKPOINT.md`).

## 4. Settlement engines

All implement the `SettlementEngine` trait (`crates/loopsettle-core/src/engine.rs`) and are fed
in streaming, in arrival order, by `drive()`.

| Engine | In plain language | Formal rule |
|---|---|---|
| `GrossRealTime` | Like Pix: each payment settles immediately if funds suffice. | On arrival at `t`: if `payer_balance ≥ v`, transfer; otherwise reject. Delay 0. |
| `DeferredNet { N }` | Netting at fixed times (N times a day). | At instants `k·(24h/N)`, applies each participant's accumulated net position. Delay = time to the next cycle. |
| `AtomicMicroCycle { window, delay, mode }` | Every short window, look for "rings" of payments in the queue and settle each ring at once. | FIFO queue per pair; DFS finds a directed cycle (≤ 6 participants) among queue heads; the cycle settles atomically if every participant covers its net position in the cycle. At the deadline `arrival + delay`, the payment goes gross. Modes: `QueueAll` (everything waits) and `QueueOnShortfall` (only payments without funds wait). |

## 5. Metrics

- **Liquidity needed by an engine**: sum over participants of the peak balance drawdown with
  unlimited opening balance (D3). For gross settlement it is UB.
- **Liquidity saving**: `1 − engine_liquidity / UB`; and the share of the `UB − LB` gap
  recovered: `(UB − engine_liquidity) / (UB − LB)`.
- **Delays**: p50, p95, p99 of the time from arrival to settlement, from a log-linear histogram
  (relative error < 1.6%; the reported percentile is the bucket's lower bound). Compared with
  the Pix timing target (Manual de Tempos do Pix): 50% settled within 2.8 s (the measured delay
  is only what the engine adds; see limitations).
- **Potential capacity (BCB-study style)**: `Σ_i min(inflows_i, outflows_i) / gross volume`
  over the period, ignoring time order.
- **Potential liquidity saving**: `(UB − LB) / UB` — what a single end-of-day netting would save.
- **Ladder**: Hodge L1 (diagnostic) ≥? maximum CDFD (static ceiling) ≥ cycle netting (static
  executable) ≥ micro-cycles (executable over time).
- **Constrained liquidity**: opening balance = a fraction of each participant's UB; rejected
  value and delays are measured.

## 6. Potential vs realizable (the central comparison)

The BCB study measures what *could* be netted if same-day inflows and outflows offset each
other, regardless of order. LoopSettle measures, in the same scenario, what is actually saved
when settlement has to respect time: deferred netting at ever shorter intervals (from once a day
down to every 10 s) and atomic micro-cycles with a maximum delay of 2 to 10 s. The gap between
the two measures is the central result (`results/potential-vs-realizable/`).

## 7. Calibration

See [`PARAMETERS.md`](PARAMETERS.md): shares from Pix keys per ISPB, mean ticket, B2B share in
count and value, and the scale factor, all from Sep 2026 (BCB Olinda API).

## 8. Reproducibility

- Every simulation takes a `seed`; the PRNG is the project's own (xoshiro256**), stable across
  platforms, including wasm (invariant 7, tested).
- Every published number comes from `cargo run --release -p loopsettle-cli -- experiment <name>`,
  which writes `results/<name>/summary.json` with the seed, code version and command. No
  timestamp is written: re-running reproduces the files byte for byte.
- `make rebuild` redoes everything from scratch.
