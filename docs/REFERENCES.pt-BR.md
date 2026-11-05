# Referências — situação de verificação

> [English version](REFERENCES.md)

> Base para a bibliografia dos artigos. Cada referência tem um **status**:
> **confirmada** (título, autores e ano conferidos em fonte pública), **parcial** (existe, mas
> algum detalhe ainda precisa ser conferido no texto integral) ou **não localizada** (não
> encontrada em busca pública; não citar até confirmar).
>
> Última conferência: 05/11/2026. As seções 1, 2, 3 e 4b foram conferidas nos **textos
> integrais** enviados pelo dono (os números de página abaixo são páginas do PDF). A versão do
> JFMI 2026 (seção 1, versão 2) não foi lida: só o texto de 2021.

## 1. Hellqvist & Korpinen — base metodológica

| Campo | Valor | Status |
|---|---|---|
| Versão 1 | Hellqvist, M. & Korpinen, K. (2021). *Instant payments as a new normal: Case study of liquidity impacts for the Finnish market*. Bank of Finland Economics Review 7/2021. | confirmada |
| Versão 2 | Hellqvist, M. & Korpinen, K. (2026). *Instant payments as the new normal: how much more money do the banks need?* Journal of Financial Market Infrastructures, 13(2), março de 2026. | confirmada |
| Dados | Dados sintéticos, por transação, que imitam os fluxos normais de transferências de varejo (*retail credit transfers*) entre os participantes finlandeses do sistema STEP2 (EBA Clearing). | confirmada (resumo) |
| Comparação | Necessidade de liquidez com os **6 ciclos de liquidação** diários do STEP2 (5 diurnos + 1 noturno) vs o modo instantâneo pleno; 1.000 dias sintéticos, transações Poisson-lognormal ajustadas às estatísticas mensais por participante do STEP2 (abril de 2020), duas topologias; também simularam de 1 a 6 ciclos com o BoF-PSS3 em 30 dias (p. 2, 4–5 do PDF). | confirmada (texto integral) |
| Definição | Liquidez necessária = os menores recursos que evitam qualquer rejeição: no modo instantâneo, a posição mínima ao longo do dia (o **limite superior** dos estudos de LBTR); com ciclos, a mínima entre as posições de fim de ciclo (o **limite inferior**, Koponen & Soramäki 1998). Mesma definição do UB/LB do LoopSettle (D3) (p. 3–4 do PDF). | confirmada (texto integral) |
| Resultado agregado | A necessidade diária de liquidez do sistema sobe **2,7% em média** e **menos de 8,6–8,7% em 95% dos dias** (€ 8,6 mi em média, < € 28 mi no p95, contra € 324 mi com 6 ciclos). Em 30 dias: um ciclo diário € 231 mi, instantâneo € 335 mi (instantâneo / 1 ciclo = 1,45×); 65% do aumento total ocorre no primeiro passo, de uma compensação diária para dois ciclos (p. 2, 9–11 do PDF). | confirmada (texto integral) |
| Número citado | "8,6%" é o **percentil 95**, não o máximo nem a média (a média é 2,7%); a base de comparação são 6 ciclos, não uma compensação diária. | corrigido (texto integral, p. 2 do PDF) |
| Heterogeneidade | Variações entre bancos e entre dias podem ser grandes, mesmo com fluxos normais; daí a ênfase em reservas de liquidez (*buffers*). | confirmada (resumo) |
| Previsibilidade | A magnitude do aumento pode ser antecipada a partir de estatísticas conhecidas *ex ante* do fluxo de cada banco. | confirmada (resumo) |
| Topologia | A topologia dos fluxos bilaterais não afeta o aumento por banco "pelo menos na configuração usada no nosso estudo". Os aumentos relativos são maiores para bancos cuja necessidade com ciclos é pequena (p. 2, 10–11 do PDF). | confirmada (texto integral) |
| Ainda aberto | Número exato de bancos (não lido nas partes conferidas) e o que mudou na versão do JFMI 2026. | pendente (menor) |

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
| Medida | Eq. (3.2), por banco: 2·Σ_d min(Saídas, Entradas) / Σ_d (Saídas + Entradas), diária, apresentada em quartis entre os bancos. Banco mediano **57% a 80%**, p25 18%–45%, p75 73%–90% no período (a partir de nov/2020) (p. 16 do PDF). A "capacidade potencial" do LoopSettle, Σ_i min / Σ_i saídas, é o **agregado ponderado por valor** da mesma fórmula; o `results/SUMMARY.pt-BR.md` agora também traz os quartis por banco (p25 63%, mediana 78%, p75 91%) para comparar igual com igual. | confirmada (texto integral) |
| Ressalva | O texto não representa necessariamente a visão oficial do BCB (aviso do próprio paper). | confirmada |

