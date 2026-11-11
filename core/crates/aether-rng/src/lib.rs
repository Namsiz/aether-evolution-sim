//! `aether-rng`: bit-exact Mulberry32 PRNG aligned with the JS oracle in `living/src/world.js`.
//!
//! **Why this design?**  
//! The virtual world's determinism contract requires that Rust-side simulation produces
//! identical state hashes to the JS oracle for any given seed. JS `mulberry32` uses
//! 32-bit bitwise operations on float64-backed numbers; Rust must replicate the exact
//! bit patterns via `u32` wrapping arithmetic.
//!
//! **Algorithm reference** (from world.js L13-27):
//! ```js
//! function mulberry32(seed) {
//!   let a = seed >>> 0;
//!   return function () {
//!     a |= 0;
//!     a = (a + 0x6d2b79f5) | 0;
//!     let t = Math.imul(a ^ (a >>> 15), 1 | a);
//!     t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
//!     return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
//!   };
//! }
//! ```
//!
//! **Key alignment points:**
//! - `>>> 0` / `| 0` → Rust `u32` naturally wraps at 2^32
//! - `Math.imul(x, y)` → `(x as i32).wrapping_mul(y as i32) as u32` (same bit pattern)
//! - Final division by `4294967296.0` (`2^32`) converts u32 to [0, 1) f64
//! - Integer overflow is intentional wrapping, never panicking → `wrapping_*` ops
//!
//! **Not thread-safe by design**: sim core runs single-threaded in wasm; native parallel
//! mode uses per-entity RNG substreams derived from this same primitive.

#[derive(Debug, Clone)]
pub struct Mulberry32 {
    /// Internal 32-bit state, updated each call via wrapping addition of the constant.
    state: u32,
}

/// JS-imul equivalent: signed 32-bit multiplication preserving low 32 bits.
#[inline]
fn imul(a: i32, b: i32) -> u32 {
    (a.wrapping_mul(b)) as u32
}

impl Mulberry32 {
    /// Create a new PRNG seeded from a `u32`. Matches JS `mulberry32(seed)`.
    pub fn new(seed: u32) -> Self {
        Mulberry32 { state: seed }
    }

    /// Return the next value in [0, 2^32). Bit-for-bit identical to the raw u32
    /// before normalization to f64 in the JS version.
    pub fn next_u32(&mut self) -> u32 {
        // a |= 0; a = (a + 0x6d2b79f5) | 0
        self.state = self.state.wrapping_add(0x6d2b79f5);
        let a = self.state;

        // t = Math.imul(a ^ (a >>> 15), 1 | a)
        let t = imul((a ^ (a >> 15)) as i32, (1u32 | a) as i32);

        // t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
        let t = t.wrapping_add(imul((t ^ (t >> 7)) as i32, (61u32 | t) as i32)) ^ t;

        // return ((t ^ (t >>> 14)) >>> 0) — cast to u32 (already is)
        t ^ (t >> 14)
    }

    /// Return the next value in [0, 1) as f64. Matches JS `return ... / 4294967296`.
    pub fn next_f64(&mut self) -> f64 {
        self.next_u32() as f64 / 4294967296.0
    }

    /// Return a value in [lo, hi) as f64: `lo + rng * (hi - lo)`.
    pub fn next_range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.next_f64() * (hi - lo)
    }

    /// Return a clone with identical internal state — enables "replay from snapshot"
    /// for differential testing (对拍).
    #[must_use]
    pub fn snapshot(&self) -> Self {
        self.clone()
    }

    /// Capture the internal 32-bit state so a stream can be resumed later
    /// (Phase 6 save/restore): `Mulberry32::from_state(rng.state())` reproduces
    /// the exact remaining sequence.
    #[must_use]
    pub fn state(&self) -> u32 {
        self.state
    }

    /// Rebuild a stream from a previously captured [`Mulberry32::state`] value.
    #[must_use]
    pub fn from_state(state: u32) -> Self {
        Mulberry32 { state }
    }

    /// Consume N samples and return them as a slice-owned vector. Useful for benchmarking.
    pub fn collect_f64(&mut self, n: usize) -> Vec<f64> {
        (0..n).map(|_| self.next_f64()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_five_match_js_oracle_seed_20261102() {
        let mut rng = Mulberry32::new(20261102);
        let expected = [
            0.07848557480610907f64,
            0.4848272386007011f64,
            0.20832762983627617f64,
            0.39460715488530695f64,
            0.2319970102980733f64,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            let got = rng.next_f64();
            assert_eq!(
                got.to_bits(),
                exp.to_bits(),
                "sample[{}] bit mismatch: got {:x}, expected {:x}",
                i,
                got.to_bits(),
                exp.to_bits()
            );
        }
    }

    #[test]
    fn next_u32_returns_valid_range() {
        let mut rng = Mulberry32::new(0);
        for _ in 0..1000 {
            let v = rng.next_u32();
            assert!(v < u32::MAX, "next_u32 out of range: {}", v);
        }
    }

    #[test]
    fn next_range_bounds() {
        let mut rng = Mulberry32::new(12345);
        for _ in 0..100 {
            let v = rng.next_range(2.5, 7.5);
            assert!(v >= 2.5 && v < 7.5, "next_range out of bounds: {}", v);
        }
    }

    #[test]
    fn clone_preserves_state() {
        let mut rng = Mulberry32::new(777);
        let mut snap = rng.snapshot();
        // Clone must produce identical output to original starting from same seed
        assert_eq!(rng.next_u32(), snap.next_u32(), "clone's first output must match original's first");
        assert_eq!(rng.next_u32(), snap.next_u32(), "clone's second output must match original's second");
        // After consuming some from original, original and clone diverge
        for _ in 0..50 {
            rng.next_u32();
        }
        let jumped = rng.next_u32();
        let still_orig = snap.next_u32(); // should be original's 2nd value (was already consumed once above)
        assert_ne!(jumped, still_orig, "advanced original must differ from clone");
    }

    #[test]
    fn different_seed_different_sequence() {
        let mut a = Mulberry32::new(1);
        let mut b = Mulberry32::new(2);
        assert_ne!(a.next_f64(), b.next_f64(), "different seeds must differ immediately");
    }

    #[test]
    fn same_seed_reproducible() {
        let mut a = Mulberry32::new(42);
        let mut b = Mulberry32::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_f64().to_bits(), b.next_f64().to_bits());
        }
    }

    #[test]
    fn state_roundtrip_preserves_stream() {
        // Phase 6 save/restore contract: from_state(state()) resumes the exact
        // remaining sequence.
        let mut a = Mulberry32::new(99);
        for _ in 0..10 {
            a.next_u32();
        }
        let mut b = Mulberry32::from_state(a.state());
        for _ in 0..1000 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }
}
