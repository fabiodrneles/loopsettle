//! Streaming reader for real payment files, so institutions can run LoopSettle on their own
//! data without the data leaving their machine (W9, `docs/PAPER_READINESS.md`).
//!
//! Format: CSV with a header line containing at least the columns `time`, `from`, `to` and
//! `amount` (any order, extra columns ignored). `time` is either seconds since the start of the
//! file's period (`12.5`), a clock time `HH:MM:SS[.fff]` (`09:30:00.250`), or milliseconds if the
//! column is called `t_ms`. `from`/`to` are any participant identifiers. `amount` is a decimal
//! number in currency units (`1234.56`), converted to cents exactly (no floating point).
//! Rows must be sorted by time. Rows with `from == to` or a non-positive amount are skipped and
//! counted.
//!
//! Memory: one map of participant identifiers; payments are never all held in memory. Each
//! analysis re-reads the file (`CsvPayments::open` again), which keeps memory O(participants).

use loopsettle_core::{Cents, Payment, TimeMs};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};

/// Column positions and the identifier map built by a first pass over the file.
#[derive(Clone, Debug)]
pub struct CsvSchema {
    pub path: String,
    time: usize,
    time_is_ms: bool,
    from: usize,
    to: usize,
    amount: usize,
    /// identifier -> dense participant index, in order of first appearance
    pub ids: HashMap<String, u32>,
    pub names: Vec<String>,
    pub rows: u64,
    pub skipped: u64,
    pub first_ms: TimeMs,
    pub last_ms: TimeMs,
}

fn parse_cents(s: &str) -> Option<Cents> {
    let s = s.trim().trim_matches('"');
    let (neg, s) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s),
    };
    let (int, frac) = match s.split_once('.') {
        Some((i, f)) => (i, f),
        None => (s, ""),
    };
    if int.is_empty() && frac.is_empty() || !int.bytes().all(|b| b.is_ascii_digit()) || !frac.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let units: i64 = if int.is_empty() { 0 } else { int.parse().ok()? };
    let mut cents: i64 = 0;
    for (k, b) in frac.bytes().take(2).enumerate() {
        cents += (b - b'0') as i64 * if k == 0 { 10 } else { 1 };
    }
    // round half up on the third decimal
    if frac.len() > 2 && frac.as_bytes()[2] >= b'5' {
        cents += 1;
    }
    let v = units.checked_mul(100)?.checked_add(cents)?;
    Some(if neg { -v } else { v })
}

fn parse_time_ms(s: &str, is_ms: bool) -> Option<TimeMs> {
    let s = s.trim().trim_matches('"');
    if is_ms {
        return s.parse().ok();
    }
    if s.contains(':') {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return None;
        }
        let h: u64 = parts[0].parse().ok()?;
        let m: u64 = parts[1].parse().ok()?;
        let sec: f64 = parts[2].parse().ok()?;
        return Some(((h * 3600 + m * 60) as f64 * 1000.0 + sec * 1000.0).round() as TimeMs);
    }
    let sec: f64 = s.parse().ok()?;
    (sec >= 0.0).then(|| (sec * 1000.0).round() as TimeMs)
}

