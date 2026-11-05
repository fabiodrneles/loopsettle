//! Alternative settlement engines: deferred multilateral net settlement and the new
//! atomic micro-cycle engine.

use crate::engine::{SettlementEngine, Sink};
use crate::ledger::Ledger;
use crate::types::{Cents, ParticipantId, Payment, SettleKind, TimeMs, MS_PER_DAY};
use std::collections::{BTreeMap, VecDeque};

// =======================================================================================
// DeferredNet
// =======================================================================================

/// Deferred net settlement at fixed times (`cycles_per_day` evenly spaced cycles, the last
/// one at the end of each day; UPI-style with 10 cycles/day as a reference).
///
/// Every payment settles in the next cycle, so its settlement time is known on arrival:
/// the engine reports it immediately and keeps only the per-participant net deltas
/// (O(n) memory, no payment queue). Net debits are applied with [`Ledger::force_batch`]
/// (assumes collateralised DNS; balances may go negative in constrained mode).
#[derive(Clone, Debug)]
pub struct DeferredNet {
    pub cycles_per_day: u32,
    interval: TimeMs,
    next_cycle: TimeMs,
    deltas: Vec<Cents>,
    pending: usize,
}

impl DeferredNet {
    pub fn new(cycles_per_day: u32, participants: usize) -> Self {
        assert!(cycles_per_day >= 1);
        let interval = MS_PER_DAY / cycles_per_day as TimeMs;
        DeferredNet { cycles_per_day, interval, next_cycle: interval, deltas: vec![0; participants], pending: 0 }
    }
    fn settle_batch(&mut self, ledger: &mut Ledger) {
        if self.pending > 0 {
            ledger.force_batch(&self.deltas);
            self.deltas.iter_mut().for_each(|d| *d = 0);
            self.pending = 0;
        }
    }
}

impl SettlementEngine for DeferredNet {
    fn name(&self) -> String {
        format!("deferred_net_{}x", self.cycles_per_day)
    }
    fn advance_to(&mut self, t: TimeMs, ledger: &mut Ledger, _s: &mut dyn Sink) {
        while self.next_cycle <= t {
            self.settle_batch(ledger);
            self.next_cycle += self.interval;
        }
    }
    fn submit(&mut self, p: Payment, _l: &mut Ledger, sink: &mut dyn Sink) {
        self.deltas[p.payer as usize] -= p.amount;
        self.deltas[p.payee as usize] += p.amount;
        self.pending += 1;
        sink.settled(&p, self.next_cycle, SettleKind::Deferred);
    }
    fn finish(&mut self, _t: TimeMs, ledger: &mut Ledger, _s: &mut dyn Sink) {
        self.settle_batch(ledger);
    }
    fn queued(&self) -> usize {
        self.pending
    }
}

// =======================================================================================
// AtomicMicroCycle
// =======================================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueMode {
    /// Every payment waits in the queue (up to `max_delay_ms`) for an offsetting cycle.
    QueueAll,
    /// Settle gross on arrival when the payer has funds; queue only on a shortfall
    /// (classic liquidity-saving mechanism). Queued payments are retried gross every window.
    QueueOnShortfall,
}

#[derive(Clone, Copy, Debug)]
pub struct MicroCycleConfig {
    pub window_ms: TimeMs,
    pub max_delay_ms: TimeMs,
    pub mode: QueueMode,
    /// Longest cycle searched (2 = bilateral only). Bounded to keep the search cheap.
    pub max_cycle_len: usize,
}

impl MicroCycleConfig {
    pub fn new(window_ms: TimeMs, max_delay_ms: TimeMs, mode: QueueMode) -> Self {
        MicroCycleConfig { window_ms, max_delay_ms, mode, max_cycle_len: 6 }
    }
}

/// Atomic micro-cycles: every `window_ms` the engine searches the queue for directed cycles
/// of payments (one payment per edge, the oldest on each edge) and settles each cycle
/// atomically — all its payments or none (invariant 4). A payment still queued
/// `max_delay_ms` after arrival goes to gross settlement (rejected if the payer lacks funds).
///
/// Queues are FIFO per (payer, payee) edge, stored in `BTreeMap`s so iteration order — and
/// therefore every result — is deterministic (a `HashMap` iterates in random order).
pub struct AtomicMicroCycle {
    pub cfg: MicroCycleConfig,
    /// out[payer][payee] = FIFO of queued payments on that edge
    out: Vec<BTreeMap<ParticipantId, VecDeque<Payment>>>,
    /// all queued payments in arrival order (deadline = arrival + max_delay is monotonic)
    deadlines: VecDeque<Payment>,
    next_tick: TimeMs,
    queued: usize,
    scratch: Vec<Cents>,
    /// optional log of settled cycles: (time, payment ids, offset = min payment)
    pub log: Option<Vec<(TimeMs, Vec<u64>, Cents)>>,
    pub cycles_settled: u64,
    pub cycles_failed: u64,
}

