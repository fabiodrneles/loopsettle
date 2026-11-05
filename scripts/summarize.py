#!/usr/bin/env python3
"""Build results/SUMMARY.md from results/<experiment>/{summary.json,*.csv}.

Only reads what the experiments wrote; every number in SUMMARY.md comes from a file
produced by `cargo run --release -p loopsettle-cli -- experiment <name>`.
"""
import csv
import json
import os

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "results")


def load(exp):
    with open(os.path.join(ROOT, exp, "summary.json"), encoding="utf-8") as f:
        return json.load(f)


def rows(exp, name):
    with open(os.path.join(ROOT, exp, name), encoding="utf-8") as f:
        return list(csv.DictReader(f))


def num(x, d=2):
    try:
        v = float(x)
    except (TypeError, ValueError):
        return str(x)
    if abs(v) >= 1e6:
        return f"{v:,.0f}".replace(",", " ")
    return f"{v:.{d}f}"


def pct(x, d=2):
    return f"{100 * float(x):.{d}f}%"


def table(header, body):
    out = ["| " + " | ".join(header) + " |", "|" + "---|" * len(header)]
    out += ["| " + " | ".join(str(c) for c in r) + " |" for r in body]
    return "\n".join(out)


def meta(s):
    return f"_seed `{s['seed']}`, código `{s['code_version']}`, comando `{s['command']}`._"


