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
  accumulated debit position — **confirmed in the full text**, BoFER 7/2021 pp. 3–4: the
  "upper bound" in instant mode and the minimum of end-of-cycle positions with cycles), runs in a single pass and keeps invariant 3
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

## D6 — Proportional/local CDFD variant (BFF) postponed — **done 12 Nov 2026**
- **Update:** implemented in `crates/loopsettle-topology/src/cdfd_bff.rs` from the paper's
  Algorithm 2 (section 5.2 and Appendix F.2, read in the HTML version), in `f64`; column `cdfd_bff`
  of the ladder. Checked by properties on 1,000 random graphs and two hand examples; the paper's
  own example (Figure 1) has no per-edge values.
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
- **Update, 11 Nov 2026:** strict per-payer FIFO is now an option
  (`QueueDiscipline::StrictFifoPerPayer`, engine spec `micro:shortfall:<window>:<delay>:<len>:fifo`);
  bypass stays the default. The gross retry now runs to a fixed point in each window: before, it
  released at most one payment per edge per window (REVIEW 82).
- **Revert:** omit the sixth field of the engine spec (or pass `bypass`).

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

## D18 — Intraday heterogeneity by institution class (W2)
- **Decision:** optional class-specific out/in hourly multipliers mixed with a flat profile by
  `profile_strength` λ and rescaled so every class keeps its daily volume. The payee is drawn in
  two exact stages (class by hour, then participant), keeping O(n²) memory. λ = 0 keeps the old
  code path, so every earlier result is byte-identical.
- **Alternatives:** per-participant profiles (no data to set them); Hawkes bursts (D8).
- **Why:** stationary arrivals make the end-of-day potential vanish as 1/√N; systematic
  imbalances are the only mechanism that could make longer-horizon netting matter at scale, so
  the study must show results with and without them.
- **Revert:** set `profile_strength = 0` (default).

## D19 — Calibrated receiver bias, default kept at 0.4
- **Decision:** σ = 1.0 (band 0.8–1.2) is the value calibrated to BCB WP 619; experiments that
  answer the central question (`robustness`, `heterogeneity`) report it next to σ = 0.4. The
  default stays 0.4 so that earlier experiments and figures remain comparable.
- **Alternatives:** switch the default to 1.0 and regenerate every result.
- **Why:** σ = 1.0 makes every saving smaller (LB/UB ≈ 0.97), so keeping 0.4 as the default is
  the conservative choice for the claim "savings are small"; the paper should quote both.
- **Revert / alternative:** set `receiver_bias_sigma = 1.0` in the experiment TOMLs and run
  `make all`.

## D20 — Funding windows: prefunding measured inside closed windows
- **Decision:** outside STR hours the Conta PI cannot be topped up, so the relevant need is the
  drawdown from the start of each closed window (nights, the weekend); inside open windows top-ups
  are assumed unlimited. Five days (Thursday to Monday) with the sourced hourly profile and day
  volumes (`funding-windows`).
- **Alternatives:** model top-up behaviour and the Conta PI remuneration explicitly; holidays.
- **Why:** it is how the BCB describes the liquidity constraint (Relatório Anual do SPI 2024,
  p. 15; Relatório de Gestão do Pix 2023, p. 16).
- **Revert:** the one-day UB of the other experiments remains available.

## D21 — Real-data commands output aggregates only
- **Decision:** `loopsettle bound/settle` stream the file, keep only per-participant state and
  write aggregates; per-participant rows only with `--per-participant`. Output goes to
  `loopsettle-out/` (git-ignored), never to `results/`.
- **Why:** an institution must be able to run it on confidential data and share the output.

## D22 — Specs are working documents, in Portuguese only
- **Decision:** `specs/` holds the post-v0.1.0 roadmap as short specifications (context and
  evidence, requirements, acceptance criteria, how to validate). They are written in Portuguese
  only; everything a user reads (README, docs, results) stays bilingual.
- **Alternatives:** twin files for every spec.
- **Why:** specs are instructions for whoever continues the build and change often; a twin for
  each doubles the cost without a reader for it. CLAUDE.md asks for documentation in Portuguese.
- **Revert:** translate `specs/*.md` to `*.md` + `*.pt-BR.md` as in D17.

## D23 — The delay bound is reported in two forms; the tight one is the result
- **Decision:** the liquidity floor `max_s [CO_i(s) − CI_i(s+d)]` (`UB − floor` = tight bound)
  is the bound the study reports; the window form `O_i[t* − d, t*]` is kept as a corollary and in
  every existing CSV column so earlier results stay comparable.
- **Alternatives:** replace the window form everywhere; keep only the window form.
- **Why:** the tight bound is 14× smaller for 10 s in the calibrated scenario and is attained by
  a single participant, so it is the stronger and more honest statement; it also showed that
  claim 1f (window bound explains retail vs large-value) was an artefact of the window form.
- **Revert:** ignore the `tight_*` and `liquidity_floor` columns.

