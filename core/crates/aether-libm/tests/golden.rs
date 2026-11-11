//! Golden-sample bit-exactness tests for aether-libm against Node 22.22.1
//! (V8 12.4 fdlibm `src/base/ieee754.cc`).
//!
//! Files are produced by `world/core/scripts/gen_libm_gold.mjs`:
//! - full 1e6-sample run: `/tmp/libm_gold_sample_<fn>.bin`
//! - committed 1e4-sample prefix: `tests/gold/libm_gold_<fn>.bin`
//!
//! Inputs are regenerated with aether-rng Mulberry32(20261102) exactly as the
//! JS generator does, so only the output f64s are stored. A test passes only
//! when every output matches bit-for-bit (`to_bits()` equality; NaN inputs are
//! compared by class, since NaN payloads are engine-canonicalized identically
//! in both runtimes but excluded from strict comparison for robustness).

use aether_rng::Mulberry32;

const SEED: u32 = 20261102;
const TWO_PI: f64 = std::f64::consts::PI * 2.0;
const N_FULL: usize = 1_000_000;
const N_COMMIT: usize = 10_000;

fn load_bytes(name: &str, full: bool) -> Option<Vec<u8>> {
    if full {
        if let Ok(bytes) = std::fs::read(format!("/tmp/libm_gold_sample_{name}.bin")) {
            return Some(bytes);
        }
    }
    let manifest = env!("CARGO_MANIFEST_DIR");
    std::fs::read(format!("{manifest}/tests/gold/libm_gold_{name}.bin")).ok()
}

fn gold_path_label(name: &str, full: bool) -> String {
    if full {
        format!("/tmp/libm_gold_sample_{name}.bin (1e6)")
    } else {
        format!("tests/gold/libm_gold_{name}.bin (1e4 commit prefix)")
    }
}

fn assert_bits_eq(name: &str, idx: usize, input: f64, expect: f64, got: f64, n_mismatch: &mut usize) {
    if !((expect.is_nan() && got.is_nan()) || expect.to_bits() == got.to_bits()) {
        if *n_mismatch == 0 {
            panic!(
                "{name}[{idx}]: input={input:e} 0x{:016x} expect=0x{:016x} ({}e) got=0x{:016x} ({}e)",
                input.to_bits(), expect.to_bits(), expect, got.to_bits(), got,
            );
        }
        *n_mismatch += 1;
    }
}

fn run_pairs(name: &str, f: fn(f64) -> f64, mapper: fn(f64) -> f64, n_special: usize) {
    let mut n_mismatch = 0usize;
    let mut tested = 0usize;
    for (full, n) in [(true, N_FULL), (false, N_COMMIT)] {
        let Some(bytes) = load_bytes(name, full) else {
            if full {
                eprintln!("skip {name}: /tmp golden file absent, committed 1e4 prefix already checked");
                continue;
            }
            panic!("missing committed golden file tests/gold/libm_gold_{name}.bin — run scripts/gen_libm_gold.mjs");
        };
        let n_samples = bytes.len() / 16;
        let mut rng = Mulberry32::new(SEED);
        for i in 0..n_samples {
            let x = if i < n_special {
                f64::from_le_bytes(bytes[i * 16..i * 16 + 8].try_into().unwrap())
            } else {
                mapper(rng.next_f64())
            };
            let expect = f64::from_le_bytes(bytes[i * 16 + 8..i * 16 + 16].try_into().unwrap());
            let got = f(x);
            assert_bits_eq(name, i, x, expect, got, &mut n_mismatch);
            tested += 1;
            if !full && tested >= n + n_special {
                break;
            }
        }
        assert_eq!(n_mismatch, 0, "{name}: {} bit mismatches over {} samples ({})", n_mismatch, tested, gold_path_label(name, full));
        println!("{name}: {tested} samples bit-exact ({})", gold_path_label(name, full));
    }
}

fn run_triples(name: &str, f: fn(f64, f64) -> f64, mapper_a: fn(f64) -> f64, mapper_b: fn(f64) -> f64, n_special: usize) {
    let mut n_mismatch = 0usize;
    let mut tested = 0usize;
    for (full, n) in [(true, N_FULL), (false, N_COMMIT)] {
        let Some(bytes) = load_bytes(name, full) else {
            if full {
                eprintln!("skip {name}: /tmp golden file absent, committed 1e4 prefix already checked");
                continue;
            }
            panic!("missing committed golden file tests/gold/libm_gold_{name}.bin — run scripts/gen_libm_gold.mjs");
        };
        let n_samples = bytes.len() / 24;
        let mut rng_a = Mulberry32::new(SEED);
        let mut rng_b = Mulberry32::new(SEED);
        for i in 0..n_samples {
            let (a, b) = if i < n_special {
                (
                    f64::from_le_bytes(bytes[i * 24..i * 24 + 8].try_into().unwrap()),
                    f64::from_le_bytes(bytes[i * 24 + 8..i * 24 + 16].try_into().unwrap()),
                )
            } else {
                (mapper_a(rng_a.next_f64()), mapper_b(rng_b.next_f64()))
            };
            let expect = f64::from_le_bytes(bytes[i * 24 + 16..i * 24 + 24].try_into().unwrap());
            let got = f(a, b);
            assert_bits_eq(name, i, a, expect, got, &mut n_mismatch);
            tested += 1;
            if !full && tested >= n + n_special {
                break;
            }
        }
        assert_eq!(n_mismatch, 0, "{name}: {} bit mismatches over {} samples ({})", n_mismatch, tested, gold_path_label(name, full));
        println!("{name}: {tested} samples bit-exact ({})", gold_path_label(name, full));
    }
}

#[test]
fn golden_sin() {
    run_pairs("sin", aether_libm::sin, |r| r * TWO_PI, 30);
}

#[test]
fn golden_cos() {
    run_pairs("cos", aether_libm::cos, |r| r * TWO_PI, 30);
}

#[test]
fn golden_exp() {
    run_pairs("exp", aether_libm::exp, |r| r * TWO_PI, 31);
}

#[test]
fn golden_atan2() {
    run_triples("atan2", aether_libm::atan2, |r| (r * 2.0 - 1.0) * TWO_PI, |r| (r * 2.0 - 1.0) * TWO_PI, 32);
}

#[test]
fn golden_pow() {
    run_triples("pow", aether_libm::pow, |r| r, |r| 1.0 + r * 0.7, 27);
}