impl AtomicMicroCycle {
    pub fn new(cfg: MicroCycleConfig, participants: usize) -> Self {
        assert!(cfg.window_ms > 0 && cfg.max_cycle_len >= 2);
        AtomicMicroCycle {
            cfg,
            out: vec![BTreeMap::new(); participants],
            deadlines: VecDeque::new(),
            next_tick: cfg.window_ms,
            queued: 0,
            scratch: vec![0; participants],
            log: None,
            cycles_settled: 0,
            cycles_failed: 0,
        }
    }

    pub fn with_log(mut self) -> Self {
        self.log = Some(Vec::new());
        self
    }

    fn enqueue(&mut self, p: Payment) {
        self.out[p.payer as usize].entry(p.payee).or_default().push_back(p);
        self.deadlines.push_back(p);
        self.queued += 1;
    }

    /// `true` if `p` is still queued (it is then necessarily the head of its edge: payments
    /// leave an edge queue only from the front, and older ones expire first).
    fn is_head(&self, p: &Payment) -> bool {
        self.out[p.payer as usize].get(&p.payee).and_then(|q| q.front()).map(|h| h.id == p.id).unwrap_or(false)
    }

    fn pop_head(&mut self, payer: ParticipantId, payee: ParticipantId) -> Payment {
        let m = &mut self.out[payer as usize];
        let q = m.get_mut(&payee).expect("edge queue exists");
        let p = q.pop_front().expect("edge queue non-empty");
        if q.is_empty() {
            m.remove(&payee);
        }
        self.queued -= 1;
        p
    }

