# 00 — Auditoria de 11/11/2026: achados com evidência

Base: commit `cc2b1d4` (v0.1.0 + PRs 9–12). Lidos por inteiro: `loopsettle-core`
(`engine.rs`, `engines.rs`, `ledger.rs`, `metrics.rs`, `lib.rs`), `loopsettle-topology`
(`cdfd.rs`, `netting.rs`, `hodge.rs`, `lib.rs`), `loopsettle-sim` (`bound.rs`, `stream.rs`,
`runner.rs`, parte de `scenario.rs`), `loopsettle-cli` (`main.rs`, partes de `realdata.rs`,
`experiments3.rs`, `experiments5.rs`), `docs/ASSESSMENT`, `CLAIMS`, `LIMITATIONS`.
**Não lidos:** `calibrate.rs`, `csvfile.rs` (só assinaturas), `toml_lite.rs`, `json.rs`,
`rng.rs`, `linalg.rs`, `graph.rs`, `experiments.rs`, `experiments2.rs`, `experiments4.rs`,
`loopsettle-wasm`, `web/`, `scripts/`. Quem continuar deve auditar esses.

Legenda: `MEDIDO` (número e comando), `LIDO` (visto no código, sem teste), `HIPÓTESE`.

## O que está certo (não mexer sem motivo)

| Item | Evidência |
|---|---|
| `cargo test --workspace` passa | MEDIDO nesta sessão, Linux, Rust 1.97 |
| CSVs de `baseline`, `microcycle-sweep`, `robustness` e `regimes` se reproduzem byte a byte | MEDIDO: rodei os quatro e `git diff` só mostrou `code_version` e tempos nos JSON |
| Conservação e atomicidade do `Ledger::try_batch` | LIDO: checa o efeito líquido por pagador e só aplica se todos passam |
| Prova e teste do limite de atraso (forma de janela) | LIDO + teste `engines_never_beat_the_delay_bound` |
| CDFD máxima como transbordo de custo mínimo | LIDO + oráculo de PL em 1.000 grafos |

## Achados

### A1 — O limite de atraso era folgado por construção. **Resolvido (spec 01).** MEDIDO
A forma de janela usa só o instante do pico `t*` e conta o próprio pagamento que faz o pico.
Usando todos os instantes: `piso_i(d) = max_s [CO_i(s) − CI_i(s+d)]`. No varejo calibrado, 10 s:
0,761% → 0,054% do UB; a compensação a cada 10 s obtém 51% do limite apertado (antes 3,6%).
`results/regimes/bound_curve.csv`.

### A2 — A afirmação 1f de `CLAIMS` não se sustenta com o limite apertado. **Corrigida.** MEDIDO
No alto valor estilizado com 2.000 pagamentos/dia: janela 13,7%, apertado 0,003%
(`results/regimes/regimes.csv`). O limite de janela não explica as economias de LSM em alto
valor; a explicação é a base de comparação (FIFO com liquidez restrita). Ver spec 06.
Ainda citam o argumento antigo e precisam de revisão: `docs/ASSESSMENT*.md` (Atualização 3),
`README*.md`, `paper/paper.pt-BR.md`, `docs/NOVELTY*.md`, `docs/PAPER_OUTLINE*.md`. Ver spec 02.

### A3 — `QueueOnShortfall` libera no máximo um pagamento por aresta por janela. LIDO
`engines.rs:276` (`retry_gross`): o instantâneo filtra `is_head` antes do laço, então quando a
cabeça de uma aresta liquida, o próximo da mesma aresta só é tentado na janela seguinte. Também
não repete a passada quando um recebimento libera saldo para alguém já visitado. Efeito
esperado: atraso superestimado e mais rejeições por prazo em liquidez restrita
(`microcycle-sweep/shortfall.csv`). Não afeta os resultados com liquidez ilimitada. Spec 04.

### A4 — Em `QueueOnShortfall`, pagamento novo fura a fila do próprio pagador. LIDO, conhecido (D13)
`engines.rs` `submit`: liquida bruto na chegada mesmo com pagamentos mais antigos do mesmo
pagador na fila. É uma política válida (bypass FIFO), mas precisa ser opção nomeada ao lado de
FIFO estrito, porque a literatura compara as duas. Specs 04 e 05.

