//! Scale of the maximum circulation (CDFD) — specs/08, T2.
//! `cargo run --release -p loopsettle-topology --example bench_cdfd_scale [-- max_edges]`
//!
//! Random payment-like graphs (heavy-tailed node sizes and edge values) with 10^4 .. 10^6 edges;
//! time, augmentations and peak memory of `cdfd_max`. Stops at the first size that takes more
//! than 60 s.

use loopsettle_core::rng::{AliasTable, Rng};
use loopsettle_topology::*;
use std::time::Instant;

fn graph(n: u32, m: usize, seed: u64) -> FlowGraph {
    let mut rng = Rng::new(seed);
    let size: Vec<f64> = (0..n).map(|i| 1.0 / (1.0 + i as f64).powf(0.8)).collect();
    let table = AliasTable::new(&size);
    let mut seen = std::collections::HashSet::with_capacity(m);
    let mut t = Vec::with_capacity(m);
    while t.len() < m {
        let a = table.sample(&mut rng) as u32;
        let b = table.sample(&mut rng) as u32;
        if a != b && seen.insert((a, b)) {
            t.push((a, b, (rng.pareto(1_000.0, 1.3) as i64).max(1) * 100));
        }
    }
    FlowGraph::from_triples(n as usize, &t)
}

fn main() {
    let max_edges: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1_000_000);
    println!("nodes,edges,seconds,augmentations,circular_share,peak_rss_mib");
    for &(n, m) in &[(1_000u32, 10_000usize), (10_000, 100_000), (100_000, 1_000_000)] {
        if m > max_edges {
            break;
        }
        let g = graph(n, m, 11);
        let t0 = Instant::now();
        let c = cdfd_max(&g);
        let s = t0.elapsed().as_secs_f64();
        println!("{},{},{:.3},{},{:.6},{:.0}", g.n, g.edges.len(), s, c.augmentations, c.circular_total as f64 / g.total_flow() as f64, loopsettle_core::sys::peak_rss_mib());
        if s > 60.0 {
            println!("stopped: more than 60 s");
            break;
        }
    }
}
