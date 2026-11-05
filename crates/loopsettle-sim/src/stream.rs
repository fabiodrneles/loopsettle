//! Streaming payment generator: an `Iterator<Item = Payment>` in arrival order.
//! Memory is O(n^2) for the payee alias tables, independent of the number of payments.

use crate::scenario::{Population, ScenarioConfig};
use loopsettle_core::rng::{AliasTable, Rng};
use loopsettle_core::{Payment, TimeMs};

const HOUR_MS: f64 = 3_600_000.0;

/// Class-specific intraday profiles (W2). Payee drawn in two exact stages: first the payee's
/// class with weight `W[payer][class] * in_mult[class][hour]`, then the participant inside that
/// class with the time-independent weights. Memory stays O(n^2), as without profiles.
struct Profiles {
    payer_by_hour: Vec<AliasTable>,
    in_mult: [[f64; 24]; 4],
    members: [Vec<usize>; 4],
    /// per payer: total base weight towards each class
    class_weight: Vec<[f64; 4]>,
    /// per payer, per class: alias table over `members[class]` (None if all weights are zero)
    within: Vec<[Option<AliasTable>; 4]>,
}

pub struct PaymentStream {
    cfg: ScenarioConfig,
    rng: Rng,
    payer: AliasTable,
    payee: Vec<AliasTable>,
    profiles: Option<Profiles>,
    /// arrival rate per ms for each hour of the day
    rate: Vec<f64>,
    t: f64,
    horizon: f64,
    next_id: u64,
}

impl PaymentStream {
    pub fn new(cfg: &ScenarioConfig) -> Self {
        let pop = Population::build(cfg);
        Self::with_population(cfg, &pop)
    }

    pub fn with_population(cfg: &ScenarioConfig, pop: &Population) -> Self {
        let n = pop.len();
        let payer = AliasTable::new(&pop.activity);
        let base = |i: usize, j: usize| -> f64 {
            if i == j {
                return 0.0;
            }
            let same = if pop.class[i] == pop.class[j] { 1.0 + cfg.homophily } else { 1.0 };
            pop.activity[j].powf(cfg.payee_size_exp) * pop.receiver_bias[j] * same
        };
        let (payee, profiles) = if cfg.profile_strength > 0.0 {
            let out_mult = cfg.class_multipliers(true);
            let in_mult = cfg.class_multipliers(false);
            let ci: Vec<usize> = pop.class.iter().map(|c| c.index()).collect();
            let payer_by_hour = (0..24)
                .map(|h| AliasTable::new(&(0..n).map(|i| pop.activity[i] * out_mult[ci[i]][h]).collect::<Vec<_>>()))
                .collect();
            let mut members: [Vec<usize>; 4] = Default::default();
            for (j, &c) in ci.iter().enumerate() {
                members[c].push(j);
            }
            let mut class_weight = Vec::with_capacity(n);
            let mut within = Vec::with_capacity(n);
            for i in 0..n {
                let mut cw = [0.0; 4];
                let mut wt: [Option<AliasTable>; 4] = Default::default();
                for c in 0..4 {
                    let w: Vec<f64> = members[c].iter().map(|&j| base(i, j)).collect();
                    cw[c] = w.iter().sum();
                    if cw[c] > 0.0 {
                        wt[c] = Some(AliasTable::new(&w));
                    }
                }
                class_weight.push(cw);
                within.push(wt);
            }
            (Vec::new(), Some(Profiles { payer_by_hour, in_mult, members, class_weight, within }))
        } else {
            let payee = (0..n).map(|i| AliasTable::new(&(0..n).map(|j| base(i, j)).collect::<Vec<_>>())).collect();
            (payee, None)
        };
        assert_eq!(cfg.hourly.len(), 24, "hourly profile needs 24 weights");
        let hsum: f64 = cfg.hourly.iter().sum();
        let rate = cfg.hourly.iter().map(|w| cfg.payments_per_day * w / hsum / HOUR_MS).collect();
        PaymentStream {
            cfg: cfg.clone(),
            rng: Rng::derive(cfg.seed, 0x57AE),
            payer,
            payee,
            profiles,
            rate,
            t: 0.0,
            horizon: cfg.horizon_ms() as f64,
            next_id: 0,
        }
    }

    pub fn config(&self) -> &ScenarioConfig {
        &self.cfg
    }
}

