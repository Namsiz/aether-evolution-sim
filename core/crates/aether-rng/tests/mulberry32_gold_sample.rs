/// Gold sample integration test: compare 10^6 Rust-generated floats against JS oracle output.
///
/// The golden binary is produced once by `scripts/gen_gold_sample.mjs` (run from living/)
/// and committed to /tmp/mulberry32_gold_sample_bytes.bin. If missing, the test skips with
/// a hint to regenerate.
///
/// Run with:  cargo test -p aether-rng --test mulberry32_gold_sample
#[path = "../src/lib.rs"]
mod aether_rng;

use aether_rng::Mulberry32;

const GOLDEN_SEED: u32 = 20261102;
const N: usize = 1_000_000;

fn load_golden() -> Vec<f64> {
    let bytes = std::fs::read("/tmp/mulberry32_gold_sample_bytes.bin")
        .expect("golden binary not found — run scripts/gen_gold_sample.mjs from world/living/");
    assert_eq!(bytes.len(), N * 8, "golden file length must be {} bytes (got {})", N * 8, bytes.len());
    bytes.chunks_exact(8)
        .map(|c| f64::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

#[test]
fn bit_exact_1m_samples_against_js_oracle() {
    let golden = load_golden();
    let mut rng = Mulberry32::new(GOLDEN_SEED);
    for (i, &expected) in golden.iter().enumerate() {
        let got = rng.next_f64();
        if got.to_bits() != expected.to_bits() {
            panic!(
                "bit-exact mismatch at index {}: got 0x{:016x}, expected 0x{:016x}",
                i,
                got.to_bits(),
                expected.to_bits()
            );
        }
    }
}
