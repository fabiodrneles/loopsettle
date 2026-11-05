# LoopSettle

**Simulador aberto de liquidez circular em pagamentos instantâneos** ·
*An open simulator of circular liquidity in instant payment systems*

[Português](#português) | [English](#english)

---

## Português

### Pergunta de pesquisa

> Quanto da liquidez que os participantes do Pix precisam manter é circular, quanto disso é
> recuperável na prática, e qual o custo em atraso de recuperá-la com micro-ciclos atômicos?

O Pix liquida cada pagamento de forma bruta e em tempo real. Parte do dinheiro que os
participantes precisam ter na conta de liquidação só "dá voltas" (A→B→C→A) e poderia, em
tese, ser compensada. O LoopSettle mede essa parte com três lentes — decomposição de
**Hodge** (diagnóstico), **CDFD** máxima (teto estático) e **netting por ciclos**
(executável) — e compara a liquidez e os atrasos de vários desenhos de liquidação, inclusive
um mecanismo novo de **micro-ciclos atômicos**.

### Como rodar (3 comandos)

```bash
cargo test --workspace                                                   # testes e invariantes
cargo run --release -p loopsettle-cli -- experiment potential-vs-realizable  # comparação central
cd web && npm install && npm run wasm && npm run dev                     # interface no navegador
```

Tudo de uma vez, do zero: `make rebuild` (testes, todos os experimentos,
`results/SUMMARY.md` e build da interface). Lista de experimentos:
`cargo run --release -p loopsettle-cli -- list`.

Requisitos: Rust estável (com target `wasm32-unknown-unknown`), `wasm-pack`, Node LTS,
Python 3 (só biblioteca padrão). Roda num i3 de 4ª geração com 4 GB de RAM: cada experimento
usa menos de 20 MiB e termina em segundos (números em `CHECKPOINT.md`).

### Estrutura

| Caminho | Conteúdo |
|---|---|
| `crates/loopsettle-core` | tipos, ledger em centavos, motores `GrossRealTime`, `DeferredNet`, `AtomicMicroCycle`, métricas em streaming |
| `crates/loopsettle-topology` | Hodge (gradiente conjugado à mão), netting por ciclos, CDFD máxima (fluxo de custo mínimo exato) |
| `crates/loopsettle-sim` | gerador sintético calibrado, em streaming; calibração; runner |
| `crates/loopsettle-cli` | experimentos reprodutíveis → `results/<nome>/` (CSV, JSON, SVG) |
| `crates/loopsettle-wasm` + `web/` | interface Vite + React + Canvas 2D, simulação num Web Worker |
| `data/` | download dos agregados públicos do Pix (API Olinda do BCB) |
| `experiments/` | definição de cada experimento (TOML) |
| `docs/` | `METHODOLOGY.md`, `PARAMETERS.md`, `LIMITATIONS.md`, `CLAIMS.md`, `NOVELTY.md`, `PAPER_OUTLINE.md` |
| `DECISIONS.md`, `REVIEW.md` | decisões de desenho e pacote de revisão |

### Trabalhos de referência

- Hellqvist, M. & Korpinen, K. *Instant payments as the new normal* (Banco da Finlândia,
  2021; JFMI, 2026) — comparação de liquidez entre modelo com ciclos e modelo instantâneo,
  com dados sintéticos calibrados. **Base metodológica que estendemos.**
- Estudo sobre Pix e liquidez bancária (BCB, Working Paper 619) — perda de netting no Pix;
  mede capacidade **potencial**, não realizável.
- *Circular Directional Flow Decomposition of Networks* (arXiv 2506.12546).
- Bech, M. & Soramäki, K. *Gridlock Resolution in Interbank Payment Systems* (Banco da
  Finlândia, DP 9/2001).
- PixSim (arXiv 2609.30684) — exemplo de calibração com fonte para cada parâmetro.

### Limitações (resumo — ver `docs/LIMITATIONS.md`)

- **Dados sintéticos calibrados, não dados interbancários reais.** Só agregados públicos são
  usados; a estrutura bilateral (quem paga a quem) não é pública e é a principal hipótese.
- Escala reduzida: 200 participantes e 1 milhão de pagamentos por dia (o Pix real tem
  ~239 milhões por dia); a liquidez absoluta não escala linearmente.
- O motor de micro-ciclos usa uma heurística simples (um pagamento por aresta, ciclos de até
  6 participantes), não um otimizador completo.

### Licença

Apache-2.0 — ver `LICENSE`.

---

## English

### Research question

> How much of the liquidity that Pix participants must hold is circular, how much of it is
> recoverable in practice, and what delay does recovering it with atomic micro-cycles cost?

Pix settles every payment gross and in real time. Part of the money participants keep in
their settlement accounts only goes around in loops (A→B→C→A) and could, in principle, be
netted. LoopSettle measures that part with three lenses — **Hodge** decomposition
(diagnostic), maximum **CDFD** circulation (static ceiling) and **cycle netting**
(executable) — and compares liquidity and delays across settlement designs, including a new
**atomic micro-cycle** mechanism.

### Run it (3 commands)

```bash
cargo test --workspace
cargo run --release -p loopsettle-cli -- experiment potential-vs-realizable
cd web && npm install && npm run wasm && npm run dev
```

Everything from scratch: `make rebuild`. Code is in English; documentation is in Portuguese
(this README is bilingual).

### Key references

Hellqvist & Korpinen (Bank of Finland 2021; JFMI 2026); BCB Working Paper 619;
arXiv 2506.12546 (CDFD); Bech & Soramäki (BoF DP 9/2001); PixSim (arXiv 2609.30684).

### Limitations

Synthetic data calibrated to public aggregates — **not real interbank data**; reduced scale;
heuristic micro-cycle search. See `docs/LIMITATIONS.md`.

### License

Apache-2.0.
