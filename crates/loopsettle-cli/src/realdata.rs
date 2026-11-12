//! `loopsettle bound <payments.csv>` and `loopsettle settle <payments.csv>`: run LoopSettle on
//! a real payment file. The file never leaves the machine and is never loaded whole into
//! memory; the outputs are aggregates (per-participant rows only with `--per-participant`).
//! File format: `crates/loopsettle-sim/src/csvfile.rs`.

use crate::out::{f, table};
use loopsettle_core::json::Json;
use loopsettle_sim::bound::{delay_bounds_from, liquidity_floors_from};
use loopsettle_sim::csvfile::CsvSchema;
use loopsettle_sim::{run_payments, EngineSpec, Opening};
use std::path::PathBuf;

const DEFAULT_DELAYS_S: &[f64] = &[1.0, 2.0, 5.0, 10.0, 60.0, 900.0, 3600.0, 21600.0, 86400.0];
const DEFAULT_ENGINES: &[&str] = &["gross", "micro:all:500:2000", "micro:all:500:10000", "deferred:8640", "deferred:96", "deferred:24", "deferred:1"];

struct Opts {
    file: String,
    out: PathBuf,
    delays_ms: Vec<u64>,
    engines: Vec<String>,
    per_participant: bool,
    /// refuse per-participant output for files with fewer participants than this
    min_participants: usize,
    /// write the original identifiers of the file instead of P001, P002, ...
    keep_names: bool,
    report: bool,
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let file = args.first().ok_or("missing payments file")?.clone();
    let mut o = Opts {
        file: file.clone(),
        out: PathBuf::from("loopsettle-out").join(std::path::Path::new(&file).file_stem().and_then(|s| s.to_str()).unwrap_or("input")),
        delays_ms: DEFAULT_DELAYS_S.iter().map(|s| (s * 1000.0) as u64).collect(),
        engines: DEFAULT_ENGINES.iter().map(|s| s.to_string()).collect(),
        per_participant: false,
        min_participants: 5,
        keep_names: false,
        report: false,
    };
    let mut i = 1;
    while i < args.len() {
        let val = || args.get(i + 1).cloned().ok_or(format!("{} needs a value", args[i]));
        match args[i].as_str() {
            "--out" => o.out = PathBuf::from(val()?),
            "--delays" => o.delays_ms = val()?.split(',').map(|s| s.trim().parse::<f64>().map(|x| (x * 1000.0) as u64).map_err(|e| format!("--delays: {e}"))).collect::<Result<_, _>>()?,
            "--engines" => o.engines = val()?.split(',').map(|s| s.trim().to_string()).collect(),
            "--min-participants" => o.min_participants = val()?.parse().map_err(|e| format!("--min-participants: {e}"))?,
            flag @ ("--per-participant" | "--keep-names" | "--report") => {
                match flag {
                    "--per-participant" => o.per_participant = true,
                    "--keep-names" => o.keep_names = true,
                    _ => o.report = true,
                }
                i += 1;
                continue;
            }
            other => return Err(format!("unknown option {other}")),
        }
        i += 2;
    }
    Ok(o)
}

fn write(dir: &PathBuf, name: &str, content: &str) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let p = dir.join(name);
    std::fs::write(&p, content).map_err(|e| format!("{}: {e}", p.display()))?;
    eprintln!("  wrote {}", p.display());
    Ok(())
}

