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
  <img alt="Docs" src="https://img.shields.io/badge/docs-PT%20%7C%20EN-555.svg">
</p>

<p align="center">
  <b>Simulador aberto que mede quanto da liquidez de um sistema de pagamentos instantâneos dá voltas entre as instituições — e quanto disso é recuperável na prática.</b><br>
  <i>An open simulator that measures how much of the liquidity in an instant payment system goes around in loops — and how much of it can actually be recovered.</i>
</p>

<p align="center">
  <b><a href="#português">Português</a></b> ·
  <a href="results/SUMMARY.pt-BR.md">Resultados</a> ·
  <a href="#instalação-do-zero">Instalação</a> ·
  <a href="docs/METHODOLOGY.pt-BR.md">Metodologia</a> ·
  <a href="docs/LIMITATIONS.pt-BR.md">Limitações</a> ·
  <a href="docs/REFERENCES.pt-BR.md">Referências</a>
  <br>
  <b><a href="#english">English</a></b> ·
  <a href="results/SUMMARY.md">Results</a> ·
  <a href="#installation-from-scratch">Installation</a> ·
  <a href="docs/METHODOLOGY.md">Methodology</a> ·
  <a href="docs/LIMITATIONS.md">Limitations</a> ·
  <a href="docs/REFERENCES.md">References</a>
</p>

---

## Português

<p align="center">
  <img src="docs/img/ui-overview.png" alt="Interface do LoopSettle: rede de participantes com o componente circular em verde, controles dos micro-ciclos e painéis de liquidez" width="100%">
</p>
<p align="center"><sub>A interface roda inteira no navegador (Rust → WebAssembly, num Web Worker): 40 participantes e 100 mil pagamentos simulados em 0,5 s. Botão no canto para trocar para inglês.</sub></p>

### Em uma frase

Nos sistemas de pagamentos instantâneos com liquidação bruta — Pix (Brasil), FedNow (EUA), TIPS
(zona do euro), NPP (Austrália) — cada banco precisa ter dinheiro em caixa antes de cada
transferência. Parte desse dinheiro só "dá voltas" (A paga B, B paga C, C paga A). O LoopSettle
mede quanto é essa parte,
quanto dela daria para compensar e, principalmente, **quanto dessa compensação vira liquidez
economizada de verdade** quando se respeita a ordem em que os pagamentos acontecem.

### O problema

Cada vez mais países adotam pagamentos instantâneos liquidados de forma **bruta e em tempo
real**: cada pagamento é liquidado sozinho, na hora, e o banco pagador precisa ter saldo no banco
central antes de cada transferência. Nos sistemas antigos, os bancos podiam acumular pagamentos e
compensar entradas com saídas (*netting*), economizando liquidez. No Brasil, onde o Pix funciona
assim desde 2020, o estudo do Banco Central (Working Paper 619) mostra que os bancos perderam essa
capacidade de adiar e compensar, e passaram a reter mais ativos líquidos. O Banco da Finlândia
fez a mesma pergunta para a Europa.

