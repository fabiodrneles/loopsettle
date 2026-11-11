# Metodologia

> [English version](METHODOLOGY.md)

> Cada métrica em linguagem simples e formal. Números: `results/SUMMARY.pt-BR.md`.

## 1. Liquidez necessária: limite superior (UB) e inferior (LB)

Seja `x_i(t)` a posição acumulada do participante `i` até o instante `t`:
`x_i(t) = entradas_i(t) − saídas_i(t)`, somando os pagamentos na ordem de chegada.

- **UB (limite superior)**: `UB_i = max(0, −min_t x_i(t))`. É o menor saldo inicial que
  evita qualquer rejeição no motor `GrossRealTime` (liquidação bruta em tempo real, como o
  Pix). Prova: o saldo em `t` é `saldo_inicial + x_i(t)`, e um pagamento só é rejeitado se o
  saldo ficaria negativo. O teste `ub_is_tight` verifica que, com saldo inicial = UB, nada
  é rejeitado, e que a liquidez consumida no modo ilimitado é exatamente UB.
- **LB (limite inferior)**: `LB_i = max(0, −x_i(T))`, a posição devedora líquida no fim do
  período `T`. É o que uma única compensação multilateral no fim do período exigiria.

Sempre `LB_i ≤ UB_i`, e `Σ_i x_i(T) = 0`.

### Exemplo à mão (teste `hand_example_ub_lb`)

Participantes A, B, C, saldos ilimitados:

| t | pagamento | x_A | x_B | x_C |
|---|---|---|---|---|
| 1 | A→B 100 | −100 | +100 | 0 |
| 2 | B→C 50 | −100 | +50 | +50 |
| 3 | C→A 70 | −30 | +50 | −20 |
| 4 | B→A 30 | 0 | +20 | −20 |

- UB = (100, 0, 20), total **120**.
- LB = (0, 0, 20), total **20**.
- Com saldo inicial (100, 0, 20) nada é rejeitado; com (99, 0, 20) o primeiro pagamento de
  A é rejeitado.

A diferença UB − LB = 100 é a liquidez que existe só por causa da **ordem temporal** dos
pagamentos: um sistema que pudesse compensar tudo no fim do período a economizaria.

## 2. Medição da liquidez de um motor

Cada motor roda com saldo inicial "ilimitado" (`UNLIMITED = 2^50` centavos). A liquidez que
o motor precisou é o **pico de queda do saldo** (`saldo_inicial − menor saldo atingido`).
Para `GrossRealTime` isso é exatamente o UB. Ver `DECISIONS.pt-BR.md`, D3.

## 3. Três lentes de circularidade (a "escada")

Seja `F` o grafo agregado de fluxos (bruto: `i→j` e `j→i` podem coexistir).

1. **Hodge (diagnóstico).** No fluxo líquido `f`, `f = B1ᵀφ + B2ψ + h`: gradiente
   (fluxo "morro abaixo" de um potencial φ por participante), rotacional (circulação em
   triângulos) e harmônico (ciclos maiores). `L0 φ = B1 f` e `(B2ᵀB2) ψ = B2ᵀ(f − B1ᵀφ)`,
   ambos resolvidos por gradiente conjugado.
2. **CDFD máxima (teto estático).** Maior circulação `c` com `0 ≤ c_e ≤ F_e` e conservação
   em cada nó. Respeita o limite de cada aresta.
3. **Netting por ciclos (executável).** Cancela, ciclo a ciclo, o menor fluxo do ciclo.

**Triângulo (teste fixo, invariante 8):** A→B 10, B→C 5, C→A 5.

| lente | por aresta | total | fração |
|---|---|---|---|
| Hodge (rotacional) | 6,67 | 20 | 100% (L1) |
| CDFD máxima | 5 | 15 | 75% |
| Netting por ciclos | 5 | 15 | 75% |

Hodge projeta `f` na direção do ciclo `(1,1,1)` e atribui a média 20/3 = 6,67 a cada
aresta, **acima** do fluxo de 5 em B→C e C→A: não respeita limites de aresta. Por isso é um
diagnóstico (quanta "forma circular" o fluxo tem), não um valor compensável.

Em geral: CDFD ≥ netting guloso (teste de propriedade) e, no benchmark de 200 nós / 5.000
arestas, Hodge L1 ≫ CDFD > netting (ver `CHECKPOINT.md`).

## 4. Motores de liquidação

Todos implementam o trait `SettlementEngine` (`crates/loopsettle-core/src/engine.rs`) e são
alimentados em streaming, em ordem de chegada, por `drive()`.

