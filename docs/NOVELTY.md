# NOVELTY — o que o LoopSettle faz em relação aos trabalhos de referência

> 🇬🇧 [English version](en/NOVELTY.md)

> Para a seção de contribuição do artigo. As colunas dos outros trabalhos resumem o que o
> `CLAUDE.md` diz sobre eles; **o revisor deve conferir cada célula nas fontes** antes de
> publicar (marcadas "a conferir" quando dependem de detalhe que não verificamos).
>
> Situação de verificação de cada referência: [`REFERENCES.md`](REFERENCES.md). O PixSim
> (arXiv 2609.30684) **não foi localizado** em busca pública; não citar até confirmar.

| Dimensão | LoopSettle | Hellqvist & Korpinen (BoF 2021; JFMI 2026) | BCB WP 619 (Gonzalez, Ma & Zeng, 2025) | CDFD (Homs-Dones et al., arXiv 2506.12546) | BoF-PSS | PSSimPy | PixSim (arXiv 2609.30684, **não localizado**) |
|---|---|---|---|---|---|---|---|
| Objeto | Liquidez circular no pagamento instantâneo | Liquidez: ciclos vs instantâneo | Perda de netting no Pix | Decomposição de fluxos em rede | Simulador de RTGS/LSM | Simulador de sistemas de pagamento (a conferir) | Simulador do Pix |
| Dados | Sintéticos calibrados com agregados públicos do Pix (set/2026), tudo com fonte ou `ASSUMPTION` | Sintéticos calibrados | Dados do BCB (a conferir: confidenciais) | Redes em geral | Dados do usuário | (a conferir) | Calibração com fonte por parâmetro |
| Medida de netting | Três lentes lado a lado: Hodge, CDFD máx., ciclos | Liquidez por modelo | Capacidade **potencial** (sem ordem temporal) | Componente circular | Algoritmos de LSM/gridlock | (a conferir) | (a conferir) |
| Potencial **vs realizável** no tempo | **Sim** — mesma rede, de 1×/dia a 10 s e micro-ciclos | Parcial (ciclos fixos) | Não (só potencial) | Não (estático) | Realizável, sem medida potencial | (a conferir) | (a conferir) |
| Mecanismo novo | **Micro-ciclos atômicos** com atraso máximo e modo "só na falta" | — | — | — | LSMs clássicos | — | — |
| Escada diagnóstico → execução | **Sim** (Hodge ≥ CDFD ≥ ciclos ≥ micro-ciclos, com distâncias) | Não | Não | CDFD vs outras decomposições (a conferir) | Não | Não | Não |
| CDFD exata sem PL | **Sim** (fluxo de custo mínimo, validada contra PL) | — | — | Algoritmo próprio (a conferir) | — | — | — |
| Invariantes testadas por propriedade | **Sim** (8 invariantes, proptest) | — | — | — | (a conferir) | (a conferir) | (a conferir) |
| Reprodutibilidade | Seed + versão do código em cada resultado; `make rebuild` refaz tudo | (a conferir) | Não (dados restritos) | (a conferir) | Ferramenta, não estudo | (a conferir) | (a conferir) |
| Roda no navegador | **Sim** (wasm + Web Worker) | Não | Não | Não | Não (desktop) | Não | (a conferir) |
| Hardware | i3 4ª ger., 4 GB; < 20 MiB por experimento | — | — | — | — | — | — |
| Licença | Apache-2.0 | artigo | artigo | artigo | (a conferir) | (a conferir) | (a conferir) |

## Contribuições propostas (a validar com o revisor)

1. Medida **realizável** da economia de liquidez no tempo, lado a lado com a medida
   **potencial** usada no estudo do BCB, no mesmo cenário.
2. A **escada** de três lentes de circularidade, com o caso do triângulo como exemplo
   mínimo de por que Hodge não é um montante compensável.
3. O mecanismo de **micro-ciclos atômicos** e sua curva de troca economia × atraso,
   comparada à meta de 2,8 s do Manual de Tempos do Pix.
4. Um simulador **aberto, reprodutível e leve**, com calibração em dados públicos do Pix.
