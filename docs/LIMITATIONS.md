# Known limitations

> [Versão em português](LIMITATIONS.pt-BR.md)

1. **Synthetic data.** No real interbank data is used. The generator reproduces public
   aggregates (volume, value, mean ticket, B2B share, concentration by keys), but not the real
   bilateral structure or intraday dynamics.
2. **The bilateral structure is an assumption.** The gravity model with receiver bias (D9)
   decides how much of the flow is reciprocal and how much is directional. The LB/UB ratio, the
   circular fraction and every saving depend on it (sensitivity in
   `results/baseline/sensitivity.csv`).
3. **Size ∝ Pix keys.** Each institution's share uses keys per ISPB as a proxy for transactions
   (D11). Digital banks may have many keys per customer.
4. **Scale.** 200 participants and 1 million payments/day (factor 239). The number of payments
   per micro-cycle window is ~239× smaller than in real Pix; the chance of finding cycles in a
   short window should grow at real scale. Absolute liquidity (R$) does not scale linearly.
5. **Hourly profile and bursts.** The intraday profile is an assumption (no public hourly series
   used) and the arrival process has no bursts (Hawkes, paydays).
6. **One day.** The analysis period is one day; there are no cross-day effects and no real
   opening balances.
7. **Heuristic micro-cycles.** One queue head per edge, cycles of up to 6, simple blocking on
   shortfall (D12). A full gridlock optimiser could settle more.
8. **The measured delay is only the delay added by the engine.** The real processing time of the
   SPI and of payment service providers (part of the 2.8 s target in the Pix timing manual) is
   not modelled.
9. **Proportional CDFD in its non-local form.** BFF (D6) is implemented with the paper's Algorithm 2
   (stationary distributions per strongly connected component), which the paper states gives the
   same decomposition as the local dynamics; the local dynamics themselves are not implemented, and
   the paper has no per-edge example to reproduce, so the check is by properties only.
10. **Incomplete external validation.** The `topology <csv>` command is ready, but the on-chain
    stablecoin sample has not been downloaded (Phase 8).
11. **Hodge on dense graphs.** The Hodge curl fraction on the net graph (≈0.98) is dominated by
    the dimension of the cycle space (m − n + 1 ≫ n − 1): random noise projects almost entirely
    onto cycles. It is a shape diagnostic, not a nettable amount.
12. **Web interface checked visually only in an emulated browser.** On 5 Nov 2026 it was opened
    in desktop Chromium (1440 px) and an emulated iPhone 13, in Portuguese and English
    (screenshots in `docs/img/`). It has not yet been opened on the reference laptop (i3, 4 GB).
13. **Calibrated to one system.** Only Pix aggregates are used. Engines exist for the settlement
    models of other systems (FedNow, TIPS, NPP, UPI, Faster Payments — see
    [`INSTANT_PAYMENTS.md`](INSTANT_PAYMENTS.md)), but no other system has been calibrated; their
    payment sizes, intraday profiles and concentration differ.
14. **Stationary arrivals.** One hourly profile shared by all participants and constant bilateral
    weights (D8, D9). Under these assumptions the end-of-day netting saving shrinks as
    1/√(volume) (`results/robustness/scale.csv`); systematic, participant-specific intraday
    imbalances (payroll, acquirers, fintech funding) would not. See
    [`PAPER_READINESS.md`](PAPER_READINESS.md), W2.
    *Update (W2):* class-specific profiles are now available (`profile_strength`, D18); with them
    the potential no longer collapses (2.8% at 30M with the calibrated σ), but their shapes are
    assumptions until per-participant intraday data are used.
15. **Funding windows simplified.** `funding-windows` models STR hours and the weekend, but not
    holidays, participants' actual top-up behaviour, the Conta PI remuneration limits, or the
    rediscount line; open windows assume unlimited top-ups.
16. **Large-value regime is stylised.** `regimes` uses an assumed large-value system to show how
    the bound depends on payment lumpiness; it is not calibrated to T2, Lynx or the STR.
17. **Constrained-liquidity runs.** `liquidity-frontier` assumes that a rejected payment is not
    retried by the payer, that balances are fixed for the day (no top-ups, no intraday credit), and,
    for rule `ub`, that each participant's balance is proportional to its own peak of that same
    day. Participants would adjust their balances to the design in use; the experiment measures
    the short-run response with given balances. It uses 300,000 payments a day.
18. **Time structure of responses.** The base generator draws arrivals independently, so a
    participant's receipts right after it pays are there by chance. `echo-sweep` shows that the
    central result depends on this: fast bilateral returns make second-scale netting worthwhile.
    Echo share, delay and direction are assumptions; no public data identify them.
19. **Shape of the value distribution.** The generator matches the public mean ticket but not its
    percentiles: the SPI annual report 2025 gives p50 R$ 36, p90 R$ 400, p99 R$ 3,900 (December
    2025), while the model gives about R$ 91, 749 and 4,908. Typical payments are too large and
    too uniform. A fit is known (REVIEW 98) but not applied; results that depend on the size of
    single payments relative to liquidity (the delay bound, rejections under fixed balances) may
    move when it is.
