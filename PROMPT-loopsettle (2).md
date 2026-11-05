# Prompt de construção — Projeto "LoopSettle"
### Simulador de liquidez circular em pagamentos instantâneos

> Como usar: salve este arquivo como `CLAUDE.md` na raiz de uma pasta vazia, abra o Claude Code nela e diga: **"Construa o projeto completo seguindo o CLAUDE.md, no modo autônomo."** O agente executa todas as fases em sequência, registra o progresso em `PROGRESS.md` e as decisões em `DECISIONS.md`. Se a sessão acabar no meio, abra uma nova e diga **"Continue de onde parou, conforme o PROGRESS.md."**

## 0. Modo de execução: AUTÔNOMO

- Execute **todas as fases (0 a 10) em sequência**, sem esperar minha aprovação entre elas.
- Uma fase só está concluída quando seus **critérios de aceite** passam. Se não passarem, corrija antes de avançar.
- Quando houver uma decisão de desenho que afete os resultados do estudo, **não pare**: escolha a opção mais defensável na literatura citada, implemente-a e registre em `DECISIONS.md` (o que decidiu, alternativas, por quê, e como reverter). Se possível, deixe a alternativa como parâmetro configurável.
- Pare e me pergunte **somente** se: (a) faltar uma ferramenta que você não consegue instalar; (b) o orçamento de memória ou tempo da seção 2 for impossível de cumprir; (c) uma ação for irreversível fora da pasta do projeto.
- Mantenha `PROGRESS.md` sempre atualizado: fase atual, o que está pronto, o que falta, números medidos (tempo e RAM) e o próximo passo exato. Ele deve permitir retomar o trabalho numa sessão nova sem contexto anterior.
- Faça commits pequenos ao final de cada etapa, para que nada se perca se a sessão terminar.
- **Objetivo final:** entregar o simulador completo, testado e com todos os experimentos rodados, mais um pacote de revisão (Fase 10) para que eu e meu revisor analisemos cada ponto antes de escrever o artigo.

---

## 1. Contexto do projeto

Você vai me ajudar a construir o **LoopSettle**, um simulador aberto que mede quanto da liquidez exigida por um sistema de pagamentos instantâneos (como o Pix, que liquida cada transação de forma bruta e em tempo real) corresponde a **fluxo circular** — dinheiro que dá voltas entre instituições (A→B→C→A) e poderia ser compensado — e quanto disso é **compensável de fato** respeitando regras reais.

O simulador deve:
1. Gerar fluxos sintéticos de pagamentos entre participantes, calibrados com dados públicos agregados.
2. Liquidar esses fluxos sob diferentes desenhos de sistema e comparar a liquidez necessária e os atrasos.
3. Medir a circularidade da rede com duas lentes: **diagnóstico** (decomposição de Hodge) e **execução** (algoritmo de ciclos que respeita fluxos positivos e o menor pagamento do ciclo).
4. Exibir tudo numa interface web interativa que roda no navegador, sem servidor.

### Trabalhos de referência (citar no README e nos artigos)
- Hellqvist & Korpinen, *Instant payments as the new normal* (Banco da Finlândia, 2021; JFMI, 2026) — comparação de liquidez entre modelo com ciclos e modelo instantâneo, com dados sintéticos calibrados. **É a base metodológica que estendemos.**
- Estudo sobre Pix e liquidez bancária (BCB Working Paper 619) — perda de netting no Pix; mede capacidade **potencial**, não realizável.
- *Circular Directional Flow Decomposition of Networks* (arXiv 2506.12546) — decomposição que respeita conservação de fluxo.
- Bech & Soramäki, *Gridlock Resolution in Interbank Payment Systems* (BoF DP 9/2001) — algoritmo de ciclos.
- PixSim (arXiv 2609.30684) — exemplo de calibração em que cada parâmetro tem fonte.

### Pergunta de pesquisa
> Quanto da liquidez que os participantes do Pix precisam manter é circular, quanto disso é recuperável na prática, e qual o custo em atraso de recuperá-la com micro-ciclos atômicos?

---

## 2. Restrições de hardware (OBRIGATÓRIAS)

Minha máquina: **Intel Core i3 de 4ª geração (2 núcleos / 4 threads), 4 GB de RAM.** Todas as decisões devem respeitar isso:

