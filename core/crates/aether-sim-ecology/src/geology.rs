//! geology.rs — Phase 18: plate tectonics + eustatic sea level + biome classes.
//!
//! Fourth pillar after speciation/competition/cataclysm: the GEOLOGIC time
//! layer. Plates drift, converge (orogeny → elevation delta → suit
//! re-derivation → persistent geographic barriers feeding the Phase 15
//! speciation engine's geographic-isolation gate), diverge (rift subsidence →
//! future sea inlets), and drive a deterministic eustatic sea-level cycle.
//!
//! Determinism contract (mirrors cataclysm.rs):
//!   * `geology_step` consumes ZERO draws from rng/prng/fish_rng. Plate
//!     initialization uses a DEDICATED Mulberry32 stream seeded from the sim
//!     seed (`seed ^ GEO_RNG_TAG`) — never the ecology streams — so the
//!     checksum trajectory of a geology-free run is bit-identical to the
//!     Phase 17 baseline (zero-side-effect contract: with GEO_TICKS_PER_STEP
//!     → ∞ the ecology layer never sees the geology layer at all).
//!   * Plate motion / collisions / uplift / sea level are pure arithmetic on
//!     the plate state; earthquakes reuse the Phase 17 cataclysm engine
//!     (`trigger_cataclysm`, kind impact) at deterministic collision points.
//!   * `geology_step` is only invoked from `Ecology::update` AFTER a complete
//!     GEO_TICKS_PER_STEP tick interval (never inside a substep), so the
//!     ecology checksum trajectory stays segment-continuous across boundaries.
//!
//! Sea-level scope cut (documented for Phase 18 review): the eustatic cycle
//! drives `compute_ocean_mask` (rendering + geographic readout) but NOT the
//! suit grid — only tectonic uplift/subsidence (`elevation_delta`) re-derives
//! suitability, so a zero-velocity plate world stays bit-exact vs Phase 17.

use crate::ecology::{Ecology, CELL, CATACLYSM_IMPACT, GRASS_N, GRASS_SPECIES, GRID_N, GRID_SPAN};
use crate::terrain::{plant_suitability, terrain_height, WATER_LEVEL};
use aether_rng::Mulberry32;

/// Ecology ticks per geologic step (10k ticks ≈ 1000 sim-seconds).
pub const GEO_TICKS_PER_STEP: u32 = 10_000;
/// Hard cap on tracked plates (O(n²) collision scan is fine at this size).
pub const PLATE_CAP: usize = 8;

pub const PLATE_CONTINENTAL: u8 = 0;
pub const PLATE_OCEANIC: u8 = 1;
pub const PLATE_RIFT: u8 = 2;

/// Per-cell biome classification (derived from effective height, stateless).
pub const BIOME_CLASS_OCEAN: u8 = 0;
pub const BIOME_CLASS_COAST: u8 = 1;
pub const BIOME_CLASS_FOREST: u8 = 2;
pub const BIOME_CLASS_ALPINE: u8 = 3;

/// Dedicated geology RNG tag: `seed ^ GEO_RNG_TAG` seeds the plate-perturb
/// stream. ASCII "GEO1" — never collides with the ecology stream tags.
const GEO_RNG_TAG: u32 = 0x4745_4F31;
/// Plate radii by type (world units; GRID_SPAN = 1080, half-span = 540).
const RADIUS_CONTINENTAL: f64 = 240.0;
const RADIUS_OCEANIC: f64 = 170.0;
const RADIUS_RIFT: f64 = 150.0;
/// Uplift gain: delta += 0.5 × approach speed per collision step (spec).
const UPLIFT_GAIN: f64 = 0.5;
/// Max accumulated uplift per cell (orogeny saturates, no runaway peaks).
const UPLIFT_MAX: f64 = 80.0;
/// Rift subsidence per step within the rift plate's radius (sea-inlet carve).
const RIFT_SUBSIDENCE: f64 = 0.8;
/// Max accumulated subsidence per cell.
const SUBSIDENCE_MAX: f64 = -60.0;
/// Eustatic cycle: amplitude (world units) / period in geologic steps.
const SEA_LEVEL_AMP: f64 = 40.0;
const SEA_LEVEL_PERIOD: f64 = 64.0;
/// Earthquake parameters (reuses the Phase 17 impact cataclysm).
const QUAKE_RADIUS: f64 = 45.0;
const QUAKE_SEVERITY_GAIN: f64 = 0.06;
const QUAKE_SEVERITY_MAX: f64 = 0.25;
const QUAKE_DURATION: u32 = 200;
/// Uplift application radius around a collision midpoint (world units).
const UPLIFT_RADIUS: f64 = 60.0;
/// Seismic index = collisions over the last SEISMIC_WINDOW geo steps.
const SEISMIC_WINDOW: usize = 10;

