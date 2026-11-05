# Resumo dos resultados

> [English version](SUMMARY.md)

Gerado por `python scripts/summarize.py` a partir de `results/*/`. Não edite à mão: rode `make all`.
Valores em reais são do cenário **sintético** (1 milhão de pagamentos/dia; cada pagamento simulado representa ~239 pagamentos reais). As frações são o resultado principal; valores absolutos não escalam linearmente para o Pix real.

## 1. Calibração (real vs simulado)
_seed `20260930`, código `0.1.0+git.7329107`, comando `cargo run --release -p loopsettle-cli -- experiment calibration`._

| agregado | real | simulado | erro relativo |
|---|---|---|---|
| daily_count (scaled) | 238 990 813 | 239 041 001 | 0.02% |
| daily_value_brl (scaled) | 107 504 810 068 | 105 928 420 096 | -1.47% |
| ticket_mean_brl | 449.8282 | 443.1391 | -1.49% |
| tail_count_share (B2B proxy) | 0.0430 | 0.0425 | -1.18% |
| tail_value_share (B2B proxy) | 0.5205 | 0.5142 | -1.21% |
| top5_share | 0.5603 | 0.5598 | -0.09% |
| top10_share | 0.7895 | 0.7893 | -0.03% |
| top20_share | 0.9221 | 0.9216 | -0.05% |
| hhi | 0.0860 | 0.0859 | -0.09% |

## 2. Baseline: liquidez e escada de circularidade
_seed `20260930`, código `0.1.0+git.7329107`, comando `cargo run --release -p loopsettle-cli -- experiment baseline`._

| medida | valor |
|---|---|
| pagamentos | 1000210 |
| volume bruto (R$) | 443 232 184 |
| UB — liquidez do bruto em tempo real (R$) | 71 271 128 |
| LB — posição líquida de fim de dia (R$) | 64 666 472 |
| LB / UB | 0.9073 |
| UB / volume bruto | 0.1608 |
| capacidade potencial (estilo BCB) | 85.41% |

Escada (frações do fluxo total do grafo; Hodge L1 pode passar de 1):

| grafo | arestas | Hodge L1 | Hodge L2 rot. | CDFD máx. | ciclos (guloso) | triângulos |
|---|---|---|---|---|---|---|
| gross | 10492 | 1.1835 | 0.9770 | 0.8315 | 0.7955 | 113417 |
| net | 6620 | 1.7301 | 0.9770 | 0.1742 | 0.1443 | 113417 |

Razão de pagamentos compensáveis por banco, definição do BCB WP 619 (eq. 3.2, 2·min(saídas, entradas)/(saídas + entradas)): p25 63.2%, mediana 78.3%, p75 90.8%. No Pix real, o WP 619 reporta mediana de 57% a 80%, p25 de 18% a 45% e p75 de 73% a 90% (nov/2020 em diante).

Correlação potencial de Hodge × posição líquida: 0.9971.

Sensibilidade ao viés recebedor (principal ASSUMPTION da estrutura bilateral):

| σ viés | UB (R$) | LB/UB | potencial | Hodge L1 bruto | CDFD bruto | ciclos bruto | CDFD líquido | economia micro-ciclo | p95 (ms) |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 29 689 174 | 0.591 | 96.04% | 1.038 | 0.958 | 0.934 | 0.644 | 0.0480% | 9984 |
| 0.2 | 49 241 487 | 0.798 | 91.13% | 1.105 | 0.907 | 0.864 | 0.320 | 0.0051% | 9984 |
| 0.4 | 71 271 128 | 0.907 | 85.41% | 1.183 | 0.831 | 0.795 | 0.174 | 0.0120% | 9984 |
| 0.8 | 110 276 702 | 0.950 | 76.35% | 1.304 | 0.696 | 0.669 | 0.068 | 0.0034% | 9984 |

Figura: [`baseline/fig_ladder.svg`](baseline/fig_ladder.svg)

## 3. Potencial vs realizável (comparação central)
_seed `20260930`, código `0.1.0+git.7329107`, comando `cargo run --release -p loopsettle-cli -- experiment potential-vs-realizable`._

