# CLAIMS — what the study can and cannot claim

> [Versão em português](CLAIMS.pt-BR.md)

Evidence = a file in `results/` produced by a command (see
[`results/SUMMARY.md`](../results/SUMMARY.md)).

## Can claim (with the evidence shown)

| # | Claim | Evidence | Condition |
|---|---|---|---|
| 1 | In the calibrated synthetic scenario, the **potential** liquidity saving (one end-of-day netting: 9.27% of UB) is ~630× the **realizable** saving with atomic micro-cycles of up to 10 s (0.0147% of UB). The BCB-style potential volume measure (85% of volume) measures something else and must not be compared directly with a liquidity saving. | `potential-vs-realizable/central.csv` | synthetic scenario; holds for σ ∈ [0; 0.8] (`baseline/sensitivity.csv`) |
| 2 | Settling a lot of value inside cycles (up to 63%) does not imply saving liquidity: the liquidity peak is driven by imbalances lasting hours. | `microcycle-sweep/sweep.csv`; `potential-vs-realizable/engines.csv` (saving grows with the interval: 0.03% every 10 s, 0.32% every 15 min, 9.27% once a day) | same |
| 3 | The liquidity saving of deferred netting grows monotonically with the interval between cycles, and only one daily netting recovers the whole UB−LB gap. | `potential-vs-realizable/engines.csv` | no intraday opening balances |
| 4 | The three circularity lenses do not coincide: Hodge (diagnostic) > maximum CDFD > greedy netting, with a large gap on the net graph (0.98 / 0.17 / 0.14). | `baseline/ladder.csv`; triangle test | Hodge ≥ CDFD observed, not proven |
| 5 | The Hodge decomposition does not respect edge bounds and is not a nettable amount (triangle: 6.67 vs 5). | test `triangle_case` | general (mathematical) |
| 6 | The maximum CDFD can be computed exactly, without LP, by min-cost flow; the result equals an LP solver's on 1,000 graphs. | test `cdfd_matches_lp_oracle_on_1000_random_graphs` | general |
| 7 | Micro-cycles with a maximum delay ≥ 5 s miss the target of 50% settled within 2.8 s; with 2 s they meet it. | `microcycle-sweep/sweep.csv` | the measured delay is only the engine's |
| 8 | Under the Finnish study's logic, the instant system needs 1.1–1.5× the liquidity of one daily netting, and the gap falls to 1.01–1.03× with 48 cycles a day. | `finland-replication/ratios.csv` | depends on the bias σ |
| 9 | The simulator is bit-for-bit reproducible and runs on modest hardware (4th-gen i3, 4 GB). | invariant 7; `make rebuild` in 120 s; peak < 20 MiB per experiment | — |

## Cannot claim

| # | Forbidden claim | Why |
|---|---|---|
| 1 | "Pix could save R$ X billion of liquidity." | Synthetic data; absolute liquidity does not scale linearly with the factor 239; LB/UB depends on an assumption (σ). |
| 2 | "X% of Pix liquidity is circular." | We only have public aggregates; the bilateral structure is an assumption. |
| 3 | "Micro-cycles do not work in real Pix." | 239× smaller scale per window; simple heuristic; not tested with real per-second volume. |
| 4 | "The BCB study overstates netting capacity." | We did not reproduce the BCB data; we reproduce the *logic* of the measure on synthetic data. |
| 5 | "We replicated Hellqvist & Korpinen." | We applied their logic to another scenario; the comparison is of orders of magnitude only. |
| 6 | "The Hodge potential × net position correlation validates the model." | On a dense graph it is almost mechanical. |
| 7 | "A large participant going offline does not affect liquidity." | Weak stress scenario (full resubmission, peak at the end of the day). |
| 8 | Any claim about the proportional CDFD variant or about on-chain data. | Not implemented / not run. |
