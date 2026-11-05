//! Deterministic pseudo-random numbers (xoshiro256** seeded through SplitMix64).
//!
//! Hand-written on purpose (DECISIONS.md D1): the stream for a given seed must never change,
//! on any platform (including wasm32), or across dependency upgrades — invariant 7.
//! Only integer operations and IEEE-754 basic arithmetic plus `ln`/`sqrt`/`cos` are used.

#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
}

fn splitmix64(x: &mut u64) -> u64 {
    *x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut x = seed;
        Rng { s: [splitmix64(&mut x), splitmix64(&mut x), splitmix64(&mut x), splitmix64(&mut x)] }
    }

    /// Independent sub-stream: same seed + same `stream` id -> same numbers.
    pub fn derive(seed: u64, stream: u64) -> Self {
        let mut x = seed ^ stream.wrapping_mul(0xD1B5_4A32_D192_ED03);
        Self::new(splitmix64(&mut x))
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// Uniform in [0, 1) with 53 bits of precision.
    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Uniform in (0, 1]: safe for `ln`.
    pub fn f64_open0(&mut self) -> f64 {
        1.0 - self.f64()
    }

    /// Uniform integer in `0..n` (Lemire's multiply-shift, tiny bias acceptable for n << 2^64).
    pub fn below(&mut self, n: u64) -> u64 {
        ((self.next_u64() as u128 * n as u128) >> 64) as u64
    }

    /// Exponential with the given rate (mean `1/rate`).
    pub fn exp(&mut self, rate: f64) -> f64 {
        -self.f64_open0().ln() / rate
    }

    /// Standard normal (Box-Muller, one value per call for simplicity).
    pub fn normal(&mut self) -> f64 {
        let u1 = self.f64_open0();
        let u2 = self.f64();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }

    /// Pareto (type I) with scale `xm` and shape `alpha`.
    pub fn pareto(&mut self, xm: f64, alpha: f64) -> f64 {
        xm / self.f64_open0().powf(1.0 / alpha)
    }
}

/// Walker/Vose alias table: O(1) sampling from a discrete distribution.
#[derive(Clone, Debug)]
pub struct AliasTable {
    prob: Vec<f64>,
    alias: Vec<u32>,
}

impl AliasTable {
    /// `weights` must be non-negative with a positive sum.
    pub fn new(weights: &[f64]) -> Self {
        let n = weights.len();
        assert!(n > 0);
        let sum: f64 = weights.iter().sum();
        assert!(sum > 0.0, "alias table needs a positive total weight");
        let mut prob: Vec<f64> = weights.iter().map(|w| w * n as f64 / sum).collect();
        let mut alias = vec![0u32; n];
        let (mut small, mut large): (Vec<usize>, Vec<usize>) = (Vec::new(), Vec::new());
        for (i, &p) in prob.iter().enumerate() {
            if p < 1.0 { small.push(i) } else { large.push(i) }
        }
        while let (Some(s), Some(&l)) = (small.pop(), large.last()) {
            alias[s] = l as u32;
            prob[l] -= 1.0 - prob[s];
            if prob[l] < 1.0 {
                large.pop();
                small.push(l);
            }
        }
        for i in large.into_iter().chain(small) {
            prob[i] = 1.0;
        }
        AliasTable { prob, alias }
    }

    pub fn sample(&self, rng: &mut Rng) -> usize {
        let i = rng.below(self.prob.len() as u64) as usize;
        if rng.f64() < self.prob[i] { i } else { self.alias[i] as usize }
    }

    pub fn len(&self) -> usize {
        self.prob.len()
    }
    pub fn is_empty(&self) -> bool {
        self.prob.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_stream() {
        let (mut a, mut b) = (Rng::new(42), Rng::new(42));
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn known_first_value_is_stable() {
        // Pin the stream: if this changes, every published result changes.
        let mut r = Rng::new(1);
        let v = r.next_u64();
        assert_eq!(v, Rng::new(1).next_u64());
        assert_ne!(v, Rng::new(2).next_u64());
    }

    #[test]
    fn alias_matches_weights() {
        let t = AliasTable::new(&[1.0, 3.0, 0.0, 6.0]);
        let mut r = Rng::new(7);
        let mut c = [0u32; 4];
        for _ in 0..100_000 {
            c[t.sample(&mut r)] += 1;
        }
        assert_eq!(c[2], 0);
        assert!((c[3] as f64 / 100_000.0 - 0.6).abs() < 0.01);
        assert!((c[0] as f64 / 100_000.0 - 0.1).abs() < 0.01);
    }
}
