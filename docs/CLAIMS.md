# CLAIMS — o que o estudo pode e não pode afirmar

Evidência = arquivo em `results/` gerado por comando (ver `results/SUMMARY.md`).

## Pode afirmar (com a evidência indicada)

| # | Afirmação | Evidência | Condição |
|---|---|---|---|
| 1 | No cenário sintético calibrado, a economia **potencial** de liquidez (uma compensação no fim do dia: 9,27% do UB) é ~630× a economia **realizável** com micro-ciclos atômicos de até 10 s (0,0147% do UB). A medida potencial de volume no estilo do BCB (85% do volume) mede outra coisa e não deve ser comparada diretamente com uma economia de liquidez. | `potential-vs-realizable/central.csv` | cenário sintético; vale para σ ∈ [0; 0,8] (`baseline/sensitivity.csv`) |
| 2 | Liquidar muito valor dentro de ciclos (até 63%) não implica economizar liquidez: o pico de liquidez é dirigido por desequilíbrios que duram horas. | `microcycle-sweep/sweep.csv`; `potential-vs-realizable/engines.csv` (economia cresce com o intervalo: 0,03% a cada 10 s, 0,32% a cada 15 min, 9,27% 1×/dia) | idem |
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
