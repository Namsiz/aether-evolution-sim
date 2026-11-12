#!/usr/bin/env python3
"""Phase 18 surgical patcher: Geological + Biochemical double-layer extension.

Applies the Phase 18 diff to five existing files via exact search/replace
blocks. Every block must match exactly `count` times or the script aborts
WITHOUT writing anything (atomic per-file: all edits of a file are applied to
an in-memory string first).

New files (NOT touched by this script, shipped alongside):
  core/crates/aether-sim-ecology/src/geology.rs
  core/crates/aether-sim-ecology/src/biochemistry.rs
  core/crates/aether-sim-ecology/tests/scale_transition_test.rs
  living/src/renderer-geology.mjs
  living/src/renderer-biochemistry.mjs
  living/test_scale_transition.mjs

Usage:
    cd ~/workspace/world
    python3 core/patch_phase18.py --check   # dry run: verify all blocks match
    python3 core/patch_phase18.py --apply   # apply
"""
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent  # world/

DRY_RUN = False


def s(text):
    """Dedent-free literal block (keeps the author's exact indentation)."""
    return text


def apply_edits(rel_path, edits):
    path = ROOT / rel_path
    text = path.read_text(encoding="utf-8")
    for i, (search, replace, count) in enumerate(edits):
        n = text.count(search)
        if n == 0 and count == 1 and text.count(replace) >= 1:
            continue  # already applied (idempotent re-run)
        if n != count:
            raise SystemExit(
                f"FATAL: {rel_path} edit#{i}: expected {count} occurrence(s), "
                f"found {n}. Aborting, file NOT modified.\n"
                f"--- search block ---\n{search}\n--------------------")
        text = text.replace(search, replace)
    if not DRY_RUN:
        path.write_text(text, encoding="utf-8")
    print(f"  {'[check] ' if DRY_RUN else ''}patched {rel_path}: "
          f"{len(edits)} edits OK")


# ---------------------------------------------------------------------------
# lib.rs / main.rs — declare the new modules in both module trees
# ---------------------------------------------------------------------------

LIB = "core/crates/aether-sim-ecology/src/lib.rs"
LIB_EDITS = [
    (s('''pub mod cataclysm; // Phase 17: cataclysms + recovery dynamics
pub mod wasm;'''),
     s('''pub mod cataclysm; // Phase 17: cataclysms + recovery dynamics
pub mod geology; // Phase 18: plate tectonics + eustatic sea level
pub mod biochemistry; // Phase 18: fast/slow nutrient pools + fertility feedback
pub mod wasm;'''), 1),
]

MAIN = "core/crates/aether-sim-ecology/src/main.rs"
MAIN_EDITS = [
    (s('''mod cataclysm; // Phase 17: cataclysms + recovery dynamics

use ecology::'''),
     s('''mod cataclysm; // Phase 17: cataclysms + recovery dynamics
mod geology; // Phase 18: plate tectonics + eustatic sea level
mod biochemistry; // Phase 18: fast/slow nutrient pools + fertility feedback

use ecology::'''), 1),
]

# ---------------------------------------------------------------------------
# save_state.rs — v4 → v5
# ---------------------------------------------------------------------------

SAVESTATE = "core/crates/aether-sim-ecology/src/save_state.rs"

