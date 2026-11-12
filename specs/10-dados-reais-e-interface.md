# 10 — Dados reais, interoperabilidade e interface (P1)

> **Estado em 12/11/2026:** **R2 feito** (rótulos `P001...`, `--keep-names`, `--min-participants`;
> falta suprimir linhas que revelam um pagamento único). **R3 feito** (`--report`: `report.md`,
> `report.pt-BR.md`, `fig_bound.svg`; teste `privacy_defaults_and_report`). R1, R4, R5 a fazer.

## Contexto

O W9 do `PAPER_READINESS` diz o que transforma o estudo em referência: outras pessoas usarem.
O caminho mais curto é um banco central ou uma tesouraria rodar `loopsettle bound` nos próprios
dados e publicar o resultado. Com o piso de liquidez (spec 01), esse comando passou a dar uma
resposta forte em duas passadas: "nenhum mecanismo com prazo `d` economiza mais que X% aqui".
Esta spec trata de tornar esse uso seguro, simples e verificável.

## R1 — Contrato do arquivo de entrada

- Documentar em `data/README*.md` o esquema aceito (colunas, formatos de horário, separador,
  moeda em unidades com 2 casas ou em centavos), com exemplos válidos e inválidos.
- Mapeamento para campos de mensagens ISO 20022 de liquidação (pacs.008/pacs.002: instante de
  aceitação, agente devedor, agente credor, valor interbancário): **conferir os nomes dos
  campos na especificação antes de escrever**, não de memória.
- Arquivos fora de ordem: hoje o leitor rejeita (`reads_a_small_file_and_rejects_unsorted`).
  Acrescentar `loopsettle sort <in> <out>` com ordenação externa (blocos em disco, RAM < 1 GB)
  ou documentar o comando `sort` equivalente do sistema.
- Liquidados × rejeitados: se o arquivo tiver coluna de estado, usar só os liquidados e
  informar quantos foram ignorados.
- **AC:** arquivo de 50 milhões de linhas processado em streaming com RAM < 1 GB (gerar com
  `loopsettle export`); tempos medidos.

## R2 — Privacidade da saída (reforço da D21)

- Padrão continua "só agregados". Com `--per-participant`: aviso explícito, e opção
  `--min-participants K` (padrão 5) que recusa a saída por participante quando houver menos de
  K participantes, e suprime linhas cujo valor permitiria identificar um pagamento único
  (um participante com um só pagamento no pico).
- Saída nunca contém identificadores originais, a menos de `--keep-names`.
- **AC:** teste com arquivo de 3 participantes: `--per-participant` falha com mensagem clara.
- Um parágrafo em `docs/` explicando, para o jurídico de uma instituição, exatamente o que
  sai do comando.

## R3 — Relatório pronto para compartilhar

`loopsettle bound <arquivo> --report`: gera `report.md` (e `report.pt-BR.md`) com UB, LB, a
curva piso × d (2 s a 1 dia), a figura SVG, a versão do código, o hash do arquivo de entrada
(não o conteúdo) e um parágrafo fixo de interpretação e limitações. **AC:** o relatório do
arquivo sintético exportado bate com os números de `results/regimes/`.

## R4 — Interface web

- **Rodar o próprio arquivo no navegador**, sem enviar nada a servidor: o usuário escolhe um
  CSV, o Web Worker lê em blocos (`File.stream()`) e chama o wasm. Exibir de forma visível "o
  arquivo não sai deste computador". Limite de tamanho com mensagem (por exemplo 200 MB).
- Painel novo "Piso de liquidez": curva piso × d com o obtido por cada motor sobreposto.
- `loopsettle-wasm` precisa expor o piso; como ele usa duas passadas, no navegador guardar o
  fluxo em `Vec<Payment>` é aceitável até ~2 milhões de pagamentos (32 MB).
- Publicar em GitHub Pages por workflow (W9 item 4). **Só quando o dono pedir**: publicar é
  ação externa (CLAUDE.md, seção 0, item c, e "Passo opcional").
- **AC:** as capturas de `scripts/screenshots.cjs` regeneradas em PT e EN; teste de fumaça do
  wasm cobre o piso; funciona em 375 px de largura.

## R5 — Uso como biblioteca

- `README` de cada crate com um exemplo de 15 linhas (rodar um motor num `Vec<Payment>`).
- Exemplo `examples/custom_engine.rs`: implementar a trait `SettlementEngine` para um
  mecanismo novo e passá-lo pelos testes de invariantes (expor as funções de teste como
  módulo `loopsettle_core::testing` atrás de uma feature). É o que deixa outro pesquisador
  testar o próprio LSM em 50 linhas, e é o argumento mais forte de "ferramenta de referência".
- Saída em JSON com esquema versionado (spec 09 E6) para quem quiser analisar em Python ou R;
  um caderno de exemplo em `examples/` que só lê os CSVs (sem dependência nova no projeto).

## R6 — Publicação (W7, do lado do dono)

Tag `v0.2.0` depois das specs 02, 04 e 09; DOI no Zenodo; preprint com o piso de liquidez
como resultado principal. Nada disso é automático: são ações do dono.

## Como validar antes de implementar

```bash
cargo run --release -q -p loopsettle-cli -- export baseline /tmp/p.csv --payments 50000 --participants 30
```

```bash
cargo run --release -q -p loopsettle-cli -- bound /tmp/p.csv --out /tmp/out
```

Ler `csvfile.rs` e `realdata.rs` por inteiro (a auditoria só leu parte).
