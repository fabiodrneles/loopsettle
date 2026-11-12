#!/usr/bin/env python3
"""Build results/SUMMARY.md (English) and results/SUMMARY.pt-BR.md (Portuguese) from results/<experiment>/{summary.json,*.csv}.

Only reads what the experiments wrote; every number in both summaries comes from a file
produced by `cargo run --release -p loopsettle-cli -- experiment <name>`.
"""
import csv
import json
import os
import sys

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
    "nulls": "Valores nulos (spec 08, T4): o nulo analítico de Hodge e o nível esperado só pelos volumes de quem paga e pela popularidade de quem recebe, embaralhando o recebedor de cada par (20 sorteios):",
    "c_nulls": ["medida", "observado", "nulo (média)", "nulo (dp)", "observado / nulo", "nulo"],
    "c_ladder": ["grafo", "arestas", "Hodge L1", "Hodge L2 rot.", "CDFD máx.", "CDFD proporcional (BFF)", "ciclos (guloso)", "triângulos"],
    "corr": "Correlação potencial de Hodge × posição líquida: {}.",
    "wp619": "Razão de pagamentos compensáveis por banco, definição do BCB WP 619 (eq. 3.2, 2·min(saídas, entradas)/(saídas + entradas)): p25 {}, mediana {}, p75 {}. No Pix real, o WP 619 reporta mediana de 57% a 80%, p25 de 18% a 45% e p75 de 73% a 90% (nov/2020 em diante).",
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
    "h5": "## 5. Estresse: participante fora do ar por 30 minutos",
    "outage": "Participante {p} fora de {h}h por {m} min: {n} pagamentos (R$ {v}) adiados, em média {s} s.",
    "c_stress": ["cenário", "UB total (R$)", "UB do participante (R$)", "motor", "liquidez (R$)", "valor rejeitado", "p95 (ms)"],
    "cases": "Estresse redesenhado (REVIEW 39): quem cai e quando. Para cada caso, o efeito sobre a liquidez necessária dos **outros** participantes (UB com liquidez ilimitada) e, com saldo inicial igual ao UB de base de cada um, o valor rejeitado por motor (100% reenviado na volta; ASSUMPTION).",
    "c_cases": ["alvo", "part.", "posição líquida (R$)", "início", "pagamentos adiados", "Δ UB total", "Δ UB dos outros (líq.)", "outros com UB maior", "maior aumento (R$)", "motor", "valor rejeitado"],
    "h6": "## 6. Lógica do estudo finlandês: ciclos fixos vs instantâneo",
    "ratio": "Razão liquidez instantânea / liquidez com N ciclos por dia (média sobre sementes):",
    "c_fin": ["σ viés", "ciclos/dia", "razão média", "dp", "mín", "máx", "sementes"],
    "h7": "## 7. Robustez: sementes, escala, comprimento do ciclo e limite de atraso",
    "r1": "R1 — várias sementes × viés recebedor (média, desvio-padrão e intervalo de 95%; limite = teto para qualquer mecanismo com esse atraso máximo):",
    "c_seeds": ["σ viés", "motor", "sementes", "economia média", "dp (p.p.)", "IC 95%", "limite (janela)", "LB/UB", "limite apertado [IC 95%]", "obtido / apertado"],
    "r2": "R2 — escala (mesmo cenário, de 1 a 30 milhões de pagamentos por dia):",
    "c_scale": ["pagamentos/dia", "fator de escala", "motor", "economia vs UB", "limite (janela)", "UB / volume", "limite apertado"],
    "r3": "R3 — comprimento máximo do micro-ciclo (janela 0,5 s, atraso 10 s):",
    "c_len": ["comprimento máx.", "economia vs UB", "valor em ciclos"],
    "r5": "R5 — razão por participante: liquidez instantânea / liquidez com N ciclos por dia:",
    "c_banks": ["ciclos/dia", "participantes com razão", "só precisam no instantâneo", "não precisam", "p10", "mediana", "p90", "máx."],
    "h8": "## 8. Calibração do viés recebedor no BCB WP 619 (W2b)",
    "cal_intro": "Quartis da razão de pagamentos compensáveis por banco (WP 619, eq. 3.2) para cada σ e multiplicador das IPs, média de 3 sementes. Alvo, com dados reais do Pix: p25 18–45%, mediana 57–80%, p75 73–90%.",
    "c_cal8": ["σ viés", "mult. IP", "p25", "mediana", "p75", "quartis dentro do alvo", "perda (p.p.²)", "LB/UB"],
    "cal_best": "Melhor ajuste: σ = {}, multiplicador das IPs = {} (p25 {}, mediana {}, p75 {}).",
    "h9": "## 9. Heterogeneidade intradiária e escala (W2, critério de morte K1)",
    "het_intro": "Perfis intradiários por classe de instituição com intensidade λ (0 = um perfil para todos, 1 = perfis completos), em σ = 0,4 e no σ calibrado:",
    "c_het": ["σ viés", "λ", "pagamentos/dia", "motor", "economia vs UB", "limite (janela)", "LB/UB", "limite apertado"],
    "k1": "Critério K1 (parar se o limite de 10 s passar de 2% do UB com 30 milhões de pagamentos): pior caso {} na forma de janela e {} na forma apertada; disparou: {}.",
    "yes": "sim", "no": "não",
    "hodge_null": "Valor nulo de Hodge (fluxos aleatórios no mesmo grafo líquido, m = {} arestas, n = {} nós): (m − n + 1)/m = {}, contra {} observado — em grafo denso, a fração rotacional mede sobretudo a densidade.",
    "h10": "## 10. Janelas de abastecimento da Conta PI: noites e fim de semana (W3)",
    "fw_intro": "A Conta PI só recebe aportes das Reservas com o STR aberto (6h30–18h30 nos dias úteis, mais 30 minutos). Quinta a segunda, perfil horário e volumes por dia do Relatório Anual do SPI 2024. Necessidade = maior esvaziamento a partir do início da janela; compensação = uma liquidação líquida no fim da janela.",
    "c_fw": ["pagamentos/quinta", "σ viés", "λ", "janela", "tipo", "horas", "pré-financiamento (R$)", "economia de compensar no fim", "limite 10 s (janela)", "limite 15 min (janela)", "limite apertado 10 s", "limite apertado 15 min"],
    "closed": "fechada", "open": "aberta",
    "pf_intro": "Pré-financiamento abaixo da necessidade (camada de abastecimento, spec 05 M5): cada participante entra nas janelas fechadas com uma fração da sua maior necessidade; com o STR aberto, aportes ilimitados. Valor rejeitado em % do valor bruto dos cinco dias, para o bruto e para filas que esperam até 1 hora (recebimentos ou reabertura). Uma semente, a menor escala.",
    "c_pf": ["σ viés", "λ", "fração da necessidade", "motor", "valor rejeitado"],
    "h11": "## 11. Regimes: varejo × alto valor, e curva do limite",
    "h15": "## 15. Circularidade por horizonte: valor que circula × liquidez recuperável",
    "ch_intro": "Para cada largura w, o dia é cortado em janelas; dentro de cada janela calcula-se a circulação máxima (CDFD) dos fluxos agregados. A soma é o valor que *poderia ser cancelado* compensando dentro de janelas de largura w. Ao lado, para o mesmo w como prazo: o limite apertado e o que a compensação a cada w obtém. Valor cancelado não é liquidez economizada. Médias de 5 sementes.",
    "c_ch": ["janela / prazo (s)", "valor que circula (% do bruto) [IC 95%]", "só bilateral (% do bruto)", "limite apertado (% do UB)", "obtido a cada w (% do UB)", "limite apertado (% do bruto)", "circula / recuperável"],
    "h14": "## 14. Pagamentos em eco: circularidade de curto prazo (teste mais duro da afirmação central)",
    "ec_intro": "Uma parcela de todos os pagamentos é resposta a um recebimento: quem recebe paga o mesmo valor pouco depois, para um recebedor qualquer (retorno 0) ou direto de volta a quem pagou (retorno 1). ASSUMPTION: teste de estresse, não comportamento calibrado. Médias de 8 sementes, 1 milhão de pagamentos por dia, σ = 0,4.",
    "c_ec": ["parcela de eco", "atraso do eco (s)", "retorno", "motor", "economia [IC 95%]", "limite apertado [IC 95%]", "valor em ciclos", "LB/UB"],
    "ec_kill": "Critério de morte ({}): pior caso {}; disparou: {}.",
    "ec_scale": "O mesmo caso com mais volume (experimento `echo-scale`, {} sementes): o limite apertado de 10 s em % do UB cai com expoente {} no número de pagamentos; em valor, cresce com expoente {}.",
    "c_escale": ["pagamentos/dia", "economia a cada 10 s [IC 95%]", "limite apertado [IC 95%]", "UB / volume"],
    "ec_sens": "Sensibilidade do caso que disparou o critério (20% de eco, 1 s, de volta a quem pagou), mudando uma coisa de cada vez; prazo de 10 s:",
    "c_sens": ["variante", "motor", "economia [IC 95%]", "limite apertado [IC 95%]"],
    "ec_diag": "Diagnóstico de ordem: só atrasar (hold, ordem de chegada mantida) contra motores que liquidam uns pagamentos antes e outros depois, com ecos adiante:",
    "c_diag": ["caso", "motor", "economia [IC 95%]"],
    "h13": "## 13. O sanduíche: pisos de liquidez ≤ ótimo offline ≤ motores",
    "og_intro": "Sistema pequeno numa hora movimentada (20 participantes, cerca de 1.500 pagamentos, 20 sementes; ASSUMPTION, não calibrado). Para cada prazo: a economia máxima segundo os pisos por participante, segundo o ótimo offline do sistema inteiro (programa linear, pagamentos fracionáveis, lotes arbitrários) e a obtida pelo melhor motor. Gerado por `cargo run --release -p loopsettle-sim --example optimality_gap`.",
    "c_og": ["σ viés", "prazo (s)", "limite dos pisos", "ótimo offline [IC 95%]", "melhor motor", "ótimo / pisos", "motor / ótimo", "instâncias com ótimo acima dos pisos"],
    "h12": "## 12. Liquidez restrita: rejeição, atraso e liquidez equivalente",
    "lf_intro": "Saldos de abertura fixos (regra `ub`: α × UB de cada participante; regra `outflow`: α × UB total, repartido pelo volume de saída) e o que acontece em cada desenho. Dois horizontes: espera de até 10 s (prazo de pagamento instantâneo) e de até 1 hora (regime dos mecanismos de economia de liquidez de alto valor). Pagamento rejeitado não é tentado de novo (ASSUMPTION). Médias de 10 sementes, 300 mil pagamentos por dia.",
    "c_lf": ["cenário", "regra", "α", "motor", "valor rejeitado [IC 95%]", "atraso p95 (s)", "até 2,8 s"],
    "lf_eq": "Liquidez equivalente: o menor α em que cada desenho rejeita no máximo o alvo, e a economia em relação ao bruto (diferença pareada por semente, interpolação linear na grade de α):",
    "c_eq": ["cenário", "regra", "alvo (valor rejeitado)", "motor", "sementes que atingem", "α equivalente", "economia vs bruto [IC 95%]"],
    "lf_mono": "Coerência: a rejeição do bruto subiu quando a liquidez subiu em {} passos da grade (esperado: 0).",
    "rg_intro": "O limite de atraso na forma de janela depende do tamanho dos pagamentos em relação à liquidez necessária; o limite apertado (UB − piso de liquidez, METHODOLOGY §9b) não: ele fica perto de zero nos dois regimes. Sistema de alto valor estilizado (ASSUMPTION): 20 participantes, mediana de R$ 1 milhão, cauda pesada.",
    "c_rg": ["caso", "participantes", "pagamentos", "pagamento médio / UB médio", "atraso máx. (ms)", "economia micro-ciclos", "limite (janela)", "limite apertado"],
    "curve": "Curva do limite (W5): economia obtida pela compensação a cada d contra o limite de atraso para o mesmo d (varejo, Pix calibrado; médias de 10 sementes):",
    "fbp": "O limite apertado por participante (uma semente): quanto cada um poderia economizar, no máximo, em % do próprio UB:",
    "c_fbp": ["d (s)", "participantes com UB > 0", "p10", "mediana", "p90", "máximo", "parcela do limite total nos 5 maiores"],
    "c_curve": ["d (s)", "ciclos/dia", "economia obtida", "limite janela (até 100%)", "obtido / janela", "limite apertado [IC 95%]", "obtido / apertado [IC 95%]"],
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
    "nulls": "Null values (spec 08, T4): the analytic Hodge null and the level expected from payers' volumes and payees' popularity alone, shuffling the payee of every pair (20 draws):",
    "c_nulls": ["measure", "observed", "null (mean)", "null (sd)", "observed / null", "null"],
    "c_ladder": ["graph", "edges", "Hodge L1", "Hodge L2 curl", "max CDFD", "proportional CDFD (BFF)", "cycles (greedy)", "triangles"],
    "corr": "Hodge potential × net position correlation: {}.",
    "wp619": "Per-bank nettable ratio, BCB WP 619 definition (eq. 3.2, 2·min(out, in)/(out + in)): p25 {}, median {}, p75 {}. On real Pix data WP 619 reports a median of 57% to 80%, p25 of 18% to 45% and p75 of 73% to 90% (from Nov 2020).",
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
    "h5": "## 5. Stress: a participant offline for 30 minutes",
    "outage": "Participant {p} offline from {h}h for {m} min: {n} payments (R$ {v}) delayed, on average {s} s.",
    "c_stress": ["scenario", "total UB (R$)", "participant's UB (R$)", "engine", "liquidity (R$)", "value rejected", "p95 (ms)"],
    "cases": "Redesigned stress (REVIEW 39): who goes offline and when. For each case, the effect on the liquidity need of **everyone else** (UB with unlimited liquidity) and, with each opening balance equal to its baseline UB, the value each engine rejects (100% resubmitted on return; ASSUMPTION).",
    "c_cases": ["target", "part.", "net position (R$)", "start", "deferred payments", "Δ total UB", "Δ others' UB (net)", "others with higher UB", "largest increase (R$)", "engine", "value rejected"],
    "h6": "## 6. Finnish study logic: fixed cycles vs instant",
    "ratio": "Ratio of instant liquidity to liquidity with N cycles per day (mean over seeds):",
    "c_fin": ["σ bias", "cycles/day", "mean ratio", "sd", "min", "max", "seeds"],
    "h7": "## 7. Robustness: seeds, scale, cycle length and delay bound",
    "r1": "R1 — many seeds × receiver bias (mean, standard deviation and 95% interval; bound = ceiling for any mechanism with that maximum delay):",
    "c_seeds": ["σ bias", "engine", "seeds", "mean saving", "sd (p.p.)", "95% CI", "window bound", "LB/UB", "tight bound [95% CI]", "realised / tight"],
    "r2": "R2 — scale (same scenario, 1 to 30 million payments per day):",
    "c_scale": ["payments/day", "scale factor", "engine", "saving vs UB", "window bound", "UB / volume", "tight bound"],
    "r3": "R3 — maximum micro-cycle length (0.5 s window, 10 s delay):",
    "c_len": ["max length", "saving vs UB", "value in cycles"],
    "r5": "R5 — per-participant ratio: instant liquidity / liquidity with N cycles per day:",
    "c_banks": ["cycles/day", "participants with ratio", "need liquidity only if instant", "need none", "p10", "median", "p90", "max"],
    "h8": "## 8. Receiver-bias calibration to BCB WP 619 (W2b)",
    "cal_intro": "Quartiles of the per-bank nettable ratio (WP 619, eq. 3.2) for each σ and payment-institution multiplier, mean of 3 seeds. Target, on real Pix data: p25 18–45%, median 57–80%, p75 73–90%.",
    "c_cal8": ["σ bias", "PI mult.", "p25", "median", "p75", "quartiles inside target", "loss (p.p.²)", "LB/UB"],
    "cal_best": "Best fit: σ = {}, payment-institution multiplier = {} (p25 {}, median {}, p75 {}).",
    "h9": "## 9. Intraday heterogeneity and scale (W2, kill criterion K1)",
    "het_intro": "Class-specific intraday profiles with strength λ (0 = one profile for everyone, 1 = full profiles), at σ = 0.4 and at the calibrated σ:",
    "c_het": ["σ bias", "λ", "payments/day", "engine", "saving vs UB", "window bound", "LB/UB", "tight bound"],
    "k1": "Kill criterion K1 (stop if the 10 s bound exceeds 2% of UB at 30 million payments): worst case {} in the window form and {} in the tight form; fired: {}.",
    "yes": "yes", "no": "no",
    "hodge_null": "Hodge null value (random flows on the same net graph, m = {} edges, n = {} nodes): (m − n + 1)/m = {}, against {} observed — on a dense graph the curl share mostly measures density.",
    "h10": "## 10. Conta PI funding windows: nights and the weekend (W3)",
    "fw_intro": "The Conta PI can be topped up from reserves only while the STR is open (6:30–18:30 on business days, plus 30 minutes). Thursday to Monday, hourly profile and day volumes from the SPI annual report 2024. Need = largest drawdown from the start of the window; netting = one net settlement at the end of the window.",
    "c_fw": ["payments/Thursday", "σ bias", "λ", "window", "kind", "hours", "prefunding need (R$)", "saving of netting at the end", "window bound 10 s", "window bound 15 min", "tight bound 10 s", "tight bound 15 min"],
    "closed": "closed", "open": "open",
    "pf_intro": "Prefunding below the need (funding layer, spec 05 M5): each participant enters the closed windows with a fraction of its largest need; while STR is open, top-ups are unlimited. Rejected value in % of the gross value of the five days, for gross settlement and for queues that wait up to one hour (for receipts or the reopening). One seed, smallest scale.",
    "c_pf": ["σ bias", "λ", "fraction of the need", "engine", "rejected value"],
    "h11": "## 11. Regimes: retail vs large-value, and the bound curve",
    "h15": "## 15. Circularity by horizon: value that circulates vs liquidity that can be recovered",
    "ch_intro": "For each width w the day is cut into windows; inside each window the maximum circulation (CDFD) of the aggregated flows is computed. The sum is the value that *could be cancelled* by netting inside windows of width w. Next to it, for the same w as a deadline: the tight bound and what netting every w realises. Cancelled value is not saved liquidity. Means over 5 seeds.",
    "c_ch": ["window / deadline (s)", "circulating value (% of gross) [95% CI]", "bilateral only (% of gross)", "tight bound (% of UB)", "realised every w (% of UB)", "tight bound (% of gross)", "circulating / recoverable"],
    "h14": "## 14. Echo payments: short-horizon circularity (the hardest test of the central claim)",
    "ec_intro": "A share of all payments is a response to a receipt: the payee pays the same amount shortly after, to a payee drawn as usual (return 0) or straight back to the payer (return 1). ASSUMPTION: a stress test, not a calibrated behaviour. Means over 8 seeds, 1 million payments a day, σ = 0.4.",
    "c_ec": ["echo share", "echo delay (s)", "return", "engine", "saving [95% CI]", "tight bound [95% CI]", "value in cycles", "LB/UB"],
    "ec_kill": "Kill criterion ({}): worst case {}; fired: {}.",
    "ec_scale": "The same case at higher volume (experiment `echo-scale`, {} seeds): the 10 s tight bound in % of UB falls with exponent {} in the number of payments; in money it grows with exponent {}.",
    "c_escale": ["payments/day", "saving of netting every 10 s [95% CI]", "tight bound [95% CI]", "UB / volume"],
    "ec_sens": "Sensitivity of the case that fired the criterion (20% echo share, 1 s, straight back), one change at a time; 10 s deadline:",
    "c_sens": ["variant", "engine", "saving [95% CI]", "tight bound [95% CI]"],
    "ec_diag": "Ordering diagnostic: delaying only (hold, arrival order kept) against engines that settle some payments earlier and others later, with onward echoes:",
    "c_diag": ["case", "engine", "saving [95% CI]"],
    "h13": "## 13. The sandwich: liquidity floors ≤ offline optimum ≤ engines",
    "og_intro": "A small system in a busy hour (20 participants, about 1,500 payments, 20 seeds; ASSUMPTION, not calibrated). For each deadline: the largest saving according to the per-participant floors, according to the offline optimum of the whole system (linear programme, divisible payments, arbitrary batches) and the one realised by the best engine. Generated by `cargo run --release -p loopsettle-sim --example optimality_gap`.",
    "c_og": ["σ bias", "deadline (s)", "floor bound", "offline optimum [95% CI]", "best engine", "optimum / floors", "engine / optimum", "instances with optimum above floors"],
    "h12": "## 12. Constrained liquidity: rejection, delay and equivalent liquidity",
    "lf_intro": "Fixed opening balances (rule `ub`: α × each participant's UB; rule `outflow`: α × total UB, shared by outflow volume) and what happens under each design. Two horizons: holding a payment for at most 10 s (an instant-payment deadline) and for at most 1 hour (the regime of large-value liquidity-saving mechanisms). A rejected payment is not retried (ASSUMPTION). Means over 10 seeds, 300,000 payments a day.",
    "c_lf": ["scenario", "rule", "α", "engine", "rejected value [95% CI]", "p95 delay (s)", "within 2.8 s"],
    "lf_eq": "Equivalent liquidity: the smallest α at which each design rejects at most the target, and the saving relative to gross (paired difference by seed, linear interpolation on the α grid):",
    "c_eq": ["scenario", "rule", "target (rejected value)", "engine", "seeds reaching it", "equivalent α", "saving vs gross [95% CI]"],
    "lf_mono": "Coherence: gross rejection rose when liquidity rose in {} grid steps (expected: 0).",
    "rg_intro": "The window form of the delay bound depends on payment size relative to liquidity needs; the tight bound (UB − liquidity floor, METHODOLOGY §9b) does not: it stays near zero in both regimes. Stylised large-value system (ASSUMPTION): 20 participants, median R$ 1 million, heavy tail.",
    "c_rg": ["case", "participants", "payments", "avg payment / avg UB", "max delay (ms)", "micro-cycle saving", "window bound", "tight bound"],
    "curve": "Bound curve (W5): realised saving of netting every d vs the delay bound for the same d (retail, Pix calibrated; means over 10 seeds):",
    "fbp": "The tight bound per participant (one seed): the most each one could save, in % of its own UB:",
    "c_fbp": ["d (s)", "participants with UB > 0", "p10", "median", "p90", "max", "share of the total bound in the top 5"],
    "c_curve": ["d (s)", "cycles/day", "realised saving", "window bound (capped at 100%)", "realised / window", "tight bound [95% CI]", "realised / tight [95% CI]"],
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
                    [[r["graph"], r["edges"], num(r["hodge_l1_cyclic"], 4), num(r["hodge_l2_curl"], 4), num(r["cdfd_max"], 4), num(r["cdfd_bff"], 4), num(r["cycle_netting"], 4), r["hodge_triangles"]]
                     for r in rows("baseline", "ladder.csv")]))
    md += ["", T["nulls"], ""]
    md.append(table(T["c_nulls"], [[r["measure"], num(r["observed"], 4), num(r["null_mean"], 4), num(r["null_sd"], 4), num(r["observed_over_null"], 3), r["null"]] for r in rows("baseline", "nulls.csv")]))
    md.append("")
    # WP 619 eq. (3.2): per-bank 2*min(out, in)/(out + in), reported as quartiles across banks
    wp = sorted(2 * min(float(r["out_brl"]), float(r["in_brl"])) / (float(r["out_brl"]) + float(r["in_brl"]))
                for r in rows("baseline", "participants.csv") if float(r["out_brl"]) + float(r["in_brl"]) > 0)
    qq = lambda q: wp[int(q * (len(wp) - 1))]
    md += ["", T["wp619"].format(pct(qq(0.25), 1), pct(qq(0.5), 1), pct(qq(0.75), 1))]
    net = [r for r in rows("baseline", "ladder.csv") if r["graph"] == "net"][0]
    m_edges, n_nodes = int(net["edges"]), int(s["scenario"]["participants"])
    md += ["", T["hodge_null"].format(m_edges, n_nodes, num((m_edges - n_nodes + 1) / m_edges, 3), num(net["hodge_l2_curl"], 3))]
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
    md += ["", T["cases"], ""]
    md.append(table(T["c_cases"],
                    [[r["target"], r["participant"], num(r["participant_net_position_brl"]), r["start"], r["deferred_payments"],
                      f"{float(r['ub_total_change_pct']):.4f}%", f"{float(r['others_ub_net_change_pct']):.4f}%", r["others_with_higher_ub"],
                      num(r["largest_other_increase_brl"]), r["engine"], pct(r["rejected_value_share_at_baseline_ub"], 4)]
                     for r in rows("stress", "cases.csv")]))
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
                      f"[{float(r['saving_pct_ci95_low']):.4f}; {float(r['saving_pct_ci95_high']):.4f}]%", f"{float(r['delay_bound_pct_mean']):.3f}%", num(r["lb_over_ub_mean"], 3),
                      f"{float(r['tight_bound_pct_mean']):.4f}% [{float(r['tight_bound_pct_ci95_low']):.4f}; {float(r['tight_bound_pct_ci95_high']):.4f}]", num(r["saving_over_tight_bound"], 3)]
                     for r in rows("robustness", "seeds_summary.csv")]))
    md += ["", T["r2"], ""]
    md.append(table(T["c_scale"],
                    [[num(r["payments_per_day"], 0), num(r["scale_factor"], 1), r["engine"], f"{float(r['saving_vs_ub_pct']):.4f}%", f"{float(r['delay_bound_pct']):.3f}%", num(r["ub_over_gross"], 4), f"{float(r['tight_bound_pct']):.4f}%"]
                     for r in rows("robustness", "scale.csv")]))
    md += ["", T["r3"], ""]
    md.append(table(T["c_len"], [[r["max_cycle_len"], f"{float(r['saving_vs_ub_pct']):.4f}%", pct(r["value_share_settled_in_cycles"])] for r in rows("robustness", "cycle_len.csv")]))
    md += ["", T["r5"], ""]
    md.append(table(T["c_banks"], [[r["cycles_per_day"], r["participants_with_ratio"], r["needs_liquidity_only_if_instant"], r["needs_no_liquidity"],
                                     num(r["ratio_p10"], 3), num(r["ratio_p50"], 3), num(r["ratio_p90"], 3), num(r["ratio_max"], 2)] for r in rows("robustness", "participants_ratio.csv")]))
    md += ["", T["fig"] + " [`robustness/fig_scale.svg`](robustness/fig_scale.svg)", ""]

    # --- bias calibration (W2b)
    s = load("bias-calibration")
    b = s["best"]
    md += [T["h8"], meta(s, T), "", T["cal_intro"], ""]
    md.append(table(T["c_cal8"], [[r["receiver_bias_sigma"], r["pi_receiver_mult"], pct(r["ratio_p25"], 1), pct(r["ratio_p50"], 1), pct(r["ratio_p75"], 1),
                                    r["quartiles_inside_wp619_ranges"], num(r["loss_pp2"], 1), num(r["lb_over_ub"], 3)] for r in rows("bias-calibration", "grid.csv")]))
    md += ["", T["cal_best"].format(b["receiver_bias_sigma"], b["pi_receiver_mult"], pct(b["ratio_p25"], 1), pct(b["ratio_p50"], 1), pct(b["ratio_p75"], 1)), ""]

    # --- heterogeneity (W2)
    s = load("heterogeneity")
    md += [T["h9"], meta(s, T), "", T["het_intro"], ""]
    md.append(table(T["c_het"], [[r["receiver_bias_sigma"], r["profile_strength"], num(r["payments_per_day"], 0), r["engine"], f"{float(r['saving_vs_ub_pct']):.4f}%",
                                   f"{float(r['delay_bound_pct']):.3f}%", num(r["lb_over_ub"], 3), f"{float(r['tight_bound_pct']):.4f}%"] for r in rows("heterogeneity", "heterogeneity.csv")]))
    k1 = s["kill_criterion_k1"]["cases"]
    worst = max(c["bound_10s_pct_of_ub"] for c in k1)
    worst_tight = max(c["tight_bound_10s_pct_of_ub"] for c in k1)
    md += ["", T["k1"].format(f"{worst:.2f}%", f"{worst_tight:.3f}%", T["yes"] if any(c["k1_fires"] for c in k1) else T["no"]), ""]

    # --- funding windows (W3)
    s = load("funding-windows")
    md += [T["h10"], meta(s, T), "", T["fw_intro"], ""]
    md.append(table(T["c_fw"], [[num(r["payments_per_thursday"], 0), r["receiver_bias_sigma"], r["profile_strength"], r["window"], T["closed"] if r["kind"] == "closed" else T["open"], r["hours"],
                                  num(r["prefunding_need_brl"]), f"{float(r['netting_saving_pct']):.2f}%", f"{float(r['bound_10s_pct']):.2f}%", f"{float(r['bound_15min_pct']):.2f}%", f"{float(r['tight_bound_10s_pct']):.3f}%", f"{float(r['tight_bound_15min_pct']):.2f}%"]
                                 for r in rows("funding-windows", "windows.csv")]))
    md.append("")

    md += [T["pf_intro"], ""]
    md.append(table(T["c_pf"], [[r["receiver_bias_sigma"], r["profile_strength"], r["prefund_fraction_of_need"], r["engine"], f"{float(r['rejected_value_pct_of_gross']):.3f}%"]
                                 for r in rows("funding-windows", "prefunding.csv")]))
    md.append("")

    # --- regimes and the bound curve
    s = load("regimes")
    md += [T["h11"], meta(s, T), "", T["rg_intro"], ""]
    md.append(table(T["c_rg"], [[r["case"], r["participants"], r["payments"], num(r["avg_payment_over_avg_participant_ub"], 4), r["max_delay_ms"], f"{float(r['micro_cycle_saving_pct']):.4f}%",
                                  f"{float(r['delay_bound_pct']):.3f}%", f"{float(r['tight_bound_pct']):.4f}%"] for r in rows("regimes", "regimes.csv")]))
    md += ["", T["curve"], ""]
    md.append(table(T["c_curve"], [[r["max_delay_s"], r["deferred_cycles_per_day"], f"{float(r['realised_saving_pct']):.4f}%", f"{float(r['delay_bound_pct_capped_100']):.2f}%",
                                     num(r["realised_over_bound"], 3), f"{float(r['tight_bound_pct']):.3f}% [{float(r['tight_ci95_low']):.3f}; {float(r['tight_ci95_high']):.3f}]",
                                     f"{float(r['realised_over_tight_bound']):.2f} [{float(r['realised_over_tight_ci95_low']):.2f}; {float(r['realised_over_tight_ci95_high']):.2f}]"] for r in rows("regimes", "bound_curve.csv")]))
    md += ["", T["fig"] + " [`regimes/fig_bound.svg`](regimes/fig_bound.svg)", "", T["fbp"], ""]
    md.append(table(T["c_fbp"], [[r["max_delay_s"], r["participants_with_ub"], f"{float(r['tight_bound_pct_of_own_ub_p10']):.3f}%", f"{float(r['p50']):.3f}%", f"{float(r['p90']):.3f}%",
                                   f"{float(r['max']):.2f}%", pct(r["share_of_total_bound_in_top5_participants"], 1)] for r in rows("regimes", "floor_by_participant.csv")]))
    md.append("")
    # --- liquidity frontier (specs/06)
    s = load("liquidity-frontier")
    md += [T["h12"], meta(s, T), "", T["lf_intro"], ""]
    show = {"0.8", "0.95", "1"}
    md.append(table(T["c_lf"], [[r["scenario"], r["rule"], r["alpha"], r["engine"],
                                  f"{float(r['rejected_value_pct_mean']):.2f}% [{float(r['rejected_value_pct_ci95_low']):.2f}; {float(r['rejected_value_pct_ci95_high']):.2f}]",
                                  f"{float(r['delay_p95_ms_mean']) / 1000:.0f}", pct(r["share_within_2800ms_mean"], 1)]
                                 for r in rows("liquidity-frontier", "frontier.csv") if r["alpha"] in show and r["rule"] == "ub"]))
    md += ["", T["lf_eq"], ""]
    md.append(table(T["c_eq"], [[r["scenario"], r["rule"], r["target_rejected_value_pct"] + "%", r["engine"], r["seeds_reaching_target"], num(r["equivalent_alpha_mean"], 4),
                                  f"{float(r['liquidity_saving_vs_gross_pct_mean']):.2f}% [{float(r['liquidity_saving_vs_gross_pct_ci95_low']):.2f}; {float(r['liquidity_saving_vs_gross_pct_ci95_high']):.2f}]"]
                                 for r in rows("liquidity-frontier", "equivalent_liquidity.csv") if r["rule"] == "ub" and r["engine"] != "gross_real_time"]))
    md += ["", T["lf_mono"].format(s["gross_rejection_rose_with_liquidity_grid_steps"]), "", T["fig"] + " [`liquidity-frontier/fig_frontier.svg`](liquidity-frontier/fig_frontier.svg)", ""]
    # --- optimality gap (specs/03); written by an example, so it has no summary.json
    md += [T["h13"], "", T["og_intro"], ""]
    md.append(table(T["c_og"], [[r["receiver_bias_sigma"], f"{int(r['max_delay_ms']) // 1000}", f"{float(r['floor_bound_pct_mean']):.3f}%",
                                  f"{float(r['lp_bound_pct_mean']):.3f}% [{float(r['lp_bound_pct_ci95_low']):.3f}; {float(r['lp_bound_pct_ci95_high']):.3f}]",
                                  f"{float(r['best_engine_saving_pct_mean']):.3f}%", num(r["lp_over_floor_bound"], 4), num(r["engine_over_lp_bound"], 3),
                                  f"{r['instances_with_lp_above_floors']}/{r['seeds']}"] for r in rows("optimality-gap", "gap.csv")]))
    md.append("")
    # --- echo payments (specs/07, G2)
    s = load("echo-sweep")
    md += [T["h14"], meta(s, T), "", T["ec_intro"], ""]
    keep = {"micro:all:500:10000", "deferred:8640"}
    md.append(table(T["c_ec"], [[r["echo_share"], f"{float(r['echo_delay_ms']) / 1000:g}", r["echo_return_prob"], r["engine"],
                                  f"{float(r['saving_pct_mean']):.3f}% [{float(r['saving_pct_ci95_low']):.3f}; {float(r['saving_pct_ci95_high']):.3f}]",
                                  f"{float(r['tight_bound_pct_mean']):.3f}% [{float(r['tight_bound_pct_ci95_low']):.3f}; {float(r['tight_bound_pct_ci95_high']):.3f}]",
                                  pct(r["cycle_value_share_mean"], 0), num(r["lb_over_ub_mean"], 3)] for r in rows("echo-sweep", "echo.csv") if r["engine"] in keep]))
    k = s["kill_criterion"]
    md += ["", T["ec_kill"].format(k["rule"], f"{k['worst_ci95_high_pct']:.2f}%", T["yes"] if k["fires"] else T["no"]), "", T["ec_sens"], ""]
    ci = lambda r, a: f"{float(r[a + '_mean']):.3f}% [{float(r[a + '_ci95_low']):.3f}; {float(r[a + '_ci95_high']):.3f}]"
    md.append(table(T["c_sens"], [[r["variant"], r["engine"], ci(r, "saving_pct"), ci(r, "tight_bound_pct")] for r in rows("echo-sweep", "sensitivity.csv")]))
    md += ["", T["ec_diag"], ""]
    md.append(table(T["c_diag"], [[r["case"], r["engine"], ci(r, "saving_pct")] for r in rows("echo-sweep", "order_diagnostic.csv")]))
    es = load("echo-scale")
    sl = es["loglog_slopes_with_echo"]
    er = rows("echo-scale", "scale.csv")
    md += ["", T["ec_scale"].format(er[0]["seeds"], f"{sl['tight_bound_pct_of_ub']:.2f}", f"{sl['tight_bound_in_money']:.2f}"), ""]
    md.append(table(T["c_escale"], [[num(r["payments_per_day"], 0), f"{float(r['netting_10s_saving_pct_mean']):.3f}% [{float(r['saving_ci95_low']):.3f}; {float(r['saving_ci95_high']):.3f}]",
                                      f"{float(r['tight_bound_10s_pct_mean']):.3f}% [{float(r['tight_ci95_low']):.3f}; {float(r['tight_ci95_high']):.3f}]", num(r["ub_over_gross_mean"], 4)] for r in er]))
    md.append("")
    # --- circularity by horizon (specs/08, T3)
    s = load("circularity-horizon")
    md += [T["h15"], meta(s, T), "", T["ch_intro"], ""]
    md.append(table(T["c_ch"], [[f"{float(r['window_s']):g}", f"{float(r['circulating_value_pct_of_gross']):.2f}% [{float(r['circulating_ci95_low']):.2f}; {float(r['circulating_ci95_high']):.2f}]",
                                  f"{float(r['bilateral_offset_pct_of_gross']):.2f}%", f"{float(r['tight_bound_pct_of_ub']):.3f}%", f"{float(r['netting_every_w_saving_pct_of_ub']):.3f}%",
                                  f"{float(r['tight_bound_pct_of_gross']):.4f}%", num(r["circulating_over_recoverable"], 0)] for r in rows("circularity-horizon", "horizon.csv")]))
    md += ["", T["fig"] + " [`circularity-horizon/fig_horizon.svg`](circularity-horizon/fig_horizon.svg)", ""]
    return "\n".join(md) + "\n"


