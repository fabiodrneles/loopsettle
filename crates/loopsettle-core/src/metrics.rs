//! Streaming metrics: nothing here stores individual payments.
//!
//! * [`FlowStats`] looks at the *arrival* stream (engine independent): gross volume,
//!   liquidity upper bound (UB), lower bound (LB), the "potential" netting capacity and
//!   the aggregated pair matrix used by the topology crate.
//! * [`RunMetrics`] is a [`Sink`] attached to one engine: delays, rejections, how much
//!   value was settled gross / in cycles / deferred.

use crate::engine::Sink;
use crate::types::{Cents, ParticipantId, Payment, SettleKind, TimeMs};

// ---------------------------------------------------------------------------------------
// Log-linear histogram (HDR-style) for delay percentiles in O(1) memory.
// ---------------------------------------------------------------------------------------

const SUB_BITS: u32 = 6; // 64 sub-buckets per power of two -> relative error < 1.6 %
const SUB: u64 = 1 << SUB_BITS;

/// Histogram of non-negative integers (milliseconds). Values below 64 are exact; above,
/// each bucket spans at most 1/64 of its value. Percentiles return the bucket's lower
/// bound, so they are never overstated.
#[derive(Clone, Debug)]
pub struct LogHistogram {
    counts: Vec<u64>,
    total: u64,
    max: u64,
    sum: u128,
}

impl Default for LogHistogram {
    fn default() -> Self {
        LogHistogram {
            counts: vec![0; 64 * SUB as usize],
            total: 0,
            max: 0,
            sum: 0,
        }
    }
}

impl LogHistogram {
    fn index(v: u64) -> usize {
        if v < SUB {
            return v as usize;
        }
        let exp = 63 - v.leading_zeros(); // floor(log2 v) >= SUB_BITS
        let shift = exp - SUB_BITS;
        let mantissa = (v >> shift) - SUB; // 0..SUB
        ((exp - SUB_BITS + 1) as u64 * SUB + mantissa) as usize
    }
    fn lower_bound(i: usize) -> u64 {
        let i = i as u64;
        if i < SUB {
            return i;
        }
        let block = i / SUB; // >= 1
        let mantissa = i % SUB;
        (SUB + mantissa) << (block - 1)
    }
    pub fn record(&mut self, v: u64) {
        self.counts[Self::index(v)] += 1;
        self.total += 1;
        self.sum += v as u128;
        self.max = self.max.max(v);
    }
    pub fn count(&self) -> u64 {
        self.total
    }
    pub fn max(&self) -> u64 {
        self.max
    }
    pub fn mean(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.sum as f64 / self.total as f64
        }
    }
    /// `q` in [0, 1]. Returns 0 for an empty histogram.
    pub fn quantile(&self, q: f64) -> u64 {
        if self.total == 0 {
            return 0;
        }
        let rank = ((q * self.total as f64).ceil() as u64).clamp(1, self.total);
        let mut acc = 0;
        for (i, &c) in self.counts.iter().enumerate() {
            acc += c;
            if acc >= rank {
                return Self::lower_bound(i).min(self.max);
            }
        }
        self.max
    }
    /// Lower and upper bound of the `q` quantile: the true value lies in `[lo, hi]`
    /// (`hi - lo` is at most 1/64 of the value; both are equal below 64).
    pub fn quantile_bounds(&self, q: f64) -> (u64, u64) {
        if self.total == 0 {
            return (0, 0);
        }
        let rank = ((q * self.total as f64).ceil() as u64).clamp(1, self.total);
        let mut acc = 0;
        for (i, &c) in self.counts.iter().enumerate() {
            acc += c;
            if acc >= rank {
                let lo = Self::lower_bound(i).min(self.max);
                let hi = (Self::lower_bound(i + 1).saturating_sub(1))
                    .min(self.max)
                    .max(lo);
                return (lo, hi);
            }
        }
        (self.max, self.max)
    }
    /// Fraction of recorded values `<= v`. **Approximate** above 63: it counts the whole
    /// bucket that contains `v` (up to 1/64 too far). For a regulatory threshold use the exact
    /// counters of [`RunMetrics::share_within_ms`]. (Exact when `v` is a bucket lower bound or < 64.)
    pub fn fraction_at_most(&self, v: u64) -> f64 {
        if self.total == 0 {
            return 1.0;
        }
        let last = Self::index(v);
        let n: u64 = self.counts[..=last].iter().sum();
        n as f64 / self.total as f64
    }
}

