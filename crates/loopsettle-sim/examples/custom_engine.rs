//! How to test your own settlement mechanism with LoopSettle.
//!
//! ```text
//! cargo run --release -p loopsettle-sim --example custom_engine
//! ```
//!
//! 1. Implement [`SettlementEngine`] (four methods).
//! 2. Run it next to the built-in engines on the same payments with [`drive`].
//! 3. Compare the liquidity it needs with the liquidity floor: no mechanism with the same
//!    maximum delay can need less (docs/METHODOLOGY.md, section 9b).
//!
//! The example mechanism, `ReceiptReactive`, holds each payment until its payer has received at
//! least as much as it has paid (or until the deadline), a simple "pay out of receipts" rule.

use loopsettle_core::*;
use loopsettle_sim::bound::liquidity_floors;
use loopsettle_sim::{PaymentStream, ScenarioConfig};
use std::collections::VecDeque;

struct ReceiptReactive {
    max_delay_ms: TimeMs,
    /// per participant: payments waiting, oldest first
    waiting: Vec<VecDeque<Payment>>,
    /// per participant: value received minus value paid so far
    net: Vec<Cents>,
    /// all waiting payments in arrival order, to find the next deadline
    deadlines: VecDeque<Payment>,
    queued: usize,
}

impl ReceiptReactive {
    fn new(max_delay_ms: TimeMs, n: usize) -> Self {
        ReceiptReactive {
            max_delay_ms,
            waiting: vec![VecDeque::new(); n],
            net: vec![0; n],
            deadlines: VecDeque::new(),
            queued: 0,
        }
    }
    fn settle(&mut self, p: Payment, t: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink) {
        if ledger.transfer(p.payer, p.payee, p.amount) {
            self.net[p.payer as usize] -= p.amount;
            self.net[p.payee as usize] += p.amount;
            sink.settled(&p, t, SettleKind::Gross);
            self.release(p.payee, t, ledger, sink); // the payee may now pay out of this receipt
        } else {
            sink.rejected(&p, t);
        }
    }
    /// Pay `who`'s waiting payments, oldest first, while receipts cover them.
    fn release(&mut self, who: ParticipantId, t: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink) {
        while let Some(p) = self.waiting[who as usize].front().copied() {
            if self.net[who as usize] < p.amount {
                break;
            }
            self.waiting[who as usize].pop_front();
            self.queued -= 1;
            self.settle(p, t, ledger, sink);
        }
    }
}

impl SettlementEngine for ReceiptReactive {
    fn name(&self) -> String {
        format!("receipt_reactive_d{}", self.max_delay_ms)
    }
    fn advance_to(&mut self, t: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink) {
        // Deadlines: a payment still waiting when its time is up is paid anyway.
        while let Some(p) = self.deadlines.front().copied() {
            if p.t_arrival + self.max_delay_ms > t {
                break;
            }
            self.deadlines.pop_front();
            let q = &mut self.waiting[p.payer as usize];
            if q.front().map(|h| h.id) == Some(p.id) {
                q.pop_front();
                self.queued -= 1;
                self.settle(p, p.t_arrival + self.max_delay_ms, ledger, sink);
            }
        }
    }
    fn submit(&mut self, p: Payment, ledger: &mut Ledger, sink: &mut dyn Sink) {
        if self.waiting[p.payer as usize].is_empty() && self.net[p.payer as usize] >= p.amount {
            self.settle(p, p.t_arrival, ledger, sink);
        } else {
            self.waiting[p.payer as usize].push_back(p);
            self.deadlines.push_back(p);
            self.queued += 1;
        }
    }
    fn finish(&mut self, _t: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink) {
        if let Some(last) = self
            .deadlines
            .back()
            .map(|p| p.t_arrival + self.max_delay_ms)
        {
            self.advance_to(last, ledger, sink);
        }
    }
    fn queued(&self) -> usize {
        self.queued
    }
}

fn main() {
    let cfg = ScenarioConfig {
        seed: 1,
        participants: 30,
        payments_per_day: 100_000.0,
        n_big: 3,
        n_mid: 5,
        ..Default::default()
    };
    let n = cfg.participants;
    let d = 10_000;
    let mut runs = vec![
        EngineRun::new(Box::new(GrossRealTime), Ledger::unlimited(n)),
        EngineRun::new(Box::new(ReceiptReactive::new(d, n)), Ledger::unlimited(n)),
        EngineRun::new(
            Box::new(DeferredNet::new((MS_PER_DAY / d) as u32, n)),
            Ledger::unlimited(n),
        ),
    ];
    let mut flow = FlowStats::new(n, false);
    drive(
        PaymentStream::new(&cfg),
        &mut runs,
        &mut flow,
        cfg.horizon_ms(),
    );

    let ub = flow.ub_total();
    let floor: i128 = liquidity_floors(&cfg, &[d])[0]
        .iter()
        .map(|&v| v as i128)
        .sum();
    println!(
        "payments {}   UB {:.2}   liquidity floor for a {} s deadline {:.2} ({:.4}% below UB)",
        flow.count,
        ub as f64 / 100.0,
        d / 1000,
        floor as f64 / 100.0,
        100.0 * (ub - floor) as f64 / ub as f64
    );
    for r in &runs {
        let need: i128 = r.ledger.peak_drawdowns().iter().map(|&x| x as i128).sum();
        assert!(
            need >= floor,
            "no engine with this deadline can need less than the floor"
        );
        println!(
            "{:28} liquidity {:>14.2}  saving {:>8.4}% of UB  p95 delay {:>6} ms  rejected {}",
            r.engine.name(),
            need as f64 / 100.0,
            100.0 * (ub - need) as f64 / ub as f64,
            r.metrics.delay.quantile(0.95),
            r.metrics.rejected_count
        );
    }
}
