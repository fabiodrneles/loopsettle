# PAPER READINESS — specification of everything needed before writing

> [Versão em português](PAPER_READINESS.pt-BR.md)
>
> Follows from [`ASSESSMENT.md`](ASSESSMENT.md) (verdict: PIVOT). Each item has an owner
> (**Claude** = can be done in a session; **Owner** = needs you), an acceptance criterion and a
> status. The study is "ready to write" when every item marked *required* is done and no kill
> criterion (K1–K4) has fired.

## Status board

| ID | Workstream | Required | Owner | Status |
|---|---|---|---|---|
| W0 | Sources only the owner can download | yes | Owner | **4 of 15 received** (#1, #4, #5, #6 — read and recorded in `REFERENCES.md`) |
| W1 | Reporting hygiene (multi-seed, bands over σ, no "630×") | yes | Claude | **done** in this change (robustness experiment, CLAIMS, README) |
| W2 | Intraday heterogeneity model (kill criterion K1) | yes | Claude (+ owner for data) | **done**: K1 not fired (`results/heterogeneity/`); calibrating the profiles to real data needs W0 #16 |
| W2b | Calibrate receiver bias to the WP 619 per-bank distribution (p25 18–45%, median 57–80%) | yes | Claude | **done**: σ = 1.0 (`results/bias-calibration/`) |
| W9 | Becoming a reference (see section W9) | yes | Owner + Claude | to do |
| W3 | Liquidity need as the BCB rules define it (K3) | yes | Claude, after W0 | to do |
| W4 | Literature check for novelty (K2) | yes | Claude + owner | to do |
| W5 | Delay bound: theorem, curve over d, tightness | yes | Claude | theorem and test **done**; curve to do |
| W6 | Remaining weaknesses (strict FIFO, stress, Hodge null) | partly | Claude | to do |
| W7 | Paper and software packaging (outline, JOSS, preprint, Zenodo) | yes | Claude + owner | to do |
| W8 | Run the UI on the reference laptop (i3, 4 GB) | no | Owner | to do |

## W0 — Sources to download and send

These sites are blocked from the cloud environment. Download each PDF and **send it in the chat**
(attachment). Do not commit PDFs to the repository: most are copyrighted. For each one, what is
needed is listed.

| # | Document | Link | What is needed from it |
|---|---|---|---|
| 1 | Hellqvist & Korpinen (2021), BoF Economics Review 7/2021 | [PDF](https://publications.bof.fi/bitstream/10024/43594/1/BoFER_7_2021.pdf) | the aggregate increase (is it 8.6%?), number of banks and days, cycles per day in the STEP2 baseline, exact definition of liquidity need, per-bank results |
| 2 | Hellqvist & Korpinen (2026), JFMI 13(2) | [Risk.net](https://www.risk.net/node/7963364) (paywalled; only if you have access) | what changed from the 2021 version |
| 3 | Discussant slides, Economics of Payments X | [PDF](https://www.suomenpankki.fi/globalassets/en/research/seminars-and-conferences/conferences-and-workshops/documents/eopx2021/alexandrova-kabadjova_discussion_instant-payments-as-a-new-normal_presentation.pdf) | where the 8.6% figure comes from |
| 4 | Gonzalez, Ma & Zeng (2025), BCB WP 619 | [PDF](https://www.bcb.gov.br/content/publicacoes/WorkingPaperSeries/WP619.pdf) | the exact definition of their netting/"lost offsetting" measure and its numbers (the 57–80% quoted earlier) — LoopSettle's "potential" must match it or say how it differs |
| 5 | Homs-Dones et al. (2025), CDFD | [arXiv PDF](https://arxiv.org/pdf/2506.12546) | the definition of maximum circularity (confirm it equals our min-cost formulation) and the BFF algorithm (D6) |
| 6 | Johnson, McAndrews & Soramäki (2004), FRBNY EPR | [PDF](https://www.newyorkfed.org/medialibrary/media/research/epr/04v10n3/0412mcanpdf.pdf) | savings vs delay for 1-hour and 6-hour netting and RRGS; whether they state any delay bound |
| 7 | Martin & McAndrews (2008), JME / FRBNY SR 282 | [RePEc page with PDF link](https://ideas.repec.org/p/fip/fednsr/282.html) | the LSM welfare argument, to position ours |
| 8 | Denbee & McLafferty (2012), CHAPS LSM simulation | [IGI Global](https://igi-global.com/chapter/content/68694) · [BoE Financial Stability Paper 7 (overview)](https://ideas.repec.org/p/boe/finsta/0007.html) | savings vs delay curve in CHAPS |
| 9 | Bank of Canada WP 06-20 (LVTS delay–liquidity trade-off) | [RePEc](https://ideas.repec.org/p/bca/bocawp/06-20.html) | the trade-off curve, to compare with ours |
| 10 | DNB WP 682, liquidity saved by an LSM | [PDF](https://www.dnb.nl/media/m5xdz2nx/working-paper-no-682_tcm47.pdf) | method and results |
| 11 | Bech & Soramäki (2001), BoF DP 9/2001 | [RePEc](https://ideas.repec.org/p/zbw/bofrdp/rdp2001_009.html) | gridlock resolution algorithm, for the methods section |
| 12 | BCB — Pix regulation and SPI rules | [Pix page at the BCB](https://www.bcb.gov.br/estabilidadefinanceira/pix) → *Regulamentação* (Regulamento do Pix, Resolução BCB nº 1/2020; rules of the SPI and the Conta PI) | how the Conta PI is funded (transfers from Reservas, when), what happens at night and on weekends, whether intraday credit exists for Pix |
| 13 | BCB — STR operating hours | [STR page at the BCB](https://www.bcb.gov.br/estabilidadefinanceira/str) → *Grade horária* | the window in which Conta PI ↔ Reservas transfers are possible |
| 14 | BCB — Manual de Tempos do Pix | [Pix page](https://www.bcb.gov.br/estabilidadefinanceira/pix) → *Manuais* | confirm the 2.8 s / 50% target used in the results |
| 15 | BCB — Pix statistics by hour of day, if published | [Pix statistics](https://www.bcb.gov.br/estabilidadefinanceira/estatisticaspix) · [Olinda open data (the endpoint `data/fetch_pix.py` uses)](https://olinda.bcb.gov.br/olinda/servico/Pix_DadosAbertos/versao/v1/odata) | any intraday distribution (by hour, by type of participant) to calibrate W2 |

The BCB links point to section pages; if one has moved, search the BCB site for the document title.

Optional, for a second-country comparison: Banxico's SPEI description
([PDF](https://www.banxico.org.mx/sistemas-de-pago/d/%7B89B6CCF0-6070-7389-3DD5-B27AC4ECD9D1%7D.pdf))
and Payments Canada's RTR material ([page](https://www.payments.ca/how-canadas-real-time-rail-will-transform-account-account-payments)).


**Added after W2/W2b (5 Nov 2026):**

| # | Document | Link | What is needed from it |
|---|---|---|---|
| 16 | BCB — SPI annual report 2024 | [PDF](https://www.bcb.gov.br/content/estabilidadefinanceira/relatorios_SPI/relatorio_anual_spi_2024.pdf) | the section "average number of transactions by time slot": replaces the assumed hourly profile (`hourly`) with a sourced one; any breakdown by type of participant calibrates W2 |
| 17 | BCB — Relatório de Gestão do Pix (2023) | [PDF](https://www.bcb.gov.br/content/estabilidadefinanceira/pix/relatorio_de_gestao_pix/relatorio_gestao_pix_2023.pdf) | Conta PI funding, liquidity management and SPI operating rules (W3) |
| 18 | ECB — TIPS and RT1, implications for liquidity management (2018) | [PDF](https://www.ecb.europa.eu/paym/groups/pdf/mmcg/20181203/Item_5_2_Instant_Payment_Services_TIPS_and_RT1_Implications_for_Liquidity_Management.pdf) | how euro-area instant systems fund liquidity (W3, W4) |
| 19 | Federal Reserve — FedNow readiness guide, managing liquidity | [PDF](https://explore.fednow.org/resources/readiness-guide-managing-liquidity-in-an-instant-payments-world.pdf) | FedNow liquidity tools and windows (W3, W4) |
| 20 | Bank of Canada — Lynx liquidity and LSM studies | [SDP 2020-9](https://www.bankofcanada.ca/2020/09/staff-discussion-paper-2020-9/) · [SDP 2022-3](https://www.bankofcanada.ca/2022/02/staff-discussion-paper-2022-3/) | recent LSM-savings results to position ours (W4) |
| 21 | Bank of Finland — 22nd Simulator Seminar (2024), Hellqvist: "Pitfalls on the Path to Instant Payments – Liquidity Risk on Finnish Instant Payments Market" | [seminar page](https://suomenpankki.fi/en/money-and-payments/payment-and-settlement-system-simulator/events/22nd-seminar-2024) | the slides (closest recent work; checks K2) |

**Acceptance:** every "to check" item in [`REFERENCES.md`](REFERENCES.md) is resolved, with the
page number of the source.

## W1 — Reporting hygiene *(done)*

- Every headline number comes with seeds and a 95% CI (`results/robustness/seeds_summary.csv`).
- Results are reported as bands over σ ∈ [0; 0.8], not at σ = 0.4 only.
- The "potential 9.27% vs realizable 0.0147% (630×)" headline is removed from README and CLAIMS;
  the scale dependence is stated.
- **Acceptance:** every headline number in README and CLAIMS is a multi-seed mean with CI; single-seed rows in the README table are labelled. ✔ (the outline is rewritten in W7)

## W2 — Intraday heterogeneity (kill criterion K1)

- **What:** participant-specific intraday profiles instead of one shared hourly profile.
  Classes to model: payroll payers (bursts on fixed hours/days), acquirers and payment
  institutions (inflows during the day, outflows in settlement batches), fintechs funding their
  Conta PI, big banks. Parameters flagged `ASSUMPTION` unless calibrated with W0 #15.
- **Why:** stationary arrivals make the potential collapse as 1/√N (`robustness/scale.csv`);
  systematic imbalances would not. This is the main open threat to the potential side.
- **How:** add an optional `profiles` table to the scenario TOML (per class, 24 or 96 weights);
  keep the current behaviour as default so existing results stay byte-identical.
- **Acceptance:** `robustness` re-run with heterogeneous profiles at 1M–30M; report the 10 s
  bound and realised saving. **K1 fires** if the 10 s bound exceeds 2% of UB at 30M.

## W3 — Liquidity need as the BCB rules define it (kill criterion K3)

- **What:** after W0 #12–13, model the funding windows of the Conta PI: periods when transfers
  from Reservas are possible (STR open) and periods when they are not (nights, weekends,
  holidays). Liquidity need becomes the maximum drawdown *within each closed window* plus the
  top-ups needed in open windows. Run over several days, including a weekend.
- **Why:** UB over one day with zero opening balance may not be the constraint banks actually
  face.
- **Acceptance:** an experiment `funding-windows` with UB per window type; the central table
  recomputed under that definition. **K3 fires** if intraday peaks inside open windows never
  bind (always covered from Reservas).

## W4 — Literature check (kill criterion K2)

- **What:** a structured search for studies of liquidity-saving mechanisms or netting under
  second-scale deadlines in instant/fast payment systems, and for any statement of the delay
  bound. Places: BIS/CPMI publications, ECB (TIPS, RT1), Bank of Canada (RTR), Federal Reserve
  (FedNow), Bank of England (RT2, Faster Payments), BoF-PSS simulator seminar proceedings since
  2018, JFMI, SSRN.
- **Acceptance:** a related-work table in [`NOVELTY.md`](NOVELTY.md) with the closest five
  works and what each does not do. **K2 fires** if one of them already reports second-scale LSM
  savings in an instant system or states the bound.

## W5 — Delay bound

- **Done:** theorem and proof ([`METHODOLOGY.md`](METHODOLOGY.md#9-the-delay-bound), section
  9); per-participant test against micro-cycle and deferred engines
  (`crates/loopsettle-sim/src/bound.rs`).
- **To do:** a curve of realised saving and bound over d ∈ {1 s, 10 s, 1 min, 15 min, 1 h, 6 h,
  1 day}; a tightness ratio (realised / bound) with CIs; the figure goes in the paper's results
  section.
- **Acceptance:** `fig_bound.svg` generated by a command, in `results/robustness/`.

## W6 — Remaining weaknesses

| Item | What | Required |
|---|---|---|
| Hodge null | report (m − n + 1)/m next to every Hodge number, or drop Hodge from the main ladder | yes |
| Strict FIFO | optional strict per-payer FIFO in `QueueOnShortfall` (REVIEW 14) | no |
| Stress | outage at the participant's peak time, or with partial abandonment (REVIEW 39) — or drop it from the paper | yes (either way) |
| BFF CDFD | proportional CDFD (D6), after W0 #5 | no |
| On-chain | external validation on stablecoin transfers (Phase 8) | no |

## W7 — Packaging

- Rewrite [`PAPER_OUTLINE.md`](PAPER_OUTLINE.md) around the new central claim (bound + measured
  gap + why potential measures mislead).
- `paper/paper.md` for JOSS (summary, statement of need, references), `CITATION.cff` v0.2.0,
  Zenodo release with DOI.
- Preprint: SSRN (economics) or arXiv q-fin.GN, after W2–W4.
- **Acceptance:** `make all` reproduces every number of the preprint; JOSS checklist satisfied.

## W9 — Becoming a reference

What turns a good study into a reference is that **other people use it**. In order of impact:

1. **A tool central banks can run on their own data, without sharing it.** Add
   `loopsettle bound <payments.csv>` and `loopsettle settle <payments.csv>`: given a file of real
   payments (from, to, amount, time), report UB, LB, the delay bound for any *d* and the savings
   of each engine. The data never leaves the institution; only aggregates come out. This is the
   single feature most likely to get LoopSettle used by the BCB, the Bank of Canada (RTR launching
   now) or a bank treasury. **Claude.**
2. **Published, citable work.** JOSS paper (software) and a preprint (SSRN / arXiv q-fin) with the
   delay bound and the calibrated results; Zenodo DOI on a v0.2.0 release. **Claude drafts; owner
   submits.**
3. **The right audience.** Submit to the Bank of Finland Simulator Seminar (held every late
   August; it is where Hellqvist & Korpinen present) and to CEMLA payments forums; offer a short
   talk to the BCB research department. **Owner.**
4. **A public demo.** Publish the web UI on GitHub Pages so anyone can try it without installing
   anything. **Claude.**
5. **A data partnership.** With per-participant intraday data, the open question (0.2% vs 3% of
   recoverable liquidity) becomes a measured number. The tool in item 1 is what makes that
   conversation possible. **Owner.**

## W8 — Reference laptop

Open the web UI (`cd web && npm install && npm run wasm && npm run dev`) on the i3 / 4 GB laptop
and note load time and fluidity. **Owner.**