| Motor | Em linguagem simples | Regra formal |
|---|---|---|
| `GrossRealTime` | Como o Pix: cada pagamento liquida na hora, se houver saldo. | Na chegada `t`: se `saldo_pagador ≥ v`, transfere; senão, rejeita. Atraso 0. |
| `DeferredNet { N }` | Compensação em horários fixos (N vezes por dia). | Nos instantes `k·(24h/N)`, aplica a posição líquida acumulada de cada participante. Atraso = tempo até o próximo ciclo. |
| `AtomicMicroCycle { janela, atraso, modo }` | A cada janela curta, procura "rodas" de pagamentos na fila e liquida cada roda de uma vez. | Filas FIFO por par; DFS acha um ciclo dirigido (≤ 6 participantes) entre as cabeças das filas; o ciclo liquida atomicamente se cada participante cobre sua posição líquida no ciclo. No prazo `chegada + atraso`, vai para bruto. Modos: `QueueAll` (tudo espera) e `QueueOnShortfall` (só espera quem não tem saldo). |

## 5. Métricas

- **Liquidez necessária de um motor**: soma, entre participantes, do pico de queda do saldo
  com saldo inicial ilimitado (D3). Para o bruto, é o UB.
- **Economia de liquidez**: `1 − liquidez_motor / UB`; e a fração da distância `UB − LB`
  recuperada: `(UB − liquidez_motor) / (UB − LB)`.
- **Atrasos**: p50, p95, p99 do tempo entre chegada e liquidação, por histograma
  log-linear (erro relativo < 1,6%; o percentil reportado é o limite inferior do balde).
  Comparação com a meta do Manual de Tempos do Pix: 50% liquidados em até 2,8 s (o atraso
  medido é só o que o motor adiciona; ver limitações).
- **Capacidade potencial (estilo do estudo do BCB)**: `Σ_i min(entradas_i, saídas_i) /
  volume bruto` no período, ignorando a ordem temporal.
- **Economia potencial de liquidez**: `(UB − LB) / UB` — o que uma única compensação no fim
  do dia economizaria.
- **Escada**: Hodge L1 (diagnóstico) ≥? CDFD máxima (teto estático) ≥ netting por ciclos
  (executável estático) ≥ micro-ciclos (executável no tempo).
- **Liquidez restrita**: saldo inicial = fração do UB de cada participante; mede-se valor
  rejeitado e atrasos.

## 6. Potencial vs realizável (a comparação central)

O estudo do BCB mede o que *poderia* ser compensado se entradas e saídas do mesmo dia se
anulassem, sem olhar a ordem. O LoopSettle mede, no mesmo cenário, o que de fato se
economiza quando a liquidação tem de respeitar o tempo: compensação diferida em intervalos
cada vez menores (de 1 por dia até a cada 10 s) e micro-ciclos atômicos com atraso máximo
de 2 a 10 s. A distância entre as duas medidas é o resultado central
(`results/potential-vs-realizable/`).

## 7. Calibração

Ver `docs/PARAMETERS.pt-BR.md`: participações por chaves Pix por ISPB, ticket médio, participação
do B2B em quantidade e valor, e fator de escala, todos de set/2026 (API Olinda do BCB).

## 8. Reprodutibilidade

- Toda simulação recebe uma `seed`; o PRNG é próprio (xoshiro256**), estável entre
  plataformas, inclusive wasm (invariante 7, testada).
- Todo número publicado vem de `cargo run --release -p loopsettle-cli -- experiment <nome>`,
  que grava `results/<nome>/summary.json` com seed, versão do código e comando. Nenhum
  carimbo de tempo é gravado: rodar de novo reproduz os arquivos byte a byte.
- `make rebuild` refaz tudo do zero.

## 9. O limite de atraso

**Em linguagem simples.** Um mecanismo que pode segurar cada pagamento por no máximo *d*
segundos só consegue mudar *quando* os pagamentos liquidam, e em no máximo *d*. No momento em
que um participante chega à sua posição devedora mais funda na liquidação bruta, os únicos
pagamentos que um motor com prazo ainda pode estar segurando são os que chegaram nos últimos *d*
segundos. Então o máximo que ele consegue economizar é o que esse participante pagou nesses *d*
segundos — por mais esperto que seja o algoritmo de compensação.

**Formalmente.** Com saldos de abertura ilimitados, seja `x_i(t)` a posição acumulada do
participante `i` no `GrossRealTime` e `t*_i` o primeiro instante em que ela atinge o mínimo, de
modo que `UB_i = −x_i(t*_i)`. Tome qualquer motor em que todo pagamento que chega em `a` liquide
em `[a, a + d]`, e seja `y_i(t)` a posição acumulada de `i` nesse motor. Em `t*`, todo pagamento
ainda não liquidado chegou em `[t* − d, t*]`, então

```text
y_i(t*) ≤ x_i(t*) + O_i[t* − d, t*]          (O_i = valor pago por i na janela)
L_i = max(0, −min_t y_i(t)) ≥ −y_i(t*)   ⇒   UB_i − L_i ≤ O_i[t*_i − d, t*_i].
```

O limite vale para micro-ciclos atômicos com `max_delay_ms = d`, para compensação diferida a
cada `d` e para qualquer resolvedor ótimo de gridlock com o mesmo prazo. Ele é calculado em duas
passadas em streaming (`loopsettle_sim::bound::delay_bounds`) e testado participante a
participante contra os motores (`engines_never_beat_the_delay_bound`). É um teto, não uma
estimativa: no cenário calibrado, a economia obtida com 10 s é cerca de 1,5% dele (cerca de 28% da forma apertada da §9b; a compensação a cada 10 s obtém cerca de 46%)
(`results/robustness/seeds_summary.csv`).

