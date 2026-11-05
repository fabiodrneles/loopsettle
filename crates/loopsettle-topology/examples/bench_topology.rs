//! Phase 2 acceptance benchmark: 200 participants, ~5,000 edges.
//! `cargo run --release -p loopsettle-topology --example bench_topology`

use loopsettle_core::rng::Rng;
use loopsettle_topology::*;
use std::time::Instant;

fn main() {
    let n: u32 = 200;
    let target_edges = 5_000;
    let mut rng = Rng::new(7);
    // Heterogeneous sizes so the graph looks like a payment network (hubs + periphery).
    let size: Vec<f64> = (0..n).map(|i| 1.0 / (1.0 + i as f64).powf(0.8)).collect();
    let table = loopsettle_core::rng::AliasTable::new(&size);
    let mut t = Vec::new();
    let mut seen = std::collections::HashSet::new();
    while seen.len() < target_edges {
        let a = table.sample(&mut rng) as u32;
        let b = table.sample(&mut rng) as u32;
        if a != b && seen.insert((a, b)) {
            let w = (rng.pareto(1_000.0, 1.3) as i64).max(1) * 100;
            t.push((a, b, w));
        }
    }
    let g = FlowGraph::from_triples(n as usize, &t);
    let t0 = Instant::now();
    let r = analyze(&g, &HodgeConfig::default());
    let dt = t0.elapsed();
    println!("{}", r.to_json().to_string_pretty());
    println!(
        "BENCH nodes={} edges={} elapsed_s={:.3} peak_rss_mib={:.1}",
        g.n,
        g.edges.len(),
        dt.as_secs_f64(),
        loopsettle_core::sys::peak_rss_mib()
    );
}