O LoopSettle usa o **Pix como caso calibrado**, porque ele publica os agregados necessários, mas
os motores e as métricas valem para qualquer sistema do mesmo tipo. Veja
[onde o seu sistema se encaixa](#onde-o-seu-sistema-se-encaixa).

Uma pergunta continuava sem resposta: **quanto dessa compensação perdida vira, de fato,
liquidez que poderia ser liberada?** E dá para recuperar parte dela sem abrir mão da
experiência instantânea do usuário?

> **Pergunta de pesquisa.** Num sistema de pagamentos instantâneos com liquidação bruta — com o
> Pix como caso calibrado —, quanto da liquidez que os participantes precisam manter é circular,
> quanto disso é recuperável na prática, e qual o custo em atraso de recuperá-la com micro-ciclos
> atômicos?

### O resultado principal

<p align="center">
  <img src="results/potential-vs-realizable/fig_potential_vs_realizable.svg" alt="Economia de liquidez em função do tempo máximo de espera" width="720">
</p>

No cenário sintético calibrado com dados públicos do Pix (200 participantes; 10 sementes; de 1 a
30 milhões de pagamentos por dia):

- **85% do volume** de pagamentos poderia ser "compensado" pela medida potencial (entradas e
  saídas do mesmo dia, sem olhar a ordem no tempo).
- **Micro-ciclos atômicos de até 10 s liquidam até 63% do valor dentro de ciclos, mas
  economizam cerca de 0,01% da liquidez** (0,0116%, IC 95% [0,008; 0,015]%), qualquer que seja
  o tamanho máximo do ciclo, e a economia vai a zero quando o volume cresce.
- **Há um teto demonstrável:** um mecanismo que segura cada pagamento por no máximo *d* nunca
  precisa de menos liquidez do que o **piso de liquidez** de cada participante: o maior valor de
  "tudo o que ele pagou até um instante, menos tudo o que recebeu até *d* depois". Para 10 s, em
  10 sementes, esse teto de economia vai de 0,014% da liquidez (estrutura calibrada ao WP 619) a
  0,14% (sem viés recebedor), e a compensação a cada 10 s obtém de 38% a 46% dele. Uma forma
  mais simples e mais folgada do mesmo limite (o valor pago nos *d* segundos antes do pico) dá
  0,7% a 3,3%.
- Compensar em horizontes maiores economiza mais (0,5% a cada 15 minutos; 7,8% uma vez por dia,
  com 1 milhão de pagamentos), **mas esse "potencial" encolhe com o volume**. Com a estrutura
  bilateral calibrada em dados reais do Pix (BCB WP 619) e 30 milhões de pagamentos por dia, uma
  compensação diária recupera **entre 0,2% e 2,8%** da liquidez, conforme o quanto os bancos têm
  padrões intradiários diferentes entre si — a pergunta que só dados por participante respondem
  ([avaliação](docs/ASSESSMENT.pt-BR.md)).
- **A restrição real é o fim de semana.** A Conta PI só recebe aportes com o STR aberto (dias
  úteis, 6h30–18h30), então os bancos pré-financiam as noites e as 59,5 horas do fim de semana —
  cerca de 2,7× o pico de um dia. Mesmo aí, compensar no fim do fim de semana economizaria só
  0,4–1,0%: a necessidade vem do desequilíbrio líquido acumulado, não de fluxo circular.

> **Volume compensável não é liquidez economizada.** O LoopSettle torna visível e mensurável a
> distância entre essas duas medidas.

Todos os números estão em [`results/SUMMARY.pt-BR.md`](results/SUMMARY.pt-BR.md), com a semente, a versão
do código e o comando que os gerou. O que o estudo pode e não pode afirmar está em
[`docs/CLAIMS.pt-BR.md`](docs/CLAIMS.pt-BR.md).

*(As figuras geradas pelos experimentos têm rótulos em inglês, a língua do código; as legendas
e o [`results/SUMMARY.pt-BR.md`](results/SUMMARY.pt-BR.md) explicam cada uma em português.)*

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
    <td align="center"><sub><b>Liquidez necessária por desenho</b>, do bruto em tempo real ao mínimo teórico (LB).</sub></td>
  </tr>
  <tr>
    <td><img src="docs/img/ui-sweep.png" alt="Economia de liquidez dos micro-ciclos para cada janela de busca"></td>
    <td><img src="docs/img/ui-constrained.png" alt="Tabela com liquidez restrita: valor rejeitado, atraso p95 e fração liquidada em até 2,8 s"></td>
  </tr>
  <tr>
    <td align="center"><sub><b>Economia × janela.</b> A economia dos micro-ciclos para cada janela de busca de ciclos: sempre perto de zero.</sub></td>
    <td align="center"><sub><b>Liquidez restrita.</b> Com saldo inicial menor que o necessário: valor rejeitado e cumprimento da meta de 2,8 s.</sub></td>
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
- Trocar o **idioma** (português/inglês) no botão do canto, ou abrir com `?lang=pt` ou
  `?lang=en`.

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

As capturas, nos dois idiomas, são regeneradas por comando: `node scripts/screenshots.cjs`
(com a interface no ar).

### Como funciona

```mermaid
flowchart LR
  A["Dados públicos<br/>(caso calibrado: Pix,<br/>API Olinda / BCB)"] --> B["Gerador sintético<br/>calibrado, em streaming"]
  B --> C["Motores de liquidação"]
  C --> C1["Bruto em tempo real<br/>(Pix, FedNow, TIPS, NPP)"]
  C --> C2["Compensação diferida<br/>1× ao dia … a cada 10 s<br/>(UPI, Faster Payments)"]
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
| `GrossRealTime` | liquida cada pagamento na chegada, se houver saldo (o modelo do Pix, FedNow, TIPS e NPP) |
| `DeferredNet` | compensação multilateral em ciclos fixos, de 1 por dia a 1 a cada 10 s (o modelo do UPI e do Faster Payments; lógica do estudo finlandês) |
| `AtomicMicroCycle` | espera alguns segundos, encontra ciclos na fila e liquida cada ciclo de uma vez, tudo ou nada; modos `QueueAll` (todos passam pela fila) e `QueueOnShortfall` (só na falta de saldo, estilo LSM) |

**Como a liquidez é medida**

- **UB (limite superior):** menor saldo inicial que evita qualquer rejeição no bruto em tempo
  real = máximo, ao longo do dia, de saídas acumuladas − entradas acumuladas.
- **LB (limite inferior):** posição líquida multilateral do fim do dia, o que uma única
  compensação diária exigiria.
- **Economia de um desenho** = 1 − liquidez do desenho ÷ UB.

Definições formais, exemplo à mão e provas em [`docs/METHODOLOGY.pt-BR.md`](docs/METHODOLOGY.pt-BR.md).

### Onde o seu sistema se encaixa

O Pix é o caso calibrado, mas a pergunta vale para qualquer país. Para o cliente, todos os
sistemas abaixo são instantâneos; o que muda a liquidez é **como os bancos acertam as contas
entre si**, e o LoopSettle tem um motor para cada modelo:

<table>
  <tr>
    <td width="40%"><img src="docs/img/ui-fit.png" alt="Painel da interface: onde o seu sistema se encaixa, com os modelos de liquidação e exemplos de países"></td>
    <td>

| Modelo entre bancos | Exemplos | Motor |
|---|---|---|
| Bruto em tempo real | Pix (Brasil), FedNow (EUA), TIPS (zona do euro), NPP (Austrália) | `GrossRealTime` |
| Bruto, conta pré-financiada | RTP (EUA), Real-Time Rail (Canadá, em lançamento) | `GrossRealTime` com saldo limitado |
| Compensação diferida em ciclos | UPI (Índia, 10/dia), Faster Payments (Reino Unido, 3/dia), FAST (Singapura, 2/dia), Interac e-Transfer (Canadá, 1/dia) | `DeferredNet` |
| Compensação a cada poucos segundos | SPEI (México, em média a cada 1,9 s) | `AtomicMicroCycle` |

</td>
  </tr>
</table>

- **Nos EUA**, o FedNow funciona como o Pix: o resultado central se aplica diretamente ao desenho.
- **No Canadá**, o Interac e-Transfer é instantâneo para o cliente, mas os bancos compensam o saldo
  líquido uma vez por dia, pelo Lynx (modelo `DeferredNet`). O novo Real-Time Rail, com lançamento
  previsto para o fim de 2026, liquida em tempo real contra contas pré-financiadas no Bank of
  Canada: é exatamente o tipo de sistema para o qual a lógica do estudo finlandês mede o custo da
  migração.
- **No México**, o SPEI já roda um algoritmo de compensação a cada 1,9 s em média: é o análogo real
  mais próximo dos micro-ciclos atômicos.
- **Os números** (0,01%, 0,5%, 7,8%) vêm do cenário calibrado com o Pix e não valem
  automaticamente para outro país: recalibrar exige os agregados públicos dele.

Tabela completa, com operador, data de início, modelo de liquidação e fonte de cada sistema, em
[`docs/INSTANT_PAYMENTS.pt-BR.md`](docs/INSTANT_PAYMENTS.pt-BR.md).

### Resultados em números

| Medida | Valor |
|---|---|
| Erro da calibração frente aos agregados públicos do Pix | ≤ 1,5% em todos os 9 agregados |
| Liquidez do bruto em tempo real (UB) / volume bruto | 16,1% |
| Liquidez com uma compensação diária (LB) / UB | 0,91 |
| Escada de circularidade (grafo bruto): Hodge L1 → CDFD → ciclos | 1,18 → 0,83 → 0,80 |
| Escada de circularidade (grafo líquido): Hodge L2 → CDFD → ciclos | 0,98 → 0,17 → 0,14 |
| Instantâneo vs 1 ciclo por dia (lógica do estudo finlandês) | 1,10× (σ = 0,4) a 1,50× (σ = 0) a liquidez |
| Instantâneo vs 6 ciclos por dia (a base do STEP2) | 1,04× a 1,14× |
| Micro-ciclos com espera de 2 s: meta do Pix de 50% em até 2,8 s | cumprida (100%) |
| Micro-ciclos com espera de 10 s: mesma meta | não cumprida (43%) |
| Economia dos micro-ciclos de até 10 s † | 0,0116% do UB, IC 95% [0,008; 0,015]% |
| Teto para qualquer mecanismo de até 10 s † | 0,80% do UB (0,27% com 30 milhões de pagamentos) |
| Economia de uma compensação diária † | 7,8%, IC 95% [4,5; 11,1]% (1,7% com 30 milhões) |

<sub>Linhas sem marca: semente `20260930`, 1 milhão de pagamentos por dia. † 10 sementes, σ = 0,4 (`results/robustness/`).</sub>

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

<p align="center">
  <img src="results/baseline/fig_ladder.svg" alt="Escada de circularidade: Hodge, CDFD máxima e netting por ciclos nos grafos bruto e líquido" width="720">
</p>
<p align="center"><sub>A escada de circularidade: o diagnóstico (Hodge) atribui ao ciclo muito mais do que o teto exato (CDFD) e do que um algoritmo de compensação consegue cancelar.</sub></p>

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
| **Sistema** | STEP2, varejo, Finlândia | Pagamentos instantâneos com liquidação bruta; caso calibrado: Pix, Brasil (24×7, banco central) |
| **Dados** | Sintéticos, imitando os fluxos do STEP2 | Sintéticos, calibrados com agregados públicos do Pix (erro ≤ 1,5%); cada parâmetro com fonte ou marcado `ASSUMPTION` |
| **Desenhos comparados** | Ciclos fixos × instantâneo | Ciclos fixos (1 a 8.640 por dia) × instantâneo × **micro-ciclos atômicos** (novo) |
| **Definição de liquidez** | Menor saldo sem rejeição: posição mínima do dia (instantâneo) e mínima das posições de fim de ciclo (ciclos) | A mesma (UB e LB), confirmada no texto deles |
| **Resultado agregado** | Contra os 6 ciclos diários do STEP2: +2,7% em média e menos de 8,6% em 95% dos dias. Contra 1 ciclo diário: 1,45× a liquidez[^bof] | Contra 6 ciclos: +3,9% (σ = 0,4) a +13,7% (σ = 0). Contra 1 ciclo: 1,10× a 1,50×: **mesma ordem de grandeza** |
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
> comparamos ordens de grandeza (ver [`docs/CLAIMS.pt-BR.md`](docs/CLAIMS.pt-BR.md), item "não pode" 5).

[^bof]: Hellqvist & Korpinen (2021), p. 2 e 9–11: 1.000 dias sintéticos para o aumento médio e o
    percentil 95; 30 dias para a comparação com 1 ciclo (€ 231 mi contra € 335 mi). Ver
    [`docs/REFERENCES.pt-BR.md`](docs/REFERENCES.pt-BR.md).

#### Os outros trabalhos

| Trabalho | O que fez | O que o LoopSettle faz com ele |
|---|---|---|
| Gonzalez, Ma & Zeng, **BCB WP 619** (2025) | Com dados reais do Pix, mostra que os bancos perderam a capacidade de adiar e compensar pagamentos e passaram a reter mais ativos líquidos | Reproduz a medida **potencial** de compensação (sem ordem no tempo) em dados sintéticos e a coloca ao lado da economia **realizável** no tempo |
| Homs-Dones, MacKay, Sansom & Zhou, **CDFD** (arXiv 2506.12546, 2025) | Decompõe o fluxo de uma rede em parte circular e parte direcional respeitando conservação e o limite de cada aresta | Implementa a decomposição de **circularidade máxima** de forma exata (fluxo de custo mínimo) e a usa como teto estático; a variante proporcional (BFF) fica para depois |
| Bech & Soramäki, **BoF DP 9/2001** | Algoritmos de resolução de *gridlock* por ciclos em sistemas interbancários | Netting por ciclos e a base dos micro-ciclos atômicos, agora com atraso máximo e a meta de tempo do Pix |

Comparação completa, ponto a ponto, em [`docs/NOVELTY.pt-BR.md`](docs/NOVELTY.pt-BR.md).

### Comece em 3 comandos

```bash
cargo test --workspace                                                       # invariantes e testes
cargo run --release -p loopsettle-cli -- experiment potential-vs-realizable  # comparação central
cd web && npm install && npm run wasm && npm run dev                         # interface
```

Primeira vez? Instale as ferramentas pelo guia [Instalação do zero](#instalação-do-zero), logo
abaixo. Tudo de uma vez: `make rebuild`. Roda num Intel i3 de 4ª geração com 4 GB de RAM: cada
experimento usa menos de 20 MiB e termina em segundos.

### Instalação do zero

Este guia parte de um computador sem nada instalado e chega ao simulador, aos experimentos e à
interface rodando. Na primeira vez leva de 20 a 40 minutos, quase tudo download e a primeira
compilação. Precisa de internet e de cerca de 3 GB livres em disco. Foi pensado para a máquina de
referência do projeto (Intel i3 de 4ª geração, 4 GB de RAM).

**O que será instalado**

| Ferramenta | Para quê | Obrigatória? | Versões testadas |
|---|---|---|---|
| Git | baixar o código | sim | qualquer recente |
| Rust (rustup + cargo) | simulador, testes e experimentos | sim | estável 1.97–1.99 |
| Compilador C/linker do sistema | o Rust usa para gerar o executável | sim | MSVC Build Tools (Windows), `build-essential` (Linux), Xcode CLT (macOS) |
| Target `wasm32-unknown-unknown` | compilar o simulador para o navegador | só para a interface | — |
| `wasm-pack` | empacotar o WebAssembly | só para a interface | 0.13.1 e 0.15.1 |
| Node.js LTS (com npm) | interface web | só para a interface | 22 e 24 |
| Python 3 | gerar `results/SUMMARY*.md` e baixar os dados públicos do Pix | opcional | 3.10 ou mais nova (só biblioteca padrão) |
| GNU Make | reconstruir tudo com um comando | opcional | 4.x |

Quer só rodar o simulador e os experimentos? Bastam Git, Rust e o compilador do sistema (passos 1
e 2 do seu sistema e o passo 3 comum). A interface precisa também do target wasm, do `wasm-pack` e
do Node.

#### Passo 1 — ferramentas do sistema

<details open>
<summary><b>Windows 10/11</b> (use o toolchain nativo; <b>não</b> use WSL2 com 4 GB de RAM)</summary>

Abra o **PowerShell** (menu Iniciar → "PowerShell") e rode um comando por vez. O `winget` já vem
no Windows 10/11 atualizado.

```powershell
winget install --id Git.Git -e
```

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

```powershell
winget install --id Rustlang.Rustup -e
```

```powershell
winget install --id OpenJS.NodeJS.LTS -e
```

Opcionais (resumo dos resultados e `make`):

```powershell
winget install --id Python.Python.3.12 -e
```

```powershell
winget install --id ezwinports.make -e
```

**Feche e abra o terminal** depois das instalações, para o `PATH` ser atualizado. Sem `winget`:
baixe o Git em [git-scm.com](https://git-scm.com/download/win), o Rust em
[rustup.rs](https://rustup.rs) (o instalador oferece instalar o Visual Studio Build Tools; aceite,
com a carga "Desenvolvimento para desktop com C++"), o Node LTS em [nodejs.org](https://nodejs.org)
e o Python em [python.org](https://www.python.org/downloads/) (marque "Add python.exe to PATH").

Os comandos dos passos seguintes funcionam no PowerShell e no **Git Bash** (instalado junto com o
Git). Onde forem diferentes, os dois aparecem.
</details>

<details>
<summary><b>Linux</b> (Ubuntu/Debian; Fedora no fim)</summary>

```bash
sudo apt update && sudo apt install -y build-essential git curl python3 make
```

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

```bash
source "$HOME/.cargo/env"
```

Node LTS pelo `nvm` (não precisa de `sudo` e não mexe no Node do sistema):

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
```

```bash
source "$HOME/.nvm/nvm.sh" && nvm install --lts
```

Com 4 GB de RAM, ative 2 GB de swap antes de compilar (evita travar o computador):

```bash
sudo fallocate -l 2G /swapfile && sudo chmod 600 /swapfile && sudo mkswap /swapfile && sudo swapon /swapfile
```

Fedora: troque a primeira linha por `sudo dnf install -y gcc git make python3 curl`.
</details>

<details>
<summary><b>macOS</b></summary>

```bash
xcode-select --install
```

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

```bash
source "$HOME/.cargo/env"
```

Node LTS: instale pelo [nodejs.org](https://nodejs.org) ou, com Homebrew, `brew install node@22`.
Git, `make` e Python 3 já vêm com as Xcode Command Line Tools.
</details>

#### Passo 2 — Rust pronto para a interface (todos os sistemas)

```bash
rustup default stable
```

```bash
rustup target add wasm32-unknown-unknown
```

```bash
npm install -g wasm-pack
```

O `npm` baixa o `wasm-pack` já compilado. Se ele falhar (rede corporativa, por exemplo), compile
pelo Cargo — leva de 5 a 15 minutos num i3:

```bash
cargo install wasm-pack --locked
```

#### Passo 3 — baixar o código e conferir

```bash
git clone https://github.com/fabiodrneles/loopsettle.git
```

```bash
cd loopsettle
```

Confira as versões (cada comando deve imprimir um número, não um erro):

```bash
git --version && cargo --version && rustup target list --installed && wasm-pack --version && node --version && npm --version
```

Todos os comandos daqui em diante são rodados **na raiz do repositório** (a pasta `loopsettle`).

#### Passo 4 — primeiro uso

1. **Testes** (na primeira vez baixa as dependências e compila; cerca de 1 a 2 minutos no i3):

   ```bash
   cargo test --workspace
   ```

   Esperado: várias linhas `test result: ok.` e nenhuma `FAILED`.

2. **Lista de experimentos:**

   ```bash
   cargo run --release -p loopsettle-cli -- list
   ```

3. **A comparação central** (gera CSV, JSON e SVG em `results/potential-vs-realizable/`):

   ```bash
   cargo run --release -p loopsettle-cli -- experiment potential-vs-realizable
   ```

4. **A interface** (num segundo terminal, também na raiz do repositório):

   ```bash
   cd web
   ```

   ```bash
   npm install
   ```

   ```bash
   npm run wasm
   ```

   ```bash
   npm run dev
   ```

   Abra o endereço que aparece no terminal (normalmente <http://localhost:5173>). Para parar:
   `Ctrl+C`. Versão otimizada: `npm run build` e depois `npm run preview` (normalmente
   <http://localhost:4173>). Sempre que mudar o código Rust, rode `npm run wasm` de novo.

5. **Tudo do zero** (testes, todos os experimentos, os resumos e a interface; cerca de 2 minutos
   no i3 depois da primeira compilação):

   ```bash
   make rebuild PYTHON=python3
   ```

   No Windows, use `make rebuild` (lá o Python se chama `python`). Sem `make`, o equivalente no
   Git Bash, Linux ou macOS é:

   ```bash
   cargo test --workspace && for e in $(cargo run --release -q -p loopsettle-cli -- list | cut -d' ' -f1); do cargo run --release -q -p loopsettle-cli -- experiment $e || break; done && cargo run --release -q -p loopsettle-cli -- topology data/samples/triangle.csv && python3 -I scripts/summarize.py
   ```

   E no PowerShell:

   ```powershell
   cargo test --workspace; cargo run --release -q -p loopsettle-cli -- list | ForEach-Object { cargo run --release -q -p loopsettle-cli -- experiment ($_ -split ' ')[0] }; cargo run --release -q -p loopsettle-cli -- topology data/samples/triangle.csv; python -I scripts/summarize.py
   ```

Depois de `cargo build --release -p loopsettle-cli`, o executável pode ser chamado direto, sem o
`cargo run`: `./target/release/loopsettle list` (Windows: `.\target\release\loopsettle.exe list`).
Nos exemplos abaixo, `loopsettle` significa um desses dois jeitos.

#### Referência de comandos

| Comando | O que faz | Saída |
|---|---|---|
| `cargo test --workspace` | todos os testes, incluindo as 8 invariantes por propriedades | terminal |
| `cargo test -p loopsettle-core` | testes de um crate só (`-core`, `-topology`, `-sim`, `-cli`) | terminal |
| `loopsettle list` | lista os experimentos e o que cada um responde | terminal |
| `loopsettle experiment <nome>` | roda um experimento com a semente fixa do seu TOML | `results/<nome>/` (CSV, `summary.json`, `run.json`, SVG) |
| `loopsettle experiment <nome> --quick` | o mesmo com 10× menos pagamentos, para conferir rápido; **não** use para números publicados | `results/<nome>/` |
| `loopsettle bound <arquivo.csv> [--delays 1,10,60] [--out PASTA] [--report] [--per-participant]` | UB, LB e o teto de economia de qualquer mecanismo com atraso máximo d (segundos) nos seus pagamentos | `loopsettle-out/<arquivo>/` |
| `loopsettle settle <arquivo.csv> [--engines ...] [--delays ...] [--out PASTA] [--report] [--per-participant]` | o mesmo, mais a liquidação em cada desenho | `loopsettle-out/<arquivo>/` |
| `loopsettle export <experimento> <saida.csv> [--payments N] [--participants N]` | grava os pagamentos sintéticos de um cenário num CSV no formato que `bound`/`settle` leem | o CSV |
| `loopsettle topology <transferencias.csv>` | escada de circularidade (Hodge, CDFD, ciclos) de qualquer lista `from,to,amount` | `results/topology-<arquivo>/` |
| `python3 -I scripts/summarize.py` | gera os resumos a partir de `results/` | `results/SUMMARY.md` e `SUMMARY.pt-BR.md` |
| `python3 -I data/fetch_pix.py --month 202609 --keys-date 2026-09-30` | baixa de novo os agregados públicos do Pix (API Olinda do BCB) e recalcula a calibração | `data/raw/`, `data/calibration/pix.toml` |
| `node scripts/wasm-smoke.mjs` | confere o WebAssembly no Node (depois de `npm run wasm`) | terminal |
| `make all` / `make rebuild` | tudo / tudo apagando `results/` antes | `results/`, `web/dist/` |
| `make test`, `make experiments`, `make summary`, `make web`, `make wasm`, `make data` | partes do `make all` | — |

**Desenhos de liquidação** (`--engines`, separados por vírgula): `gross` (bruto em tempo real, como
o Pix); `deferred:N` (compensação multilateral N vezes por dia); `micro:all:J:D` ou
`micro:shortfall:J:D` (micro-ciclos atômicos com janela de J ms e atraso máximo de D ms; `all` põe
todo pagamento na fila, `shortfall` só quando falta saldo); um quinto número opcional limita o
comprimento do ciclo (padrão 6), por exemplo `micro:all:1000:10000:4`.

**Formato do arquivo de pagamentos** (`bound`, `settle`): CSV com cabeçalho e as colunas `time`
(segundos, `HH:MM:SS[.fff]`) ou `t_ms` (milissegundos), `from`, `to` e `amount` (em unidades da
moeda, convertidas para centavos sem ponto flutuante), ordenado pelo horário. Exemplo pronto:
[`data/samples/payments_example.csv`](data/samples/payments_example.csv).

#### Configuração

- **Cenários e experimentos** ficam em [`experiments/*.toml`](experiments): semente, número de
  participantes, pagamentos por dia, motores, varreduras. Edite o arquivo e rode o experimento de
  novo; o resultado vai para `results/<nome>/`. Para voltar ao original:
  `git checkout experiments/<nome>.toml`. Os números publicados usam os arquivos como estão no
  repositório.
- **Calibração** com dados públicos: [`data/calibration/pix.toml`](data/calibration/pix.toml)
  (gerado por `data/fetch_pix.py`; a origem de cada parâmetro está em
  [`docs/PARAMETERS.pt-BR.md`](docs/PARAMETERS.pt-BR.md)).
- **Compilação leve:** [`.cargo/config.toml`](.cargo/config.toml) limita a 2 tarefas paralelas
  (máquina de referência com 2 núcleos e 4 GB). Com mais núcleos e memória, pode aumentar `jobs`.
- **Editor:** com o rust-analyzer no VS Code e 4 GB de RAM, desligue a checagem ao salvar:
  `"rust-analyzer.checkOnSave": false` nas configurações.
- **`make`:** as variáveis `CARGO` e `PYTHON` trocam os programas usados, por exemplo
  `make all PYTHON=python3`.

#### Problemas comuns

| Sintoma | Causa provável | Solução |
|---|---|---|
| `cargo: command not found` / `'cargo' não é reconhecido` | o terminal foi aberto antes de instalar o Rust | feche e abra o terminal; no Linux/macOS, `source "$HOME/.cargo/env"`; no Git Bash, `export PATH="$HOME/.cargo/bin:$PATH"` |
| Windows: `linker 'link.exe' not found` | falta o Visual Studio Build Tools com C++ | rode de novo o comando do Build Tools do passo 1 (ou o instalador, carga "Desenvolvimento para desktop com C++") |
| Linux: `linker 'cc' not found` | falta o compilador C | `sudo apt install -y build-essential` |
| `can't find crate for 'core'` ao rodar `npm run wasm` | falta o target wasm | `rustup target add wasm32-unknown-unknown` |
| `wasm-pack: command not found` | `wasm-pack` não instalado ou terminal antigo | passo 2; feche e abra o terminal |
| `npm install -g` pede permissão (Linux/macOS) | Node do sistema instalado como root | use o Node do `nvm` (passo 1) ou `cargo install wasm-pack --locked` |
| `cannot read experiments/<nome>.toml` | o comando foi rodado fora da raiz | `cd` para a pasta `loopsettle` |
| `make: python: No such file or directory` | no Linux/macOS o Python se chama `python3` | `make all PYTHON=python3` |
| A página da interface fica em branco ou acusa falha ao carregar o `.wasm` | o WebAssembly ainda não foi gerado | em `web/`, rode `npm run wasm` antes de `npm run dev` |
| `Port 5173 is in use` | outro programa usa a porta | o Vite escolhe outra porta; use o endereço que aparece no terminal |
| O computador trava ou a compilação morre sem mensagem | falta de memória (4 GB) | feche o navegador e o editor; compile com uma tarefa só: `cargo build --release -j 1`; no Linux, ative o swap do passo 1 |
| `requires rustc 1.xx or newer` | Rust antigo | `rustup update stable` |
| A primeira compilação demora muitos minutos | normal: dependências e otimização | as próximas são incrementais e levam segundos |
| O `code_version` dos resultados termina em `-dirty` | há mudanças não commitadas fora de `results/` | é só um aviso; para números publicáveis, faça commit e rode de novo |
| Windows: compilação muito lenta | antivírus examinando cada arquivo gerado | opcional: adicione a pasta do projeto às exclusões do Windows Defender |


### Rode nos seus dados

Bancos centrais, bancos e fintechs podem rodar o LoopSettle nos **próprios pagamentos**, sem que
os dados saiam da máquina: o arquivo é lido em streaming e a saída tem só agregados.

```bash
cargo run --release -p loopsettle-cli -- bound  pagamentos.csv   # UB, LB e o teto de qualquer mecanismo com atraso d
cargo run --release -p loopsettle-cli -- settle pagamentos.csv   # + economia de cada desenho (bruto, micro-ciclos, compensação)
```

O arquivo é um CSV com as colunas `time` (segundos, `HH:MM:SS` ou `t_ms` em milissegundos),
`from`, `to` e `amount` (em unidades da moeda, convertidas em centavos sem ponto flutuante),
ordenado pelo horário. Para testar sem dados reais: `loopsettle export baseline exemplo.csv`, ou o
arquivo pronto [`data/samples/payments_example.csv`](data/samples/payments_example.csv). Detalhes em
[`data/README.md`](data/README.md#português).

### Reprodutibilidade

Cada número publicado vem de um comando com semente fixa. Rodar de novo reproduz os resultados
**byte a byte**; a semente e a versão do código ficam registradas em cada `summary.json`.

| Experimento | Pergunta | Saída |
|---|---|---|
| `calibration` | o gerador reproduz os agregados públicos do Pix? | `results/calibration/` |
| `baseline` | quanto é circular, pelas três lentes? | `results/baseline/` |
| `potential-vs-realizable` | quanto da compensação potencial vira liquidez? | `results/potential-vs-realizable/` |
| `microcycle-sweep` | qual a troca entre atraso e liquidez? | `results/microcycle-sweep/` |
| `stress` | e se um participante cair por 30 min — importa quem e quando? | `results/stress/` |
| `finland-replication` | a lógica do Banco da Finlândia aplicada ao cenário | `results/finland-replication/` |
| `robustness` | o resultado sobrevive a sementes, escala, viés e algoritmo? | `results/robustness/` |
| `bias-calibration` | qual viés recebedor reproduz os dados reais do WP 619? | `results/bias-calibration/` |
| `heterogeneity` | e se cada tipo de banco tiver seu próprio perfil ao longo do dia? | `results/heterogeneity/` |
| `funding-windows` | quanto é preciso pré-financiar para as noites e o fim de semana, com o STR fechado? | `results/funding-windows/` |
| `regimes` | o teto muda entre varejo e sistemas de alto valor? | `results/regimes/` |
| `liquidity-frontier` | com a liquidez que os participantes têm, quanto é rejeitado ou atrasa em cada desenho? | `results/liquidity-frontier/` |

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
| [`loopsettle-wasm`](crates/loopsettle-wasm) + [`web/`](web) | interface Vite + React + Canvas 2D, simulação num Web Worker, português e inglês |
| [`data/`](data) | download dos agregados públicos do Pix (API Olinda do BCB) |
| [`docs/`](docs) | metodologia, parâmetros, limitações, afirmações, novidade, referências (arquivos `*.pt-BR.md`) |

O núcleo não tem dependências externas: álgebra linear esparsa, gerador aleatório e formatos de
arquivo foram implementados no próprio projeto.

### Documentação

| Documento | Conteúdo |
|---|---|
| [`docs/INSTANT_PAYMENTS.pt-BR.md`](docs/INSTANT_PAYMENTS.pt-BR.md) | pagamentos instantâneos pelo mundo: modelo de liquidação de cada sistema e motor correspondente |
| [`docs/ASSESSMENT.pt-BR.md`](docs/ASSESSMENT.pt-BR.md) | avaliação crítica: o resultado se sustenta? ameaças, veredito e critérios de morte |
| [`docs/PAPER_READINESS.pt-BR.md`](docs/PAPER_READINESS.pt-BR.md) | especificação de tudo o que falta antes de escrever o artigo |
| [`docs/METHODOLOGY.pt-BR.md`](docs/METHODOLOGY.pt-BR.md) | cada métrica em linguagem simples e formal, com exemplo à mão |
| [`docs/PARAMETERS.pt-BR.md`](docs/PARAMETERS.pt-BR.md) | cada parâmetro do gerador: `SOURCED`, `CALIBRATED` ou `ASSUMPTION`; tabela real × simulado |
| [`docs/LIMITATIONS.pt-BR.md`](docs/LIMITATIONS.pt-BR.md) | limitações conhecidas |
| [`docs/CLAIMS.pt-BR.md`](docs/CLAIMS.pt-BR.md) | o que o estudo pode e não pode afirmar, com a evidência |
| [`docs/NOVELTY.pt-BR.md`](docs/NOVELTY.pt-BR.md) | comparação ponto a ponto com os trabalhos de referência |
| [`docs/REFERENCES.pt-BR.md`](docs/REFERENCES.pt-BR.md) | referências e situação de verificação de cada uma |
| [`docs/references/`](docs/references/README.pt-BR.md) | biblioteca: os PDFs citados, com link original, e `references.bib` |
| [`docs/PAPER_OUTLINE.pt-BR.md`](docs/PAPER_OUTLINE.pt-BR.md) | esqueleto do artigo, com figuras e tabelas |
| [`DECISIONS.pt-BR.md`](DECISIONS.pt-BR.md) | decisões de desenho: alternativas, motivo e como reverter |
| [`REVIEW.pt-BR.md`](REVIEW.pt-BR.md) | pontos abertos para a revisão antes do artigo |
| [`results/SUMMARY.pt-BR.md`](results/SUMMARY.pt-BR.md) | todos os números, gerados por comando |
| [`data/README.md`](data/README.md) | dados públicos e como baixá-los |

Todos os documentos existem nos dois idiomas: o nome sem sufixo é a versão em inglês e `*.pt-BR.md` é a versão em português (link no topo de cada arquivo).

### Limitações

O LoopSettle é explícito sobre o que não sabe:

- **Dados sintéticos calibrados, não dados interbancários reais.** Só agregados públicos são
  usados. Quem paga a quem não é público e é a principal hipótese do modelo; por isso há uma
  análise de sensibilidade ao desequilíbrio entre recebedores (σ de 0 a 0,8).
- **Escala reduzida:** 200 participantes e 1 milhão de pagamentos por dia, contra cerca de 239
  milhões no Pix real. As frações são o resultado; os valores absolutos não escalam linearmente.
- **Micro-ciclos heurísticos:** a busca de ciclos é simples, não um otimizador completo.
- **Horizonte de um dia** e perfil horário sem rajadas. No estresse, quedas de 30 minutos com
  reenvio total quase não mudam a liquidez necessária; abandono parcial não é modelado.

Lista completa em [`docs/LIMITATIONS.pt-BR.md`](docs/LIMITATIONS.pt-BR.md); hipóteses e fonte de cada
parâmetro em [`docs/PARAMETERS.pt-BR.md`](docs/PARAMETERS.pt-BR.md); pontos abertos para revisão em
[`REVIEW.pt-BR.md`](REVIEW.pt-BR.md).

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
- McMahon, C. et al. (2024). *Improving the efficiency of payments systems using quantum
  computing*. Management Science (arXiv:2209.15392). **O trabalho mais próximo**, em sistema de
  alto valor.

Situação de verificação de cada referência em [`docs/REFERENCES.pt-BR.md`](docs/REFERENCES.pt-BR.md);
os PDFs estão em [`docs/references/`](docs/references/README.pt-BR.md).

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

Pix é marca do Banco Central do Brasil. O LoopSettle é um projeto independente, sem afiliação
nem endosso do Banco Central do Brasil ou de qualquer outro operador citado; os nomes dos
sistemas são usados só para identificá-los.

---

## English

<p align="center">
  <img src="docs/img/en/ui-overview.png" alt="LoopSettle interface: network of participants with the circular component in green, micro-cycle controls and liquidity panels" width="100%">
</p>
<p align="center"><sub>The interface runs entirely in the browser (Rust → WebAssembly, in a Web Worker): 40 participants and 100,000 payments simulated in 0.5 s. A button in the corner switches to Portuguese.</sub></p>

### In one sentence

In instant payment systems with gross settlement — Pix (Brazil), FedNow (US), TIPS (euro area),
NPP (Australia) — every bank must hold cash before every transfer. Part of that cash only goes
around in loops (A pays B, B pays C, C pays A). LoopSettle measures how large that part is, how much of
it could be netted and, above all, **how much of that netting turns into liquidity actually
saved** once the order in which payments arrive is respected.

### The problem

More and more countries run instant payments with **real-time gross settlement**: each payment
settles on its own, immediately, and the payer's bank must hold funds at the central bank before
every transfer. Older systems let banks batch payments and offset inflows against outflows
(*netting*), saving liquidity. In Brazil, where Pix has worked this way since 2020, the Banco
Central do Brasil study (Working Paper 619) shows that banks lost this ability to delay and net
payments and now hold more liquid assets. The Bank of Finland asked the same question for Europe.

LoopSettle uses **Pix as the calibrated case**, because it publishes the aggregates the model
needs, but its engines and metrics apply to any system of the same kind. See
[where your system fits](#where-your-system-fits).

One question remained open: **how much of the lost netting translates into liquidity that could
actually be freed?** And can part of it be recovered without giving up the instant user
experience?

> **Research question.** In an instant payment system with gross settlement — with Pix as the
> calibrated case — how much of the liquidity participants must hold is circular, how much of it
> is recoverable in practice, and what delay does recovering it with atomic micro-cycles cost?

### Key finding

<p align="center">
  <img src="results/potential-vs-realizable/fig_potential_vs_realizable.svg" alt="Liquidity saving as a function of the maximum wait" width="720">
</p>

In a synthetic scenario calibrated to public Pix aggregates (200 participants; 10 seeds; 1 to 30
million payments per day):

- **85% of payment volume** is nettable under the potential measure (same-day inflows vs
  outflows, ignoring time order).
- **Atomic micro-cycles of up to 10 s settle up to 63% of value inside cycles, yet save about
  0.01% of liquidity** (0.0116%, 95% CI [0.008; 0.015]%), whatever the maximum cycle length,
  and the saving goes to zero as volume grows.
- **There is a provable ceiling:** a mechanism that holds each payment for at most *d* never
  needs less liquidity than each participant's **liquidity floor**: the largest value of
  "everything it has paid by some instant, minus everything it has received by *d* later". For
  10 s, over 10 seeds, that ceiling on the saving ranges from 0.014% of the liquidity (structure
  calibrated to WP 619) to 0.14% (no receiver bias), and netting every 10 s realises 38% to 46%
  of it. A simpler, looser form of the same bound (the value paid in the *d* seconds before the
  peak) gives 0.7% to 3.3%.
- Netting over longer horizons saves more (0.5% every 15 minutes; 7.8% once a day, at 1 million
  payments), **but that "potential" shrinks with volume**. With the bilateral structure calibrated
  to real Pix data (BCB WP 619) and 30 million payments a day, one daily netting recovers **between
  0.2% and 2.8%** of the liquidity, depending on how differently banks behave during the day — the
  question only per-participant data can answer ([assessment](docs/ASSESSMENT.md)).
- **The real constraint is the weekend.** The Conta PI can only be topped up while the STR is open
  (business days, 6:30–18:30), so banks prefund the nights and the 59.5 weekend hours — about 2.7×
  a day's peak. Even there, netting at the end of the weekend would save only 0.4–1.0%: the need
  comes from accumulated net imbalance, not from circular flow.

> **Nettable volume is not saved liquidity.** LoopSettle makes the gap between the two visible
> and measurable.

All numbers are in [`results/SUMMARY.md`](results/SUMMARY.md), with the seed, code version
and command that produced them. What the study can and cannot claim is in
[`docs/CLAIMS.md`](docs/CLAIMS.md).

### The interface

<table>
  <tr>
    <td width="58%"><img src="docs/img/en/ui-network.png" alt="The network of one day: participants on a circle, green edges are the maximum circular part (CDFD) and grey edges the directional part"></td>
    <td width="42%"><img src="docs/img/en/ui-potential-vs-realizable.png" alt="Potential vs realizable panel: 82% potential, 78% CDFD, 75% cycles, 11% liquidity with one daily netting and about zero with micro-cycles"></td>
  </tr>
  <tr>
    <td align="center"><sub><b>The network of one day.</b> Circle size = activity; dashed = net payer. Green is the largest possible circulation (CDFD); grey is flow that only goes one way.</sub></td>
    <td align="center"><sub><b>Potential vs realizable.</b> The ladder of measures, from netting on paper down to liquidity actually saved.</sub></td>
  </tr>
  <tr>
    <td><img src="docs/img/en/ui-day.png" alt="The day in 15-minute steps: value paid per interval and the sum of accumulated debit positions"></td>
    <td><img src="docs/img/en/ui-liquidity.png" alt="Liquidity needed by design: gross, daily netting, every 15 minutes, micro-cycles and theoretical minimum"></td>
  </tr>
  <tr>
    <td align="center"><sub><b>The day in 15-minute steps.</b> The sum of debit positions (dark line) builds up over hours: that is what requires liquidity, and second-scale micro-cycles cannot reach it.</sub></td>
    <td align="center"><sub><b>Liquidity needed by design</b>, from real-time gross to the theoretical minimum (LB).</sub></td>
  </tr>
  <tr>
    <td><img src="docs/img/en/ui-sweep.png" alt="Micro-cycle liquidity saving for each search window"></td>
    <td><img src="docs/img/en/ui-constrained.png" alt="Constrained liquidity table: value rejected, p95 delay and share settled within 2.8 s"></td>
  </tr>
  <tr>
    <td align="center"><sub><b>Saving × window.</b> Micro-cycle saving for each cycle search window: always close to zero.</sub></td>
    <td align="center"><sub><b>Constrained liquidity.</b> With less than the required opening balance: value rejected and the 2.8 s target.</sub></td>
  </tr>
</table>

<table>
  <tr>
    <td width="34%"><img src="docs/img/en/ui-mobile.png" alt="Interface on a phone in reduced mode"></td>
    <td>

**What you can do on screen**

- Move the **cycle search window** (0.5 to 10 s) and the **maximum tolerated delay** (2 to 10 s)
  and watch the liquidity saving and delays recompute.
- Change the number of participants and payments and the **receiver imbalance** (the model's
  main assumption) to test how robust the result is.
- Test **constrained liquidity**: with less than the required opening balance, how much value
  each design rejects and how many payments meet the Pix target (50% within 2.8 s).
- Change the **seed**: the same seed always gives the same result.
- Switch the **language** (Portuguese/English) with the button in the corner, or open the page
  with `?lang=en` or `?lang=pt`.

**On phones** the interface switches to a reduced mode (up to 25 participants and 30,000
payments) with the same model.

Browser scenarios are smaller than the experiments (up to 50 participants and 100,000 payments),
so on-screen numbers are not the paper's numbers. In small scenarios the micro-cycle saving can
even be slightly **negative** (−0.001%): the queue changes the settlement order and a
participant's peak can rise slightly. This is a model result, consistent with the main finding
(saving close to zero), not a bug.

</td>
  </tr>
</table>

```bash
cd web && npm install && npm run wasm && npm run dev    # opens at http://localhost:5173
```

Screenshots in both languages are regenerated with `node scripts/screenshots.cjs` while the UI is
running.

### How it works

```mermaid
flowchart LR
  A["Public data<br/>(calibrated case: Pix,<br/>BCB Olinda API)"] --> B["Calibrated synthetic<br/>generator, streaming"]
  B --> C["Settlement engines"]
  C --> C1["Real-time gross<br/>(Pix, FedNow, TIPS, NPP)"]
  C --> C2["Deferred netting<br/>1× a day … every 10 s<br/>(UPI, Faster Payments)"]
  C --> C3["Atomic micro-cycles<br/>(new mechanism)"]
  B --> D["Flow topology"]
  D --> D1["Hodge<br/>(diagnostic)"]
  D --> D2["Maximum CDFD<br/>(static ceiling)"]
  D --> D3["Cycle netting<br/>(executable)"]
  C1 & C2 & C3 --> E["Liquidity, delays and<br/>potential vs realizable"]
  D1 & D2 & D3 --> E
```

**Three lenses on circular flow**

| Lens | What it measures | Why it matters |
|---|---|---|
| **Hodge decomposition** | splits flow into a directional (gradient) and a circular component | good diagnostic, but it can assign more to cycles than exists: in the triangle A→B 10, B→C 5, C→A 5 it gives 6.67 to the cycle while only 5 is nettable |
| **Maximum CDFD** | largest circulation that respects each edge's value and flow conservation at each node | exact, realistic static ceiling (hand-written min-cost flow, checked against an LP solver on 1,000 random graphs) |
| **Cycle netting** | what a Bech–Soramäki-style netting algorithm can cancel | what is executable in practice |

**Three settlement designs side by side**

| Engine | Behaviour |
|---|---|
| `GrossRealTime` | settles each payment on arrival if funds suffice (the model of Pix, FedNow, TIPS and NPP) |
| `DeferredNet` | multilateral netting at fixed cycles, from once a day down to every 10 s (the model of UPI and Faster Payments; the Finnish study's logic) |
| `AtomicMicroCycle` | waits a few seconds, finds cycles in the queue and settles each cycle at once, all or nothing; modes `QueueAll` (everything goes through the queue) and `QueueOnShortfall` (only on shortfall, LSM style) |

**How liquidity is measured**

- **UB (upper bound):** the smallest opening balance that avoids any rejection under real-time
  gross settlement = the intraday maximum of cumulative outflows − cumulative inflows.
- **LB (lower bound):** the end-of-day multilateral net position, which a single daily netting
  would require.
- **Saving of a design** = 1 − design liquidity ÷ UB.

Formal definitions, a worked example and proofs in [`docs/METHODOLOGY.md`](docs/METHODOLOGY.md).

### Where your system fits

Pix is the calibrated case, but the question applies to any country. To customers every system
below is instant; what changes liquidity is **how banks settle with each other**, and LoopSettle
has an engine for each model:

<table>
  <tr>
    <td width="40%"><img src="docs/img/en/ui-fit.png" alt="Interface panel: where your system fits, with settlement models and example countries"></td>
    <td>

| Interbank model | Examples | Engine |
|---|---|---|
| Real-time gross | Pix (Brazil), FedNow (US), TIPS (euro area), NPP (Australia) | `GrossRealTime` |
| Gross, prefunded account | RTP (US), Real-Time Rail (Canada, launching) | `GrossRealTime` with limited balance |
| Deferred netting in cycles | UPI (India, 10/day), Faster Payments (UK, 3/day), FAST (Singapore, 2/day), Interac e-Transfer (Canada, 1/day) | `DeferredNet` |
| Netting every few seconds | SPEI (Mexico, every 1.9 s on average) | `AtomicMicroCycle` |

</td>
  </tr>
</table>

- **In the US**, FedNow works like Pix: the central result applies directly to its design.
- **In Canada**, Interac e-Transfer is instant for the customer, but banks settle the net amount once
  a day through Lynx (the `DeferredNet` model). The new Real-Time Rail, due to launch in late 2026,
  settles in real time against prefunded accounts at the Bank of Canada: exactly the kind of
  system whose migration cost the Finnish study's logic measures.
- **In Mexico**, SPEI already runs a clearing algorithm every 1.9 s on average: the closest
  real-world analogue of atomic micro-cycles.
- **The numbers** (0.01%, 0.5%, 7.8%) come from the scenario calibrated to Pix and do not carry
  over automatically to another country: re-calibrating requires that country's public aggregates.

Full table, with operator, start date, settlement model and source for each system, in
[`docs/INSTANT_PAYMENTS.md`](docs/INSTANT_PAYMENTS.md).

### Results at a glance

| Measure | Value |
|---|---|
| Calibration error against public Pix aggregates | ≤ 1.5% on all 9 aggregates |
| Real-time gross liquidity (UB) / gross volume | 16.1% |
| One daily netting (LB) / UB | 0.91 |
| Circularity ladder (gross graph): Hodge L1 → CDFD → cycles | 1.18 → 0.83 → 0.80 |
| Circularity ladder (net graph): Hodge L2 → CDFD → cycles | 0.98 → 0.17 → 0.14 |
| Instant vs 1 cycle per day (Finnish study logic) | 1.10× (σ = 0.4) to 1.50× (σ = 0) the liquidity |
| Instant vs 6 cycles per day (the STEP2 baseline) | 1.04× to 1.14× |
| Micro-cycles with 2 s wait: Pix target of 50% within 2.8 s | met (100%) |
| Micro-cycles with 10 s wait: same target | missed (43%) |
| Saving of micro-cycles up to 10 s † | 0.0116% of UB, 95% CI [0.008; 0.015]% |
| Ceiling for any mechanism up to 10 s † | 0.80% of UB (0.27% at 30 million payments) |
| Saving of one daily netting † | 7.8%, 95% CI [4.5; 11.1]% (1.7% at 30 million) |

<sub>Unmarked rows: seed `20260930`, 1 million payments per day. † 10 seeds, σ = 0.4 (`results/robustness/`).</sub>

<table>
  <tr>
    <td width="50%"><img src="results/microcycle-sweep/fig_tradeoff.svg" alt="Micro-cycles: liquidity saving versus delay"></td>
    <td width="50%"><img src="results/finland-replication/fig_finland.svg" alt="Instant liquidity versus cycles per day"></td>
  </tr>
  <tr>
    <td align="center"><sub>Micro-cycles: saving × delay</sub></td>
    <td align="center"><sub>Instant × netting cycles per day</sub></td>
  </tr>
</table>

<p align="center">
  <img src="results/baseline/fig_ladder.svg" alt="Circularity ladder: Hodge, maximum CDFD and cycle netting on the gross and net graphs" width="720">
</p>
<p align="center"><sub>The circularity ladder: the diagnostic (Hodge) assigns to cycles far more than the exact ceiling (CDFD) and than a netting algorithm can cancel.</sub></p>

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
| **System** | STEP2, retail, Finland | Instant payments with gross settlement; calibrated case: Pix, Brazil (24/7, central bank) |
| **Data** | Synthetic, mimicking STEP2 flows | Synthetic, calibrated to public Pix aggregates (error ≤ 1.5%); every parameter sourced or flagged `ASSUMPTION` |
| **Designs compared** | Fixed cycles × instant | Fixed cycles (1 to 8,640 per day) × instant × **atomic micro-cycles** (new) |
| **Liquidity definition** | Smallest balance with no rejection: minimum position over the day (instant) and minimum of end-of-cycle positions (cycles) | The same (UB and LB), confirmed in their text |
| **Aggregate result** | Against STEP2's 6 daily cycles: +2.7% on average and less than 8.6% on 95% of days. Against 1 daily cycle: 1.45× the liquidity[^bof-en] | Against 6 cycles: +3.9% (σ = 0.4) to +13.7% (σ = 0). Against 1 cycle: 1.10× to 1.50×: **same order of magnitude** |
| **Heterogeneity** | Large differences across banks and days; recommends liquidity buffers | Measured per participant (`results/baseline/participants.csv`); receiver imbalance moves the ratio from 1.10× to 1.50× |
| **Circularity** | Not measured | Three lenses (Hodge, CDFD, cycles) and the ladder between them |
| **Potential vs realizable** | Not separated | Separated: it is the central result |
| **Code** | Not found publicly (to be checked) | Open (Apache-2.0), byte-for-byte reproducible, runs on a 4 GB i3 and in the browser |

**Where both studies agree:** moving to instant settlement costs a few percentage points of
aggregate liquidity when the previous system already ran several cycles a day. **What LoopSettle
adds:** the lost liquidity sits almost entirely on hour-long horizons, so second-scale netting
does not bring it back, however much value micro-cycles settle inside cycles.

> This is **not a replication** of the Finnish study: we apply the same logic to a different
> scenario and compare orders of magnitude (see [`docs/CLAIMS.md`](docs/CLAIMS.md),
> "cannot claim" item 5).

[^bof-en]: Hellqvist & Korpinen (2021), pp. 2 and 9–11: 1,000 synthetic days for the average
    increase and the 95th percentile; 30 days for the comparison with 1 cycle (€231M vs €335M).
    See [`docs/REFERENCES.md`](docs/REFERENCES.md).

#### Other works

| Work | What it did | What LoopSettle does with it |
|---|---|---|
| Gonzalez, Ma & Zeng, **BCB WP 619** (2025) | Using real Pix data, shows that banks lost the ability to delay and net payments and now hold more liquid assets | Reproduces the **potential** netting measure (time order ignored) on synthetic data and puts it next to the **realizable** saving over time |
| Homs-Dones, MacKay, Sansom & Zhou, **CDFD** (arXiv 2506.12546, 2025) | Decomposes network flow into circular and directional parts, respecting conservation and edge bounds | Implements the **maximum circularity** decomposition exactly (min-cost flow) as a static ceiling; the proportional (BFF) variant is future work |
| Bech & Soramäki, **BoF DP 9/2001** | Cycle-based gridlock resolution in interbank systems | Cycle netting and the basis for atomic micro-cycles, now with a maximum delay and the Pix time target |

Full point-by-point comparison in [`docs/NOVELTY.md`](docs/NOVELTY.md).

### Quick start

```bash
cargo test --workspace                                                       # invariants and tests
cargo run --release -p loopsettle-cli -- experiment potential-vs-realizable  # central comparison
cd web && npm install && npm run wasm && npm run dev                         # interface
```

First time? Install the tools with the [Installation from scratch](#installation-from-scratch)
guide just below. Everything at once: `make rebuild`. Runs on a 4th-generation Intel i3 with 4 GB
of RAM: each experiment uses under 20 MiB and finishes in seconds.

### Installation from scratch

This guide starts from a computer with nothing installed and ends with the simulator, the
experiments and the interface running. The first time takes 20 to 40 minutes, mostly downloads
and the first compilation. You need an internet connection and about 3 GB of free disk space. It
was written for the project's reference machine (4th-generation Intel i3, 4 GB of RAM).

**What will be installed**

| Tool | What for | Required? | Versions tested |
|---|---|---|---|
| Git | download the code | yes | any recent |
| Rust (rustup + cargo) | simulator, tests and experiments | yes | stable 1.97–1.99 |
| The system's C compiler/linker | Rust uses it to produce the executable | yes | MSVC Build Tools (Windows), `build-essential` (Linux), Xcode CLT (macOS) |
| Target `wasm32-unknown-unknown` | compile the simulator for the browser | interface only | — |
| `wasm-pack` | package the WebAssembly | interface only | 0.13.1 and 0.15.1 |
| Node.js LTS (with npm) | web interface | interface only | 22 and 24 |
| Python 3 | generate `results/SUMMARY*.md` and download the public Pix data | optional | 3.10 or newer (standard library only) |
| GNU Make | rebuild everything with one command | optional | 4.x |

Only want the simulator and the experiments? Git, Rust and the system compiler are enough (steps
1 and 2 for your system and the common step 3). The interface also needs the wasm target,
`wasm-pack` and Node.

#### Step 1 — system tools

<details open>
<summary><b>Windows 10/11</b> (use the native toolchain; do <b>not</b> use WSL2 with 4 GB of RAM)</summary>

Open **PowerShell** (Start menu → "PowerShell") and run one command at a time. `winget` ships with
an up-to-date Windows 10/11.

```powershell
winget install --id Git.Git -e
```

```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

```powershell
winget install --id Rustlang.Rustup -e
```

```powershell
winget install --id OpenJS.NodeJS.LTS -e
```

Optional (results summary and `make`):

```powershell
winget install --id Python.Python.3.12 -e
```

```powershell
winget install --id ezwinports.make -e
```

**Close and reopen the terminal** after installing, so that `PATH` is updated. Without `winget`:
download Git from [git-scm.com](https://git-scm.com/download/win), Rust from
[rustup.rs](https://rustup.rs) (the installer offers to install the Visual Studio Build Tools;
accept, with the "Desktop development with C++" workload), Node LTS from
[nodejs.org](https://nodejs.org) and Python from [python.org](https://www.python.org/downloads/)
(tick "Add python.exe to PATH").

The commands in the next steps work in PowerShell and in **Git Bash** (installed with Git). Where
they differ, both are shown.
</details>

<details>
<summary><b>Linux</b> (Ubuntu/Debian; Fedora at the end)</summary>

```bash
sudo apt update && sudo apt install -y build-essential git curl python3 make
```

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

```bash
source "$HOME/.cargo/env"
```

Node LTS through `nvm` (no `sudo`, leaves the system Node alone):

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
```

```bash
source "$HOME/.nvm/nvm.sh" && nvm install --lts
```

With 4 GB of RAM, enable 2 GB of swap before compiling (keeps the computer from freezing):

```bash
sudo fallocate -l 2G /swapfile && sudo chmod 600 /swapfile && sudo mkswap /swapfile && sudo swapon /swapfile
```

Fedora: replace the first line with `sudo dnf install -y gcc git make python3 curl`.
</details>

<details>
<summary><b>macOS</b></summary>

```bash
xcode-select --install
```

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

```bash
source "$HOME/.cargo/env"
```

Node LTS: install from [nodejs.org](https://nodejs.org) or, with Homebrew, `brew install node@22`.
Git, `make` and Python 3 come with the Xcode Command Line Tools.
</details>

#### Step 2 — Rust ready for the interface (all systems)

```bash
rustup default stable
```

```bash
rustup target add wasm32-unknown-unknown
```

```bash
npm install -g wasm-pack
```

`npm` downloads a prebuilt `wasm-pack`. If that fails (a corporate network, for instance), build it
with Cargo — 5 to 15 minutes on an i3:

```bash
cargo install wasm-pack --locked
```

#### Step 3 — get the code and check

```bash
git clone https://github.com/fabiodrneles/loopsettle.git
```

```bash
cd loopsettle
```

Check the versions (each command should print a number, not an error):

```bash
git --version && cargo --version && rustup target list --installed && wasm-pack --version && node --version && npm --version
```

Every command from here on runs **at the repository root** (the `loopsettle` folder).

#### Step 4 — first run

1. **Tests** (the first time downloads the dependencies and compiles; about 1 to 2 minutes on the
   i3):

   ```bash
   cargo test --workspace
   ```

   Expected: several `test result: ok.` lines and no `FAILED`.

2. **List of experiments:**

   ```bash
   cargo run --release -p loopsettle-cli -- list
   ```

3. **The central comparison** (writes CSV, JSON and SVG to `results/potential-vs-realizable/`):

   ```bash
   cargo run --release -p loopsettle-cli -- experiment potential-vs-realizable
   ```

4. **The interface** (in a second terminal, also at the repository root):

   ```bash
   cd web
   ```

   ```bash
   npm install
   ```

   ```bash
   npm run wasm
   ```

   ```bash
   npm run dev
   ```

   Open the address shown in the terminal (usually <http://localhost:5173>). To stop: `Ctrl+C`.
   Optimised build: `npm run build`, then `npm run preview` (usually <http://localhost:4173>).
   Whenever the Rust code changes, run `npm run wasm` again.

5. **Everything from scratch** (tests, every experiment, the summaries and the interface; about 2
   minutes on the i3 after the first compilation):

   ```bash
   make rebuild PYTHON=python3
   ```

   On Windows, use `make rebuild` (there Python is called `python`). Without `make`, the equivalent
   in Git Bash, Linux or macOS is:

   ```bash
   cargo test --workspace && for e in $(cargo run --release -q -p loopsettle-cli -- list | cut -d' ' -f1); do cargo run --release -q -p loopsettle-cli -- experiment $e || break; done && cargo run --release -q -p loopsettle-cli -- topology data/samples/triangle.csv && python3 -I scripts/summarize.py
   ```

   And in PowerShell:

   ```powershell
   cargo test --workspace; cargo run --release -q -p loopsettle-cli -- list | ForEach-Object { cargo run --release -q -p loopsettle-cli -- experiment ($_ -split ' ')[0] }; cargo run --release -q -p loopsettle-cli -- topology data/samples/triangle.csv; python -I scripts/summarize.py
   ```

After `cargo build --release -p loopsettle-cli`, the executable can be called directly, without
`cargo run`: `./target/release/loopsettle list` (Windows: `.\target\release\loopsettle.exe list`).
In the examples below, `loopsettle` means either form.

#### Command reference

| Command | What it does | Output |
|---|---|---|
| `cargo test --workspace` | every test, including the 8 property-tested invariants | terminal |
| `cargo test -p loopsettle-core` | the tests of one crate (`-core`, `-topology`, `-sim`, `-cli`) | terminal |
| `loopsettle list` | lists the experiments and the question each answers | terminal |
| `loopsettle experiment <name>` | runs an experiment with the fixed seed in its TOML | `results/<name>/` (CSV, `summary.json`, `run.json`, SVG) |
| `loopsettle experiment <name> --quick` | the same with 10× fewer payments, for a quick check; **not** for published numbers | `results/<name>/` |
| `loopsettle bound <file.csv> [--delays 1,10,60] [--out DIR] [--report] [--per-participant]` | UB, LB and the saving ceiling of any mechanism with maximum delay d (seconds) on your payments | `loopsettle-out/<file>/` |
| `loopsettle settle <file.csv> [--engines ...] [--delays ...] [--out DIR] [--report] [--per-participant]` | the same, plus settlement under each design | `loopsettle-out/<file>/` |
| `loopsettle export <experiment> <out.csv> [--payments N] [--participants N]` | writes a scenario's synthetic payments to a CSV in the format `bound`/`settle` read | the CSV |
| `loopsettle topology <transfers.csv>` | circularity ladder (Hodge, CDFD, cycles) of any `from,to,amount` list | `results/topology-<file>/` |
| `python3 -I scripts/summarize.py` | builds the summaries from `results/` | `results/SUMMARY.md` and `SUMMARY.pt-BR.md` |
| `python3 -I data/fetch_pix.py --month 202609 --keys-date 2026-09-30` | re-downloads the public Pix aggregates (BCB Olinda API) and recomputes the calibration | `data/raw/`, `data/calibration/pix.toml` |
| `node scripts/wasm-smoke.mjs` | checks the WebAssembly in Node (after `npm run wasm`) | terminal |
| `make all` / `make rebuild` | everything / everything after deleting `results/` | `results/`, `web/dist/` |
| `make test`, `make experiments`, `make summary`, `make web`, `make wasm`, `make data` | parts of `make all` | — |

**Settlement designs** (`--engines`, comma-separated): `gross` (real-time gross, like Pix);
`deferred:N` (multilateral netting N times a day); `micro:all:W:D` or `micro:shortfall:W:D` (atomic
micro-cycles with a W ms window and a maximum delay of D ms; `all` queues every payment,
`shortfall` only when the balance is short); an optional fifth number limits the cycle length
(default 6), e.g. `micro:all:1000:10000:4`.

**Payment file format** (`bound`, `settle`): CSV with a header and the columns `time` (seconds,
`HH:MM:SS[.fff]`) or `t_ms` (milliseconds), `from`, `to` and `amount` (in currency units, converted
to cents without floating point), sorted by time. Ready-made example:
[`data/samples/payments_example.csv`](data/samples/payments_example.csv).

#### Configuration

- **Scenarios and experiments** live in [`experiments/*.toml`](experiments): seed, number of
  participants, payments per day, engines, sweeps. Edit the file and run the experiment again; the
  result goes to `results/<name>/`. To go back to the original:
  `git checkout experiments/<name>.toml`. Published numbers use the files as committed.
- **Calibration** to public data: [`data/calibration/pix.toml`](data/calibration/pix.toml)
  (written by `data/fetch_pix.py`; the source of every parameter is in
  [`docs/PARAMETERS.md`](docs/PARAMETERS.md)).
- **Light compilation:** [`.cargo/config.toml`](.cargo/config.toml) limits the build to 2 parallel
  jobs (reference machine with 2 cores and 4 GB). With more cores and memory, raise `jobs`.
- **Editor:** with rust-analyzer in VS Code and 4 GB of RAM, turn off the check on save:
  `"rust-analyzer.checkOnSave": false` in the settings.
- **`make`:** the variables `CARGO` and `PYTHON` change the programs used, e.g.
  `make all PYTHON=python3`.

#### Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| `cargo: command not found` / `'cargo' is not recognized` | the terminal was opened before Rust was installed | close and reopen it; on Linux/macOS, `source "$HOME/.cargo/env"`; in Git Bash, `export PATH="$HOME/.cargo/bin:$PATH"` |
| Windows: `linker 'link.exe' not found` | the Visual Studio Build Tools with C++ are missing | run the Build Tools command of step 1 again (or the installer, "Desktop development with C++" workload) |
| Linux: `linker 'cc' not found` | no C compiler | `sudo apt install -y build-essential` |
| `can't find crate for 'core'` when running `npm run wasm` | the wasm target is missing | `rustup target add wasm32-unknown-unknown` |
| `wasm-pack: command not found` | `wasm-pack` not installed, or an old terminal | step 2; close and reopen the terminal |
| `npm install -g` asks for permissions (Linux/macOS) | the system Node was installed as root | use Node from `nvm` (step 1) or `cargo install wasm-pack --locked` |
| `cannot read experiments/<name>.toml` | the command ran outside the root | `cd` into the `loopsettle` folder |
| `make: python: No such file or directory` | on Linux/macOS Python is called `python3` | `make all PYTHON=python3` |
| The interface page is blank or fails to load the `.wasm` | the WebAssembly has not been built yet | in `web/`, run `npm run wasm` before `npm run dev` |
| `Port 5173 is in use` | another program uses the port | Vite picks another port; use the address shown in the terminal |
| The computer freezes or the build dies without a message | out of memory (4 GB) | close the browser and the editor; build with a single job: `cargo build --release -j 1`; on Linux, enable the swap of step 1 |
| `requires rustc 1.xx or newer` | old Rust | `rustup update stable` |
| The first build takes many minutes | normal: dependencies and optimisation | later builds are incremental and take seconds |
| The results' `code_version` ends in `-dirty` | there are uncommitted changes outside `results/` | only a warning; for publishable numbers, commit and run again |
| Windows: very slow builds | the antivirus scans every generated file | optional: add the project folder to the Windows Defender exclusions |


### Run it on your own data

Central banks, banks and fintechs can run LoopSettle on **their own payments** without the data
leaving the machine: the file is streamed and the output contains aggregates only.

```bash
cargo run --release -p loopsettle-cli -- bound  payments.csv   # UB, LB and the ceiling for any mechanism with delay d
cargo run --release -p loopsettle-cli -- settle payments.csv   # + the saving of each design (gross, micro-cycles, netting)
```

The file is a CSV with columns `time` (seconds, `HH:MM:SS`, or `t_ms` in milliseconds), `from`,
`to` and `amount` (in currency units, converted to cents without floating point), sorted by time.
To try it without real data: `loopsettle export baseline example.csv`, or the ready-made
[`data/samples/payments_example.csv`](data/samples/payments_example.csv). Details in
[`data/README.md`](data/README.md#english).

### Reproducibility

Every published number comes from a command with a fixed seed. Re-running reproduces the results
**byte for byte**; the seed and code version are recorded in each `summary.json`.

| Experiment | Question | Output |
|---|---|---|
| `calibration` | does the generator reproduce the public Pix aggregates? | `results/calibration/` |
| `baseline` | how much is circular, through the three lenses? | `results/baseline/` |
| `potential-vs-realizable` | how much of the potential netting becomes liquidity? | `results/potential-vs-realizable/` |
| `microcycle-sweep` | what is the trade-off between delay and liquidity? | `results/microcycle-sweep/` |
| `stress` | what if a participant goes down for 30 min — does it matter who and when? | `results/stress/` |
| `finland-replication` | the Bank of Finland logic applied to the scenario | `results/finland-replication/` |
| `robustness` | does the result survive seeds, scale, bias and algorithm? | `results/robustness/` |
| `bias-calibration` | which receiver bias reproduces the real WP 619 data? | `results/bias-calibration/` |
| `heterogeneity` | what if each type of bank has its own intraday profile? | `results/heterogeneity/` |
| `funding-windows` | how much must be prefunded for nights and the weekend, with the STR closed? | `results/funding-windows/` |
| `regimes` | does the ceiling differ between retail and large-value systems? | `results/regimes/` |
| `liquidity-frontier` | with the liquidity participants hold, how much is rejected or delayed under each design? | `results/liquidity-frontier/` |

**Eight invariants checked by property tests** (`proptest`, thousands of random cases on every
run): conservation of money; each participant's net position preserved; no negative balances;
micro-cycle atomicity; netting never reverses a payment nor cancels more than the cycle's
smallest payment; Hodge consistency; full determinism; and the fixed triangle case.

### Architecture

| Component | Contents |
|---|---|
| [`loopsettle-core`](crates/loopsettle-core) | ledger in cents (`i64`, never floating point), settlement engines, streaming metrics |
| [`loopsettle-topology`](crates/loopsettle-topology) | Hodge (hand-written conjugate gradient), exact CDFD, cycle netting |
| [`loopsettle-sim`](crates/loopsettle-sim) | calibrated streaming synthetic generator |
| [`loopsettle-cli`](crates/loopsettle-cli) | reproducible experiments, CSV/JSON/SVG output |
| [`loopsettle-wasm`](crates/loopsettle-wasm) + [`web/`](web) | Vite + React + Canvas 2D UI, simulation in a Web Worker, Portuguese and English |
| [`data/`](data) | download of public Pix aggregates (BCB Olinda API) |
| [`docs/`](docs) | methodology, parameters, limitations, claims, novelty, references (Portuguese twins: `*.pt-BR.md`) |

The core has no external dependencies: sparse linear algebra, the random generator and file
formats are implemented in the project itself. Code and code comments are in English.

### Documentation

| Document | Contents |
|---|---|
| [`docs/INSTANT_PAYMENTS.md`](docs/INSTANT_PAYMENTS.md) | instant payment systems around the world: settlement model of each and the matching engine |
| [`docs/ASSESSMENT.md`](docs/ASSESSMENT.md) | critical assessment: does the result hold? threats, verdict and kill criteria |
| [`docs/PAPER_READINESS.md`](docs/PAPER_READINESS.md) | specification of everything needed before writing the paper |
| [`docs/METHODOLOGY.md`](docs/METHODOLOGY.md) | every metric in plain and formal language, with a worked example |
| [`docs/PARAMETERS.md`](docs/PARAMETERS.md) | every generator parameter: `SOURCED`, `CALIBRATED` or `ASSUMPTION`; real vs simulated table |
| [`docs/LIMITATIONS.md`](docs/LIMITATIONS.md) | known limitations |
| [`docs/CLAIMS.md`](docs/CLAIMS.md) | what the study can and cannot claim, with the evidence |
| [`docs/NOVELTY.md`](docs/NOVELTY.md) | point-by-point comparison with the reference works |
| [`docs/REFERENCES.md`](docs/REFERENCES.md) | references and the verification status of each |
| [`docs/references/`](docs/references/README.md) | library: the cited PDFs, with original links, and `references.bib` |
| [`docs/PAPER_OUTLINE.md`](docs/PAPER_OUTLINE.md) | paper outline, with figures and tables |
| [`DECISIONS.md`](DECISIONS.md) | design decisions: alternatives, rationale and how to revert |
| [`REVIEW.md`](REVIEW.md) | open points for review before the paper |
| [`results/SUMMARY.md`](results/SUMMARY.md) | every number, generated by a command |
| [`data/README.md`](data/README.md#english) | public data and how to download it |

### Limitations

LoopSettle is explicit about what it does not know:

- **Synthetic data calibrated to public aggregates, not real interbank data.** Only public
  aggregates are used. Who pays whom is not public and is the model's main assumption, hence the
  sensitivity analysis on receiver imbalance (σ from 0 to 0.8).
- **Reduced scale:** 200 participants and 1 million payments per day, versus about 239 million
  in real Pix. Fractions are the result; absolute values do not scale linearly.
- **Heuristic micro-cycles:** the cycle search is simple, not a full optimiser.
- **One-day horizon** and an hourly profile without bursts. In the stress test, 30-minute outages
  with full resubmission barely change the liquidity need; partial abandonment is not modelled.

Full list in [`docs/LIMITATIONS.md`](docs/LIMITATIONS.md); assumptions and the source of
every parameter in [`docs/PARAMETERS.md`](docs/PARAMETERS.md); open review points in
[`REVIEW.md`](REVIEW.md).

### References

- Hellqvist, M. & Korpinen, K. (2021). *Instant payments as a new normal: Case study of
  liquidity impacts for the Finnish market*. Bank of Finland Economics Review 7/2021.
- Hellqvist, M. & Korpinen, K. (2026). *Instant payments as the new normal: how much more money
  do the banks need?* Journal of Financial Market Infrastructures, 13(2). **The methodological
  basis this project extends.**
- Gonzalez, R. B., Ma, Y. & Zeng, Y. (2025). *The Effect of Instant Payments on the Banking
  System*. Banco Central do Brasil, Working Paper 619.
- Homs-Dones, M., MacKay, R. S., Sansom, B. & Zhou, Y. (2025). *Circular Directional Flow
  Decomposition of Networks*. arXiv:2506.12546.
- Bech, M. L. & Soramäki, K. (2001). *Gridlock Resolution in Interbank Payment Systems*. Bank of
  Finland Discussion Papers 9/2001.
- McMahon, C. et al. (2024). *Improving the efficiency of payments systems using quantum
  computing*. Management Science (arXiv:2209.15392). **The closest work**, in a large-value
  system.

Verification status of every reference in [`docs/REFERENCES.md`](docs/REFERENCES.md); the PDFs are
in [`docs/references/`](docs/references/README.md).

### How to cite

```bibtex
@software{dorneles_loopsettle,
  author  = {Dorneles, Fabio D.},
  title   = {LoopSettle: an open simulator of circular liquidity in instant payment systems},
  version = {0.1.0},
  url     = {https://github.com/fabiodrneles/loopsettle},
  license = {Apache-2.0}
}
```

The metadata are also in [`CITATION.cff`](CITATION.cff) (GitHub shows a "Cite this repository"
button).

### License

[Apache-2.0](LICENSE).

Pix is a trademark of Banco Central do Brasil. LoopSettle is an independent project, not
affiliated with or endorsed by Banco Central do Brasil or any other operator mentioned; system
names are used only to identify them.
