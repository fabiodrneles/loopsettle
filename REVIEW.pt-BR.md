# REVIEW — pontos a revisar antes de escrever o artigo

> [English version](REVIEW.md)

Cada ponto: **o que foi feito**, **onde está**, **resultado** e **risco/dúvida**.
Números de `results/SUMMARY.pt-BR.md` (seed 20260930, 200 participantes, 1 milhão de pagamentos,
calibração set/2026). Reproduzir tudo: `make rebuild`.

---

## A. Decisões de desenho (`DECISIONS.pt-BR.md`)

1. **D0 — `CHECKPOINT.md` no lugar de `PROGRESS.md`.** Pedido do usuário. Sem impacto em
   resultados.
2. **D1 — PRNG, CSV, JSON e TOML feitos à mão.** `core/src/{rng,json}.rs`,
   `sim/src/toml_lite.rs`. Resultado: determinismo bit a bit testado (invariante 7), inclusive
   entre execuções do `make`. Risco: o parser TOML aceita só um subconjunto (sem tabelas
   aninhadas nem strings multilinha); um TOML válido mais rico falha com mensagem de linha.
3. **D2 — Dependências:** `proptest` (dev), `minilp` (dev, oráculo), `wasm-bindgen`. Risco:
   nenhum no caminho principal.
4. **D3 — Liquidez = pico de queda do saldo com saldo "ilimitado".** `core/src/ledger.rs`.
   Resultado: no bruto coincide com o UB (teste `ub_is_tight`). Dúvida para o revisor: é a
   medida de Hellqvist & Korpinen? Conferir a definição exata no artigo deles. Para
   micro-ciclos, a medida ignora que esperar 10 s também pode reduzir rejeições em liquidez
   restrita (tratado à parte no experimento com liquidez restrita).
5. **D4 — Hodge no grafo líquido, L1 e L2.** `topology/src/hodge.rs`, `lib.rs`. Resultado:
   Hodge L2 rotacional = 0,977 no grafo líquido; L1 = 1,73 (líquido), 1,18 (bruto). Risco:
   a soma da massa bilateral à massa de Hodge no grafo bruto é uma convenção nossa; o
   revisor deve decidir se a escada no artigo usa o grafo líquido (mais limpo) ou o bruto.
6. **D5 — CDFD máxima via transbordo de custo mínimo.** `topology/src/cdfd.rs`. Resultado:
   igual ao `minilp` em 1.000 grafos; parte direcional acíclica (teste). Dúvida: confirmar
   que a "decomposição de circularidade máxima" do arXiv 2506.12546 é exatamente este
   problema (maximizar Σc com 0 ≤ c ≤ F e conservação). Ver também ponto 7.
7. **D6 — CDFD proporcional/local (BFF) não implementada.** Risco: o artigo da CDFD dá
   destaque à variante local; sem ela, a comparação com o artigo fica parcial.
8. **D7 — Netting por ciclos guloso (DFS, ordem de entrada).** `topology/src/netting.rs`.
   Resultado: 0,795 (bruto) vs CDFD 0,831 — a ordem gulosa perde 3,6 p.p. Dúvida: a ordem
   (lexicográfica por id de participante) favorece participantes de id baixo (os maiores).
9. **D8 — Poisson com taxa constante por hora.** `sim/src/stream.rs`. Risco: sem rajadas.
10. **D9 — Recebedor gravitacional com viés recebedor.** `sim/src/stream.rs`. Resultado: é o
    parâmetro que mais move os resultados (LB/UB de 0,59 a 0,95 quando σ vai de 0 a 0,8).
    **Maior risco do estudo** (ver ponto 15).
11. **D10 — Valores: corpo lognormal + cauda B2B.** `sim/src/calibrate.rs`. Resultado: erros
    ≤ 1,5% nos agregados.
12. **D11 — Tamanho ∝ chaves Pix.** Risco: chaves ≠ transações.
13. **D12 — Micro-ciclos: uma cabeça por aresta, ciclos ≤ 6, bloqueio na falta de saldo.**
    `core/src/engines.rs`. Risco: heurística; um otimizador completo pode compensar mais.
    Teste de sensibilidade com `max_cycle_len` ∈ {2, 3, 6, 10} ainda não rodado.
14. **D13 — `QueueOnShortfall` sem FIFO por pagador.** Resultado: com 25% do UB, rejeita
    **mais** valor que o bruto (23,23% vs 22,95%). Dúvida: implementar FIFO estrito como
    alternativa configurável antes de concluir algo sobre LSM.