## D24 — Constrained liquidity: fixed balances, two rules, no retries, equal-service comparison
- **Decision:** `liquidity-frontier` fixes opening balances by two rules (`ub`: `α × UB_i`;
  `outflow`: `α × ΣUB × out_i / Σout`), never retries a rejected payment, and compares designs by
  *equivalent liquidity*: the smallest `α` at which the rejected value is at most a target (0.01%,
  0.1%, 1%), interpolated linearly on the grid, per seed. Designs are run with a 10 s deadline and
  with a one-hour deadline. The plain queue (`queue:fifo|bypass`) is the micro-cycle engine in
  shortfall mode with the cycle search off, retried every 100 ms (1 s for the one-hour runs).
- **Alternatives:** balances from a behavioural model; retries with some probability; unbounded
  queues settled at the end of the day (the large-value convention); event-driven release instead
  of a retry tick.
- **Why:** it is the smallest design that makes LoopSettle comparable with the liquidity-saving
  literature (queue under constrained liquidity) while keeping an instant-payment deadline as the
  main case; every choice is a parameter of `experiments/liquidity-frontier.toml`.
- **Revert:** the experiment is additive; no earlier result depends on it.

## D25 — The offline optimum is an example with a test-only solver, not a CLI experiment
- **Decision:** the linear programme of the offline optimum lives in
  `crates/loopsettle-sim/tests/common/optimum.rs` and is solved with `minilp`, a dev-dependency of
  `loopsettle-sim`. Its results are generated by
  `cargo run --release -p loopsettle-sim --example optimality_gap` (also run by `make all`), not
  by `loopsettle experiment`.
- **Alternatives:** an optional `oracle` feature of the CLI; a hand-written simplex.
- **Why:** CLAUDE.md allows an LP solver only as a test oracle, never in the main path; examples
  can use dev-dependencies, so the library and the CLI stay dependency-free while every number
  still comes from one command.
- **Revert:** delete the example and `results/optimality-gap/`; nothing else depends on them.
- **Update to D21, 12 Nov 2026:** `--per-participant` now writes labels `P001...` instead of the
  file's identifiers (`--keep-names` restores them) and is refused for files with fewer than
  `--min-participants` participants (default 5). `--report` writes a bilingual report with
  aggregates only, identifying the input by an FNV-1a hash. Not done: suppressing rows that would
  reveal a single payment (a participant whose peak is one payment).

## D26 — Echo payments as a stress test, off by default, with a kill criterion fixed beforehand
- **Decision:** the generator can make a share of all payments a response to a receipt
  (`echo_share`, `echo_delay_ms`, `echo_return_prob`): the payee pays the same amount after an
  exponential delay, onwards or straight back. Echoes use their own random stream and the base
  rate is lowered so the expected volume is unchanged; with `echo_share = 0` the stream is
  byte-identical to before. The experiment fixed its criterion before running: the central claim
  is conditional if the 10 s tight bound exceeds 2% of UB for a share of at most 20%.
- **Alternatives:** Hawkes arrivals; partial amounts; echoes of echoes; calibrating the share.
- **Why:** it is the simplest mechanism that creates the short-horizon circularity second-scale
  netting could use, so it is the most adverse test of the claim that such netting saves nothing.
- **Outcome:** the criterion fired for echoes that return to the payer (ASSESSMENT, update 5).
- **Revert:** `echo_share = 0` (the default).

## D27 — Gridlock resolution in the manner of Bech & Soramäki, from secondary descriptions
- **Decision:** `gridlock:fifo|bypass:<window>:<delay>` queues payments on a shortfall and, every
  window, settles simultaneously the largest set of queued payments that is a prefix of every
  payer's queue and leaves no balance negative (start with everything; while a participant is
  below zero, drop the last payment still included in its queue). `bilateral:<mode>:<window>:<delay>`
  is the micro-cycle engine restricted to cycles of two.
- **Alternatives:** the exact algorithm variants of the 2001 paper (the paper is not in
  `docs/references/` and was not read); partial settlement; optimisation-based solvers.
- **Why:** it is the classic liquidity-saving mechanism the literature compares against, and it
  settles batches that are not cycles, which the micro-cycle search cannot.
- **Caveat (resolved 13 Nov 2026):** implemented from secondary descriptions, then checked against
  the paper (Bank of Finland DP 9/2001, §3.3, copy supplied by the owner): Step 1 include every
  queued payment, Step 2 compute ex-post balances, Step 3 remove the last payment of any deficient
  bank and repeat; the result does not depend on the bank picked (footnote 9). `run_gridlock` is
  this algorithm; the name "Bech-Soramäki" can be used. REVIEW 97.
- **Revert:** do not use the `gridlock` engine; nothing else depends on it.

## D28 — Funding as a layer outside the engines, with reserves outside the system
- **Decision:** a `FundingPolicy` acts between engine events at instants it chooses; money it
  moves comes from reserves outside the system (`Ledger::fund`), so invariant 1 becomes
  "accounts plus reserves are constant". Engines are told when balances changed from outside
  (`SettlementEngine::funded`) so queued payments are retried. Intraday credit is a per-account
  lower limit (`with_credit_limits`). Remuneration of balances and the opportunity cost of
  prefunding are not modelled (no source).
