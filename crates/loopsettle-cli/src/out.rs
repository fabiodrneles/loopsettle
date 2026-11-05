//! Writing results: `results/<experiment>/` gets CSV tables, a JSON summary and the
//! metadata (seed, code version, command) needed to reproduce them. No timestamps are
//! written, so re-running a command reproduces its files byte for byte.

use loopsettle_core::json::Json;
use std::fs;
use std::path::PathBuf;

pub struct Results {
    pub dir: PathBuf,
    pub summary: Json,
}

/// Code version: `git describe` of the working tree if available, else the crate version.
pub fn code_version() -> String {
    let git = std::process::Command::new("git").args(["describe", "--always", "--dirty", "--tags"]).output();
    match git {
        Ok(o) if o.status.success() => format!("{}+git.{}", env!("CARGO_PKG_VERSION"), String::from_utf8_lossy(&o.stdout).trim()),
        _ => env!("CARGO_PKG_VERSION").to_string(),
    }
}

impl Results {
    pub fn new(name: &str, seed: u64, config_file: &str) -> Self {
        let dir = PathBuf::from("results").join(name);
        fs::create_dir_all(&dir).expect("cannot create results directory");
        let summary = Json::obj()
            .set("experiment", name)
            .set("seed", seed)
            .set("code_version", code_version())
            .set("config_file", config_file)
            .set("command", format!("cargo run --release -p loopsettle-cli -- experiment {name}"));
        Results { dir, summary }
    }

    pub fn csv(&self, file: &str, content: &str) {
        let p = self.dir.join(file);
        fs::write(&p, content).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        eprintln!("  wrote {}", p.display());
    }

    pub fn set(&mut self, key: &str, v: impl Into<Json>) {
        self.summary.push(key, v);
    }

    pub fn finish(mut self) {
        self.summary.push("peak_rss_mib", loopsettle_core::sys::peak_rss_mib().round());
        let p = self.dir.join("summary.json");
        let mut s = self.summary.to_string_pretty();
        s.push('\n');
        fs::write(&p, s).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        eprintln!("  wrote {}", p.display());
    }
}

/// Build a CSV from a header and rows of already-formatted cells.
pub fn table(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut s = header.join(",");
    s.push('\n');
    for r in rows {
        s.push_str(&r.join(","));
        s.push('\n');
    }
    s
}

pub fn f(x: f64) -> String {
    loopsettle_core::json::fmt_f64(x)
}
