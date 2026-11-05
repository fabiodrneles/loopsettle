# DECISIONS — decisões de desenho

> [English version](DECISIONS.md)

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
- **Por quê:** corresponde à medida atribuída a Hellqvist & Korpinen (liquidez como máximo
  da posição devedora acumulada — **confirmada no texto integral**, BoFER 7/2021 p. 3–4: o
  "limite superior" no instantâneo e a mínima das posições de fim de ciclo com ciclos), roda em uma única passada e mantém a invariante 3 (saldo nunca
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

## D8 — Processo de chegada: Poisson com taxa constante por hora
- **Decisão:** intensidade por partes (24 pesos horários), simulação exata por exponenciais.
- **Alternativas:** taxa contínua (ex.: soma de senoides) com *thinning*; processos com
  autoexcitação (Hawkes) para rajadas.
- **Por quê:** simples, exato, rápido e calibrável com dados horários se surgirem.
  Não modela rajadas (ex.: dia de pagamento de salário) — listado como limitação.
- **Reverter:** trocar `PaymentStream::next`.

## D9 — Recebedor por modelo gravitacional com viés recebedor
- **Decisão:** `P(j | i) ∝ atividade_j^β · viés_j · (1 + h·[mesma classe])`, `P(i→i) = 0`.
- **Por quê:** gera rede densa e majoritariamente recíproca (como o P2P do Pix) mais um
  componente direcional controlado (viés recebedor), sem precisar de dados bilaterais que
  não são públicos. Os parâmetros do viés controlam diretamente a fração circular e estão
  marcados `ASSUMPTION`; a análise de sensibilidade fica no experimento `baseline`.
- **Reverter:** pesos em `PaymentStream::with_population`.

## D10 — Calibração dos valores por mistura "corpo + cauda B2B"
- **Decisão:** a cauda de Pareto representa pagamentos empresariais grandes: sua
  probabilidade = participação do B2B na quantidade (4,30%) e sua média é resolvida para
  que carregue a participação do B2B no valor (52,05%); o corpo lognormal fica com o resto.
  O ticket médio (R$ 449,83) sai exato analiticamente.
- **Alternativas:** ajustar só o ticket médio (um parâmetro livre, cauda arbitrária);
  usar faixas de valor (não publicadas pela API).
- **Por quê:** usa dois agregados públicos em vez de um, e a fração de valor concentrada em
  poucos pagamentos grandes é o que mais pesa na liquidez.
- **Reverter:** `calibrate::apply`; ou sobrescrever `tail_prob`/`tail_xm_brl` no TOML.

## D11 — Tamanho dos participantes ∝ chaves Pix por ISPB
- **Decisão:** a participação de cada instituição como pagadora/recebedora usa a
  participação em chaves Pix (`ChavesPix`, 2026-09-30), dado público por participante.
- **Alternativas:** não há dado público de transações por participante na API usada.
- **Risco:** chaves ≠ transações (bancos digitais têm muitas chaves por cliente). Marcado como
  proxy SOURCED com ASSUMPTION embutida.

## D12 — Micro-ciclos: uma cabeça de fila por aresta, ciclos de até 6 participantes
- **Decisão:** filas FIFO por par (pagador, recebedor). A cada janela, DFS procura um ciclo
  dirigido entre arestas com fila não vazia (comprimento ≤ `max_cycle_len`, padrão 6) e tenta
  liquidar atomicamente **o pagamento mais antigo de cada aresta** do ciclo (`try_batch`).
  Se faltar saldo a algum participante para cobrir sua posição líquida no ciclo, a aresta de
  saída dele é bloqueada naquela janela e a busca continua. Repete até não haver ciclo.
- **Alternativas:** resolver um problema de otimização por janela (máximo subconjunto
  liquidável, como nos algoritmos de gridlock completos); combinar vários pagamentos da
  mesma aresta num ciclo.
- **Por quê:** é a forma mais simples, determinística e barata que respeita FIFO por aresta,
  atomicidade e o limite "nunca compensa mais que o menor pagamento do ciclo". Subestima o
  potencial de um otimizador completo — registrado como limitação.
- **Configurável:** `max_cycle_len` (`micro:<modo>:<janela>:<atraso>:<len>`).

## D13 — `QueueOnShortfall`: sem FIFO por pagador (bypass)
- **Decisão:** um pagamento liquida bruto na chegada se houver saldo, mesmo que o pagador
  tenha pagamentos mais antigos na fila; os enfileirados são re-tentados brutos a cada
  janela, em ordem de chegada (FIFO por aresta), e rejeitados no prazo se ainda faltar saldo.
- **Alternativas:** FIFO estrito por pagador (como em vários RTGS).
- **Por quê:** é o comportamento mais favorável à liquidação imediata do Pix. Efeito
  observado: com liquidez restrita, o bypass pode **aumentar** levemente a rejeição em valor
  (pagamentos novos consomem o saldo que liberaria os antigos). Documentado nos resultados.

## D14 — Pagamentos vencidos vão para bruto; se faltar saldo, rejeitados
- **Decisão:** no prazo `chegada + max_delay_ms` (exato, não arredondado à janela), o
  pagamento é tentado bruto; sem saldo, é rejeitado (como o Pix rejeita). Uma janela que
  coincide com o prazo roda antes (última chance de compensar).

## D15 — `DeferredNet` liquida sempre (DNS colateralizado)
- **Decisão:** cada ciclo aplica as posições líquidas sem checar saldo (pode ficar negativo
  no modo restrito); o atraso de cada pagamento é conhecido na chegada (próximo ciclo), então
  o motor guarda só os deltas por participante (memória O(n)).
- **Por quê:** é como se mede a liquidez de sistemas de compensação diferida (ex.: UPI,
  estudo finlandês): a necessidade é a posição devedora nos horários de liquidação.

## D16 — Interface: uma função wasm que devolve JSON; React só desenha
- **Decisão:** o wasm expõe `simulate(seed, participantes, pagamentos, janela, atraso,
  viés, fração de liquidez) -> string JSON`; o Web Worker chama e o React desenha em Canvas
  2D. Calibração pública embutida com `include_str!`. Limites: 50 participantes, 100 mil
  pagamentos. `wasm-opt` desligado (evita download extra; o .wasm tem 332 KB).
- **Alternativas:** expor objetos Rust ao JS (mais bindings, mais código de cola).
- **Por quê:** fronteira mínima e testável nativamente (`cargo test -p loopsettle-wasm`).

## D17 — Documentação e interface bilíngues (português e inglês)
- **Decisão:** o README tem duas seções completas (português e inglês), com capturas em cada
  idioma (`docs/img/` e `docs/img/en/`). **Nos demais arquivos o inglês é o padrão**, como é
  costume no GitHub: o nome sem sufixo é a versão em inglês e o gêmeo em português fica ao lado,
  com sufixo `.pt-BR.md` — `docs/<NOME>.md` ↔ `docs/<NOME>.pt-BR.md`, `DECISIONS.md` ↔
  `DECISIONS.pt-BR.md`, `REVIEW.md` ↔ `REVIEW.pt-BR.md`, `CHECKPOINT.md` ↔ `CHECKPOINT.pt-BR.md`,
  `results/SUMMARY.md` ↔ `results/SUMMARY.pt-BR.md` (ambos gerados por `scripts/summarize.py`);
  `data/README.md` e `results/README.md` têm os dois idiomas no mesmo arquivo. A interface web
  tem seletor PT/EN (`?lang=pt|en`, lembrado por visitante; padrão pelo idioma do navegador). O
  `CLAUDE.md` é o roteiro do dono para o agente de código e fica como foi escrito.
- **Alternativas:** `README.pt-BR.md` separado; português como nome padrão (rejeitado: um
  leitor de inglês que abre uma pasta no GitHub cairia no português).
- **Por quê:** pedido do usuário (documentação completa nos dois idiomas); o público-alvo
  (bancos centrais, pesquisadores de pagamentos, revisores do JOSS) é internacional.
- **Regra:** qualquer mudança num documento deve ser feita nos dois idiomas no mesmo commit.
