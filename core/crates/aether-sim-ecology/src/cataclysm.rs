//! cataclysm.rs — Phase 17: extinction events (cataclysm) + recovery dynamics.
//!
//! Third pillar of the macro-evolution triptych (speciation → competition →
//! extinction & recovery). Cataclysms are experimenter-injected through the
//! C-ABI (`sim_trigger_cataclysm`) — the engine never fires one on its own —
//! and all bookkeeping here is deterministic: no RNG draws, plain Vec scans
//! in trigger order, so the seeded RNG streams are untouched. With no active
//! cataclysm every environment multiplier is exactly 1.0 and `x * 1.0` is
//! bit-exact f64, so cataclysm-free runs keep their checksum trajectory.
//!
//! Two event kinds:
//!   * impact (kind 0) — immediate blast: grass/mineral cells inside the
//!     radius are scaled down by (1 − severity) and every agent inside takes
//!     `severity × CATACLYSM_DAMAGE` energy damage (deaths reuse the regular
//!     `die()` path, cause "cataclysm", so carcass/mineral feedback stays
//!     intact). Followed by a dust winter: the daylight multiplier drops to
//!     `1 − severity` at injection and recovers linearly to 1.0 over
//!     `dust_recovery_ticks`; while dust is up, a metabolic stress factor
//!     `(1 − dust) × CATACLYSM_STRESS_SCALE` multiplies every drain — and
//!     every drain scales with the genome's `metabolism` gene, which yields
//!     the measurable K-Pg-style survivor bias against high-maintenance
//!     phenotypes.
//!   * climate shift (kind 1) — global grass-growth throttle: the logistic
//!     growth rate is multiplied by (1 + climate_delta) = (1 − severity)
//!     while the event is active, then snaps back. No direct mortality; the
//!     starvation cascade does the killing.
//!
//! Persistence: SaveState v4 carries the full table (history + live effect
//! state) as the compact JSON string `Cataclysms`; pre-v4 saves restore with
//! an empty table (serde default) and keep running identically.

use crate::ecology::{
    Cataclysm, CELL, Ecology, CATACLYSM_CLIMATE, CATACLYSM_DAMAGE, CATACLYSM_IMPACT,
    CATACLYSM_STRESS_SCALE, GRID_N, GRID_SPAN,
};

impl Ecology {
    /// Inject a cataclysm and apply its immediate effects. Returns the
    /// cataclysm id (≥ 1), or 0 on invalid parameters. Deterministic: same
    /// seed + same injection sequence → same checksum trajectory.
    ///
    /// `severity` ∈ [0, 1] scales the blast/dust for impacts and doubles as
    /// the negative growth delta for climate shifts (`climate_delta =
    /// -severity`). `duration` is the live-effect window in ticks (dust
    /// recovery for impacts, throttled growth for climate shifts).
    pub fn trigger_cataclysm(
        &mut self,
        kind: u8,
        x: f64,
        z: f64,
        radius: f64,
        severity: f64,
        duration: u32,
    ) -> u32 {
        if kind > CATACLYSM_CLIMATE { return 0; }
        if !(0.0..=1.0).contains(&severity) { return 0; }
        if radius < 0.0 || duration == 0 { return 0; }
        if kind == CATACLYSM_IMPACT {
            self.impact_blast(x, z, radius, severity);
        }
        let id = self.next_cataclysm_id;
        self.next_cataclysm_id += 1;
        self.cataclysms.push(Cataclysm {
            id,
            kind,
            tick: self.tick_count(),
            x,
            z,
            radius,
            severity,
            dust_recovery_ticks: if kind == CATACLYSM_IMPACT { duration } else { 0 },
            climate_delta: if kind == CATACLYSM_CLIMATE { -severity } else { 0.0 },
            ends_at: self.tick_count() + duration,
            remaining: duration,
            active: true,
        });
        id
    }

    /// Immediate impact effects: resource wipe inside the radius + direct
    /// energy damage (deaths go through the regular `die()` path so carcass
    /// creation, occupancy-lock release and fitness records stay consistent).
    /// Cell coverage is decided on cell centers, matching the grass-grid
    /// convention used by the foraging/deposit code.
    fn impact_blast(&mut self, x: f64, z: f64, radius: f64, severity: f64) {
        let keep = (1.0 - severity) as f32;
        let half = GRID_SPAN / 2.0;
        let r2 = radius * radius;
        let n = (GRID_N * GRID_N) as usize;
        for idx in 0..n {
            let col = (idx % GRID_N as usize) as f64;
            let row = (idx / GRID_N as usize) as f64;
            let cx = -half + (col + 0.5) * CELL;
            let cz = -half + (row + 0.5) * CELL;
            let dx = cx - x;
            let dz = cz - z;
            if dx * dx + dz * dz > r2 { continue; }
            for sp in 0..self.plants.len() { self.grass_p[sp][idx] *= keep; }
            self.grass[idx] *= keep;
            self.mineral_field[idx] *= keep;
        }
        let hit: Vec<usize> = self.agents.iter().enumerate()
            .filter(|(_, a)| {
                if !a.alive { return false; }
                let dx = a.x - x;
                let dz = a.z - z;
                (dx * dx + dz * dz).sqrt() <= radius
            })
            .map(|(i, _)| i)
            .collect();
        for i in hit {
            self.agents[i].energy -= severity * CATACLYSM_DAMAGE;
            if self.agents[i].energy <= 0.0 {
                self.die(i, "cataclysm");
            }
        }
    }

    /// Per-substep countdown (called from `update`). Pure bookkeeping, zero
    /// RNG: `active` flips to false exactly when `remaining` hits 0.
    pub(crate) fn tick_cataclysms(&mut self) {
        for c in &mut self.cataclysms {
            if c.remaining > 0 {
                c.remaining -= 1;
                if c.remaining == 0 { c.active = false; }
            }
        }
    }

    /// Current environment multipliers (dust, growth). Pure function of the
    /// cataclysm table: dust = Π (1 − severity × remaining/duration) over
    /// active impacts; growth = Π (1 + climate_delta) over active climate
    /// shifts. Exactly (1.0, 1.0) with no active cataclysm.
    pub fn cataclysm_env_mods(&self) -> (f64, f64) {
        let mut dust = 1.0f64;
        let mut grow = 1.0f64;
        for c in &self.cataclysms {
            if !c.active { continue; }
            if c.kind == CATACLYSM_IMPACT {
                if c.dust_recovery_ticks > 0 {
                    dust *= 1.0 - c.severity * (c.remaining as f64 / c.dust_recovery_ticks as f64);
                }
            } else if c.kind == CATACLYSM_CLIMATE {
                grow *= 1.0 + c.climate_delta;
            }
        }
        (dust.clamp(0.0, 1.0), grow)
    }

    /// Additive metabolic-stress factor from impact dust (0.0 with no active
    /// impact). Drain formulas multiply by (1 + this); because every drain
    /// scales with the genome's `metabolism` gene, high-maintenance
    /// phenotypes lose energy proportionally faster → measurable K-Pg
    /// survivor bias.
    pub fn cataclysm_stress_scale(&self) -> f64 {
        let (dust, _) = self.cataclysm_env_mods();
        (1.0 - dust) * CATACLYSM_STRESS_SCALE
    }

    /// Number of cataclysms ever injected (history + live).
    pub fn cataclysm_count(&self) -> u32 {
        self.cataclysms.len() as u32
    }

    /// Cataclysm record by index (trigger order).
    pub fn cataclysm_at(&self, idx: u32) -> Option<&Cataclysm> {
        self.cataclysms.get(idx as usize)
    }
}
