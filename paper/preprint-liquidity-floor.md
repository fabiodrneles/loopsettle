# The liquidity floor: how much liquidity can any delay-bounded mechanism save in an instant payment system?

**Fabio D. Dorneles**

*Draft for the author and the reviewer (13 Nov 2026). Not for circulation. Every number comes from
a command in the LoopSettle repository (section 8); the places that need the authors' judgement are
marked **[TODO]**. Written in English for SSRN and the Journal of Financial Market Infrastructures;
the decisions and the evidence behind each statement are in `docs/CLAIMS.md`, `DECISIONS.md` and
`REVIEW.md`.*

---

## Abstract

Instant payment systems settle each payment gross, at once, and therefore need more intraday
liquidity than systems that net. A natural idea is to recover part of it with mechanisms that hold
payments for a few seconds and offset them: queues, gridlock resolution, netting cycles. How much
can such a mechanism save, at most? We give a closed-form answer that holds for **any** mechanism
in which every payment settles within a deadline *d*: the **liquidity floor** of each participant,
`floor_i(d) = max(0, max_s [CO_i(s) − CI_i(s + d)])`, where `CO_i(s)` is what it has paid by time
*s* and `CI_i(s + d)` what it has been paid by *d* later. The floor equals the classic upper bound of
Koponen and Soramäki (1998) at *d* = 0 and their lower bound when *d* covers the day; it is attained
for each participant on its own; and on dense small systems the sum of the floors equals the
offline optimum of the whole system in 116 of 120 instances. It needs two streaming passes over a
payment file, so a central bank can compute it on its own data. On a synthetic Pix scenario
calibrated to public data, the floor for *d* = 10 s leaves at most 0.028% of the gross-settlement
liquidity to save (95% CI [0.016; 0.041]), and existing mechanisms realise about half of that. The
result is conditional: if a sizeable share of payments returns to the payer within seconds, the
ceiling rises to several percent. **[TODO: the reviewer decides whether the abstract leads with the
theorem or with the Pix number.]**

**Keywords:** instant payments, intraday liquidity, liquidity-saving mechanisms, payment system
simulation, netting, Pix. **JEL:** E42, E58, G21.

---

## 1. Introduction

**[TODO: opening paragraph by the author: why liquidity in instant payments matters to central
banks and to the participants that hold it, with the Pix numbers of the SPI annual report 2025.]**

Brazil's Pix settled 7.10 billion interbank payments in December 2025 (Banco Central do Brasil,
*Relatório Anual do SPI 2025*), each one gross and final within seconds, against balances held in
dedicated accounts at the central bank. Gross settlement is safe and fast, but every participant
must hold enough to cover the largest gap between what it pays and what it receives during the day.
Liquidity-saving mechanisms (LSMs) in large-value systems reduce that need by queueing payments and
settling them in offsetting batches; the classic simulation studies of the Bank of Finland measured
those savings (Koponen and Soramäki 1998; Bech and Soramäki 2001). Instant payments impose a hard
constraint the large-value literature did not: a payment may wait seconds, not hours.

This paper asks a question that does not depend on the design of the mechanism: **given a
deadline *d*, how much liquidity can any mechanism that settles every payment within *d* save?**

**Contributions.**

1. **A closed-form bound for any delay-bounded mechanism** (section 4). The liquidity floor of each
   participant is the least liquidity any such mechanism can need; it interpolates between the
   upper and lower bounds of Koponen and Soramäki (1998), which until now were joined only by
   simulation.
2. **Tightness.** The floor is attained for one participant on its own (by holding its payments for
   exactly *d*); and the sum of the floors coincides with the offline optimum of the whole system in
   almost every dense instance tested (section 5).
3. **A computation a central bank can run.** Two streaming passes over a payment file, memory
   proportional to the number of participants (section 5). It is implemented in an open simulator
   that runs on the user's own data without the data leaving the machine.
4. **Evidence for Pix and its condition** (section 6). On a calibrated synthetic scenario the floor
   leaves almost nothing to second-scale mechanisms; the statement fails when payments are returned
   to the payer within the deadline, which identifies the statistic that decides the question.

## 2. Related work