// ---------------------------------------------------------------------------------------
// Arrival-stream statistics.
// ---------------------------------------------------------------------------------------

/// Engine-independent statistics of the payment stream.
#[derive(Clone, Debug)]
pub struct FlowStats {
    pub n: usize,
    pub count: u64,
    pub gross_value: i128,
    pub out_value: Vec<Cents>,
    pub in_value: Vec<Cents>,
    /// running (in - out) per participant
    cum_net: Vec<Cents>,
    /// lowest running (in - out) seen
    min_cum_net: Vec<Cents>,
    /// gross aggregated pair matrix, row-major `pair[payer * n + payee]`; empty if disabled
    pair: Vec<Cents>,
}

impl FlowStats {
    /// `with_pairs` keeps an `n x n` matrix of aggregated flows (8 n^2 bytes).
    pub fn new(n: usize, with_pairs: bool) -> Self {
        FlowStats {
            n,
            count: 0,
            gross_value: 0,
            out_value: vec![0; n],
            in_value: vec![0; n],
            cum_net: vec![0; n],
            min_cum_net: vec![0; n],
            pair: if with_pairs {
                vec![0; n * n]
            } else {
                Vec::new()
            },
        }
    }

    pub fn observe(&mut self, p: &Payment) {
        let (a, b) = (p.payer as usize, p.payee as usize);
        self.count += 1;
        self.gross_value += p.amount as i128;
        self.out_value[a] = self.out_value[a]
            .checked_add(p.amount)
            .expect("outflow overflows i64 cents");
        self.in_value[b] = self.in_value[b]
            .checked_add(p.amount)
            .expect("inflow overflows i64 cents");
        self.cum_net[a] -= p.amount;
        self.cum_net[b] += p.amount;
        if self.cum_net[a] < self.min_cum_net[a] {
            self.min_cum_net[a] = self.cum_net[a];
        }
        if !self.pair.is_empty() {
            self.pair[a * self.n + b] += p.amount;
        }
    }

    /// Final multilateral net position (in - out) per participant.
    pub fn net_position(&self, p: ParticipantId) -> Cents {
        self.cum_net[p as usize]
    }
    /// **UB**: smallest opening balance that avoids any rejection under `GrossRealTime`
    /// = max over time of (cumulative out - cumulative in), floored at zero.
    pub fn ub(&self, p: ParticipantId) -> Cents {
        -self.min_cum_net[p as usize]
    }
    /// **LB**: end-of-period multilateral net debit (what a single deferred net cycle needs).
    pub fn lb(&self, p: ParticipantId) -> Cents {
        (-self.cum_net[p as usize]).max(0)
    }
    pub fn ub_total(&self) -> i128 {
        (0..self.n as u32).map(|p| self.ub(p) as i128).sum()
    }
    pub fn lb_total(&self) -> i128 {
        (0..self.n as u32).map(|p| self.lb(p) as i128).sum()
    }
    pub fn ubs(&self) -> Vec<Cents> {
        (0..self.n as u32).map(|p| self.ub(p)).collect()
    }
    /// "Potential" netting capacity in the style of the BCB Pix study: per participant,
    /// the part of its outflows that same-period inflows could fund, **ignoring the order
    /// of payments**: `sum_i min(out_i, in_i) / gross`.
    pub fn potential_offset_fraction(&self) -> f64 {
        if self.gross_value == 0 {
            return 0.0;
        }
        let s: i128 = (0..self.n)
            .map(|i| self.out_value[i].min(self.in_value[i]) as i128)
            .sum();
        s as f64 / self.gross_value as f64
    }
    /// Aggregated pair flow `payer -> payee` (requires `with_pairs`).
    pub fn pair(&self, payer: usize, payee: usize) -> Cents {
        self.pair[payer * self.n + payee]
    }
    pub fn has_pairs(&self) -> bool {
        !self.pair.is_empty()
    }
}

