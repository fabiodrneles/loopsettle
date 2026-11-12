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
    indisponibilidade no horário do pico do participante, ou de um recebedor líquido. → Refeito no item 77.
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
    pagamentos grandes, contra 0,76% no varejo. **Lido em 5/11/2026 (arXiv v3):** o artigo trata do
    LVTS do Canadá com dados reais (lotes de 70 pagamentos, ≈ 90 s, C$ 240 milhões por dia
    economizados contra FIFO, ≈ 1–2% da liquidez alocada); a medida dele (soma dos mNDP) é igual
    ao nosso UB. Os 14–30% vêm da palestra separada de 2024 sobre o T2, cuja base (FIFO com fila)
    não é o nosso UB. O revisor deve conferir a linha em `docs/REFERENCES.pt-BR.md` §7.
77. **Estresse refeito (REVIEW 39, W6).** `results/stress/cases.csv`: 3 alvos (maior, maior
    pagador líquido, maior recebedor líquido) × 3 horários (10h, 18h, 30 min antes do pico
    devedor do próprio alvo). Efeito sobre o UB dos outros ≤ 0,063%; valor rejeitado com saldo
    inicial = UB de base ≤ 0,08%. Uma queda de 30 minutos com reenvio total quase não muda a
    liquidez necessária, porque ela é definida pela deriva acumulada do dia, não pelo horário
    dentro do dia — o mesmo motivo de os micro-ciclos economizarem pouco. Não modelado: abandono
    parcial, quedas acima de 30 min.
78. **PDFs de referência no repositório.** `docs/references/pdf/` guarda 19 dos 20 documentos citados
    (o preprint da CDFD só tem link: a licença do arXiv não permite redistribuir), com índice e
    `references.bib`. Cada um é redistribuído como publicado pelo autor ou
    instituição, com o link original; o revisor deve confirmar que a licença de cada um permite
    isso antes de divulgar o repositório (a remoção a pedido está declarada).
79. **Piso de liquidez (limite de atraso apertado), 11/11/2026.** `bound.rs::liquidity_floors`,
    METHODOLOGY §9b, CLAIMS 1g, D23. Para 10 s o limite cai de 0,76% para 0,054% do UB e a
    compensação a cada 10 s obtém 51% dele (`results/regimes/bound_curve.csv`). Com 10 sementes
    (`robustness/seeds_summary.csv`): de 0,014% (σ = 1,0) a 0,138% (σ = 0). **Riscos:** as rodadas
    de heterogeneidade e de janelas de abastecimento ainda usam a forma de janela (spec 02); o teorema é simples o
    bastante para já existir na literatura (spec 01, pergunta em aberto 1) — conferir antes de
    afirmar novidade; o piso é por participante e a soma não é atingível (spec 03).
80. **Afirmação 1f revista.** Com o limite apertado, o sistema de alto valor estilizado também dá
    ≈ 0 (≤ 0,004%), então o limite de janela não reconcilia o LoopSettle com McMahon et al.; a
    base de comparação (fila FIFO com liquidez restrita) reconcilia, e ela ainda não é simulada
    (spec 06). A "Atualização 3" do ASSESSMENT, o README, o NOVELTY, o PAPER_OUTLINE e o rascunho
    do JOSS ainda trazem o argumento antigo (spec 02, FR6). **Corrigir antes de submeter
    qualquer texto.**
81. **Auditoria de 11/11/2026 e o conjunto de specs.** `specs/00-auditoria.md` lista o que foi
    lido, o que não foi e 14 achados com rótulo de evidência. Defeitos lidos e ainda sem teste:
    `retry_gross` libera um pagamento por aresta por janela (só afeta resultados com liquidez
    restrita), `fraction_at_most(2800)` conta atrasos até 2.815 ms, `DeferredNet` trunca o
    intervalo quando o número de ciclos não divide o dia (nenhum resultado publicado afetado). A
    reprodutibilidade bit a bit entre plataformas não foi verificada (spec 09 E3).