/// One lithospheric plate. Velocities are world-units per GEO step.
#[derive(Clone, Debug)]
pub struct Plate {
    pub id: u32,
    pub x: f64,
    pub z: f64,
    pub vx: f64,
    pub vz: f64,
    pub angle: f64,
    pub type_: u8,
}

impl Plate {
    #[inline]
    pub fn radius(&self) -> f64 {
        match self.type_ {
            PLATE_OCEANIC => RADIUS_OCEANIC,
            PLATE_RIFT => RADIUS_RIFT,
            _ => RADIUS_CONTINENTAL,
        }
    }
}

/// Geologic layer state. All fields are public so tests can configure
/// scenarios directly (mechanism-level injection, Phase 16 lesson).
#[derive(Clone, Debug)]
pub struct GeologicState {
    pub plates: Vec<Plate>,
    /// ±Δ relative to the base coastline; positive = marine transgression.
    pub sea_level: f64,
    /// Read-only display label, refreshed at each geologic step.
    pub epoch_name: &'static str,
    /// Ecology tick_count at the last completed geologic step (0 = never).
    pub last_geo_tick: u32,
    /// Collisions in the last SEISMIC_WINDOW steps (ring, cursor = next slot).
    pub(crate) collision_ring: [u32; SEISMIC_WINDOW],
    pub(crate) collision_cursor: usize,
    /// Smoothed tectonic activity in [0, 1]: ring sum / SEISMIC_WINDOW.
    pub seismic_index: f64,
}

impl GeologicState {
    /// Initial 3-plate configuration (spec: Pangaea core + one convergent
    /// oceanic plate + one westward-rifting plate). Hardcoded layout with a
    /// ≤5-draw jitter from the DEDICATED geology stream (never the ecology
    /// streams — zero-side-effect contract).
    pub fn new(seed: u32) -> Self {
        let mut grng = Mulberry32::new(seed ^ GEO_RNG_TAG);
        // 5 draws total: center jitter (2) + velocity jitter (3).
        let jx = (grng.next_f64() - 0.5) * 40.0;
        let jz = (grng.next_f64() - 0.5) * 40.0;
        let jv1 = (grng.next_f64() - 0.5) * 0.4;
        let jv2 = (grng.next_f64() - 0.5) * 0.4;
        let jv3 = (grng.next_f64() - 0.5) * 0.4;
        let plates = vec![
            // 1. Pangaea-like continental core near the origin, drifting east
            //    slowly (converges on the oceanic plate from the west).
            Plate { id: 1, x: jx, z: jz, vx: 0.4, vz: 0.0, angle: 0.0, type_: PLATE_CONTINENTAL },
            // 2. Oceanic plate east of the core, driving west: the convergent
            //    boundary that will raise the future orogenic belt.
            Plate { id: 2, x: 430.0 + jx * 0.5, z: 60.0 + jz * 0.5, vx: -3.0 + jv1, vz: jv2 * 0.5, angle: 0.0, type_: PLATE_OCEANIC },
            // 3. Rift plate on the west flank, drifting away from the core:
            //    divergent boundary carving the future western sea inlet.
            Plate { id: 3, x: -420.0, z: -120.0, vx: -0.8 + jv3, vz: 0.0, angle: 0.0, type_: PLATE_RIFT },
        ];
        GeologicState {
            plates,
            sea_level: 0.0,
            epoch_name: "Pangaea Assembly",
            last_geo_tick: 0,
            collision_ring: [0; SEISMIC_WINDOW],
            collision_cursor: 0,
            seismic_index: 0.0,
        }
    }

