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
9. **Proportional CDFD not implemented** (D6); the ladder uses only the maximum CDFD.
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
