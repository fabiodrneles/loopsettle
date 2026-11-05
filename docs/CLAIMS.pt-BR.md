# CLAIMS — o que o estudo pode e não pode afirmar

> [English version](CLAIMS.md)

Evidência = arquivo em `results/` gerado por comando (ver `results/SUMMARY.pt-BR.md`).

## Pode afirmar (com a evidência indicada)

| # | Afirmação | Evidência | Condição |
|---|---|---|---|
| 1 | Micro-ciclos atômicos de até 10 s economizam cerca de 0,01% da liquidez necessária: 0,0116% do UB, IC 95% [0,0079; 0,0152]% (10 sementes, σ = 0,4); médias ≤ 0,018% em σ ∈ [0; 0,8]; cai para ≤ 0 quando o volume cresce até 30 milhões de pagamentos por dia; não depende do comprimento máximo do ciclo (2 a 10). A medida potencial de volume no estilo do BCB (85% do volume) mede outra coisa e não deve ser comparada diretamente com uma economia de liquidez. | `robustness/seeds_summary.csv`, `robustness/scale.csv`, `robustness/cycle_len.csv` | cenário sintético, chegadas estacionárias |
| 1b | Nenhum mecanismo que liquide cada pagamento em até d consegue economizar mais do que o valor que cada participante paga nos d antes do seu pico na liquidação bruta (limite de atraso). Para d = 10 s, esse teto fica entre 0,27% e 0,80% do UB em todas as configurações testadas. | teorema em `METHODOLOGY.pt-BR.md` §9; teste `engines_never_beat_the_delay_bound`; `robustness/scale.csv` | geral para o limite; números do cenário sintético |
| 1d | Com o viés recebedor calibrado no BCB WP 619 (σ = 1,0), a economia de uma compensação no fim do dia com 30 milhões de pagamentos por dia fica entre 0,24% do UB (sem heterogeneidade intradiária) e 2,8% (perfis por classe completos); compensar a cada 15 minutos economiza 0,05–0,09%; compensar em segundos ≈ 0. O limite de atraso de 10 s fica ≤ 0,46% em todos os casos de heterogeneidade (critério de morte K1 não disparou). | `heterogeneity/heterogeneity.csv`, `bias-calibration/grid.csv` | os formatos dos perfis por classe são ASSUMPTION (D18) |
| 1e | Com as janelas de abastecimento da Conta PI (aportes só com o STR aberto), o fim de semana é a janela de pré-financiamento que restringe: cerca de 2,7× o maior esvaziamento de um dia, e uma compensação no fim do fim de semana economizaria só 0,4–1,0% dele (σ calibrado, 10 milhões de pagamentos por quinta-feira); o limite de 10 s dentro dele é ≤ 0,23%. | `funding-windows/windows.csv` | perfil horário e volumes por dia com fonte; estrutura bilateral sintética |
| 1f | O limite de atraso depende do quanto os pagamentos são grandes em relação à liquidez necessária: para 10 s, é 0,76% do UB no cenário de varejo, mas 13,7% num sistema de alto valor estilizado com 2.000 pagamentos por dia; então não contradiz as grandes economias de LSM relatadas para sistemas de alto valor. Para atrasos de 10 s a um dia, a compensação obtida fica estável em 3–5% do limite. | `regimes/regimes.csv`, `regimes/bound_curve.csv` | o cenário de alto valor é ASSUMPTION |
| 1c | Com chegadas estacionárias, a economia de uma compensação no fim do dia encolhe com 1/√(volume): 9,27% do UB com 1 milhão de pagamentos por dia, 1,68% com 30 milhões. Medidas de potencial calculadas em amostras pequenas a superestimam. | `robustness/scale.csv` | chegadas estacionárias (D8, D9) |
| 2 | Liquidar muito valor dentro de ciclos (até 63%) não implica economizar liquidez: o pico de liquidez é dirigido por desequilíbrios que duram horas. | `microcycle-sweep/sweep.csv`; `robustness/seeds_summary.csv` (σ = 0,4, 10 sementes: 0,019% a cada 10 s, 0,50% a cada 15 min, 7,8% uma vez por dia) | idem |
| 3 | A economia de liquidez da compensação diferida cresce de forma monótona com o intervalo entre ciclos, e só uma compensação diária recupera toda a distância UB−LB. | `potential-vs-realizable/engines.csv` | sem saldos intradiários de abertura |
| 4 | As três lentes de circularidade não coincidem: Hodge (diagnóstico) > CDFD máxima > netting guloso, com distância grande no grafo líquido (0,98 / 0,17 / 0,14). | `baseline/ladder.csv`; teste do triângulo | Hodge ≥ CDFD observado, não provado |
| 5 | A decomposição de Hodge não respeita limites de aresta e não é um montante compensável (triângulo: 6,67 vs 5). | teste `triangle_case` | geral (matemático) |
| 6 | A CDFD máxima pode ser calculada exatamente, sem PL, por fluxo de custo mínimo; resultado igual ao de um solver de PL em 1.000 grafos. | teste `cdfd_matches_lp_oracle_on_1000_random_graphs` | geral |
| 7 | Micro-ciclos com atraso máximo ≥ 5 s não cumprem a meta de 50% liquidados em até 2,8 s; com 2 s cumprem. | `microcycle-sweep/sweep.csv` | atraso medido é só o do motor |
| 8 | Na lógica do estudo finlandês, o sistema instantâneo exige 1,1–1,5× a liquidez de uma compensação diária, e a diferença cai para 1,01–1,03× com 48 ciclos por dia. | `finland-replication/ratios.csv` | depende do viés σ |
| 9 | O simulador é reprodutível bit a bit e roda em hardware modesto (i3 de 4ª geração, 4 GB). | invariante 7; `make rebuild` em 120 s; pico < 20 MiB por experimento | — |

