//! ecology.rs — bit-exact port of living/src/ecology.js (L106-L1776).
//!
//! Golden target: seed=20261102, dt=0.1, ticks=300 → checksum 0xa6da17d3ab958ddf.
//! Full port (SCAV/OMN/cooperation/speciation/spread/fruiting included) because any
//! omitted RNG draw or state mutation would desync the checksum stream.
//!
//! Rust pattern: per-agent state is accessed via `self.agents[idx]` directly for both
//! reads and writes; helper methods that scan other agents take `idx` and skip it.
//! This avoids long-lived `&mut Agent` refs and keeps everything in safe Rust.

use super::terrain::*;
use aether_biome::{Biome, BiomeRegistry};
use aether_rng::Mulberry32;
use crate::save_state;
use crate::biochemistry::{NutrientPools, mineralization_path_of};
use crate::geology::GeologicState;
use std::collections::HashMap;

/// Phase 31: species identity is runtime registry data, not a code enum.
/// `Agent.kind` holds a registry id; the display/checksum name lives in the
/// `KindDef`. The six legacy species take ids 0-5 in this order — the same
/// order the per-kind evolution histories, fitness records and the C-ABI
/// count export were already indexed by, so a legacy id IS its slot.
pub type SpeciesKind = u16;

pub const HERB: SpeciesKind = 0;
pub const PRED: SpeciesKind = 1;
pub const SCAV: SpeciesKind = 2;
pub const OMN: SpeciesKind = 3;
pub const FISH: SpeciesKind = 4;
pub const DECO: SpeciesKind = 5;
/// Kinds registered by every constructor; runtime registrations append after.
pub const LEGACY_KIND_COUNT: usize = 6;

/// Which `Biome` capacity field applies to a species. `Fallback` reads the
/// kind's own `global_cap` everywhere (DECO, and any future kind that does
/// not couple to biome carrying capacities).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BiomeCap { Herb, Pred, Scav, Omn, Fish, Fallback }

/// Genome factory template. Phase 32's open-loci system generalizes this into
/// per-locus data; Phase 31 keeps the four proven draw patterns as-is so the
/// RNG stream order is untouched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenomeTemplate { Herb, Fish, Deco, General }

/// Which `update_*` behavior loop drives the kind. Phase 34 kind-forking
/// derives new kinds from a parent's template instead of hardcoding names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BehaviorTemplate { Herb, Pred, Scav, Omn, Fish, Deco }

/// Phase 33: a kind's diet as data (the former C6 "who-eats-whom" hardcoded
/// branches). Six legacy recipes are precisely expressible here; unknown /
/// newly-registered kinds fall back to `conservative()` (eat carcass +
/// plants, no live prey) — never a panic. `prey_kinds` order is the legacy
/// PRED two-nearest scan order (HERB then FISH), pinned by the equivalence
/// test; the runtime PRED prey scan itself stays on the legacy path this
/// phase (matrix-scan is a Phase 34 re-baseline).
#[derive(Clone, Debug, PartialEq)]
pub struct Diet {
    /// Feeds on plant material (grazing / filter-feeding). `true` for the
    /// legacy HERB/SCAV/OMN/FISH; `false` for PRED/DECO.
    pub eats_plant: bool,
    /// Explicit per-type whitelist (indices into `Ecology::plants`). Empty
    /// = "all registered layers" when `eats_plant` is set; a non-empty list
    /// restricts to those types. Legacy six use empty lists (they eat every
    /// plant layer).
    pub plant_types: Vec<u32>,
    /// Feeds on carcasses (scavenging / decomposing). `true` for PRED/SCAV/
    /// OMN/DECO; `false` for HERB/FISH.
    pub eats_carcass: bool,
    /// Live prey kinds this kind hunts. Legacy PRED = [HERB, FISH]; every
    /// other legacy kind = empty.
    pub prey_kinds: Vec<SpeciesKind>,
}

impl Diet {
    /// Conservative unknown-kind default (spec: "only eat carcass + grass"):
    /// grazes every plant layer, scavenges carcasses, hunts nothing. Used by
    /// `KindDef::conservative` and any registry entry lacking a diet.
    pub fn conservative() -> Self {
        Diet { eats_plant: true, plant_types: Vec::new(), eats_carcass: true, prey_kinds: Vec::new() }
    }
}

/// Phase 33: named trophic-transfer-efficiency parameter slot (~0.10
/// magnitude). Carried as data only; it takes effect against a real energy
/// ledger in Phase 35. Deliberately NOT wired into the drain/gain dynamics
/// this phase, so the legacy trajectory stays bit-exact.
pub const TROPHIC_EFF: f64 = 0.10;

/// One species-role record: every constant that used to live in a
/// `match kind { … _ => unreachable!() }` arm is data here.
#[derive(Clone, Debug)]
pub struct KindDef {
    pub id: SpeciesKind,
    pub name: &'static str,
    pub behavior: BehaviorTemplate,
    pub genome_template: GenomeTemplate,
    pub biome_cap: BiomeCap,
    /// Population cap where no biome covers the position; also the crowding
    /// denominator and the `Fallback` biome cap.
    pub global_cap: usize,
    pub max_energy: f64,
    pub repro_cooldown: f64,
    pub range_base: f64,
    pub mature_age: f64,
    pub repro_base_threshold: f64,
    /// (parent drain, mate drain) on successful paired reproduction.
    pub repro_drain_pair: (f64, f64),
    pub repro_drain_solo: f64,
    /// (base, span) of the spawn-time starting-energy draw.
    pub start_energy: (f64, f64),
    /// (min, max) clamp for the speed gene under mutation.
    pub speed_clamp: (f64, f64),
    pub mutates_fin_color: bool,
    pub mutates_deco_eff: bool,
    /// Phase 33: diet data (who this kind eats). Legacy six are populated in
    /// `legacy_kind_defs`; unknown kinds get the conservative default.
    pub diet: Diet,
}

impl KindDef {
    /// Conservative default for kinds appearing outside the registry (a save
    /// file from a newer engine phase): pred-like behavior, general genome,
    /// no biome coupling. Never panics — that is this phase's contract.
    fn conservative(name: &'static str) -> Self {
        KindDef {
            id: 0,
            name,
            behavior: BehaviorTemplate::Pred,
            genome_template: GenomeTemplate::General,
            biome_cap: BiomeCap::Fallback,
            global_cap: 1024,
            max_energy: 190.0,
            repro_cooldown: 76.0,
            range_base: 280.0,
            mature_age: 56.0,
            repro_base_threshold: 0.96,
            repro_drain_pair: (58.0, 48.0),
            repro_drain_solo: 76.0,
            start_energy: (92.0, 30.0),
            speed_clamp: (3.4, 8.6),
            mutates_fin_color: false,
            mutates_deco_eff: false,
            diet: Diet::conservative(),
        }
    }
}

/// Append-only species registry. Ids are stable across a run: legacy six at
/// 0-5, runtime registrations after. Names are the save-file and C-ABI
/// identity (checksum v1 keeps hashing the name's first byte, unchanged).
#[derive(Clone, Debug)]
pub struct KindRegistry {
    defs: Vec<KindDef>,
}

impl KindRegistry {
    pub fn legacy() -> Self {
        KindRegistry { defs: legacy_kind_defs() }
    }

    pub fn len(&self) -> usize { self.defs.len() }

    pub fn defs(&self) -> &[KindDef] { &self.defs }

    /// Registry lookup. An id outside the registry cannot be produced by
    /// `spawn` (which resolves through the registry); if one arrives from a
    /// test/edge path it reads the PRED definition instead of panicking —
    /// the Phase 31 replacement for every former `unreachable!()`.
    pub fn def(&self, id: SpeciesKind) -> &KindDef {
        self.defs.get(id as usize).unwrap_or(&self.defs[PRED as usize])
    }

    pub fn name(&self, id: SpeciesKind) -> &'static str { self.def(id).name }

    pub fn id_of(&self, name: &str) -> Option<SpeciesKind> {
        self.defs.iter().position(|d| d.name == name).map(|i| i as SpeciesKind)
    }

    /// Register a new species definition; returns its stable id.
    pub fn register(&mut self, def: KindDef) -> SpeciesKind {
        let id = self.defs.len() as SpeciesKind;
        let mut def = def;
        def.id = id;
        self.defs.push(def);
        id
    }

    /// Save-load path: map a saved kind name back to its id, registering a
    /// conservative definition for names this engine build does not know
    /// (v5 saves only contain the six legacy names, so the fallback is
    /// forward-compatibility, not a live path today).
    pub fn resolve_or_register(&mut self, name: &str) -> SpeciesKind {
        if let Some(id) = self.id_of(name) { return id; }
        let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
        self.register(KindDef::conservative(leaked))
    }
}

/// Field-by-field snapshot of the former `match kind` arms; the
/// `phase31_registry_test` suite asserts every value against this table.
fn legacy_kind_defs() -> Vec<KindDef> {
    vec![
        KindDef {
            id: HERB, name: "herb",
            behavior: BehaviorTemplate::Herb,
            genome_template: GenomeTemplate::Herb,
            biome_cap: BiomeCap::Herb,
            global_cap: MAX_HERB,
            max_energy: 150.0,
            repro_cooldown: HERB_REPRO_COOLDOWN,
            range_base: 170.0,
            mature_age: 30.0,
            repro_base_threshold: 0.86,
            repro_drain_pair: (38.0, 30.0),
            repro_drain_solo: 52.0,
            start_energy: (78.0, 25.0),
            speed_clamp: (2.0, 8.4),
            mutates_fin_color: false,
            mutates_deco_eff: false,
            diet: Diet { eats_plant: true, plant_types: Vec::new(), eats_carcass: false, prey_kinds: Vec::new() },
        },
        KindDef {
            id: PRED, name: "pred",
            behavior: BehaviorTemplate::Pred,
            genome_template: GenomeTemplate::General,
            biome_cap: BiomeCap::Pred,
            global_cap: MAX_PRED,
            max_energy: 190.0,
            repro_cooldown: PRED_REPRO_COOLDOWN,
            range_base: 280.0,
            mature_age: 56.0,
            repro_base_threshold: 0.96,
            repro_drain_pair: (58.0, 48.0),
            repro_drain_solo: 76.0,
            start_energy: (92.0, 30.0),
            speed_clamp: (3.4, 8.6),
            mutates_fin_color: false,
            mutates_deco_eff: false,
            diet: Diet { eats_plant: false, plant_types: Vec::new(), eats_carcass: true, prey_kinds: vec![HERB, FISH] },
        },
        KindDef {
            id: SCAV, name: "scav",
            behavior: BehaviorTemplate::Scav,
            genome_template: GenomeTemplate::General,
            biome_cap: BiomeCap::Scav,
            global_cap: MAX_SCAV,
            max_energy: 120.0,
            repro_cooldown: SCAV_REPRO_COOLDOWN,
            range_base: 200.0,
            mature_age: 56.0,
            repro_base_threshold: 0.96,
            repro_drain_pair: (58.0, 48.0),
            repro_drain_solo: 76.0,
            start_energy: (55.0, 20.0),
            speed_clamp: (3.4, 8.6),
            mutates_fin_color: false,
            mutates_deco_eff: false,
            diet: Diet { eats_plant: true, plant_types: Vec::new(), eats_carcass: true, prey_kinds: Vec::new() },
        },
        KindDef {
            id: OMN, name: "omn",
            behavior: BehaviorTemplate::Omn,
            genome_template: GenomeTemplate::General,
            biome_cap: BiomeCap::Omn,
            global_cap: MAX_OMN,
            max_energy: 135.0,
            repro_cooldown: OMN_REPRO_COOLDOWN,
            range_base: 185.0,
            mature_age: 56.0,
            repro_base_threshold: 0.96,
            repro_drain_pair: (58.0, 48.0),
            repro_drain_solo: 76.0,
            start_energy: (65.0, 22.0),
            speed_clamp: (3.4, 8.6),
            mutates_fin_color: false,
            mutates_deco_eff: false,
            diet: Diet { eats_plant: true, plant_types: Vec::new(), eats_carcass: true, prey_kinds: Vec::new() },
        },
        KindDef {
            id: FISH, name: "fish",
            behavior: BehaviorTemplate::Fish,
            genome_template: GenomeTemplate::Fish,
            biome_cap: BiomeCap::Fish,
            global_cap: MAX_FISH,
            max_energy: 110.0,
            repro_cooldown: FISH_REPRO_COOLDOWN,
            range_base: 120.0,
            mature_age: 25.0,
            repro_base_threshold: 0.65,
            repro_drain_pair: (20.0, 16.0),
            repro_drain_solo: 46.0,
            start_energy: (60.0, 18.0),
            speed_clamp: (1.8, 6.8),
            mutates_fin_color: true,
            mutates_deco_eff: false,
            diet: Diet { eats_plant: true, plant_types: Vec::new(), eats_carcass: false, prey_kinds: Vec::new() },
        },
        KindDef {
            id: DECO, name: "deco",
            behavior: BehaviorTemplate::Deco,
            genome_template: GenomeTemplate::Deco,
            biome_cap: BiomeCap::Fallback,
            global_cap: MAX_DECO,
            max_energy: DECO_MAX_ENERGY,
            repro_cooldown: DECO_REPRO_COOLDOWN,
            range_base: DECO_RANGE_BASE,
            mature_age: 30.0,
            repro_base_threshold: 0.96,
            repro_drain_pair: (58.0, 48.0),
            repro_drain_solo: 45.0,
            start_energy: (50.0, 20.0),
            speed_clamp: (3.4, 8.6),
            mutates_fin_color: false,
            mutates_deco_eff: true,
            diet: Diet { eats_plant: false, plant_types: Vec::new(), eats_carcass: true, prey_kinds: Vec::new() },
        },
    ]
}

pub const GRID_N: u32 = 76;
pub const GRID_SPAN: f64 = 1080.0;
pub const CELL: f64 = GRID_SPAN / (GRID_N as f64);
const MAX_HERB: usize = 420;
const MAX_PRED: usize = 110;
const MAX_SCAV: usize = 40;
const MAX_OMN: usize = 80;
const MAX_FISH: usize = 60;
const MAX_DECO: usize = 80;
const FISH_REPRO_COOLDOWN: f64 = 45.0;
const FISH_BITE_INTERVAL: f64 = 3.0;
const FISH_SPEED_MULT: f64 = 1.2;
const DECO_REPRO_COOLDOWN: f64 = 50.0;
const DECO_BITE_INTERVAL: f64 = 2.0;
const DECO_SPEED_MULT: f64 = 0.8;
const DECO_MAX_ENERGY: f64 = 100.0;
/// Base reproduction threshold shared by SCAV / OMN / PRED (and DECO under legacy mode).
/// Phase 7: DECO's effective threshold is individualized as `DECO_REPRO_BASE * (2.0 - deco_efficiency)`.
const DECO_REPRO_BASE: f64 = 0.96;
const DECO_RANGE_BASE: f64 = 150.0;
/// Phase 4-C MineralField: decomposers convert carcass mass into soil
/// minerals. Deposit = take × deco_efficiency × rate per bite; the field
/// decays each tick and boosts grass logistic growth.
/// NOTE: spec drafted 0.002, but under real competition (SCAV/OMN take most
/// carcass mass, carcasses decay 1.4/tick) that yields a peak field sum of
/// only ~0.03 over 600 ticks. 0.008 clears the >0.1 acceptance gate with the
/// same dynamics; see deco_cycle_test and the Phase 4-C report.
const MINERAL_DEPOSIT_RATE: f64 = 0.008;
const MINERAL_DECAY: f64 = 0.9995;
const GROWTH_BONUS_PER_MINERAL: f64 = 0.3;
const WATER_ZONE_THRESHOLD: f64 = WATER_LEVEL + 2.0;
/// Energy lost per 0.1-tick step while stranded out of water (spec: 0.05/tick).
const FISH_DRY_DEATH_RATE: f64 = 0.5;
/// Passive filter-feeding gain per time unit while submerged (plankton/algae).
/// Sized so a fish recovers the ~36 energy cost of a breeding cycle in
/// roughly 240 ticks — faster than predation removes fish at steady state.
const FISH_FILTER_GAIN: f64 = 1.5;
const BITE_INTERVAL: f64 = 2.1;
const PRED_BITE_INTERVAL: f64 = 9.0;
const HERB_REPRO_COOLDOWN: f64 = 32.0;
const PRED_REPRO_COOLDOWN: f64 = 76.0;
const SCAV_REPRO_COOLDOWN: f64 = 40.0;
const OMN_REPRO_COOLDOWN: f64 = 38.0;

const HELP_ENABLED: bool = true;
const HELP_RADIUS: f64 = 22.0;
const HELP_CD_MIN: f64 = 8.0;
const HELP_CD_RAND: f64 = 4.0;
const HELP_NEEDY: f64 = 0.4;
const HELP_AGE_MIN: f64 = 30.0;
const HELP_IMAGE_MAX: i32 = 5;

const WINTER_HELP_SCALE: f64 = 0.6;
const WIN_TEST_THRESHOLD_MIN: u32 = 50;
const WIN_AGG_THRESHOLD: u32 = 250;
const WIN_AGG_PULL: f64 = 0.3;
const WIN_AGG_NEIGHBORS: usize = 3;
const WIN_AGG_PROB: f64 = 0.03;

const SIG_ENABLED: bool = true;
const SIG_TOL_SCALE: f64 = 1.0;
const SIG_MUT_PREF: f64 = 0.06;
const SIG_MUT_TOL: f64 = 0.04;
const SIG_TOL_MAX: f64 = 0.25;
const SIG_TOL_MIN: f64 = 0.05;
const SIG_PREF_INIT: f64 = 1.03;

/// One registered plant type (Phase 33: the open production layer). The
/// default world carries the three legacy entries, field-for-field identical
/// to the former `GRASS_SPECIES` table; `Ecology::register_plant` appends
/// new types. Registered types only ever ADD a whole extra layer, so they
/// never perturb the bit-exact legacy-3 trajectory (no re-baseline needed
/// for the default world).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PlantType {
    pub r_max: f64,
    pub moist_opt: f64,
    pub moist_w: f64,
    pub palatability: f64,
    pub fruiting: f64,
}
// Backward-compatible alias (the struct was `GrassSpecies` through Phase 32).
pub type GrassSpecies = PlantType;

/// The three default plant types — bit-identical to the former
/// `GRASS_SPECIES: [GrassSpecies; 3]` (same literal fields, same order).
pub const LEGACY_PLANTS: [PlantType; 3] = [
    PlantType { r_max: 1.4, moist_opt: 0.38, moist_w: 0.18, palatability: 1.3, fruiting: 0.12 },
    PlantType { r_max: 1.2, moist_opt: 0.05, moist_w: 0.16, palatability: 0.5, fruiting: 0.0 },
    PlantType { r_max: 1.2, moist_opt: 0.18, moist_w: 0.36, palatability: 0.85, fruiting: 0.06 },
];
/// Default (legacy) plant-layer count. The engine organizes itself by the
/// registry length (`Ecology::plants`, 3 in the default world); this constant
/// stays as the legacy 3-layer reference and the v6→v7 save backfill target.
pub const GRASS_N: usize = 3;

const FRUIT_COST: f64 = 0.5;
/// Phase 33: per-plant effective r_max = `r_max * (1 - fruiting * FRUIT_COST)`
/// — the runtime replacement for the former compile-time `RMAX_EFF` table.
/// For the three legacy entries it reproduces that table bit-exactly (the
/// operands are the same f64 literals in the same expression order).
pub fn rmax_eff(p: &PlantType) -> f64 { p.r_max * (1.0 - p.fruiting * FRUIT_COST) }
const FRUIT_ENERGY: f64 = 0.5;
const SEED_LOAD: f64 = 0.05;
const GUT_DECAY: f64 = 0.0085;
const TRAIL_SLOTS: usize = 16;

// ----- Phase 32: open loci (variable-length heritable genome) --------------
//
// `Locus` is one extra heritable gene beyond the pinned 17-dim core
// phenotype (which stays bit-exact). A genome's `ext` list is unbounded:
// loci enter via the gene-duplication operator in `mutate` (de-novo
// genesis when empty, Ohno-style duplication otherwise) and exit via
// deletion; `mix_genome` inherits them by id-pairing. No functional
// semantics are assigned to a locus this phase — value/sigma/clamp are
// carried data only, wired to behaviour in Phase 33/34.
//
// Determinism discipline: with `genesis_on == false` (default) the operator
// makes zero extra RNG draws, `ext` stays empty, and every trajectory / 
// checksum is bit-identical to pre-Phase-32. With `genesis_on == true`
// exactly ONE extra main-stream draw (`dup_roll`) is made per mutate call,
// independent of `ext` length — locus growth never changes the per-agent
// mutation draw budget (§4 discipline: no "loci double → draws double").
/// Add-roll probability mass: `dup_roll < P_DUP` triggers an add (de-novo
/// genesis when `ext` is empty, gene-duplication of the highest-id locus
/// when non-empty). The de-novo value is derived from the same roll
/// (`dup_roll / P_DUP`, uniform on [0,1)) so the draw budget stays constant.
pub const P_DUP: f64 = 0.06;
/// Delete-roll probability mass: `P_DUP <= dup_roll < P_DUP + P_DEL` removes
/// the highest-id locus (only when `ext` is non-empty).
pub const P_DEL: f64 = 0.02;
/// Per-locus expression upkeep: each ext locus multiplies the shared
/// crowding/epistasis drain by `(1 + LOCI_UPKEEP)` — an economic (not
/// rule) cap that raises the metabolic cost of a long genome and holds
/// `ext` below an upper bound without a hard size limit. 0 loci ⇒ ×1.0
/// (bit-exact no-op).
pub const LOCI_UPKEEP: f64 = 0.002;
/// Default mutation sigma of a de-novo locus (its future per-locus mutation
/// step size; carried but unused this phase — reserved for Phase 33/34).
pub const LOCUS_SIGMA: f64 = 0.05;
/// Neutral clamp band for a de-novo locus value (semantics-free unit range).
pub const LOCUS_CLAMP_MIN: f64 = 0.0;
pub const LOCUS_CLAMP_MAX: f64 = 1.0;

/// One extra heritable gene slot (Phase 32 open loci). See the module
/// comment above for the determinism discipline and Phase 33/34 handoff.
#[derive(Clone, Debug, PartialEq)]
pub struct Locus {
    /// Monotonic id within the individual (0-based; a new locus = max+1).
    pub id: u32,
    /// Locus value (unit band; no behavioural meaning yet this phase).
    pub value: f64,
    /// Per-locus mutation sigma (carried data; unused until Phase 33/34).
    pub sigma: f64,
    /// Per-locus value clamp band.
    pub clamp_min: f64,
    pub clamp_max: f64,
}

#[derive(Clone, Debug)]
pub struct Genome {
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
    /// Phase 4-C DECO-only gene: scales both carcass processing throughput
    /// and mineral deposit strength. Non-DECO kinds carry the default 1.0.
    pub deco_efficiency: f64,
    pub hull_ratio: f64,
    pub limb_scale: f64,
    /// Phase 32: open-locus extension slots. Default `Vec::new()` (the
    /// pinned 17-dim core above is bit-exact and always present); the list
    /// is empty whenever `genesis_on == false` and grows/decays via the
    /// duplication operator when genesis is on.
    pub ext: Vec<Locus>,
}

#[derive(Debug)]
pub struct Agent {
    pub id: u32,
    pub kind: SpeciesKind,
    pub x: f64, pub z: f64, pub y: f64,
    pub prev_x: f64, pub prev_z: f64, pub prev_heading: f64,
    pub heading: f64, pub speed: f64, pub desired_speed: f64,
    pub energy: f64, pub max_energy: f64,
    pub age: f64,
    pub genome: Genome,
    pub generation: u32,
    pub state: &'static str,
    pub bite: f64,
    pub repro: f64,
    pub target_x: f64, pub target_z: f64, pub target_id: u32,
    pub think: f64, pub phase: f64,
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
    pub home_x: f64, pub home_z: f64,
    pub range: f64,
    pub lineage: Option<&'static str>,
    /// Phase 15: species clade membership. 0 = root clade of the kind (the
    /// value every fresh/legacy agent carries until the engine buckets it).
    pub clade_id: u32,
    pub offspring: u32,
}

#[derive(Debug)]
pub struct Carcass {
    pub x: f64,
    pub z: f64,
    pub energy: f64,
    pub age: f64,
    /// Phase 8 occupancy lock: `Some(agent_id)` while a decomposer/scavenger
    /// holds exclusive feeding rights. Released when the owner dies, moves
    /// >5.0 away, or the carcass ages past 60.0 (stale-lock timeout).
    pub occupied_by: Option<u32>,
}

#[derive(Debug)]
struct FitnessRecord {
    gen: u32,
    age: f64,
    offspring: u32,
    fitness: f64,
}

#[derive(Debug, Clone)]
pub struct EvolutionSample {
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
    /// Phase 7+: mean deco_efficiency among alive DECO at sample time (0.0 for non-DECO kinds via default init).
    pub deco_eff: f64,
    /// Phase 7+: std deviation of deco_efficiency among alive DECO at sample time.
    pub deco_eff_std: f64,
}

#[derive(Debug, Clone)]
struct IsolatedGroup {
    code: String,
    kind: &'static str,
    since: f64,
}

#[derive(Debug, Default, Clone)]
struct DepositStats {
    count: u64,
    dist_sum: f64,
    far_5: u64,
    far_20: u64,
    colonize: u64,
    samples: Vec<f64>,
    sample_cursor: usize,
}

#[derive(Debug, Clone)]
pub struct HistorySample {
    pub t: f64,
    pub season: i32,
    pub h: usize,
    pub p: usize,
    pub scav: usize,
    pub omn: usize,
    pub plant: f64,
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

#[derive(Debug, Clone)]
pub struct SelectionPressure {
    pub plant: f64,
    pub pred_ratio: f64,
    pub crowding: f64,
}

#[derive(Debug, Clone)]
pub struct LineageInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub avg_speed: f64,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct SpeciationEvent {
    pub t: f64,
    pub parent_kind: &'static str,
    pub lineages: Vec<LineageInfo>,
    pub cv_speed: f64,
    pub gap_idx: usize,
    pub name_suffix: String,
}

// ----- Phase 15: species clades / speciation engine ---------------------------
//
// Clade = one branch of the species phylogeny. Membership lives on
// `Agent.clade_id` (0 = root); the SpeciesClade records are the tree/fossil
// view the C-ABI exports read. Detection runs every SPECIATION_CHECK_TICKS
// from sample(): a 1-D MST over deco_efficiency gaps is cut at edges ≥
// SPECIATION_DISTANCE_THRESHOLD, and a cut only becomes a new species when
// geographic + temporal isolation also hold. The legacy HERB `SpeciationEvent`
// above keeps its name and semantics untouched.
/// Detector cadence in ticks (sample() runs on the 20-tick grid, so the
/// effective cadence is exactly 60 ticks).
const SPECIATION_CHECK_TICKS: u32 = 60;
/// MST cut threshold on the 1-D deco_efficiency distance. deco_efficiency is
/// clamped to [0.5, 1.5] by mutate, so 0.35 is roughly a third of the legal
/// band — a real bimodal split, not mutation noise.
pub const SPECIATION_DISTANCE_THRESHOLD: f64 = 0.35;
/// Geographic isolation gate: the nearest cross-cluster pair must be farther
/// apart than this (DECO home range is ≈150-180 world units).
pub const GEO_ISOLATION_DIST: f64 = 100.0;
/// Minimum ticks between two speciation events.
pub const MIN_SPEC_TIME_TICKS: u32 = 200;
/// A split is only declared when both sides keep at least this many members
/// (a one-agent "species" is sampling noise and dies within a pass).
const SPECIATION_MIN_SIDE: usize = 2;

// ----- Phase 16: niche axis + competitive exclusion ---------------------------
//
// Niche model: each clade carries a 1-D `niche_offset` in [-1, 1]. A living
// daughter clade re-derives its position every detection pass from its
// parent's position plus the current centroid deco_efficiency gap
// (NICHE_EFF_SCALE maps the legal eff band [0.5, 1.5] onto [-1, 1]); the root
// clade is pinned at 0. The value is therefore a pure function of saved
// clade state + live genomes — no RNG draws, no extra saved drift history —
// so a restored sim re-derives bit-identical niches (save/restore contract).
/// eff gap -> niche displacement: (eff_child - eff_parent) * 2 spans [-1, 1]
/// for the legal eff band [0.5, 1.5].
pub const NICHE_EFF_SCALE: f64 = 2.0;
/// Two clades whose niche positions are closer than this compete for the
/// same resources (1-D Gause overlap window).
pub const NICHE_OVERLAP_THRESHOLD: f64 = 0.15;
/// Extra metabolic multiplier applied to the fitness-loser of an overlapping
/// clade pair (+15 % drain). Only computed in SELECTIVE mode — in legacy mode
/// deco_efficiency is fitness-neutral by contract, so burdening the
/// lower-efficiency clade would smuggle selection into the neutral control
/// (`legacy_mode_shows_no_selection`).
pub const COMPETITION_BURDEN: f64 = 0.15;
/// Cap on the summed burden of one clade (multi-overlap stacking).
pub const COMPETITION_BURDEN_MAX: f64 = 0.5;
/// Wander-heading bias in radians: a clade at niche +/-1 roams up to
/// +/-0.9 rad off its unbiased random heading, spatially separating clades.
pub const NICHE_HEADING_BIAS: f64 = 0.9;
/// Fraction of a bite's mineral boost re-deposited into the niche-direction
/// neighbor cell (DECO mineral preference direction).
pub const NICHE_DEPOSIT_BIAS: f64 = 0.25;
/// Event log kinds (CladeSpeciationEvent.event_kind / ABI event-record
/// offset 12, formerly the reserved u32).
pub const EVENT_KIND_SPECIATION: u8 = 0;
pub const EVENT_KIND_EXTINCTION: u8 = 1;
pub const EVENT_KIND_COMPETITIVE_EXCLUSION: u8 = 2;

// ----- Phase 17: cataclysms + recovery dynamics --------------------------------
//
// Cataclysms are experimenter-injected extinction events (C-ABI only — the
// engine never fires one on its own). All bookkeeping is deterministic: no
// RNG draws anywhere in the cataclysm path, and with no active cataclysm
// every environment multiplier is exactly 1.0 (f64 x*1.0 is bit-exact), so
// the checksum trajectory of cataclysm-free runs is untouched.
/// kind 0: asteroid impact — local resource wipe + mass mortality + dust.
pub const CATACLYSM_IMPACT: u8 = 0;
/// kind 1: climate shift — global grass-growth throttle, no direct mortality.
pub const CATACLYSM_CLIMATE: u8 = 1;
/// Energy struck off every agent inside the impact radius (scaled by
/// severity). Deaths reuse the regular die() path (cause "cataclysm").
pub const CATACLYSM_DAMAGE: f64 = 60.0;
/// Peak additive metabolic-stress factor at full dust cover: drain formulas
/// scale with the genome's metabolism gene, so this yields a measurable
/// K-Pg-style selection against high-maintenance phenotypes.
pub const CATACLYSM_STRESS_SCALE: f64 = 1.5;

/// One cataclysm record: the injection event plus its live effect state.
/// `remaining` counts down once per substep in `update`; `active` flips to
/// false at zero. Persisted in SaveState v4 via the compact JSON string.
#[derive(Clone, Debug)]
pub struct Cataclysm {
    pub id: u32,
    pub kind: u8,
    /// Injection tick (ecology tick_count at trigger time).
    pub tick: u32,
    pub x: f64,
    pub z: f64,
    pub radius: f64,
    pub severity: f64,
    /// Impact only: dust multiplier recovers linearly over this many ticks.
    pub dust_recovery_ticks: u32,
    /// Climate only: additive growth modifier (negative = throttled).
    pub climate_delta: f64,
    /// Last tick the effect can act (tick + duration at injection).
    pub ends_at: u32,
    /// Live countdown; > 0 ⇔ active.
    pub remaining: u32,
    pub active: bool,
}

/// One branch of the species phylogeny. `members` is a cached view rebuilt
/// from live agents at each detection pass; `death_tick` freezes the record
/// when the last member dies (fossil).
#[derive(Clone, Debug)]
pub struct SpeciesClade {
    pub id: u32,
    pub parent_id: Option<u32>,
    pub kind: &'static str,
    pub name: String,
    pub birth_tick: u32,
    pub death_tick: Option<u32>,
    pub centroid_genome: Genome,
    pub members: Vec<u32>,
    /// Phase 16: 1-D niche position in [-1, 1]. Living daughters re-derive it
    /// every pass (parent position + centroid eff gap); fossils freeze it.
    pub niche_offset: f64,
}

/// One clade-based speciation event (parent clade → one or more daughter
/// clades in the same detection pass).
#[derive(Clone, Debug)]
pub struct CladeSpeciationEvent {
    pub tick: u32,
    pub parent_clade_id: u32,
    pub child_clade_ids: Vec<u32>,
    pub genetic_distance: f64,
    /// Phase 16: 0 = speciation, 1 = plain extinction, 2 = competitive
    /// exclusion (the losing clade carried a nonzero burden when its members
    /// emptied). Appended at the tail; JSON key "kind" defaults to 0.
    pub event_kind: u8,
}

/// Tiny union-find over MST vertex slots (n ≤ MAX_DECO = 80; no rank or
/// path compression needed at this size).
struct Dsu {
    parent: Vec<usize>,
}

impl Dsu {
    fn new(n: usize) -> Self {
        Dsu { parent: (0..n).collect() }
    }
    fn find(&self, mut x: usize) -> usize {
        while self.parent[x] != x { x = self.parent[x]; }
        x
    }
    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra != rb { self.parent[ra] = rb; }
    }
}