impl CsvSchema {
    /// First pass: find the columns, build the identifier map, check the time order.
    pub fn scan(path: &str) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| format!("{path}: {e}"))?;
        let mut lines = BufReader::new(file).lines();
        let header = lines.next().ok_or(format!("{path}: empty file"))?.map_err(|e| e.to_string())?;
        let cols: Vec<String> = header.split(',').map(|c| c.trim().trim_matches('"').to_lowercase()).collect();
        let find = |names: &[&str]| cols.iter().position(|c| names.contains(&c.as_str()));
        let (time, time_is_ms) = match (find(&["t_ms"]), find(&["time", "t", "timestamp"])) {
            (Some(i), _) => (i, true),
            (None, Some(i)) => (i, false),
            _ => return Err(format!("{path}: header needs a `time` (or `t_ms`) column; found {cols:?}")),
        };
        let from = find(&["from", "payer", "sender"]).ok_or(format!("{path}: header needs a `from` column"))?;
        let to = find(&["to", "payee", "receiver"]).ok_or(format!("{path}: header needs a `to` column"))?;
        let amount = find(&["amount", "value"]).ok_or(format!("{path}: header needs an `amount` column"))?;
        let mut s = CsvSchema { path: path.into(), time, time_is_ms, from, to, amount, ids: HashMap::new(), names: Vec::new(), rows: 0, skipped: 0, first_ms: 0, last_ms: 0 };
        let mut last: Option<TimeMs> = None;
        for (k, line) in lines.enumerate() {
            let line = line.map_err(|e| e.to_string())?;
            if line.trim().is_empty() {
                continue;
            }
            match s.parse_row(&line)? {
                None => s.skipped += 1,
                Some((t, a, b, _)) => {
                    if let Some(l) = last {
                        if t < l {
                            return Err(format!("{path}: line {}: time goes backwards; sort the file by time first", k + 2));
                        }
                    } else {
                        s.first_ms = t;
                    }
                    last = Some(t);
                    for id in [a, b] {
                        if !s.ids.contains_key(id) {
                            s.ids.insert(id.to_string(), s.names.len() as u32);
                            s.names.push(id.to_string());
                        }
                    }
                    s.rows += 1;
                }
            }
        }
        s.last_ms = last.unwrap_or(0);
        if s.names.len() < 2 {
            return Err(format!("{path}: need at least two participants"));
        }
        Ok(s)
    }

    /// `Ok(None)` for rows to skip (self-payments, non-positive amounts).
    fn parse_row<'a>(&self, line: &'a str) -> Result<Option<(TimeMs, &'a str, &'a str, Cents)>, String> {
        let c: Vec<&str> = line.split(',').map(|x| x.trim().trim_matches('"')).collect();
        let need = self.time.max(self.from).max(self.to).max(self.amount);
        if c.len() <= need {
            return Err(format!("{}: row has {} columns, expected at least {}: {line}", self.path, c.len(), need + 1));
        }
        let t = parse_time_ms(c[self.time], self.time_is_ms).ok_or(format!("{}: bad time `{}`", self.path, c[self.time]))?;
        let amt = parse_cents(c[self.amount]).ok_or(format!("{}: bad amount `{}`", self.path, c[self.amount]))?;
        if amt <= 0 || c[self.from] == c[self.to] {
            return Ok(None);
        }
        Ok(Some((t, c[self.from], c[self.to], amt)))
    }

    pub fn participants(&self) -> usize {
        self.names.len()
    }

    /// A fresh streaming pass over the payments, times shifted so the first payment is at 0.
    pub fn payments(&self) -> Result<CsvPayments, String> {
        let file = std::fs::File::open(&self.path).map_err(|e| format!("{}: {e}", self.path))?;
        let mut lines = BufReader::new(file).lines();
        lines.next();
        Ok(CsvPayments { schema: self.clone_light(), lines, next_id: 0 })
    }

    fn clone_light(&self) -> CsvSchema {
        self.clone()
    }
}

pub struct CsvPayments {
    schema: CsvSchema,
    lines: std::io::Lines<BufReader<std::fs::File>>,
    next_id: u64,
}

