//! save_state.rs — Phase 6 save/restore schema (`aether-sim-save-state`).
//!
//! Sim-internal serialization, fully independent from the render-packet v3
//! `Snapshot` (`aether-render-packet`): different schema_name, own version
//! counter starting at 1, and — unlike AgentView — complete internal state
//! (full genomes, all three RNG streams, carcasses, isolated groups,
//! evolution history, deposit statistics, biome registry).
//!
//! Bit-exactness contract: `Ecology::from_save_state(eco.to_save_state())`
//! must reproduce the exact remaining RNG stream and every state variable
//! that feeds `Ecology::update`, so a restored sim continues with checksums
//! identical to the un-restored one. Restore therefore rebuilds the struct
//! directly and never re-runs `init_grass`/`seed_life` (both consume main-stream
//! RNG draws).
//!
//! PascalCase field naming mirrors the render-packet style; serde_json
//! roundtrips f32/f64 losslessly (shortest-repr), so checksums survive the
//! JSON hop unchanged.
//!
//! Version history:
//! v1 → v2: Phase 6/7/8 additive fields (`#[serde(default)]` keeps v1 loads).
//! v2 → v3 (Phase 16): niche state rides inside the compact JSON clade/event
//! strings (`niche` / `kind` keys) plus the new `CladeNicheBurdens` string —
//! all three are `#[serde(default)]`, so v2 saves restore with
//! niche_offset = 0 / event_kind = 0 / empty burdens.
//! v3 → v4 (Phase 17): cataclysm history + live effect state as the compact
//! JSON string `Cataclysms` plus `NextCataclysmId` — both `#[serde(default)]`,
/// so v3 saves restore with an empty cataclysm table (pre-cataclysm worlds
/// keep running with identical checksums).
/// v4 → v5 (Phase 18): geologic layer (`Geology` compact JSON string: plates +
/// sea level + epoch + last stepped tick + sparse uplift map) plus the three
/// nutrient-pool floats (`NutrientFast`/`NutrientSlow`/`NutrientDepositRate`).
/// All four are `#[serde(default)]`: pre-v5 restores land on empty plates
/// (geology_step becomes a permanent no-op) and the neutral baseline pools
/// (fast=snow=0.5, deposit_rate=1.0) — bit-exact continuation.
/// v5 → v6 (Phase 32): open-locus fields. `SaveAgent.ext_loci` (a variable-
/// length Locus list; `#[serde(default)]` → pre-v6 saves restore with empty
/// ext) plus the two sim-level `GenesisOn` / `ExtDupEvents` flags (both
/// `#[serde(default)]` → legacy restores land on genesis OFF, zero counter).
/// Restoring `GenesisOn` is required for the Phase 32 save/restore contract:
/// a genesis-ON world keeps its duplication operator running after a
/// restore, so the flag rides the schema alongside the per-agent ext list.
/// v6 → v7 (Phase 33): open plant registry + variable-length per-plant
/// layers. `Plants` (the registered `PlantType` list; `#[serde(default)]` →
/// pre-v7 saves restore to the three legacy entries), `TakenBuf` and
/// `SaveHistorySample.plant_by` become `Vec` (sized to the plant count
/// instead of the fixed `GRASS_N`), and `SaveAgent.cargo` backfills to the
/// plant count on older saves. The default-3 world roundtrips bit-exactly.

use serde::{Deserialize, Serialize};


pub const SAVE_STATE_VERSION: u32 = 7;
pub const SAVE_STATE_SCHEMA_NAME: &str = "aether-sim-save-state";

/// DECO selection-pressure mode selector (persisted in SaveState v2).
/// mode=0 → legacy neutral (Phase 6 behavior, no efficiency-driven fitness).
/// mode=1 → selective pressure active (Phase 7+; threshold = 0.96*(2-eff), metabolic term).
pub const DECO_CHOICE_LEGACY: u8 = 0;
pub const DECO_CHOICE_SELECTIVE: u8 = 1;

