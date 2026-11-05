# CLAIMS — what the study can and cannot claim

> [Versão em português](CLAIMS.pt-BR.md)

Evidence = a file in `results/` produced by a command (see
[`results/SUMMARY.md`](../results/SUMMARY.md)).

## Can claim (with the evidence shown)

| # | Claim | Evidence | Condition |
|---|---|---|---|
| 1 | Atomic micro-cycles of up to 10 s save about 0.01% of the liquidity need: 0.0116% of UB, 95% CI [0.0079; 0.0152]% (10 seeds, σ = 0.4); means ≤ 0.018% across σ ∈ [0; 0.8]; it falls to ≤ 0 as volume grows to 30M payments/day; it does not depend on the maximum cycle length (2 to 10). The BCB-style potential volume measure (85% of volume) measures something else and must not be compared directly with a liquidity saving. | `robustness/seeds_summary.csv`, `robustness/scale.csv`, `robustness/cycle_len.csv` | synthetic scenario, stationary arrivals |
| 1b | No mechanism that settles each payment within d can save more than the value each participant pays in the d before its gross-settlement peak (delay bound). For d = 10 s this ceiling is 0.27–0.80% of UB in every configuration tested. | theorem in `METHODOLOGY.md` §9; test `engines_never_beat_the_delay_bound`; `robustness/scale.csv` | general for the bound; numbers for the synthetic scenario |
| 1d | With the receiver bias calibrated to BCB WP 619 (σ = 1.0), the saving of one end-of-day netting at 30M payments/day is between 0.24% of UB (no intraday heterogeneity) and 2.8% (full class profiles); 15-minute netting saves 0.05–0.09%; second-scale netting ≈ 0. The 10 s delay bound stays ≤ 0.46% in every heterogeneity case (kill criterion K1 not fired). | `heterogeneity/heterogeneity.csv`, `bias-calibration/grid.csv` | class profile shapes are an ASSUMPTION (D18) |
| 1e | With the Conta PI's funding windows (top-ups only while the STR is open), the weekend is the binding prefunding window: about 2.7× a day's peak drawdown, and one netting at the end of the weekend would save only 0.4–1.0% of it (calibrated σ, 10M payments per Thursday); the 10 s bound inside it is ≤ 0.23%. | `funding-windows/windows.csv` | sourced hourly profile and day volumes; bilateral structure synthetic |
| 1f | The delay bound depends on how lumpy payments are relative to liquidity needs: for 10 s it is 0.76% of UB in the retail scenario but 13.7% in a stylised large-value system with 2,000 payments a day; so it does not contradict large LSM savings reported for large-value systems. Across delays from 10 s to one day, realised netting saves a stable 3–5% of the bound. | `regimes/regimes.csv`, `regimes/bound_curve.csv` | large-value scenario is an ASSUMPTION |
| 1c | Under stationary arrivals, the saving of one end-of-day netting shrinks as 1/√(volume): 9.27% of UB at 1M payments/day, 1.68% at 30M. Potential measures computed on small samples overstate it. | `robustness/scale.csv` | stationary arrivals (D8, D9) |
| 2 | Settling a lot of value inside cycles (up to 63%) does not imply saving liquidity: the liquidity peak is driven by imbalances lasting hours. | `microcycle-sweep/sweep.csv`; `robustness/seeds_summary.csv` (σ = 0.4, 10 seeds: 0.019% every 10 s, 0.50% every 15 min, 7.8% once a day) | same |
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
| 10 | "Potential liquidity saving is 9.27%" or "realizable is 630× smaller than potential". | The potential depends on scale (9.27% at 1M payments/day, 1.68% at 30M) and on the seed (σ = 0.4: CI [4.5; 11.1]%); the ratio is not a stable number (`robustness/`). |
| 11 | Any single-seed number as a result. | Use the 10-seed means and CIs of `robustness/seeds_summary.csv`. |
| 12 | "The Hodge curl share shows that X% of the flow is circular." | On the baseline net graph the random-flow null value is (m − n + 1)/m = 0.970 against 0.977 observed (`METHODOLOGY.md` §9). |
| 13 | "LoopSettle reproduces / refutes McMahon et al. (2024)." | Different systems (LVTS and T2, large-value) and, for T2, a different baseline (FIFO with queues under constrained liquidity). The LVTS paper's measure (aggregate mNDP) equals our UB, but on real large-value data we do not have; `regimes` only shows the bound is not restrictive in lumpy flows. |
| 9 | "FedNow / TIPS / UPI would save X% of liquidity." | The scenario is calibrated to Pix only; other systems need their own public aggregates. The engines and the qualitative mechanism apply ([`INSTANT_PAYMENTS.md`](INSTANT_PAYMENTS.md)); the numbers do not carry over. |
