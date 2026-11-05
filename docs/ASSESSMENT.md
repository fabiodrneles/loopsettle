# ASSESSMENT — does LoopSettle have a result?

> [Versão em português](ASSESSMENT.pt-BR.md)
>
> Answer to the review request of 5 Nov 2026 ("does LoopSettle have a result, or is it a waste
> of time?"). Every statement cites a file and a number. New evidence comes from the
> `robustness` experiment (`results/robustness/`, section 7 of
> [`results/SUMMARY.md`](../results/SUMMARY.md)) and from the delay bound proved in
> [`METHODOLOGY.md`](METHODOLOGY.md#9-the-delay-bound) and tested in
> `crates/loopsettle-sim/src/bound.rs`.

## Verdict: **PIVOT**

The tool is good and one result is solid, but the headline as currently written ("potential
9.27% vs realizable 0.0147%, about 630× smaller") **does not survive scale**: the 9.27% shrinks
with the square root of volume and the ratio is not a stable number. What survives is narrower
and, we think, more useful:

> **New central claim.** In an instant payment system, any settlement mechanism that delays
> each payment by at most *d* can save at most the value each participant pays in the *d*
> seconds before its liquidity peak. For second-scale deadlines this ceiling is below 1% of the
> liquidity need in every configuration we tested, and the realised saving is about 0.01% —
> independent of the cycle-search algorithm, the random seed and the scale. "Potential netting"
> measures (85% of volume, 9% of liquidity) therefore say almost nothing about what a
> time-bounded mechanism can recover; whether longer horizons recover more depends on
> *systematic* intraday imbalances that public aggregates do not identify.

Next step: the four experiments in [`PAPER_READINESS.md`](PAPER_READINESS.md), then a
**preprint** (SSRN / arXiv q-fin) and a **JOSS** paper for the software. JFMI becomes realistic
only with real intraday data or a validated intraday-heterogeneity model (workstream W2).

## A. Does the central result hold?

| Threat | Evidence | Verdict |
|---|---|---|
| **1. Scale** (239× fewer payments per window) | `robustness/scale.csv`: micro-cycles ≤10 s save 0.0147% at 1M payments/day, 0.0027% at 3M, −0.0005% at 10M, −0.0016% at 30M (scale factor 8). The delay bound for 10 s falls from 0.76% to 0.27% and then flattens. | **Survives** for micro-cycles: more volume does not create recoverable liquidity; it goes to zero. **Fails** for the "potential": end-of-day netting falls from 9.27% to 1.68% (exponent −0.50, i.e. ∝ 1/√N). Extrapolated to real Pix volume (factor 1) it would be about 0.6% — in this model. |
| **2. Receiver bias σ** | `robustness/seeds_summary.csv`: mean micro-cycle (10 s) saving 0.0178% (σ=0), 0.0116% (σ=0.4), 0.0033% (σ=0.8); the 10 s bound is 3.3%, 0.80% and 0.70%. LB/UB moves from 0.70 to 0.97. | **Survives** on σ ∈ [0; 0.8]: means ≤ 0.018% of UB; the worst single run of the 30 is 0.103% (σ=0, seed 8, `robustness/seeds.csv`). Outside that range not tested (σ < 0 is meaningless; σ > 0.8 makes LB/UB → 1 and every saving smaller). |
| **3. Noise near zero** | Same file, 10 seeds: σ=0.4 micro 10 s 0.0116%, 95% CI [0.0079; 0.0152]%; all 10 seeds positive. The non-monotonic σ pattern in the old single-seed table (0.048 / 0.005 / 0.012 / 0.003) was noise: with 10 seeds it is monotonic (0.018 > 0.012 > 0.003). | **Resolved**: the value is small *and* measured precisely. Single-seed numbers must no longer be reported; the end-of-day saving has a wide CI (σ=0.4: 7.8%, CI [4.5; 11.1]%). |
| **4. UB model** (no opening balance, one day) | Not tested. The bound shows *why* the result looks the way it does: the peak is set by hour-long drift. But the model has a single hourly profile shared by all participants and stationary bilateral weights (D8, D9), so it cannot produce systematic intraday imbalances (payroll, acquirer settlement, fintech funding patterns) — the very thing longer-horizon netting would recover. Nor does it model the STR operating window or overnight prefunding of the Conta PI. | **Open — the biggest remaining threat**, but it affects the *potential* side, not the micro-cycle side: systematic intraday imbalances last minutes to hours, so the 10 s bound still applies. |

Algorithm objection (REVIEW 13): `robustness/cycle_len.csv` — saving 0.0155% with cycles of
length 2 (bilateral only), 0.0100% (3), 0.0147% (6), 0.0145% (10). The heuristic is not the
bottleneck, and the bound holds for *any* algorithm, including an optimal solver.

## B. Is it new?

What, to our knowledge (to be confirmed by the literature check W4), had not been shown:

1. **A model-free ceiling** on the saving of any delay-bounded mechanism (the delay bound),
   checked participant by participant against simulated engines.
2. **Second-scale deadlines in retail instant payments**: the classic liquidity-saving studies
   use large-value systems and accept delays of minutes to hours — Johnson, McAndrews & Soramäki
   (2004) simulate 1-hour and 6-hour netting and receipt-reactive settlement on Fedwire; Denbee &
   McLafferty (2012) find savings above 30% in CHAPS with delays from minutes to over two hours;
   Hellqvist & Korpinen (2021/2026) compare fixed cycles with instant settlement. None of them
   imposes the 2.8–10 s deadline of an instant scheme.
3. **The collapse of "potential netting" under stationary flows** (∝ 1/√N): a warning against
   reading potential-capacity measures as recoverable liquidity.
4. **An open, reproducible simulator** of instant payment settlement with property-tested
   invariants, runnable on a laptop and in the browser.

What is **not** new: the liquidity–delay trade-off itself (e.g. Bank of Canada WP 06-20 on
LVTS), LSM design economics (Martin & McAndrews 2008), and the idea that LSMs work by
reshuffling queued payments (Bech & Soramäki 2001). The intuition "near-real-time netting saves
little" is plausible to practitioners; the contribution is to bound it and measure it.

## C. Does anyone care?

- **Central banks designing or extending instant systems** — the Bank of Canada (RTR,
  prefunded, launching Q4 2026), the Federal Reserve (FedNow liquidity management), the
  Eurosystem (TIPS), the BCB (Pix). The bound is a quick test they can run on their own
  intraday data: *if the flow near each participant's peak is small, a second-scale LSM is not
  worth building; look at prefunding windows and intraday credit instead.*
- **Payment-system researchers** (BoF-PSS community, CPMI): an open tool and a reproducible
  benchmark.
- **Bank treasuries** sizing Conta PI balances: the per-participant heterogeneity
  (`robustness/participants_ratio.csv`: among participants that need liquidity in both designs,
  instant/daily ratio median 1.16×, p90 2.90×, max 19×; 104 of 200 need liquidity only under
  instant settlement) mirrors the Finnish study's main finding.

Would it change a decision? Possibly one: whether to invest in a second-scale liquidity-saving
mechanism for an instant system. That is a real but narrow decision.

## D. What is wrong or weak (most severe first)

1. **The 9.27% / "630×" headline is a scale artifact** (threat 1 above). It must leave the README,
   CLAIMS and the paper outline (done in this change).
2. **Stationary arrivals**: one hourly profile for everyone and constant bilateral weights
   (D8, D9). This is what makes the potential collapse as 1/√N; real intraday heterogeneity
   would not. Needs workstream W2.
3. **UB definition**: one day, zero opening balance, no STR window, no overnight/weekend
   prefunding of the Conta PI, no intraday credit (LIMITATIONS 6). The paper must define
   "liquidity need" the way the BCB rules do (workstream W3; documents in the readiness spec).
4. **σ is not identified** by any public datum: LB/UB ranges 0.70–0.97 with it
   (`seeds_summary.csv`). Report all results as bands over σ.
5. **Hodge rung is uninformative on this graph**: for random flows on a connected graph with
   m edges and n nodes, the expected share of the squared norm outside the gradient space is
   (m − n + 1)/m = (6620 − 200 + 1)/6620 = 0.970; the observed L2 curl share is 0.977
   (`baseline/ladder.csv`). Hodge here measures graph density, not circularity. Drop it from the
   central claim or keep it as a cautionary methods note with this null value.
6. **The delay bound is loose**: realised saving is about 1.5% of the bound
   (σ=0.4: 0.0116% vs 0.80%). It is a ceiling, not an estimate; say so.
7. **Unverified citations** behind two comparisons: the Finnish 8.6% figure and the exact
   definition of the BCB WP 619 netting measure (`docs/REFERENCES.md`). Misdescribing WP 619
   would be the most embarrassing error a reviewer could find.
8. **Minor**: weak stress scenario (REVIEW 39); `QueueOnShortfall` without strict FIFO (D13);
   micro-cycles can slightly *raise* the peak at large scale (−0.0016% at 30M), which the paper
   should mention.

## E. Value independent of the thesis

Yes. As software it meets the JOSS profile: Apache-2.0, public repository, eight
property-tested invariants plus the new delay-bound test, byte-for-byte reproducible
experiments, a 5 MiB memory footprint, a bilingual browser UI and documentation. As portfolio
it shows rare engineering-plus-domain work. Even if the paper stalls, a JOSS paper is
achievable on its own.

## Kill criteria — stop (or downgrade to a software-only paper) if:

| # | Experiment | Stop if |
|---|---|---|
| K1 | W2: heterogeneous intraday profiles (participant-specific hourly patterns calibrated to any public intraday data) | the 10 s delay bound rises above **2% of UB** at 30M payments/day — then second-scale mechanisms could matter and the "negligible" claim is false |
| K2 | W4: literature check (CPMI, ECB/TIPS, Bank of Canada/RTR, Fed/FedNow, BoF-PSS papers since 2018) | a published study already reports savings of LSMs under **second-scale deadlines in an instant system**, or states the delay bound — then the paper is a replication; keep JOSS only |
| K3 | W3: BCB rules on Conta PI funding, STR hours and intraday credit | the binding constraint is shown to be something the model cannot represent (e.g. only overnight/weekend prefunding matters and intraday peaks are always covered from Reservas) — then the intraday question is moot; pivot to the prefunding-window question or stop the paper |
| K4 | W1: rerun with all fixes | the realised 10 s saving's 95% CI ever exceeds **0.1% of UB** in the calibrated band σ ∈ [0; 0.8] — then "about 0.01%" is wrong and must be re-derived |

## Sources

- Johnson, McAndrews & Soramäki (2004), FRBNY Economic Policy Review 10(3):
  [PDF](https://www.newyorkfed.org/medialibrary/media/research/epr/04v10n3/0412mcanpdf.pdf)
- Denbee & McLafferty (2012), *Liquidity Saving in CHAPS: A Simulation Study*:
  [IGI Global](https://igi-global.com/chapter/content/68694)
- Martin & McAndrews (2008), *Liquidity-saving mechanisms*, JME 55(3):
  [RePEc](https://econpapers.repec.org/RePEc:eee:moneco:v:55:y:2008:i:3:p:554-567)
- Bank of Canada WP 06-20, LVTS delay–liquidity trade-off: [RePEc](https://ideas.repec.org/p/bca/bocawp/06-20.html)
- Other references: [`REFERENCES.md`](REFERENCES.md)

## Update — full texts read (5 Nov 2026, same day)

The owner sent four full texts. What they settle:

| Point | Finding | Effect on the assessment |
|---|---|---|
| Finnish figure (D.7) | "8.6%" is the **95th percentile** of the increase against **6 cycles**; the average is 2.7%. Against one daily cycle, instant needs 1.45× (BoFER 7/2021 pp. 2, 9–11). LoopSettle: +3.9% (σ=0.4) to +13.7% (σ=0) against 6 cycles; 1.10×–1.50× against 1 cycle. | resolved; same order of magnitude, now on the right baseline |
| Liquidity definition (D3, threat 4 in part) | Their liquidity need is exactly LoopSettle's UB/LB (pp. 3–4). | the definition is standard; threat 4 now concerns only the funding windows (W3), not the measure itself |
| BCB WP 619 measure (D.7) | Eq. (3.2), per bank, reported as quartiles: median 57–80%, p25 18–45%, p75 73–90% on real Pix data (PDF p. 16). LoopSettle on the same statistic: median 78%, p75 91%, but **p25 63%**. | resolved; and it gives the **first real-data target for σ**: the model has too few one-sided banks. Calibrating σ (or a per-class bias) to WP 619's p25 makes weakness D.4 partly identifiable. New item W2b. |
| Delay bound novelty (K2) | Johnson, McAndrews & Soramäki (2004) state no bound; their hour-scale netting saves 2–6% (1 h) to 6–26% (6 h) of peak overdraft in Fedwire (Table 3). | K2 does not fire on the closest work; the literature check W4 continues for post-2018 work |
| CDFD | Maximum circularity is exactly the unit-cost min-cost problem LoopSettle solves; BFF is fully specified (Algorithm 1) (arXiv 2506.12546, §5.1–5.2). | D5 confirmed; D6 can now be implemented |

The verdict stays **PIVOT**; no kill criterion has fired.
