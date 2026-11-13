//! The common interface of all settlement engines, and the `GrossRealTime` engine.

use crate::ledger::Ledger;
use crate::types::{Payment, SettleKind, TimeMs};

/// Receives settlement events. Metrics collectors implement this.
pub trait Sink {
    fn settled(&mut self, p: &Payment, t: TimeMs, kind: SettleKind);
    fn rejected(&mut self, p: &Payment, t: TimeMs);
}

/// A settlement engine consumes payments in arrival order.
///
/// Driving protocol (see [`crate::drive`]):
/// 1. before each payment, `advance_to(p.t_arrival)` lets the engine fire internal timers
///    (netting cycles, windows, deadlines) due at or before that instant;
/// 2. `submit(p)`;
/// 3. at the end, `finish(t_end)` settles or rejects everything still queued.
///
/// `&mut dyn Sink` (a trait object) is used instead of a generic parameter so engines can
/// be stored together as `Box<dyn SettlementEngine>` and swept over in a single pass.
pub trait SettlementEngine {
    fn name(&self) -> String;
    fn advance_to(&mut self, t: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink);
    fn submit(&mut self, p: Payment, ledger: &mut Ledger, sink: &mut dyn Sink);
    fn finish(&mut self, t_end: TimeMs, ledger: &mut Ledger, sink: &mut dyn Sink);
    /// Number of payments currently waiting inside the engine.
    fn queued(&self) -> usize;
    /// Balances were changed from outside the engine (a funding policy topped accounts up).
    /// Engines that remember which participants are worth retrying must forget that here.
    /// Default: nothing to do.
    fn funded(&mut self) {}
    /// Engine-specific counters for reports (e.g. cycles settled). Default: none.
    fn stats(&self) -> Vec<(&'static str, f64)> {
        Vec::new()
    }
}

/// Settles every payment on arrival if the payer has funds; otherwise rejects it.
/// This is the Pix behaviour (SPI: gross, real time, no queue).
#[derive(Default, Clone, Debug)]
pub struct GrossRealTime;

impl SettlementEngine for GrossRealTime {
    fn name(&self) -> String {
        "gross_real_time".into()
    }
    fn advance_to(&mut self, _t: TimeMs, _l: &mut Ledger, _s: &mut dyn Sink) {}
    fn submit(&mut self, p: Payment, ledger: &mut Ledger, sink: &mut dyn Sink) {
        if ledger.transfer(p.payer, p.payee, p.amount) {
            sink.settled(&p, p.t_arrival, SettleKind::Gross);
        } else {
            sink.rejected(&p, p.t_arrival);
        }
    }
    fn finish(&mut self, _t: TimeMs, _l: &mut Ledger, _s: &mut dyn Sink) {}
    fn queued(&self) -> usize {
        0
    }
}
