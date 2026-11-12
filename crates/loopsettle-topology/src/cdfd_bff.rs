//! Balanced Flow Forwarding (BFF): the proportional, locally computable circular-directional
//! decomposition of Homs-Dones, MacKay, Sansom & Zhou (arXiv 2506.12546, section 5.2,
//! Algorithms 1 and 2). Implemented here in its non-local form (Algorithm 2), which the paper
//! states gives the same decomposition as the local dynamics:
//!
//! ```text
//! BFF(w): c = 0
//!   for each strongly connected component C of w (with at least one edge):
//!     P = w_C with each row divided by the node's out-weight inside C   (proportions)
//!     pi = stationary distribution of P            (pi P = pi, sum pi = 1)
//!     x  = pi * min_i (out_i / pi_i)               (the largest invariant vector <= out)
//!     c~_ij = x_i * P_ij                           (balanced: in = out at every node)
//!     c = c + c~ + BFF(w_C - c~)
//! ```
//!
//! Each round saturates at least one node's out-flow inside its component, which then stops
//! being part of any cycle, so there are at most `n` rounds. Unlike the maximum circulation
//! ([`crate::cdfd`]) the circular part keeps the flow proportions of each node; the paper reports
//! that its circularity is never above the maximum (an observation there, checked here on random
//! graphs). The result is real-valued (`f64`): it is a diagnostic, not a settlement amount.

use crate::graph::FlowGraph;

#[derive(Clone, Debug)]
pub struct CdfdBff {
    pub circular: Vec<f64>,
    pub directional: Vec<f64>,
    pub circular_total: f64,
    /// number of Maximal_Invariant computations (component rounds)
    pub rounds: usize,
}

/// Stationary distribution of an irreducible row-stochastic matrix `p` (k x k, row-major), by
/// Gaussian elimination with partial pivoting on `pi (P - I) = 0`, `sum pi = 1`.
fn stationary(p: &[f64], k: usize) -> Vec<f64> {
    if k == 1 {
        return vec![1.0];
    }
    // A = (P - I)^T, last row replaced by ones; solve A pi = e_k
    let mut a = vec![0.0; k * k];
    let mut b = vec![0.0; k];
    for i in 0..k {
        for j in 0..k {
            a[j * k + i] = p[i * k + j] - if i == j { 1.0 } else { 0.0 };
        }
    }
    for j in 0..k {
        a[(k - 1) * k + j] = 1.0;
    }
    b[k - 1] = 1.0;
    for col in 0..k {
        let piv = (col..k).max_by(|&x, &y| a[x * k + col].abs().partial_cmp(&a[y * k + col].abs()).unwrap()).unwrap();
        if piv != col {
            for j in 0..k {
                a.swap(col * k + j, piv * k + j);
            }
            b.swap(col, piv);
        }
        let d = a[col * k + col];
        for r in col + 1..k {
            let f = a[r * k + col] / d;
            if f != 0.0 {
                for j in col..k {
                    a[r * k + j] -= f * a[col * k + j];
                }
                b[r] -= f * b[col];
            }
        }
    }
    let mut x = vec![0.0; k];
    for r in (0..k).rev() {
        let s: f64 = (r + 1..k).map(|j| a[r * k + j] * x[j]).sum();
        x[r] = (b[r] - s) / a[r * k + r];
    }
    // clean tiny negative round-off; irreducible chains have pi > 0
    let tot: f64 = x.iter().map(|v| v.max(0.0)).sum();
    x.iter().map(|v| v.max(0.0) / tot).collect()
}