15. **D14 — Vencidos vão para bruto; sem saldo, rejeitados.**
16. **D15 — `DeferredNet` sempre liquida (DNS colateralizado).** Saldo pode ficar negativo
    no modo restrito — por isso o `deferred_net_10x` nunca rejeita no estresse.
17. **D16 — Interface: uma função wasm que devolve JSON.**

## B. Hipóteses (`ASSUMPTION` em `docs/PARAMETERS.pt-BR.md`)

18. `participants = 200` (escala). 19. `days = 1`. 20. `payments_per_day = 1.000.000`.
21. `zipf_s` (não usado com calibração). 22. `n_big = 5`, `n_mid = 15`.
23. `fintech_frac = 0,5`. 24. `payee_size_exp = 1`. 25. `homophily = 0,3`.
26. **`receiver_bias_sigma = 0,4`** — ver ponto 10; sem calibração possível com dados
    públicos usados. Sugestão: buscar algum agregado público de posições líquidas/uso de
    liquidez no SPI para ancorar LB/UB.
27. `pi_receiver_mult = 1,3`. 28. Perfil horário (`hourly`). 29. `body_sigma = 1,4`.
30. `tail_alpha = 1,3`. 31. `max_amount = R$ 5 mi`. 32. Estresse: 100% re-enviados.
33. Proxy embutido: B2B = "cauda" de valores grandes (pagamentos B2B não são
    necessariamente os maiores).

## C. Resultados a examinar

34. **Potencial vs realizável (resultado central).** Potencial estilo BCB = 85,4% do volume;
    economia potencial de liquidez com uma compensação no fim do dia = 9,27% do UB;
    realizável com micro-ciclos (≤10 s) = 0,0147% do UB, embora 63% do valor seja liquidado
    dentro de ciclos. Compensação a cada 10 s: 0,028%; a cada 15 min: 0,32%.
    Interpretação proposta: o UB é dominado por desequilíbrios de horas (deriva), não por
    ruído de segundos. **Risco:** depende da escala (ponto 41) e do viés (ponto 10). Com
    σ = 0, a economia sobe para 0,048% — continua desprezível.
35. **Escada:** bruto Hodge L1 1,18 > potencial 0,85 > CDFD 0,83 > ciclos 0,80;
    líquido Hodge L2 0,98 ≫ CDFD 0,17 > ciclos 0,14. A desigualdade "Hodge ≥ CDFD" valeu em
    todas as sensibilidades, mas não é um teorema (por isso o "≥?" do CLAUDE.md).
36. **Hodge em grafo denso** (LIMITATIONS 11): a fração rotacional alta pode ser efeito de
    dimensão. Sugestão de teste: Hodge sobre fluxos puramente aleatórios simétricos (sem
    estrutura) no mesmo grafo — se der ≈0,97 também, a métrica não discrimina estrutura.
37. **Correlação potencial de Hodge × posição líquida = 0,997** (Spearman 0,93). Quase
    mecânica em grafo denso (φ ≈ posição/n). Não deve ser vendida como descoberta.
38. **Finlândia:** instantâneo precisa de 1,10× (σ=0,4) a 1,50× (σ=0) a liquidez de 1 ciclo
    por dia; 1,01–1,03× com 48 ciclos. Dúvida: comparar com a ordem de grandeza publicada
    por Hellqvist & Korpinen (conferir o número no artigo).
39. **Estresse:** efeito desprezível no UB (+0,005%). O UB do participante 0 não muda porque
    seu pico de posição devedora está no fim do dia, e todos os pagamentos adiados são
    re-enviados. Dúvida: o cenário de estresse é fraco; considerar abandono parcial, ou
    indisponibilidade no horário do pico do participante, ou de um recebedor líquido.
40. **Liquidez restrita:** micro-ciclos `QueueAll` reduzem pouco a rejeição (22,43% vs 22,95%
    com 25% do UB) ao custo de p95 = 10 s e só 42% liquidados em até 2,8 s (meta do Pix: 50%).
    Só a configuração com atraso de 2 s cumpre a meta.
41. **Escala:** com 239× menos pagamentos por segundo, cada janela tem 239× menos candidatos
    a ciclo. Sugestão: rodar o `microcycle-sweep` com 10 e 30 milhões de pagamentos
    (cabe na memória: uso atual ~6 MiB; o tempo cresce ~linearmente a partir de 16 s por
    milhão, ou seja ~3 min e ~8 min).
42. **Percentis quantizados:** o histograma reporta o limite inferior do balde (ex.: 9.984 ms
    para 10.000 ms). Diferença < 1,6%; documentado.

