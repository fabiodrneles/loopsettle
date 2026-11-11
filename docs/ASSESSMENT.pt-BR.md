# AVALIAÇÃO — o LoopSettle tem um resultado?

> [English version](ASSESSMENT.md)
>
> Resposta ao pedido de revisão de 05/11/2026 ("o LoopSettle tem um resultado ou é perda de
> tempo?"). Cada afirmação cita um arquivo e um número. A evidência nova vem do experimento
> `robustness` (`results/robustness/`, seção 7 de
> [`results/SUMMARY.pt-BR.md`](../results/SUMMARY.pt-BR.md)) e do limite de atraso demonstrado em
> [`METHODOLOGY.pt-BR.md`](METHODOLOGY.pt-BR.md#9-o-limite-de-atraso) e testado em
> `crates/loopsettle-sim/src/bound.rs`.

## Veredito: **PIVOTAR**

A ferramenta é boa e um resultado é sólido, mas a manchete como está escrita hoje ("potencial
de 9,27% contra realizável de 0,0147%, cerca de 630× menor") **não sobrevive à escala**: os
9,27% encolhem com a raiz quadrada do volume, e a razão não é um número estável. O que sobrevive
é mais estreito e, achamos, mais útil:

> **Nova afirmação central.** Num sistema de pagamentos instantâneos, qualquer mecanismo de
> liquidação que atrase cada pagamento em no máximo *d* consegue economizar, no máximo, o valor
> que cada participante paga nos *d* segundos antes do seu pico de liquidez. Com prazo de
> 10 s, esse teto vai de 0,06% a 3,3% da liquidez necessária nas configurações testadas
> (corrigido em 05/11/2026; a versão anterior dizia "abaixo de 1%"), e a economia obtida é de cerca de 0,01% — independentemente do algoritmo de busca de
> ciclos, da semente e da escala. As medidas de "compensação potencial" (85% do volume, 9% da
> liquidez) quase não dizem nada sobre o que um mecanismo com prazo consegue recuperar; se
> horizontes mais longos recuperam mais depende de desequilíbrios intradiários *sistemáticos*,
> que os agregados públicos não identificam.

Próximo passo: os quatro experimentos de [`PAPER_READINESS.pt-BR.md`](PAPER_READINESS.pt-BR.md),
depois um **preprint** (SSRN / arXiv q-fin) e um artigo no **JOSS** sobre o software. O JFMI só
fica realista com dados intradiários reais ou um modelo validado de heterogeneidade intradiária
(frente W2).

## A. O resultado central se sustenta?

| Ameaça | Evidência | Veredito |
|---|---|---|
| **1. Escala** (239× menos pagamentos por janela) | `robustness/scale.csv`: micro-ciclos de até 10 s economizam 0,0147% com 1 milhão de pagamentos por dia, 0,0027% com 3 milhões, −0,0005% com 10 milhões e −0,0016% com 30 milhões (fator de escala 8). O limite para 10 s cai de 0,76% para 0,27% e depois estabiliza. | **Sobrevive** para os micro-ciclos: mais volume não cria liquidez recuperável, a economia vai a zero. **Não sobrevive** para o "potencial": a compensação de fim de dia cai de 9,27% para 1,68% (expoente −0,50, ou seja, ∝ 1/√N). Extrapolada para o volume real do Pix (fator 1), seria cerca de 0,6% — neste modelo. |
| **2. Viés recebedor σ** | `robustness/seeds_summary.csv`: economia média dos micro-ciclos (10 s) de 0,0178% (σ=0), 0,0116% (σ=0,4) e 0,0033% (σ=0,8); o limite para 10 s é 3,3%, 0,80% e 0,70%. LB/UB vai de 0,70 a 0,97. | **Sobrevive** em σ ∈ [0; 0,8]: médias ≤ 0,018% do UB; a pior rodada das 30 deu 0,103% (σ=0, semente 8, `robustness/seeds.csv`). Fora dessa faixa não foi testado (σ < 0 não faz sentido; σ > 0,8 leva LB/UB → 1 e reduz qualquer economia). |
| **3. Ruído perto de zero** | Mesmo arquivo, 10 sementes: σ=0,4, micro 10 s, 0,0116%, IC 95% [0,0079; 0,0152]%; as 10 sementes positivas. O padrão não monótono da tabela antiga, de uma semente só (0,048 / 0,005 / 0,012 / 0,003), era ruído: com 10 sementes ele é monótono (0,018 > 0,012 > 0,003). | **Resolvido**: o valor é pequeno *e* medido com precisão. Números de uma semente só não devem mais ser publicados; a economia de fim de dia tem IC largo (σ=0,4: 7,8%, IC [4,5; 11,1]%). |
| **4. Modelo de UB** (sem saldo de abertura, um dia) | Não testado. O limite mostra *por que* o resultado sai assim: o pico vem de deriva que dura horas. Mas o modelo tem um único perfil horário para todos e pesos bilaterais estacionários (D8, D9), então não consegue gerar desequilíbrios intradiários sistemáticos (folha de pagamento, liquidação de credenciadoras, padrões de funding de fintechs) — justamente o que uma compensação em horizonte mais longo recuperaria. Também não modela a janela de funcionamento do STR nem o pré-financiamento noturno da Conta PI. | **Aberta — a maior ameaça que resta**, mas ela afeta o lado do *potencial*, não o dos micro-ciclos: desequilíbrios sistemáticos duram de minutos a horas, então o limite de 10 s continua valendo. |

Objeção do algoritmo (REVIEW 13): `robustness/cycle_len.csv` — economia de 0,0155% com ciclos de
tamanho 2 (só pares bilaterais), 0,0100% (3), 0,0147% (6) e 0,0145% (10). A heurística não é o
gargalo, e o limite vale para *qualquer* algoritmo, inclusive um otimizador perfeito.

## B. É novo?

O que, até onde sabemos (a confirmar na revisão de literatura W4), ninguém tinha mostrado:

1. **Um teto que não depende do modelo** para a economia de qualquer mecanismo com prazo
   máximo (o limite de atraso), conferido participante a participante contra os motores
   simulados.
2. **Prazos de segundos em pagamentos instantâneos de varejo**: os estudos clássicos de
   mecanismos de economia de liquidez usam sistemas de alto valor e aceitam atrasos de minutos a
   horas — Johnson, McAndrews & Soramäki (2004) simulam compensação de 1 e 6 horas e liquidação
   reativa a recebimentos no Fedwire; Denbee & McLafferty (2012) acham economia acima de 30% no
   CHAPS com atrasos de minutos a mais de duas horas; Hellqvist & Korpinen (2021/2026) comparam
   ciclos fixos com instantâneo. Nenhum impõe o prazo de 2,8–10 s de um esquema instantâneo.
3. **O colapso da "compensação potencial" sob fluxos estacionários** (∝ 1/√N): um alerta contra
   ler medidas de capacidade potencial como liquidez recuperável.
4. **Um simulador aberto e reprodutível** de liquidação de pagamentos instantâneos, com
   invariantes testadas por propriedades, que roda num notebook e no navegador.

O que **não** é novo: a troca entre liquidez e atraso em si (por exemplo, Bank of Canada WP
06-20 sobre o LVTS), a economia do desenho de LSMs (Martin & McAndrews 2008) e a ideia de que
LSMs funcionam reordenando pagamentos na fila (Bech & Soramäki 2001). A intuição "compensar em
quase tempo real economiza pouco" é plausível para quem trabalha na área; a contribuição é
limitá-la e medi-la.

## C. Interessa a alguém?

- **Bancos centrais que desenham ou ampliam sistemas instantâneos** — Bank of Canada (RTR,
  pré-financiado, em lançamento no 4º trimestre de 2026), Federal Reserve (gestão de liquidez do
  FedNow), Eurosistema (TIPS), BCB (Pix). O limite é um teste rápido que eles podem rodar nos
  próprios dados intradiários: *se o fluxo perto do pico de cada participante é pequeno, não vale
  construir um LSM de segundos; olhe para as janelas de pré-financiamento e para o crédito
  intradiário.*
- **Pesquisadores de sistemas de pagamento** (comunidade do BoF-PSS, CPMI): uma ferramenta aberta
  e um benchmark reprodutível.
- **Tesourarias de bancos** que dimensionam o saldo da Conta PI: a heterogeneidade por
  participante (`robustness/participants_ratio.csv`: entre os participantes que precisam de
  liquidez nos dois desenhos, a razão instantâneo/diário tem mediana 1,16×, p90 2,90× e máximo
  19×; 104 dos 200 só precisam de liquidez no instantâneo) reproduz o achado principal do estudo
  finlandês.

Mudaria alguma decisão? Possivelmente uma: investir ou não num mecanismo de economia de
liquidez de segundos num sistema instantâneo. É uma decisão real, mas estreita.

## D. O que está errado ou fraco (do mais grave ao menos grave)

1. **A manchete de 9,27% / "630×" é artefato de escala** (ameaça 1 acima). Ela precisa sair do
   README, do CLAIMS e do esqueleto do artigo (feito nesta mudança).
2. **Chegadas estacionárias**: um perfil horário para todos e pesos bilaterais constantes (D8,
   D9). É isso que faz o potencial cair com 1/√N; a heterogeneidade intradiária real não cairia.
   Precisa da frente W2.
3. **Definição do UB**: um dia, saldo de abertura zero, sem janela do STR, sem pré-financiamento
   noturno ou de fim de semana da Conta PI, sem crédito intradiário (LIMITATIONS 6). O artigo
   precisa definir "liquidez necessária" como as regras do BCB definem (frente W3; documentos na
   especificação).
4. **σ não é identificado** por nenhum dado público: LB/UB vai de 0,70 a 0,97 com ele
   (`seeds_summary.csv`). Todos os resultados devem ser reportados como faixas sobre σ.
5. **O degrau de Hodge não informa nada neste grafo**: para fluxos aleatórios num grafo conexo
   com m arestas e n nós, a fração esperada da norma ao quadrado fora do espaço gradiente é
   (m − n + 1)/m = (6620 − 200 + 1)/6620 = 0,970; a fração rotacional L2 observada é 0,977
   (`baseline/ladder.csv`). Aqui Hodge mede a densidade do grafo, não a circularidade. Tirar da
   afirmação central, ou manter como nota de cautela metodológica com esse valor nulo.
6. **O limite de atraso é folgado**: a economia obtida é cerca de 1,5% do limite (σ=0,4:
   0,0116% contra 0,80%). É um teto, não uma estimativa; dizer isso.
7. **Citações não conferidas** em duas comparações: o 8,6% do estudo finlandês e a definição
   exata da medida de compensação do BCB WP 619 (`docs/REFERENCES.pt-BR.md`). Descrever errado o
   WP 619 seria o erro mais constrangedor que um revisor poderia achar.
8. **Menores**: cenário de estresse fraco (REVIEW 39); `QueueOnShortfall` sem FIFO estrito (D13);
   em escala grande, os micro-ciclos podem *aumentar* um pouco o pico (−0,0016% com 30 milhões),
   o que o artigo deve mencionar.

## E. Valor independente da tese

Sim. Como software, atende ao perfil do JOSS: Apache-2.0, repositório público, oito invariantes
testadas por propriedades mais o novo teste do limite de atraso, experimentos reprodutíveis
byte a byte, uso de memória de 5 MiB, interface no navegador e documentação bilíngues. Como
portfólio, mostra uma combinação rara de engenharia e domínio. Mesmo que o artigo não avance, um
artigo no JOSS é viável por si só.

## Critérios de morte — parar (ou rebaixar para um artigo só de software) se:

| # | Experimento | Parar se |
|---|---|---|
| K1 | W2: perfis intradiários heterogêneos (padrões horários próprios de cada participante, calibrados com algum dado intradiário público) | o limite de 10 s passar de **2% do UB** com 30 milhões de pagamentos por dia — aí mecanismos de segundos poderiam importar e a afirmação "desprezível" é falsa |
| K2 | W4: revisão de literatura (CPMI, ECB/TIPS, Bank of Canada/RTR, Fed/FedNow, artigos com o BoF-PSS desde 2018) | um estudo publicado já reportar a economia de LSMs com **prazos de segundos num sistema instantâneo**, ou já enunciar o limite de atraso — aí o artigo é uma replicação; ficar só com o JOSS |
| K3 | W3: regras do BCB sobre funding da Conta PI, horário do STR e crédito intradiário | ficar demonstrado que a restrição relevante é algo que o modelo não representa (por exemplo, só o pré-financiamento noturno/de fim de semana importa e os picos intradiários são sempre cobertos com Reservas) — aí a pergunta intradiária perde sentido; pivotar para a pergunta da janela de pré-financiamento ou parar o artigo |
| K4 | W1: rodar de novo com todas as correções | o IC 95% da economia de 10 s passar de **0,1% do UB** em algum ponto da faixa σ ∈ [0; 0,8] — aí "cerca de 0,01%" está errado e precisa ser refeito |

## Fontes

As mesmas da [versão em inglês](ASSESSMENT.md#sources).

## Atualização — textos integrais lidos (05/11/2026, no mesmo dia)

O dono enviou quatro textos integrais. O que eles resolvem:

| Ponto | Achado | Efeito na avaliação |
|---|---|---|
| Número finlandês (D.7) | "8,6%" é o **percentil 95** do aumento contra **6 ciclos**; a média é 2,7%. Contra um ciclo diário, o instantâneo precisa de 1,45× (BoFER 7/2021 p. 2, 9–11). LoopSettle: +3,9% (σ=0,4) a +13,7% (σ=0) contra 6 ciclos; 1,10×–1,50× contra 1 ciclo. | resolvido; mesma ordem de grandeza, agora na base certa |
| Definição de liquidez (D3, parte da ameaça 4) | A liquidez necessária deles é exatamente o UB/LB do LoopSettle (p. 3–4). | a definição é padrão; a ameaça 4 passa a tratar só das janelas de abastecimento (W3), não da medida em si |
| Medida do BCB WP 619 (D.7) | Eq. (3.2), por banco, apresentada em quartis: mediana 57–80%, p25 18–45%, p75 73–90% com dados reais do Pix (p. 16 do PDF). O LoopSettle na mesma estatística: mediana 78%, p75 91%, mas **p25 63%**. | resolvido; e dá o **primeiro alvo com dado real para σ**: o modelo tem poucos bancos "de mão única". Calibrar σ (ou um viés por classe) no p25 do WP 619 torna a fraqueza D.4 em parte identificável. Novo item W2b. |
| Novidade do limite de atraso (K2) | Johnson, McAndrews & Soramäki (2004) não enunciam limite; a compensação deles em horas economiza de 2–6% (1 h) a 6–26% (6 h) do pico de descoberto no Fedwire (Tabela 3). | K2 não dispara no trabalho mais próximo; a revisão W4 continua para os trabalhos depois de 2018 |
| CDFD | A circularidade máxima é exatamente o problema de custo mínimo com custo unitário que o LoopSettle resolve; o BFF está totalmente especificado (Algoritmo 1) (arXiv 2506.12546, §5.1–5.2). | D5 confirmada; a D6 já pode ser implementada |

O veredito continua **PIVOTAR**; nenhum critério de morte disparou.

## Atualização 2 — W2 e W2b (05/11/2026)

- **Viés recebedor calibrado (W2b).** σ = 1,0 põe os quartis da razão de pagamentos
  compensáveis por banco do modelo (36% / 61% / 83%) dentro das faixas do WP 619 com dados reais;
  σ = 0,4 não punha (p25 64%) (`results/bias-calibration/grid.csv`). A fraqueza D.4 passa a ser
  em parte identificada por dado real.
- **Critério de morte K1 não disparou (W2).** Com perfis intradiários por classe, o limite de
  atraso de 10 s com 30 milhões de pagamentos por dia é no máximo 0,46% do UB (σ = 0,4) e 0,08%
  (σ = 1,0), muito abaixo de 2% (`results/heterogeneity/`).
- **Com heterogeneidade, o potencial deixa de desabar.** Com o σ calibrado, a compensação de fim
  de dia com 30 milhões economiza 0,24% do UB sem heterogeneidade intradiária e 2,8% com perfis por
  classe completos; compensar a cada 15 minutos, 0,05–0,09%; micro-ciclos ≈ 0.
- **Afirmação central revista:** em escala realista e com a estrutura bilateral calibrada, a
  compensação recupera entre ~0,2% e ~3% da liquidez que um sistema instantâneo prende, e *só* em
  horizontes de várias horas; mecanismos de segundos não recuperam nada mensurável, e nenhum
  deles pode recuperar mais que o limite de atraso. Qual ponta da faixa 0,2–3% é a verdadeira
  depende dos padrões intradiários por participante — o único insumo que exige dados de um banco
  central.

O veredito continua **PIVOTAR**, com a afirmação acima. Nenhum critério de morte disparou.

## Atualização 3 — ferramenta para dados reais, janelas de abastecimento, regimes (05/11/2026)

- **K3 não disparou (W3).** Os documentos do BCB mostram que a Conta PI só recebe aportes com o STR
  aberto, então a restrição relevante é o pré-financiamento das noites e dos fins de semana. No
  `funding-windows`, o fim de semana precisa de ≈ 2,7× o maior esvaziamento de um dia, e uma
  compensação no fim do fim de semana economizaria só 0,4–1,0% disso (σ calibrado); o limite de 10 s
  dentro dele é ≤ 0,23%. O resultado central vale, portanto, na janela que de fato restringe.
- **K2 não disparou, mas o trabalho mais próximo mudou.** McMahon et al. (2024) relatam 14–30% de
  economia com reordenação abaixo de 5 s no T2. O `regimes` mostra por que isso é coerente com o
  LoopSettle: o limite de atraso cresce com o tamanho dos pagamentos (limite de 10 s de 0,76% no
  varejo contra 13,7% com 2.000 pagamentos grandes por dia). O artigo (arXiv v3, lido em 5/11/2026)
  trata do LVTS do Canadá com dados reais: lotes de 70 pagamentos (≈ 90 s) economizam C$ 240
  milhões por dia contra FIFO, cerca de 1–2% da liquidez alocada — a mesma ordem do limite do
  LoopSettle no varejo com 60 s (2,7% do UB), com a mesma medida (mNDP = UB).
- **W5 feita.** Para d de 10 s a um dia, a compensação obtida fica em 3–5% do limite
  (`regimes/bound_curve.csv`): o limite é folgado, mas por um fator estável.
- **W9.1 feita.** `loopsettle bound/settle` rodam em arquivos de pagamentos reais, com saída só de
  agregados.

### Já dá para começar a escrever?

**Sim — o preprint e o artigo do JOSS podem ser rascunhados agora.** Todas as frentes obrigatórias
estão feitas ou reduzidas a conferências que não mudam os resultados. O artigo de McMahon et al. foi
lido (W4) e o experimento de estresse foi refeito (W6, `results/stress/cases.csv`); faltam os itens do seu lado (notebook de referência,
submissão). O esqueleto revisto está em [`PAPER_OUTLINE.pt-BR.md`](PAPER_OUTLINE.pt-BR.md). O que deve
ficar fora do texto até existirem dados reais por participante: qualquer número de economia potencial
no Pix real além da faixa de 0,2–3%, e qualquer afirmação sobre sistemas de outros países.

## Atualização 4 — o piso de liquidez (11/11/2026)

Este arquivo é um registro datado; o texto acima fica como foi escrito. O que mudou desde então:

- **A fraqueza D.6 ("o limite de atraso é folgado") está resolvida.** O piso de liquidez
  ([`METHODOLOGY.pt-BR.md`](METHODOLOGY.pt-BR.md) §9b) é uma forma mais apertada do mesmo limite.
  Para 10 s, em 10 sementes, o teto da economia vai de 0,014% do UB (σ = 1,0) a 0,138% (σ = 0), em
  vez de 0,7–3,3%, e a compensação a cada 10 s obtém 38–46% dele
  (`results/robustness/seeds_summary.csv`). Leia todo "0,06–3,3%" acima como a forma de janela.
- **A reconciliação com McMahon et al. da Atualização 3 está retirada.** Ela se apoiava em o
  limite de janela ser grande (13,7%) no sistema de alto valor estilizado. O limite apertado ali é
  ≤ 0,004% (`results/regimes/regimes.csv`): contra a liquidação bruta em tempo real com liquidez
  ilimitada, mecanismos de segundos economizam ≈ 0 nos dois regimes. Os 14–30% deles são medidos
  contra uma fila FIFO com liquidez restrita. O LoopSettle ainda não simula essa base
  (`specs/06`), então a afirmação honesta hoje é "pergunta diferente, ainda não comparada".
- **K1 com o limite apertado:** no máximo 0,042% do UB com 30 milhões de pagamentos por dia e
  perfis por classe (`results/heterogeneity/heterogeneity.csv`), contra o limiar de 2%.
- **O K2 vale também para o teorema novo.** O piso é simples o bastante para já estar na
  literatura (cálculo de redes, troca entre liquidez e atraso em RTGS). Ainda não conferido.
- **Reprodutibilidade:** os resultados são idênticos byte a byte em Linux, Windows e macOS (CI).

O veredito continua **PIVOTAR**, com uma afirmação central mais afiada: *nenhum mecanismo com
prazo de 10 s consegue economizar mais do que cerca de 0,01–0,14% da liquidez que um sistema
instantâneo precisa, e a compensação simples já obtém cerca de 40% desse teto.*
