//! Aggregated directed flow graph between participants.

use loopsettle_core::{Cents, FlowStats};

/// One directed edge `from -> to` carrying a positive aggregated flow in cents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    pub from: u32,
    pub to: u32,
    pub flow: Cents,
}

/// Directed graph with positive integer edge flows. Reciprocal edges (`i->j` and `j->i`)
/// are allowed in a *gross* graph; a *net* graph has at most one edge per unordered pair.
#[derive(Clone, Debug, Default)]
pub struct FlowGraph {
    pub n: usize,
    pub edges: Vec<Edge>,
}

impl FlowGraph {
    pub fn new(n: usize, edges: Vec<Edge>) -> Self {
        for e in &edges {
            assert!(
                e.flow > 0 && e.from != e.to && (e.from as usize) < n && (e.to as usize) < n,
                "bad edge {e:?}"
            );
        }
        FlowGraph { n, edges }
    }

    /// Convenience constructor from `(from, to, flow)` triples, merging duplicates.
    pub fn from_triples(n: usize, t: &[(u32, u32, Cents)]) -> Self {
        let mut m = std::collections::BTreeMap::new();
        for &(a, b, v) in t {
            if v > 0 && a != b {
                *m.entry((a, b)).or_insert(0) += v;
            }
        }
        Self::new(
            n,
            m.into_iter()
                .map(|((from, to), flow)| Edge { from, to, flow })
                .collect(),
        )
    }

    /// Gross graph: one edge per ordered pair with positive aggregated flow.
    pub fn gross_from(flow: &FlowStats) -> Self {
        assert!(flow.has_pairs(), "FlowStats must be built with pairs");
        let n = flow.n;
        let mut edges = Vec::new();
        for a in 0..n {
            for b in 0..n {
                let v = flow.pair(a, b);
                if a != b && v > 0 {
                    edges.push(Edge {
                        from: a as u32,
                        to: b as u32,
                        flow: v,
                    });
                }
            }
        }
        FlowGraph { n, edges }
    }

    /// Net graph: bilateral netting of every pair, oriented from the net payer.
    pub fn net(&self) -> Self {
        let mut m = std::collections::BTreeMap::new();
        for e in &self.edges {
            let (key, sign) = if e.from < e.to {
                ((e.from, e.to), 1)
            } else {
                ((e.to, e.from), -1)
            };
            *m.entry(key).or_insert(0i64) += sign * e.flow;
        }
        let edges = m
            .into_iter()
            .filter(|&(_, v)| v != 0)
            .map(|((a, b), v)| {
                if v > 0 {
                    Edge {
                        from: a,
                        to: b,
                        flow: v,
                    }
                } else {
                    Edge {
                        from: b,
                        to: a,
                        flow: -v,
                    }
                }
            })
            .collect();
        FlowGraph { n: self.n, edges }
    }

    pub fn total_flow(&self) -> i128 {
        self.edges.iter().map(|e| e.flow as i128).sum()
    }

    /// Net position (in - out) of every node for flows `w` on `self.edges`.
    pub fn net_positions_of(&self, w: &[Cents]) -> Vec<i128> {
        let mut d = vec![0i128; self.n];
        for (e, &x) in self.edges.iter().zip(w) {
            d[e.from as usize] -= x as i128;
            d[e.to as usize] += x as i128;
        }
        d
    }
    pub fn net_positions(&self) -> Vec<i128> {
        let w: Vec<Cents> = self.edges.iter().map(|e| e.flow).collect();
        self.net_positions_of(&w)
    }

    /// `true` if the sub-graph of edges with `w[e] > 0` has no directed cycle.
    pub fn is_acyclic_on(&self, w: &[Cents]) -> bool {
        let n = self.n;
        let mut indeg = vec![0u32; n];
        let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
        for (e, &x) in self.edges.iter().zip(w) {
            if x > 0 {
                adj[e.from as usize].push(e.to);
                indeg[e.to as usize] += 1;
            }
        }
        // Kahn's algorithm: acyclic iff every node gets removed.
        let mut stack: Vec<usize> = (0..n).filter(|&v| indeg[v] == 0).collect();
        let mut seen = 0;
        while let Some(v) = stack.pop() {
            seen += 1;
            for &u in &adj[v] {
                indeg[u as usize] -= 1;
                if indeg[u as usize] == 0 {
                    stack.push(u as usize);
                }
            }
        }
        seen == n
    }
}
