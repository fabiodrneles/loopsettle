//! Settlement-account ledger in cents.
//!
//! The ledger enforces the "no negative balance" rule for every operation except
//! [`Ledger::force_batch`], which is used only by deferred net settlement (that design
//! assumes participants pre-fund or collateralise their net debit, see DECISIONS.md).

use crate::types::{Cents, ParticipantId};

/// "Unlimited" opening balance used to *measure* liquidity needs: with this much money no
/// payment is ever refused, and the liquidity an engine needed is `opening - min_balance`.
/// 2^50 cents ~ R$ 11 trillion per participant; thousands of participants still fit in `i64`.
pub const UNLIMITED: Cents = 1 << 50;

#[derive(Clone, Debug)]
pub struct Ledger {
    opening: Vec<Cents>,
    balance: Vec<Cents>,
    min_balance: Vec<Cents>,
    /// net funds moved in from outside the system (reserves) per participant; see [`Ledger::fund`]
    external: Vec<Cents>,
    /// intraday credit line per participant: the balance may go down to `-credit_limit`
    credit_limit: Vec<Cents>,
}

impl Ledger {
    pub fn new(opening: Vec<Cents>) -> Self {
        assert!(opening.iter().all(|&b| b >= 0), "opening balances must be >= 0");
        let n = opening.len();
        Ledger { min_balance: opening.clone(), balance: opening.clone(), opening, external: vec![0; n], credit_limit: vec![0; n] }
    }

    /// Every participant starts with [`UNLIMITED`] liquidity (measurement mode).
    pub fn unlimited(n: usize) -> Self {
        Self::new(vec![UNLIMITED; n])
    }

    pub fn len(&self) -> usize {
        self.balance.len()
    }
    pub fn is_empty(&self) -> bool {
        self.balance.is_empty()
    }
    pub fn balance(&self, p: ParticipantId) -> Cents {
        self.balance[p as usize]
    }
    pub fn balances(&self) -> &[Cents] {
        &self.balance
    }
    pub fn opening(&self) -> &[Cents] {
        &self.opening
    }
    /// Sum of all balances (constant by construction: invariant 1).
    pub fn total(&self) -> i128 {
        self.balance.iter().map(|&b| b as i128).sum()
    }
    /// Liquidity actually consumed by participant `p`: opening balance minus the lowest
    /// balance it ever reached (peak drawdown).
    pub fn peak_drawdown(&self, p: ParticipantId) -> Cents {
        self.opening[p as usize] - self.min_balance[p as usize]
    }
    pub fn peak_drawdowns(&self) -> Vec<Cents> {
        (0..self.len() as u32).map(|p| self.peak_drawdown(p)).collect()
    }

    pub fn can_pay(&self, p: ParticipantId, amount: Cents) -> bool {
        self.balance[p as usize] + self.credit_limit[p as usize] >= amount
    }

    /// Give every participant an intraday credit line (specs/05, M5): its balance may go down
    /// to `-limits[p]`. With all limits at zero (the default) no balance is ever negative.
    pub fn with_credit_limits(mut self, limits: Vec<Cents>) -> Self {
        assert!(limits.len() == self.len() && limits.iter().all(|&l| l >= 0), "one non-negative limit per participant");
        self.credit_limit = limits;
        self
    }
    pub fn credit_limit(&self, p: ParticipantId) -> Cents {
        self.credit_limit[p as usize]
    }
    /// Largest overdraft `p` ever had (0 if its balance never went below zero).
    pub fn max_credit_used(&self, p: ParticipantId) -> Cents {
        (-self.min_balance[p as usize]).max(0)
    }

    /// Move `delta` cents between the participant's reserves, outside the system, and its
    /// settlement account (`delta > 0`: top-up; `delta < 0`: withdrawal). Money enters or
    /// leaves the system, so conservation holds for `total() - external_total()`
    /// ([`Ledger::total_with_reserves`]). A withdrawal may not take the balance below zero.
    pub fn fund(&mut self, p: ParticipantId, delta: Cents) {
        let i = p as usize;
        assert!(self.balance[i] + delta >= 0 || delta >= 0, "a withdrawal cannot overdraw the account");
        self.external[i] = self.external[i].checked_add(delta).expect("external funding overflows i64 cents");
        self.apply(p, delta);
    }
    /// Net funds `p` has brought in from its reserves so far.
    pub fn external(&self, p: ParticipantId) -> Cents {
        self.external[p as usize]
    }
    pub fn external_total(&self) -> i128 {
        self.external.iter().map(|&b| b as i128).sum()
    }
    /// Balances minus external funding: constant whatever the engines and funding policies do
    /// (invariant 1 extended to accounts plus reserves).
    pub fn total_with_reserves(&self) -> i128 {
        self.total() - self.external_total()
    }

    /// Gross transfer; refuses (returns `false`, no change) if the payer lacks funds.
    pub fn transfer(&mut self, payer: ParticipantId, payee: ParticipantId, amount: Cents) -> bool {
        debug_assert!(amount > 0);
        if !self.can_pay(payer, amount) {
            return false;
        }
        self.apply(payer, -amount);
        self.apply(payee, amount);
        true
    }

    /// Atomic batch of transfers `(payer, payee, amount)`: either all are applied or none.
    /// Feasibility is checked on the *net* effect per participant, which is what an atomic
    /// (simultaneous) settlement needs. `scratch` must hold `len()` zeros and is returned
    /// zeroed; passing it in avoids one allocation per call in hot loops.
    pub fn try_batch(&mut self, legs: &[(ParticipantId, ParticipantId, Cents)], scratch: &mut [Cents]) -> bool {
        for &(a, b, v) in legs {
            scratch[a as usize] -= v;
            scratch[b as usize] += v;
        }
        let ok = legs.iter().all(|&(a, _, _)| self.balance[a as usize] + scratch[a as usize] + self.credit_limit[a as usize] >= 0);
        for &(a, b, _) in legs {
            for p in [a, b] {
                let d = scratch[p as usize];
                if d != 0 {
                    if ok {
                        self.apply(p, d);
                    }
                    scratch[p as usize] = 0;
                }
            }
        }
        ok
    }

    /// Apply net deltas unconditionally (deferred net settlement only). `deltas` must sum to 0.
    pub fn force_batch(&mut self, deltas: &[Cents]) {
        debug_assert_eq!(deltas.iter().map(|&d| d as i128).sum::<i128>(), 0);
        for (p, &d) in deltas.iter().enumerate() {
            if d != 0 {
                self.apply(p as u32, d);
            }
        }
    }

    fn apply(&mut self, p: ParticipantId, delta: Cents) {
        let i = p as usize;
        // Checked: a silent wrap-around on real data would corrupt every result after it.
        self.balance[i] = self.balance[i].checked_add(delta).expect("balance overflows i64 cents");
        if self.balance[i] < self.min_balance[i] {
            self.min_balance[i] = self.balance[i];
        }
    }
}