- **Orçamento de memória:** qualquer execução do simulador deve usar no máximo **1 GB de RAM**. Nunca carregue todas as transações em memória: processe em **streaming** e guarde só agregados por janela de tempo.
- **Compilação Rust leve:**
  - `CARGO_BUILD_JOBS=2` (configure em `.cargo/config.toml`).
  - Perfil `dev` com `opt-level = 1` para o crate e `opt-level = 2` para dependências; `debug = "line-tables-only"`.
  - Perfil `release` com `codegen-units = 4`, `lto = "thin"`.
  - **Mínimo de dependências.** Antes de adicionar qualquer crate, justifique e prefira implementar à mão rotinas simples.
  - Prefira `cargo check` durante o desenvolvimento; só rode `cargo build --release` quando necessário.
- **Sem bibliotecas pesadas de álgebra linear no início.** Implemente à mão: matrizes esparsas CSR simples, produto matriz-vetor e **gradiente conjugado** (CG) para resolver os sistemas de Laplaciano. Isso é suficiente para o MVP.
- **Frontend leve:** Vite + React + TypeScript. Nada de Next.js. Renderização com **Canvas 2D** (não WebGPU, cuja compatibilidade com a GPU integrada dessa geração é incerta). Simulação sempre num **Web Worker**.
- **Ambiente:** se eu estiver no Windows, use o toolchain nativo do Windows, **não WSL2** (WSL2 consome RAM demais com 4 GB). Se eu estiver no Linux, recomende ativar zram/swap de 2–4 GB.
- **Editor:** avise se o rust-analyzer estiver pesado e sugira desativar o `cargo check` automático ao salvar (`"rust-analyzer.checkOnSave": false`).
- Não dependa de instruções SIMD específicas; compile para o target padrão.

---

## 3. Regras de trabalho

1. **Dados:** use apenas dados **públicos** (API de dados abertos do Pix do BCB, dados on-chain de stablecoins) e **sintéticos**. **Nunca** use, peça ou sugira dados do meu empregador ou de clientes.
2. **Reprodutibilidade:** toda simulação recebe uma `seed`. Todo número que for para um artigo deve ser gerado por um comando: `cargo run --release -p loopsettle-cli -- experiment <nome>`.
3. **Testes primeiro nas invariantes.** Use testes baseados em propriedades (crate `proptest`) para as regras que nunca podem quebrar (seção 6).
4. **Honestidade nos resultados:** todo resultado indica o que é medido e o que é suposição. Cada parâmetro do gerador tem fonte, calibração ou está marcado como `ASSUMPTION` num arquivo `PARAMETERS.md`.
5. **Registre antes de codar:** ao iniciar cada fase, anote em `PROGRESS.md` o que vai fazer e por quê. Mantenha commits pequenos e com mensagens claras.
6. **Código em inglês, documentação em português** (README bilíngue no final).
7. Licença: **Apache-2.0**. Inclua o arquivo `LICENSE` na raiz, o campo `license = "Apache-2.0"` em todos os `Cargo.toml` e no `package.json` do `web/`, e a seção de licença no README.

---

## 4. Arquitetura

```
loopsettle/
├── Cargo.toml                 # workspace
├── .cargo/config.toml         # jobs=2, perfis leves
├── crates/
│   ├── loopsettle-core/          # tipos, ledger, motores de liquidação
│   ├── loopsettle-topology/      # incidências, Hodge (CG), circularidade, netting por ciclos
│   ├── loopsettle-sim/           # gerador sintético calibrado, loop de eventos em streaming
│   ├── loopsettle-cli/           # experimentos reprodutíveis, saída CSV/JSON
│   └── loopsettle-wasm/          # bindings wasm-bindgen para o navegador
├── web/                       # Vite + React + TS, Web Worker, Canvas 2D
├── data/                      # scripts de download dos dados públicos (não versionar dados brutos grandes)
├── experiments/               # definições dos experimentos (TOML)
├── docs/                      # metodologia, PARAMETERS.md, limitações
└── README.md
```