    /// Geologic step index implied by `last_geo_tick` (0 before the first
    /// step; survives save/restore without an extra field).
    #[inline]
    pub fn step_index(&self) -> u32 {
        self.last_geo_tick / GEO_TICKS_PER_STEP
    }
}

/// Epoch label for a geologic step index (deterministic, no RNG).
pub fn epoch_of_step(step: u32) -> &'static str {
    if step < 8 {
        "Pangaea Assembly"
    } else if step < 24 {
        "Rift & Drift"
    } else if step < 64 {
        "Marine Cycle"
    } else {
        "Open Seaway"
    }
}

// ---------------------------------------------------------------------------
// Ecology integration
// ---------------------------------------------------------------------------

impl Ecology {
    /// Advance one geologic step: drift → collisions (uplift + quake) →
    /// rift subsidence → eustatic sea level → seismic index → deposit-rate
    /// feedback → topography/biome rebuild. Zero RNG draws.
    /// No-op (silent) when the plate table is empty — legacy v4 restores.
    pub fn geology_step(&mut self) {
        if self.geo.plates.is_empty() {
            return;
        }
        let half = GRID_SPAN / 2.0;

        // 1. Drift + bounds clamp (velocity zeroes at the world rim).
        for p in self.geo.plates.iter_mut() {
            p.x += p.vx;
            p.z += p.vz;
            if p.x < -half { p.x = -half; if p.vx < 0.0 { p.vx = 0.0; } }
            if p.x > half { p.x = half; if p.vx > 0.0 { p.vx = 0.0; } }
            if p.z < -half { p.z = -half; if p.vz < 0.0 { p.vz = 0.0; } }
            if p.z > half { p.z = half; if p.vz > 0.0 { p.vz = 0.0; } }
        }

        // 2. Collisions: O(n²) brute force (n ≤ PLATE_CAP = 8).
        let n = self.geo.plates.len();
        let mut collisions: Vec<(f64, f64, f64)> = Vec::new(); // (mx, mz, approach)
        let mut collision_count = 0u32;
        let snapshot: Vec<(f64, f64, f64)> = self.geo.plates.iter()
            .map(|p| (p.x, p.z, p.radius()))
            .collect();
        for i in 0..n {
            for j in (i + 1)..n {
                let (ax, az, ar) = snapshot[i];
                let (bx, bz, br) = snapshot[j];
                let dx = bx - ax;
                let dz = bz - az;
                let dist = (dx * dx + dz * dz).sqrt();
                if dist >= ar + br || dist == 0.0 {
                    continue;
                }
                // Relative velocity along the center line (approach > 0).
                let rvx = self.geo.plates[j].vx - self.geo.plates[i].vx;
                let rvz = self.geo.plates[j].vz - self.geo.plates[i].vz;
                let nx = dx / dist;
                let nz = dz / dist;
                let approach = -(rvx * nx + rvz * nz);
                if approach <= 0.0 {
                    continue; // already separating — no new collision energy
                }
                // Elastic-ish bounce: reflect the normal components, damped 0.5.
                let imp_i = self.geo.plates[i].vx * nx + self.geo.plates[i].vz * nz;
                let imp_j = self.geo.plates[j].vx * nx + self.geo.plates[j].vz * nz;
                let d = 0.5 * (imp_i - imp_j);
                self.geo.plates[i].vx -= d * nx;
                self.geo.plates[i].vz -= d * nz;
                self.geo.plates[j].vx += d * nx;
                self.geo.plates[j].vz += d * nz;
                let mx = 0.5 * (ax + bx);
                let mz = 0.5 * (az + bz);
                collisions.push((mx, mz, approach));
                collision_count += 1;
            }
        }

        // 3. Uplift at collision midpoints + quake injection (cataclysm engine).
        let mut delta_changed = false;
        for &(mx, mz, approach) in &collisions {
            self.apply_uplift(mx, mz, UPLIFT_RADIUS, UPLIFT_GAIN * approach, UPLIFT_MAX);
            delta_changed = true;
            let severity = (approach * QUAKE_SEVERITY_GAIN).clamp(0.05, QUAKE_SEVERITY_MAX);
            // Deterministic earthquake: reuses the Phase 17 impact path
            // (grass/mineral wipe + energy damage through the regular die()).
            self.trigger_cataclysm(CATACLYSM_IMPACT, mx, mz, QUAKE_RADIUS, severity, QUAKE_DURATION);
        }

        // 4. Rift subsidence: rift plates carve a depression under themselves.
        // Snapshot rift plates to avoid simultaneous immutable + mutable borrow of `self`.
        let rifts: Vec<(f64, f64, f64)> = self.geo.plates.iter()
            .filter(|p| p.type_ == PLATE_RIFT)
            .map(|p| (p.x, p.z, p.radius()))
            .collect();
        for (px, pz, r) in &rifts {
            self.apply_uplift(*px, *pz, r * 0.6, -RIFT_SUBSIDENCE, SUBSIDENCE_MAX);
            delta_changed = true;
        }

        // 5. Eustatic sea level: deterministic sinusoid over the step index.
        let step = self.geo.step_index() + 1;
        self.geo.sea_level = SEA_LEVEL_AMP
            * aether_libm::sin((step as f64 / SEA_LEVEL_PERIOD) * std::f64::consts::PI * 2.0);

        // 6. Seismic index ring + deposit-rate feedback (biochemistry read).
        self.geo.collision_ring[self.geo.collision_cursor] = collision_count;
        self.geo.collision_cursor = (self.geo.collision_cursor + 1) % SEISMIC_WINDOW;
        let sum: u32 = self.geo.collision_ring.iter().sum();
        self.geo.seismic_index = (sum as f64 / SEISMIC_WINDOW as f64).min(1.0);
        self.update_deposit_rate_from_tectonics(self.geo.seismic_index);

        // 7. Topography → suit/moist/biome re-derivation (bit-identical no-op
        //    when elevation_delta is still all zeros).
        if delta_changed {
            self.rebuild_biome_registry();
        }

        // 8. Bookkeeping: tick stamp + epoch label.
        self.geo.last_geo_tick = self.tick_count();
        self.geo.epoch_name = epoch_of_step(self.geo.step_index());
    }