/// Strongly connected components (iterative Tarjan) of the graph made of edges with
/// `alive[e]`. Returns a component id per node.
fn scc(n: usize, edges: &[(u32, u32)], alive: &[bool]) -> Vec<usize> {
    let mut start = vec![0usize; n + 1];
    for (e, &(a, _)) in edges.iter().enumerate() {
        if alive[e] {
            start[a as usize + 1] += 1;
        }
    }
    for i in 0..n {
        start[i + 1] += start[i];
    }
    let mut adj = vec![0u32; start[n]];
    let mut fill = start.clone();
    for (e, &(a, b)) in edges.iter().enumerate() {
        if alive[e] {
            adj[fill[a as usize]] = b;
            fill[a as usize] += 1;
        }
    }
    const NONE: usize = usize::MAX;
    let (mut index, mut low, mut comp) = (vec![NONE; n], vec![0usize; n], vec![NONE; n]);
    let mut on = vec![false; n];
    let (mut stack, mut next, mut ncomp) = (Vec::new(), 0usize, 0usize);
    let mut call: Vec<(usize, usize)> = Vec::new();
    for root in 0..n {
        if index[root] != NONE {
            continue;
        }
        call.push((root, start[root]));
        index[root] = next;
        low[root] = next;
        next += 1;
        stack.push(root);
        on[root] = true;
        while let Some(&mut (v, ref mut it)) = call.last_mut() {
            if *it < start[v + 1] {
                let w = adj[*it] as usize;
                *it += 1;
                if index[w] == NONE {
                    index[w] = next;
                    low[w] = next;
                    next += 1;
                    stack.push(w);
                    on[w] = true;
                    call.push((w, start[w]));
                } else if on[w] {
                    low[v] = low[v].min(index[w]);
                }
            } else {
                call.pop();
                if let Some(&(u, _)) = call.last() {
                    low[u] = low[u].min(low[v]);
                }
                if low[v] == index[v] {
                    while let Some(w) = stack.pop() {
                        on[w] = false;
                        comp[w] = ncomp;
                        if w == v {
                            break;
                        }
                    }
                    ncomp += 1;
                }
            }
        }
    }
    comp
}

