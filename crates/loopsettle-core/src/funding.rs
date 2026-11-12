//! Funding policies (specs/05, M5): what participants do with their settlement accounts over
//! time — bring funds in from reserves, take them out — as a layer between the driver and the
//! engines. A policy acts at instants it chooses; the driver lets the engine catch up to each
//! instant first, so events keep their order in time.
//!
//! Money moved by a policy comes from outside the system ([`Ledger::fund`]); conservation then
//! holds for accounts plus reserves ([`Ledger::total_with_reserves`]).

use crate::ledger::{Ledger, UNLIMITED};
use crate::types::{Cents, TimeMs};

pub trait FundingPolicy {
    /// The next instant at which the policy wants to act, if any.
    fn next_instant(&self) -> Option<TimeMs>;
    /// Act at `next_instant()` and move on to the following one.
    fn apply(&mut self, ledger: &mut Ledger);
}

/// The funding rule of DECISIONS.md, D20, as a policy: while top-ups are possible (the central
/// bank's RTGS is open) liquidity is not a constraint; when a closed window begins, each
/// participant is left with a fixed prefunded balance, which is all it has until the window
/// ends.
///
/// `closed` = the closed windows `[start, end)`, sorted and disjoint. At `start` every balance
/// is set to `prefund[p]`; at `end` it is set back to [`UNLIMITED`]. Run it on a ledger opened
/// with [`Ledger::unlimited`]. Under gross settlement a participant then has a payment rejected
/// in a window exactly when its prefunding is below its drawdown in that window
/// (`loopsettle_sim::bound::window_stats`).
pub struct PrefundClosedWindows {
    closed: Vec<(TimeMs, TimeMs)>,
    prefund: Vec<Cents>,
    /// index into the flattened list of instants: 2k = start of window k, 2k + 1 = its end
    next: usize,
}

impl PrefundClosedWindows {
    pub fn new(closed: Vec<(TimeMs, TimeMs)>, prefund: Vec<Cents>) -> Self {
        assert!(closed.iter().all(|w| w.0 < w.1) && closed.windows(2).all(|w| w[0].1 <= w[1].0), "closed windows must be sorted and disjoint");
        assert!(prefund.iter().all(|&b| b >= 0));
        PrefundClosedWindows { closed, prefund, next: 0 }
    }
}

impl FundingPolicy for PrefundClosedWindows {
    fn next_instant(&self) -> Option<TimeMs> {
        self.closed.get(self.next / 2).map(|w| if self.next % 2 == 0 { w.0 } else { w.1 })
    }
    fn apply(&mut self, ledger: &mut Ledger) {
        let closing = self.next % 2 == 0;
        for p in 0..ledger.len() as u32 {
            let target = if closing { self.prefund[p as usize] } else { UNLIMITED };
            ledger.fund(p, target - ledger.balance(p));
        }
        self.next += 1;
    }
}

/// Periodic top-ups to a target while a funding window is open (and nothing while it is
/// closed): every `period_ms` inside an open window, a participant whose balance is below its
/// target brings it up to the target. Balances above the target are left alone.
///
/// Unlike [`PrefundClosedWindows`], liquidity is finite at all times, so payments can be
/// rejected between two top-ups as well.
pub struct TopUpToTarget {
    open: Vec<(TimeMs, TimeMs)>,
    period_ms: TimeMs,
    target: Vec<Cents>,
    window: usize,
    at: TimeMs,
    /// number of top-ups made and their total value
    pub top_ups: u64,
    pub topped_up: i128,
}

impl TopUpToTarget {
    pub fn new(open: Vec<(TimeMs, TimeMs)>, period_ms: TimeMs, target: Vec<Cents>) -> Self {
        assert!(period_ms > 0 && open.iter().all(|w| w.0 < w.1) && open.windows(2).all(|w| w[0].1 <= w[1].0), "open windows must be sorted and disjoint");
        let at = open.first().map(|w| w.0).unwrap_or(0);
        TopUpToTarget { open, period_ms, target, window: 0, at, top_ups: 0, topped_up: 0 }
    }
}

impl FundingPolicy for TopUpToTarget {
    fn next_instant(&self) -> Option<TimeMs> {
        (self.window < self.open.len()).then_some(self.at)
    }
    fn apply(&mut self, ledger: &mut Ledger) {
        for p in 0..ledger.len() as u32 {
            let short = self.target[p as usize] - ledger.balance(p);
            if short > 0 {
                ledger.fund(p, short);
                self.top_ups += 1;
                self.topped_up += short as i128;
            }
        }
        // next instant: one period later, or the start of the next open window
        self.at += self.period_ms;
        if self.at >= self.open[self.window].1 {
            self.window += 1;
            if let Some(w) = self.open.get(self.window) {
                self.at = w.0;
            }
        }
    }
}