## D. Testes difíceis de passar / armadilhas encontradas

43. `hodge_invariants` falhou no grafo vazio: tolerância `1e-7·|E|` virava 0 (`0 < 0`).
    Corrigido para `|E| + 1`. Não era erro do algoritmo.
44. Busca de ciclos limitada por comprimento: a primeira versão marcava como "explorado" um
    nó cuja subárvore foi cortada pelo limite — podia perder ciclos. Corrigido antes do
    primeiro teste (flag `truncated` propagada). Os testes de propriedade passaram de
    primeira depois disso; vale o revisor ler `find_cycle` com atenção.
45. Underflow possível no salto de janelas ociosas (`t - next_tick`), corrigido na revisão do
    código, antes dos testes.
46. API Olinda: `$skip` devolve HTTP 500 e datas exigem literal OData sem aspas; o script
    baixa tudo numa requisição e aborta se puder ter truncado.
47. Reprodutibilidade dos arquivos: a versão do código mudava ao commitar os próprios
    resultados; agora é o último commit fora de `results/`, e tempos/RAM vão para `run.json`.

## E. Limitações conhecidas (`docs/LIMITATIONS.pt-BR.md`)

48. Dados sintéticos; 49. estrutura bilateral hipotética; 50. chaves como proxy; 51. escala;
52. perfil horário sem fonte e sem rajadas; 53. horizonte de um dia; 54. micro-ciclos
heurísticos; 55. atraso medido é só o do motor; 56. sem CDFD proporcional; 57. validação
on-chain não feita; 58. Hodge em grafo denso; 59. interface: **verificada visualmente em
05/11/2026** (Chromium desktop 1440 px e iPhone 13 emulado, em português e inglês; capturas em
`docs/img/` e `docs/img/en/`,
geradas por `node scripts/screenshots.cjs`). Falta testar no notebook real (i3, 4 GB).

## F. Itens que dependem do revisor

60. Referências: conferidas por busca pública em 05/11/2026; situação de cada uma em
    `docs/REFERENCES.pt-BR.md`. Ainda falta abrir os textos integrais (os PDFs do Banco da
    Finlândia não abriram deste ambiente).
61. URL do repositório em `Cargo.toml`/`CITATION.cff` (`github.com/fabiodrneles/loopsettle`)
    é suposição até o repositório existir.
62. **PixSim (arXiv 2609.30684) não localizado.** Nenhuma busca pública encontrou o trabalho.
    Saiu do README; continua citado em `docs/NOVELTY.pt-BR.md` como "a confirmar". Não citar no
    artigo sem confirmar o identificador.
63. **Título da CDFD.** O arXiv 2506.12546 se chama *Circular Directional Flow Decomposition of
    Networks*. O título *How circular is a directed network?* e a publicação na *Royal Society
    Open Science*, que estavam no README, não foram confirmados; o README agora cita o arXiv.
64. **Número do estudo finlandês (≤ 8,6%).** Vem do resumo do debatedor no Economics of
    Payments X, não do texto do artigo. Conferir no BoFER 7/2021 e na versão JFMI 13(2), e
    conferir quantos ciclos por dia o modelo de referência do STEP2 usa: a comparação justa
    com o LoopSettle é contra esse número de ciclos (nosso +2,6% com 10 ciclos/dia, +9,5% com
    1 ciclo/dia, σ = 0,4).
65. **Heterogeneidade por banco (ponto forte do estudo finlandês).** O LoopSettle tem UB e LB
    por participante (`baseline/participants.csv`), mas o experimento `finland-replication`
    só reporta a razão agregada. Sugestão: adicionar a distribuição da razão instantâneo/ciclos
    por participante (participantes com LB = 0 precisam de tratamento à parte).
66. **Bug corrigido na interface:** em `npm run dev` a simulação nunca terminava ("Simulando…"
    para sempre). Causa: o React StrictMode (só em desenvolvimento) desmonta e remonta o
    componente; a limpeza encerrava o Web Worker e o `useMemo` devolvia o mesmo worker morto. O
    build de produção não era afetado. Correção em `web/src/App.tsx` (worker criado dentro do
    `useEffect`). Também corrigido o corte dos rótulos dos eixos nos gráficos (`canvas.ts`).
67. **Economia negativa no navegador:** no cenário padrão da interface (40 participantes, 100
    mil pagamentos), a economia dos micro-ciclos com janela de 1–2 s sai −0,001%. Coerente com o
    resultado central, mas vale o revisor decidir se o artigo menciona que micro-ciclos podem
    aumentar levemente o pico.
