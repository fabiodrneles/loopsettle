# NOVELTY — what LoopSettle does relative to the reference works

> [Versão em português](NOVELTY.pt-BR.md)
>
> For the paper's contribution section. The columns for the other works summarise what
> `CLAUDE.md` says about them; **the reviewer must check every cell against the sources** before
> publishing (cells marked "to check" depend on details we have not verified).
>
> Verification status of each reference: [`REFERENCES.md`](REFERENCES.md). PixSim
> (arXiv 2609.30684) **was not found** in public searches; do not cite it until confirmed.

| Dimension | LoopSettle | Hellqvist & Korpinen (BoF 2021; JFMI 2026) | BCB WP 619 (Gonzalez, Ma & Zeng, 2025) | CDFD (Homs-Dones et al., arXiv 2506.12546) | BoF-PSS | PSSimPy | PixSim (arXiv 2609.30684, **not found**) |
|---|---|---|---|---|---|---|---|
| Object | Circular liquidity in instant payments | Liquidity: cycles vs instant | Loss of netting in Pix | Flow decomposition in networks | RTGS/LSM simulator | Payment system simulator (to check) | Pix simulator |
| Data | Synthetic, calibrated to public Pix aggregates (Sep 2026), every parameter sourced or `ASSUMPTION` | Synthetic, calibrated | BCB data (to check: confidential) | Networks in general | User's data | (to check) | Sourced calibration per parameter |
| Netting measure | Three lenses side by side: Hodge, max CDFD, cycles | Liquidity by model | **Potential** capacity (no time order) | Circular component | LSM/gridlock algorithms | (to check) | (to check) |
| Potential **vs realizable** over time | **Yes** — same network, from 1×/day to 10 s and micro-cycles | Partial (fixed cycles) | No (potential only) | No (static) | Realizable, no potential measure | (to check) | (to check) |
| New mechanism | **Atomic micro-cycles** with maximum delay and "shortfall only" mode | — | — | — | Classic LSMs | — | — |
| Diagnostic → execution ladder | **Yes** (Hodge ≥ CDFD ≥ cycles ≥ micro-cycles, with gaps) | No | No | CDFD vs other decompositions (to check) | No | No | No |
| Exact CDFD without LP | **Yes** (min-cost flow, validated against LP) | — | — | Own algorithm (to check) | — | — | — |
| Property-tested invariants | **Yes** (8 invariants, proptest) | — | — | — | (to check) | (to check) | (to check) |
| Reproducibility | Seed + code version in every result; `make rebuild` redoes everything | (to check) | No (restricted data) | (to check) | Tool, not a study | (to check) | (to check) |
| Runs in the browser | **Yes** (wasm + Web Worker) | No | No | No | No (desktop) | No | (to check) |
| Hardware | 4th-gen i3, 4 GB; < 20 MiB per experiment | — | — | — | — | — | — |
| Licence | Apache-2.0 | paper | paper | paper | (to check) | (to check) | (to check) |

## Proposed contributions (to validate with the reviewer)

1. A **realizable** measure of liquidity saving over time, side by side with the **potential**
   measure used in the BCB study, in the same scenario.
2. The **ladder** of three circularity lenses, with the triangle case as the minimal example of
   why Hodge is not a nettable amount.
3. The **atomic micro-cycle** mechanism and its saving × delay trade-off curve, compared with
   the 2.8 s target of the Pix timing manual.
4. An **open, reproducible and lightweight** simulator calibrated to public Pix data.

## Closest works found after the first draft (5 Nov 2026)

| Work | What it does | What LoopSettle adds |
|---|---|---|
| McMahon et al. (2024), payment reordering in T2 | Optimised reordering in short windows (most delays < 5 s) saves 14–30% of liquidity vs FIFO in a **large-value** system | The **retail instant** case, a **model-free bound** that explains why the same idea saves ~0 there and can save a lot in lumpy large-value flows (`results/regimes/`: 10 s bound 0.76% retail vs 13.7% with 2,000 large payments/day), and an open implementation |
| Hellqvist (2024), instant payment liquidity risk | Outage and contagion risk in Finnish instant payments with constrained liquidity | Liquidity *saving* rather than risk; the weekend funding window; a calibration target from real Pix data (WP 619) |
| Bank of Canada SDPs 2020-9, 2022-3 | LSM liquidity efficiency in Lynx (large-value) with real LVTS data | Instant retail; the delay bound; a tool other central banks can run on their own data (`loopsettle bound`) |
