# PARAMETERS — parâmetros do gerador sintético

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
| `scale_factor` | 1 | CALIBRATED (Fase 4) | = transações reais por dia ÷ `payments_per_day`. |
| `zipf_s` | 1,1 | ASSUMPTION (substituído por `shares`) | Concentração de tamanhos quando não há participações calibradas. |
| `shares` | — | SOURCED (Fase 4) | Participação de cada instituição (proxy: chaves Pix por participante). |
| `n_big`, `n_mid` | 5, 15 | ASSUMPTION | Classes por ranking (grandes bancos, bancos médios). |
| `fintech_frac` | 0,5 | ASSUMPTION | Divisão do restante entre fintechs e instituições de pagamento. |
| `payee_size_exp` (β) | 1,0 | ASSUMPTION | Recebedor sorteado ∝ tamanho^β ("gravidade"). |
| `homophily` | 0,3 | ASSUMPTION | Peso extra para recebedor da mesma classe. |
| `receiver_bias_sigma` | 0,4 | ASSUMPTION | Viés recebedor lognormal por participante: gera posições líquidas não nulas (fluxo direcional). |
| `pi_receiver_mult` | 1,3 | ASSUMPTION | IPs (credenciadores, P2B) recebem relativamente mais. |
| `hourly` | perfil de 24 pesos | ASSUMPTION | Noite baixa, platô diurno, pico no fim da tarde. Sem série horária pública usada. |
| `amounts.body_median` | R$ 40 | ASSUMPTION → CALIBRATED (Fase 4) | Mediana do corpo lognormal. |
| `amounts.body_sigma` | 1,4 | ASSUMPTION | Dispersão do corpo. |
| `amounts.tail_prob` | 0,04 | CALIBRATED (Fase 4) | Ajustado para reproduzir o ticket médio público. |
| `amounts.tail_xm`, `tail_alpha` | R$ 1.000; 1,3 | ASSUMPTION | Cauda de Pareto (pagamentos PJ grandes). |
| `amounts.max_amount` | R$ 5.000.000 | ASSUMPTION | Truncamento da cauda. |
| Outage: re-tentativa | 100% re-enviados no fim | ASSUMPTION | Experimento `stress`: nenhum pagamento abandonado. |

## Processo de chegada

Poisson não homogêneo com taxa constante por hora (`hourly`), gerado de forma exata
(sorteio exponencial; ao cruzar a fronteira da hora, recomeça na fronteira — propriedade de
falta de memória). Pagador ∝ `activity`; recebedor por *alias table* por pagador.
