<p align="center">
  <img src="docs/img/banner.svg" alt="LoopSettle — circular liquidity in instant payment systems" width="100%">
</p>

<p align="center">
  <a href="LICENSE"><img alt="License: Apache-2.0" src="https://img.shields.io/badge/license-Apache--2.0-blue.svg"></a>
  <img alt="Rust" src="https://img.shields.io/badge/core-Rust-b7410e.svg">
  <img alt="WebAssembly" src="https://img.shields.io/badge/browser-WebAssembly-654ff0.svg">
  <img alt="Reproducible" src="https://img.shields.io/badge/results-bit--for--bit%20reproducible-2ea44f.svg">
  <img alt="Property tests" src="https://img.shields.io/badge/invariants-8%20property%20tests-2ea44f.svg">
  <img alt="Data" src="https://img.shields.io/badge/data-public%20%2B%20synthetic-555.svg">
</p>

<p align="center">
  <b>Simulador aberto que mede quanto da liquidez de um sistema de pagamentos instantâneos dá voltas entre as instituições — e quanto disso é recuperável na prática.</b><br>
  <i>An open simulator that measures how much of the liquidity in an instant payment system goes around in loops — and how much of it can actually be recovered.</i>
</p>

<p align="center">
  <a href="#português">Português</a> ·
  <a href="#english">English</a> ·
  <a href="results/SUMMARY.md">Resultados</a> ·
  <a href="docs/METHODOLOGY.md">Metodologia</a> ·
  <a href="docs/LIMITATIONS.md">Limitações</a> ·
  <a href="docs/REFERENCES.md">Referências</a>
</p>

<p align="center">
  <img src="docs/img/ui-overview.png" alt="Interface do LoopSettle: rede de participantes com o componente circular em verde, controles dos micro-ciclos e painéis de liquidez" width="100%">
</p>
<p align="center"><sub>A interface roda inteira no navegador (Rust → WebAssembly, num Web Worker). 40 participantes e 100 mil pagamentos simulados em 0,5 s.</sub></p>

---

## Português

### Em uma frase

O Pix precisa que cada banco tenha dinheiro em caixa antes de cada transferência. Parte desse
dinheiro só "dá voltas" (A paga B, B paga C, C paga A). O LoopSettle mede quanto é essa parte,
quanto dela daria para compensar e, principalmente, **quanto dessa compensação vira liquidez
economizada de verdade** quando se respeita a ordem em que os pagamentos acontecem.

### O problema

O Pix liquida cada pagamento de forma **bruta e em tempo real**: cada instituição precisa ter
saldo na sua Conta PI antes de cada transferência. Nos sistemas antigos, os bancos podiam
acumular pagamentos e compensar entradas com saídas (*netting*), economizando liquidez. O
estudo do Banco Central sobre o Pix (Working Paper 619) mostra que os bancos perderam essa
capacidade de adiar e compensar, e passaram a reter mais ativos líquidos.

Uma pergunta continuava sem resposta: **quanto dessa compensação perdida vira, de fato,
liquidez que poderia ser liberada?** E dá para recuperar parte dela sem abrir mão da
experiência instantânea do usuário?

> **Pergunta de pesquisa.** Quanto da liquidez que os participantes do Pix precisam manter é
> circular, quanto disso é recuperável na prática, e qual o custo em atraso de recuperá-la com
> micro-ciclos atômicos?

### O resultado principal

<p align="center">
  <img src="results/potential-vs-realizable/fig_potential_vs_realizable.svg" alt="Economia de liquidez em função do tempo máximo de espera" width="720">
</p>

No cenário sintético calibrado com dados públicos do Pix (200 participantes, 1 milhão de
pagamentos por dia, semente `20260930`):

- **85% do volume** de pagamentos poderia ser "compensado" pela medida potencial (entradas e
  saídas do mesmo dia, sem olhar a ordem no tempo).
- Mas a **economia de liquidez realizável** depende do horizonte da compensação: **9,3%** com
  uma compensação diária, **0,3%** a cada 15 minutos e **menos de 0,03%** com ciclos de
  segundos.
- **Micro-ciclos atômicos de até 10 s liquidam até 63% do valor dentro de ciclos, mas
  economizam menos de 0,02% da liquidez.** A necessidade de liquidez vem de desequilíbrios
  que duram horas (um banco que paga mais do que recebe a manhã inteira), não de ruído de
  segundos.