| medida | valor | significado | status |
|---|---|---|---|
| potential_offset_fraction_bcb_style | 0.85410 | share of gross outflows fundable by same-day inflows, order ignored | measured, no time |
| potential_liquidity_saving_end_of_day | 0.09267 | (UB - LB) / UB: saving of one end-of-day net settlement | measured |
| static_ceiling_cdfd_max_gross | 0.83146 | max circulation respecting edge bounds / gross flow | measured, aggregated, no time |
| static_greedy_cycle_netting_gross | 0.79549 | greedy cycle cancellation / gross flow | measured, aggregated, no time |
| hodge_l1_cyclic_gross | 1.18348 | diagnostic; can exceed 1 | measured, aggregated, no time |
| realizable_liquidity_saving_micro_cycle_all_w500_d10000 | 0.00015 | best atomic micro-cycle config: 1 - liquidity / UB | measured in time |
| realizable_value_in_cycles_micro_cycle_all_w500_d10000 | 0.63425 | value settled inside atomic cycles / settled value | measured in time |
| realizable_liquidity_saving_deferred_net_96x | 0.00325 | deferred net settlement at this frequency | measured in time |
| realizable_liquidity_saving_deferred_net_8640x | 0.00028 | deferred net settlement at this frequency | measured in time |

Por desenho de liquidação (liquidez ilimitada; espera máxima em segundos):

| espera máx. (s) | motor | liquidez (R$) | economia vs UB | fração de UB−LB recuperada | p50 (ms) | p95 (ms) | ≤ 2,8 s | valor em ciclos |
|---|---|---|---|---|---|---|---|---|
| 0 | gross_real_time | 71 271 128 | 0.0000% | 0.0000 | 0 | 0 | 100.00% | 0.00% |
| 86400 | deferred_net_1x | 64 666 472 | 9.2669% | 1.0000 | 31457280 | 60293120 | 0.00% | 0.00% |
| 43200 | deferred_net_2x | 68 150 773 | 4.3781% | 0.4724 | 18350080 | 40894464 | 0.01% | 0.00% |
| 21600 | deferred_net_4x | 68 602 509 | 3.7443% | 0.4041 | 10616832 | 20447232 | 0.01% | 0.00% |
| 8640 | deferred_net_10x | 69 592 930 | 2.3547% | 0.2541 | 4259840 | 8192000 | 0.03% | 0.00% |
| 3600 | deferred_net_24x | 70 519 539 | 1.0545% | 0.1138 | 1785856 | 3407872 | 0.08% | 0.00% |
| 900 | deferred_net_96x | 71 039 610 | 0.3248% | 0.0351 | 446464 | 851968 | 0.32% | 0.00% |
| 300 | deferred_net_288x | 71 163 915 | 0.1504% | 0.0162 | 149504 | 282624 | 0.96% | 0.00% |
| 60 | deferred_net_1440x | 71 220 418 | 0.0712% | 0.0077 | 29952 | 56832 | 4.70% | 0.00% |
| 10 | deferred_net_8640x | 71 251 432 | 0.0276% | 0.0030 | 4992 | 9472 | 28.11% | 0.00% |
| 10 | micro_cycle_all_w500_d10000 | 71 260 662 | 0.0147% | 0.0016 | 4672 | 9984 | 43.06% | 63.42% |
| 10 | micro_cycle_all_w1000_d10000 | 71 262 596 | 0.0120% | 0.0013 | 4864 | 9984 | 42.12% | 62.66% |
| 5 | micro_cycle_all_w1000_d5000 | 71 268 147 | 0.0042% | 0.0005 | 4352 | 4992 | 41.62% | 53.51% |
| 2 | micro_cycle_all_w500_d2000 | 71 269 961 | 0.0016% | 0.0002 | 2000 | 2000 | 100.00% | 36.23% |

Figura: [`potential-vs-realizable/fig_potential_vs_realizable.svg`](potential-vs-realizable/fig_potential_vs_realizable.svg)

## 4. Micro-ciclos: economia × atraso
_seed `20260930`, código `0.1.0+git.7329107`, comando `cargo run --release -p loopsettle-cli -- experiment microcycle-sweep`._

