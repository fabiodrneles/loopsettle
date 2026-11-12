//! Maximum circular component of a flow network, in the sense of the Circular Directional
//! Flow Decomposition (CDFD, arXiv 2506.12546): find a circulation `c` with
//! `0 <= c_e <= F_e` and conservation at every node that maximises `sum_e c_e`; the rest
//! `d = F - c` is the directional component.
//!
//! **Reformulation used here.** `d` has the same divergence as `F` (because `c` is
//! divergence-free) and `0 <= d <= F`, so maximising `sum c` = minimising `sum d`:
//! a min-cost transshipment with unit cost per edge, capacities `F_e`, supplies = net
//! outflow of each node. We solve it exactly on integer cents with a primal-dual method: a
//! Dijkstra with Johnson potentials per phase, then a blocking flow on the zero-reduced-cost
//! arcs (1 million edges in about 8 s and 120 MiB; `examples/bench_cdfd_scale.rs`). An optimal `d` is acyclic: a positive cycle in
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
    /// primal-dual phases (one Dijkstra each)
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

    /// Successive shortest paths, one Dijkstra per augmenting path (the original method; kept as
    /// the reference for tests). Returns (flow, augmentations).
    #[cfg(test)]
    fn run_ssp(&mut self, s: usize, t: usize, want: i128) -> (i128, usize) {
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

    /// Primal-dual min-cost flow: one Dijkstra with Johnson potentials fixes the distances, then
    /// a blocking flow (Dinic) is pushed through all arcs of zero reduced cost at once. Every
    /// path pushed in a phase has the current shortest cost, so the result is a min-cost flow as
    /// with successive shortest paths, with one Dijkstra per *distance* instead of one per path.
    /// Potentials use `min(dist, dist[t])`, which keeps every residual reduced cost >= 0.
    /// Returns (flow, phases).
    fn run(&mut self, s: usize, t: usize, want: i128) -> (i128, usize) {
        let n = self.g.len();
        let mut pot = vec![0i64; n];
        let mut sent: i128 = 0;
        let mut phases = 0;
        let mut dist = vec![i64::MAX; n];
        let mut level = vec![u32::MAX; n];
        let mut cur = vec![0usize; n];
        let mut queue: Vec<u32> = Vec::with_capacity(n);
        while sent < want {
            // 1. shortest distances in reduced costs
            dist.iter_mut().for_each(|d| *d = i64::MAX);
            dist[s] = 0;
            let mut heap = BinaryHeap::new();
            heap.push(Reverse((0i64, s as u32)));
            while let Some(Reverse((d, v))) = heap.pop() {
                let v = v as usize;
                if d > dist[v] {
                    continue;
                }
                for a in &self.g[v] {
                    if a.cap > 0 {
                        let w = a.to as usize;
                        let nd = d + a.cost + pot[v] - pot[w];
                        if nd < dist[w] {
                            dist[w] = nd;
                            heap.push(Reverse((nd, w as u32)));
                        }
                    }
                }
            }
            if dist[t] == i64::MAX {
                break;
            }
            let dt = dist[t];
            for v in 0..n {
                pot[v] += dist[v].min(dt);
            }
            phases += 1;
            // 2. blocking flows on the admissible arcs (residual, reduced cost 0) until t is
            // no longer reachable through them
            let admissible = |a: &Arc, pot: &[i64], v: usize| a.cap > 0 && a.cost + pot[v] - pot[a.to as usize] == 0;
            loop {
                level.iter_mut().for_each(|l| *l = u32::MAX);
                level[s] = 0;
                queue.clear();
                queue.push(s as u32);
                let mut qi = 0;
                while qi < queue.len() {
                    let v = queue[qi] as usize;
                    qi += 1;
                    for a in &self.g[v] {
                        let w = a.to as usize;
                        if level[w] == u32::MAX && admissible(a, &pot, v) {
                            level[w] = level[v] + 1;
                            queue.push(w as u32);
                        }
                    }
                }
                if level[t] == u32::MAX || sent >= want {
                    break;
                }
                cur.iter_mut().for_each(|c| *c = 0);
                // iterative DFS with current-arc pointers; `path` holds (node, arc index)
                let mut path: Vec<(usize, usize)> = Vec::new();
                let mut v = s;
                while sent < want {
                    if v == t {
                        let mut f = (want - sent).min(Cents::MAX as i128) as Cents;
                        for &(u, i) in &path {
                            f = f.min(self.g[u][i].cap);
                        }
                        for &(u, i) in &path {
                            let a = &mut self.g[u][i];
                            a.cap -= f;
                            let (to, rev) = (a.to as usize, a.rev as usize);
                            self.g[to][rev].cap += f;
                        }
                        sent += f as i128;
                        // back up to the first saturated arc
                        let k = path.iter().position(|&(u, i)| self.g[u][i].cap == 0).unwrap_or(0);
                        path.truncate(k);
                        v = path.last().map(|&(u, i)| self.g[u][i].to as usize).unwrap_or(s);
                        continue;
                    }
                    let mut advanced = false;
                    while cur[v] < self.g[v].len() {
                        let a = &self.g[v][cur[v]];
                        let w = a.to as usize;
                        if level[w] == level[v] + 1 && admissible(a, &pot, v) {
                            path.push((v, cur[v]));
                            v = w;
                            advanced = true;
                            break;
                        }
                        cur[v] += 1;
                    }
                    if !advanced {
                        // dead end: retreat and skip the arc that led here
                        level[v] = u32::MAX;
                        match path.pop() {
                            Some((u, _)) => {
                                cur[u] += 1;
                                v = u;
                            }
                            None => break,
                        }
                    }
                }
            }
        }
        (sent, phases)
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

#[cfg(test)]
mod tests {
    use super::*;
    use loopsettle_core::rng::Rng;

    /// The primal-dual solver and the original successive-shortest-paths solver reach the same
    /// optimum (same circular total) on random graphs. Per-edge splits may differ when the
    /// optimum is not unique; the total is unique.
    #[test]
    fn primal_dual_equals_successive_shortest_paths() {
        let mut rng = Rng::new(808);
        for _ in 0..400 {
            let n = 2 + rng.below(12) as usize;
            let m = rng.below(40) as usize;
            let t: Vec<(u32, u32, i64)> = (0..m).map(|_| (rng.below(n as u64) as u32, rng.below(n as u64) as u32, 1 + rng.below(1_000) as i64)).collect();
            let g = FlowGraph::from_triples(n, &t);
            let fast = cdfd_max(&g);
            // reference
            let (s, tt) = (n, n + 1);
            let mut mcf = Mcf { g: (0..n + 2).map(|_| Vec::new()).collect() };
            for e in &g.edges {
                mcf.add(e.from as usize, e.to as usize, e.flow, 1);
            }
            let mut supply = 0i128;
            for (v, &x) in g.net_positions().iter().enumerate() {
                if x < 0 {
                    mcf.add(s, v, (-x) as Cents, 0);
                    supply += -x;
                } else if x > 0 {
                    mcf.add(v, tt, x as Cents, 0);
                }
            }
            let (sent, _) = mcf.run_ssp(s, tt, supply);
            assert_eq!(sent, supply);
            let cost: i128 = g.edges.iter().map(|e| e.flow as i128).sum::<i128>() - fast.circular_total;
            // the reference's cost = flow on original edges = directional total
            let ref_dir: i128 = (0..n).flat_map(|v| mcf.g[v].iter().filter(|a| a.cost == -1).map(|a| a.cap as i128).collect::<Vec<_>>()).sum();
            assert_eq!(cost, ref_dir, "same minimum directional total");
            assert_eq!(fast.directional_total, ref_dir);
        }
    }
}
