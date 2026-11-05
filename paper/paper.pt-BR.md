---
# RASCUNHO EM PORTUGUÊS, só para leitura do autor. Não é o arquivo enviado ao JOSS.
# O JOSS recebe paper/paper.md, em inglês, escrito a partir desta versão depois da revisão.
title: 'LoopSettle: um simulador aberto para medir quanto da liquidez dos pagamentos instantâneos a compensação recupera de fato'
tags:
  - Rust
  - WebAssembly
  - pagamentos instantâneos
  - liquidez
  - compensação multilateral
  - simulação de sistemas de pagamentos
authors:
  - name: Fabio Darci Dorneles
    orcid: 0000-0000-0000-0000   # [PREENCHER: ORCID do autor]
    affiliation: 1
affiliations:
  - name: Pesquisador independente, Brasil   # [CONFIRMAR: afiliação]
    index: 1
date: 5 novembro 2026
bibliography: paper.bib
---

# Resumo

Nos sistemas de pagamentos instantâneos com liquidação bruta, como o Pix no Brasil, o FedNow
nos Estados Unidos e o TIPS na zona do euro, cada transferência é liquidada sozinha, na hora, e
o banco pagador precisa ter saldo no banco central antes de cada pagamento, a qualquer hora.
Parte desse saldo parece apenas "dar voltas": o banco A paga B, B paga C e C paga A. Se esses
pagamentos fossem compensados entre si, os bancos precisariam de menos dinheiro parado.

O LoopSettle é um simulador aberto que mede quanto dessa compensação aparente vira liquidez
economizada de verdade quando se respeita a ordem em que os pagamentos chegam. Ele gera fluxos
sintéticos de pagamentos calibrados com dados públicos, liquida o mesmo fluxo sob vários desenhos
de sistema (bruto em tempo real, compensação diferida em ciclos fixos e micro-ciclos atômicos com
espera máxima por pagamento) e compara a liquidez exigida e os atrasos de cada um. Também mede a
circularidade da rede por três lentes diferentes e calcula um teto, válido para qualquer
algoritmo, para a economia de qualquer mecanismo que segure cada pagamento por no máximo *d*
segundos. O mesmo núcleo roda na linha de comando, em arquivos de pagamentos reais (com saída só de
agregados) e no navegador, via WebAssembly.

# Declaração de necessidade

Bancos centrais e pesquisadores medem a perda de compensação nos pagamentos instantâneos com
medidas "potenciais", que comparam entradas e saídas de um dia sem olhar a ordem no tempo. O
estudo do Banco Central do Brasil sobre o Pix [@gonzalez2025instant] usa uma medida desse tipo,
por banco; o Banco da Finlândia comparou a liquidez de ciclos fixos com a do modo instantâneo em
dados sintéticos calibrados [@hellqvist2021instant]. Faltava uma ferramenta aberta que pusesse,
lado a lado e no mesmo fluxo, a medida potencial e a economia **realizável** por mecanismos com
atraso limitado, e que permitisse a qualquer banco central rodar a mesma análise nos próprios
dados sem compartilhá-los.

O LoopSettle foi feito para esse público: pesquisadores de sistemas de pagamentos, analistas de
bancos centrais e operadores de sistemas instantâneos que avaliam mecanismos de economia de
liquidez. Ele responde a três perguntas concretas: quanto da liquidez exigida é circular, quanto
disso um mecanismo com espera máxima *d* consegue recuperar, e qual o custo em atraso de
recuperá-lo.

# Estado da arte

O simulador BoF-PSS2 do Banco da Finlândia é a referência histórica para simular liquidez e
mecanismos de economia de liquidez em sistemas de liquidação bruta
[@bofpss2]; é gratuito para pesquisa não comercial, mas distribuído sob licença e com código
fechado. O PSSimPy é uma biblioteca em Python para simular sistemas de pagamentos de alto valor,
com filas, estratégias de bancos e testes de falha [@see2024pssimpy]. Os dois tratam de sistemas
de alto valor e de mecanismos clássicos de fila; nenhum mede a distância entre compensação
potencial e realizável nem calcula um teto para mecanismos com atraso limitado.

Os trabalhos mais próximos reordenam pagamentos em lotes curtos antes da liquidação em sistemas de
alto valor: com dados reais do sistema canadense, lotes de 70 pagamentos, com cerca de 90 segundos
de espera, reduzem a liquidez necessária em relação à ordem de chegada [@mcmahon2024quantum].
Estudos anteriores mediram o ganho de mecanismos diferidos em dados históricos do Fedwire
[@johnson2004economizing]. Para medir circularidade, o LoopSettle implementa a decomposição de
Hodge e a decomposição circular-direcional de circularidade máxima [@homsdones2025cdfd], além do
cancelamento de ciclos no estilo da resolução de travamentos de Bech e Soramäki [@bech2001gridlock].

A justificativa para construir em vez de contribuir com uma ferramenta existente é tripla: as
ferramentas disponíveis não são abertas ou não tratam do varejo instantâneo; a pergunta central
(potencial contra realizável, com um teto independente do algoritmo) exige que os desenhos de
sistema rodem no mesmo fluxo, numa só passada; e a análise precisa rodar em dados confidenciais na
máquina de quem os detém, com memória pequena e saída só de agregados.

# Desenho do software