> **Volume compensável não é liquidez economizada.** O LoopSettle torna visível e mensurável a
> distância entre essas duas medidas.

Todos os números estão em [`results/SUMMARY.md`](results/SUMMARY.md), com a semente, a versão
do código e o comando que os gerou. O que o estudo pode e não pode afirmar está em
[`docs/CLAIMS.md`](docs/CLAIMS.md).

### A interface

<table>
  <tr>
    <td width="58%"><img src="docs/img/ui-network.png" alt="A rede de um dia: participantes em círculo, arestas verdes são a parte circular máxima (CDFD) e cinzas a parte direcional"></td>
    <td width="42%"><img src="docs/img/ui-potential-vs-realizable.png" alt="Painel potencial vs realizável: 82% potencial, 78% CDFD, 75% ciclos, 11% de liquidez com uma compensação diária e cerca de zero com micro-ciclos"></td>
  </tr>
  <tr>
    <td align="center"><sub><b>A rede de um dia.</b> Tamanho do círculo = atividade; tracejado = pagador líquido. Em verde, a maior circulação possível (CDFD); em cinza, o fluxo que só vai numa direção.</sub></td>
    <td align="center"><sub><b>Potencial × realizável.</b> A escada das medidas, da compensação no papel até a liquidez poupada de fato.</sub></td>
  </tr>
  <tr>
    <td><img src="docs/img/ui-day.png" alt="O dia em intervalos de 15 minutos: valor pago por intervalo e soma das posições devedoras acumuladas"></td>
    <td><img src="docs/img/ui-liquidity.png" alt="Liquidez necessária por desenho: bruto, compensação diária, a cada 15 minutos, micro-ciclos e mínimo teórico"></td>
  </tr>
  <tr>
    <td align="center"><sub><b>O dia em 15 minutos.</b> A soma das posições devedoras (linha escura) cresce por horas: é isso que exige liquidez, e micro-ciclos de segundos não alcançam.</sub></td>
    <td align="center"><sub><b>Liquidez necessária por desenho</b>, do bruto em tempo real (Pix) ao mínimo teórico (LB).</sub></td>
  </tr>
</table>

<table>
  <tr>
    <td width="34%"><img src="docs/img/ui-mobile.png" alt="Interface no celular em modo reduzido"></td>
    <td>

**O que dá para fazer na tela**

- Mexer na **janela de busca de ciclos** (0,5 a 10 s) e no **atraso máximo tolerado** (2 a 10 s)
  e ver a economia de liquidez e os atrasos se recalcularem.
- Mudar o número de participantes, de pagamentos e o **desequilíbrio entre recebedores** (a
  principal hipótese do modelo) para testar a robustez do resultado.
- Testar **liquidez restrita**: com saldo inicial menor que o necessário, quanto valor é
  rejeitado em cada desenho e quantos pagamentos cumprem a meta do Pix (50% em até 2,8 s).
- Trocar a **semente**: a mesma semente sempre produz o mesmo resultado.

**No celular** a interface entra em modo reduzido (até 25 participantes e 30 mil pagamentos)
com o mesmo modelo.

Os cenários do navegador são menores que os dos experimentos (até 50 participantes e 100 mil
pagamentos), então os números da tela não são os do artigo. Em cenários pequenos a economia dos
micro-ciclos pode até sair levemente **negativa** (−0,001%): a fila muda a ordem das
liquidações e o pico de um participante pode subir um pouco. É um resultado do modelo, coerente
com o achado principal (economia próxima de zero), e não um erro.

</td>
  </tr>
</table>

```bash
cd web && npm install && npm run wasm && npm run dev    # abre em http://localhost:5173
```

As capturas são regeneradas por comando: `node scripts/screenshots.cjs` (com a interface no ar).

### Como funciona

```mermaid
flowchart LR
  A["Dados abertos do Pix<br/>(API Olinda / BCB)"] --> B["Gerador sintético<br/>calibrado, em streaming"]
  B --> C["Motores de liquidação"]
  C --> C1["Bruto em tempo real<br/>(como o Pix)"]
  C --> C2["Compensação diferida<br/>1× ao dia … a cada 10 s"]
  C --> C3["Micro-ciclos atômicos<br/>(mecanismo novo)"]
  B --> D["Topologia do fluxo"]
  D --> D1["Hodge<br/>(diagnóstico)"]
  D --> D2["CDFD máxima<br/>(teto estático)"]
  D --> D3["Netting por ciclos<br/>(executável)"]
  C1 & C2 & C3 --> E["Liquidez, atrasos e<br/>potencial × realizável"]
  D1 & D2 & D3 --> E
```

