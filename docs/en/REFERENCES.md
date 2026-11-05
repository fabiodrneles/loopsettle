# References — verification status

> 🇧🇷 [Versão em português](../REFERENCES.md)
>
> Basis for the papers' bibliography. Each reference has a **status**: **confirmed** (title,
> authors and year checked against a public source), **partial** (it exists, but some detail
> still has to be checked in the full text) or **not found** (not found in public searches; do
> not cite until confirmed).
>
> Last check: 5 Nov 2026, by web search. The Bank of Finland PDFs could not be opened from this
> environment (network blocked), so the numbers in section 1 come from abstracts and indexes
> (EconStor, RePEc, Risk.net) and must be checked against the full text before submission.

## 1. Hellqvist & Korpinen — methodological basis

| Field | Value | Status |
|---|---|---|
| Version 1 | Hellqvist, M. & Korpinen, K. (2021). *Instant payments as a new normal: Case study of liquidity impacts for the Finnish market*. Bank of Finland Economics Review 7/2021. | confirmed |
| Version 2 | Hellqvist, M. & Korpinen, K. (2026). *Instant payments as the new normal: how much more money do the banks need?* Journal of Financial Market Infrastructures, 13(2), March 2026. | confirmed |
| Data | Synthetic transaction-level data mimicking normal retail credit transfer flows between Finnish participants in the STEP2 system (EBA Clearing). | confirmed (abstract) |
| Comparison | Liquidity needs in the cycle-based model vs full instant mode. | confirmed (abstract) |
| Aggregate result | A small aggregate increase, especially when payments were already split into several batches. | confirmed (abstract) |
| Figure quoted | Daily liquidity needs rise by **at most 8.6%** with full migration. | **partial**: from the discussant's summary (Alexandrova-Kabadjova, Economics of Payments X, 2021), not from the paper's text |
| Heterogeneity | Variations between banks and between days can be large even for normal flows; hence the emphasis on liquidity buffers. | confirmed (abstract) |
| Predictability | The size of the increase can be anticipated from *ex ante* known statistics of each bank's payment flow. | confirmed (abstract) |
| Topology | With sufficient liquidity, bank-level results do not depend on the network topology. | partial (discussant's summary) |
| To check | Number of banks, days and simulation runs (the slides mention 15 banks and Monte Carlo); how many cycles per day the STEP2 reference model uses; the exact definition of liquidity need. | pending |

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
| To check | The exact definition of the lost-netting measure (the "potential capacity" LoopSettle reproduces in style) and the 57%–80% figures quoted in earlier conversations. | **pending** |
| Caveat | The text does not necessarily represent the BCB's official view (the paper's own disclaimer). | confirmed |

Sources: [PDF at the BCB](https://bcb.gov.br/content/publicacoes/WorkingPaperSeries/WP619.pdf) ·
[RePEc](https://econpapers.repec.org/RePEc:bcb:wpaper:619)

## 3. CDFD — circular-directional decomposition

| Field | Value | Status |
|---|---|---|
| Reference | Homs-Dones, M., MacKay, R. S., Sansom, B. & Zhou, Y. (2025). *Circular Directional Flow Decomposition of Networks*. arXiv:2506.12546 [physics.soc-ph]. | confirmed |
| Content | Splits flow into a circular and an acyclic part; circularity index between 0 and 1; two benchmarks: **maximum circularity** (implemented in LoopSettle) and **BFF — Balanced Flow Forwarding** (proportional/local variant, **not** implemented). | confirmed |
| Journal version | An earlier README cited *"How circular is a directed network? A flow decomposition approach"*, *Royal Society Open Science* (2026). | **not found**: cite the arXiv version until confirmed |

Sources: [arXiv](https://arxiv.org/abs/2506.12546) · [WRAP Warwick](https://wrap.warwick.ac.uk/190782)

## 4. Bech & Soramäki — gridlock resolution

| Field | Value | Status |
|---|---|---|
| Reference | Bech, M. L. & Soramäki, K. (2001). *Gridlock Resolution in Interbank Payment Systems*. Bank of Finland Discussion Papers 9/2001. | confirmed (classic reference) |

## 5. PixSim

| Field | Value | Status |
|---|---|---|
| Reference | "PixSim", arXiv 2609.30684 (cited in `CLAUDE.md` as an example of calibration with a source per parameter). | **not found**: no public search found this work. **Do not cite** until someone confirms the identifier. Removed from the README. |

## 6. Tools cited in `NOVELTY.md`

BoF-PSS (the Bank of Finland simulator) and PSSimPy: check version, licence and features before
using them in the paper's novelty table.
