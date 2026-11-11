//! Offline optimum of delay-bounded settlement, as a linear programme (specs/03).
//!
//! Given all payments in advance and a maximum delay `d`, choose when each payment settles in
//! `[arrival, arrival + d]` so that the total liquidity `sum_i L_i` is smallest, where `L_i` is
//! the lowest point of participant `i`'s position. Payments settled at the same instant settle
//! atomically (only the net effect counts), as in `Ledger::try_batch`.
//!
//! The programme lets a payment be split across instants, so its value is a **lower bound** on
//! what any engine with maximum delay `d` needs — one that knows the future and may settle
//! arbitrary batches, not only cycles. It is at least the sum of the per-participant liquidity
//! floors, which are its relaxation to one participant at a time:
//!
//! ```text
//! sum_i floor_i(d)  <=  LP(d)  <=  optimum with whole payments  <=  any engine with delay d
//! ```
//!
//! Variables: `Z[p][k]` = share of payment `p` settled by instant `k` (non-decreasing, 0 before
//! its arrival, 1 at its deadline); `L[i]`. Constraint, for every participant and instant:
//! `L_i + inflow settled by k - outflow settled by k >= 0`. Shares that are certainly 1 (deadline
//! passed) are constants, so each row only holds the payments whose window is open.
//!
//! Solved with `minilp` (dev-dependency): a test oracle, never used by the library or the CLI.

use loopsettle_core::{Payment, TimeMs};
use minilp::{ComparisonOp, OptimizationDirection, Problem, Variable};

/// Minimum total liquidity, in the same unit as the amounts. `None` if the solver fails.
pub fn lp_min_liquidity(n: usize, payments: &[Payment], d: TimeMs) -> Option<f64> {
    if payments.is_empty() {
        return Some(0.0);
    }
    // Instants at which something can happen: arrivals and deadlines.
    let mut t: Vec<TimeMs> = payments.iter().flat_map(|p| [p.t_arrival, p.t_arrival + d]).collect();
    t.sort_unstable();
    t.dedup();
    let idx = |x: TimeMs| t.binary_search(&x).unwrap();
    // Amounts are scaled to around 1 for the solver's tolerances.
    let scale = payments.iter().map(|p| p.amount as f64).sum::<f64>() / payments.len() as f64;
    let mut lp = Problem::new(OptimizationDirection::Minimize);
    let l: Vec<Variable> = (0..n).map(|_| lp.add_var(1.0, (0.0, f64::INFINITY))).collect();
    // z[p] = variables for instants first..last-1 of the payment's window (at `last` it is 1)
    let mut z: Vec<(usize, usize, Vec<Variable>)> = Vec::with_capacity(payments.len());
    for p in payments {
        let (first, last) = (idx(p.t_arrival), idx(p.t_arrival + d));
        let vars: Vec<Variable> = (first..last).map(|_| lp.add_var(0.0, (0.0, 1.0))).collect();
        for w in vars.windows(2) {
            lp.add_constraint(&[(w[1], 1.0), (w[0], -1.0)][..], ComparisonOp::Ge, 0.0);
        }
        z.push((first, last, vars));
    }
    // Position rows. `done[i]` = net value of payments whose deadline has passed (constants).
    let mut by_last: Vec<Vec<usize>> = vec![Vec::new(); t.len()];
    let mut by_first: Vec<Vec<usize>> = vec![Vec::new(); t.len()];
    for (j, (first, last, _)) in z.iter().enumerate() {
        by_first[*first].push(j);
        by_last[*last].push(j);
    }
    let mut done = vec![0.0f64; n];
    let mut open: Vec<Vec<usize>> = vec![Vec::new(); n]; // open payments touching each participant
    for k in 0..t.len() {
        for &j in &by_last[k] {
            let p = &payments[j];
            let v = p.amount as f64 / scale;
            done[p.payer as usize] -= v;
            done[p.payee as usize] += v;
            for who in [p.payer as usize, p.payee as usize] {
                open[who].retain(|&x| x != j);
            }
        }
        for &j in &by_first[k] {
            let (first, last, _) = &z[j];
            if first < last {
                let p = &payments[j];
                open[p.payer as usize].push(j);
                open[p.payee as usize].push(j);
            }
        }
        for i in 0..n {
            let mut row: Vec<(Variable, f64)> = vec![(l[i], 1.0)];
            for &j in &open[i] {
                let p = &payments[j];
                let v = p.amount as f64 / scale;
                let var = z[j].2[k - z[j].0];
                row.push((var, if p.payee as usize == i { v } else { -v }));
            }
            // L_i + (variable part) + done_i >= 0
            if row.len() > 1 || done[i] < 0.0 {
                lp.add_constraint(&row[..], ComparisonOp::Ge, -done[i]);
            }
        }
    }
    lp.solve().ok().map(|s| s.objective() * scale)
}