**9b. O piso de liquidez (forma apertada do limite).** O argumento acima usa um único instante,
`t*`. Usando todos os instantes, a afirmação fica mais forte. Seja `CO_i(s)` o valor que `i` já
pagou até `s` e `CI_i(s)` o valor que já recebeu até `s` (horários de chegada, inclusive). Em
qualquer motor com atraso máximo `d`, no instante `s + d` todo pagamento que chegou até `s` já
liquidou, e nenhum recebimento liquida antes de chegar, então

```text
−y_i(s + d) ≥ CO_i(s) − CI_i(s + d)      para todo s
piso_i(d) = max(0, max_s [CO_i(s) − CI_i(s + d)])  ≤  L_i  ≤  UB_i = piso_i(0).
```

`UB_i − piso_i(d)` nunca passa do limite de janela (tome `s = t* − d`), e o `piso_i` é atingido
por um participante sozinho com "seguro meus pagamentos por exatamente `d` e recebo tudo na
hora"; por isso é o melhor limite que usa só os fluxos do próprio `i`. É calculado em streaming
com uma passada atrasada e uma adiantada (`loopsettle_sim::bound::liquidity_floors`), testado
contra os motores no mesmo teste e informado por `loopsettle bound` (`liquidity_floor`,
`tight_bound`). No cenário de varejo calibrado (`results/regimes/bound_curve.csv`, uma semente),
o limite apertado para 10 s é 0,054% do UB, contra 0,76% na forma de janela, e a compensação a
cada 10 s obtém 51% dele; com atraso de um dia ele é exatamente UB − LB. No sistema de alto valor
estilizado, o limite de janela só é grande porque conta o pagamento que faz o pico; o limite
apertado ali é ≤ 0,004%.

**Valor nulo de Hodge.** Para fluxos aleatórios num grafo conexo com `m` arestas e `n` nós, a
fração esperada da norma ao quadrado fora do espaço gradiente é `(m − n + 1)/m`. No grafo
líquido do baseline (`m = 6620`, `n = 200`) isso dá 0,970, contra uma fração rotacional L2
observada de 0,977: em grafos densos, a fração rotacional de Hodge mede sobretudo a densidade do
grafo.

## 10. Janelas de abastecimento

A Conta PI só recebe aportes das Reservas com o STR aberto (6h30–18h30 nos dias úteis, mais uma
janela de 30 minutos para aportes; BCB, Relatório Anual do SPI 2024, p. 15). Dentro dessa janela, um
pico intradiário pode ser coberto na hora; fora dela — toda noite e o fim de semana inteiro — o
participante precisa ter, no início da janela, tudo o que vai precisar até ela acabar. Para uma
janela fechada `[a, b)`, com `x_i` a posição acumulada:

```text
need_i = max(0, x_i(a) − min_{a≤t<b} x_i(t))      pré-financiamento no início da janela
net_i  = max(0, x_i(a) − x_i(b))                  o que uma compensação no fim da janela exige
```

O limite de atraso da seção 9 vale dentro da janela, com `t*` o mínimo da janela. Calculado por
`loopsettle_sim::bound::window_stats`; experimento `funding-windows` (quinta a segunda, perfil
horário e volumes por dia com fonte).

**O piso de liquidez dentro de uma janela.** A forma apertada da seção 9b vale aqui com um termo
a mais. A necessidade de um motor na janela é `y_i(a) − min_{a≤t<b} y_i(t)`. Recebimentos dos
últimos `d` antes da janela podem ainda não ter liquidado quando ela abre, então
`y_i(a) ≥ x_i(a) − I_i[a − d, a)`, e o argumento do piso dá `−y_i(t) ≥ CO_i(t − d) − CI_i(t)`.
Juntando:

```text
necessidade_i ≥ max_{s ≥ a, s + d < b} [(CO_i(s) − CO_i(a)) − (CI_i(s + d) − CI_i(a))] − I_i[a − d, a)
```

com piso em zero; com `d = 0` é a necessidade bruta acima. Calculado por
`loopsettle_sim::bound::window_floors` e testado contra os motores reconstruindo a posição de cada
motor a partir dos eventos de liquidação (`engines_never_need_less_than_the_window_floor`).

## 11. Rodar em dados reais

`loopsettle bound <arquivo.csv>` calcula UB, LB e o limite de atraso para vários `d`;
`loopsettle settle <arquivo.csv>` também roda os motores de liquidação. O arquivo é lido em
streaming duas vezes (o limite precisa de duas passadas), a memória é O(participantes) e os valores
são convertidos em centavos de forma exata. O teste `csv_path_equals_in_memory_path` confere que o
caminho pelo arquivo dá o mesmo UB, LB, liquidez dos motores e limite que o caminho em memória.

