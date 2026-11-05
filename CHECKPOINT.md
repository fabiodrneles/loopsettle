# CHECKPOINT — progresso da construção do LoopSettle

> Este arquivo é o `PROGRESS.md` exigido pelo `CLAUDE.md` (o usuário pediu o nome
> "checkpoint"). Ele deve permitir retomar o trabalho numa sessão nova, sem contexto.
> Para retomar: abra o Claude Code em `E:\loopsettle` e diga
> **"Continue de onde parou, conforme o CHECKPOINT.md."**

## Estado atual

- **Fase atual:** 1 — Núcleo de liquidação (`loopsettle-core`)
- **Próximo passo exato:** implementar tipos (`Payment`, `Ledger`), trait `SettlementEngine`,
  motor `GrossRealTime`, cálculo de UB/LB, testes de propriedade das invariantes 1, 3 e 7.

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

## Histórico de fases

- [x] Fase 0 — Ambiente
- [ ] Fase 1 — Núcleo de liquidação
- [ ] Fase 2 — Topologia
- [ ] Fase 3 — Gerador sintético e streaming
- [ ] Fase 4 — Calibração com dados públicos
- [ ] Fase 5 — Motores alternativos
- [ ] Fase 6 — Experimentos reprodutíveis
- [ ] Fase 7 — Interface web
- [ ] Fase 8 — Validação externa (opcional)
- [ ] Fase 9 — Documentação
- [ ] Fase 10 — Pacote de revisão