/// Internal 32-bit state of a mulberry32 stream (`aether-rng`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct RngSnapshot {
    pub state: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveGenome {
    pub speed: f64,
    pub sense: f64,
    pub size: f64,
    pub metabolism: f64,
    pub lifespan: f64,
    pub hue: f64,
    pub bold: f64,
    pub curious: f64,
    pub social: f64,
    pub help_thr: Option<f64>,
    pub pref_sig: Option<f64>,
    pub tol_sig: Option<f64>,
    pub fin_color: f64,
    pub deco_efficiency: f64,
    #[serde(default)]
    pub hull_ratio: f64,
    #[serde(default)]
    pub limb_scale: f64,
}

/// Phase 32: one serialized open-locus slot (mirrors `ecology::Locus`).
/// All five fields are plain scalars; a pre-v6 save that lacks the whole
/// `ext_loci` array deserializes to an empty `Vec` via the parent's
/// `#[serde(default)]`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct SaveLocus {
    pub id: u32,
    pub value: f64,
    pub sigma: f64,
    pub clamp_min: f64,
    pub clamp_max: f64,
}

/// Phase 33: one serialized plant type (mirrors `ecology::PlantType`). The
/// registered-plant list rides `SaveState.plants`; a pre-v7 save that lacks
/// the whole list deserializes to an empty `Vec` via the parent's
/// `#[serde(default)]` and is backfilled to the three legacy entries on load.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct SavePlant {
    pub r_max: f64,
    pub moist_opt: f64,
    pub moist_w: f64,
    pub palatability: f64,
    pub fruiting: f64,
}

/// Complete agent state (superset of render-packet AgentView: full genome,
/// target/think/phase, trail ring buffer, hydration, fatigue, cargo, lineage).
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveAgent {
    pub id: u32,
    pub kind: String,
    pub x: f64,
    pub z: f64,
    pub y: f64,
    pub prev_x: f64,
    pub prev_z: f64,
    pub prev_heading: f64,
    pub heading: f64,
    pub speed: f64,
    pub desired_speed: f64,
    pub energy: f64,
    pub max_energy: f64,
    pub age: f64,
    pub genome: SaveGenome,
    pub generation: u32,
    pub state: String,
    pub bite: f64,
    pub repro: f64,
    pub target_x: f64,
    pub target_z: f64,
    pub target_id: u32,
    pub think: f64,
    pub phase: f64,
    pub alive: bool,
    pub activity: f64,
    pub hydration: f64,
    pub cargo: Vec<f32>,
    pub trail_t: Vec<f64>,
    pub trail_x: Vec<f64>,
    pub trail_z: Vec<f64>,
    pub trail_head: usize,
    pub trail_n: usize,
    pub trail_slot: i32,
    pub fatigue: f64,
    pub fear: f64,
    pub image: i32,
    pub help_cd: f64,
    pub curiosity: f64,
    pub home_x: f64,
    pub home_z: f64,
    pub range: f64,
    pub lineage: Option<String>,
    /// Phase 15: clade id (0 = root/unassigned). `#[serde(default)]` keeps
    /// pre-Phase-15 v2 saves loadable — they restore with clade_count = 0.
    #[serde(default)]
    pub clade_id: u32,
    pub offspring: u32,
    /// Phase 32: open-locus slots (variable-length; `#[serde(default)]` →
    /// pre-v6 saves restore with an empty ext list, i.e. genesis OFF).
    #[serde(default)]
    pub ext_loci: Vec<SaveLocus>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveCarcass {
    pub x: f64,
    pub z: f64,
    pub energy: f64,
    pub age: f64,
    /// Phase 8: agent id of the current occupant lock (DECO/SCAV exclusive
    /// feeding rights). `#[serde(default)]` keeps Phase 6-era v2 saves
    /// deserializable — old saves restore all locks to `None`.
    #[serde(default)]
    pub occupied_by: Option<u32>,
}

/// One isolated (reproductively separated) lineage group. Array order in
/// `SaveState.isolated_groups` preserves `isolated_order` (first-registration
/// order), which the sim relies on for cluster iteration.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveIsolatedGroup {
    pub name: String,
    pub code: String,
    pub kind: String,
    pub since: f64,
}