### Conceitos centrais
- **Participante:** instituição com uma conta de liquidação (equivalente à Conta PI) e saldo inicial.
- **Pagamento:** `(id, t_chegada, pagador, recebedor, valor)`, com valor em centavos (`i64`), nunca ponto flutuante para dinheiro.
- **Motor de liquidação** (trait comum `SettlementEngine`):
  1. `GrossRealTime` — liquida cada pagamento na chegada se houver saldo; senão rejeita (comportamento do Pix).
  2. `DeferredNet { cycles_per_day }` — compensação multilateral em horários fixos (estilo UPI, com 10 ciclos por dia como referência).
  3. `AtomicMicroCycle { window_ms, max_delay_ms, mode }` — **o mecanismo novo**: a cada `window_ms`, procura ciclos na fila e liquida cada ciclo de forma atômica (tudo ou nada). Pagamentos que ultrapassarem `max_delay_ms` vão para liquidação bruta. Dois modos:
     - `QueueAll`: todo pagamento passa pela fila.
     - `QueueOnShortfall`: liquida bruto se houver saldo; só enfileira quando falta liquidez (estilo LSM clássico).

---

## 5. Métricas (todas por participante e para o sistema)

- **Volume bruto** liquidado.
- **Liquidez necessária (limite superior, UB):** menor saldo inicial que evita qualquer rejeição em `GrossRealTime` = máximo, ao longo do tempo, de (saídas acumuladas − entradas acumuladas).
- **Liquidez mínima teórica (limite inferior, LB):** posição líquida multilateral do fim do período (o que o `DeferredNet` de um ciclo só exigiria).
- **Economia de liquidez** de cada motor em relação ao UB, em valor e em %.
- **Atrasos:** p50, p95 e p99 do tempo entre chegada e liquidação. Compare com a meta do Manual de Tempos do Pix (50% liquidados em até 2,8 s).
- **Fração circular (Hodge):** norma do componente rotacional ÷ norma do fluxo líquido.
- **Fração compensável (ciclos):** valor cancelado pelo algoritmo de ciclos ÷ volume bruto.
- **Fração circular máxima (CDFD):** maior componente circular possível que respeita conservação de fluxo e o limite de cada aresta, ÷ volume bruto. É o teto realista e estático do que poderia ser compensado.
- **Fração circular proporcional (CDFD local/BFF):** a variante localmente computável descrita no artigo da CDFD, se a implementação couber no orçamento.
- **Escada das três medidas:** Hodge (diagnóstico, pode superestimar) ≥? CDFD máxima (teto estático realista) ≥ ciclos/micro-ciclos (realizável no tempo). Documente sempre as três lado a lado e a distância entre elas; essa escada é um resultado central do estudo.
- **Lacuna diagnóstico vs execução:** diferença entre a fração de Hodge, a de CDFD e a realizada pelos motores.
- **Capacidade potencial vs realizável:** reproduza a medida "potencial" no estilo do estudo do Pix (entradas vs saídas no mesmo dia, sem considerar ordem temporal) e compare com a economia **realizável** do motor `AtomicMicroCycle`. Esta é a comparação central do estudo.

---

## 6. Invariantes (testes de propriedade obrigatórios)

1. **Conservação:** a soma dos saldos de todos os participantes nunca muda.
2. **Posição líquida preservada:** depois de qualquer compensação, a posição líquida de cada participante é igual à de liquidar tudo bruto.
3. **Sem saldo negativo** em `GrossRealTime` e `AtomicMicroCycle`.
4. **Atomicidade:** num micro-ciclo, ou todos os pagamentos do ciclo liquidam, ou nenhum.
5. **Netting nunca inverte direção** de um pagamento nem cancela mais do que o menor pagamento do ciclo.
6. **Hodge:** gradiente + rotacional + harmônico = fluxo original (tolerância numérica); o rotacional tem divergência zero.
7. **Determinismo:** mesma `seed` → mesmos resultados, bit a bit.
8. **Caso do triângulo** (teste fixo): A→B 10, B→C 5, C→A 5 ⇒ netting realizável cancela 5; Hodge atribui 6,67 ao componente circular. Documente essa diferença no teste.

---

## 7. Fases de construção

Cada fase só termina quando os **critérios de aceite** passam. Não pule fases.

### Fase 0 — Ambiente
- Instalar Rust (rustup), target `wasm32-unknown-unknown`, `wasm-pack`, Node LTS.
- Criar o workspace, `.cargo/config.toml` com os perfis leves e um CI simples (GitHub Actions) rodando `cargo test`.
- **Não** crie repositório no GitHub nem publique nada no crates.io agora: trabalhe só localmente, com git na pasta do projeto. A reserva do nome fica para um passo opcional, que eu pedirei quando quiser (ver "Passo opcional — Reserva do nome", no fim deste arquivo).
- **Aceite:** `cargo test` roda em menos de 2 minutos na minha máquina; RAM de pico da compilação < 2,5 GB.

