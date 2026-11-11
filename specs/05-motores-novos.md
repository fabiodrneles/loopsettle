# 05 — Motores novos (P1)

## Contexto

O LoopSettle tem três desenhos: bruto sem fila, compensação diferida e micro-ciclos. Para ser
referência ele precisa cobrir os mecanismos que a literatura e os sistemas reais usam, todos
atrás da mesma trait `SettlementEngine` e sob as mesmas invariantes. A ordem abaixo é de valor
científico por esforço.

## M1 — `QueuedGross` (FIFO com fila): o baseline da literatura de LSM — **FEITO em 11/11/2026**

> Implementado como `queue:fifo|bypass:<prazo>[:<nova_tentativa_ms>]`: o motor de micro-ciclos em
> `QueueOnShortfall` com `search_cycles = false`. A liberação é por janela de nova tentativa
> (100 ms por padrão), não por evento; trocar por evento continua em aberto. Testes:
> `queue_equals_gross_when_liquidity_suffices`, `queue_under_constrained_liquidity_is_consistent`,
> e as invariantes por propriedades com `search_cycles` sorteado.

Bruto com fila quando falta saldo; libera quando chega recebimento. É o "FIFO" contra o qual
McMahon et al. (2024), Denbee & McLafferty (2012) e os estudos com o BoF-PSS medem economia.

- Parâmetros: `discipline {Fifo, BypassFifo}`, `max_delay_ms: Option` (None = espera até o fim).
- É o `AtomicMicroCycle` em `QueueOnShortfall` sem a busca de ciclos e com liberação por evento
  (na chegada de um crédito ao pagador), não por janela. Implementar depois da correção C1/C2.
- Spec de motor: `queue:fifo|bypass[:max_delay_ms]`.
- **AC:** com liquidez ilimitada é idêntico ao `GrossRealTime` (teste); com liquidez = UB
  nunca enfileira; conservação, sem saldo negativo, determinismo (proptest).

## M2 — `HoldToDeadline`: o motor que atinge o piso de um participante

Segura todo pagamento por exatamente `d` e liquida bruto. Trivial de implementar e útil como
instrumento: mostra que só atrasar, sem compensar, já muda a liquidez, e separa o efeito
"atraso" do efeito "compensação" nos micro-ciclos.

- Variante `hold_only: Vec<ParticipantId>`: só os participantes listados seguram. Com um
  participante só, a liquidez dele tem de ser **igual** ao `piso_i(d)`.
- **AC:** teste de igualdade `L_i = piso_i(d)` para o participante único que segura, em 5
  cenários. Isso fecha a prova de que o piso é atingível (spec 01).

## M3 — Compensação parcial multilateral no estilo Bech-Soramäki

O algoritmo do artigo de 2001 resolve gridlock achando o maior subconjunto de pagamentos na
fila, respeitando FIFO por banco, que liquida simultaneamente sem saldo negativo: começa com
todos, e enquanto algum banco ficar negativo remove o último pagamento da fila dele.

- Roda a cada `window_ms` sobre a fila do `QueuedGross`.
- Difere dos micro-ciclos: liquida lotes que não são ciclos e usa o saldo disponível.
- **AC:** exemplo do artigo reproduzido em teste; invariantes 1 a 5; nunca liquida menos valor
  que a liberação bruta simples no mesmo instante (propriedade).
- Ler o artigo em `docs/references/` antes; a descrição acima é de memória e precisa ser
  conferida (`HIPÓTESE` quanto aos detalhes).

## M4 — Compensação bilateral contínua (offsetting)

Na chegada de `A→B`, procura na fila um `B→A` e liquida os dois pelo líquido. É o mecanismo de
vários RTGS (por exemplo o T2). Caso particular dos micro-ciclos com `max_cycle_len = 2` e
gatilho por evento. **AC:** igual ao micro-ciclo de tamanho 2 quando `window_ms → 1`.

## M5 — Política de abastecimento (funding) como camada

Hoje a liquidez é medida com saldo ilimitado ou fixada na abertura. Falta o que os
participantes fazem de fato: aportar e retirar durante o dia.

- `FundingPolicy` (trait): chamada em instantes definidos, pode creditar ou debitar a conta de
  liquidação contra uma conta de reservas externa (fora da conservação do sistema: a invariante
  1 passa a valer para conta + reservas).
- Políticas: `None`; `TopUpInWindow { open_ms, close_ms, target }` (aporta até a meta só com o
  STR aberto, como a D20 descreve); `IntradayCredit { limit }` (saldo pode ficar negativo até o
  limite, com contagem de uso).
- **AC:** `funding-windows` reescrito sobre essa camada dá os mesmos números de hoje com a
  política equivalente (teste de regressão), e ganha a métrica "valor rejeitado por falta de
  pré-financiamento" para metas abaixo da necessidade.
- Decisão a registrar: como tratar a remuneração e o custo de oportunidade (fora do escopo
  até haver fonte).

## Requisitos comuns

- Cada motor: testes de propriedade das invariantes 1, 3, 7 e, quando houver lote, 2, 4, 5.
- Cada motor com prazo: entra em `engines_never_beat_the_delay_bound` (piso e janela).
- `EngineSpec::parse`/`to_json`, `loopsettle list`, `docs/METHODOLOGY*.md` e
  `docs/INSTANT_PAYMENTS*.md` (qual sistema real usa qual mecanismo, com fonte).
- Nenhuma dependência nova.

## Como validar antes de implementar

Ler as referências de cada mecanismo em `docs/references/` e `docs/REFERENCES*.md`; se um
artigo não estiver na biblioteca, pedir ao dono (não descrever de memória num documento
público).
