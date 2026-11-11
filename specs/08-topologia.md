# 08 — Topologia: BFF, escala e circularidade temporal (P2)

## T1 — CDFD proporcional / local (BFF), pendência D6

O ASSESSMENT registra que o Algoritmo 1 do artigo (arXiv 2506.12546, §5.2) está totalmente
especificado e que a D6 já pode ser implementada.

- Ler o artigo (só há link: `docs/REFERENCES*.md`; a licença não permite guardar o PDF) e
  transcrever o algoritmo em pseudocódigo para `docs/METHODOLOGY*.md` antes de codar.
- `loopsettle-topology/src/cdfd_local.rs`: `cdfd_bff(g) -> Cdfd` com a mesma struct de saída.
- **AC:** (a) conservação em cada nó e `0 ≤ c_e ≤ F_e` (proptest); (b) `circular_total(BFF) ≤
  circular_total(máxima)` em 1.000 grafos aleatórios; (c) os exemplos numéricos do artigo, se
  houver, reproduzidos em teste com a página citada; (d) `Ladder` e `ladder.csv` ganham a
  coluna `cdfd_bff`; (e) 200 nós e 5.000 arestas em < 5 s e < 300 MB.
- A escada passa a ter quatro degraus; atualizar a afirmação 4 de `CLAIMS*.md` e remover o
  item 8 de "não pode afirmar" só na parte que trata da variante proporcional.

## T2 — Escala do fluxo de custo mínimo (A14)

- Benchmark novo: grafos aleatórios e livres de escala com 10⁴, 10⁵ e 10⁶ arestas e valores
  de cauda pesada; medir tempo, aumentos e RAM de `cdfd_max`.
- Se 10⁵ arestas não couber em 60 s: trocar caminhos mínimos sucessivos por escalonamento de
  capacidade, ou resolver por componente fortemente conexa (a circulação máxima só existe
  dentro de cada componente; arestas entre componentes são 100% direcionais). A decomposição
  por componentes é barata (Tarjan) e exata: fazer primeiro.
- **AC:** resultado idêntico ao atual em todos os testes existentes e no oráculo de PL; tempo
  medido registrado em `CHECKPOINT*.md`.

## T3 — Circularidade temporal: a ponte entre a escada e os motores

Hoje a escada (Hodge ≥ CDFD ≥ ciclos) é calculada no grafo agregado do dia, e a economia dos
motores é medida no tempo. As duas não conversam: o grafo agregado tem 80% de fluxo circular e
os micro-ciclos economizam 0,01% de liquidez. Falta a medida que liga uma à outra.

- **Curva de circularidade por horizonte:** agregar o fluxo em janelas de largura `w`
  (10 s, 1 min, 15 min, 1 h, 6 h, 1 dia), calcular a CDFD máxima em cada janela e somar:
  `C(w) = Σ_janelas circular_total / volume bruto`. `C(1 dia)` é o degrau atual da escada;
  `C(10 s)` é o teto do que ciclos dentro de 10 s poderiam cancelar em **valor**.
- Em streaming: a matriz de pares da janela é esparsa para `w` pequeno; usar mapa esparso por
  janela e descartar ao fechar. Para `w` grande cai no caso atual (matriz densa n²).
- **Valor cancelado não é liquidez economizada.** Reportar `C(w)` ao lado do limite apertado
  para o mesmo `w`: a diferença entre os dois é o resultado (muito valor circula, pouca
  liquidez se recupera) e é a figura que explica o estudo para quem não é da área.
- **AC:** `experiment circularity-horizon` gera `horizon.csv` e `fig_horizon.svg` com 10
  sementes; `C(w)` é não decrescente em `w` quando as janelas são aninhadas (teste; provar ou
  achar contraexemplo antes de afirmar no texto); memória < 300 MB.

## T4 — Saída padrão do comando `topology` com nulo e incerteza

- `loopsettle topology <csv>` passa a imprimir, para cada medida, o valor do nulo de
  recebedores embaralhados (spec 07 G4) com média e desvio de 20 embaralhamentos, e o nulo
  analítico de Hodge `(m − n + 1)/m`.
- **AC:** no triângulo de `data/samples/triangle.csv` os números atuais não mudam; a seção 2
  de `results/SUMMARY*.md` ganha as colunas de nulo.

## T5 — Validação externa on-chain (Fase 8, pendente)

Só depois de T2 (escala). Uma amostra pública de transferências de stablecoin, com script em
`data/`, sem versionar o dado bruto, ≤ 1 GB de RAM. Registrar fonte, data e licença. Se o
ambiente não tiver acesso de rede ao provedor, deixar o script e as instruções, e marcar como
não rodado (não inventar números).

## Como validar antes de implementar

```bash
cargo test -p loopsettle-topology
```

```bash
cargo run --release -p loopsettle-topology --example bench_topology
```
