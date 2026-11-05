# PARAMETERS — parâmetros do gerador sintético

> [English version](PARAMETERS.md)

Status: **SOURCED** (vem diretamente de dado público), **CALIBRATED** (ajustado para
reproduzir um agregado público) ou **ASSUMPTION** (escolha de modelagem sem fonte direta).

Código: `crates/loopsettle-sim/src/scenario.rs` (`ScenarioConfig`, `AmountModel`).
Os valores **CALIBRATED/SOURCED** são preenchidos na Fase 4 a partir de
`data/calibration/pix.toml` (gerado por `data/fetch_pix.py`).

| Parâmetro | Padrão | Status | Fonte / justificativa |
|---|---|---|---|
| `participants` | 200 | ASSUMPTION | Escala reduzida; o SPI tem ~900 participantes diretos+indiretos. Os 200 maiores concentram a maior parte do volume. |
| `days` | 1 | ASSUMPTION | Período de análise = 1 dia (LB = posição líquida do dia). |
| `payments_per_day` | 1.000.000 | ASSUMPTION | Orçamento da máquina (seção 2). O fator de escala é registrado em `scale_factor`. |
| `scale_factor` | 238,99 | CALIBRATED | = 238.990.813 Pix/dia (set/2026) ÷ 1.000.000 simulados. Cada pagamento simulado "representa" ~239 pagamentos reais; a liquidez absoluta NÃO escala linearmente (ver limitações). |
| `zipf_s` | 1,1 | ASSUMPTION (substituído por `shares`) | Concentração de tamanhos quando não há participações calibradas. |
| `shares` | 200 maiores de 862 ISPBs | SOURCED (proxy) | Chaves Pix por ISPB em 2026-09-30 (`ChavesPix`). ASSUMPTION embutida: participação em transações ∝ participação em chaves. Os 200 maiores detêm 99,3% das chaves; renormalizado. |
| `n_big`, `n_mid` | 5, 15 | ASSUMPTION | Classes por ranking (grandes bancos, bancos médios). |
| `fintech_frac` | 0,5 | ASSUMPTION | Divisão do restante entre fintechs e instituições de pagamento. |
| `payee_size_exp` (β) | 1,0 | ASSUMPTION | Recebedor sorteado ∝ tamanho^β ("gravidade"). |
| `homophily` | 0,3 | ASSUMPTION | Peso extra para recebedor da mesma classe. |
| `receiver_bias_sigma` | 0,4 | ASSUMPTION | Viés recebedor lognormal por participante: gera posições líquidas não nulas (fluxo direcional). |
| `pi_receiver_mult` | 1,3 | ASSUMPTION | IPs (credenciadores, P2B) recebem relativamente mais. |
| `hourly` | perfil de 24 pesos | ASSUMPTION | Noite baixa, platô diurno, pico no fim da tarde. Sem série horária pública usada. |
| `amounts.body_median` | R$ 84,58 (calibrado) | CALIBRATED | Resolve média do corpo = (1 − 0,5205)·449,83 / (1 − 0,04296) = R$ 225,4 com σ = 1,4. |
| `amounts.body_sigma` | 1,4 | ASSUMPTION | Dispersão do corpo. |
| `amounts.tail_prob` | 0,04296 | SOURCED (proxy) | = participação do B2B na quantidade de Pix (set/2026). A cauda representa pagamentos empresariais grandes. |
| `amounts.tail_xm` | R$ 1.375,26 (calibrado) | CALIBRATED | Resolve (bisseção) média da cauda = 0,5205·449,83/0,04296 = R$ 5.450, i.e. a cauda carrega a participação do B2B no valor (52,05%). |
| `amounts.tail_alpha` | 1,3 | ASSUMPTION | Expoente da cauda de Pareto. |
| `amounts.max_amount` | R$ 5.000.000 | ASSUMPTION | Truncamento da cauda. |
| Outage: re-tentativa | 100% re-enviados no fim | ASSUMPTION | Experimento `stress`: nenhum pagamento abandonado. |

## Processo de chegada

Poisson não homogêneo com taxa constante por hora (`hourly`), gerado de forma exata
(sorteio exponencial; ao cruzar a fronteira da hora, recomeça na fronteira — propriedade de
falta de memória). Pagador ∝ `activity`; recebedor por *alias table* por pagador.

## Alvos públicos (set/2026) e tabela real vs simulado

Fonte: `data/calibration/pix.toml` (gerado por `python data/fetch_pix.py --month 202609
--keys-date 2026-09-30`). Reproduzir a tabela:
`cargo run --release -p loopsettle-cli -- experiment calibration` → `results/calibration/`.

| Agregado | Real | Simulado | Erro relativo |
|---|---|---|---|
| Pix por dia (escalado) | 238.990.813 | 239.041.001 | +0,02% |
| Valor por dia, R$ (escalado) | 107,5 bi | 105,9 bi | −1,47% |
| Ticket médio, R$ | 449,83 | 443,14 | −1,49% |
| Fração da quantidade na cauda (proxy B2B) | 0,04296 | 0,04246 | −1,18% |
| Fração do valor na cauda (proxy B2B) | 0,5205 | 0,5142 | −1,21% |
| Top 5 (participação) | 0,5603 | 0,5598 | −0,09% |
| Top 10 | 0,7895 | 0,7893 | −0,03% |
| Top 20 | 0,9221 | 0,9216 | −0,05% |
| HHI | 0,0860 | 0,0859 | −0,09% |

Notas honestas: (i) a concentração bate quase por construção, pois as participações
alimentam diretamente o sorteio do pagador; o que se valida é a implementação, não o modelo.
(ii) O ticket médio é sensível à cauda pesada (α = 1,3): o erro de −1,5% é ruído amostral
de 1 milhão de sorteios. (iii) Nada aqui calibra a **estrutura bilateral** (quem paga a quem),
que não é pública: ela é o principal `ASSUMPTION` do estudo.