    /// Depth-first search for a directed cycle (length <= max_cycle_len) among non-blocked
    /// queued edges. Returns the cycle as a list of nodes `[v0, v1, ..., vk-1]` (edges
    /// v0->v1, ..., vk-1->v0).
    ///
    /// Colors: 0 = unvisited, 1 = on the current path, 2 = fully explored with no cycle
    /// reachable. A node whose subtree was cut by the length bound is *not* fully explored,
    /// so it goes back to 0 (and the "truncated" flag propagates to its ancestors).
    fn find_cycle(&self, blocked: &BTreeMap<(u32, u32), ()>, color: &mut [u8]) -> Option<Vec<u32>> {
        struct Frame {
            v: u32,
            after: Option<u32>,
            truncated: bool,
        }
        let n = self.out.len();
        color.iter_mut().for_each(|c| *c = 0);
        let mut path: Vec<u32> = Vec::new();
        for root in 0..n as u32 {
            if color[root as usize] != 0 || self.out[root as usize].is_empty() {
                continue;
            }
            let mut stack = vec![Frame { v: root, after: None, truncated: false }];
            color[root as usize] = 1;
            path.clear();
            path.push(root);
            while let Some(top) = stack.last_mut() {
                let v = top.v;
                let range = match top.after {
                    None => self.out[v as usize].range(..),
                    Some(a) => self.out[v as usize].range(a + 1..),
                };
                let mut next = None;
                for (&w, _) in range {
                    top.after = Some(w);
                    if blocked.contains_key(&(v, w)) {
                        continue;
                    }
                    match color[w as usize] {
                        1 => {
                            let k = path.iter().position(|&x| x == w).unwrap();
                            return Some(path[k..].to_vec());
                        }
                        0 if path.len() < self.cfg.max_cycle_len => {
                            next = Some(w);
                            break;
                        }
                        0 => top.truncated = true,
                        _ => {}
                    }
                }
                match next {
                    Some(w) => {
                        color[w as usize] = 1;
                        path.push(w);
                        stack.push(Frame { v: w, after: None, truncated: false });
                    }
                    None => {
                        let done = stack.pop().unwrap();
                        path.pop();
                        color[done.v as usize] = if done.truncated { 0 } else { 2 };
                        if done.truncated {
                            if let Some(parent) = stack.last_mut() {
                                parent.truncated = true;
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Settle all cycles that can be settled now.
    fn run_cycles(&mut self, t: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink) {
        let mut blocked: BTreeMap<(u32, u32), ()> = BTreeMap::new();
        let mut color = vec![0u8; self.out.len()];
        let mut legs: Vec<(u32, u32, Cents)> = Vec::new();
        while let Some(cyc) = self.find_cycle(&blocked, &mut color) {
            legs.clear();
            for (i, &a) in cyc.iter().enumerate() {
                let b = cyc[(i + 1) % cyc.len()];
                let head = self.out[a as usize][&b].front().unwrap();
                legs.push((a, b, head.amount));
            }
            if ledger.try_batch(&legs, &mut self.scratch) {
                let offset = legs.iter().map(|l| l.2).min().unwrap();
                let mut ids = Vec::with_capacity(legs.len());
                for i in 0..legs.len() {
                    let (a, b, _) = legs[i];
                    let p = self.pop_head(a, b);
                    ids.push(p.id);
                    sink.settled(&p, t, SettleKind::Cycle);
                }
                self.cycles_settled += 1;
                if let Some(l) = self.log.as_mut() {
                    l.push((t, ids, offset));
                }
            } else {
                // Block the out-edge of every participant that cannot cover its net debit in
                // this cycle; the search continues without those edges for this window.
                self.cycles_failed += 1;
                let mut any = false;
                for (i, &a) in cyc.iter().enumerate() {
                    let prev = legs[(i + legs.len() - 1) % legs.len()].2;
                    if ledger.balance(a) + prev - legs[i].2 < 0 {
                        blocked.insert((a, legs[i].1), ());
                        any = true;
                    }
                }
                debug_assert!(any);
            }
        }
    }

    /// QueueOnShortfall: retry queued payments gross in arrival order (FIFO per edge).
    fn retry_gross(&mut self, t: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink) {
        let snapshot: Vec<Payment> = self.deadlines.iter().copied().filter(|p| self.is_head(p)).collect();
        for p in snapshot {
            if self.is_head(&p) && ledger.transfer(p.payer, p.payee, p.amount) {
                self.pop_head(p.payer, p.payee);
                sink.settled(&p, t, SettleKind::Gross);
            }
        }
    }

    fn tick(&mut self, t: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink) {
        if self.queued == 0 {
            return;
        }
        self.run_cycles(t, ledger, sink);
        if self.cfg.mode == QueueMode::QueueOnShortfall && self.queued > 0 {
            self.retry_gross(t, ledger, sink);
        }
    }

    fn expire(&mut self, p: Payment, ledger: &mut Ledger, sink: &mut dyn Sink) {
        let t = p.t_arrival + self.cfg.max_delay_ms;
        if !self.is_head(&p) {
            return; // already settled
        }
        let p = self.pop_head(p.payer, p.payee);
        if ledger.transfer(p.payer, p.payee, p.amount) {
            sink.settled(&p, t, SettleKind::Gross);
        } else {
            sink.rejected(&p, t);
        }
    }
}

impl SettlementEngine for AtomicMicroCycle {
    fn name(&self) -> String {
        let m = match self.cfg.mode {
            QueueMode::QueueAll => "all",
            QueueMode::QueueOnShortfall => "shortfall",
        };
        format!("micro_cycle_{m}_w{}_d{}", self.cfg.window_ms, self.cfg.max_delay_ms)
    }

    /// Fire window ticks and deadlines due at or before `t`, in time order (a tick before
    /// a deadline at the same instant: the payment gets a last chance to be netted).
    fn advance_to(&mut self, t: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink) {
        loop {
            // Pop deadlines that are no longer queued (already settled) lazily.
            // Drop deadline entries of payments already settled (see `is_head`).
            while let Some(front) = self.deadlines.front() {
                if self.is_head(front) {
                    break;
                }
                self.deadlines.pop_front();
            }
            let dl = self.deadlines.front().map(|p| p.t_arrival + self.cfg.max_delay_ms);
            let tick = self.next_tick;
            match dl {
                Some(d) if d <= t && d < tick => {
                    let p = self.deadlines.pop_front().unwrap();
                    self.expire(p, ledger, sink);
                }
                _ if tick <= t => {
                    self.tick(tick, ledger, sink);
                    self.next_tick += self.cfg.window_ms;
                    if self.queued == 0 && self.next_tick < t {
                        // Nothing queued: jump over idle windows (they would do nothing).
                        let k = (t - self.next_tick) / self.cfg.window_ms;
                        self.next_tick += k * self.cfg.window_ms;
                    }
                }
                _ => break,
            }
        }
    }

    fn submit(&mut self, p: Payment, ledger: &mut Ledger, sink: &mut dyn Sink) {
        if self.cfg.mode == QueueMode::QueueOnShortfall && ledger.transfer(p.payer, p.payee, p.amount) {
            sink.settled(&p, p.t_arrival, SettleKind::Gross);
            return;
        }
        if self.cfg.max_delay_ms == 0 {
            // No waiting allowed: behaves like gross settlement.
            if ledger.transfer(p.payer, p.payee, p.amount) {
                sink.settled(&p, p.t_arrival, SettleKind::Gross);
            } else {
                sink.rejected(&p, p.t_arrival);
            }
            return;
        }
        self.enqueue(p);
    }

    fn finish(&mut self, _t: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink) {
        // Keep the clock running until every queued payment has settled or expired.
        if let Some(last) = self.deadlines.back().map(|p| p.t_arrival + self.cfg.max_delay_ms) {
            self.advance_to(last, ledger, sink);
        }
        debug_assert_eq!(self.queued, 0);
    }

    fn queued(&self) -> usize {
        self.queued
    }

    fn stats(&self) -> Vec<(&'static str, f64)> {
        vec![("cycles_settled", self.cycles_settled as f64), ("cycles_failed", self.cycles_failed as f64)]
    }
}
