# Paper outline

> [Versão em português](PAPER_OUTLINE.pt-BR.md)
>
> Rewritten on 5 Nov 2026 around the revised central claim ([`ASSESSMENT.md`](ASSESSMENT.md),
> update 3). Section titles and the figures/tables of each section only; the text is written with
> the reviewer.

**Working title:** *How much liquidity can netting recover in instant payments? A delay bound and
evidence from a calibrated Pix simulator.*

## Preprint (SSRN / arXiv q-fin)

### 1. Introduction
- The question; the four contributions: the delay bound; potential vs realizable, measured robustly;
  the weekend funding window; an open tool that runs on real data.

### 2. Instant payments and their liquidity
- Table 1 — Interbank settlement models of 11 systems ([`INSTANT_PAYMENTS.md`](INSTANT_PAYMENTS.md)).
- 2.1 Pix: SPI, Conta PI, STR window, Conta PI remuneration (BCB SPI report 2024; Pix management report 2023).
- 2.2 Related work: Hellqvist & Korpinen (2021/2026); BCB WP 619; Johnson, McAndrews & Soramäki (2004);
  McMahon et al. (2024); Bank of Canada SDPs 2020-9 and 2022-3; Hellqvist (2024).
- Table 2 — Point-by-point comparison ([`NOVELTY.md`](NOVELTY.md)).

### 3. Model
- 3.1 Participants, payments, ledger; 3.2 generator and calibration
  (Table 3 — parameters and status; Table 4 — real vs simulated, including WP 619 per-bank quartiles);
- 3.3 Settlement engines (gross, deferred netting, atomic micro-cycles); 3.4 liquidity measures
  (UB, LB, funding windows).

### 4. The delay bound
- The liquidity floor: theorem and proof ([`METHODOLOGY.md`](METHODOLOGY.md) §9b), with the window form
  (§9) as a corollary; Figure 1 — realised saving vs tight and window bounds over d
  (`regimes/fig_bound.svg`); Table 5 — the bound over seeds and receiver bias
  (`robustness/seeds_summary.csv`).
- Not in this section any more: "the bound separates retail from large-value". The tight bound is
  ≈ 0 in both (`regimes/regimes.csv`); the comparison with large-value LSM studies needs the
  constrained-liquidity experiment (`specs/06`) and goes to the discussion as an open point until then.

### 5. Results
- 5.1 Potential vs realizable, with seeds, scale and cycle length — Table 6 (`robustness/seeds_summary.csv`),
  Figure 2 (`robustness/fig_scale.svg`).
- 5.2 Calibrated bilateral structure and intraday heterogeneity — Table 7 (`heterogeneity/heterogeneity.csv`).
- 5.3 Funding windows: nights and the weekend — Table 8 (`funding-windows/windows.csv`).
- 5.4 Per-bank heterogeneity, compared with the Finnish study — Table 9 (`robustness/participants_ratio.csv`).
- 5.5 **When the result fails: fast bilateral returns** — Table 10 (`echo-sweep/echo.csv`). The condition of
  the central claim; it belongs in the abstract.
- 5.6 Constrained liquidity at equal service, 10 s vs one hour — Table 11 (`liquidity-frontier/equivalent_liquidity.csv`),
  Figure 3 (`liquidity-frontier/fig_frontier.svg`).
- 5.7 Floors vs the offline optimum — Table 12 (`optimality-gap/gap.csv`).

### 6. Discussion
- Policy: second-scale LSMs in instant systems; where liquidity actually binds (weekends); who bears
  it (payment institutions without access to reserves or rediscount); what a central bank can measure
  with `loopsettle bound`.
- Limitations ([`LIMITATIONS.md`](LIMITATIONS.md)); what cannot be claimed ([`CLAIMS.md`](CLAIMS.md)).

### 7. Conclusion

### Appendices
- A. Invariants and property tests; B. Reproducibility (`make all`); C. Circularity lenses (Hodge, CDFD,
  cycles) and the Hodge null value; D. Running on real data.

## JOSS paper (software)
- Summary; Statement of need (open, reproducible instant-payment settlement simulator, runs on real
  files and in the browser); Functionality (engines, bound, experiments, CLI, web UI); Quality
  (property tests, CI, byte-for-byte reproducibility); References.