82. **Correções nos motores e nas métricas (spec 04), 11/11/2026.** Testes em
    `crates/loopsettle-core/tests/spec04.rs`, cada um falhava antes da correção. (a) `retry_gross`
    liberava um pagamento por aresta por janela e não revisitava pagadores que recebiam saldo
    depois, na mesma passada; agora vai até o ponto fixo. (b) O ciclo `k` do `DeferredNet` fica em
    `floor(k·dia/c)`, então o último ciclo do dia cai exatamente no fim dele. (c) A parcela
    liquidada em até 2,8 s é contada de forma exata (o histograma contava atrasos até 2.815 ms).
    (d) FIFO estrito por pagador como opção. (e) Somas com checagem no ledger. **Efeito nos
    resultados:** com liquidez ilimitada, todos os CSVs ficam idênticos, exceto a coluna
    `share_within_2800ms` (mudanças abaixo de 0,001, por exemplo 0,4252 → 0,4245; a afirmação 7
    não é afetada); o `microcycle-sweep/shortfall.csv` mudou nas linhas `micro_cycle_shortfall`
    (valor rejeitado com 25% do UB de 23,2275% para 23,2270%; atraso p95 de 920 para 1.216 ms). O
    defeito era real, mas o efeito nos números publicados é desprezível. **Não feito:** a busca
    incremental de ciclos (C5). Custo medido do motor de micro-ciclos: cerca de 0,6 s por milhão
    de pagamentos (10 milhões: 26,1 s contra 19,8 s do bruto, com a leitura do CSV incluída), então
    não é o gargalo; a reescrita precisa manter exatamente a ordem dos ciclos e fica para quando
    as rodadas em escala precisarem.