### A5 — `DeferredNet` trunca o intervalo. LIDO
`engines.rs:32`: `interval = MS_PER_DAY / cycles_per_day`. Para divisores não exatos (7, 9,
11, 13...) os ciclos derivam e o último não cai no fim do dia, ao contrário do que a
documentação da struct diz. Os experimentos atuais usam 1, 4, 6, 10, 24, 48, 96, 1440 e
8640 ciclos por dia, e todos dividem 86.400.000 ms sem resto, então nenhum resultado publicado
é afetado. Risco só para usuários do CLI. Spec 04.

### A6 — `fraction_at_most(2800)` conta até 2.815 ms. LIDO
`metrics.rs:86`: soma o balde inteiro que contém 2.800 (largura 32 ms nessa faixa). A métrica
"parcela liquidada em até 2,8 s" pode ficar otimista. `quantile` devolve o limite inferior do
balde (subestima até 1,6%). Os atrasos de micro-ciclo são múltiplos da janela na maioria, então
o efeito hoje deve ser pequeno, mas a métrica é uma meta regulatória. Spec 04.

### A7 — `find_cycle` recomeça do zero após cada ciclo liquidado. LIDO
`engines.rs:234` (`run_cycles`): custo O(ciclos × (n + m)) por janela, com `BTreeMap` para
arestas bloqueadas. Com 30 milhões de pagamentos/dia e janela de 10 s pode dominar o tempo.
Sem número medido: medir antes de otimizar. Spec 04.

### A8 — Tudo é medido com liquidez ilimitada contra bruto sem fila. LIDO
Único experimento restrito: `shortfall.csv` no `microcycle-sweep`. Não há a curva
liquidez × rejeição × atraso, nem o baseline "FIFO com fila" da literatura. Spec 06.

### A9 — Estatística desigual entre experimentos. LIDO
`robustness` usa 10 sementes e IC; `regimes`, `funding-windows`, `heterogeneity`, `stress`,
`finland-replication`, `baseline` usam uma. Os números novos do limite apertado também são de
uma semente. As sementes rodam em série (`experiments3.rs:97`), com 4 threads ociosas. Spec 09.

### A10 — O CI só roda `cargo test`. LIDO
`.github/workflows/ci.yml`: sem `clippy`, sem `fmt --check`, sem build do wasm, sem teste de
regressão dos resultados, sem verificação de documentos gêmeos. Spec 09.

### A11 — Reprodutibilidade entre plataformas não foi verificada. HIPÓTESE
O gerador usa `exp`, `ln`, `powf` de `f64` (16 ocorrências em `crates/`), que em Rust chamam a
libm do sistema. Linux (glibc) e Windows (MSVC) podem divergir no último bit, o que mudaria
valores em centavos e, daí, todos os resultados. Nesta sessão os CSVs versionados se
reproduziram em Linux; **falta saber em que sistema eles foram gerados** e comparar com o
Windows do dono. Se divergirem, a afirmação 9 de `CLAIMS` ("bit a bit") vale só por plataforma.
Spec 09.

### A12 — Chegadas de Poisson puro, sem rajadas. LIDO, conhecido (LIMITATIONS 5)
Importa mais agora: o limite apertado depende de quanto entra nos `d` segundos depois de cada
saída, e rajadas correlacionadas (folha, lotes de credenciadora) mudam exatamente isso. Spec 07.

### A13 — Hodge não informa neste grafo. Conhecido (ASSESSMENT D.5)
Sem ação nova além da spec 08 (modelo nulo como saída padrão do comando `topology`).

### A14 — CDFD por caminhos mínimos sucessivos não tem teste de escala além de 5.000 arestas. LIDO
`cdfd.rs`: um Dijkstra por aumento; o número de aumentos não tem limite polinomial forte.
Grafos on-chain (Fase 8) podem ter 10⁵–10⁶ arestas. Spec 08.

## Não é problema (verificado para não gastar tempo de novo)

- `Ledger::try_batch` checa só pagadores: correto, recebedor puro nunca fica negativo.
- Invariante "a frente de `deadlines` ainda na fila é cabeça da sua aresta": vale, porque
  pagamentos mais antigos da mesma aresta saem antes ou expiram antes.
- `UNLIMITED = 2^50` centavos: cabe em `i64` com milhares de participantes; somas em `i128`.
- Potenciais de Johnson no fluxo de custo mínimo: custos iniciais ≥ 0, potenciais zero válidos.
