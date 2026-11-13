//! Combinatorial Hodge decomposition of an edge flow on the 2-complex of the graph
//! (nodes, edges, triangles):
//!
//! `f = gradient + curl + harmonic`, where
//! * `gradient = B1^T phi`, with `L0 phi = B1 f` (`L0 = B1 B1^T`, the graph Laplacian);
//! * `curl = B2 psi`, with `(B2^T B2) psi = B2^T (f - gradient)`;
//! * `harmonic = f - gradient - curl` (cycles not spanned by the triangles kept).
//!
//! `B1` is the node-edge incidence matrix (column of edge `u->v` has -1 at `u`, +1 at `v`)
//! and `B2` the edge-triangle incidence matrix. Both are applied matrix-free.
//! Both linear systems are solved with hand-written CG (see `linalg`).

use crate::graph::FlowGraph;
use crate::linalg::{cg, norm1, norm2, CgReport};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct HodgeConfig {
    /// Maximum number of triangles in `B2`; beyond it, remaining cycles end up in `harmonic`.
    pub max_triangles: usize,
    pub tol: f64,
    pub max_iter: usize,
}

impl Default for HodgeConfig {
    fn default() -> Self {
        HodgeConfig {
            max_triangles: 500_000,
            tol: 1e-10,
            max_iter: 20_000,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Hodge {
    /// The (net) graph that was decomposed; component vectors are indexed by its edges.
    pub graph: FlowGraph,
    pub flow: Vec<f64>,
    pub potential: Vec<f64>,
    pub gradient: Vec<f64>,
    pub curl: Vec<f64>,
    pub harmonic: Vec<f64>,
    pub triangles: usize,
    pub triangles_capped: bool,
    pub cg_gradient: CgReport,
    pub cg_curl: CgReport,
}

/// Triangle as three `(edge index, sign)` pairs: the boundary of the oriented triangle.
type Tri = [(u32, f64); 3];

fn triangles(g: &FlowGraph, cap: usize) -> (Vec<Tri>, bool) {
    // Undirected neighbour lists restricted to higher-numbered neighbours.
    let mut up: Vec<Vec<u32>> = vec![Vec::new(); g.n];
    let mut idx: HashMap<(u32, u32), (u32, f64)> = HashMap::with_capacity(g.edges.len());
    for (i, e) in g.edges.iter().enumerate() {
        let (a, b, s) = if e.from < e.to {
            (e.from, e.to, 1.0)
        } else {
            (e.to, e.from, -1.0)
        };
        idx.insert((a, b), (i as u32, s));
        up[a as usize].push(b);
    }
    for l in up.iter_mut() {
        l.sort_unstable();
    }
    let mut out = Vec::new();
    for a in 0..g.n as u32 {
        for (k, &b) in up[a as usize].iter().enumerate() {
            // common higher neighbours c > b of a and b (sorted-list intersection)
            let (la, lb) = (&up[a as usize][k + 1..], &up[b as usize]);
            let (mut i, mut j) = (0, 0);
            while i < la.len() && j < lb.len() {
                match la[i].cmp(&lb[j]) {
                    std::cmp::Ordering::Less => i += 1,
                    std::cmp::Ordering::Greater => j += 1,
                    std::cmp::Ordering::Equal => {
                        let c = la[i];
                        if out.len() == cap {
                            return (out, true);
                        }
                        // Oriented triangle a -> b -> c -> a.
                        let (eab, sab) = idx[&(a, b)];
                        let (ebc, sbc) = idx[&(b, c)];
                        let (eac, sac) = idx[&(a, c)];
                        out.push([(eab, sab), (ebc, sbc), (eac, -sac)]);
                        i += 1;
                        j += 1;
                    }
                }
            }
        }
    }
    (out, false)
}

/// Decompose the **net** flow of `g` (reciprocal edges are netted first).
pub fn hodge(g: &FlowGraph, cfg: &HodgeConfig) -> Hodge {
    let g = g.net();
    let (n, m) = (g.n, g.edges.len());
    let f: Vec<f64> = g.edges.iter().map(|e| e.flow as f64).collect();
    let ends: Vec<(usize, usize)> = g
        .edges
        .iter()
        .map(|e| (e.from as usize, e.to as usize))
        .collect();

    // Gradient: L0 phi = B1 f.
    let mut rhs = vec![0.0; n];
    for (k, &(u, v)) in ends.iter().enumerate() {
        rhs[u] -= f[k];
        rhs[v] += f[k];
    }
    let mut phi = vec![0.0; n];
    let cg_gradient = cg(
        |x, y| {
            y.iter_mut().for_each(|v| *v = 0.0);
            for &(u, v) in &ends {
                let d = x[v] - x[u];
                y[v] += d;
                y[u] -= d;
            }
        },
        &rhs,
        &mut phi,
        cfg.tol,
        cfg.max_iter,
    );
    let gradient: Vec<f64> = ends.iter().map(|&(u, v)| phi[v] - phi[u]).collect();
    let rest: Vec<f64> = f.iter().zip(&gradient).map(|(a, b)| a - b).collect();

    // Curl: (B2^T B2) psi = B2^T rest.
    let (tris, capped) = triangles(&g, cfg.max_triangles);
    let b2t = |x: &[f64], y: &mut [f64]| {
        for (t, tri) in tris.iter().enumerate() {
            y[t] = tri.iter().map(|&(e, s)| s * x[e as usize]).sum();
        }
    };
    let b2 = |x: &[f64], y: &mut [f64]| {
        y.iter_mut().for_each(|v| *v = 0.0);
        for (t, tri) in tris.iter().enumerate() {
            for &(e, s) in tri {
                y[e as usize] += s * x[t];
            }
        }
    };
    let mut rhs2 = vec![0.0; tris.len()];
    b2t(&rest, &mut rhs2);
    let mut psi = vec![0.0; tris.len()];
    let mut tmp = vec![0.0; m];
    let cg_curl = cg(
        |x, y| {
            b2(x, &mut tmp);
            b2t(&tmp, y);
        },
        &rhs2,
        &mut psi,
        cfg.tol,
        cfg.max_iter,
    );
    let mut curl = vec![0.0; m];
    b2(&psi, &mut curl);
    let harmonic: Vec<f64> = rest.iter().zip(&curl).map(|(a, b)| a - b).collect();

    Hodge {
        triangles: tris.len(),
        triangles_capped: capped,
        graph: g,
        flow: f,
        potential: phi,
        gradient,
        curl,
        harmonic,
        cg_gradient,
        cg_curl,
    }
}

impl Hodge {
    /// Divergence `B1 x` of an edge vector.
    pub fn divergence(&self, x: &[f64]) -> Vec<f64> {
        let mut d = vec![0.0; self.graph.n];
        for (e, &v) in self.graph.edges.iter().zip(x) {
            d[e.from as usize] -= v;
            d[e.to as usize] += v;
        }
        d
    }
    /// Cyclic part = curl + harmonic (everything that is divergence-free).
    pub fn cyclic(&self) -> Vec<f64> {
        self.curl
            .iter()
            .zip(&self.harmonic)
            .map(|(a, b)| a + b)
            .collect()
    }
    /// The spec's "circular fraction": ||curl|| / ||f|| (L2).
    pub fn curl_fraction_l2(&self) -> f64 {
        ratio(norm2(&self.curl), norm2(&self.flow))
    }
    /// ||curl + harmonic|| / ||f|| (L2).
    pub fn cyclic_fraction_l2(&self) -> f64 {
        ratio(norm2(&self.cyclic()), norm2(&self.flow))
    }
    /// sum |curl + harmonic| / sum f (L1): same units as money, comparable with CDFD/cycles.
    pub fn cyclic_fraction_l1(&self) -> f64 {
        ratio(norm1(&self.cyclic()), norm1(&self.flow))
    }
    /// Squared-energy shares (gradient, curl, harmonic); they sum to 1 by orthogonality.
    pub fn energy_shares(&self) -> (f64, f64, f64) {
        let t = crate::linalg::dot(&self.flow, &self.flow);
        let s = |x: &[f64]| ratio(crate::linalg::dot(x, x), t);
        (s(&self.gradient), s(&self.curl), s(&self.harmonic))
    }
}

pub(crate) fn ratio(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        0.0
    } else {
        a / b
    }
}
