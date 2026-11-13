# PARAMETERS — parâmetros do gerador sintético

> [English version](PARAMETERS.md)

Status: **SOURCED** (vem diretamente de dado público), **CALIBRATED** (ajustado para
reproduzir um agregado público) ou **ASSUMPTION** (escolha de modelagem sem fonte direta).

Código: `crates/loopsettle-sim/src/scenario.rs` (`ScenarioConfig`, `AmountModel`).
Os valores **CALIBRATED/SOURCED** são preenchidos na Fase 4 a partir de
`data/calibration/pix.toml` (gerado por `data/fetch_pix.py`).

| Parâmetro | Padrão | Status | Fonte / justificativa |
|---|---|---|---|
| `participants` | 200 | ASSUMPTION | Escala reduzida; o SPI tinha 907 participantes no fim de 2025, 214 diretos e 693 indiretos (relatório anual do SPI de 2025; os indiretos emitem cerca de 7% dos Pix interbancários). Os 200 maiores concentram a maior parte do volume. |
| `days` | 1 | ASSUMPTION | Período de análise = 1 dia (LB = posição líquida do dia). |
| `payments_per_day` | 1.000.000 | ASSUMPTION | Orçamento da máquina (seção 2). O fator de escala é registrado em `scale_factor`. |
| `scale_factor` | 238,99 | CALIBRATED | = 238.990.813 Pix/dia (set/2026) ÷ 1.000.000 simulados. Cada pagamento simulado "representa" ~239 pagamentos reais; a liquidez absoluta NÃO escala linearmente (ver limitações). |
| `zipf_s` | 1,1 | ASSUMPTION (substituído por `shares`) | Concentração de tamanhos quando não há participações calibradas. |
| `shares` | 200 maiores de 862 ISPBs | SOURCED (proxy) | Chaves Pix por ISPB em 2026-09-30 (`ChavesPix`). ASSUMPTION embutida: participação em transações ∝ participação em chaves. Os 200 maiores detêm 99,3% das chaves; renormalizado. |
| `n_big`, `n_mid` | 5, 15 | ASSUMPTION | Classes por ranking (grandes bancos, bancos médios). |
| `fintech_frac` | 0,5 | ASSUMPTION | Divisão do restante entre fintechs e instituições de pagamento. |
| `payee_size_exp` (β) | 1,0 | ASSUMPTION | Recebedor sorteado ∝ tamanho^β ("gravidade"). |
| `homophily` | 0,3 | ASSUMPTION | Peso extra para recebedor da mesma classe. |
| `receiver_bias_sigma` | 0,4 (na maioria dos experimentos); **1,0 calibrado** | ASSUMPTION → **CALIBRATED** (faixa 0,8–1,2) | Viés recebedor lognormal por participante: gera posições líquidas não nulas (fluxo direcional). Calibrado nos quartis da razão de pagamentos compensáveis por banco do BCB WP 619, eq. (3.2), com dados reais do Pix (p25 18–45%, mediana 57–80%, p75 73–90%): σ = 1,0 põe os três quartis dentro (36% / 61% / 83%) (`results/bias-calibration/`). O padrão continua 0,4 por continuidade (D19). |
| `pi_receiver_mult` | 1,3 | ASSUMPTION | IPs (credenciadores, P2B) recebem relativamente mais. |
| `hourly` | perfil de 24 pesos | ASSUMPTION | Noite baixa, platô diurno, pico no fim da tarde. Ainda sem série horária pública usada (o relatório anual do SPI 2024 tem uma: W0 nº 16). |
| `profile_strength` (λ) | 0 | ASSUMPTION | Intensidade dos perfis intradiários por classe (W2, D18); 0 = todos seguem `hourly`. Variado em 0 / 0,5 / 1 no `heterogeneity`. |
| `CLASS_OUT_PROFILE`, `CLASS_IN_PROFILE` | ver `scenario.rs` | ASSUMPTION | Só formatos: folha de pagamento de manhã (grandes bancos), varejo à noite (fintechs), repasse noturno e recebimento diurno das credenciadoras (instituições de pagamento). Reescalados para que cada classe mantenha seu volume diário. |
| `amounts.body_median` | R$ 35,58 (calibrado) | CALIBRATED | Média do corpo = (1 − 0,5205) · 449,83 / (1 − 0,003) = R$ 216,3 com σ = 1,9. |
| `amounts.body_sigma` | 1,9 (calibrado; 1,4 antes da D30) | CALIBRATED | Ajustado junto com `tail_prob` aos percentis públicos p50, p90, p99 do valor de um Pix interbancário (R$ 36 / 400 / 3.900, *Relatório Anual do SPI 2025*, p. 10, dez/2025): mínimos quadrados das razões em log numa grade, mantendo o ticket médio e a parcela de valor da cauda (`calibrate::fit_percentiles`, D30). |
| `amounts.tail_prob` | 0,003 (calibrado; 0,04296 = participação do B2B na quantidade antes da D30) | CALIBRATED | Fração da quantidade na cauda de pagamentos grandes, ajustada junto com `body_sigma` (D30). A participação do B2B na quantidade (4,3%) não a define mais: a maioria dos pagamentos B2B não é grande. `scenario.amounts_fit = "mean"` volta à regra antiga. |
| `amounts.tail_xm` | R$ 22.424 (calibrado) | CALIBRATED | Resolve (bisseção) média da cauda = 0,5205 · 449,83 / 0,003 = R$ 78.051, isto é, a cauda carrega a participação do B2B no valor (52,05%). |
| `amounts.tail_alpha` | 1,3 | ASSUMPTION | Expoente da cauda de Pareto. |
| `amounts.max_amount` | R$ 5.000.000 | ASSUMPTION | Truncamento da cauda. |
| Outage: re-tentativa | 100% re-enviados no fim | ASSUMPTION | Experimento `stress`: nenhum pagamento abandonado. |
| `hourly_profile = "spi2024"` | (opção) | SOURCED | Média de Pix por faixa de horário em 2024, BCB Relatório Anual do SPI 2024, Gráfico 12, p. 13 (`SPI_2024_HOURLY`); usado no `funding-windows`. |
| `day_weights` | (opção) | SOURCED | Volumes por tipo de dia em 2024 (seg 157,61 mi … sex 179,60 mi, sáb 157,72 mi, dom 113,54 mi), mesmo relatório, Gráfico 11, p. 12 (`SPI_2024_WEEKDAY`); usado no `funding-windows`. |
| Janela do STR | 6h30–19h nos dias úteis | SOURCED | 6h30–18h30 mais a janela de 30 minutos para aportes (mesmo relatório, p. 15); `funding-windows`. |
| Regime de alto valor | 20 participantes, mediana R$ 1 mi, cauda α 1,3 | ASSUMPTION | Sistema estilizado só para o `regimes`. |
| Liquidez restrita: novas tentativas | pagamentos rejeitados nunca são tentados de novo | ASSUMPTION | Só no `liquidity-frontier`. |
| Liquidez restrita: saldos | `α × UB_i` ou `α × ΣUB × saída_i / Σsaída`, α de 0,5 a 2 | ASSUMPTION | Só no `liquidity-frontier`; a regra `ub` olha para trás. |
| Pagamentos em eco | parcela 0 (desligado) por padrão; varredura de 5–50%, atraso de 1–60 s, adiante ou de volta | ASSUMPTION | Só no `echo-sweep`: teste de estresse da circularidade de curto prazo. |

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
| Pix por dia (escalado) | 238.990.813 | 238.900.714 | −0,04% |
| Valor por dia, R$ (escalado) | 107,5 bi | 109,6 bi | +1,91% |
| Ticket médio, R$ | 449,83 | 458,60 | +1,95% |
| Fração da quantidade na cauda (ajustada) | 0,003 | 0,00304 | +1,23% |
| Fração do valor na cauda (proxy B2B) | 0,5205 | 0,5291 | +1,64% |
| Valor p50, R$ (relatório do SPI, dez/2025) | 36 | 35,76 | −0,67% |
| Valor p90, R$ | 400 | 419,14 | +4,79% |
| Valor p99, R$ | 3.900 | 3.783,69 | −2,98% |
| Top 5 (participação) | 0,5603 | 0,5603 | +0,00% |
| Top 10 | 0,7895 | 0,7895 | −0,01% |
| Top 20 | 0,9221 | 0,9222 | +0,01% |
| HHI | 0,0860 | 0,0860 | +0,05% |

