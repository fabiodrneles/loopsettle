# REVIEW — points to review before writing the paper

> [Versão em português](REVIEW.pt-BR.md)

Each point: **what was done**, **where it is**, **result** and **risk/question**.
Numbers from [`results/SUMMARY.md`](results/SUMMARY.md) (seed 20260930, 200 participants,
1 million payments, Sep 2026 calibration). Reproduce everything: `make rebuild`.

---

## A. Design decisions ([`DECISIONS.md`](DECISIONS.md))

1. **D0 — `CHECKPOINT.md` instead of `PROGRESS.md`.** User request. No impact on results.
2. **D1 — Hand-written PRNG, CSV, JSON and TOML.** `core/src/{rng,json}.rs`,
   `sim/src/toml_lite.rs`. Result: bit-for-bit determinism tested (invariant 7), including across
   `make` runs. Risk: the TOML parser accepts only a subset (no nested tables or multi-line
   strings); a richer valid TOML fails with a line-numbered message.
3. **D2 — Dependencies:** `proptest` (dev), `minilp` (dev, oracle), `wasm-bindgen`. Risk: none on
   the main path.
4. **D3 — Liquidity = peak balance drawdown with an "unlimited" balance.** `core/src/ledger.rs`.
   Result: for gross settlement it equals UB (test `ub_is_tight`). Question for the reviewer: is
   this Hellqvist & Korpinen's measure? Check the exact definition in their paper. For
   micro-cycles, the measure ignores that waiting 10 s may also reduce rejections under
   constrained liquidity (handled separately in the constrained-liquidity experiment).
5. **D4 — Hodge on the net graph, L1 and L2.** `topology/src/hodge.rs`, `lib.rs`. Result: Hodge L2
   curl = 0.977 on the net graph; L1 = 1.73 (net), 1.18 (gross). Risk: adding the bilateral mass
   to the Hodge mass on the gross graph is our own convention; the reviewer should decide whether
   the paper's ladder uses the net graph (cleaner) or the gross one.
6. **D5 — Maximum CDFD via min-cost transshipment.** `topology/src/cdfd.rs`. Result: equal to
   `minilp` on 1,000 graphs; acyclic directional part (tested). Question: confirm that the
   "maximum circularity decomposition" of arXiv 2506.12546 is exactly this problem (maximise Σc
   with 0 ≤ c ≤ F and conservation). See also point 7.
7. **D6 — Proportional/local CDFD (BFF) not implemented.** Risk: the CDFD paper gives prominence
   to the local variant; without it, the comparison with the paper is partial.
8. **D7 — Greedy cycle netting (DFS, input order).** `topology/src/netting.rs`. Result: 0.795
   (gross) vs CDFD 0.831 — the greedy order loses 3.6 p.p. Question: the order (lexicographic by
   participant id) favours low-id participants (the largest ones).
9. **D8 — Poisson with a constant rate per hour.** `sim/src/stream.rs`. Risk: no bursts.
10. **D9 — Gravity payee with receiver bias.** `sim/src/stream.rs`. Result: it is the parameter
    that moves the results most (LB/UB from 0.59 to 0.95 as σ goes from 0 to 0.8). **The study's
    biggest risk** (see point 26).
11. **D10 — Amounts: lognormal body + B2B tail.** `sim/src/calibrate.rs`. Result: errors ≤ 1.5%
    on the aggregates.
12. **D11 — Size ∝ Pix keys.** Risk: keys ≠ transactions.
13. **D12 — Micro-cycles: one head per edge, cycles ≤ 6, blocking on shortfall.**
    `core/src/engines.rs`. Risk: heuristic; a full optimiser could net more. Sensitivity test with
    `max_cycle_len` ∈ {2, 3, 6, 10} not yet run.
14. **D13 — `QueueOnShortfall` without per-payer FIFO.** Result: with 25% of UB, it rejects
    **more** value than gross settlement (23.23% vs 22.95%). Question: implement strict FIFO as a
    configurable alternative before concluding anything about LSMs.
15. **D14 — Expired payments go gross; without funds, they are rejected.**
16. **D15 — `DeferredNet` always settles (collateralised DNS).** Balances may go negative in
    constrained mode — that is why `deferred_net_10x` never rejects under stress.
