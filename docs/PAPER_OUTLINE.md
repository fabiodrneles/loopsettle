# Paper outline

> 🇧🇷 [Versão em português](PAPER_OUTLINE.pt-BR.md)

Section titles and the list of figures/tables in each section only. The text will be written
with the reviewer.

## 1. Introduction
- Table 1 — Research question and the three measures (potential, static, realizable).

## 2. Related work
- Table 2 — Point-by-point comparison ([`NOVELTY.md`](NOVELTY.md)).

## 3. Model
### 3.1 Participants, payments and ledger
### 3.2 Settlement engines (gross, deferred netting, atomic micro-cycles)
- Figure 1 — Diagram of the atomic micro-cycle (queue per edge, cycle, deadline).
### 3.3 Liquidity needed: UB, LB and peak balance drawdown
- Table 3 — Worked example (A, B, C) from [`METHODOLOGY.md`](METHODOLOGY.md).
### 3.4 Three lenses on circularity
- Figure 2 — Triangle case: Hodge 6.67 vs CDFD/cycles 5.

## 4. Data and calibration
- Table 4 — Parameters and status ([`PARAMETERS.md`](PARAMETERS.md)).
- Table 5 — Real vs simulated (`results/calibration/calibration.csv`).

## 5. Implementation and validation
- Table 6 — Invariants and property tests.
- Table 7 — Performance on the reference hardware (`CHECKPOINT.md`).

## 6. Results
### 6.1 Liquidity and circularity in the calibrated scenario
- Table 8 — UB, LB, potential (`results/baseline/summary.json`).
- Figure 3 — Circularity ladder (`results/baseline/fig_ladder.svg`).
### 6.2 Potential vs realizable
- Figure 4 — Saving × maximum wait (`results/potential-vs-realizable/fig_potential_vs_realizable.svg`).
- Table 9 — Central measures (`results/potential-vs-realizable/central.csv`).
### 6.3 Micro-cycles: saving × delay
- Figure 5 — Trade-off curve (`results/microcycle-sweep/fig_tradeoff.svg`).
- Table 10 — Constrained liquidity (`results/microcycle-sweep/shortfall.csv`).
### 6.4 Fixed cycles vs instant (Finnish study logic)
- Figure 6 — Liquidity ratio (`results/finland-replication/fig_finland.svg`).
### 6.5 Stress
- Table 11 — 30-minute outage (`results/stress/stress.csv`).

## 7. Sensitivity and robustness
- Table 12 — Receiver bias (`results/baseline/sensitivity.csv`).
- (To do) Scale of 10–30 million payments; `max_cycle_len`; strict FIFO.

## 8. Limitations
- List from [`LIMITATIONS.md`](LIMITATIONS.md).

## 9. Conclusion

## Appendices
- A. Proof that UB is the smallest balance without rejection; B. CDFD as min-cost
  transshipment; C. Reproducibility (`make rebuild`).