| janela (ms) | atraso máx. (ms) | economia vs UB | p50 (ms) | p95 (ms) | p99 (ms) | ≤ 2,8 s | valor em ciclos |
|---|---|---|---|---|---|---|---|
| 500 | 2000 | 0.0016% | 2000 | 2000 | 2000 | 100.00% | 36.23% |
| 1000 | 2000 | 0.0011% | 2000 | 2000 | 2000 | 100.00% | 32.22% |
| 2000 | 2000 | 0.0005% | 2000 | 2000 | 2000 | 100.00% | 25.36% |
| 5000 | 2000 | 0.0001% | 2000 | 2000 | 2000 | 100.00% | 10.04% |
| 10000 | 2000 | 0.0000% | 2000 | 2000 | 2000 | 100.00% | 4.98% |
| 500 | 5000 | 0.0036% | 4096 | 4992 | 4992 | 43.16% | 54.96% |
| 1000 | 5000 | 0.0042% | 4352 | 4992 | 4992 | 41.62% | 53.51% |
| 2000 | 5000 | 0.0044% | 4992 | 4992 | 4992 | 38.13% | 50.61% |
| 5000 | 5000 | 0.0047% | 4992 | 4992 | 4992 | 22.09% | 41.06% |
| 10000 | 5000 | 0.0002% | 4992 | 4992 | 4992 | 11.05% | 21.45% |
| 500 | 10000 | 0.0147% | 4672 | 9984 | 9984 | 43.06% | 63.42% |
| 1000 | 10000 | 0.0120% | 4864 | 9984 | 9984 | 42.12% | 62.66% |
| 2000 | 10000 | 0.0141% | 5376 | 9984 | 9984 | 39.92% | 61.81% |
| 5000 | 10000 | 0.0053% | 6912 | 9984 | 9984 | 24.03% | 58.44% |
| 10000 | 10000 | 0.0078% | 9600 | 9984 | 9984 | 13.43% | 51.89% |

Liquidez restrita (saldo inicial = fração do UB de cada participante):

| fração do UB | motor | valor rejeitado | p95 (ms) | ≤ 2,8 s | valor em ciclos |
|---|---|---|---|---|---|
| 0.25 | gross_real_time | 22.95% | 0 | 100.00% | 0.00% |
| 0.25 | micro_cycle_shortfall_w1000_d10000 | 23.23% | 920 | 96.20% | 0.29% |
| 0.25 | micro_cycle_all_w1000_d10000 | 22.43% | 9984 | 42.50% | 51.43% |
| 0.5 | gross_real_time | 12.70% | 0 | 100.00% | 0.00% |
| 0.5 | micro_cycle_shortfall_w1000_d10000 | 12.66% | 0 | 97.95% | 0.10% |
| 0.5 | micro_cycle_all_w1000_d10000 | 12.46% | 9984 | 42.52% | 56.72% |
| 0.75 | gross_real_time | 5.82% | 0 | 100.00% | 0.00% |
| 0.75 | micro_cycle_shortfall_w1000_d10000 | 5.81% | 0 | 99.22% | 0.00% |
| 0.75 | micro_cycle_all_w1000_d10000 | 5.82% | 9984 | 42.35% | 59.81% |

Figura: [`microcycle-sweep/fig_tradeoff.svg`](microcycle-sweep/fig_tradeoff.svg)

## 5. Estresse: maior participante fora do ar por 30 minutos
_seed `20260930`, código `0.1.0+git.7329107`, comando `cargo run --release -p loopsettle-cli -- experiment stress`._

Participante 0 fora de 10.0h por 30 min: 11692 pagamentos (R$ 5 866 760) adiados, em média 903 s.

| cenário | UB total (R$) | UB do participante (R$) | motor | liquidez (R$) | valor rejeitado | p95 (ms) |
|---|---|---|---|---|---|---|
| baseline | 71 271 128 | 3 137 734 | gross_real_time | 71 271 128 | 0.0000% | 0 |
| baseline | 71 271 128 | 3 137 734 | micro_cycle_all_w1000_d10000 | 71 262 596 | 0.0000% | 9984 |
| baseline | 71 271 128 | 3 137 734 | micro_cycle_shortfall_w1000_d10000 | 71 271 128 | 0.0000% | 0 |
| baseline | 71 271 128 | 3 137 734 | deferred_net_10x | 69 592 930 | 0.0000% | 8192000 |
| outage | 71 274 781 | 3 137 734 | gross_real_time | 71 274 781 | 0.0000% | 0 |
| outage | 71 274 781 | 3 137 734 | micro_cycle_all_w1000_d10000 | 71 265 015 | 0.0000% | 9984 |
| outage | 71 274 781 | 3 137 734 | micro_cycle_shortfall_w1000_d10000 | 71 274 781 | 0.0000% | 0 |
| outage | 71 274 781 | 3 137 734 | deferred_net_10x | 69 592 930 | 0.0000% | 8192000 |
| outage_constrained_1xUB | 71 274 781 | 3 137 734 | gross_real_time | 71 263 981 | 0.0027% | 0 |
| outage_constrained_1xUB | 71 274 781 | 3 137 734 | micro_cycle_all_w1000_d10000 | 71 258 821 | 0.0015% | 9984 |
| outage_constrained_1xUB | 71 274 781 | 3 137 734 | micro_cycle_shortfall_w1000_d10000 | 71 263 981 | 0.0027% | 0 |
| outage_constrained_1xUB | 71 274 781 | 3 137 734 | deferred_net_10x | 69 592 930 | 0.0000% | 8192000 |

