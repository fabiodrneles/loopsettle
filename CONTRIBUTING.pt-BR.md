# Como contribuir com o LoopSettle

> [English version](CONTRIBUTING.md)

Obrigado pelo interesse. O LoopSettle é mantido por uma pessoa, então o processo é simples.

## Relatar um problema ou tirar uma dúvida

Abra uma [issue](https://github.com/fabiodrneles/loopsettle/issues) com o que você rodou (o comando
exato), o que esperava e o que aconteceu, mais o seu sistema operacional e a saída de
`cargo --version`. Dúvidas sobre o método ou os resultados também são bem-vindas como issues.

**Nunca anexe dados reais de pagamentos**, nem dados de empregador ou de clientes. O LoopSettle usa
só dados públicos e sintéticos; para relatar um problema com `loopsettle bound` ou `settle`,
reproduza com `loopsettle export` ou com [`data/samples/payments_example.csv`](data/samples/payments_example.csv).

## Propor uma mudança

1. Abra uma issue antes de qualquer coisa além de uma correção pequena, para combinarmos o caminho.
2. Trabalhe numa branch e abra um pull request para a `main`.
3. Antes de abrir, rode `cargo test --workspace`. Se mudou um experimento ou o modelo, rode também o
   `cargo run --release -p loopsettle-cli -- experiment <nome>` afetado e faça commit dos
   `results/` gerados de novo (commit do código antes, para os resultados terem versão de código
   limpa).
4. Decisões de desenho que mudam resultados vão para o [`DECISIONS.pt-BR.md`](DECISIONS.pt-BR.md)
   (o quê, alternativas, por quê, como reverter); todo parâmetro do gerador precisa de fonte ou do
   rótulo `ASSUMPTION` em [`docs/PARAMETERS.pt-BR.md`](docs/PARAMETERS.pt-BR.md).
5. Código e comentários em inglês; documentação em inglês e português (mudou um documento, muda o
   gêmeo `.pt-BR.md` no mesmo commit).

As oito invariantes testadas por propriedades (`cargo test`) precisam passar sempre; uma mudança que
enfraqueça alguma delas não será aceita.

## Expectativa de suporte

Issues e pull requests são respondidos conforme a disponibilidade, normalmente em até duas semanas.
Não há suporte pago. Problemas de segurança: abra uma issue sem detalhes de exploração e peça um
canal privado.

## Licença

Ao contribuir, você concorda que sua contribuição é licenciada sob a
[Licença Apache 2.0](LICENSE).
