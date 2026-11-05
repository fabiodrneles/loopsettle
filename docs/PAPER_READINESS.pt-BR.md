# PRONTIDÃO PARA O ARTIGO — especificação de tudo o que falta antes de escrever

> [English version](PAPER_READINESS.md)
>
> Decorre de [`ASSESSMENT.pt-BR.md`](ASSESSMENT.pt-BR.md) (veredito: PIVOTAR). Cada item tem um
> responsável (**Claude** = dá para fazer numa sessão; **Você** = depende de você), um critério de
> aceite e uma situação. O estudo está "pronto para escrever" quando todos os itens
> *obrigatórios* estiverem feitos e nenhum critério de morte (K1–K4) tiver disparado.

## Quadro de situação

| ID | Frente | Obrigatório | Responsável | Situação |
|---|---|---|---|---|
| W0 | Fontes que só você consegue baixar | sim | Você | **4 de 15 recebidas** (nº 1, 4, 5 e 6 — lidas e registradas no `REFERENCES.pt-BR.md`) |
| W1 | Higiene dos resultados (várias sementes, faixas sobre σ, sem "630×") | sim | Claude | **feito** nesta mudança (experimento de robustez, CLAIMS, README) |
| W2 | Modelo de heterogeneidade intradiária (critério K1) | sim | Claude (+ você para dados) | a fazer |
| W2b | Calibrar o viés recebedor na distribuição por banco do WP 619 (p25 18–45%, mediana 57–80%) | sim | Claude | a fazer |
| W3 | Liquidez necessária como as regras do BCB definem (K3) | sim | Claude, depois de W0 | a fazer |
| W4 | Revisão de literatura sobre a novidade (K2) | sim | Claude + você | a fazer |
| W5 | Limite de atraso: teorema, curva em d, folga | sim | Claude | teorema e teste **feitos**; curva a fazer |
| W6 | Fraquezas restantes (FIFO estrito, estresse, nulo de Hodge) | em parte | Claude | a fazer |
| W7 | Empacotamento do artigo e do software (esqueleto, JOSS, preprint, Zenodo) | sim | Claude + você | a fazer |
| W8 | Rodar a interface no notebook de referência (i3, 4 GB) | não | Você | a fazer |

## W0 — Fontes para baixar e me enviar

Estes sites estão bloqueados no ambiente da nuvem. Baixe cada PDF e **me envie aqui no chat**
(como anexo). Não coloque os PDFs no repositório: a maioria tem direitos autorais. Para cada um,
está listado o que preciso tirar dele.

