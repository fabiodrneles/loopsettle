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
| Reference | Bech, M. L. & Soramäki, K. (2001). *Gridlock Resolution in Interbank Payment Systems*. Bank of Finland Discussion Papers 9/2001. | **read** (full text, copy supplied by the owner; not stored: no licence to redistribute) |
| Algorithm | §3.3: Step 1 include every queued payment; Step 2 compute each bank's ex-post balance; Step 3 pick any bank below zero and remove the *last* payment of its queue, repeat. Optimal for the gridlock-resolution problem with the liquidity and sequence (FIFO per bank) constraints, equations 5–6; the result does not depend on which deficient bank is picked (footnote 9, appendix). Developed by Danmarks Nationalbank for KRONOS. | confirmed. **LoopSettle's `gridlock` engine is this algorithm** (`engines.rs::run_gridlock`; D27). |
| Bounds and indicator | UB as in Koponen & Soramäki; LB = max(0, received − sent) (eq. 7); delay indicator ρ (eq. 9), the 1998 one. Liquidity levels written `L_i = α (UB_i − LB_i)` (eq. 8), α ∈ {0, 0.2, …, 1}: without the `+ LB_i` of the 1998 equation 4 and of the paper's own text ("just enough liquidity to settle all the days payments" at the lower extreme). LoopSettle follows the 1998 form. | confirmed (p. 19) |
| Results | Danish (64 days) and Finnish (100 days) RTGS data; the algorithm reduces queuing "at all levels of liquidity, but in particular when intra-day liquidity is scarce", by up to 50% (Abstract; §6). | confirmed |

## 4a. Koponen & Soramäki — intraday liquidity needs (origin of UB and LB)

| Field | Value | Status |
|---|---|---|
| Reference | Koponen, R. & Soramäki, K. (1998). *Intraday Liquidity Needs in a Modern Interbank Payment System: A Simulation Approach*. Bank of Finland Studies E:14. https://hdl.handle.net/10419/212947 | **read** (full text from EconStor, supplied by the owner; EconStor terms forbid public redistribution, so only the citation is kept) |
| Bounds | Equation 1: theoretical lower bound = end-of-day net debit (RTGS with queuing, no gridlock); equation 2: theoretical upper bound = largest cumulative net debit over the day (RTGS without queuing, immediate settlement). §2.3.1, Figure 2. | confirmed. **These are LoopSettle's LB and UB.** |
| Curve between the bounds | The trade-off between B (LB) and C (UB) is obtained **by simulation**, at 11 liquidity levels `LA_i = LB_i + LL (UB_i − LB_i)` (equation 4, §6.1). No closed form for a deadline. | confirmed: the delay-bounded floor of METHODOLOGY §9b is not in this study. |
| Indicators | ρ (settlement delay, eq. 5): sum over minutes of queued value / sum over minutes of cumulative outgoing value; π (liquidity usage, eq. 6): peak liquidity used / gross value sent. System values: numerators and denominators summed over banks. | confirmed; implemented as `RunMetrics::rho` and `metrics::pi`. |
| Queue rules | FIFO with per-bank "closed" status (Figure 9) = LoopSettle's `StrictFifoPerPayer`; Bypass FIFO (§3.1.2) = `BypassFifo`, D13. Netting of queues (§3.1.3): all queues at once if every bank covers its net (equation 3), otherwise nothing ("illiquidity, not gridlock"). | confirmed; implemented as `netq:fifo|bypass:<window>:<delay>`. |
| Results | Netting the queues every 20, 5 or 1 minutes "is effective in reducing settlement delay only for the higher liquidity levels and even then only with minor effect" (§7.4.3); splitting of payments works better (§7.4.4); bilateral netting of queues proposed as future research (§8.4). | confirmed |

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

## 7. Second batch of full texts (5 Nov 2026)

