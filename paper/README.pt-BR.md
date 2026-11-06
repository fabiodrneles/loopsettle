# Artigo para o JOSS — pasta de trabalho

> [English version](README.md)

| Arquivo | O que é |
|---|---|
| [`paper.pt-BR.md`](paper.pt-BR.md) | **rascunho em português para leitura do autor** (não vai para o JOSS) |
| `paper.md` | versão em inglês enviada ao JOSS — será escrita a partir do rascunho em português, depois da sua revisão |
| [`paper.bib`](paper.bib) | bibliografia do artigo (subconjunto de [`../docs/references/references.bib`](../docs/references/references.bib) mais os softwares citados) |

O formato segue a documentação oficial do JOSS (lida em 05/11/2026): metadados em YAML; seções
obrigatórias **Summary, Statement of need, State of the field, Software design, Research impact
statement, AI usage disclosure, Acknowledgements, References**; de 750 a 1.750 palavras (o
rascunho tem cerca de 1.350); sem documentação de API no texto. Para compilar o PDF, o JOSS usa o
Pandoc (Docker `openjournals/inara` ou a GitHub Action `openjournals/openjournals-draft-action`),
que será configurado junto com o `paper.md`.

## Quando podemos submeter: a partir de maio de 2027

A triagem do JOSS rejeita, antes da revisão, submissões que falham em qualquer um destes quatro
critérios. Situação do LoopSettle hoje:

| Critério do JOSS (obrigatório) | Situação em 05/11/2026 | O que falta |
|---|---|---|
| 1. Repositório público há **mais de seis meses**, com desenvolvimento ativo no período | criado em 05/11/2026; 55 commits, todos no mesmo dia | esperar até **pelo menos 06/05/2027**, desenvolvendo em público até lá |
| 2. **Impacto em pesquisa demonstrado** (artigos ou preprints que usam o software; uso por outros) | só uso do próprio autor, ainda sem preprint | publicar o **preprint** primeiro (ele passa a ser a evidência); buscar ao menos um usuário externo (banco central, pesquisador) |
| 3. Boas práticas de código aberto (para projeto de um autor só: histórico ao longo do tempo, releases ou changelog, testes e CI, documentação, CONTRIBUTING, expectativas de suporte) | testes, CI e documentação ok; faltam releases e CONTRIBUTING | criar `CONTRIBUTING`, `CHANGELOG`, a declaração de suporte e a **tag v0.1.0** agora, e novas releases a cada mudança relevante |
| 4. **Desenvolvimento iterativo** ao longo do tempo (não um único surto de commits) | um único surto | usar issues e PRs para cada melhoria nos próximos meses (por exemplo: CDFD proporcional, FIFO estrito, validação on-chain, respostas a leitores do preprint) |

Consequência para a ordem dos artigos: **primeiro o preprint**, depois o JOSS. O preprint não conta
como publicação prévia para o JOSS e é justamente a evidência de impacto que o critério 2 pede.

## O que só você pode preencher

- **ORCID** e **afiliação** (hoje marcados com `[PREENCHER]`).
- **Declaração de uso de IA:** o JOSS exige ferramentas e versões, onde foram usadas, o tipo de
  ajuda e a confirmação de que um humano revisou e validou tudo e tomou as decisões centrais de
  desenho. Uma declaração incompleta é tratada como falta ética. Revise o texto do rascunho
  palavra por palavra e acrescente o modelo e a versão.
- **Conflitos de interesse e financiamento:** o JOSS pede que se declare qualquer vínculo
  profissional relevante (inclusive emprego na área de pagamentos) e qualquer apoio financeiro.
- **Pessoa que testou a instalação:** o JOSS pede que, para software novo, um colega tenha
  instalado e testado.

## Pontos de atenção do texto

- A seção de impacto em pesquisa é a mais fraca hoje, porque depende do preprint.
- O estado da arte cita o BoF-PSS2 (gratuito para pesquisa, código fechado, sob licença) e o
  PSSimPy (Python, alto valor; a licença não aparece no PyPI). Confira os dois antes de enviar.
- A autoria do PSSimPy no `paper.bib` veio do PyPI (Kenneth See e Hanzholah Shobri); confira no
  artigo do ICIS 2024.