def main():
    md = ["# Resumo dos resultados", "",
          "Gerado por `python scripts/summarize.py` a partir de `results/*/`. Não edite à mão: rode `make all`.",
          "Valores em reais são do cenário **sintético** (1 milhão de pagamentos/dia; cada pagamento simulado representa ~239 pagamentos reais). "
          "As frações são o resultado principal; valores absolutos não escalam linearmente para o Pix real.", ""]

    # --- calibration
    s = load("calibration")
    md += ["## 1. Calibração (real vs simulado)", meta(s), ""]
    md.append(table(["agregado", "real", "simulado", "erro relativo"],
                    [[r["aggregate"], num(r["real"], 4), num(r["simulated"], 4), pct(r["relative_error"])] for r in rows("calibration", "calibration.csv")]))
    md.append("")

    # --- baseline
    s = load("baseline")
    fl = s["flow"]
    md += ["## 2. Baseline: liquidez e escada de circularidade", meta(s), "",
           table(["medida", "valor"], [
               ["pagamentos", fl["payments"]],
               ["volume bruto (R$)", num(fl["gross_value_brl"])],
               ["UB — liquidez do bruto em tempo real (R$)", num(fl["ub_brl"])],
               ["LB — posição líquida de fim de dia (R$)", num(fl["lb_brl"])],
               ["LB / UB", num(fl["lb_over_ub"], 4)],
               ["UB / volume bruto", num(fl["ub_over_gross"], 4)],
               ["capacidade potencial (estilo BCB)", pct(fl["potential_offset_fraction"])],
           ]), "", "Escada (frações do fluxo total do grafo; Hodge L1 pode passar de 1):", ""]
    md.append(table(["grafo", "arestas", "Hodge L1", "Hodge L2 rot.", "CDFD máx.", "ciclos (guloso)", "triângulos"],
                    [[r["graph"], r["edges"], num(r["hodge_l1_cyclic"], 4), num(r["hodge_l2_curl"], 4), num(r["cdfd_max"], 4), num(r["cycle_netting"], 4), r["hodge_triangles"]]
                     for r in rows("baseline", "ladder.csv")]))
    md += ["", f"Correlação potencial de Hodge × posição líquida: {num(s['topology']['hodge_potential_vs_net_position_corr'], 4)}.", "",
           "Sensibilidade ao viés recebedor (principal ASSUMPTION da estrutura bilateral):", ""]
    md.append(table(["σ viés", "UB (R$)", "LB/UB", "potencial", "Hodge L1 bruto", "CDFD bruto", "ciclos bruto", "CDFD líquido", "economia micro-ciclo", "p95 (ms)"],
                    [[r["receiver_bias_sigma"], num(r["ub_brl"]), num(r["lb_over_ub"], 3), pct(r["potential_offset"]), num(r["hodge_l1_gross"], 3),
                      num(r["cdfd_max_gross"], 3), num(r["cycles_gross"], 3), num(r["cdfd_max_net"], 3), f"{float(r['micro_saving_pct']):.4f}%", r["micro_p95_ms"]]
                     for r in rows("baseline", "sensitivity.csv")]))
    md += ["", "Figura: [`baseline/fig_ladder.svg`](baseline/fig_ladder.svg)", ""]

    # --- potential vs realizable
    s = load("potential-vs-realizable")
    md += ["## 3. Potencial vs realizável (comparação central)", meta(s), ""]
    md.append(table(["medida", "valor", "significado", "status"],
                    [[r["measure"], num(r["value"], 5), r["meaning"], r["status"]] for r in rows("potential-vs-realizable", "central.csv")]))
    md += ["", "Por desenho de liquidação (liquidez ilimitada; espera máxima em segundos):", ""]
    md.append(table(["espera máx. (s)", "motor", "liquidez (R$)", "economia vs UB", "fração de UB−LB recuperada", "p50 (ms)", "p95 (ms)", "≤ 2,8 s", "valor em ciclos"],
                    [[r["max_wait_s"], r["engine"], num(r["liquidity_brl"]), f"{float(r['saving_vs_ub_pct']):.4f}%", num(r["recovered_share_of_ub_minus_lb"], 4),
                      r["delay_p50_ms"], r["delay_p95_ms"], pct(r["share_within_2800ms"]), pct(r["cycle_value_share"])]
                     for r in rows("potential-vs-realizable", "engines.csv")]))
    md += ["", "Figura: [`potential-vs-realizable/fig_potential_vs_realizable.svg`](potential-vs-realizable/fig_potential_vs_realizable.svg)", ""]

    # --- sweep
    s = load("microcycle-sweep")
    md += ["## 4. Micro-ciclos: economia × atraso", meta(s), ""]
    md.append(table(["janela (ms)", "atraso máx. (ms)", "economia vs UB", "p50 (ms)", "p95 (ms)", "p99 (ms)", "≤ 2,8 s", "valor em ciclos"],
                    [[r["window_ms"], r["max_delay_ms"], f"{float(r['saving_vs_ub_pct']):.4f}%", r["delay_p50_ms"], r["delay_p95_ms"], r["delay_p99_ms"],
                      pct(r["share_within_2800ms"]), pct(r["cycle_value_share"])] for r in rows("microcycle-sweep", "sweep.csv")]))
    md += ["", "Liquidez restrita (saldo inicial = fração do UB de cada participante):", ""]
    md.append(table(["fração do UB", "motor", "valor rejeitado", "p95 (ms)", "≤ 2,8 s", "valor em ciclos"],
                    [[r["liquidity_fraction_of_ub"], r["engine"], pct(r["rejected_value_share"]), r["delay_p95_ms"], pct(r["share_within_2800ms"]), pct(r["cycle_value_share"])]
                     for r in rows("microcycle-sweep", "shortfall.csv")]))
    md += ["", "Figura: [`microcycle-sweep/fig_tradeoff.svg`](microcycle-sweep/fig_tradeoff.svg)", ""]

    # --- stress
    s = load("stress")
    o = s["outage"]
    md += ["## 5. Estresse: maior participante fora do ar por 30 minutos", meta(s), "",
           f"Participante {o['participant']} fora de {o['start_ms'] / 3.6e6:.1f}h por {o['duration_ms'] / 60000:.0f} min: "
           f"{o['deferred_payments']} pagamentos (R$ {num(o['deferred_value_brl'])}) adiados, em média {o['mean_deferral_ms'] / 1000:.0f} s.", ""]
    md.append(table(["cenário", "UB total (R$)", "UB do participante (R$)", "motor", "liquidez (R$)", "valor rejeitado", "p95 (ms)"],
                    [[r["scenario"], num(r["ub_total_brl"]), num(r["ub_of_offline_participant_brl"]), r["engine"], num(r["liquidity_brl"]), pct(r["rejected_value_share"], 4), r["delay_p95_ms"]]
                     for r in rows("stress", "stress.csv")]))
    md.append("")

    # --- finland
    s = load("finland-replication")
    md += ["## 6. Lógica do estudo finlandês: ciclos fixos vs instantâneo", meta(s), "",
           "Razão liquidez instantânea / liquidez com N ciclos por dia (média sobre sementes):", ""]
    md.append(table(["σ viés", "ciclos/dia", "razão média", "dp", "mín", "máx", "sementes"],
                    [[r["receiver_bias_sigma"], r["cycles_per_day"], num(r["instant_over_deferred_liquidity_mean"], 3), num(r["sd"], 3), num(r["min"], 3), num(r["max"], 3), r["seeds"]]
                     for r in rows("finland-replication", "ratios.csv")]))
    md += ["", "Figura: [`finland-replication/fig_finland.svg`](finland-replication/fig_finland.svg)", ""]

    path = os.path.join(ROOT, "SUMMARY.md")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(md) + "\n")
    print(f"wrote {os.path.normpath(path)}")


if __name__ == "__main__":
    main()
