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
    /// echo payments (see `ScenarioConfig::echo_share`); `None` when off
    echo: Option<Echo>,
}

/// A scheduled echo: (time ms, tie-break sequence, payer, payee, amount in cents).
type DueEcho = (TimeMs, u64, u32, u32, i64);

/// State of the echo mechanism. It has its own random stream, so switching echoes on does not
/// change which base payments are drawn for a given seed, only how many.
struct Echo {
    rng: Rng,
    /// probability that a base payment triggers an echo: share / (1 - share), so that echoes
    /// are `share` of all payments
    prob: f64,
    mean_delay_ms: f64,
    return_prob: f64,
    /// scheduled echoes, earliest first: (time ms, tie-break sequence, payer, payee, amount)
    due: std::collections::BinaryHeap<std::cmp::Reverse<DueEcho>>,
    seq: u64,
    /// the next base payment, drawn ahead so that echoes due before it can be emitted first
    ahead: Option<Payment>,
    base_done: bool,
    /// payments emitted so far (ids are assigned in arrival order, base and echo alike)
    emitted: u64,
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
        assert!((0.0..1.0).contains(&cfg.echo_share), "echo_share must be in [0, 1)");
        // Echoes are `echo_share` of all payments, so base arrivals are the rest: the expected
        // number of payments a day does not change.
        let base_per_day = cfg.payments_per_day * (1.0 - cfg.echo_share);
        let rate = cfg.hourly.iter().map(|w| base_per_day * w / hsum / HOUR_MS).collect();
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
            echo: (cfg.echo_share > 0.0).then(|| Echo {
                rng: Rng::derive(cfg.seed, 0xEC40),
                prob: cfg.echo_share / (1.0 - cfg.echo_share),
                mean_delay_ms: cfg.echo_delay_ms.max(1.0),
                return_prob: cfg.echo_return_prob,
                due: Default::default(),
                seq: 0,
                ahead: None,
                base_done: false,
                emitted: 0,
            }),
        }
    }

    pub fn config(&self) -> &ScenarioConfig {
        &self.cfg
    }
}

impl PaymentStream {
    /// Non-homogeneous Poisson process with piecewise-constant hourly rate: draw an
    /// exponential gap at the current hour's rate; if it crosses the hour boundary, jump to
    /// the boundary and redraw (exact, by memorylessness of the exponential).
    fn next_base(&mut self) -> Option<Payment> {
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
            let a = match &self.profiles {
                None => self.payer.sample(&mut self.rng),
                Some(p) => p.payer_by_hour[hour].sample(&mut self.rng),
            };
            let b = Self::draw_payee(&self.payee, &self.profiles, a, hour, &mut self.rng);
            let amount = self.cfg.amounts.sample_cents(&mut self.rng);
            let id = self.next_id;
            self.next_id += 1;
            return Some(Payment { id, t_arrival: self.t as TimeMs, payer: a as u32, payee: b as u32, amount });
        }
    }
}

impl PaymentStream {
    /// Payee of a payment sent by `a` in hour-of-day `hour`. An associated function (no
    /// `&self`) so the caller can pass either random stream while other fields are borrowed.
    fn draw_payee(payee: &[AliasTable], profiles: &Option<Profiles>, a: usize, hour: usize, rng: &mut Rng) -> usize {
        match profiles {
            None => payee[a].sample(rng),
            Some(p) => {
                let q: [f64; 4] = std::array::from_fn(|c| p.class_weight[a][c] * p.in_mult[c][hour]);
                let mut u = rng.f64() * q.iter().sum::<f64>();
                let mut c = 0;
                while c < 3 && (u >= q[c] || q[c] == 0.0) {
                    u -= q[c];
                    c += 1;
                }
                // floating-point guard: fall back to the last class with weight
                while p.within[a][c].is_none() {
                    c = (c + 3) % 4;
                }
                let k = p.within[a][c].as_ref().unwrap().sample(rng);
                p.members[c][k]
            }
        }
    }
}

impl Iterator for PaymentStream {
    type Item = Payment;

    /// Without echoes: the base process. With echoes: base payments and scheduled echoes merged
    /// in time order (a base payment first when both fall on the same millisecond); each base
    /// payment may schedule one echo, echoes do not echo.
    fn next(&mut self) -> Option<Payment> {
        if self.echo.is_none() {
            return self.next_base();
        }
        // Draw the next base payment ahead (this cannot borrow `self.echo` at the same time).
        let need = self.echo.as_ref().map(|e| e.ahead.is_none() && !e.base_done).unwrap_or(false);
        if need {
            let p = self.next_base();
            let e = self.echo.as_mut().unwrap();
            e.base_done = p.is_none();
            e.ahead = p;
        }
        let horizon = self.horizon as TimeMs;
        let e = self.echo.as_mut().unwrap();
        let echo_first = match (&e.ahead, e.due.peek()) {
            (Some(b), Some(std::cmp::Reverse(d))) => d.0 < b.t_arrival,
            (None, Some(_)) => true,
            _ => false,
        };
        let mut out = if echo_first {
            let std::cmp::Reverse((t, _, payer, payee, amount)) = e.due.pop().unwrap();
            Payment { id: 0, t_arrival: t, payer, payee, amount }
        } else {
            let p = e.ahead.take()?;
            // Does the payee respond to this receipt?
            if e.rng.f64() < e.prob {
                let t = p.t_arrival + e.rng.exp(1.0 / e.mean_delay_ms).ceil().max(1.0) as TimeMs;
                if t < horizon {
                    let from = p.payee as usize;
                    let to = if e.rng.f64() < e.return_prob {
                        p.payer as usize
                    } else {
                        let hour = ((t / HOUR_MS as TimeMs) % 24) as usize;
                        Self::draw_payee(&self.payee, &self.profiles, from, hour, &mut e.rng)
                    };
                    e.due.push(std::cmp::Reverse((t, e.seq, from as u32, to as u32, p.amount)));
                    e.seq += 1;
                }
            }
            p
        };
        out.id = e.emitted;
        e.emitted += 1;
        Some(out)
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
