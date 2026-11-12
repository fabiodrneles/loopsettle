# 07 — Gerador: rajadas, vários dias, modelos nulos (P2)

## Contexto

Achado A12 e LIMITATIONS 5, 6, 14. O limite apertado (spec 01) depende de uma única coisa do
fluxo: **quanto cada participante recebe nos `d` segundos depois de cada saída**. Chegadas de
Poisson independentes dão a esse número o menor valor possível para um dado volume; rajadas e
correlação entre saídas e entradas o aumentam ou diminuem. Logo o resultado "segundos não
recuperam nada" precisa ser testado contra processos de chegada mais realistas. Cada extensão
abaixo é um parâmetro com padrão desligado, para que os resultados atuais não mudem.

## G1 — Rajadas (sobredispersão)

- Processo de Cox simples: a taxa horária é multiplicada por um fator aleatório por bloco de
  `burst_block_ms` (padrão 60 s), lognormal com média 1 e desvio `burst_sigma` (padrão 0 =
  Poisson atual). Mantém o volume esperado e a amostragem exata por memorylessness dentro do
  bloco. Alternativa registrada: Hawkes (mais realista, mais caro, mais parâmetros).
- Rajadas **por participante**: lotes (`batch_prob`, `batch_size`) em que um pagador emite
  dezenas de pagamentos no mesmo segundo (folha, repasse de credenciadora). É o caso que mais
  mexe no limite: muitas saídas juntas, sem entradas correspondentes.
- **AC:** com `burst_sigma = 0` e `batch_prob = 0` o fluxo é idêntico byte a byte ao atual
  (teste); o índice de dispersão (variância/média das contagens por minuto) cresce com
  `burst_sigma` (teste estatístico com tolerância); `PARAMETERS*.md` marca os novos como
  `ASSUMPTION` até haver fonte.

## G2 — Reciprocidade no tempo (resposta a recebimentos) — **FEITO em 12/11/2026; o critério disparou**

> `echo_share`, `echo_delay_ms`, `echo_return_prob` em `ScenarioConfig`; experimento `echo-sweep`.
> Eco adiante: nada muda (limite ≤ 0,18% com 20% de eco). Eco de volta a quem pagou: limite de
> 4,2% e economia obtida de 3,9% com 20% de eco a 1 s. A afirmação central virou condicional
> (CLAIMS 1j, ASSESSMENT atualização 5, REVIEW 87). Sensibilidade feita (REVIEW 88): metade do valor 1,7%, um quarto 0,6%, σ = 1,0 1,8%, 3× o
> volume 2,1%; a piora dos micro-ciclos vem de liquidar fora da ordem (CLAIMS 1k). Com 10× o volume: 1,2%
> (cai perto de 1/√volume). Em aberto: rodar com 30 milhões e achar o argumento para essa queda.

Parte dos pagamentos é consequência de um recebimento (o cliente recebe e paga em seguida).
Parâmetro `echo_prob`: com essa probabilidade, um pagamento recebido por `j` gera um pagamento
de saída de `j` após um atraso exponencial de média `echo_delay_ms`. Isso cria exatamente a
circularidade de curto prazo que os micro-ciclos poderiam capturar; é o teste mais duro para a
afirmação central. **AC:** experimento `echo-sweep` com `echo_prob ∈ {0; 0,05; 0,2; 0,5}` e
`echo_delay ∈ {1 s, 10 s, 60 s}` reporta piso, obtido e IC; `ASSESSMENT` ganha um critério de
morte novo (sugestão: se o limite apertado de 10 s passar de 2% do UB para `echo_prob ≤ 0,2`,
a afirmação "desprezível" passa a depender de um parâmetro não identificado e deve dizer isso).

## G3 — Vários dias com saldo transportado

- O `funding-windows` já simula 5 dias. Generalizar: `days` qualquer, perfil por dia da semana
  (`SPI_2024_WEEKDAY` já existe), dias especiais (`day_weights` e um multiplicador de 5º dia
  útil; fonte a procurar nas estatísticas do Pix, marcar `ASSUMPTION` enquanto não houver).
- Métricas por dia e do período: UB do período ≠ soma dos UB diários; reportar os dois.
- **AC:** 30 dias × 1 milhão de pagamentos em < 1 GB (streaming já garante) e tempo medido.

## G4 — Modelos nulos embutidos

Para cada medida que o estudo reporta, um nulo calculado pelo próprio comando:
- **Embaralhamento de horários** (mantém pares e valores, permuta os instantes dentro do dia):
  separa estrutura temporal de estrutura bilateral. Em streaming: exige duas passadas ou uma
  reamostragem por seed; aceitar a versão "regerar o fluxo com outra semente só para o relógio".
- **Embaralhamento de recebedores** (mantém pagador, valor e instante): separa a estrutura
  bilateral. 
- Hodge: o nulo `(m − n + 1)/m` já documentado vira coluna de `ladder.csv`.
- **AC:** `baseline` e `loopsettle bound` aceitam `--null time|payee` e informam a medida
  observada, a do nulo e a razão.

## G5 — Perfis por participante a partir de arquivo

`profile_file`: CSV `participante, hora, peso_saida, peso_entrada`. Permite a um banco central
calibrar os perfis com dados próprios sem tocar no código (hoje são constantes por classe,
D18). **AC:** com o arquivo equivalente às constantes atuais, resultado idêntico.

## Como validar antes de implementar

1. Ler `stream.rs` inteiro e `rng.rs` (não auditado): confirmar que os fluxos derivados
   (`Rng::derive`) permitem acrescentar sorteios sem deslocar a sequência existente. Se o novo
   sorteio usar o mesmo gerador, os resultados atuais mudam mesmo com o parâmetro desligado:
   usar um `Rng::derive(seed, <novo rótulo>)` separado para cada extensão.
2. Medir o custo por pagamento antes e depois (`bench_stream`); orçamento: ≤ 20% mais lento
   com tudo desligado.

## Riscos

- Mais parâmetros `ASSUMPTION` enfraquecem o estudo se virarem padrão. Regra: extensões entram
  como **análise de sensibilidade**, nunca no cenário base, até terem fonte.