**Bounds on intraday liquidity.** Koponen and Soramäki (1998, §2.3) define the theoretical upper
bound (the largest cumulative net debit under gross settlement without queuing, their equation 2)
and lower bound (the end-of-day net debit, equation 1) of a bank's liquidity need. Between the two,
the trade-off between liquidity and settlement delay is obtained by simulation at liquidity levels
`LB + LL (UB − LB)` (their equation 4; also Bech and Soramäki 2001, equation 8). Rivadeneyra and
Zhang (Bank of Canada SDP 2020-9) compute the minimum liquidity "allowing for the maximum delay",
where delay means postponement to the end of the day, by iterative simulation. We did not find a
bound parameterised by a deadline in these sources or in a limited search of the later literature
(`docs/REFERENCES.md`); **[TODO: complete the search in the proceedings of the Bank of Finland
simulator seminar and in JFMI before the word "first" is used; meanwhile "to our knowledge".]**

**Liquidity-saving mechanisms.** Bech and Soramäki (2001) give the gridlock-resolution algorithm
that settles the largest queued set respecting each bank's FIFO order; Johnson, McAndrews and
Soramäki (2004) measure deferred-settlement mechanisms on Fedwire data; McMahon et al. (2024) study
LSMs for large-value systems against a FIFO queue under constrained liquidity.

**Instant payments and liquidity.** Hellqvist and Korpinen (2021) compare the liquidity of a
cycle-based and an instant system with calibrated synthetic data; Gonzalez, Ma and Zeng (2025,
BCB WP 619) measure the netting lost by Pix and its *potential* capacity per bank.

**Analogy.** The floor is the payment-system counterpart of a backlog bound in network calculus
(Le Boudec and Thiran 2001), with outflows in the role of arrivals and delayed inflows in the role
of service. **[TODO: the reviewer judges whether this analogy helps or distracts.]**

## 3. Setting

Participants `i = 1, …, n` hold settlement accounts. A payment is `(a, i, j, v)`: arrival time *a*,
payer *i*, payee *j ≠ i*, value *v* > 0 in cents. Over a period `[0, T]` define, for participant
*i*, the cumulative values paid and received by time *s*, counted at arrival:

```text
CO_i(s) = Σ { v : payments of i with a ≤ s }        CI_i(s) = Σ { v : payments to i with a ≤ s }
```

A **settlement mechanism** decides when each payment settles; settlement moves value from payer to
payee and may be simultaneous for several payments (batches, netting). It is **delay-bounded by
*d*** if every payment arriving at *a* settles in `[a, a + d]`. Let `y_i(t)` be *i*'s cumulative
settled position (received minus paid) under the mechanism. The liquidity the mechanism needs from
*i* is `L_i = max(0, −min_t y_i(t))`, the smallest opening balance that keeps *i* non-negative.

Gross settlement on arrival (`d = 0`) gives `y_i(t) = CI_i(t) − CO_i(t)` and
`UB_i = max(0, max_t [CO_i(t) − CI_i(t)])`, the upper bound of Koponen and Soramäki (1998). One
netting at the end of the period gives `LB_i = max(0, CO_i(T) − CI_i(T))`, their lower bound.

## 4. The liquidity floor

**Definition.** For a deadline `d ≥ 0`,

```text
floor_i(d) = max(0, max_{s ∈ [0, T]} [CO_i(s) − CI_i(min(s + d, T))]).
```

**Theorem 1 (bound).** For every mechanism delay-bounded by *d* and every participant *i*,
`L_i ≥ floor_i(d)`.

*Proof.* Fix *s*. At time `s + d` (or *T*), every payment of *i* that arrived by *s* has settled, so
*i* has paid at least `CO_i(s)`. No payment settles before it arrives, so *i* has received at most
`CI_i(s + d)`. Hence `−y_i(s + d) ≥ CO_i(s) − CI_i(s + d)`, and `L_i ≥ −y_i(s + d)`. Take the
maximum over *s*. ∎

**Properties.**

1. `floor_i(0) = UB_i` and `floor_i(d) = LB_i` for `d ≥ T` (since `CO_i` is non-decreasing, the
   maximum over *s* of `CO_i(s) − CI_i(T)` is reached at `s = T`).