    /// Add `amount` to `elevation_delta` inside `radius` of (cx, cz),
    /// clamped to `limit` per cell (positive = uplift, negative = subsidence).
    pub(crate) fn apply_uplift(&mut self, cx: f64, cz: f64, radius: f64, amount: f64, limit: f64) {
        let half = GRID_SPAN / 2.0;
        let r2 = radius * radius;
        let n = (GRID_N * GRID_N) as usize;
        for idx in 0..n {
            let col = (idx % GRID_N as usize) as f64;
            let row = (idx / GRID_N as usize) as f64;
            let cellx = -half + (col + 0.5) * CELL;
            let cellz = -half + (row + 0.5) * CELL;
            let dx = cellx - cx;
            let dz = cellz - cz;
            if dx * dx + dz * dz > r2 {
                continue;
            }
            let cur = self.elevation_delta[idx] as f64;
            let next = if amount >= 0.0 {
                (cur + amount).min(limit)
            } else {
                (cur + amount).max(limit)
            };
            self.elevation_delta[idx] = next as f32;
        }
    }

    /// Effective terrain height at (x, z): base noise terrain + tectonic delta.
    #[inline]
    pub fn height_at(&self, x: f64, z: f64) -> f64 {
        let half = GRID_SPAN / 2.0;
        let col = ((x + half) / CELL).floor();
        let row = ((z + half) / CELL).floor();
        if col < 0.0 || row < 0.0 || col >= GRID_N as f64 || row >= GRID_N as f64 {
            return terrain_height(x, z);
        }
        let idx = (row as usize) * GRID_N as usize + col as usize;
        terrain_height(x, z) + self.elevation_delta[idx] as f64
    }

