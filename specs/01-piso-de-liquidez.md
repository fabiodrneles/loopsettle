# 01 — Piso de liquidez (limite apertado) — IMPLEMENTADO em 11/11/2026

## Enunciado

Para qualquer mecanismo em que todo pagamento que chega em `a` liquida em `[a, a + d]`, com
`CO_i(s)` = valor que `i` pagou até `s` e `CI_i(s)` = valor que `i` recebeu até `s` (horários
de chegada, inclusive):

```text
piso_i(d) = max(0, max_s [CO_i(s) − CI_i(s + d)]) ≤ L_i ≤ UB_i
```

Prova: no instante `s + d`, todo pagamento de `i` que chegou até `s` já liquidou (prazo), e
nenhum recebimento liquida antes de chegar, logo `−y_i(s+d) ≥ CO_i(s) − CI_i(s+d)`.

Propriedades (todas testadas):
- `piso_i(0) = UB_i`; `piso_i` não cresce com `d`; `piso_i(∞) ≥ LB_i`.
- `UB_i − piso_i(d) ≤ O_i[t* − d, t*]` (o limite de janela é o caso `s = t* − d`).
- É atingido por um participante sozinho (segura os próprios pagamentos por exatamente `d`,
  recebe na hora). A soma sobre participantes em geral não é atingível: é isso que a spec 03 mede.

## O que foi feito

| Onde | O quê |
|---|---|
| `crates/loopsettle-sim/src/bound.rs` | `liquidity_floors_from`, `liquidity_floors`: uma passada atrasada e uma adiantada por `d`, simultâneas; memória O(prazos × participantes) |
| mesmo arquivo, testes | `floor_hand_example`, `floor_is_ub_at_zero_delay_and_never_grows`; `engines_never_beat_the_delay_bound` agora também exige `L_i ≥ piso_i` e `UB_i − piso_i ≤ janela_i` para 6 motores × 5 cenários × 25 participantes |
| `crates/loopsettle-cli/src/realdata.rs` | `loopsettle bound` informa `liquidity_floor`, `tight_bound`, `tight_bound_pct_of_ub`; com `--per-participant`, colunas `floor_<d>s` |
| `crates/loopsettle-cli/src/experiments5.rs` | `regimes.csv` ganha `tight_bound_pct`; `bound_curve.csv` ganha `tight_bound_pct` e `realised_over_tight_bound`; a figura ganha a série |
| `docs/METHODOLOGY*.md` §9b, `docs/CLAIMS*.md` 1f (revista) e 1g (nova), `scripts/summarize.py` | documentação |

## Números (uma semente, σ = 0,4; ver spec 02 para o que falta)

| d | economia obtida (diferida a cada d) | limite janela | limite apertado | obtido / apertado |
|---|---|---|---|---|
| 10 s | 0,028% | 0,76% | 0,054% | 0,51 |
| 60 s | 0,071% | 2,67% | 0,25% | 0,28 |
| 15 min | 0,32% | 7,96% | 3,20% | 0,10 |
| 1 h | 1,05% | 23,1% | 7,69% | 0,14 |
| 6 h | 3,74% | > 100% | 9,23% | 0,41 |
| 1 dia | 9,27% | > 100% | 9,27% | 1,00 |

## Convenções de fronteira (quem mexer nos motores precisa manter)

- "Estado no instante `t`" é o estado depois de todos os eventos com horário ≤ `t`.
- Um pagamento que chega em `a` com prazo `d` tem de estar liquidado no estado do instante
  `a + d`. `AtomicMicroCycle::advance_to` faz a janela antes do prazo no mesmo instante.
- `DeferredNet` com intervalo `d`: pagamento que chega exatamente no horário do ciclo entra no
  ciclo seguinte (atraso exatamente `d`, permitido).
- Um motor novo com prazo que não respeite isso vai falhar no teste, e o defeito estará no motor.

## Perguntas em aberto

1. Existe enunciado igual na literatura? Procurar em: cálculo de redes (curvas de chegada e de
   serviço), literatura de "liquidity-delay trade-off" em RTGS (Bank of Canada WP 06-20,
   Galbiati & Soramäki, Jurgilas & Martin), Denbee & McLafferty. O critério K2 do ASSESSMENT
   se aplica também a este teorema. Se existir, citar e manter como implementação.
2. O piso por participante ignora que os recebimentos de `i` são pagamentos de outros que também
   podem ser segurados. A spec 03 trata disso.
