# DECISIONS — decisões de desenho

Cada entrada: o que foi decidido, alternativas, por quê, e como reverter.

## D0 — Arquivo de progresso chama-se `CHECKPOINT.md`
- **Decisão:** o `PROGRESS.md` do `CLAUDE.md` é o `CHECKPOINT.md` (pedido explícito do usuário).
- **Reverter:** `git mv CHECKPOINT.md PROGRESS.md`.

## D1 — Sem dependências para RNG, CSV, JSON e TOML
- **Decisão:** PRNG (SplitMix64/xoshiro256**), escrita de CSV/JSON e um parser de um
  subconjunto de TOML são implementados à mão.
- **Alternativas:** `rand`, `serde`/`serde_json`, `toml`.
- **Por quê:** a regra de dependências mínimas (seção 2) e o determinismo bit a bit
  (invariante 7) — um PRNG próprio não muda de comportamento com atualizações de crates.
- **Reverter:** trocar os módulos `rng`, `json`, `toml_lite` pelos crates equivalentes.

## D2 — Dependências aceitas
- `proptest` (dev): exigido pela seção 3.
- `minilp` (dev, só em `loopsettle-topology`): oráculo de PL nos testes da CDFD (seção 7, Fase 2).
- `wasm-bindgen` (só em `loopsettle-wasm`): bindings do navegador; sem ele seria preciso
  escrever a ABI JS↔wasm à mão.

## D3 — Liquidez de um motor = pico de queda do saldo com saldo "ilimitado"
- **Decisão:** para medir a liquidez que cada motor exige, todos os participantes começam
  com `UNLIMITED = 2^50` centavos; a liquidez necessária é `saldo_inicial − menor saldo`.
  No `GrossRealTime` isso coincide com o UB (provado por teste de propriedade).
- **Alternativas:** busca binária do menor saldo inicial que não gera rejeições/atrasos
  acima de um limite (mais cara: várias execuções por participante).
- **Por quê:** é a medida usada por Hellqvist & Korpinen (liquidez como máximo da posição
  devedora acumulada), roda em uma única passada e mantém a invariante 3 (saldo nunca
  negativo) válida mesmo no modo de medição.
- **Reverter / alternativa:** os motores também aceitam saldos finitos (modo restrito),
  usado no experimento com `QueueOnShortfall` (ver D-futuras).

## D4 — Hodge sobre o grafo **líquido**, com normas L2 e L1
- **Decisão:** a decomposição de Hodge usa o fluxo líquido bilateral (uma aresta por par,
  orientada do pagador líquido). Reportamos `‖curl‖/‖F‖` (L2, a "fração circular" da seção 5),
  `‖curl+harm‖/‖F‖` (L2) e `Σ|curl+harm| / ΣF` (L1, mesma unidade monetária da CDFD e do
  netting, usada na "escada"). No grafo bruto, a parte bilateral compensada (2-ciclos) é
  somada à massa L1 de Hodge, para comparar com CDFD/ciclos brutos.
- **Alternativas:** só L2 (padrão na literatura de Hodge) — mas não é comparável com valores
  em reais; Hodge sobre fluxo bruto — matematicamente não definido (o fluxo de aresta do
  complexo é antissimétrico).
- **Por quê:** a escada (Hodge ≥ CDFD ≥ ciclos) só faz sentido na mesma unidade. Resultado
  observado: a massa L1 de Hodge pode passar de 100% do fluxo (componentes com sinal oposto
  ao fluxo), o que mostra que Hodge é diagnóstico, não montante compensável.
- **Triângulos:** limite configurável `HodgeConfig::max_triangles` (padrão 500 mil); ciclos
  não cobertos vão para o componente harmônico (reportado à parte).
- **Reverter:** `Ladder::hodge_curl_l2` já está disponível; trocar a coluna usada na escada.

## D5 — CDFD máxima via transbordo de custo mínimo (caminhos mínimos sucessivos)
- **Decisão:** em vez de "circulação de custo −1" com cancelamento de ciclos negativos,
  resolvemos o problema equivalente: minimizar `Σ d_e` com `0 ≤ d ≤ F` e mesma divergência
  de `F` (custos unitários não negativos), por caminhos mínimos sucessivos com Dijkstra +
  potenciais de Johnson, em centavos inteiros. `c = F − d`.
- **Alternativas:** cancelamento de ciclos negativos (mínimo ciclo médio); PL genérico.
- **Por quê:** é exato, inteiro, sem dependências e simples; custos não negativos evitam
  Bellman-Ford. Validado contra `minilp` em 1.000 grafos aleatórios.
- **Reverter:** substituir `cdfd::cdfd_max`; o teste de oráculo continua valendo.

## D6 — Variante proporcional/local da CDFD (BFF) adiada
- **Decisão:** não implementada nesta versão.
- **Por quê:** a especificação exata do algoritmo local precisa ser conferida no artigo
  (arXiv 2506.12546) antes de implementar; implementar "de memória" arriscaria publicar uma
  métrica com nome do artigo e semântica diferente. A CDFD máxima (teto estático) já
  responde à pergunta central.
- **Como completar:** ler a seção do algoritmo local no artigo, implementar em
  `cdfd.rs` como `cdfd_proportional`, e adicionar a coluna à escada.

## D7 — Netting por ciclos: DFS determinística com "arco corrente"
- **Decisão:** cancelamento guloso de ciclos (mínimo do ciclo), explorando arestas na ordem
  de entrada (determinístico), custo O(n·m). Roda no grafo bruto (inclui 2-ciclos
  bilaterais) e no líquido.
- **Alternativas:** ordem por valor (maiores primeiro), ciclos curtos primeiro.
- **Por quê:** reproduz o espírito de Bech & Soramäki (resolver gridlock cancelando ciclos)
  e é o algoritmo "realizável" mais simples; a CDFD dá o teto. A diferença entre os dois é
  reportada.
- **Reverter / alternativa:** mudar a ordem de `adj` em `netting.rs`.
