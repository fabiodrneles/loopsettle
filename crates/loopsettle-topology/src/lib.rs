//! # loopsettle-topology
//!
//! Circularity of an aggregated payment network, through three lenses:
//!
//! | lens | module | meaning |
//! |---|---|---|
//! | Hodge (diagnostic) | [`hodge`] | energy of divergence-free flow; may overstate what can be netted |
//! | CDFD maximum (static ceiling) | [`cdfd`] | largest circulation with `0 <= c <= F` |
//! | Cycle netting (executable) | [`netting`] | greedy cancellation of the minimum of each cycle |
//!
//! [`analyze`] computes all three side by side (the "ladder").

pub mod cdfd;
pub mod graph;
pub mod hodge;
pub mod linalg;
pub mod netting;

pub use cdfd::{cdfd_max, Cdfd};
pub use graph::{Edge, FlowGraph};
pub use hodge::{hodge, Hodge, HodgeConfig};
pub use netting::{cycle_netting, NettingResult};

use loopsettle_core::json::Json;

/// The three circularity measures for one graph (all fractions of the graph's total flow).
#[derive(Clone, Debug)]
pub struct Ladder {
    pub nodes: usize,
    pub edges: usize,
    pub total_flow: i128,
    /// L1 Hodge cyclic share: sum |curl + harmonic| / sum F (on the net graph).
    pub hodge_l1: f64,
    /// L2 Hodge curl share ||curl|| / ||F|| (the spec's "fração circular").
    pub hodge_curl_l2: f64,
    /// L2 Hodge cyclic share ||curl + harmonic|| / ||F||.
    pub hodge_cyclic_l2: f64,
    pub hodge_triangles: usize,
    pub hodge_triangles_capped: bool,
    /// CDFD maximum circulation / total flow.
    pub cdfd_max: f64,
    /// Greedy cycle netting / total flow.
    pub cycles: f64,
    pub cycles_found: usize,
}

impl Ladder {
    pub fn to_json(&self) -> Json {
        Json::obj()
            .set("nodes", self.nodes)
            .set("edges", self.edges)
            .set("total_flow_cents", self.total_flow)
            .set("hodge_l1_cyclic", self.hodge_l1)
            .set("hodge_l2_curl", self.hodge_curl_l2)
            .set("hodge_l2_cyclic", self.hodge_cyclic_l2)
            .set("hodge_triangles", self.hodge_triangles)
            .set("hodge_triangles_capped", self.hodge_triangles_capped)
            .set("cdfd_max", self.cdfd_max)
            .set("cycle_netting", self.cycles)
            .set("cycles_found", self.cycles_found)
    }
}

/// Full topology report: the ladder on the **gross** graph (bilateral offsetting counts as
/// circular: 2-cycles) and on the **net** graph (after bilateral netting).
#[derive(Clone, Debug)]
pub struct TopologyReport {
    pub gross: Ladder,
    pub net: Ladder,
    /// Share of gross flow removed by bilateral netting alone.
    pub bilateral_share: f64,
    /// Pearson correlation between the Hodge potential and the net position of each node
    /// (a qualitative check reported by Hodge studies of money flows).
    pub potential_vs_net_corr: f64,
}

impl TopologyReport {
    pub fn to_json(&self) -> Json {
        Json::obj()
            .set("gross", self.gross.to_json())
            .set("net", self.net.to_json())
            .set("bilateral_share", self.bilateral_share)
            .set("hodge_potential_vs_net_position_corr", self.potential_vs_net_corr)
    }
}

fn ladder(g: &FlowGraph, h: &Hodge) -> Ladder {
    let total = g.total_flow();
    let frac = |x: i128| if total == 0 { 0.0 } else { x as f64 / total as f64 };
    let c = cdfd_max(g);
    let nt = cycle_netting(g, false);
    // Hodge always works on the net flow; express its L1 cyclic mass relative to *this*
    // graph's total so that gross and net ladders are comparable with CDFD and cycles.
    let hodge_l1_mass: f64 = h.cyclic().iter().map(|x| x.abs()).sum();
    Ladder {
        nodes: g.n,
        edges: g.edges.len(),
        total_flow: total,
        hodge_l1: if total == 0 { 0.0 } else { hodge_l1_mass / total as f64 },
        hodge_curl_l2: h.curl_fraction_l2(),
        hodge_cyclic_l2: h.cyclic_fraction_l2(),
        hodge_triangles: h.triangles,
        hodge_triangles_capped: h.triangles_capped,
        cdfd_max: frac(c.circular_total),
        cycles: frac(nt.cancelled_total),
        cycles_found: nt.cycles,
    }
}

/// Run every lens on `g` (gross graph; reciprocal edges allowed).
pub fn analyze(g: &FlowGraph, cfg: &HodgeConfig) -> TopologyReport {
    let net = g.net();
    let h = hodge(g, cfg);
    let mut gross_l = ladder(g, &h);
    // On the gross graph the Hodge lens adds the bilaterally offset flow as circular too
    // (a 2-cycle i->j->i is divergence-free), so it is comparable with CDFD and cycles.
    let bilateral = g.total_flow() - net.total_flow();
    let hodge_l1_net_mass: f64 = h.cyclic().iter().map(|x| x.abs()).sum();
    if g.total_flow() > 0 {
        gross_l.hodge_l1 = (hodge_l1_net_mass + bilateral as f64) / g.total_flow() as f64;
    }
    let net_l = ladder(&net, &h);
    let net_pos: Vec<f64> = net.net_positions().iter().map(|&x| x as f64).collect();
    TopologyReport {
        gross: gross_l,
        net: net_l,
        bilateral_share: if g.total_flow() == 0 { 0.0 } else { bilateral as f64 / g.total_flow() as f64 },
        potential_vs_net_corr: pearson(&h.potential, &net_pos),
    }
}

pub fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    if n < 2.0 {
        return 0.0;
    }
    let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
    let (mut sab, mut saa, mut sbb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        sab += (x - ma) * (y - mb);
        saa += (x - ma) * (x - ma);
        sbb += (y - mb) * (y - mb);
    }
    if saa == 0.0 || sbb == 0.0 { 0.0 } else { sab / (saa * sbb).sqrt() }
}
