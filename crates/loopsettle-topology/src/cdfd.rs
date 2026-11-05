//! Maximum circular component of a flow network, in the sense of the Circular Directional
//! Flow Decomposition (CDFD, arXiv 2506.12546): find a circulation `c` with
//! `0 <= c_e <= F_e` and conservation at every node that maximises `sum_e c_e`; the rest
//! `d = F - c` is the directional component.
//!
//! **Reformulation used here.** `d` has the same divergence as `F` (because `c` is
//! divergence-free) and `0 <= d <= F`, so maximising `sum c` = minimising `sum d`:
//! a min-cost transshipment with unit cost per edge, capacities `F_e`, supplies = net
//! outflow of each node. We solve it exactly with successive shortest paths (Dijkstra with
//! Johnson potentials) on integer cents. An optimal `d` is acyclic: a positive cycle in
//! `d` could be moved to `c`, lowering the cost, contradicting optimality.
//!
//! The proportional / locally computable CDFD variant is not implemented (DECISIONS.md).

use crate::graph::FlowGraph;
use loopsettle_core::Cents;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Clone, Debug)]
pub struct Cdfd {
    pub circular: Vec<Cents>,
    pub directional: Vec<Cents>,
    pub circular_total: i128,
    pub directional_total: i128,
    pub augmentations: usize,
}

struct Arc {
    to: u32,
    cap: Cents,
    cost: i64,
    rev: u32,
}

struct Mcf {
    g: Vec<Vec<Arc>>,
}

impl Mcf {
    /// Adds arc `a -> b` and its residual twin; returns the twin's index in `g[b]`.
    fn add(&mut self, a: usize, b: usize, cap: Cents, cost: i64) -> usize {
        let (ra, rb) = (self.g[b].len() as u32, self.g[a].len() as u32);
        self.g[a].push(Arc { to: b as u32, cap, cost, rev: ra });
        self.g[b].push(Arc { to: a as u32, cap: 0, cost: -cost, rev: rb });
        ra as usize
    }

    /// Min-cost flow from `s` to `t` of value up to `want`. Returns (flow, augmentations).
    fn run(&mut self, s: usize, t: usize, want: i128) -> (i128, usize) {
        let n = self.g.len();
        let mut pot = vec![0i64; n]; // all initial costs >= 0, so zero potentials are valid
        let mut sent: i128 = 0;
        let mut aug = 0;
        let mut dist = vec![i64::MAX; n];
        let mut prev: Vec<(u32, u32)> = vec![(u32::MAX, 0); n];
        while sent < want {
            dist.iter_mut().for_each(|d| *d = i64::MAX);
            dist[s] = 0;
            let mut heap = BinaryHeap::new();
            heap.push(Reverse((0i64, s as u32)));
            while let Some(Reverse((d, v))) = heap.pop() {
                let v = v as usize;
                if d > dist[v] {
                    continue;
                }
                for (i, a) in self.g[v].iter().enumerate() {
                    if a.cap > 0 {
                        let w = a.to as usize;
                        let nd = d + a.cost + pot[v] - pot[w];
                        if nd < dist[w] {
                            dist[w] = nd;
                            prev[w] = (v as u32, i as u32);
                            heap.push(Reverse((nd, w as u32)));
                        }
                    }
                }
            }
            if dist[t] == i64::MAX {
                break;
            }
            for v in 0..n {
                if dist[v] != i64::MAX {
                    pot[v] += dist[v];
                }
            }
            // Bottleneck along the path.
            let mut f = (want - sent).min(Cents::MAX as i128) as Cents;
            let mut v = t;
            while v != s {
                let (u, i) = prev[v];
                f = f.min(self.g[u as usize][i as usize].cap);
                v = u as usize;
            }
            let mut v = t;
            while v != s {
                let (u, i) = prev[v];
                let a = &mut self.g[u as usize][i as usize];
                a.cap -= f;
                let (to, rev) = (a.to as usize, a.rev as usize);
                self.g[to][rev].cap += f;
                v = u as usize;
            }
            sent += f as i128;
            aug += 1;
        }
        (sent, aug)
    }
}

pub fn cdfd_max(g: &FlowGraph) -> Cdfd {
    let n = g.n;
    let (s, t) = (n, n + 1);
    let mut m = Mcf { g: (0..n + 2).map(|_| Vec::new()).collect() };
    let handles: Vec<usize> = g.edges.iter().map(|e| m.add(e.from as usize, e.to as usize, e.flow, 1)).collect();
    let net = g.net_positions(); // in - out
    let mut supply: i128 = 0;
    for (v, &x) in net.iter().enumerate() {
        if x < 0 {
            m.add(s, v, (-x) as Cents, 0);
            supply += -x;
        } else if x > 0 {
            m.add(v, t, x as Cents, 0);
        }
    }
    let (sent, augmentations) = m.run(s, t, supply);
    assert_eq!(sent, supply, "F itself is feasible, so the full supply must be routable");
    // Flow on edge e = residual capacity of its reverse arc (stored in node `e.to`).
    let directional: Vec<Cents> = g.edges.iter().zip(&handles).map(|(e, &rev)| m.g[e.to as usize][rev].cap).collect();
    let circular: Vec<Cents> = g.edges.iter().zip(&directional).map(|(e, d)| e.flow - d).collect();
    let circular_total = circular.iter().map(|&c| c as i128).sum();
    let directional_total = directional.iter().map(|&c| c as i128).sum();
    Cdfd { circular, directional, circular_total, directional_total, augmentations }
}
