# 09 — Estatística, reprodutibilidade e CI (P0)

Achados A9, A10, A11. Objetivo: que qualquer número do repositório tenha incerteza declarada,
se reproduza em qualquer máquina e não possa mudar sem alguém perceber.

## E1 — Várias sementes em todos os experimentos (A9)

- Função comum em `loopsettle-cli` (hoje `mean_sd`, `t95`, `quantile` vivem em
  `experiments3.rs`): `replicate(cfg, seeds, f) -> Vec<T>` e `summarize(values) -> {média,
  desvio, IC 95%, mín., máx.}`. Mover para um módulo `stats.rs` com testes.
- Todo experimento aceita `[experiment] seeds = [...]` (padrão: as 10 de `robustness.toml`) e
  grava, para cada métrica numérica, `_mean`, `_ci_lo`, `_ci_hi`. `--quick` usa 2 sementes.
- Experimentos de uma semente hoje: `baseline`, `potential-vs-realizable`, `microcycle-sweep`,
  `stress`, `finland-replication`, `heterogeneity`, `funding-windows`, `regimes`.
- **AC:** `scripts/summarize.py` só imprime números com IC, exceto onde a tabela é declarada
  "ilustração de uma semente"; `docs/CLAIMS*.md` cita IC em toda afirmação quantitativa.

## E2 — Sementes em paralelo dentro do orçamento

- `std::thread::scope`, sem dependência nova. Número de threads = `min(núcleos físicos, 2)` no
  padrão (o i3 tem 2 núcleos), configurável por `LOOPSETTLE_THREADS`.
- Cada thread roda uma semente inteira; os resultados são **ordenados por semente** antes de
  gravar, para a saída não depender do agendamento.
- Memória: cada execução usa poucos MiB (medido: 5 a 16 MiB), então 2 a 4 em paralelo cabem.
- **AC:** saída byte a byte igual com 1, 2 e 4 threads (teste no CI); `make rebuild` medido
  antes e depois e registrado em `CHECKPOINT*.md`.

## E3 — Reprodutibilidade entre plataformas (A11, HIPÓTESE)

1. **Verificar primeiro.** Descobrir em que sistema os CSVs versionados foram gerados (os
   commits "results: ... (clean code version)"). Rodar `make rebuild` em Linux x86-64 (feito
   nesta sessão para 4 experimentos: idênticos), Windows MSVC (máquina do dono) e, se possível,
   macOS arm64. Comparar com `git diff --stat results/`.
2. **Se divergir:** as funções transcendentais vêm da libm do sistema. Opções, em ordem:
   (a) implementar `exp`, `ln` e `powf` próprios e determinísticos em `loopsettle-core/src/
   rng.rs` (só são usados na amostragem; precisão de 1e-12 basta); (b) crate `libm` (Rust
   puro, uma dependência pequena, justificar pela regra de dependências). Em qualquer caso os
   resultados mudam uma vez: regenerar tudo num commit só, explicado.
3. **Se não divergir:** registrar as plataformas testadas em `CLAIMS*.md` (afirmação 9) e
   manter o teste no CI (E4) para pegar regressão.
- A soma em `f64` em ordem fixa já é determinística; não usar `mul_add`/FMA implícito
  (conferir que nenhum `RUSTFLAGS` liga `target-cpu=native`: o CLAUDE.md já proíbe).

## E4 — CI que protege os resultados (A10)

`.github/workflows/ci.yml` passa a ter:

| Job | Comando | Para quê |
|---|---|---|
| test | `cargo test --workspace --locked` | já existe |
| lint | `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` | higiene (corrigir os avisos atuais antes de ligar o `-D`) |
| golden | `make experiments-quick` + comparação com `results-golden/` | nenhum número muda sem um commit que atualize o golden |
| matrix | o job `golden` em `ubuntu-latest`, `windows-latest`, `macos-latest` | E3, continuamente |
| wasm | `wasm-pack build` + `node scripts/wasm-smoke.mjs` | a interface não quebra |
| docs | script que confere que todo `X.md` listado tem `X.pt-BR.md` alterado no mesmo PR | D17 |

- `results-golden/`: saídas `--quick` (cenário pequeno, segundos) de todos os experimentos,
  versionadas. Mais um arquivo `results/MANIFEST.sha256` com o hash de cada CSV completo,
  gerado por `make rebuild` e conferido por `make verify`.
- **AC:** um PR que mude um motor sem atualizar o golden fica vermelho; o tempo total do CI
  fica abaixo de 10 minutos.

## E5 — Testes que faltam

- **Diferencial dos motores:** um simulador de referência ingênuo (guarda todos os pagamentos,
  O(n²), só em teste) para `GrossRealTime`, `DeferredNet` e para os micro-ciclos com busca
  exaustiva de ciclos em instâncias minúsculas; comparar com os motores em proptest.
- **Fuzz dos leitores:** `csvfile.rs` e `toml_lite.rs` nunca entram em pânico com entrada
  arbitrária (proptest com strings aleatórias; hoje `main.rs` faz `unwrap` em vários pontos).
- **Metamórficos:** multiplicar todos os valores por k multiplica UB, LB e pisos por k;
  permutar rótulos de participantes não muda os totais; deslocar todos os horários por Δ não
  muda nada. Baratos e pegam erros de índice e de unidade.
- **Mutação (opcional):** `cargo-mutants` uma vez, fora do CI, para achar código sem teste;
  registrar o resultado em `REVIEW*.md`.

## E6 — Proveniência

Cada `summary.json` já tem semente, versão e comando. Acrescentar: hash do `Cargo.lock`,
`rustc --version`, tripla do alvo, hash dos arquivos de calibração usados, e o esquema
(`"schema": "loopsettle.result/1"`) para que ferramentas de terceiros leiam com segurança.

## Como validar antes de implementar

```bash
cargo clippy --workspace --all-targets 2>&1 | tail -20
```

```bash
make rebuild && git diff --stat results/
```