17. **D16 — Interface: one wasm function returning JSON.**

## B. Assumptions (`ASSUMPTION` in [`docs/PARAMETERS.md`](docs/PARAMETERS.md))

18. `participants = 200` (scale). 19. `days = 1`. 20. `payments_per_day = 1,000,000`.
21. `zipf_s` (unused with calibration). 22. `n_big = 5`, `n_mid = 15`.
23. `fintech_frac = 0.5`. 24. `payee_size_exp = 1`. 25. `homophily = 0.3`.
26. **`receiver_bias_sigma = 0.4`** — see point 10; it cannot be calibrated with the public data
    used. Suggestion: look for a public aggregate of net positions/liquidity use in the SPI to
    anchor LB/UB.
27. `pi_receiver_mult = 1.3`. 28. Hourly profile (`hourly`). 29. `body_sigma = 1.4`.
30. `tail_alpha = 1.3`. 31. `max_amount = R$ 5 M`. 32. Stress: 100% resubmitted.
33. Embedded proxy: B2B = the "tail" of large amounts (B2B payments are not necessarily the
    largest).

## C. Results to examine

34. **Potential vs realizable (central result).** BCB-style potential = 85.4% of volume;
    potential liquidity saving from one end-of-day netting = 9.27% of UB; realizable with
    micro-cycles (≤10 s) = 0.0147% of UB, although 63% of value settles inside cycles. Netting
    every 10 s: 0.028%; every 15 min: 0.32%. Proposed interpretation: UB is dominated by
    hour-long imbalances (drift), not by second-scale noise. **Risk:** it depends on scale
    (point 41) and on the bias (point 10). With σ = 0 the saving rises to 0.048% — still
    negligible.
35. **Ladder:** gross Hodge L1 1.18 > potential 0.85 > CDFD 0.83 > cycles 0.80; net Hodge L2 0.98 ≫
    CDFD 0.17 > cycles 0.14. The inequality "Hodge ≥ CDFD" held in every sensitivity run, but it is
    not a theorem (hence the "≥?" in CLAUDE.md).
36. **Hodge on a dense graph** (LIMITATIONS 11): the high curl fraction may be a dimension effect.
    Suggested test: Hodge on purely random symmetric flows (no structure) on the same graph — if it
    also gives ≈0.97, the metric does not discriminate structure.
37. **Hodge potential × net position correlation = 0.997** (Spearman 0.93). Almost mechanical on a
    dense graph (φ ≈ position/n). It should not be presented as a finding.
38. **Finland:** instant needs 1.10× (σ=0.4) to 1.50× (σ=0) the liquidity of one cycle per day;
    1.01–1.03× with 48 cycles. Question: compare with the order of magnitude published by
    Hellqvist & Korpinen (check the figure in the paper; see point 64).
39. **Stress:** negligible effect on UB (+0.005%). Participant 0's UB does not change because its
    debit peak is at the end of the day, and all delayed payments are resubmitted. Question: the
    stress scenario is weak; consider partial abandonment, an outage at the participant's peak
    time, or an outage of a net receiver. → Redesigned in item 77.
40. **Constrained liquidity:** `QueueAll` micro-cycles reduce rejection only slightly (22.43% vs
    22.95% at 25% of UB) at the cost of a 10 s p95.
41. **Scale:** with 239× fewer payments per second, each window has 239× fewer cycle candidates.
    Suggestion: run `microcycle-sweep` with 10 and 30 million payments (fits in memory: current
    use ~6 MiB; time grows ~linearly from 16 s per million, i.e. ~3 min and ~8 min).
42. **Quantised percentiles:** the histogram reports the bucket's lower bound (e.g. 9,984 ms for
    10,000 ms). Difference < 1.6%; documented.

## D. Tests that were hard to pass / pitfalls found

43. `hodge_invariants` failed on the empty graph: the tolerance `1e-7·|E|` became 0 (`0 < 0`).
    Fixed to `|E| + 1`. Not an algorithm error.
44. Length-limited cycle search: the first version marked as "explored" a node whose subtree had
    been cut by the limit — it could miss cycles. Fixed before the first test (`truncated` flag
    propagated). The property tests passed first time after that; the reviewer should read
    `find_cycle` carefully.