2. `floor_i(d)` is non-increasing in *d* (`CI_i` is non-decreasing).
3. **Window form (corollary).** Let `t*_i` be the first time *i* reaches its gross-settlement
   peak. Taking `s = t*_i − d` gives `UB_i − floor_i(d) ≤ CO_i(t*_i) − CO_i(t*_i − d)`: no
   mechanism saves more than what *i* paid in the *d* before its peak. This weaker form is easier to
   explain and much looser in practice (section 6).

**Theorem 2 (attainment for one participant).** For each *i* there is a mechanism delay-bounded by
*d* with `L_i = floor_i(d)`: settle every payment of *i* exactly *d* after its arrival and every
payment to *i* on arrival.

*Proof.* Under that mechanism `−y_i(t) = CO_i(t − d) − CI_i(t)`, whose maximum over *t* is
`floor_i(d)` (substitute `s = t − d`). ∎

Theorem 2 is checked to the cent in the simulator (test `a_single_holder_needs_exactly_its_floor`).
Attainment for all participants at once is not guaranteed: one participant's delayed payment is
another's delayed receipt. The exact system-wide question is a linear programme (all payments
known in advance; any split of each payment over `[a, a + d]`); its value `LP(d)` satisfies

```text
Σ_i floor_i(d)  ≤  LP(d)  ≤  optimum with whole payments  ≤  Σ_i L_i (any mechanism with deadline d).
```

**[TODO: decide whether to present the LP formally in an appendix; it is in
`crates/loopsettle-sim/tests/common/optimum.rs` and METHODOLOGY §9c.]**

## 5. Tightness and computation

**The sandwich in practice.** On dense small systems (20 participants, about 1,500 payments in one
hour, deadlines of 2, 10 and 60 s, 20 seeds), the offline optimum equals the sum of the floors in
116 of 120 instances and is within 0.3% in the other four; the best of the simulator's engines
realises 44–61% of it (`results/optimality-gap/gap.csv`). On 900 small random instances, 37 show a
strict gap, so the equality is empirical, not general. The slack between engines and the bound is
therefore in the engines, not in the bound.

**Computation.** `floor_i(d)` needs, for each *s*, the cumulative inflow at `s + d`. Two passes over
a time-ordered file, one reading inflows *d* ahead of the outflows, give all floors in O(N) time and
O(n) memory for *N* payments and *n* participants (`loopsettle_sim::bound::liquidity_floors`). The
command `loopsettle bound <file>` reports UB, LB and the floor for a list of deadlines; 50 million
payments were processed under 1 GB of memory (REVIEW 94).

## 6. Evidence: a calibrated Pix scenario

**Data.** No interbank payment data are public. The scenario is synthetic: 200 participants with
sizes from the public Pix key counts per institution, Poisson arrivals with a daily profile, values
fitted to the public mean ticket and to the p50, p90 and p99 of the SPI annual report 2025 (D30),
interbank volume from the Olinda open-data API, and a receiver imbalance fitted to the per-bank
quartiles of BCB WP 619 (σ = 0.4; `docs/PARAMETERS.md`). Every number below is the mean over 10
seeds with a 95% confidence interval.

**Table 1 — The 10 s bound and what mechanisms realise** (`results/robustness/seeds_summary.csv`,
1 million payments a day).

| Receiver imbalance σ | Floor-based ceiling, % of UB | Window form, % of UB | Netting every 10 s realises | Micro-cycles (10 s) save, % of UB |
|---|---|---|---|---|
| 0 | 0.075 [0.044; 0.107] | 3.14 | 54% | 0.011 [0.001; 0.021] |
| 0.4 (fitted) | 0.028 [0.016; 0.041] | 1.30 | 46% | 0.002 [−0.009; 0.012] |
| 1.0 | 0.010 [0.006; 0.014] | 0.78 | 50% | 0.001 [−0.004; 0.006] |

**Figure 1 — The bound over the deadline** (`results/regimes/fig_bound.svg`): realised saving, the
tight ceiling and the window form from 10 s to one day. Netting realises 47% of the ceiling at 10 s
[37; 57], 28% at 15 minutes and 63% at six hours; at one day the ceiling equals UB − LB.