/// Phase 15: species-clade tree + speciation-event log, stored as compact JSON
/// strings inside the schema. Keeping them as `String` (instead of typed
/// Vec<Struct> fields) avoids monomorphizing extra serde visitors for two new
/// struct types into the wasm binary; parsing happens in
/// `Ecology::from_save_state`. Defaults keep pre-Phase-15 saves loadable —
/// they restore into single-clade (root-only) mode.
#[derive(Default, Clone, Debug)]
pub struct SavedClades(pub String);

impl serde::Serialize for SavedClades {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}
impl<'de> serde::Deserialize<'de> for SavedClades {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s: String = serde::Deserialize::deserialize(d)?;
        Ok(Self(s))
    }
}

#[derive(Default, Clone, Debug)]
pub struct SavedEvents(pub String);

impl serde::Serialize for SavedEvents {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}
impl<'de> serde::Deserialize<'de> for SavedEvents {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s: String = serde::Deserialize::deserialize(d)?;
        Ok(Self(s))
    }
}

/// Phase 16: per-clade competitive burden map as a compact JSON string
/// (`[[cladeId,burden],…]`, same trick as SavedClades/SavedEvents — no extra
/// serde visitors in the wasm binary). Parsed in `Ecology::from_save_state`;
/// empty on pre-Phase-16 saves (burdens rebuild within one detection pass).
#[derive(Default, Clone, Debug)]
pub struct SavedBurdens(pub String);

impl serde::Serialize for SavedBurdens {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}
impl<'de> serde::Deserialize<'de> for SavedBurdens {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s: String = serde::Deserialize::deserialize(d)?;
        Ok(Self(s))
    }
}