## 6. Lógica do estudo finlandês: ciclos fixos vs instantâneo
_seed `20260930`, código `0.1.0+git.7329107`, comando `cargo run --release -p loopsettle-cli -- experiment finland-replication`._

Razão liquidez instantânea / liquidez com N ciclos por dia (média sobre sementes):

| σ viés | ciclos/dia | razão média | dp | mín | máx | sementes |
|---|---|---|---|---|---|---|
| 0 | 1 | 1.496 | 0.177 | 1.327 | 1.691 | 5 |
| 0 | 2 | 1.301 | 0.139 | 1.138 | 1.487 | 5 |
| 0 | 4 | 1.188 | 0.058 | 1.111 | 1.235 | 5 |
| 0 | 6 | 1.137 | 0.054 | 1.084 | 1.212 | 5 |
| 0 | 10 | 1.113 | 0.035 | 1.071 | 1.152 | 5 |
| 0 | 24 | 1.046 | 0.010 | 1.036 | 1.058 | 5 |
| 0 | 48 | 1.031 | 0.008 | 1.021 | 1.040 | 5 |
| 0.2 | 1 | 1.199 | 0.047 | 1.130 | 1.252 | 5 |
| 0.2 | 2 | 1.133 | 0.035 | 1.092 | 1.165 | 5 |
| 0.2 | 4 | 1.093 | 0.020 | 1.063 | 1.112 | 5 |
| 0.2 | 6 | 1.072 | 0.021 | 1.043 | 1.096 | 5 |
| 0.2 | 10 | 1.046 | 0.012 | 1.027 | 1.056 | 5 |
| 0.2 | 24 | 1.031 | 0.019 | 1.018 | 1.063 | 5 |
| 0.2 | 48 | 1.015 | 0.005 | 1.010 | 1.022 | 5 |
| 0.4 | 1 | 1.095 | 0.049 | 1.036 | 1.169 | 5 |
| 0.4 | 2 | 1.066 | 0.034 | 1.032 | 1.122 | 5 |
| 0.4 | 4 | 1.044 | 0.016 | 1.026 | 1.069 | 5 |
| 0.4 | 6 | 1.039 | 0.014 | 1.023 | 1.061 | 5 |
| 0.4 | 10 | 1.026 | 0.007 | 1.015 | 1.034 | 5 |
| 0.4 | 24 | 1.021 | 0.013 | 1.010 | 1.041 | 5 |
| 0.4 | 48 | 1.008 | 0.002 | 1.006 | 1.009 | 5 |

Figura: [`finland-replication/fig_finland.svg`](finland-replication/fig_finland.svg)

## 7. Robustez: sementes, escala, comprimento do ciclo e limite de atraso
_seed `20260930`, código `0.1.0+git.b5a639d`, comando `cargo run --release -p loopsettle-cli -- experiment robustness`._

R1 — várias sementes × viés recebedor (média, desvio-padrão e intervalo de 95%; limite = teto para qualquer mecanismo com esse atraso máximo):