- **Alternatives:** funding inside each engine; credit as a separate account; a cost model.
- **Why:** it is the smallest change that represents what D20 describes (top-ups only while STR is
  open) for every engine at once, and it keeps earlier results byte-identical (no policy = no
  change).
- **Revert:** run without a policy (the default).

## D29 — The Bank of Finland design as an experiment: ρ, π, liquidity levels, complete netting
- **Decision:** `bof-replication` reproduces the design of Koponen & Soramäki (1998): opening
  balance `LB_i + LL (UB_i − LB_i)` at 11 levels (their equation 4), payments may wait until the
  end of the day, and designs are compared by ρ (`RunMetrics::rho`, equation 5) and π
  (`metrics::pi`, equation 6). A new engine, `netq:fifo|bypass:<window>:<delay>[:<retry>]`, is
  their netting of queues: everything at once if every participant covers its net (equation 3),
  otherwise nothing. `gridlock` and `netq` take an optional gross-retry interval
  (`MicroCycleConfig::batch_every`), so a design can net every 20 minutes while retrying the queue
  every second.
- **Choices inside it:** (a) ρ in continuous time (ms), not per minute; (b) a rejected payment
  counts as waiting until the end of the day (the 1998 framework has none: everything settles by
  the end of the day); (c) system ρ sums numerators and denominators over participants, as the
  1998 text does (the weights are then each payer's area of cumulative outflows, not exactly its
  share of value, as the text says); (d) the 2001 paper writes the levels as `α (UB_i − LB_i)`
  (equation 8), without `+ LB_i`; we follow 1998 and the 2001 paper's own words; (e) a stylised
  large-value scenario at 1,500 payments a day (`ASSUMPTION`, the one of `regimes`) checks that the
  implementation reproduces the published effect where the published studies found it.
- **Why the retry interval:** without it, "net every 20 minutes" also meant "retry the queue every
  20 minutes", which raised ρ against a queue retried every second (measured: +0.23 pp at the
  lowest level). The 1998 simulator checks its queue on every event.
- **Alternatives:** per-minute ρ (closer to the text, coarser); excluding rejected payments (then
  ρ would reward rejecting); adding the levels to `liquidity-frontier` (heavier, and it would change
  its reference results).
- **Revert:** drop the experiment; nothing else uses `netq`, `rho`, `pi` or `batch_every`
  (`batch_every = 1` is the old behaviour, and all earlier reference results are unchanged).

## D30 — The value distribution is fitted to the public percentiles
- **Decision:** with `[value_percentiles]` in the calibration file, `calibrate::apply` fits the
  body dispersion (`body_sigma`) and the tail's count share (`tail_prob`) to the p50, p90 and p99
  of the value of an interbank Pix (R$ 36 / 400 / 3,900, *Relatório Anual do SPI 2025*, p. 10,
  December 2025), by least squares of the log ratios on a grid of exact decimals (tail 0.001 to
  0.100 by 0.001; σ 0.80 to 3.00 by 0.02), every candidate holding the mean ticket and the tail's
  share of value (B2B, 52%). Result: σ = 1.9, tail 0.3%; p50 R$ 35.76, p90 R$ 419, p99 R$ 3,784.
- **Before:** σ = 1.4 (`ASSUMPTION`) and tail = B2B count share (4.3%): right mean, wrong shape
  (p50 R$ 91.5). Typical payments were 2.5 times too large.
- **Alternatives:** fit α of the Pareto tail too (the three percentiles do not identify it: the
  tail starts above p99); a different body law; weight the percentiles differently.
- **Choices inside it:** the normal CDF uses a written-out `erfc` (Numerical Recipes, error
  < 1.2e-7) so the fit does not depend on a platform's special functions; the grid is exact
  decimals so the chosen pair is the same everywhere. The percentiles (December 2025) and the other
  targets (September 2026) are from different months (`ASSUMPTION`).
- **Revert:** `scenario.amounts_fit = "mean"` in an experiment file, or remove `[value_percentiles]`
  from `data/calibration/pix.toml`. Every calibrated result changed with this decision (REVIEW 99).

## D31 — The preprint draft is written in English only
- **Decision:** `paper/preprint-liquidity-floor.md`, the draft of the paper on the liquidity floor,
  exists only in English, the language of SSRN and of the target journal; it is listed as a
  single-language document in `scripts/check_twins.py`. It is a draft for the author and the
  reviewer, with the points that need their judgement marked `[TODO]` (CLAUDE.md, phase 10: the
  authors write the paper).
- **Alternatives:** a Portuguese twin (doubles the work on a text that will be rewritten); drafting
  in Portuguese and translating at the end.
- **Revert:** add `paper/preprint-liquidity-floor.pt-BR.md` and remove the entry from the list.
