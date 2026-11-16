//! patch.rs — Phase 45: patch disturbance + resource heterogeneity.
//!
//! The 44.2 million-tick observatory pinned the "curse of stability": a
//! full-capacity steady state eliminates every vacant niche and the clade
//! speciation engine never fires (0 events / 1M ticks). Phase 45 adds the
//! missing niche dynamics at a scale BELOW the Phase 17 cataclysm layer:
//!
//! * Patch disturbance (Connell 1978 intermediate disturbance, Levins 1969
//!   patch dynamics): every tick, with per-tick probability `patch_p`, a
//!   small-radius patch fires at a random location. Grass inside the radius
//!   is wiped (a local vacancy — recolonization succeeds from surviving
//!   neighbours, each founder group a genetic bottleneck) and the mineral
//!   field inside the radius is pulsed up (post-disturbance eutrophication —
//!   the "fertile wake" that succession biology expects; it decays back
//!   through the regular `MINERAL_DECAY` path while boosting local regrowth
//!   via `GROWTH_BONUS_PER_MINERAL`).
//!
//! * Resource heterogeneity (static field): the per-cell grass regrowth rate
//!   carries a fixed-seed smooth noise multiplier, so different regions of
//!   the world hold different carrying capacities even at steady state —
//!   spatial selection gradients the detector can split on.
//!
//! Determinism discipline (mirrors cataclysm.rs): one dedicated RNG stream
//! (`seed ^ PATCH_RNG_TAG`), exactly one trigger draw per tick plus two
//! location draws per fired event. Grass/mineral edits flow through the
//! regular fields, so the checksum trajectory moves only through real
//! dynamics (approved tenth re-baseline). Persistence: SaveState v15 carries
//! the toggle, probability, stream state and event counter.

use crate::ecology::{Ecology, GRID_N};

/// Dedicated patch-stream tag (`seed ^ PATCH_RNG_TAG`). Never mixed with the
/// main/prng/fish/culture/abio/group/brain streams.
pub const PATCH_RNG_TAG: u32 = 0x50_41_54_43; // "PATC"

/// Patch layer toggle. Default on (this phase deliberately changes the
/// dynamics — the tenth re-baseline).
pub const PATCH_ON: bool = true;

/// Per-tick probability of one patch event. Calibration target: intermediate
/// disturbance — enough events per 100k ticks to keep patches in an
/// extinction-colonization cycle, few enough that the world is never
/// globally grazed to zero.
pub const PATCH_P: f64 = 0.001;

/// Patch radius in grid cells (CELL ≈ 14.2 world units, so 4 cells ≈ 57).
pub const PATCH_RADIUS_CELLS: f64 = 4.0;

/// Post-disturbance eutrophication: the mineral pulse added to every cell
/// inside the radius (clamped to [0,1], decays via MINERAL_DECAY). Scale
/// calibration: below the intensity of the hotspot DECO itself constructs
/// (deposit-fed cells sit at 0.5-1.0), so the random pulses enrich the soil
/// without overwriting the mineral landscape that DECO's local adaptation
/// lives on — at 0.5 per event the pulses flattened that landscape and the
/// first natural DECO split never fired (Phase-45 calibration runs).
pub const PATCH_MINERAL_PULSE: f64 = 0.1;

/// Amplitude of the static regrowth-noise field: per-cell multiplier drawn
/// from [1-AMP, 1+AMP] over a smooth bilinear lattice.
pub const REGROW_NOISE_AMP: f64 = 0.15;

/// Lattice step (in cells) of the bilinear noise field — 8 cells ≈ 114 world
/// units, giving region-scale (not cell-scale) heterogeneity.
pub const REGROW_NOISE_LATTICE_STEP: usize = 8;

/// Fixed seed of the static heterogeneity field. The field is a constant of
/// the MODEL, not a per-world random variable: every seed and every restore
/// derives the identical landscape, no SaveState space is spent on it.
pub const REGROW_NOISE_SEED: u32 = 0x4845_5452; // "HETR"