// ---------------------------------------------------------------------------------------
// Per-engine metrics.
// ---------------------------------------------------------------------------------------

/// Metrics of one engine run. Implements [`Sink`].
#[derive(Clone, Debug)]
pub struct RunMetrics {
    pub settled_count: u64,
    pub settled_value: i128,
    pub value_by_kind: [i128; 3],
    pub count_by_kind: [u64; 3],
    pub rejected_count: u64,
    pub rejected_value: i128,
    pub rejected_value_by_payer: Vec<Cents>,
    /// delay (settlement time - arrival time) of settled payments, in ms
    pub delay: LogHistogram,
    /// value-weighted delay sum (cents * ms), for the value-weighted mean delay
    pub value_delay_sum: f64,
    /// delay thresholds (ms) counted exactly; default `[2_800]`, the Pix 50% target
    pub sla_thresholds_ms: Vec<TimeMs>,
    /// number of settled payments with delay `<=` each threshold
    pub sla_counts: Vec<u64>,
    /// exact sums for the settlement delay indicator rho (cents * ms): value times settlement
    /// time and value times arrival time of settled payments, value times arrival time of
    /// rejected ones
    pub value_settle_time_sum: i128,
    pub value_arrival_sum_settled: i128,
    pub value_arrival_sum_rejected: i128,
}

/// The Pix "Manual de Tempos" target used across the reports: 50% settled within 2.8 s.
pub const SLA_PIX_MS: TimeMs = 2_800;

pub fn kind_index(k: SettleKind) -> usize {
    match k {
        SettleKind::Gross => 0,
        SettleKind::Cycle => 1,
        SettleKind::Deferred => 2,
    }
}

impl RunMetrics {
    pub fn new(n: usize) -> Self {
        RunMetrics {
            settled_count: 0,
            settled_value: 0,
            value_by_kind: [0; 3],
            count_by_kind: [0; 3],
            rejected_count: 0,
            rejected_value: 0,
            rejected_value_by_payer: vec![0; n],
            delay: LogHistogram::default(),
            value_delay_sum: 0.0,
            sla_thresholds_ms: vec![SLA_PIX_MS],
            sla_counts: vec![0],
            value_settle_time_sum: 0,
            value_arrival_sum_settled: 0,
            value_arrival_sum_rejected: 0,
        }
    }
    /// Same, with other exact delay thresholds.
    pub fn with_sla_thresholds(n: usize, thresholds_ms: &[TimeMs]) -> Self {
        let mut m = Self::new(n);
        m.sla_thresholds_ms = thresholds_ms.to_vec();
        m.sla_counts = vec![0; thresholds_ms.len()];
        m
    }
    /// Exact share of settled payments with delay `<= ms`; `None` if `ms` is not one of the
    /// configured thresholds. 1.0 when nothing settled (as `fraction_at_most`).
    pub fn share_within_ms(&self, ms: TimeMs) -> Option<f64> {
        let k = self.sla_thresholds_ms.iter().position(|&t| t == ms)?;
        Some(if self.settled_count == 0 {
            1.0
        } else {
            self.sla_counts[k] as f64 / self.settled_count as f64
        })
    }
    /// Settlement delay indicator **rho** of Koponen & Soramaki (1998, section 6.2, equation
    /// 5), in continuous time: the integral over the day of the value waiting in the queue,
    /// divided by the integral of the cumulative value submitted. 0 = everything settles on
    /// arrival; 1 = nothing settles before `t_end`. A rejected payment counts as waiting until
    /// `t_end` (it never settles; the 1998 framework has no rejections). Value-weighted over
    /// payers, as the system rho of the study. 0 when nothing was submitted.
    pub fn rho(&self, t_end: TimeMs) -> f64 {
        let t = t_end as i128;
        let waiting = self.value_settle_time_sum - self.value_arrival_sum_settled
            + t * self.rejected_value
            - self.value_arrival_sum_rejected;
        let submitted = t * (self.settled_value + self.rejected_value)
            - self.value_arrival_sum_settled
            - self.value_arrival_sum_rejected;
        if submitted <= 0 {
            0.0
        } else {
            waiting as f64 / submitted as f64
        }
    }
    pub fn value_weighted_mean_delay_ms(&self) -> f64 {
        if self.settled_value == 0 {
            0.0
        } else {
            self.value_delay_sum / self.settled_value as f64
        }
    }
}

