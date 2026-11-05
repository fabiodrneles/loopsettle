# results/

> [English](#english) · [Português](#português)

## English

Every file in this folder is produced by a command with a fixed seed (`20260930`) and is
reproduced **byte for byte** when re-run. Do not edit by hand: run `make all` (or
`make rebuild` to start from scratch).

**Start here:** [`SUMMARY.md`](SUMMARY.md) — every number in tables, with the seed, code version
and command of each experiment.

| Folder | Question | Command | Main files |
|---|---|---|---|
| [`calibration/`](calibration) | Does the generator reproduce the public Pix aggregates? | `cargo run --release -p loopsettle-cli -- experiment calibration` | `calibration.csv` (real vs simulated, relative error) |
| [`baseline/`](baseline) | How much of the flow is circular, through the three lenses? | `… experiment baseline` | `ladder.csv` (Hodge / CDFD / cycles), `engines.csv`, `participants.csv` (UB and LB per participant), `sensitivity.csv` (receiver bias), `fig_ladder.svg` |
| [`potential-vs-realizable/`](potential-vs-realizable) | How much of the potential netting becomes saved liquidity? (central result) | `… experiment potential-vs-realizable` | `central.csv` (each measure with its meaning and status), `engines.csv` (every design), `fig_potential_vs_realizable.svg` |
| [`microcycle-sweep/`](microcycle-sweep) | What is the trade-off between delay and liquidity saving? | `… experiment microcycle-sweep` | `sweep.csv` (window × max delay), `shortfall.csv` (constrained liquidity), `fig_tradeoff.svg` |
| [`stress/`](stress) | What if a participant is offline for 30 minutes — and does it matter who and when? | `… experiment stress` | `stress.csv` (largest participant at 10:00), `cases.csv` (3 targets × 3 start times: spillover and rejections) |
| [`finland-replication/`](finland-replication) | The Bank of Finland logic (fixed cycles vs instant) in this scenario | `… experiment finland-replication` | `ratios.csv` (instant / N cycles per day, mean over 5 seeds), `fig_finland.svg` |
| [`robustness/`](robustness) | Does the result survive seeds, scale, receiver bias and cycle length? | `… experiment robustness` | `seeds_summary.csv` (10 seeds, 95% CIs), `scale.csv` (1M–30M), `cycle_len.csv`, `participants_ratio.csv`, `fig_scale.svg` |
| [`bias-calibration/`](bias-calibration) | Which receiver bias reproduces the per-bank nettable ratios of BCB WP 619 (real Pix data)? | `… experiment bias-calibration` | `grid.csv` (σ × payment-institution multiplier, quartiles and fit) |
| [`heterogeneity/`](heterogeneity) | What if each type of institution has its own intraday profile? (kill criterion K1) | `… experiment heterogeneity` | `heterogeneity.csv` (λ × scale × σ, savings and delay bound) |
| [`funding-windows/`](funding-windows) | How much must be prefunded for nights and the weekend, when the STR is closed? | `… experiment funding-windows` | `windows.csv` (need, end-of-window netting, delay bounds per window) |
| [`regimes/`](regimes) | Does the delay bound differ between retail and large-value systems? Realised saving vs bound over d | `… experiment regimes` | `regimes.csv`, `bound_curve.csv`, `fig_bound.svg` |
| [`topology-triangle/`](topology-triangle) | The fixed triangle case (A→B 10, B→C 5, C→A 5) | `… topology data/samples/triangle.csv` | `ladder.csv` |

In each folder, `summary.json` holds the scenario, seed, code version and command;
`run.json` holds the machine measurements (time and RAM), the only file that changes between
runs. Amounts are in reais (R$) from the **synthetic** scenario: each simulated payment stands
for ~239 real payments, and absolute values do not scale linearly to real Pix — the fractions
are the result. What the numbers can and cannot support: [`../docs/CLAIMS.md`](../docs/CLAIMS.md).

---

## Português

Cada arquivo desta pasta é gerado por um comando com semente fixa (`20260930`) e é reproduzido
**byte a byte** ao rodar de novo. Não edite à mão: rode `make all` (ou `make rebuild` para
começar do zero).

**Comece por aqui:** [`SUMMARY.pt-BR.md`](SUMMARY.pt-BR.md) — todos os números em tabelas, com a
semente, a versão do código e o comando de cada experimento.

| Pasta | Pergunta | Comando | Arquivos principais |
|---|---|---|---|
| [`calibration/`](calibration) | O gerador reproduz os agregados públicos do Pix? | `cargo run --release -p loopsettle-cli -- experiment calibration` | `calibration.csv` (real × simulado, erro relativo) |
| [`baseline/`](baseline) | Quanto do fluxo é circular, pelas três lentes? | `… experiment baseline` | `ladder.csv` (Hodge / CDFD / ciclos), `engines.csv`, `participants.csv` (UB e LB por participante), `sensitivity.csv` (viés recebedor), `fig_ladder.svg` |
| [`potential-vs-realizable/`](potential-vs-realizable) | Quanto da compensação potencial vira liquidez economizada? (resultado central) | `… experiment potential-vs-realizable` | `central.csv` (cada medida com significado e status), `engines.csv` (todos os desenhos), `fig_potential_vs_realizable.svg` |
| [`microcycle-sweep/`](microcycle-sweep) | Qual a troca entre atraso e economia de liquidez? | `… experiment microcycle-sweep` | `sweep.csv` (janela × atraso máximo), `shortfall.csv` (liquidez restrita), `fig_tradeoff.svg` |
| [`stress/`](stress) | E se um participante ficar fora do ar por 30 minutos — importa quem e quando? | `… experiment stress` | `stress.csv` (maior participante às 10h), `cases.csv` (3 alvos × 3 horários: efeito nos outros e rejeições) |
| [`finland-replication/`](finland-replication) | A lógica do Banco da Finlândia (ciclos fixos × instantâneo) neste cenário | `… experiment finland-replication` | `ratios.csv` (instantâneo / N ciclos por dia, média de 5 sementes), `fig_finland.svg` |
| [`robustness/`](robustness) | O resultado sobrevive a sementes, escala, viés recebedor e comprimento do ciclo? | `… experiment robustness` | `seeds_summary.csv` (10 sementes, IC 95%), `scale.csv` (1–30 milhões), `cycle_len.csv`, `participants_ratio.csv`, `fig_scale.svg` |
| [`bias-calibration/`](bias-calibration) | Que viés recebedor reproduz as razões de compensação por banco do BCB WP 619 (dados reais do Pix)? | `… experiment bias-calibration` | `grid.csv` (σ × multiplicador das IPs, quartis e ajuste) |
| [`heterogeneity/`](heterogeneity) | E se cada tipo de instituição tiver seu próprio perfil ao longo do dia? (critério de morte K1) | `… experiment heterogeneity` | `heterogeneity.csv` (λ × escala × σ, economias e limite de atraso) |
| [`funding-windows/`](funding-windows) | Quanto é preciso pré-financiar para as noites e o fim de semana, com o STR fechado? | `… experiment funding-windows` | `windows.csv` (necessidade, compensação no fim da janela, limites de atraso por janela) |
| [`regimes/`](regimes) | O limite de atraso muda entre varejo e sistemas de alto valor? Economia obtida × limite em função de d | `… experiment regimes` | `regimes.csv`, `bound_curve.csv`, `fig_bound.svg` |
| [`topology-triangle/`](topology-triangle) | O caso fixo do triângulo (A→B 10, B→C 5, C→A 5) | `… topology data/samples/triangle.csv` | `ladder.csv` |

Em cada pasta, `summary.json` guarda o cenário, a semente, a versão do código e o comando; o
`run.json` guarda as medições da máquina (tempo e RAM), o único arquivo que muda entre
execuções. Os valores em reais (R$) são do cenário **sintético**: cada pagamento simulado
representa ~239 pagamentos reais e os valores absolutos não escalam linearmente para o Pix
real — as frações são o resultado. O que os números permitem ou não afirmar:
[`../docs/CLAIMS.pt-BR.md`](../docs/CLAIMS.pt-BR.md). Os rótulos das figuras estão em inglês.