| σ viés | motor | sementes | economia média | dp (p.p.) | IC 95% | limite de atraso | LB/UB |
|---|---|---|---|---|---|---|---|
| 0 | micro:all:500:2000 | 10 | 0.0057% | 0.0065 | [0.0010; 0.0103]% | 2.175% | 0.697 |
| 0 | micro:all:500:10000 | 10 | 0.0178% | 0.0329 | [-0.0057; 0.0414]% | 3.338% | 0.697 |
| 0 | deferred:8640 | 10 | 0.0563% | 0.0158 | [0.0450; 0.0676]% | 3.338% | 0.697 |
| 0 | deferred:96 | 10 | 1.9168% | 0.4515 | [1.5938; 2.2397]% | 25.450% | 0.697 |
| 0 | deferred:1 | 10 | 30.3105% | 7.2141 | [25.1502; 35.4708]% | 727.995% | 0.697 |
| 0.4 | micro:all:500:2000 | 10 | 0.0006% | 0.0009 | [0.0000; 0.0013]% | 0.540% | 0.922 |
| 0.4 | micro:all:500:10000 | 10 | 0.0116% | 0.0051 | [0.0079; 0.0152]% | 0.800% | 0.922 |
| 0.4 | deferred:8640 | 10 | 0.0185% | 0.0079 | [0.0129; 0.0242]% | 0.800% | 0.922 |
| 0.4 | deferred:96 | 10 | 0.5021% | 0.1736 | [0.3779; 0.6262]% | 8.188% | 0.922 |
| 0.4 | deferred:1 | 10 | 7.8040% | 4.6106 | [4.5060; 11.1020]% | 344.062% | 0.922 |
| 0.8 | micro:all:500:2000 | 10 | 0.0002% | 0.0006 | [-0.0002; 0.0007]% | 0.643% | 0.965 |
| 0.8 | micro:all:500:10000 | 10 | 0.0033% | 0.0040 | [0.0005; 0.0062]% | 0.696% | 0.965 |
| 0.8 | deferred:8640 | 10 | 0.0080% | 0.0050 | [0.0045; 0.0116]% | 0.696% | 0.965 |
| 0.8 | deferred:96 | 10 | 0.3231% | 0.3655 | [0.0617; 0.5845]% | 4.035% | 0.965 |
| 0.8 | deferred:1 | 10 | 3.4763% | 2.6393 | [1.5884; 5.3642]% | 222.628% | 0.965 |
| 1 | micro:all:500:2000 | 10 | -0.0006% | 0.0019 | [-0.0019; 0.0008]% | 0.769% | 0.977 |
| 1 | micro:all:500:10000 | 10 | 0.0039% | 0.0063 | [-0.0006; 0.0085]% | 0.889% | 0.977 |
| 1 | deferred:8640 | 10 | 0.0055% | 0.0032 | [0.0032; 0.0078]% | 0.889% | 0.977 |
| 1 | deferred:96 | 10 | 0.2047% | 0.2217 | [0.0461; 0.3632]% | 3.627% | 0.977 |
| 1 | deferred:1 | 10 | 2.3371% | 1.7073 | [1.1158; 3.5584]% | 188.333% | 0.977 |

R2 — escala (mesmo cenário, de 1 a 30 milhões de pagamentos por dia):

| pagamentos/dia | fator de escala | motor | economia vs UB | limite de atraso | UB / volume |
|---|---|---|---|---|---|
| 1 000 000 | 239.0 | micro:all:500:2000 | 0.0016% | 0.567% | 0.1608 |
| 1 000 000 | 239.0 | micro:all:500:10000 | 0.0147% | 0.761% | 0.1608 |
| 1 000 000 | 239.0 | deferred:8640 | 0.0276% | 0.761% | 0.1608 |
| 1 000 000 | 239.0 | deferred:96 | 0.3248% | 7.956% | 0.1608 |
| 1 000 000 | 239.0 | deferred:1 | 9.2669% | 275.951% | 0.1608 |
| 3 000 000 | 79.7 | micro:all:500:2000 | 0.0005% | 0.388% | 0.1501 |
| 3 000 000 | 79.7 | micro:all:500:10000 | 0.0027% | 0.439% | 0.1501 |
| 3 000 000 | 79.7 | deferred:8640 | 0.0092% | 0.439% | 0.1501 |
| 3 000 000 | 79.7 | deferred:96 | 0.4132% | 6.190% | 0.1501 |
| 3 000 000 | 79.7 | deferred:1 | 5.7640% | 233.968% | 0.1501 |
| 10 000 000 | 23.9 | micro:all:500:2000 | 0.0002% | 0.219% | 0.1398 |
| 10 000 000 | 23.9 | micro:all:500:10000 | -0.0004% | 0.276% | 0.1398 |
| 10 000 000 | 23.9 | deferred:8640 | 0.0078% | 0.276% | 0.1398 |
| 10 000 000 | 23.9 | deferred:96 | 0.2188% | 3.763% | 0.1398 |
| 10 000 000 | 23.9 | deferred:1 | 2.9492% | 257.365% | 0.1398 |
| 30 000 000 | 8.0 | micro:all:500:2000 | 0.0001% | 0.080% | 0.1361 |
| 30 000 000 | 8.0 | micro:all:500:10000 | -0.0016% | 0.271% | 0.1361 |
| 30 000 000 | 8.0 | deferred:8640 | 0.0052% | 0.271% | 0.1361 |
| 30 000 000 | 8.0 | deferred:96 | 0.1610% | 3.405% | 0.1361 |
| 30 000 000 | 8.0 | deferred:1 | 1.6789% | 248.574% | 0.1361 |

R3 — comprimento máximo do micro-ciclo (janela 0,5 s, atraso 10 s):

