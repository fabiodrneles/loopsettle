# Instant payment systems around the world — where LoopSettle applies

> [Versão em português](INSTANT_PAYMENTS.pt-BR.md)
>
> LoopSettle is calibrated to Brazil's Pix because Pix publishes the aggregates the generator
> needs. The question it asks — how much of the liquidity an instant payment system ties up is
> circular, and how much of it can actually be recovered — applies to every country with instant
> payments. This page shows which settlement model each system uses and which LoopSettle engine
> represents it.
>
> Last checked: 5 Nov 2026, by public web search; SPEI and Canada checked against primary sources (Banxico, Interac, Bank of Canada). Rows marked **to check** rely on secondary
> sources and must be confirmed against the operator's rulebook before being cited.

## The key distinction: how banks settle with each other

To the customer, every system below is instant: the payee is credited within seconds, 24/7. What
changes the **liquidity** banks must hold is how the banks settle among themselves:

| Interbank model | What it means | LoopSettle engine |
|---|---|---|
| **Real-time gross settlement (RTGS)** | each payment settles on its own, immediately, in central bank money; the payer's bank needs the funds *before* each payment | `GrossRealTime` |
| **Gross, against a prefunded joint account** | as above, but against a pool of funds the banks prefund at the central bank | `GrossRealTime` with a limited opening balance (constrained-liquidity mode) |
| **Deferred net settlement (DNS)** | banks exchange payments instantly but settle only their *net* positions, in a few cycles a day (often prefunded or collateralised) | `DeferredNet { cycles_per_day }` |
| **Netting every few seconds** | a clearing algorithm runs in quick succession and settles the payments the current balances allow | `AtomicMicroCycle` (closest analogue) |

LoopSettle's central result — *nettable volume is not saved liquidity* — is about systems in the
first two groups, which are the ones asking whether a liquidity-saving mechanism is worth adding.
For systems in the third group, the `finland-replication` experiment answers the reverse
question: how much more liquidity a move to RTGS would require.

## Systems

| System | Country / area | Operator | Live since | Interbank settlement | Engine | Status |
|---|---|---|---|---|---|---|
| **Pix** (SPI) | Brazil | Banco Central do Brasil | Nov 2020 | RTGS in the participants' Conta PI at the central bank; balances cannot go negative | `GrossRealTime` | confirmed |
| **FedNow** | United States | Federal Reserve | Jul 2023 | RTGS in Federal Reserve master accounts | `GrossRealTime` | confirmed |
| **RTP** | United States | The Clearing House | Nov 2017 | prefunded real-time gross settlement against a joint account at the New York Fed | `GrossRealTime` (limited balance) | confirmed |
| **TIPS** (SCT Inst) | Euro area | Eurosystem (ECB) | Nov 2018 | RTGS in euro central bank money, each payment in under 10 s | `GrossRealTime` | confirmed |
| **NPP** + Fast Settlement Service | Australia | NPP Australia / Reserve Bank of Australia | Feb 2018 | line-by-line real-time settlement of each payment in exchange settlement accounts, no netting | `GrossRealTime` | confirmed |
| **FAST** | Singapore | Banking industry / MAS | 2014 | real-time service; how FAST positions settle in MEPS+ is not settled by the sources found | — | **to check** |
| **SPEI** | Mexico | Banco de México | Aug 2004 | settlement by a clearing algorithm that repeatedly selects the sets of payments that can settle with participants' current balances — every 1.9 s on average, inside the system, in central bank money (Banxico) | `AtomicMicroCycle` (analogue) | confirmed (Banxico) |
| **UPI** | India | NPCI / Reserve Bank of India | 2016 | deferred net settlement: NPCI computes net positions and settles them in the RBI's RTGS, 10 cycles a day for authorised transactions | `DeferredNet { 10 }` | confirmed (secondary sources) |
| **Faster Payments** | United Kingdom | Pay.UK / Bank of England | May 2008 | deferred net settlement at the Bank of England, three cycles per business day, prefunded | `DeferredNet { 3 }` | confirmed (secondary sources) |
| **Interac e-Transfer** | Canada | Interac Corp. | 2003 | instant to the customer; deferred net settlement in 24-hour cycles: Interac computes each participant's amount and participants pay each other through Lynx (Canada's RTGS); migration to the RTR planned | `DeferredNet { 1 }` | confirmed (Interac disclosure, 2025) |
| **Real-Time Rail (RTR)** | Canada | Payments Canada | not yet live: phased launch targeted for Q4 2026 | real-time settlement against prefunded settlement accounts at the Bank of Canada, funded and defunded through Lynx | `GrossRealTime` (limited balance) | confirmed (Bank of Canada policy); launch pending |

## What this does and does not mean

- **It does mean** that LoopSettle's engines and metrics can be applied to these systems: the
  `GrossRealTime`, `DeferredNet` and `AtomicMicroCycle` engines are the mechanisms above.
