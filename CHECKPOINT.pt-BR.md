# CHECKPOINT — progresso da construção do LoopSettle

> [English version](CHECKPOINT.md)

> Este arquivo é o `PROGRESS.md` exigido pelo `CLAUDE.md` (o usuário pediu o nome
> "checkpoint"). Ele deve permitir retomar o trabalho numa sessão nova, sem contexto.
> Para retomar: abra o Claude Code em `E:\loopsettle` e diga
> **"Continue de onde parou, conforme o CHECKPOINT.md."** (a versão em inglês
> `CHECKPOINT.md` e esta versão `CHECKPOINT.pt-BR.md` têm o mesmo conteúdo).

## Estado atual

- **11/11/2026 — auditoria, piso de liquidez, specs.** Resultado novo: o piso de liquidez
  (`bound.rs::liquidity_floors`, METHODOLOGY §9b) é a forma apertada, por participante, do limite
  de atraso; para 10 s ele é 0,054% do UB em vez de 0,76%. A afirmação 1f foi revista por causa
  dele. O roteiro agora está em [`specs/`](specs/README.md): 11 especificações com evidência,
  critérios de aceite e passos de validação. **Próximo passo exato:** specs 04 (correções nos
  motores) → 09 (sementes, CI) → 02 (propagar o limite apertado e corrigir os documentos que ainda
  trazem o argumento antigo da 1f). Medido aqui (Linux, 4 núcleos): `cargo test --workspace`
  passa; `experiment regimes` 3,2 s, 7 MiB; os CSVs de baseline, microcycle-sweep, robustness e
  regimes se reproduzem byte a byte.
- **11/11/2026, mais tarde — specs 04 e 09 executadas.** Correções nos motores com testes de
  regressão (`tests/spec04.rs`), resultados-ouro conferidos no CI em três plataformas, sementes
  em paralelo. **Próximo passo exato:** spec 02 FR3–FR7 (janelas de abastecimento com o limite
  apertado, a tabela do sanduíche e os documentos que ainda trazem a afirmação 1f antiga), depois
  a spec 06. Antes de mexer em motor ou gerador, rode `make verify`; para aceitar uma mudança de
  resultados, rode `make golden` e faça o commit. Medido (Linux, 4 núcleos, 2 threads):
  `robustness` 75 s, `heterogeneity` 127 s, `funding-windows` 36 s, os demais abaixo de 8 s; pico
  de RAM de no máximo 16 MiB; checagem-ouro 50 s.
- **11/11/2026, mais tarde ainda — specs 02 (FR1–FR6) e 06 executadas; spec 05 M1.** Experimento
  novo `liquidity-frontier` (4 min 46 s, 20 MiB). **Próximo passo exato:** spec 03 (ótimo offline
  em instâncias pequenas), depois spec 05 M2 (`HoldToDeadline`, que prova que o piso é atingido) e
  M3; spec 02 FR7 e spec 10 para a interface web. Em aberto do REVIEW 84: grade de α mais fina
  perto de 1.
- **11/11/2026, fim da sessão — spec 03 executada.** Pisos = ótimo offline em 116 de 120
  instâncias (`results/optimality-gap/`, 55 s). **Próximo passo exato:** spec 05 M2 e M3, spec 10
  (relatório para dados reais, painel web), depois 07 e 08; os itens em aberto de cada spec estão
  no topo dela.
- **12/11/2026 — pagamentos em eco: um critério de morte disparou.** A afirmação central é
  condicional à ausência de devoluções bilaterais rápidas (CLAIMS 1j, ASSESSMENT atualização 5,
  REVIEW 87). Também feito: `--report` e saída por participante privada (spec 10 R2, R3), grade de
  liquidez mais fina (REVIEW 86). **Próximo passo exato:** levar a condição para
  `paper/paper.pt-BR.md` e `docs/PAPER_OUTLINE*.md`; depois os itens em aberto do REVIEW 87 (ecos
  com valor parcial, outras escalas). Medido: `echo-sweep` 148 s, 10 MiB; `liquidity-frontier`
  349 s, 20 MiB.
- **12/11/2026, fim da sessão.** Feito desde a última entrada: `echo-scale` (o efeito do eco cai
  com expoente −0,52 no volume), motores `gridlock` e `bilateral` (spec 05 M3, M4; D27), curva do
  limite com 10 sementes, `circularity-horizon` (spec 08 T3), exemplo `custom_engine` (spec 10 R5),
  busca de literatura (REFERENCES §8). **Não feito, na ordem pedida pelo dono:** spec 05 M5
  (política de abastecimento); spec 08 T1 (BFF), T2 (escala do fluxo de custo mínimo), T4, T5; spec
  10 R1 (ordenação externa, mapeamento ISO 20022), R4 (painel web); várias sementes em `baseline`,
  `potential-vs-realizable`, `microcycle-sweep`, `stress`, `heterogeneity`, `funding-windows`.
  **Próximo passo exato:** spec 05 M5, depois esses, nessa ordem. Sempre rodar `make verify` antes
  de commitar.
- **12/11/2026 — spec 05 completa** (M5, camada de abastecimento; M3 ainda sem conferência com o
  artigo de 2001, cujos servidores estão bloqueados aqui). **Próximo passo exato:** spec 08 T1, T2,
  T4, T5; spec 10 R1, R4; várias sementes nos seis experimentos antigos.
- **Fase atual:** construção concluída (Fases 0–10). Fase 8 parcial (on-chain pendente).
- **Próximo passo exato:** revisão humana guiada por `REVIEW.pt-BR.md` (69 pontos; 62–69 são da
  revisão do README de 05/11/2026). Sugestões de
  trabalho adicional estão nos pontos 13, 14, 36, 39 e 41 do `REVIEW.pt-BR.md`. Reservar o nome
  (GitHub/crates.io) só quando o usuário pedir ("vamos reservar o nome").
