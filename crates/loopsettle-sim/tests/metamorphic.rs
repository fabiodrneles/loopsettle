//! Metamorphic and robustness tests (specs/09, E5). They need no oracle: a known change to the
//! input must change the output in a known way. Cheap, and they catch unit and index errors.

use loopsettle_core::rng::Rng;
use loopsettle_core::{Cents, Payment, TimeMs};
use loopsettle_sim::bound::{delay_bounds_from, liquidity_floors_from};
use loopsettle_sim::{run_payments, EngineSpec, Opening, PaymentStream, ScenarioConfig};

const N: usize = 20;
const DELAYS: [TimeMs; 3] = [2_000, 60_000, 3_600_000];

fn base() -> Vec<Payment> {
    let cfg = ScenarioConfig {
        seed: 7,
        participants: N,
        payments_per_day: 15_000.0,
        n_big: 2,
        n_mid: 4,
        ..Default::default()
    };
    PaymentStream::new(&cfg).collect()
}

struct Measures {
    ub: Vec<Cents>,
    lb: Vec<Cents>,
    floors: Vec<Vec<Cents>>,
    bounds: Vec<Vec<Cents>>,
    /// liquidity per participant: gross, deferred 24x, micro-cycle
    engines: Vec<Vec<Cents>>,
}

fn measure(ps: &[Payment]) -> Measures {
    let specs = [
        EngineSpec::Gross,
        EngineSpec::parse("deferred:24").unwrap(),
        EngineSpec::parse("micro:all:1000:5000").unwrap(),
    ];
    let t_end = ps.last().map(|p| p.t_arrival).unwrap_or(0);
    let o = run_payments(N, ps.iter().copied(), &specs, &Opening::Unlimited, t_end);
    Measures {
        ub: (0..N as u32).map(|i| o.flow.ub(i)).collect(),
        lb: (0..N as u32).map(|i| o.flow.lb(i)).collect(),
        floors: liquidity_floors_from(N, || ps.iter().copied(), &DELAYS),
        bounds: delay_bounds_from(N, || ps.iter().copied(), &DELAYS),
        engines: o
            .results
            .iter()
            .map(|r| r.liquidity_by_participant.clone())
            .collect(),
    }
}

fn times(v: &[Cents], k: Cents) -> Vec<Cents> {
    v.iter().map(|x| x * k).collect()
}

/// Multiplying every amount by k multiplies every liquidity measure by k (also the engines:
/// which payments form a cycle does not depend on the amounts when liquidity is unlimited).
#[test]
fn scaling_amounts_scales_every_liquidity_measure() {
    let ps = base();
    let a = measure(&ps);
    let k = 7;
    let scaled: Vec<Payment> = ps
        .iter()
        .map(|p| Payment {
            amount: p.amount * k,
            ..*p
        })
        .collect();
    let b = measure(&scaled);
    assert_eq!(b.ub, times(&a.ub, k));
    assert_eq!(b.lb, times(&a.lb, k));
    for j in 0..DELAYS.len() {
        assert_eq!(b.floors[j], times(&a.floors[j], k));
        assert_eq!(b.bounds[j], times(&a.bounds[j], k));
    }
    for e in 0..a.engines.len() {
        assert_eq!(b.engines[e], times(&a.engines[e], k), "engine {e}");
    }
}

/// Renaming participants permutes the per-participant measures and nothing else. (The
/// micro-cycle engine is excluded on purpose: its greedy search explores participants in index
/// order, so its result may legitimately depend on the labels.)
#[test]
fn relabelling_participants_permutes_the_measures() {
    let ps = base();
    let a = measure(&ps);
    let perm = |i: u32| (i * 7 + 3) % N as u32; // 7 is coprime with 20: a bijection
    let relabelled: Vec<Payment> = ps
        .iter()
        .map(|p| Payment {
            payer: perm(p.payer),
            payee: perm(p.payee),
            ..*p
        })
        .collect();
    let b = measure(&relabelled);
    for i in 0..N as u32 {
        let (x, y) = (i as usize, perm(i) as usize);
        assert_eq!(a.ub[x], b.ub[y]);
        assert_eq!(a.lb[x], b.lb[y]);
        for j in 0..DELAYS.len() {
            assert_eq!(a.floors[j][x], b.floors[j][y]);
            assert_eq!(a.bounds[j][x], b.bounds[j][y]);
        }
        assert_eq!(a.engines[0][x], b.engines[0][y], "gross");
        assert_eq!(a.engines[1][x], b.engines[1][y], "deferred");
    }
}

/// Shifting every arrival by the same amount changes no bound. (Engines with a clock — cycle
/// times, windows — are anchored at t = 0 and are excluded.)
#[test]
fn shifting_time_changes_no_bound() {
    let ps = base();
    let a = measure(&ps);
    let shifted: Vec<Payment> = ps
        .iter()
        .map(|p| Payment {
            t_arrival: p.t_arrival + 123_457,
            ..*p
        })
        .collect();
    let b = measure(&shifted);
    assert_eq!(a.ub, b.ub);
    assert_eq!(a.lb, b.lb);
    assert_eq!(a.floors, b.floors);
    assert_eq!(a.bounds, b.bounds);
    assert_eq!(a.engines[0], b.engines[0]);
}

/// The readers must return `Err`, never panic, on arbitrary input (they are fed files written
/// by other people). Random text from a fixed seed, biased towards the characters the formats
/// use, plus mutations of valid inputs.
#[test]
fn readers_never_panic_on_garbage() {
    let alphabet: Vec<char> = "abcxyz0123456789 \t\n\r,;:._-+=[]\"'#€é\u{0}"
        .chars()
        .collect();
    let valid_toml = "[scenario]\nseed = 1\nparticipants = 10\nhourly = [1.0, 2.5]\nname = \"x\"\n";
    let valid_csv = "time,from,to,amount\n08:00:00,A,B,10.00\n08:00:01,B,C,5\n";
    let mut rng = Rng::new(20261111);
    let dir = std::env::temp_dir().join(format!("loopsettle-fuzz-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("f.csv");
    for case in 0..2_000 {
        let mut s: Vec<char> = match case % 4 {
            0 => valid_toml.chars().collect(),
            1 => valid_csv.chars().collect(),
            _ => Vec::new(),
        };
        let edits = 1 + rng.below(40) as usize;
        for _ in 0..edits {
            let c = alphabet[rng.below(alphabet.len() as u64) as usize];
            let pos = rng.below(s.len() as u64 + 1) as usize;
            if !s.is_empty() && rng.below(3) == 0 {
                s.remove(pos.min(s.len() - 1));
            } else {
                s.insert(pos, c);
            }
        }
        let text: String = s.into_iter().collect();
        let _ = loopsettle_sim::toml_lite::parse(&text);
        std::fs::write(&file, &text).unwrap();
        if let Ok(schema) = loopsettle_sim::csvfile::CsvSchema::scan(file.to_str().unwrap()) {
            if let Ok(it) = schema.payments() {
                let _ = it.count();
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}