- **It does not mean** that LoopSettle's numbers hold for them. The scenario is calibrated to Pix
  aggregates only (volume, ticket, B2B share, concentration). Payment sizes, intraday profiles
  and participant concentration differ by country. Re-calibrating to another system needs its
  own public aggregates (see [`PARAMETERS.md`](PARAMETERS.md) for what is needed), and the
  `CLAIMS.md` rules apply: no country-level liquidity figures without that calibration.
- **Timing targets differ.** The 2.8 s reference used in the results is the Pix target (50% of
  payments settled within 2.8 s). TIPS settles each payment in under 10 s.

## Sources

- Pix: [Circular BCB nº 4.027/2020 (summary)](https://www.migalhas.com.br/depeso/329524/banco-central-institui-o-sistema-de-pagamentos-instantaneos--spi--e-a-conta-pagamentos-instantaneos) ·
  [Olhar Digital, 3 years of Pix](https://olhardigital.com.br/2023/11/16/pro/tres-anos-do-pix-o-sistema-que-revolucionou-os-pagamentos-no-brasil/)
- FedNow: [Federal Reserve Bank of Cleveland, July 2023](https://clevelandfed.org/sp-20230712-update-on-the-federal-reserves-instant-payments-service) ·
  [FedNow product sheet](https://www.nafcu.org/sites/default/files/fednow-product-sheet.pdf)
- RTP: [Federal Reserve, interest on RTP joint balances](https://federalreserve.gov/paymentsystems/files/interest_on_rtp_joint_balances.pdf) ·
  [TCH implementation checklist](https://www.corporateone.coop/getmedia/74e17dcb-9bff-460e-b4da-c4b3c714ee23/RTP_Impl_Checklist_9_2020_vf-(6).pdf)
- TIPS: [ECB, TARGET instant payment service](https://www.ecb.europa.eu/paym/intro/news/html/ecb.mipnews170622.da.html) ·
  [Finextra, ECB launches TIPS](https://www.finextra.com/newsarticle/33043/ecb-launches-real-time-payments-system)
- NPP / FSS: [RBA Bulletin, Sep 2018](https://www.rba.gov.au/publications/bulletin/2018/sep/the-new-payments-platform-and-fast-settlement-service.html) ·
  [RBA, Fast Settlement Service](https://www.rba.gov.au/rits/info/fastsettlementservice.htm)
- FAST / MEPS+: [MAS, MEPS+](https://www.mas.gov.sg/news/media-releases/2006/successful-implementation-of-meps) ·
  [Mastercard, Singapore FAST](https://www.mastercard.com/am/en/news-and-trends/Insights/2022/singapore-fast.html)
- SPEI: [World Bank](https://openknowledge.worldbank.org/server/api/core/bitstreams/ac864d82-3b34-58b9-ba3a-163164442a20/content) ·
  [Banco de México at CEMLA, 2021](https://clon.cemla.org/actividades/2021-final/2021-09-ii-reunion-responsables-infraestructuras-de-mercados-financieros/4b.%20Moreno_CEMLA%20FPS%20Mexico%20SPEI%20BdeM.pdf)
- UPI: [NPCI, UPI settlement process](https://www.npci.org.in/PDF/npci/others/UPI-Settlement-Process.pdf) ·
  [Business Today, settlement cycles from 3 Nov 2025](https://www.businesstoday.in/personal-finance/news/story/upi-users-on-alert-settlement-rules-to-change-from-november-3-heres-what-can-be-expected-495471-2025-09-24)
- Faster Payments: [BIS CPSS 105 (UK)](https://www.bis.org/publ/cpss105_uk.pdf) ·
  [Pay.UK, net sender caps](https://www.wearepay.uk/understanding-changes-to-the-faster-payment-system-net-sender-caps/)
- SPEI (primary): [Banco de México, SPEI description](https://www.banxico.org.mx/sistemas-de-pago/d/%7B89B6CCF0-6070-7389-3DD5-B27AC4ECD9D1%7D.pdf)
- Interac e-Transfer (primary): [Interac, PPS self-assessment disclosure 2025](https://www.interac.ca/wp-content/uploads/2025/09/Interac-PPS-e-Transfer-Self-Assessment-2025-VF-Disclosure-Submission.pdf)
- RTR (primary): [Bank of Canada, settlement account access policy for the Real-Time Rail](https://www.bankofcanada.ca/core-functions/financial-system/bank-canadas-settlement-account-policies-for-payments-canada-payment-systems/bank-canada-settlement-account-access-policy-real-time-rail/)
- Canada: [Bank of Canada, payment systems it oversees](https://www.bankofcanada.ca/core-functions/financial-system/payment-systems/) ·
  [Bank of Canada designates Interac e-Transfer](https://www.finextra.com/pressarticle/83658/bank-of-canada-designates-interac-etransfer-as-a-prominent-payment-system) ·
  [Real-Time Rail on track for Q4 2026](https://platform.intelligence.regxelerator.com/updates/b02be15b633e5427f73a003899655469f4bc1a794ae83319c766dfcb8bacdf7d/payments-canada-confirms-real-time-rail-is-on-track-for-phased-q4-2026-launch-with-centralized-f)