/// Phase 17: cataclysm history + live effect state as a compact JSON string
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveEvolutionSample {
    pub t: f64,
    pub gen: u32,
    pub count: usize,
    pub speed: f64,
    pub sense: f64,
    pub size: f64,
    pub metabolism: f64,
    pub bold: f64,
    pub curious: f64,
    pub social: f64,
    pub help_thr: f64,
    /// Phase 7: mean deco_efficiency of the population at sample time (DECO-only; other kinds are NaN-ish 0.0 via serde default).
    #[serde(default)]
    pub DecoEffMean: f64,
    /// Phase 7: std deviation of deco_efficiency (sample-population std; default 0.0 for v1-era samples).
    #[serde(default)]
    pub DecoEffStd: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveFitnessRecord {
    pub gen: u32,
    pub age: f64,
    pub offspring: u32,
    pub fitness: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveHistorySample {
    pub t: f64,
    pub season: i32,
    pub h: usize,
    pub p: usize,
    pub scav: usize,
    pub omn: usize,
    pub plant: f64,
    /// Phase 33: per-plant-layer amount, sized to the plant registry instead
    /// of the fixed `[f64; 3]` (variable-length `Vec` for v7).
    pub plant_by: Vec<f64>,
    pub speed_h: f64,
    pub sense_h: f64,
    pub size_h: f64,
    pub speed_p: f64,
    pub sense_p: f64,
    pub gap_hp: f64,
    pub coop_share_h: f64,
    pub help_thr_h: f64,
    pub sig_mean_h: f64,
    pub sig_var_h: f64,
    pub pref_mean_h: f64,
    pub tol_mean_h: f64,
    pub gen: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveIsolationPoint {
    pub t: f64,
    pub dist: f64,
    pub pairs: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveLineageInfo {
    pub id: String,
    pub name: String,
    pub avg_speed: f64,
    pub count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveSpeciationEvent {
    pub t: f64,
    pub parent_kind: String,
    pub lineages: Vec<SaveLineageInfo>,
    pub cv_speed: f64,
    pub gap_idx: usize,
    pub name_suffix: String,
}

/// Ring-buffer statistics of the DECO mineral deposit diagnostics
/// (samples capped at 4000, cursor wraps).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct SaveDepositStats {
    pub count: u64,
    pub dist_sum: f64,
    pub far_5: u64,
    pub far_20: u64,
    pub colonize: u64,
    pub samples: Vec<f64>,
    pub sample_cursor: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveBiome {
    pub id: u32,
    pub name: String,
    pub center_x: f64,
    pub center_z: f64,
    pub radius: f64,
    pub suit_modifier: f64,
    pub max_herb: usize,
    pub max_pred: usize,
    pub max_scav: usize,
    pub max_omn: usize,
    pub max_fish: Option<usize>,
    pub home_range_mod: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveSelectionPressure {
    pub plant: f64,
    pub pred_ratio: f64,
    pub crowding: f64,
}

/// Full sim state for save/restore. Independent from render-packet `Snapshot`
/// (Meta.version=3): this schema's `Version` starts at 1 and only counts
/// save-state revisions.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveState {
    pub version: u32,
    pub schema_name: String,
    pub engine_version: String,
    pub seed: u32,
    pub tick_count: u32,
    pub time: f64,
    pub season: i32,
    pub daylight: f64,
    pub rain: f64,
    pub fires: Vec<(f64, f64)>,
    pub torch: Option<(f64, f64)>,
    /// Main stream (movement, reproduction, immigration...).
    pub rng_state: RngSnapshot,
    /// Second stream (prng: mate choice / mutation helpers).
    pub prng_state: RngSnapshot,
    /// FISH-internal stream (kept separate so the main streams stay aligned
    /// with the Phase 4-A oracle until cross-species interactions occur).
    pub fish_rng_state: RngSnapshot,
    pub next_id: u32,
    pub agents: Vec<SaveAgent>,
    pub carcasses: Vec<SaveCarcass>,
    pub grass: Vec<f32>,
    pub grass_by_species: Vec<Vec<f32>>,
    pub suit_eff_by_species: Vec<Vec<f32>>,
    /// Phase 33: the registered plant list. `#[serde(default)]` → pre-v7
    /// saves (no field) restore with an empty list, backfilled to the three
    /// legacy entries by `from_save_state`; the per-plant layers
    /// (`grass_by_species`/`suit_eff_by_species`) are sized to it.
    #[serde(default)]
    pub plants: Vec<SavePlant>,
    pub suit: Vec<f32>,
    pub shade: Vec<f32>,
    pub moist: Vec<f32>,
    pub mineral_field: Vec<f32>,
    pub grid_acc: f64,
    pub births: u32,
    pub deaths: u32,
    pub kills: u32,
    pub starved: u32,
    pub aged: u32,
    pub barrier_blocks: u32,
    pub sig_attempts: u32,
    pub sig_blocks: u32,
    pub deposit_stats: SaveDepositStats,
    pub fruit_e_h: f64,
    pub leaf_e_h: f64,
    pub help_count: u32,
    pub reject_count: u32,
    pub help_cost_e_h: f64,
    pub drain_e_h: f64,
    pub speciation_fired: u32,
    pub gene_barrier_strength: f64,
    pub isolated_groups: Vec<SaveIsolatedGroup>,
    pub isolation_history: Vec<SaveIsolationPoint>,
    /// Six per-kind evolution histories (herb, pred, scav, omn, fish, deco).
    pub evolution: Vec<Vec<SaveEvolutionSample>>,
    /// Six per-kind lifetime-fitness records of dead agents.
    pub generation_fitness: Vec<Vec<SaveFitnessRecord>>,
    pub history: Vec<SaveHistorySample>,
    pub last_sample: u32,
    /// Phase 33: variable-length per-plant scratch buffer (was `[f64; 3]`).
    pub taken_buf: Vec<f64>,
    pub spread_buf: Option<Vec<f32>>,
    pub h_cache: u32,
    pub season_meta: f64,
    pub player_x: Option<f64>,
    pub player_z: Option<f64>,
    pub selection_pressure: SaveSelectionPressure,
    pub speciation_event: Option<SaveSpeciationEvent>,
    pub last_speciation: f64,
    /// Phase 15: species phylogeny (clades incl. fossils, split events, and
    /// the engine's id/timing state), as compact JSON strings. Defaults keep
    /// pre-Phase-15 saves loadable — they restore into single-clade mode.
    /// Since v3, clade records also carry a `niche` key and event records a
    /// `kind` key (both default 0 on older saves).
    #[serde(default)]
    pub SpeciesClades: SavedClades,
    #[serde(default)]
    pub NextCladeId: u32,
    #[serde(default)]
    pub SpeciationEvents: SavedEvents,
    #[serde(default)]
    pub LastCladeSpecTick: u32,
    /// Phase 16: competitive-exclusion burden map (compact JSON string).
    /// `#[serde(default)]` keeps pre-Phase-16 saves loadable — they restore
    /// with no active burdens; the next detection pass recomputes them.
    #[serde(default)]
    pub CladeNicheBurdens: SavedBurdens,
    /// Phase 17: cataclysm history + live effect state (compact JSON string).
    /// `#[serde(default)]` keeps pre-Phase-17 (v3) saves loadable — they
    /// restore with an empty table and id counter 1.
    #[serde(default)]
    pub Cataclysms: SavedCataclysms,
    #[serde(default)]
    pub NextCataclysmId: u32,
    /// `None` = registry empty (default `Ecology::new` path, no biomes
    /// registered); `Some(list)` = biomes in registration order (the
    /// `with_biomes` path, including the implicit full-world + aquatic
    /// entries). This distinction is behavior-relevant for `find_biome`.
    pub biomes: Option<Vec<SaveBiome>>,
    /// Phase 7+: DECO selection-pressure mode (persisted across saves/restore).
    /// `0` = legacy neutral (Phase 6 behaviour, all species have neutral genomes);
    /// `1` = selective pressure active (deco_efficiency becomes a real fitness trait).
    /// Default-serialized to `0` for v1-era saves, restoring bit-exact legacy behaviour.
    #[serde(default)]
    pub DecoEffChoiceMode: u8,
    /// Phase 8: whether carcass occupancy locking is enabled.
    /// `true` = exclusive feeding rights per carcass (first-arrival priority
    /// for DECO/SCAV); `false` = legacy multi-agent access (no locking).
    /// Defaults to `true` when absent — new simulations get locks; old v1/v2-era
    /// saves serialized before this field was added are treated as locked by default
    /// so the feature does not silently disable on an older save file.
    #[serde(default = "default_carcass_lock_enabled")]
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
    /// Phase 32: whether the open-locus gene-duplication operator is active
    /// in this sim. `#[serde(default)]` → pre-v6 (legacy) restores land on
    /// genesis OFF, preserving the pre-Phase-32 bit-exact trajectory.
    #[serde(default)]
    pub GenesisOn: bool,
    /// Phase 32: gene-duplication event counter (bookkeeping only — it never
    /// feeds the trajectory or any checksum; carried so a restored sim keeps
    /// the running count). `#[serde(default)]` → 0 on legacy saves.
    #[serde(default)]
    pub ExtDupEvents: u64,
}

/// Map a runtime string back to a `'static str`. Known constants are matched
/// so the hot comparisons (`a.kind == HERB`) share the same backing memory;
/// anything else (agent behavior states, biome names) is leaked once —
/// save/restore is a rare operation and the leaked set is bounded by the
/// distinct strings in one save file.
pub(crate) fn intern_str(s: &str) -> &'static str {
    match s {
        "herb" => "herb",
        "pred" => "pred",
        "scav" => "scav",
        "omn" => "omn",
        "fish" => "fish",
        "deco" => "deco",
        other => Box::leak(other.to_string().into_boxed_str()),
    }
}

/// Default for the `CarcassLockEnabled` save-state field: new sims (without an
/// explicit v1-era value) get Phase 8 locking enabled by default.
fn default_carcass_lock_enabled() -> bool {
    true
}
