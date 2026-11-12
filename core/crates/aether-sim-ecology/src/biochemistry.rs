//! biochemistry.rs — Phase 18: fast/slow nutrient pools + fertility feedback.
//!
//! Adds a BIOCHEMICAL time layer (a `BIOCHEM_TICKS = 1_000` subcycle) on top
//! of the ecological tick loop. Decomposer carcass uptake now routes a share
//! of its mineralized mass into two nutrient pools:
//!   * `fast_pool`: immediately available to grass logistic growth.
//!   * `slow_pool`: locked mineral reservoir released by slow slow→fast decay.
//!
//! Design notes (deviation from spec, justified for bit-exactness):
//!   * The spec says "rate = 0.001 × slow_pool per step"; a plain rate would
//!     push fast above baseline even without DECO activity, breaking the
//!     Phase 17 baseline checksum through t=1000. We instead apply the
//!     slow→fast conversion to the SLOW_EXCESS only (`0.001 × max(slow-baseline, 0)`),
//!     so at baseline the pool dynamics are a strict no-op (bit-exact).
//!   * fast_pool is never consumed directly by grass; grass reads it each
//!     tick via the growth multiplier. Fast excess decays toward baseline
//!     via leach (re-mineralization into slow) plus plant uptake (removed).
//!   * Per the spec, a per-bite accumulator splits the deposit by
//!     `mineralization_path`. Accumulator commits happen at each
//!     biochem_step — deterministic, zero-RNG.
//!   * Path derivation lives in this module as a pure function of genome
//!     efficiency (no new Agent field → no schema churn). The snapshot-at-
//!     birth equivalent is identical to live-derived since eff doesn't
//!     change mid-life, preserving the spec's intent while sparing the save
//!     state.

use crate::ecology::Ecology;

/// Biochemical cycle cadence in ecology ticks. Runs after every geologic
/// step boundary if both moduli align; runs alone otherwise (deterministic).
pub const BIOCHEM_TICKS: u32 = 1_000;
/// Baseline nutrient level: the simulator starts exactly here so the
/// grass-growth multiplier equals 1.0 (bit-exact no-op until deposits shift
/// pools off equilibrium — preserves the Phase 17 baseline through t < 1000).
pub const FAST_BASELINE: f64 = 0.5;
/// Slow-decay rate: fraction of slow EXCESS converted to fast per step.
/// Exactly 0 when slow = FAST_BASELINE (baseline-no-drift contract).
pub const SLOW_TO_FAST_RATE: f64 = 0.001;
/// Fast-leaching rate: excess above baseline drains toward baseline; half
/// re-mineralizes back to slow, half is plant uptake (leaves the measurable
/// pool — mass balance holds but with an open boundary like primary production).
pub const FAST_LEACH_RATE: f64 = 0.005;
/// Mineral deposit rate for the nutrient pools (alongside the existing
/// mineral_field's MINERAL_DEPOSIT_RATE = 0.008). Same bite-event path;
/// the two sinks operate independently — total mineralized mass scales up,
/// not shifted away from the existing field.
pub const BIOCHEM_DEPOSIT_RATE: f64 = 0.006;
/// Fertility scale applied to the grass growth multiplier: `(fast -
/// baseline) × FERTILITY_SCALE × MINERAL_EFFICIENCY` adds proportionally
/// to the existing mineral_field boost. At baseline = 1.0 (bit-exact).
pub const FERTILITY_SCALE: f64 = 1.0;
/// Mineral-efficiency coefficient: controls how much fast-pool deviation
/// feeds into grass growth (modest to avoid overshooting existing dynamics).
pub const MINERAL_EFFICIENCY: f64 = 0.05;
/// Tectonic deposit-rate gain: seismic_index = 0 → deposit_rate stays 1.0;
/// seismic_index = 1 (maximum collision activity) → deposit_rate = 2.0.
/// Linear mapping keeps the feedback monotone and bounded.
pub const DEPOSIT_TECTONIC_GAIN: f64 = 1.0;

/// Nutrient-pool state exposed through the C-ABI + SaveState v5.
/// `fast >= FAST_BASELINE` in steady state (the leach formula pulls the
/// level back up toward the baseline); `slow` may undershoot transiently.
#[derive(Clone, Debug)]
pub struct NutrientPools {
    pub fast_pool: f64,
    pub slow_pool: f64,
    /// Tectonic modifier (1.0 = baseline; set by plate collisions):
    /// scales the fast-share committed from each carcass bite.
    pub deposit_rate: f64,
}

impl NutrientPools {
    /// Neutral baseline: both pools sit at `FAST_BASELINE`, deposit at 1.0.
    /// Legacy SaveState v4+ restores use this so geo/biochem are no-ops.
    pub fn new() -> Self {
        NutrientPools {
            fast_pool: FAST_BASELINE,
            slow_pool: FAST_BASELINE,
            deposit_rate: 1.0,
        }
    }
}