    /// Overwrite the tectonic elevation delta layer (spec signature: f64 in,
    /// stored as f32 per cell — f64→f32 is the storage precision) and
    /// re-derive the terrain-dependent layers.
    pub(crate) fn apply_topographic_mask(&mut self, new_elevation_map: &[f64]) {
        let n = (GRID_N * GRID_N) as usize;
        debug_assert_eq!(new_elevation_map.len(), n, "topographic mask length");
        for (i, &v) in new_elevation_map.iter().take(n).enumerate() {
            self.elevation_delta[i] = v as f32;
        }
        self.rebuild_biome_registry();
    }

    /// Re-derive the terrain-dependent layers from the current topography:
    /// suit / moist / suit_eff_by (bit-identical to `init_grass` when the
    /// delta layer is all zeros — pure functions, zero RNG). The biome-class
    /// view is stateless (`biome_class_at`); the BiomeRegistry itself is
    /// user-authored + aquatic and is intentionally left untouched.
    pub(crate) fn rebuild_biome_registry(&mut self) {
        let half = GRID_SPAN / 2.0;
        // Suit re-derivation (same expression as init_grass, delta-aware).
        for j in 0..GRID_N as usize {
            for i in 0..GRID_N as usize {
                let x = -half + (i as f64 + 0.5) * CELL;
                let z = -half + (j as f64 + 0.5) * CELL;
                let idx = j * GRID_N as usize + i;
                let h = self.height_at(x, z);
                self.suit_mut()[idx] = plant_suitability_delta_aware(x, z, h) as f32;
            }
        }
        // Water-edge moisture BFS (verbatim init_grass logic, delta-aware);
        // moisture resets first so re-derivation is idempotent.
        for m in self.moist_mut().iter_mut() {
            *m = 0.0;
        }
        let mut frontier: Vec<usize> = Vec::new();
        for j in 0..GRID_N as usize {
            for i in 0..GRID_N as usize {
                let x = -half + (i as f64 + 0.5) * CELL;
                let z = -half + (j as f64 + 0.5) * CELL;
                if self.height_at(x, z) < WATER_LEVEL + 1.2 {
                    frontier.push(j * GRID_N as usize + i);
                }
            }
        }
        let mut d = 0;
        while d < 12 && !frontier.is_empty() {
            let mut next = Vec::new();
            for &idx in &frontier {
                if self.moist_mut()[idx] > 0.0 { continue; }
                self.moist_mut()[idx] = (0.45 - d as f64 * 0.035) as f32;
                let i = (idx % GRID_N as usize) as isize;
                let j = (idx / GRID_N as usize) as isize;
                for (ni, nj) in [(i + 1, j), (i - 1, j), (i, j + 1), (i, j - 1)] {
                    if ni >= 0 && nj >= 0 && (ni as u32) < GRID_N && (nj as u32) < GRID_N {
                        next.push((nj as usize) * GRID_N as usize + ni as usize);
                    }
                }
            }
            frontier = next;
            d += 1;
        }
        // suit_eff_by refresh with the re-derived moisture (init_grass formula).
        for idx in 0..(GRID_N * GRID_N) as usize {
            let s = self.suit_mut()[idx] as f64;
            if s <= 0.0 {
                for sp in 0..GRASS_N { self.suit_eff_by_mut(sp)[idx] = 0.0; }
                continue;
            }
            let m = self.moist_mut()[idx] as f64;
            for sp in 0..GRASS_N {
                let tr = &GRASS_SPECIES[sp];
                let exp_arg = -(((m - tr.moist_opt) / tr.moist_w).powi(2));
                let eff = aether_libm::exp(exp_arg);
                self.suit_eff_by_mut(sp)[idx] = (s * eff) as f32;
            }
        }
    }