Notas honestas: (i) a concentração bate quase por construção, pois as participações
alimentam diretamente o sorteio do pagador; o que se valida é a implementação, não o modelo.
(ii) O ticket médio é sensível à cauda pesada (α = 1,3), agora carregada por 0,3% dos pagamentos:
o erro de +1,95% é ruído amostral de 1 milhão de sorteios (cerca de 3 mil deles na cauda; era
−1,5% antes da D30). (iii) Nada aqui calibra a **estrutura bilateral** (quem paga a quem),
que não é pública: ela é o principal `ASSUMPTION` do estudo. (iv) **Abrangência das contagens:
só entre instituições.** O recurso Olinda `EstatisticasTransacoesPix` "não inclui Pix liquidados
nos livros do participante, isto é, transações não enviadas para liquidação no SPI" (a própria
documentação): o Pix dentro da mesma instituição, que nunca toca a reserva, já está fora, como o
simulador supõe (instituição pagadora ≠ recebedora). O relatório anual do SPI de 2025 confirma a
ordem de grandeza: 7,10 bilhões de Pix interbancários em dezembro de 2025. (v) **Percentis de valor.** O
mesmo relatório dá p50 R$ 36, p90 R$ 400, p99 R$ 3.900 (dezembro de 2025). Antes da D30 o modelo
de valores dava cerca de R$ 91, 749 e 4.908; `body_sigma` e `tail_prob` agora são ajustados a eles
(tabela acima; REVIEW 98). Os percentis são de dezembro de 2025 e os outros alvos, de setembro de
2026 (`ASSUMPTION`: a forma muda devagar). Fontes: documentação Olinda
`Pix_DadosAbertos`; Banco Central do Brasil, *Relatório Anual do SPI 2025* (bcb.gov.br).
