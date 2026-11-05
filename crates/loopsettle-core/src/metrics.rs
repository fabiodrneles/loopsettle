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
        LogHistogram { counts: vec![0; 64 * SUB as usize], total: 0, max: 0, sum: 0 }
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
        if self.total == 0 { 0.0 } else { self.sum as f64 / self.total as f64 }
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
    /// Fraction of recorded values `<= v` (exact when `v` is a bucket lower bound or < 64).
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
            pair: if with_pairs { vec![0; n * n] } else { Vec::new() },
        }
    }

    pub fn observe(&mut self, p: &Payment) {
        let (a, b) = (p.payer as usize, p.payee as usize);
        self.count += 1;
        self.gross_value += p.amount as i128;
        self.out_value[a] += p.amount;
        self.in_value[b] += p.amount;
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
        let s: i128 = (0..self.n).map(|i| self.out_value[i].min(self.in_value[i]) as i128).sum();
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
}

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
        }
    }
    pub fn value_weighted_mean_delay_ms(&self) -> f64 {
        if self.settled_value == 0 { 0.0 } else { self.value_delay_sum / self.settled_value as f64 }
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
        self.value_delay_sum += p.amount as f64 * d as f64;
    }
    fn rejected(&mut self, p: &Payment, _t: TimeMs) {
        self.rejected_count += 1;
        self.rejected_value += p.amount as i128;
        self.rejected_value_by_payer[p.payer as usize] += p.amount;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            assert!(lb <= v && (v - lb) as f64 / v as f64 <= 1.0 / 64.0, "v={v} lb={lb}");
        }
    }
}
