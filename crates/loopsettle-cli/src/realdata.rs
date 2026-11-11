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
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let file = args.first().ok_or("missing payments file")?.clone();
    let mut o = Opts {
        file: file.clone(),
        out: PathBuf::from("loopsettle-out").join(std::path::Path::new(&file).file_stem().and_then(|s| s.to_str()).unwrap_or("input")),
        delays_ms: DEFAULT_DELAYS_S.iter().map(|s| (s * 1000.0) as u64).collect(),
        engines: DEFAULT_ENGINES.iter().map(|s| s.to_string()).collect(),
        per_participant: false,
    };
    let mut i = 1;
    while i < args.len() {
        let val = || args.get(i + 1).cloned().ok_or(format!("{} needs a value", args[i]));
        match args[i].as_str() {
            "--out" => o.out = PathBuf::from(val()?),
            "--delays" => o.delays_ms = val()?.split(',').map(|s| s.trim().parse::<f64>().map(|x| (x * 1000.0) as u64).map_err(|e| format!("--delays: {e}"))).collect::<Result<_, _>>()?,
            "--engines" => o.engines = val()?.split(',').map(|s| s.trim().to_string()).collect(),
            "--per-participant" => {
                o.per_participant = true;
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
        let mut prows = Vec::new();
        for i in 0..n {
            let mut row = vec![s.names[i].clone(), f(r.flow.out_value[i] as f64 / 100.0), f(r.flow.in_value[i] as f64 / 100.0), f(r.flow.ub(i as u32) as f64 / 100.0), f(r.flow.lb(i as u32) as f64 / 100.0)];
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
