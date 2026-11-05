# data/

Somente dados **públicos** e **sintéticos**. Dados brutos grandes não são versionados
(`data/raw/` está no `.gitignore`).

| Arquivo | O que é | Como gerar |
|---|---|---|
| `fetch_pix.py` | Baixa agregados do Pix (API Olinda do BCB) | `python data/fetch_pix.py --month 202609 --keys-date 2026-09-30` |
| `calibration/pix.toml` | Alvos de calibração (versionado, pequeno) | gerado pelo script acima |
| `raw/` | Respostas brutas da API (não versionado) | gerado pelo script acima |
| `samples/triangle.csv` | Caso fixo do triângulo para o comando `topology` | — |

Nota técnica: o servidor Olinda devolve HTTP 500 para `$skip`; o script faz uma única
requisição com `$top` grande e aborta se o resultado puder ter sido truncado.

## Validação externa (Fase 8): transferências on-chain de stablecoins

O comando abaixo roda a escada de circularidade (Hodge / CDFD / ciclos) em **qualquer**
lista de transferências no formato `from,to,amount` (com cabeçalho), lendo em streaming:

```
cargo run --release -p loopsettle-cli -- topology caminho/para/transferencias.csv
```

Fontes públicas possíveis para uma amostra pequena (ex.: um dia de transferências de USDC
entre os 200 endereços mais ativos), a escolher pelo revisor:

- conjunto público `crypto_ethereum.token_transfers` do Google BigQuery
  (filtrar por `token_address` do USDC e por um intervalo de blocos);
- a API de um explorador de blocos (normalmente exige uma chave gratuita).

Depois de exportar, renomeie as colunas para `from,to,amount` (valor em unidades do token).
**Status:** comando implementado e testado no caso do triângulo; a amostra on-chain ainda
não foi baixada (requer escolher a fonte e, possivelmente, uma chave de API).