impl Iterator for CsvPayments {
    type Item = Payment;
    fn next(&mut self) -> Option<Payment> {
        loop {
            let line = self.lines.next()?.ok()?;
            if line.trim().is_empty() {
                continue;
            }
            // The first pass already validated every row, so errors cannot happen here.
            if let Ok(Some((t, a, b, amount))) = self.schema.parse_row(&line) {
                let id = self.next_id;
                self.next_id += 1;
                return Some(Payment { id, t_arrival: t - self.schema.first_ms, payer: self.schema.ids[a], payee: self.schema.ids[b], amount });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cents_are_exact() {
        assert_eq!(parse_cents("1234.56"), Some(123_456));
        assert_eq!(parse_cents("0.1"), Some(10));
        assert_eq!(parse_cents("7"), Some(700));
        assert_eq!(parse_cents("2.345"), Some(235));
        assert_eq!(parse_cents("-3.10"), Some(-310));
        assert_eq!(parse_cents("1e3"), None);
        assert_eq!(parse_cents(""), None);
    }

    #[test]
    fn times_in_three_formats() {
        assert_eq!(parse_time_ms("12.5", false), Some(12_500));
        assert_eq!(parse_time_ms("09:30:00.250", false), Some(34_200_250));
        assert_eq!(parse_time_ms("1500", true), Some(1_500));
    }

    /// Running through a CSV file gives exactly the same UB, LB and delay bound as running the
    /// same payments in memory (participants are renumbered, so totals are compared).
    #[test]
    fn csv_path_equals_in_memory_path() {
        use crate::bound::{delay_bounds, delay_bounds_from};
        use crate::runner::{run, run_payments, EngineSpec, Opening};
        use crate::{PaymentStream, ScenarioConfig};
        let cfg = ScenarioConfig { participants: 15, payments_per_day: 20_000.0, n_big: 2, n_mid: 3, ..Default::default() };
        let path = std::env::temp_dir().join("loopsettle_csv_equiv.csv");
        let mut txt = String::from("t_ms,from,to,amount\n");
        for p in PaymentStream::new(&cfg) {
            txt.push_str(&format!("{},bank{},bank{},{}.{:02}\n", p.t_arrival, p.payer, p.payee, p.amount / 100, p.amount % 100));
        }
        std::fs::write(&path, txt).unwrap();
        let s = CsvSchema::scan(path.to_str().unwrap()).unwrap();
        let specs = [EngineSpec::Gross, EngineSpec::parse("micro:all:500:10000").unwrap()];
        let mem = run(&cfg, &specs, &Opening::Unlimited, false, None);
        let file = run_payments(s.participants(), s.payments().unwrap(), &specs, &Opening::Unlimited, cfg.horizon_ms());
        assert_eq!((file.ub(), file.lb()), (mem.ub(), mem.lb()));
        assert_eq!(file.results[1].liquidity, mem.results[1].liquidity);
        let d = [2_000u64, 10_000, 900_000];
        let tot = |b: Vec<Vec<i64>>| b.iter().map(|v| v.iter().sum::<i64>()).collect::<Vec<_>>();
        // the file's clock starts at its first payment; shift the synthetic stream the same way
        let first = PaymentStream::new(&cfg).next().unwrap().t_arrival;
        let shifted = || PaymentStream::new(&cfg).map(move |mut p| { p.t_arrival -= first; p });
        assert_eq!(tot(delay_bounds_from(s.participants(), || s.payments().unwrap(), &d)), tot(delay_bounds_from(cfg.participants, shifted, &d)));
        assert_eq!(tot(delay_bounds(&cfg, &d)), tot(delay_bounds_from(cfg.participants, shifted, &d)));
    }

    #[test]
    fn reads_a_small_file_and_rejects_unsorted() {
        let dir = std::env::temp_dir();
        let ok = dir.join("loopsettle_csv_ok.csv");
        std::fs::write(&ok, "time,from,to,amount,memo\n08:00:00,A,B,10.00,x\n08:00:01,B,C,5\n08:00:02,C,C,1\n08:00:03,C,A,5.5\n").unwrap();
        let s = CsvSchema::scan(ok.to_str().unwrap()).unwrap();
        assert_eq!((s.participants(), s.rows, s.skipped), (3, 3, 1));
        let v: Vec<Payment> = s.payments().unwrap().collect();
        assert_eq!(v.len(), 3);
        assert_eq!((v[0].t_arrival, v[0].amount, v[2].t_arrival, v[2].amount), (0, 1000, 3000, 550));
        let bad = dir.join("loopsettle_csv_bad.csv");
        std::fs::write(&bad, "time,from,to,amount\n10,A,B,1\n5,B,A,1\n").unwrap();
        assert!(CsvSchema::scan(bad.to_str().unwrap()).unwrap_err().contains("backwards"));
    }
}
