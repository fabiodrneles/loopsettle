# References — verification status

> [Versão em português](REFERENCES.pt-BR.md)
>
> Basis for the papers' bibliography. Each reference has a **status**: **confirmed** (title,
> authors and year checked against a public source), **partial** (it exists, but some detail
> still has to be checked in the full text) or **not found** (not found in public searches; do
> not cite until confirmed).
>
> Last check: 5 Nov 2026. Sections 1, 2, 3 and 4b were checked against the **full texts** sent by
> the owner (page numbers below are PDF pages). The JFMI 2026 version (section 1, version 2) was
> not read: only the 2021 text.

## 1. Hellqvist & Korpinen — methodological basis

| Field | Value | Status |
|---|---|---|
| Version 1 | Hellqvist, M. & Korpinen, K. (2021). *Instant payments as a new normal: Case study of liquidity impacts for the Finnish market*. Bank of Finland Economics Review 7/2021. | confirmed |
| Version 2 | Hellqvist, M. & Korpinen, K. (2026). *Instant payments as the new normal: how much more money do the banks need?* Journal of Financial Market Infrastructures, 13(2), March 2026. | confirmed |
| Data | Synthetic transaction-level data mimicking normal retail credit transfer flows between Finnish participants in the STEP2 system (EBA Clearing). | confirmed (abstract) |
| Comparison | Liquidity needs with STEP2's **6 settlement cycles** a day (5 daytime + 1 night-time) vs full instant mode; 1,000 synthetic days, Poisson-lognormal transactions matched to STEP2 participant-level monthly statistics (April 2020), two topologies; cycle counts 1–6 also simulated with BoF-PSS3 on 30 days (PDF p. 2, 4–5). | confirmed (full text) |
| Definition | Liquidity need = the smallest funds that avoid any rejection: in instant mode the minimum position over the day (the **upper bound** of RTGS studies); with cycles the minimum among end-of-cycle positions (the **lower bound**, Koponen & Soramäki 1998). Same definition as LoopSettle's UB/LB (D3) (PDF p. 3–4). | confirmed (full text) |
| Aggregate result | System-level daily liquidity need rises by **2.7% on average** and by **less than 8.6–8.7% on 95% of days** (€8.6M average, < €28M at p95, against €324M with 6 cycles). On 30 days: one daily cycle €231M, instant €335M (instant / 1 cycle = 1.45×); 65% of the total increase happens in the first step from one daily netting to two cycles (PDF p. 2, 9–11). | confirmed (full text) |
| Figure quoted | "8.6%" is the **95th percentile**, not a maximum or an average (the average is 2.7%); the baseline is 6 cycles, not one daily netting. | corrected (full text, PDF p. 2) |
| Heterogeneity | Variations between banks and between days can be large even for normal flows; hence the emphasis on liquidity buffers. | confirmed (abstract) |
| Predictability | The size of the increase can be anticipated from *ex ante* known statistics of each bank's payment flow. | confirmed (abstract) |
| Topology | The topology of bilateral flows does not affect the bank-level increase "at least in the setup used in our study". Relative increases are largest for banks whose cycle-mode need is small (PDF p. 2, 10–11). | confirmed (full text) |
| Still open | Exact number of banks (not read in the parts checked) and what changed in the JFMI 2026 version. | pending (minor) |

