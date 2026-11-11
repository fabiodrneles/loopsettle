//! Alternative settlement engines: deferred multilateral net settlement and the new
//! atomic micro-cycle engine.

use crate::engine::{SettlementEngine, Sink};
use crate::ledger::Ledger;
use crate::types::{Cents, ParticipantId, Payment, SettleKind, TimeMs, MS_PER_DAY};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};

// =======================================================================================
// DeferredNet
// =======================================================================================

/// Deferred net settlement at fixed times (`cycles_per_day` evenly spaced cycles, the last
/// one exactly at the end of each day; UPI-style with 10 cycles/day as a reference).
///
/// Every payment settles in the next cycle, so its settlement time is known on arrival:
/// the engine reports it immediately and keeps only the per-participant net deltas
/// (O(n) memory, no payment queue). Net debits are applied with [`Ledger::force_batch`]
/// (assumes collateralised DNS; balances may go negative in constrained mode).
#[derive(Clone, Debug)]
pub struct DeferredNet {
    pub cycles_per_day: u32,
    /// index of the next cycle (1 = first cycle of day 0)
    next_index: u64,
    next_cycle: TimeMs,
    deltas: Vec<Cents>,
    pending: usize,
}

impl DeferredNet {
    pub fn new(cycles_per_day: u32, participants: usize) -> Self {
        assert!(cycles_per_day >= 1);
        let mut d = DeferredNet { cycles_per_day, next_index: 1, next_cycle: 0, deltas: vec![0; participants], pending: 0 };
        d.next_cycle = d.cycle_time(1);
        d
    }
    /// Time of cycle `k >= 1`: cycle `j` of a day is at `floor(j * day / cycles_per_day)`, so
    /// the last cycle of every day falls exactly at the end of the day even when
    /// `cycles_per_day` does not divide it (no rounding error accumulates).
    fn cycle_time(&self, k: u64) -> TimeMs {
        let c = self.cycles_per_day as u64;
        (k / c) * MS_PER_DAY + (k % c) * MS_PER_DAY / c
    }
    /// Longest gap between two consecutive cycles = the engine's maximum settlement delay.
    pub fn max_delay_ms(&self) -> TimeMs {
        MS_PER_DAY.div_ceil(self.cycles_per_day as TimeMs)
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
            self.next_index += 1;
            self.next_cycle = self.cycle_time(self.next_index);
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

/// Order in which one payer's queued payments may settle gross in `QueueOnShortfall`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum QueueDiscipline {
    /// A payment that fits settles even if an older, larger payment of the same payer is
    /// still queued (bypass FIFO; DECISIONS.md D13). The default.
    #[default]
    BypassFifo,
    /// A payer's payments settle gross strictly in arrival order: a new payment queues behind
    /// anything the payer already has queued, and a retry stops at the payer's first payment
    /// that does not fit. Cycles may still settle a queued payment out of order (that is
    /// what a liquidity-saving mechanism is for).
    StrictFifoPerPayer,
}

#[derive(Clone, Copy, Debug)]
pub struct MicroCycleConfig {
    pub window_ms: TimeMs,
    pub max_delay_ms: TimeMs,
    pub mode: QueueMode,
    /// Longest cycle searched (2 = bilateral only). Bounded to keep the search cheap.
    pub max_cycle_len: usize,
    /// Only used by `QueueOnShortfall`.
    pub discipline: QueueDiscipline,
    /// `false` turns the cycle search off: with `QueueOnShortfall` the engine is then a plain
    /// queue (settle gross when funds allow, retry every window, reject at the deadline) — the
    /// "FIFO queue" baseline of the liquidity-saving-mechanism literature.
    pub search_cycles: bool,
    /// Use the slow, obviously-correct gross retry (every queued payment visited in every
    /// pass) instead of the event-driven one. Same results by construction; kept so the two
    /// can be compared in tests. Leave `false`.
    pub reference_retry: bool,
}

impl MicroCycleConfig {
    pub fn new(window_ms: TimeMs, max_delay_ms: TimeMs, mode: QueueMode) -> Self {
        MicroCycleConfig { window_ms, max_delay_ms, mode, max_cycle_len: 6, discipline: QueueDiscipline::BypassFifo, search_cycles: true, reference_retry: false }
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
    /// out[payer][payee] = FIFO of queued payments on that edge, each with its queue sequence
    /// number (the order in which payments entered the queue: arrival order)
    out: Vec<BTreeMap<ParticipantId, VecDeque<(u64, Payment)>>>,
    next_seq: u64,
    /// participants whose queued payments must be retried gross at the next window: they were
    /// credited, or one of their queued payments left the queue, since their last retry
    pending: Vec<bool>,
    pending_list: Vec<ParticipantId>,
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
            next_seq: 0,
            pending: vec![false; participants],
            pending_list: Vec::new(),
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
        self.out[p.payer as usize].entry(p.payee).or_default().push_back((self.next_seq, p));
        self.next_seq += 1;
        self.deadlines.push_back(p);
        self.queued += 1;
    }

