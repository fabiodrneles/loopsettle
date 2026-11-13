//! Phase 2 invariants: 2 (net positions preserved), 5 (netting never flips direction nor
//! cancels more than the cycle minimum), 6 (Hodge reconstructs F; curl is divergence-free),
//! 8 (the fixed triangle case). Plus: CDFD == LP oracle on 1000 random graphs, CDFD's
//! directional part is acyclic, and the ladder ordering CDFD >= cycle netting.

use loopsettle_core::rng::Rng;
use loopsettle_topology::*;
use proptest::prelude::*;

fn random_graph(n: u32, max_e: usize) -> impl Strategy<Value = FlowGraph> {
    prop::collection::vec((0..n, 1..n, 1i64..1_000), 0..max_e).prop_map(move |v| {
        let t: Vec<(u32, u32, i64)> = v
            .into_iter()
            .map(|(a, off, w)| (a, (a + off) % n, w))
            .collect();
        FlowGraph::from_triples(n as usize, &t)
    })
}

#[test]
fn triangle_case() {
    // Invariant 8. A=0, B=1, C=2: A->B 10, B->C 5, C->A 5.
    let g = FlowGraph::from_triples(3, &[(0, 1, 10), (1, 2, 5), (2, 0, 5)]);

    // Executable netting: the cycle A->B->C->A carries min(10,5,5) = 5.
    let nt = cycle_netting(&g, true);
    assert_eq!(nt.cycles, 1);
    let ab = g
        .edges
        .iter()
        .position(|e| e.from == 0 && e.to == 1)
        .unwrap();
    assert_eq!(nt.cancelled, vec![5, 5, 5].into_iter().collect::<Vec<_>>());
    assert_eq!(nt.residual[ab], 5);

    // Hodge assigns the mean edge flow (10+5+5)/3 = 6.67 to the circular component on every
    // edge: it projects F onto the cycle direction (1,1,1) and does NOT respect the edge
    // bounds -- 6.67 > 5 on B->C and C->A. This is why Hodge is a diagnostic, not an
    // executable netting amount (see docs/METHODOLOGY.md).
    let h = hodge(&g, &HodgeConfig::default());
    for (k, c) in h.cyclic().iter().enumerate() {
        assert!((c - 20.0 / 3.0).abs() < 1e-8, "edge {k}: {c}");
    }
    assert_eq!(h.triangles, 1);
    assert!(
        h.harmonic.iter().all(|x| x.abs() < 1e-8),
        "filled triangle -> all of it is curl"
    );

    // CDFD maximum: 5 per edge = 15 of 20 (75 %), same as netting here.
    let c = cdfd_max(&g);
    assert_eq!(c.circular_total, 15);
    assert!(g.is_acyclic_on(&c.directional));

    let r = analyze(&g, &HodgeConfig::default());
    assert!(
        (r.net.hodge_l1 - 1.0).abs() < 1e-9,
        "Hodge L1 = 20/20 = 100 %"
    );
    assert!((r.net.cdfd_max - 0.75).abs() < 1e-12);
    assert!((r.net.cycles - 0.75).abs() < 1e-12);
}

#[test]
fn cdfd_beats_greedy_netting_example() {
    // Two cycles share edge 0->1 (capacity 10): 0->1->0 (min 3) and 0->1->2->0 (min 7).
    // Max circulation uses both fully: 2*3 + 3*7 = 27. Greedy netting can never exceed it.
    let g = FlowGraph::from_triples(3, &[(0, 1, 10), (1, 0, 3), (1, 2, 7), (2, 0, 7)]);
    let c = cdfd_max(&g);
    let nt = cycle_netting(&g, false);
    assert!(c.circular_total >= nt.cancelled_total);
    assert_eq!(c.circular_total, 2 * 3 + 3 * 7);
}

fn lp_max_circulation(g: &FlowGraph) -> f64 {
    use minilp::{ComparisonOp, OptimizationDirection, Problem};
    let mut p = Problem::new(OptimizationDirection::Maximize);
    let vars: Vec<_> = g
        .edges
        .iter()
        .map(|e| p.add_var(1.0, (0.0, e.flow as f64)))
        .collect();
    for v in 0..g.n as u32 {
        let mut row = Vec::new();
        for (k, e) in g.edges.iter().enumerate() {
            if e.from == v {
                row.push((vars[k], 1.0));
            }
            if e.to == v {
                row.push((vars[k], -1.0));
            }
        }
        if !row.is_empty() {
            p.add_constraint(&row[..], ComparisonOp::Eq, 0.0);
        }
    }
    p.solve().map(|s| s.objective()).unwrap_or(0.0)
}