## Não pode afirmar

| # | Afirmação proibida | Por quê |
|---|---|---|
| 1 | "O Pix poderia economizar R$ X bilhões de liquidez." | Dados sintéticos; a liquidez absoluta não escala linearmente com o fator 239; LB/UB depende de hipótese (σ). |
| 2 | "X% da liquidez do Pix é circular." | Só temos agregados públicos; a estrutura bilateral é hipótese. |
| 3 | "Micro-ciclos não funcionam no Pix real." | Escala 239× menor por janela; heurística simples; não testado com volume real por segundo. |
| 4 | "O estudo do BCB superestima a capacidade de netting." | Não reproduzimos os dados do BCB; reproduzimos a *lógica* da medida em dados sintéticos. |
| 5 | "Replicamos Hellqvist & Korpinen." | Aplicamos a lógica deles a outro cenário; comparação só de ordem de grandeza. |
| 6 | "A correlação potencial × posição líquida valida o modelo." | Em grafo denso é quase mecânica. |
| 7 | "Um participante grande fora do ar não afeta a liquidez." | Cenário de estresse fraco (re-envio total, pico no fim do dia). |
| 8 | Qualquer afirmação sobre a variante proporcional da CDFD ou sobre dados on-chain. | Não implementado / não rodado. |
| 10 | "A economia potencial de liquidez é 9,27%" ou "o realizável é 630× menor que o potencial". | O potencial depende da escala (9,27% com 1 milhão de pagamentos por dia, 1,68% com 30 milhões) e da semente (σ = 0,4: IC [4,5; 11,1]%); a razão não é um número estável (`robustness/`). |
| 11 | Qualquer número de uma semente só como resultado. | Usar as médias e os IC de 10 sementes de `robustness/seeds_summary.csv`. |
| 12 | "A fração rotacional de Hodge mostra que X% do fluxo é circular." | No grafo líquido do baseline, o valor nulo para fluxos aleatórios é (m − n + 1)/m = 0,970, contra 0,977 observado (`METHODOLOGY.pt-BR.md` §9). |
| 13 | "O LoopSettle reproduz / refuta McMahon et al. (2024)." | Outro sistema (T2, alto valor) e outra base de comparação (FIFO com fila e liquidez restrita); o `regimes` só mostra que o limite não restringe em fluxos com poucos pagamentos grandes. |
| 9 | "O FedNow / TIPS / UPI economizaria X% de liquidez." | O cenário é calibrado só com o Pix; outros sistemas precisam dos próprios agregados públicos. Os motores e o mecanismo qualitativo se aplicam ([`INSTANT_PAYMENTS.pt-BR.md`](INSTANT_PAYMENTS.pt-BR.md)); os números não se transferem. |