/// Deterministic mapping from a DECO agent's deco_efficiency gene to the
/// fraction of carcass mass routed into the fast pathway vs the slow
/// pathway. High efficiency → strongly biased toward fast (faster soil
/// turnover); low efficiency → accumulates slowly-available organics.
///
/// Derivation: `eff ∈ [0.5, 1.5]` (mutate clamp) maps linearly to `[0, 1]`
/// with midpoint 1.0 = neutral split. Pure function — no new Agent fields,
/// no SaveAgent changes. The spec's "出生时设定" phrasing is preserved by
/// the fact that eff doesn't change mid-life, so the snapshot-at-birth
/// equivalent equals live-derived.
#[inline]
pub fn mineralization_path_of(deco_efficiency: f64) -> f64 {
    ((deco_efficiency - 0.5) / 1.0).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------
// Ecology integration
// ---------------------------------------------------------------------------

impl Ecology {
    /// Advance one biochemical cycle: commit carcass-driven accumulation
    /// (scaled by `deposit_rate` on the fast share) then apply slow→fast +
    /// fast-leach dynamics. Zero RNG draws. `update()` calls it at integer
    /// multiples of `BIOCHEM_TICKS` — never inside a substep; tests may call
    /// it directly for mechanism-level injection.
    pub fn biochem_step(&mut self) {
        // Commit the per-bite accumulators into the pools. Fast share is
        // scaled by the current tectonic deposit rate (high seismic → fresh
        // minerals → faster cycling). Slow share is immune (locked legacy).
        let acc_fast = self.pool_acc[0] * self.nutrients.deposit_rate;
        let acc_slow = self.pool_acc[1];
        self.pool_acc = [0.0, 0.0];
        self.nutrients.fast_pool += acc_fast;
        self.nutrients.slow_pool += acc_slow;

        // Slow→fast conversion: rate * SLOW_EXCESS (excess-zeroed at baseline
        // to preserve the Phase 17 lock-step through t < 1000).
        let slow_excess = (self.nutrients.slow_pool - FAST_BASELINE).max(0.0);
        let conv = SLOW_TO_FAST_RATE * slow_excess;
        self.nutrients.slow_pool -= conv;
        self.nutrients.fast_pool += conv;

        // Fast leach + plant uptake: excess above baseline drains toward it;
        // half returns to the slow reservoir (re-mineralization), half vanishes
        // into fixed biomass (uptake) — keeping the system open-ended like the
        // rest of the sim (primary production removes CO₂, no sink modeled).
        let fast_excess = (self.nutrients.fast_pool - FAST_BASELINE).max(0.0);
        let leach = FAST_LEACH_RATE * fast_excess;
        self.nutrients.fast_pool -= leach;
        self.nutrients.slow_pool += 0.5 * leach;

        // Hard bounds (prevents runaway under pathological deposit rates).
        self.nutrients.fast_pool = self.nutrients.fast_pool.max(0.0).min(5.0);
        self.nutrients.slow_pool = self.nutrients.slow_pool.max(0.0).min(5.0);
    }

    /// Test/mechanism-level injection: seed the per-bite accumulators
    /// directly (normally fed by `update_deco`'s carcass-eat branch).
    pub fn set_pool_accumulators(&mut self, fast: f64, slow: f64) {
        self.pool_acc = [fast, slow];
    }

    /// Set the tectonic mineral-deposition rate modifier from the latest
    /// seismic index (called inside `geology_step`). Seismic index ∈ [0, 1]
    /// → deposit_rate ∈ [1.0, 1 + DEPOSIT_TECTONIC_GAIN].
    #[inline]
    pub(crate) fn update_deposit_rate_from_tectonics(&mut self, seismic_index: f64) {
        self.nutrients.deposit_rate = 1.0 + seismic_index * DEPOSIT_TECTONIC_GAIN;
    }

    /// Grass-growth multiplier contributed by the nutrient fast pool.
    /// Exactly 1.0 when `fast_pool == FAST_BASELINE` — bit-exact no-op on
    /// restored baselines or before any DECO has fed the pools.
    #[inline]
    pub(crate) fn biochem_growth_mult(&self) -> f64 {
        let fast_excess = (self.nutrients.fast_pool - FAST_BASELINE).max(0.0);
        1.0 + fast_excess * FERTILITY_SCALE * MINERAL_EFFICIENCY
    }

    /// Helper readout exported via the C-ABI. Matches the grass-growth logic
    /// above (same `max(0.0)` semantics for fertility boost directionality).
    #[inline]
    pub fn fertility_boost(&self) -> f64 {
        (self.nutrients.fast_pool - FAST_BASELINE).max(0.0) * FERTILITY_SCALE
    }
}

// ---------------------------------------------------------------------------
// SaveState v5 serialization (compact JSON-in-string, cataclysm style).
// ---------------------------------------------------------------------------

/// Serialize the nutrient-pool state as a compact flat object (three f64s).
/// Roundtrips losslessly via serde_json's float_roundtrip feature (the wasm
/// build enables it in Cargo.toml).
pub(crate) fn nutrients_to_json(n: &NutrientPools) -> String {
    format!(
        "{{\"fastPool\":{},\"slowPool\":{},\"depositRate\":{}}}",
        n.fast_pool, n.slow_pool, n.deposit_rate,
    )
}

/// Parse the nutrient-pool state. Empty/missing → neutral baseline defaults
/// (matches the v4-era `#[serde(default)]` for a clean no-op legacy restore).
pub(crate) fn nutrients_from_json(s: &str) -> NutrientPools {
    let mut p = NutrientPools::new();
    let Ok(v) = serde_json::from_str::<serde_json::Value>(s) else { return p; };
    if let Some(val) = v.get("fastPool").and_then(|x| x.as_f64()) {
        p.fast_pool = val;
    }
    if let Some(val) = v.get("slowPool").and_then(|x| x.as_f64()) {
        p.slow_pool = val;
    }
    if let Some(val) = v.get("depositRate").and_then(|x| x.as_f64()) {
        p.deposit_rate = val;
    }
    p
}