### Fase 1 — Núcleo de liquidação (`loopsettle-core`)
- Tipos, ledger em centavos, motor `GrossRealTime`, cálculo de UB e LB.
- **Aceite:** invariantes 1, 3 e 7 passando; exemplo pequeno à mão documentado.

### Fase 2 — Topologia (`loopsettle-topology`)
- Matriz de incidência nó-aresta (B1) a partir do fluxo líquido agregado entre pares.
- Detecção de triângulos e matriz aresta-triângulo (B2), com limite configurável de triângulos para caber na memória.
- Decomposição de Hodge via CG: gradiente (resolver L0·φ = B1·F), rotacional e harmônico.
- Netting por ciclos (busca de ciclos estilo Bech-Soramäki, cancelando o mínimo do ciclo).
- **CDFD (Circular Directional Flow Decomposition, arXiv 2506.12546):**
  - Leia o artigo e implemente a **decomposição de circularidade máxima**: encontrar a circulação `c` com `0 ≤ c_e ≤ F_e` em cada aresta e conservação em cada nó, maximizando o total circulante; o restante `F − c` é o componente direcional (acíclico).
  - Formule como problema de circulação de custo mínimo (custo −1 por unidade) e resolva com um algoritmo exato implementado à mão (por exemplo, cancelamento de ciclos negativos ou caminhos aumentantes), sem dependências pesadas.
  - Implemente também a variante proporcional/local descrita no artigo, se couber no orçamento; caso contrário, registre em `DECISIONS.md` por que ficou para depois.
  - Use um solver de programação linear em Rust puro (por exemplo, `minilp`) **apenas como oráculo de teste** em grafos pequenos, nunca no caminho principal.
- **Aceite:** invariantes 2, 5, 6 e 8 passando; CDFD bate com o oráculo de PL em 1.000 grafos aleatórios pequenos; componente direcional da CDFD é acíclico; 200 participantes e ~5.000 arestas processados em < 5 s e < 300 MB.

### Fase 3 — Gerador sintético e simulação em streaming (`loopsettle-sim`)
- Participantes com tamanhos heterogêneos (poucos grandes, muitos pequenos — inclua fintechs e instituições de pagamento).
- Chegadas de pagamentos por processo de Poisson com intensidade variando ao longo do dia; matriz de preferência entre pares; distribuição de valores com cauda pesada.
- Execução em streaming: ~1 milhão de pagamentos por execução sem manter todos em memória (amostra escalada; documente o fator de escala).
- **Aceite:** 1 milhão de pagamentos em < 60 s e < 1 GB; resultados idênticos com a mesma seed.

### Fase 4 — Calibração com dados públicos
- Script em `data/` para baixar da API de dados abertos do Pix do BCB (plataforma Olinda): quantidade e volume mensal de transações, ticket médio e chaves por participante.
- Ajustar o gerador para reproduzir os agregados públicos (volume, ticket médio, concentração por participante).
- Registrar cada parâmetro em `docs/PARAMETERS.md` como `SOURCED`, `CALIBRATED` ou `ASSUMPTION`.
- **Aceite:** tabela "real vs simulado" para cada agregado usado, com erro relativo.

### Fase 5 — Motores alternativos
- `DeferredNet` e `AtomicMicroCycle` (dois modos).
- Varredura de parâmetros: `window_ms` ∈ {500, 1000, 2000, 5000, 10000}, `max_delay_ms` ∈ {2000, 5000, 10000}.
- **Aceite:** invariante 4 passando; curva "economia de liquidez × atraso p95" gerada por um comando.

### Fase 6 — Experimentos reprodutíveis (`loopsettle-cli`)
- `experiment baseline` — UB, LB e fração circular no cenário calibrado.
- `experiment potential-vs-realizable` — a comparação central.
- `experiment microcycle-sweep` — a curva de troca.
- `experiment stress` — um participante grande fica fora do ar por 30 minutos.
- `experiment finland-replication` — rodar a lógica do estudo finlandês (ciclos fixos vs instantâneo) no cenário sintético, para comparar ordens de grandeza.
- **Aceite:** cada experimento gera CSV + JSON em `results/<nome>/` com a seed e a versão do código registradas.