45. Possible underflow when skipping idle windows (`t - next_tick`), fixed in code review, before
    the tests.
46. Olinda API: `$skip` returns HTTP 500 and dates require an unquoted OData literal; the script
    downloads everything in one request and aborts if it may have been truncated.
47. File reproducibility: the code version changed when committing the results themselves; it is
    now the last commit outside `results/`, and times/RAM go to `run.json`.

## E. Known limitations ([`docs/LIMITATIONS.md`](docs/LIMITATIONS.md))

48. Synthetic data; 49. hypothetical bilateral structure; 50. keys as a proxy; 51. scale;
52. hourly profile without a source and without bursts; 53. one-day horizon; 54. heuristic
micro-cycles; 55. the measured delay is only the engine's; 56. no proportional CDFD; 57. on-chain
validation not done; 58. Hodge on a dense graph; 59. interface: **checked visually on 5 Nov 2026**
(desktop Chromium 1440 px and an emulated iPhone 13, in Portuguese and English; screenshots in
`docs/img/` and `docs/img/en/`, generated by `node scripts/screenshots.cjs`). Still to be tested on
the real laptop (i3, 4 GB).

## F. Items that depend on the reviewer

60. References: checked by public search on 5 Nov 2026; status of each one in
    [`docs/REFERENCES.md`](docs/REFERENCES.md). The full texts still need to be opened (the
    Bank of Finland PDFs could not be opened from this environment).
61. The repository URL in `Cargo.toml`/`CITATION.cff` (`github.com/fabiodrneles/loopsettle`) is an
    assumption until the repository is public.
62. **PixSim (arXiv 2609.30684) not found.** No public search found the work. Removed from the
    README; still listed in `docs/NOVELTY.md` as "not found". Do not cite it in the paper without
    confirming the identifier.
63. **CDFD title.** arXiv 2506.12546 is called *Circular Directional Flow Decomposition of
    Networks*. The title *How circular is a directed network?* and publication in *Royal Society
    Open Science*, which were in the README, were not confirmed; the README now cites the arXiv.
64. **The Finnish study's figure (≤ 8.6%).** It comes from the discussant's summary at Economics of
    Payments X, not from the paper's text. Check it in BoFER 7/2021 and in the JFMI 13(2) version,
    and check how many cycles per day the STEP2 reference model uses: the fair comparison with
    LoopSettle is against that number of cycles (our +2.6% with 10 cycles/day, +9.5% with 1
    cycle/day, σ = 0.4).
65. **Per-bank heterogeneity (a strength of the Finnish study).** LoopSettle has UB and LB per
    participant (`baseline/participants.csv`), but the `finland-replication` experiment reports only
    the aggregate ratio. Suggestion: add the distribution of the instant/cycles ratio per
    participant (participants with LB = 0 need separate treatment).
66. **Interface bug fixed:** under `npm run dev` the simulation never finished ("Simulando…"
    forever). Cause: React StrictMode (development only) unmounts and remounts the component; the
    cleanup terminated the Web Worker and `useMemo` returned the same dead worker. The production
    build was not affected. Fixed in `web/src/App.tsx` (worker created inside `useEffect`). Clipped
    chart axis labels were also fixed (`canvas.ts`).
67. **Negative saving in the browser:** in the interface's default scenario (40 participants,
    100,000 payments), the micro-cycle saving with a 1–2 s window comes out at −0.001%. Consistent
    with the central result, but the reviewer should decide whether the paper mentions that
    micro-cycles can slightly increase the peak.
68. **Bilingual documentation (D17).** README, `docs/`, `DECISIONS`, `REVIEW`, `results/SUMMARY` and
    the web UI exist in Portuguese and English. The translation was done by hand from the
    Portuguese originals; the reviewer should check the technical terms (e.g. "compensação" =
    netting, "liquidação bruta em tempo real" = real-time gross settlement, "Conta PI" = Pix
    settlement account).