/// `settle = true` also runs the settlement engines (gross, micro-cycles, deferred netting).
pub fn run(args: &[String], settle: bool) -> Result<(), String> {
    let o = parse(args)?;
    let s = CsvSchema::scan(&o.file)?;
    let n = s.participants();
    if o.per_participant && n < o.min_participants {
        return Err(format!(
            "--per-participant refused: the file has {n} participants, fewer than --min-participants {} (rows of so few participants can identify each one's payments); aggregates are still available without --per-participant",
            o.min_participants
        ));
    }
    let span_ms = s.last_ms - s.first_ms;
    eprintln!("{}: {} payments, {} participants, {} rows skipped, span {:.1} h", o.file, s.rows, n, s.skipped, span_ms as f64 / 3.6e6);

    let mut specs = vec![EngineSpec::Gross];
    if settle {
        for e in &o.engines {
            let sp = EngineSpec::parse(e)?;
            if sp != EngineSpec::Gross {
                specs.push(sp);
            }
        }
    }
    let r = run_payments(n, s.payments()?, &specs, &Opening::Unlimited, span_ms);
    let ub = r.ub();
    let lb = r.lb();
    let bounds = delay_bounds_from(n, || s.payments().expect("file was readable a moment ago"), &o.delays_ms);
    let floors = liquidity_floors_from(n, || s.payments().expect("file was readable a moment ago"), &o.delays_ms);
    let pct = |x: i128| if ub == 0 { 0.0 } else { 100.0 * x as f64 / ub as f64 };

    let mut brows = Vec::new();
    let mut bjson = Vec::new();
    for (k, &d) in o.delays_ms.iter().enumerate() {
        let total: i128 = bounds[k].iter().map(|&v| v as i128).sum();
        let floor: i128 = floors[k].iter().map(|&v| v as i128).sum();
        brows.push(vec![f(d as f64 / 1000.0), f(total as f64 / 100.0), f(pct(total)), f(floor as f64 / 100.0), f((ub - floor) as f64 / 100.0), f(pct(ub - floor))]);
        bjson.push(
            Json::obj()
                .set("max_delay_s", d as f64 / 1000.0)
                .set("bound", total as f64 / 100.0)
                .set("bound_pct_of_ub", pct(total))
                .set("liquidity_floor", floor as f64 / 100.0)
                .set("tight_bound", (ub - floor) as f64 / 100.0)
                .set("tight_bound_pct_of_ub", pct(ub - floor)),
        );
    }
    let bheader = ["max_delay_s", "delay_bound", "delay_bound_pct_of_ub", "liquidity_floor", "tight_bound", "tight_bound_pct_of_ub"];
    println!("UB (liquidity for real-time gross settlement): {:.2}", ub as f64 / 100.0);
    println!("LB (one netting at the end of the file's period): {:.2}  (LB/UB = {:.4})", lb as f64 / 100.0, if ub == 0 { 0.0 } else { lb as f64 / ub as f64 });
    println!("\nCeiling on the saving of ANY mechanism that settles each payment within d\n(delay_bound = window form; tight_bound = UB - liquidity floor, always <= delay_bound):\n{}", table(&bheader, &brows));
    write(&o.out, "bound.csv", &table(&bheader, &brows))?;

    let mut summary = Json::obj()
        .set("file", o.file.as_str())
        .set("payments", s.rows)
        .set("rows_skipped", s.skipped)
        .set("participants", n)
        .set("span_hours", span_ms as f64 / 3.6e6)
        .set("gross_value", r.flow.gross_value as f64 / 100.0)
        .set("ub", ub as f64 / 100.0)
        .set("lb", lb as f64 / 100.0)
        .set("delay_bound", Json::Arr(bjson))
        .set("note", "aggregates only; amounts in the file's currency units; bound per docs/METHODOLOGY.md section 9");

    if settle {
        let eheader = ["engine", "liquidity", "saving_vs_ub_pct", "delay_p50_ms", "delay_p95_ms", "share_within_2800ms", "value_share_in_cycles"];
        let mut erows = Vec::new();
        let mut ejson = Vec::new();
        for e in &r.results {
            let m = &e.metrics;
            let settled = m.settled_value.max(1) as f64;
            erows.push(vec![
                e.name.clone(),
                f(e.liquidity as f64 / 100.0),
                f(100.0 * e.saving_vs(ub)),
                e.delay_q(0.5).to_string(),
                e.delay_q(0.95).to_string(),
                f(m.share_within_ms(2_800).unwrap_or_else(|| m.delay.fraction_at_most(2_800))),
                f(m.value_by_kind[1] as f64 / settled),
            ]);
            ejson.push(e.to_json(ub, lb));
        }
        println!("\nSettlement designs (unlimited liquidity; saving relative to UB):\n{}", table(&eheader, &erows));
        write(&o.out, "engines.csv", &table(&eheader, &erows))?;
        summary = summary.set("engines", Json::Arr(ejson));
    }
    if o.per_participant {
        eprintln!(
            "  note: participants.csv has one row per participant ({}); review it before sharing",
            if o.keep_names { "with the file's own identifiers" } else { "labelled P001... in order of first appearance; pass --keep-names for the file's identifiers" }
        );
        let mut prows = Vec::new();
        for i in 0..n {
            // Original identifiers leave the machine only on request.
            let label = if o.keep_names { s.names[i].clone() } else { format!("P{:03}", i + 1) };
            let mut row = vec![label, f(r.flow.out_value[i] as f64 / 100.0), f(r.flow.in_value[i] as f64 / 100.0), f(r.flow.ub(i as u32) as f64 / 100.0), f(r.flow.lb(i as u32) as f64 / 100.0)];
            row.extend(bounds.iter().map(|b| f(b[i] as f64 / 100.0)));
            row.extend(floors.iter().map(|b| f(b[i] as f64 / 100.0)));
            prows.push(row);
        }
        let mut h: Vec<String> = ["participant", "out", "in", "ub", "lb"].iter().map(|s| s.to_string()).collect();
        h.extend(o.delays_ms.iter().map(|d| format!("bound_{}s", d / 1000)));
        h.extend(o.delays_ms.iter().map(|d| format!("floor_{}s", d / 1000)));
        let hr: Vec<&str> = h.iter().map(String::as_str).collect();
        write(&o.out, "participants.csv", &table(&hr, &prows))?;
    }
    if o.report {
        let curve: Vec<(f64, f64, f64, f64)> = o
            .delays_ms
            .iter()
            .enumerate()
            .map(|(k, &d)| {
                let floor: i128 = floors[k].iter().map(|&v| v as i128).sum();
                (d as f64 / 1000.0, floor as f64 / 100.0, (ub - floor) as f64 / 100.0, pct(ub - floor))
            })
            .collect();
        let engines: Vec<(String, f64, u64)> = if settle { r.results.iter().map(|e| (e.name.clone(), 100.0 * e.saving_vs(ub), e.delay_q(0.95))).collect() } else { Vec::new() };
        let file_hash = std::fs::File::open(&o.file).map(|fh| crate::out::fnv1a64(std::io::Read::bytes(std::io::BufReader::new(fh)).map_while(Result::ok))).unwrap_or(0);
        let info = ReportInfo { payments: s.rows, skipped: s.skipped, participants: n, span_hours: span_ms as f64 / 3.6e6, gross: r.flow.gross_value as f64 / 100.0, ub: ub as f64 / 100.0, lb: lb as f64 / 100.0, file_hash, curve: &curve, engines: &engines };
        write(&o.out, "report.md", &report(&info, false))?;
        write(&o.out, "report.pt-BR.md", &report(&info, true))?;
        let chart = crate::svg::Chart {
            title: "Ceiling on the liquidity saving of any mechanism with deadline d".into(),
            x_label: "maximum settlement delay d (s, log)".into(),
            y_label: "% of UB".into(),
            series: vec![crate::svg::Series { label: "tight bound (UB - liquidity floor)".into(), points: curve.iter().map(|c| (c.0, c.3)).collect(), line: true }],
            log_x: true,
        };
        write(&o.out, "fig_bound.svg", &chart.render())?;
    }
    let mut js = summary.to_string_pretty();
    js.push('\n');
    write(&o.out, "summary.json", &js)?;
    Ok(())
}

