# Esqueleto do artigo

> [English version](PAPER_OUTLINE.md)
>
> Reescrito em 05/11/2026 em torno da afirmação central revista ([`ASSESSMENT.pt-BR.md`](ASSESSMENT.pt-BR.md),
> atualização 3). Só títulos de seções e as figuras/tabelas de cada uma; o texto será escrito com o
> revisor.

**Título provisório:** *Quanta liquidez a compensação consegue recuperar em pagamentos
instantâneos? Um limite de atraso e evidências de um simulador calibrado com o Pix.*

## Preprint (SSRN / arXiv q-fin)

### 1. Introdução
- A pergunta; as quatro contribuições: o limite de atraso; potencial × realizável medido com
  robustez; a janela de abastecimento do fim de semana; uma ferramenta aberta que roda em dados reais.

### 2. Pagamentos instantâneos e sua liquidez
- Tabela 1 — Modelos de liquidação entre bancos de 11 sistemas ([`INSTANT_PAYMENTS.pt-BR.md`](INSTANT_PAYMENTS.pt-BR.md)).
- 2.1 Pix: SPI, Conta PI, janela do STR, remuneração da Conta PI (relatório do SPI 2024; relatório de gestão do Pix 2023).
- 2.2 Trabalhos relacionados: Hellqvist & Korpinen (2021/2026); BCB WP 619; Johnson, McAndrews &
  Soramäki (2004); McMahon et al. (2024); Bank of Canada SDP 2020-9 e 2022-3; Hellqvist (2024).
- Tabela 2 — Comparação ponto a ponto ([`NOVELTY.pt-BR.md`](NOVELTY.pt-BR.md)).

### 3. Modelo
- 3.1 Participantes, pagamentos, ledger; 3.2 gerador e calibração
  (Tabela 3 — parâmetros e status; Tabela 4 — real × simulado, incluindo os quartis por banco do WP 619);
- 3.3 Motores de liquidação (bruto, compensação diferida, micro-ciclos atômicos); 3.4 medidas de
  liquidez (UB, LB, janelas de abastecimento).

### 4. O limite de atraso
- O piso de liquidez: teorema e demonstração ([`METHODOLOGY.pt-BR.md`](METHODOLOGY.pt-BR.md) §9b), com a
  forma de janela (§9) como corolário; Figura 1 — economia obtida × limites apertado e de janela em
  função de d (`regimes/fig_bound.svg`); Tabela 5 — o limite por semente e por viés recebedor
  (`robustness/seeds_summary.csv`).
- Sai desta seção: "o limite separa o varejo do alto valor". O limite apertado é ≈ 0 nos dois
  (`regimes/regimes.csv`); a comparação com os estudos de LSM em alto valor depende do experimento com
  liquidez restrita (`specs/06`) e, até lá, vai para a discussão como ponto em aberto.

### 5. Resultados
- 5.1 Potencial × realizável, com sementes, escala e comprimento do ciclo — Tabela 6
  (`robustness/seeds_summary.csv`), Figura 2 (`robustness/fig_scale.svg`).
- 5.2 Estrutura bilateral calibrada e heterogeneidade intradiária — Tabela 7 (`heterogeneity/heterogeneity.csv`).
- 5.3 Janelas de abastecimento: noites e fim de semana — Tabela 8 (`funding-windows/windows.csv`).
- 5.4 Heterogeneidade por banco, comparada ao estudo finlandês — Tabela 9 (`robustness/participants_ratio.csv`).
- 5.5 **Quando o resultado falha: devoluções bilaterais rápidas** — Tabela 10 (`echo-sweep/echo.csv`). É a
  condição da afirmação central; entra no resumo.
- 5.6 Liquidez restrita com serviço igual, 10 s × uma hora — Tabela 11 (`liquidity-frontier/equivalent_liquidity.csv`),
  Figura 3 (`liquidity-frontier/fig_frontier.svg`).
- 5.7 Pisos × ótimo offline — Tabela 12 (`optimality-gap/gap.csv`).

### 6. Discussão
- Política: LSMs de segundos em sistemas instantâneos; onde a liquidez de fato restringe (fins de
  semana); quem arca com ela (instituições de pagamento sem acesso a reservas nem a redesconto); o que
  um banco central consegue medir com o `loopsettle bound`.
- Limitações ([`LIMITATIONS.pt-BR.md`](LIMITATIONS.pt-BR.md)); o que não pode ser afirmado ([`CLAIMS.pt-BR.md`](CLAIMS.pt-BR.md)).

### 7. Conclusão

### Apêndices
- A. Invariantes e testes de propriedade; B. Reprodutibilidade (`make all`); C. Lentes de circularidade
  (Hodge, CDFD, ciclos) e o valor nulo de Hodge; D. Rodar em dados reais.

## Artigo do JOSS (software)
- Resumo; Declaração de necessidade (simulador aberto e reprodutível de liquidação de pagamentos
  instantâneos, que roda em arquivos reais e no navegador); Funcionalidades (motores, limite,
  experimentos, CLI, interface web); Qualidade (testes de propriedade, CI, reprodutibilidade byte a
  byte); Referências.