SAVESTATE_EDITS = [
    # 1. Version bump + doc note appended to the version history header.
    (s('''pub const SAVE_STATE_VERSION: u32 = 4;'''),
     s('''pub const SAVE_STATE_VERSION: u32 = 5;'''), 1),

    (s('''/// so v3 saves restore with an empty cataclysm table (pre-cataclysm worlds
/// keep running with identical checksums).'''),
     s('''/// so v3 saves restore with an empty cataclysm table (pre-cataclysm worlds
/// keep running with identical checksums).
/// v4 → v5 (Phase 18): geologic layer (`Geology` compact JSON string: plates +
/// sea level + epoch + last stepped tick + sparse uplift map) plus the three
/// nutrient-pool floats (`NutrientFast`/`NutrientSlow`/`NutrientDepositRate`).
/// All four are `#[serde(default)]`: pre-v5 restores land on empty plates
/// (geology_step becomes a permanent no-op) and the neutral baseline pools
/// (fast=snow=0.5, deposit_rate=1.0) — bit-exact continuation.'''), 1),

    # 2. New compact-string type SavedGeology (same trick as SavedCataclysms —
    #    avoids extra serde visitors in the wasm binary).
    (s('''/// Phase 17: cataclysm history + live effect state as a compact JSON string
/// (`[{"id":…,"kind":…,…},…]`, same trick as SavedClades/SavedEvents/
/// SavedBurdens — no extra serde visitors in the wasm binary). Parsed in
/// `Ecology::from_save_state`; empty on pre-Phase-17 (v3) saves.
#[derive(Default, Clone, Debug)]
pub struct SavedCataclysms(pub String);

impl serde::Serialize for SavedCataclysms {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}
impl<'de> serde::Deserialize<'de> for SavedCataclysms {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s: String = serde::Deserialize::deserialize(d)?;
        Ok(Self(s))
    }
}'''),
     s('''/// Phase 17: cataclysm history + live effect state as a compact JSON string
/// (`[{"id":…,"kind":…,…},…]`, same trick as SavedClades/SavedEvents/
/// SavedBurdens — no extra serde visitors in the wasm binary). Parsed in
/// `Ecology::from_save_state`; empty on pre-Phase-17 (v3) saves.
#[derive(Default, Clone, Debug)]
pub struct SavedCataclysms(pub String);

impl serde::Serialize for SavedCataclysms {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}
impl<'de> serde::Deserialize<'de> for SavedCataclysms {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s: String = serde::Deserialize::deserialize(d)?;
        Ok(Self(s))
    }
}

/// Phase 18: geologic layer (plate positions/velocities, eustatic sea level,
/// epoch label, last stepped tick, sparse uplift map) as a compact JSON
/// string — same hand-rolled pattern as SavedClades/SavedEvents/
/// SavedBurdens/SavedCataclysms. Parsed in `geology::geology_from_json`.
/// Pre-Phase-18 saves restore with an empty plate list, so `geology_step`
/// becomes a permanent no-op (bit-exact legacy continuation).
#[derive(Default, Clone, Debug)]
pub struct SavedGeology(pub String);

impl serde::Serialize for SavedGeology {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}
impl<'de> serde::Deserialize<'de> for SavedGeology {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s: String = serde::Deserialize::deserialize(d)?;
        Ok(Self(s))
    }
}'''), 1),

    # 3. SaveState struct tail: append Geology + three nutrient-pool fields
    #    after `CarcaseLockEnabled`.
    (s('''    #[serde(default = "default_carcass_lock_enabled")]
    pub CarcaseLockEnabled: bool,
}'''),
     s('''    #[serde(default = "default_carcass_lock_enabled")]
    pub CarcaseLockEnabled: bool,
    /// Phase 18: geologic layer (compact JSON string, see `SavedGeology`).
    /// `#[serde(default)]` keeps pre-Phase-18 saves loadable — they restore
    /// with an empty plate list, so `geology_step` becomes a no-op.
    #[serde(default)]
    pub Geology: SavedGeology,
    /// Phase 18: nutrient pools. Legacy restore defaults to the neutral
    /// baseline (fast_pool = slow_pool = 0.5, deposit_rate = 1.0), matching
    /// `NutrientPools::new()`, so the grass-growth multiplier reads exactly
    /// 1.0 and the Phase 17 checksum trajectory is preserved through
    /// t < BIOCHEM_TICKS.
    #[serde(default)]
    pub NutrientFast: f64,
    #[serde(default)]
    pub NutrientSlow: f64,
    #[serde(default)]
    pub NutrientDepositRate: f64,
}'''), 1),
]

# ---------------------------------------------------------------------------
# ecology.rs — imports, struct fields, constructors, update hook, DECO split,
# to_save_state / from_save_state wiring
# ---------------------------------------------------------------------------

ECOLOGY = "core/crates/aether-sim-ecology/src/ecology.rs"