def check_sandwich():
    """Every row of every result file must satisfy realised <= tight bound <= window bound.

    The bounds are theorems (docs/METHODOLOGY.md, sections 9, 9b and 10), so a violation means a
    bug in an engine or in a bound, and the summary must not be written.
    """
    eps = 1e-6  # the files hold 6 decimals
    bad = []

    def chk(exp, name, realised, tight, window):
        for i, r in enumerate(rows(exp, name)):
            t, w = float(r[tight]), float(r[window])
            if realised is not None and float(r[realised]) > t + eps:
                bad.append(f"{exp}/{name} row {i + 2}: {realised}={r[realised]} > {tight}={r[tight]}")
            if t > w + eps:
                bad.append(f"{exp}/{name} row {i + 2}: {tight}={r[tight]} > {window}={r[window]}")

    chk("robustness", "seeds.csv", "saving_vs_ub_pct", "tight_bound_pct", "delay_bound_pct")
    chk("robustness", "scale.csv", "saving_vs_ub_pct", "tight_bound_pct", "delay_bound_pct")
    chk("heterogeneity", "heterogeneity.csv", "saving_vs_ub_pct", "tight_bound_pct", "delay_bound_pct")
    chk("regimes", "regimes.csv", "micro_cycle_saving_pct", "tight_bound_pct", "delay_bound_pct")
    for i, r in enumerate(rows("echo-sweep", "echo.csv")):
        if float(r["saving_pct_mean"]) > float(r["tight_bound_pct_mean"]) + eps:
            bad.append(f"echo-sweep/echo.csv row {i + 2}: mean saving above the mean tight bound")
    for d in ("10s", "15min", "1h"):
        chk("funding-windows", "windows.csv", None, f"tight_bound_{d}_pct", f"bound_{d}_pct")
    for i, r in enumerate(rows("regimes", "bound_curve.csv")):
        if float(r["realised_saving_pct"]) > float(r["tight_bound_pct"]) + eps:
            bad.append(f"regimes/bound_curve.csv row {i + 2}: realised above the tight bound")
    if bad:
        sys.exit("sandwich check failed (realised <= tight <= window):\n  " + "\n  ".join(bad))


def main():
    check_sandwich()
    for name, T in (("SUMMARY.md", EN), ("SUMMARY.pt-BR.md", PT)):
        path = os.path.join(ROOT, name)
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(build(T))
        print(f"wrote {os.path.normpath(path)}")


if __name__ == "__main__":
    main()
