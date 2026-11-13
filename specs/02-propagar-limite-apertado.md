# 02 — Propagar o limite apertado para todos os experimentos e documentos (P0)

## Contexto

O piso de liquidez (spec 01) só aparece em `regimes` e em `loopsettle bound`. Os experimentos
que sustentam a afirmação central (`robustness`, `heterogeneity`, `funding-windows`) e todos os
documentos de síntese ainda usam só a forma de janela. Os números do limite apertado são de uma
semente com σ = 0,4, o que `CLAIMS` ("não pode afirmar" 11) proíbe publicar.

## Estado em 11/11/2026

**FR1 feito** (`experiments3.rs`, `results/robustness/`): 10 sementes × σ ∈ {0; 0,4; 0,8; 1,0}
e escala de 1 a 30 milhões. Limite apertado de 10 s: 0,138% (σ = 0), 0,041% [0,025; 0,056]
(σ = 0,4), 0,018% (σ = 0,8), 0,014% [0,007; 0,021] (σ = 1,0); com 30 milhões, 0,024%. Obtido /
apertado: 0,38–0,46 (diferida a cada 10 s), 0,13–0,28 (micro-ciclos). `CLAIMS` 1g já usa esses
números. **FR2 feito** (`experiments4.rs`, `results/heterogeneity/`): com perfis por classe e 30
milhões de pagamentos, o limite apertado de 10 s é no máximo 0,042% (janela: 0,46%); K1 segue
longe de disparar. **FR3 feito** (`bound.rs::window_floors`, METHODOLOGY §10): dentro das janelas fechadas o piso
ganha um termo de fronteira (o que foi recebido nos `d` antes da janela); é mais apertado que a
forma de janela nas 120 combinações do `funding-windows`; no fim de semana com σ calibrado o
limite de 10 s é ≤ 0,03%. **FR4 feito** de outro modo: as tabelas das seções 7, 9, 10 e 11 do
`SUMMARY` trazem obtido, apertado e janela lado a lado, e `scripts/summarize.py::check_sandwich`
recusa escrever o resumo se alguma linha violar obtido ≤ apertado ≤ janela (AC2). **FR6 feito**
(README, ASSESSMENT Atualização 4, NOVELTY, PAPER_OUTLINE, METHODOLOGY); o rascunho do JOSS **citava**
sim o limite antigo e o argumento de alto valor (a busca não pegou por causa de uma quebra de
linha); corrigido em 12/11/2026. **Faltam FR5 (distribuição do piso por participante) e FR7 (interface
web).** A escala (R2) e a heterogeneidade continuam com uma
semente por ponto.

## Requisitos

- **FR1.** `robustness` (`experiments3.rs`, struct `Measure`): acrescentar `tight_pct` por motor
  (via `liquidity_floors`), com média, desvio e IC 95% por σ em `seeds_summary.csv`, e a coluna
  em `scale.csv` (1 a 30 milhões de pagamentos/dia).
- **FR2.** `heterogeneity` (`experiments4.rs:121`): coluna do limite apertado ao lado da de
  janela. Reavaliar o critério de morte K1 com o apertado (deve ficar ainda mais longe de 2%).
- **FR3.** `funding-windows` (`bound.rs` `window_stats`): piso dentro de cada janela fechada,
  partindo da posição no início da janela: `max_s [(CO(s) − CO(a)) − (CI(min(s+d, b)) − CI(a))]`
  para `s ∈ [a, b)`. Escrever o teste à mão ao lado de `window_need_is_drawdown_from_the_window_start`.
- **FR4.** Nova tabela "sanduíche" em `results/SUMMARY*.md`: para cada d ∈ {2, 5, 10, 60, 900,
  3600} s: obtido (micro-ciclo, diferida), apertado, janela, com IC.
- **FR5.** Curva do piso por participante: `regimes/floor_by_participant.csv` com a distribuição
  (p10, p50, p90, máx.) de `(UB_i − piso_i)/UB_i` para 10 s e 15 min. Mostra se algum
  participante isolado teria ganho relevante mesmo com o agregado perto de zero.
- **FR6.** Documentos: trocar, onde o limite de atraso é citado como resultado, a forma de
  janela pela apertada (mantendo a de janela como corolário): `README*.md`,
  `docs/ASSESSMENT*.md` (nova "Atualização 4": o item D.6 "o limite é folgado" está resolvido;
  a reconciliação com McMahon et al. mudou), `docs/NOVELTY*.md`, `docs/PAPER_OUTLINE*.md`,
  `paper/paper.pt-BR.md`, `docs/CLAIMS*.md` 1b, 1d, 1e (faixas novas com IC), `CHANGELOG.md`.
- **FR7.** `web/` e `loopsettle-wasm`: mostrar o limite apertado no painel "potencial vs
  realizável" (pode ficar para a spec 10 se o wasm não expuser a passada dupla).

## Critérios de aceite

- AC1. `seeds_summary.csv` tem `tight_bound_pct_mean` e IC para cada σ ∈ {0; 0,4; 0,8; 1,0}.
- AC2. Em toda linha de todo CSV: `obtido ≤ apertado ≤ janela` (teste automático em
  `scripts/` ou em `cargo test` que leia os CSVs de `results/`).
- AC3. Nenhum documento afirma mais que o limite de janela "explica" a diferença entre varejo e
  alto valor (`grep -ri "13[.,]7" docs README* paper` só em contexto histórico).
- AC4. `make rebuild` passa; os CSVs que não ganharam coluna ficam idênticos byte a byte.

## Como validar antes de implementar

```bash
cargo test -p loopsettle-sim bound
```

```bash
cargo run --release -q -p loopsettle-cli -- experiment regimes
```

Conferir que `results/regimes/bound_curve.csv` reproduz a tabela da spec 01.

## Riscos

- Custo: `liquidity_floors` gera o fluxo `1 + nº de prazos` vezes. Com 30 milhões de pagamentos
  e 4 prazos são 150 milhões de pagamentos gerados (≈ 35 s a 0,23 s por milhão). Aceitável;
  se pesar, juntar a passada atrasada com a passada do `drive`.
- A manchete muda de "o teto é 0,06–3,3%" para algo como "o teto é ~0,05% e os motores simples
  já obtêm metade". É uma afirmação mais forte e mais fácil de atacar: precisa das 10 sementes,
  dos quatro σ e da escala antes de ir para o artigo.