#[test]
fn cdfd_matches_lp_oracle_on_1000_random_graphs() {
    let mut rng = Rng::new(20_250_612);
    for case in 0..1000 {
        let n = 2 + rng.below(7) as u32; // 2..8 nodes
        let m = rng.below(20) as usize;
        let t: Vec<(u32, u32, i64)> = (0..m)
            .map(|_| {
                let a = rng.below(n as u64) as u32;
                let b = (a + 1 + rng.below(n as u64 - 1) as u32) % n;
                (a, b, 1 + rng.below(1000) as i64)
            })
            .collect();
        let g = FlowGraph::from_triples(n as usize, &t);
        let c = cdfd_max(&g);
        let lp = lp_max_circulation(&g);
        assert!(
            (c.circular_total as f64 - lp).abs() < 1e-6 * (1.0 + lp),
            "case {case}: cdfd {} lp {lp} graph {g:?}",
            c.circular_total
        );
        assert!(
            g.is_acyclic_on(&c.directional),
            "case {case}: directional part has a cycle"
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// Invariants 2 and 5 for cycle netting.
    #[test]
    fn netting_invariants(g in random_graph(7, 30)) {
        let nt = cycle_netting(&g, true);
        // 2: net positions unchanged.
        prop_assert_eq!(g.net_positions_of(&nt.residual), g.net_positions());
        // 5: no direction flip / no over-cancellation per edge.
        for (e, (&r, &c)) in g.edges.iter().zip(nt.residual.iter().zip(&nt.cancelled)) {
            prop_assert!(r >= 0 && c >= 0 && r + c == e.flow);
        }
        // 5: each cycle cancels exactly the minimum of its current flows (replayed).
        let mut cur: Vec<i64> = g.edges.iter().map(|e| e.flow).collect();
        for (cyc, amt) in nt.log.as_ref().unwrap() {
            let min = cyc.iter().map(|&k| cur[k as usize]).min().unwrap();
            prop_assert_eq!(*amt, min);
            // it is a real directed cycle
            for w in 0..cyc.len() {
                let (a, b) = (&g.edges[cyc[w] as usize], &g.edges[cyc[(w + 1) % cyc.len()] as usize]);
                prop_assert_eq!(a.to, b.from);
            }
            for &k in cyc { cur[k as usize] -= amt; }
        }
        prop_assert_eq!(&cur, &nt.residual);
        // Greedy netting ends acyclic and never beats the CDFD optimum.
        prop_assert!(g.is_acyclic_on(&nt.residual));
        let c = cdfd_max(&g);
        prop_assert!(c.circular_total >= nt.cancelled_total);
        // CDFD also preserves net positions (2) and respects 0 <= c <= F.
        prop_assert_eq!(g.net_positions_of(&c.directional), g.net_positions());
        for (e, &x) in g.edges.iter().zip(&c.circular) { prop_assert!(x >= 0 && x <= e.flow); }
    }

    /// Invariant 6: gradient + curl + harmonic = F; curl and harmonic are divergence-free;
    /// the three parts are orthogonal.
    #[test]
    fn hodge_invariants(g in random_graph(8, 40)) {
        let h = hodge(&g, &HodgeConfig { tol: 1e-12, ..Default::default() });
        let scale = 1.0 + h.flow.iter().fold(0.0f64, |a, x| a.max(x.abs()));
        for k in 0..h.flow.len() {
            let s = h.gradient[k] + h.curl[k] + h.harmonic[k];
            prop_assert!((s - h.flow[k]).abs() < 1e-9 * scale);
        }
        for d in h.divergence(&h.curl).iter().chain(h.divergence(&h.harmonic).iter()) {
            prop_assert!(d.abs() < 1e-7 * scale, "divergence {}", d);
        }
        let dot = |a: &[f64], b: &[f64]| a.iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
        let tol = 1e-7 * scale * scale * (1 + h.flow.len()) as f64;
        prop_assert!(dot(&h.gradient, &h.curl).abs() < tol);
        prop_assert!(dot(&h.gradient, &h.harmonic).abs() < tol);
        prop_assert!(dot(&h.curl, &h.harmonic).abs() < tol);
    }
}
