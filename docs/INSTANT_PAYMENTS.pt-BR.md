# Pagamentos instantâneos pelo mundo — onde o LoopSettle se aplica

> [English version](INSTANT_PAYMENTS.md)
>
> O LoopSettle é calibrado com o Pix porque o Pix publica os agregados de que o gerador precisa.
> A pergunta do estudo — quanto da liquidez que um sistema de pagamentos instantâneos prende é
> circular, e quanto disso dá para recuperar de fato — vale para qualquer país com pagamentos
> instantâneos. Esta página mostra o modelo de liquidação de cada sistema e qual motor do
> LoopSettle o representa.
>
> Última conferência: 05/11/2026, por busca pública na web; SPEI, Canadá e FAST conferidos em fontes primárias (Banxico, Interac, Bank of Canada, MAS). Linhas
> marcadas "fontes secundárias" ainda devem ser confirmadas no regulamento do operador antes de
> serem citadas.

## A diferença que importa: como os bancos acertam as contas entre si

Para o cliente, todos os sistemas abaixo são instantâneos: o recebedor é creditado em segundos,
24 horas por dia. O que muda a **liquidez** que os bancos precisam manter é como eles liquidam
entre si:

| Modelo entre bancos | O que significa | Motor do LoopSettle |
|---|---|---|
| **Liquidação bruta em tempo real (LBTR/RTGS)** | cada pagamento liquida sozinho, na hora, em dinheiro do banco central; o banco pagador precisa ter o saldo *antes* de cada pagamento | `GrossRealTime` |
| **Bruta, contra conta conjunta pré-financiada** | como acima, mas contra um fundo que os bancos depositam antes no banco central | `GrossRealTime` com saldo inicial limitado (modo de liquidez restrita) |
| **Compensação diferida (DNS)** | os bancos trocam os pagamentos na hora, mas liquidam só as posições *líquidas*, em alguns ciclos por dia (muitas vezes pré-financiados ou com garantia) | `DeferredNet { ciclos_por_dia }` |
| **Compensação a cada poucos segundos** | um algoritmo de compensação roda em intervalos curtos e liquida os pagamentos que os saldos do momento permitem | `AtomicMicroCycle` (o análogo mais próximo) |

O resultado central do LoopSettle — *volume compensável não é liquidez economizada* — diz
respeito aos sistemas dos dois primeiros grupos, que são os que se perguntam se vale adicionar um
mecanismo de economia de liquidez. Para os do terceiro grupo, o experimento
`finland-replication` responde à pergunta inversa: quanta liquidez a mais a migração para o
modelo bruto exigiria.

## Sistemas

| Sistema | País / região | Operador | Em operação desde | Liquidação entre bancos | Motor | Status |
|---|---|---|---|---|---|---|
| **Pix** (SPI) | Brasil | Banco Central do Brasil | nov/2020 | bruta em tempo real nas Contas PI dos participantes no banco central; o saldo não pode ficar negativo | `GrossRealTime` | confirmado |
| **FedNow** | Estados Unidos | Federal Reserve | jul/2023 | bruta em tempo real nas *master accounts* do Federal Reserve | `GrossRealTime` | confirmado |
| **RTP** | Estados Unidos | The Clearing House | nov/2017 | bruta em tempo real, pré-financiada, contra uma conta conjunta no Fed de Nova York | `GrossRealTime` (saldo limitado) | confirmado |
| **TIPS** (SCT Inst) | Zona do euro | Eurosistema (BCE) | nov/2018 | bruta em tempo real em dinheiro do banco central em euro, cada pagamento em menos de 10 s | `GrossRealTime` | confirmado |
| **NPP** + Fast Settlement Service | Austrália | NPP Australia / Reserve Bank of Australia | fev/2018 | liquidação individual em tempo real de cada pagamento nas contas de liquidação, sem compensação | `GrossRealTime` | confirmado |
| **FAST** | Singapura | Banking Computer Services (operador) / MAS (liquidação) | mar/2014 | instantâneo para o cliente; compensação diferida: a câmara compensa as obrigações do FAST e as liquida no MEPS+ (o sistema bruto do MAS) em dois ciclos por dia útil, contra garantias depositadas no MAS | `DeferredNet { 2 }` | confirmado (MAS, Banco Mundial) |
| **SPEI** | México | Banco de México | ago/2004 | liquidação por um algoritmo de compensação que escolhe, repetidamente, os conjuntos de pagamentos que podem ser liquidados com os saldos do momento — em média a cada 1,9 s, dentro do próprio sistema, em dinheiro do banco central (Banxico) | `AtomicMicroCycle` (análogo) | confirmado (Banxico) |
| **UPI** | Índia | NPCI / Reserve Bank of India | 2016 | compensação diferida: a NPCI calcula as posições líquidas e as liquida no sistema bruto do RBI, 10 ciclos por dia para transações autorizadas | `DeferredNet { 10 }` | confirmado (fontes secundárias) |
| **Faster Payments** | Reino Unido | Pay.UK / Bank of England | mai/2008 | compensação diferida no Bank of England, três ciclos por dia útil, pré-financiada | `DeferredNet { 3 }` | confirmado (fontes secundárias) |
| **Interac e-Transfer** | Canadá | Interac Corp. | 2003 | instantâneo para o cliente; compensação diferida em ciclos de 24 horas: a Interac calcula o valor de cada participante e os participantes pagam uns aos outros pelo Lynx (o sistema bruto do Canadá); migração para o RTR prevista | `DeferredNet { 1 }` | confirmado (divulgação da Interac, 2025) |
| **Real-Time Rail (RTR)** | Canadá | Payments Canada | ainda não opera: lançamento em fases previsto para o 4º trimestre de 2026 | liquidação em tempo real contra contas de liquidação pré-financiadas no Bank of Canada, abastecidas e esvaziadas pelo Lynx | `GrossRealTime` (saldo limitado) | confirmado (política do Bank of Canada); lançamento pendente |

## O que isso significa e o que não significa

- **Significa** que os motores e as métricas do LoopSettle se aplicam a esses sistemas: os
  motores `GrossRealTime`, `DeferredNet` e `AtomicMicroCycle` são os mecanismos acima.
- **Não significa** que os números do LoopSettle valem para eles. O cenário é calibrado só com
  agregados do Pix (volume, ticket, participação do B2B, concentração). Valores de pagamento,
  perfil horário e concentração de participantes mudam de país para país. Recalibrar para outro
  sistema exige os agregados públicos dele (ver em [`PARAMETERS.pt-BR.md`](PARAMETERS.pt-BR.md)
  o que é preciso), e as regras do `CLAIMS.pt-BR.md` valem: nenhum número de liquidez por país
  sem essa calibração.
- **As metas de tempo são diferentes.** A referência de 2,8 s usada nos resultados é a meta do
  Pix (50% dos pagamentos liquidados em até 2,8 s). O TIPS liquida cada pagamento em menos de 10 s.

## Fontes

As mesmas da [versão em inglês](INSTANT_PAYMENTS.md#sources), sistema por sistema.
