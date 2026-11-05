//! Upper bound on the liquidity any delay-bounded settlement mechanism can save.
//!
//! Claim (proved in `docs/METHODOLOGY.md`, section 9): with unlimited opening balances, let
//! `x_i(t)` be participant `i`'s cumulative position under real-time gross settlement and
//! `t*_i` the first time `x_i` reaches its minimum (so `UB_i = -x_i(t*_i)`). For **any**
//! engine in which every payment settles within `d` ms of its arrival (atomic micro-cycles
//! with `max_delay_ms = d`, deferred netting with an interval of `d`, or an optimal gridlock
//! solver with the same deadline), the saving `UB_i - L_i` is at most the value `i` paid in
//! the window `[t*_i - d, t*_i]`:
//!
//! ```text
//! y_i(t*) <= x_i(t*) + O_i[t* - d, t*]      (only payments that arrived in the last d ms
//!                                             can still be unsettled at t*)
//! L_i = max(0, -min_t y_i(t)) >= -y_i(t*)  =>  UB_i - L_i <= O_i[t* - d, t*]
//! ```
//!
//! The bound does not depend on how clever the cycle search is, which is why it answers the
//! "maybe a better algorithm would save more" objection.

use crate::scenario::ScenarioConfig;
use crate::stream::PaymentStream;
use loopsettle_core::{Cents, TimeMs};

/// For each delay in `delays_ms`, the per-participant bound (cents) on the liquidity saving of
/// any mechanism whose maximum settlement delay is that value. Two streaming passes over the
/// scenario (the stream is regenerated from the seed, so memory stays O(participants)).
pub fn delay_bounds(cfg: &ScenarioConfig, delays_ms: &[TimeMs]) -> Vec<Vec<Cents>> {
    let n = cfg.participants;
    // Pass 1: time of the first minimum of each participant's gross cumulative position.
    let mut x = vec![0i64; n];
    let mut min = vec![0i64; n];
    let mut t_min: Vec<Option<TimeMs>> = vec![None; n];
    for p in PaymentStream::new(cfg) {
        let (a, b) = (p.payer as usize, p.payee as usize);
        x[a] -= p.amount;
        x[b] += p.amount;
        if x[a] < min[a] {
            min[a] = x[a];
            t_min[a] = Some(p.t_arrival);
        }
    }
    // Pass 2: outflows of each participant in [t* - d, t*], for every d.
    let mut out = vec![vec![0i64; n]; delays_ms.len()];
    for p in PaymentStream::new(cfg) {
        let a = p.payer as usize;
        if let Some(ts) = t_min[a] {
            if p.t_arrival <= ts {
                for (k, &d) in delays_ms.iter().enumerate() {
                    if p.t_arrival >= ts.saturating_sub(d) {
                        out[k][a] += p.amount;
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{run, EngineSpec, Opening};

    /// The bound holds participant by participant for every delay-bounded engine.
    #[test]
    fn engines_never_beat_the_delay_bound() {
        for (seed, profile_strength) in [(1u64, 0.0), (2, 0.0), (3, 0.0), (4, 1.0), (5, 0.5)] {
            let cfg = ScenarioConfig { seed, participants: 25, payments_per_day: 40_000.0, n_big: 3, n_mid: 5, profile_strength, ..Default::default() };
            // (engine, its maximum settlement delay in ms)
            let cases = [
                ("micro:all:500:2000", 2_000u64),
                ("micro:all:1000:10000", 10_000),
                ("micro:all:5000:10000:2", 10_000),
                ("deferred:8640", 10_000),
                ("deferred:96", 900_000),
                ("deferred:24", 3_600_000),
            ];
            let mut specs = vec![EngineSpec::Gross];
            specs.extend(cases.iter().map(|(s, _)| EngineSpec::parse(s).unwrap()));
            let o = run(&cfg, &specs, &Opening::Unlimited, false, None);
            let delays: Vec<u64> = cases.iter().map(|&(_, d)| d).collect();
            let bounds = delay_bounds(&cfg, &delays);
            for (k, r) in o.results[1..].iter().enumerate() {
                for i in 0..cfg.participants {
                    let ub = o.flow.ub(i as u32).max(0);
                    let saving = ub - r.liquidity_by_participant[i];
                    assert!(
                        saving <= bounds[k][i],
                        "seed {seed}, {}: participant {i} saved {saving} > bound {}",
                        cases[k].0,
                        bounds[k][i]
                    );
                }
            }
        }
    }

    #[test]
    fn bound_grows_with_the_delay() {
        let cfg = ScenarioConfig { participants: 20, payments_per_day: 20_000.0, n_big: 2, n_mid: 3, ..Default::default() };
        let b = delay_bounds(&cfg, &[1_000, 10_000, 100_000]);
        for i in 0..cfg.participants {
            assert!(b[0][i] <= b[1][i] && b[1][i] <= b[2][i]);
        }
    }
}