/// Bijective base-26 label for child clades: 1→"A", 2→"B", … 26→"Z", 27→"AA".
fn clade_letter(mut n: u32) -> String {
    let mut out = Vec::new();
    while n > 0 {
        n -= 1;
        out.push((b'A' + (n % 26) as u8) as char);
        n /= 26;
    }
    out.reverse();
    out.into_iter().collect()
}

/// Insertion sort over tiny slices (≤ MAX_DECO). Replaces `slice::sort*` in
/// the speciation engine: the std generic sort stack monomorphizes ~15 KB of
/// driftsort/quicksort machinery per element type into the wasm binary.
fn insertion_sort_by<T, F: Fn(&T, &T) -> std::cmp::Ordering>(v: &mut [T], cmp: F) {
    for i in 1..v.len() {
        let mut j = i;
        while j > 0 && cmp(&v[j - 1], &v[j]) == std::cmp::Ordering::Greater {
            v.swap(j - 1, j);
            j -= 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct IsolatedGroupStat {
    pub name: String,
    pub kind: &'static str,
    pub since: f64,
    pub count: usize,
}

#[derive(Debug, Clone)]
pub struct IsolationHistoryPoint {
    pub t: f64,
    pub dist: f64,
    pub pairs: usize,
}

#[derive(Debug, Clone)]
pub struct DepositSummary {
    pub count: u64,
    pub mean_dist_cells: f64,
    pub p50: f64,
    pub p90: f64,
    pub far_5: u64,
    pub far_20: u64,
    pub colonize: u64,
}

#[derive(Debug, Clone)]
pub struct Stats {
    pub time: f64,
    pub season: i32,
    pub h: usize, pub p: usize, pub scav: usize, pub omn: usize,
    pub plant: f64,
    pub plant_by: Vec<f64>,
    pub speed_h: f64, pub sense_h: f64, pub size_h: f64,
    pub speed_p: f64, pub sense_p: f64,
    pub gap_hp: f64, pub coop_share_h: f64, pub help_thr_h: f64,
    pub sig_mean_h: f64, pub sig_var_h: f64, pub pref_mean_h: f64, pub tol_mean_h: f64,
    pub gen: u32,
    pub births: u32, pub deaths: u32, pub kills: u32, pub starved: u32, pub aged: u32,
    pub trend_h: f64, pub trend_p: f64,
    pub selection_pressure: SelectionPressure,
    pub cur_fit_h: f64, pub cur_fit_p: f64,
    pub ev_hist_h: Vec<EvolutionSample>,
    pub ev_hist_p: Vec<EvolutionSample>,
    pub ev_hist_scav: Vec<EvolutionSample>,
    pub ev_hist_omn: Vec<EvolutionSample>,
    pub ev_hist_fish: Vec<EvolutionSample>,
    pub ev_hist_deco: Vec<EvolutionSample>,
    pub deco_count: usize,
    pub speciation_event: Option<SpeciationEvent>,
    pub gene_barrier: f64,
    pub barrier_blocks: u32,
    pub isolated_groups: Vec<IsolatedGroupStat>,
    pub isolation_history: Vec<IsolationHistoryPoint>,
    pub grass_species: Vec<GrassSpecies>,
    pub cargo_total: f64,
    pub fruit_energy_share_h: f64,
    pub deposit: Option<DepositSummary>,
    pub help_count: u32, pub reject_count: u32, pub help_cost_share_h: f64,
    pub winter_coop_share_h: f64,
    pub sig_pair_fail_rate: f64,
    pub sig_pair_attempts: u32,
    pub sig_pair_fails: u32,
    /// Phase 7+: instantaneous mean deco_efficiency across all alive DECO agents.
    pub deco_eff_mean: f64,
    /// Phase 7+: instantaneous std deviation of deco_efficiency across all alive DECO agents.
    pub deco_eff_std: f64,
}


impl Genome {
    fn has_sig_genes(&self) -> bool {
        self.help_thr.is_some() && self.pref_sig.is_some() && self.tol_sig.is_some()
    }

    /// Neutral deco-genome template (efficiency 1.0). Used when a restored
    /// clade only carries its centroid efficiency — the full centroid is
    /// recomputed from live members on the next detection pass.
    fn default_deco() -> Self {
        Genome {
            speed: 1.0, sense: 1.0, size: 1.0, metabolism: 1.0, lifespan: 1.0,
            hue: 0.0, bold: 0.0, curious: 0.0, social: 0.0,
            help_thr: None, pref_sig: None, tol_sig: None,
            fin_color: 0.0, deco_efficiency: 1.0,
            hull_ratio: 1.0, limb_scale: 3.0,
            ext: Vec::new(),
        }
    }
}

pub struct Ecology {
    pub rng: Mulberry32,
    pub prng: Mulberry32,
    /// Dedicated stream for FISH-internal draws. Keeps the main rng/prng
    /// streams bit-identical to the Phase 4-A oracle until a genuine
    /// cross-species interaction (predation, carcass scavenging) occurs.
    pub fish_rng: Mulberry32,
    pub agents: Vec<Agent>,
    pub carcasses: Vec<Carcass>,
    next_id: u32,
    pub time: f64,
    tick_count: u32,
    pub daylight: f64,
    pub season: i32,
    pub fires: Vec<(f64, f64)>,
    pub torch: Option<(f64, f64)>,
    pub rain: f64,
    // Phase 17: cataclysm.rs mutates the resource fields directly (impact
    // blast); the public accessors remain authoritative for reads.
    pub(crate) grass: Vec<f32>,
    pub(crate) grass_p: Vec<Vec<f32>>,
    suit_eff_by: Vec<Vec<f32>>,
    /// Phase 33: plant registry (open production layer). Default = the three
    /// legacy entries (`LEGACY_PLANTS.to_vec()`); `register_plant` appends a
    /// new layer. Every per-species layer (`grass_p`/`suit_eff_by`/
    /// `Agent.cargo`/`taken_buf`) is organized by `plants.len()`; at the
    /// default length 3 the numeric path is bit-identical to the former
    /// fixed `GRASS_N` engine. `pub(crate)` so `geology`/`cataclysm` can
    /// read the per-plant constants (r_max/moist_opt/moist_w).
    pub(crate) plants: Vec<PlantType>,
    suit: Vec<f32>,
    shade: Vec<f32>,
    moist: Vec<f32>,
    /// Phase 4-C: soil mineral density per grid cell, same layout as `grass_p`.
    /// Decomposers deposit into it while feeding on carcasses; it decays each
    /// tick and multiplies grass logistic growth. Clamped to [0, 1].
    pub(crate) mineral_field: Vec<f32>,
    grid_acc: f64,
    pub grid_n: u32,
    pub grid_span: f64,
    pub cell: f64,
    pub births: u32,
    pub deaths: u32,
    pub kills: u32,
    pub starved: u32,
    pub aged: u32,
    barrier_blocks: u32,
    sig_attempts: u32,
    sig_blocks: u32,
    deposit_stats: DepositStats,
    fruit_e_h: f64,
    leaf_e_h: f64,
    help_count: u32,
    reject_count: u32,
    help_cost_e_h: f64,
    drain_e_h: f64,
    pub speciation_fired: u32,
    history: Vec<HistorySample>,
    last_sample: u32,
    gene_barrier_strength: f64,
    isolated_groups: HashMap<&'static str, IsolatedGroup>,
    isolated_order: Vec<&'static str>,
    isolation_history: Vec<IsolationHistoryPoint>,
    evolution: Vec<Vec<EvolutionSample>>,
    generation_fitness: Vec<Vec<FitnessRecord>>,
    /// Phase 31: species registry — kind identity as runtime data.
    pub kinds: KindRegistry,
    taken_buf: Vec<f64>,
    spread_buf: Option<Vec<f32>>,
    h_cache: u32,
    season_meta: f64,
    player_x: Option<f64>,
    player_z: Option<f64>,
    pub selection_pressure: SelectionPressure,
    pub speciation_event: Option<SpeciationEvent>,
    last_speciation: f64,
    pub seed: u32,
    pub biome_reg: BiomeRegistry,
    /// Phase 7+: DECO selection-pressure mode. See `save_state::DECO_CHOICE_*`.
    pub deco_eff_choice_mode: u8,
    /// Phase 8: carcass occupancy lock toggle. When `true`, a carcass can be
    /// claimed by one DECO/SCAV at a time (first-arrival priority). Independent
    /// from `deco_eff_choice_mode` so the two mechanisms can be A/B-tested.
    pub carcass_lock_enabled: bool,
    /// Phase 15: species phylogeny state. `species_clades` includes extinct
    /// branches (the fossil record); `speciation_events` is the split log.
    pub species_clades: Vec<SpeciesClade>,
    pub next_clade_id: u32,
    pub speciation_events: Vec<CladeSpeciationEvent>,
    last_clade_spec_tick: u32,
    /// Phase 16: per-clade metabolic burden from niche competition, rebuilt
    /// every detection pass (clade id -> multiplier >= 0). Vec of pairs in
    /// first-seen order — deterministic iteration, no HashMap in wasm.
    pub clade_burdens: Vec<(u32, f64)>,
    /// Phase 17: cataclysm injection log (history + live effect state), in
    /// trigger order. See `cataclysm.rs`; persisted in SaveState v4.
    pub cataclysms: Vec<Cataclysm>,
    pub(crate) next_cataclysm_id: u32,
    /// Phase 18: geological layer (plates, eustatic sea level, seismic
    /// index). Initialized fresh from the seed in `new()` via the DEDICATED
    /// geology RNG stream (never the ecology streams); deserialized from the
    /// compact JSON string in `SavedGeology` during `from_save_state` (v5+
    /// only; pre-v5 gets the serde default: empty plates → step no-op).
    pub geo: GeologicState,
    /// Phase 18: biochemical layer (fast/slow nutrient pools + tectonic
    /// deposit modifier). Baseline at construction matches
    /// `NutrientPools::new()`: fast=snow=0.5, deposit_rate=1.0 — the
    /// grass-growth multiplier reads exactly 1.0 (bit-exact no-op).
    pub nutrients: NutrientPools,
    /// Phase 18: additive terrain offset per grid cell (world-height units).
    /// Initially all zeros; mutated by `apply_uplift` during collision/rift
    /// phases and folded through `rebuild_biome_registry` onto suit/moist.
    pub(crate) elevation_delta: Vec<f32>,
    /// Phase 18: per-bite accumulator feeding the pools at each biochem_step
    /// commit boundary (fast share scaled by deposit_rate; slow share
    /// unaffected). Pure bookkeeping — ecology only sees it via biochem_step.
    pub(crate) pool_acc: [f64; 2],
    /// Phase 32: open-locus genesis operator toggle (default `false` = the
    /// pre-Phase-32 bit-exact trajectory: zero extra RNG draws, `ext` empty).
    /// Flip with `set_genesis`; persisted in SaveState v6 (`GenesisOn`).
    pub genesis_on: bool,
    /// Phase 32: count of gene-duplication events fired (de-novo genesis or
    /// Ohno duplication of the highest-id locus). Pure bookkeeping — it never
    /// feeds the trajectory, only observability / determinism cross-checks.
    pub ext_dup_events: u64,
}

impl Ecology {
    pub fn new(seed: u32) -> Self {
        let n = (GRID_N * GRID_N) as usize;
        let kinds = KindRegistry::legacy();
        let evolution: Vec<Vec<EvolutionSample>> = (0..kinds.len()).map(|_| Vec::new()).collect();
        let generation_fitness: Vec<Vec<FitnessRecord>> = (0..kinds.len()).map(|_| Vec::new()).collect();
        // Phase 33: plant registry default = the three legacy entries; the
        // per-species layers are all sized to `plants.len()` (3 by default).
        let plants = LEGACY_PLANTS.to_vec();
        let pn = plants.len();
        let mut eco = Ecology {
            rng: Mulberry32::new(seed),
            prng: Mulberry32::new(seed.wrapping_add(101)),
            fish_rng: Mulberry32::new(seed ^ 0xF15E5),
            agents: Vec::new(),
            carcasses: Vec::new(),
            next_id: 1,
            time: 0.0,
            tick_count: 0,
            daylight: 1.0,
            season: 0,
            fires: Vec::new(),
            torch: None,
            rain: 0.0,
            grass: vec![0.0; n],
            grass_p: (0..pn).map(|_| vec![0.0f32; n]).collect(),
            suit_eff_by: (0..pn).map(|_| vec![0.0f32; n]).collect(),
            plants,
            suit: vec![0.0; n],
            shade: vec![0.0; n],
            moist: vec![0.0; n],
            mineral_field: vec![0.0; n],
            grid_acc: 0.0,
            grid_n: GRID_N,
            grid_span: GRID_SPAN,
            cell: CELL,
            births: 0,
            deaths: 0,
            kills: 0,
            starved: 0,
            aged: 0,
            barrier_blocks: 0,
            sig_attempts: 0,
            sig_blocks: 0,
            deposit_stats: DepositStats::default(),
            fruit_e_h: 0.0,
            leaf_e_h: 0.0,
            help_count: 0,
            reject_count: 0,
            help_cost_e_h: 0.0,
            drain_e_h: 0.0,
            speciation_fired: 0,
            species_clades: Vec::new(),
            next_clade_id: 0,
            speciation_events: Vec::new(),
            last_clade_spec_tick: 0,
            clade_burdens: Vec::new(),
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
            pool_acc: [0.0; 2],
            history: Vec::new(),
            last_sample: 0,
            gene_barrier_strength: 0.5,
            isolated_groups: HashMap::new(),
            isolated_order: Vec::new(),
            isolation_history: Vec::new(),
            evolution,
            generation_fitness,
            kinds,
            taken_buf: vec![0.0f64; pn],
            spread_buf: None,
            h_cache: 0,
            season_meta: 1.0,
            player_x: None,
            player_z: None,
            selection_pressure: SelectionPressure { plant: 0.0, pred_ratio: 0.0, crowding: 0.0 },
            speciation_event: None,
            last_speciation: 0.0,
            seed,
            biome_reg: BiomeRegistry::default(),
            deco_eff_choice_mode: save_state::DECO_CHOICE_SELECTIVE,
            carcass_lock_enabled: true,
            // Phase 32: open-locus genesis starts OFF (bit-exact legacy run).
            genesis_on: false,
            ext_dup_events: 0,
        };
        eco.init_grass();
        eco.seed_life();
        eco.sample();
        eco
    }

    /// Public constructor with explicit biomes; falls back to single
    /// full-world biome when `biomes` is empty (backward-compatible path).
    pub fn with_biomes(seed: u32, biomes: Vec<Biome>) -> Self {
        let n = (GRID_N * GRID_N) as usize;
        let kinds = KindRegistry::legacy();
        let evolution: Vec<Vec<EvolutionSample>> = (0..kinds.len()).map(|_| Vec::new()).collect();
        let generation_fitness: Vec<Vec<FitnessRecord>> = (0..kinds.len()).map(|_| Vec::new()).collect();
        // Phase 33: plant registry default = the three legacy entries; the
        // per-species layers are all sized to `plants.len()` (3 by default).
        let plants = LEGACY_PLANTS.to_vec();
        let pn = plants.len();
        let mut eco = Ecology {
            rng: Mulberry32::new(seed),
            prng: Mulberry32::new(seed.wrapping_add(101)),
            fish_rng: Mulberry32::new(seed ^ 0xF15E5),
            agents: Vec::new(),
            carcasses: Vec::new(),
            next_id: 1,
            time: 0.0,
            tick_count: 0,
            daylight: 1.0,
            season: 0,
            fires: Vec::new(),
            torch: None,
            rain: 0.0,
            grass: vec![0.0; n],
            grass_p: (0..pn).map(|_| vec![0.0f32; n]).collect(),
            suit_eff_by: (0..pn).map(|_| vec![0.0f32; n]).collect(),
            plants,
            suit: vec![0.0; n],
            shade: vec![0.0; n],
            moist: vec![0.0; n],
            mineral_field: vec![0.0; n],
            grid_acc: 0.0,
            grid_n: GRID_N,
            grid_span: GRID_SPAN,
            cell: CELL,
            births: 0,
            deaths: 0,
            kills: 0,
            starved: 0,
            aged: 0,
            barrier_blocks: 0,
            sig_attempts: 0,
            sig_blocks: 0,
            deposit_stats: DepositStats::default(),
            fruit_e_h: 0.0,
            leaf_e_h: 0.0,
            help_count: 0,
            reject_count: 0,
            help_cost_e_h: 0.0,
            drain_e_h: 0.0,
            speciation_fired: 0,
            species_clades: Vec::new(),
            next_clade_id: 0,
            speciation_events: Vec::new(),
            last_clade_spec_tick: 0,
            clade_burdens: Vec::new(),
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
            pool_acc: [0.0; 2],
            history: Vec::new(),
            last_sample: 0,
            gene_barrier_strength: 0.5,
            isolated_groups: HashMap::new(),
            isolated_order: Vec::new(),
            isolation_history: Vec::new(),
            evolution,
            generation_fitness,
            kinds,
            taken_buf: vec![0.0f64; pn],
            spread_buf: None,
            h_cache: 0,
            season_meta: 1.0,
            player_x: None,
            player_z: None,
            selection_pressure: SelectionPressure { plant: 0.0, pred_ratio: 0.0, crowding: 0.0 },
            speciation_event: None,
            last_speciation: 0.0,
            seed,
            biome_reg: if biomes.is_empty() {
                // Default full-world biome: covers entire SIM_RADIUS, uses global caps.
                let mut reg = BiomeRegistry::new();
                reg.add_biome(Biome {
                    id: 0,
                    name: "full-world",
                    center_x: 0.0,
                    center_z: 0.0,
                    radius: SIM_RADIUS,
                    suit_modifier: 1.0,
                    max_herb: MAX_HERB,
                    max_pred: MAX_PRED,
                    max_scav: MAX_SCAV,
                    max_omn: MAX_OMN,
                    max_fish: Some(MAX_FISH),
                    home_range_mod: 1.0,
                });
                // Aquatic biome overlays the central water region. Registered
                // after full-world so it wins the overlap inside its circle.
                reg.add_biome(Self::aquatic_biome());
                reg
            } else {
                // User biomes win overlaps; aquatic is registered first so
                // custom land biomes keep priority over the water region.
                let mut reg = BiomeRegistry::new();
                reg.add_biome(Self::aquatic_biome());
                for b in biomes { reg.add_biome(b); }
                reg
            },
            deco_eff_choice_mode: save_state::DECO_CHOICE_SELECTIVE,
            carcass_lock_enabled: true,
            // Phase 32: open-locus genesis starts OFF (bit-exact legacy run).
            genesis_on: false,
            ext_dup_events: 0,
        };
        eco.init_grass();
        eco.seed_life();
        eco.sample();
        eco
    }

    // ----- kind registry (Phase 31) ------------------------------------

    pub fn kind_def(&self, id: SpeciesKind) -> &KindDef { self.kinds.def(id) }

    pub fn kind_name(&self, id: SpeciesKind) -> &'static str { self.kinds.name(id) }

    /// Register a new species at runtime (ghost kinds in tests now; Phase 34
    /// kind-forking later). Appends per-kind history/fitness slots so the id
    /// is always indexable.
    pub fn register_kind(&mut self, def: KindDef) -> SpeciesKind {
        let id = self.kinds.register(def);
        while self.evolution.len() <= id as usize { self.evolution.push(Vec::new()); }
        while self.generation_fitness.len() <= id as usize { self.generation_fitness.push(Vec::new()); }
        id
    }

    // ----- plant registry (Phase 33) -------------------------------------

    /// Number of registered plant types (drives every per-plant layer; 3 by
    /// default).
    pub fn plant_count(&self) -> usize { self.plants.len() }

    /// Registered plant types, in registration order.
    pub fn plants(&self) -> &[PlantType] { &self.plants }

    /// A kind's diet data (Phase 33). Unknown ids read the PRED definition
    /// (the registry contract), whose diet is the conservative default —
    /// never a panic.
    pub fn diet_of(&self, id: SpeciesKind) -> &Diet { &self.kind_def(id).diet }

    /// Register a new plant type (Phase 33: the open production layer).
    /// Appends a whole extra layer to every per-plant structure and pads all
    /// live agents' seed cargo with a zero slot. The new layer's
    /// suitability is derived from the current `suit`/`moist` (the
    /// `init_grass` formula) so the type can actually grow; its grass
    /// starts at 0 and fills in via the logistic growth loop.
    ///
    /// This is a NEW era: it adds a layer to the default trajectory, so it
    /// must run *after* the golden anchors are captured; on the default-3
    /// world (no registration) the numeric path stays bit-exact. Returns the
    /// new plant's index.
    pub fn register_plant(&mut self, p: PlantType) -> u32 {
        let idx = self.plants.len();
        let n = self.grass.len();
        // Suitability of the new layer from the current suit/moist (the
        // init_grass formula) so the new plant can grow.
        let mut eff = vec![0.0f32; n];
        for i in 0..n {
            let s = self.suit[i] as f64;
            if s <= 0.0 { continue; }
            let m = self.moist[i] as f64;
            let exp_arg = -(((m - p.moist_opt) / p.moist_w).powi(2));
            eff[i] = (s * aether_libm::exp(exp_arg)) as f32;
        }
        self.plants.push(p);
        self.grass_p.push(vec![0.0f32; n]);
        self.suit_eff_by.push(eff);
        self.taken_buf.push(0.0);
        for a in &mut self.agents {
            if a.cargo.len() < idx + 1 { a.cargo.resize(idx + 1, 0.0); }
        }
        idx as u32
    }

    // ----- Phase 31 Part B: spatial index (feature-gated, zero default cost) --

    /// Live agents as (Vec index, x, z) in ascending index order — the
    /// canonical input for `spatial_index::SpatialIndex::rebuild`.
    #[cfg(feature = "spatial-index")]
    pub fn live_positions(&self) -> Vec<(usize, f64, f64)> {
        self.agents
            .iter()
            .enumerate()
            .filter(|(_, a)| a.alive)
            .map(|(i, a)| (i, a.x, a.z))
            .collect()
    }

    /// One rebuild of the uniform-grid index over the live agent set. Part C
    /// will call this once per tick (agents move) and route `nearest` through
    /// it; today it exists only to make the equivalence tests exercise the
    /// real sim shape.
    #[cfg(feature = "spatial-index")]
    pub fn build_spatial_index(&self) -> crate::spatial_index::SpatialIndex {
        let mut idx = crate::spatial_index::SpatialIndex::new();
        let live = self.live_positions();
        idx.rebuild(&live);
        idx
    }

    // ----- biome helpers -----------------------------------------------

    /// The aquatic biome: overlays the central water region (radius =
    /// SIM_RADIUS x 0.6). suit_modifier=0 (no land grass value from this
    /// biome); carries the FISH population cap and a tighter home range.
    fn aquatic_biome() -> Biome {
        Biome {
            id: 9,
            name: "aquatic",
            center_x: 0.0,
            center_z: 0.0,
            radius: SIM_RADIUS * 0.6,
            suit_modifier: 0.0,
            max_herb: MAX_HERB,
            max_pred: MAX_PRED,
            max_scav: MAX_SCAV,
            max_omn: MAX_OMN,
            max_fish: Some(MAX_FISH),
            home_range_mod: 0.9,
        }
    }

    /// Per-biome alive count for a species. Biomes not covering any agent are
    /// naturally zero-counted, and agents outside all biomes use the default
    /// full-world biome (id=0) when present.
    pub fn biome_count(&self, kind: SpeciesKind, biome_id: u32) -> usize {
        self.agents
            .iter()
            .filter(|a| {
                if !a.alive || a.kind != kind { return false; }
                // Agent's biome = last biome containing its position, or 0
                let agent_bid = self.biome_reg
                    .find_biome(a.x, a.z)
                    .map(|b| b.id)
                    .unwrap_or(0);
                agent_bid == biome_id
            })
            .count()
    }

    /// Per-biome capacity of a kind, resolved through its `KindDef`.
    fn biome_cap_in(&self, kind: SpeciesKind, b: &Biome) -> usize {
        let def = self.kind_def(kind);
        match def.biome_cap {
            BiomeCap::Herb => b.max_herb,
            BiomeCap::Pred => b.max_pred,
            BiomeCap::Scav => b.max_scav,
            BiomeCap::Omn => b.max_omn,
            BiomeCap::Fish => b.max_fish.unwrap_or(def.global_cap),
            BiomeCap::Fallback => def.global_cap,
        }
    }

    /// Effective max for a species at a position. When no biome covers `(x, z)`
    /// the kind's own `global_cap` is used (matches legacy full-world path).
    pub fn biome_max_for(&self, kind: SpeciesKind, x: f64, z: f64) -> usize {
        match self.biome_reg.find_biome(x, z) {
            Some(b) => self.biome_cap_in(kind, b),
            None => self.kind_def(kind).global_cap,
        }
    }

    // ----- initialization ---------------------------------------------------

    fn init_grass(&mut self) {
        let half = GRID_SPAN / 2.0;
        for j in 0..GRID_N as usize {
            for i in 0..GRID_N as usize {
                let x = -half + (i as f64 + 0.5) * CELL;
                let z = -half + (j as f64 + 0.5) * CELL;
                let idx = j * GRID_N as usize + i;
                self.suit[idx] = plant_suitability(x, z) as f32;
            }
        }
        // Water edge moisture: BFS from water-adjacent cells (JS L172-191)
        let mut frontier: Vec<usize> = Vec::new();
        for j in 0..GRID_N as usize {
            for i in 0..GRID_N as usize {
                let x = -half + (i as f64 + 0.5) * CELL;
                let z = -half + (j as f64 + 0.5) * CELL;
                if terrain_height(x, z) < WATER_LEVEL + 1.2 {
                    frontier.push(j * GRID_N as usize + i);
                }
            }
        }
        let mut d = 0;
        while d < 12 && !frontier.is_empty() {
            let mut next = Vec::new();
            for &idx in &frontier {
                if self.moist[idx] > 0.0 { continue; }
                self.moist[idx] = (0.45 - d as f64 * 0.035) as f32;
                let i = (idx % GRID_N as usize) as isize;
                let j = (idx / GRID_N as usize) as isize;
                for (ni, nj) in [(i + 1, j), (i - 1, j), (i, j + 1), (i, j - 1)] {
                    if ni >= 0 && ni < GRID_N as isize && nj >= 0 && nj < GRID_N as isize {
                        next.push((nj as usize) * GRID_N as usize + ni as usize);
                    }
                }
            }
            frontier = next;
            d += 1;
        }
        // Initial grass allocation: total split by rMax x moisture suitability
        let pn = self.plants.len();
        for j in 0..GRID_N as usize {
            for i in 0..GRID_N as usize {
                let x = -half + (i as f64 + 0.5) * CELL;
                let z = -half + (j as f64 + 0.5) * CELL;
                let idx = j * GRID_N as usize + i;
                let suit = self.suit[idx];
                if suit <= 0.02 { continue; }
                let patch = 0.55 + 0.45 * aether_libm::sin(x * 0.031 + z * 0.017) * aether_libm::sin(z * 0.023 - x * 0.011);
                let total_init = (suit as f64) * (patch * 0.88 + self.rng.next_f64() * 0.18).clamp(0.0, 1.0);
                let mut wsum = 0.0f64;
                for s in 0..pn {
                    let tr = &self.plants[s];
                    let exp_arg = -(((self.moist[idx] as f64 - tr.moist_opt) / tr.moist_w).powi(2));
                    wsum += tr.r_max * aether_libm::exp(exp_arg);
                }
                if wsum <= 0.0 { wsum = 1.0; }
                for s in 0..pn {
                    let tr = &self.plants[s];
                    let exp_arg = -(((self.moist[idx] as f64 - tr.moist_opt) / tr.moist_w).powi(2));
                    let eff = aether_libm::exp(exp_arg);
                    self.suit_eff_by[s][idx] = ((suit as f64) * eff) as f32;
                    self.grass_p[s][idx] = (total_init * (tr.r_max * eff) / wsum) as f32;
                }
                self.grass[idx] = total_init as f32;
            }
        }
    }

    fn seed_life(&mut self) {
        let herb_count = 132usize;
        let pred_count = 12usize;
        let mut tries = 0;
        while self.count(HERB) < herb_count && tries < 9000 {
            tries += 1;
            let a = self.rng.next_f64() * std::f64::consts::PI * 2.0;
            let r = 95.0 + self.rng.next_f64().sqrt() * 360.0;
            let x = aether_libm::cos(a) * r;
            let z = aether_libm::sin(a) * r;
            let h = terrain_height(x, z);
            if h < WATER_LEVEL + 2.0 || h > 72.0 || slope_at(x, z) > 0.72 { continue; }
            let g = self.random_genome(HERB);
            let age_h = 20.0 + self.rng.next_f64() * 90.0;
            self.spawn(HERB, x, z, Some(g), 0, age_h);
        }
        tries = 0;
        while self.count(PRED) < pred_count && tries < 5000 {
            tries += 1;
            let a = self.rng.next_f64() * std::f64::consts::PI * 2.0;
            let r = 190.0 + self.rng.next_f64().sqrt() * 300.0;
            let x = aether_libm::cos(a) * r;
            let z = aether_libm::sin(a) * r;
            let h = terrain_height(x, z);
            if h < WATER_LEVEL + 1.5 || h > 85.0 || slope_at(x, z) > 0.75 { continue; }
            let g = self.random_genome(PRED);
            let age_p = 30.0 + self.rng.next_f64() * 110.0;
            self.spawn(PRED, x, z, Some(g), 0, age_p);
        }


        let herd: [(f64, f64); 18] = [
            (215.0, 292.0), (228.0, 304.0), (200.0, 305.0), (236.0, 286.0), (205.0, 318.0),
            (192.0, 330.0), (222.0, 315.0), (244.0, 294.0), (208.0, 278.0), (186.0, 292.0),
            (230.0, 326.0), (198.0, 285.0), (216.0, 305.0), (240.0, 312.0), (202.0, 295.0),
            (226.0, 296.0), (212.0, 324.0), (234.0, 298.0),
        ];
        for (x, z) in herd.iter() {
            let h = terrain_height(*x, *z);
            if h < WATER_LEVEL + 1.2 || slope_at(*x, *z) > 0.8 { continue; }
            let g = self.random_genome(HERB);
            let age_h2 = 25.0 + self.rng.next_f64() * 55.0;
                if let Some(_) = self.spawn(HERB, *x, *z, Some(g), 0, age_h2) {
                let idx = self.agents.len() - 1;
                self.agents[idx].genome.size = 1.12 + self.rng.next_f64() * 0.3;
                self.agents[idx].energy = self.agents[idx].max_energy * 0.68;
            }
        }
        for (x, z) in [(148.0, 300.0), (168.0, 255.0)].iter() {
            if terrain_height(*x, *z) > WATER_LEVEL + 1.2 {
                let g = self.random_genome(PRED);
                let age_p2 = 35.0 + self.rng.next_f64() * 50.0;
                if let Some(_) = self.spawn(PRED, *x, *z, Some(g), 0, age_p2) {
                    let idx = self.agents.len() - 1;
                    self.agents[idx].genome.size = 1.1 + self.rng.next_f64() * 0.25;
                }
            }
        }
        // SCAV batch
        let mut scav_tries = 0;
        while self.count(SCAV) < 8 && scav_tries < 3000 {
            scav_tries += 1;
            let a = self.rng.next_f64() * std::f64::consts::PI * 2.0;
            let r = 150.0 + self.rng.next_f64().sqrt() * 300.0;
            let x = aether_libm::cos(a) * r;
            let z = aether_libm::sin(a) * r;
            let h = terrain_height(x, z);
            if h < WATER_LEVEL + 1.5 || h > 80.0 || slope_at(x, z) > 0.75 { continue; }
            let g = self.random_genome(PRED); // SCAV uses PRED genome template per JS L278
let age_sc = 25.0 + self.rng.next_f64() * 60.0;
                self.spawn(SCAV, x, z, Some(g), 0, age_sc);
        }
        // OMN batch
        let mut omn_tries = 0;
        while self.count(OMN) < 12 && omn_tries < 3000 {
            omn_tries += 1;
            let a = self.rng.next_f64() * std::f64::consts::PI * 2.0;
            let r = 120.0 + self.rng.next_f64().sqrt() * 320.0;
            let x = aether_libm::cos(a) * r;
            let z = aether_libm::sin(a) * r;
            let h = terrain_height(x, z);
            if h < WATER_LEVEL + 1.5 || h > 85.0 || slope_at(x, z) > 0.75 { continue; }
            let g = self.random_genome(HERB); // OMN uses HERB genome template per JS L290
let age_om = 25.0 + self.rng.next_f64() * 60.0;
                self.spawn(OMN, x, z, Some(g), 0, age_om);
        }
        // FISH batch — aquatic zone only (terrain below WATER_LEVEL + 1.5)
        let mut fish_tries = 0;
        while self.count(FISH) < 30 && fish_tries < 3000 {
            fish_tries += 1;
            let a = self.fish_rng.next_f64() * std::f64::consts::PI * 2.0;
            let r = 80.0 + self.fish_rng.next_f64().sqrt() * 200.0;
            let x = aether_libm::cos(a) * r;
            let z = aether_libm::sin(a) * r;
            let h = terrain_height(x, z);
            // Spawn preferentially in the shallow-water edge zone (1.0..1.5)
            // where plant_suitability may yield non-zero grass for grazing.
            if h < WATER_LEVEL + 1.0 || h >= WATER_LEVEL + 1.5 || slope_at(x, z) > 0.8 { continue; }
            let g = self.random_genome(FISH);
            let age_f = 5.0 + self.fish_rng.next_f64() * 20.0;
            self.spawn(FISH, x, z, Some(g), 0, age_f);
        }
        // DECO batch (Phase 4-C): random land positions, away from water edge.
        let mut deco_tries = 0;
        while self.count(DECO) < 15 && deco_tries < 3000 {
            deco_tries += 1;
            let a = self.rng.next_f64() * std::f64::consts::PI * 2.0;
            let r = 130.0 + self.rng.next_f64().sqrt() * 320.0;
            let x = aether_libm::cos(a) * r;
            let z = aether_libm::sin(a) * r;
            let h = terrain_height(x, z);
            if h < WATER_LEVEL + 2.0 || h > 82.0 || slope_at(x, z) > 0.75 { continue; }
            let g = self.random_genome(DECO);
            let age_d = 20.0 + self.rng.next_f64() * 50.0;
            self.spawn(DECO, x, z, Some(g), 0, age_d);
        }
    }

    // ----- RNG helpers ------------------------------------------------------

    #[inline]
    fn triangular(&mut self, a: f64, c: f64, b: f64) -> f64 {
        let u = self.rng.next_f64();
        let f = (c - a) / (b - a);
        if u < f {
            a + (u * (b - a) * (c - a)).sqrt()
        } else {
            b - ((1.0 - u) * (b - a) * (b - c)).sqrt()
        }
    }

    // ----- genome factories -------------------------------------------------

    /// Phase 19: deterministic morphology from agent ID (no RNG consumed).
    /// Uses a 16-bit FNV split so JS can replicate from SLOT_ID.
    #[inline]
    fn morpho_pair(id: u32) -> (f64, f64) {
        let h = id.wrapping_mul(0x9E3779B9);
        let hr = 0.6 + ((h & 0xFFFF) as f64 / 65535.0) * 0.8;
        let ls = 2.0 + (((h >> 16) & 0xFFFF) as f64 / 65535.0) * 2.0;
        (hr, ls)
    }

    fn random_genome(&mut self, kind: SpeciesKind) -> Genome {
        let (hr, ls) = Self::morpho_pair(self.next_id);
        let template = self.kind_def(kind).genome_template;
        match template {
            GenomeTemplate::Herb => {
            Genome {
                speed: 3.8 + self.rng.next_f64() * 4.2,
                sense: 24.0 + self.rng.next_f64() * 44.0,
                size: 0.78 + self.rng.next_f64() * 0.5,
                metabolism: 0.82 + self.rng.next_f64() * 0.38,
                lifespan: 260.0 + self.rng.next_f64() * 220.0,
                hue: self.rng.next_f64(),
                bold: self.prng.next_f64(),
                curious: self.prng.next_f64(),
                social: self.prng.next_f64(),
                help_thr: Some((self.rng.next_f64() + self.rng.next_f64() - 1.0) * 1.8),
                pref_sig: Some(self.triangular(SIG_PREF_INIT - 0.15, SIG_PREF_INIT, SIG_PREF_INIT + 0.3).clamp(0.58, 1.7)),
                tol_sig: Some(self.triangular(0.15, 0.20, 0.25).clamp(SIG_TOL_MIN, SIG_TOL_MAX)),
                fin_color: 0.0,
                deco_efficiency: 1.0,
            hull_ratio: hr, limb_scale: ls,
            ext: Vec::new(),
            }
            }
            GenomeTemplate::Fish => {
            Genome {
                speed: 3.2 + self.fish_rng.next_f64() * 3.0,
                sense: 20.0 + self.fish_rng.next_f64() * 30.0,
                size: 0.7 + self.fish_rng.next_f64() * 0.45,
                metabolism: 0.7 + self.fish_rng.next_f64() * 0.3,
                lifespan: 300.0 + self.fish_rng.next_f64() * 200.0,
                hue: self.fish_rng.next_f64(),
                bold: self.fish_rng.next_f64(),
                curious: self.fish_rng.next_f64(),
                social: self.fish_rng.next_f64(),
                help_thr: None,
                pref_sig: None,
                tol_sig: None,
                fin_color: self.fish_rng.next_f64(),
                deco_efficiency: 1.0,
            hull_ratio: hr, limb_scale: ls,
            ext: Vec::new(),
            }
            }
            GenomeTemplate::Deco => {
            Genome {
                speed: 3.0 + self.rng.next_f64() * 2.4,
                sense: 30.0 + self.rng.next_f64() * 40.0,
                size: 0.7 + self.rng.next_f64() * 0.4,
                metabolism: 0.7 + self.rng.next_f64() * 0.3,
                lifespan: 280.0 + self.rng.next_f64() * 180.0,
                hue: self.rng.next_f64(),
                bold: self.prng.next_f64(),
                curious: self.prng.next_f64(),
                social: self.prng.next_f64(),
                help_thr: None,
                pref_sig: None,
                tol_sig: None,
                fin_color: 0.0,
                deco_efficiency: 0.5 + self.rng.next_f64(),
            hull_ratio: hr, limb_scale: ls,
            ext: Vec::new(),
            }
            }
            GenomeTemplate::General => {
            Genome {
                speed: 4.6 + self.rng.next_f64() * 3.4,
                sense: 34.0 + self.rng.next_f64() * 46.0,
                size: 0.92 + self.rng.next_f64() * 0.42,
                metabolism: 0.95 + self.rng.next_f64() * 0.4,
                lifespan: 300.0 + self.rng.next_f64() * 220.0,
                hue: self.rng.next_f64(),
                bold: self.prng.next_f64(),
                curious: self.prng.next_f64(),
                social: self.prng.next_f64(),
                help_thr: None,
                pref_sig: None,
                tol_sig: None,
                fin_color: 0.0,
                deco_efficiency: 1.0,
            hull_ratio: hr, limb_scale: ls,
            ext: Vec::new(),
            }
            }
        }
    }

    fn mutate(&mut self, g: &Genome, kind: SpeciesKind) -> Genome {
        let mut out = g.clone();
        let pop_size = self.count(kind);
        let (sp_lo, sp_hi) = self.kind_def(kind).speed_clamp;
        let mut_fin = self.kind_def(kind).mutates_fin_color;
        let mut_deco = self.kind_def(kind).mutates_deco_eff;
        let drift = if pop_size < 20 { 1.0 + (20.0 - pop_size as f64) / 20.0 } else { 1.0 };
        out.speed = (g.speed * (1.0 + (self.rng.next_f64() - 0.5) * 0.09 * drift)).clamp(sp_lo, sp_hi);
        out.sense = (g.sense * (1.0 + (self.rng.next_f64() - 0.5) * 0.1 * drift)).clamp(14.0, 130.0);
        out.size = (g.size * (1.0 + (self.rng.next_f64() - 0.5) * 0.07 * drift)).clamp(0.58, 1.7);
        out.metabolism = (g.metabolism * (1.0 + (self.rng.next_f64() - 0.5) * 0.06 * drift)).clamp(0.62, 1.6);
        out.lifespan = (g.lifespan * (1.0 + (self.rng.next_f64() - 0.5) * 0.08 * drift)).clamp(180.0, 560.0);
        out.hue = (g.hue + (self.rng.next_f64() - 0.5) * 0.06 * drift + 1.0) % 1.0;
        out.bold = (g.bold + (self.prng.next_f64() - 0.5) * 0.14 * drift).clamp(0.0, 1.0);
        out.curious = (g.curious + (self.prng.next_f64() - 0.5) * 0.14 * drift).clamp(0.0, 1.0);
        out.social = (g.social + (self.prng.next_f64() - 0.5) * 0.14 * drift).clamp(0.0, 1.0);
        if mut_fin {
            out.fin_color = (g.fin_color + (self.fish_rng.next_f64() - 0.5) * 0.1 * drift).clamp(0.0, 1.0);
        }
        if mut_deco {
            out.deco_efficiency = (g.deco_efficiency * (1.0 + (self.rng.next_f64() - 0.5) * 0.1 * drift)).clamp(0.5, 1.5);
        }
        if let Some(h) = g.help_thr {
            out.help_thr = Some((h + (self.rng.next_f64() - 0.5) * 1.2 * drift).clamp(-5.0, 5.0));
        }
        if let Some(p) = g.pref_sig {
            out.pref_sig = Some((p + (self.rng.next_f64() - 0.5) * 2.0 * SIG_MUT_PREF * drift).clamp(0.58, 1.7));
        }
        if let Some(t) = g.tol_sig {
            out.tol_sig = Some((t + (self.rng.next_f64() - 0.5) * 2.0 * SIG_MUT_TOL * drift).clamp(SIG_TOL_MIN, SIG_TOL_MAX));
        }
        // Phase 32: gene-duplication operator. When genesis is ON exactly ONE
        // extra main-stream draw (`dup_roll`) is made here, at this fixed
        // position after every core-locus mutation draw, INDEPENDENT of the
        // current `ext` length (the §4 discipline: locus growth never changes
        // the per-agent draw budget). When genesis is OFF this block is
        // skipped entirely: zero draws, `out.ext` stays empty, and the
        // trajectory is bit-identical to pre-Phase-32.
        if self.genesis_on {
            let dup_roll = self.rng.next_f64();
            if dup_roll < P_DUP {
                // Add. Highest-id locus = last (the list is id-sorted).
                match out.ext.last() {
                    Some(last) => {
                        // Gene duplication (Ohno): copy the highest-id locus
                        // verbatim with a fresh id = max+1. The sole legal
                        // source of a NEW heritable slot once ext is non-empty.
                        let new_id = last.id + 1;
                        out.ext.push(Locus {
                            id: new_id,
                            value: last.value,
                            sigma: last.sigma,
                            clamp_min: last.clamp_min,
                            clamp_max: last.clamp_max,
                        });
                    }
                    None => {
                        // De-novo genesis: the first locus of an empty ext.
                        // Value derived from the same roll — `dup_roll/P_DUP`
                        // is uniform on [0,1) whenever this branch fires — so
                        // no second draw is needed and the budget stays
                        // independent of ext length. This is how a brand-new
                        // heritable trait can appear (OEE condition ①).
                        let v = (dup_roll / P_DUP).clamp(LOCUS_CLAMP_MIN, LOCUS_CLAMP_MAX);
                        out.ext.push(Locus {
                            id: 0,
                            value: v,
                            sigma: LOCUS_SIGMA,
                            clamp_min: LOCUS_CLAMP_MIN,
                            clamp_max: LOCUS_CLAMP_MAX,
                        });
                    }
                }
                self.ext_dup_events += 1;
            } else if dup_roll < P_DUP + P_DEL {
                // Deletion: drop the highest-id locus (last). No-op on empty.
                if !out.ext.is_empty() {
                    out.ext.pop();
                }
            }
            // else: no-op — `ext` unchanged, no further draws.
        }
        out
    }

    /// Public for registry / ghost-kind unit tests (see phase31_registry_test).
    pub fn count(&self, kind: SpeciesKind) -> usize {
        self.agents.iter().filter(|a| a.alive && a.kind == kind).count()
    }

    fn mix_genome(&mut self, a: &Genome, b: &Genome, kind: SpeciesKind) -> Genome {
        let mut g = Genome {
            speed: (a.speed + b.speed) * 0.5,
            sense: (a.sense + b.sense) * 0.5,
            size: (a.size + b.size) * 0.5,
            metabolism: (a.metabolism + b.metabolism) * 0.5,
            lifespan: (a.lifespan + b.lifespan) * 0.5,
            hue: (a.hue + b.hue) * 0.5,
            bold: (a.bold + b.bold) * 0.5,
            curious: (a.curious + b.curious) * 0.5,
            social: (a.social + b.social) * 0.5,
            help_thr: None,
            pref_sig: None,
            tol_sig: None,
            fin_color: (a.fin_color + b.fin_color) * 0.5,
            deco_efficiency: (a.deco_efficiency + b.deco_efficiency) * 0.5,
            hull_ratio: (a.hull_ratio + b.hull_ratio) * 0.5,
            limb_scale: (a.limb_scale + b.limb_scale) * 0.5,
            ext: Vec::new(),
        };
        if let (Some(x), Some(y)) = (a.help_thr, b.help_thr) { g.help_thr = Some((x + y) * 0.5); }
        if let (Some(x), Some(y)) = (a.pref_sig, b.pref_sig) { g.pref_sig = Some((x + y) * 0.5); }
        if let (Some(x), Some(y)) = (a.tol_sig, b.tol_sig) { g.tol_sig = Some((x + y) * 0.5); }
        // Phase 32: id-paired ext-locus inheritance (before the duplication
        // operator, which runs inside the mutate call below). Core-locus
        // averaging above is unchanged.
        g.ext = Self::mix_ext(&a.ext, &b.ext);
        self.mutate(&g, kind)
    }

    /// Phase 32: id-paired ext-locus inheritance. Over the UNION of the two
    /// parents' ids: present in both → value/sigma/clamp averaged; present
    /// in only one → that locus copied verbatim (the absent parent's slot is
    /// dropped, not zero-filled). The result is id-sorted, which keeps
    /// `ext.last()`/`ext.pop()` in the duplication operator pointing at the
    /// highest-id locus. Deterministic: pure function of the two inputs.
    fn mix_ext(a: &[Locus], b: &[Locus]) -> Vec<Locus> {
        if a.is_empty() && b.is_empty() { return Vec::new(); }
        let mut ids: Vec<u32> = a.iter().map(|l| l.id).chain(b.iter().map(|l| l.id)).collect();
        ids.sort_unstable();
        ids.dedup();
        let mut out = Vec::with_capacity(ids.len());
        for &id in &ids {
            let la = a.iter().find(|l| l.id == id);
            let lb = b.iter().find(|l| l.id == id);
            match (la, lb) {
                (Some(x), Some(y)) => out.push(Locus {
                    id,
                    value: (x.value + y.value) * 0.5,
                    sigma: (x.sigma + y.sigma) * 0.5,
                    clamp_min: (x.clamp_min + y.clamp_min) * 0.5,
                    clamp_max: (x.clamp_max + y.clamp_max) * 0.5,
                }),
                (Some(x), None) => out.push(x.clone()),
                (None, Some(y)) => out.push(y.clone()),
                (None, None) => {}
            }
        }
        out
    }

    // ----- entity management ------------------------------------------------

    fn spawn(&mut self, kind: SpeciesKind, x: f64, z: f64, genome: Option<Genome>, generation: u32, age: f64) -> Option<&mut Agent> {
        // Use biome-specific cap when available; otherwise fall back to global.
        let def_max_e = self.kind_def(kind).max_energy;
        let def_repro_cd = self.kind_def(kind).repro_cooldown;
        let (se0, se1) = self.kind_def(kind).start_energy;
        let cap = self.biome_max_for(kind, x, z);
        if self.count(kind) >= cap { return None; }
        let g = match genome {
            Some(g) => g,
            None => {
                let rg = self.random_genome(kind);
                self.mutate(&rg, kind)
            }
        };
        let max_e = def_max_e;
        let repro_cd = def_repro_cd;
        let range_base = self.kind_def(kind).range_base;
        let h = terrain_height(x, z);
        let curious_val = g.curious;
        // JS spawn evaluates fields in source order: heading consumes rng FIRST, then energy.
        // So we must match that order exactly — heading draws before energy draws.
        // FISH draws come from the dedicated fish_rng stream (Phase 4-B isolation).
        let (heading_val, start_energy, bite0, repro0, think0, phase0, curiosity0) = if kind == FISH {
            let hv = self.fish_rng.next_f64() * std::f64::consts::PI * 2.0;
            let se = 60.0 + self.fish_rng.next_f64() * 18.0;
            let bi = self.fish_rng.next_f64() * 0.8;
            let rp = self.fish_rng.next_f64() * repro_cd;
            let th = self.fish_rng.next_f64() * 0.4;
            let ph = self.fish_rng.next_f64() * std::f64::consts::PI * 2.0;
            let cu = 0.3 + self.fish_rng.next_f64() * 0.4;
            (hv, se, bi, rp, th, ph, cu)
        } else {
            let hv = self.rng.next_f64() * std::f64::consts::PI * 2.0;
            let se = se0 + self.rng.next_f64() * se1;
            (hv, se, 0.0, 0.0, 0.0, 0.0, 0.0)
        };
        let a = Agent {
            id: { let v = self.next_id; self.next_id += 1; v },
            kind,
            x, z, y: h,
            prev_x: x, prev_z: z, prev_heading: 0.0,
            heading: heading_val,
            speed: 0.0, desired_speed: 0.0,
            energy: start_energy, max_energy: max_e,
            age,
            genome: g,
            generation,
            state: "wander",
            bite: if kind == FISH { bite0 } else { self.rng.next_f64() * 0.8 },
            repro: if kind == FISH { repro0 } else { self.rng.next_f64() * repro_cd },
            target_x: x, target_z: z, target_id: 0,
            think: if kind == FISH { think0 } else { self.rng.next_f64() * 0.4 },
            phase: if kind == FISH { phase0 } else { self.rng.next_f64() * std::f64::consts::PI * 2.0 },
            alive: true,
            activity: 0.5,
            hydration: 1.0,
            cargo: vec![0.0f32; self.plants.len()],
            trail_t: vec![0.0; TRAIL_SLOTS],
            trail_x: vec![0.0; TRAIL_SLOTS],
            trail_z: vec![0.0; TRAIL_SLOTS],
            trail_head: 0, trail_n: 0,
            trail_slot: -1,
            fatigue: 0.0, fear: 0.0,
            image: 0, help_cd: 0.0,
            curiosity: if kind == FISH { curiosity0 } else { 0.3 + self.rng.next_f64() * 0.4 },
            home_x: x, home_z: z,
            range: range_base * (0.85 + curious_val * 0.5),
            lineage: None,
            clade_id: 0,
            offspring: 0,
        };
        let idx = self.agents.len();
        self.agents.push(a);
        Some(&mut self.agents[idx])
    }

    // ----- grass field helpers ----------------------------------------------

    fn cell_index(&self, x: f64, z: f64) -> isize {
        let i = ((x + self.grid_span / 2.0) / self.cell).floor() as isize;
        let j = ((z + self.grid_span / 2.0) / self.cell).floor() as isize;
        if i < 0 || j < 0 || i as u32 >= self.grid_n || j as u32 >= self.grid_n { return -1; }
        j * self.grid_n as isize + i
    }

    fn cell_coord(idx: isize, out: &mut [f64; 2]) {
        let half = GRID_SPAN / 2.0;
        let i = (idx % GRID_N as isize) as usize;
        let j = (idx / GRID_N as isize) as usize;
        out[0] = -half + (i as f64 + 0.5) * CELL;
        out[1] = -half + (j as f64 + 0.5) * CELL;
    }

    fn clamp_home(&self, a_x: f64, a_z: f64, a_home_x: f64, a_home_z: f64, a_range: f64, tx: f64, tz: f64) -> (f64, f64) {
        // Biome at home position modulates effective range: agents anchored in
        // a smaller biome (lower home_range_mod) roam proportionally less.
        let range_mod = self.biome_reg.home_range_mod(a_home_x, a_home_z);
        let eff_range = a_range * range_mod;
        let dx = tx - a_home_x;
        let dz = tz - a_home_z;
        let d = (dx * dx + dz * dz).sqrt();
        if d > eff_range {
            let k = (eff_range * 0.6 / d).max(0.0);
            return (a_home_x + dx * k, a_home_z + dz * k);
        }
        (tx, tz)
    }

    fn steer(a_x: &mut f64, a_z: &mut f64, a_heading: &mut f64, a_desired_speed: &mut f64,
             tx: f64, tz: f64, dt: f64, desired_speed: f64) {
        let dx = tx - *a_x;
        let dz = tz - *a_z;
        if dx * dx + dz * dz > 0.01 {
            let want = aether_libm::atan2(dx, dz);
            let mut diff = want - *a_heading;
            while diff > std::f64::consts::PI { diff -= std::f64::consts::PI * 2.0; }
            while diff < -std::f64::consts::PI { diff += std::f64::consts::PI * 2.0; }
            let turn = diff.clamp(-2.8 * dt, 2.8 * dt);
            *a_heading += turn;
        }
        *a_desired_speed = desired_speed;
    }

    fn avoid(nx: f64, nz: f64, cur_x: f64, cur_z: f64, cur_heading: &mut f64) -> (f64, f64) {
        if (nx * nx + nz * nz) > (SIM_RADIUS - 25.0).powi(2) {
            *cur_heading = aether_libm::atan2(-cur_x, -cur_z);
            let h = *cur_heading;
            return (cur_x + aether_libm::sin(h) * 0.5, cur_z + aether_libm::cos(h) * 0.5);
        }
        let th = terrain_height(nx, nz);
        if th < WATER_LEVEL + 0.65 {
            let mut h = *cur_heading;
            h += std::f64::consts::FRAC_PI_2 * 0.6;
            *cur_heading = h;
            return (cur_x, cur_z);
        }
        (nx, nz)
    }

    // ----- nearest search (skips idx) ---------------------------------------

    fn nearest<F: Fn(&Agent) -> bool>(&self, idx: usize, kind: SpeciesKind, max_dist: f64, pred: Option<F>) -> Option<(usize, f64)> {
        let best_d2 = max_dist * max_dist;
        let ai_x = self.agents[idx].x;
        let ai_z = self.agents[idx].z;
        let mut best_j: Option<usize> = None;
        let mut best_d2_cur = best_d2;
        for (j, b) in self.agents.iter().enumerate() {
            if j == idx { continue; }
            if !b.alive || b.kind != kind { continue; }
            if let Some(ref p) = pred {
                if !p(b) { continue; }
            }
            let dx = b.x - ai_x;
            let dz = b.z - ai_z;
            let d2 = dx * dx + dz * dz;
            if d2 < best_d2_cur {
                best_d2_cur = d2;
                best_j = Some(j);
            }
        }
        best_j.map(|j| (j, best_d2_cur.sqrt()))
    }

    // ----- find grass cell --------------------------------------------------

    // JS _findGrass semantics: returns {x, z, amount, cell} with cell = -1 when
    // nothing found. Comparison constants must be f64 (JS reads the Float32
    // value into a double before comparing).
    fn find_grass(&mut self, idx: usize) -> (f64, f64, isize, f64) {
        let ci = self.cell_index(self.agents[idx].x, self.agents[idx].z);
        if ci >= 0 && (self.grass[ci as usize] as f64) > 0.2 {
            return (self.agents[idx].x, self.agents[idx].z, ci, self.grass[ci as usize] as f64);
        }
        let sense = self.agents[idx].genome.sense;
        let fish_probes = self.agents[idx].kind == FISH;
        let (mut best_score, mut best_cell) = (-1e300f64, -1isize);
        for _k in 0..14 {
            let ang = if fish_probes { self.fish_rng.next_f64() } else { self.rng.next_f64() } * std::f64::consts::PI * 2.0;
            let r = (if fish_probes { self.fish_rng.next_f64() } else { self.rng.next_f64() }).sqrt() * sense;
            let cx = self.agents[idx].x + aether_libm::cos(ang) * r;
            let cz = self.agents[idx].z + aether_libm::sin(ang) * r;
            let cidx = self.cell_index(cx, cz);
            if cidx < 0 { continue; }
            let b = self.grass[cidx as usize] as f64;
            if b < 0.08 { continue; }
            let score = b - r / sense * 0.32;
            if score > best_score {
                best_score = score;
                best_cell = cidx;
            }
        }
        if best_cell < 0 { return (0.0, 0.0, -1, 0.0); }
        let mut coord = [0.0f64; 2];
        Self::cell_coord(best_cell, &mut coord);
        (coord[0], coord[1], best_cell, self.grass[best_cell as usize] as f64)
    }

    // ----- take grass (water-filling, bit-exact) ----------------------------

    fn take_grass(&mut self, a_idx: usize, ci: usize, want: f64) -> f64 {
        let take = want.min(self.grass[ci] as f64);
        if take <= 0.0 { return 0.0; }
        let pn = self.plants.len();
        let mut wsum = 0.0f64;
        let mut w = vec![0.0f64; pn];
        for s in 0..pn {
            w[s] = self.grass_p[s][ci] as f64 * self.plants[s].palatability;
            wsum += w[s];
        }
        if wsum <= 0.0 { return 0.0; }
        self.taken_buf.fill(0.0);
        let mut actual = 0.0f64;
        let mut remaining = take;
        let mut open: Vec<usize> = (0..pn).collect();
        for _round in 0..pn {
            if remaining <= 1e-9 || open.is_empty() { break; }
            let wsum_open: f64 = open.iter().map(|&s| w[s]).sum();
            if wsum_open <= 0.0 { break; }
            let r0 = remaining;
            let mut next = Vec::new();
            for &s in &open {
                let share = r0 * (w[s] / wsum_open);
                let part = share.min(self.grass_p[s][ci] as f64);
                self.grass_p[s][ci] = ((self.grass_p[s][ci] as f64) - part) as f32;
                self.taken_buf[s] += part;
                actual += part;
                remaining -= part;
                if part >= share - 1e-9 && self.grass_p[s][ci] > 1e-9 {
                    next.push(s);
                }
            }
            open = next;
        }
        self.grass[ci] = ((self.grass[ci] as f64) - actual) as f32;
        // fruiting path — kept for checksum parity even though Phase 2 excludes fruiting behavior
        let mut fruit = 0.0f64;
        for s in 0..pn {
            let f = self.taken_buf[s] * self.plants[s].fruiting;
            if f <= 0.0 { continue; }
            fruit += f;
            if self.agents[a_idx].cargo.len() > s {
                self.agents[a_idx].cargo[s] = ((self.agents[a_idx].cargo[s] as f64) + f * SEED_LOAD) as f32;
            }
        }
        if fruit > 0.0 {
            let e = fruit * FRUIT_ENERGY * 62.0;
            self.agents[a_idx].energy = (self.agents[a_idx].energy + e).min(self.agents[a_idx].max_energy);
            if self.agents[a_idx].kind == HERB {
                self.fruit_e_h += e;
            }
        }
        actual
    }

    // ----- try help (indirect reciprocity) ----------------------------------

    fn try_help(&mut self, a_idx: usize, ci: usize) -> bool {
        if !HELP_ENABLED { return false; }
        let a = &self.agents[a_idx];
        if a.kind != HERB || a.age < HELP_AGE_MIN || a.energy < HELP_NEEDY * a.max_energy || a.help_cd > 0.0 || a.state != "graze" && a.state != "patrol" && a.state != "wander" {
            // JS checks: a.state === 'flee' → skip; other states participate
            // Actually JS: a.state === 'flee' → false (already filtered by caller)
            // In _updateHerb, before reaching here a.state can be 'flee','graze','wander'
            // But we check inside: JS says if state==='flee' return false
        }
        // Simplified: JS only checks for 'flee'. Let me replicate exactly.
        // Re-do based on JS L513 condition chain.
        drop(a);  // already dropped
        // Read fresh
        if self.agents[a_idx].kind != HERB { return false; }
        if self.agents[a_idx].age < HELP_AGE_MIN { return false; }
        if self.agents[a_idx].energy < HELP_NEEDY * self.agents[a_idx].max_energy { return false; }
        if self.agents[a_idx].help_cd > 0.0 { return false; }
        if self.agents[a_idx].state == "flee" { return false; }
        // Winter modulation
        let is_winter_modulated = self.season == 3 && if self.h_cache > 0 { self.h_cache } else { 0 } < WIN_TEST_THRESHOLD_MIN;
        let effective_help_thr = if is_winter_modulated {
            (self.agents[a_idx].genome.help_thr.unwrap_or(0.0) * WINTER_HELP_SCALE)
        } else {
            self.agents[a_idx].genome.help_thr.unwrap_or(0.0)
        };
        let mut needy: Option<usize> = None;
        let mut needy_d2 = HELP_RADIUS * HELP_RADIUS;
        let mut willing: Option<usize> = None;
        let mut willing_d2 = HELP_RADIUS * HELP_RADIUS;
        for (j, n) in self.agents.iter().enumerate() {
            if j == a_idx { continue; }
            if !n.alive || n.kind != HERB { continue; }
            if n.energy >= HELP_NEEDY * n.max_energy { continue; }
            let dx = n.x - self.agents[a_idx].x;
            let dz = n.z - self.agents[a_idx].z;
            let d2 = dx * dx + dz * dz;
            if d2 < needy_d2 { needy_d2 = d2; needy = Some(j); }
            if n.image > effective_help_thr as i32 && d2 < willing_d2 {
                willing_d2 = d2;
                willing = Some(j);
            }
        }
        if needy.is_none() { return false; }
        let line_a = self.agents[a_idx].lineage;
        let lineage_bite = match line_a { Some("迅聚") => 1.4, Some("缓独") => 0.7, _ => 1.0 };
        if let Some(w_idx) = willing {
            // Help path: transfer this mouthful to willing neighbor
            self.agents[a_idx].help_cd = HELP_CD_MIN + self.rng.next_f64() * HELP_CD_RAND;
            let taken = self.take_grass(a_idx, ci, 0.2 * lineage_bite);
            let e = taken * 62.0;
            if let Some(ref mut w) = self.agents.get_mut(w_idx) {
                w.energy = (w.energy + e).min(w.max_energy);
            }
            self.leaf_e_h += e;
            self.help_cost_e_h += e;
            self.help_count += 1;
            let img = self.agents[a_idx].image.min(HELP_IMAGE_MAX);
            self.agents[a_idx].image = (img + 1) as i32;
            let interval = BITE_INTERVAL * (0.8 + self.rng.next_f64() * 0.4) / lineage_bite;
            self.agents[a_idx].bite = interval;
            return true;
        } else {
            // Reject path
            self.agents[a_idx].help_cd = HELP_CD_MIN + self.rng.next_f64() * HELP_CD_RAND;
            self.reject_count += 1;
            let img = self.agents[a_idx].image.max(-HELP_IMAGE_MAX);
            self.agents[a_idx].image = (img - 1) as i32;
            return false;
        }
    }

    // ----- winter aggregation pull ------------------------------------------

    fn winter_aggregation_pull(&mut self, idx: usize, dt: f64, desired: f64) {
        if self.season != 3 { return; }
        let hc = self.h_cache;
        if hc >= WIN_AGG_THRESHOLD { return; }
        let st = self.agents[idx].state;
        if st != "wander" && st != "patrol" { return; }
        if self.rng.next_f64() >= WIN_AGG_PROB { return; }
        let ki = self.agents[idx].kind;
        let ax = self.agents[idx].x;
        let az = self.agents[idx].z;
        // top-K nearest neighbors of same kind
        let mut near: Vec<(f64, f64, f64)> = Vec::new();
        for (j, b) in self.agents.iter().enumerate() {
            if j == idx { continue; }
            if !b.alive || b.kind != ki { continue; }
            let dx = b.x - ax;
            let dz = b.z - az;
            near.push((dx * dx + dz * dz, b.x, b.z));
        }
        if near.is_empty() { return; }
        near.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        near.truncate(WIN_AGG_NEIGHBORS);
        let n = near.len();
        let cx: f64 = near.iter().map(|&(_, x, _)| x).sum::<f64>() / n as f64;
        let cz: f64 = near.iter().map(|&(_, _, z)| z).sum::<f64>() / n as f64;
        let dx = cx - ax;
        let dz = cz - az;
        let dd2 = dx * dx + dz * dz;
        if dd2 < 0.01 { return; }
        let dist_c = dd2.sqrt();
        let mag = (dist_c).min(WIN_AGG_PULL * desired.max(1.0) * dt);
        let (nx, nz) = (ax + dx / dist_c * mag, az + dz / dist_c * mag);
        let wp = self.clamp_home(ax, az, self.agents[idx].home_x, self.agents[idx].home_z, self.agents[idx].range, nx, nz);
        self.agents[idx].target_x = wp.0;
        self.agents[idx].target_z = wp.1;
    }

    // ----- gut transit (internal digestion / seed dispersal) ----------------
    // Kept for bit-exactness; fruiting/deposit side effects are preserved.
    fn gut_transit(&mut self, idx: usize, dt: f64) {
        let agent_cargo = self.agents[idx].cargo.clone();
        if agent_cargo.is_empty() { return; }
        let pn = self.plants.len();
        let keep = aether_libm::exp(-GUT_DECAY * dt);
        let ci = self.cell_index(self.agents[idx].x, self.agents[idx].z);
        let ok = ci >= 0 && (ci as usize) < self.suit.len() && self.suit[ci as usize] > 0.02;
        let st = &mut self.deposit_stats;
        let mut deposited = false;
        let mut new_cargo = agent_cargo.clone();
        for s in 0..pn {
            let c = agent_cargo[s] as f64;
            if c <= 1e-12 { continue; }
            let out = c * (1.0 - keep);
            new_cargo[s] = ((agent_cargo[s] as f64) - out) as f32;
            if !ok || out <= 0.0 { continue; }
            if (self.grass_p[s][ci as usize] as f64) < 0.01 { st.colonize += 1; }
            let new_val = (self.grass_p[s][ci as usize] as f64 + out).min(1.0);
            self.grass_p[s][ci as usize] = new_val as f32;
            deposited = true;
            let t0 = self.time - 1.0 / GUT_DECAY;
            let mut sx = self.agents[idx].x;
            let mut sz = self.agents[idx].z;
            if self.agents[idx].trail_n > 0 {
                let start = ((self.agents[idx].trail_head as isize - self.agents[idx].trail_n as isize + TRAIL_SLOTS as isize) % TRAIL_SLOTS as isize) as usize;
                let mut found = false;
                for k in (0..self.agents[idx].trail_n).rev() {
                    let ti = (start + k) % TRAIL_SLOTS;
                    if self.agents[idx].trail_t[ti] <= t0 {
                        sx = self.agents[idx].trail_x[ti];
                        sz = self.agents[idx].trail_z[ti];
                        found = true;
                        break;
                    }
                }
                if !found {
                    sx = self.agents[idx].trail_x[start];
                    sz = self.agents[idx].trail_z[start];
                }
            }
            let cells = js_hypot(self.agents[idx].x - sx, self.agents[idx].z - sz) / self.cell;
            st.count += 1;
            st.dist_sum += cells;
            if cells > 5.0 { st.far_5 += 1; }
            if cells > 20.0 { st.far_20 += 1; }
            if st.count % 4096 == 0 {
                if st.samples.len() < 4000 { st.samples.push(cells); }
                else { st.samples[st.sample_cursor % 4000] = cells; st.sample_cursor += 1; }
            }
        }
        // Write back decayed cargo so gut transit properly depletes seeds across ticks (bug: previously new_cargo was discarded).
        self.agents[idx].cargo = new_cargo;
        if deposited && ci >= 0 {
            let idx_ci = ci as usize;
            let mut gsum = 0.0f64;
            for s in 0..pn { gsum += self.grass_p[s][idx_ci] as f64; }
            self.grass[idx_ci] = gsum as f32;
        }
    }

    // ----- reproduction -----------------------------------------------------

    fn pair_trait_distance(&self, a_idx: usize, b_idx: usize) -> f64 {
        let ga = &self.agents[a_idx].genome;
        let gb = &self.agents[b_idx].genome;
        let dspeed = (ga.speed - gb.speed).abs() / 6.4;
        let dsense = (ga.sense - gb.sense).abs() / 116.0;
        let dsize = (ga.size - gb.size).abs() / 1.12;
        (dspeed + dsense + dsize) / 3.0
    }

    fn sig_match(&self, ga: &Genome, gb: &Genome) -> f64 {
        if !ga.has_sig_genes() || !gb.has_sig_genes() { return 1.0; }
        let d = ((ga.size - gb.pref_sig.unwrap_or(ga.size)).abs() + (gb.size - ga.pref_sig.unwrap_or(gb.size)).abs()) / 1.12;
        let tol_avg = ((ga.tol_sig.unwrap_or(0.15) + gb.tol_sig.unwrap_or(0.15)) * 0.5) * SIG_TOL_SCALE;
        if tol_avg <= 0.0 { return if d <= 1e-9 { 1.0 } else { 0.0 }; }
        (1.0 - (d / (2.0 * tol_avg)).clamp(0.0, 1.0)).max(0.0)
    }

    fn reproduce(&mut self, idx: usize, dt: f64) {
        let (mature, cooldown) = {
            let def = self.kind_def(self.agents[idx].kind);
            (def.mature_age, def.repro_cooldown)
        };
        if self.agents[idx].repro > 0.0 || self.agents[idx].age < mature { return; }
        // ---- Phase 4-A: biome-driven density-dependent reproduction brake ----
        let a_kind = self.agents[idx].kind;
        let a_x = self.agents[idx].x;
        let a_z = self.agents[idx].z;
        let bi = self.biome_reg.find_biome(a_x, a_z);
        let (bi_count, bi_cap) = match bi {
            Some(bm) => (
                self.count(a_kind), // global count for species present in this biome
                self.biome_cap_in(a_kind, bm),
            ),
            None => (self.count(a_kind), usize::MAX),
        };
        let density_factor = if bi_cap > 0 { (bi_count as f64) / (bi_cap as f64) } else { 0.0 };
        let base_threshold = self.kind_def(a_kind).repro_base_threshold;
        let mut threshold = base_threshold * (1.0 + density_factor.max(0.0)).min(1.5);
        // FISH density brake ramps at half rate and caps at 0.72 so a healthy
        // school can still breed; the MAX_FISH spawn cap handles overpopulation.
        if a_kind == FISH {
            threshold = (base_threshold * (1.0 + density_factor * 0.5)).min(0.72);
        }
        // Existing hard switch: when H > 250 (global) boost threshold to 0.90
        if a_kind == HERB && self.h_cache > 250 { threshold = threshold.max(0.90); }
        if a_kind == PRED {
            let pred_n = self.count(PRED);
            if pred_n < 8 { threshold = 0.90; }
            else if (pred_n as f64) > (MAX_PRED as f64) * 0.35 { threshold = 1.0; }
        }
        // FISH does not reproduce in winter
        if a_kind == FISH && self.season == 3 { return; }
        // Phase 7: DECO individualized reproduction threshold in selective mode.
        // eff=1.5 → 0.48 (easy breed), eff=1.0 → 0.96 (neutral, legacy), eff=0.5 → 1.44 (effectively sterile).
        // The density brake above is replaced for DECO by this mapping: since eff=1.0 yields
        // exactly the legacy 0.96, the density factor is reapplied multiplicatively here.
        if a_kind == DECO && self.deco_eff_choice_mode == save_state::DECO_CHOICE_SELECTIVE {
            threshold = DECO_REPRO_BASE * (2.0 - self.agents[idx].genome.deco_efficiency)
                * (1.0 + density_factor.max(0.0)).min(1.5);
        }
        if self.agents[idx].energy < self.agents[idx].max_energy * threshold { return; }
        // Find mate via nearest. FISH pair over a wider radius: they are
        // spread along the narrow shoreline band, so nearest-neighbor distance
        // is much larger than for land species.
        let mate_id = self.agents[idx].id; let mate_max_e = self.agents[idx].max_energy; let mate_kind = self.agents[idx].kind;
        let mate_radius = if mate_kind == FISH { 60.0 } else { 11.0 };
        let mate_min_e_const = if mate_kind == FISH { 0.55 } else { 0.68 };
        let selected_mode = self.deco_eff_choice_mode;
        let mate_filter = Some(move |b: &Agent| {
            if b.id != mate_id && b.age > 30.0 && b.repro <= 0.0 {
                // Phase 7: DECO in selective mode requires both parties to meet their individualized threshold.
                let min_e = if mate_kind == DECO && selected_mode == save_state::DECO_CHOICE_SELECTIVE && b.kind == DECO {
                    DECO_REPRO_BASE * (2.0 - b.genome.deco_efficiency)
                } else {
                    mate_min_e_const
                };
                b.energy > b.max_energy * min_e
            } else {
                false
            }
        });
            let mate_opt = self.nearest(idx, mate_kind, mate_radius, mate_filter);
        if let Some((mate_j, _)) = mate_opt {
            let b_energy = self.agents[mate_j].energy;
            let b_max_e = self.agents[mate_j].max_energy;
            let b_age = self.agents[mate_j].age;
            // Hue compatibility
            let hue_diff = (self.agents[idx].genome.hue - self.agents[mate_j].genome.hue).abs();
            if hue_diff > 0.35 {
                let roll = if mate_kind == FISH { self.fish_rng.next_f64() } else { self.rng.next_f64() };
                if roll < 0.5 { return; }
            }
            // Pair compatible
            let a_is_iso = self.agents[idx].lineage.is_some() && self.isolated_groups.contains_key(self.agents[idx].lineage.unwrap());
            let b_is_iso = self.agents[mate_j].lineage.is_some() && self.isolated_groups.contains_key(self.agents[mate_j].lineage.unwrap());
            let lineage_a = self.agents[idx].lineage;
            let lineage_b = self.agents[mate_j].lineage;
            if self.gene_barrier_strength > 0.0 && (a_is_iso || b_is_iso) && lineage_a != lineage_b {
                self.barrier_blocks += 1;
                return;
            }
            let mut dist = self.pair_trait_distance(idx, mate_j);
            if let (Some(la), Some(lb)) = (lineage_a, lineage_b) {
                if la != lb { dist += 0.22; }
            }
            let barrier_roll = if a_kind == FISH { self.fish_rng.next_f64() } else { self.rng.next_f64() };
            if barrier_roll >= (1.0 - dist * self.gene_barrier_strength).clamp(0.0, 1.0) { return; }
            if SIG_ENABLED && self.agents[idx].kind == HERB && self.agents[mate_j].kind == HERB
                && self.agents[idx].genome.has_sig_genes() && self.agents[mate_j].genome.has_sig_genes() {
                self.sig_attempts += 1;
                if self.rng.next_f64() >= self.sig_match(&self.agents[idx].genome, &self.agents[mate_j].genome) {
                    self.sig_blocks += 1;
                    return;
                }
            }
            let a_geno = self.agents[idx].genome.clone();
            let b_geno = self.agents[mate_j].genome.clone();
            let a_kind = self.agents[idx].kind;
            let a_x = self.agents[idx].x;
            let a_z = self.agents[idx].z;
            let b_x = self.agents[mate_j].x;
            let b_z = self.agents[mate_j].z;
            let a_gen = self.agents[idx].generation;
            let b_gen = self.agents[mate_j].generation;
            let a_lineage = self.agents[idx].lineage;
            let b_lineage = self.agents[mate_j].lineage;
            let g = self.mix_genome(&a_geno, &b_geno, a_kind);
            let (jx, jz) = if a_kind == FISH {
                ((self.fish_rng.next_f64() - 0.5) * 2.5, (self.fish_rng.next_f64() - 0.5) * 2.5)
            } else {
                ((self.rng.next_f64() - 0.5) * 2.5, (self.rng.next_f64() - 0.5) * 2.5)
            };
            let child_x = (a_x + b_x) * 0.5 + jx;
            let child_z = (a_z + b_z) * 0.5 + jz;
            let max_gen_child = a_gen.max(b_gen) + 1;
            let parent_clade = self.agents[idx].clade_id;
            if let Some(child) = self.spawn(a_kind, child_x, child_z, Some(g), max_gen_child, 0.0) {
                child.lineage = a_lineage.or(b_lineage);
                child.clade_id = parent_clade;
                let (drain_herb, drain_pred) = self.kind_def(self.agents[idx].kind).repro_drain_pair;
                self.agents[idx].energy -= drain_herb;
                self.agents[mate_j].energy -= drain_pred;
                if HELP_ENABLED && self.agents[idx].kind == HERB {
                    self.drain_e_h += 68.0;
                }
                self.agents[idx].repro = cooldown;
                self.agents[mate_j].repro = cooldown;
                self.agents[idx].offspring = self.agents[idx].offspring + 1;
                self.agents[mate_j].offspring = self.agents[mate_j].offspring + 1;
                self.births += 1;
                return;
            }
        }
        // Asexual reproduction
        if self.agents[idx].age > 45.0 && self.agents[idx].energy > self.agents[idx].max_energy * 0.94 {
            let a_geno_a2 = self.agents[idx].genome.clone();
            let a_kind_a2 = self.agents[idx].kind;
            let a_gen_a2 = self.agents[idx].generation;
            let a_lineage_a2 = self.agents[idx].lineage;
            let g = self.mutate(&a_geno_a2, a_kind_a2);
            let ang = if a_kind_a2 == FISH { self.fish_rng.next_f64() } else { self.rng.next_f64() } * std::f64::consts::PI * 2.0;
            let cx = self.agents[idx].x + aether_libm::cos(ang) * 3.0;
            let cz = self.agents[idx].z + aether_libm::sin(ang) * 3.0;
            let parent_clade = self.agents[idx].clade_id;
            if let Some(child) = self.spawn(a_kind_a2, cx, cz, Some(g), a_gen_a2 + 1, 0.0) {
                child.lineage = a_lineage_a2;
                child.clade_id = parent_clade;
                let drain = self.kind_def(self.agents[idx].kind).repro_drain_solo;
                self.agents[idx].energy -= drain;
                if HELP_ENABLED && self.agents[idx].kind == HERB {
                    self.drain_e_h += 52.0;
                }
                self.agents[idx].repro = cooldown * 1.4;
                self.agents[idx].offspring = self.agents[idx].offspring + 1;
                self.births += 1;
            }
        }
    }

    pub(crate) fn die(&mut self, idx: usize, cause: &'static str) {
        if !self.agents[idx].alive { return; }
        self.agents[idx].alive = false;
        self.deaths += 1;
        if cause == "starved" { self.starved += 1; }
        else if cause == "aged" { self.aged += 1; }
        let offspring = self.agents[idx].offspring;
        let lifetime_fitness = (self.agents[idx].age / self.agents[idx].genome.lifespan) * (1.0 + offspring as f64) / self.agents[idx].genome.metabolism;
        let kind_idx = self.agents[idx].kind as usize;
        self.generation_fitness[kind_idx].push(FitnessRecord { gen: self.agents[idx].generation, age: self.agents[idx].age, offspring, fitness: lifetime_fitness });
        let energy = 4.0 + self.agents[idx].genome.size * 9.0 + self.agents[idx].energy.max(0.0) * 0.16;
        // Phase 8: release any carcass lock held by the dying agent.
        if self.carcass_lock_enabled {
            let dead_id = self.agents[idx].id;
            for c in self.carcasses.iter_mut() { if c.occupied_by == Some(dead_id) { c.occupied_by = None; } }
        }
        self.carcasses.push(Carcass { x: self.agents[idx].x, z: self.agents[idx].z, energy, age: 0.0, occupied_by: None });
    }

    // ----- herbivore behaviour ---------------------------------------------

    fn update_herb(&mut self, idx: usize, h: f64) {
        let kind = self.agents[idx].kind;
        // Phase 33: diet gate — HERB feeds on plants (eats_plant = true). Read
        // from the registry so a future registered herb-kind with a different
        // diet behaves correctly; for the legacy six it is a guaranteed-true
        // no-op (bit-exact).
        let herb_eats_plant = self.diet_of(kind).eats_plant;
        self.agents[idx].age += h;
        let g_speed = self.agents[idx].genome.speed;
        let g_sense = self.agents[idx].genome.sense;
        let g_metabolism = self.agents[idx].genome.metabolism;
        let g_bold = self.agents[idx].genome.bold;
        let mut g_size = self.agents[idx].genome.size;
        self.agents[idx].bite -= h;
        self.agents[idx].repro -= h;
        self.agents[idx].think -= h;
        if HELP_ENABLED {
            self.agents[idx].help_cd = (self.agents[idx].help_cd - h).max(0.0);
        }
        let y_val = self.agents[idx].y;
        self.agents[idx].hydration = (self.agents[idx].hydration - 0.0025 * h + if y_val < WATER_LEVEL + 1.5 { 0.10 * h } else { 0.0 }).min(1.0);

        let lineage_a = self.agents[idx].lineage;
        let lineage_drain = match lineage_a { Some("迅聚") => 1.15, Some("缓独") => 0.85, _ => 1.0 };
        let lineage_bite = match lineage_a { Some("迅聚") => 1.4, Some("缓独") => 0.7, _ => 1.0 };

        self.agents[idx].fatigue = (self.agents[idx].fatigue + self.agents[idx].activity * 0.06 * h
            - if self.agents[idx].activity < 0.15 { 0.22 * h } else { 0.0 }).clamp(0.0, 1.0);
        self.agents[idx].curiosity = (self.agents[idx].curiosity + 0.02 * h).clamp(0.0, 1.0);

        let threat_opt: Option<(usize, f64)> = self.nearest(idx, PRED, g_sense * 0.95, None::<fn(&Agent) -> bool>);
        let resting = self.daylight < 0.12 && threat_opt.is_none();
        let mut desired = 0.4 + g_speed * 0.18;
        if let Some((_tj, _td)) = threat_opt {
            self.agents[idx].fear = (self.agents[idx].fear + (1.0 - g_bold * 0.6) * 1.5 * h).clamp(0.0, 1.0);
        } else {
            self.agents[idx].fear = (self.agents[idx].fear - 0.5 * h).clamp(0.0, 1.0);
        }

        // Player alert — playerX/playerZ are always None in headless mode, skip
        // Fire alert — fires vec is always empty in headless mode, skip
        // Torch — none in headless mode, skip

        if let Some((threat_j, _)) = threat_opt {
            self.agents[idx].state = "flee";
            let away = aether_libm::atan2(self.agents[idx].x - self.agents[threat_j].x, self.agents[idx].z - self.agents[threat_j].z);
            self.agents[idx].heading = away;
            desired = g_speed * (1.32 - g_bold * 0.25);
            let drain_flee = (0.16 * h * g_metabolism).max(0.0);
            self.agents[idx].energy -= drain_flee;
            if HELP_ENABLED { self.drain_e_h += drain_flee; }
        } else if self.agents[idx].think <= 0.0 {
            self.agents[idx].think = 0.35 + self.rng.next_f64() * 0.4;
            let social_high = self.agents[idx].genome.social > 0.8;
            if social_high {
                let my_id = self.agents[idx].id;
                let mate_opt = self.nearest(idx, HERB, 15.0, Some(|b: &Agent| b.id != my_id));
                if let Some((mate_j, _)) = mate_opt {
                    self.agents[idx].target_x = self.agents[mate_j].x;
                    self.agents[idx].target_z = self.agents[mate_j].z;
                    self.agents[idx].state = "graze";
                    let dm = js_hypot(self.agents[mate_j].x - self.agents[idx].x, self.agents[mate_j].z - self.agents[idx].z);
                    desired = if dm > 3.0 { g_speed * 0.6 } else { 0.0 };
                } else if self.agents[idx].energy < self.agents[idx].max_energy * 0.84 {
                    let (px, pz, pc, _) = self.find_grass(idx);
                    if pc >= 0 {
                        self.agents[idx].target_x = px;
                        self.agents[idx].target_z = pz;
                        self.agents[idx].state = "graze";
                        let d = js_hypot(px - self.agents[idx].x, pz - self.agents[idx].z);
                        desired = if d > 2.0 { g_speed * (0.55 + (1.0 - self.agents[idx].energy / self.agents[idx].max_energy) * 0.45) } else { 0.0 };
                    } else {
                        self.agents[idx].state = "wander";
                        let wa = self.rng.next_f64() * std::f64::consts::PI * 2.0;
                        let wr = if resting { 14.0 } else { 35.0 };
                        let wp = self.clamp_home(self.agents[idx].x, self.agents[idx].z,
                            self.agents[idx].home_x, self.agents[idx].home_z,
                            self.agents[idx].range,
                            self.agents[idx].x + aether_libm::cos(wa) * wr,
                            self.agents[idx].z + aether_libm::sin(wa) * wr);
                        self.agents[idx].target_x = wp.0;
                        self.agents[idx].target_z = wp.1;
                        desired = g_speed * 0.55;
                    }
                }
            } else if self.agents[idx].energy < self.agents[idx].max_energy * 0.84 {
                let (px, pz, pc, _) = self.find_grass(idx);
                if pc >= 0 {
                    self.agents[idx].target_x = px;
                    self.agents[idx].target_z = pz;
                    self.agents[idx].state = "graze";
                    let d = js_hypot(px - self.agents[idx].x, pz - self.agents[idx].z);
                    desired = if d > 2.0 { g_speed * (0.55 + (1.0 - self.agents[idx].energy / self.agents[idx].max_energy) * 0.45) } else { 0.0 };
                } else {
                    self.agents[idx].state = "wander";
                    let wa = self.rng.next_f64() * std::f64::consts::PI * 2.0;
                    let wr = if resting { 14.0 } else { 35.0 };
                    let wp = self.clamp_home(self.agents[idx].x, self.agents[idx].z,
                        self.agents[idx].home_x, self.agents[idx].home_z,
                        self.agents[idx].range,
                        self.agents[idx].x + aether_libm::cos(wa) * wr,
                        self.agents[idx].z + aether_libm::sin(wa) * wr);
                    self.agents[idx].target_x = wp.0;
                    self.agents[idx].target_z = wp.1;
                    desired = g_speed * 0.55;
                }
            } else {
                self.agents[idx].state = "wander";
                let wx = self.agents[idx].x + (self.rng.next_f64() - 0.5) * (if resting { 18.0 } else { 45.0 });
                let wz = self.agents[idx].z + (self.rng.next_f64() - 0.5) * (if resting { 18.0 } else { 45.0 });
                let wp = self.clamp_home(self.agents[idx].x, self.agents[idx].z,
                    self.agents[idx].home_x, self.agents[idx].home_z,
                    self.agents[idx].range, wx, wz);
                self.agents[idx].target_x = wp.0;
                self.agents[idx].target_z = wp.1;
                desired = g_speed * (if resting { 0.12 } else { 0.3 });
            }
        }
        self.winter_aggregation_pull(idx, h, desired);

        let a_state = self.agents[idx].state;
        if a_state != "flee" {
            let d = js_hypot(self.agents[idx].target_x - self.agents[idx].x, self.agents[idx].target_z - self.agents[idx].z);
            if d < 1.6 {
                self.agents[idx].state = "graze";
                desired = 0.0;
            } else {
                {
                    let tx = self.agents[idx].target_x;
                    let tz = self.agents[idx].target_z;
                    let dx = tx - self.agents[idx].x;
                    let dz = tz - self.agents[idx].z;
                    if dx * dx + dz * dz > 0.01 {
                        let want = aether_libm::atan2(dx, dz);
                        let mut diff = want - self.agents[idx].heading;
                        while diff > std::f64::consts::PI { diff -= std::f64::consts::PI * 2.0; }
                        while diff < -std::f64::consts::PI { diff += std::f64::consts::PI * 2.0; }
                        let turn = diff.clamp(-2.8 * h, 2.8 * h);
                        self.agents[idx].heading += turn;
                    }
                    self.agents[idx].desired_speed = desired;
                }
            }
        }

        self.agents[idx].speed = self.agents[idx].speed + (self.agents[idx].desired_speed - self.agents[idx].speed) * (h * 5.0).min(1.0);
        let activity = self.agents[idx].speed / g_speed.max(1.0);
        self.agents[idx].activity = activity;

        let epistasis = 1.0 + (g_speed / 8.0) * (g_size - 1.0) * 0.15 + (g_sense / 100.0) * (g_metabolism - 1.0) * 0.1;
        let mut drain = g_metabolism * (0.34 + 0.62 * activity * activity + 0.12 * (g_speed / 8.0).powi(2) + 0.08 * (g_sense / 130.0)) * epistasis;
        // Phase 32: ext-locus expression upkeep — an ECONOMIC cap (not a
        // rule cap). Each ext locus multiplies the shared crowding/epistasis
        // drain by (1 + LOCI_UPKEEP): 0 loci ⇒ ×1.0 (bit-exact no-op), and
        // as a genome's ext grows its metabolic cost rises, holding `ext`
        // below an upper bound without a hard size limit.
        drain *= 1.0 + (self.agents[idx].genome.ext.len() as f64) * LOCI_UPKEEP;
        let plasticity = 1.0 + if self.daylight < 0.3 { -0.06 } else { 0.0 }
            + if self.agents[idx].hydration < 0.4 { 0.04 } else { 0.0 };
        let cap_for_kind = self.kind_def(kind).global_cap as f64;
        let local_crowd = self.count(kind) as f64 / cap_for_kind;
        // Actually MAX_HERB/MAX_PRED etc are u32 consts; casting via match
        let crowd_penalty = if local_crowd > 0.4 { 1.0 + (local_crowd - 0.4) * 0.5 } else { 1.0 };
        let thirst_h = if self.agents[idx].hydration < 0.3 { 1.5 } else { 1.0 };
        let total_drain = drain * thirst_h * crowd_penalty * plasticity * (self.season_meta).max(0.0) * lineage_drain * h;
        if HELP_ENABLED { self.drain_e_h += total_drain; }
        self.agents[idx].energy -= total_drain;

        let ci_val = self.cell_index(self.agents[idx].x, self.agents[idx].z);
        if herb_eats_plant
            && ci_val >= 0 && (ci_val as usize) < self.grass.len() && (self.grass[ci_val as usize] as f64) > 0.1
            && self.agents[idx].bite <= 0.0 && self.agents[idx].speed < g_speed * 0.45 {
            let helped = self.try_help(idx, ci_val as usize);
            if !helped {
                let take = self.take_grass(idx, ci_val as usize, 0.2 * lineage_bite);
                        self.leaf_e_h += take * 62.0;
                self.agents[idx].energy = (self.agents[idx].energy + take * 62.0).min(self.agents[idx].max_energy);
                self.agents[idx].hydration = (self.agents[idx].hydration + 0.18).min(1.0);
                let interval = BITE_INTERVAL * (0.8 + self.rng.next_f64() * 0.4) / lineage_bite;
                self.agents[idx].bite = interval;
                if (self.grass[ci_val as usize] as f64) < 0.03 && self.rng.next_f64() < 0.1 {
                    self.agents[idx].think = 0.0;
                }
            }
        }

        let nx = self.agents[idx].x + aether_libm::sin(self.agents[idx].heading) * self.agents[idx].speed * h;
        let nz = self.agents[idx].z + aether_libm::cos(self.agents[idx].heading) * self.agents[idx].speed * h;
        let (mx, mz) = Self::avoid(nx, nz, self.agents[idx].x, self.agents[idx].z, &mut self.agents[idx].heading);
        self.agents[idx].x = mx;
        self.agents[idx].z = mz;
        self.agents[idx].y = terrain_height(mx, mz);

        if self.agents[idx].energy <= 0.0 {
            self.die(idx, "starved");
        } else if self.agents[idx].age > self.agents[idx].genome.lifespan {
            self.die(idx, "aged");
        } else if self.agents[idx].repro <= 0.0 && self.agents[idx].energy > self.agents[idx].max_energy * 0.72 {
            self.reproduce(idx, h);
        }
    }

    // ----- predator behaviour -----------------------------------------------

    fn update_pred(&mut self, idx: usize, h: f64) {
        let p_kind = self.agents[idx].kind;
        // Phase 33: diet gates — PRED hunts live prey (prey_kinds = [HERB,
        // FISH], the legacy two-nearest scan order) and scavenges carcasses.
        // Both read from the registry; for the legacy PRED they are true
        // (bit-exact), and the prey scan itself stays on the legacy path this
        // phase (a data-driven matrix scan is a Phase 34 re-baseline).
        let pred_has_prey = !self.diet_of(p_kind).prey_kinds.is_empty();
        let pred_eats_carcass = self.diet_of(p_kind).eats_carcass;
        self.agents[idx].age += h;
        let g_speed = self.agents[idx].genome.speed;
        let g_sense = self.agents[idx].genome.sense;
        let g_metabolism = self.agents[idx].genome.metabolism;
        let g_size = self.agents[idx].genome.size;
        self.agents[idx].bite -= h;
        self.agents[idx].repro -= h;
        self.agents[idx].think -= h;
        let y_val = self.agents[idx].y;
        self.agents[idx].hydration = (self.agents[idx].hydration - 0.002 * h + if y_val < WATER_LEVEL + 1.5 { 0.10 * h } else { 0.0 }).min(1.0);
        self.agents[idx].fatigue = (self.agents[idx].fatigue + self.agents[idx].activity * 0.05 * h
            - if self.agents[idx].activity < 0.2 { 0.2 * h } else { 0.0 }).clamp(0.0, 1.0);
        self.agents[idx].curiosity = (self.agents[idx].curiosity + 0.015 * h).clamp(0.0, 1.0);
        self.agents[idx].fear = (self.agents[idx].fear - 0.3 * h).clamp(0.0, 1.0);
        let mut desired = 0.3 + g_speed * 0.16;
        let hungry = self.agents[idx].energy < self.agents[idx].max_energy * 0.9;

        if self.agents[idx].think <= 0.0 || self.agents[idx].target_id == 0 {
            self.agents[idx].think = 0.22 + self.rng.next_f64() * 0.28;
            // Phase 4-B: dual-predator. P hunts HERB on land and FISH in water.
            // We pick the nearest prey of either kind, but fish in water are
            // reachable only from shallow water (predator stays on land edge).
            let mut prey_opt: Option<(usize, f64)> = None;
            if hungry && pred_has_prey {
                let my_x = self.agents[idx].x;
                let my_z = self.agents[idx].z;
                let my_ground = terrain_height(my_x, my_z);
                let h_opt = self.nearest(idx, HERB, g_sense, None::<fn(&Agent) -> bool>);
                let f_opt = self.nearest(idx, FISH, g_sense, None::<fn(&Agent) -> bool>);
                // Choose prey: when near shore (my_ground < WATER_LEVEL + 3.0)
                // the predator is a fish specialist — fish are slower to flee
                // (no fear response) and concentrated, so they are preferred
                // whenever reachable. Inland predators hunt herb only.
                let fish_reachable = my_ground < WATER_LEVEL + 3.0;
                match (h_opt, f_opt) {
                    (Some((hj, hd)), Some((fj, _fd))) => {
                        if fish_reachable {
                            prey_opt = Some((fj, _fd));
                        } else {
                            prey_opt = Some((hj, hd));
                        }
                    }
                    (Some(x), None) => prey_opt = Some(x),
                    (None, Some(x)) => { if fish_reachable { prey_opt = Some(x); } else { prey_opt = None; } }
                    (None, None) => prey_opt = None,
                }
            }
            let mut best_carcass: Option<usize> = None;
            if pred_eats_carcass {
                let mut bd = (g_sense * 1.2).powi(2);
                let ax = self.agents[idx].x;
                let az = self.agents[idx].z;
                for (j, c) in self.carcasses.iter().enumerate() {
                    let dx = c.x - ax;
                    let dz = c.z - az;
                    let d2 = dx * dx + dz * dz;
                    if d2 < bd { bd = d2; best_carcass = Some(j); }
                }
            }
            if let Some(bc) = best_carcass {
                self.agents[idx].target_x = self.carcasses[bc].x;
                self.agents[idx].target_z = self.carcasses[bc].z;
                self.agents[idx].state = "scavenge";
                self.agents[idx].target_id = 0;
            } else if let Some((prey_j, _)) = prey_opt {
                self.agents[idx].target_id = self.agents[prey_j].id;
                self.agents[idx].target_x = self.agents[prey_j].x;
                self.agents[idx].target_z = self.agents[prey_j].z;
                self.agents[idx].state = "hunt";
            } else {
                self.agents[idx].target_id = 0;
                let wx = self.agents[idx].x + (self.rng.next_f64() - 0.5) * 95.0;
                let wz = self.agents[idx].z + (self.rng.next_f64() - 0.5) * 95.0;
                let wp = self.clamp_home(self.agents[idx].x, self.agents[idx].z,
                    self.agents[idx].home_x, self.agents[idx].home_z,
                    self.agents[idx].range, wx, wz);
                self.agents[idx].target_x = wp.0;
                self.agents[idx].target_z = wp.1;
                self.agents[idx].state = "patrol";
            }
        }

        // Track prey by targetId
        let mut prey_alive_idx: Option<usize> = None;
        if self.agents[idx].target_id != 0 {
            let tid = self.agents[idx].target_id;
            for (j, b) in self.agents.iter().enumerate() {
                if b.id == tid && b.alive {
                    prey_alive_idx = Some(j);
                    break;
                }
            }
            if let Some(pj) = prey_alive_idx {
                self.agents[idx].target_x = self.agents[pj].x;
                self.agents[idx].target_z = self.agents[pj].z;
                self.agents[idx].state = "hunt";
            } else {
                self.agents[idx].target_id = 0;
                self.agents[idx].think = 0.0;
            }
        }

        if self.agents[idx].state == "hunt" || self.agents[idx].state == "scavenge" {
            desired = g_speed * 1.1;
        } else {
            desired = g_speed * 0.32;
        }

        // player / fire / torch avoidance — always empty in headless mode

        {
                    let tx = self.agents[idx].target_x;
                    let tz = self.agents[idx].target_z;
                    let dx = tx - self.agents[idx].x;
                    let dz = tz - self.agents[idx].z;
                    if dx * dx + dz * dz > 0.01 {
                        let want = aether_libm::atan2(dx, dz);
                        let mut diff = want - self.agents[idx].heading;
                        while diff > std::f64::consts::PI { diff -= std::f64::consts::PI * 2.0; }
                        while diff < -std::f64::consts::PI { diff += std::f64::consts::PI * 2.0; }
                        let turn = diff.clamp(-2.8 * h, 2.8 * h);
                        self.agents[idx].heading += turn;
                    }
                    self.agents[idx].desired_speed = desired;
                }
        self.agents[idx].speed = self.agents[idx].speed + (self.agents[idx].desired_speed - self.agents[idx].speed) * (h * 5.0).min(1.0);
        let activity = self.agents[idx].speed / g_speed.max(1.0);
        self.agents[idx].activity = activity;
        let drain = g_metabolism * (0.46 + 0.78 * activity * activity + 0.14 * (g_speed / 10.5).powi(2));
        let thirst_p = if self.agents[idx].hydration < 0.3 { 1.5 } else { 1.0 };
        self.agents[idx].energy -= drain * thirst_p * self.season_meta * h;

        let nx = self.agents[idx].x + aether_libm::sin(self.agents[idx].heading) * self.agents[idx].speed * h;
        let nz = self.agents[idx].z + aether_libm::cos(self.agents[idx].heading) * self.agents[idx].speed * h;
        let (mx, mz) = Self::avoid(nx, nz, self.agents[idx].x, self.agents[idx].z, &mut self.agents[idx].heading);
        self.agents[idx].x = mx;
        self.agents[idx].z = mz;
        self.agents[idx].y = terrain_height(mx, mz);

        // Bite prey if close enough
        if let Some(pj) = prey_alive_idx {
            if self.agents[idx].bite <= 0.0 {
                let prey_x = self.agents[pj].x;
                let prey_z = self.agents[pj].z;
                let prey_energy = self.agents[pj].energy;
                let prey_size = self.agents[pj].genome.size;
                let d = js_hypot(prey_x - self.agents[idx].x, prey_z - self.agents[idx].z);
                let reach = 1.9 + g_size * 0.8;
                if d < reach && hungry {
                    let gain = 18.0 + prey_energy * 0.22 + prey_size * 11.0;
                    self.agents[idx].energy = (self.agents[idx].energy + gain * 0.8).min(self.agents[idx].max_energy);
                    self.agents[idx].hydration = (self.agents[idx].hydration + 0.4).min(1.0);
                    let bite_interval = PRED_BITE_INTERVAL + self.rng.next_f64() * 5.0;
                    self.agents[idx].bite = bite_interval;
                    self.kills += 1;
                    self.deaths += 1;
                    self.agents[pj].alive = false;
                    self.carcasses.push(Carcass { x: prey_x, z: prey_z, energy: gain * 0.22, age: 0.0, occupied_by: None });
                    if self.carcasses.len() > 60 { self.carcasses.remove(0); }
                    self.agents[idx].target_id = 0;
                    self.agents[idx].think = 0.0;
                    self.agents[idx].state = "feed";
                }
            }
        }

        // Scavenge carcass
        if self.agents[idx].state == "scavenge" && self.agents[idx].bite <= 0.0 {
            for i in (0..self.carcasses.len()).rev() {
                let c_x = self.carcasses[i].x;
                let c_z = self.carcasses[i].z;
                let c_energy = self.carcasses[i].energy;
                if (c_x - self.agents[idx].x).hypot(c_z - self.agents[idx].z) < 2.2 {
                    self.agents[idx].energy = (self.agents[idx].energy + c_energy).min(self.agents[idx].max_energy);
                    self.agents[idx].hydration = (self.agents[idx].hydration + 0.15).min(1.0);
                    self.carcasses.remove(i);
                    self.agents[idx].bite = 1.0;
                    self.agents[idx].state = "feed";
                    self.agents[idx].target_id = 0;
                    self.agents[idx].think = 0.0;
                    break;
                }
            }
        }

        if self.agents[idx].energy <= 0.0 {
            self.die(idx, "starved");
        } else if self.agents[idx].age > self.agents[idx].genome.lifespan {
            self.die(idx, "aged");
        } else if self.agents[idx].repro <= 0.0 && self.agents[idx].energy > self.agents[idx].max_energy * 0.85 {
            self.reproduce(idx, h);
        }
    }

    // ----- scavenger / omnivore behaviour ------------------------------------

    fn update_scav(&mut self, idx: usize, h: f64) {
        let s_kind = self.agents[idx].kind;
        // Phase 33: diet gates — SCAV scavenges carcasses and grazes grass
        // (eats_carcass / eats_plant both true for the legacy kind, bit-exact;
        // read from the registry so a registered variant can drop a source).
        let scav_eats_carcass = self.diet_of(s_kind).eats_carcass;
        self.agents[idx].age += h;
        let g_speed = self.agents[idx].genome.speed;
        let g_sense = self.agents[idx].genome.sense;
        let g_metabolism = self.agents[idx].genome.metabolism;
        self.agents[idx].bite -= h;
        self.agents[idx].repro -= h;
        self.agents[idx].think -= h;
        self.agents[idx].fear = (self.agents[idx].fear - 0.5 * h).clamp(0.0, 1.0);
        if self.agents[idx].think <= 0.0 {
            self.agents[idx].think = 0.3 + self.rng.next_f64() * 0.3;
            let mut best: Option<usize> = None;
            let mut bd = (g_sense * 1.2).powi(2);
            let ax = self.agents[idx].x;
            let az = self.agents[idx].z;
            for (j, c) in self.carcasses.iter().enumerate() {
                let dx = c.x - ax;
                let dz = c.z - az;
                let d2 = dx * dx + dz * dz;
                if d2 < bd { bd = d2; best = Some(j); }
            }
            if let Some(b) = best {
                self.agents[idx].target_x = self.carcasses[b].x;
                self.agents[idx].target_z = self.carcasses[b].z;
                self.agents[idx].state = "scavenge";
            } else {
                self.agents[idx].state = "wander";
                let wa = self.rng.next_f64() * std::f64::consts::PI * 2.0;
                let wp = self.clamp_home(self.agents[idx].x, self.agents[idx].z,
                    self.agents[idx].home_x, self.agents[idx].home_z,
                    self.agents[idx].range,
                    self.agents[idx].x + aether_libm::cos(wa) * 30.0,
                    self.agents[idx].z + aether_libm::sin(wa) * 30.0);
                self.agents[idx].target_x = wp.0;
                self.agents[idx].target_z = wp.1;
            }
        }
        if self.agents[idx].state != "flee" {
            let d = js_hypot(self.agents[idx].target_x - self.agents[idx].x, self.agents[idx].target_z - self.agents[idx].z);
            if d < 1.6 {
                self.agents[idx].desired_speed = 0.0;
            } else {
                {
                    let tx = self.agents[idx].target_x;
                    let tz = self.agents[idx].target_z;
                    let dx = tx - self.agents[idx].x;
                    let dz = tz - self.agents[idx].z;
                    if dx * dx + dz * dz > 0.01 {
                        let want = aether_libm::atan2(dx, dz);
                        let mut diff = want - self.agents[idx].heading;
                        while diff > std::f64::consts::PI { diff -= std::f64::consts::PI * 2.0; }
                        while diff < -std::f64::consts::PI { diff += std::f64::consts::PI * 2.0; }
                        let turn = diff.clamp(-2.8 * h, 2.8 * h);
                        self.agents[idx].heading += turn;
                    }
                    self.agents[idx].desired_speed = g_speed * 0.55;
                }
            }
        }
        self.agents[idx].speed = self.agents[idx].speed + (self.agents[idx].desired_speed - self.agents[idx].speed) * (h * 5.0).min(1.0);
        let activity = self.agents[idx].speed / g_speed.max(1.0);
        self.agents[idx].activity = activity;
        self.agents[idx].x += aether_libm::sin(self.agents[idx].heading) * self.agents[idx].speed * h;
        self.agents[idx].z += aether_libm::cos(self.agents[idx].heading) * self.agents[idx].speed * h;
        self.agents[idx].y = terrain_height(self.agents[idx].x, self.agents[idx].z);

        // Scavenge carcass if close (Phase 8: skip carcasses locked by another agent)
        let self_id = self.agents[idx].id;
        let ci = if scav_eats_carcass {
            self.carcasses.iter().position(|c|
                js_hypot(c.x - self.agents[idx].x, c.z - self.agents[idx].z) < 1.8
                && c.energy > 0.2
                && self.carcass_claimable(c, self_id))
        } else { None };
        if let Some(ci) = ci {
            if self.agents[idx].bite <= 0.0 {
                let take = 4.0f64.min(self.carcasses[ci].energy);
                self.carcasses[ci].energy -= take;
                self.agents[idx].energy = (self.agents[idx].energy + take * 7.0).min(self.agents[idx].max_energy);
                self.agents[idx].bite = 1.5;
                if self.carcasses[ci].energy <= 0.2 { self.carcasses.remove(ci); }
            } else if self.agents[idx].energy < self.agents[idx].max_energy * 0.7 && self.agents[idx].bite <= 0.0 {
                let gi = self.cell_index(self.agents[idx].x, self.agents[idx].z);
                if gi >= 0 && (gi as usize) < self.grass.len() && (self.grass[gi as usize] as f64) > 0.1 {
                    let take = self.take_grass(idx, gi as usize, 0.15);
                    self.agents[idx].energy = (self.agents[idx].energy + take * 55.0).min(self.agents[idx].max_energy);
                    self.agents[idx].bite = 2.0;
                }
            }
        } else if self.agents[idx].energy < self.agents[idx].max_energy * 0.7 && self.agents[idx].bite <= 0.0 {
            let gi = self.cell_index(self.agents[idx].x, self.agents[idx].z);
            if gi >= 0 && (gi as usize) < self.grass.len() && (self.grass[gi as usize] as f64) > 0.1 {
                let take = self.take_grass(idx, gi as usize, 0.15);
                self.agents[idx].energy = (self.agents[idx].energy + take * 55.0).min(self.agents[idx].max_energy);
                self.agents[idx].bite = 2.0;
            }
        }
        self.agents[idx].energy -= g_metabolism * (0.18 + self.agents[idx].activity * 0.3) * self.season_meta * h;
        if self.agents[idx].energy <= 0.0 { self.die(idx, "starved"); }
        else if self.agents[idx].age > self.agents[idx].genome.lifespan { self.die(idx, "aged"); }
        else if self.agents[idx].repro <= 0.0 && self.agents[idx].energy > self.agents[idx].max_energy * 0.78 { self.reproduce(idx, h); }
    }

    fn update_omn(&mut self, idx: usize, h: f64) {
        let o_kind = self.agents[idx].kind;
        // Phase 33: diet gate — OMN scavenges carcasses (eats_carcass = true
        // for the legacy kind, bit-exact; a registered variant can drop it).
        let omn_eats_carcass = self.diet_of(o_kind).eats_carcass;
        self.agents[idx].age += h;
        let g_speed = self.agents[idx].genome.speed;
        let g_sense = self.agents[idx].genome.sense;
        let g_metabolism = self.agents[idx].genome.metabolism;
        self.agents[idx].bite -= h;
        self.agents[idx].repro -= h;
        self.agents[idx].think -= h;
        self.agents[idx].fear = (self.agents[idx].fear - 0.5 * h).clamp(0.0, 1.0);
        let threat: Option<(usize, f64)> = self.nearest(idx, PRED, g_sense * 0.9, None::<fn(&Agent) -> bool>);
        if let Some((tj, _)) = threat {
            self.agents[idx].state = "flee";
            let away = aether_libm::atan2(self.agents[idx].x - self.agents[tj].x, self.agents[idx].z - self.agents[tj].z);
            self.agents[idx].heading = away;
            self.agents[idx].desired_speed = g_speed * 1.25;
        } else if self.agents[idx].think <= 0.0 {
            self.agents[idx].think = 0.3 + self.rng.next_f64() * 0.3;
            let mut best: Option<usize> = None;
            let mut bd = (g_sense * 1.2).powi(2);
            let ax = self.agents[idx].x;
            let az = self.agents[idx].z;
            for (j, c) in self.carcasses.iter().enumerate() {
                let dx = c.x - ax;
                let dz = c.z - az;
                let d2 = dx * dx + dz * dz;
                if d2 < bd { bd = d2; best = Some(j); }
            }
            let need_energy = self.agents[idx].energy < self.agents[idx].max_energy * 0.7;
            if best.is_some() && need_energy {
                let b = best.unwrap();
                self.agents[idx].target_x = self.carcasses[b].x;
                self.agents[idx].target_z = self.carcasses[b].z;
                self.agents[idx].state = "scavenge";
            } else {
                let (px, pz, pc, _) = self.find_grass(idx);
                if pc >= 0 {
                    self.agents[idx].target_x = px;
                    self.agents[idx].target_z = pz;
                    self.agents[idx].state = "graze";
                } else {
                    self.agents[idx].state = "wander";
                    let wa = self.rng.next_f64() * std::f64::consts::PI * 2.0;
                    let wp = self.clamp_home(self.agents[idx].x, self.agents[idx].z,
                        self.agents[idx].home_x, self.agents[idx].home_z,
                        self.agents[idx].range,
                        self.agents[idx].x + aether_libm::cos(wa) * 30.0,
                        self.agents[idx].z + aether_libm::sin(wa) * 30.0);
                    self.agents[idx].target_x = wp.0;
                    self.agents[idx].target_z = wp.1;
                }
            }
        }
        if self.agents[idx].state != "flee" {
            let d = js_hypot(self.agents[idx].target_x - self.agents[idx].x, self.agents[idx].target_z - self.agents[idx].z);
            if d < 1.6 {
                self.agents[idx].desired_speed = 0.0;
            } else {
                {
                    let tx = self.agents[idx].target_x;
                    let tz = self.agents[idx].target_z;
                    let dx = tx - self.agents[idx].x;
                    let dz = tz - self.agents[idx].z;
                    if dx * dx + dz * dz > 0.01 {
                        let want = aether_libm::atan2(dx, dz);
                        let mut diff = want - self.agents[idx].heading;
                        while diff > std::f64::consts::PI { diff -= std::f64::consts::PI * 2.0; }
                        while diff < -std::f64::consts::PI { diff += std::f64::consts::PI * 2.0; }
                        let turn = diff.clamp(-2.8 * h, 2.8 * h);
                        self.agents[idx].heading += turn;
                    }
                    self.agents[idx].desired_speed = g_speed * 0.55;
                }
            }
        }
        self.agents[idx].speed = self.agents[idx].speed + (self.agents[idx].desired_speed - self.agents[idx].speed) * (h * 5.0).min(1.0);
        let activity = self.agents[idx].speed / g_speed.max(1.0);
        self.agents[idx].activity = activity;
        self.agents[idx].x += aether_libm::sin(self.agents[idx].heading) * self.agents[idx].speed * h;
        self.agents[idx].z += aether_libm::cos(self.agents[idx].heading) * self.agents[idx].speed * h;
        self.agents[idx].y = terrain_height(self.agents[idx].x, self.agents[idx].z);

        let ci = if omn_eats_carcass {
            self.carcasses.iter().position(|c| js_hypot(c.x - self.agents[idx].x, c.z - self.agents[idx].z) < 1.8 && c.energy > 0.2)
        } else { None };
        if let Some(ci) = ci {
            if self.agents[idx].bite <= 0.0 {
                let take = 3.0f64.min(self.carcasses[ci].energy);
                self.carcasses[ci].energy -= take;
                self.agents[idx].energy = (self.agents[idx].energy + take * 8.0).min(self.agents[idx].max_energy);
                self.agents[idx].bite = 1.8;
                if self.carcasses[ci].energy <= 0.2 { self.carcasses.remove(ci); }
            }
        }
        if self.agents[idx].bite <= 0.0 && self.agents[idx].energy < self.agents[idx].max_energy * 0.9 {
            let gi = self.cell_index(self.agents[idx].x, self.agents[idx].z);
            if gi >= 0 && (gi as usize) < self.grass.len() && (self.grass[gi as usize] as f64) > 0.1 {
                let take = self.take_grass(idx, gi as usize, 0.18);
                self.agents[idx].energy = (self.agents[idx].energy + take * 58.0).min(self.agents[idx].max_energy);
                self.agents[idx].bite = 2.2;
            }
        }
        self.agents[idx].energy -= g_metabolism * (0.2 + self.agents[idx].activity * 0.32) * self.season_meta * h;
        if self.agents[idx].energy <= 0.0 { self.die(idx, "starved"); }
        else if self.agents[idx].age > self.agents[idx].genome.lifespan { self.die(idx, "aged"); }
        else if self.agents[idx].repro <= 0.0 && self.agents[idx].energy > self.agents[idx].max_energy * 0.8 { self.reproduce(idx, h); }
    }

    // ----- fish behaviour (Phase 4-B) ---------------------------------------

    fn update_fish(&mut self, idx: usize, h: f64) {
        let f_kind = self.agents[idx].kind;
        // Phase 33: diet gate — FISH grazes aquatic plants (eats_plant = true
        // for the legacy kind, bit-exact; a registered variant can drop it).
        let fish_eats_plant = self.diet_of(f_kind).eats_plant;
        self.agents[idx].age += h;
        let g_speed = self.agents[idx].genome.speed;
        let g_sense = self.agents[idx].genome.sense;
        let g_metabolism = self.agents[idx].genome.metabolism;
        let g_size = self.agents[idx].genome.size;
        self.agents[idx].bite -= h;
        self.agents[idx].repro -= h;
        self.agents[idx].think -= h;
        self.agents[idx].fear = (self.agents[idx].fear - 0.5 * h).clamp(0.0, 1.0);

        // Dehydration: fish stranded on land lose energy each tick.
        // Filter-feeding: ambient plankton/algae provide a baseline intake
        // while submerged (grazing alone cannot sustain fish — aquatic grass
        // is sparse in the shallow fringe).
        let ground_h = terrain_height(self.agents[idx].x, self.agents[idx].z);
        let in_water = ground_h < WATER_ZONE_THRESHOLD;
        if !in_water {
            self.agents[idx].energy -= FISH_DRY_DEATH_RATE * (h / 0.1);
        } else {
            self.agents[idx].energy = (self.agents[idx].energy + FISH_FILTER_GAIN * h).min(self.agents[idx].max_energy);
        }

        // Season metabolism for fish: [spring, summer, autumn, winter]
        let season_meta_fish = [1.1f64, 1.0, 0.9, 0.7][self.season as usize];

        let speed_mult = FISH_SPEED_MULT;
        let mut desired = 0.4 + g_speed * speed_mult * 0.18;

        if self.agents[idx].think <= 0.0 {
            self.agents[idx].think = 0.4 + self.fish_rng.next_f64() * 0.4;
            let social_high = self.agents[idx].genome.social > 0.8;
            let my_id = self.agents[idx].id;
            let school_mates = if social_high {
                self.nearest(idx, FISH, g_sense, Some(|b: &Agent| b.id != my_id))
            } else { None };

            if social_high && school_mates.is_some() {
                let (mate_j, _) = school_mates.unwrap();
                // Schooling: steer toward same-kind neighbor (no cap on group size
                // here; the biome density brake handles population control).
                self.agents[idx].target_x = self.agents[mate_j].x;
                self.agents[idx].target_z = self.agents[mate_j].z;
                self.agents[idx].state = "school";
                let dm = js_hypot(self.agents[mate_j].x - self.agents[idx].x, self.agents[mate_j].z - self.agents[idx].z);
                desired = if dm > 4.0 { g_speed * speed_mult * 0.6 } else { 0.0 };
            } else if self.agents[idx].energy < self.agents[idx].max_energy * 0.86 {
                let (px, pz, pc, _) = self.find_grass(idx);
                if pc >= 0 {
                    self.agents[idx].target_x = px;
                    self.agents[idx].target_z = pz;
                    self.agents[idx].state = "graze";
                    let d = js_hypot(px - self.agents[idx].x, pz - self.agents[idx].z);
                    desired = if d > 2.0 { g_speed * speed_mult * (0.55 + (1.0 - self.agents[idx].energy / self.agents[idx].max_energy) * 0.45) } else { 0.0 };
                } else {
                    self.agents[idx].state = "wander";
                    let wa = self.fish_rng.next_f64() * std::f64::consts::PI * 2.0;
                    let wp = self.clamp_home(self.agents[idx].x, self.agents[idx].z,
                        self.agents[idx].home_x, self.agents[idx].home_z,
                        self.agents[idx].range,
                        self.agents[idx].x + aether_libm::cos(wa) * 30.0,
                        self.agents[idx].z + aether_libm::sin(wa) * 30.0);
                    self.agents[idx].target_x = wp.0;
                    self.agents[idx].target_z = wp.1;
                    desired = g_speed * speed_mult * 0.55;
                }
            } else {
                self.agents[idx].state = "wander";
                let wx = self.agents[idx].x + (self.fish_rng.next_f64() - 0.5) * 45.0;
                let wz = self.agents[idx].z + (self.fish_rng.next_f64() - 0.5) * 45.0;
                let wp = self.clamp_home(self.agents[idx].x, self.agents[idx].z,
                    self.agents[idx].home_x, self.agents[idx].home_z,
                    self.agents[idx].range, wx, wz);
                self.agents[idx].target_x = wp.0;
                self.agents[idx].target_z = wp.1;
                desired = g_speed * speed_mult * 0.3;
            }
        }

        let a_state = self.agents[idx].state;
        if a_state != "flee" {
            let d = js_hypot(self.agents[idx].target_x - self.agents[idx].x, self.agents[idx].target_z - self.agents[idx].z);
            if d < 1.6 {
                self.agents[idx].state = "graze";
                desired = 0.0;
            } else {
                let tx = self.agents[idx].target_x;
                let tz = self.agents[idx].target_z;
                let dx = tx - self.agents[idx].x;
                let dz = tz - self.agents[idx].z;
                if dx * dx + dz * dz > 0.01 {
                    let want = aether_libm::atan2(dx, dz);
                    let mut diff = want - self.agents[idx].heading;
                    while diff > std::f64::consts::PI { diff -= std::f64::consts::PI * 2.0; }
                    while diff < -std::f64::consts::PI { diff += std::f64::consts::PI * 2.0; }
                    let turn = diff.clamp(-2.8 * h, 2.8 * h);
                    self.agents[idx].heading += turn;
                }
                self.agents[idx].desired_speed = desired;
            }
        }

        self.agents[idx].speed = self.agents[idx].speed + (self.agents[idx].desired_speed - self.agents[idx].speed) * (h * 5.0).min(1.0);
        let activity = self.agents[idx].speed / (g_speed * speed_mult).max(1.0);
        self.agents[idx].activity = activity;

        let epistasis = 1.0 + (g_speed / 8.0) * (g_size - 1.0) * 0.15 + (g_sense / 100.0) * (g_metabolism - 1.0) * 0.1;
        let mut drain = g_metabolism * (0.12 + 0.25 * activity) * epistasis;
        // Phase 33: ext-locus expression upkeep (matches update_herb). No-op
        // when `ext` is empty (the default genesis-OFF world, ×1.0 bit-exact);
        // only takes effect when genesis is ON and loci are present.
        drain *= 1.0 + (self.agents[idx].genome.ext.len() as f64) * LOCI_UPKEEP;
        let cap_for_kind = MAX_FISH as f64;
        let local_crowd = self.count(FISH) as f64 / cap_for_kind;
        let crowd_penalty = if local_crowd > 0.4 { 1.0 + (local_crowd - 0.4) * 0.5 } else { 1.0 };
        let total_drain = drain * crowd_penalty * season_meta_fish * h;
        self.agents[idx].energy -= total_drain;

        // Movement (no jump; fish stay near the ground surface y=terrain)
        let nx = self.agents[idx].x + aether_libm::sin(self.agents[idx].heading) * self.agents[idx].speed * h;
        let nz = self.agents[idx].z + aether_libm::cos(self.agents[idx].heading) * self.agents[idx].speed * h;
        self.agents[idx].x = nx;
        self.agents[idx].z = nz;
        self.agents[idx].y = terrain_height(nx, nz);

        // Hard water boundary: fish cannot leave WATER_LEVEL + 3.0
        let post_h = terrain_height(self.agents[idx].x, self.agents[idx].z);
        if post_h >= WATER_LEVEL + 3.0 {
            // Push back toward home (deep water)
            let back = aether_libm::atan2(self.agents[idx].home_x - self.agents[idx].x, self.agents[idx].home_z - self.agents[idx].z);
            self.agents[idx].heading = back;
            self.agents[idx].x = self.agents[idx].x + aether_libm::sin(back) * 1.0;
            self.agents[idx].z = self.agents[idx].z + aether_libm::cos(back) * 1.0;
            self.agents[idx].y = terrain_height(self.agents[idx].x, self.agents[idx].z);
        }

        // Feeding: bite when grass present, water-efficient (×1.5 vs herb).
        // Phase 4-B biome hook: inside the aquatic biome (or in water when no
        // registry biome applies, e.g. Ecology::new) the want coefficient is
        // boosted ×1.5; on land outside aquatic the fish does not graze.
        if self.agents[idx].bite <= 0.0 && self.agents[idx].energy < self.agents[idx].max_energy * 0.9 {
            let gi = self.cell_index(self.agents[idx].x, self.agents[idx].z);
            let in_aquatic = match self.biome_reg.find_biome(self.agents[idx].x, self.agents[idx].z) {
                Some(b) => b.name == "aquatic",
                None => in_water,
            };
            let want = if in_aquatic { 0.12 * 1.5 } else { 0.0 };
            if fish_eats_plant && want > 0.0 && gi >= 0 && (gi as usize) < self.grass.len() && (self.grass[gi as usize] as f64) > 0.1 {
                let take = self.take_grass(idx, gi as usize, want);
                self.agents[idx].energy = (self.agents[idx].energy + take * 62.0 * 1.5).min(self.agents[idx].max_energy);
                self.agents[idx].bite = FISH_BITE_INTERVAL;
            }
        }

        if self.agents[idx].energy <= 0.0 {
            self.die(idx, "starved");
        } else if self.agents[idx].age > self.agents[idx].genome.lifespan {
            self.die(idx, "aged");
        } else if self.agents[idx].repro <= 0.0 && self.agents[idx].energy > self.agents[idx].max_energy * 0.68 {
            self.reproduce(idx, h);
        }
    }

    // ----- decomposer behaviour (Phase 4-C) ----------------------------------

    /// DECO three-state machine: search → approach → eat. Carcass-only diet
    /// (no grass fallback), wide sense×1.5 perception, slow speed×0.8, and a
    /// mineral deposit into `mineral_field` per bite that boosts grass growth.
    fn update_deco(&mut self, idx: usize, h: f64) {
        self.agents[idx].age += h;
        let g_speed = self.agents[idx].genome.speed;
        let g_sense = self.agents[idx].genome.sense;
        let g_metabolism = self.agents[idx].genome.metabolism;
        let deco_eff = self.agents[idx].genome.deco_efficiency;
        // Phase 16: the agent's clade niche modulates foraging direction and
        // mineral-deposit bias below (0.0 for root / unassigned agents).
        let clade_niche = self.clade_niche_of(self.agents[idx].clade_id);
        // Phase 33: diet gate — DECO is a carcass-only decomposer
        // (eats_carcass = true for the legacy kind, bit-exact; a registered
        // variant can drop the source and fall back to wander).
        let deco_eats_carcass = self.diet_of(self.agents[idx].kind).eats_carcass;
        self.agents[idx].bite -= h;
        self.agents[idx].repro -= h;
        self.agents[idx].think -= h;
        self.agents[idx].fear = (self.agents[idx].fear - 0.5 * h).clamp(0.0, 1.0);

        // Season metabolism [spring, summer, autumn, winter] — same curve as herb.
        let season_meta_deco = [1.1f64, 1.0, 0.9, 0.7][self.season as usize];
        let speed_mult = DECO_SPEED_MULT;

        // State 1 — searching: nearest UNLOCKED carcass within sense×1.5, else wander.
        // Phase 8: skip carcasses whose occupancy lock is held by another agent.
        if self.agents[idx].think <= 0.0 {
            self.agents[idx].think = 0.35 + self.rng.next_f64() * 0.35;
            let mut best: Option<usize> = None;
            let mut bd = (g_sense * 1.5) * (g_sense * 1.5);
            let ax = self.agents[idx].x;
            let az = self.agents[idx].z;
            let self_id = self.agents[idx].id;
            if deco_eats_carcass {
                for (j, c) in self.carcasses.iter().enumerate() {
                    // Phase 8: skip carcasses whose occupancy lock is held by another agent.
                    if !self.carcass_claimable(c, self_id) { continue; }
                    let dx = c.x - ax;
                    let dz = c.z - az;
                    let d2 = dx * dx + dz * dz;
                    if d2 < bd { bd = d2; best = Some(j); }
                }
            }
            if let Some(b) = best {
                self.agents[idx].target_x = self.carcasses[b].x;
                self.agents[idx].target_z = self.carcasses[b].z;
                self.agents[idx].state = "approach";
            } else {
                self.agents[idx].state = "wander";
                // Phase 16: clade niche biases the wander heading — diverging
                // clades roam in different directions (spatial niche split).
                let wa = self.rng.next_f64() * std::f64::consts::PI * 2.0 + clade_niche * NICHE_HEADING_BIAS;
                let wp = self.clamp_home(self.agents[idx].x, self.agents[idx].z,
                    self.agents[idx].home_x, self.agents[idx].home_z,
                    self.agents[idx].range,
                    self.agents[idx].x + aether_libm::cos(wa) * 30.0,
                    self.agents[idx].z + aether_libm::sin(wa) * 30.0);
                self.agents[idx].target_x = wp.0;
                self.agents[idx].target_z = wp.1;
            }
        }

        // State 2 — approaching: steer toward the carcass; arrive → try lock → eat.
        let d_target = js_hypot(self.agents[idx].target_x - self.agents[idx].x, self.agents[idx].target_z - self.agents[idx].z);
        let desired = match self.agents[idx].state {
            "approach" => {
                if d_target < 1.0 {
                    // Phase 8: occupancy lock arbitration at arrival. Find the
                    // carcass we steered toward; if it is free (or ours), claim
                    // it. If another agent got there first, abandon → wander.
                    let tx = self.agents[idx].target_x;
                    let tz = self.agents[idx].target_z;
                    let self_id = self.agents[idx].id;
                    let arrival = self.carcasses.iter().position(|c| js_hypot(c.x - tx, c.z - tz) < 1.5);
                    let can_claim = match arrival {
                        Some(j) => self.carcass_claimable(&self.carcasses[j], self_id),
                        None => false, // target vanished mid-approach
                    };
                    if can_claim {
                        self.acquire_carcass_lock(arrival.unwrap(), self_id);
                        self.agents[idx].state = "eat";
                    } else {
                        self.agents[idx].state = "wander";
                        // Phase 16: same heading bias as the search branch.
                        let wa = self.rng.next_f64() * std::f64::consts::PI * 2.0 + clade_niche * NICHE_HEADING_BIAS;
                        let wp = self.clamp_home(self.agents[idx].x, self.agents[idx].z,
                            self.agents[idx].home_x, self.agents[idx].home_z,
                            self.agents[idx].range,
                            self.agents[idx].x + aether_libm::cos(wa) * 30.0,
                            self.agents[idx].z + aether_libm::sin(wa) * 30.0);
                        self.agents[idx].target_x = wp.0;
                        self.agents[idx].target_z = wp.1;
                    }
                }
                if d_target > 1.0 { g_speed * speed_mult * 0.8 } else { 0.0 }
            }
            "eat" => 0.0,
            _ => g_speed * speed_mult * 0.4,
        };
        if self.agents[idx].state != "wander" && d_target > 0.01 {
            let dx = self.agents[idx].target_x - self.agents[idx].x;
            let dz = self.agents[idx].target_z - self.agents[idx].z;
            if dx * dx + dz * dz > 0.01 {
                let want = aether_libm::atan2(dx, dz);
                let mut diff = want - self.agents[idx].heading;
                while diff > std::f64::consts::PI { diff -= std::f64::consts::PI * 2.0; }
                while diff < -std::f64::consts::PI { diff += std::f64::consts::PI * 2.0; }
                let turn = diff.clamp(-2.8 * h, 2.8 * h);
                self.agents[idx].heading += turn;
            }
        }
        self.agents[idx].desired_speed = desired;

        self.agents[idx].speed = self.agents[idx].speed + (self.agents[idx].desired_speed - self.agents[idx].speed) * (h * 5.0).min(1.0);
        let activity = self.agents[idx].speed / (g_speed * speed_mult).max(1.0);
        self.agents[idx].activity = activity;

        self.agents[idx].x += aether_libm::sin(self.agents[idx].heading) * self.agents[idx].speed * h;
        self.agents[idx].z += aether_libm::cos(self.agents[idx].heading) * self.agents[idx].speed * h;
        self.agents[idx].y = terrain_height(self.agents[idx].x, self.agents[idx].z);

        // State 3 — eating: bite the nearest carcass within 1.0.
        let ci = self.carcasses.iter().position(|c| js_hypot(c.x - self.agents[idx].x, c.z - self.agents[idx].z) < 1.0 && c.energy > 0.2);
        if let Some(ci) = ci {
            if self.agents[idx].bite <= 0.0 {
                let take = DECO_BITE_INTERVAL.min(self.carcasses[ci].energy);
                self.carcasses[ci].energy -= take;
                self.agents[idx].energy = (self.agents[idx].energy + take * 62.0 * deco_eff).min(self.agents[idx].max_energy);
                self.agents[idx].bite = DECO_BITE_INTERVAL;

                // Mineral deposit: carcass mass → soil minerals at the deco's
                // cell, diffused to the 4-neighborhood (Gaussian r=1, ×0.4).
                let gi = self.cell_index(self.agents[idx].x, self.agents[idx].z);
                if gi >= 0 {
                    let g = gi as usize;
                    let boost = take * deco_eff * MINERAL_DEPOSIT_RATE;
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
                    if g < self.mineral_field.len() {
                        self.mineral_field[g] = (self.mineral_field[g] as f64 + boost).min(1.0) as f32;
                        let gi_col = (g % GRID_N as usize) as isize;
                        let gi_row = (g / GRID_N as usize) as isize;
                        for (ni, nj) in [(gi_col + 1, gi_row), (gi_col - 1, gi_row), (gi_col, gi_row + 1), (gi_col, gi_row - 1)] {
                            if ni >= 0 && nj >= 0 && (ni as u32) < GRID_N && (nj as u32) < GRID_N {
                                let ng = (nj as usize) * GRID_N as usize + ni as usize;
                                if self.suit[ng] > 0.02 {
                                    self.mineral_field[ng] = (self.mineral_field[ng] as f64 + boost * 0.4).min(1.0) as f32;
                                }
                            }
                        }
                        // Phase 16: niche-directed deposit — clades with
                        // diverging niches enrich different directions, which
                        // spatially partitions the soil-mineral resource.
                        if clade_niche.abs() > 0.05 {
                            let theta = clade_niche * std::f64::consts::PI;
                            let (dxn, dzn) = (aether_libm::cos(theta), aether_libm::sin(theta));
                            let cands = [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)];
                            let mut best = cands[0];
                            let mut best_dot = f64::NEG_INFINITY;
                            for &cand in &cands {
                                let d = dxn * cand.0 as f64 + dzn * cand.1 as f64;
                                if d > best_dot { best_dot = d; best = cand; }
                            }
                            let dni = gi_col + best.0 as isize;
                            let dnj = gi_row + best.1 as isize;
                            if dni >= 0 && dnj >= 0 && (dni as u32) < GRID_N && (dnj as u32) < GRID_N {
                                let ng = (dnj as usize) * GRID_N as usize + dni as usize;
                                if self.suit[ng] > 0.02 {
                                    self.mineral_field[ng] = (self.mineral_field[ng] as f64 + boost * NICHE_DEPOSIT_BIAS).min(1.0) as f32;
                                }
                            }
                        }
                    }
                }

                if self.carcasses[ci].energy <= 0.2 { self.carcasses.remove(ci); }
                self.agents[idx].state = "search";
            }
        }

        // Metabolism: slow decomposer, low base drain.
        let cap_for_kind = MAX_DECO as f64;
        let local_crowd = self.count(DECO) as f64 / cap_for_kind;
        let crowd_penalty = if local_crowd > 0.4 { 1.0 + (local_crowd - 0.4) * 0.5 } else { 1.0 };
        // Phase 7: in selective mode, higher deco_efficiency reduces metabolic drain by up to 10%.
        // eff ∈ [0.5, 1.5] → mod ∈ [1.1, 0.9]; neutral at eff=1.0 (mod=1.0).
        let eff_metab_mod = if self.deco_eff_choice_mode == save_state::DECO_CHOICE_SELECTIVE {
            1.2 - 0.2 * deco_eff
        } else { 1.0 };
        // Phase 16: competitive-exclusion burden — the loser of an overlapping
        // clade pair drains +15 % faster (burden map rebuilt every detection
        // pass; always empty in legacy mode).
        let niche_burden = self.clade_burden_of(self.agents[idx].clade_id);
        // Phase 17: impact-dust metabolic stress also hits DECO (its drain
        // path uses its own season table, bypassing self.season_meta).
        let cat_stress = 1.0 + self.cataclysm_stress_scale();
        let mut drain = g_metabolism * (0.12 + 0.25 * activity);
        // Phase 33: ext-locus expression upkeep (matches update_herb). No-op
        // when `ext` is empty (default genesis-OFF world, ×1.0 bit-exact);
        // only takes effect when genesis is ON and loci are present.
        drain *= 1.0 + (self.agents[idx].genome.ext.len() as f64) * LOCI_UPKEEP;
        let total_drain = drain * crowd_penalty * season_meta_deco * h * eff_metab_mod * (1.0 + niche_burden) * cat_stress;
        self.agents[idx].energy -= total_drain;

        if self.agents[idx].energy <= 0.0 {
            self.die(idx, "starved");
        } else if self.agents[idx].age > self.agents[idx].genome.lifespan {
            self.die(idx, "aged");
        } else if self.agents[idx].repro <= 0.0 && self.agents[idx].energy > self.agents[idx].max_energy * 0.8 {
            self.reproduce(idx, h);
        }
    }

    // ----- main update loop -------------------------------------------------

    pub fn update(&mut self, dt: f64, daylight: f64) {
        if dt <= 0.0 { return; }
        // Phase 17: cataclysm env modulation. Dust darkens the daylight (also
        // drives the resting/plasticity thresholds downstream); the climate
        // shift throttles grass growth below. With no active cataclysm both
        // multipliers are exactly 1.0 — f64 x*1.0 is bit-exact, so cataclysm-
        // free runs keep their checksum trajectory untouched.
        let (dust_mult, grow_mult) = self.cataclysm_env_mods();
        self.daylight = daylight * dust_mult;
        let steps = ((dt / 0.1).round() as i32).max(1);
        let h = dt / steps as f64;
        for _s in 0..steps {
            self.time += h;
            self.tick_count += 1;
            // Phase 17: per-substep cataclysm countdown (pure bookkeeping,
            // zero RNG).
            self.tick_cataclysms();

            self.season = ((self.time / 1800.0) as i32) % 4;
            let season_grass = [1.3f64, 0.6, 0.8, 0.25][self.season as usize];
            let season_meta = [1.0f64, 1.15, 0.9, 0.75][self.season as usize];
            // Phase 17: impact-dust metabolic stress — the additive factor
            // scales with (1 − dust), so every drain (∝ genome metabolism)
            // rises while the sky is dark (K-Pg-style high-maintenance
            // penalty). Exactly 1.0 without active impacts → bit-exact no-op.
            self.season_meta = season_meta * (1.0 + self.cataclysm_stress_scale());

            // Grass logistic growth — mineral deposits (Phase 4-C) boost the
            // per-cell rate by up to +30% before the per-species update.
            // Phase 17: grow_mult = climate-shift throttle; self.daylight
            // already carries the dust darkening.
            let grow = 0.0056 * season_grass * grow_mult * (0.18 + 0.82 * self.daylight) * (1.0 + self.rain * 0.02);
            // Phase 18: nutrient fast-pool fertility (exactly 1.0 at baseline
            // → bit-exact no-op; spec: "fast pool 直接调节 grass logistic
            // 增长的 baseline rate", centered on FAST_BASELINE so existing
            // calibration absorbs the multiplier naturally).
            let fert_mult = self.biochem_growth_mult();
            let pn = self.plants.len();
            for i in 0..self.grass.len() {
                let suit = self.suit[i];
                if suit <= 0.02 { continue; }
                let mut total = 0.0f64;
                for s in 0..pn { total += self.grass_p[s][i] as f64; }
                if total >= 1.0 { continue; }
                let shade_f = 1.0 - self.shade[i] as f64 * 0.55;
                let cell_grow = grow * (1.0 + self.mineral_field[i] as f64 * GROWTH_BONUS_PER_MINERAL)
                    * fert_mult;
                for sp in 0..pn {
                    let b = self.grass_p[sp][i] as f64;
                    let nb = b + self.suit_eff_by[sp][i] as f64 * cell_grow * rmax_eff(&self.plants[sp]) * (1.0 - total) * (0.42 + b) * shade_f * h;
                    self.grass_p[sp][i] = if nb > 1.0 { 1.0 } else { nb } as f32;
                }
                let mut gsum = 0.0f64;
                for s in 0..pn { gsum += self.grass_p[s][i] as f64; }
                self.grass[i] = gsum as f32;
            }

            // Grass spread (Phase 2: kept for checksum parity)
            if self.tick_count % 40 == 0 {
                let mut buf = self.spread_buf.take().unwrap_or_else(|| vec![0.0f32; self.grass.len()]);
                for sp in 0..pn {
                    buf.copy_from_slice(&self.grass_p[sp]);
                    for j in 1..(GRID_N as usize) - 1 {
                        for i in 1..(GRID_N as usize) - 1 {
                            let idx = j * GRID_N as usize + i;
                            let gv = self.grass_p[sp][idx];
                            if gv < 0.55 || self.suit[idx] < 0.1 { continue; }
                            let spread = gv * 0.01;
                            buf[idx] -= spread * 4.0;
                            if self.suit[idx + 1] > 0.02 { buf[idx + 1] = (buf[idx + 1] + spread).min(1.0); }
                            if self.suit[idx - 1] > 0.02 { buf[idx - 1] = (buf[idx - 1] + spread).min(1.0); }
                            if self.suit[idx + GRID_N as usize] > 0.02 { buf[idx + GRID_N as usize] = (buf[idx + GRID_N as usize] + spread).min(1.0); }
                            if self.suit[idx - GRID_N as usize] > 0.02 { buf[idx - GRID_N as usize] = (buf[idx - GRID_N as usize] + spread).min(1.0); }
                        }
                    }
                    self.grass_p[sp].copy_from_slice(&buf);
                }
                for i in 0..self.grass.len() {
                    let mut gsum = 0.0f64;
                    for s in 0..pn { gsum += self.grass_p[s][i] as f64; }
                    self.grass[i] = gsum as f32;
                }
                self.spread_buf = Some(buf);
            }

            if self.tick_count % 100 == 0 { self.h_cache = self.count(HERB) as u32; }

            // Main agent loop: visit newly spawned agents in the same tick (matches JS for-of over growing array)
            let mut i = 0usize;
            while i < self.agents.len() {
                if self.agents[i].alive {
                    let kind = self.agents[i].kind;
                    self.agents[i].prev_x = self.agents[i].x;
                    self.agents[i].prev_z = self.agents[i].z;
                    self.agents[i].prev_heading = self.agents[i].heading;
                    if !self.agents[i].cargo.is_empty() {
                        let slot = (self.time / 8.0).floor() as i32;
                        if slot != self.agents[i].trail_slot {
                            self.agents[i].trail_slot = slot;
                            let th = self.agents[i].trail_head;
                            self.agents[i].trail_t[th] = self.time;
                            self.agents[i].trail_x[th] = self.agents[i].x;
                            self.agents[i].trail_z[th] = self.agents[i].z;
                            self.agents[i].trail_head = (th + 1) % TRAIL_SLOTS;
                            if self.agents[i].trail_n < TRAIL_SLOTS { self.agents[i].trail_n += 1; }
                        }
                        self.gut_transit(i, h);
                    }
                    // Phase 4-A: biome hook before kind dispatch. Agents in a
                    // biome with exceeded per-species capacity get no special
                    // handling here (reproduce handles the brake), but the
                    // lookup is kept for downstream spatial features.
                    let _biome_ref = self.biome_reg.find_biome(self.agents[i].x, self.agents[i].z);
                    let beh = self.kind_def(kind).behavior;
                    match beh {
                        BehaviorTemplate::Herb => self.update_herb(i, h),
                        BehaviorTemplate::Scav => self.update_scav(i, h),
                        BehaviorTemplate::Omn => self.update_omn(i, h),
                        BehaviorTemplate::Fish => self.update_fish(i, h),
                        BehaviorTemplate::Deco => self.update_deco(i, h),
                        BehaviorTemplate::Pred => self.update_pred(i, h),
                    }
                }
                i += 1;
            }

            for c in self.carcasses.iter_mut() { c.age += h; }
            // Phase 8: prune stale occupancy locks.
            if self.carcass_lock_enabled {
                for i in (0..self.carcasses.len()).rev() {
                    let c = &self.carcasses[i];
                    let mut stale = false;
                    if let Some(owner_id) = c.occupied_by {
                        if c.age > 60.0 { stale = true; } else {
                            // Find whether the owner still stands within 5.0 of the carcass.
                            if let Some(a) = self.agents.iter().find(|a| a.id == owner_id && a.alive) {
                                if (a.x - c.x).hypot(a.z - c.z) > 5.0 { stale = true; }
                            } else { stale = true; } // owner no longer alive
                        }
                    }
                    if stale { self.carcasses[i].occupied_by = None; }
                }
            }
            for i in (0..self.carcasses.len()).rev() {
                self.carcasses[i].energy -= 1.4 * h;
                if self.carcasses[i].energy <= 0.2 { self.carcasses.remove(i); }
            }

            // Mineral field decay (Phase 4-C): applied once per step so the
            // field persists only where decomposers actively feed.
            for m in &mut self.mineral_field {
                *m = (*m as f64 * MINERAL_DECAY) as f32;
            }

            if !self.agents.is_empty() && self.agents.iter().any(|a| !a.alive) {
                self.agents.retain(|a| a.alive);
            }

            // Endangered predator immigration
            if (self.count(PRED) as u32) < 5 {
                let imm_roll = self.rng.next_f64();
                if imm_roll < 0.002 {
                    let ang = self.rng.next_f64() * std::f64::consts::PI * 2.0;
                    let r = 350.0 + self.rng.next_f64().sqrt() * 200.0;
                    let x = aether_libm::cos(ang) * r;
                    let z = aether_libm::sin(ang) * r;
                    let th = terrain_height(x, z);
                    if th > WATER_LEVEL + 1.5 && th < 80.0 && slope_at(x, z) < 0.7 {
                        let age_imm = 30.0 + self.rng.next_f64() * 80.0;
                        let g_imm = self.random_genome(PRED);
                        self.spawn(PRED, x, z, Some(g_imm), 0, age_imm);
                    }
                }
            }

            // Safety valve: prune oldest when over cap
            let total_cap = MAX_HERB + MAX_PRED + MAX_SCAV + MAX_OMN + MAX_FISH + MAX_DECO;
            if self.agents.len() > total_cap + 60 {
                self.agents.sort_by(|a, b| b.age.partial_cmp(&a.age).unwrap());
                self.agents.truncate(total_cap);
            }

            if self.tick_count - self.last_sample >= 20 {
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
    }

    // ----- sample / stats / checksum ----------------------------------------

    fn sample(&mut self) {
        let mut h = 0usize;
        let mut p = 0usize;
        let mut speed_h = 0.0f64;
        let mut sense_h = 0.0f64;
        let mut size_h = 0.0f64;
        let mut speed_p = 0.0f64;
        let mut sense_p = 0.0f64;
        let mut scav = 0usize;
        let mut omn = 0usize;
        let mut gen = 0u32;
        let mut coop_h = 0usize;
        let mut help_thr_sum = 0.0f64;
        let mut pref_sum_h = 0.0f64;
        let mut tol_sum_h = 0.0f64;
        let mut size2_h = 0.0f64;
        for a in &self.agents {
            if !a.alive { continue; }
            gen = gen.max(a.generation);
            match a.kind {
                HERB => {
                    h += 1;
                    speed_h += a.genome.speed;
                    sense_h += a.genome.sense;
                    size_h += a.genome.size;
                    if a.image > 0 { coop_h += 1; }
                    help_thr_sum += a.genome.help_thr.unwrap_or(0.0);
                    if a.genome.pref_sig.is_some() {
                        pref_sum_h += a.genome.pref_sig.unwrap();
                        tol_sum_h += a.genome.tol_sig.unwrap_or(0.0);
                    }
                    size2_h += a.genome.size * a.genome.size;
                }
                SCAV => { scav += 1; }
                OMN => { omn += 1; }
                // FISH falls into the catch-all (legacy JS parity, counted as p);
                // DECO is tracked separately via Stats.deco_count, not in p.
                DECO => {}
                _ => {
                    p += 1;
                    speed_p += a.genome.speed;
                    sense_p += a.genome.sense;
                }
            }
        }
        let plant = self.plant_amount();
        let plant_by = self.plant_amount_by();
        self.history.push(HistorySample {
            t: self.time,
            season: self.season,
            h, p, scav, omn,
            plant,
            plant_by,
            speed_h: if h > 0 { speed_h / h as f64 } else { 0.0 },
            sense_h: if h > 0 { sense_h / h as f64 } else { 0.0 },
            size_h: if h > 0 { size_h / h as f64 } else { 0.0 },
            speed_p: if p > 0 { speed_p / p as f64 } else { 0.0 },
            sense_p: if p > 0 { sense_p / p as f64 } else { 0.0 },
            gap_hp: if h > 0 && p > 0 { speed_h / h as f64 - speed_p / p as f64 } else { 0.0 },
            coop_share_h: if h > 0 { coop_h as f64 / h as f64 } else { 0.0 },
            help_thr_h: if h > 0 { help_thr_sum / h as f64 } else { 0.0 },
            sig_mean_h: if h > 0 { size_h / h as f64 } else { 0.0 },
            sig_var_h: if h > 0 { size2_h / h as f64 - (size_h / h as f64).powi(2) } else { 0.0 },
            pref_mean_h: if h > 0 { pref_sum_h / h as f64 } else { 0.0 },
            tol_mean_h: if h > 0 { tol_sum_h / h as f64 } else { 0.0 },
            gen,
        });
        if self.history.len() > 240 { self.history.remove(0); }

        // Evolution stats every 600 ticks
        if self.tick_count % 600 == 0 {
            for (kind_idx, kind) in [(0, HERB), (1, PRED), (2, SCAV), (3, OMN), (4, FISH), (5, DECO)].iter() {
                let alive: Vec<&Agent> = self.agents.iter().filter(|a| a.alive && a.kind == *kind).collect();
                if alive.is_empty() { continue; }
                let n = alive.len();
                let avg_speed: f64 = alive.iter().map(|a| a.genome.speed).sum::<f64>() / n as f64;
                let avg_sense: f64 = alive.iter().map(|a| a.genome.sense).sum::<f64>() / n as f64;
                let avg_size: f64 = alive.iter().map(|a| a.genome.size).sum::<f64>() / n as f64;
                let avg_metab: f64 = alive.iter().map(|a| a.genome.metabolism).sum::<f64>() / n as f64;
                let avg_bold: f64 = alive.iter().map(|a| a.genome.bold).sum::<f64>() / n as f64;
                let avg_curious: f64 = alive.iter().map(|a| a.genome.curious).sum::<f64>() / n as f64;
                let avg_social: f64 = alive.iter().map(|a| a.genome.social).sum::<f64>() / n as f64;
                let avg_help: f64 = alive.iter().map(|a| a.genome.help_thr.unwrap_or(0.0)).sum::<f64>() / n as f64;
                let fin_color_avg: f64 = if *kind == FISH { alive.iter().map(|a| a.genome.fin_color).sum::<f64>() / n as f64 } else { 0.0 };
                // Phase 7: record deco_efficiency stats for DECO samples.
                let avg_deco: f64 = if *kind == DECO {
                    alive.iter().map(|a| a.genome.deco_efficiency).sum::<f64>() / n as f64
                } else { 0.0 };
                let deco_std: f64 = if *kind == DECO {
                    let m = avg_deco;
                    (alive.iter().map(|a| (a.genome.deco_efficiency - m).powi(2)).sum::<f64>() / n as f64).sqrt()
                } else { 0.0 };
                let max_gen = alive.iter().map(|a| a.generation).max().unwrap();
                self.evolution[*kind_idx].push(EvolutionSample {
                    t: self.time, gen: max_gen, count: n,
                    speed: avg_speed, sense: avg_sense, size: avg_size,
                    metabolism: avg_metab, bold: avg_bold, curious: avg_curious,
                    social: avg_social, help_thr: avg_help,
                    deco_eff: avg_deco, deco_eff_std: deco_std,
                });
                if self.evolution[*kind_idx].len() > 100 { self.evolution[*kind_idx].remove(0); }

                // Speciation detection for HERB only
                if *kind == HERB && alive.len() >= 30 {
                    let speeds: Vec<f64> = alive.iter().map(|a| a.genome.speed).collect();
                    let avg_spd = speeds.iter().sum::<f64>() / speeds.len() as f64;
                    let cv = (speeds.iter().map(|v| (v - avg_spd).powi(2)).sum::<f64>() / speeds.len() as f64).sqrt() / avg_spd.abs().max(1e-9);
                    let socials: Vec<f64> = alive.iter().map(|a| a.genome.social).collect();
                    let social_high = socials.iter().filter(|&&s| s > 0.6).count() as f64 / speeds.len() as f64;
                    let social_low = socials.iter().filter(|&&s| s < 0.4).count() as f64 / speeds.len() as f64;
                    if cv > 0.12 && social_high > 0.15 && social_low > 0.15 {
                        // Labeling mutates agents, so track indices in array order
                        // (must match JS's alive.filter() order exactly).
                        let alive_idx: Vec<usize> = self.agents.iter().enumerate()
                            .filter(|(_, a)| a.alive && a.kind == HERB)
                            .map(|(i, _)| i)
                            .collect();
                        let mut sorted_idx = alive_idx.clone();
                        sorted_idx.sort_by(|&a, &b| self.agents[a].genome.speed.partial_cmp(&self.agents[b].genome.speed).unwrap());
                        let sorted_speeds: Vec<f64> = sorted_idx.iter().map(|&i| self.agents[i].genome.speed).collect();
                        let mut max_gap = 0.0f64;
                        let mut gap_idx = 0usize;
                        for i in 1..sorted_speeds.len() {
                            let gap = sorted_speeds[i] - sorted_speeds[i - 1];
                            if gap > max_gap { max_gap = gap; gap_idx = i; }
                        }
                        let low_avg: f64 = sorted_speeds[..gap_idx].iter().sum::<f64>() / (gap_idx as f64).max(1.0);
                        let high_avg: f64 = sorted_speeds[gap_idx..].iter().sum::<f64>() / ((sorted_speeds.len() - gap_idx) as f64).max(1.0);
                        // JS Math.round — truncation via `as usize` would be wrong.
                        let count_fast = (social_high * speeds.len() as f64 + 0.5) as usize;
                        let count_slow = (social_low * speeds.len() as f64 + 0.5) as usize;
                        let lineages = vec![
                            LineageInfo { id: "social_fast", name: "迅聚", avg_speed: low_avg, count: count_fast },
                            LineageInfo { id: "solitary_slow", name: "缓独", avg_speed: high_avg, count: count_slow },
                        ];
                        self.speciation_event = Some(SpeciationEvent {
                            t: self.time, parent_kind: self.kind_name(HERB), lineages,
                            cv_speed: (cv * 1000.0).round() / 1000.0,
                            gap_idx,
                            name_suffix: format!("#{}", self.speciation_fired + 1),
                        });
                        if self.speciation_fired < 3 && self.time - self.last_speciation > 60.0 {
                            self.last_speciation = self.time;
                            self.speciation_fired += 1;
                        }
                        // 谱系标记：每次 speciation event 都重打标（不受 speciationFired 节流）；
                        // 已隔离谱系的个体不重划（保持隔离群成员稳定）。打标不消耗 RNG。
                        for (i, &idx) in sorted_idx.iter().enumerate() {
                            let cur = self.agents[idx].lineage.unwrap_or("");
                            if self.isolated_groups.contains_key(cur) { continue; }
                            self.agents[idx].lineage = Some(if i < gap_idx { "迅聚" } else { "缓独" });
                        }
                        // 隔离登记：分化群之间生殖隔离；code 按 JS 的 keys().length + 1 递增
                        for name in ["迅聚", "缓独"] {
                            if !self.isolated_groups.contains_key(name) {
                                let code = format!("隔离群-{}", self.isolated_groups.len() + 1);
                                self.isolated_groups.insert(name, IsolatedGroup { code, kind: self.kind_name(HERB), since: self.time });
                                self.isolated_order.push(name);
                            }
                        }
                    }
                }
            }
        }

        // Isolation history — iterate clusters in first-occurrence order (JS
        // Object.keys insertion order) so the FP summation order matches.
        let iso_names: std::collections::HashSet<&str> = self.isolated_groups.keys().cloned().collect();
        if iso_names.len() >= 2 {
            let mut iso_clusters: HashMap<&str, Vec<usize>> = HashMap::new();
            let mut iso_key_order: Vec<&str> = Vec::new();
            for (j, a) in self.agents.iter().enumerate() {
                if !a.alive || a.kind != HERB || a.lineage.is_none() { continue; }
                let lin = a.lineage.unwrap();
                if iso_names.contains(lin) {
                    let entry = iso_clusters.entry(lin).or_insert_with(Vec::new);
                    if entry.is_empty() { iso_key_order.push(lin); }
                    entry.push(j);
                }
            }
            let iso_keys: Vec<&str> = iso_key_order.into_iter().filter(|k| !iso_clusters[k].is_empty()).collect();
            if iso_keys.len() >= 2 {
                let mut sum = 0.0f64;
                let mut pairs = 0usize;
                for ki in 0..iso_keys.len() {
                    for kj in (ki + 1)..iso_keys.len() {
                        for &ai in &iso_clusters[iso_keys[ki]] {
                            for &bi in &iso_clusters[iso_keys[kj]] {
                                sum += self.pair_trait_distance(ai, bi);
                                pairs += 1;
                            }
                        }
                    }
                }
                if pairs > 0 {
                    self.isolation_history.push(IsolationHistoryPoint { t: self.time, dist: (sum / pairs as f64 * 10000.0).round() / 10000.0, pairs });
                    if self.isolation_history.len() > 200 { self.isolation_history.remove(0); }
                }
            }
        }

        let herb_count = self.count(HERB);
        let pred_count = self.count(PRED);
        self.selection_pressure = SelectionPressure {
            plant,
            pred_ratio: if herb_count > 0 { pred_count as f64 / herb_count as f64 } else { 0.0 },
            crowding: (herb_count + pred_count) as f64 / (MAX_HERB + MAX_PRED) as f64,
        };

        // Phase 15: clade speciation detection. Pure bookkeeping — consumes no
        // RNG draws, so RNG streams and the golden checksum are unaffected.
        if self.tick_count % SPECIATION_CHECK_TICKS == 0 {
            self.check_speciation();
        }
    }

    fn plant_amount(&self) -> f64 {
        let mut s = 0.0f64;
        let mut w = 0usize;
        for i in 0..self.grass.len() {
            if self.suit[i] > 0.02 { s += self.grass[i] as f64; w += 1; }
        }
        if w > 0 { s / w as f64 } else { 0.0 }
    }

    fn plant_amount_by(&self) -> Vec<f64> {
        let pn = self.plants.len();
        let mut out = vec![0.0f64; pn];
        let mut w = 0usize;
        for i in 0..self.grass.len() {
            if self.suit[i] > 0.02 {
                for s in 0..pn { out[s] += self.grass_p[s][i] as f64; }
                w += 1;
            }
        }
        if w > 0 { out.into_iter().map(|v| v / w as f64).collect() } else { out }
    }

    // ----- Phase 15: speciation engine ---------------------------------------

    /// Arithmetic-mean genome over `list` (Option genes averaged over their
    /// carriers, None when no carrier).
    fn mean_genome(list: &[&Agent]) -> Genome {
        let n = (list.len().max(1)) as f64;
        let mut acc = Genome {
            speed: 0.0, sense: 0.0, size: 0.0, metabolism: 0.0, lifespan: 0.0,
            hue: 0.0, bold: 0.0, curious: 0.0, social: 0.0,
            help_thr: None, pref_sig: None, tol_sig: None,
            fin_color: 0.0, deco_efficiency: 0.0,
            hull_ratio: 0.0, limb_scale: 0.0,
            ext: Vec::new(),
        };
        let mut opt_sum = [0.0f64; 3];
        let mut opt_cnt = [0usize; 3];
        for a in list {
            let g = &a.genome;
            acc.speed += g.speed;
            acc.sense += g.sense;
            acc.size += g.size;
            acc.metabolism += g.metabolism;
            acc.lifespan += g.lifespan;
            acc.hue += g.hue;
            acc.bold += g.bold;
            acc.curious += g.curious;
            acc.social += g.social;
            acc.fin_color += g.fin_color;
            acc.deco_efficiency += g.deco_efficiency;
            acc.hull_ratio += g.hull_ratio;
            acc.limb_scale += g.limb_scale;
            for (k, v) in [g.help_thr, g.pref_sig, g.tol_sig].into_iter().enumerate() {
                if let Some(x) = v { opt_sum[k] += x; opt_cnt[k] += 1; }
            }
        }
        acc.speed /= n;
        acc.sense /= n;
        acc.size /= n;
        acc.metabolism /= n;
        acc.lifespan /= n;
        acc.hue /= n;
        acc.bold /= n;
        acc.curious /= n;
        acc.social /= n;
        acc.fin_color /= n;
        acc.deco_efficiency /= n;
        acc.hull_ratio /= n;
        acc.limb_scale /= n;
        acc.help_thr = (opt_cnt[0] > 0).then(|| opt_sum[0] / opt_cnt[0] as f64);
        acc.pref_sig = (opt_cnt[1] > 0).then(|| opt_sum[1] / opt_cnt[1] as f64);
        acc.tol_sig = (opt_cnt[2] > 0).then(|| opt_sum[2] / opt_cnt[2] as f64);
        acc
    }

    /// Prim's MST over the complete graph with 1-D |Δdeco_efficiency| weights.
    /// Returns n−1 edges (weight, from, to) with vertex = position in `eff`.
    /// Deterministic: the lowest-index frontier vertex wins distance ties.
    fn mst_edges_1d(eff: &[f64]) -> Vec<(f64, usize, usize)> {
        let n = eff.len();
        if n < 2 { return Vec::new(); }
        let mut in_tree = vec![false; n];
        let mut best_w = vec![f64::INFINITY; n];
        let mut best_from = vec![usize::MAX; n];
        in_tree[0] = true;
        for k in 1..n {
            best_w[k] = (eff[k] - eff[0]).abs();
            best_from[k] = 0;
        }
        let mut edges = Vec::with_capacity(n - 1);
        for _ in 1..n {
            let mut pick = usize::MAX;
            let mut pick_w = f64::INFINITY;
            for k in 0..n {
                if !in_tree[k] && best_w[k] < pick_w {
                    pick_w = best_w[k];
                    pick = k;
                }
            }
            if pick == usize::MAX { break; }
            in_tree[pick] = true;
            edges.push((pick_w, best_from[pick], pick));
            for k in 0..n {
                if !in_tree[k] {
                    let d = (eff[k] - eff[pick]).abs();
                    if d < best_w[k] { best_w[k] = d; best_from[k] = pick; }
                }
            }
        }
        edges
    }

    /// Minimum Euclidean distance (world units) between two agent groups.
    fn min_cross_dist(agents: &[Agent], deco_idx: &[usize], ga: &[usize], gb: &[usize]) -> f64 {
        let mut best = f64::INFINITY;
        for &p in ga {
            let a = &agents[deco_idx[p]];
            for &q in gb {
                let b = &agents[deco_idx[q]];
                let d = (a.x - b.x).hypot(a.z - b.z);
                if d < best { best = d; }
            }
        }
        best
    }

    /// Phase 15 speciation detector; fires every SPECIATION_CHECK_TICKS from
    /// sample(). Consumes no RNG draws, so checksums are unaffected.
    /// `#[inline(never)]`: inlining this (plus detect_splits) into the hot
    /// sample() monomorphizes multi-KB of engine into one wasm function and
    /// starves LLVM's shared-code passes — measured +11 KB on the binary.
    #[inline(never)]
    fn check_speciation(&mut self) {
        let now = self.tick_count;
        let deco_idx: Vec<usize> = self.agents.iter().enumerate()
            .filter(|(_, a)| a.alive && a.kind == DECO)
            .map(|(i, _)| i)
            .collect();

        // (a) rebuild cached member lists + extinction bookkeeping.
        for clade in self.species_clades.iter_mut() { clade.members.clear(); }
        let known: Vec<u32> = self.species_clades.iter().map(|c| c.id).collect();
        for &i in &deco_idx {
            let cid = self.agents[i].clade_id;
            if let Some(pos) = known.iter().position(|&k| k == cid) {
                self.species_clades[pos].members.push(self.agents[i].id);
            }
        }
        // Phase 16: a clade whose member count hits zero freezes its record
        // AND logs an extinction event in the unified log. kind 2 = the clade
        // carried a competitive burden in the previous pass (Gause exclusion),
        // kind 1 = plain death (resource loss / old age).
        let mut extinct: Vec<(u32, u8)> = Vec::new();
        for clade in self.species_clades.iter_mut() {
            if clade.members.is_empty() && clade.death_tick.is_none() {
                clade.death_tick = Some(now);
                let kind = if self.clade_burdens.iter().any(|(id, b)| *id == clade.id && *b > 0.0) {
                    EVENT_KIND_COMPETITIVE_EXCLUSION
                } else {
                    EVENT_KIND_EXTINCTION
                };
                extinct.push((clade.id, kind));
            }
        }
        for (id, kind) in extinct {
            self.speciation_events.push(CladeSpeciationEvent {
                tick: now,
                parent_clade_id: id,
                child_clade_ids: Vec::new(),
                genetic_distance: 0.0,
                event_kind: kind,
            });
        }
        if deco_idx.is_empty() { return; }

        // (b) lazy root clade. Id 0 doubles as Agent.clade_id's default, so
        // fresh sims and legacy (pre-Phase-15) saves land here on first pass.
        let has_root = self.species_clades.iter().any(|c| c.kind == self.kind_name(DECO) && c.parent_id.is_none());
        if !has_root {
            let refs: Vec<&Agent> = deco_idx.iter().map(|&i| &self.agents[i]).collect();
            let centroid = Self::mean_genome(&refs);
            let id = self.next_clade_id;
            self.next_clade_id += 1;
            self.species_clades.push(SpeciesClade {
                id,
                parent_id: None,
                kind: self.kind_name(DECO),
                name: self.kind_name(DECO).to_string(),
                birth_tick: 0,
                death_tick: None,
                centroid_genome: centroid,
                members: deco_idx.iter().map(|&i| self.agents[i].id).collect(),
                niche_offset: 0.0,
            });
        }
        // Re-point DECOs whose clade id no longer exists to the root.
        let root_id = self.species_clades.iter()
            .find(|c| c.kind == self.kind_name(DECO) && c.parent_id.is_none()).unwrap().id;
        for &i in &deco_idx {
            let cid = self.agents[i].clade_id;
            if cid != root_id && !self.species_clades.iter().any(|c| c.id == cid) {
                self.agents[i].clade_id = root_id;
            }
        }

        // (c) split detection behind the time gate.
        if deco_idx.len() >= 2 && now.saturating_sub(self.last_clade_spec_tick) >= MIN_SPEC_TIME_TICKS {
            self.detect_splits(now, &deco_idx);
        }

        // (d) refresh cached members + centroids from live agents.
        self.refresh_clade_stats(&deco_idx);

        // (e) Phase 16: niche re-derivation + competitive-exclusion burdens
        // (impl in niche.rs; pure bookkeeping — consumes no RNG draws).
        self.update_niches_and_competition();
    }

    /// MST gap detection + clade split. `deco_idx` holds the agent slots of
    /// all alive DECOs; vertex positions below index into it. Deliberately
    /// HashMap/std-sort free: n ≤ MAX_DECO = 80, and every generic sort or
    /// hash-map instantiation would monomorphize multi-KB machinery into the
    /// wasm binary (insertion sorts over tiny vecs are both smaller and fast).
    #[inline(never)]
    fn detect_splits(&mut self, now: u32, deco_idx: &[usize]) {
        let n = deco_idx.len();
        let eff: Vec<f64> = deco_idx.iter().map(|&i| self.agents[i].genome.deco_efficiency).collect();
        let all_edges = Self::mst_edges_1d(&eff);
        let cut: Vec<(f64, usize, usize)> = all_edges.iter()
            .filter(|(w, _, _)| *w >= SPECIATION_DISTANCE_THRESHOLD)
            .copied()
            .collect();
        if cut.is_empty() { return; }

        // Forest of genomic clusters: union only the sub-threshold MST edges.
        // comp_root[p] = union-find root; roots[] dedupes them in first-seen
        // order so every later structure is a plain parallel Vec.
        let mut uf = Dsu::new(n);
        for &(w, a, b) in &all_edges {
            if w < SPECIATION_DISTANCE_THRESHOLD { uf.union(a, b); }
        }
        let comp_root: Vec<usize> = (0..n).map(|p| uf.find(p)).collect();
        let mut roots: Vec<usize> = Vec::new();
        for &r in &comp_root {
            if !roots.contains(&r) { roots.push(r); }
        }
        let mut comp_members: Vec<Vec<usize>> = vec![Vec::new(); roots.len()];
        for p in 0..n {
            let k = roots.iter().position(|&x| x == comp_root[p]).unwrap();
            comp_members[k].push(p);
        }
        // Majority clade per cluster (ties → lowest clade id).
        let mut comp_clade: Vec<u32> = Vec::with_capacity(roots.len());
        for members in &comp_members {
            let mut tally: Vec<(u32, usize)> = Vec::new();
            for &p in members {
                let cid = self.agents[deco_idx[p]].clade_id;
                match tally.iter_mut().find(|(c, _)| *c == cid) {
                    Some((_, cnt)) => *cnt += 1,
                    None => tally.push((cid, 1)),
                }
            }
            let mut best = (0u32, 0usize);
            for &(cid, cnt) in &tally {
                if cnt > best.1 || (cnt == best.1 && cid < best.0) { best = (cid, cnt); }
            }
            comp_clade.push(best.0);
        }

        // Heaviest cut first (MST edge order); a cluster buds off at most once
        // per pass while a parent cluster may emit several children in one pass.
        let mut used_as_child = vec![false; roots.len()];
        // (parent clade, min member id, member positions, cut weight)
        let mut splits: Vec<(u32, u32, Vec<usize>, f64)> = Vec::new();
        for &(w, a, b) in &cut {
            let ka = roots.iter().position(|&x| x == comp_root[a]).unwrap();
            let kb = roots.iter().position(|&x| x == comp_root[b]).unwrap();
            if used_as_child[ka] || used_as_child[kb] { continue; }
            if comp_clade[ka] != comp_clade[kb] { continue; }
            let (parent_side, child_side) = if comp_members[ka].len() <= comp_members[kb].len() {
                (kb, ka)
            } else {
                (ka, kb)
            };
            if comp_members[child_side].len() < SPECIATION_MIN_SIDE
                || comp_members[parent_side].len() < SPECIATION_MIN_SIDE {
                continue;
            }
            let dist = Self::min_cross_dist(
                &self.agents, deco_idx,
                &comp_members[parent_side], &comp_members[child_side],
            );
            if dist <= GEO_ISOLATION_DIST { continue; }
            used_as_child[child_side] = true;
            let min_id = comp_members[child_side].iter()
                .map(|&p| self.agents[deco_idx[p]].id).min().unwrap();
            splits.push((comp_clade[parent_side], min_id, comp_members[child_side].clone(), w));
        }
        if splits.is_empty() { return; }

        // Deterministic order: parent clade id, then smallest member agent id.
        insertion_sort_by(splits.as_mut_slice(), |a, b| (a.0, a.1).cmp(&(b.0, b.1)));
        let mut last_parent: Option<u32> = None;
        let mut event: Option<CladeSpeciationEvent> = None;
        for (parent, _min_id, members, w) in splits {
            if last_parent != Some(parent) {
                if let Some(ev) = event.take() {
                    self.speciation_events.push(ev);
                }
                event = Some(CladeSpeciationEvent {
                    tick: now,
                    parent_clade_id: parent,
                    child_clade_ids: Vec::new(),
                    genetic_distance: 0.0,
                    event_kind: EVENT_KIND_SPECIATION,
                });
                last_parent = Some(parent);
            }
            let kind = self.species_clades.iter().find(|c| c.id == parent)
                .map(|c| c.kind).unwrap_or(self.kind_name(DECO));
            let cid = self.next_clade_id;
            self.next_clade_id += 1;
            let refs: Vec<&Agent> = members.iter().map(|&p| &self.agents[deco_idx[p]]).collect();
            let centroid = Self::mean_genome(&refs);
            // Phase 16: birth-time niche = parent niche + centroid eff gap
            // (update_niches_and_competition re-derives it on the same pass;
            // this initial value keeps the record self-consistent).
            let (parent_niche, parent_eff) = self.species_clades.iter()
                .find(|c| c.id == parent)
                .map(|p| (p.niche_offset, p.centroid_genome.deco_efficiency))
                .unwrap_or((0.0, centroid.deco_efficiency));
            let niche0 = (parent_niche
                + (centroid.deco_efficiency - parent_eff) * NICHE_EFF_SCALE)
                .clamp(-1.0, 1.0);
            for &p in members.iter() {
                self.agents[deco_idx[p]].clade_id = cid;
            }
            self.species_clades.push(SpeciesClade {
                id: cid,
                parent_id: Some(parent),
                kind,
                name: format!("{}-{}", kind, clade_letter(cid)),
                birth_tick: now,
                death_tick: None,
                centroid_genome: centroid,
                members: members.iter().map(|&p| self.agents[deco_idx[p]].id).collect(),
                niche_offset: niche0,
            });
            let ev = event.as_mut().unwrap();
            ev.child_clade_ids.push(cid);
            if w > ev.genetic_distance { ev.genetic_distance = w; }
        }
        if let Some(ev) = event.take() {
            self.speciation_events.push(ev);
        }
        self.last_clade_spec_tick = now;
    }

    /// Rebuild `members` from live agents and recompute centroid genomes for
    /// living clades (extinct clades keep their last frozen centroid).
    #[inline(never)]
    fn refresh_clade_stats(&mut self, deco_idx: &[usize]) {
        for clade in self.species_clades.iter_mut() { clade.members.clear(); }
        for &i in deco_idx {
            let cid = self.agents[i].clade_id;
            if let Some(pos) = self.species_clades.iter().position(|c| c.id == cid) {
                self.species_clades[pos].members.push(self.agents[i].id);
            }
        }
        let alive: Vec<&Agent> = deco_idx.iter().map(|&i| &self.agents[i]).collect();
        let mut centroids: Vec<(u32, Genome)> = Vec::with_capacity(self.species_clades.len());
        for clade in self.species_clades.iter() {
            if clade.members.is_empty() {
                centroids.push((clade.id, clade.centroid_genome.clone()));
                continue;
            }
            let refs: Vec<&Agent> = alive.iter().copied()
                .filter(|a| clade.members.contains(&a.id))
                .collect();
            centroids.push((clade.id, Self::mean_genome(&refs)));
        }
        for (cid, g) in centroids {
            if let Some(c) = self.species_clades.iter_mut().find(|c| c.id == cid) {
                c.centroid_genome = g;
            }
        }
    }

    /// Phylogeny readouts for the C-ABI layer.
    pub fn species_clades(&self) -> &[SpeciesClade] { &self.species_clades }
    pub fn clade_events(&self) -> &[CladeSpeciationEvent] { &self.speciation_events }

    pub fn stats(&self) -> Stats {
        let last = self.history.last().cloned().unwrap_or(HistorySample {
            t: 0.0, season: 0, h: 0, p: 0, scav: 0, omn: 0,
            plant: 0.0, plant_by: vec![0.0; self.plants.len()],
            speed_h: 0.0, sense_h: 0.0, size_h: 0.0,
            speed_p: 0.0, sense_p: 0.0, gap_hp: 0.0,
            coop_share_h: 0.0, help_thr_h: 0.0,
            sig_mean_h: 0.0, sig_var_h: 0.0, pref_mean_h: 0.0, tol_mean_h: 0.0,
            gen: 0,
        });
        let first = self.history.first().cloned().unwrap_or(last.clone());
        let mut cur_fit_h = 0.0f64;
        let mut n_fit_h = 0usize;
        let mut cur_fit_p = 0.0f64;
        let mut n_fit_p = 0usize;
        let mut cargo_total = 0.0f64;
        for a in &self.agents {
            if !a.alive { continue; }
            let off = a.offspring;
            let fit = (a.age / a.genome.lifespan) * (1.0 + off as f64) / a.genome.metabolism;
            match a.kind {
                HERB => { cur_fit_h += fit; n_fit_h += 1; }
                PRED => { cur_fit_p += fit; n_fit_p += 1; }
                _ => {}
            }
            if !a.cargo.is_empty() {
                for &c in &a.cargo { cargo_total += c as f64; }
            }
        }
        let fruit_e = self.fruit_e_h;
        let leaf_e = self.leaf_e_h;
        let st = &self.deposit_stats;
        let deposit_summary = if st.count > 0 {
            let mut samples = st.samples.clone();
            samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let q = |p: f64| -> f64 {
                if samples.is_empty() { return 0.0; }
                let idx = (p * samples.len() as f64).min((samples.len() - 1) as f64) as usize;
                (samples[idx] * 10.0).round() / 10.0
            };
            Some(DepositSummary {
                count: st.count,
                mean_dist_cells: (st.dist_sum / st.count as f64 * 10.0).round() / 10.0,
                p50: q(0.5), p90: q(0.9),
                far_5: st.far_5, far_20: st.far_20,
                colonize: st.colonize,
            })
        } else { None };
        let winter_coop = {
            let w_samples = self.history.iter().filter(|s| s.season == 3).collect::<Vec<_>>();
            if w_samples.is_empty() { 0.0 }
            else { w_samples.iter().map(|s| s.coop_share_h).sum::<f64>() / w_samples.len() as f64 }
        };
        Stats {
            time: self.time, season: self.season,
            h: last.h, p: last.p, scav: last.scav, omn: last.omn,
            plant: last.plant, plant_by: last.plant_by,
            speed_h: last.speed_h, sense_h: last.sense_h, size_h: last.size_h,
            speed_p: last.speed_p, sense_p: last.sense_p,
            gap_hp: last.gap_hp, coop_share_h: last.coop_share_h,
            help_thr_h: last.help_thr_h,
            sig_mean_h: last.sig_mean_h, sig_var_h: last.sig_var_h,
            pref_mean_h: last.pref_mean_h, tol_mean_h: last.tol_mean_h,
            gen: last.gen,
            births: self.births, deaths: self.deaths, kills: self.kills,
            starved: self.starved, aged: self.aged,
            trend_h: if first.speed_h != 0.0 { last.speed_h - first.speed_h } else { 0.0 },
            trend_p: if first.speed_p != 0.0 { last.speed_p - first.speed_p } else { 0.0 },
            selection_pressure: self.selection_pressure.clone(),
            cur_fit_h: if n_fit_h > 0 { cur_fit_h / n_fit_h as f64 } else { 0.0 },
            cur_fit_p: if n_fit_p > 0 { cur_fit_p / n_fit_p as f64 } else { 0.0 },
            ev_hist_h: self.evolution[0].clone(),
            ev_hist_p: self.evolution[1].clone(),
            ev_hist_scav: self.evolution[2].clone(),
            ev_hist_omn: self.evolution[3].clone(),
            ev_hist_fish: self.evolution[4].clone(),
            ev_hist_deco: self.evolution[5].clone(),
            deco_count: self.count(DECO),
            speciation_event: self.speciation_event.clone(),
            gene_barrier: self.gene_barrier_strength,
            barrier_blocks: self.barrier_blocks,
            isolated_groups: self.isolated_groups.iter().map(|(name, g)| {
                let cnt = self.agents.iter().filter(|a| a.alive && a.lineage == Some(*name)).count();
                IsolatedGroupStat { name: g.code.clone(), kind: g.kind, since: g.since, count: cnt }
            }).collect(),
            isolation_history: self.isolation_history.clone(),
            grass_species: self.plants.clone(),
            cargo_total: (cargo_total * 10000.0).round() / 10000.0,
            fruit_energy_share_h: if fruit_e + leaf_e > 0.0 { (fruit_e / (fruit_e + leaf_e) * 10000.0).round() / 10000.0 } else { 0.0 },
            deposit: deposit_summary,
            help_count: self.help_count,
            reject_count: self.reject_count,
            help_cost_share_h: if self.help_cost_e_h + self.drain_e_h > 0.0 {
                (self.help_cost_e_h / (self.help_cost_e_h + self.drain_e_h) * 10000.0).round() / 10000.0
            } else { 0.0 },
            winter_coop_share_h: winter_coop,
            sig_pair_fail_rate: if self.sig_attempts > 0 { (self.sig_blocks as f64 / self.sig_attempts as f64 * 10000.0).round() / 10000.0 } else { 0.0 },
            sig_pair_attempts: self.sig_attempts,
            sig_pair_fails: self.sig_blocks,
            // Phase 7: instantaneous DECO efficiency distribution.
            deco_eff_mean: {
                let de: Vec<f64> = self.agents.iter()
                    .filter(|a| a.alive && a.kind == DECO)
                    .map(|a| a.genome.deco_efficiency).collect();
                if de.is_empty() { 0.0 } else { de.iter().sum::<f64>() / de.len() as f64 }
            },
            deco_eff_std: {
                let de: Vec<f64> = self.agents.iter()
                    .filter(|a| a.alive && a.kind == DECO)
                    .map(|a| a.genome.deco_efficiency).collect();
                if de.len() < 2 { 0.0 } else {
                    let m = de.iter().sum::<f64>() / de.len() as f64;
                    (de.iter().map(|v| (v - m).powi(2)).sum::<f64>() / de.len() as f64).sqrt()
                }
            },
        }
    }

    /// v1 golden-checksum contract: mixes the kind NAME's first byte (the
    /// value every legacy anchor, including `0x790184996f3acefd`, was pinned
    /// against). Delegates to `checksum_with` with the v1 kind field.
    pub fn checksum(&self) -> String {
        self.checksum_with(|a| self.kind_name(a.kind).as_bytes()[0] as u64)
    }

    /// Phase 31 (Q2 ruling: 31 parallel export, 34 switch): identical mix
    /// stream to v1 except the kind field is the registry id (u32) instead of
    /// the name's first byte. Six legacy ids (0-5) never collide with the
    /// first bytes (102/104/111/112/115/100), so v1 and v2 differ by
    /// construction and each is deterministic on its own.
    pub fn checksum_v2(&self) -> String {
        self.checksum_with(|a| a.kind as u32 as u64)
    }

    /// Shared mix stream. `kind_field` is the only divergence point between
    /// the v1 (name byte) and v2 (registry id) checksums — keeping it a
    /// single source prevents the two streams from drifting.
    fn checksum_with(&self, kind_field: impl Fn(&Agent) -> u64) -> String {
        const MOD: u64 = 0x0000_0100_0000_01B3; // FNV-1a 64 prime 0x100000001B3
        let mut h: u128 = 14695981039346656033;
        let mut mix = |val: u64| {
            // JS: h ^= BigInt(v) & 0xffffffffn — only the LOW 32 BITS participate in XOR!
            h ^= (val & 0xFFFFFFFF) as u128;
            h = (h * MOD as u128) & 0xFFFFFFFF_FFFFFFFF;
        };
        for a in &self.agents {
            if !a.alive { continue; }
            mix(kind_field(a));
            mix(a.id as u64);
            // negative floors (x/z) must wrap like BigInt & 0xffffffff, not saturate
            mix((a.x * 100.0).floor() as i64 as u64);
            mix((a.z * 100.0).floor() as i64 as u64);
            mix((a.energy * 100.0).floor() as i64 as u64);
            mix((a.age * 10.0).floor() as i64 as u64);
        }
        for i in (0..self.grass.len()).step_by(7) {
            mix((self.grass[i] as f64 * 1000.0).floor() as i64 as u64);
        }
        mix((self.time * 10.0).floor() as i64 as u64);
        mix(self.season as u64);
        mix(self.rain.floor() as u64);
        mix(self.births as u64);
        mix(self.deaths as u64);
        mix(self.kills as u64);
        mix(self.speciation_fired as u64);
        format!("0x{:016x}", h as u64)
    }

    pub fn living(&self) -> &[Agent] { &self.agents }

    // Accessors for external bit-level grass diagnostics.
    pub fn grass(&self) -> &[f32] { &self.grass }
    pub fn grass_p(&self, sp: usize) -> &[f32] { &self.grass_p[sp] }
    pub fn suit(&self) -> &[f32] { &self.suit }
    pub(crate) fn suit_mut(&mut self) -> &mut [f32] { &mut self.suit }
    pub fn moist(&self) -> &[f32] { &self.moist }
    pub(crate) fn moist_mut(&mut self) -> &mut [f32] { &mut self.moist }
    pub fn shade(&self) -> &[f32] { &self.shade }
    pub fn suit_eff_by(&self, sp: usize) -> &[f32] { &self.suit_eff_by[sp] }
    pub(crate) fn suit_eff_by_mut(&mut self, sp: usize) -> &mut [f32] { &mut self.suit_eff_by[sp] }
    pub fn mineral_field(&self) -> &[f32] { &self.mineral_field }
    /// Scenario helper (Phase 4-C test baseline): marks every living agent of
    /// `kind` dead so the species stays extinct for the rest of the run —
    /// used to build a DECO-absent control group in A/B growth comparisons.
    pub fn kill_kind(&mut self, kind: SpeciesKind) {
        for a in &mut self.agents {
            if a.alive && a.kind == kind { a.alive = false; }
        }
        self.agents.retain(|a| a.alive);
    }
    /// Scenario helper (Phase 4-C): directly sets a mineral-field cell. Used
    /// by the A/B grass-growth micro-test to isolate the deposit→growth leg
    /// of the decomposer loop from agent-dynamics noise.
    pub fn inject_mineral(&mut self, cell: usize, value: f32) {
        if cell < self.mineral_field.len() {
            self.mineral_field[cell] = value.clamp(0.0, 1.0);
        }
    }
    pub fn tick_count(&self) -> u32 { self.tick_count }

    /// Phase 32: enable/disable the open-locus gene-duplication operator.
    /// `false` (default) keeps the pre-Phase-32 bit-exact trajectory (zero
    /// extra RNG draws, `ext` always empty). `true` adds the single fixed
    /// `dup_roll` draw per mutate call so new heritable slots can appear,
    /// inherit, and pay an expression upkeep. Takes effect on the NEXT
    /// `mutate` call (i.e. at the next birth / asexual split).
    pub fn set_genesis(&mut self, on: bool) {
        self.genesis_on = on;
    }

    /// Phase 32: total number of ext loci across all alive agents (0 when
    /// genesis is off). Observability for the OEE emergence + upkeep-cap
    /// acceptance tests; not part of any checksum.
    pub fn ext_total_loci(&self) -> u32 {
        self.agents.iter().filter(|a| a.alive)
            .map(|a| a.genome.ext.len() as u32)
            .sum()
    }

    /// Phase 32: fraction of alive agents carrying at least one ext locus
    /// (0.0 when genesis is off). This is the OEE condition ① readout —
    /// a nonzero value proves new heritable traits have appeared.
    pub fn ext_bearer_share(&self) -> f64 {
        let mut total = 0u32;
        let mut bearers = 0u32;
        for a in self.agents.iter() {
            if !a.alive { continue; }
            total += 1;
            if !a.genome.ext.is_empty() { bearers += 1; }
        }
        if total == 0 { 0.0 } else { bearers as f64 / total as f64 }
    }

    /// Phase 32: largest `ext` length observed among alive agents right now
    /// (0 when genesis is off). The upkeep-cap test samples this over a long
    /// run and records the typical maximum.
    pub fn ext_max_len(&self) -> usize {
        self.agents.iter().filter(|a| a.alive)
            .map(|a| a.genome.ext.len())
            .max()
            .unwrap_or(0)
    }
}

// ----- Phase 6: save/restore (aether-sim-save-state v1) -----------------------
//
// The render-packet Snapshot only carries render views; restoring from it
// would desync (RNG streams, carcasses, next_id, deposit stats are missing).
// These two functions capture every field `update` reads or mutates so a
// restored sim continues with identical checksums.

fn genome_to_save(g: &Genome) -> save_state::SaveGenome {
    save_state::SaveGenome {
        speed: g.speed, sense: g.sense, size: g.size,
        metabolism: g.metabolism, lifespan: g.lifespan, hue: g.hue,
        bold: g.bold, curious: g.curious, social: g.social,
        help_thr: g.help_thr, pref_sig: g.pref_sig, tol_sig: g.tol_sig,
        fin_color: g.fin_color, deco_efficiency: g.deco_efficiency,
        hull_ratio: g.hull_ratio, limb_scale: g.limb_scale,
    }
}

fn genome_from_save(g: save_state::SaveGenome) -> Genome {
    Genome {
        speed: g.speed, sense: g.sense, size: g.size,
        metabolism: g.metabolism, lifespan: g.lifespan, hue: g.hue,
        bold: g.bold, curious: g.curious, social: g.social,
        help_thr: g.help_thr, pref_sig: g.pref_sig, tol_sig: g.tol_sig,
        fin_color: g.fin_color, deco_efficiency: g.deco_efficiency,
        hull_ratio: g.hull_ratio, limb_scale: g.limb_scale,
        ext: Vec::new(),
    }
}

// Phase 32: open-locus slot <-> save-state roundtrip (lossless; f64 scalars
// survive the serde_json shortest-repr hop unchanged).
fn locus_to_save(l: &Locus) -> save_state::SaveLocus {
    save_state::SaveLocus {
        id: l.id, value: l.value, sigma: l.sigma,
        clamp_min: l.clamp_min, clamp_max: l.clamp_max,
    }
}

fn locus_from_save(l: save_state::SaveLocus) -> Locus {
    Locus {
        id: l.id, value: l.value, sigma: l.sigma,
        clamp_min: l.clamp_min, clamp_max: l.clamp_max,
    }
}

fn agent_to_save(a: &Agent, kinds: &KindRegistry) -> save_state::SaveAgent {
    let g = &a.genome;
    save_state::SaveAgent {
        id: a.id,
        kind: kinds.name(a.kind).to_string(),
        x: a.x, z: a.z, y: a.y,
        prev_x: a.prev_x, prev_z: a.prev_z, prev_heading: a.prev_heading,
        heading: a.heading, speed: a.speed, desired_speed: a.desired_speed,
        energy: a.energy, max_energy: a.max_energy, age: a.age,
        genome: genome_to_save(g),
        generation: a.generation,
        state: a.state.to_string(),
        bite: a.bite, repro: a.repro,
        target_x: a.target_x, target_z: a.target_z, target_id: a.target_id,
        think: a.think, phase: a.phase,
        alive: a.alive,
        activity: a.activity, hydration: a.hydration,
        cargo: a.cargo.clone(),
        trail_t: a.trail_t.clone(), trail_x: a.trail_x.clone(), trail_z: a.trail_z.clone(),
        trail_head: a.trail_head, trail_n: a.trail_n, trail_slot: a.trail_slot,
        fatigue: a.fatigue, fear: a.fear, image: a.image,
        help_cd: a.help_cd, curiosity: a.curiosity,
        home_x: a.home_x, home_z: a.home_z, range: a.range,
        lineage: a.lineage.map(|s| s.to_string()),
        clade_id: a.clade_id,
        offspring: a.offspring,
        ext_loci: a.genome.ext.iter().map(locus_to_save).collect(),
    }
}

fn evo_to_save(e: &EvolutionSample) -> save_state::SaveEvolutionSample {
    save_state::SaveEvolutionSample {
        t: e.t, gen: e.gen, count: e.count, speed: e.speed, sense: e.sense,
        size: e.size, metabolism: e.metabolism, bold: e.bold, curious: e.curious,
        social: e.social, help_thr: e.help_thr,
        DecoEffMean: e.deco_eff, DecoEffStd: e.deco_eff_std,
    }
}

fn hist_to_save(h: &HistorySample) -> save_state::SaveHistorySample {
    save_state::SaveHistorySample {
        t: h.t, season: h.season, h: h.h, p: h.p, scav: h.scav, omn: h.omn,
        plant: h.plant, plant_by: h.plant_by.clone(),
        speed_h: h.speed_h, sense_h: h.sense_h, size_h: h.size_h,
        speed_p: h.speed_p, sense_p: h.sense_p, gap_hp: h.gap_hp,
        coop_share_h: h.coop_share_h, help_thr_h: h.help_thr_h,
        sig_mean_h: h.sig_mean_h, sig_var_h: h.sig_var_h,
        pref_mean_h: h.pref_mean_h, tol_mean_h: h.tol_mean_h, gen: h.gen,
    }
}

fn spec_event_to_save(e: &SpeciationEvent) -> save_state::SaveSpeciationEvent {
    save_state::SaveSpeciationEvent {
        t: e.t,
        parent_kind: e.parent_kind.to_string(),
        lineages: e.lineages.iter().map(|l| save_state::SaveLineageInfo {
            id: l.id.to_string(), name: l.name.to_string(),
            avg_speed: l.avg_speed, count: l.count,
        }).collect(),
        cv_speed: e.cv_speed, gap_idx: e.gap_idx, name_suffix: e.name_suffix.clone(),
    }
}

/// Compact camelCase JSON view of a SpeciesClade for the save-state string
/// fields (mirrored by from_save_state's JsonClade and the JS phylogeny
/// reader). Only centroid_eff survives the roundtrip of the centroid genome;
/// the remaining fields are recomputed from live members on the next pass.
/// Hand-rolled via format!: pulling in serde_json::Value/json! for these two
/// records measurably inflated the wasm binary (Value machinery ≈ tens of KB).
fn clade_to_json(c: &SpeciesClade) -> String {
    let parent = match c.parent_id {
        Some(p) => p.to_string(),
        None => "null".to_string(),
    };
    let death = match c.death_tick {
        Some(t) => t.to_string(),
        None => "null".to_string(),
    };
    format!(
        "{{\"id\":{},\"parentId\":{},\"kind\":\"{}\",\"name\":\"{}\",\"birthTick\":{},\"deathTick\":{},\"centroidEff\":{},\"aliveCount\":{},\"niche\":{}}}",
        c.id, parent, c.kind, c.name, c.birth_tick, death,
        c.centroid_genome.deco_efficiency, c.members.len(), c.niche_offset,
    )
}

/// Compact camelCase JSON view of a CladeSpeciationEvent (same contract as
/// clade_to_json).
fn clade_event_to_json(e: &CladeSpeciationEvent) -> String {
    let children = e.child_clade_ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(",");
    format!(
        "{{\"tick\":{},\"parentCladeId\":{},\"childCladeIds\":[{}],\"geneticDistance\":{},\"kind\":{}}}",
        e.tick, e.parent_clade_id, children, e.genetic_distance, e.event_kind,
    )
}

/// Compact camelCase JSON view of a Cataclysm for the SaveState v4 string
/// field (same hand-rolled format! trick as clade_to_json — no extra serde
/// visitors in the wasm binary).
fn cataclysm_to_json(c: &Cataclysm) -> String {
    format!(
        "{{\"id\":{},\"kind\":{},\"tick\":{},\"x\":{},\"z\":{},\"radius\":{},\"severity\":{},\"dustRecoveryTicks\":{},\"climateDelta\":{},\"endsAt\":{},\"remaining\":{},\"active\":{}}}",
        c.id, c.kind, c.tick, c.x, c.z, c.radius, c.severity,
        c.dust_recovery_ticks, c.climate_delta, c.ends_at, c.remaining,
        if c.active { 1 } else { 0 },
    )
}

impl Ecology {
    /// Phase 8 helper: acquire an exclusive occupancy lock on a carcass.
    /// An agent holds at most one lock at a time; if it already holds another
    /// carcass, that prior lock is dropped first (transitioning to a new
    /// target mid-session). Callers must verify `carcasses[j]` still exists.
    fn acquire_carcass_lock(&mut self, j: usize, owner_id: u32) {
        if !self.carcass_lock_enabled { return; }
        // Drop any previous lock held by this agent.
        for (k, c) in self.carcasses.iter_mut().enumerate() {
            if k != j && c.occupied_by == Some(owner_id) { c.occupied_by = None; }
        }
        self.carcasses[j].occupied_by = Some(owner_id);
    }

    /// Phase 8 helper — returns true when `agent_id` may eat from `c`:
    /// either locking is disabled, or the carcass is unowned, or it's already owned by us.
    #[inline]
    fn carcass_claimable(&self, c: &Carcass, agent_id: u32) -> bool {
        !self.carcass_lock_enabled || c.occupied_by.is_none() || c.occupied_by == Some(agent_id)
    }

    /// Serialize the complete internal state (save_state v1 schema).
    pub fn to_save_state(&self) -> save_state::SaveState {
        save_state::SaveState {
            version: save_state::SAVE_STATE_VERSION,
            schema_name: save_state::SAVE_STATE_SCHEMA_NAME.to_string(),
            engine_version: env!("CARGO_PKG_VERSION").to_string(),
            seed: self.seed,
            tick_count: self.tick_count,
            time: self.time,
            season: self.season,
            daylight: self.daylight,
            rain: self.rain,
            fires: self.fires.clone(),
            torch: self.torch,
            rng_state: save_state::RngSnapshot { state: self.rng.state() },
            prng_state: save_state::RngSnapshot { state: self.prng.state() },
            fish_rng_state: save_state::RngSnapshot { state: self.fish_rng.state() },
            next_id: self.next_id,
            agents: self.agents.iter().map(|a| agent_to_save(a, &self.kinds)).collect(),
            carcasses: self.carcasses.iter().map(|c| save_state::SaveCarcass {
                x: c.x, z: c.z, energy: c.energy, age: c.age,
                occupied_by: c.occupied_by,
            }).collect(),
            grass: self.grass.clone(),
            grass_by_species: self.grass_p.iter().cloned().collect(),
            suit_eff_by_species: self.suit_eff_by.iter().cloned().collect(),
            // Phase 33: the registered plant list (default = the three
            // legacy entries). Sized to drive the per-plant layers on restore.
            plants: self.plants.iter().map(|p| save_state::SavePlant {
                r_max: p.r_max, moist_opt: p.moist_opt, moist_w: p.moist_w,
                palatability: p.palatability, fruiting: p.fruiting,
            }).collect(),
            suit: self.suit.clone(),
            shade: self.shade.clone(),
            moist: self.moist.clone(),
            mineral_field: self.mineral_field.clone(),
            grid_acc: self.grid_acc,
            births: self.births, deaths: self.deaths, kills: self.kills,
            starved: self.starved, aged: self.aged,
            barrier_blocks: self.barrier_blocks,
            sig_attempts: self.sig_attempts, sig_blocks: self.sig_blocks,
            deposit_stats: save_state::SaveDepositStats {
                count: self.deposit_stats.count,
                dist_sum: self.deposit_stats.dist_sum,
                far_5: self.deposit_stats.far_5,
                far_20: self.deposit_stats.far_20,
                colonize: self.deposit_stats.colonize,
                samples: self.deposit_stats.samples.clone(),
                sample_cursor: self.deposit_stats.sample_cursor,
            },
            fruit_e_h: self.fruit_e_h, leaf_e_h: self.leaf_e_h,
            help_count: self.help_count, reject_count: self.reject_count,
            help_cost_e_h: self.help_cost_e_h, drain_e_h: self.drain_e_h,
            speciation_fired: self.speciation_fired,
            gene_barrier_strength: self.gene_barrier_strength,
            // In `isolated_order` sequence: the sim iterates clusters in
            // first-registration order, so restore must rebuild it identically.
            isolated_groups: self.isolated_order.iter().filter_map(|name| {
                self.isolated_groups.get(*name).map(|g| save_state::SaveIsolatedGroup {
                    name: name.to_string(), code: g.code.clone(),
                    kind: g.kind.to_string(), since: g.since,
                })
            }).collect(),
            isolation_history: self.isolation_history.iter().map(|p| {
                save_state::SaveIsolationPoint { t: p.t, dist: p.dist, pairs: p.pairs }
            }).collect(),
            evolution: self.evolution.iter()
                .map(|hist| hist.iter().map(evo_to_save).collect())
                .collect(),
            generation_fitness: self.generation_fitness.iter()
                .map(|recs| recs.iter().map(|r| save_state::SaveFitnessRecord {
                    gen: r.gen, age: r.age, offspring: r.offspring, fitness: r.fitness,
                }).collect())
                .collect(),
            history: self.history.iter().map(hist_to_save).collect(),
            last_sample: self.last_sample,
            taken_buf: self.taken_buf.clone(),
            spread_buf: self.spread_buf.clone(),
            h_cache: self.h_cache,
            season_meta: self.season_meta,
            player_x: self.player_x,
            player_z: self.player_z,
            selection_pressure: save_state::SaveSelectionPressure {
                plant: self.selection_pressure.plant,
                pred_ratio: self.selection_pressure.pred_ratio,
                crowding: self.selection_pressure.crowding,
            },
            speciation_event: self.speciation_event.as_ref().map(spec_event_to_save),
            last_speciation: self.last_speciation,
            // Compact JSON-in-string fields (see save_state::SavedClades).
            SpeciesClades: save_state::SavedClades(format!(
                "[{}]",
                self.species_clades.iter().map(clade_to_json).collect::<Vec<String>>().join(",")
            )),
            NextCladeId: self.next_clade_id,
            SpeciationEvents: save_state::SavedEvents(format!(
                "[{}]",
                self.speciation_events.iter().map(clade_event_to_json).collect::<Vec<String>>().join(",")
            )),
            LastCladeSpecTick: self.last_clade_spec_tick,
            // Phase 16: burden map as compact JSON (see save_state::SavedBurdens).
            CladeNicheBurdens: save_state::SavedBurdens(format!(
                "[{}]",
                self.clade_burdens.iter()
                    .map(|(id, b)| format!("[{},{}]", id, b))
                    .collect::<Vec<String>>()
                    .join(",")
            )),
            // Phase 17: cataclysm history + live effect state (compact JSON,
            // see save_state::SavedCataclysms).
            Cataclysms: save_state::SavedCataclysms(format!(
                "[{}]",
                self.cataclysms.iter().map(cataclysm_to_json).collect::<Vec<String>>().join(",")
            )),
            NextCataclysmId: self.next_cataclysm_id,
            // None ⇔ empty registry (`Ecology::new` path); Some ⇔ registered
            // list in insertion order (`with_biomes` path incl. full-world +
            // aquatic). The two paths differ behaviorally (aquatic
            // home_range_mod 0.9), so the distinction must survive the roundtrip.
            biomes: {
                let list = self.biome_reg.biomes();
                if list.is_empty() {
                    None
                } else {
                    Some(list.iter().map(|b| save_state::SaveBiome {
                        id: b.id, name: b.name.to_string(),
                        center_x: b.center_x, center_z: b.center_z, radius: b.radius,
                        suit_modifier: b.suit_modifier,
                        max_herb: b.max_herb, max_pred: b.max_pred,
                        max_scav: b.max_scav, max_omn: b.max_omn,
                        max_fish: b.max_fish, home_range_mod: b.home_range_mod,
                    }).collect())
                }
            },
            DecoEffChoiceMode: self.deco_eff_choice_mode,
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
            // Phase 32: open-locus operator state (genesis toggle + dup
            // counter) rides the schema so a genesis-ON restore keeps the
            // duplication operator running (bit-exact continuation).
            GenesisOn: self.genesis_on,
            ExtDupEvents: self.ext_dup_events,
        }
    }

    /// Rebuild an Ecology from a save state. Constructs the struct directly —
    /// `init_grass`/`seed_life` are NOT re-run because both consume main-stream
    /// RNG draws (replaying them would shift every subsequent random event).
    /// Panics on schema mismatch or corrupted grid layers (length drift); the
    /// C-ABI wrapper maps JSON parse failures to a null handle instead.
    pub fn from_save_state(state: save_state::SaveState) -> Self {
        assert_eq!(
            state.schema_name, save_state::SAVE_STATE_SCHEMA_NAME,
            "not an {} document (schema_name = {})",
            save_state::SAVE_STATE_SCHEMA_NAME, state.schema_name
        );
        if state.version > save_state::SAVE_STATE_VERSION {
            panic!(
                "unsupported future save state version {} (current is {}); upgrade the sim first",
                state.version, save_state::SAVE_STATE_VERSION
            );
        }
        // Older versions (e.g. v1 → v2) are accepted; missing fields default via
        // serde #[serde(default)] and restore legacy Phase 6 behaviour (mode=0).
        let deco_eff_choice_mode = state.DecoEffChoiceMode;
        let carcass_lock_enabled = state.CarcaseLockEnabled;
        // Phase 31: resolve saved kind names to registry ids (v5 saves only
        // carry the six legacy names; unknown names get conservative defs).
        let mut kinds = KindRegistry::legacy();
        let kind_ids: Vec<SpeciesKind> = state.agents.iter()
            .map(|a| kinds.resolve_or_register(&a.kind)).collect();

        let n = state.grass.len();
        // Phase 33: restore the plant registry. Pre-v7 saves carry no list
        // (`#[serde(default)]` → empty), so they land on the three legacy
        // entries (bit-exact legacy continuation); the per-plant layers are
        // sized to this count.
        let plants: Vec<PlantType> = if state.plants.is_empty() {
            LEGACY_PLANTS.to_vec()
        } else {
            state.plants.iter().map(|p| PlantType {
                r_max: p.r_max, moist_opt: p.moist_opt, moist_w: p.moist_w,
                palatability: p.palatability, fruiting: p.fruiting,
            }).collect()
        };
        let pn = plants.len();
        assert_eq!(state.grass_by_species.len(), pn, "grass_by_species layer count");
        assert_eq!(state.suit_eff_by_species.len(), pn, "suit_eff_by_species layer count");
        for (name, v) in state.grass_by_species.iter().chain(state.suit_eff_by_species.iter())
            .enumerate()
            .map(|(i, v)| (i, v))
        {
            let _ = name;
            assert_eq!(v.len(), n, "grid layer length mismatch");
        }
        assert_eq!(state.suit.len(), n, "suit length");
        assert_eq!(state.shade.len(), n, "shade length");
        assert_eq!(state.moist.len(), n, "moist length");
        assert_eq!(state.mineral_field.len(), n, "mineral_field length");
        assert_eq!(state.evolution.len(), 6, "evolution history slot count");
        assert_eq!(state.generation_fitness.len(), 6, "generation_fitness slot count");

        let grass_p = state.grass_by_species;
        let suit_eff_by = state.suit_eff_by_species;
        // Phase 33: per-plant scratch buffer sized to the plant registry
        // (a valid save already carries the right length; resize guards a
        // shorter list so `take_grass` never indexes out of bounds).
        let mut taken_buf = state.taken_buf;
        taken_buf.resize(pn, 0.0);

        let evolution: Vec<Vec<EvolutionSample>> = state.evolution.into_iter()
            .map(|hist| hist.into_iter().map(|e| EvolutionSample {
                t: e.t, gen: e.gen, count: e.count, speed: e.speed, sense: e.sense,
                size: e.size, metabolism: e.metabolism, bold: e.bold, curious: e.curious,
                social: e.social, help_thr: e.help_thr,
                deco_eff: e.DecoEffMean, deco_eff_std: e.DecoEffStd,
            }).collect::<Vec<_>>()).collect();
        let generation_fitness: Vec<Vec<FitnessRecord>> = state.generation_fitness.into_iter()
            .map(|recs| recs.into_iter().map(|r| FitnessRecord {
                gen: r.gen, age: r.age, offspring: r.offspring, fitness: r.fitness,
            }).collect::<Vec<_>>()).collect();

        let mut isolated_groups: HashMap<&'static str, IsolatedGroup> = HashMap::new();
        let mut isolated_order: Vec<&'static str> = Vec::new();
        for g in state.isolated_groups {
            let name = save_state::intern_str(&g.name);
            isolated_groups.insert(name, IsolatedGroup {
                code: g.code,
                kind: save_state::intern_str(&g.kind),
                since: g.since,
            });
            isolated_order.push(name);
        }

        // Phase 15: restore clade tree + event log from the compact JSON
        // strings. Legacy v2 saves carry the serde defaults (empty strings),
        // restoring single-clade mode.
        let json_clades: Vec<serde_json::Value> = serde_json::from_str(&state.SpeciesClades.0)
            .unwrap_or_default();
        let json_events: Vec<serde_json::Value> = serde_json::from_str(&state.SpeciationEvents.0)
            .unwrap_or_default();
        let species_clades = json_clades.iter().map(|c| {
            let id = c["id"].as_u64().unwrap_or(0) as u32;
            let parent_id = c.get("parentId").and_then(|v| v.as_u64()).map(|v| v as u32);
            let kind = c["kind"].as_str().unwrap_or("deco").to_string();
            let name = c["name"].as_str().unwrap_or("").to_string();
            let birth_tick = c["birthTick"].as_u64().unwrap_or(0) as u32;
            let death_tick = c.get("deathTick").and_then(|v| v.as_u64()).map(|v| v as u32);
            let centroid_eff = c["centroidEff"].as_f64().unwrap_or(1.0);
            // Phase 16: pre-v3 saves carry no "niche" key -> 0.0; the next
            // detection pass re-derives it from the parent chain.
            let niche = c.get("niche").and_then(|v| v.as_f64()).unwrap_or(0.0);
            SpeciesClade {
                id,
                parent_id,
                kind: save_state::intern_str(&kind),
                name,
                birth_tick,
                death_tick,
                centroid_genome: { let mut g = Genome::default_deco(); g.deco_efficiency = centroid_eff; g },
                members: Vec::new(),
                niche_offset: niche,
            }
        }).collect();
        let speciation_events = json_events.iter().map(|e| CladeSpeciationEvent {
            tick: e["tick"].as_u64().unwrap_or(0) as u32,
            parent_clade_id: e["parentCladeId"].as_u64().unwrap_or(0) as u32,
            child_clade_ids: e.get("childCladeIds")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().map(|v| v.as_u64().unwrap_or(0) as u32).collect())
                .unwrap_or_default(),
            genetic_distance: e["geneticDistance"].as_f64().unwrap_or(0.0),
            // Phase 16: pre-v3 saves default to speciation (0).
            event_kind: e.get("kind").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
        }).collect();
        let next_clade_id = state.NextCladeId;
        let last_clade_spec_tick = state.LastCladeSpecTick;

        // Phase 17: cataclysm table. Pre-v4 saves carry the serde default
        // (empty string) → empty history, id counter 1 (pre-cataclysm world,
        // bit-identical continuation).
        let cataclysms: Vec<Cataclysm> =
            serde_json::from_str::<serde_json::Value>(&state.Cataclysms.0)
                .ok()
                .and_then(|v| v.as_array().cloned())
                .map(|arr| {
                    arr.iter().filter_map(|c| {
                        Some(Cataclysm {
                            id: c.get("id")?.as_u64()? as u32,
                            kind: c.get("kind")?.as_u64()? as u8,
                            tick: c.get("tick")?.as_u64()? as u32,
                            x: c.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            z: c.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            radius: c.get("radius").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            severity: c.get("severity").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            dust_recovery_ticks: c.get("dustRecoveryTicks").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                            climate_delta: c.get("climateDelta").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            ends_at: c.get("endsAt").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                            remaining: c.get("remaining").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                            active: c.get("active").and_then(|v| v.as_u64()).unwrap_or(0) == 1,
                        })
                    }).collect::<Vec<Cataclysm>>()
                })
                .unwrap_or_default();
        let next_cataclysm_id = if state.NextCataclysmId > 0 { state.NextCataclysmId } else { 1 };

        // Phase 16: competitive-exclusion burden map `[[cladeId,burden],…]`.
        // Pre-v3 saves carry the serde default (empty string) -> no burdens;
        // the next detection pass recomputes them from live clades.
        let clade_burdens: Vec<(u32, f64)> =
            serde_json::from_str::<serde_json::Value>(&state.CladeNicheBurdens.0)
                .ok()
                .and_then(|v| v.as_array().cloned())
                .map(|arr| {
                    arr.iter().filter_map(|pair| {
                        let id = pair.get(0)?.as_u64()? as u32;
                        let b = pair.get(1)?.as_f64()?;
                        Some((id, b))
                    }).collect::<Vec<(u32, f64)>>()
                })
                .unwrap_or_default();

        // Phase 18: geologic state. Pre-v5 saves carry the serde default
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

        let history = state.history.into_iter().map(|h| HistorySample {
            t: h.t, season: h.season, h: h.h, p: h.p, scav: h.scav, omn: h.omn,
            plant: h.plant, plant_by: h.plant_by,
            speed_h: h.speed_h, sense_h: h.sense_h, size_h: h.size_h,
            speed_p: h.speed_p, sense_p: h.sense_p, gap_hp: h.gap_hp,
            coop_share_h: h.coop_share_h, help_thr_h: h.help_thr_h,
            sig_mean_h: h.sig_mean_h, sig_var_h: h.sig_var_h,
            pref_mean_h: h.pref_mean_h, tol_mean_h: h.tol_mean_h, gen: h.gen,
        }).collect();

        let speciation_event = state.speciation_event.map(|e| SpeciationEvent {
            t: e.t,
            parent_kind: save_state::intern_str(&e.parent_kind),
            lineages: e.lineages.into_iter().map(|l| LineageInfo {
                id: save_state::intern_str(&l.id),
                name: save_state::intern_str(&l.name),
                avg_speed: l.avg_speed, count: l.count,
            }).collect(),
            cv_speed: e.cv_speed, gap_idx: e.gap_idx, name_suffix: e.name_suffix,
        });

        let mut biome_reg = BiomeRegistry::new();
        if let Some(list) = state.biomes {
            for b in list {
                biome_reg.add_biome(Biome {
                    id: b.id,
                    name: save_state::intern_str(&b.name),
                    center_x: b.center_x, center_z: b.center_z, radius: b.radius,
                    suit_modifier: b.suit_modifier,
                    max_herb: b.max_herb, max_pred: b.max_pred,
                    max_scav: b.max_scav, max_omn: b.max_omn,
                    max_fish: b.max_fish, home_range_mod: b.home_range_mod,
                });
            }
        }

        Ecology {
            rng: Mulberry32::from_state(state.rng_state.state),
            prng: Mulberry32::from_state(state.prng_state.state),
            fish_rng: Mulberry32::from_state(state.fish_rng_state.state),
            agents: state.agents.into_iter().enumerate().map(|(i, a)| {
                let g = a.genome;
                // Phase 32: variable-length ext-locus list (empty on pre-v6
                // saves via the serde default). Rebuilt in save order; the
                // per-locus scalars roundtrip losslessly.
                let genome = {
                    let mut gm = genome_from_save(g);
                    gm.ext = a.ext_loci.into_iter().map(locus_from_save).collect();
                    gm
                };
                // Phase 33: backfill seed cargo to the plant-registry length
                // (pre-v7 saves carry exactly 3; a shorter/corrupt list pads
                // with 0.0 so per-layer indices stay in bounds).
                let mut cargo = a.cargo;
                if cargo.len() < pn { cargo.resize(pn, 0.0); }
                Agent {
                    id: a.id,
                    kind: kind_ids[i],
                    x: a.x, z: a.z, y: a.y,
                    prev_x: a.prev_x, prev_z: a.prev_z, prev_heading: a.prev_heading,
                    heading: a.heading, speed: a.speed, desired_speed: a.desired_speed,
                    energy: a.energy, max_energy: a.max_energy, age: a.age,
                    genome,
                    generation: a.generation,
                    state: save_state::intern_str(&a.state),
                    bite: a.bite, repro: a.repro,
                    target_x: a.target_x, target_z: a.target_z, target_id: a.target_id,
                    think: a.think, phase: a.phase,
                    alive: a.alive,
                    activity: a.activity, hydration: a.hydration,
                    cargo,
                    trail_t: a.trail_t, trail_x: a.trail_x, trail_z: a.trail_z,
                    trail_head: a.trail_head, trail_n: a.trail_n, trail_slot: a.trail_slot,
                    fatigue: a.fatigue, fear: a.fear, image: a.image,
                    help_cd: a.help_cd, curiosity: a.curiosity,
                    home_x: a.home_x, home_z: a.home_z, range: a.range,
                    lineage: a.lineage.map(|s| save_state::intern_str(&s)),
                    clade_id: a.clade_id,
                    offspring: a.offspring,
                }
            }).collect(),
            carcasses: state.carcasses.into_iter().map(|c| Carcass {
                x: c.x, z: c.z, energy: c.energy, age: c.age,
                occupied_by: c.occupied_by,
            }).collect(),
            next_id: state.next_id,
            time: state.time,
            tick_count: state.tick_count,
            daylight: state.daylight,
            season: state.season,
            fires: state.fires,
            torch: state.torch,
            rain: state.rain,
            grass: state.grass,
            grass_p,
            suit_eff_by,
            plants,
            suit: state.suit,
            shade: state.shade,
            moist: state.moist,
            mineral_field: state.mineral_field,
            grid_acc: state.grid_acc,
            grid_n: GRID_N,
            grid_span: GRID_SPAN,
            cell: CELL,
            births: state.births,
            deaths: state.deaths,
            kills: state.kills,
            starved: state.starved,
            aged: state.aged,
            barrier_blocks: state.barrier_blocks,
            sig_attempts: state.sig_attempts,
            sig_blocks: state.sig_blocks,
            deposit_stats: DepositStats {
                count: state.deposit_stats.count,
                dist_sum: state.deposit_stats.dist_sum,
                far_5: state.deposit_stats.far_5,
                far_20: state.deposit_stats.far_20,
                colonize: state.deposit_stats.colonize,
                samples: state.deposit_stats.samples,
                sample_cursor: state.deposit_stats.sample_cursor,
            },
            fruit_e_h: state.fruit_e_h,
            leaf_e_h: state.leaf_e_h,
            help_count: state.help_count,
            reject_count: state.reject_count,
            help_cost_e_h: state.help_cost_e_h,
            drain_e_h: state.drain_e_h,
            speciation_fired: state.speciation_fired,
            history,
            last_sample: state.last_sample,
            gene_barrier_strength: state.gene_barrier_strength,
            isolated_groups,
            isolated_order,
            isolation_history: state.isolation_history.into_iter().map(|p| {
                IsolationHistoryPoint { t: p.t, dist: p.dist, pairs: p.pairs }
            }).collect(),
            evolution,
            generation_fitness,
            taken_buf,
            spread_buf: state.spread_buf,
            h_cache: state.h_cache,
            season_meta: state.season_meta,
            player_x: state.player_x,
            player_z: state.player_z,
            selection_pressure: SelectionPressure {
                plant: state.selection_pressure.plant,
                pred_ratio: state.selection_pressure.pred_ratio,
                crowding: state.selection_pressure.crowding,
            },
            speciation_event,
            last_speciation: state.last_speciation,
            species_clades,
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
            seed: state.seed,
            biome_reg,
            kinds,
            deco_eff_choice_mode,
            carcass_lock_enabled,
            // Phase 32: open-locus operator state (defaults keep pre-v6
            // restores on genesis OFF with a zero counter — bit-exact legacy
            // continuation; a v6 genesis-ON save keeps the operator running).
            genesis_on: state.GenesisOn,
            ext_dup_events: state.ExtDupEvents,
        }
    }
}