**Scale.** At 30 million payments a day the 10 s ceiling is 0.033% of UB (one seed;
`results/robustness/scale.csv`); with class-specific intraday profiles it stays ≤ 0.037%
(`results/heterogeneity/`). Longer horizons are different: one end-of-day netting saves 14.4% of UB
at 1 million payments a day and 4.0–9.2% at 30 million.

**Where the result fails.** The ceiling depends on how fast payments are reciprocated. When 20% of
payments are returned to the payer about one second after they are received, the 10 s ceiling rises
to 7.6% of UB [4.3; 10.8] and netting every 10 s realises 7.1%; the effect falls with volume (0.99%
at 30 million payments a day) and when only part of the amount returns
(`results/echo-sweep/`, `results/echo-scale/`). Echoes that go onwards to third parties change
nothing. A pre-registered criterion (ceiling above 2% for an echo share of at most 20%) fired, so the
claim is stated as conditional. **The share of value reciprocated bilaterally within the deadline is
the statistic that decides the question, and the floor measures its effect directly on a file.**

**Comparison with the large-value literature.** With the Bank of Finland design (balances
`LB + level × (UB − LB)`, queues until the end of the day, delay indicator ρ), the Bech–Soramäki
algorithm cuts ρ by 30% at the lower bound in a stylised large-value system with 1,500 payments a
day, as their paper reports, but by at most 2.5% (not significant) in the Pix scenario
(`results/bof-replication/`).

## 7. Discussion

**For central banks.** The floor turns a design debate into a measurement: before building a
second-scale LSM, compute `floor(d)` on a day of real payments. If `UB − Σ floor(d)` is a small
share of UB, no design with that deadline can be worth it. **[TODO: the author writes the policy
paragraph for Pix: Conta PI, STR windows, payment institutions without access to reserves.]**

**For research.** The floor separates two questions that simulation studies usually mix: how much
the deadline allows (a property of the flows) and how much a given algorithm achieves (a property of
the design).

## 8. Limitations

- The scenario is synthetic; only aggregates are public. The bilateral structure (who pays whom) is
  an assumption, and the fit of the receiver imbalance depends on the value model (`docs/LIMITATIONS.md`, 19).
- Arrivals are stationary apart from the daily profile; fast reciprocity is studied as a stress test,
  not measured.
- The system-wide equality of floor and optimum is empirical, on small dense instances; the linear
  programme does not scale to Pix volumes.
- Liquidity is measured with unlimited balances; behaviour (participants changing when they pay) is
  not modelled.

## 9. Conclusion

**[TODO: the author writes it after the reviewer's pass.]**

## Code and data availability

All results are produced by `cargo run --release -p loopsettle-cli -- experiment <name>` in the
LoopSettle repository, Apache-2.0, archived on Zenodo **[TODO: DOI after the first release]**.
Public inputs: Banco Central do Brasil Olinda open-data API (Pix statistics, September 2026) and the
*Relatório Anual do SPI 2025*. No confidential data were used.

## References

- Banco Central do Brasil (2026). *Relatório Anual do SPI 2025*.
- Bech, M. L. and Soramäki, K. (2001). Gridlock resolution in interbank payment systems. Bank of
  Finland Discussion Papers 9/2001.
- Gonzalez, R. B., Ma, Y. and Zeng, Y. (2025). The effect of instant payments on the banking system.
  Banco Central do Brasil Working Paper 619. **[TODO: confirm the series number in the bibliography.]**
- Hellqvist, M. and Korpinen, K. (2021). Instant payments as a new normal: case study of liquidity
  impacts for the Finnish market. Bank of Finland.
- Johnson, K., McAndrews, J. J. and Soramäki, K. (2004). Economizing on liquidity with deferred
  settlement mechanisms. FRBNY Economic Policy Review 10(3), 51–72.
- Koponen, R. and Soramäki, K. (1998). Intraday liquidity needs in a modern interbank payment
  system: a simulation approach. Bank of Finland Studies E:14.
- Le Boudec, J.-Y. and Thiran, P. (2001). *Network Calculus*. Lecture Notes in Computer Science 2050.
  Springer.
- McMahon, C. et al. (2024). Improving the efficiency of payments systems using quantum computing.
- Rivadeneyra, F. and Zhang, N. (2020). Bank of Canada Staff Discussion Paper 2020-9. **[TODO: full
  title from `docs/references/`.]**