O LoopSettle é um workspace em Rust com cinco crates: o núcleo de liquidação (`loopsettle-core`),
a topologia (`loopsettle-topology`), o gerador e o motor de simulação (`loopsettle-sim`), a linha
de comando (`loopsettle-cli`) e os bindings para o navegador (`loopsettle-wasm`), usados por uma
interface em React que roda a simulação num Web Worker. As escolhas principais e suas trocas foram:

- **Dinheiro em centavos inteiros (`i64`), nunca em ponto flutuante**, para que as invariantes de
  conservação valham exatamente, e não dentro de uma tolerância.
- **Streaming numa só passada.** Os pagamentos são gerados ou lidos sob demanda e entregues a todos
  os motores ao mesmo tempo; a memória cresce com o número de participantes, não com o de
  pagamentos. Um milhão de pagamentos roda em menos de um segundo, com poucos megabytes. A troca é
  que métricas que exigiriam reler o fluxo (como o teto de atraso) refazem o fluxo a partir da
  semente, em vez de guardá-lo.
- **Uma interface comum de motor de liquidação**, com três motores: bruto em tempo real, compensação
  diferida com *N* ciclos por dia e micro-ciclos atômicos, que procuram ciclos na fila a cada janela
  e liquidam cada ciclo inteiro ou nada, com dois modos (todo pagamento na fila, ou só quando falta
  saldo).
- **Algoritmos exatos implementados à mão, com oráculos de teste.** A decomposição circular de
  circularidade máxima é resolvida como um problema de fluxo de custo mínimo, sem biblioteca de
  programação linear no caminho principal; um solver de programação linear em Rust puro serve só de
  oráculo nos testes, em mil grafos aleatórios. A decomposição de Hodge usa gradiente conjugado sem
  montar matrizes densas.
- **Invariantes testadas por propriedades.** Oito regras que nunca podem quebrar, como conservação
  do dinheiro, preservação da posição líquida, ausência de saldo negativo, atomicidade dos ciclos e
  determinismo, são verificadas com milhares de casos aleatórios a cada execução.
- **Reprodutibilidade por construção.** Cada experimento é um arquivo TOML com semente fixa; cada
  resultado grava a semente, a versão do código e o comando que o gerou, e rodar de novo reproduz os
  arquivos byte a byte. Um único comando reconstrói todos os resultados e figuras.
- **Leveza deliberada.** Poucas dependências e perfis de compilação leves permitem compilar, testar
  e rodar tudo num notebook com 2 núcleos e 4 GB de RAM, a máquina de referência do projeto.

# Declaração de impacto em pesquisa

[ATENÇÃO, autor: esta é a seção que o JOSS avalia com mais rigor. Hoje só existe uso pelo próprio
autor. Antes de enviar, ela precisa citar o preprint e, idealmente, algum uso externo.]

O LoopSettle sustenta um estudo sobre liquidez em pagamentos instantâneos [@dorneles2027preprint]
[PREENCHER quando o preprint estiver publicado]. Nesse estudo, num cenário calibrado com dados
públicos do Pix, uma medida potencial do mesmo tipo da usada pelo BCB, somada para o sistema todo,
indica que 85% do volume seria compensável (no Pix real, o banco mediano do WP 619 fica entre 57% e
80%), mas micro-ciclos de
até 10 segundos economizam cerca de 0,01% da liquidez, e o teto demonstrado para qualquer mecanismo
com essa espera fica abaixo de 1%. Os mesmos comandos mostram por que a reordenação dá resultado em
fluxos de alto valor, com poucos pagamentos grandes.

Para facilitar o reúso, o repositório traz: todos os experimentos e resultados reproduzíveis, com
um resumo gerado por comando; os comandos `bound` e `settle`, que rodam a análise em arquivos de
pagamentos reais sem que os dados saiam da máquina; um guia de instalação do zero para Windows,
Linux e macOS; e documentação de metodologia, parâmetros, limitações e afirmações em português e
inglês. [PREENCHER: usuários externos, apresentações ou contatos com bancos centrais, se houver.]

# Declaração de uso de IA

[A CONFIRMAR PELO AUTOR, palavra por palavra: o JOSS considera falta ética uma declaração
incompleta ou imprecisa.]

O código, os testes, a documentação e o rascunho deste artigo foram produzidos com ampla
assistência do Claude (Anthropic), usado pelo Claude Code [PREENCHER: modelo e versão]. A
assistência incluiu geração e refatoração de código, escrita de testes, redação e tradução da
documentação e redação deste texto. O autor definiu a pergunta de pesquisa, a especificação do
projeto, as restrições de desenho e as regras de validação (invariantes, critérios de morte
registrados antes dos testes de robustez, limites do que o estudo pode afirmar), tomou as decisões
de desenho registradas no repositório e revisou, editou e validou o código, os resultados e o texto.
A correção do código é verificada por testes de propriedades, por oráculos independentes e pela
reprodução byte a byte de todos os resultados.

# Agradecimentos

[PREENCHER: apoio financeiro, se houver; ou declarar que não houve.] [PREENCHER: conflitos de
interesse. O JOSS pede que se declare qualquer vínculo profissional relevante, inclusive emprego
na área de pagamentos.] O LoopSettle é independente e não tem relação com o Banco Central do Brasil
nem endosso dele; Pix é marca do BCB.

# Referências
