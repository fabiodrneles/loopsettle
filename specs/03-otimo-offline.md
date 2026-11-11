# 03 — Ótimo offline e o sanduíche piso ≤ ótimo ≤ motores (P1)

## Contexto

Hoje temos, para um prazo `d`: um limite inferior de liquidez por participante (piso, spec 01)
e limites superiores (os motores). Entre eles há uma folga (obtido / apertado = 0,5 em 10 s,
0,1 em 15 min). A pergunta que um revisor fará: **a folga é do limite ou dos motores?** Só o
ótimo do sistema inteiro responde. Nenhum simulador aberto de pagamentos entrega isso; é o
resultado que faz do LoopSettle um instrumento de medida, e não só um simulador.

## Formulação

Dados os pagamentos `p = (a_p, i_p → j_p, v_p)` e o prazo `d`. Instantes relevantes
`T = {a_p} ∪ {a_p + d}`. Variável `z_{p,τ} ≥ 0`: fração de `p` liquidada no instante `τ`,
`τ ∈ T ∩ [a_p, a_p + d]`, `Σ_τ z_{p,τ} = 1`. Liquidação simultânea num instante é atômica
(vale o efeito líquido, como em `Ledger::try_batch`).

```text
y_i(τ) = Σ_{p: j_p = i} v_p Σ_{τ' ≤ τ} z_{p,τ'} − Σ_{p: i_p = i} v_p Σ_{τ' ≤ τ} z_{p,τ'}
min Σ_i L_i   sujeito a   L_i ≥ −y_i(τ) ∀ i, τ;   L_i ≥ 0
```

- **PL (z fracionário):** permite fracionar pagamentos. Dá um limite inferior do sistema,
  `Σ piso_i ≤ PL`. É o "piso do sistema".
- **PLI (z binário):** o ótimo verdadeiro com pagamentos indivisíveis. `PL ≤ PLI ≤ motores`.

## Requisitos

- **FR1.** `loopsettle-sim/src/optimum.rs` (ou crate de testes): construir o PL para instâncias
  pequenas e resolver com `minilp` como **dev-dependency / feature opcional `oracle`**, nunca
  no caminho principal (CLAUDE.md, Fase 2). Tamanho alvo: ≤ 12 participantes, ≤ 400 pagamentos.
- **FR2.** PLI por força bruta ou branch-and-bound para ≤ 20 pagamentos, só em teste, para
  validar o PL e os motores em casos minúsculos.
- **FR3.** Teste de propriedade (proptest): para fluxos aleatórios pequenos e
  d ∈ {0, 1, 5, 50, ∞}: `Σ piso_i ≤ PL ≤ PLI ≤ L(motor)` para cada motor com prazo `d`;
  `PL(d=0) = UB`; `PL(d=∞) = LB`.
- **FR4.** Experimento `optimality-gap`: cenários reduzidos (10 participantes, 1 hora do dia,
  ~300 pagamentos, 20 sementes, σ ∈ {0,4; 1,0}), d ∈ {2, 10, 60, 600} s. Saída: `gap.csv` com
  `piso`, `PL`, melhor motor, e as razões `(UB − PL)/(UB − piso)` e `(UB − motor)/(UB − PL)`,
  com IC. Figura `fig_sandwich.svg`.
- **FR5 (pesquisa, opcional).** Método que escale: o PL tem estrutura de fluxo em rede
  expandida no tempo (cada pagamento é um arco com janela de tempo; `L_i` é capacidade na
  origem). Investigar se vira fluxo de custo mínimo resolvível pelo `Mcf` de `cdfd.rs`, ou
  decompor por janelas deslizantes de largura `2d` (pagamentos separados por mais de `d` não
  interagem diretamente, mas as posições acumuladas acoplam tudo: provar ou refutar antes de usar).
- **FR6.** Documentar em `METHODOLOGY*.md` §9c e em `CLAIMS*.md` o que o sanduíche permite
  afirmar em escala pequena e o que **não** se transfere para a escala do Pix.

## Critérios de aceite

- AC1. FR3 passa com 500 casos, e há um exemplo calculado à mão fixado em teste (sugestão: o
  triângulo A→B 10, B→C 5, C→A 5 com chegadas em 0, 1 e 2 ms e d = 2 ms; escrever no teste a
  conta de `UB`, `piso`, `PL` e `PLI` passo a passo, como no teste do triângulo da topologia).
- AC2. `experiment optimality-gap` roda em < 60 s e < 1 GB e grava CSV + JSON com sementes.
- AC3. O texto diz com números onde está a folga: no limite, nos motores, ou nos dois.

## Como validar antes de implementar

1. Conferir que `minilp` já é dev-dependency de `loopsettle-topology` (`Cargo.toml`) e que
   resolve PLs com ~5.000 variáveis em tempo aceitável; se não, reduzir o tamanho alvo.
2. Fazer à mão um caso de 3 pagamentos e conferir `PL`, `PLI` e `piso`.

## Riscos e perguntas

- O PL pode ser degenerado e lento no `minilp`; a grade `T` cresce com 2 × pagamentos. Mitigar
  agregando instantes sem evento.
- Atomicidade "no mesmo instante" é permissiva (qualquer conjunto simultâneo). Os motores reais
  só liquidam simultaneamente ciclos; o ótimo pode usar lotes que não são ciclos. Declarar isso:
  o ótimo é do "liquidante onisciente com lotes arbitrários", um teto para qualquer LSM.
- Se `PL ≈ Σ piso_i` nos casos pequenos, o piso por participante já é praticamente o ótimo do
  sistema, e isso é um resultado publicável por si só.