impl Sink for RunMetrics {
    fn settled(&mut self, p: &Payment, t: TimeMs, kind: SettleKind) {
        debug_assert!(t >= p.t_arrival);
        let d = t - p.t_arrival;
        self.settled_count += 1;
        self.settled_value += p.amount as i128;
        self.value_by_kind[kind_index(kind)] += p.amount as i128;
        self.count_by_kind[kind_index(kind)] += 1;
        self.delay.record(d);
        for (k, &th) in self.sla_thresholds_ms.iter().enumerate() {
            if d <= th {
                self.sla_counts[k] += 1;
            }
        }
        self.value_delay_sum += p.amount as f64 * d as f64;
        self.value_settle_time_sum += p.amount as i128 * t as i128;
        self.value_arrival_sum_settled += p.amount as i128 * p.t_arrival as i128;
    }
    fn rejected(&mut self, p: &Payment, _t: TimeMs) {
        self.rejected_count += 1;
        self.rejected_value += p.amount as i128;
        self.rejected_value_by_payer[p.payer as usize] += p.amount;
        self.value_arrival_sum_rejected += p.amount as i128 * p.t_arrival as i128;
    }
}

/// Liquidity usage indicator **pi** of Koponen & Soramaki (1998, section 6.3, equation 6):
/// liquidity used (sum of peak drawdowns) over the gross value of payments sent. 0 = no
/// liquidity from outside the system; 1 = as much as the gross value. 0 when nothing was sent.
pub fn pi(liquidity_used: i128, gross_value: i128) -> f64 {
    if gross_value <= 0 {
        0.0
    } else {
        liquidity_used as f64 / gross_value as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pay(t: TimeMs, v: Cents) -> Payment {
        Payment {
            id: 0,
            t_arrival: t,
            payer: 0,
            payee: 1,
            amount: v,
        }
    }

    /// rho by hand. Day of 100 ms. A payment of 10 arrives at 0 and settles at 50: it waits
    /// 10*50 = 500 out of a submitted area of 10*100 = 1000, so rho = 0.5. Immediate
    /// settlement gives 0, rejection (never settled) gives 1.
    #[test]
    fn rho_by_hand() {
        let mut m = RunMetrics::new(2);
        m.settled(&pay(0, 10), 50, SettleKind::Gross);
        assert_eq!(m.rho(100), 0.5);
        let mut m = RunMetrics::new(2);
        m.settled(&pay(20, 10), 20, SettleKind::Gross);
        assert_eq!(m.rho(100), 0.0);
        let mut m = RunMetrics::new(2);
        m.rejected(&pay(20, 10), 30);
        assert_eq!(m.rho(100), 1.0);
        // value-weighted: 10 waits half its time, 30 none -> 500 / (1000 + 3000)
        let mut m = RunMetrics::new(2);
        m.settled(&pay(0, 10), 50, SettleKind::Gross);
        m.settled(&pay(0, 30), 0, SettleKind::Gross);
        assert_eq!(m.rho(100), 0.125);
        assert_eq!(RunMetrics::new(2).rho(100), 0.0);
        assert_eq!(pi(25, 100), 0.25);
    }

    #[test]
    fn histogram_small_values_exact() {
        let mut h = LogHistogram::default();
        for v in 0..50 {
            h.record(v);
        }
        assert_eq!(h.quantile(0.5), 24);
        assert_eq!(h.quantile(1.0), 49);
    }

    #[test]
    fn histogram_relative_error_bounded() {
        for &v in &[64u64, 100, 1_000, 2_800, 12_345, 86_400_000] {
            let lb = LogHistogram::lower_bound(LogHistogram::index(v));
            assert!(
                lb <= v && (v - lb) as f64 / v as f64 <= 1.0 / 64.0,
                "v={v} lb={lb}"
            );
        }
    }
}