**Três lentes para medir o dinheiro que dá voltas**

| Lente | O que mede | Por que importa |
|---|---|---|
| **Decomposição de Hodge** | separa o fluxo em componente direcional (gradiente) e circular | ótimo diagnóstico, mas pode atribuir ao ciclo mais do que existe: no triângulo A→B 10, B→C 5, C→A 5, Hodge dá 6,67 ao ciclo, e só 5 são compensáveis |
| **CDFD máxima** | maior circulação possível respeitando o valor de cada aresta e a conservação em cada nó | teto realista e exato do que poderia ser compensado (fluxo de custo mínimo implementado à mão, conferido contra um solver de programação linear em 1.000 grafos aleatórios) |
| **Netting por ciclos** | o que um algoritmo de compensação estilo Bech–Soramäki consegue cancelar | o que é executável na prática |

**Três desenhos de liquidação, lado a lado**

| Motor | Comportamento |
|---|---|
| `GrossRealTime` | liquida cada pagamento na chegada, se houver saldo (o modelo do Pix) |
| `DeferredNet` | compensação multilateral em ciclos fixos, de 1 por dia a 1 a cada 10 s (estilo UPI e estudo finlandês) |
| `AtomicMicroCycle` | espera alguns segundos, encontra ciclos na fila e liquida cada ciclo de uma vez, tudo ou nada; modos `QueueAll` (todos passam pela fila) e `QueueOnShortfall` (só na falta de saldo, estilo LSM) |

**Como a liquidez é medida**

- **UB (limite superior):** menor saldo inicial que evita qualquer rejeição no bruto em tempo
  real = máximo, ao longo do dia, de saídas acumuladas − entradas acumuladas.
- **LB (limite inferior):** posição líquida multilateral do fim do dia, o que uma única
  compensação diária exigiria.
- **Economia de um desenho** = 1 − liquidez do desenho ÷ UB.

### Resultados em números

| Medida | Valor |
|---|---|
| Erro da calibração frente aos agregados públicos do Pix | ≤ 1,5% em todos os 9 agregados |
| Liquidez do bruto em tempo real (UB) / volume bruto | 16,1% |
| Liquidez com uma compensação diária (LB) / UB | 0,91 |
| Escada de circularidade (grafo bruto): Hodge L1 → CDFD → ciclos | 1,18 → 0,83 → 0,80 |
| Escada de circularidade (grafo líquido): Hodge L2 → CDFD → ciclos | 0,98 → 0,17 → 0,14 |
| Instantâneo vs 1 ciclo por dia (lógica do estudo finlandês) | 1,10× (σ = 0,4) a 1,50× (σ = 0) a liquidez |
| Instantâneo vs 10 ciclos por dia | 1,03× a 1,11× |
| Micro-ciclos com espera de 2 s: meta do Pix de 50% em até 2,8 s | cumprida (100%) |
| Micro-ciclos com espera de 10 s: mesma meta | não cumprida (43%) |

<table>
  <tr>
    <td width="50%"><img src="results/microcycle-sweep/fig_tradeoff.svg" alt="Micro-ciclos: economia de liquidez versus atraso"></td>
    <td width="50%"><img src="results/finland-replication/fig_finland.svg" alt="Liquidez instantânea versus ciclos por dia"></td>
  </tr>
  <tr>
    <td align="center"><sub>Micro-ciclos: economia × atraso</sub></td>
    <td align="center"><sub>Instantâneo × ciclos de compensação por dia</sub></td>
  </tr>
</table>

### Diálogo com a literatura

#### Banco da Finlândia: o ponto de partida

Hellqvist e Korpinen (Banco da Finlândia, 2021; *JFMI*, 2026) perguntaram **quanto dinheiro a
mais os bancos precisam** quando um sistema de compensação em ciclos vira instantâneo. Eles
geraram transações sintéticas que imitam as transferências de varejo entre os participantes
finlandeses do STEP2 e compararam a liquidez do modelo em ciclos com a do modo instantâneo. O
LoopSettle parte exatamente dessa lógica e a estende.

