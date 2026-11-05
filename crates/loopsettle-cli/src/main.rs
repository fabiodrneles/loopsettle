//! `loopsettle` — reproducible experiments.
//!
//! ```text
//! cargo run --release -p loopsettle-cli -- experiment <name>   # experiments/<name>.toml
//! cargo run --release -p loopsettle-cli -- list
//! ```
//! Argument parsing is hand-written (no `clap`) to keep compile times low.

mod experiments;
mod experiments2;
mod out;
mod svg;

use loopsettle_sim::toml_lite;
use std::time::Instant;

fn usage() -> ! {
    eprintln!("usage: loopsettle experiment <name> [--quick]\n       loopsettle list");
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
        Some("topology") => {
            let path = args.get(1).unwrap_or_else(|| usage());
            if let Err(e) = experiments2::topology_csv(path) {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        }
        Some("experiment") => {
            let name = args.get(1).unwrap_or_else(|| usage());
            let quick = args.iter().any(|a| a == "--quick");
            let path = format!("experiments/{name}.toml");
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