| Source | What it settles | Status |
|---|---|---|
| McMahon et al. — *Payment reordering as a liquidity savings mechanism in T2* (GoodLabs Studio with Korpinen and Laine; 22nd BoF Simulator Seminar, 2024; slides, [`references/pdf/mcmahon-2024-t2-reordering-slides.pdf`](references/pdf/mcmahon-2024-t2-reordering-slides.pdf)) | Reordering payments in tumbling/sliding windows with most delays under 5 s saves 13.8–21.4% of liquidity in simulations, "up to 30% daily vs FIFO", in T2 (large-value). The baseline is FIFO with queues under constrained liquidity, not LoopSettle's unlimited-liquidity UB. | slides read |
| McMahon, McGillivray, Desai, Rivadeneyra, Lam, Lo, Marsden and Skavysh — *Improving the efficiency of payments systems using quantum computing* (arXiv [2209.15392](https://arxiv.org/abs/2209.15392) v3, Jan 2023, CC BY 4.0, [`references/pdf/mcmahon-2023-quantum-reordering-arxiv.pdf`](references/pdf/mcmahon-2023-quantum-reordering-arxiv.pdf); journal version: Management Science 2024, doi [10.1287/mnsc.2023.00314](https://doi.org/10.1287/mnsc.2023.00314)) | **The closest work to LoopSettle, with real data.** Canada's LVTS (large-value), 30 sampled days of 2015–2017, non-urgent payments 8:00–18:00, ≈ 23,000 a day (§4). Each batch of 70 payments (≈ 90 s average wait to fill it, plus ≈ 5 s to solve) is reordered by a hybrid quantum solver to minimise the aggregate maximum net debit position, mNDP (§3.2; with a zero opening balance this is LoopSettle's UB); every payment of the batch settles. Average end-of-day saving vs FIFO: C$240 M a day (median C$129 M; above C$1 B on 3 days; one day negative), Table 1; 25.3% of batches improvable, 49% with no participant moving its mNDP (§5.3). Batches of 140 add C$94–326 M on two test days (Table 3). Participants allocate C$15–20 B of liquidity a day (footnote 2), so the saving is of the order of 1–2% of allocated liquidity — **our arithmetic, not theirs**. They note that large payments with no offsetting flow in the batch erase gains (§5.4), the same mechanism as the delay bound. | **read (arXiv v3)**; the journal version was not read |
| Hellqvist (2024), *Instant payment liquidity risk*, 22nd BoF Simulator Seminar | Liquidity *risk* of Finnish instant payments: outages (2 h, full day), contagion coefficient, topology sampling, end-customer resubmission. No netting mechanism and no bound. | slides read; K2 not fired |
| Rivadeneyra & Zhang, Bank of Canada SDP 2020-9 | Lynx (large-value RTGS with LSM) on LVTS data: with $14.6 bn initial liquidity, 28 min weighted delay and $17.3 bn liquidity usage. | abstract read |
| Bank of Canada SDP 2022-3 | Lynx offsetting mechanism: about $10 bn of collateral settles all payments by their deadlines, half of LVTS collateral, at the cost of delays and operational interventions. | abstract read |
| ECB (2018), *TIPS and RT1: implications for liquidity management* | Liquidity received in TIPS/RT1 at weekends and T2 holidays can only be used on the next T2 day; weekend funding creates arbitrage between systems. | read; supports W3 |
| Federal Reserve, *FedNow readiness guide: managing liquidity* | FedNow settles individually 24x7; intraday credit only during the Fed's business day; the discount window does not open outside standard hours; 20 s timeout also for liquidity transfers. | read; supports W3 |
| BCB, *Relatório Anual do SPI 2024* | Hourly profile (Graph 12, p. 13) and day-type volumes (Graph 11, p. 12) — now SOURCED in LoopSettle; Conta PI top-ups via STR only 6:30–18:30 on business days plus 30 min (p. 15); top-ups exceed R$ 87 bn/day, larger on Fridays for the weekend; SPI rediscount line residual (R$ 169 M in 2024 vs R$ 13.68 tn of top-ups); Conta PI balances remunerated at Selic within limits (Res. BCB 195/2022, 412/2024). | read |
| BCB, *Relatório de Gestão do Pix 2023* | Outside STR hours the only liquidity sources are electronic-money balances, reserve requirements on demand deposits and the costly SPI liquidity line — the last two for financial institutions only; payment institutions and small banks bore a higher opportunity cost, hence the Conta PI remuneration since Aug 2022 (PDF p. 16). | read |
| Other 2024 seminar talks (Heijmans & van der Woerd on failing participants in LVPS; Mattern & Meyer on participant criticality in RTGS; Drott on DLT cross-border platforms; Polo, Fritz & Rodloff on T2 analytics; Vacirca on initial margin; Brown on retail payment adoption; Lubbersen & Bolt on card rebates) | Not about instant-payment liquidity saving; Heijmans & van der Woerd and Mattern & Meyer are relevant to a future, stronger stress experiment. | skimmed |

## 8. Search for prior statements of the liquidity floor (12 Nov 2026)

Question: has anyone stated the bound `max_s [CO_i(s) − CI_i(s + d)]` (the least liquidity of any
mechanism with maximum delay `d`), or the equality of the per-participant floors with the offline
optimum? Four web searches (intraday liquidity bounds with a delay parameter; liquidity–delay
trade-off; network calculus applied to payment systems; minimum liquidity under a maximum-delay
constraint) and one full-text check.

| What was found | Status |
|---|---|
| The two bounds without a delay parameter — the upper bound (immediate gross settlement = largest cumulative net debit position) and the lower bound (one netting at the end of the day) — are standard and attributed to Koponen & Soramäki (1998, Bank of Finland Studies E:14, sections "Theoretical bounds" and "Real bounds") and Bech & Soramäki (2001/2002). LoopSettle's UB and LB are these. | **read** on 13 Nov 2026 (section 4a): equations 1 and 2 of the 1998 study; neither study states a bound with a deadline. |
| Rivadeneyra & Zhang (Bank of Canada SDP 2020-9, in `docs/references/pdf/`) compute "the minimum liquidity necessary for a given set of payments allowing for the maximum delay". | **read** (p. 10, footnote 13, and section 4.1): "maximum delay" there means delaying failed payments to the end of the day, and the minimum is found by iterative simulation. No deadline `d`, no closed form. Not the floor. |
| Liquidity–delay trade-off by simulation: Leinonen & Soramäki (1999), Enge & Øverli (2006), Arjani (2006), Galbiati & Soramäki (2007), DNB WP 682 (2020), ECB WP 1291 (2011). | search summaries only; none is reported as stating a bound parameterised by a deadline. |
| Network calculus (arrival curves, backlog and delay bounds). | no source found that applies it to payment-system liquidity; the floor is the analogue of a backlog bound with the roles of arrivals and service played by outflows and delayed inflows. |

**Conclusion to use in the paper:** no prior statement was found, in a limited search. That is not
evidence of novelty. Koponen & Soramäki (1998) and Bech & Soramäki (2001) were read on 13 Nov 2026:
neither has it (the curve between the bounds is simulated there). Before claiming it: search the Bank of Finland simulator-seminar proceedings and the *Journal of Financial
Market Infrastructures*, and ask one specialist. The safe wording meanwhile is "to our knowledge".