Sources: [EconStor](https://www.econstor.eu/handle/10419/243289) ·
[RePEc/IDEAS (BoFER 7/2021)](https://ideas.repec.org/p/zbw/bofecr/72021.html) ·
[Risk.net (JFMI)](https://www.risk.net/node/7963364) ·
[PDF BoFER 7/2021](https://publications.bof.fi/bitstream/10024/43594/1/BoFER_7_2021.pdf) ·
[authors' slides](https://www.suomenpankki.fi/globalassets/en/research/seminars-and-conferences/conferences-and-workshops/documents/eopx2021/korpinen_liquidity-impacts-of-migration-to-instant-payments_presentation.pdf) ·
[discussant's slides](https://www.suomenpankki.fi/globalassets/en/research/seminars-and-conferences/conferences-and-workshops/documents/eopx2021/alexandrova-kabadjova_discussion_instant-payments-as-a-new-normal_presentation.pdf)

## 2. The Pix study — BCB Working Paper 619

| Field | Value | Status |
|---|---|---|
| Reference | Gonzalez, R. B., Ma, Y. & Zeng, Y. (2025). *The Effect of Instant Payments on the Banking System*. Banco Central do Brasil, Working Paper Series 619, March 2025. | confirmed |
| Data | Administrative banking data and transaction-level Pix data (not public). | confirmed (abstract) |
| Finding | Instant payments remove banks' ability to delay and net payments; banks hold more liquid assets (mainly government bonds), which reduces credit intermediation. Causal identification with an instrument based on passive payment timeouts. | confirmed (abstract) |
| Measure | Eq. (3.2), per bank: 2·Σ_d min(Outflows, Inflows) / Σ_d (Outflows + Inflows), daily, plotted as quartiles across banks. Median bank **57% to 80%**, p25 18%–45%, p75 73%–90% over the sample (from Nov 2020) (PDF p. 16). LoopSettle's "potential capacity" Σ_i min / Σ_i out is the **value-weighted aggregate** of the same formula; `results/SUMMARY.md` now also reports the per-bank quartiles (p25 63%, median 78%, p75 91%) for a like-for-like comparison. | confirmed (full text) |
| Caveat | The text does not necessarily represent the BCB's official view (the paper's own disclaimer). | confirmed |

Sources: [PDF at the BCB](https://bcb.gov.br/content/publicacoes/WorkingPaperSeries/WP619.pdf) ·
[RePEc](https://econpapers.repec.org/RePEc:bcb:wpaper:619)

## 3. CDFD — circular-directional decomposition

| Field | Value | Status |
|---|---|---|
| Reference | Homs-Dones, M., MacKay, R. S., Sansom, B. & Zhou, Y. (2025). *Circular Directional Flow Decomposition of Networks*. arXiv:2506.12546 [physics.soc-ph]. | confirmed |
| Content | Splits flow into a circular and an acyclic part; circularity index between 0 and 1; two benchmarks: **maximum circularity** (implemented in LoopSettle) and **BFF — Balanced Flow Forwarding** (proportional/local variant, **not** implemented). | confirmed |
| Maximum circularity | Section 5.1: the minimum-cost problem (3) with all costs 1 — minimise Σ d with 0 ≤ d ≤ w and the same net flows. Exactly LoopSettle's formulation (D5) (PDF p. 9–10). | confirmed (full text) |
| BFF | Section 5.2, Algorithm 1: remove sinks, take the maximal invariant flow of the proportions w̄ (fixed point of a_out(t+1) = min(a_out(t) w̄, a_out(t)), Theorem 3), subtract, repeat. Polynomial-time version in Appendix F. Implementable for D6 (PDF p. 10–11). | confirmed (full text) |
| Journal version | An earlier README cited *"How circular is a directed network? A flow decomposition approach"*, *Royal Society Open Science* (2026). | **not found**: cite the arXiv version until confirmed |

Sources: [arXiv](https://arxiv.org/abs/2506.12546) · [WRAP Warwick](https://wrap.warwick.ac.uk/190782)

## 4. Bech & Soramäki — gridlock resolution

| Field | Value | Status |
|---|---|---|
| Reference | Bech, M. L. & Soramäki, K. (2001). *Gridlock Resolution in Interbank Payment Systems*. Bank of Finland Discussion Papers 9/2001. | confirmed (classic reference) |

## 4b. Johnson, McAndrews & Soramäki — deferred settlement mechanisms

| Field | Value | Status |
|---|---|---|
| Reference | Johnson, K., McAndrews, J. J. & Soramäki, K. (2004). *Economizing on liquidity with deferred settlement mechanisms*. FRBNY Economic Policy Review 10(3), 51–72. | confirmed (full text) |
| Method | Historical Fedwire data; 20%, 50% or 80% of payments routed to one-hour netting, six-hour netting or receipt-reactive gross settlement (RRGS). | confirmed |
| Results | Peak overdraft vs RTGS: one-hour netting −1.9% / −4.7% / −5.6%; six-hour netting −5.9% / −8.7% / −25.9%; RRGS +4.4% / +2.8% / −12.5% (average overdraft −44% at 80%) (Table 3, PDF p. 7). | confirmed |
| Delay bound | The paper does **not** state a bound on the saving of a delay-bounded mechanism (no such statement found in the full text). | confirmed (relevant to kill criterion K2) |

## 5. PixSim

| Field | Value | Status |
|---|---|---|
| Reference | "PixSim", arXiv 2609.30684 (cited in `CLAUDE.md` as an example of calibration with a source per parameter). | **not found**: no public search found this work. **Do not cite** until someone confirms the identifier. Removed from the README. |

## 6. Tools cited in `NOVELTY.md`

BoF-PSS (the Bank of Finland simulator) and PSSimPy: check version, licence and features before
using them in the paper's novelty table.
