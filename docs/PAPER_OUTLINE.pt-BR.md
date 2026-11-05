# Esqueleto do artigo

> 🇬🇧 [English version](PAPER_OUTLINE.md)

Só títulos e a lista de figuras/tabelas de cada seção. O texto será escrito com o revisor.

## 1. Introdução
- Tabela 1 — Pergunta de pesquisa e as três medidas (potencial, estática, realizável).

## 2. Trabalhos relacionados
- Tabela 2 — Comparação ponto a ponto (`docs/NOVELTY.pt-BR.md`).

## 3. Modelo
### 3.1 Participantes, pagamentos e ledger
### 3.2 Motores de liquidação (bruto, compensação diferida, micro-ciclos atômicos)
- Figura 1 — Diagrama do micro-ciclo atômico (fila por aresta, ciclo, prazo).
### 3.3 Liquidez necessária: UB, LB e pico de queda do saldo
- Tabela 3 — Exemplo à mão (A, B, C) de `docs/METHODOLOGY.pt-BR.md`.
### 3.4 Três lentes de circularidade
- Figura 2 — Caso do triângulo: Hodge 6,67 vs CDFD/ciclos 5.

## 4. Dados e calibração
- Tabela 4 — Parâmetros e status (`docs/PARAMETERS.pt-BR.md`).
- Tabela 5 — Real vs simulado (`results/calibration/calibration.csv`).

## 5. Implementação e validação
- Tabela 6 — Invariantes e testes de propriedade.
- Tabela 7 — Desempenho no hardware de referência (`CHECKPOINT.md`).

## 6. Resultados
### 6.1 Liquidez e circularidade no cenário calibrado
- Tabela 8 — UB, LB, potencial (`results/baseline/summary.json`).
- Figura 3 — Escada de circularidade (`results/baseline/fig_ladder.svg`).
### 6.2 Potencial vs realizável
- Figura 4 — Economia × espera máxima (`results/potential-vs-realizable/fig_potential_vs_realizable.svg`).
- Tabela 9 — Medidas centrais (`results/potential-vs-realizable/central.csv`).
### 6.3 Micro-ciclos: economia × atraso
- Figura 5 — Curva de troca (`results/microcycle-sweep/fig_tradeoff.svg`).
- Tabela 10 — Liquidez restrita (`results/microcycle-sweep/shortfall.csv`).
### 6.4 Ciclos fixos vs instantâneo (lógica do estudo finlandês)
- Figura 6 — Razão de liquidez (`results/finland-replication/fig_finland.svg`).
### 6.5 Estresse
- Tabela 11 — Indisponibilidade de 30 min (`results/stress/stress.csv`).

## 7. Sensibilidade e robustez
- Tabela 12 — Viés recebedor (`results/baseline/sensitivity.csv`).
- (A fazer) Escala 10–30 milhões de pagamentos; `max_cycle_len`; FIFO estrito.

## 8. Limitações
- Lista de `docs/LIMITATIONS.pt-BR.md`.

## 9. Conclusão

## Apêndices
- A. Prova de que UB é o menor saldo sem rejeição; B. CDFD como transbordo de custo mínimo;
  C. Reprodutibilidade (`make rebuild`).