83. **Reprodutibilidade, estatística e CI (spec 09), 11/11/2026.** (a) `results-golden/` e
    `scripts/golden.py`: uma rodada pequena de cada experimento, comparada byte a byte no CI em
    Linux, Windows e macOS, e com 1 e 2 threads. Primeira rodada: idêntica nas três plataformas
    ([run 43061345632](https://github.com/fabiodrneles/loopsettle/actions/runs/43061345632)); a
    dúvida da auditoria sobre plataformas (A11) fica resolvida para os cenários-ouro. Foi
    confirmado que a checagem falha quando um arquivo muda. (b) O CI também roda o clippy com
    `-D warnings`, o build e o teste de fumaça do wasm e a checagem de documentos bilíngues
    (`scripts/check_twins.py`). (c) `crates/loopsettle-sim/tests/metamorphic.rs`: multiplicar
    valores, renomear participantes, deslocar horários, e 2.000 entradas de lixo nos leitores de
    TOML e CSV (nenhum pânico). (d) Sementes e casos rodam em paralelo (`stats::par_map`, 2 threads
    por padrão, `LOOPSETTLE_THREADS`); a ordem da saída é fixa e os resultados são idênticos com
    1, 2 e 4 threads; o `robustness` caiu de 118 s para 75 s em 4 núcleos. (e) O `summary.json`
    registra versão do esquema, plataforma, rustc e um hash do `Cargo.lock`. **Não feito:**
    `cargo fmt --check` (o código não está no formato do rustfmt: 295 diferenças com a largura
    padrão; reformatar é um commit separado e ruidoso, a decidir pelo dono); várias sementes em
    `baseline`, `potential-vs-realizable`, `microcycle-sweep`, `stress`, `heterogeneity`,
    `funding-windows` e `regimes` (a função de apoio existe, falta dar a cada tabela as colunas
    de IC); o simulador ingênuo de referência para testes diferenciais; `results/MANIFEST.sha256`.
84. **Liquidez restrita (spec 06) e o motor de fila (spec 05, M1), 11/11/2026.**
    `experiments6.rs`, `results/liquidity-frontier/`, CLAIMS 1h, D24, METHODOLOGY §11. Resultado
    com serviço igual (1% do valor rejeitado, regra `ub`, 10 sementes): os desenhos de 10 s
    economizam 0,01% de liquidez (IC incluem zero) em todos os cenários; os de uma hora economizam
    0,4–0,6% (varejo, σ = 1,0), 1,1–2,3% (varejo, σ = 0,4) e 2,8–4,8% (alto valor estilizado).
    **Riscos:** (a) o `α` equivalente perto de 1 é interpolado entre 0,95 e 1,0, onde a rejeição
    do bruto cai de cerca de 4% para exatamente 0: uma grade mais fina ali deixaria o alvo de 1%
    mais preciso; (b) a regra `ub` dá a cada participante exatamente o próprio pico, então α = 1 é
    um fio de navalha; a regra `outflow` está no `frontier.csv`, mas não na tabela de liquidez
    equivalente do resumo (poucas sementes atingem alvos baixos com α ≤ 2); (c) sem novas
    tentativas; (d) o sistema de alto valor não é calibrado, então os 2,8–4,8% são uma direção, não
    uma estimativa, e ficam longe dos 14–30% de McMahon et al.; (e) uma hipótese minha errada foi
    pega por um teste: uma fila de 10 s *não* rejeita menos que o bruto (rejeita 0,08% a mais em
    valor no cenário pequeno de teste), que é o mesmo mecanismo do resultado principal.
    **Desempenho:** a nova tentativa bruta foi reescrita para visitar só os participantes que
    receberam crédito (levava mais de 10 minutos com prazos de uma hora; agora o experimento
    inteiro leva 4 min 46 s); a versão literal fica como `reference_retry` e as duas dão
    liquidações idênticas em 30.000 casos aleatórios. Isso substitui o item C5 da spec 04 para a
    nova tentativa; a busca de ciclos em si não mudou.
85. **Ótimo offline e o sanduíche (spec 03), 11/11/2026.** `tests/common/optimum.rs`,
    `tests/optimum.rs`, exemplo `optimality_gap`, CLAIMS 1i, D25, METHODOLOGY §9c. O ótimo da PL é
    igual à soma dos pisos em 116 de 120 instâncias pequenas e densas; o melhor motor obtém 44–61%
    dele. **Riscos:** (a) a PL permite fracionar pagamentos e lotes arbitrários, então limita por
    baixo o ótimo com pagamentos inteiros; a checagem inteira por força bruta da spec (FR2) não foi
    escrita; (b) as instâncias são pequenas (20 participantes, 1.500 pagamentos) e não calibradas;
    um prazo de 300 s não terminou em 9 minutos, então a afirmação cobre prazos de até 60 s; (c)
    "pisos = ótimo" é uma regularidade empírica, não um teorema: um teste aleatório acha folga
    estrita em 4% de instâncias minúsculas. Uma prova de quando vale a igualdade seria uma
    contribuição; (d) `results/optimality-gap/` não entra na checagem-ouro (soluções de PL em
    ponto flutuante não têm garantia de serem idênticas entre plataformas; os arquivos guardam de
    2 a 4 casas).
86. **Fronteira de liquidez refeita com grade mais fina, 12/11/2026.** O risco (a) do ponto 84 era
    real: com passos de 0,01 entre 0,97 e 1, as economias de uma hora com 1% rejeitado são 1,1–1,4%
    (varejo, σ = 1,0), 2,2–3,2% (σ = 0,4) e 3,7–5,3% (alto valor), até o dobro dos valores da
    primeira rodada. O resultado de 10 s não mudou (IC incluem zero). Os números do ponto 84 estão
    superados; a CLAIMS 1h traz os novos e diz que a medida é sensível à grade. Custo: o experimento
    agora leva 5 min 49 s em 4 núcleos com 2 threads, acima da meta de 5 minutos da spec 06 (na
    máquina de referência será mais). Também novo: `loopsettle bound/settle --report` e saída por
    participante privada (spec 10, R2 e R3; D21 atualizada).
87. **Pagamentos em eco (spec 07, G2): o critério de morte disparou, 12/11/2026.** `stream.rs`
    (mecanismo de eco), `experiments7.rs`, `results/echo-sweep/`, CLAIMS 1j e "não pode afirmar" 14,
    D26, ASSESSMENT atualização 5. Com ecos devolvidos a quem pagou, parcela de 20% e atraso de
    1 s: limite apertado de 10 s de 4,2% do UB, obtido 3,9%. Com ecos adiante: nada muda. **Este é
    o ponto mais importante para o revisor:** toda frase do tipo "segundos não recuperam nada" no
    README, no rascunho do JOSS e no esqueleto do artigo precisa levar a condição. **Riscos:** (a)
    os ecos repetem o valor exato, o caso mais favorável à compensação; valores parciais dariam
    menos; (b) um só σ (0,4) e uma só escala (1 milhão); com mais volume a parte estacionária
    encolhe, mas a parte do eco não deveria; (c) IC largos com 8 sementes (em alguns casos poucas
    sementes dominam); (d) os micro-ciclos precisaram de até 4,5% *mais* liquidez que o bruto com
    50% de eco adiante a 1 s (IC [−7,5; −1,5]): segurar pagamentos que seriam financiados pelo
    próprio gatilho custa caro, e isso merece um olhar próprio; (e) não se sabe se os fluxos
    reais têm devoluções bilaterais rápidas.
88. **Continuação do teste de eco, 12/11/2026.** `echo-sweep/sensitivity.csv` e
    `order_diagnostic.csv`, CLAIMS 1j (ampliada) e 1k. (a) O resultado condicional vale na direção
    em todas as variações testadas, com tamanho menor: limite apertado de 10 s de 1,7% (metade do
    valor), 0,6% (um quarto), 1,8% (σ = 1,0), 2,1% (três vezes mais pagamentos), contra 4,2% na
    varredura. A queda com o volume não era esperada e tem IC sobrepostos ([0,8; 3,4] contra
    [2,2; 6,2]); uma rodada com 10 e 30 milhões resolveria se ela é real. (b) O risco (d) do ponto
    87 tem explicação: o motor de espera (só atraso, ordem mantida) economiza exatamente 0,000% em
    todos os casos, então a perda dos micro-ciclos com eco adiante vem de liquidar fora da ordem de
    chegada. "Exatamente zero" é plausível (deslocar toda liquidação pelo mesmo atraso desloca
    cada curva de posição sem mudar a profundidade, a menos de efeitos de borda), mas é uma
    observação, ainda não uma afirmação demonstrada.
