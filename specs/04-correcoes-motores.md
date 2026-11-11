# 04 — Correções nos motores e nas métricas (P0)

Achados A3, A4, A5, A6, A7 de `00-auditoria.md`. Todos `LIDO`: o primeiro passo de cada um é
escrever o teste que demonstra o defeito.

## C1 — `retry_gross` libera um pagamento por aresta por janela (A3)

`crates/loopsettle-core/src/engines.rs`, `retry_gross`.

- **Teste que deve falhar hoje:** liquidez restrita; A com saldo 0 enfileira 3 pagamentos de 10
  para B (mesma aresta); em seguida A recebe 30 de C (C com saldo). Na janela seguinte, os três
  deveriam liquidar no mesmo instante. Hoje: um por janela.
- **Segundo teste:** cadeia A→B, B→C na fila, B sem saldo, A com saldo, e a ordem de chegada
  é B→C antes de A→B. Depois de A→B liquidar, B→C deveria liquidar na mesma janela.
- **Correção:** repetir a passada até ponto fixo (nenhuma liquidação na passada), ou fila de
  trabalho: quando `p` liquida, reexaminar a nova cabeça da aresta de `p` e as cabeças das
  arestas que saem do recebedor. Manter a ordem determinística (ordem de chegada).
- **AC:** os dois testes passam; invariantes 1 a 5 e 7 continuam passando; com liquidez
  ilimitada nenhum CSV muda; `microcycle-sweep/shortfall.csv` muda e a mudança é explicada em
  `REVIEW.md` (esperado: menos rejeições e atraso menor).

## C2 — Disciplina de fila explícita em `QueueOnShortfall` (A4, D13)

- Novo campo `MicroCycleConfig::discipline: {BypassFifo, StrictFifoPerPayer}`; padrão
  `BypassFifo` (comportamento atual). Com `StrictFifoPerPayer`, um pagamento novo só liquida
  bruto na chegada se o pagador não tiver nada na fila, e `retry_gross` para no primeiro
  pagamento do pagador que não couber.
- Especificação de motor: `micro:shortfall:<janela>:<atraso>[:len[:fifo|bypass]]`.
- **AC:** teste em que o bypass liquida um pagamento pequeno novo e o FIFO estrito não;
  `EngineSpec::parse` e `to_json` cobrem o campo; D13 atualizada nos dois idiomas.

## C3 — `DeferredNet` com número de ciclos que não divide o dia (A5)

- Calcular o horário do ciclo `k` como `k * MS_PER_DAY / cycles_per_day` (inteiro, sem
  acumular erro), de modo que o último ciclo de cada dia caia exatamente no fim do dia.
- **Teste:** `cycles_per_day = 7`, três dias; o 7º, o 14º e o 21º ciclos caem em múltiplos
  exatos de `MS_PER_DAY`; `deferred:1` continua igual ao LB (`runner.rs`, teste existente).
- **AC:** nenhum CSV de `results/` muda (todos os valores usados dividem 86.400.000).
- Atenção: o teste do limite usa "atraso máximo = intervalo". Com intervalos desiguais, o
  atraso máximo passa a ser `ceil(MS_PER_DAY / cycles_per_day)`.

## C4 — Métrica da meta de 2,8 s exata (A6)

`crates/loopsettle-core/src/metrics.rs`.

- `RunMetrics` ganha contadores exatos para limiares de SLA configuráveis (padrão `[2_800]`
  ms; o Manual de Tempos do Pix também tem um percentil 99: conferir o valor no documento em
  `docs/references/` antes de fixar um segundo limiar).
- `LogHistogram::quantile`: documentar o erro (≤ 1/64 para baixo) e oferecer
  `quantile_bounds(q) -> (inferior, superior)`.
- **Teste:** 100 atrasos de 2.810 ms: `share_within_2800ms` = 0 (hoje dá 1).
- **AC:** `share_settled_within_2800ms` em todos os JSON passa a vir do contador exato; os
  CSVs que mudarem são listados em `REVIEW.md`.

## C5 — Busca de ciclos incremental (A7)

Só depois de **medir**: rodar `experiment robustness` e ver quanto do tempo é `run_cycles` com
30 milhões de pagamentos/dia (usar `perf` ou contadores de tempo por função).

- Se for relevante (> 30% do tempo): manter o estado da DFS entre ciclos da mesma janela
  (técnica do "arco corrente", já usada em `loopsettle-topology/src/netting.rs`), e trocar
  `blocked: BTreeMap<(u32,u32),()>` por conjunto de bits por aresta ativa.
- **Exigência dura:** os resultados não podem mudar. O conjunto de ciclos liquidados e a ordem
  têm de ser idênticos aos da versão atual; fixar isso com um teste diferencial que roda as
  duas implementações (a antiga fica atrás de `#[cfg(test)]`) em 200 fluxos aleatórios e
  compara o log de ciclos (`AtomicMicroCycle::with_log`).
- Se a ordem tiver de mudar por desempenho, isso é decisão que muda resultados: registrar em
  `DECISIONS.md` e regenerar tudo.

## C6 — Pequenos

- `drive` (`lib.rs`) entra em pânico com pagamento inválido (`amount <= 0`, pagador =
  recebedor). Para arquivos reais o `csvfile` deve filtrar e contar antes; conferir que
  `rows_skipped` cobre os dois casos e que há teste.
- `Ledger::apply` e `FlowStats::observe` somam `i64` sem checagem. Em `release` o estouro é
  silencioso. Como o projeto promete rodar em dados reais de terceiros, usar `checked_add` com
  erro claro nesses dois pontos (medir o custo: deve ser desprezível) ou validar no `scan` do
  CSV que a soma dos valores cabe com folga.
- `AtomicMicroCycle::finish` usa `debug_assert_eq!(self.queued, 0)`: virar `assert`.

## Como validar o conjunto

```bash
cargo test --workspace
```

```bash
make rebuild
```

```bash
git diff --stat results/
```

Esperado: mudam só os arquivos de liquidez restrita e os que usam a parcela de 2,8 s.