| comprimento máx. | economia vs UB | valor em ciclos |
|---|---|---|
| 2 | 0.0155% | 45.45% |
| 3 | 0.0100% | 61.08% |
| 6 | 0.0147% | 63.42% |
| 10 | 0.0145% | 63.47% |

R5 — razão por participante: liquidez instantânea / liquidez com N ciclos por dia:

| ciclos/dia | participantes com razão | só precisam no instantâneo | não precisam | p10 | mediana | p90 | máx. |
|---|---|---|---|---|---|---|---|
| 1 | 78 | 104 | 18 | 1.001 | 1.161 | 2.897 | 19.11 |
| 10 | 159 | 23 | 18 | 1.000 | 1.125 | 2.777 | 22.01 |

Figura: [`robustness/fig_scale.svg`](robustness/fig_scale.svg)

## 8. Calibração do viés recebedor no BCB WP 619 (W2b)
_seed `20260930`, código `0.1.0+git.6eeca1c`, comando `cargo run --release -p loopsettle-cli -- experiment bias-calibration`._

Quartis da razão de pagamentos compensáveis por banco (WP 619, eq. 3.2) para cada σ e multiplicador das IPs, média de 3 sementes. Alvo, com dados reais do Pix: p25 18–45%, mediana 57–80%, p75 73–90%.

| σ viés | mult. IP | p25 | mediana | p75 | quartis dentro do alvo | perda (p.p.²) | LB/UB |
|---|---|---|---|---|---|---|---|
| 0.4 | 1.3 | 64.3% | 80.3% | 91.2% | 0 | 1307.3 | 0.891 |
| 0.4 | 2 | 59.2% | 76.0% | 88.4% | 2 | 874.2 | 0.895 |
| 0.4 | 3 | 52.1% | 70.6% | 85.6% | 2 | 445.9 | 0.911 |
| 0.6 | 1.3 | 54.7% | 73.4% | 88.4% | 2 | 611.8 | 0.934 |
| 0.6 | 2 | 53.7% | 71.5% | 85.6% | 2 | 517.0 | 0.934 |
| 0.6 | 3 | 49.0% | 66.6% | 84.2% | 2 | 317.3 | 0.944 |
| 0.8 | 1.3 | 45.7% | 67.0% | 85.5% | 2 | 219.9 | 0.957 |
| 0.8 | 2 | 44.5% | 66.3% | 84.7% | 3 | 183.8 | 0.955 |
| 0.8 | 3 | 41.9% | 63.7% | 81.7% | 3 | 130.6 | 0.959 |
| 1 | 1.3 | 36.2% | 61.1% | 83.3% | 3 | 79.5 | 0.966 |
| 1 | 2 | 36.6% | 59.2% | 82.6% | 3 | 113.5 | 0.964 |
| 1 | 3 | 34.5% | 57.6% | 78.8% | 3 | 134.7 | 0.965 |
| 1.2 | 1.3 | 27.8% | 55.4% | 78.8% | 2 | 192.4 | 0.976 |
| 1.2 | 2 | 29.3% | 54.3% | 81.0% | 2 | 206.6 | 0.971 |
| 1.2 | 3 | 29.2% | 52.6% | 78.6% | 2 | 266.4 | 0.973 |
| 1.5 | 1.3 | 17.6% | 46.3% | 72.3% | 0 | 769.8 | 0.980 |
| 1.5 | 2 | 19.2% | 45.9% | 76.0% | 2 | 691.7 | 0.982 |
| 1.5 | 3 | 19.6% | 45.2% | 76.5% | 2 | 711.2 | 0.979 |
| 2 | 1.3 | 7.5% | 29.8% | 60.9% | 0 | 2499.1 | 0.986 |
| 2 | 2 | 8.1% | 32.7% | 62.6% | 0 | 2189.2 | 0.987 |
| 2 | 3 | 8.2% | 33.2% | 64.6% | 0 | 2072.2 | 0.988 |
| 2.5 | 1.3 | 2.5% | 16.5% | 49.8% | 0 | 4550.5 | 0.993 |
| 2.5 | 2 | 2.7% | 19.1% | 50.9% | 0 | 4210.9 | 0.993 |
| 2.5 | 3 | 2.9% | 21.1% | 52.4% | 0 | 3911.8 | 0.994 |

Melhor ajuste: σ = 1, multiplicador das IPs = 1.3 (p25 36.2%, mediana 61.1%, p75 83.3%).

## 9. Heterogeneidade intradiária e escala (W2, critério de morte K1)
_seed `20260930`, código `0.1.0+git.6eeca1c`, comando `cargo run --release -p loopsettle-cli -- experiment heterogeneity`._