ECOLOGY_EDITS = [
    # 1. Import both new modules at the top.
    (s('''use crate::save_state;
use std::collections::HashMap;'''),
     s('''use crate::save_state;
use crate::biochemistry::{NutrientPools, mineralization_path_of};
use crate::geology::GeologicState;
use std::collections::HashMap;'''), 1),

    # 2. Ecology struct tail: add geo / nutrients / elevation_delta / pool_acc
    #    after next_cataclysm_id.
    (s('''    /// Phase 17: cataclysm injection log (history + live effect state), in
    /// trigger order. See `cataclysm.rs`; persisted in SaveState v4.
    pub cataclysms: Vec<Cataclysm>,
    pub(crate) next_cataclysm_id: u32,
}'''),
     s('''    /// Phase 17: cataclysm injection log (history + live effect state), in
    /// trigger order. See `cataclysm.rs`; persisted in SaveState v4.
    pub cataclysms: Vec<Cataclysm>,
    pub(crate) next_cataclysm_id: u32,
    /// Phase 18: geological layer (plates, eustatic sea level, seismic
    /// index). Initialized fresh from the seed in `new()` via the DEDICATED
    /// geology RNG stream (never the ecology streams); deserialized from the
    /// compact JSON string in `SavedGeology` during `from_save_state` (v5+
    /// only; pre-v5 gets the serde default: empty plates → step no-op).
    pub(crate) geo: GeologicState,
    /// Phase 18: biochemical layer (fast/slow nutrient pools + tectonic
    /// deposit modifier). Baseline at construction matches
    /// `NutrientPools::new()`: fast=snow=0.5, deposit_rate=1.0 — the
    /// grass-growth multiplier reads exactly 1.0 (bit-exact no-op).
    pub(crate) nutrients: NutrientPools,
    /// Phase 18: additive terrain offset per grid cell (world-height units).
    /// Initially all zeros; mutated by `apply_uplift` during collision/rift
    /// phases and folded through `rebuild_biome_registry` onto suit/moist.
    pub(crate) elevation_delta: Vec<f32>,
    /// Phase 18: per-bite accumulator feeding the pools at each biochem_step
    /// commit boundary (fast share scaled by deposit_rate; slow share
    /// unaffected). Pure bookkeeping — ecology only sees it via biochem_step.
    pub(crate) pool_acc: [f64; 2],
}'''), 1),

    # 3. Both constructors init the new fields (2 occurrences).
    (s('''            clade_burdens: Vec::new(),
            cataclysms: Vec::new(),
            next_cataclysm_id: 1,'''),
     s('''            clade_burdens: Vec::new(),
            cataclysms: Vec::new(),
            next_cataclysm_id: 1,
            // Phase 18: geo/biochem layers start in locked baseline states.
            // Zero-side-effect contract: `geo.last_geo_tick == 0` (first step
            // fires at GEO_TICKS_PER_STEP) and `nutrients.fast_pool ==
            // FAST_BASELINE` (growth multiplier == 1.0 exactly) — neither
            // perturbs the trajectory before its step boundary.
            geo: GeologicState::new(seed),
            nutrients: NutrientPools::new(),
            elevation_delta: vec![0.0; n],
            pool_acc: [0.0; 2],'''), 2),

    # 4. update() hook: fire geology_step then biochem_step AFTER the substep
    #    loop completes (never inside a substep).
    (s('''            if self.tick_count - self.last_sample >= 20 {
                self.last_sample = self.tick_count;
                self.sample();
            }
        }
    }'''),
     s('''            if self.tick_count - self.last_sample >= 20 {
                self.last_sample = self.tick_count;
                self.sample();
            }
        }

        // Phase 18: scale-transition hooks — geology FIRST (terrain changes
        // feed the biochemistry read); both strictly after the full substep
        // loop so the ecology checksum trajectory stays segment-continuous.
        // `geology_step` is a no-op when plates is empty (legacy v4+), and
        // both hooks are zero-RNG by contract.
        if self.tick_count % crate::geology::GEO_TICKS_PER_STEP == 0 {
            self.geology_step();
        }
        if self.tick_count % crate::biochemistry::BIOCHEM_TICKS == 0 {
            self.biochem_step();
        }
    }'''), 1),

    # 5. Grass growth: hoist the biochem fertility multiplier before the loop.
    (s('''            let grow = 0.0056 * season_grass * grow_mult * (0.18 + 0.82 * self.daylight) * (1.0 + self.rain * 0.02);
            for i in 0..self.grass.len() {'''),
     s('''            let grow = 0.0056 * season_grass * grow_mult * (0.18 + 0.82 * self.daylight) * (1.0 + self.rain * 0.02);
            // Phase 18: nutrient fast-pool fertility (exactly 1.0 at baseline
            // → bit-exact no-op; spec: "fast pool 直接调节 grass logistic
            // 增长的 baseline rate", centered on FAST_BASELINE so existing
            // calibration absorbs the multiplier naturally).
            let fert_mult = self.biochem_growth_mult();
            for i in 0..self.grass.len() {'''), 1),

    # 6. cell_grow: multiply in the fertility multiplier.
    (s('''                let cell_grow = grow * (1.0 + self.mineral_field[i] as f64 * GROWTH_BONUS_PER_MINERAL);'''),
     s('''                let cell_grow = grow * (1.0 + self.mineral_field[i] as f64 * GROWTH_BONUS_PER_MINERAL)
                    * fert_mult;'''), 1),

    # 7. DECO carcass-eat branch: route a share of mineralized mass into the
    #    fast/slow nutrient pools via the accumulator (mineral_field deposit
    #    itself is LEFT INTACT — spec says "不只", meaning in addition to).
    (s('''                    let boost = take * deco_eff * MINERAL_DEPOSIT_RATE;
                    if g < self.mineral_field.len() {'''),
     s('''                    let boost = take * deco_eff * MINERAL_DEPOSIT_RATE;
                    // Phase 18: split a share of the mineralized mass into
                    // the fast/slow nutrient pools (accumulator; committed at
                    // the next biochem_step). The mineral_field deposit above
                    // is UNCHANGED (spec "不只堆积在 mineral_field" = in
                    // addition to), so deco_cycle_test and other deposit
                    // gates keep passing bit-exactly.
                    let pool_in = take * deco_eff * crate::biochemistry::BIOCHEM_DEPOSIT_RATE;
                    let path = mineralization_path_of(deco_eff);
                    self.pool_acc[0] += pool_in * path;         // fast share
                    self.pool_acc[1] += pool_in * (1.0 - path); // slow share
                    if g < self.mineral_field.len() {'''), 1),

    # 8. to_save_state: serialize geo (compact JSON) + the three pool floats.
    (s('''            DecoEffChoiceMode: self.deco_eff_choice_mode,
            CarcaseLockEnabled: self.carcass_lock_enabled,
        }
    }'''),
     s('''            DecoEffChoiceMode: self.deco_eff_choice_mode,
            CarcaseLockEnabled: self.carcass_lock_enabled,
            // Phase 18: geologic layer (plates + sea level + epoch + last-step
            // tickstamp + sparse uplift map) and nutrient-pool readout. All
            // f64 values roundtrip through JSON losslessly (serde_json
            // float_roundtrip is enabled in Cargo.toml). `SavedGeology`
            // keeps the serde visitor footprint flat (hand-rolled pattern
            // shared with SavedClades/SavedEvents/SavedBurdens/
            // SavedCataclysms), with its own parser in geology.rs.
            Geology: save_state::SavedGeology(crate::geology::geology_to_json(
                &self.geo, &self.elevation_delta)),
            NutrientFast: self.nutrients.fast_pool,
            NutrientSlow: self.nutrients.slow_pool,
            NutrientDepositRate: self.nutrients.deposit_rate,
        }
    }'''), 1),

    # 9. from_save_state: parse the geologic layer + nutrient pools (pre-v5 →
    #    empty plates / neutral baseline).
    (s('''        let history = state.history.into_iter().map(|h| HistorySample {'''),
     s('''        // Phase 18: geologic state. Pre-v5 saves carry the serde default
        // (empty string) → empty plates → geology_step is a no-op forever.
        // Uplift pairs are parsed from the sparse [[idx,val],…] array inside
        // the Geology JSON; the parser returns a dense Vec<f32> of length
        // `n` wired straight into `elevation_delta`.
        let n_cells = state.grass.len();
        let (geo_parsed, uplift_parsed) =
            crate::geology::geology_from_json(&state.Geology.0, n_cells);
        // Phase 18: nutrient pools. Pre-v5 carries serde defaults (0.0) —
        // non-positive or non-finite values snap to the neutral baseline
        // (fast=snow=0.5, deposit_rate=1.0) so legacy restores land on the
        // exact `NutrientPools::new()` state (bit-exact continuation).
        let nutrients = NutrientPools {
            fast_pool: if state.NutrientFast.is_finite() && state.NutrientFast > 0.0 {
                state.NutrientFast
            } else {
                crate::biochemistry::FAST_BASELINE
            },
            slow_pool: if state.NutrientSlow.is_finite() && state.NutrientSlow > 0.0 {
                state.NutrientSlow
            } else {
                crate::biochemistry::FAST_BASELINE
            },
            deposit_rate: if state.NutrientDepositRate.is_finite() && state.NutrientDepositRate > 0.0 {
                state.NutrientDepositRate
            } else {
                1.0
            },
        };

        let history = state.history.into_iter().map(|h| HistorySample {'''), 1),

    # 10. from_save_state struct literal: wire geo / nutrients /
    #     elevation_delta / pool_acc in (before the closing brace).
    (s('''            species_clades,
            next_clade_id,
            speciation_events,
            last_clade_spec_tick,
            clade_burdens,
            cataclysms,
            next_cataclysm_id,
            seed: state.seed,'''),
     s('''            species_clades,
            next_clade_id,
            speciation_events,
            last_clade_spec_tick,
            clade_burdens,
            cataclysms,
            next_cataclysm_id,
            geo: geo_parsed,
            nutrients,
            elevation_delta: uplift_parsed,
            pool_acc: [0.0; 2],
            seed: state.seed,'''), 1),
]

