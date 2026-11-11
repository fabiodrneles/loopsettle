//! # loopsettle-core
//!
//! Types, the cents ledger, the [`SettlementEngine`] trait, settlement engines and
//! streaming metrics. No dependencies.

pub mod engine;
pub mod engines;
pub mod json;
pub mod ledger;
pub mod metrics;
pub mod rng;
pub mod sys;
pub mod types;

pub use engine::{GrossRealTime, SettlementEngine, Sink};
pub use engines::{AtomicMicroCycle, DeferredNet, MicroCycleConfig, QueueDiscipline, QueueMode};
pub use ledger::{Ledger, UNLIMITED};
pub use metrics::{FlowStats, LogHistogram, RunMetrics, SLA_PIX_MS};
pub use types::*;

/// One engine together with its own ledger and metrics.
pub struct EngineRun {
    pub engine: Box<dyn SettlementEngine>,
    pub ledger: Ledger,
    pub metrics: RunMetrics,
}

impl EngineRun {
    pub fn new(engine: Box<dyn SettlementEngine>, ledger: Ledger) -> Self {
        let n = ledger.len();
        EngineRun { engine, ledger, metrics: RunMetrics::new(n) }
    }
}

/// Feeds an arrival-ordered payment stream to several engines in a single pass.
///
/// `stream` is any iterator (generic `I`, so the generator is inlined, no boxing), which is
/// what makes streaming possible: payments are produced, settled and dropped one at a time.
/// Panics if the stream is not sorted by arrival time.
pub fn drive<I>(stream: I, runs: &mut [EngineRun], flow: &mut FlowStats, t_end: TimeMs)
where
    I: IntoIterator<Item = Payment>,
{
    let mut last = 0;
    for p in stream {
        assert!(p.t_arrival >= last, "stream must be sorted by arrival time");
        assert!(p.amount > 0 && p.payer != p.payee, "invalid payment {p:?}");
        last = p.t_arrival;
        flow.observe(&p);
        for r in runs.iter_mut() {
            r.engine.advance_to(p.t_arrival, &mut r.ledger, &mut r.metrics);
            r.engine.submit(p, &mut r.ledger, &mut r.metrics);
        }
    }
    let t_end = t_end.max(last);
    for r in runs.iter_mut() {
        r.engine.advance_to(t_end, &mut r.ledger, &mut r.metrics);
        r.engine.finish(t_end, &mut r.ledger, &mut r.metrics);
    }
}
