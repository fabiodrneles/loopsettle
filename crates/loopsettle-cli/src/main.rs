//! `loopsettle` — reproducible experiments.
//!
//! ```text
//! cargo run --release -p loopsettle-cli -- experiment <name>   # experiments/<name>.toml
//! cargo run --release -p loopsettle-cli -- list
//! ```
//! Argument parsing is hand-written (no `clap`) to keep compile times low.

mod experiments;
mod experiments2;
mod experiments3;
mod experiments4;
mod experiments5;
mod experiments6;
mod experiments7;
mod experiments8;
mod out;
mod realdata;
mod stats;
mod svg;

use loopsettle_sim::toml_lite;
use std::time::Instant;

fn usage() -> ! {
    eprintln!(
        "usage: loopsettle experiment <name> [--quick] [--config FILE.toml]\n       loopsettle list\n       loopsettle bound  <payments.csv> [--delays 1,10,60] [--out DIR] [--report] [--per-participant [--min-participants 5] [--keep-names]]\n       loopsettle settle <payments.csv> [--engines gross,deferred:96,micro:all:500:10000] [--delays ...] [--out DIR] [--per-participant]\n       loopsettle sort <in.csv> <out.csv> [--block-rows 1000000]\n       loopsettle export <experiment> <out.csv> [--payments N] [--participants N]\n       loopsettle topology <transfers.csv> [--nulls 20]"
    );
    std::process::exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("list") => {
            for (name, desc) in experiments::ALL {
                println!("{name:28} {desc}");
            }
        }
        Some("export") => {
            if let Err(e) = realdata::export(&args[1..]) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        Some(cmd @ ("bound" | "settle")) => {
            if let Err(e) = realdata::run(&args[1..], cmd == "settle") {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        Some("sort") => {
            let (input, output) = match (args.get(1), args.get(2)) {
                (Some(i), Some(o)) => (i, o),
                _ => usage(),
            };
            let block = match args.iter().position(|a| a == "--block-rows") {
                Some(i) => args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or_else(|| usage()),
                None => 1_000_000,
            };
            let t0 = Instant::now();
            match loopsettle_sim::csvfile::sort_file(input, output, block) {
                Ok(n) => eprintln!("sorted {n} rows into {output} in {:.1} s, peak RSS {:.0} MiB", t0.elapsed().as_secs_f64(), loopsettle_core::sys::peak_rss_mib()),
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some("topology") => {
            let path = args.get(1).unwrap_or_else(|| usage());
            let shuffles = match args.iter().position(|a| a == "--nulls") {
                Some(i) => args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or_else(|| usage()),
                None => 20,
            };
            if let Err(e) = experiments2::topology_csv(path, shuffles) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        Some("experiment") => {
            let name = args.get(1).unwrap_or_else(|| usage());
            let quick = args.iter().any(|a| a == "--quick");
            let path = match args.iter().position(|a| a == "--config") {
                Some(i) => args.get(i + 1).cloned().unwrap_or_else(|| usage()),
                None => format!("experiments/{name}.toml"),
            };
            let src = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                eprintln!("cannot read {path}: {e} (run from the repository root)");
                std::process::exit(1)
            });
            let doc = toml_lite::parse(&src).unwrap_or_else(|e| {
                eprintln!("{path}: {e}");
                std::process::exit(1)
            });
            let t0 = Instant::now();
            eprintln!("experiment {name} ({path})");
            if let Err(e) = experiments::run(name, &path, &doc, quick) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
            eprintln!(
                "done in {:.1} s, peak RSS {:.0} MiB",
                t0.elapsed().as_secs_f64(),
                loopsettle_core::sys::peak_rss_mib()
            );
        }
        _ => usage(),
    }
}