69. **International framing (`docs/INSTANT_PAYMENTS.md`).** The README, the UI and the docs now
    present LoopSettle as a study of instant payments with gross settlement, with Pix as the
    calibrated case, and map 11 systems to the engines. SPEI, Interac e-Transfer and Canada's RTR
    were checked against primary sources (Banxico, Interac, Bank of Canada); Singapore's FAST was checked
    against MAS sources too, so no row is left "to check". The SPEI row matters for the paper: Banxico describes a clearing algorithm that
    settles the payable sets every 1.9 s on average, a real-world analogue of atomic micro-cycles
    and a natural comparison; Canada's RTR (prefunded, launching Q4 2026) is a live case for the
    Finnish study's migration question. Pix is a high-renown
    trademark of the BCB (INPI, June 2026); the README and the UI state that LoopSettle is
    independent and not endorsed by the BCB.
70. **Robustness experiment and verdict (5 Nov 2026).** `results/robustness/` and
    [`docs/ASSESSMENT.md`](docs/ASSESSMENT.md): micro-cycle saving ≈ 0.01% survives seeds, σ,
    scale and cycle length; the end-of-day "potential" shrinks as 1/√(volume), so the old
    "630×" headline was removed. Verdict PIVOT; the remaining work is specified in
    [`docs/PAPER_READINESS.md`](docs/PAPER_READINESS.md) with kill criteria K1–K4.
71. **Delay bound.** New module `crates/loopsettle-sim/src/bound.rs` and METHODOLOGY §9; the
    reviewer should check the proof and the test `engines_never_beat_the_delay_bound`.
72. **Full texts read (5 Nov 2026).** BoFER 7/2021, BCB WP 619, Johnson et al. (2004) and the
    CDFD paper: items 4, 6 and 64 are resolved (definitions and figures confirmed, see
    `docs/REFERENCES.md` and the update section of `docs/ASSESSMENT.md`). New: WP 619's per-bank
    quartiles are a real-data target for σ (model p25 63% vs 18–45% in Pix) — workstream W2b.
73. **W2 and W2b done.** σ calibrated to WP 619 (1.0; band 0.8–1.2, D19); class-specific intraday
    profiles (D18). Kill criterion K1 not fired (10 s bound ≤ 0.46% of UB). With calibrated σ, the
    end-of-day potential at 30M is 0.24%–2.8% depending on intraday heterogeneity — the remaining
    unknown, measurable only with per-participant intraday data (a BCB partnership question).
    The reviewer should check the profile shapes in `scenario.rs` (`CLASS_OUT_PROFILE`,
    `CLASS_IN_PROFILE`).
74. **Real-data commands (W9.1).** `loopsettle bound/settle/export`; the reviewer should try
    `bound data/samples/payments_example.csv` and check the format rules in `csvfile.rs`.
75. **Funding windows (W3, K3 not fired).** The weekend is the binding window (≈ 2.7× a day's
    peak); netting at its end saves 0.4–1.0%. Open question: holidays and real top-up behaviour.
76. **McMahon et al. (2024)** is the closest work (large-value, 14–30% savings with < 5 s
    reordering). `regimes` shows the bound is 13.7% in a lumpy large-value scenario vs 0.76% in
    retail. **Read on 5 Nov 2026 (arXiv v3):** the paper is on Canada's LVTS with real data
    (batches of 70 payments, ≈ 90 s, C$240 M/day saved vs FIFO, ≈ 1–2% of allocated liquidity);
    its measure (aggregate mNDP) equals our UB. The 14–30% figure is from the separate 2024 T2
    talk, whose FIFO-with-queue baseline is not our UB. The reviewer should check the row in
    `docs/REFERENCES.md` §7.
77. **Stress redesigned (REVIEW 39, W6).** `results/stress/cases.csv`: 3 targets (largest, top net
    payer, top net receiver) × 3 starts (10:00, 18:00, 30 min before the target's own debit
    peak). Spillover on the others' UB ≤ 0.063%; rejected value with openings at the baseline UB
    ≤ 0.08%. A 30-minute outage with full resubmission barely moves liquidity needs, because
    they are set by the day's cumulative drift, not by intraday timing — the same reason
    micro-cycles save little. Not modelled: partial abandonment, outages longer than 30 min.
78. **Reference PDFs in the repository.** `docs/references/pdf/` holds 19 of the 20 documents
    cited (the CDFD preprint is only linked: its arXiv licence does not allow redistribution),
    with an index and `references.bib`. Each is redistributed as published by its
    author or institution, with the original link; the reviewer should confirm that each
    licence allows it before the repository is promoted (removal on request is stated).