| | Banco da Finlândia (Hellqvist & Korpinen) | LoopSettle |
|---|---|---|
| **Pergunta** | Quanto a mais de liquidez o instantâneo exige em relação aos ciclos? | A mesma, e o caminho inverso: quanto dá para **recuperar** sem perder o instantâneo, e a que custo em atraso? |
| **Sistema** | STEP2, varejo, Finlândia | Pix, Brasil (instantâneo, 24×7, liquidação bruta pelo banco central) |
| **Dados** | Sintéticos, imitando os fluxos do STEP2 | Sintéticos, calibrados com agregados públicos do Pix (erro ≤ 1,5%); cada parâmetro com fonte ou marcado `ASSUMPTION` |
| **Desenhos comparados** | Ciclos fixos × instantâneo | Ciclos fixos (1 a 8.640 por dia) × instantâneo × **micro-ciclos atômicos** (novo) |
| **Resultado agregado** | Aumento pequeno, no máximo 8,6% ao dia, segundo o resumo do debatedor[^bof] | Instantâneo exige +9,5% em relação a 1 ciclo diário e +2,6% em relação a 10 ciclos por dia (σ = 0,4): **mesma ordem de grandeza** |
| **Heterogeneidade** | Diferenças grandes entre bancos e entre dias; recomenda reservas de liquidez | Medida por participante (`results/baseline/participants.csv`); a sensibilidade ao desequilíbrio entre recebedores muda a razão de 1,10× para 1,50× |
| **Circularidade** | Não mede | Três lentes (Hodge, CDFD, ciclos) e a escada entre elas |
| **Potencial × realizável** | Não separa | Separa: é o resultado central |
| **Código** | Não localizado publicamente (a conferir) | Aberto (Apache-2.0), reprodutível byte a byte, roda num i3 com 4 GB e no navegador |

**Onde os dois estudos concordam:** a migração para o instantâneo custa poucos pontos
percentuais de liquidez agregada quando o sistema anterior já tinha vários ciclos por dia.
**O que o LoopSettle acrescenta:** a liquidez perdida está quase toda em horizontes de horas.
Por isso ela não volta com compensação em segundos, por mais valor que os micro-ciclos consigam
liquidar dentro de ciclos.

> Isto **não é uma replicação** do estudo finlandês: aplicamos a mesma lógica a outro cenário e
> comparamos ordens de grandeza (ver [`docs/CLAIMS.md`](docs/CLAIMS.md), item "não pode" 5).

[^bof]: O valor de 8,6% vem do resumo da discussão do trabalho no Economics of Payments X
    (Banco da Finlândia, 2021) e precisa ser conferido no texto integral. Situação de cada
    referência em [`docs/REFERENCES.md`](docs/REFERENCES.md).

#### Os outros trabalhos

| Trabalho | O que fez | O que o LoopSettle faz com ele |
|---|---|---|
| Gonzalez, Ma & Zeng, **BCB WP 619** (2025) | Com dados reais do Pix, mostra que os bancos perderam a capacidade de adiar e compensar pagamentos e passaram a reter mais ativos líquidos | Reproduz a medida **potencial** de compensação (sem ordem no tempo) em dados sintéticos e a coloca ao lado da economia **realizável** no tempo |
| Homs-Dones, MacKay, Sansom & Zhou, **CDFD** (arXiv 2506.12546, 2025) | Decompõe o fluxo de uma rede em parte circular e parte direcional respeitando conservação e o limite de cada aresta | Implementa a decomposição de **circularidade máxima** de forma exata (fluxo de custo mínimo) e a usa como teto estático; a variante proporcional (BFF) fica para depois |
| Bech & Soramäki, **BoF DP 9/2001** | Algoritmos de resolução de *gridlock* por ciclos em sistemas interbancários | Netting por ciclos e a base dos micro-ciclos atômicos, agora com atraso máximo e a meta de tempo do Pix |

Comparação completa, ponto a ponto, em [`docs/NOVELTY.md`](docs/NOVELTY.md).

### Comece em 3 comandos

```bash
cargo test --workspace                                                       # invariantes e testes
cargo run --release -p loopsettle-cli -- experiment potential-vs-realizable  # comparação central
cd web && npm install && npm run wasm && npm run dev                         # interface
```

Requisitos: Rust estável com o target `wasm32-unknown-unknown`, `wasm-pack` e Node LTS.
Tudo de uma vez, do zero: `make rebuild`. Roda num Intel i3 de 4ª geração com 4 GB de RAM:
cada experimento usa menos de 20 MiB e termina em segundos.

### Reprodutibilidade