impl Iterator for PaymentStream {
    type Item = Payment;

    /// Non-homogeneous Poisson process with piecewise-constant hourly rate: draw an
    /// exponential gap at the current hour's rate; if it crosses the hour boundary, jump to
    /// the boundary and redraw (exact, by memorylessness of the exponential).
    fn next(&mut self) -> Option<Payment> {
        loop {
            if self.t >= self.horizon {
                return None;
            }
            let hour = ((self.t / HOUR_MS) as usize) % 24;
            let boundary = ((self.t / HOUR_MS).floor() + 1.0) * HOUR_MS;
            let day = (self.t / (24.0 * HOUR_MS)) as usize;
            let r = self.rate[hour] * self.cfg.day_weights.get(day).copied().unwrap_or(1.0);
            let gap = if r > 0.0 { self.rng.exp(r) } else { f64::INFINITY };
            if self.t + gap >= boundary {
                self.t = boundary;
                continue;
            }
            self.t += gap;
            if self.t >= self.horizon {
                return None;
            }
            let (a, b) = match &self.profiles {
                None => {
                    let a = self.payer.sample(&mut self.rng);
                    (a, self.payee[a].sample(&mut self.rng))
                }
                Some(p) => {
                    let a = p.payer_by_hour[hour].sample(&mut self.rng);
                    let q: [f64; 4] = std::array::from_fn(|c| p.class_weight[a][c] * p.in_mult[c][hour]);
                    let mut u = self.rng.f64() * q.iter().sum::<f64>();
                    let mut c = 0;
                    while c < 3 && (u >= q[c] || q[c] == 0.0) {
                        u -= q[c];
                        c += 1;
                    }
                    // floating-point guard: fall back to the last class with weight
                    while p.within[a][c].is_none() {
                        c = (c + 3) % 4;
                    }
                    let k = p.within[a][c].as_ref().unwrap().sample(&mut self.rng);
                    (a, p.members[c][k])
                }
            };
            let amount = self.cfg.amounts.sample_cents(&mut self.rng);
            let id = self.next_id;
            self.next_id += 1;
            return Some(Payment { id, t_arrival: self.t as TimeMs, payer: a as u32, payee: b as u32, amount });
        }
    }
}

/// Stress scenario: participant `who` is offline in `[start, start + duration)`. Payments
/// it sends or receives in that window cannot be processed (in Pix the receiving PSP must
/// confirm the credit) and are retried in one burst when it comes back (ASSUMPTION: every
/// such payment is retried, none is abandoned).
pub struct Outage<I: Iterator<Item = Payment>> {
    inner: std::iter::Peekable<I>,
    who: u32,
    start: TimeMs,
    end: TimeMs,
    buffer: std::collections::VecDeque<Payment>,
    /// payments deferred by the outage and the sum of their deferral (ms)
    pub deferred_count: u64,
    pub deferred_value: i128,
    pub deferral_ms_sum: u128,
}

impl<I: Iterator<Item = Payment>> Outage<I> {
    pub fn new(inner: I, who: u32, start: TimeMs, duration: TimeMs) -> Self {
        Outage {
            inner: inner.peekable(),
            who,
            start,
            end: start + duration,
            buffer: Default::default(),
            deferred_count: 0,
            deferred_value: 0,
            deferral_ms_sum: 0,
        }
    }
}

impl<I: Iterator<Item = Payment>> Iterator for Outage<I> {
    type Item = Payment;
    fn next(&mut self) -> Option<Payment> {
        loop {
            // Release the burst once the stream reaches the end of the outage.
            let reached_end = match self.inner.peek() {
                Some(p) => p.t_arrival >= self.end,
                None => true,
            };
            if reached_end {
                if let Some(mut p) = self.buffer.pop_front() {
                    self.deferral_ms_sum += (self.end - p.t_arrival) as u128;
                    p.t_arrival = self.end;
                    return Some(p);
                }
            }
            let p = self.inner.next()?;
            let involved = p.payer == self.who || p.payee == self.who;
            if involved && p.t_arrival >= self.start && p.t_arrival < self.end {
                self.deferred_count += 1;
                self.deferred_value += p.amount as i128;
                self.buffer.push_back(p);
                continue;
            }
            return Some(p);
        }
    }
}