/// The static per-cell regrowth-noise multiplier field (bilinear lattice).
/// Pure function of REGROW_NOISE_SEED — identical across worlds and restores.
pub fn generate_regrow_noise() -> Vec<f32> {
    let n = (GRID_N * GRID_N) as usize;
    let ln = (GRID_N as usize) / REGROW_NOISE_LATTICE_STEP + 2;
    let mut rng = aether_rng::Mulberry32::new(REGROW_NOISE_SEED);
    let lo = (1.0 - REGROW_NOISE_AMP) as f32;
    let span = (2.0 * REGROW_NOISE_AMP) as f32;
    let lat: Vec<f32> = (0..ln * ln)
        .map(|_| lo + rng.next_f64() as f32 * span)
        .collect();
    let mut out = Vec::with_capacity(n);
    for row in 0..GRID_N as usize {
        for col in 0..GRID_N as usize {
            let u = col as f64 / REGROW_NOISE_LATTICE_STEP as f64;
            let v = row as f64 / REGROW_NOISE_LATTICE_STEP as f64;
            let li = (u as usize).min(ln - 2);
            let lj = (v as usize).min(ln - 2);
            let fu = u - li as f64;
            let fv = v - lj as f64;
            let a = lat[lj * ln + li] as f64;
            let b = lat[lj * ln + li + 1] as f64;
            let c = lat[(lj + 1) * ln + li] as f64;
            let d = lat[(lj + 1) * ln + li + 1] as f64;
            let val = a * (1.0 - fu) * (1.0 - fv)
                + b * fu * (1.0 - fv)
                + c * (1.0 - fu) * fv
                + d * fu * fv;
            out.push(val as f32);
        }
    }
    out
}

impl Ecology {
    /// Per-substep patch roll (called from `update`, after the cataclysm
    /// countdown). One trigger draw per tick; a fired event takes two more
    /// draws for the patch center. Zero cost when `patch_on` is false
    /// (no draw at all — the A/B arm stays bit-exact with pre-Phase-45).
    pub(crate) fn tick_patch(&mut self) {
        if !self.patch_on { return; }
        if self.patch_rng.next_f64() >= self.patch_p { return; }
        let col = (self.patch_rng.next_f64() * GRID_N as f64) as u32;
        let row = (self.patch_rng.next_f64() * GRID_N as f64) as u32;
        self.fire_patch(col, row);
    }

    /// Apply one patch event centered on cell (col,row): grass wipe +
    /// mineral pulse inside `PATCH_RADIUS_CELLS`. Cell coverage is decided
    /// on cell centers, matching the cataclysm blast convention.
    pub fn fire_patch(&mut self, col: u32, row: u32) {
        let half = crate::ecology::GRID_SPAN / 2.0;
        let cell = crate::ecology::CELL;
        let cx = -half + (col as f64 + 0.5) * cell;
        let cz = -half + (row as f64 + 0.5) * cell;
        let r = PATCH_RADIUS_CELLS * cell;
        let r2 = r * r;
        let n = (GRID_N * GRID_N) as usize;
        let pn = self.plants.len();
        for idx in 0..n {
            let c = (idx % GRID_N as usize) as f64;
            let w = (idx / GRID_N as usize) as f64;
            let x = -half + (c + 0.5) * cell;
            let z = -half + (w + 0.5) * cell;
            let dx = x - cx;
            let dz = z - cz;
            if dx * dx + dz * dz > r2 { continue; }
            for sp in 0..pn { self.grass_p[sp][idx] = 0.0; }
            self.grass[idx] = 0.0;
            let m = (self.mineral_field[idx] as f64 + PATCH_MINERAL_PULSE).min(1.0);
            self.mineral_field[idx] = m as f32;
        }
        self.patch_events += 1;
        self.patch_last_tick = self.tick_count();
        self.patch_last_x = cx;
        self.patch_last_z = cz;
    }

    /// Cumulative patch events (telemetry).
    pub fn patch_event_count(&self) -> u64 {
        self.patch_events
    }
}
