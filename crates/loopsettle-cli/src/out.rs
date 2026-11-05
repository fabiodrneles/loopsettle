//! Writing results: `results/<experiment>/` gets CSV tables, a JSON summary and the
//! metadata (seed, code version, command) needed to reproduce them. No timestamps are
//! written, so re-running a command reproduces its files byte for byte.

use loopsettle_core::json::Json;
use std::fs;
use std::path::PathBuf;

pub struct Results {
    pub dir: PathBuf,
    pub summary: Json,
    /// machine measurements (time, memory): written to `run.json`, which is NOT expected to
    /// be byte-identical across runs, unlike `summary.json`
    pub run: Json,
}

/// Code version: `git describe` of the working tree if available, else the crate version.
/// The tree counts as "dirty" only for changes outside `results/` (writing results must not
/// change the version recorded by the next experiment of the same `make all`).
pub fn code_version() -> String {
    use std::process::Command;
    // Last commit that touched anything outside results/ (committing results must not
    // change the version they record).
    let git = Command::new("git").args(["log", "-1", "--format=%h", "--", ".", ":(exclude)results"]).output();
    match git {
        Ok(o) if o.status.success() => {
            let dirty = Command::new("git")
                .args(["status", "--porcelain", "--untracked-files=no", "--", ".", ":(exclude)results"])
                .output()
                .map(|s| !s.stdout.is_empty())
                .unwrap_or(false);
            format!("{}+git.{}{}", env!("CARGO_PKG_VERSION"), String::from_utf8_lossy(&o.stdout).trim(), if dirty { "-dirty" } else { "" })
        }
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
        Results { dir, summary, run: Json::obj() }
    }

    pub fn csv(&self, file: &str, content: &str) {
        let p = self.dir.join(file);
        fs::write(&p, content).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        eprintln!("  wrote {}", p.display());
    }

    pub fn set(&mut self, key: &str, v: impl Into<Json>) {
        self.summary.push(key, v);
    }

    /// Replace the default `experiment <name>` command (for commands that are not experiments).
    pub fn set_command(&mut self, cmd: &str) {
        if let Json::Obj(kv) = &mut self.summary {
            if let Some(e) = kv.iter_mut().find(|(k, _)| k == "command") {
                e.1 = Json::from(cmd);
            }
        }
    }

    pub fn measure(&mut self, key: &str, v: impl Into<Json>) {
        self.run.push(key, v);
    }

    pub fn finish(mut self) {
        self.run.push("peak_rss_mib", loopsettle_core::sys::peak_rss_mib().round());
        for (name, j) in [("summary.json", &self.summary), ("run.json", &self.run)] {
            let p = self.dir.join(name);
            let mut s = j.to_string_pretty();
            s.push('\n');
            fs::write(&p, s).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
            eprintln!("  wrote {}", p.display());
        }
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