    /// Note that `who` may now be able to settle something it could not before (see
    /// `retry_gross`). Cheap and idempotent.
    fn mark(&mut self, who: ParticipantId) {
        if !self.pending[who as usize] {
            self.pending[who as usize] = true;
            self.pending_list.push(who);
        }
    }

    /// `true` if `p` is still queued (it is then necessarily the head of its edge: payments
    /// leave an edge queue only from the front, and older ones expire first).
    fn is_head(&self, p: &Payment) -> bool {
        self.out[p.payer as usize].get(&p.payee).and_then(|q| q.front()).map(|h| h.1.id == p.id).unwrap_or(false)
    }

    fn pop_head(&mut self, payer: ParticipantId, payee: ParticipantId) -> Payment {
        let m = &mut self.out[payer as usize];
        let q = m.get_mut(&payee).expect("edge queue exists");
        let (_, p) = q.pop_front().expect("edge queue non-empty");
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
                legs.push((a, b, head.1.amount));
            }
            if ledger.try_batch(&legs, &mut self.scratch) {
                let offset = legs.iter().map(|l| l.2).min().unwrap();
                let mut ids = Vec::with_capacity(legs.len());
                for i in 0..legs.len() {
                    let (a, b, _) = legs[i];
                    let p = self.pop_head(a, b);
                    ids.push(p.id);
                    sink.settled(&p, t, SettleKind::Cycle);
                    self.mark(a); // one of its payments left the queue
                    self.mark(b); // it was credited
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

    /// QueueOnShortfall: retry queued payments gross in arrival order, to a fixed point.
    ///
    /// **What it computes** (the definition, implemented literally by
    /// [`Self::retry_gross_reference`]): one pass visits every queued payment in arrival order
    /// and settles those that are at the head of their edge and fit, so several payments of
    /// the same edge can go in one window. A settlement credits a payee that an earlier part
    /// of the pass may have skipped for lack of funds, so passes repeat until one settles
    /// nothing. With `StrictFifoPerPayer`, a payer is skipped for the rest of the pass after
    /// its first payment that does not fit.
    ///
    /// **How.** Visiting every queued payment in every window costs O(queue) per window, which
    /// is too slow when payments may wait for an hour. An attempt can only succeed for a payer
    /// that was credited, or lost a queued payment (expiry, cycle), since it was last tried;
    /// every other attempt of the definition fails and changes nothing. So only those payers
    /// (`pending`) are visited, through a heap of edge heads ordered by queue sequence, which
    /// reproduces the arrival order of the definition. A payer credited in the middle of a pass
    /// joins it for its payments that come later in that order, exactly as in the definition.
    /// `tests/engine_invariants.rs` checks that both versions produce identical settlements.
    fn retry_gross(&mut self, t: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink) {
        if self.cfg.reference_retry {
            return self.retry_gross_reference(t, ledger, sink);
        }
        let strict = self.cfg.discipline == QueueDiscipline::StrictFifoPerPayer;
        let n = self.out.len();
        let mut in_pass = vec![false; n];
        let mut stuck = vec![false; if strict { n } else { 0 }];
        // (queue sequence, payer, payee) of edge heads still to visit in this pass
        let mut heap: BinaryHeap<Reverse<(u64, ParticipantId, ParticipantId)>> = BinaryHeap::new();
        while !self.pending_list.is_empty() && self.queued > 0 {
            let set = std::mem::take(&mut self.pending_list);
            in_pass.iter_mut().for_each(|b| *b = false);
            stuck.iter_mut().for_each(|b| *b = false);
            for &x in &set {
                self.pending[x as usize] = false;
                in_pass[x as usize] = true;
                for (&payee, q) in &self.out[x as usize] {
                    heap.push(Reverse((q.front().expect("edge queues are never empty").0, x, payee)));
                }
            }
            while let Some(Reverse((seq, x, payee))) = heap.pop() {
                if strict && stuck[x as usize] {
                    continue;
                }
                let amount = self.out[x as usize][&payee].front().expect("heap entries are current edge heads").1.amount;
                if !ledger.transfer(x, payee, amount) {
                    if strict {
                        stuck[x as usize] = true;
                    }
                    continue;
                }
                let p = self.pop_head(x, payee);
                sink.settled(&p, t, SettleKind::Gross);
                // The next payment of the same edge is now a head; it comes later in the order.
                if let Some(next) = self.out[x as usize].get(&payee).and_then(|q| q.front()) {
                    heap.push(Reverse((next.0, x, payee)));
                }
                // The payee was credited: it is retried in the next pass, and joins this one
                // for its payments after `seq` (the definition has already visited, and failed,
                // the earlier ones in this pass).
                self.mark(payee);
                if !in_pass[payee as usize] {
                    let y = payee as usize;
                    let oldest = self.out[y].values().map(|q| q.front().unwrap().0).min();
                    // Strict FIFO: if its oldest payment was already visited in this pass, the
                    // definition has marked the payer stuck for the rest of the pass.
                    let joins = match oldest {
                        None => false,
                        Some(o) => !strict || o > seq,
                    };
                    if joins {
                        in_pass[y] = true;
                        for (&to, q) in &self.out[y] {
                            let head = q.front().unwrap().0;
                            if head > seq {
                                heap.push(Reverse((head, payee, to)));
                            }
                        }
                    } else if strict && oldest.is_some() {
                        in_pass[y] = true;
                        stuck[y] = true;
                    }
                }
            }
        }
        // Nothing is queued any more, or nobody is pending: stale marks are harmless.
    }

    /// The definition of the gross retry, implemented literally (see [`Self::retry_gross`]).
    fn retry_gross_reference(&mut self, t: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink) {
        let strict = self.cfg.discipline == QueueDiscipline::StrictFifoPerPayer;
        let mut stuck = vec![false; if strict { self.out.len() } else { 0 }];
        loop {
            let mut progressed = false;
            stuck.iter_mut().for_each(|b| *b = false);
            // Index loop: `deadlines` is not modified here, and copying a `Payment` out of it
            // ends the borrow before the queues are changed.
            for i in 0..self.deadlines.len() {
                let p = self.deadlines[i];
                // Still queued <=> head of its edge *now* (older payments of the edge were
                // visited earlier in this pass).
                if !self.is_head(&p) || (strict && stuck[p.payer as usize]) {
                    continue;
                }
                if ledger.transfer(p.payer, p.payee, p.amount) {
                    self.pop_head(p.payer, p.payee);
                    sink.settled(&p, t, SettleKind::Gross);
                    progressed = true;
                } else if strict {
                    stuck[p.payer as usize] = true;
                }
            }
            if !progressed || self.queued == 0 {
                break;
            }
        }
    }

    fn tick(&mut self, t: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink) {
        if self.queued == 0 {
            return;
        }
        if self.cfg.search_cycles {
            self.run_cycles(t, ledger, sink);
        }
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
        self.mark(p.payer); // one of its payments left the queue
        if ledger.transfer(p.payer, p.payee, p.amount) {
            sink.settled(&p, t, SettleKind::Gross);
            self.mark(p.payee);
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
        let strict = self.cfg.discipline == QueueDiscipline::StrictFifoPerPayer;
        if !self.cfg.search_cycles && self.cfg.mode == QueueMode::QueueOnShortfall {
            return format!("queue_{}_d{}", if strict { "fifo" } else { "bypass" }, self.cfg.max_delay_ms);
        }
        let f = if strict { "_fifo" } else { "" };
        format!("micro_cycle_{m}_w{}_d{}{f}", self.cfg.window_ms, self.cfg.max_delay_ms)
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
        let behind_queue = self.cfg.discipline == QueueDiscipline::StrictFifoPerPayer && !self.out[p.payer as usize].is_empty();
        if self.cfg.mode == QueueMode::QueueOnShortfall && !behind_queue && ledger.transfer(p.payer, p.payee, p.amount) {
            sink.settled(&p, p.t_arrival, SettleKind::Gross);
            self.mark(p.payee);
            return;
        }
        if self.cfg.max_delay_ms == 0 {
            // No waiting allowed: behaves like gross settlement.
            if ledger.transfer(p.payer, p.payee, p.amount) {
                sink.settled(&p, p.t_arrival, SettleKind::Gross);
                self.mark(p.payee);
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
        assert_eq!(self.queued, 0, "every queued payment settles or expires by its deadline");
    }

    fn queued(&self) -> usize {
        self.queued
    }

    fn stats(&self) -> Vec<(&'static str, f64)> {
        vec![("cycles_settled", self.cycles_settled as f64), ("cycles_failed", self.cycles_failed as f64)]
    }
}
