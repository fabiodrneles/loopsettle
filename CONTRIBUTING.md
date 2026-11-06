# Contributing to LoopSettle

> [Versão em português](CONTRIBUTING.pt-BR.md)

Thank you for your interest. LoopSettle is maintained by one person, so the process is light.

## Reporting a problem or asking a question

Open an [issue](https://github.com/fabiodrneles/loopsettle/issues) with what you ran (the exact
command), what you expected and what happened, plus your operating system and the output of
`cargo --version`. Questions about the method or the results are welcome as issues too.

**Never attach real payment data**, nor data from any employer or client. LoopSettle only uses
public and synthetic data; to report a problem with `loopsettle bound` or `settle`, reproduce it
with `loopsettle export` or with [`data/samples/payments_example.csv`](data/samples/payments_example.csv).

## Proposing a change

1. Open an issue first for anything beyond a small fix, so we can agree on the approach.
2. Work on a branch and open a pull request against `main`.
3. Before opening it, run `cargo test --workspace`. If you changed an experiment or the model,
   also run the affected `cargo run --release -p loopsettle-cli -- experiment <name>` and commit
   the regenerated `results/` (commit the code first, so the results carry a clean code version).
4. Design decisions that change results go in [`DECISIONS.md`](DECISIONS.md) (what, alternatives,
   why, how to revert); every generator parameter needs a source or the `ASSUMPTION` label in
   [`docs/PARAMETERS.md`](docs/PARAMETERS.md).
5. Code and comments in English; documentation in English and Portuguese (when a document changes,
   its `.pt-BR.md` twin changes in the same commit).

The eight property-tested invariants (`cargo test`) must always pass; a change that weakens one of
them will not be merged.

## Support expectations

Issues and pull requests are answered on a best-effort basis, usually within two weeks. There is no
paid support. Security problems: open an issue without exploit details and ask for a private
channel.

## Licence

By contributing you agree that your contribution is licensed under the
[Apache License 2.0](LICENSE).
