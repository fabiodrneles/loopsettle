# 06 — Fronteira com liquidez restrita: liquidez × rejeição × atraso (P1)

> **Estado em 11/11/2026: feito** (`experiments6.rs`, `results/liquidity-frontier/`, CLAIMS 1h,
> D24, REVIEW 84). Diferenças em relação ao texto abaixo: os desenhos rodam em dois horizontes
> (10 s e 1 hora), porque com 10 s nada muda e a comparação com a literatura só aparece com espera
> longa; a grade de α vai até 2; o Bech-Soramäki (M3) não entrou. Resposta à hipótese do FR4: sim
> na direção (esperas de 1 h economizam 3,7–5,3% no alto valor estilizado contra 1,1–1,4% no varejo
> calibrado), mas só com espera longa. **12/11/2026:** grade refinada (passos de 0,01 entre 0,97 e 1); os números mudaram até 2×,
> ver REVIEW 86; o tempo subiu para 5 min 49 s. Melhorias pendentes: regra
> `outflow` na tabela de liquidez equivalente; política de nova tentativa; M3.

## Contexto

Achado A8. Todos os resultados centrais medem "quanta liquidez cada desenho precisa para nunca
rejeitar" com saldo ilimitado. Um operador de sistema instantâneo faz a pergunta inversa:
**com a liquidez que os participantes de fato mantêm, quantos pagamentos são rejeitados ou
atrasam, e quanto um mecanismo reduz isso?** É também a única forma de comparar com McMahon et
al. (2024), que medem economia contra FIFO com fila sob liquidez restrita. Com o limite
apertado (spec 01) mostrando ~0 nos dois regimes sob liquidez ilimitada, esta spec passa a ser
onde a diferença entre varejo e alto valor tem de aparecer, se existir.

Num sistema instantâneo, rejeição é o que o usuário final vê (o Pix falha). Isso dá ao estudo
uma métrica de bem-estar que os estudos de alto valor não têm.

## Requisitos

- **FR1.** Experimento `liquidity-frontier`. Saldo de abertura de cada participante
  `= α × UB_i`, com `α ∈ {0,5; 0,6; 0,7; 0,8; 0,9; 0,95; 1,0}`. Regra alternativa
  configurável: `α × LB_i + β × (UB_i − LB_i)` e "proporcional ao volume de saída" (um
  participante não conhece o próprio UB de antemão). Registrar a regra como decisão.
- **FR2.** Motores: `gross` (rejeita), `queue:fifo` e `queue:bypass` (spec 05 M1) com prazo de
  10 s, `micro:shortfall` e `micro:all` com janelas 500 ms e 2 s e prazos 2, 5, 10 s, e o
  Bech-Soramäki (M3) quando existir.
- **FR3.** Métricas por (α, motor), com 10 sementes e IC 95%: parcela rejeitada em quantidade
  e em valor; atraso p50, p95, p99 dos liquidados; parcela liquidada em até 2,8 s (contador
  exato, spec 04 C4); valor liquidado em ciclos; **liquidez equivalente**: o menor α com que o
  motor atinge a mesma taxa de rejeição que o `gross` tem em α = 1 (zero) ou em um alvo dado
  (por exemplo 0,01% do valor). A diferença de α é a economia de liquidez "a serviço igual".
- **FR4.** Rodar em três cenários: varejo calibrado σ = 0,4, varejo σ = 1,0 e o alto valor
  estilizado do `regimes`. Hipótese a testar: no alto valor a liquidez equivalente dos
  mecanismos cai bastante (como em McMahon et al.); no varejo, pouco.
- **FR5.** Saídas: `frontier.csv`, `equivalent_liquidity.csv`, `fig_frontier.svg` (rejeição em
  valor × α, uma linha por motor, escala log no eixo y), `summary.json`.
- **FR6.** Um teste de coerência: em α = 1 nenhum motor rejeita; a rejeição do `gross` não
  cresce quando α cresce (por semente).

## Critérios de aceite

- AC1. Um comando gera tudo; tempo < 5 min e RAM < 1 GB no hardware de referência (se 7 α × ~10
  motores × 10 sementes × 1 milhão de pagamentos não couber, reduzir para 300 mil pagamentos e
  documentar o fator, ou usar as threads da spec 09).
- AC2. `docs/CLAIMS*.md` ganha afirmações novas só com IC, e a 1f recebe a conclusão final
  sobre varejo × alto valor.
- AC3. `docs/NOVELTY*.md` compara a medida de "liquidez equivalente" com a de McMahon et al.
  citando a página do artigo (o PDF está em `docs/references/pdf/`).

## Como validar antes de implementar

1. Ler `results/microcycle-sweep/shortfall.csv` e o código que o gera (`experiments2.rs`) para
   reaproveitar a montagem de saldos restritos.
2. Fazer a correção C1 da spec 04 primeiro: sem ela o `micro:shortfall` sai pior do que é.
3. Reler a seção de metodologia de McMahon et al. para alinhar a definição de baseline.

## Riscos e perguntas

- **Rejeição em cascata:** no Pix real um pagamento rejeitado pode ser tentado de novo pelo
  usuário. Sem modelo de nova tentativa, a rejeição é "primeira tentativa". Declarar; uma
  política de nova tentativa pode ser parâmetro (ASSUMPTION).
- **Liquidez é endógena:** participantes ajustam o saldo ao desenho. O experimento mede
  resposta de curto prazo com saldos dados. Declarar em `LIMITATIONS*.md`.
- O saldo `α × UB_i` usa informação do próprio dia (olhar para a frente). A regra "proporcional
  ao volume" evita isso; reportar as duas.