Cada número publicado vem de um comando com semente fixa. Rodar de novo reproduz os resultados
**byte a byte**; a semente e a versão do código ficam registradas em cada `summary.json`.

| Experimento | Pergunta | Saída |
|---|---|---|
| `calibration` | o gerador reproduz os agregados públicos do Pix? | `results/calibration/` |
| `baseline` | quanto é circular, pelas três lentes? | `results/baseline/` |
| `potential-vs-realizable` | quanto da compensação potencial vira liquidez? | `results/potential-vs-realizable/` |
| `microcycle-sweep` | qual a troca entre atraso e liquidez? | `results/microcycle-sweep/` |
| `stress` | o que acontece se um participante grande cair por 30 min? | `results/stress/` |
| `finland-replication` | a lógica do Banco da Finlândia aplicada ao cenário | `results/finland-replication/` |

**Oito invariantes testadas por propriedades** (`proptest`, milhares de casos aleatórios a cada
execução): conservação do dinheiro; preservação da posição líquida de cada participante;
ausência de saldo negativo; atomicidade dos micro-ciclos; netting que nunca inverte a direção
de um pagamento nem cancela mais que o menor pagamento do ciclo; consistência da decomposição de
Hodge; determinismo total; e o caso fixo do triângulo.

### Arquitetura

| Componente | Conteúdo |
|---|---|
| [`loopsettle-core`](crates/loopsettle-core) | ledger em centavos (`i64`, nunca ponto flutuante), motores de liquidação, métricas em streaming |
| [`loopsettle-topology`](crates/loopsettle-topology) | Hodge (gradiente conjugado implementado à mão), CDFD exata, netting por ciclos |
| [`loopsettle-sim`](crates/loopsettle-sim) | gerador sintético calibrado, em streaming |
| [`loopsettle-cli`](crates/loopsettle-cli) | experimentos reprodutíveis, saída em CSV, JSON e SVG |
| [`loopsettle-wasm`](crates/loopsettle-wasm) + [`web/`](web) | interface Vite + React + Canvas 2D, simulação num Web Worker |
| [`data/`](data) | download dos agregados públicos do Pix (API Olinda do BCB) |
| [`docs/`](docs) | metodologia, parâmetros, limitações, afirmações, novidade, referências |

O núcleo não tem dependências externas: álgebra linear esparsa, gerador aleatório e formatos de
arquivo foram implementados no próprio projeto.

### Limitações

O LoopSettle é explícito sobre o que não sabe:

- **Dados sintéticos calibrados, não dados interbancários reais.** Só agregados públicos são
  usados. Quem paga a quem não é público e é a principal hipótese do modelo; por isso há uma
  análise de sensibilidade ao desequilíbrio entre recebedores (σ de 0 a 0,8).
- **Escala reduzida:** 200 participantes e 1 milhão de pagamentos por dia, contra cerca de 239
  milhões no Pix real. As frações são o resultado; os valores absolutos não escalam linearmente.
- **Micro-ciclos heurísticos:** a busca de ciclos é simples, não um otimizador completo.
- **Horizonte de um dia** e perfil horário sem rajadas; o cenário de estresse ainda é fraco.

Lista completa em [`docs/LIMITATIONS.md`](docs/LIMITATIONS.md); hipóteses e fonte de cada
parâmetro em [`docs/PARAMETERS.md`](docs/PARAMETERS.md); pontos abertos para revisão em
[`REVIEW.md`](REVIEW.md).

### Referências

- Hellqvist, M. & Korpinen, K. (2021). *Instant payments as a new normal: Case study of
  liquidity impacts for the Finnish market*. Bank of Finland Economics Review 7/2021.
- Hellqvist, M. & Korpinen, K. (2026). *Instant payments as the new normal: how much more money
  do the banks need?* Journal of Financial Market Infrastructures, 13(2). **Base metodológica
  que este projeto estende.**
- Gonzalez, R. B., Ma, Y. & Zeng, Y. (2025). *The Effect of Instant Payments on the Banking
  System*. Banco Central do Brasil, Working Paper 619.
- Homs-Dones, M., MacKay, R. S., Sansom, B. & Zhou, Y. (2025). *Circular Directional Flow
  Decomposition of Networks*. arXiv:2506.12546.
- Bech, M. L. & Soramäki, K. (2001). *Gridlock Resolution in Interbank Payment Systems*. Bank of
  Finland Discussion Papers 9/2001.

