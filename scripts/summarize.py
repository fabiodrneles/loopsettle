#!/usr/bin/env python3
"""Build results/SUMMARY.md (English) and results/SUMMARY.pt-BR.md (Portuguese) from results/<experiment>/{summary.json,*.csv}.

Only reads what the experiments wrote; every number in both summaries comes from a file
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


def meta(s, T):
    return f"_seed `{s['seed']}`, {T['code']} `{s['code_version']}`, {T['command']} `{s['command']}`._"


PT = {
    "title": "# Resumo dos resultados",
    "switch": "> [English version](SUMMARY.md)",
    "intro1": "Gerado por `python scripts/summarize.py` a partir de `results/*/`. Não edite à mão: rode `make all`.",
    "intro2": "Valores em reais são do cenário **sintético** (1 milhão de pagamentos/dia; cada pagamento simulado representa ~239 pagamentos reais). "
              "As frações são o resultado principal; valores absolutos não escalam linearmente para o Pix real.",
    "code": "código", "command": "comando", "fig": "Figura:",
    "h1": "## 1. Calibração (real vs simulado)",
    "c_cal": ["agregado", "real", "simulado", "erro relativo"],
    "h2": "## 2. Baseline: liquidez e escada de circularidade",
    "c_kv": ["medida", "valor"],
    "r_payments": "pagamentos", "r_gross": "volume bruto (R$)",
    "r_ub": "UB — liquidez do bruto em tempo real (R$)", "r_lb": "LB — posição líquida de fim de dia (R$)",
    "r_ub_gross": "UB / volume bruto", "r_potential": "capacidade potencial (estilo BCB)",
    "ladder": "Escada (frações do fluxo total do grafo; Hodge L1 pode passar de 1):",
    "c_ladder": ["grafo", "arestas", "Hodge L1", "Hodge L2 rot.", "CDFD máx.", "ciclos (guloso)", "triângulos"],
    "corr": "Correlação potencial de Hodge × posição líquida: {}.",
    "sens": "Sensibilidade ao viés recebedor (principal ASSUMPTION da estrutura bilateral):",
    "c_sens": ["σ viés", "UB (R$)", "LB/UB", "potencial", "Hodge L1 bruto", "CDFD bruto", "ciclos bruto", "CDFD líquido", "economia micro-ciclo", "p95 (ms)"],
    "h3": "## 3. Potencial vs realizável (comparação central)",
    "c_central": ["medida", "valor", "significado", "status"],
    "by_design": "Por desenho de liquidação (liquidez ilimitada; espera máxima em segundos):",
    "c_engines": ["espera máx. (s)", "motor", "liquidez (R$)", "economia vs UB", "fração de UB−LB recuperada", "p50 (ms)", "p95 (ms)", "≤ 2,8 s", "valor em ciclos"],
    "h4": "## 4. Micro-ciclos: economia × atraso",
    "c_sweep": ["janela (ms)", "atraso máx. (ms)", "economia vs UB", "p50 (ms)", "p95 (ms)", "p99 (ms)", "≤ 2,8 s", "valor em ciclos"],
    "constrained": "Liquidez restrita (saldo inicial = fração do UB de cada participante):",
    "c_short": ["fração do UB", "motor", "valor rejeitado", "p95 (ms)", "≤ 2,8 s", "valor em ciclos"],
    "h5": "## 5. Estresse: maior participante fora do ar por 30 minutos",
    "outage": "Participante {p} fora de {h}h por {m} min: {n} pagamentos (R$ {v}) adiados, em média {s} s.",
    "c_stress": ["cenário", "UB total (R$)", "UB do participante (R$)", "motor", "liquidez (R$)", "valor rejeitado", "p95 (ms)"],
    "h6": "## 6. Lógica do estudo finlandês: ciclos fixos vs instantâneo",
    "ratio": "Razão liquidez instantânea / liquidez com N ciclos por dia (média sobre sementes):",
    "c_fin": ["σ viés", "ciclos/dia", "razão média", "dp", "mín", "máx", "sementes"],
    "h7": "## 7. Robustez: sementes, escala, comprimento do ciclo e limite de atraso",
    "r1": "R1 — várias sementes × viés recebedor (média, desvio-padrão e intervalo de 95%; limite = teto para qualquer mecanismo com esse atraso máximo):",
    "c_seeds": ["σ viés", "motor", "sementes", "economia média", "dp (p.p.)", "IC 95%", "limite de atraso", "LB/UB"],
    "r2": "R2 — escala (mesmo cenário, de 1 a 30 milhões de pagamentos por dia):",
    "c_scale": ["pagamentos/dia", "fator de escala", "motor", "economia vs UB", "limite de atraso", "UB / volume"],
    "r3": "R3 — comprimento máximo do micro-ciclo (janela 0,5 s, atraso 10 s):",
    "c_len": ["comprimento máx.", "economia vs UB", "valor em ciclos"],
    "r5": "R5 — razão por participante: liquidez instantânea / liquidez com N ciclos por dia:",
    "c_banks": ["ciclos/dia", "participantes com razão", "só precisam no instantâneo", "não precisam", "p10", "mediana", "p90", "máx."],
}

EN = {
    "title": "# Results summary",
    "switch": "> [Versão em português](SUMMARY.pt-BR.md)",
    "intro1": "Generated by `python scripts/summarize.py` from `results/*/`. Do not edit by hand: run `make all`.",
    "intro2": "Amounts in reais come from the **synthetic** scenario (1 million payments/day; each simulated payment stands for ~239 real payments). "
              "Fractions are the main result; absolute values do not scale linearly to real Pix.",
    "code": "code", "command": "command", "fig": "Figure:",
    "h1": "## 1. Calibration (real vs simulated)",
    "c_cal": ["aggregate", "real", "simulated", "relative error"],
    "h2": "## 2. Baseline: liquidity and circularity ladder",
    "c_kv": ["measure", "value"],
    "r_payments": "payments", "r_gross": "gross volume (R$)",
    "r_ub": "UB — real-time gross liquidity (R$)", "r_lb": "LB — end-of-day net position (R$)",
    "r_ub_gross": "UB / gross volume", "r_potential": "potential capacity (BCB style)",
    "ladder": "Ladder (fractions of the graph's total flow; Hodge L1 can exceed 1):",
    "c_ladder": ["graph", "edges", "Hodge L1", "Hodge L2 curl", "max CDFD", "cycles (greedy)", "triangles"],
    "corr": "Hodge potential × net position correlation: {}.",
    "sens": "Sensitivity to receiver bias (the main ASSUMPTION of the bilateral structure):",
    "c_sens": ["σ bias", "UB (R$)", "LB/UB", "potential", "gross Hodge L1", "gross CDFD", "gross cycles", "net CDFD", "micro-cycle saving", "p95 (ms)"],
    "h3": "## 3. Potential vs realizable (central comparison)",
    "c_central": ["measure", "value", "meaning", "status"],
    "by_design": "By settlement design (unlimited liquidity; maximum wait in seconds):",
    "c_engines": ["max wait (s)", "engine", "liquidity (R$)", "saving vs UB", "share of UB−LB recovered", "p50 (ms)", "p95 (ms)", "≤ 2.8 s", "value in cycles"],
    "h4": "## 4. Micro-cycles: saving × delay",
    "c_sweep": ["window (ms)", "max delay (ms)", "saving vs UB", "p50 (ms)", "p95 (ms)", "p99 (ms)", "≤ 2.8 s", "value in cycles"],
    "constrained": "Constrained liquidity (opening balance = a fraction of each participant's UB):",
    "c_short": ["fraction of UB", "engine", "value rejected", "p95 (ms)", "≤ 2.8 s", "value in cycles"],
    "h5": "## 5. Stress: largest participant offline for 30 minutes",
    "outage": "Participant {p} offline from {h}h for {m} min: {n} payments (R$ {v}) delayed, on average {s} s.",
    "c_stress": ["scenario", "total UB (R$)", "participant's UB (R$)", "engine", "liquidity (R$)", "value rejected", "p95 (ms)"],
    "h6": "## 6. Finnish study logic: fixed cycles vs instant",
    "ratio": "Ratio of instant liquidity to liquidity with N cycles per day (mean over seeds):",
    "c_fin": ["σ bias", "cycles/day", "mean ratio", "sd", "min", "max", "seeds"],
    "h7": "## 7. Robustness: seeds, scale, cycle length and delay bound",
    "r1": "R1 — many seeds × receiver bias (mean, standard deviation and 95% interval; bound = ceiling for any mechanism with that maximum delay):",
    "c_seeds": ["σ bias", "engine", "seeds", "mean saving", "sd (p.p.)", "95% CI", "delay bound", "LB/UB"],
    "r2": "R2 — scale (same scenario, 1 to 30 million payments per day):",
    "c_scale": ["payments/day", "scale factor", "engine", "saving vs UB", "delay bound", "UB / volume"],
    "r3": "R3 — maximum micro-cycle length (0.5 s window, 10 s delay):",
    "c_len": ["max length", "saving vs UB", "value in cycles"],
    "r5": "R5 — per-participant ratio: instant liquidity / liquidity with N cycles per day:",
    "c_banks": ["cycles/day", "participants with ratio", "need liquidity only if instant", "need none", "p10", "median", "p90", "max"],
}


def build(T):
    md = [T["title"], "", T["switch"], "", T["intro1"], T["intro2"], ""]

    # --- calibration
    s = load("calibration")
    md += [T["h1"], meta(s, T), ""]
    md.append(table(T["c_cal"],
                    [[r["aggregate"], num(r["real"], 4), num(r["simulated"], 4), pct(r["relative_error"])] for r in rows("calibration", "calibration.csv")]))
    md.append("")

    # --- baseline
    s = load("baseline")
    fl = s["flow"]
    md += [T["h2"], meta(s, T), "",
           table(T["c_kv"], [
               [T["r_payments"], fl["payments"]],
               [T["r_gross"], num(fl["gross_value_brl"])],
               [T["r_ub"], num(fl["ub_brl"])],
               [T["r_lb"], num(fl["lb_brl"])],
               ["LB / UB", num(fl["lb_over_ub"], 4)],
               [T["r_ub_gross"], num(fl["ub_over_gross"], 4)],
               [T["r_potential"], pct(fl["potential_offset_fraction"])],
           ]), "", T["ladder"], ""]
    md.append(table(T["c_ladder"],
                    [[r["graph"], r["edges"], num(r["hodge_l1_cyclic"], 4), num(r["hodge_l2_curl"], 4), num(r["cdfd_max"], 4), num(r["cycle_netting"], 4), r["hodge_triangles"]]
                     for r in rows("baseline", "ladder.csv")]))
    md += ["", T["corr"].format(num(s['topology']['hodge_potential_vs_net_position_corr'], 4)), "",
           T["sens"], ""]
    md.append(table(T["c_sens"],
                    [[r["receiver_bias_sigma"], num(r["ub_brl"]), num(r["lb_over_ub"], 3), pct(r["potential_offset"]), num(r["hodge_l1_gross"], 3),
                      num(r["cdfd_max_gross"], 3), num(r["cycles_gross"], 3), num(r["cdfd_max_net"], 3), f"{float(r['micro_saving_pct']):.4f}%", r["micro_p95_ms"]]
                     for r in rows("baseline", "sensitivity.csv")]))
    md += ["", T["fig"] + " [`baseline/fig_ladder.svg`](baseline/fig_ladder.svg)", ""]

    # --- potential vs realizable
    s = load("potential-vs-realizable")
    md += [T["h3"], meta(s, T), ""]
    md.append(table(T["c_central"],
                    [[r["measure"], num(r["value"], 5), r["meaning"], r["status"]] for r in rows("potential-vs-realizable", "central.csv")]))
    md += ["", T["by_design"], ""]
    md.append(table(T["c_engines"],
                    [[r["max_wait_s"], r["engine"], num(r["liquidity_brl"]), f"{float(r['saving_vs_ub_pct']):.4f}%", num(r["recovered_share_of_ub_minus_lb"], 4),
                      r["delay_p50_ms"], r["delay_p95_ms"], pct(r["share_within_2800ms"]), pct(r["cycle_value_share"])]
                     for r in rows("potential-vs-realizable", "engines.csv")]))
    md += ["", T["fig"] + " [`potential-vs-realizable/fig_potential_vs_realizable.svg`](potential-vs-realizable/fig_potential_vs_realizable.svg)", ""]

    # --- sweep
    s = load("microcycle-sweep")
    md += [T["h4"], meta(s, T), ""]
    md.append(table(T["c_sweep"],
                    [[r["window_ms"], r["max_delay_ms"], f"{float(r['saving_vs_ub_pct']):.4f}%", r["delay_p50_ms"], r["delay_p95_ms"], r["delay_p99_ms"],
                      pct(r["share_within_2800ms"]), pct(r["cycle_value_share"])] for r in rows("microcycle-sweep", "sweep.csv")]))
    md += ["", T["constrained"], ""]
    md.append(table(T["c_short"],
                    [[r["liquidity_fraction_of_ub"], r["engine"], pct(r["rejected_value_share"]), r["delay_p95_ms"], pct(r["share_within_2800ms"]), pct(r["cycle_value_share"])]
                     for r in rows("microcycle-sweep", "shortfall.csv")]))
    md += ["", T["fig"] + " [`microcycle-sweep/fig_tradeoff.svg`](microcycle-sweep/fig_tradeoff.svg)", ""]

    # --- stress
    s = load("stress")
    o = s["outage"]
    md += [T["h5"], meta(s, T), "",
           T["outage"].format(p=o['participant'], h=f"{o['start_ms'] / 3.6e6:.1f}", m=f"{o['duration_ms'] / 60000:.0f}",
                              n=o['deferred_payments'], v=num(o['deferred_value_brl']), s=f"{o['mean_deferral_ms'] / 1000:.0f}"), ""]
    md.append(table(T["c_stress"],
                    [[r["scenario"], num(r["ub_total_brl"]), num(r["ub_of_offline_participant_brl"]), r["engine"], num(r["liquidity_brl"]), pct(r["rejected_value_share"], 4), r["delay_p95_ms"]]
                     for r in rows("stress", "stress.csv")]))
    md.append("")

    # --- finland
    s = load("finland-replication")
    md += [T["h6"], meta(s, T), "",
           T["ratio"], ""]
    md.append(table(T["c_fin"],
                    [[r["receiver_bias_sigma"], r["cycles_per_day"], num(r["instant_over_deferred_liquidity_mean"], 3), num(r["sd"], 3), num(r["min"], 3), num(r["max"], 3), r["seeds"]]
                     for r in rows("finland-replication", "ratios.csv")]))
    md += ["", T["fig"] + " [`finland-replication/fig_finland.svg`](finland-replication/fig_finland.svg)", ""]

    # --- robustness
    s = load("robustness")
    md += [T["h7"], meta(s, T), "", T["r1"], ""]
    md.append(table(T["c_seeds"],
                    [[r["receiver_bias_sigma"], r["engine"], r["seeds"], f"{float(r['saving_pct_mean']):.4f}%", f"{float(r['saving_pct_sd']):.4f}",
                      f"[{float(r['saving_pct_ci95_low']):.4f}; {float(r['saving_pct_ci95_high']):.4f}]%", f"{float(r['delay_bound_pct_mean']):.3f}%", num(r["lb_over_ub_mean"], 3)]
                     for r in rows("robustness", "seeds_summary.csv")]))
    md += ["", T["r2"], ""]
    md.append(table(T["c_scale"],
                    [[num(r["payments_per_day"], 0), num(r["scale_factor"], 1), r["engine"], f"{float(r['saving_vs_ub_pct']):.4f}%", f"{float(r['delay_bound_pct']):.3f}%", num(r["ub_over_gross"], 4)]
                     for r in rows("robustness", "scale.csv")]))
    md += ["", T["r3"], ""]
    md.append(table(T["c_len"], [[r["max_cycle_len"], f"{float(r['saving_vs_ub_pct']):.4f}%", pct(r["value_share_settled_in_cycles"])] for r in rows("robustness", "cycle_len.csv")]))
    md += ["", T["r5"], ""]
    md.append(table(T["c_banks"], [[r["cycles_per_day"], r["participants_with_ratio"], r["needs_liquidity_only_if_instant"], r["needs_no_liquidity"],
                                     num(r["ratio_p10"], 3), num(r["ratio_p50"], 3), num(r["ratio_p90"], 3), num(r["ratio_max"], 2)] for r in rows("robustness", "participants_ratio.csv")]))
    md += ["", T["fig"] + " [`robustness/fig_scale.svg`](robustness/fig_scale.svg)", ""]
    return "\n".join(md) + "\n"


def main():
    for name, T in (("SUMMARY.md", EN), ("SUMMARY.pt-BR.md", PT)):
        path = os.path.join(ROOT, name)
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(build(T))
        print(f"wrote {os.path.normpath(path)}")


if __name__ == "__main__":
    main()