    /// Cells submerged at the current sea level (base terrain + delta
    /// compared against WATER_LEVEL + sea_level). Pure readout: drives the
    /// minimap + geographic-isolation analysis, NOT the suit grid (scope cut
    /// documented in the module header).
    pub fn compute_ocean_mask(&self) -> Vec<bool> {
        let half = GRID_SPAN / 2.0;
        let surface = WATER_LEVEL + self.geo.sea_level;
        let n = (GRID_N * GRID_N) as usize;
        let mut mask = vec![false; n];
        for j in 0..GRID_N as usize {
            for i in 0..GRID_N as usize {
                let x = -half + (i as f64 + 0.5) * CELL;
                let z = -half + (j as f64 + 0.5) * CELL;
                let idx = j * GRID_N as usize + i;
                mask[idx] = self.height_at(x, z) < surface;
            }
        }
        mask
    }

    /// Stateless per-cell biome class from the effective height (aligned with
    /// the plant_suitability cutoffs: <WATER+1.2 ocean, <WATER+7 coast,
    /// >78 alpine, else forest).
    pub fn biome_class_at(&self, idx: usize) -> u8 {
        let half = GRID_SPAN / 2.0;
        let col = (idx % GRID_N as usize) as f64;
        let row = (idx / GRID_N as usize) as f64;
        let x = -half + (col + 0.5) * CELL;
        let z = -half + (row + 0.5) * CELL;
        let h = self.height_at(x, z);
        if h < WATER_LEVEL + 1.2 {
            BIOME_CLASS_OCEAN
        } else if h > 78.0 {
            BIOME_CLASS_ALPINE
        } else if h < WATER_LEVEL + 7.0 {
            BIOME_CLASS_COAST
        } else {
            BIOME_CLASS_FOREST
        }
    }

    // ---- test/external accessors (Ecology fields stay pub(crate)) --------

    /// Read-only view of the plate table (for tests / C-ABI).
    pub fn geo_plates(&self) -> &[Plate] { &self.geo.plates }
    /// Replace the plate table (mechanism-level test injection).
    pub fn set_geo_plates(&mut self, plates: Vec<Plate>) { self.geo.plates = plates; }
    /// Current eustatic sea-level anomaly.
    pub fn geo_sea_level(&self) -> f64 { self.geo.sea_level }
    /// Override sea level directly (test injection).
    pub fn set_geo_sea_level(&mut self, v: f64) { self.geo.sea_level = v; }
    /// Tick stamp of the last completed geologic step (0 = never stepped).
    pub fn geo_last_step_tick(&self) -> u32 { self.geo.last_geo_tick }
    /// Smoothed tectonic activity in [0, 1].
    pub fn geo_seismic_index(&self) -> f64 { self.geo.seismic_index }
    /// Read a single elevation-delta cell (tests + minimap probes).
    #[inline]
    pub fn elevation_delta_at(&self, idx: usize) -> f32 {
        self.elevation_delta.get(idx).copied().unwrap_or(0.0)
    }
    /// Write a single elevation-delta cell (mechanism-level test injection).
    #[inline]
    pub fn set_elevation_delta_at(&mut self, idx: usize, v: f32) {
        if idx < self.elevation_delta.len() {
            self.elevation_delta[idx] = v;
        }
    }
    /// Public re-derivation hook (tests drive it after direct delta writes;
    /// production code calls it from `geology_step` / `apply_topographic_mask`).
    pub fn recompute_biome_layers(&mut self) {
        self.rebuild_biome_registry();
    }
}

/// Suitability re-derivation with a delta-aware height. Mirrors
/// `terrain::plant_suitability` exactly, but takes the effective height as an
/// argument so tectonic uplift/subsidence feeds through. With delta = 0 the
/// result is bit-identical to `plant_suitability(x, z)`.
fn plant_suitability_delta_aware(x: f64, z: f64, h: f64) -> f64 {
    use crate::terrain::{clamp, js_hypot, slope_at, smoothstep};
    if h < WATER_LEVEL + 1.2 || h > 78.0 {
        return 0.0;
    }
    let s = slope_at(x, z);
    let wet = smoothstep(230.0, 120.0, js_hypot(x, z)) * 0.35 + 0.65;
    clamp(
        smoothstep(WATER_LEVEL + 1.2, WATER_LEVEL + 7.0, h)
            * (1.0 - smoothstep(0.42, 0.85, s))
            * (1.0 - smoothstep(58.0, 88.0, h))
            * wet,
        0.0,
        1.0,
    )
}

