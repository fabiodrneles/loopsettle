# Metodologia

> Documento vivo: cada fase acrescenta a sua seção. A versão final (Fase 9) explica cada
> métrica em linguagem simples e formal.

## 1. Liquidez necessária: limite superior (UB) e inferior (LB)

Seja `x_i(t)` a posição acumulada do participante `i` até o instante `t`:
`x_i(t) = entradas_i(t) − saídas_i(t)`, somando os pagamentos na ordem de chegada.

- **UB (limite superior)**: `UB_i = max(0, −min_t x_i(t))`. É o menor saldo inicial que
  evita qualquer rejeição no motor `GrossRealTime` (liquidação bruta em tempo real, como o
  Pix). Prova: o saldo em `t` é `saldo_inicial + x_i(t)`, e um pagamento só é rejeitado se o
  saldo ficaria negativo. O teste `ub_is_tight` verifica que, com saldo inicial = UB, nada
  é rejeitado, e que a liquidez consumida no modo ilimitado é exatamente UB.
- **LB (limite inferior)**: `LB_i = max(0, −x_i(T))`, a posição devedora líquida no fim do
  período `T`. É o que uma única compensação multilateral no fim do período exigiria.

Sempre `LB_i ≤ UB_i`, e `Σ_i x_i(T) = 0`.

### Exemplo à mão (teste `hand_example_ub_lb`)

Participantes A, B, C, saldos ilimitados:

| t | pagamento | x_A | x_B | x_C |
|---|---|---|---|---|
| 1 | A→B 100 | −100 | +100 | 0 |
| 2 | B→C 50 | −100 | +50 | +50 |
| 3 | C→A 70 | −30 | +50 | −20 |
| 4 | B→A 30 | 0 | +20 | −20 |

- UB = (100, 0, 20), total **120**.
- LB = (0, 0, 20), total **20**.
- Com saldo inicial (100, 0, 20) nada é rejeitado; com (99, 0, 20) o primeiro pagamento de
  A é rejeitado.

A diferença UB − LB = 100 é a liquidez que existe só por causa da **ordem temporal** dos
pagamentos: um sistema que pudesse compensar tudo no fim do período a economizaria.

## 2. Medição da liquidez de um motor

Cada motor roda com saldo inicial "ilimitado" (`UNLIMITED = 2^50` centavos). A liquidez que
o motor precisou é o **pico de queda do saldo** (`saldo_inicial − menor saldo atingido`).
Para `GrossRealTime` isso é exatamente o UB. Ver `DECISIONS.md`, D3.
