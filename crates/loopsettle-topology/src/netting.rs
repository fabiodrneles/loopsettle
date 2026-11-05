//! Cycle netting in the spirit of Bech & Soramäki (2001): find a directed cycle of positive
//! flows, cancel the smallest flow on it from every edge of the cycle, repeat until the
//! remaining graph is acyclic.
//!
//! Implementation: a single depth-first search with a "current arc" per node, the classic
//! flow-decomposition technique. When the search closes a cycle it cancels it and backs up
//! to the tail of the first edge that became zero, so the total work is O(n * m) instead
//! of restarting the search after every cycle. The result is deterministic (edges are
//! explored in input order) but depends on that order: greedy cycle cancelling is *not*
//! guaranteed to reach the maximum circulation (that is what [`crate::cdfd`] computes).

use crate::graph::FlowGraph;
use loopsettle_core::Cents;

#[derive(Clone, Debug)]
pub struct NettingResult {
    /// Flow left on each edge after netting (>= 0 and <= original: direction never flips).
    pub residual: Vec<Cents>,
    /// Amount cancelled on each edge.
    pub cancelled: Vec<Cents>,
    /// Sum of `cancelled` over edges.
    pub cancelled_total: i128,
    pub cycles: usize,
    /// If requested: each cycle as (edge indices, amount cancelled on each of its edges).
    pub log: Option<Vec<(Vec<u32>, Cents)>>,
}

pub fn cycle_netting(g: &FlowGraph, record: bool) -> NettingResult {
    let n = g.n;
    let mut res: Vec<Cents> = g.edges.iter().map(|e| e.flow).collect();
    // Out-adjacency in CSR form (edge ids per source node, in input order).
    let mut start = vec![0usize; n + 1];
    for e in &g.edges {
        start[e.from as usize + 1] += 1;
    }
    for i in 0..n {
        start[i + 1] += start[i];
    }
    let mut fill = start.clone();
    let mut adj = vec![0u32; g.edges.len()];
    for (i, e) in g.edges.iter().enumerate() {
        adj[fill[e.from as usize]] = i as u32;
        fill[e.from as usize] += 1;
    }

    const WHITE: u8 = 0;
    const GRAY: u8 = 1;
    const BLACK: u8 = 2;
    let mut color = vec![WHITE; n];
    let mut ptr = start.clone(); // current arc of each node
    let mut pos = vec![usize::MAX; n]; // position of a gray node in `path`
    let mut path: Vec<u32> = Vec::new(); // nodes on the DFS path
    let mut path_e: Vec<u32> = Vec::new(); // edge from path[k] to path[k+1]
    let mut cycles = 0usize;
    let mut log = if record { Some(Vec::new()) } else { None };

    for root in 0..n as u32 {
        if color[root as usize] != WHITE {
            continue;
        }
        color[root as usize] = GRAY;
        pos[root as usize] = 0;
        path.push(root);
        while let Some(&v) = path.last() {
            let vi = v as usize;
            // Skip exhausted edges and edges into finished nodes.
            while ptr[vi] < start[vi + 1] {
                let e = adj[ptr[vi]] as usize;
                if res[e] > 0 && color[g.edges[e].to as usize] != BLACK {
                    break;
                }
                ptr[vi] += 1;
            }
            if ptr[vi] == start[vi + 1] {
                color[vi] = BLACK;
                path.pop();
                path_e.pop();
                continue;
            }
            let e = adj[ptr[vi]];
            let w = g.edges[e as usize].to;
            if color[w as usize] == WHITE {
                color[w as usize] = GRAY;
                pos[w as usize] = path.len();
                path.push(w);
                path_e.push(e);
                continue;
            }
            // `w` is gray: cycle path[pos[w]..] + (v -> w).
            let k0 = pos[w as usize];
            let mut cyc: Vec<u32> = path_e[k0..].to_vec();
            cyc.push(e);
            let amount = cyc.iter().map(|&x| res[x as usize]).min().unwrap();
            for &x in &cyc {
                res[x as usize] -= amount;
            }
            cycles += 1;
            // Back up to the tail of the first edge (in path order) that became zero.
            let first_zero = cyc.iter().position(|&x| res[x as usize] == 0).unwrap();
            let keep = k0 + first_zero + 1; // nodes path[..keep] stay on the path
            for &u in &path[keep..] {
                color[u as usize] = WHITE;
                pos[u as usize] = usize::MAX;
            }
            path.truncate(keep);
            path_e.truncate(keep - 1);
            if let Some(l) = log.as_mut() {
                l.push((cyc, amount));
            }
        }
    }

    let cancelled: Vec<Cents> = g.edges.iter().zip(&res).map(|(e, r)| e.flow - r).collect();
    let cancelled_total = cancelled.iter().map(|&c| c as i128).sum();
    NettingResult { residual: res, cancelled, cancelled_total, cycles, log }
}