Situação de verificação de cada referência em [`docs/REFERENCES.md`](docs/REFERENCES.md).

### Como citar

```bibtex
@software{dorneles_loopsettle,
  author  = {Dorneles, Fabio D.},
  title   = {LoopSettle: an open simulator of circular liquidity in instant payment systems},
  version = {0.1.0},
  url     = {https://github.com/fabiodrneles/loopsettle},
  license = {Apache-2.0}
}
```

Os metadados também estão em [`CITATION.cff`](CITATION.cff) (o GitHub mostra o botão
"Cite this repository").

### Licença

[Apache-2.0](LICENSE).

---

## English

### In one sentence

Pix requires every bank to hold cash before every transfer. Part of that cash only goes around
in loops (A pays B, B pays C, C pays A). LoopSettle measures how large that part is, how much of
it could be netted and, above all, **how much of that netting turns into liquidity actually
saved** once the order in which payments arrive is respected.

### The problem

Brazil's Pix settles every payment **gross and in real time**: each institution must hold funds
in its settlement account (Conta PI) before every transfer. Older systems let banks batch
payments and offset inflows against outflows (*netting*), saving liquidity. The Banco Central do
Brasil study of Pix (Working Paper 619) shows that banks lost this ability to delay and net
payments and now hold more liquid assets.

One question remained open: **how much of the lost netting translates into liquidity that could
actually be freed?** And can part of it be recovered without giving up the instant user
experience?

> **Research question.** How much of the liquidity Pix participants must hold is circular, how
> much of it is recoverable in practice, and what delay does recovering it with atomic
> micro-cycles cost?

### Key finding

In a synthetic scenario calibrated to public Pix aggregates (200 participants, 1 million
payments per day, seed `20260930`):

- **85% of payment volume** is nettable under the potential measure (same-day inflows vs
  outflows, ignoring time order).
- The **realizable liquidity saving** depends on the netting horizon: **9.3%** with one daily
  netting cycle, **0.3%** every 15 minutes and **under 0.03%** with second-scale cycles.
- **Atomic micro-cycles of up to 10 s settle up to 63% of value inside cycles, yet save under
  0.02% of liquidity.** Liquidity needs come from imbalances lasting hours (a bank paying out
  more than it receives all morning), not from second-scale noise.

> **Nettable volume is not saved liquidity.** LoopSettle makes the gap between the two visible
> and measurable.

All numbers are in [`results/SUMMARY.md`](results/SUMMARY.md), with the seed, code version and
command that produced them. What the study can and cannot claim is in
[`docs/CLAIMS.md`](docs/CLAIMS.md).

### The interface

The screenshots above show the browser app (labels in Portuguese). It runs fully client-side:
the Rust simulator is compiled to WebAssembly and runs in a Web Worker, so the page never
freezes.

- **The network of one day:** participants on a circle (size = activity, dashed = net payer);
  green edges are the maximum circulation (CDFD), grey edges the directional flow.
- **Potential vs realizable:** the ladder from netting on paper down to liquidity actually
  saved.
- **Sliders** for the micro-cycle search window (0.5–10 s), the maximum tolerated delay
  (2–10 s), the number of participants and payments, and the receiver imbalance (the model's
  main assumption).
- **Constrained liquidity:** with less than the required opening balance, how much value each
  design rejects and how many payments meet the Pix target (50% within 2.8 s).
- **Phones** get a reduced mode (up to 25 participants and 30,000 payments) with the same model.

Browser scenarios are smaller than the experiments, so on-screen numbers are not the paper's
numbers. In small scenarios the micro-cycle saving can even be slightly **negative** (−0.001%):
the queue changes the settlement order and a participant's peak can rise slightly. This is a
model result, consistent with the main finding (saving close to zero), not a bug.

```bash
cd web && npm install && npm run wasm && npm run dev    # opens at http://localhost:5173
```

Screenshots are regenerated with `node scripts/screenshots.cjs` while the UI is running.

### How it works

**Three lenses on circular flow**

| Lens | What it measures | Why it matters |
|---|---|---|
| **Hodge decomposition** | splits flow into a directional (gradient) and a circular component | good diagnostic, but it can assign more to cycles than exists: in the triangle A→B 10, B→C 5, C→A 5 it gives 6.67 to the cycle while only 5 is nettable |
| **Maximum CDFD** | largest circulation that respects each edge's value and flow conservation at each node | exact, realistic static ceiling (hand-written min-cost flow, checked against an LP solver on 1,000 random graphs) |
| **Cycle netting** | what a Bech–Soramäki-style netting algorithm can cancel | what is executable in practice |