79. **Liquidity floor (tight delay bound), 11 Nov 2026.** `bound.rs::liquidity_floors`,
    METHODOLOGY §9b, CLAIMS 1g, D23. For 10 s the bound drops from 0.76% to 0.054% of UB and
    netting every 10 s realises 51% of it (`results/regimes/bound_curve.csv`). With 10 seeds
    (`robustness/seeds_summary.csv`): 0.014% (σ = 1.0) to 0.138% (σ = 0). **Risks:** heterogeneity
    and funding-window runs still use the window form (spec 02); the theorem is simple
    enough that it may already exist in the literature (spec 01, open question 1) — check before
    claiming novelty; the floor is per participant and the sum is not attainable (spec 03).
80. **Claim 1f revised.** With the tight bound the stylised large-value system also shows ≈ 0
    (≤ 0.004%), so the window bound does not reconcile LoopSettle with McMahon et al.; the
    baseline (FIFO queue under constrained liquidity) does, and that is not yet simulated
    (spec 06). ASSESSMENT "Update 3", README, NOVELTY, PAPER_OUTLINE and the JOSS draft still
    carry the old argument (spec 02, FR6). **This must be fixed before any text is submitted.**
81. **Audit of 11 Nov 2026 and the spec set.** `specs/00-auditoria.md` lists what was read, what
    was not, and 14 findings with evidence labels. Read-but-untested defects: `retry_gross`
    releases one payment per edge per window (affects constrained-liquidity results only),
    `fraction_at_most(2800)` counts delays up to 2,815 ms, `DeferredNet` truncates the interval
    for cycle counts that do not divide the day (no published result affected). Cross-platform
    bit-for-bit reproducibility is unverified (spec 09 E3).
82. **Engine and metric fixes (spec 04), 11 Nov 2026.** Tests in
    `crates/loopsettle-core/tests/spec04.rs`, each failing before its fix. (a) `retry_gross`
    released one payment per edge per window and did not revisit payers funded later in the same
    pass; now it runs to a fixed point. (b) `DeferredNet` cycle `k` is at `floor(k·day/c)`, so the
    last cycle of a day is exactly at its end. (c) The share settled within 2.8 s is counted
    exactly (the histogram counted delays up to 2,815 ms). (d) Strict per-payer FIFO as an
    option. (e) Checked sums in the ledger. **Effect on results:** with unlimited liquidity every
    CSV is identical except the column `share_within_2800ms` (changes below 0.001, e.g. 0.4252 →
    0.4245; claim 7 is unaffected); `microcycle-sweep/shortfall.csv` changed in the
    `micro_cycle_shortfall` rows (rejected value at 25% of UB 23.2275% → 23.2270%; p95 delay
    920 → 1,216 ms). The defect was real but its effect on the published numbers is negligible.
    **Not done:** the incremental cycle search (C5). Measured cost of the micro-cycle engine:
    about 0.6 s per million payments (10 million payments: 26.1 s against 19.8 s for gross, CSV
    parsing included), so it is not the bottleneck; the rewrite must keep the exact cycle order
    and is left for when scale runs need it.