| # | Documento | Link | O que preciso dele |
|---|---|---|---|
| 1 | Hellqvist & Korpinen (2021), BoF Economics Review 7/2021 | [PDF](https://publications.bof.fi/bitstream/10024/43594/1/BoFER_7_2021.pdf) | o aumento agregado (é 8,6%?), número de bancos e de dias, quantos ciclos por dia no modelo de referência do STEP2, a definição exata de liquidez necessária, os resultados por banco |
| 2 | Hellqvist & Korpinen (2026), JFMI 13(2) | [Risk.net](https://www.risk.net/node/7963364) (pago; só se você tiver acesso) | o que mudou em relação à versão de 2021 |
| 3 | Slides do debatedor, Economics of Payments X | [PDF](https://www.suomenpankki.fi/globalassets/en/research/seminars-and-conferences/conferences-and-workshops/documents/eopx2021/alexandrova-kabadjova_discussion_instant-payments-as-a-new-normal_presentation.pdf) | de onde vem o número de 8,6% |
| 4 | Gonzalez, Ma & Zeng (2025), BCB WP 619 | [PDF](https://www.bcb.gov.br/content/publicacoes/WorkingPaperSeries/WP619.pdf) | a definição exata da medida de compensação perdida e seus números (os 57–80% citados antes) — o "potencial" do LoopSettle tem de bater com ela ou dizer em que difere |
| 5 | Homs-Dones et al. (2025), CDFD | [PDF no arXiv](https://arxiv.org/pdf/2506.12546) | a definição de circularidade máxima (confirmar que é igual à nossa formulação de custo mínimo) e o algoritmo BFF (D6) |
| 6 | Johnson, McAndrews & Soramäki (2004), FRBNY EPR | [PDF](https://www.newyorkfed.org/medialibrary/media/research/epr/04v10n3/0412mcanpdf.pdf) | economia × atraso para compensação de 1 e 6 horas e para o RRGS; se enunciam algum limite de atraso |
| 7 | Martin & McAndrews (2008), JME / FRBNY SR 282 | [página no RePEc com link para o PDF](https://ideas.repec.org/p/fip/fednsr/282.html) | o argumento de bem-estar dos LSMs, para posicionar o nosso |
| 8 | Denbee & McLafferty (2012), simulação de LSM no CHAPS | [IGI Global](https://igi-global.com/chapter/content/68694) · [BoE Financial Stability Paper 7 (visão geral)](https://ideas.repec.org/p/boe/finsta/0007.html) | a curva economia × atraso no CHAPS |
| 9 | Bank of Canada WP 06-20 (troca atraso × liquidez no LVTS) | [RePEc](https://ideas.repec.org/p/bca/bocawp/06-20.html) | a curva de troca, para comparar com a nossa |
| 10 | DNB WP 682, liquidez economizada por um LSM | [PDF](https://www.dnb.nl/media/m5xdz2nx/working-paper-no-682_tcm47.pdf) | método e resultados |
| 11 | Bech & Soramäki (2001), BoF DP 9/2001 | [RePEc](https://ideas.repec.org/p/zbw/bofrdp/rdp2001_009.html) | o algoritmo de resolução de gridlock, para a seção de métodos |
| 12 | BCB — regulamentação do Pix e regras do SPI | [página do Pix no BCB](https://www.bcb.gov.br/estabilidadefinanceira/pix) → *Regulamentação* (Regulamento do Pix, Resolução BCB nº 1/2020; regras do SPI e da Conta PI) | como a Conta PI é abastecida (transferências das Reservas, quando), o que acontece à noite e nos fins de semana, se existe crédito intradiário para o Pix |
| 13 | BCB — horário do STR | [página do STR no BCB](https://www.bcb.gov.br/estabilidadefinanceira/str) → *Grade horária* | a janela em que são possíveis transferências Conta PI ↔ Reservas |
| 14 | BCB — Manual de Tempos do Pix | [página do Pix](https://www.bcb.gov.br/estabilidadefinanceira/pix) → *Manuais* | confirmar a meta de 2,8 s / 50% usada nos resultados |
| 15 | BCB — estatísticas do Pix por hora do dia, se publicadas | [estatísticas do Pix](https://www.bcb.gov.br/estabilidadefinanceira/estatisticaspix) · [dados abertos Olinda (o endereço que o `data/fetch_pix.py` usa)](https://olinda.bcb.gov.br/olinda/servico/Pix_DadosAbertos/versao/v1/odata) | qualquer distribuição intradiária (por hora, por tipo de participante) para calibrar a W2 |

Os links do BCB apontam para as páginas de seção; se algum tiver mudado de lugar, procure o
título do documento no site do BCB.

Opcional, para comparar com outro país: a descrição do SPEI pelo Banxico
([PDF](https://www.banxico.org.mx/sistemas-de-pago/d/%7B89B6CCF0-6070-7389-3DD5-B27AC4ECD9D1%7D.pdf))
e o material da Payments Canada sobre o RTR ([página](https://www.payments.ca/how-canadas-real-time-rail-will-transform-account-account-payments)).

**Aceite:** todos os itens "a conferir" de [`REFERENCES.pt-BR.md`](REFERENCES.pt-BR.md)
resolvidos, com o número da página da fonte.

## W1 — Higiene dos resultados *(feito)*

- Todo número de destaque vem com sementes e IC de 95% (`results/robustness/seeds_summary.csv`).
- Os resultados são reportados como faixas sobre σ ∈ [0; 0,8], não só em σ = 0,4.
- A manchete "potencial de 9,27% contra realizável de 0,0147% (630×)" saiu do README e do
  CLAIMS; a dependência da escala está dita.
- **Aceite:** todo número de destaque do README e do CLAIMS é média de várias sementes com IC; as linhas de semente única da tabela do README estão marcadas. ✔ (o esqueleto é reescrito na W7)

## W2 — Heterogeneidade intradiária (critério de morte K1)

- **O quê:** perfis intradiários próprios de cada participante, no lugar de um perfil horário
  único. Classes a modelar: pagadores de folha (rajadas em horários e dias fixos), credenciadoras
  e instituições de pagamento (entradas durante o dia, saídas em lotes de liquidação), fintechs
  abastecendo a Conta PI, grandes bancos. Parâmetros marcados `ASSUMPTION`, salvo se calibrados
  com a fonte W0 nº 15.
- **Por quê:** chegadas estacionárias fazem o potencial cair com 1/√N (`robustness/scale.csv`);
  desequilíbrios sistemáticos não cairiam. É a principal ameaça aberta do lado do potencial.
- **Como:** uma tabela opcional `profiles` no TOML do cenário (por classe, 24 ou 96 pesos),
  mantendo o comportamento atual como padrão para que os resultados existentes continuem
  idênticos byte a byte.
- **Aceite:** `robustness` rodado de novo com perfis heterogêneos, de 1 a 30 milhões de
  pagamentos; reportar o limite de 10 s e a economia obtida. **K1 dispara** se o limite de 10 s
  passar de 2% do UB com 30 milhões.

## W3 — Liquidez necessária como as regras do BCB definem (critério de morte K3)

- **O quê:** depois das fontes W0 nº 12–13, modelar as janelas de abastecimento da Conta PI:
  períodos em que transferências das Reservas são possíveis (STR aberto) e períodos em que não são
  (noites, fins de semana, feriados). A liquidez necessária passa a ser o maior esvaziamento
  *dentro de cada janela fechada*, mais os aportes necessários nas janelas abertas. Rodar por
  vários dias, incluindo um fim de semana.
- **Por quê:** o UB de um dia, com saldo de abertura zero, pode não ser a restrição que os bancos
  realmente enfrentam.
- **Aceite:** um experimento `funding-windows` com o UB por tipo de janela; a tabela central
  recalculada com essa definição. **K3 dispara** se os picos intradiários dentro das janelas
  abertas nunca forem a restrição (sempre cobertos com Reservas).

## W4 — Revisão de literatura (critério de morte K2)

- **O quê:** uma busca organizada por estudos de mecanismos de economia de liquidez ou de
  compensação com prazos de segundos em sistemas de pagamento instantâneo, e por qualquer
  enunciado do limite de atraso. Onde procurar: publicações do BIS/CPMI, BCE (TIPS, RT1), Bank of
  Canada (RTR), Federal Reserve (FedNow), Bank of England (RT2, Faster Payments), anais do seminário
  do simulador BoF-PSS desde 2018, JFMI, SSRN.
- **Aceite:** uma tabela de trabalhos relacionados no [`NOVELTY.pt-BR.md`](NOVELTY.pt-BR.md) com os
  cinco trabalhos mais próximos e o que cada um não faz. **K2 dispara** se um deles já reportar a
  economia de LSMs com prazos de segundos num sistema instantâneo, ou já enunciar o limite.

## W5 — Limite de atraso

- **Feito:** teorema e demonstração ([`METHODOLOGY.pt-BR.md`](METHODOLOGY.pt-BR.md#9-o-limite-de-atraso),
  seção 9); teste participante a participante contra os motores de micro-ciclos e de compensação
  diferida (`crates/loopsettle-sim/src/bound.rs`).
- **A fazer:** uma curva da economia obtida e do limite para d ∈ {1 s, 10 s, 1 min, 15 min, 1 h,
  6 h, 1 dia}; uma razão de folga (obtido / limite) com IC; a figura entra na seção de resultados
  do artigo.
- **Aceite:** `fig_bound.svg` gerado por comando, em `results/robustness/`.

## W6 — Fraquezas restantes

| Item | O quê | Obrigatório |
|---|---|---|
| Nulo de Hodge | reportar (m − n + 1)/m ao lado de cada número de Hodge, ou tirar Hodge da escada principal | sim |
| FIFO estrito | FIFO estrito por pagador, opcional, no `QueueOnShortfall` (REVIEW 14) | não |
| Estresse | indisponibilidade no horário de pico do participante, ou com abandono parcial (REVIEW 39) — ou tirar do artigo | sim (um dos dois) |
| CDFD BFF | CDFD proporcional (D6), depois da fonte W0 nº 5 | não |
| On-chain | validação externa com transferências de stablecoins (Fase 8) | não |

## W7 — Empacotamento

- Reescrever o [`PAPER_OUTLINE.pt-BR.md`](PAPER_OUTLINE.pt-BR.md) em torno da nova afirmação
  central (limite + distância medida + por que as medidas de potencial enganam).
- `paper/paper.md` para o JOSS (resumo, declaração de necessidade, referências), `CITATION.cff`
  v0.2.0, release no Zenodo com DOI.
- Preprint: SSRN (economia) ou arXiv q-fin.GN, depois de W2–W4.
- **Aceite:** `make all` reproduz todos os números do preprint; checklist do JOSS atendido.

## W8 — Notebook de referência

Abrir a interface web (`cd web && npm install && npm run wasm && npm run dev`) no notebook i3 /
4 GB e anotar o tempo de carregamento e a fluidez. **Você.**
