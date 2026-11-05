# Biblioteca de referências

> [English version](README.md)

Esta pasta guarda os documentos que o LoopSettle cita, para que o estudo e os artigos possam ser
conferidos sem caçar cada fonte. O que cada documento estabelece (definições, números, páginas)
está registrado em [`../REFERENCES.pt-BR.md`](../REFERENCES.pt-BR.md); as entradas BibTeX de todos
estão em [`references.bib`](references.bib).

**Direitos autorais.** Cada PDF é redistribuído sem alteração, como publicado pelo autor ou pela
instituição, para pesquisa e conferência. Os direitos continuam com autores e editores. Cada linha
traz o link original: cite o original, não esta cópia. Se você detém os direitos de um documento e
quer que ele seja retirado, abra uma issue e ele será removido.

**O que não fica aqui.** Documento cuja licença não permite redistribuição só tem o link. É o caso
do preprint da CDFD (licença não exclusiva do arXiv): baixe-o no arXiv. Localmente ele pode ficar
em `docs/references/local/`, pasta que o git ignora.

## Fontes centrais (métodos e calibração)

| Arquivo | Referência | Original | Usado para |
|---|---|---|---|
| [`hellqvist-korpinen-2021-bofer-7-2021.pdf`](pdf/hellqvist-korpinen-2021-bofer-7-2021.pdf) | Hellqvist, M. & Korpinen, K. (2021). *Instant payments as a new normal: case study of liquidity impacts for the Finnish market*. BoF Economics Review 7/2021. | [publications.bof.fi](https://publications.bof.fi/bitstream/10024/43594/1/BoFER_7_2021.pdf) | base metodológica; definições de UB/LB; `finland-replication` |
| [`gonzalez-ma-zeng-2025-bcb-wp619.pdf`](pdf/gonzalez-ma-zeng-2025-bcb-wp619.pdf) | Gonzalez, R. B., Ma, Y. & Zeng, Y. (2025). *The Effect of Instant Payments on the Banking System*. BCB Working Paper 619. | [bcb.gov.br](https://www.bcb.gov.br/content/publicacoes/WorkingPaperSeries/WP619.pdf) | medida de compensação "potencial"; calibração de σ (W2b) |
| — (só link) | Homs-Dones, M., MacKay, R. S., Sansom, B. & Zhou, Y. (2025). *Circular Directional Flow Decomposition of Networks*. arXiv:2506.12546. | [arXiv](https://arxiv.org/abs/2506.12546) | CDFD máxima (D5); variante BFF (D6) |
| [`johnson-mcandrews-soramaki-2004-frbny-epr.pdf`](pdf/johnson-mcandrews-soramaki-2004-frbny-epr.pdf) | Johnson, K., McAndrews, J. J. & Soramäki, K. (2004). *Economizing on liquidity with deferred settlement mechanisms*. FRBNY Economic Policy Review 10(3). | [newyorkfed.org](https://www.newyorkfed.org/medialibrary/media/research/epr/04v10n3/0412mcanpdf.pdf) | evidência anterior sobre mecanismos diferidos; sem limite de atraso (K2) |
| [`mcmahon-2023-quantum-reordering-arxiv.pdf`](pdf/mcmahon-2023-quantum-reordering-arxiv.pdf) | McMahon, C., McGillivray, D., Desai, A., Rivadeneyra, F., Lam, J.-P., Lo, T., Marsden, D. & Skavysh, V. (2023). *Improving the efficiency of payments systems using quantum computing*. arXiv:2209.15392v3 (CC BY 4.0); Management Science (2024), doi 10.1287/mnsc.2023.00314. | [arXiv](https://arxiv.org/abs/2209.15392) | trabalho mais próximo: reordenação no LVTS do Canadá, dados reais |
| [`mcmahon-2024-t2-reordering-slides.pdf`](pdf/mcmahon-2024-t2-reordering-slides.pdf) | McMahon et al. (2024). *Payment reordering as a liquidity savings mechanism in T2*. 22º Simulator Seminar do BoF (slides). | [página do seminário](https://suomenpankki.fi/en/money-and-payments/payment-and-settlement-system-simulator/events/22nd-seminar-2024) | trabalho mais próximo (T2, alto valor) |
| [`bcb-2025-relatorio-anual-spi-2024.pdf`](pdf/bcb-2025-relatorio-anual-spi-2024.pdf) | Banco Central do Brasil (2025). *Relatório Anual do SPI 2024*. | [bcb.gov.br](https://www.bcb.gov.br/content/estabilidadefinanceira/relatorios_SPI/relatorio_anual_spi_2024.pdf) | perfis por hora e por dia da semana (SOURCED); janelas de abastecimento do STR (W3) |
| [`bcb-2023-relatorio-gestao-pix.pdf`](pdf/bcb-2023-relatorio-gestao-pix.pdf) | Banco Central do Brasil (2023). *Relatório de Gestão do Pix: concepção e primeiros anos de funcionamento (2020–2022)*. | [bcb.gov.br](https://www.bcb.gov.br/content/estabilidadefinanceira/pix/relatorio_de_gestao_pix/relatorio_gestao_pix_2023.pdf) | fontes de liquidez fora do horário do STR (W3) |

## Outros sistemas instantâneos e de alto valor

| Arquivo | Referência | Original | Usado para |
|---|---|---|---|
| [`rivadeneyra-zhang-2020-boc-sdp-2020-9.pdf`](pdf/rivadeneyra-zhang-2020-boc-sdp-2020-9.pdf) | Rivadeneyra, F. & Zhang, N. (2020). *Liquidity usage and payment delay estimates of the new Canadian high value payments system*. Bank of Canada SDP 2020-9. | [bankofcanada.ca](https://www.bankofcanada.ca/2020/09/staff-discussion-paper-2020-9/) | troca liquidez × atraso no Lynx |
| [`rivadeneyra-zhang-2022-boc-sdp-2022-3.pdf`](pdf/rivadeneyra-zhang-2022-boc-sdp-2022-3.pdf) | Rivadeneyra, F. & Zhang, N. (2022). *Payment coordination and liquidity efficiency in the new Canadian wholesale payments system*. Bank of Canada SDP 2022-3. | [bankofcanada.ca](https://www.bankofcanada.ca/2022/02/staff-discussion-paper-2022-3/) | mecanismo de compensação do Lynx |
| [`schneider-2018-ecb-tips-rt1-liquidity.pdf`](pdf/schneider-2018-ecb-tips-rt1-liquidity.pdf) | Schneider, M. (2018). *Instant payment services TIPS and RT1: implications for liquidity management*. ECB MMCG, 28/11/2018 (slides). | [ecb.europa.eu](https://www.ecb.europa.eu/paym/groups/pdf/mmcg/20181203/Item_5_2_Instant_Payment_Services_TIPS_and_RT1_Implications_for_Liquidity_Management.pdf) | abastecimento no fim de semana (W3) |
| [`frb-fednow-readiness-guide-liquidity.pdf`](pdf/frb-fednow-readiness-guide-liquidity.pdf) | Federal Reserve Financial Services. *FedNow readiness guide — preparing for 24x7 operations: managing liquidity in an instant payments world*. | [frbservices.org](https://www.frbservices.org/financial-services/fednow) | liquidez 24x7, horário do crédito intradia (W3) |
| [`hellqvist-2024-instant-payment-liquidity-risk-slides.pdf`](pdf/hellqvist-2024-instant-payment-liquidity-risk-slides.pdf) | Hellqvist, M. (2024). *Instant payment liquidity risk*. 22º Simulator Seminar do BoF (slides). | [página do seminário](https://suomenpankki.fi/en/money-and-payments/payment-and-settlement-system-simulator/events/22nd-seminar-2024) | quedas e risco de liquidez (estresse) |

## Outras palestras do 22º Simulator Seminar do BoF (2024) — só contexto

| Arquivo | Referência |
|---|---|
| [`heijmans-vanderwoerd-2024-failing-participant-slides.pdf`](pdf/heijmans-vanderwoerd-2024-failing-participant-slides.pdf) | Heijmans, R. & van der Woerd, E. *What is the impact of a failing participant in a LVPS on its environment and does time matter?* |
| [`mattern-meyer-2024-participant-criticality-slides.pdf`](pdf/mattern-meyer-2024-participant-criticality-slides.pdf) | Mattern, J. & Meyer, C. *Who is critical, how and when? A granular analysis of the criticality of participants in RTGS payment systems.* |
| [`drott-2024-dlt-cross-border-slides.pdf`](pdf/drott-2024-dlt-cross-border-slides.pdf) | Drott, C. *Think globally, settle locally? Multilateral platforms for cross-border payments based on DLT.* |
| [`polo-friz-rodloff-testi-2024-t2-analytics-slides.pdf`](pdf/polo-friz-rodloff-testi-2024-t2-analytics-slides.pdf) | Polo Friz, L., Rodloff, A. & Testi, S. *T2 analytics with simulations.* |
| [`vacirca-2024-initial-margin-procyclicality-slides.pdf`](pdf/vacirca-2024-initial-margin-procyclicality-slides.pdf) | Pank Roulund, R., Koci, R. & Vacirca, F. *An objective-based approach to limit initial margin procyclicality.* |
| [`brown-2024-retail-payment-markets-slides.pdf`](pdf/brown-2024-retail-payment-markets-slides.pdf) | Brown, M. *The dynamics of retail payment markets and consumer adoption of payment instruments.* |
| [`lubbersen-bolt-2024-card-rebates-slides.pdf`](pdf/lubbersen-bolt-2024-card-rebates-slides.pdf) | Lubbersen, V. & Bolt, W. *Rebates and competition between payment card networks.* |

Fonte de todas as palestras: [22nd Payment and Settlement System Simulator Seminar](https://suomenpankki.fi/en/money-and-payments/payment-and-settlement-system-simulator/events/22nd-seminar-2024),
Banco da Finlândia, Helsinque, 29–30 de agosto de 2024.

## Citados, mas não guardados

- Bech, M. L. & Soramäki, K. (2001). *Gridlock resolution in interbank payment systems*. Bank of
  Finland Discussion Papers 9/2001 ([IDEAS](https://ideas.repec.org/p/zbw/bofrdp/rdp2001_009.html)).
- Homs-Dones et al. (2025), CDFD — ver acima.
