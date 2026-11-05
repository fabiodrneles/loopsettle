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
