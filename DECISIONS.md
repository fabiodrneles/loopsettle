# DECISIONS — design decisions

> [Versão em português](DECISIONS.pt-BR.md)

Each entry: what was decided, alternatives, why, and how to revert.

## D0 — The progress file is called `CHECKPOINT.md`
- **Decision:** the `PROGRESS.md` required by `CLAUDE.md` is `CHECKPOINT.md` (explicit user request).
- **Revert:** `git mv CHECKPOINT.md PROGRESS.md`.

## D1 — No dependencies for RNG, CSV, JSON and TOML
- **Decision:** the PRNG (SplitMix64/xoshiro256**), CSV/JSON writing and a parser for a subset of
  TOML are hand-written.
- **Alternatives:** `rand`, `serde`/`serde_json`, `toml`.
- **Why:** the minimal-dependency rule (CLAUDE.md, section 2) and bit-for-bit determinism
  (invariant 7) — an in-house PRNG does not change behaviour when crates are updated.
- **Revert:** replace the `rng`, `json`, `toml_lite` modules with the equivalent crates.

## D2 — Accepted dependencies
- `proptest` (dev): required by section 3.
- `minilp` (dev, only in `loopsettle-topology`): LP oracle in the CDFD tests (section 7, Phase 2).
- `wasm-bindgen` (only in `loopsettle-wasm`): browser bindings; without it the JS↔wasm ABI would
  have to be written by hand.

## D3 — An engine's liquidity = peak balance drawdown with an "unlimited" balance
- **Decision:** to measure the liquidity each engine needs, every participant starts with
  `UNLIMITED = 2^50` cents; liquidity needed is `opening_balance − lowest balance`. In
  `GrossRealTime` this equals UB (proven by a property test).
- **Alternatives:** binary search for the smallest opening balance that produces no
  rejections/delays above a threshold (costlier: several runs per participant).
- **Why:** it matches the measure attributed to Hellqvist & Korpinen (liquidity as the maximum
  accumulated debit position — **to be checked against the full text**, see
  [`docs/REFERENCES.md`](docs/REFERENCES.md)), runs in a single pass and keeps invariant 3
  (balance never negative) valid even in measurement mode.
- **Revert / alternative:** the engines also accept finite balances (constrained mode), used in
  the experiment with `QueueOnShortfall`.

## D4 — Hodge on the **net** graph, with L2 and L1 norms
- **Decision:** the Hodge decomposition uses the bilateral net flow (one edge per pair, oriented
  from the net payer). We report `‖curl‖/‖F‖` (L2, the "circular fraction" of section 5),
  `‖curl+harm‖/‖F‖` (L2) and `Σ|curl+harm| / ΣF` (L1, the same monetary unit as CDFD and
  netting, used in the "ladder"). On the gross graph, the bilateral offset part (2-cycles) is
  added to the Hodge L1 mass, for comparison with gross CDFD/cycles.
- **Alternatives:** L2 only (standard in the Hodge literature) — but not comparable with amounts
  in reais; Hodge on gross flow — not mathematically defined (edge flow on the complex is
  antisymmetric).
- **Why:** the ladder (Hodge ≥ CDFD ≥ cycles) only makes sense in the same unit. Observed result:
  the Hodge L1 mass can exceed 100% of the flow (components with the opposite sign to the flow),
  which shows that Hodge is a diagnostic, not a nettable amount.
- **Triangles:** configurable limit `HodgeConfig::max_triangles` (default 500,000); cycles not
  covered go into the harmonic component (reported separately).
- **Revert:** `Ladder::hodge_curl_l2` is already available; switch the column used in the ladder.

## D5 — Maximum CDFD via min-cost transshipment (successive shortest paths)
- **Decision:** instead of a "cost −1 circulation" with negative-cycle cancelling, we solve the
  equivalent problem: minimise `Σ d_e` with `0 ≤ d ≤ F` and the same divergence as `F`
  (non-negative unit costs), by successive shortest paths with Dijkstra + Johnson potentials, in
  integer cents. `c = F − d`.
