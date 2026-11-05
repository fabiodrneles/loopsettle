# Metodologia

> Cada métrica em linguagem simples e formal. Números: `results/SUMMARY.md`.

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
Para `GrossRealTime` isso é exatamente o UB. Ver `DECISIONS.md`, D3.

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

Ver `docs/PARAMETERS.md`: participações por chaves Pix por ISPB, ticket médio, participação
do B2B em quantidade e valor, e fator de escala, todos de set/2026 (API Olinda do BCB).

## 8. Reprodutibilidade

- Toda simulação recebe uma `seed`; o PRNG é próprio (xoshiro256**), estável entre
  plataformas, inclusive wasm (invariante 7, testada).
- Todo número publicado vem de `cargo run --release -p loopsettle-cli -- experiment <nome>`,
  que grava `results/<nome>/summary.json` com seed, versão do código e comando. Nenhum
  carimbo de tempo é gravado: rodar de novo reproduz os arquivos byte a byte.
- `make rebuild` refaz tudo do zero.
