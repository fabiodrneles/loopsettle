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
    time, or an outage of a net receiver.
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
