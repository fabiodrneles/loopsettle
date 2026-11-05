//! Basic domain types. Money is always integer cents (`i64`), never floating point.

/// Index of a participant (an institution with a settlement account, like a Pix "Conta PI").
pub type ParticipantId = u32;
/// Money in cents.
pub type Cents = i64;
/// Simulation time in milliseconds since the start of the run.
pub type TimeMs = u64;

pub const MS_PER_SECOND: TimeMs = 1_000;
pub const MS_PER_DAY: TimeMs = 86_400_000;

/// A payment instruction `(id, t_arrival, payer, payee, amount)`.
///
/// `Copy` is derived because the struct is 32 bytes of plain data: copying it is cheaper
/// and simpler than juggling references through queues.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Payment {
    pub id: u64,
    pub t_arrival: TimeMs,
    pub payer: ParticipantId,
    pub payee: ParticipantId,
    pub amount: Cents,
}

/// How a payment was settled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettleKind {
    /// Settled individually and gross (real-time gross settlement).
    Gross,
    /// Settled inside an atomic cycle (micro-cycle netting).
    Cycle,
    /// Settled in a deferred multilateral net batch.
    Deferred,
}