**Three settlement designs side by side**

| Engine | Behaviour |
|---|---|
| `GrossRealTime` | settles each payment on arrival if funds suffice (the Pix model) |
| `DeferredNet` | multilateral netting at fixed cycles, from once a day down to every 10 s (UPI style, as in the Finnish study) |
| `AtomicMicroCycle` | waits a few seconds, finds cycles in the queue and settles each cycle at once, all or nothing; modes `QueueAll` and `QueueOnShortfall` (LSM style) |

**How liquidity is measured**

- **UB (upper bound):** the smallest opening balance that avoids any rejection under real-time
  gross settlement = the intraday maximum of cumulative outflows − cumulative inflows.
- **LB (lower bound):** the end-of-day multilateral net position, which a single daily netting
  would require.
- **Saving of a design** = 1 − design liquidity ÷ UB.

### Results at a glance

| Measure | Value |
|---|---|
| Calibration error against public Pix aggregates | ≤ 1.5% on all 9 aggregates |
| Real-time gross liquidity (UB) / gross volume | 16.1% |
| One daily netting (LB) / UB | 0.91 |
| Circularity ladder (gross graph): Hodge L1 → CDFD → cycles | 1.18 → 0.83 → 0.80 |
| Circularity ladder (net graph): Hodge L2 → CDFD → cycles | 0.98 → 0.17 → 0.14 |
| Instant vs 1 cycle per day (Finnish study logic) | 1.10× (σ = 0.4) to 1.50× (σ = 0) the liquidity |
| Instant vs 10 cycles per day | 1.03× to 1.11× |
| Micro-cycles with 2 s wait: Pix target of 50% within 2.8 s | met (100%) |
| Micro-cycles with 10 s wait: same target | missed (43%) |

### Dialogue with the literature

#### Bank of Finland: the starting point

Hellqvist and Korpinen (Bank of Finland, 2021; *JFMI*, 2026) asked **how much more money banks
need** when a cycle-based clearing system becomes instant. They generated synthetic transactions
mimicking retail credit transfers between Finnish STEP2 participants and compared liquidity under
the cycle-based model with full instant mode. LoopSettle starts from exactly this logic and
extends it.

| | Bank of Finland (Hellqvist & Korpinen) | LoopSettle |
|---|---|---|
| **Question** | How much more liquidity does instant settlement need than cycles? | The same, plus the reverse path: how much can be **recovered** without losing instant settlement, at what delay cost? |
| **System** | STEP2, retail, Finland | Pix, Brazil (instant, 24/7, gross settlement by the central bank) |
| **Data** | Synthetic, mimicking STEP2 flows | Synthetic, calibrated to public Pix aggregates (error ≤ 1.5%); every parameter sourced or flagged `ASSUMPTION` |
| **Designs compared** | Fixed cycles × instant | Fixed cycles (1 to 8,640 per day) × instant × **atomic micro-cycles** (new) |
| **Aggregate result** | Small increase, at most 8.6% per day according to the discussant's summary[^bof-en] | Instant needs +9.5% over 1 daily cycle and +2.6% over 10 cycles per day (σ = 0.4): **same order of magnitude** |
| **Heterogeneity** | Large differences across banks and days; recommends liquidity buffers | Measured per participant (`results/baseline/participants.csv`); receiver imbalance moves the ratio from 1.10× to 1.50× |
| **Circularity** | Not measured | Three lenses (Hodge, CDFD, cycles) and the ladder between them |
| **Potential vs realizable** | Not separated | Separated: it is the central result |
| **Code** | Not found publicly (to be checked) | Open (Apache-2.0), byte-for-byte reproducible, runs on a 4 GB i3 and in the browser |

**Where both studies agree:** moving to instant settlement costs a few percentage points of
aggregate liquidity when the previous system already ran several cycles a day. **What LoopSettle
adds:** the lost liquidity sits almost entirely on hour-long horizons, so second-scale netting
does not bring it back, however much value micro-cycles settle inside cycles.

> This is **not a replication** of the Finnish study: we apply the same logic to a different
> scenario and compare orders of magnitude (see [`docs/CLAIMS.md`](docs/CLAIMS.md)).