/// `loopsettle export <experiment> <out.csv> [--payments N] [--participants N]`: write a
/// synthetic scenario as a payment file in the format `bound`/`settle` read (for trying them
/// out, or for comparing a real file with the calibrated model).
pub fn export(args: &[String]) -> Result<(), String> {
    use std::io::Write;
    let name = args.first().ok_or("missing experiment name")?;
    let out = args.get(1).ok_or("missing output file")?;
    let src = std::fs::read_to_string(format!("experiments/{name}.toml")).map_err(|e| format!("experiments/{name}.toml: {e}"))?;
    let doc = loopsettle_sim::toml_lite::parse(&src)?;
    let mut cfg = loopsettle_sim::ScenarioConfig::from_doc(&doc)?;
    let mut i = 2;
    while i + 1 < args.len() {
        let v: f64 = args[i + 1].parse().map_err(|e| format!("{}: {e}", args[i]))?;
        match args[i].as_str() {
            "--payments" => cfg.payments_per_day = v,
            "--participants" => cfg.participants = v as usize,
            o => return Err(format!("unknown option {o}")),
        }
        i += 2;
    }
    let file = std::fs::File::create(out).map_err(|e| format!("{out}: {e}"))?;
    let mut w = std::io::BufWriter::new(file);
    writeln!(w, "t_ms,from,to,amount").map_err(|e| e.to_string())?;
    let mut k = 0u64;
    for p in loopsettle_sim::PaymentStream::new(&cfg) {
        writeln!(w, "{},P{:03},P{:03},{}.{:02}", p.t_arrival, p.payer, p.payee, p.amount / 100, p.amount % 100).map_err(|e| e.to_string())?;
        k += 1;
    }
    eprintln!("wrote {out}: {k} payments, {} participants (synthetic, seed {})", cfg.participants, cfg.seed);
    Ok(())
}