Perfis intradiários por classe de instituição com intensidade λ (0 = um perfil para todos, 1 = perfis completos), em σ = 0,4 e no σ calibrado:

| σ viés | λ | pagamentos/dia | motor | economia vs UB | limite de atraso | LB/UB |
|---|---|---|---|---|---|---|
| 0.4 | 0 | 1 000 000 | micro:all:500:10000 | 0.0147% | 0.761% | 0.907 |
| 0.4 | 0 | 1 000 000 | deferred:8640 | 0.0276% | 0.761% | 0.907 |
| 0.4 | 0 | 1 000 000 | deferred:96 | 0.3248% | 7.956% | 0.907 |
| 0.4 | 0 | 1 000 000 | deferred:1 | 9.2669% | 275.951% | 0.907 |
| 0.4 | 0 | 10 000 000 | micro:all:500:10000 | -0.0004% | 0.276% | 0.971 |
| 0.4 | 0 | 10 000 000 | deferred:8640 | 0.0078% | 0.276% | 0.971 |
| 0.4 | 0 | 10 000 000 | deferred:96 | 0.2188% | 3.763% | 0.971 |
| 0.4 | 0 | 10 000 000 | deferred:1 | 2.9492% | 257.365% | 0.971 |
| 0.4 | 0 | 30 000 000 | micro:all:500:10000 | -0.0016% | 0.271% | 0.983 |
| 0.4 | 0 | 30 000 000 | deferred:8640 | 0.0052% | 0.271% | 0.983 |
| 0.4 | 0 | 30 000 000 | deferred:96 | 0.1610% | 3.405% | 0.983 |
| 0.4 | 0 | 30 000 000 | deferred:1 | 1.6789% | 248.574% | 0.983 |
| 0.4 | 0.5 | 1 000 000 | micro:all:500:10000 | 0.0172% | 2.550% | 0.852 |
| 0.4 | 0.5 | 1 000 000 | deferred:8640 | 0.0335% | 2.550% | 0.852 |
| 0.4 | 0.5 | 1 000 000 | deferred:96 | 1.2077% | 15.574% | 0.852 |
| 0.4 | 0.5 | 1 000 000 | deferred:1 | 14.7590% | 304.849% | 0.852 |
| 0.4 | 0.5 | 10 000 000 | micro:all:500:10000 | -0.0018% | 0.337% | 0.933 |
| 0.4 | 0.5 | 10 000 000 | deferred:8640 | 0.0101% | 0.337% | 0.933 |
| 0.4 | 0.5 | 10 000 000 | deferred:96 | 0.3752% | 5.998% | 0.933 |
| 0.4 | 0.5 | 10 000 000 | deferred:1 | 6.7259% | 272.271% | 0.933 |
| 0.4 | 0.5 | 30 000 000 | micro:all:500:10000 | -0.0060% | 0.464% | 0.933 |
| 0.4 | 0.5 | 30 000 000 | deferred:8640 | 0.0053% | 0.464% | 0.933 |
| 0.4 | 0.5 | 30 000 000 | deferred:96 | 0.1974% | 5.591% | 0.933 |
| 0.4 | 0.5 | 30 000 000 | deferred:1 | 6.6604% | 286.060% | 0.933 |
| 0.4 | 1 | 1 000 000 | micro:all:500:10000 | 0.0128% | 0.990% | 0.857 |
| 0.4 | 1 | 1 000 000 | deferred:8640 | 0.0241% | 0.990% | 0.857 |
| 0.4 | 1 | 1 000 000 | deferred:96 | 0.9513% | 10.635% | 0.857 |
| 0.4 | 1 | 1 000 000 | deferred:1 | 14.3374% | 298.137% | 0.857 |
| 0.4 | 1 | 10 000 000 | micro:all:500:10000 | 0.0059% | 0.436% | 0.903 |
| 0.4 | 1 | 10 000 000 | deferred:8640 | 0.0098% | 0.436% | 0.903 |
| 0.4 | 1 | 10 000 000 | deferred:96 | 0.3377% | 4.929% | 0.903 |
| 0.4 | 1 | 10 000 000 | deferred:1 | 9.6710% | 325.432% | 0.903 |
| 0.4 | 1 | 30 000 000 | micro:all:500:10000 | 0.0025% | 0.376% | 0.902 |
| 0.4 | 1 | 30 000 000 | deferred:8640 | 0.0117% | 0.376% | 0.902 |
| 0.4 | 1 | 30 000 000 | deferred:96 | 0.2144% | 4.963% | 0.902 |
| 0.4 | 1 | 30 000 000 | deferred:1 | 9.7541% | 319.580% | 0.902 |
| 1 | 0 | 1 000 000 | micro:all:500:10000 | 0.0037% | 0.378% | 0.966 |
| 1 | 0 | 1 000 000 | deferred:8640 | 0.0047% | 0.378% | 0.966 |
| 1 | 0 | 1 000 000 | deferred:96 | 0.2096% | 3.695% | 0.966 |
| 1 | 0 | 1 000 000 | deferred:1 | 3.4386% | 187.450% | 0.966 |
| 1 | 0 | 10 000 000 | micro:all:500:10000 | -0.0010% | 0.149% | 0.990 |
| 1 | 0 | 10 000 000 | deferred:8640 | 0.0052% | 0.149% | 0.990 |
| 1 | 0 | 10 000 000 | deferred:96 | 0.1292% | 1.702% | 0.990 |
| 1 | 0 | 10 000 000 | deferred:1 | 0.9660% | 178.282% | 0.990 |
| 1 | 0 | 30 000 000 | micro:all:500:10000 | 0.0001% | 0.062% | 0.998 |
| 1 | 0 | 30 000 000 | deferred:8640 | 0.0026% | 0.062% | 0.998 |
| 1 | 0 | 30 000 000 | deferred:96 | 0.0493% | 1.453% | 0.998 |
| 1 | 0 | 30 000 000 | deferred:1 | 0.2411% | 184.471% | 0.998 |
| 1 | 0.5 | 1 000 000 | micro:all:500:10000 | 0.0012% | 1.448% | 0.934 |
| 1 | 0.5 | 1 000 000 | deferred:8640 | 0.0065% | 1.448% | 0.934 |
| 1 | 0.5 | 1 000 000 | deferred:96 | 0.3304% | 5.716% | 0.934 |
| 1 | 0.5 | 1 000 000 | deferred:1 | 6.6153% | 193.797% | 0.934 |
| 1 | 0.5 | 10 000 000 | micro:all:500:10000 | -0.0021% | 0.187% | 0.979 |
| 1 | 0.5 | 10 000 000 | deferred:8640 | 0.0099% | 0.187% | 0.979 |
| 1 | 0.5 | 10 000 000 | deferred:96 | 0.0879% | 2.999% | 0.979 |
| 1 | 0.5 | 10 000 000 | deferred:1 | 2.1255% | 188.905% | 0.979 |
| 1 | 0.5 | 30 000 000 | micro:all:500:10000 | -0.0009% | 0.084% | 0.987 |
| 1 | 0.5 | 30 000 000 | deferred:8640 | 0.0036% | 0.084% | 0.987 |
| 1 | 0.5 | 30 000 000 | deferred:96 | 0.0842% | 2.914% | 0.987 |
| 1 | 0.5 | 30 000 000 | deferred:1 | 1.3235% | 204.964% | 0.987 |
| 1 | 1 | 1 000 000 | micro:all:500:10000 | 0.0093% | 0.509% | 0.944 |
| 1 | 1 | 1 000 000 | deferred:8640 | 0.0116% | 0.509% | 0.944 |
| 1 | 1 | 1 000 000 | deferred:96 | 0.3144% | 3.755% | 0.944 |
| 1 | 1 | 1 000 000 | deferred:1 | 5.5902% | 202.667% | 0.944 |
| 1 | 1 | 10 000 000 | micro:all:500:10000 | -0.0003% | 0.115% | 0.970 |
| 1 | 1 | 10 000 000 | deferred:8640 | 0.0094% | 0.115% | 0.970 |
| 1 | 1 | 10 000 000 | deferred:96 | 0.1663% | 2.671% | 0.970 |
| 1 | 1 | 10 000 000 | deferred:1 | 3.0419% | 197.235% | 0.970 |
| 1 | 1 | 30 000 000 | micro:all:500:10000 | 0.0036% | 0.065% | 0.972 |
| 1 | 1 | 30 000 000 | deferred:8640 | 0.0087% | 0.065% | 0.972 |
| 1 | 1 | 30 000 000 | deferred:96 | 0.0924% | 2.869% | 0.972 |
| 1 | 1 | 30 000 000 | deferred:1 | 2.8436% | 205.891% | 0.972 |

Critério K1 (parar se o limite de 10 s passar de 2% do UB com 30 milhões de pagamentos): pior caso 0.46%; disparou: não.