- **Interface verificada visualmente em 05/11/2026** (Chromium desktop e iPhone 13 emulado;
  capturas em `docs/img/ui-*.png`, regeneradas por `node scripts/screenshots.cjs`). Corrigido o
  travamento de `npm run dev` (React StrictMode encerrava o Web Worker; ver REVIEW 66). Falta
  abrir no notebook real (i3, 4 GB).
- **Documentação bilíngue (D17):** README com seções completas em PT e EN (capturas em cada
  idioma: `docs/img/` e `docs/img/en/`); inglês é o nome padrão e o português tem sufixo `.pt-BR.md` (D17),
  `results/SUMMARY.md` + `SUMMARY.pt-BR.md` (gerados por `scripts/summarize.py`), `data/README.md`
  nos dois idiomas, interface com seletor PT/EN (`?lang=pt|en`). Regra: mudou um documento,
  muda o gêmeo no mesmo commit.
- **05/11/2026, mais tarde:** CLI para dados reais (`bound`/`settle`/`export`), W2/W2b, W3
  (`funding-windows`) e W5 (`regimes`, curva do limite) feitas; K1–K3 não dispararam; a próxima fase é
  escrever (preprint e JOSS, `docs/PAPER_OUTLINE.pt-BR.md`).
- **05/11/2026, fechamento da fase de construção:** artigo de McMahon et al. lido (arXiv v3; LVTS,
  dados reais; REVIEW 76); estresse refeito com 3 alvos × 3 horários (`results/stress/cases.csv`,
  REVIEW 77); biblioteca de referências com os PDFs em `docs/references/` (REVIEW 78); README com
  guia de instalação do zero (Windows, Linux, macOS), referência de comandos e problemas comuns.
  Pendências obrigatórias do código: nenhuma. Opcionais: CDFD proporcional/BFF (D6), FIFO estrito
  (REVIEW 14), validação on-chain (Fase 8). Do seu lado: abrir a interface no notebook i3 e
  submeter. **Próxima fase: só artigos.**
- **JOSS (05/11/2026):** rascunho em português em `paper/paper.pt-BR.md` para você ler, com
  `paper/paper.bib` e a lista de prontidão `paper/README.pt-BR.md`. A triagem do JOSS exige mais de
  seis meses de histórico público e iterativo e impacto em pesquisa demonstrado, então a ordem é
  **preprint primeiro, JOSS a partir de maio de 2027**. CONTRIBUTING e CHANGELOG criados; falta
  criar a tag v0.1.0.
- **Referências conferidas** por busca pública: ver `docs/REFERENCES.pt-BR.md` (PixSim não
  localizado; título da CDFD corrigido; número de 8,6% do estudo finlandês a conferir).

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
| 10 | `make rebuild` completo | 120 s | < 1,2 GB (build wasm) | ✅ |
| 2 | `bench_topology` (200 nós, 5.000 arestas, 31.062 triângulos) | 0,22 s | 6,9 MB | < 5 s e < 300 MB ✅ |

## Histórico de fases

- [x] Fase 0 — Ambiente
- [x] Fase 1 — Núcleo de liquidação
  - Pronto: `types`, `ledger` (centavos, sem saldo negativo, lote atômico), trait
    `SettlementEngine` + `GrossRealTime`, `drive()` (streaming, vários motores numa passada),
    `FlowStats` (UB, LB, capacidade potencial, matriz de pares), `RunMetrics` + histograma
    log-linear de atrasos, PRNG próprio + alias table, escritor JSON.
  - Testar: `cargo test -p loopsettle-core` (11 testes; invariantes 1, 3, 7 por proptest;
    exemplo à mão em `docs/METHODOLOGY.pt-BR.md`).
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
    (todos os erros ≤ 1,5%; tabela em `docs/PARAMETERS.pt-BR.md`).
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
  - Verificado visualmente em 05/11/2026 (Chromium desktop e iPhone 13 emulado, PT e EN).
    Falta abrir no notebook real (i3, 4 GB).
- [x] Fase 8 — Validação externa (opcional) — parcial
  - Feito: correlação potencial de Hodge × posição líquida = 0,997 (Pearson), 0,926
    (Spearman) no baseline (reproduz qualitativamente; quase mecânico em grafo denso).
    Comando `loopsettle topology <csv>` para qualquer lista `from,to,amount`.
  - Pendente: baixar amostra on-chain de stablecoin (ver `data/README.md`).
- [x] Fase 9 — Documentação
  - README bilíngue, `docs/METHODOLOGY.pt-BR.md` completo, `docs/LIMITATIONS.pt-BR.md`,
    `CITATION.cff`, `.zenodo.json` (Zenodo); JOSS fica para depois.
- [x] Fase 10 — Pacote de revisão
  - `REVIEW.pt-BR.md`, `results/SUMMARY.pt-BR.md` (gerado por `scripts/summarize.py`), `docs/CLAIMS.pt-BR.md`,
    `docs/NOVELTY.pt-BR.md`, `docs/PAPER_OUTLINE.pt-BR.md`; figuras SVG em `results/*/`.
  - Aceite: `make rebuild` (do zero: testes + 6 experimentos + resumo + build web + teste de
    fumaça do wasm) terminou com exit 0 em 120 s. Rodar os experimentos de novo reproduz
    `summary.json`, CSV, SVG e `SUMMARY.md` byte a byte (só `run.json`, com tempo e RAM, varia).