- **Alternatives:** negative-cycle cancelling (minimum mean cycle); generic LP.
- **Why:** it is exact, integer, dependency-free and simple; non-negative costs avoid
  Bellman-Ford. Validated against `minilp` on 1,000 random graphs.
- **Revert:** replace `cdfd::cdfd_max`; the oracle test still applies.

## D6 — Proportional/local CDFD variant (BFF) postponed
- **Decision:** not implemented in this version.
- **Why:** the exact specification of the local algorithm must be checked in the paper
  (arXiv 2506.12546) before implementing; implementing it "from memory" would risk publishing a
  metric with the paper's name and different semantics. The maximum CDFD (static ceiling)
  already answers the central question.
- **How to complete:** read the local-algorithm section of the paper, implement it in `cdfd.rs`
  as `cdfd_proportional`, and add the column to the ladder.

## D7 — Cycle netting: deterministic DFS with a "current arc"
- **Decision:** greedy cycle cancelling (cycle minimum), exploring edges in input order
  (deterministic), cost O(n·m). Runs on the gross graph (including bilateral 2-cycles) and on
  the net graph.
- **Alternatives:** order by value (largest first), shortest cycles first.
- **Why:** it reproduces the spirit of Bech & Soramäki (resolving gridlock by cancelling cycles)
  and is the simplest "realizable" algorithm; CDFD gives the ceiling. The gap between the two is
  reported.
- **Revert / alternative:** change the order of `adj` in `netting.rs`.

## D8 — Arrival process: Poisson with a constant rate per hour
- **Decision:** piecewise intensity (24 hourly weights), exact simulation by exponentials.
- **Alternatives:** continuous rate (e.g. a sum of sinusoids) with *thinning*; self-exciting
  processes (Hawkes) for bursts.
- **Why:** simple, exact, fast and calibratable with hourly data if it becomes available. It does
  not model bursts (e.g. payday) — listed as a limitation.
- **Revert:** replace `PaymentStream::next`.

## D9 — Payee by a gravity model with receiver bias
- **Decision:** `P(j | i) ∝ activity_j^β · bias_j · (1 + h·[same class])`, `P(i→i) = 0`.
- **Why:** it produces a dense, mostly reciprocal network (like Pix P2P) plus a controlled
  directional component (receiver bias), without needing bilateral data that are not public.
  The bias parameters directly control the circular fraction and are flagged `ASSUMPTION`; the
  sensitivity analysis is in the `baseline` experiment.
- **Revert:** weights in `PaymentStream::with_population`.

## D10 — Amount calibration by a "body + B2B tail" mixture
- **Decision:** the Pareto tail represents large business payments: its probability = the B2B
  share of count (4.30%) and its mean is solved so that it carries the B2B share of value
  (52.05%); the lognormal body takes the rest. The mean ticket (R$ 449.83) is exact analytically.
- **Alternatives:** fit only the mean ticket (one free parameter, arbitrary tail); use value
  bands (not published by the API).
- **Why:** it uses two public aggregates instead of one, and the share of value concentrated in a
  few large payments is what weighs most on liquidity.
- **Revert:** `calibrate::apply`; or override `tail_prob`/`tail_xm_brl` in the TOML.

## D11 — Participant size ∝ Pix keys per ISPB
- **Decision:** each institution's share as payer/payee uses its share of Pix keys (`ChavesPix`,
  2026-09-30), public data per participant.
- **Alternatives:** there are no public per-participant transaction data in the API used.
- **Risk:** keys ≠ transactions (digital banks have many keys per customer). Flagged as a SOURCED
  proxy with an embedded ASSUMPTION.

## D12 — Micro-cycles: one queue head per edge, cycles of up to 6 participants
- **Decision:** FIFO queues per (payer, payee) pair. In each window, a DFS looks for a directed
  cycle among edges with a non-empty queue (length ≤ `max_cycle_len`, default 6) and tries to
  settle atomically **the oldest payment on each edge** of the cycle (`try_batch`). If any
  participant lacks funds to cover its net position in the cycle, its outgoing edge is blocked
  for that window and the search continues. Repeats until no cycle is left.
