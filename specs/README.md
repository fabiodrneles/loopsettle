# Especificações do LoopSettle (roteiro pós-v0.1.0)

> Escritas em 11/11/2026 a partir de uma auditoria do código, dos testes e dos resultados.
> Servem para que outro agente (ou pessoa) **valide** cada ponto e depois **execute**, sem
> precisar do contexto da sessão que as escreveu. Exceção registrada à D17: specs são documentos
> de trabalho e ficam só em português (ver D22 em `DECISIONS.pt-BR.md`).

## O que torna o LoopSettle uma referência (e o que ainda falta)

A ferramenta já é sólida: invariantes por propriedades, resultados reprodutíveis byte a byte,
memória de poucos MiB, comandos para dados reais. O que a auditoria mostrou é que a contribuição
científica mais forte não é o simulador em si, e sim **limites que valem para qualquer
mecanismo**, que qualquer banco central pode calcular nos próprios dados em duas passadas. Três
frentes decidem se o trabalho vira referência:

1. **Limites apertados e o ótimo exato** (specs 01, 02, 03). O piso de liquidez já foi
   implementado nesta sessão e é 14× mais apertado que o limite antigo. Falta propagá-lo e
   fechar o "sanduíche" piso ≤ ótimo ≤ motores.
2. **A base de comparação certa para sistemas com fila** (specs 05, 06). Hoje todo resultado
   usa liquidez ilimitada e compara com o bruto sem fila. A literatura de LSM (McMahon et al.,
   BoF-PSS) compara com FIFO sob liquidez restrita. Sem isso o LoopSettle não conversa com ela.
3. **Confiança nos números** (specs 04, 08, 09). Correções de motor, estatística com várias
   sementes em tudo, e proteção contra regressão de resultados no CI.

## Índice e ordem recomendada

| # | Spec | Tipo | Estado | Prioridade |
|---|---|---|---|---|
| 00 | [Auditoria: achados com evidência](00-auditoria.md) | registro | escrito | ler primeiro |
| 01 | [Piso de liquidez (limite apertado)](01-piso-de-liquidez.md) | ciência | **implementado** | — |
| 02 | [Propagar o limite apertado](02-propagar-limite-apertado.md) | ciência + docs | FR1–FR6 feitos; FR7 (interface web) a fazer | P0 |
| 03 | [Ótimo offline e o sanduíche](03-otimo-offline.md) | ciência | **feito** (sem FR2 nem FR5; ver REVIEW 85) | P1 |
| 04 | [Correções nos motores e nas métricas](04-correcoes-motores.md) | correção | **feito** (C1–C4, C6); C5 medido e adiado | P0 |
| 05 | [Motores novos](05-motores-novos.md) | funcionalidade | M1 e M2 feitos; M3–M5 a fazer | P1 |
| 06 | [Fronteira com liquidez restrita](06-liquidez-restrita.md) | ciência | **feito** (sem M3; ver REVIEW 84) | P1 |
| 07 | [Gerador: rajadas, vários dias, modelos nulos](07-gerador.md) | modelo | a fazer | P2 |
| 08 | [Topologia: BFF, escala, circularidade temporal](08-topologia.md) | ciência | a fazer | P2 |
| 09 | [Estatística, reprodutibilidade e CI](09-estatistica-reprodutibilidade-ci.md) | qualidade | E2, E3, E4, E6 feitos; E1 e E5 parciais | P0 |
| 10 | [Dados reais, interoperabilidade e interface](10-dados-reais-e-interface.md) | adoção | R2 e R3 feitos; R1, R4, R5 a fazer | P1 |

Ordem sugerida: 04 → 09 → 02 → 06 → 03 → 05 → 10 → 07 → 08. A 04 e a 09 vêm antes porque
toda rodada nova de resultados deve sair já com os motores corrigidos e com várias sementes.

## Regras para quem for executar

1. **Valide antes de implementar.** Cada spec tem uma seção "Como validar". Achados marcados
   `LIDO` foram vistos no código, mas não reproduzidos por teste; `MEDIDO` tem número e comando;
   `HIPÓTESE` precisa ser confirmada. Se a validação contradisser a spec, corrija a spec e
   registre em `REVIEW.md`, não implemente assim mesmo.
2. **Teste primeiro.** Para cada correção, escreva o teste que falha, depois o código.
   Nenhum teste existente pode ser enfraquecido para passar.
3. **Orçamento do CLAUDE.md, seção 2:** 1 GB de RAM por execução, streaming, sem dependências
   novas sem justificativa. `minilp` e `proptest` só em testes.
4. **Resultados.** Mudou um motor ou o gerador: rode `make rebuild`, confira o que mudou em
   `results/` com `git diff --stat`, e explique cada mudança de número no commit e em
   `REVIEW.md`. Nunca publique número de uma semente só (CLAIMS, "não pode afirmar" 11).
5. **Documentos gêmeos (D17).** Mudou `X.md`, mude `X.pt-BR.md` no mesmo commit. Specs são a
   exceção.
6. **Decisões que mudam resultados** vão para `DECISIONS.md` com alternativa e como reverter, e
   ficam configuráveis quando possível.
7. **Honestidade.** Se um resultado novo enfraquecer uma afirmação de `docs/CLAIMS.md`, mude a
   afirmação. Isso já aconteceu nesta sessão com a 1f.
8. Ao terminar uma spec, mude o estado na tabela acima, atualize `CHECKPOINT.md` e acrescente
   os pontos novos em `REVIEW.md`.

## Formato de cada spec

Contexto e evidência → Requisitos (FR) → Critérios de aceite (AC) → Como validar →
Arquivos → Riscos e perguntas em aberto.
