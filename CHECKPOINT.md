# CHECKPOINT — progresso da construção do LoopSettle

> Este arquivo é o `PROGRESS.md` exigido pelo `CLAUDE.md` (o usuário pediu o nome
> "checkpoint"). Ele deve permitir retomar o trabalho numa sessão nova, sem contexto.
> Para retomar: abra o Claude Code em `E:\loopsettle` e diga
> **"Continue de onde parou, conforme o CHECKPOINT.md."**

## Estado atual

- **Fase atual:** 5 — Motores alternativos
- **Próximo passo exato:** `DeferredNet { cycles_per_day }` e `AtomicMicroCycle
  { window_ms, max_delay_ms, mode: QueueAll | QueueOnShortfall }` em `loopsettle-core`;
  teste de propriedade da invariante 4 (atomicidade); comando da curva economia × p95.

## Ambiente (Fase 0) — CONCLUÍDA

| Item | Valor |
|---|---|
| SO | Windows 10 Pro 19045 (toolchain nativo MSVC, sem WSL2) |
| CPU / RAM | i3-4160 (2C/4T), 4 GB (≈ 0,9 GB livres no momento da medição) |
| Rust | 1.99.1 (rustup, `stable-x86_64-pc-windows-msvc`) + target `wasm32-unknown-unknown` |
| wasm-pack | 0.13.1 (binário pré-compilado em `~/.cargo/bin`) |
| Node / npm | 24.19 / 11.17 |
| Python | 3.13 (scripts de dados, só stdlib) |
| make | GNU Make 4.4.1 |

Notas de ambiente:
- No Git Bash, rode `export PATH="$HOME/.cargo/bin:$PATH"` se `cargo` não for encontrado
  (o PATH só é atualizado em shells novos após a instalação do rustup).
- Medição de tempo/RAM: `powershell -ExecutionPolicy Bypass -File scripts/measure.ps1 "<cmd>"`.
- rust-analyzer: na máquina de 4 GB, recomenda-se `"rust-analyzer.checkOnSave": false`.

## Números medidos

| Fase | Comando | Tempo | RAM de pico | Aceite |
|---|---|---|---|---|
| 0 | `cargo test --workspace` (frio, com proptest) | 61 s | 502 MB | < 120 s e < 2,5 GB ✅ |
| 3 | `bench_stream` (1.000.984 pagamentos, GRT + matriz de pares, 2×) | 0,23 s/execução | 4,1 MB | < 60 s e < 1 GB ✅ |
| 2 | `bench_topology` (200 nós, 5.000 arestas, 31.062 triângulos) | 0,22 s | 6,9 MB | < 5 s e < 300 MB ✅ |

## Histórico de fases

- [x] Fase 0 — Ambiente
- [x] Fase 1 — Núcleo de liquidação
  - Pronto: `types`, `ledger` (centavos, sem saldo negativo, lote atômico), trait
    `SettlementEngine` + `GrossRealTime`, `drive()` (streaming, vários motores numa passada),
    `FlowStats` (UB, LB, capacidade potencial, matriz de pares), `RunMetrics` + histograma
    log-linear de atrasos, PRNG próprio + alias table, escritor JSON.
  - Testar: `cargo test -p loopsettle-core` (11 testes; invariantes 1, 3, 7 por proptest;
    exemplo à mão em `docs/METHODOLOGY.md`).
- [x] Fase 2 — Topologia
  - Pronto: `FlowGraph` (bruto/líquido), CSR + CG à mão, Hodge (B1, B2 matrix-free,
    limite de triângulos), netting por ciclos (DFS arco corrente), CDFD máxima
    (min-cost flow exato), `analyze()` → escada bruta e líquida + correlação potencial×posição.
  - Testar: `cargo test -p loopsettle-topology` (invariantes 2, 5, 6, 8; CDFD = `minilp`
    em 1.000 grafos; parte direcional acíclica).
  - Benchmark: `cargo run --release -p loopsettle-topology --example bench_topology`.
  - Pendente (D6): variante proporcional/local da CDFD.
- [x] Fase 3 — Gerador sintético e streaming
  - Pronto: `ScenarioConfig`/`Population` (classes por ranking, viés recebedor),
    `PaymentStream` (Poisson por hora, alias tables, lognormal + Pareto truncada),
    `Outage` (indisponibilidade com re-envio em rajada), `toml_lite`.
  - Testar: `cargo test -p loopsettle-sim`; benchmark
    `cargo run --release -p loopsettle-sim --example bench_stream`.
- [x] Fase 4 — Calibração com dados públicos
  - Pronto: `data/fetch_pix.py` (Olinda; o servidor não aceita `$skip` → requisição única
    com verificação de truncamento), `data/calibration/pix.toml` (set/2026),
    `loopsettle_sim::calibrate`, CLI `experiment calibration`, `ScenarioConfig::from_doc`.
  - Testar: `cargo run --release -p loopsettle-cli -- experiment calibration`
    (todos os erros ≤ 1,5%; tabela em `docs/PARAMETERS.md`).
- [ ] Fase 5 — Motores alternativos
- [ ] Fase 6 — Experimentos reprodutíveis
- [ ] Fase 7 — Interface web
- [ ] Fase 8 — Validação externa (opcional)
- [ ] Fase 9 — Documentação
- [ ] Fase 10 — Pacote de revisão
