# Guia de uso do LoopSettle

[English](GUIDE.md) · **Português**

Este guia explica, passo a passo, o que o LoopSettle faz, como ele funciona, como usá-lo e como
ler o que ele produz. Não pressupõe conhecimento prévio de sistemas de pagamento nem de
programação em Rust. Quem quiser as definições formais encontra-as em
[`METHODOLOGY.pt-BR.md`](METHODOLOGY.pt-BR.md).

## Sumário

1. [O que o simulador faz](#1-o-que-o-simulador-faz)
2. [As ideias que você precisa conhecer](#2-as-ideias-que-você-precisa-conhecer)
3. [Três maneiras de usar](#3-três-maneiras-de-usar)
4. [Instalar](#4-instalar)
5. [Usar a interface no navegador](#5-usar-a-interface-no-navegador)
6. [Rodar os experimentos prontos](#6-rodar-os-experimentos-prontos)
7. [Analisar os seus próprios pagamentos](#7-analisar-os-seus-próprios-pagamentos)
8. [Escolher e escrever os desenhos de liquidação](#8-escolher-e-escrever-os-desenhos-de-liquidação)
9. [Mudar um cenário](#9-mudar-um-cenário)
10. [Problemas comuns](#10-problemas-comuns)
11. [Onde aprofundar](#11-onde-aprofundar)

## 1. O que o simulador faz

Quando alguém faz um Pix para uma pessoa de **outra instituição**, a instituição de quem paga
transfere dinheiro para a de quem recebe, na hora, numa conta que cada uma mantém no Banco
Central (a Conta PI). Para isso, cada instituição precisa deixar dinheiro parado nessa conta: a
**liquidez**. Dinheiro parado custa caro. (Um Pix entre duas contas da mesma instituição é
resolvido nos livros dela e não usa essa conta.)

Ao longo do dia, parte do dinheiro dá voltas: o banco A paga o B, o B paga o C, o C paga o A. Em
tese, essas voltas poderiam ser compensadas, e os bancos precisariam de menos liquidez. O
LoopSettle responde a três perguntas:

1. **Quanto** do dinheiro que circula entre as instituições dá voltas?
2. **Quanto disso dá para aproveitar na prática**, respeitando regras reais (cada pagamento
   liquidado por inteiro, sem saldo negativo, em poucos segundos)?
3. **Qual o custo em atraso** de aproveitar?

Para responder, ele:

- **gera pagamentos sintéticos** parecidos com o Pix real, calibrados com dados públicos do Banco
  Central (quantidade, valor médio, distribuição dos valores, concentração entre instituições);
- **liquida esses pagamentos de várias formas** (os "desenhos" ou "motores": bruto em tempo real
  como o Pix, compensação em horários fixos, micro-ciclos e outros) e mede a liquidez que cada
  forma exige e o atraso que impõe;
- **mede as voltas** do dinheiro com três lentes matemáticas diferentes;
- **calcula um limite** que vale para qualquer mecanismo: quanto, no máximo, um sistema que
  liquida cada pagamento em até `d` segundos pode economizar.

Também roda **nos seus próprios dados**: você dá um arquivo de pagamentos, e ele devolve os mesmos
números, sem que os dados saiam do computador.

## 2. As ideias que você precisa conhecer

| Termo | O que significa |
|---|---|
| **Participante** | uma instituição com conta de liquidação (banco, fintech, instituição de pagamento). |
| **Liquidação bruta em tempo real** | cada pagamento é liquidado sozinho, na chegada, se houver saldo. É o Pix. No código: `gross`. |
| **Compensação (netting)** | somar o que cada um deve e tem a receber e transferir só a diferença. Exige esperar. |
| **Ciclo** | uma volta de pagamentos (A→B→C→A). Um ciclo pode ser liquidado de uma vez, transferindo só a diferença. |
| **UB (limite superior)** | a liquidez que um participante precisa para que nenhum pagamento seja recusado no bruto em tempo real: o maior "buraco" que as saídas abrem frente às entradas ao longo do dia. |
| **LB (limite inferior)** | a liquidez mínima teórica: o saldo devedor do fim do dia. É o que bastaria se tudo fosse compensado de uma vez, no fim do dia. |
| **Economia de liquidez** | quanto um desenho reduz a liquidez frente ao UB, em %: `1 − liquidez do desenho ÷ UB`. |
| **Piso de liquidez** | a menor liquidez que **qualquer** mecanismo que liquida cada pagamento em até `d` segundos pode exigir. Se o piso para 10 s é quase igual ao UB, nenhum mecanismo de 10 s economiza quase nada. |
| **Atraso p50, p95** | metade dos pagamentos (p50) ou 95% deles (p95) liquidou em até esse tempo. |
| **Meta de 2,8 s** | o Manual de Tempos do Pix pede que 50% dos pagamentos sejam liquidados em até 2,8 s. |
| **ρ (rho)** | indicador de atraso do Banco da Finlândia: 0 = tudo na hora, 1 = nada antes do fim do dia. |
| **π (pi)** | indicador de uso de liquidez: liquidez usada ÷ valor total pago. |
| **Semente (seed)** | o número que fixa o sorteio dos pagamentos. A mesma semente dá sempre o mesmo resultado, em qualquer computador. |
| **Calibração** | ajustar o gerador para reproduzir números públicos. Cada parâmetro está em [`PARAMETERS.pt-BR.md`](PARAMETERS.pt-BR.md) como `SOURCED` (da fonte), `CALIBRATED` (ajustado) ou `ASSUMPTION` (suposição). |

**Um exemplo à mão.** Três bancos, saldo zero. A paga 10 a B; depois B paga 5 a C; depois C paga 5
a A.

- No bruto em tempo real, A precisa ter 10 na conta para pagar o primeiro: **UB de A = 10**. B
  recebe 10 antes de pagar 5, então não precisa de nada; C recebe 5 antes de pagar 5, idem.
- No fim do dia, A pagou 10 e recebeu 5: deve 5. **LB de A = 5**.
- A volta A→B→C→A de valor 5 poderia ser compensada: só restam os outros 5 de A para B. É a
  diferença entre o UB (10) e o LB (5).

## 3. Três maneiras de usar

| Você quer... | Use | Precisa instalar |
|---|---|---|
| ver o simulador funcionando e mexer nos parâmetros | a **interface no navegador** (seção 5) | Node.js e Rust, ou só um navegador se alguém publicar a página |
| reproduzir os números do estudo | os **experimentos prontos** (seção 6) | Rust |
| analisar pagamentos da sua instituição | os comandos **`bound` e `settle`** (seção 7), ou o painel "Analisar o seu arquivo" da interface | Rust (ou a interface) |

## 4. Instalar

O passo a passo completo, para Windows e Linux, partindo de um computador sem nada, está no
README, na seção [Instalação do zero](../README.md#instalação-do-zero). Em resumo:

1. Instale o **Git**, o **Rust** (pelo `rustup`) e, para a interface, o **Node.js LTS**.
2. Baixe o código:

```bash
git clone https://github.com/fabiodrneles/loopsettle.git
```

3. Entre na pasta e confira que tudo funciona (a primeira compilação leva alguns minutos):

```bash
cd loopsettle
```

```bash
cargo test --workspace
```

Todos os comandos deste guia são dados **a partir da pasta `loopsettle`**.

## 5. Usar a interface no navegador

Para abrir:

```bash
cd web && npm install && npm run wasm && npm run dev
```

Depois abra `http://localhost:5173`. A simulação roda no próprio navegador; nada é enviado a
servidor.

**Os controles**

- **Participantes, pagamentos:** o tamanho do cenário. No celular a interface reduz o tamanho
  sozinha.
- **Desequilíbrio entre recebedores:** o quanto alguns participantes recebem mais do que pagam. É
  a principal suposição do modelo; mexer nele mostra se o resultado depende dela.
- **Janela de busca de ciclos** (0,5 a 10 s) e **atraso máximo** (2 a 10 s): os parâmetros dos
  micro-ciclos.
- **Semente:** troque para ver outro dia sorteado com as mesmas regras.
- **Idioma:** botão no canto, ou `?lang=pt` / `?lang=en` no endereço.

**Os painéis, e como lê-los**

| Painel | O que mostra | Como ler |
|---|---|---|
| **A rede** | participantes em círculo (tamanho = atividade; tracejado = paga mais do que recebe) e os fluxos entre eles | em verde, a parte do fluxo que dá voltas (circulação máxima); em cinza, a que só vai numa direção |
| **Potencial × realizável** | a "escada" das medidas: do quanto daria para compensar no papel até a liquidez poupada de fato | quanto maior a queda de um degrau para o outro, maior a distância entre o que parece possível e o que se consegue |
| **O dia em 15 minutos** | o valor pago em cada intervalo e a soma das posições devedoras | se a linha escura sobe por horas, a liquidez é exigida por desequilíbrios longos, que segundos de espera não resolvem |
| **Liquidez por desenho** | a liquidez que cada forma de liquidar exige, do bruto ao mínimo teórico (LB) | quanto mais baixa a barra, menos dinheiro parado |
| **Economia × janela** | a economia dos micro-ciclos para cada janela | perto de zero significa que esperar alguns segundos não ajuda |
| **Liquidez restrita** | com saldo menor que o necessário: valor recusado, atraso e cumprimento da meta de 2,8 s | compara os desenhos quando falta dinheiro |
| **Piso de liquidez** | o piso para cada prazo `d`, com o resultado de cada desenho por cima | nenhum desenho pode ficar abaixo do piso; a distância até ele é o que ainda daria para ganhar |
| **Analisar o seu arquivo** | os mesmos números para um CSV seu (formato na seção 7) | o arquivo não sai do computador |

Os cenários do navegador são menores que os do estudo (até 50 participantes e 100 mil
pagamentos), então os números da tela não são os do artigo. Em cenários pequenos, a economia dos
micro-ciclos pode sair levemente negativa: a fila muda a ordem das liquidações. Não é erro.

## 6. Rodar os experimentos prontos

Cada número do estudo vem de um comando. Para ver a lista:

```bash
cargo run --release -p loopsettle-cli -- list
```

Para rodar um (por exemplo, a comparação central):

```bash
cargo run --release -p loopsettle-cli -- experiment potential-vs-realizable
```

Acrescente `--quick` para uma versão dez vezes menor, em segundos. Para rodar tudo e refazer o
resumo:

```bash
make experiments summary
```

**Onde ficam os resultados.** Cada experimento grava em `results/<nome>/`:

| Arquivo | O que tem |
|---|---|
| `*.csv` | as tabelas, uma linha por caso; abrem em qualquer planilha |
| `summary.json` | os números principais, a semente, a versão do código e o comando que gerou tudo |
| `run.json` | tempo e memória da execução |
| `*.svg` | as figuras; abrem no navegador |

O jeito mais fácil de ler é o resumo de todos os experimentos, em tabelas, em
[`results/SUMMARY.pt-BR.md`](../results/SUMMARY.pt-BR.md). Ele é regenerado por comando a cada
rodada.

**Como ler um número com incerteza.** Os experimentos repetem o dia com várias sementes e mostram
a média com um intervalo de confiança de 95%, assim: `1,42% [0,57; 2,26]`. Leia como "a média foi
1,42%, e o valor verdadeiro do modelo provavelmente está entre 0,57% e 2,26%". Se o intervalo de
uma diferença inclui o zero (por exemplo `−0,02 [−0,05; +0,01]`), não dá para dizer que houve
efeito.

## 7. Analisar os seus próprios pagamentos

**O formato do arquivo.** Um CSV com uma linha por pagamento e estas colunas:

| Coluna | Conteúdo | Exemplo |
|---|---|---|
| `time` ou `t_ms` | horário: segundos, `HH:MM:SS`, ou milissegundos (`t_ms`) | `40168` |
| `from` | quem paga (qualquer identificador) | `P003` |
| `to` | quem recebe | `P000` |
| `amount` | valor em unidades da moeda, com até 2 casas | `43.20` |
| `status` (opcional) | só entram as linhas liquidadas (`settled`, `liquidado`, `ACSC` e equivalentes) | `settled` |

As linhas precisam estar em ordem de horário. Se não estiverem, ordene antes (funciona com
arquivos maiores que a memória):

```bash
cargo run --release -p loopsettle-cli -- sort bruto.csv pagamentos.csv
```

Para treinar sem dados reais, use o arquivo de exemplo `data/samples/payments_example.csv` (5.123
pagamentos, 20 participantes).

**Passo 1: os limites.**

```bash
cargo run --release -p loopsettle-cli -- bound data/samples/payments_example.csv
```

A saída começa assim:

```text
UB (liquidity for real-time gross settlement): 588035.67
LB (one netting at the end of the file's period): 401155.75  (LB/UB = 0.6822)
```

Leia: no bruto em tempo real, o sistema todo precisa de 588.035,67 de liquidez; com uma única
compensação no fim do dia, de 401.155,75 (68% disso). A diferença é o máximo que **qualquer**
mecanismo pode economizar.

Depois vem uma tabela, um prazo `d` por linha:

```text
max_delay_s,delay_bound,delay_bound_pct_of_ub,liquidity_floor,tight_bound,tight_bound_pct_of_ub
10,33458.2,5.689825,588035.67,0,0
900,94397.33,16.052994,557184.66,30851.01,5.246452
3600,193333.8,32.877903,513820.7,74214.97,12.620828
```

A coluna que importa é `tight_bound_pct_of_ub`: a economia máxima de qualquer mecanismo que
liquida cada pagamento em até `max_delay_s` segundos. Neste arquivo, com 10 s, **0%**: nenhum
mecanismo de 10 s economiza nada. Com 15 minutos (900 s), até 5,2%; com uma hora, até 12,6%.
`liquidity_floor` é o piso (UB − economia máxima). `delay_bound` é uma forma mais simples e mais
folgada do mesmo limite.

**Passo 2: os desenhos.**

```bash
cargo run --release -p loopsettle-cli -- settle data/samples/payments_example.csv
```

Acrescenta uma tabela, um desenho por linha:

```text
engine,liquidity,saving_vs_ub_pct,delay_p50_ms,delay_p95_ms,share_within_2800ms,value_share_in_cycles
gross_real_time,588035.67,0,0,0,1,0
micro_cycle_all_w500_d10000,588035.67,0,9984,9984,0.008198,0.029111
```

`saving_vs_ub_pct` é a economia; `delay_p50_ms` e `delay_p95_ms`, os atrasos em milissegundos;
`share_within_2800ms`, a fração liquidada em até 2,8 s (a meta do Pix pede 0,5 ou mais);
`value_share_in_cycles`, a fração do valor liquidada em ciclos. Escolha os desenhos com
`--engines` (seção 8) e os prazos com `--delays 1,10,60`.

**Passo 3: um relatório para compartilhar.**

```bash
cargo run --release -p loopsettle-cli -- bound pagamentos.csv --report
```

Gera `report.md`, `report.pt-BR.md` e uma figura, com os números, a versão do código, uma
impressão digital do arquivo (não o conteúdo) e um parágrafo de interpretação e limitações.

**Onde fica a saída.** Em `loopsettle-out/<nome do arquivo>/`, ou na pasta que você indicar com
`--out`.

**Privacidade.** Por padrão a saída tem só totais do sistema. Com `--per-participant` sai uma
linha por participante, com nomes trocados por `P001`, `P002`... e recusada se houver menos de 5
participantes (`--min-participants`). Os identificadores originais só aparecem com `--keep-names`.

## 8. Escolher e escrever os desenhos de liquidação

Cada desenho é escrito como um texto curto. Tempos em milissegundos.

| Texto | Desenho |
|---|---|
| `gross` | bruto em tempo real: liquida na chegada se houver saldo (o Pix) |
| `deferred:96` | compensação de tudo em 96 horários fixos por dia (a cada 15 min) |
| `micro:all:500:10000` | micro-ciclos: todo pagamento espera; a cada 0,5 s procura ciclos; quem esperar 10 s vai para o bruto |
| `micro:shortfall:500:10000` | micro-ciclos só quando falta saldo |
| `queue:fifo:10000` | fila simples: espera até 10 s por dinheiro que chega, sem compensação |
| `hold:10000` | segura todo pagamento 10 s e liquida bruto (para separar o efeito de esperar do efeito de compensar) |
| `gridlock:fifo:60000:3600000` | resolução de gridlock de Bech & Soramäki (2001), a cada minuto |
| `netq:fifo:60000:3600000` | compensação completa das filas de Koponen & Soramäki (1998), a cada minuto |
| `bilateral:all:500:10000` | só compensação entre pares (A↔B) |

Exemplo:

```bash
cargo run --release -p loopsettle-cli -- settle pagamentos.csv --engines gross,deferred:96,micro:all:500:10000
```

Para saber o que cada sistema real usa (Pix, FedNow, UPI...), veja
[`INSTANT_PAYMENTS.pt-BR.md`](INSTANT_PAYMENTS.pt-BR.md).

## 9. Mudar um cenário

Os experimentos leem arquivos de texto em `experiments/<nome>.toml`. Para testar outra hipótese,
copie um, mude e rode com `--config`:

```bash
cargo run --release -p loopsettle-cli -- experiment baseline --config meu-cenario.toml
```

As chaves mais úteis, na seção `[scenario]`:

| Chave | O que muda |
|---|---|
| `seed` | a semente |
| `participants` | número de participantes |
| `payments_per_day` | pagamentos por dia (a amostra; o fator de escala converte para o tamanho real) |
| `days` | número de dias |
| `receiver_bias_sigma` | desequilíbrio entre recebedores (0 = equilibrado) |
| `calibration` | arquivo com os números públicos (`data/calibration/pix.toml`) |
| `amounts_fit` | `percentiles` (padrão: forma dos valores ajustada aos percentis do Banco Central) ou `mean` (método anterior) |
| `hourly_profile` | `default` ou `spi2024` (perfil de horários) |

A lista completa, com a fonte de cada valor, está em [`PARAMETERS.pt-BR.md`](PARAMETERS.pt-BR.md).

## 10. Problemas comuns

| Sintoma | O que fazer |
|---|---|
| `cannot read experiments/....toml` | rode o comando a partir da pasta `loopsettle` |
| o arquivo é recusado por estar fora de ordem | ordene com `loopsettle sort` (seção 7) |
| `--per-participant refused` | o arquivo tem menos de 5 participantes; use só os totais, ou ajuste `--min-participants` sabendo que poucas linhas podem identificar pagamentos |
| linhas ignoradas (`rows skipped (invalid)`) | confira separador, cabeçalho e valores; o comando informa quantas foram ignoradas |
| a primeira compilação demora | é normal (minutos); as seguintes são rápidas |
| computador lento ou com pouca memória | use `--quick`; cada experimento usa poucas dezenas de MiB |
| números da interface diferentes dos do estudo | normal: o cenário do navegador é menor |

## 11. Onde aprofundar

| Quero entender... | Leia |
|---|---|
| cada métrica, com fórmula e exemplo | [`METHODOLOGY.pt-BR.md`](METHODOLOGY.pt-BR.md) |
| de onde vem cada parâmetro | [`PARAMETERS.pt-BR.md`](PARAMETERS.pt-BR.md) |
| o que o estudo pode e não pode afirmar | [`CLAIMS.pt-BR.md`](CLAIMS.pt-BR.md) |
| os limites do modelo | [`LIMITATIONS.pt-BR.md`](LIMITATIONS.pt-BR.md) |
| todos os números | [`results/SUMMARY.pt-BR.md`](../results/SUMMARY.pt-BR.md) |
| o formato dos dados | [`data/README.md`](../data/README.md) |
| usar o simulador como biblioteca em Rust | os `README.md` de cada pasta em `crates/` |