83. **Reproducibility, statistics and CI (spec 09), 11 Nov 2026.** (a) `results-golden/` and
    `scripts/golden.py`: a small run of every experiment, compared byte for byte in CI on Linux,
    Windows and macOS, and with 1 and 2 threads. First run: identical on the three platforms
    ([run 43061345632](https://github.com/fabiodrneles/loopsettle/actions/runs/43061345632)), so
    the cross-platform doubt of the audit (A11) is settled for the golden scenarios. The check was
    confirmed to fail on a changed file. (b) CI also runs clippy with `-D warnings`, the wasm build
    and smoke test, and the bilingual-document check (`scripts/check_twins.py`). (c)
    `crates/loopsettle-sim/tests/metamorphic.rs`: scaling amounts, relabelling participants,
    shifting time, and 2,000 garbage inputs to the TOML and CSV readers (no panic found). (d)
    Seeds and cases run in parallel (`stats::par_map`, 2 threads by default,
    `LOOPSETTLE_THREADS`); output order is fixed, results identical with 1, 2 and 4 threads;
    `robustness` went from 118 s to 75 s on 4 cores. (e) `summary.json` records a schema version,
    platform, rustc and a hash of `Cargo.lock`. **Not done:** `cargo fmt --check` (the code is
    not rustfmt-formatted: 295 diffs with the default width; reformatting is a separate, noisy
    commit to be decided by the owner); several seeds in `baseline`, `potential-vs-realizable`,
    `microcycle-sweep`, `stress`, `heterogeneity`, `funding-windows` and `regimes` (the helper
    exists, each experiment's tables still need their CI columns); the naive reference simulator
    for differential tests; `results/MANIFEST.sha256`.
84. **Constrained liquidity (spec 06) and the queue engine (spec 05, M1), 11 Nov 2026.**
    `experiments6.rs`, `results/liquidity-frontier/`, CLAIMS 1h, D24, METHODOLOGY §11. Result at
    equal service (1% of value rejected, rule `ub`, 10 seeds): 10 s designs save 0.01% of
    liquidity (CIs include zero) in every scenario; one-hour designs save 0.4–0.6% (retail,
    σ = 1.0), 1.1–2.3% (retail, σ = 0.4) and 2.8–4.8% (stylised large-value). **Risks:** (a) the
    equivalent `α` near 1 is interpolated between 0.95 and 1.0, where gross rejection falls from
    about 4% to exactly 0: a finer grid there would make the 1% target more precise; (b) rule
    `ub` gives every participant exactly its own peak, so α = 1 is a knife edge; rule `outflow`
    is reported in `frontier.csv` but not in the summary's equivalent-liquidity table (few seeds
    reach low targets within α ≤ 2); (c) no retries; (d) the large-value system is not
    calibrated, so the 2.8–4.8% is a direction, not an estimate, and is far from the 14–30% of
    McMahon et al.; (e) a wrong assumption of mine was caught by a test: a 10 s queue does *not*
    reject less than gross (it rejects 0.08% more value in the small test scenario), which is the
    same mechanism as the main result. **Performance:** the gross retry was rewritten to visit
    only credited participants (it took more than 10 minutes with one-hour deadlines; now the
    whole experiment takes 4 min 46 s); the literal version stays as `reference_retry` and the two
    give identical settlements in 30,000 random cases. This replaces item C5 of spec 04 for the
    retry; the cycle search itself is unchanged.
85. **Offline optimum and the sandwich (spec 03), 11 Nov 2026.** `tests/common/optimum.rs`,
    `tests/optimum.rs`, example `optimality_gap`, CLAIMS 1i, D25, METHODOLOGY §9c. The LP optimum
    equals the sum of the floors in 116 of 120 dense small instances; the best engine realises
    44–61% of it. **Risks:** (a) the LP lets payments be split and batches be arbitrary, so it
    bounds the whole-payment optimum from below; the brute-force integer check of the spec (FR2)
    was not written; (b) instances are small (20 participants, 1,500 payments) and not
    calibrated; a 300 s deadline did not finish within 9 minutes, so the claim covers deadlines up
    to 60 s; (c) "floors = optimum" is an empirical regularity, not a theorem: a random test finds
    strict gaps in 4% of tiny instances. A proof of when equality holds would be a contribution;
    (d) `results/optimality-gap/` is not part of the golden check (LP solutions in floating point
    are not guaranteed identical across platforms; the files keep 2 to 4 decimals).
86. **Liquidity frontier rerun with a finer grid, 12 Nov 2026.** Risk (a) of point 84 was real: with
    steps of 0.01 between 0.97 and 1 the one-hour savings at 1% rejected are 1.1–1.4% (retail,
    σ = 1.0), 2.2–3.2% (σ = 0.4) and 3.7–5.3% (large-value), up to twice the first run's values.
    The 10 s result did not move (CIs include zero). The numbers of point 84 are superseded;
    CLAIMS 1h carries the new ones and says the measure is grid-sensitive. Cost: the experiment
    now takes 5 min 49 s on 4 cores with 2 threads, above the 5-minute target of spec 06 (it will
    be longer on the reference machine). Also new: `loopsettle bound/settle --report` and private
    per-participant output (spec 10, R2 and R3; D21 updated).