// ---------------------------------------------------------------------------
// SaveState v5 serialization (compact JSON-in-string, cataclysm style)
// ---------------------------------------------------------------------------

/// Serialize the geologic layer: plates + sea level + epoch + last tick +
/// the SPARSE uplift map (`[[idx, delta], …]` — usually tiny).
pub(crate) fn geology_to_json(geo: &GeologicState, uplift: &[f32]) -> String {
    let plates: Vec<String> = geo.plates.iter().map(|p| {
        format!(
            "{{\"id\":{},\"x\":{},\"z\":{},\"vx\":{},\"vz\":{},\"angle\":{},\"type\":{}}}",
            p.id, p.x, p.z, p.vx, p.vz, p.angle, p.type_,
        )
    }).collect();
    let uplift_pairs: Vec<String> = uplift.iter().enumerate()
        .filter(|(_, &v)| v != 0.0)
        .map(|(i, &v)| format!("[{},{}]", i, v))
        .collect();
    format!(
        "{{\"plates\":[{}],\"seaLevel\":{},\"epoch\":\"{}\",\"lastGeoTick\":{},\"uplift\":[{}]}}",
        plates.join(","),
        geo.sea_level,
        geo.epoch_name,
        geo.last_geo_tick,
        uplift_pairs.join(","),
    )
}

/// Parse the geologic layer JSON. Malformed input restores an empty-plate
/// (geology-inert) state, matching the v4-era serde default.
pub(crate) fn geology_from_json(s: &str, n_cells: usize) -> (GeologicState, Vec<f32>) {
    let mut state = GeologicState {
        plates: Vec::new(),
        sea_level: 0.0,
        epoch_name: "Pangaea Assembly",
        last_geo_tick: 0,
        collision_ring: [0; SEISMIC_WINDOW],
        collision_cursor: 0,
        seismic_index: 0.0,
    };
    let mut uplift: Vec<f32> = vec![0.0; n_cells];
    let Ok(v) = serde_json::from_str::<serde_json::Value>(s) else {
        return (state, uplift);
    };
    if let Some(arr) = v.get("plates").and_then(|p| p.as_array()) {
        for p in arr {
            let Some(p) = p.as_object() else { continue };
            state.plates.push(Plate {
                id: p.get("id").and_then(|x| x.as_u64()).unwrap_or(0) as u32,
                x: p.get("x").and_then(|x| x.as_f64()).unwrap_or(0.0),
                z: p.get("z").and_then(|x| x.as_f64()).unwrap_or(0.0),
                vx: p.get("vx").and_then(|x| x.as_f64()).unwrap_or(0.0),
                vz: p.get("vz").and_then(|x| x.as_f64()).unwrap_or(0.0),
                angle: p.get("angle").and_then(|x| x.as_f64()).unwrap_or(0.0),
                type_: p.get("type").and_then(|x| x.as_u64()).unwrap_or(0) as u8,
            });
        }
    }
    state.sea_level = v.get("seaLevel").and_then(|x| x.as_f64()).unwrap_or(0.0);
    state.last_geo_tick = v.get("lastGeoTick").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
    state.epoch_name = if state.plates.is_empty() {
        "Pangaea Assembly"
    } else {
        epoch_of_step(state.step_index())
    };
    if let Some(arr) = v.get("uplift").and_then(|u| u.as_array()) {
        for pair in arr {
            let (Some(i), Some(val)) = (
                pair.get(0).and_then(|x| x.as_u64()),
                pair.get(1).and_then(|x| x.as_f64()),
            ) else { continue };
            let idx = i as usize;
            if idx < n_cells {
                uplift[idx] = val as f32;
            }
        }
    }
    (state, uplift)
}