### Fase 7 — Interface web (`loopsettle-wasm` + `web/`)
- Bindings mínimos: rodar uma simulação pequena (até ~50 participantes, ~100 mil pagamentos) no navegador.
- Telas: (1) rede com participantes como círculos e fluxos como setas, componente circular destacado em outra cor; (2) controles deslizantes de `window_ms` e `max_delay_ms`; (3) gráficos de liquidez necessária e de atrasos; (4) painel "potencial vs realizável".
- Simulação no Web Worker; interface nunca trava.
- **Aceite:** abre e roda fluido no meu notebook num navegador comum; funciona no celular em modo reduzido.

### Fase 8 — Validação externa (opcional, depois do MVP)
- Rodar a parte de topologia em dados públicos on-chain de transferências de stablecoins (amostra pequena que caiba no orçamento de memória).
- Reproduzir, de forma qualitativa, resultados de estudos com Hodge em fluxos de dinheiro (por exemplo, a correlação entre potencial de Hodge e posição líquida).

### Fase 9 — Documentação e publicação
- README com: pergunta de pesquisa, como rodar em 3 comandos, trabalhos de referência, **limitações** (dados sintéticos calibrados, não dados interbancários reais).
- `docs/METHODOLOGY.md` explicando cada métrica em linguagem simples e formal.
- Preparar o repositório para Zenodo (DOI) e, mais tarde, submissão ao JOSS.

### Fase 10 — Pacote de revisão (entrega final)
Não escreva o artigo. Prepare o material para que eu e meu revisor analisemos cada ponto e escrevamos juntos:
- `REVIEW.md` com uma lista numerada de **todos os pontos a revisar**, cada um com: o que foi feito, onde está no código, qual resultado produziu e qual o risco ou a dúvida. Inclua obrigatoriamente: cada entrada de `DECISIONS.md`, cada `ASSUMPTION` de `PARAMETERS.md`, as limitações conhecidas e qualquer teste que tenha sido difícil de passar.
- `results/SUMMARY.md` com os números principais de cada experimento, em tabelas, e as figuras em PNG/SVG geradas por comando.
- `docs/CLAIMS.md` listando as afirmações que o estudo **pode** fazer com base nos resultados e as que **não pode**, com a evidência de cada uma.
- `docs/NOVELTY.md` comparando, ponto a ponto, o que o LoopSettle faz e o que fazem os trabalhos de referência (Banco da Finlândia, estudo do Pix/BCB, artigo da CDFD, BoF-PSS, PSSimPy, PixSim), para sustentar a seção de contribuição do artigo.
- Um esqueleto do artigo (`docs/PAPER_OUTLINE.md`) apenas com títulos de seções e a lista de figuras e tabelas que entram em cada uma.
- **Aceite:** um comando (`make all` ou script equivalente) reconstrói do zero todos os resultados, figuras e o resumo; `cargo test` passa; a interface web abre e roda.

---

## 8. Como quero que você trabalhe comigo

- Eu sou engenheiro de software backend (Go) e estou aprendendo Rust: **explique as decisões de Rust que não forem óbvias**, sem excesso.
- Escolhas de desenho que mudem os resultados (por exemplo, como modelar a fila) seguem a regra da seção 0: decida, registre em `DECISIONS.md` e deixe configurável quando possível.
- Se algo estourar o orçamento de memória ou de tempo da minha máquina, reduza a escala de forma documentada; só pare se for impossível cumprir o orçamento.
- No fim de cada fase, atualize `PROGRESS.md` com: o que foi feito, como testar, números medidos (tempo e RAM) e o que vem na próxima fase.

Comece pela **Fase 0**: verifique meu sistema operacional, as ferramentas instaladas e a memória disponível. Se faltar algo que você consiga instalar, instale; se não conseguir, me diga exatamente o que instalar. Depois siga pelas fases até a Fase 10 sem parar.

---

## Passo opcional — Reserva do nome (só quando eu pedir)

Não faça este passo durante a construção. Só quando eu disser **"vamos reservar o nome"**:
- Criar o repositório público `loopsettle` no meu GitHub e enviar o código, usando a ferramenta `gh`. Me guie em passos simples e numerados para eu autorizar o login no navegador.
- Publicar a versão `0.0.1` do crate `loopsettle` no crates.io, com conteúdo real mínimo, descrição, licença e link do repositório. Antes, confirme que o nome continua livre (`cargo search loopsettle`).
- Me explique, um passo de cada vez, como entrar em crates.io com minha conta do GitHub, confirmar meu e-mail, gerar um token de publicação e colá-lo quando você rodar `cargo login`. Nunca grave o token em arquivos do projeto nem em commits.
