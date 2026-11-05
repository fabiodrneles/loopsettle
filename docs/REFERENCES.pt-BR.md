# Referências — situação de verificação

> 🇬🇧 [English version](REFERENCES.md)

> Base para a bibliografia dos artigos. Cada referência tem um **status**:
> **confirmada** (título, autores e ano conferidos em fonte pública), **parcial** (existe, mas
> algum detalhe ainda precisa ser conferido no texto integral) ou **não localizada** (não
> encontrada em busca pública; não citar até confirmar).
>
> Última conferência: 05/11/2026, por busca na web. Os PDFs do Banco da Finlândia não puderam
> ser abertos deste ambiente (rede bloqueada), então os números da seção 1 vêm de resumos e
> índices (EconStor, RePEc, Risk.net) e devem ser conferidos no texto integral antes da submissão.

## 1. Hellqvist & Korpinen — base metodológica

| Campo | Valor | Status |
|---|---|---|
| Versão 1 | Hellqvist, M. & Korpinen, K. (2021). *Instant payments as a new normal: Case study of liquidity impacts for the Finnish market*. Bank of Finland Economics Review 7/2021. | confirmada |
| Versão 2 | Hellqvist, M. & Korpinen, K. (2026). *Instant payments as the new normal: how much more money do the banks need?* Journal of Financial Market Infrastructures, 13(2), março de 2026. | confirmada |
| Dados | Dados sintéticos, por transação, que imitam os fluxos normais de transferências de varejo (*retail credit transfers*) entre os participantes finlandeses do sistema STEP2 (EBA Clearing). | confirmada (resumo) |
| Comparação | Necessidade de liquidez no modelo com ciclos (*cycle-based*) vs no modo instantâneo pleno. | confirmada (resumo) |
| Resultado agregado | Aumento agregado pequeno, sobretudo quando os pagamentos já eram divididos em vários lotes. | confirmada (resumo) |
| Número citado | Aumento diário da necessidade de liquidez de **no máximo 8,6%** com a migração completa. | **parcial**: vem do resumo do debatedor (Alexandrova-Kabadjova, Economics of Payments X, 2021), não do texto do artigo |
| Heterogeneidade | Variações entre bancos e entre dias podem ser grandes, mesmo com fluxos normais; daí a ênfase em reservas de liquidez (*buffers*). | confirmada (resumo) |
| Previsibilidade | A magnitude do aumento pode ser antecipada a partir de estatísticas conhecidas *ex ante* do fluxo de cada banco. | confirmada (resumo) |
| Topologia | Com liquidez suficiente, os resultados por banco não dependem da topologia da rede. | parcial (resumo do debatedor) |
| A conferir | Número de bancos, de dias e de simulações (os slides citam 15 bancos e Monte Carlo); quantos ciclos por dia no modelo de referência do STEP2; a definição exata de liquidez necessária. | pendente |

Fontes: [EconStor](https://www.econstor.eu/handle/10419/243289) ·
[RePEc/IDEAS (BoFER 7/2021)](https://ideas.repec.org/p/zbw/bofecr/72021.html) ·
[Risk.net (JFMI)](https://www.risk.net/node/7963364) ·
[PDF BoFER 7/2021](https://publications.bof.fi/bitstream/10024/43594/1/BoFER_7_2021.pdf) ·
[slides dos autores](https://www.suomenpankki.fi/globalassets/en/research/seminars-and-conferences/conferences-and-workshops/documents/eopx2021/korpinen_liquidity-impacts-of-migration-to-instant-payments_presentation.pdf) ·
[slides do debatedor](https://www.suomenpankki.fi/globalassets/en/research/seminars-and-conferences/conferences-and-workshops/documents/eopx2021/alexandrova-kabadjova_discussion_instant-payments-as-a-new-normal_presentation.pdf)

## 2. Estudo do Pix — BCB Working Paper 619

| Campo | Valor | Status |
|---|---|---|
| Referência | Gonzalez, R. B., Ma, Y. & Zeng, Y. (2025). *The Effect of Instant Payments on the Banking System*. Banco Central do Brasil, Working Paper Series 619, março de 2025. | confirmada |
| Dados | Dados administrativos bancários e dados do Pix por transação (não públicos). | confirmada (resumo) |
| Achado | Pagamentos instantâneos tiram dos bancos a capacidade de adiar e compensar pagamentos; os bancos passam a reter mais ativos líquidos (sobretudo títulos públicos), o que reduz a intermediação de crédito. Identificação causal com instrumento baseado em *timeouts* passivos de pagamento. | confirmada (resumo) |
| A conferir | A definição exata da medida de netting perdido (a "capacidade potencial" que o LoopSettle reproduz em estilo) e os números de 57% a 80% citados em conversas anteriores. | **pendente** |
| Ressalva | O texto não representa necessariamente a visão oficial do BCB (aviso do próprio paper). | confirmada |

Fontes: [PDF no BCB](https://bcb.gov.br/content/publicacoes/WorkingPaperSeries/WP619.pdf) ·
[RePEc](https://econpapers.repec.org/RePEc:bcb:wpaper:619)

## 3. CDFD — decomposição circular-direcional

| Campo | Valor | Status |
|---|---|---|
| Referência | Homs-Dones, M., MacKay, R. S., Sansom, B. & Zhou, Y. (2025). *Circular Directional Flow Decomposition of Networks*. arXiv:2506.12546 [physics.soc-ph]. | confirmada |
| Conteúdo | Separa o fluxo em parte circular e parte acíclica; índice de circularidade entre 0 e 1; dois referenciais: **circularidade máxima** (implementada no LoopSettle) e **BFF — Balanced Flow Forwarding** (variante proporcional/local, **não** implementada). | confirmada |
| Versão em periódico | Uma versão anterior do README citava *"How circular is a directed network? A flow decomposition approach"*, *Royal Society Open Science* (2026). | **não localizada**: até confirmar, citar o arXiv |

Fontes: [arXiv](https://arxiv.org/abs/2506.12546) · [WRAP Warwick](https://wrap.warwick.ac.uk/190782)

## 4. Bech & Soramäki — resolução de *gridlock*

| Campo | Valor | Status |
|---|---|---|
| Referência | Bech, M. L. & Soramäki, K. (2001). *Gridlock Resolution in Interbank Payment Systems*. Bank of Finland Discussion Papers 9/2001. | confirmada (referência clássica) |

## 5. PixSim

| Campo | Valor | Status |
|---|---|---|
| Referência | "PixSim", arXiv 2609.30684 (citado no `CLAUDE.md` como exemplo de calibração com fonte por parâmetro). | **não localizada**: nenhuma busca pública encontrou esse trabalho. **Não citar** até alguém confirmar o identificador. Saiu do README. |

## 6. Ferramentas citadas em `docs/NOVELTY.pt-BR.md`

BoF-PSS (simulador do Banco da Finlândia) e PSSimPy: conferir versão, licença e
funcionalidades antes de usá-los na tabela de novidade do artigo.