68. **Documentação bilíngue (D17).** README, `docs/`, `DECISIONS`, `REVIEW`, `results/SUMMARY` e
    a interface web existem em português e inglês. A tradução foi feita à mão a partir dos
    originais em português; o revisor deve conferir os termos técnicos (ex.: "compensação" =
    netting, "liquidação bruta em tempo real" = real-time gross settlement, "Conta PI" = Pix
    settlement account).
69. **Enquadramento internacional (`docs/INSTANT_PAYMENTS.pt-BR.md`).** O README, a interface e a
    documentação agora apresentam o LoopSettle como estudo de pagamentos instantâneos com
    liquidação bruta, com o Pix como caso calibrado, e relacionam 11 sistemas aos motores. SPEI,
    Interac e-Transfer e o RTR do Canadá foram conferidos em fontes primárias (Banxico, Interac,
    Bank of Canada); o FAST de Singapura também foi
    conferido em fontes do MAS, e nenhuma linha ficou "a conferir". A linha do SPEI importa para o artigo: o
    Banxico descreve um algoritmo de compensação que liquida os conjuntos possíveis a cada 1,9 s
    em média, um análogo real dos micro-ciclos atômicos e uma comparação natural; o RTR do Canadá
    (pré-financiado, em lançamento no 4º trimestre de 2026) é um caso vivo da pergunta de migração
    do estudo finlandês. Pix é marca de alto renome do BCB (INPI, junho de 2026);
    o README e a interface dizem que o LoopSettle é independente, sem endosso do BCB.
70. **Experimento de robustez e veredito (05/11/2026).** `results/robustness/` e
    [`docs/ASSESSMENT.pt-BR.md`](docs/ASSESSMENT.pt-BR.md): a economia dos micro-ciclos (≈ 0,01%)
    sobrevive a sementes, σ, escala e comprimento do ciclo; o "potencial" de fim de dia encolhe
    com 1/√(volume), e por isso a antiga manchete de "630×" saiu. Veredito PIVOTAR; o que falta
    está especificado em [`docs/PAPER_READINESS.pt-BR.md`](docs/PAPER_READINESS.pt-BR.md), com os
    critérios de morte K1–K4.
71. **Limite de atraso.** Novo módulo `crates/loopsettle-sim/src/bound.rs` e METHODOLOGY §9; o
    revisor deve conferir a demonstração e o teste `engines_never_beat_the_delay_bound`.
72. **Textos integrais lidos (05/11/2026).** BoFER 7/2021, BCB WP 619, Johnson et al. (2004) e o
    artigo da CDFD: os itens 4, 6 e 64 estão resolvidos (definições e números confirmados, ver
    `docs/REFERENCES.pt-BR.md` e a seção de atualização de `docs/ASSESSMENT.pt-BR.md`). Novo: os
    quartis por banco do WP 619 são um alvo com dado real para σ (p25 do modelo 63% contra 18–45%
    no Pix) — frente W2b.
73. **W2 e W2b feitas.** σ calibrado no WP 619 (1,0; faixa 0,8–1,2, D19); perfis intradiários por
    classe (D18). O critério K1 não disparou (limite de 10 s ≤ 0,46% do UB). Com o σ calibrado, o
    potencial de fim de dia com 30 milhões fica entre 0,24% e 2,8%, conforme a heterogeneidade
    intradiária — a incógnita que resta, mensurável só com dados intradiários por participante
    (uma questão de parceria com o BCB). O revisor deve conferir os formatos dos perfis em
    `scenario.rs` (`CLASS_OUT_PROFILE`, `CLASS_IN_PROFILE`).
74. **Comandos para dados reais (W9.1).** `loopsettle bound/settle/export`; o revisor deve testar
    `bound data/samples/payments_example.csv` e conferir as regras de formato em `csvfile.rs`.
75. **Janelas de abastecimento (W3, K3 não disparou).** O fim de semana é a janela que restringe
    (≈ 2,7× o pico de um dia); compensar no fim dele economiza 0,4–1,0%. Em aberto: feriados e o
    comportamento real de aportes.
76. **McMahon et al. (2024)** é o trabalho mais próximo (alto valor, 14–30% de economia com
    reordenação < 5 s). O `regimes` mostra que o limite é 13,7% num cenário de alto valor com poucos
    pagamentos grandes, contra 0,76% no varejo. É preciso ler o artigo completo antes de submeter; a
    base deles (FIFO com fila) não é o nosso UB.
