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
    delay_bounds_from(cfg.participants, || PaymentStream::new(cfg), delays_ms)
}

/// Same bound for any payment source sorted by arrival time (e.g. a real payment file);
/// `make` must produce the same stream every time it is called.
pub fn delay_bounds_from<I, F>(n: usize, make: F, delays_ms: &[TimeMs]) -> Vec<Vec<Cents>>
where
    I: Iterator<Item = loopsettle_core::Payment>,
    F: Fn() -> I,
{
    // Pass 1: time of the first minimum of each participant's gross cumulative position.
    let mut x = vec![0i64; n];
    let mut min = vec![0i64; n];
    let mut t_min: Vec<Option<TimeMs>> = vec![None; n];
    for p in make() {
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
    for p in make() {
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

/// Liquidity a participant must prefund for a window `[start, end)` in which its settlement
/// account cannot be topped up (W3: e.g. the Conta PI outside STR hours, nights and weekends).
#[derive(Clone, Debug, Default)]
pub struct WindowStats {
    /// per participant: drawdown from the window's opening position, max(0, x(start) - min x)
    pub need: Vec<Cents>,
    /// per participant: what one netting at the end of the window needs, max(0, x(start) - x(end))
    pub net: Vec<Cents>,
    /// per delay, per participant: the delay bound inside the window (section 9 of METHODOLOGY)
    pub bound: Vec<Vec<Cents>>,
}

/// Per-window prefunding need, end-of-window netting need and delay bound, for windows that
/// may overlap. Two streaming passes; memory O(windows x participants).
pub fn window_stats<I, F>(n: usize, make: F, windows: &[(TimeMs, TimeMs)], delays_ms: &[TimeMs]) -> Vec<WindowStats>
where
    I: Iterator<Item = loopsettle_core::Payment>,
    F: Fn() -> I,
{
    let w = windows.len();
    let mut x = vec![0i64; n];
    let mut x0 = vec![vec![0i64; n]; w];
    let mut started = vec![false; w];
    let mut min = vec![vec![0i64; n]; w];
    let mut t_min: Vec<Vec<Option<TimeMs>>> = vec![vec![None; n]; w];
    let mut xend = vec![vec![0i64; n]; w];
    let mut ended = vec![false; w];
    // Pass 1. A window "starts" just before the first payment at or after `start` and "ends"
    // just before the first payment at or after `end`, so positions are taken between payments.
    #[allow(clippy::too_many_arguments)]
    fn open(t: TimeMs, windows: &[(TimeMs, TimeMs)], x: &[i64], started: &mut [bool], ended: &mut [bool], x0: &mut [Vec<i64>], min: &mut [Vec<i64>], xend: &mut [Vec<i64>]) {
        for k in 0..windows.len() {
            if !started[k] && t >= windows[k].0 {
                started[k] = true;
                x0[k] = x.to_vec();
                min[k] = x.to_vec();
            }
            if started[k] && !ended[k] && t >= windows[k].1 {
                ended[k] = true;
                xend[k] = x.to_vec();
            }
        }
    }
    for p in make() {
        open(p.t_arrival, windows, &x, &mut started, &mut ended, &mut x0, &mut min, &mut xend);
        let (a, b) = (p.payer as usize, p.payee as usize);
        x[a] -= p.amount;
        x[b] += p.amount;
        for k in 0..w {
            if started[k] && !ended[k] && x[a] < min[k][a] {
                min[k][a] = x[a];
                t_min[k][a] = Some(p.t_arrival);
            }
        }
    }
    open(TimeMs::MAX, windows, &x, &mut started, &mut ended, &mut x0, &mut min, &mut xend);
    // Pass 2: outflows in [t* - d, t*] for each window.
    let mut bound = vec![vec![vec![0i64; n]; delays_ms.len()]; w];
    for p in make() {
        let a = p.payer as usize;
        for k in 0..w {
            if let Some(ts) = t_min[k][a] {
                if p.t_arrival <= ts {
                    for (j, &d) in delays_ms.iter().enumerate() {
                        if p.t_arrival >= ts.saturating_sub(d) {
                            bound[k][j][a] += p.amount;
                        }
                    }
                }
            }
        }
    }
    (0..w)
        .map(|k| WindowStats {
            need: (0..n).map(|i| (x0[k][i] - min[k][i]).max(0)).collect(),
            net: (0..n).map(|i| (x0[k][i] - xend[k][i]).max(0)).collect(),
            bound: std::mem::take(&mut bound[k]),
        })
        .collect()
}

#[cfg(test)]
mod window_tests {
    use super::*;
    use loopsettle_core::Payment;

    #[test]
    fn window_need_is_drawdown_from_the_window_start() {
        let p = |t, a, b, v| Payment { id: 0, t_arrival: t, payer: a, payee: b, amount: v };
        // A pays 10 before the window, then inside the window pays 5 and receives 2.
        let v = vec![p(1, 0, 1, 10), p(10, 0, 1, 5), p(12, 1, 0, 2), p(30, 0, 1, 1)];
        let s = window_stats(2, || v.clone().into_iter(), &[(5, 20)], &[3]);
        assert_eq!(s[0].need, vec![5, 0]);
        assert_eq!(s[0].net, vec![3, 0]);
        // the minimum of A inside the window is at t = 10; outflows in [7, 10] = 5
        assert_eq!(s[0].bound[0], vec![5, 0]);
    }
}