/// BFF decomposition of `g` (any graph; reciprocal edges allowed).
pub fn cdfd_bff(g: &FlowGraph) -> CdfdBff {
    let m = g.edges.len();
    let ends: Vec<(u32, u32)> = g.edges.iter().map(|e| (e.from, e.to)).collect();
    let orig: Vec<f64> = g.edges.iter().map(|e| e.flow as f64).collect();
    let mut rest = orig.clone();
    let mut circ = vec![0.0f64; m];
    let mut rounds = 0;
    loop {
        // an edge is alive while a meaningful part of it is left
        let alive: Vec<bool> = (0..m).map(|e| rest[e] > 1e-9 * orig[e]).collect();
        let comp = scc(g.n, &ends, &alive);
        // group the alive edges that stay inside a component
        let mut by_comp: std::collections::BTreeMap<usize, Vec<usize>> = std::collections::BTreeMap::new();
        for e in 0..m {
            let (a, b) = ends[e];
            if alive[e] && comp[a as usize] == comp[b as usize] {
                by_comp.entry(comp[a as usize]).or_default().push(e);
            }
        }
        if by_comp.is_empty() {
            break;
        }
        for es in by_comp.values() {
            rounds += 1;
            // local indices of the component's nodes
            let mut local: std::collections::BTreeMap<u32, usize> = std::collections::BTreeMap::new();
            for &e in es {
                for v in [ends[e].0, ends[e].1] {
                    let k = local.len();
                    local.entry(v).or_insert(k);
                }
            }
            let k = local.len();
            let mut out = vec![0.0; k];
            for &e in es {
                out[local[&ends[e].0]] += rest[e];
            }
            let mut p = vec![0.0; k * k];
            for &e in es {
                let (i, j) = (local[&ends[e].0], local[&ends[e].1]);
                p[i * k + j] += rest[e] / out[i];
            }
            let pi = stationary(&p, k);
            let scale = (0..k).filter(|&i| pi[i] > 0.0).map(|i| out[i] / pi[i]).fold(f64::INFINITY, f64::min);
            for &e in es {
                let i = local[&ends[e].0];
                let x = pi[i] * scale;
                // the saturating node(s) give exactly their remaining flow
                let take = if out[i] - x <= 1e-12 * out[i] { rest[e] } else { (x * rest[e] / out[i]).min(rest[e]) };
                circ[e] += take;
                rest[e] -= take;
                if rest[e] <= 1e-9 * orig[e] {
                    rest[e] = 0.0;
                }
            }
        }
    }
    let circular_total = circ.iter().sum();
    let directional = orig.iter().zip(&circ).map(|(f, c)| f - c).collect();
    CdfdBff { circular: circ, directional, circular_total, rounds }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cdfd_max;
    use loopsettle_core::rng::Rng;

    fn tri(n: usize, t: &[(u32, u32, i64)]) -> FlowGraph {
        FlowGraph::from_triples(n, t)
    }

    fn balance_ok(g: &FlowGraph, c: &[f64]) -> bool {
        let mut d = vec![0.0; g.n];
        for (e, &x) in g.edges.iter().zip(c) {
            d[e.from as usize] -= x;
            d[e.to as usize] += x;
        }
        let tot: f64 = c.iter().sum::<f64>().max(1.0);
        d.iter().all(|v| v.abs() <= 1e-7 * tot)
    }

    /// One cycle: the whole minimum goes round. A->B 10, B->C 5, C->A 5: 5 on each edge.
    #[test]
    fn triangle_by_hand() {
        let g = tri(3, &[(0, 1, 10), (1, 2, 5), (2, 0, 5)]);
        let b = cdfd_bff(&g);
        let c: Vec<f64> = b.circular.iter().map(|x| (x * 1e6).round() / 1e6).collect();
        assert_eq!(c, vec![5.0, 5.0, 5.0]);
        assert!((cdfd_max(&g).circular_total as f64 - b.circular_total).abs() < 1e-9);
    }

    /// Two cycles through B->A with different proportions; BFF keeps them and gets less than
    /// the maximum. A->B 2, A->C 2, B->A 2, C->B 2 (8 in all).
    /// Proportions: A sends 1/2 to B and 1/2 to C; B and C send everything onwards.
    /// Stationary distribution pi ~ (1, 1, 1/2) for (A, B, C); out = (4, 2, 2); scale =
    /// min(4/1, 2/1, 2/(1/2)) = 2, so x = (2, 2, 1) and
    /// c = A->B 1, A->C 1, B->A 2, C->B 1 (total 5). Then B->A is used up, nothing is left in a
    /// cycle. The maximum circulation is A->C->B->A with 2 on each edge: 6.
    #[test]
    fn proportional_split_by_hand() {
        let g = tri(3, &[(0, 1, 2), (0, 2, 2), (1, 0, 2), (2, 1, 2)]);
        let b = cdfd_bff(&g);
        let get = |a: u32, z: u32| {
            let k = g.edges.iter().position(|e| e.from == a && e.to == z).unwrap();
            (b.circular[k] * 1e9).round() / 1e9
        };
        assert_eq!((get(0, 1), get(0, 2), get(1, 0), get(2, 1)), (1.0, 1.0, 2.0, 1.0));
        assert!((b.circular_total - 5.0).abs() < 1e-9);
        assert_eq!(cdfd_max(&g).circular_total, 6);
    }

    /// On random graphs: c is a circulation, 0 <= c <= F, BFF <= maximum circulation, and the
    /// directional rest has no cycle left.
    #[test]
    fn bff_properties_on_1000_random_graphs() {
        let mut rng = Rng::new(2506);
        for case in 0..1_000 {
            let n = 2 + rng.below(9) as usize;
            let m = rng.below(30) as usize;
            let t: Vec<(u32, u32, i64)> = (0..m).map(|_| (rng.below(n as u64) as u32, rng.below(n as u64) as u32, 1 + rng.below(10_000) as i64)).collect();
            let g = tri(n, &t);
            let b = cdfd_bff(&g);
            assert!(balance_ok(&g, &b.circular), "case {case}: not a circulation");
            for (e, &c) in g.edges.iter().zip(&b.circular) {
                assert!(c >= -1e-9 && c <= e.flow as f64 * (1.0 + 1e-12) + 1e-9, "case {case}: 0 <= c <= F");
            }
            let max = cdfd_max(&g).circular_total as f64;
            assert!(b.circular_total <= max * (1.0 + 1e-9) + 1e-6, "case {case}: BFF {} > max {}", b.circular_total, max);
            // nothing circular is left: the edges with directional flow > 0 form no cycle
            let alive: Vec<bool> = b.directional.iter().zip(&g.edges).map(|(d, e)| *d > 1e-9 * e.flow as f64).collect();
            let ends: Vec<(u32, u32)> = g.edges.iter().map(|e| (e.from, e.to)).collect();
            let comp = scc(g.n, &ends, &alive);
            assert!(ends.iter().zip(&alive).all(|(&(a, z), &al)| !al || comp[a as usize] != comp[z as usize]), "case {case}: a cycle is left in d");
        }
    }
}
