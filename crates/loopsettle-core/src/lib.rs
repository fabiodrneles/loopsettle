//! # loopsettle-core
//!
//! Types, the cents ledger, the [`SettlementEngine`] trait, settlement engines and
//! streaming metrics. No dependencies.

pub mod engine;
pub mod engines;
pub mod funding;
pub mod json;
pub mod ledger;
pub mod metrics;
pub mod rng;
pub mod sys;
pub mod types;

pub use engine::{GrossRealTime, SettlementEngine, Sink};
pub use engines::{
    AtomicMicroCycle, DeferredNet, HoldToDeadline, MicroCycleConfig, QueueDiscipline, QueueMode,
};
pub use funding::{FundingPolicy, PrefundClosedWindows, TopUpToTarget};
pub use ledger::{Ledger, UNLIMITED};
pub use metrics::{pi, FlowStats, LogHistogram, RunMetrics, SLA_PIX_MS};
pub use types::*;

/// One engine together with its own ledger and metrics.
pub struct EngineRun {
    pub engine: Box<dyn SettlementEngine>,
    pub ledger: Ledger,
    pub metrics: RunMetrics,
    /// what participants do with their accounts over time (top-ups, prefunding); `None` = nothing
    pub funding: Option<Box<dyn FundingPolicy>>,
}

impl EngineRun {
    pub fn new(engine: Box<dyn SettlementEngine>, ledger: Ledger) -> Self {
        let n = ledger.len();
        EngineRun {
            engine,
            ledger,
            metrics: RunMetrics::new(n),
            funding: None,
        }
    }

    pub fn with_funding(mut self, policy: Box<dyn FundingPolicy>) -> Self {
        self.funding = Some(policy);
        self
    }

    /// Bring the run to time `t`: every funding instant due by `t` is applied in time order,
    /// each after the engine has processed its own events up to that instant.
    fn advance_to(&mut self, t: TimeMs) {
        while let Some(f) = self
            .funding
            .as_ref()
            .and_then(|p| p.next_instant())
            .filter(|&f| f <= t)
        {
            self.engine
                .advance_to(f, &mut self.ledger, &mut self.metrics);
            self.funding
                .as_mut()
                .expect("checked above")
                .apply(&mut self.ledger);
            self.engine.funded();
        }
        self.engine
            .advance_to(t, &mut self.ledger, &mut self.metrics);
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
            r.advance_to(p.t_arrival);
            r.engine.submit(p, &mut r.ledger, &mut r.metrics);
        }
    }
    let t_end = t_end.max(last);
    for r in runs.iter_mut() {
        r.advance_to(t_end);
        r.engine.finish(t_end, &mut r.ledger, &mut r.metrics);
    }
}

/// The examples in this crate's README are compiled and run as tests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
