//! `optimality-gap` (specs/03): where do the engines stand between the liquidity floors and the
//! offline optimum?
//!
//! ```text
//! cargo run --release -p loopsettle-sim --example optimality_gap
//! ```
//!
//! Writes `results/optimality-gap/gap.csv` and `instances.csv`. It is an example, not a
//! `loopsettle experiment`, because it needs the LP solver, which is a dev-dependency (a test
//! oracle that the library and the CLI must not depend on; DECISIONS.md, D25).
//!
//! Instances: a small system with a busy hour. `PAYMENTS` payments of a synthetic day among
//! `PARTICIPANTS` participants, with the day compressed into one hour so that payments are a
//! few seconds apart and deadlines of 2 to 60 s make them interact. ASSUMPTION: not calibrated;
//! the purpose is to compare three numbers on the same instance, not to estimate a level.

#[path = "../tests/common/optimum.rs"]
mod optimum;

use loopsettle_core::{Payment, TimeMs};
use loopsettle_sim::bound::liquidity_floors_from;
use loopsettle_sim::{run_payments, EngineSpec, Opening, PaymentStream, ScenarioConfig};
use std::fmt::Write;

const PARTICIPANTS: usize = 20;
const PAYMENTS: f64 = 1_500.0;
const SEEDS: u64 = 20;
const DELAYS_MS: [TimeMs; 3] = [2_000, 10_000, 60_000];
const SIGMAS: [f64; 2] = [0.4, 1.0];

fn mean_ci(v: &[f64]) -> (f64, f64, f64) {
    // 95% Student t for 19 degrees of freedom (SEEDS = 20)
    const T19: f64 = 2.093;
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    let sd = (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    let h = T19 * sd / n.sqrt();
    (m, m - h, m + h)
}

fn main() {
    assert_eq!(SEEDS, 20, "mean_ci uses the t value for 20 seeds");
    let t0 = std::time::Instant::now();
    let mut inst = String::from("receiver_bias_sigma,seed,max_delay_ms,payments,ub,floors_total,lp_optimum,best_engine,best_engine_liquidity\n");
    let mut out = String::from(
        "receiver_bias_sigma,max_delay_ms,seeds,floor_bound_pct_mean,floor_bound_pct_ci95_low,floor_bound_pct_ci95_high,lp_bound_pct_mean,lp_bound_pct_ci95_low,lp_bound_pct_ci95_high,best_engine_saving_pct_mean,best_engine_saving_pct_ci95_low,best_engine_saving_pct_ci95_high,lp_over_floor_bound,engine_over_lp_bound,instances_with_lp_above_floors\n",
    );
    for &sigma in &SIGMAS {
        for &d in &DELAYS_MS {
            let (mut fb, mut lb, mut eb) = (Vec::new(), Vec::new(), Vec::new());
            let mut strict = 0;
            for seed in 1..=SEEDS {
                let cfg = ScenarioConfig {
                    seed,
                    participants: PARTICIPANTS,
                    payments_per_day: PAYMENTS,
                    n_big: 2,
                    n_mid: 3,
                    receiver_bias_sigma: sigma,
                    ..Default::default()
                };
                // compress the day into one hour (24x), keeping order and relative spacing
                let ps: Vec<Payment> = PaymentStream::new(&cfg)
                    .map(|p| Payment {
                        t_arrival: p.t_arrival / 24,
                        ..p
                    })
                    .collect();
                let t_end = ps.last().map(|p| p.t_arrival).unwrap_or(0);
                let names = [
                    format!("micro:all:100:{d}"),
                    format!("micro:all:500:{d}"),
                    format!("micro:all:{d}:{d}"),
                    format!("deferred:{}", 86_400_000 / d),
                ];
                let mut specs = vec![EngineSpec::Gross];
                specs.extend(names.iter().map(|s| EngineSpec::parse(s).unwrap()));
                let o = run_payments(
                    PARTICIPANTS,
                    ps.iter().copied(),
                    &specs,
                    &Opening::Unlimited,
                    t_end,
                );
                let ub = o.ub() as f64;
                let floors: f64 = liquidity_floors_from(PARTICIPANTS, || ps.iter().copied(), &[d])
                    [0]
                .iter()
                .map(|&v| v as f64)
                .sum();
                let lp = optimum::lp_min_liquidity(PARTICIPANTS, &ps, d)
                    .expect("the LP is feasible: gross settlement is a solution");
                let best = o.results[1..].iter().min_by_key(|r| r.liquidity).unwrap();
                assert!(
                    floors <= lp * (1.0 + 1e-6) + 1e-6
                        && lp <= best.liquidity as f64 * (1.0 + 1e-6) + 1e-6,
                    "sandwich violated: {floors} {lp} {}",
                    best.liquidity
                );
                if lp > floors * (1.0 + 1e-6) + 1e-6 {
                    strict += 1;
                }
                let pct = |x: f64| if ub > 0.0 { 100.0 * (ub - x) / ub } else { 0.0 };
                fb.push(pct(floors));
                lb.push(pct(lp));
                eb.push(pct(best.liquidity as f64));
                writeln!(
                    inst,
                    "{sigma},{seed},{d},{},{:.0},{:.0},{:.2},{},{}",
                    ps.len(),
                    ub,
                    floors,
                    lp,
                    best.name,
                    best.liquidity
                )
                .unwrap();
            }
            let (f, fl, fh) = mean_ci(&fb);
            let (l, ll, lh) = mean_ci(&lb);
            let (e, el, eh) = mean_ci(&eb);
            writeln!(
                out,
                "{sigma},{d},{SEEDS},{f:.4},{fl:.4},{fh:.4},{l:.4},{ll:.4},{lh:.4},{e:.4},{el:.4},{eh:.4},{:.4},{:.4},{strict}",
                if f > 0.0 { l / f } else { 0.0 },
                if l > 0.0 { e / l } else { 0.0 }
            )
            .unwrap();
            eprintln!("  sigma {sigma}, d {d} ms: floor bound {f:.2}%, LP bound {l:.2}%, best engine {e:.2}% ({:.0} s)", t0.elapsed().as_secs_f64());
        }
    }
    let dir = std::path::Path::new("results/optimality-gap");
    std::fs::create_dir_all(dir).expect("cannot create results/optimality-gap");
    std::fs::write(dir.join("gap.csv"), &out).unwrap();
    std::fs::write(dir.join("instances.csv"), &inst).unwrap();
    eprintln!(
        "wrote results/optimality-gap/gap.csv and instances.csv in {:.0} s",
        t0.elapsed().as_secs_f64()
    );
    print!("{out}");
}
