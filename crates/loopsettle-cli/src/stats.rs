//! Small statistics and a deterministic parallel map, shared by the experiments (specs/09).

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Sample mean and standard deviation (n - 1 in the denominator; 0 for a single value).
pub fn mean_sd(v: &[f64]) -> (f64, f64) {
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    let var = if v.len() > 1 { v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0) } else { 0.0 };
    (m, var.sqrt())
}

/// Two-sided 95% Student t critical value (a table is enough: up to 30 degrees of freedom,
/// then the normal value).
pub fn t95(df: usize) -> f64 {
    const T: [f64; 30] = [
        12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.160, 2.145, 2.131, 2.120, 2.110, 2.101, 2.093, 2.086, 2.080, 2.074,
        2.069, 2.064, 2.060, 2.056, 2.052, 2.048, 2.045, 2.042,
    ];
    if df == 0 {
        f64::NAN
    } else if df <= 30 {
        T[df - 1]
    } else {
        1.96
    }
}

/// Mean with its 95% confidence interval: `(mean, low, high)`. With one value the interval is
/// not defined and both ends are NaN (shown as such, never as a zero-width interval).
pub fn mean_ci95(v: &[f64]) -> (f64, f64, f64) {
    let (m, sd) = mean_sd(v);
    let half = t95(v.len().saturating_sub(1)) * sd / (v.len() as f64).sqrt();
    (m, m - half, m + half)
}

/// Linear-interpolation quantile of an already sorted slice (NaN if empty).
pub fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let pos = q * (sorted.len() - 1) as f64;
    let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
    sorted[lo] + (sorted[hi] - sorted[lo]) * (pos - lo as f64)
}

/// Worker threads for [`par_map`]: `LOOPSETTLE_THREADS` if set, else the number of cores
/// capped at 2 (the reference machine has 2 cores and 4 GB; each run needs a few MiB).
pub fn threads() -> usize {
    std::env::var("LOOPSETTLE_THREADS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&n| n >= 1)
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1).min(2))
}

/// `items.iter().map(f)` on several threads. The output is in the order of `items` whatever
/// the scheduling, so results do not depend on the number of threads.
///
/// `std::thread::scope` lets the workers borrow `items` and `f` without `Arc`: the scope
/// guarantees they finish before the borrow ends. Each worker takes the next free index from
/// an atomic counter (work stealing is not needed for a handful of long tasks).
pub fn par_map<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let workers = threads().min(items.len());
    if workers <= 1 {
        return items.iter().map(f).collect();
    }
    let next = AtomicUsize::new(0);
    let out: Mutex<Vec<Option<R>>> = Mutex::new((0..items.len()).map(|_| None).collect());
    std::thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= items.len() {
                    break;
                }
                let r = f(&items[i]);
                out.lock().unwrap()[i] = Some(r);
            });
        }
    });
    out.into_inner().unwrap().into_iter().map(|r| r.expect("every item was computed")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_sd_and_ci() {
        let (m, sd) = mean_sd(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        assert!((m - 5.0).abs() < 1e-12 && (sd - (32.0f64 / 7.0).sqrt()).abs() < 1e-12);
        let (m, lo, hi) = mean_ci95(&[1.0, 2.0, 3.0]);
        // sd = 1, n = 3, t(2) = 4.303: half width = 4.303 / sqrt(3)
        assert!((m - 2.0).abs() < 1e-12 && (hi - lo - 2.0 * 4.303 / 3f64.sqrt()).abs() < 1e-9);
        assert!(mean_ci95(&[1.0]).1.is_nan());
    }

    #[test]
    fn quantile_interpolates() {
        let v = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(quantile(&v, 0.0), 1.0);
        assert_eq!(quantile(&v, 1.0), 4.0);
        assert!((quantile(&v, 0.5) - 2.5).abs() < 1e-12);
    }

    #[test]
    fn par_map_keeps_input_order() {
        let items: Vec<u64> = (0..37).collect();
        // uneven work, so threads finish out of order
        let got = par_map(&items, |&x| (0..(37 - x) * 20_000).fold(x, |a, b| a.wrapping_mul(31).wrapping_add(b)));
        let want: Vec<u64> = items.iter().map(|&x| (0..(37 - x) * 20_000).fold(x, |a, b| a.wrapping_mul(31).wrapping_add(b))).collect();
        assert_eq!(got, want);
    }
}