struct ReportInfo<'a> {
    payments: u64,
    skipped: u64,
    participants: usize,
    span_hours: f64,
    gross: f64,
    ub: f64,
    lb: f64,
    file_hash: u64,
    /// (delay s, liquidity floor, tight bound, tight bound % of UB)
    curve: &'a [(f64, f64, f64, f64)],
    /// (engine, saving % of UB, p95 delay ms); empty for `bound`
    engines: &'a [(String, f64, u64)],
}

/// A report that can be shared: aggregates only, the method in two paragraphs, what the numbers
/// do and do not mean. The input file is identified by a hash, never by its content or path.
fn report(i: &ReportInfo, pt: bool) -> String {
    let n2 = |x: f64| format!("{x:.2}");
    let mut m = String::new();
    let mut line = |s: String| {
        m.push_str(&s);
        m.push('\n');
    };
    if pt {
        line("# LoopSettle — relatório de liquidez do arquivo de pagamentos\n".into());
        line(format!("Gerado pelo LoopSettle {} (`loopsettle bound/settle --report`). Arquivo de entrada identificado só pelo hash FNV-1a `{:016x}`; nenhum pagamento individual aparece aqui.\n", crate::out::code_version(), i.file_hash));
        line("## O arquivo\n".into());
        line(format!("- {} pagamentos entre {} participantes, em {:.1} horas ({} linhas ignoradas: valor não positivo ou pagador igual ao recebedor).", i.payments, i.participants, i.span_hours, i.skipped));
        line(format!("- Valor bruto: {}.", n2(i.gross)));
        line(format!("- **UB** (liquidez para liquidar tudo na hora, sem rejeitar nada): {}.", n2(i.ub)));
        line(format!("- **LB** (o que uma única compensação no fim do período exigiria): {} (LB/UB = {:.4}).\n", n2(i.lb), if i.ub > 0.0 { i.lb / i.ub } else { 0.0 }));
        line("## Quanto um mecanismo com prazo pode economizar, no máximo\n".into());
        line("Para cada prazo `d`, o **piso de liquidez** é a menor liquidez que *qualquer* mecanismo que liquide cada pagamento em até `d` depois da chegada pode precisar, somada sobre os participantes. `UB − piso` é o teto da economia, seja qual for o algoritmo.\n".into());
        line("| prazo d (s) | piso de liquidez | teto da economia | teto em % do UB |\n|---|---|---|---|".into());
    } else {
        line("# LoopSettle — liquidity report of a payment file\n".into());
        line(format!("Generated by LoopSettle {} (`loopsettle bound/settle --report`). The input file is identified only by its FNV-1a hash `{:016x}`; no individual payment appears here.\n", crate::out::code_version(), i.file_hash));
        line("## The file\n".into());
        line(format!("- {} payments among {} participants, over {:.1} hours ({} rows skipped: non-positive amount or payer equal to payee).", i.payments, i.participants, i.span_hours, i.skipped));
        line(format!("- Gross value: {}.", n2(i.gross)));
        line(format!("- **UB** (liquidity to settle everything on arrival, rejecting nothing): {}.", n2(i.ub)));
        line(format!("- **LB** (what a single netting at the end of the period would need): {} (LB/UB = {:.4}).\n", n2(i.lb), if i.ub > 0.0 { i.lb / i.ub } else { 0.0 }));
        line("## The most a deadline-bound mechanism can save\n".into());
        line("For each deadline `d`, the **liquidity floor** is the least liquidity that *any* mechanism settling every payment within `d` of its arrival can need, summed over participants. `UB − floor` is the ceiling on the saving, whatever the algorithm.\n".into());
        line("| deadline d (s) | liquidity floor | ceiling on the saving | ceiling, % of UB |\n|---|---|---|---|".into());
    }
    for c in i.curve {
        line(format!("| {} | {} | {} | {:.4}% |", c.0, n2(c.1), n2(c.2), c.3));
    }
    line(String::new());
    line(if pt { "Figura: [`fig_bound.svg`](fig_bound.svg).\n".into() } else { "Figure: [`fig_bound.svg`](fig_bound.svg).\n".into() });
    if !i.engines.is_empty() {
        if pt {
            line("## O que os desenhos simulados obtêm (liquidez ilimitada)\n".into());
            line("| desenho | economia em % do UB | atraso p95 (ms) |\n|---|---|---|".into());
        } else {
            line("## What the simulated designs realise (unlimited liquidity)\n".into());
            line("| design | saving, % of UB | p95 delay (ms) |\n|---|---|---|".into());
        }
        for (name, saving, p95) in i.engines {
            line(format!("| {name} | {saving:.4}% | {p95} |"));
        }
        line(String::new());
    }
    if pt {
        line("## Como ler\n".into());
        line("- O teto é um teorema (docs/METHODOLOGY.pt-BR.md, seção 9b), não uma estimativa: vale para micro-ciclos, compensação diferida, filas e qualquer otimizador com o mesmo prazo. Ele é atingido por um participante isolado; a soma sobre todos pode não ser atingível ao mesmo tempo.".into());
        line("- Se o teto para o prazo que interessa for pequeno, um mecanismo de economia de liquidez com esse prazo não se paga, por melhor que seja.".into());
        line("- Um teto grande não garante economia: diz só que ela não está excluída.".into());
        line("\n## Limites deste relatório\n".into());
        line("- Mede o período do arquivo, com saldos de abertura zero e sem aportes durante o período.".into());
        line("- UB e LB dependem de o arquivo ter *todos* os pagamentos liquidados entre esses participantes no período.".into());
        line("- Os valores estão nas unidades do arquivo.".into());
    } else {
        line("## How to read it\n".into());
        line("- The ceiling is a theorem (docs/METHODOLOGY.md, section 9b), not an estimate: it covers micro-cycles, deferred netting, queues and any optimiser with the same deadline. It is attained by one participant alone; the sum over all of them may not be attainable at once.".into());
        line("- If the ceiling for the deadline of interest is small, a liquidity-saving mechanism with that deadline cannot pay off, however good it is.".into());
        line("- A large ceiling does not guarantee a saving: it only says one is not ruled out.".into());
        line("\n## Limits of this report\n".into());
        line("- It measures the period of the file, with zero opening balances and no top-ups during the period.".into());
        line("- UB and LB depend on the file holding *all* payments settled among these participants in the period.".into());
        line("- Amounts are in the file's own units.".into());
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn csv(dir: &std::path::Path, participants: usize) -> String {
        std::fs::create_dir_all(dir).unwrap();
        let mut t = String::from("t_ms,from,to,amount\n");
        for k in 0..40usize {
            t.push_str(&format!("{},bank{},bank{},{}.50\n", k * 700, k % participants, (k + 1) % participants, 10 + k));
        }
        let p = dir.join("p.csv");
        std::fs::write(&p, t).unwrap();
        p.to_str().unwrap().to_string()
    }

    /// Per-participant rows are refused for small files, labelled P001... by default, and the
    /// report holds aggregates only (no identifier of the file).
    #[test]
    fn privacy_defaults_and_report() {
        let dir = std::env::temp_dir().join(format!("loopsettle-realdata-{}", std::process::id()));
        let arg = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let small = csv(&dir.join("small"), 3);
        let out = dir.join("out");
        let o = out.to_str().unwrap();
        let e = run(&arg(&[&small, "--per-participant", "--out", o]), false).unwrap_err();
        assert!(e.contains("refused") && e.contains("3 participants"), "{e}");
        assert!(run(&arg(&[&small, "--out", o]), false).is_ok(), "aggregates are always available");

        let big = csv(&dir.join("big"), 8);
        run(&arg(&[&big, "--per-participant", "--report", "--delays", "1,10", "--out", o]), true).unwrap();
        let parts = std::fs::read_to_string(out.join("participants.csv")).unwrap();
        assert!(parts.contains("P001") && !parts.contains("bank"), "identifiers are replaced by default");
        for name in ["report.md", "report.pt-BR.md"] {
            let r = std::fs::read_to_string(out.join(name)).unwrap();
            assert!(r.contains("FNV-1a") && r.contains("| 10 |") && !r.contains("bank") && !r.contains("p.csv"), "{name}");
        }
        assert!(out.join("fig_bound.svg").exists());
        run(&arg(&[&big, "--per-participant", "--keep-names", "--out", o]), false).unwrap();
        assert!(std::fs::read_to_string(out.join("participants.csv")).unwrap().contains("bank0"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
