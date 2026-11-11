# Methodology

> [Versão em português](METHODOLOGY.pt-BR.md)
>
> Each metric in plain language and in formal terms. Numbers: [`results/SUMMARY.md`](../results/SUMMARY.md).

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
reached`). For `GrossRealTime` this is exactly UB. See [`DECISIONS.md`](../DECISIONS.md), D3.

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

## 9. The delay bound

**In plain language.** A mechanism that may hold each payment for at most *d* seconds can only
change *when* payments settle, by at most *d*. At the moment a participant reaches its deepest
debit position under gross settlement, the only payments a delay-bounded engine can still be
holding back are those that arrived in the last *d* seconds. So the most it can save is what
that participant paid in those *d* seconds — however clever the netting algorithm is.

**Formally.** With unlimited opening balances, let `x_i(t)` be participant `i`'s cumulative
position under `GrossRealTime` and `t*_i` the first time it reaches its minimum, so
`UB_i = −x_i(t*_i)`. Take any engine in which every payment arriving at `a` settles in
`[a, a + d]`, and let `y_i(t)` be `i`'s cumulative position in that engine. At `t*`, every
payment still unsettled arrived in `[t* − d, t*]`, so

```text
y_i(t*) ≤ x_i(t*) + O_i[t* − d, t*]          (O_i = value paid by i in the window)
L_i = max(0, −min_t y_i(t)) ≥ −y_i(t*)   ⇒   UB_i − L_i ≤ O_i[t*_i − d, t*_i].
```

The bound covers atomic micro-cycles with `max_delay_ms = d`, deferred netting every `d`, and
any optimal gridlock solver with the same deadline. It is computed in two streaming passes
(`loopsettle_sim::bound::delay_bounds`) and tested participant by participant against the
engines (`engines_never_beat_the_delay_bound`). It is a ceiling, not an estimate: in the
calibrated scenario the realised 10 s saving is about 1.5% of it (about 28% of the tight form of §9b; netting every 10 s realises about 46%)
(`results/robustness/seeds_summary.csv`).

**9b. The liquidity floor (tight form of the bound).** The argument above uses one instant,
`t*`. Using every instant gives a stronger statement. Let `CO_i(s)` be the value `i` has paid by
time `s` and `CI_i(s)` the value it has been paid by `s` (arrival times, inclusive). In any
engine with maximum delay `d`, at time `s + d` every payment that arrived by `s` has settled, and
no incoming payment settles before it arrives, so

```text
−y_i(s + d) ≥ CO_i(s) − CI_i(s + d)      for every s
floor_i(d) = max(0, max_s [CO_i(s) − CI_i(s + d)])  ≤  L_i  ≤  UB_i = floor_i(0).
```

`UB_i − floor_i(d)` is never larger than the window bound (take `s = t* − d`), and `floor_i` is
attained for one participant alone by "hold my payments for exactly `d`, receive everything at
once", so it is the best bound that uses only `i`'s own flows (test `a_single_holder_needs_exactly_its_floor`, with the `hold:<d>:<i>` engine: equality to the cent). It is computed in streaming with
one lagging and one leading pass (`loopsettle_sim::bound::liquidity_floors`), tested against the
engines in the same test, and reported by `loopsettle bound` (`liquidity_floor`, `tight_bound`).
In the calibrated retail scenario (`results/regimes/bound_curve.csv`, one seed) the tight bound
for 10 s is 0.054% of UB against 0.76% for the window form, and netting every 10 s realises 51%
of it; for a one-day delay it equals UB − LB exactly. In the stylised large-value system the
window bound is large only because it counts the payment that makes the peak; the tight bound
there is ≤ 0.004%.

**9c. The offline optimum and the sandwich.** The floor is a bound for one participant at a time;
the sum over participants need not be attainable, because one participant's delayed payment is
another's delayed receipt. The exact question is a linear programme: given all payments in
advance and a deadline `d`, choose the share of each payment settled at each instant of
`[arrival, arrival + d]` so that `Σ_i L_i` is smallest, with `L_i ≥ −y_i(τ)` at every instant and
simultaneous settlements counted by their net effect. Its value sits between the two:

```text
Σ_i floor_i(d)  ≤  LP(d)  ≤  optimum with whole payments  ≤  L(any engine with deadline d)
```

`LP(0) = UB` and `LP(∞) = LB`. The programme is solved only as a test oracle
(`crates/loopsettle-sim/tests/optimum.rs`: two hand examples and 300 random instances) and in the
`optimality_gap` example. On dense small systems (20 participants, about 1,500 payments in an
hour, 20 seeds, deadlines of 2, 10 and 60 s) the optimum equals the sum of the floors in 116 of
120 instances and is within 0.3% of it in the other four, while the best engine realises 44–61%
of it (`results/optimality-gap/gap.csv`). So the slack between engines and the bound is in the
engines, not in the bound.

**Hodge null value.** For random flows on a connected graph with `m` edges and `n` nodes, the
expected share of the squared norm outside the gradient space is `(m − n + 1)/m`. On the net
graph of the baseline (`m = 6620`, `n = 200`) that is 0.970, against an observed L2 curl share
of 0.977: on dense graphs the Hodge curl share mostly measures graph density.

## 10. Funding windows

The Conta PI can be topped up from reserves only while the STR is open (6:30–18:30 on business
days, plus a 30-minute top-up window; BCB, Relatório Anual do SPI 2024, p. 15). Inside that window
an intraday peak can be covered as it happens; outside it — each night and the whole weekend — a
participant must hold, at the window's start, everything it will need until the window ends.
For a closed window `[a, b)`, with `x_i` the cumulative position:

```text
need_i = max(0, x_i(a) − min_{a≤t<b} x_i(t))      prefunding at the start of the window
net_i  = max(0, x_i(a) − x_i(b))                  what one netting at the end of the window needs
```

The delay bound of section 9 applies inside the window with `t*` the window's minimum. Computed
by `loopsettle_sim::bound::window_stats`; experiment `funding-windows` (Thursday to Monday, sourced
hourly profile and day volumes).

**The liquidity floor inside a window.** The tight form of section 9b carries over with one extra
term. An engine's need in the window is `y_i(a) − min_{a≤t<b} y_i(t)`. Receipts of the last `d`
before the window may still be unsettled when it opens, so `y_i(a) ≥ x_i(a) − I_i[a − d, a)`, and
the floor argument gives `−y_i(t) ≥ CO_i(t − d) − CI_i(t)`. Together:

```text
need_i ≥ max_{s ≥ a, s + d < b} [(CO_i(s) − CO_i(a)) − (CI_i(s + d) − CI_i(a))] − I_i[a − d, a)
```

floored at zero; with `d = 0` it is the gross need above. Computed by
`loopsettle_sim::bound::window_floors` and tested against the engines by rebuilding each engine's
position from its settlement events (`engines_never_need_less_than_the_window_floor`).

## 11. Constrained liquidity

Sections 2 to 10 ask how much liquidity a design *needs* when balances are unlimited. An operator
asks the reverse: with the liquidity participants hold, how much is rejected or delayed? The
`liquidity-frontier` experiment fixes the opening balances and runs every design on the same
payments.

- **Balances.** Rule `ub`: `α × UB_i` (it uses each participant's own peak of the day, so it has
  hindsight). Rule `outflow`: `α × ΣUB × out_i / Σout` (no knowledge of when the peak falls).
- **Designs.** Gross (reject on arrival); a plain queue (`queue:fifo|bypass:<deadline>`: wait for
  incoming funds, reject at the deadline); micro-cycles. Each with a 10 s deadline and with a
  one-hour deadline. A rejected payment is not retried (`ASSUMPTION`).
- **Equivalent liquidity.** For a target share of rejected value (0.01%, 0.1%, 1%), the smallest
  `α` at which a design meets it, interpolated linearly on the `α` grid for each seed. The saving
  of a design is the paired difference with gross, `(α_gross − α_design) / α_gross`, averaged over
  seeds. This is liquidity saved *at equal service*, the comparison the liquidity-saving-mechanism
  literature makes; the saving of sections 5 and 9 is the special case of a target of exactly zero.

The gross retry inside the queue is defined as repeated passes over the queue in arrival order
until a pass settles nothing; the implementation visits only participants that were credited
since their last retry and is tested to give identical settlements (`event_driven_retry_equals_the_reference`).

## 12. Running on real data

`loopsettle bound <file.csv>` computes UB, LB and the delay bound for several `d`;
`loopsettle settle <file.csv>` also runs the settlement engines. The file is streamed twice (the
bound needs two passes), memory is O(participants), and amounts are converted to cents exactly.
The test `csv_path_equals_in_memory_path` checks that the file path gives the same UB, LB, engine
liquidity and bound as the in-memory path.

