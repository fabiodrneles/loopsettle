# CHECKPOINT — progresso da construção do LoopSettle

> Este arquivo é o `PROGRESS.md` exigido pelo `CLAUDE.md` (o usuário pediu o nome
> "checkpoint"). Ele deve permitir retomar o trabalho numa sessão nova, sem contexto.
> Para retomar: abra o Claude Code em `E:\loopsettle` e diga
> **"Continue de onde parou, conforme o CHECKPOINT.md."**

## Estado atual

- **Fase atual:** 8 — Validação externa (opcional) → depois 9 e 10
- **Próximo passo exato:** Fase 8 qualitativa (correlação potencial de Hodge × posição
  líquida, já calculada em `results/baseline`); on-chain fica documentado como pendente
  (exige baixar dados externos). Depois README, METHODOLOGY final, Zenodo, `make all`,
  pacote de revisão (REVIEW, SUMMARY, CLAIMS, NOVELTY, PAPER_OUTLINE).

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
| 5 | `experiment microcycle-sweep` (16 motores + 3×3 restritos, 1M pagamentos) | 11,6 s | 6 MiB | — |
| 6 | `experiment baseline` (inclui topologia de 10.492 arestas e 4 sensibilidades) | 5,9 s | 16 MiB | < 1 GB ✅ |
| 7 | `wasm-pack build --release` (primeiro build, baixa wasm-bindgen-cli) | 432 s | 1,16 GB | < 2,5 GB ✅ |
| 7 | wasm `simulate` 50 part./100 mil pagamentos (Node) | 0,6 s | — | fluido ✅ |
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
- [x] Fase 5 — Motores alternativos
  - Pronto: `DeferredNet` (deltas O(n)), `AtomicMicroCycle` (filas FIFO por aresta, DFS
    limitada, `try_batch` atômico, prazos exatos, modos `QueueAll`/`QueueOnShortfall`),
    `loopsettle_sim::runner` (vários motores numa passada, specs `gross`, `deferred:N`,
    `micro:all|shortfall:janela:atraso[:len]`), gráficos SVG próprios.
  - Testar: `cargo test -p loopsettle-core` (invariantes 1–5, 7 nos motores; DeferredNet
    1×/dia = LB); curva: `cargo run --release -p loopsettle-cli -- experiment microcycle-sweep`
    → `results/microcycle-sweep/{sweep.csv,shortfall.csv,fig_tradeoff.svg,summary.json}`.
  - Achado preliminar: com liquidez ilimitada os micro-ciclos (≤10 s) liquidam até 63% do
    valor em ciclos mas economizam ≤0,015% do UB; no cenário calibrado LB/UB ≈ 0,91.
- [x] Fase 6 — Experimentos reprodutíveis
  - Pronto: `experiment {calibration, baseline, potential-vs-realizable, microcycle-sweep,
    stress, finland-replication}` → `results/<nome>/` (CSV + `summary.json` com seed,
    versão do código, comando, pico de RAM; SVGs).
  - Números principais (seed 20260930, 1M pagamentos, 200 participantes):
    - UB = R$ 71,27 mi; LB = R$ 64,67 mi (LB/UB = 0,91); volume bruto R$ 443 mi.
    - Potencial estilo BCB = 0,854; economia potencial fim do dia (UB−LB)/UB = 9,27%.
    - Escada bruta: Hodge L1 1,18 > CDFD 0,83 > ciclos 0,80; líquida: Hodge L2 0,98 ≫
      CDFD 0,17 > ciclos 0,14.
    - Realizável: micro-ciclo melhor (w500, d10s) economiza 0,0147% do UB, liquidando 63%
      do valor em ciclos; DeferredNet a cada 10 s: 0,028%; a cada 15 min: 0,32%.
    - Finlândia: instantâneo/1 ciclo por dia = 1,10× (σ=0,4) a 1,50× (σ=0).
    - Stress (maior participante fora 30 min): efeito desprezível no UB total.
- [x] Fase 7 — Interface web
  - Pronto: `loopsettle-wasm` (`simulate(...) -> JSON`, calibração embutida), `web/`
    (Vite 5 + React 18 + TS, Web Worker, Canvas 2D: rede com componente circular CDFD,
    sliders de janela/atraso/liquidez/cenário, barras de liquidez, curva por janela, dia em
    15 min, tabela com liquidez restrita; modo reduzido em telas ≤ 640 px ou ≤ 2 núcleos).
  - Rodar: `cd web && npm install && npm run wasm && npm run dev` (ou `npm run build &&
    npm run preview`). Teste de fumaça: `node scripts/wasm-smoke.mjs`.
  - Verificado: build ok; wasm em Node 0,2 s (20 part./20 mil) e 0,6 s (50/100 mil);
    preview serve página, `.wasm` (application/wasm) e worker.
  - **NÃO verificado:** renderização visual num navegador real e no celular (sem
    ferramenta de navegador na sessão) — o usuário deve abrir e conferir.
- [ ] Fase 8 — Validação externa (opcional)
- [ ] Fase 9 — Documentação
- [ ] Fase 10 — Pacote de revisão