[^bof-en]: The 8.6% figure comes from the discussant's summary at Economics of Payments X (Bank
    of Finland, 2021) and still has to be checked against the full text. Verification status of
    every reference is in [`docs/REFERENCES.md`](docs/REFERENCES.md).

#### Other works

| Work | What it did | What LoopSettle does with it |
|---|---|---|
| Gonzalez, Ma & Zeng, **BCB WP 619** (2025) | Using real Pix data, shows that banks lost the ability to delay and net payments and now hold more liquid assets | Reproduces the **potential** netting measure (time order ignored) on synthetic data and puts it next to the **realizable** saving over time |
| Homs-Dones, MacKay, Sansom & Zhou, **CDFD** (arXiv 2506.12546, 2025) | Decomposes network flow into circular and directional parts, respecting conservation and edge bounds | Implements the **maximum circularity** decomposition exactly (min-cost flow) as a static ceiling; the proportional (BFF) variant is future work |
| Bech & Soramäki, **BoF DP 9/2001** | Cycle-based gridlock resolution in interbank systems | Cycle netting and the basis for atomic micro-cycles, now with a maximum delay and the Pix time target |

Full point-by-point comparison in [`docs/NOVELTY.md`](docs/NOVELTY.md) (Portuguese).

### Quick start

```bash
cargo test --workspace                                                       # invariants and tests
cargo run --release -p loopsettle-cli -- experiment potential-vs-realizable  # central comparison
cd web && npm install && npm run wasm && npm run dev                         # interface
```

Requirements: stable Rust with the `wasm32-unknown-unknown` target, `wasm-pack` and Node LTS.
Everything from scratch: `make rebuild`. Runs on a 4th-generation Intel i3 with 4 GB of RAM:
each experiment uses under 20 MiB and finishes in seconds.

### Reproducibility

Every published number comes from a command with a fixed seed and reproduces **byte for byte**;
the seed and code version are recorded in each `summary.json`. The six experiments
(`calibration`, `baseline`, `potential-vs-realizable`, `microcycle-sweep`, `stress`,
`finland-replication`) write CSV, JSON and SVG to `results/<name>/`.

**Eight invariants checked by property tests** (`proptest`): conservation of money; each
participant's net position preserved; no negative balances; micro-cycle atomicity; netting never
reverses a payment nor cancels more than the cycle's smallest payment; Hodge consistency; full
determinism; and the fixed triangle case.

### Architecture

| Component | Contents |
|---|---|
| [`loopsettle-core`](crates/loopsettle-core) | ledger in cents (`i64`, never floating point), settlement engines, streaming metrics |
| [`loopsettle-topology`](crates/loopsettle-topology) | Hodge (hand-written conjugate gradient), exact CDFD, cycle netting |
| [`loopsettle-sim`](crates/loopsettle-sim) | calibrated streaming synthetic generator |
| [`loopsettle-cli`](crates/loopsettle-cli) | reproducible experiments, CSV/JSON/SVG output |
| [`loopsettle-wasm`](crates/loopsettle-wasm) + [`web/`](web) | Vite + React + Canvas 2D UI, simulation in a Web Worker |
| [`data/`](data) | download of public Pix aggregates (BCB Olinda API) |
| [`docs/`](docs) | methodology, parameters, limitations, claims, novelty, references (Portuguese) |

The core has no external dependencies: sparse linear algebra, the random generator and file
formats are implemented in the project itself. Code is in English; detailed documentation is in
Portuguese.

### Limitations

- **Synthetic data calibrated to public aggregates, not real interbank data.** Who pays whom is
  not public and is the model's main assumption, hence the sensitivity analysis on receiver
  imbalance (σ from 0 to 0.8).
- **Reduced scale:** 200 participants and 1 million payments per day, versus about 239 million
  in real Pix. Fractions are the result; absolute values do not scale linearly.
- **Heuristic micro-cycles:** the cycle search is simple, not a full optimiser.
- **One-day horizon**, hourly profile without bursts; the stress scenario is still weak.

See [`docs/LIMITATIONS.md`](docs/LIMITATIONS.md), [`docs/PARAMETERS.md`](docs/PARAMETERS.md) and
[`REVIEW.md`](REVIEW.md).

### References

See the [Portuguese section](#referências) above; verification status in
[`docs/REFERENCES.md`](docs/REFERENCES.md).

### Cite

See [`CITATION.cff`](CITATION.cff) or the BibTeX entry above.

### License

[Apache-2.0](LICENSE).
