//! Phase 3 acceptance benchmark: ~1 million payments streamed through the gross engine
//! with full flow statistics (pair matrix), twice, checking bit-identical results.
//! `cargo run --release -p loopsettle-sim --example bench_stream`

use loopsettle_core::*;
use loopsettle_sim::*;
use std::time::Instant;

fn run(cfg: &ScenarioConfig) -> (u64, Vec<i64>, Vec<i64>, i128) {
    let mut runs = vec![EngineRun::new(Box::new(GrossRealTime), Ledger::unlimited(cfg.participants))];
    let mut flow = FlowStats::new(cfg.participants, true);
    drive(PaymentStream::new(cfg), &mut runs, &mut flow, cfg.horizon_ms());
    (flow.count, flow.ubs(), runs[0].ledger.balances().to_vec(), flow.gross_value)
}

fn main() {
    let cfg = ScenarioConfig::default();
    let t0 = Instant::now();
    let a = run(&cfg);
    let t1 = t0.elapsed();
    let b = run(&cfg);
    assert_eq!(a, b, "same seed must give identical results");
    println!(
        "BENCH payments={} gross_brl={:.0} ub_total_brl={:.0} first_run_s={:.2} identical=true peak_rss_mib={:.1}",
        a.0,
        a.3 as f64 / 100.0,
        a.1.iter().map(|&x| x as f64).sum::<f64>() / 100.0,
        t1.as_secs_f64(),
        sys::peak_rss_mib()
    );
}