# ---------------------------------------------------------------------------
# wasm.rs — C-ABI exports (eight sim_* functions + record constants)
# ---------------------------------------------------------------------------

WASM = "core/crates/aether-sim-ecology/src/wasm.rs"

WASM_EDITS = [
    # 1. Record-size constants for Phase 18 (append after CATACLYSM_CAP).
    (s('''pub const CATACLYSM_RECORD_SIZE: usize = 40;
pub const CATACLYSM_CAP: usize = 64;'''),
     s('''pub const CATACLYSM_RECORD_SIZE: usize = 40;
pub const CATACLYSM_CAP: usize = 64;
/// Phase 18: plate record (sim_get_plate) — 48 bytes:
///   [id u32][x f64][z f64][vx f64][vz f64][angle f32][type u32][seismic f32]
pub const PLATE_RECORD_SIZE: usize = 48;
/// Phase 18: nutrient state readout (sim_nutrient_state) — 24 bytes:
///   [fast f64][slow f64][deposit_rate f64]
/// (Spec drafted 36 B; actual payload is 3×f64 = 24 B — documented.)
pub const NUTRIENT_RECORD_SIZE: usize = 24;'''), 1),

    # 2. Eight sim_* C-ABI exports (inserted before the Snapshot v3 serializer).
    (s('''    /// Serialize the current state as Snapshot v3 JSON. Returns a pointer to a
    /// NUL-terminated UTF-8 buffer; write its length to `out_len` when non-null.
    /// Release with `sim_free_json(ptr, len)`.'''),
     s('''    // ─── Phase 18: scale-transition ABI ─────────────────────────────────────

    /// Advance ONE geologic step IF `tick_count % GEO_TICKS_PER_STEP == 0`;
    /// otherwise a no-op. The live viewer never calls this (geology
    /// auto-advances inside `sim_step`'s update() hook chain); tests drive
    /// it directly when manual stepping is desired. Deterministic, zero RNG.
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_geology_tick(sim: *mut WasmSimCore) {
        if sim.is_null() { return; }
        let core = &mut *sim;
        if core.eco.tick_count() % crate::geology::GEO_TICKS_PER_STEP == 0 {
            core.eco.geology_step();
        }
    }

    /// Total tracked plates (≤ PLATE_CAP = 8). 0 on a null handle — also
    /// signals a legacy v4-era save restoring with no plates (geology off).
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_plate_count(sim: *mut WasmSimCore) -> u32 {
        if sim.is_null() { return 0; }
        (*sim).eco.geo_plates().len() as u32
    }

    /// Fill the 48-byte plate record at `out`:
    ///   [id u32][x f64][z f64][vx f64][vz f64][angle f32][type u32][seismic f32]
    /// Returns 1 on success, 0 on null/bounds failure (never traps).
    ///
    /// # Safety
    /// `sim` from `sim_new`; `out` writable for PLATE_RECORD_SIZE bytes.
    #[no_mangle]
    pub unsafe extern "C" fn sim_get_plate(sim: *mut WasmSimCore, idx: u32, out: *mut u8) -> u32 {
        if sim.is_null() || out.is_null() { return 0; }
        let core = &mut *sim;
        let Some(p) = core.eco.geo_plates().get(idx as usize) else { return 0; };
        let mut rec = [0u8; PLATE_RECORD_SIZE];
        rec[0..4].copy_from_slice(&p.id.to_le_bytes());
        rec[4..12].copy_from_slice(&p.x.to_le_bytes());
        rec[12..20].copy_from_slice(&p.z.to_le_bytes());
        rec[20..28].copy_from_slice(&p.vx.to_le_bytes());
        rec[28..36].copy_from_slice(&p.vz.to_le_bytes());
        rec[36..40].copy_from_slice(&(p.angle as f32).to_le_bytes());
        rec[40..44].copy_from_slice(&(p.type_ as u32).to_le_bytes());
        rec[44..48].copy_from_slice(&(core.eco.geo_seismic_index() as f32).to_le_bytes());
        std::ptr::copy_nonoverlapping(rec.as_ptr(), out, PLATE_RECORD_SIZE);
        1
    }

    /// Eustatic sea-level anomaly relative to the base coastline (world
    /// units; positive = transgression). Drives `compute_ocean_mask` but
    /// NOT the suit grid (scope cut documented in geology.rs module header).
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_sea_level(sim: *mut WasmSimCore) -> f64 {
        if sim.is_null() { return 0.0; }
        (*sim).eco.geo_sea_level()
    }

    /// Copy the current epoch name (UTF-8, ≤ cap bytes). Returns byte count
    /// written. 0 on null handle / zero cap (never traps).
    ///
    /// # Safety
    /// `sim` from `sim_new`; `buf` writable for `cap` bytes.
    #[no_mangle]
    pub unsafe extern "C" fn sim_epoch_name(sim: *mut WasmSimCore, buf: *mut u8, cap: u32) -> u32 {
        if sim.is_null() || buf.is_null() || cap == 0 { return 0; }
        let name = (*sim).eco.geo_epoch_name().as_bytes();
        let n = name.len().min(cap as usize);
        std::ptr::copy_nonoverlapping(name.as_ptr(), buf, n);
        n as u32
    }

    /// Advance ONE biochemical cycle IF `tick_count % BIOCHEM_TICKS == 0`;
    /// no-op otherwise. Deterministic, zero RNG.
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_biochem_step(sim: *mut WasmSimCore) {
        if sim.is_null() { return; }
        let core = &mut *sim;
        if core.eco.tick_count() % crate::biochemistry::BIOCHEM_TICKS == 0 {
            core.eco.biochem_step();
        }
    }

    /// Write the three nutrient-state fields to `out`:
    ///   [fast f64][slow f64][deposit_rate f64]
    /// Returns 3 (field count) on success, 0 on null (never traps).
    ///
    /// # Safety
    /// `sim` from `sim_new`; `out` writable for NUTRIENT_RECORD_SIZE bytes.
    #[no_mangle]
    pub unsafe extern "C" fn sim_nutrient_state(sim: *mut WasmSimCore, out: *mut u8) -> u32 {
        if sim.is_null() || out.is_null() { return 0; }
        let n = &(*sim).eco.nutrients;
        let mut rec = [0u8; NUTRIENT_RECORD_SIZE];
        rec[0..8].copy_from_slice(&n.fast_pool.to_le_bytes());
        rec[8..16].copy_from_slice(&n.slow_pool.to_le_bytes());
        rec[16..24].copy_from_slice(&n.deposit_rate.to_le_bytes());
        std::ptr::copy_nonoverlapping(rec.as_ptr(), out, NUTRIENT_RECORD_SIZE);
        3
    }

    /// Fertility boost contributed by the fast nutrient pool (f64; 0.0 at
    /// the neutral baseline, positive when the fast pool exceeds baseline
    /// after DECO carcass feeding). Grass growth folds it in as
    /// `1.0 + fertility_boost * MINERAL_EFFICIENCY`.
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_fertility_boost(sim: *mut WasmSimCore) -> f64 {
        if sim.is_null() { return 0.0; }
        (*sim).eco.fertility_boost()
    }

    /// Serialize the current state as Snapshot v3 JSON. Returns a pointer to a
    /// NUL-terminated UTF-8 buffer; write its length to `out_len` when non-null.
    /// Release with `sim_free_json(ptr, len)`.'''), 1),
]

# ---------------------------------------------------------------------------

ALL = [
    (LIB, LIB_EDITS),
    (MAIN, MAIN_EDITS),
    (SAVESTATE, SAVESTATE_EDITS),
    (ECOLOGY, ECOLOGY_EDITS),
    (WASM, WASM_EDITS),
]


def main():
    global DRY_RUN
    mode = sys.argv[1] if len(sys.argv) > 1 else "--check"
    if mode == "--check":
        DRY_RUN = True
    elif mode != "--apply":
        print(__doc__)
        return 2
    for rel, edits in ALL:
        apply_edits(rel, edits)
    print("ALL PATCHES " + ("VERIFIED (dry run)" if DRY_RUN else "APPLIED"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