- **Alternatives:** solve an optimisation problem per window (maximum settleable subset, as in
  full gridlock algorithms); combine several payments from the same edge in one cycle.
- **Why:** it is the simplest, deterministic and cheapest form that respects per-edge FIFO,
  atomicity and the rule "never nets more than the cycle's smallest payment". It underestimates
  what a full optimiser could do — recorded as a limitation.
- **Configurable:** `max_cycle_len` (`micro:<mode>:<window>:<delay>:<len>`).

## D13 — `QueueOnShortfall`: no per-payer FIFO (bypass)
- **Decision:** a payment settles gross on arrival if funds suffice, even if the payer has older
  payments in the queue; queued payments are retried gross in each window, in arrival order
  (per-edge FIFO), and rejected at the deadline if funds are still short.
- **Alternatives:** strict per-payer FIFO (as in several RTGS systems).
- **Why:** it is the behaviour most favourable to Pix's immediate settlement. Observed effect:
  with constrained liquidity, the bypass can slightly **increase** rejection by value (new
  payments consume the balance that would release the old ones). Documented in the results.

## D14 — Expired payments go gross; if funds are short, they are rejected
- **Decision:** at the deadline `arrival + max_delay_ms` (exact, not rounded to the window), the
  payment is tried gross; without funds, it is rejected (as Pix rejects). A window that
  coincides with the deadline runs first (last chance to net).

## D15 — `DeferredNet` always settles (collateralised DNS)
- **Decision:** each cycle applies the net positions without checking balances (they may go
  negative in constrained mode); each payment's delay is known on arrival (next cycle), so the
  engine keeps only per-participant deltas (O(n) memory).
- **Why:** this is how the liquidity of deferred net systems is measured (e.g. UPI, the Finnish
  study): the need is the debit position at settlement times.

## D16 — Interface: one wasm function returning JSON; React only draws
- **Decision:** the wasm module exposes `simulate(seed, participants, payments, window, delay,
  bias, liquidity fraction) -> JSON string`; the Web Worker calls it and React draws on Canvas
  2D. Public calibration embedded with `include_str!`. Limits: 50 participants, 100,000
  payments. `wasm-opt` disabled (avoids an extra download; the .wasm is 332 KB).
- **Alternatives:** expose Rust objects to JS (more bindings, more glue code).
- **Why:** a minimal boundary that is testable natively (`cargo test -p loopsettle-wasm`).

## D17 — Bilingual documentation and interface (Portuguese and English)
- **Decision:** the README has two complete sections (Portuguese and English) with screenshots
  in each language (`docs/img/` and `docs/img/en/`). **English is the default for every other
  file**, following the usual GitHub convention: the plain name is English and the Portuguese
  twin sits next to it with a `.pt-BR.md` suffix — `docs/<NAME>.md` ↔ `docs/<NAME>.pt-BR.md`,
  `DECISIONS.md` ↔ `DECISIONS.pt-BR.md`, `REVIEW.md` ↔ `REVIEW.pt-BR.md`, `CHECKPOINT.md` ↔
  `CHECKPOINT.pt-BR.md`, `results/SUMMARY.md` ↔ `results/SUMMARY.pt-BR.md` (both generated by
  `scripts/summarize.py`); `data/README.md` and `results/README.md` hold both languages in one
  file. The web UI has a PT/EN switch (`?lang=pt|en`, remembered per viewer; default from the
  browser language). `CLAUDE.md` is the owner's build brief for the coding agent and stays as
  written.
- **Alternatives:** separate `README.pt-BR.md`; Portuguese as the default name (rejected: an
  English reader opening a folder on GitHub would land on Portuguese).
- **Why:** the user asked for complete documentation in both languages; the target audience
  (central banks, payments researchers, JOSS reviewers) is international.
- **Rule:** any change to a document must be made in both languages in the same commit.