Fontes: [PDF no BCB](https://bcb.gov.br/content/publicacoes/WorkingPaperSeries/WP619.pdf) ·
[RePEc](https://econpapers.repec.org/RePEc:bcb:wpaper:619)

## 3. CDFD — decomposição circular-direcional

| Campo | Valor | Status |
|---|---|---|
| Referência | Homs-Dones, M., MacKay, R. S., Sansom, B. & Zhou, Y. (2025). *Circular Directional Flow Decomposition of Networks*. arXiv:2506.12546 [physics.soc-ph]. | confirmada |
| Conteúdo | Separa o fluxo em parte circular e parte acíclica; índice de circularidade entre 0 e 1; dois referenciais: **circularidade máxima** (implementada no LoopSettle) e **BFF — Balanced Flow Forwarding** (variante proporcional/local, **não** implementada). | confirmada |
| Circularidade máxima | Seção 5.1: o problema de custo mínimo (3) com todos os custos iguais a 1 — minimizar Σ d com 0 ≤ d ≤ w e os mesmos fluxos líquidos. Exatamente a formulação do LoopSettle (D5) (p. 9–10 do PDF). | confirmada (texto integral) |
| BFF | Seção 5.2, Algoritmo 1: remover sorvedouros, tomar o fluxo invariante maximal das proporções w̄ (ponto fixo de a_out(t+1) = min(a_out(t) w̄, a_out(t)), Teorema 3), subtrair, repetir. Versão em tempo polinomial no Apêndice F. Implementável para a D6 (p. 10–11 do PDF). | confirmada (texto integral) |
| Versão em periódico | Uma versão anterior do README citava *"How circular is a directed network? A flow decomposition approach"*, *Royal Society Open Science* (2026). | **não localizada**: até confirmar, citar o arXiv |

Fontes: [arXiv](https://arxiv.org/abs/2506.12546) · [WRAP Warwick](https://wrap.warwick.ac.uk/190782)

## 4. Bech & Soramäki — resolução de *gridlock*

| Campo | Valor | Status |
|---|---|---|
| Referência | Bech, M. L. & Soramäki, K. (2001). *Gridlock Resolution in Interbank Payment Systems*. Bank of Finland Discussion Papers 9/2001. | confirmada (referência clássica) |

## 4b. Johnson, McAndrews & Soramäki — mecanismos de liquidação diferida

| Campo | Valor | Status |
|---|---|---|
| Referência | Johnson, K., McAndrews, J. J. & Soramäki, K. (2004). *Economizing on liquidity with deferred settlement mechanisms*. FRBNY Economic Policy Review 10(3), 51–72. | confirmada (texto integral) |
| Método | Dados históricos do Fedwire; 20%, 50% ou 80% dos pagamentos enviados a compensação de uma hora, de seis horas ou a liquidação bruta reativa a recebimentos (RRGS). | confirmada |
| Resultados | Pico de descoberto frente ao LBTR: compensação de uma hora −1,9% / −4,7% / −5,6%; de seis horas −5,9% / −8,7% / −25,9%; RRGS +4,4% / +2,8% / −12,5% (descoberto médio −44% com 80%) (Tabela 3, p. 7 do PDF). | confirmada |
| Limite de atraso | O artigo **não** enuncia um limite para a economia de um mecanismo com prazo máximo (nenhum enunciado do tipo no texto integral). | confirmada (relevante para o critério de morte K2) |

## 5. PixSim

| Campo | Valor | Status |
|---|---|---|
| Referência | "PixSim", arXiv 2609.30684 (citado no `CLAUDE.md` como exemplo de calibração com fonte por parâmetro). | **não localizada**: nenhuma busca pública encontrou esse trabalho. **Não citar** até alguém confirmar o identificador. Saiu do README. |

## 6. Ferramentas citadas em `docs/NOVELTY.pt-BR.md`

BoF-PSS (simulador do Banco da Finlândia) e PSSimPy: conferir versão, licença e
funcionalidades antes de usá-los na tabela de novidade do artigo.
