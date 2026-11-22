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
use crate::culture::{self, KnowledgeToken};
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
    /// Phase 34: the kind this one forked from (`None` for the six legacy
    /// ancestors — they are the roots of the kind tree). Pure data event:
    /// every other field is copied from the parent at mint time.
    pub parent_kind: Option<SpeciesKind>,
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
            parent_kind: None,
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
            parent_kind: None,
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
            parent_kind: None,
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
            parent_kind: None,
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
            parent_kind: None,
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
            parent_kind: None,
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
            parent_kind: None,
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
/// Phase 59: knee and ceiling of the saturated DECO fertility threshold.
/// The legacy line asks for more than `max_energy` once `eff < 0.9583`; since
/// energy is clamped at `max_energy`, that is an absorbing sterile state, and
/// `DECO_MUT_SCALE` speciation births hand it to whole daughter clades. The
/// saturation leaves the legacy line untouched up to the knee (i.e. for
/// `eff >= 1.0`, so `eff = 1.5 -> 0.48` is unchanged) and bends the excess
/// into `(KNEE, CEIL)` with `x/(1+x)`: continuous, strictly monotone in eff,
/// and strictly below 1.0 for every legal efficiency, so `eff = 0.5` needs
/// 98.8 % of `max_energy` instead of an impossible 144 %.
const DECO_REPRO_SAT_KNEE: f64 = 0.96;
const DECO_REPRO_SAT_CEIL: f64 = 0.99;
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
/// Phase 40.1: hard ceiling on the effective reproduction energy threshold
/// (fraction of max_energy). The Phase 4-A density brake multiplies the
/// base threshold by (1 + density_factor), and at full carrying capacity
/// that pushed HERB to 1.29-1.72×max_energy — unreachable, so the population
/// stopped breeding entirely, aged as one cohort, and went extinct (the
/// 100k-tick observatory found two of three seeds dying this way).
/// Capping at 0.90 keeps the brake meaningful (breeding becomes rare at
/// saturation) without ever locking it out: well-fed individuals (90%+ of
/// max energy) can always breed. The brake's *intent* — lower birth rate
/// under crowding — is preserved; only the absorbing zero-birth state is
/// removed.
const REPRO_THRESHOLD_MAX: f64 = 0.90;
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

// ----- Phase 35: energy ledger + information cost + MTE metabolism ---------
//
// Three mechanisms land together, each with its own re-baseline accounting:
//
// 1. `EnergyLedger` — every existing energy exchange (drain, grazing,
//    predation, scavenging, reproduction, death, carcass evaporation,
//    decomposition) is booked into named pools. PURE BOOKKEEPING: no RNG
//    draw, no state mutation outside the ledger, no numeric change — the
//    trajectory through this layer alone is bit-exact.
// 2. `INFO_COST_ON` — Landauer-style information cost. Each sensor sweep
//    (`find_grass` probe batch, the predator's `nearest` scan batch, the
//    threat-detection scan) charges `INFO_COST_PER_READ` scaled by the
//    sense radius, so a wider perception is thermodynamically dearer.
//    `false` → zero extra draws, zero drain (A/B audited by test).
// 3. `MTE_ON` — Kleiber body-mass metabolic scaling: a `size^0.25`
//    multiplier on every drain (mass^0.75 with mass ∝ size^3), and a
//    body-mass-gated reproduction fee.
// 4. `TROPHIC_EFF` applied to live-prey biomass conversion (the primary
//    re-baseline source this phase).
//
// Default toggles keep the Phase-34 trajectory bit-exact; tests switch
// them ON individually to audit each mechanism in isolation.
pub const INFO_COST_PER_READ: f64 = 0.002;
/// Phase 36: information cost is LIVE (default on). The legacy constant is
/// kept as the DEFAULT value read by `Ecology::new`; per-instance toggling
/// lives on `Ecology.info_cost_on` so A/B tests can flip it at runtime while
/// the default world (and its Phase 36 anchor) keeps it ON.
pub const INFO_COST_ON: bool = true;
pub const MTE_ON: bool = false;
pub const MTE_MASS_EXPONENT: f64 = 0.75;
pub const MTE_SIZE_EXPONENT: f64 = MTE_MASS_EXPONENT / 3.0;
pub const MTE_REPRO_FEE: f64 = 0.5;

// ----- Phase 36: evolvable micro-brain (intelligence layer) --------------
//
// Design constraints (roadmap §2.3 / §4 determinism discipline):
// * `BRAIN_ON` — the shadow network runs `forward` + Hebbian learning after
//   every FSM decision, ZERO RNG draws (pure state function). The shadow
//   never drives behaviour unless `BRAIN_SWITCH_ENABLED` hands authority
//   over at a speciation-check boundary.
// * `BRAIN_UPKEEP_ON` — Herculano-Houzel-style maintenance: `BRAIN_PER_NEURON_COST
//   × n_hidden` per tick is drained from the agent's energy and booked into
//   `ledger.brain_cost`. This is the ONLY new energy term, so "big brains
//   burn faster" is an emergent selection pressure, not a rule.
// * `T_EQUIV` / `T_EQUIV_K_TICKS` — the statistical-equivalence gate for the
//   dual-controller switchover: the brain may only take authority when its
//   one-hot argmax agreed with the FSM's action for ≥ `T_EQUIV_K_TICKS`
//   consecutive ticks at ≥ `T_EQUIV` agreement ratio.
// * Learning plasticity rides the DEDICATED `BRAIN_RNG_TAG` stream for
//   structural mutation only (Ohno node duplication); the forward pass and
//   the Hebbian eligibility update themselves are zero-RNG, which keeps the
//   main/prng/fish streams bit-identical.
pub const BRAIN_ON: bool = true;
pub const BRAIN_UPKEEP_ON: bool = true;
/// Phase 42: 行为权威开闸。三脑在通过一致性门（且置信度门）后接管行为；
/// `false` 保留 Phase 36/41 的纯影子 A/B 路径（测试与对照用）。
pub const BRAIN_SWITCH_ENABLED: bool = true;
/// Statistical-equivalence agreement threshold (FSM vs brain action).
pub const T_EQUIV: f64 = 0.95;
/// Consecutive-tick window for the equivalence gate.
pub const T_EQUIV_K_TICKS: u32 = 60;
/// Phase 42: 接管前的置信度下限（元认知门）。三脑的世界模型没拟合好之前
/// （预测误差大 → 置信度低）不把行为交出去——「高置信才接管」。
pub const T_TAKEOVER_CONF: f64 = 0.50;
/// Phase 42: 经验回放默认开关（A/B 对照可运行期翻转）。
pub const REPLAY_ON: bool = true;
/// Structural-mutation probabilities (per birth event, dedicated brain RNG).
pub const BRAIN_MUTATE_P_ADD: f64 = 0.02;
pub const BRAIN_MUTATE_P_DEL: f64 = 0.01;
/// Child-inheritance weight perturbation σ.
pub const BRAIN_PERTURB_SIGMA: f64 = 0.4;
/// Deterministic per-agent init noise on the grandfather network.
pub const BRAIN_NOISE_SIGMA: f64 = 0.25;
/// Phase 41: per-birth triune gene/world noise scale.
pub const TRIUNE_MUTATE_SIGMA: f64 = 0.6;
/// Dedicated-stream seed for brain structural mutation (never main/prng/fish).
pub const BRAIN_RNG_TAG: u32 = 0x42_52_41_49;

// ----- Phase 43: abiogenesis（生命再起源）-----
//
// 米勒-尤里/热泉口化学演化：生命从无机化学自发产生。这是神之手退役后
// 「接入现实规律」的重新起源路径 —— 不是复活机制，是化学→生命的自然通道。
// 触发是涌现式的：全局存活个体 < ABIO_MIN_POP（世界接近死寂）时每 tick
// 以 p = ABIO_P_BASE × mean(候选格强度) 的概率自发萌发一个祖先个体。
// 生态饱和抑制：健康世界（pop ≥ 8）零触发、零 RNG draw。
//
// RNG 纪律：决策（触发/选格/选祖先）走专用 `seed ^ ABIO_RNG_TAG` 流；
// draw 预算固定可审计 —— pop < ABIO_MIN_POP 且候选非空时每 tick ≤ 1 次触发
// roll，命中后 +1 选格 +1 选祖先；spawn 内部的出生 draw（朝向/初始能量/
// 基因组）沿用 seed_life 同一主流路径。

/// Dedicated abiogenesis RNG stream tag (`seed ^ ABIO_RNG_TAG`).
pub const ABIO_RNG_TAG: u32 = 0x41_42_49_4F; // "ABIO"
/// Abiogenesis toggle (default ON; the A/B arm flips it to false).
pub const ABIOGENESIS_ON: bool = true;
/// Birth-inheritance toggle (default ON; the A/B arm flips it to false).
pub const TEACH_ON: bool = true;
/// 全局存活个体低于该值时化学演化才可能萌发（生态饱和抑制的硬门）。
pub const ABIO_MIN_POP: usize = 8;
/// 每 tick 基础萌发概率（乘以候选格平均强度 ∈ (0,1]）。
/// 标定：强度 ~0.4 时 p_eff ≈ 2.4e-4/tick → 死世界平均 ~4k tick 首萌发，
/// 数千~数万 tick 内自然重启；繁荣世界被 pop 门结构性抑制。
pub const ABIO_P_BASE: f64 = 0.0006;
/// 候选格阈值 —— 矿化活跃（DECO 沉积场仍在工作）。
/// mineral_field ∈ [0,1] 按活性分解位点沉积、每 tick ×MINERAL_DECAY 衰减：
/// 0.15 是「衰减间隙中仍可辨认为活跃矿化」的下限（实测基线 0.35+ 活跃格）。
pub const ABIO_MINERAL_TH: f32 = 0.15;
/// 候选格阈值 —— 水缘高湿。moist 由岸边 BFS 生成（0.45 − 0.035·d，上限
/// 0.45）：moist > 0.3 ⇔ 距水线 ~4 格内的湿润带（任务建议的 0.6 超出该场
/// 值域，永不触发；0.3 是值域内的等效「高湿」分位）。
pub const ABIO_MOIST_TH: f32 = 0.30;
/// 候选格阈值 —— 植物适宜度上半个值域（suit ∈ [0,1]，草生长需 > 0.1，
/// > 0.5 是肥沃土壤：化学能可支撑自养生存的格子）。
pub const ABIO_SUIT_TH: f32 = 0.5;

/// Authority markers on `Agent.brain_authority`.
pub const BRAIN_AUTHORITY_FSM: u8 = 0;
pub const BRAIN_AUTHORITY_BRAIN: u8 = 1;

/// Phase 36: global learning telemetry (pure observability — never feeds the
/// trajectory). Recorded so the shadow/learning audit path is inspectable
/// without touching agent energy.
#[derive(Clone, Debug, Default)]
pub struct BrainLearningStats {
    /// Total shadow forward passes run.
    pub forward_count: u64,
    /// Total Hebbian eligibility updates applied.
    pub hebbian_updates: u64,
    /// Longest FSM↔brain agreement streak observed.
    pub equiv_streak_max: u32,
    /// Number of authority switches into the brain so far.
    pub authority_switches: u32,
    /// Cumulative brain upkeep energy drained (beyond the ledger's info_cost).
    pub brain_energy_drained: f64,
}

/// Phase 41: triune-brain global telemetry (pure observability — never feeds
/// the trajectory). Per-agent counters live on `Agent.triune.stats`; this
/// aggregate sums the per-tick increments across the population.
#[derive(Clone, Debug, Default)]
pub struct TriuneLearningStats {
    pub ticks: u64,
    pub upkeep_energy: f64,
    pub decision_energy: f64,
    pub fear_events: u64,
    pub fear_conditioning: u64,
    pub memory_retrievals: u64,
    pub wm_updates: u64,
    pub rollout_steps: u64,
    pub planned_overrides: u64,
    pub inhibitions: u64,
    pub prediction_updates: u64,
    pub high_confidence_ticks: u64,
    pub max_fear: f64,
    pub fear_sum: f64,
    pub confidence_sum: f64,
    pub pred_error_sum: f64,
    pub action_hist: [u64; aether_brain::OUTPUT_ACTIONS],
    /// Phase 42: 离线回放次数。
    pub replay_updates: u64,
    /// Phase 42: 抑制闸门校准指标：gate > 0.5 的 tick 数与闸门总和。
    pub gate_open_ticks: u64,
    pub gate_sum: f64,
    /// Phase 42: 行为权威接管后的适应性记账（接管 vs 未接管的 A/B）。
    pub fitness: AuthorityFitness,
}

/// Phase 42: 按「行为权威类型」分组的累计适应性（纯观测，不进动力学）。
/// 死亡时记寿年与子代，出生时按亲代权威归类——这是「脑驱动是否更优」
/// 的直接检验数据。
#[derive(Clone, Debug, Default)]
pub struct AuthorityFitness {
    pub fsm_deaths: u64,
    pub fsm_lifespan_sum: f64,
    pub fsm_offspring_sum: u64,
    pub brain_deaths: u64,
    pub brain_lifespan_sum: f64,
    pub brain_offspring_sum: u64,
    pub fsm_parent_births: u64,
    pub brain_parent_births: u64,
}

impl AuthorityFitness {
    pub fn mean_fsm_lifespan(&self) -> f64 {
        if self.fsm_deaths == 0 { 0.0 } else { self.fsm_lifespan_sum / self.fsm_deaths as f64 }
    }
    pub fn mean_brain_lifespan(&self) -> f64 {
        if self.brain_deaths == 0 { 0.0 } else { self.brain_lifespan_sum / self.brain_deaths as f64 }
    }
}

impl TriuneLearningStats {
    pub fn mean_fear(&self) -> f64 {
        if self.ticks == 0 { 0.0 } else { self.fear_sum / self.ticks as f64 }
    }
    pub fn mean_confidence(&self) -> f64 {
        if self.ticks == 0 { 0.0 } else { self.confidence_sum / self.ticks as f64 }
    }
    pub fn mean_pred_error(&self) -> f64 {
        if self.ticks == 0 { 0.0 } else { self.pred_error_sum / self.ticks as f64 }
    }
    /// Phase 42: gate > 0.5 的 tick 占比（抑制校准主指标）。
    pub fn gate_open_ratio(&self) -> f64 {
        if self.ticks == 0 { 0.0 } else { self.gate_open_ticks as f64 / self.ticks as f64 }
    }
    pub fn mean_gate(&self) -> f64 {
        if self.ticks == 0 { 0.0 } else { self.gate_sum / self.ticks as f64 }
    }
}

/// Phase 35 energy bookkeeping pools. Pure observability — methods only
/// mutate `self`'s own float counters; nothing touches agent or world state.
#[derive(Clone, Debug, Default)]
pub struct EnergyLedger {
    /// Cumulative primary-production input (grass logistic growth in energy units).
    pub light_in: f64,
    /// Cumulative FISH filter-feeding open-boundary input.
    pub filter_in: f64,
    /// Phase 43: cumulative hot-spring chemical energy booked into
    /// abiogenesis spawns (与 light_in/filter_in 同级的开放输入通道).
    pub chemo_in: f64,
    /// Cumulative carcass evaporation (formerly 1.4/tick leak with no sink).
    pub evaporation: f64,
    /// Cumulative metabolic drain across all agents.
    pub metabolism_loss: f64,
    /// Cumulative reproduction cost (parent repro drains + optional MTE mass fee).
    pub repro_loss: f64,
    /// Cumulative DECO-mineralized biomass into soil.
    pub mineralized: f64,
    /// Cumulative start-energy invested into new offspring at spawn.
    pub offspring_invest: f64,
    /// Cumulative plant biomass eaten by herbivores/omnivores (cells × 62).
    pub grazed: f64,
    /// Cumulative prey biomass converted into consumer biomass (post-TROPHIC_EFF).
    pub predation_gain: f64,
    /// Cumulative carcass mass consumed by scavengers/omnivores/decomposers.
    pub scavenged: f64,
    /// Cumulative information-cost drain (zero while INFO_COST_ON=false).
    pub info_cost: f64,
    /// Total perception reads charged — audit trail for the info-cost test.
    pub info_reads: u64,
    /// Phase 36: cumulative brain-upkeep drain (`BRAIN_PER_NEURON_COST × n_hidden`
    /// per tick). Zeroed while `BRAIN_UPKEEP_ON` is off.
    pub brain_cost: f64,
    /// Phase 41: cumulative triune-brain drain (maintenance + rollout decision
    /// cost + inhibition cost). Zeroed while `triune_upkeep_on` is off.
    pub triune_cost: f64,
}

impl EnergyLedger {
    pub fn new() -> Self { EnergyLedger::default() }
    /// Σ of all inputs booked so far.
    pub fn total_in(&self) -> f64 { self.light_in + self.filter_in + self.chemo_in }
    /// Σ of all losses booked so far (leaves the closed system), including
    /// the Phase 36 brain-upkeep drain.
    pub fn total_loss(&self) -> f64 {
        self.evaporation + self.metabolism_loss + self.repro_loss
            + self.mineralized + self.offspring_invest + self.info_cost + self.brain_cost
            + self.triune_cost
    }
}

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

#[derive(Debug, Clone)]
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
    /// Phase 47: home region id (geographic deme). Initial members are
    /// balanced across REGION_COUNT angular sectors; births inherit the
    /// mother's region, migrants take the target region.
    pub region: u8,
    pub offspring: u32,
    /// Phase 36: the agent's micro-brain (tiny recurrent network). Default
    /// zeroed; `spawn` seeds it from the kind's grandfather reference network
    /// plus per-agent deterministic noise, `reproduce` inherits the parent's
    /// brain, and `from_save_state` restores it from the v9 schema.
    pub brain: aether_brain::Brain,
    /// Phase 36 dual-controller authority: 0 = FSM drives behaviour,
    /// 1 = the brain drives it (FSM demoted to shadow).
    pub brain_authority: u8,
    /// Phase 36: consecutive ticks where the brain's one-hot action agreed
    /// with the FSM's during the `T_EQUIV` equivalence window.
    pub brain_equiv_streak: u32,
    /// Phase 36: tick count at which the current `brain_equiv_streak` began.
    pub brain_equiv_start: u32,
    /// Phase 36: running window of FSM↔brain argmax agreements (for the
    /// `T_EQUIV` ratio test at the speciation-check boundary).
    pub brain_agree_window: u32,
    pub brain_window_len: u32,
    /// Phase 36: the FSM one-hot action of the most recent tick (shadow book).
    pub brain_last_fsm_action: u8,
    /// Phase 36: the brain one-hot action of the most recent tick (shadow book).
    pub brain_last_brain_action: u8,
    /// Phase 41: the triune brain (instinct / emotion / reason). Runs every
    /// tick as a zero-RNG shadow decision layer; its maintenance + rollout
    /// cost is drained as real energy (`ledger.triune_cost`), so brain size
    /// is under selection. Behaviour authority stays behind the existing
    /// `brain_switch_enabled` gate.
    pub triune: aether_brain::triune::TriuneBrain,
    /// Phase 42: 三脑一致度门——它在多少个连续的脑节律 tick 上与自己当时的
    /// FSM 动作一致（只有影子阶段有意义）。
    pub triune_equiv_streak: u32,
    /// Phase 42: 最近 `T_EQUIV_K_TICKS` 个脑节律 tick 的一致位图（bit=1 表示
    /// 一致）。用 64 位移位寄存器做真正的滑动窗口，而不是只涨不跌的计数器。
    pub triune_agree_bits: u64,
    /// Phase 42: 位图里已累积的样本数（封顶 `T_EQUIV_K_TICKS`）。
    pub triune_agree_n: u32,
    /// Phase 37: knowledge tokens this agent has learned (id list, in learning order).
    pub known_tokens: Vec<u32>,
    /// Phase 39.1: group membership (0 = solitary). Groups are behavioural
    /// aggregates, not multicellular bodies — the member stays a first-class
    /// individual with its own energy, genome and death.
    pub group_id: u32,
    /// Phase 39.1: sub-task role inside the group (0=GO, 1=FEED, 2=GUARD,
    /// 3=REPRO, `ROLE_NONE`=unassigned). Assigned by `group::role_scores`
    /// (genome + brain state), never scripted.
    pub role: u8,
    /// Phase 39.1: somatic damage accumulated by dirty work (GO role in a
    /// hazardous cell). Raises the offspring mutation amplitude via
    /// `1 + SOMATIC_MUTAGEN × damage`; the germline stays clean simply because
    /// REPRO-role individuals do not take dirty work.
    pub somatic_damage: f64,
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
/// MST cut threshold on the weighted multi-axis trait distance (Phase 34:
/// core speed/sense/size + ext loci + the DECO efficiency axis). All axes are
/// normalized to roughly unit scale (see `pair_mst_distance`).
///
/// Phase 47 calibration (Plan C): the Phase-34 value 0.35 was calibrated on a
/// single panmictic 80-agent population. The region-island model splits that
/// population into four ≈20-agent demes, whose between-deme MST edge peaks in
/// the 0.25-0.35 band (Phase-47 diagnostic); 0.25 is the lowest cut that still
/// sits above the within-deme mutation noise floor. The instrument remains
/// kind-uniform — only the scale changed.
///
/// INSTRUMENT, NOT SCRIPT (Phase 34 ruling): this constant and the gates
/// below are measurement parameters of the detector — they are applied
/// identically to every kind (no per-species script, no directional bias).
pub const SPECIATION_DISTANCE_THRESHOLD: f64 = 0.25;
/// Per-paired-ext-locus contribution to the MST edge weight: the mean over
/// id-paired loci of |Δvalue| (unit [0,1] band, already normalized) times
/// this static weight. Unpaired (one-sided) loci contribute 0 — symmetric,
/// no bias toward gaining or losing loci.
pub const EXT_LOCUS_WEIGHT: f64 = 0.5;
/// Geographic isolation gate: the nearest cross-cluster pair must be farther
/// apart than this (DECO home range is ≈150-180 world units).
pub const GEO_ISOLATION_DIST: f64 = 100.0;
/// Minimum ticks between two speciation events.
pub const MIN_SPEC_TIME_TICKS: u32 = 200;
/// Minimum members BOTH sides of a cut must keep for the split to count as a
/// species (Phase 49: a 2-member "species" is sampling noise — Phase 48's
/// median-3840-tick daughter lineages were exactly that noise; 8 members is
/// the smallest cohort with a real independent-evolution chance). Configurable
/// per instance (`Ecology::speciation_min_side`) so the Phase 47 A/B arm can
/// pin the legacy value.
pub const SPECIATION_MIN_SIDE_DEFAULT: usize = 8;
/// Phase 47 value (kept as the B-arm calibration for `budget_fork_off`).
pub const SPECIATION_MIN_SIDE_LEGACY: usize = 2;

// ----- Phase 49: fork budget (independent resource share per species) -------
//
// Phase 48's root cause ②③: the family cap made every derived kind share the
// root's 80 slots (any family member could fill them), and niche_offset only
// shifted the competition burden — no fork ever received its own resource
// pool, so every daughter was crowded out (median 3840 ticks). Phase 49 moves
// the carrying capacity itself: a fork CARVES a share out of the parent
// kind's budget; each kind's births are gated by its own share only, so a
// daughter's growth is no longer coupled to the root's occupancy. The guild
// total is conserved (80 for DECO): the grant moves budget parent→child, an
// extinct kind returns it up the fork lineage.
/// Upper bound of a fork grant (the Phase-49 example: root 80 → 60 + 20).
pub const FORK_BUDGET_GRANT: usize = 20;
/// Floor of a grant AND of the parent's post-grant remainder: a kind with
/// less than `2 × FORK_BUDGET_MIN` cannot fork again (80 → 60+20 → 45+15 →
/// 34+11 → 26+8 → 18+8 → 10+8 is the deepest legal chain from 80). Phase 53
/// raised the floor 5 → 8: SPECIATION_MIN_SIDE=8 declares a fork only when
/// both sides hold 8 members, but a grant of 5 capped the newborn kind's spawn
/// rate below its mint-time population — the clade shrank from day one and
/// died young (200k observation: both grandchildren went extinct despite the
/// niche minimum distance).
///
/// Phase 63 raised it 8 → 10 (`research/phase63-report.md`): an 8-slot floor
/// declares a fork at 8 members, i.e. exactly at its own budget with no
/// headroom, and 13 of 15 daughter extinctions were ≥ 99.6 % birth-blocked on
/// that share. On the Phase 57–62 seed pool the 10-slot floor cut extinctions
/// 17 → 10 at a cost of 31 → 29 forks.
///
/// Phase 64 re-ran exactly that comparison on five seeds that had never
/// entered a verdict (`research/phase64-report.md`): forks 30 → 26
/// (R = 0.867, bar 0.90) and extinctions 19 → 19 (bar 0.75; two of the five
/// seeds reversed). The extinction win did not survive out of sample, so the
/// floor is back to 8 — the nineteenth re-baseline. 10 remains reachable as
/// the `PHASE_FORK_MIN=10` arm.
pub const FORK_BUDGET_MIN: usize = 8;
/// Phase 49: is the fork-budget mechanism active? Default ON; the A/B arm
/// (`budget_fork_off`) restores the Phase 47 family-shared-cap trajectory.
pub const BUDGET_FORK_ON: bool = true;

// ----- Phase 65: fork grace (candidate W1) ----------------------------------
//
// The Phase 62/63 「建支即失稳」 class: a daughter passes MIN_SIDE=8 at the cut,
// then dissipates inside 4–10k ticks with `region_cap_blocks = 0` AND
// `kind_budget_blocks = 0`. By construction neither capacity gate ever refused
// a birth — the clade died on recruitment (6 births / 14 deaths in the Phase 62
// instance), not on slots. W1 nevertheless tests the remaining capacity
// hypothesis: a newborn clade may overdraw its grant for one window, and the
// overdraft is settled out of the parent's share if the clade is still viable
// when the window closes. Population may exceed the grant during the window;
// the slot table does not move until settlement, so the family total stays
// conserved by construction.
/// Phase 65 W1 compiled default: OFF until a pre-registered 5-seed A/B passes
/// (`research/phase65-report.md`).
pub const FORK_GRACE_ON: bool = false;
/// Grace window after a fork (5 000 ticks: covers Phase 62's 3 960-tick and
/// Phase 63's 8 710-tick dissipations at the 60-tick settlement cadence).
pub const FORK_GRACE_TICKS: u32 = 5_000;

/// Phase 66 diagnostic: a clade counts as a "new species" for this many ticks
/// after its fork. Matches the 10k window of the `建支即失稳` definition.
pub const YOUNG_CLADE_TICKS: u32 = 10_000;

// ----- Phase 66: derived-clade regional mate-radius floor -------------------
//
// Phase 66 diagnosis (research/phase66-report.md §1.1): 77.8 % of a derived
// DECO clade's failed mate searches have a fertile same-kind conspecific in
// the SAME region, just beyond the radius. The dense-side addressable density
// is 5.3 same-kind members per region, while the mean distance to that ready
// partner is 274.2 — the Phase 42.1 Allee mitigation tops out at 44 (4 x the
// legacy 11). The floor therefore follows the regional deme's spatial scale:
// `REF * sqrt(N_REF / n)`, capped. Roots and FISH are out of scope (the root
// guild is the Phase 47 island-model differentiation engine).
//
// Zero RNG, zero state read back, not persisted (same contract as the Phase
// 61/63 arm knobs): the floor only changes the search bound.
/// Compiled default: OFF until a pre-registered 5-seed A/B passes.
pub const MATE_FLOOR_ON: bool = false;
/// Density reference: the floor reaches `REF` at this many same-region members
/// (the median observed deme size, 4).
pub const MATE_FLOOR_N_REF: f64 = 4.0;
/// R1 (moderate): 300 units at the reference density, capped at 320.
pub const MATE_FLOOR_R1_REF: f64 = 300.0;
pub const MATE_FLOOR_R1_MAX: f64 = 320.0;
/// R2 (wide): 420 units, capped at 420 (screen-only calibration arm).
pub const MATE_FLOOR_R2_REF: f64 = 420.0;
pub const MATE_FLOOR_R2_MAX: f64 = 420.0;
/// R3 (small, Phase 67): 80 units at the reference density, never below 80,
/// capped at 120. Targets only the near end of the `min_ready_dist` spike at
/// 44.00 (research/phase66-report.md §5.2); the 274-mean distance tail stays
/// outside the search.
pub const MATE_FLOOR_R3_REF: f64 = 80.0;
pub const MATE_FLOOR_R3_MIN: f64 = 80.0;
pub const MATE_FLOOR_R3_MAX: f64 = 120.0;
/// Overdraft ceiling = one minimum viable cohort. A no-further-fork parent is
/// guaranteed to hold this much (its post-grant remainder floor is
/// `FORK_BUDGET_MIN`), so settlement can always be paid in the simple case;
/// deeper parents pay what they still hold and the shortfall is recorded, never
/// minted. Same ceiling as Phase 63's growth arm (`grant + 8`).
pub const FORK_GRACE_OVERDRAFT_MAX: usize = 8;

// ----- Phase 63: fork-budget growth (candidate G) ---------------------------
//
// Phase 62 §3.3/§4: the daughters die on their OWN kind budget, not on the
// regional share. A daughter is declared a species at `SPECIATION_MIN_SIDE`
// members while its grant is usually 8–11 slots, so it is born at (or within
// 3 of) its own ceiling and spends its whole life at ≥ 99.6 % birth-blocked —
// a subcritical random walk. Candidate G gives a surviving daughter room to
// grow by transferring slots out of its parent's share: every
// `FORK_BUDGET_GROWTH_STEP_TICKS` of survival earns one slot, and the transfer
// is paid only while the daughter is actually filling what it already holds.
// The family total stays 80 (parent→child move), so no new biomass is minted.
/// Phase 63 candidate G default. Arm-only until the 5-seed A/B
/// (`research/phase63-report.md`) decides; `FORK_BUDGET_GROWTH_ON = true` is
/// the promotion value after a passing run (its state then rides the save).
pub const FORK_BUDGET_GROWTH_ON: bool = false;
/// Survival ticks per earned slot (5 000: a daughter that lives through
/// Phase 62's 21k–68k extinction window earns +4 … +8 slots).
pub const FORK_BUDGET_GROWTH_STEP_TICKS: u32 = 5_000;
/// Ceiling of the earned extra: grant + 8 keeps a grown daughter at most two
/// minimum viable cohorts (`2 × SPECIATION_MIN_SIDE`), enough to fork once
/// without returning to the born-at-ceiling state.
pub const FORK_BUDGET_GROWTH_MAX_EXTRA: usize = 8;

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
/// Phase 53: minimum generational niche distance. Phase 52's 1M tree showed
/// every depth-2 descendant dies (median lifespan short): a granddaughter's
/// eff gap to its mother projects to a niche distance below
/// NICHE_PARTITION_BAND, so mother and daughter forage the same resource
/// face and the established mother wins. A fork now guarantees at least this
/// much axis separation (direction = eff gap, tie-break = speed gap; a
/// daughter at the axis boundary walks back instead of saturating).
pub const NICHE_GEN_MIN_OFFSET: f64 = 0.35;
/// Phase 53: generational niche accumulation on by default; the A/B arm
/// (`niche_gen_accum_off`) restores the Phase 52 (Phase 16) projection.
pub const NICHE_GEN_ACCUM_ON: bool = true;

// ----- Phase 54: culture × speciation coupling (real niche construction) ---
//
// Phase 37 registered structures but never let them touch the environment —
// try_place_structure only paid the builder instant energy, the world's
// grass/mineral fields were untouched, so culture could not shape niche
// space. Phase 54 wires the loop: a placed structure raises the local grass
// regrowth multiplier for STRUCTURE_ENV_RADIUS cells (additive share of
// `token_to_env_mod`'s grass_mod), decaying slowly back toward 1.0. Because
// cultural learning is already kind-local (the demoiser search is same-kind)
// and kinds fork into clades, each clade's structures cluster in its own
// home range — culture now differentiates the resource face the clade lives
// on. The growth multiplier multiplies the existing per-cell growth path, so
// every gram of extra grass still flows through the sunlight ledger.
//
// Phase 55: the boost is a **net-zero dipole**. The 80k A/B of the Phase 54
// calibration showed the trophic-cascade failure mode: value-selected
// positive tokens raise the world's total primary production, the DECO
// root stabilises, drift slows, speciation goes to zero. A placement now
// transfers regrowth potential instead of creating it — the disk absorbs
// `grass_mod × STRUCTURE_ENV_BOOST` per cell under the clamp, and the exact
// amount actually absorbed is subtracted from the surrounding ring. Culture
// can choose WHERE grass grows; it can no longer raise the ceiling. Clamp
// saturation self-limits both sides: a disk pinned at MAX absorbs nothing
// and is compensated by nothing.
/// Cells within this radius of a structure feel its regrowth boost.
pub const STRUCTURE_ENV_RADIUS: i32 = 3;
/// Phase 55: the dipole's compensation ring — cells at Chebyshev radius in
/// (STRUCTURE_ENV_RADIUS, STRUCTURE_ENV_RING] pay back what the disk
/// absorbed (7×7 disk vs 11×11 ring ≈ 49/72 spread).
pub const STRUCTURE_ENV_RING: i32 = 5;
/// Per-tick decay of the structure regrowth multiplier toward 1.0
/// (half-life ≈ 350 ticks — a structure's niche must be continuously
/// maintained by new placements to persist). Calibration note: the first
/// draft (×8, half-life 6.9k) smothered speciation entirely — 922
/// structures saturated the field and the DECO root stopped forking for
/// 80k ticks. Culture shapes the resource face; it must not own it.
pub const STRUCTURE_ENV_DECAY: f32 = 0.998;
/// Bounds of the per-cell multiplier (a negative-grass_mod token suppresses
/// regrowth just as a positive one promotes it).
pub const STRUCTURE_ENV_MAX: f32 = 1.5;
pub const STRUCTURE_ENV_MIN: f32 = 0.5;
/// Per-structure boost scale: the raw token grass_mod is ±0.05
/// (STRUCTURE_MOD_STRENGTH); ×2 keeps one structure's visible-but-modest
/// mark at ±0.1 per disk cell (the ring pays back 49/72 ≈ 0.68 of that).
/// Calibration sweep (Phase 55, 80k, seed 20261102): 0.2 / 2 / 20 with the
/// natural sign and 20 with the famine (inverted) sign — every arm had zero
/// speciation events against the OFF arm's two. The one held here is the
/// round-1 value: net-zero verified to five decimals (env_mean ≡ 1.00000 at
/// every 5k sample) with a measurable multiplier mosaic.
pub const STRUCTURE_ENV_BOOST: f32 = 2.0;
/// Phase 54 A/B: structures touch the environment. Phase 55 reworked the
/// boost into a net-zero dipole (transfer, not creation — see
/// `structure_env_dipole`) and swept the calibration: **even a near-inert
/// mosaic (BOOST=0.2, ±1 % local multiplier) ran 80k with zero speciation
/// events** against the OFF arm's two. A 200k probe of the held calibration
/// settled the question: OFF forks four times (t=34.7k/41.8k/125.9k/128.0k),
/// ON never forks — a systematic suppression, not a missed window. The
/// mechanism is *accessibility, not total*: structures are placed where
/// agents live, so the dipole concentrates regrowth into their home ranges;
/// even at conserved total production the accessible crop rises (+2 % mean
/// grass, +4 % mean population over 200k) — the same trophic cascade as
/// Phase 54, in spatial form. **Default stays OFF.** The wiring stays
/// because the net-zero contract is real (measurable, test-pinned) and
/// heterogeneity is measurable (60k grass variance ON 0.07486 > OFF
/// 0.06479) — what the dipole cannot do is *cause* speciation by nudging
/// the grass field; a supportive culture×speciation coupling needs a
/// different causal route (Phase 56: DECO-direct resources or
/// clade-exclusion foraging).
pub const STRUCTURE_ENV_ON: bool = false;

// ----- Phase 56: culture × the DECO resource face (carcass decomposition) ----
//
// Phases 54/55 spent three calibrations proving that the grass field is the
// wrong causal route: any structure effect on primary production — one-sided
// boost, net-zero dipole, even a ±1 % mosaic — stabilises the DECO root and
// zeroes speciation, because the DECO (the lineage that actually forks) eats
// carcasses, not grass. Phase 56 moves the target onto DECO's own resource
// face: carcass mass.
//
// `deco_env` is a per-cell multiplier on the carcass decay rate (`1.4/tick`
// spread over the world, previously a global constant). A cell at 0.5 keeps a
// carcass available twice as long (a larder); a cell at 1.5 rots it away in
// two thirds of the time (a desert). Structures imprint the field from the
// sign of their token's `grass_mod`: an enrichment token (grass_mod > 0)
// SLOWS decomposition around it — the enriched soil retains the carcass mass
// for its decomposers — and a depletion token accelerates it. Because each
// clade's repertoire is its own (Phase 37/45), clades that carry opposite
// token signs lay down opposite food regimes over their home ranges; the same
// carcass supply means a different life depending on whose culture shapes the
// ground.
//
// Why this should support speciation where the grass dipole could not: the
// two regimes select on different traits. In a fast-rot patch a decomposer
// must be fast and far-sensing to reach a carcass before it is gone; in a
// slow-rot one a slow, cheap decomposer survives just as well. The stimulus
// lands on exactly the axes the speciation instrument measures (speed /
// sense / size / metabolic efficiency), and it is spatial — the geographic-
// isolation gate is met by demes living in different regimes, not by a
// world-wide production lift that makes everyone comfortable.
//
// What the calibration actually showed (80k, gain 3/10 fail, gain 20 passes)
// is a sharper story: the sign split is not the driver, the CONTRAST is. A
// gentle mosaic (gain 3/10) feeds the DECO guild to its 80-slot cap — the
// Phase 54/55 stabilisation, reproduced on the food-lifetime axis — while a
// saturated one (gain 20: every placement pins the disk at 0.5× or 1.5×)
// costs the guild a quarter of its population and fragments it into regions.
// Cutting the food supply in half over 49 cells and doubling it on the 72
// cells around them is the first structure effect in four phases that buys
// forks instead of comfort.
//
// The dipole keeps Phase 55's shape but not its approximation: the ring's
// capacity is measured BEFORE the disk is stamped and the imprint is scaled
// to what the ring can pay back, so the field mean holds at 1.0 to f32
// rounding (Phase 55's own payout, truncated at the clamp, drifted the mean
// by 0.7 % at gain 10 — on a food-supply field that drift is a global change
// of the decomposition rate, exactly what the net-zero contract forbids).
/// Cells within this radius of a structure feel its decomposition imprint.
pub const STRUCTURE_DECO_RADIUS: i32 = 3;
/// Compensation ring of the decomposition dipole (same geometry as Phase 55:
/// 7×7 disk against the 11×11 ring).
pub const STRUCTURE_DECO_RING: i32 = 5;
/// Per-tick relaxation of the decomposition multiplier toward 1.0
/// (half-life ≈ 350 ticks, the Phase 55 value — a cultural larder must be
/// maintained by fresh placements to persist).
pub const STRUCTURE_DECO_DECAY: f32 = 0.998;
/// Bounds of the per-cell decay-rate multiplier.
pub const STRUCTURE_DECO_MIN: f32 = 0.5;
pub const STRUCTURE_DECO_MAX: f32 = 1.5;
/// Per-structure imprint scale: the raw token grass_mod is ±0.05
/// (STRUCTURE_MOD_STRENGTH), so ×20 pins the disk immediately at its bound
/// (0.5× or 1.5× decay rate) — a placed structure halves or doubles a
/// carcass's lifetime outright.
///
/// CALIBRATION (Phase 56, 80k, seed 20261102; OFF arm = 2 speciation events):
/// gain 3 → 0 events, gain 10 → 0 events, gain 20 → **3 events**. The failure
/// at low gain is the Phase 54/55 failure mode with a new face: a gentle
/// decomposition mosaic simply feeds the DECO guild up to its 80-slot cap
/// (g3/g10 end at deco = 80, one clade), so the population is *more* stable
/// than the OFF arm's 60. Only saturated contrast fragments the food supply
/// hard enough to cost population (g20: deco 34–45) and to isolate demes
/// (DECO absent from region 0 by t=80k, three forks).
pub const STRUCTURE_DECO_GAIN: f32 = 20.0;
/// Phase 56 A/B: structures reshape the decomposition field. **Default ON**
/// (re-baseline 15) — the first structure→speciation coupling that passes the
/// A/B bar in four phases: 80k ON 3 speciation events vs OFF 2; 200k ON 4
/// (t=23.7k/39.5k/76.4k/157.1k) vs OFF 4, plus an extinction event at 82.0k
/// the OFF arm never produces. The pre-56 anchors are preserved by the
/// `deco_resource_env_off` arm (see `phase56_deco_resource_test`).
pub const DECO_RESOURCE_ENV_ON: bool = true;

// ----- Phase 60: regional DECO carrying capacity (split trigger) -------------
//
// Phase 59 removed the old split engine's hidden driver: sterile low-efficiency
// individuals used to hold slots while never breeding, which kept the DECO guild
// below its 80-slot cap and fragmented its breeding pool. With the fertility
// fix in place the guild sits at 80 and, on seed 20261102, collapses into one
// home deme (69 decomposers in region 3 at t=80k) whose mating network stays
// well mixed — the detector sees no 8+8 genomic clusters with geographic
// isolation, so speciation goes to zero.
//
// This mechanism restores fragmentation on a capacity axis instead of a
// fertility axis. `deco_env` is the carcass decay multiplier: a cell above 1.0
// rots a carcass faster, so its larder is worth less to a decomposer. The mean
// multiplier over a region's habitable cells therefore measures how much food
// that region retains, and the region's capacity weight is
//
//   w_r = clamp(1 + GAIN · (1 − mean_env_r), W_MIN, W_MAX)
//
// Each DECO kind's Phase-49 budget is distributed over the four home regions in
// proportion to `w_r`: `cap_r = budget · w_r / Σw`. Summed over regions this is
// exactly the kind's budget, so the guild's biomass conservation is untouched
// and a daughter fork receives its own regional share instead of competing
// with the root for the same cells. A region whose culture accelerated decay
// (or whose soil is otherwise poor) carries fewer decomposers — a capacity
// hollow, not a sterile individual.
//
// Contract: this is a spawn-slot capacity, never an energy threshold. A blocked
// mother keeps her energy, remains fully fertile, and breeds as soon as her
// region has a free slot; no legal `eff` can become unreachable. The weights
// are a pure function of the current field (recomputed at the first DECO birth
// gate of a tick), so the arm needs no save-state field.
/// Phase 60 A/B: **default ON** (re-baseline 17). The 3-seed 200k A/B
/// (`research/phase60-report.md`): 22 speciation events vs the Phase 59
/// default's 8, with every low-efficiency daughter clade fertile
/// (`deco_ready > 0`); extinctions are long-lived capacity clades, not
/// birth-cliff sterile daughters. The pre-60 world is preserved by
/// `deco_region_cap_off()` (Phase 59 default anchor @60000).
pub const DECO_REGION_CAP_ON: bool = true;
/// Phase 60: weight gain per unit of regional mean decay multiplier. At the
/// uniform field (`mean = 1`) every region weighs 1.0 and the budget splits
/// evenly; the observed calibration range is `mean ∈ [0.97, 1.04]`.
pub const DECO_REGION_CAP_GAIN: f64 = 10.0;
/// Phase 60: weight bounds. A region can lose at most 75 % of its uniform
/// share and gain at most 4×; the bounds keep one extreme region from
/// starving the whole guild when a culture monopolises the field.
pub const DECO_REGION_CAP_W_MIN: f64 = 0.25;
pub const DECO_REGION_CAP_W_MAX: f64 = 4.0;
/// Phase 61 A/B: renormalise the regional split over the regions a kind
/// actually occupies instead of all four. The Phase 60 form
/// `cap_r = budget · w_r / Σ_all w` gives a clade that lives in a single
/// home quadrant at most `budget / 4` slots there — a daughter that forks off
/// with 8 members into one region is born at its cap and can never breed, so
/// it dies by attrition. Renormalising over the occupied set makes a
/// one-region clade's cap its whole budget while keeping the conservation
/// identity: summed over the *occupied* regions the caps still equal the kind
/// budget exactly (empty regions hold no members by definition).
///
/// The target region always participates in the normaliser even when it is
/// still empty, so colonising a vacant quadrant draws on that quadrant's own
/// share rather than the whole budget.
///
/// Compiled default OFF: the 3-seed 200k A/B (`research/phase61-report.md`)
/// halves extinctions (13 → 6) but collapses speciation on seed 20261102
/// (9 → 0, the guild refills to 80): the root's four-quadrant partition is
/// the speciation engine and renormalising the *root* removes it.
pub const DECO_REGION_CAP_RENORM_ON: bool = false;
/// Phase 61 A/B candidate (b): a minimum retention share for a kind in the
/// regions it already occupies. The fork gate declares a species only when
/// both sides hold at least `speciation_min_side` members (Phase 53), but the
/// Phase 60 regional split can hand a freshly forked daughter as little as
/// `budget / 4` slots per quadrant — 2 slots for an 8-budget granddaughter.
/// The split gate and the capacity gate then disagree: a species is declared
/// at 8 and permanently held below 8. This arm raises the per-region cap to
/// `max(natural share, min(budget, speciation_min_side))` for regions the
/// kind already inhabits, so an occupied region can always hold the split
/// gate's minimum (never more than the kind's own budget).
///
/// Vacant quadrants keep the plain Phase 60 share, so colonisation and the
/// root's four-deme structure are unchanged; the family-level biomass bound
/// is untouched because the Phase 49 per-kind budget gate still caps every
/// kind's total.
///
/// Compiled default OFF: the A/B (`research/phase61-report.md`) gives the best
/// trade-off of the three candidates (extinctions 13 → 4, speciation 22 → 20)
/// but still misses the "speciation must not fall" bar by 2 events, so the arm
/// stays off until a 5-seed test can judge that difference.
pub const DECO_REGION_CAP_FLOOR_ON: bool = false;
/// Phase 62 sensitivity arm (d): apply candidate (b)'s floor only to kinds
/// whose own budget is below `2 × speciation_min_side` (16). Every DERIVED
/// kind has a budget of at most `FORK_BUDGET_GRANT` (20), so the plain floor
/// lifts every daughter's occupied-region cap above its natural share and the
/// regional gate stops binding for most of them — the 5-seed re-run shows the
/// fork rate rising with it (seed 20261107: 7 → 11 forks) and extinctions
/// rising with that (1 → 8, `research/phase62-report.md`). This arm keeps the
/// Phase 60 split for the 20-budget first-generation daughters and only
/// unpinches the 15/11/8-budget grandchildren, isolating "which generation
/// does the squeezing kill" from "does the floor exist at all". Only
/// meaningful together with `deco_region_cap_floor`; not persisted.
pub const DECO_REGION_CAP_FLOOR_SMALL_ON: bool = false;

/// Phase 60: grid cell → home quadrant. Pure geometry (cell centres through
/// `Ecology::region_of`), built once per world and rebuilt on restore; never
/// saved. Making it a table keeps the per-tick weight refresh to 5 776
/// lookups instead of 5 776 `atan2` calls.
fn build_deco_cell_region_map() -> Vec<u8> {
    let n = (GRID_N * GRID_N) as usize;
    let cell = GRID_SPAN / GRID_N as f64;
    let mut out = Vec::with_capacity(n);
    for j in 0..n {
        let col = (j % GRID_N as usize) as f64;
        let row = (j / GRID_N as usize) as f64;
        let x = -GRID_SPAN / 2.0 + (col + 0.5) * cell;
        let z = -GRID_SPAN / 2.0 + (row + 0.5) * cell;
        out.push(Ecology::region_of(x, z));
    }
    out
}
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

// ----- Phase 51: niche resource partition -----------------------------------
//
// Phase 50's residual problem: the granddaughter lineages of a radiating
// ancestor (deco-A-*) still died out one by one. The niche axis carried only
// a competition burden — every daughter foraged the same resource surface as
// its parent, so separation never became a separate living.
//
// Phase 51 extends the semantics: niche_offset is also a resource-preference
// coordinate. Beyond a band, a clade's members forage more efficiently in
// cells whose LOCAL resource mix matches their side of the axis (grass-rich
// for +, mineral-rich for −) and slightly less efficiently in the opposite
// mix. The weight is a soft efficiency multiplier — never a hard switch, so
// no absorbing state can form — and it keys off the local gradient (cell
// grass vs mineral relative richness), so the effect only shows up where the
// landscape is heterogeneous (Phase 45's regrowth-noise field).
/// |niche| below this is the neutral band (Phase 50 behaviour for that clade).
pub const NICHE_PARTITION_BAND: f64 = 0.3;
/// Maximum foraging-efficiency swing of a fully banded clade (±15 %).
pub const NICHE_PREF_GAIN: f64 = 0.15;
/// Phase 51: is niche resource partition active? Default ON; the A/B arm
/// (`niche_partition_off`) restores the Phase 50 foraging trajectory.
pub const NICHE_PARTITION_ON: bool = true;

// ----- Phase 47: natal dispersal (geographic isolation maintenance) --------
//
// Phase 46 showed the first split is a one-shot event: once niche_offset
// separates the two clades, competition vanishes and the tree freezes. The
// missing ingredient is population structure. Natal dispersal gives the
// world the standard philopatry/LDD mixture (Greenwood 1980): most juveniles
// establish their home range at the natal site, a minority colonise a distant
// habitat patch. Rare long-distance founding keeps local demes partially
// isolated between colonisation waves, so drift can diverge them past the
// speciation gate without any scripted selection.
/// Per-birth probability that the juvenile's home range is founded in a
/// different region (a vacant island) instead of the natal region. Default is
/// the low-dispersal calibration; the field is public so the sweep test can
/// set it per arm.
pub const DISPERSAL_P: f64 = 0.05;
/// Distance band of a dispersal event (world units). The lower bound sits
/// well above GEO_ISOLATION_DIST so a founded deme starts as a geographic
/// outlier rather than a neighbour of the natal one.
pub const DISPERSAL_MIN_DIST: f64 = 180.0;
pub const DISPERSAL_MAX_DIST: f64 = 420.0;
/// Bounded rejection sampling for a habitat-legal landing cell. Each try
/// consumes two dedicated-stream draws; falling through after this many
/// misses returns the natal site (a failed disperser stays home).
const DISPERSAL_TRIES: usize = 8;
/// A land disperser must land at least this far (current-position clearance)
/// from every live conspecific, otherwise the "founder" would be dropped
/// inside the source cloud and be reabsorbed by gene flow. 100 = the
/// detector's own geographic-isolation gate, so a successful founder start is
/// by construction a geographic outlier.
const DISPERSAL_CLEARANCE: f64 = GEO_ISOLATION_DIST;
/// Dedicated dispersal stream (`seed ^ DISPERSAL_RNG_TAG`). Keeping it off
/// the main/prng/fish streams means `dispersal_p=0` is a bit-exact no-op and
/// the river of RNG draws every other subsystem sees never depends on how
/// many juveniles dispersed.
pub const DISPERSAL_RNG_TAG: u32 = 0x44_49_53_50; // "DISP"
/// Phase 47: number of home-range regions (geographic demes) in the island
/// model. Initial members of every kind are split into equal-sized angular
/// sectors; mating is region-local and migration is the only cross-region
/// gene flow.
pub const REGION_COUNT: u8 = 4;
/// Phase 47: DECO mutation-amplitude multiplier. Small island demes (≈20
/// individuals) need a faster drift clock than the panmictic 80-agent world
/// to cross the speciation gap inside a 200k-tick horizon. Applied to every
/// DECO-family birth; other kinds keep their legacy amplitude.
pub const DECO_MUT_SCALE: f64 = 5.0;
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
    /// Phase 34: registry kind id of this clade's family (root clades carry
    /// the ancestor kind 0-5; daughter clades carry the kind minted at fork
    /// time). `u32::MAX` only appears on clades restored from pre-v8 saves
    /// whose kind could not be resolved (backfilled from the name).
    pub kind_id: u32,
    /// Phase 34: the derived kind id minted when this clade forked off its
    /// parent (`None` for root clades and for clades restored from pre-v8
    /// saves, where no kind fork existed). Identity source: existing members
    /// keep their old kind; births INTO this clade carry this kind id.
    pub derived_kind: Option<u32>,
    /// Phase 34: trait axes active for this clade's detection pass
    /// ("speed", "sense", "size", optionally "ext", "deco_eff").
    pub axes: Vec<&'static str>,
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
    /// Phase 34: registry kind id of the affected clade (speciation: the
    /// minted derived kind; extinction/exclusion: the dying clade's kind).
    /// Additive C-ABI tail; defaults to u32::MAX on pre-Phase-34 records.
    pub kind_id: u32,
    /// Phase 49 (speciation events only): the forking kind's budget share
    /// before the fork and the first child's grant. Extinction events keep
    /// 0/0. Additive JSON/ABI tail; defaults to 0 on pre-Phase-49 records.
    pub parent_budget: u32,
    pub child_budget: u32,
}

/// Phase 61 engine-level per-clade life bookkeeping (Phase 59 §10.5): births,
/// deaths by cause, birth-refusal reasons, mate-search misses, peak standing
/// population and the extinction tick. Keyed by `(clade_id, root_kind)`
/// because clade id 0 doubles as the "unassigned" sentinel — an agent that
/// has not been attached to a materialized clade yet books into the
/// `(0, family)` slot, which then merges with the root clade when it
/// materializes (the root always takes id 0).
///
/// Pure telemetry: no RNG draws, no state the simulation reads back, and never
/// persisted. Deaths and blocks counted here are exactly the events the engine
/// already decided, so a run with a fresh counter vector is bit-identical.
#[derive(Clone, Debug, Default)]
pub struct CladeLifeStats {
    pub clade_id: u32,
    pub root_kind: SpeciesKind,
    pub births: u64,
    pub deaths: u64,
    pub deaths_starved: u64,
    pub deaths_aged: u64,
    pub deaths_predation: u64,
    pub deaths_other: u64,
    /// Births refused by the Phase 60 regional slot gate (mother's clade).
    pub region_cap_blocks: u64,
    /// Births refused by the Phase 49 kind budget / biome cap gate.
    pub kind_budget_blocks: u64,
    /// Ready-adult ticks whose mate search came back empty (one count per
    /// failed search, not per tick: the search only runs when the mother is
    /// past cooldown and over her energy threshold).
    pub mate_misses: u64,
    /// Phase 66 diagnostic: why the search came back empty (mutually
    /// exclusive, classified by the fix that would rescue the pair).
    pub mate_miss_no_kin: u64,
    pub mate_miss_other_region: u64,
    pub mate_miss_beyond_radius: u64,
    pub mate_miss_filtered: u64,
    pub mate_miss_other: u64,
    /// Phase 66 diagnostic: the same total restricted to clades younger than
    /// `YOUNG_CLADE_TICKS` at the miss (the "new species" window the phase is
    /// about), with its own cause split.
    pub mate_miss_young: u64,
    pub mate_miss_young_no_kin: u64,
    pub mate_miss_young_other_region: u64,
    pub mate_miss_young_beyond_radius: u64,
    pub mate_miss_young_filtered: u64,
    pub mate_miss_young_other: u64,
    /// Sums of the mother's kind/region population at each miss (÷ misses =
    /// mean density the search ran in).
    pub mate_miss_kind_pop_sum: u64,
    pub mate_miss_region_pop_sum: u64,
    /// Distance to the nearest *fertile* same-region conspecific on
    /// `beyond_radius` misses: what search radius the miss would have needed.
    pub mate_miss_ready_dist_sum: f64,
    pub mate_miss_ready_dist_min: f64,
    /// Births into this clade that fired natal dispersal (the family left its
    /// home quadrant; see `relocate_founders`).
    pub dispersal_out: u64,
    /// Deaths whose body lay outside the agent's own `region` tag — the
    /// stranded-founder signature of family dispersal.
    pub deaths_off_region: u64,
    pub first_tick: u32,
    pub first_birth_tick: Option<u32>,
    pub last_birth_tick: Option<u32>,
    pub peak_pop: u32,
    pub peak_tick: u32,
    /// Per-home-quadrant peak standing population of this clade (spatial
    /// evidence for a capacity death: the most slots it ever held per
    /// quadrant).
    pub region_peak: [u32; REGION_COUNT as usize],
    pub extinction_tick: Option<u32>,
    /// Sums over this clade's deaths (lifespan and end-of-life energy
    /// evidence: `death_age_sum / deaths`, `death_energy_sum / deaths`).
    pub death_age_sum: f64,
    pub death_energy_sum: f64,
    pub last_death_energy: f64,
}

/// Phase 66: the constraint that makes a ready adult's mate search fail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MateMissBucket {
    NoKin,
    OtherRegion,
    BeyondRadius,
    Filtered,
    Other,
}

impl CladeLifeStats {
    /// Phase 67: lifetime used as the P3 denominator. `now` is the run end
    /// (200 000 in the 5-seed A/B); extinct clades stop at their death tick.
    pub fn life_ticks(&self, now: u32) -> u64 {
        self.extinction_tick.unwrap_or(now).saturating_sub(self.first_tick) as u64
    }

    /// Phase 67 P3: mate misses per clade-tick (0 for an empty lifetime). The
    /// denominator is the clade's own lifetime, so arms with different clade
    /// counts (Phase 66 §11.1) are comparable.
    pub fn mate_miss_rate_per_tick(&self, now: u32) -> f64 {
        let life = self.life_ticks(now);
        if life == 0 { 0.0 } else { self.mate_misses as f64 / life as f64 }
    }

    /// Phase 67 P3: one bucket per clade-tick, on the same denominator as the
    /// total, so bucket shares stay comparable across arms.
    pub fn mate_miss_bucket_rate_per_tick(&self, bucket: MateMissBucket, now: u32) -> f64 {
        let life = self.life_ticks(now);
        if life == 0 { return 0.0; }
        let misses = match bucket {
            MateMissBucket::NoKin => self.mate_miss_no_kin,
            MateMissBucket::OtherRegion => self.mate_miss_other_region,
            MateMissBucket::BeyondRadius => self.mate_miss_beyond_radius,
            MateMissBucket::Filtered => self.mate_miss_filtered,
            MateMissBucket::Other => self.mate_miss_other,
        };
        misses as f64 / life as f64
    }
}

/// Phase 61: which spawn gate refused a birth (telemetry only).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CladeBirthBlock {
    RegionCap,
    KindBudget,
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
    /// Phase 34: per-kind speciation time gate (index = registry kind id).
    /// Replaces the single `last_clade_spec_tick` (kept for save compat) so
    /// each kind's detector runs on its own clock; padded in `register_kind`.
    pub last_spec_tick_by_kind: Vec<u32>,
    /// Phase 34: lineage labels of the most recent legacy HERB speciation
    /// event (side "a" = the low-speed cluster, side "b" = the high-speed
    /// cluster). The drain/bite instrument keys on these (the retired
    /// 迅聚/缓独 constants' dynamics, now deterministically derived names).
    pub herb_lineage_a: Option<String>,
    pub herb_lineage_b: Option<String>,
    /// Phase 35: global energy ledger (C10 closure). Pure bookkeeping —
    /// every method on it is side-effect-free outside the ledger struct,
    /// so the checksum trajectory is unaffected by its presence alone.
    pub ledger: EnergyLedger,
    /// Phase 36: brain shadow network toggle. Default `BRAIN_ON`; per-instance
    /// so A/B tests can flip it at runtime without a compile-time change.
    pub brain_on: bool,
    /// Phase 36: brain-upkeep drain toggle (`BRAIN_UPKEEP_ON` default). When
    /// on, every tick drains `BRAIN_PER_NEURON_COST × n_hidden` from each
    /// agent's energy and books it into `ledger.brain_cost`.
    pub brain_upkeep_on: bool,
    /// Phase 36: Landauer information-cost toggle (`INFO_COST_ON` default).
    /// Read by `charge_info_cost`; kept as a field so tests can A/B it while
    /// the default world pays for perception.
    pub info_cost_on: bool,
    /// Phase 36: `TROPHIC_EFF` wiring toggle. When on, the PRED bite gain is
    /// multiplied by `TROPHIC_EFF` (Lindeman 1942 transfer efficiency),
    /// replacing the Phase 35 placeholder.
    pub trophic_eff_active: bool,
    /// Phase 36: dual-controller switchover gate. Default off (FSM stays
    /// authoritative); when on, the brain may take authority at a
    /// speciation-check boundary once `T_EQUIV` agreement holds.
    pub brain_switch_enabled: bool,
    /// Phase 36: global learning telemetry (pure observability).
    pub brain_learning: BrainLearningStats,
    /// Phase 41: triune-brain toggles. `triune_on` runs the three-layer
    /// shadow (decision + learning); `triune_upkeep_on` charges its energy.
    pub triune_on: bool,
    pub triune_upkeep_on: bool,
    /// Phase 42: 离线经验回放开关（A/B 对照可运行期翻转）。
    pub replay_on: bool,
    /// Phase 41: triune-brain aggregate telemetry (observability only).
    pub triune_stats: TriuneLearningStats,
    /// Phase 36: dedicated RNG stream for brain structural mutation
    /// (`seed ^ BRAIN_RNG_TAG`). Never mixed with `rng`/`prng`/`fish_rng`,
    /// so the main/prng/fish streams stay bit-identical.
    pub brain_learning_rng: Mulberry32,
    /// Phase 37: knowledge token registry (open, growable). Default empty.
    pub token_registry: crate::culture::CultureRegistry,
    /// Phase 37: physical structure library (niche construction载体). Default empty.
    pub structure_registry: crate::culture::StructureRegistry,
    /// Phase 37: culture toggle (default on — formal dynamics stage, expects re-baseline).
    pub culture_on: bool,
    /// Phase 37: dedicated RNG stream for cultural transmission/innovation
    /// (`seed ^ TOKEN_RNG_TAG`). Never mixed with other streams.
    pub culture_rng: Mulberry32,
    /// Phase 37: global culture learning telemetry (pure observability).
    pub culture_stats: crate::culture::CultureStats,
    /// Phase 37: propagation fidelity parameter ∈ [0,1].
    pub culture_fidelity: f64,
    /// Phase 37: default bias channel for social learning.
    pub culture_bias_channel: u8, // 0=content, 1=prestige, 2=conformist, 3=all
    /// Phase 43: birth-time cultural inheritance toggle (default `TEACH_ON`).
    /// When false, birth teaching is a bit-exact no-op (zero draws, zero cost).
    pub teach_on: bool,
    /// Phase 43: abiogenesis toggle (default `ABIOGENESIS_ON`).
    pub abiogenesis_on: bool,
    /// Phase 43: dedicated abiogenesis stream (`seed ^ ABIO_RNG_TAG`).
    pub abio_rng: Mulberry32,
    /// Phase 43: cumulative abiogenesis events (telemetry).
    pub abiogenesis_events: u64,
    /// Phase 43: tick of the most recent abiogenesis event (telemetry).
    pub abiogenesis_ticks: u32,
    /// Phase 39.1: live group entities (behavioural aggregates with their own
    /// energy pool, reproduction and death). Empty ⇒ individual-level
    /// selection only.
    pub groups: Vec<crate::group::Group>,
    /// Phase 39.1: group id allocator (never reused).
    pub next_group_id: u32,
    /// Phase 39.1: group layer toggle (default `GROUP_ON`). When false the
    /// layer is a bit-exact no-op (zero draws, zero energy) — the A/B arm
    /// for every group acceptance test.
    pub group_on: bool,
    /// Phase 39.1: dedicated RNG stream (`seed ^ GROUP_RNG_TAG`) for group
    /// formation probability and propagule placement jitter.
    pub group_rng: Mulberry32,
    /// Phase 39.1: group-layer telemetry (observability only).
    pub group_stats: crate::group::GroupStats,
    /// Phase 39.1: persistence trackers for cohesion candidates (same anchor
    /// + clade seen across consecutive scans). Bookkeeping only.
    pub(crate) group_pending: Vec<crate::group::CohesionTracker>,
    /// Phase 45: patch disturbance toggle (default `PATCH_ON`).
    pub patch_on: bool,
    /// Phase 45: per-tick patch disturbance probability.
    pub patch_p: f64,
    /// Phase 45: dedicated patch stream (`seed ^ PATCH_RNG_TAG`).
    pub patch_rng: Mulberry32,
    /// Phase 45: cumulative patch events (telemetry).
    pub patch_events: u64,
    /// Phase 45: tick of the most recent patch event (telemetry).
    pub patch_last_tick: u32,
    /// Phase 45: world-x of the most recent patch center (telemetry).
    pub patch_last_x: f64,
    /// Phase 45: world-z of the most recent patch center (telemetry).
    pub patch_last_z: f64,
    /// Phase 45: static per-cell regrowth-noise multiplier (fixed-seed field,
    /// identical across worlds/restores — a constant of the model).
    pub regrow_noise: Vec<f32>,
    /// Phase 54: per-cell structure regrowth multiplier (starts at 1.0;
    /// structures raise it, STRUCTURE_ENV_DECAY pulls it back).
    pub structure_env: Vec<f32>,
    /// Phase 54 A/B toggle.
    pub structure_env_on: bool,
    /// True while any cell multiplier is above 1.0 (skips the decay pass
    /// once the field has fully relaxed).
    pub structure_env_decay_active: bool,
    /// Phase 56: per-cell carcass decay multiplier (starts at 1.0; structures
    /// push it up/down under the clamp, STRUCTURE_DECO_DECAY relaxes it back).
    pub deco_env: Vec<f32>,
    /// Phase 56 A/B toggle.
    pub deco_env_on: bool,
    /// True while any cell multiplier differs from 1.0 (skips the decay pass).
    pub deco_env_decay_active: bool,
    /// Phase 58: decomposition-multiplier bounds — the compiled constants
    /// unless an experiment arm overrides them (`deco_env_clamp_on`). Not
    /// persisted: a restored world always falls back to the constants, which
    /// is the only configuration the default trajectory uses.
    pub deco_env_min: f32,
    pub deco_env_max: f32,
    /// Phase 57: sign-neutral token value — placement gain reads
    /// `|grass_mod| × 10`, so a token's environment direction carries no
    /// energy advantage. Experimental arm only (`TOKEN_VALUE_SIGN_NEUTRAL`
    /// is false: the default world keeps the Phase 56 signed reward).
    pub token_value_sign_neutral: bool,
    /// Phase 58 design b arm: placements stamp the highest-value token
    /// (`use_gain`) instead of `known_tokens[0]`. Default false — the
    /// inheritance-order pick is the Phase 56/57 world.
    pub token_placement_by_value: bool,
    /// Phase 59 A/B toggle: true = the saturated fertility mapping (the fix),
    /// false = the Phase 58 legacy line (`repro_threshold_old_off`). Not
    /// persisted, same contract as the Phase 58 clamp: a restored world runs
    /// the compiled default.
    pub deco_repro_saturate: bool,
    /// Phase 60 A/B toggle: distribute each DECO kind's carrying budget over
    /// the four home regions by local food retention. Off = the Phase 59
    /// world (global budget only). Not persisted (same contract as the Phase
    /// 58 clamp / Phase 59 mapping arms): a restored world runs the compiled
    /// default, and the weights below are re-derived from the restored field.
    pub deco_region_cap: bool,
    /// Phase 60: per-home-region capacity weights, recomputed at the first
    /// DECO birth gate of a tick when the arm is on. Pure function of the
    /// current `deco_env` field; never saved.
    pub deco_region_weight: [f64; REGION_COUNT as usize],
    /// Phase 60: Σ of the four region weights (the budget normaliser).
    pub deco_region_weight_sum: f64,
    /// Phase 60: tick of the cached weights (stale-check for the per-tick
    /// on-demand refresh).
    pub deco_region_weight_tick: u32,
    /// Phase 60: grid cell → home quadrant, a pure function of the grid
    /// geometry; rebuilt on construction / restore, never saved.
    pub deco_cell_region: Vec<u8>,
    /// Phase 61 A/B toggle: renormalise the regional split over the regions
    /// the kind actually occupies, so a clade that lives in one quadrant can
    /// use its whole budget there instead of `budget / 4`. Not persisted (same
    /// contract as the other arm switches): a restored world runs the compiled
    /// default.
    pub deco_region_cap_renorm: bool,
    /// Phase 61 candidate (b): raise a kind's cap in the regions it already
    /// occupies to at least `min(budget, speciation_min_side)`, so an occupied
    /// quadrant can always hold the split gate's minimum. Not persisted.
    pub deco_region_cap_floor: bool,
    /// Phase 62 sensitivity arm (d): restrict that floor to kinds with a
    /// budget below `2 × speciation_min_side`. Not persisted.
    pub deco_region_cap_floor_small: bool,
    /// Phase 61 candidate (c): renormalise only the derived kinds (a forked
    /// kind's regional split covers the regions it occupies), while the six
    /// ancestor kinds keep the plain Phase 60 four-quadrant split. The root
    /// DECO deme partition is what fragments the guild into the daughter
    /// clusters the detector needs — candidate (a) renormalises it away and
    /// speciation collapses — so this arm leaves the root untouched and only
    /// unpinches the daughters. Not persisted.
    pub deco_region_cap_renorm_daughters: bool,
    /// Phase 61 runtime telemetry: per-clade birth/death counters. Pure
    /// observation — not persisted, never read by the simulation.
    pub clade_life: Vec<CladeLifeStats>,
    /// Phase 45: per-kind clade-isolation telemetry (kind id → rolling
    /// window). Pure observability — not persisted (same contract as
    /// GroupStats); rebuilt from live clades after a restore.
    pub isolation_by_kind: Vec<(u32, Vec<IsolationHistoryPoint>)>,
    /// Phase 45: 文化价值选择开关（inherit/learn 按 use_gain 排序 + 保真度
    /// 加成）。false = Phase 44.1 行为（插入序 + 基础保真度）——A/B 对照臂；
    /// 价值 EMA 记账始终开启（零 RNG 零能量，不影响轨迹）。
    pub culture_value_on: bool,
    /// Phase 47: natal dispersal probability (0.0 = bit-exact Phase-46 arm).
    pub dispersal_p: f64,
    /// Phase 47: dedicated dispersal stream (`seed ^ DISPERSAL_RNG_TAG`).
    pub dispersal_rng: Mulberry32,
    /// Phase 47: successful distant founding events (telemetry).
    pub dispersal_events: u64,
    /// Phase 47: tick of the most recent successful dispersal (telemetry).
    pub dispersal_last_tick: u32,
    /// Phase 47: region-island isolation toggle (same-region mating). Default
    /// ON; persisted so a save can freeze the island model.
    pub region_isolation: bool,
    /// Phase 47: DECO mutation-amplitude multiplier (default `DECO_MUT_SCALE`).
    /// Per-instance so mechanism-isolation tests can pin the legacy amplitude;
    /// the default world uses the island calibration.
    pub deco_mut_scale: f64,
    /// Phase 49: fork-budget toggle (independent resource share per kind).
    /// Default `BUDGET_FORK_ON`; `budget_fork_off()` restores the Phase 47
    /// family-shared-cap behavior bit-exactly.
    pub budget_fork: bool,
    /// Phase 49: per-kind carrying-capacity share (index = registry kind id;
    /// root kinds start at their `global_cap`, derived kinds at their fork
    /// grant). The guild total is conserved: grants move budget parent→child,
    /// extinctions move it back up the fork lineage.
    pub kind_budgets: Vec<usize>,
    /// Phase 63 candidate G: earn extra share from the parent by surviving
    /// (`FORK_BUDGET_GROWTH_ON`). Unlike the Phase 61/62 region arms this is
    /// persisted: the earned shares live in `kind_budgets`, so the switch must
    /// travel with the world or a restore would freeze the growth midway.
    pub budget_growth: bool,
    /// Phase 63: slots this kind has already earned through growth (index =
    /// kind id; zero for roots and for kinds that never grew). Persisted.
    pub kind_growth_grants: Vec<usize>,
    /// Phase 65 W1: fork-grace arm switch (`FORK_GRACE_ON`). Persisted: the
    /// window changes future birth decisions, so a restored world must keep it.
    pub budget_grace: bool,
    /// Phase 65 W1: tick at which each derived kind's grace window closes
    /// (index = kind id; 0 = none). Persisted.
    pub kind_grace_until: Vec<u32>,
    /// Phase 65 W1: grace window length for this instance (arm knob).
    pub grace_ticks: u32,
    /// Phase 65 W1 telemetry (transient, like `clade_life`): births admitted
    /// while the kind was already at its grant — the legacy gate would have
    /// refused them.
    pub kind_grace_skips: Vec<u64>,
    /// Peak `count(kind) − budget` observed at an admitted grace birth.
    pub kind_grace_peak_over: Vec<usize>,
    /// Slots settled out of the parent at window close.
    pub kind_grace_settled: Vec<usize>,
    /// Overdraft the parent could not pay (dead/poor parent, or a clade below
    /// MIN_SIDE). Recorded, never minted.
    pub kind_grace_unpaid: Vec<usize>,
    /// Phase 66 R1/R2 and Phase 67 R3 arm: 0 = legacy radius (default),
    /// 1 = R1, 2 = R2, 3 = R3. Arm knob, not persisted (read-back lands OFF).
    pub mate_radius_arm: u8,
    /// Phase 66 D0 telemetry (transient): mate searches whose radius the floor
    /// raised above the legacy value.
    pub radius_boost_searches: u64,
    /// Phase 66 D0 telemetry (transient): successful finds that lay beyond the
    /// legacy radius (only the floor made them possible).
    pub radius_only_finds: u64,
    /// Phase 67 D0 telemetry (transient): distance sum of the finds only the
    /// floor made possible (÷ `radius_only_finds` = mean rescue distance).
    pub radius_only_find_dist_sum: f64,
    /// Phase 63 candidates M/A: floor of the parent's post-grant remainder and
    /// floor of the daughter's grant. Both default to `FORK_BUDGET_MIN` (the
    /// Phase 49 contract); the arms override them per instance and the values
    /// are not persisted (same contract as the Phase 61 region arms).
    pub fork_budget_min: usize,
    pub fork_daughter_min: usize,
    /// Phase 49: minimum members on BOTH sides of a cut for a split to count
    /// as a species (default `SPECIATION_MIN_SIDE_DEFAULT`).
    pub speciation_min_side: usize,
    /// Phase 51: niche resource-partition toggle (default `NICHE_PARTITION_ON`).
    /// `niche_partition_off()` restores the Phase 50 foraging trajectory
    /// bit-exactly (the preference multiplier short-circuits to 1.0).
    pub niche_partition: bool,
    /// Phase 53: generational niche accumulation (A/B: `niche_gen_accum_off`
    /// restores the Phase 52 projection).
    pub niche_gen_accum: bool,
    /// Phase 51: per-band member census (telemetry; rebuilt on every detection
    /// pass). Not persisted — observability only, same contract as GroupStats.
    pub niche_partition_share: crate::niche::NichePartitionShare,
    /// Phase 51: O(1) niche lookup by clade id for the foraging hot path
    /// (index = clade id; 0.0 for unassigned / unknown ids). Rebuilt on every
    /// detection pass and after a restore — a pure cache of `species_clades`.
    pub(crate) niche_by_clade: Vec<f64>,
}

fn default_fidelity() -> f64 {
    use crate::culture::DEFAULT_FIDELITY;
    DEFAULT_FIDELITY
}

fn default_bias_channel() -> u8 {
    3 // default to all channels enabled
}

impl Ecology {
    pub fn new(seed: u32) -> Self {
        let n = (GRID_N * GRID_N) as usize;
        let kinds = KindRegistry::legacy();
        let evolution: Vec<Vec<EvolutionSample>> = (0..kinds.len()).map(|_| Vec::new()).collect();
        let generation_fitness: Vec<Vec<FitnessRecord>> = (0..kinds.len()).map(|_| Vec::new()).collect();
        // Phase 49: every registered kind starts with its own global cap as
        // resource share (root kinds; derived kinds are re-granted at fork).
        let kind_budgets: Vec<usize> = kinds.defs().iter().map(|d| d.global_cap).collect();
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
            // Phase 34: per-kind time gate (padded in register_kind to the
            // registry length) + retired-script lineage labels (empty until the
            // first legacy HERB speciation event).
            last_spec_tick_by_kind: Vec::new(),
            herb_lineage_a: None,
            herb_lineage_b: None,
            ledger: EnergyLedger::new(),
            // Phase 36: evolvable micro-brain defaults — the shadow network is
            // ON, neuron upkeep and the Landauer info cost are ON, TROPHIC_EFF
            // is wired into the PRED bite gain, and the FSM stays authoritative
            // (the dual-controller switchover gate is off).
            brain_on: BRAIN_ON,
            brain_upkeep_on: BRAIN_UPKEEP_ON,
            info_cost_on: INFO_COST_ON,
            trophic_eff_active: true,
            brain_switch_enabled: BRAIN_SWITCH_ENABLED,
            brain_learning: BrainLearningStats::default(),
            brain_learning_rng: Mulberry32::new(seed ^ BRAIN_RNG_TAG),
            // Phase 41: triune brain runs every tick and pays real upkeep.
            // Phase 42: it also takes behaviour authority once its own
            // FSM-agreement gate (and confidence floor) is met.
            triune_on: true,
            triune_upkeep_on: true,
            replay_on: REPLAY_ON,
            triune_stats: TriuneLearningStats::default(),
            // Phase 37: culture layer — ON by default for this dynamics stage,
            // with a dedicated RNG stream separate from all ecology streams.
            culture_on: crate::culture::CULTURE_ON,
            token_registry: crate::culture::CultureRegistry::new(),
            structure_registry: crate::culture::StructureRegistry::new(),
            culture_rng: Mulberry32::new(seed ^ crate::culture::TOKEN_RNG_TAG),
            culture_stats: crate::culture::CultureStats::default(),
            culture_fidelity: crate::culture::DEFAULT_FIDELITY,
            culture_bias_channel: 3, // all channels enabled by default
            // Phase 43: cultural ratchet + abiogenesis defaults.
            teach_on: TEACH_ON,
            abiogenesis_on: ABIOGENESIS_ON,
            abio_rng: Mulberry32::new(seed ^ ABIO_RNG_TAG),
            abiogenesis_events: 0,
            abiogenesis_ticks: 0,
            // Phase 39.1: group layer — ON by default with its own RNG stream;
            // empty groups ⇒ no draws and no energy flows until cohesion
            // conditions are actually met.
            groups: Vec::new(),
            next_group_id: 1,
            group_on: crate::group::GROUP_ON,
            group_rng: Mulberry32::new(seed ^ crate::group::GROUP_RNG_TAG),
            group_stats: crate::group::GroupStats::default(),
            group_pending: Vec::new(),
            patch_on: crate::patch::PATCH_ON,
            patch_p: crate::patch::PATCH_P,
            patch_rng: Mulberry32::new(seed ^ crate::patch::PATCH_RNG_TAG),
            patch_events: 0,
            patch_last_tick: 0,
            patch_last_x: 0.0,
            patch_last_z: 0.0,
            regrow_noise: crate::patch::generate_regrow_noise(),
            structure_env: vec![1.0f32; n],
            structure_env_on: STRUCTURE_ENV_ON,
            structure_env_decay_active: false,
            deco_env: vec![1.0f32; n],
            deco_env_on: DECO_RESOURCE_ENV_ON,
            deco_env_decay_active: false,
            deco_env_min: STRUCTURE_DECO_MIN,
            deco_env_max: STRUCTURE_DECO_MAX,
            token_value_sign_neutral: crate::culture::TOKEN_VALUE_SIGN_NEUTRAL,
            token_placement_by_value: false,
            deco_repro_saturate: true,
            deco_region_cap: DECO_REGION_CAP_ON,
            deco_region_weight: [1.0; REGION_COUNT as usize],
            deco_region_weight_sum: REGION_COUNT as f64,
            deco_region_weight_tick: u32::MAX,
            deco_cell_region: build_deco_cell_region_map(),
            deco_region_cap_renorm: DECO_REGION_CAP_RENORM_ON,
            deco_region_cap_floor: DECO_REGION_CAP_FLOOR_ON,
            deco_region_cap_floor_small: DECO_REGION_CAP_FLOOR_SMALL_ON,
            deco_region_cap_renorm_daughters: false,
            clade_life: Vec::new(),
            isolation_by_kind: Vec::new(),
            culture_value_on: true,
            dispersal_p: DISPERSAL_P,
            dispersal_rng: Mulberry32::new(seed ^ DISPERSAL_RNG_TAG),
            dispersal_events: 0,
            dispersal_last_tick: 0,
            region_isolation: true,
            deco_mut_scale: DECO_MUT_SCALE,
            // Phase 49: fork budget ON; every registered kind starts with its
            // own global cap as share (root kinds; derived kinds are re-granted
            // at fork time).
            budget_fork: BUDGET_FORK_ON,
            kind_budgets,
            budget_growth: FORK_BUDGET_GROWTH_ON,
            kind_growth_grants: Vec::new(),
            budget_grace: FORK_GRACE_ON,
            kind_grace_until: Vec::new(),
            grace_ticks: FORK_GRACE_TICKS,
            kind_grace_skips: Vec::new(),
            kind_grace_peak_over: Vec::new(),
            kind_grace_settled: Vec::new(),
            kind_grace_unpaid: Vec::new(),
            fork_budget_min: FORK_BUDGET_MIN,
            fork_daughter_min: FORK_BUDGET_MIN,
            mate_radius_arm: if MATE_FLOOR_ON { 1 } else { 0 },
            radius_boost_searches: 0,
            radius_only_finds: 0,
            radius_only_find_dist_sum: 0.0,
            speciation_min_side: SPECIATION_MIN_SIDE_DEFAULT,
            // Phase 51: niche resource partition ON by default (the preference
            // multiplier is 1.0 for every clade inside the neutral band, so a
            // pre-split world is bit-exact with Phase 50).
            niche_partition: NICHE_PARTITION_ON,
            niche_gen_accum: NICHE_GEN_ACCUM_ON,
            niche_partition_share: crate::niche::NichePartitionShare::default(),
            niche_by_clade: Vec::new(),
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
        // Phase 49: every registered kind starts with its own global cap as
        // resource share (root kinds; derived kinds are re-granted at fork).
        let kind_budgets: Vec<usize> = kinds.defs().iter().map(|d| d.global_cap).collect();
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
            // Phase 34: per-kind time gate (padded in register_kind to the
            // registry length) + retired-script lineage labels (empty until the
            // first legacy HERB speciation event).
            last_spec_tick_by_kind: Vec::new(),
            herb_lineage_a: None,
            herb_lineage_b: None,
            ledger: EnergyLedger::new(),
            // Phase 36: evolvable micro-brain defaults — the shadow network is
            // ON, neuron upkeep and the Landauer info cost are ON, TROPHIC_EFF
            // is wired into the PRED bite gain, and the FSM stays authoritative.
            brain_on: BRAIN_ON,
            brain_upkeep_on: BRAIN_UPKEEP_ON,
            info_cost_on: INFO_COST_ON,
            trophic_eff_active: true,
            brain_switch_enabled: BRAIN_SWITCH_ENABLED,
            brain_learning: BrainLearningStats::default(),
            brain_learning_rng: Mulberry32::new(seed ^ BRAIN_RNG_TAG),
            // Phase 41: triune brain runs every tick and pays real upkeep.
            // Phase 42: it also takes behaviour authority once its own
            // FSM-agreement gate (and confidence floor) is met.
            triune_on: true,
            triune_upkeep_on: true,
            replay_on: REPLAY_ON,
            triune_stats: TriuneLearningStats::default(),
            // Phase 37: culture layer — ON by default for this dynamics stage,
            // with a dedicated RNG stream separate from all ecology streams.
            culture_on: crate::culture::CULTURE_ON,
            token_registry: crate::culture::CultureRegistry::new(),
            structure_registry: crate::culture::StructureRegistry::new(),
            culture_rng: Mulberry32::new(seed ^ crate::culture::TOKEN_RNG_TAG),
            culture_stats: crate::culture::CultureStats::default(),
            culture_fidelity: crate::culture::DEFAULT_FIDELITY,
            culture_bias_channel: 3, // all channels enabled by default
            // Phase 43: cultural ratchet + abiogenesis defaults.
            teach_on: TEACH_ON,
            abiogenesis_on: ABIOGENESIS_ON,
            abio_rng: Mulberry32::new(seed ^ ABIO_RNG_TAG),
            abiogenesis_events: 0,
            abiogenesis_ticks: 0,
            // Phase 39.1: group layer — ON by default with its own RNG stream;
            // empty groups ⇒ no draws and no energy flows until cohesion
            // conditions are actually met.
            groups: Vec::new(),
            next_group_id: 1,
            group_on: crate::group::GROUP_ON,
            group_rng: Mulberry32::new(seed ^ crate::group::GROUP_RNG_TAG),
            group_stats: crate::group::GroupStats::default(),
            group_pending: Vec::new(),
            patch_on: crate::patch::PATCH_ON,
            patch_p: crate::patch::PATCH_P,
            patch_rng: Mulberry32::new(seed ^ crate::patch::PATCH_RNG_TAG),
            patch_events: 0,
            patch_last_tick: 0,
            patch_last_x: 0.0,
            patch_last_z: 0.0,
            regrow_noise: crate::patch::generate_regrow_noise(),
            structure_env: vec![1.0f32; n],
            structure_env_on: STRUCTURE_ENV_ON,
            structure_env_decay_active: false,
            deco_env: vec![1.0f32; n],
            deco_env_on: DECO_RESOURCE_ENV_ON,
            deco_env_decay_active: false,
            deco_env_min: STRUCTURE_DECO_MIN,
            deco_env_max: STRUCTURE_DECO_MAX,
            token_value_sign_neutral: crate::culture::TOKEN_VALUE_SIGN_NEUTRAL,
            token_placement_by_value: false,
            deco_repro_saturate: true,
            deco_region_cap: DECO_REGION_CAP_ON,
            deco_region_weight: [1.0; REGION_COUNT as usize],
            deco_region_weight_sum: REGION_COUNT as f64,
            deco_region_weight_tick: u32::MAX,
            deco_cell_region: build_deco_cell_region_map(),
            deco_region_cap_renorm: DECO_REGION_CAP_RENORM_ON,
            deco_region_cap_floor: DECO_REGION_CAP_FLOOR_ON,
            deco_region_cap_floor_small: DECO_REGION_CAP_FLOOR_SMALL_ON,
            deco_region_cap_renorm_daughters: false,
            clade_life: Vec::new(),
            isolation_by_kind: Vec::new(),
            culture_value_on: true,
            dispersal_p: DISPERSAL_P,
            dispersal_rng: Mulberry32::new(seed ^ DISPERSAL_RNG_TAG),
            dispersal_events: 0,
            dispersal_last_tick: 0,
            region_isolation: true,
            deco_mut_scale: DECO_MUT_SCALE,
            // Phase 49: fork budget ON; every registered kind starts with its
            // own global cap as share (root kinds; derived kinds are re-granted
            // at fork time).
            budget_fork: BUDGET_FORK_ON,
            kind_budgets,
            budget_growth: FORK_BUDGET_GROWTH_ON,
            kind_growth_grants: Vec::new(),
            budget_grace: FORK_GRACE_ON,
            kind_grace_until: Vec::new(),
            grace_ticks: FORK_GRACE_TICKS,
            kind_grace_skips: Vec::new(),
            kind_grace_peak_over: Vec::new(),
            kind_grace_settled: Vec::new(),
            kind_grace_unpaid: Vec::new(),
            fork_budget_min: FORK_BUDGET_MIN,
            fork_daughter_min: FORK_BUDGET_MIN,
            mate_radius_arm: if MATE_FLOOR_ON { 1 } else { 0 },
            radius_boost_searches: 0,
            radius_only_finds: 0,
            radius_only_find_dist_sum: 0.0,
            speciation_min_side: SPECIATION_MIN_SIDE_DEFAULT,
            // Phase 51: niche resource partition ON by default (see `Ecology::new`).
            niche_partition: NICHE_PARTITION_ON,
            niche_gen_accum: NICHE_GEN_ACCUM_ON,
            niche_partition_share: crate::niche::NichePartitionShare::default(),
            niche_by_clade: Vec::new(),
        };
        eco.init_grass();
        eco.seed_life();
        eco.sample();
        eco
    }

    // Phase 36: micro-brain initialization and shadow helpers.

    /// Create a fresh brain seeded from the grandfather reference network for
    /// `kind_name`, then overwrite with per-agent deterministic noise derived
    /// from `agent_id` (zero consumption of the main/prng/fish RNG streams,
    /// preserving their bit-exact order).
    pub fn initial_brain(&self, kind: SpeciesKind, agent_id: u32) -> aether_brain::Brain {
        use aether_brain::{reference_brain, BRAIN_RNG_TAG};
        let mut b = reference_brain(self.kind_name(kind));
        // Deterministic per-id noise on a dedicated seed — no draw from main/prng/fish.
        let mut nrng = aether_rng::Mulberry32::new(agent_id.wrapping_mul(0x9E3779B9).wrapping_add(BRAIN_RNG_TAG));
        b.perturb(&mut nrng, BRAIN_NOISE_SIGMA);
        b
    }

    /// Phase 41: seed the triune brain from the kind's reference genome plus
    /// per-agent deterministic noise (agent-id-derived stream, zero main RNG).
    pub fn initial_triune(&self, kind: SpeciesKind, agent_id: u32) -> aether_brain::triune::TriuneBrain {
        aether_brain::triune::TriuneBrain::reference(self.kind_name(kind), agent_id)
    }

    /// Phase 41: aggregate triune telemetry (observability only).
    pub fn triune_stats(&self) -> &TriuneLearningStats { &self.triune_stats }

    /// Sample an 8-channel sense vector from the live agent state **after** the
    /// FSM has made its decision this tick (inputs are already committed to
    /// `agent.state` / `fear` / target / etc). Zero extra RNG.
    /// Takes `grass` explicitly so callers with only scalar aliases can
    /// invoke this without overlapping borrows through `self`.
    #[inline]
    pub fn sample_sense_vector(
        &self,
        a: &Agent,
        grass: &[f32],
        cell_index_fn: impl Fn(f64, f64) -> isize,
        daylight: f64,
        out: &mut [f64; 8],
    ) {
        // 0 threat  : 1 if PRED is within sense*0.95 (legacy HERB scan).
        out[0] = if a.fear > 0.5 { 1.0 } else { 0.0 };
        // 1 hungry  : energy < 0.84 * max_energy (the grazing trigger used by
        //             every herbivore branch).
        out[1] = if a.energy < a.max_energy * 0.84 { 1.0 } else { 0.0 };
        // 2 food_near: grass cell at position >= 0.1.
        let ci = cell_index_fn(a.x, a.z);
        out[2] = if ci >= 0 {
            let gi = ci as usize;
            if gi < grass.len() && (grass[gi] as f64) > 0.1 { 1.0 } else { 0.0 }
        } else { 0.0 };
        // 3 prey_near: 1 iff the agent has a live target (PRED tracking prey).
        out[3] = if a.target_id != 0 {
            // We need the agents slice to test liveness; caller passes a thin
            // closure if needed. Here we just inspect the flag.
            1.0
        } else { 0.0 };
        // 4 social: genome.social > 0.8 (the legacy `social_high` gate).
        out[4] = if a.genome.social > 0.8 { 1.0 } else { 0.0 };
        // 5 fatigue: direct carry (in [0,1]).
        out[5] = a.fatigue.clamp(0.0, 1.0);
        // 6 daylight: world daytime in [0,1].
        out[6] = daylight;
        // 7 dist_norm: distance to target normalised by speed*10+1, clamped.
        let dx = a.target_x - a.x;
        let dz = a.target_z - a.z;
        let d = dx.hypot(dz);
        let scale = (a.desired_speed.max(a.genome.speed) * 10.0 + 1.0).max(1.0);
        out[7] = (d / scale).clamp(0.0, 1.0);
    }

    /// Phase 40.2: environment signature sampled at the agent's current cell,
    /// used as the seed vector for de-novo token invention. All channels are
    /// clamped to [-1, 1] so invented tokens live in the same space as
    /// recombined ones.
    ///
    /// 0 grass · 1 plant suitability · 2 moisture · 3 x (normalised) ·
    /// 4 z (normalised) · 5 energy fraction · 6 daylight · 7 hydration
    pub fn sample_env_features(&self, a: &Agent) -> [f64; crate::culture::TOKEN_DIM] {
        let ci = self.cell_index(a.x, a.z);
        let (grass, suit, moist) = if ci >= 0 {
            let gi = ci as usize;
            (
                if gi < self.grass.len() { self.grass[gi] as f64 } else { 0.0 },
                if gi < self.suit.len() { self.suit[gi] as f64 } else { 0.0 },
                if gi < self.moist.len() { self.moist[gi] as f64 } else { 0.0 },
            )
        } else { (0.0, 0.0, 0.0) };
        let half = GRID_SPAN / 2.0;
        [
            (grass * 2.0 - 1.0).clamp(-1.0, 1.0),
            (suit * 2.0 - 1.0).clamp(-1.0, 1.0),
            (moist * 2.0 - 1.0).clamp(-1.0, 1.0),
            (a.x / half).clamp(-1.0, 1.0),
            (a.z / half).clamp(-1.0, 1.0),
            ((a.energy / a.max_energy.max(1.0)) * 2.0 - 1.0).clamp(-1.0, 1.0),
            (self.daylight * 2.0 - 1.0).clamp(-1.0, 1.0),
            (a.hydration * 2.0 - 1.0).clamp(-1.0, 1.0),
        ]
    }

    /// Derive a deterministic 7-dim one-hot action index from an agent's FSM
    /// state / speed / repro readiness. Mirrors the high-level intent encoded
    /// in `update_*`: flee/hunt/eat/wander → move_forward, reproduce when
    /// conditions are met, help on cooperative interactions (rare).
    #[inline]
    pub fn fsm_action_from(a: &Agent) -> u8 {
        match a.state {
            "flee" => 4,
            "graze" | "feed" | "eat" => 3,
            "hunt" | "scavenge" | "approach" => 0, // move_forward
            _ if a.repro <= 0.0 && a.energy > a.max_energy * 0.72 => 5, // reproduce
            _ if a.help_cd <= 0.0 && a.image > 0 => 6, // help (approx)
            _ => 0, // default: move_forward (wander/patrol/school/search/other)
        }
    }

    /// Phase 36: run the shadow forward pass + Hebbian update on each alive
    /// agent at the end of its per-tick FSM loop. Zero additional RNG draws:
    /// the whole path is a pure function of `(agent_state, brain_state, dt)`.
    /// Side effects (when toggled on):
    /// * `energy -= upkeep` and `ledger.brain_cost += ...` (BRAIN_UPKEEP_ON)
    /// * `charge_info_cost(idx, 1)` (INFO_COST_ON)
    /// * Brain weights updated via hebbian_update; no main/prng/fish consumption.
    /// The dual-controller switchover itself runs only at speciation-check
    /// boundaries (sample() cadence), not here, to avoid per-tick overhead.
    pub fn brain_shadow(&mut self, idx: usize, h: f64) {
        if !self.brain_on && !self.triune_on { return; }
        // Pre-fetch immutable state BEFORE acquiring any mutable alias, so
        // downstream mutations can coexist with reads without NLL conflicts.
        let a_ref = &self.agents[idx];
        let mut sense = [0.0f64; aether_brain::INPUT_CHANNELS];
        self.sample_sense_vector(a_ref, &self.grass, |x, z| self.cell_index(x, z), self.daylight, &mut sense);
        let fsm_act = Self::fsm_action_from(a_ref);
        // Phase 42: 回流一致性——接管个体的 `executed` 就是上一脑节律 tick 仲裁
        // 出来、本 tick 真的被 `brain_bridge` 执行的那个动作（不再是 FSM 动作）。
        // 三层脑学的因此是它自己造成的真实转移，学习闭环自洽。
        let authority = a_ref.brain_authority == BRAIN_AUTHORITY_BRAIN;
        let executed = if authority && a_ref.triune.stats.ticks > 0 {
            a_ref.triune.last.action
        } else {
            fsm_act
        };
        let legacy_up = if self.brain_on && self.brain_upkeep_on && a_ref.brain.n_hid > 0 {
            Some(a_ref.brain.upkeep() * h)
        } else { None };
        let triune_up = if self.triune_on && self.triune_upkeep_on {
            Some(a_ref.triune.upkeep() * h)
        } else { None };
        // Upkeep drain (books into ledger).
        if let Some(cost) = legacy_up {
            let a = &mut self.agents[idx];
            a.energy = (a.energy - cost).max(0.0);
            self.ledger.brain_cost += cost;
            self.brain_learning.brain_energy_drained += cost;
        }
        if let Some(cost) = triune_up {
            let a = &mut self.agents[idx];
            a.energy = (a.energy - cost).max(0.0);
            self.ledger.triune_cost += cost;
            self.triune_stats.upkeep_energy += cost;
        }
        // Shared chemotaxis-style reward (energy vs slow EMA baseline).
        let reward = {
            let a = &mut self.agents[idx];
            a.brain.taxis_reward(a.energy)
        };
        // Legacy single-layer shadow: forward + Hebbian + streak bookkeeping.
        if self.brain_on {
            let a = &mut self.agents[idx];
            let _out = a.brain.forward(&sense); // out unused — shadow only
            self.brain_learning.forward_count += 1;
            a.brain.hebbian_update(&sense, reward, h);
            self.brain_learning.hebbian_updates += 1;
            a.brain_last_fsm_action = fsm_act;
            a.brain_last_brain_action = a.brain.argmax_action() as u8;
            if fsm_act == a.brain_last_brain_action {
                a.brain_equiv_streak += 1;
                a.brain_agree_window += 1;
            } else {
                a.brain_equiv_streak = 0;
            }
            a.brain_window_len += 1;
            if a.brain_window_len > T_EQUIV_K_TICKS { a.brain_window_len = T_EQUIV_K_TICKS; }
            if a.brain_equiv_streak > self.brain_learning.equiv_streak_max {
                self.brain_learning.equiv_streak_max = a.brain_equiv_streak;
            }
        }
        // Phase 41: three-layer shadow. Zero RNG; the executed action is the
        // FSM's current pick (the shadow learns the real transition).
        // Phase 41.1: brain rhythm — each agent does a full triune update on
        // every other tick, half the population staggered per tick. A neuron
        // does not replan every 100 ms; a ~200 ms decision rhythm is both
        // biologically plausible and halves the per-tick planning cost
        // (was 19.7 ms/frame in the wasm viewer budget, over the 16 ms gate).
        // Upkeep is still charged every tick (the brain is always alive).
        //
        let replay_on = self.replay_on;
        if self.triune_on && (self.tick_count % 2) == (idx as u32 % 2) {
            let (outcome, replayed) = {
                let a = &mut self.agents[idx];
                let o = a.triune.tick(&sense, reward, h, executed);
                // Phase 42: 离线巩固——每个脑节律 tick 额外抽一条历史经验
                // 做一次 delta-rule 更新（海马尖波涟漪重放的最小同构）。
                // 零 RNG：游标确定性轮转。与在线更新共用世界模型，但只改权重
                // （不驱动 RPE/置信度，见 `observe_impl` 的 learn_meta）。
                let r = if replay_on { a.triune.replay_step().is_some() } else { false };
                (o, r)
            };
            // 影子阶段才累积一致度（接管后 FSM 已被脑驱动，再比就没意义）。
            if !authority {
                let a = &mut self.agents[idx];
                let agree = outcome.decision.action == fsm_act;
                // 只保留最近 K 位：移位寄存器即滑动窗口，高位必须剪掉，
                // 否则存档/恢复的掩码会与运行中的值不一致（位精确契约）。
                let mask: u64 = if T_EQUIV_K_TICKS >= 64 {
                    u64::MAX
                } else {
                    (1u64 << T_EQUIV_K_TICKS) - 1
                };
                a.triune_agree_bits = ((a.triune_agree_bits << 1) | (agree as u64)) & mask;
                a.triune_agree_n = (a.triune_agree_n + 1).min(T_EQUIV_K_TICKS);
                if agree {
                    a.triune_equiv_streak += 1;
                } else {
                    a.triune_equiv_streak = 0;
                }
                if a.triune_equiv_streak > self.brain_learning.equiv_streak_max {
                    self.brain_learning.equiv_streak_max = a.triune_equiv_streak;
                }
            }
            let decision_cost = if self.triune_upkeep_on {
                outcome.decision_rate * h
            } else { 0.0 };
            if decision_cost > 0.0 {
                let a = &mut self.agents[idx];
                a.energy = (a.energy - decision_cost).max(0.0);
                self.ledger.triune_cost += decision_cost;
                self.triune_stats.decision_energy += decision_cost;
            }
            let st = &mut self.triune_stats;
            st.ticks += 1;
            st.fear_events += outcome.fear_event as u64;
            st.fear_conditioning += outcome.fear_conditioning as u64;
            st.memory_retrievals += outcome.memory_retrieved as u64;
            st.wm_updates += outcome.wm_updates as u64;
            st.rollout_steps += outcome.rollout_steps as u64;
            st.planned_overrides += outcome.planned_override as u64;
            st.inhibitions += outcome.inhibition as u64;
            st.prediction_updates += outcome.prediction_updated as u64;
            st.high_confidence_ticks += (outcome.confidence > 0.7) as u64;
            st.replay_updates += replayed as u64;
            st.gate_open_ticks += (outcome.decision.inhibit_gate > aether_brain::triune::GATE_OPEN) as u64;
            st.gate_sum += outcome.decision.inhibit_gate;
            st.max_fear = st.max_fear.max(outcome.fear);
            st.fear_sum += outcome.fear;
            st.confidence_sum += outcome.confidence;
            st.pred_error_sum += outcome.pred_error;
            st.action_hist[outcome.decision.action as usize] += 1;
        }
    }

    /// Phase 36: check whether any agent's shadow has accumulated enough
    /// FSM↔brain agreement to flip its authority from FSM → brain. Runs ONLY
    /// at speciation-check tick boundaries (`tick_count % SPECIATION_CHECK_TICKS
    /// == 0`) so it never interferes with the hot update loop.
    ///
    /// Phase 42: the agreement now comes from the **triune** brain — the network
    /// that actually takes over (checking the legacy micro-brain's agreement
    /// would hand authority to an unrelated network), plus a metacognitive
    /// confidence floor (`T_TAKEOVER_CONF`): a brain whose world model does not
    /// yet predict its own transitions has no business steering the body.
    ///
    /// `BRAIN_SWITCH_ENABLED = false` keeps the Phase 41.1 shadow-only world
    /// (the A/B arm).
    pub fn check_brain_authority_switch(&mut self) {
        if !self.brain_switch_enabled { return; }
        if self.tick_count % SPECIATION_CHECK_TICKS != 0 { return; }
        let k = T_EQUIV_K_TICKS;
        let mask: u64 = if k >= 64 { u64::MAX } else { (1u64 << k) - 1 };
        let need = ((k as f64 * T_EQUIV).ceil() as u32).min(k);
        for i in 0..self.agents.len() {
            let a = &self.agents[i];
            if !a.alive || a.brain_authority == BRAIN_AUTHORITY_BRAIN { continue; }
            if a.triune_equiv_streak < k || a.triune_agree_n < k { continue; }
            let agrees = (a.triune_agree_bits & mask).count_ones();
            let confident = a.triune.stats.ticks > 0 && a.triune.confidence >= T_TAKEOVER_CONF;
            if agrees >= need && confident {
                self.agents[i].brain_authority = BRAIN_AUTHORITY_BRAIN;
                self.brain_learning.authority_switches += 1;
            }
        }
    }

    // ----- kind registry (Phase 31) ------------------------------------

    // ----- Phase 42: 行为权威桥接 ---------------------------------------------

    /// 把朝向对准目标（与各 update_* 的转向子程序同一套语义：最大 2.8·h rad/tick）。
    fn brain_steer(&mut self, idx: usize, h: f64) {
        let dx = self.agents[idx].target_x - self.agents[idx].x;
        let dz = self.agents[idx].target_z - self.agents[idx].z;
        if dx * dx + dz * dz > 0.01 {
            let want = aether_libm::atan2(dx, dz);
            let mut diff = want - self.agents[idx].heading;
            while diff > std::f64::consts::PI { diff -= std::f64::consts::PI * 2.0; }
            while diff < -std::f64::consts::PI { diff += std::f64::consts::PI * 2.0; }
            self.agents[idx].heading += diff.clamp(-2.8 * h, 2.8 * h);
        }
    }

    /// 改变朝向并把目标点挪到新朝向上（转体=朝向+目标一起转，否则下一 tick
    /// FSM 的转向子程序会把身体掰回旧目标）。
    fn brain_aim(&mut self, idx: usize, heading: f64) {
        let x = self.agents[idx].x;
        let z = self.agents[idx].z;
        // Phase 42.1: the body refuses what the environment cannot execute.
        // A beached fish dies at FISH_DRY_DEATH_RATE = 0.5/s, and the
        // unbridged `flee`/`turn` intents happily aimed schools onto dry land
        // (every taken-over fish was dead by t=16k in the diagnostic run).
        // So for aquatic bodies the aim is clamped to the nearest heading
        // whose 10-unit lookahead stays in water — a zero-RNG physical
        // constraint, not a behaviour script.
        if self.kind_def(self.agents[idx].kind).behavior == BehaviorTemplate::Fish {
            for &off in &[0.0, 0.45, -0.45, 0.9, -0.9, 1.35, -1.35, 1.8, -1.8, 2.25, -2.25] {
                let hx = x + aether_libm::sin(heading + off) * 10.0;
                let hz = z + aether_libm::cos(heading + off) * 10.0;
                if terrain_height(hx, hz) < WATER_LEVEL + 1.5 {
                    let hh = heading + off;
                    self.agents[idx].heading = hh;
                    self.agents[idx].target_x = x + aether_libm::sin(hh) * 10.0;
                    self.agents[idx].target_z = z + aether_libm::cos(hh) * 10.0;
                    return;
                }
            }
            // No water within reach in any direction: keep the current
            // heading and target (the school hugs the shoreline instead of
            // marching inland).
            return;
        }
        self.agents[idx].heading = heading;
        self.agents[idx].target_x = x + aether_libm::sin(heading) * 10.0;
        self.agents[idx].target_z = z + aether_libm::cos(heading) * 10.0;
    }

    /// 活猎物探测（零 RNG）：物种 diet 声明了猎物、且猎物物理可达时才返回
    /// （水生猎物只能从浅水边缘捕捉——沿用 update_pred 的物理约束）。
    /// 返回 (距离, id, x, z)。
    fn brain_probe_prey(
        &self,
        idx: usize,
        prey_kinds: &[SpeciesKind],
        g_sense: f64,
    ) -> Option<(f64, u32, f64, f64)> {
        let my_ground = terrain_height(self.agents[idx].x, self.agents[idx].z);
        let fish_reachable = my_ground < WATER_LEVEL + 3.0;
        let mut best: Option<(f64, u32, f64, f64)> = None;
        let mut best_d = g_sense * g_sense;
        for &pk in prey_kinds {
            if self.kind_def(pk).behavior == BehaviorTemplate::Fish && !fish_reachable { continue; }
            if let Some((j, d)) = self.nearest(idx, pk, g_sense, None::<fn(&Agent) -> bool>) {
                if d < best_d {
                    best_d = d;
                    best = Some((d, self.agents[j].id, self.agents[j].x, self.agents[j].z));
                }
            }
        }
        best
    }

    /// 草地探测（零 RNG）：在感知半径内扫描草量最高的格子。
    /// 返回 (距离, x, z)——与 prey/carcass 探测同一形态。
    fn brain_probe_grass(&self, idx: usize, g_sense: f64) -> Option<(f64, f64, f64)> {
        let ax = self.agents[idx].x;
        let az = self.agents[idx].z;
        let ci0 = self.cell_index(ax, az);
        if ci0 < 0 { return None; }
        let gn = self.grid_n as isize;
        let i0 = ci0 % gn;
        let j0 = ci0 / gn;
        let r = (g_sense / self.cell).ceil() as isize;
        let mut best: Option<(f64, f64, f64)> = None;
        let mut best_score = 0.15f64; // below the FSM's graze floor: ignore stubble
        for j in (j0 - r).max(0)..=(j0 + r).min(gn - 1) {
            for i in (i0 - r).max(0)..=(i0 + r).min(gn - 1) {
                let gi = (j * gn + i) as usize;
                let g = self.grass[gi] as f64;
                if g <= best_score { continue; }
                let mut cc = [0.0f64; 2];
                Self::cell_coord(j * gn + i, &mut cc);
                let (cx, cz) = (cc[0], cc[1]);
                let dx = cx - ax;
                let dz = cz - az;
                let d2 = dx * dx + dz * dz;
                if d2 > g_sense * g_sense { continue; }
                // score = grass minus a small distance tiebreak
                let score = g - 0.001 * d2.sqrt();
                if score > best_score {
                    best_score = score;
                    best = Some((d2.sqrt(), cx, cz));
                }
            }
        }
        best
    }

    /// 尸体探测（零 RNG）：最近的可声明尸体（锁仲裁沿用 Phase 8 的规则）。
    /// 返回 (距离, x, z)。
    fn brain_probe_carcass(&self, idx: usize, g_sense: f64) -> Option<(f64, f64, f64)> {
        let ax = self.agents[idx].x;
        let az = self.agents[idx].z;
        let self_id = self.agents[idx].id;
        let mut bd = (g_sense * 1.2).powi(2);
        let mut best: Option<(f64, f64, f64)> = None;
        for c in self.carcasses.iter() {
            if !self.carcass_claimable(c, self_id) { continue; }
            let dx = c.x - ax;
            let dz = c.z - az;
            let d2 = dx * dx + dz * dz;
            if d2 < bd { bd = d2; best = Some((d2.sqrt(), c.x, c.z)); }
        }
        best
    }

    /// Phase 42: 三脑仲裁动作 → 运动意图的桥接层。
    ///
    /// 这是**接口层**，不是行为剧本：脑只选「做什么」（前进/左转/右转/吃/逃/
    /// 繁殖/帮助），怎么落地完全由身体能力决定——能吃植物/尸体/活猎物看
    /// `diet`，速度倍率看行为模板，lok 仲裁/啃食/繁殖生理全在原有的
    /// `update_*` 子程序里。因此桥接零 RNG、不新增感知通道、不改能量学。
    ///
    /// 意图与实现的分工：脑拥有最终一拍（在 FSM 的运动块之后覆盖
    /// `state`/`heading`/`desired_speed`），FSM 保留感知-目标维护、接近-
    /// 锁定、啃食、繁殖、死亡等全部子程序——脑是总经理，不是工人。
    ///
    /// 只在 `brain_authority == BRAIN_AUTHORITY_BRAIN` 时生效；在未更新的
    /// 脑节律 tick 上复用 `triune.last`（最近一次仲裁结果）。
    fn brain_bridge(&mut self, idx: usize, h: f64) {
        if !self.triune_on { return; }
        if self.agents[idx].brain_authority != BRAIN_AUTHORITY_BRAIN { return; }
        if self.agents[idx].triune.stats.ticks == 0 { return; }
        let action = self.agents[idx].triune.last.action as usize % aether_brain::OUTPUT_ACTIONS;
        let kind = self.agents[idx].kind;
        let tmpl = self.kind_def(kind).behavior;
        let g_speed = self.agents[idx].genome.speed;
        let g_sense = self.agents[idx].genome.sense;
        let g_bold = self.agents[idx].genome.bold;
        let speed_mult = match tmpl {
            BehaviorTemplate::Fish => FISH_SPEED_MULT,
            BehaviorTemplate::Deco => DECO_SPEED_MULT,
            _ => 1.0,
        };
        let cruise = g_speed * speed_mult * match tmpl {
            BehaviorTemplate::Pred => 0.32,
            BehaviorTemplate::Deco => 0.4,
            _ => 0.55,
        };
        let eats_plant = self.diet_of(kind).eats_plant;
        let eats_carcass = self.diet_of(kind).eats_carcass;
        let prey_kinds: Vec<SpeciesKind> = self.diet_of(kind).prey_kinds.clone();

        let desired = match action {
            // turn_left / turn_right：转体 + 慢速前移（探索/避障）
            1 | 2 => {
                let rate = 2.8 * h;
                let heading = self.agents[idx].heading + if action == 1 { -rate } else { rate };
                self.brain_aim(idx, heading);
                if self.agents[idx].state == "flee" { self.agents[idx].state = "wander"; }
                cruise * 0.5
            }
            // eat：在物种能吃的所有来源里选最近的（能力决定菜单，距离决定
            // 选择，不是菜谱式偏好）。脚下有植物视为零距离。
            3 => {
                let ci = self.cell_index(self.agents[idx].x, self.agents[idx].z);
                let here = if ci >= 0 { self.grass[ci as usize] as f64 } else { 0.0 };
                // Phase 42.1: 觅食经济学优先——脚下的食物胜过 30 单位外更肥的
                // 草格，仍有草的目标格胜过重新探针。每个 eat 决策都重新定靶
                // 会让草食者在肥草格之间跳跃、永远坐不下来喈食（实测摄入崩
                // 塨 → cohort 老龄化灭绝）。只有目标真正失效（空/太远/未设）
                // 才做 richest-cell 探针；零 RNG、扫描顺序即确定性平局规则。
                if eats_plant && here > 0.1 {
                    self.agents[idx].state = "graze";
                    self.agents[idx].desired_speed = 0.0;
                    return;
                }
                // Phase 42.1: real forage probe — “eat” must mean “go to the
                // nearest actual food”, otherwise a taken-over herb walks to
                // the stale FSM target and starves inside a full meadow
                // (grass 4200, herd extinct by t=7k in the diagnostic).
                let d_plant = if eats_plant {
                    let tx = self.agents[idx].target_x;
                    let tz = self.agents[idx].target_z;
                    let tci = self.cell_index(tx, tz);
                    let t_grass = if tci >= 0 { self.grass[tci as usize] as f64 } else { 0.0 };
                    let td = js_hypot(tx - self.agents[idx].x, tz - self.agents[idx].z);
                    if t_grass > 0.1 && td > 0.01 {
                        td
                    } else {
                        match self.brain_probe_grass(idx, g_sense) {
                            Some((d, gx, gz)) => {
                                self.agents[idx].target_x = gx;
                                self.agents[idx].target_z = gz;
                                d
                            }
                            None => f64::INFINITY,
                        }
                    }
                } else { f64::INFINITY };
                let prey = if !prey_kinds.is_empty() {
                    self.brain_probe_prey(idx, &prey_kinds, g_sense)
                } else { None };
                let carcass = if eats_carcass { self.brain_probe_carcass(idx, g_sense) } else { None };
                let mut best = (d_plant, 0u8);
                if let Some((d, _, _, _)) = prey {
                    if d < best.0 { best = (d, 1); }
                }
                if let Some((d, _, _)) = carcass {
                    if d < best.0 { best = (d, 2); }
                }
                match best.1 {
                    1 => {
                        let (_, tid, px, pz) = prey.unwrap();
                        self.agents[idx].target_id = tid;
                        self.agents[idx].target_x = px;
                        self.agents[idx].target_z = pz;
                        self.agents[idx].state = "hunt";
                        self.brain_steer(idx, h);
                        g_speed * speed_mult * 1.1
                    }
                    2 => {
                        let (_, cx, cz) = carcass.unwrap();
                        self.agents[idx].target_x = cx;
                        self.agents[idx].target_z = cz;
                        self.agents[idx].state = match tmpl {
                            BehaviorTemplate::Deco => "approach",
                            _ => "scavenge",
                        };
                        self.brain_steer(idx, h);
                        match tmpl {
                            BehaviorTemplate::Deco => g_speed * speed_mult * 0.8,
                            _ => g_speed * speed_mult * 1.1,
                        }
                    }
                    _ => {
                        self.agents[idx].state = "graze";
                        if eats_plant && here > 0.1 || d_plant < 1.6 {
                            0.0
                        } else {
                            self.brain_steer(idx, h);
                            g_speed * speed_mult * 0.75
                        }
                    }
                }
            }
            // flee：逃离最近威胁；无威胁时意图退化为移动（不演不存在的逃跑戏）
            4 => {
                let threat = self.nearest(idx, PRED, g_sense, None::<fn(&Agent) -> bool>);
                if let Some((tj, _)) = threat {
                    let away = aether_libm::atan2(
                        self.agents[idx].x - self.agents[tj].x,
                        self.agents[idx].z - self.agents[tj].z,
                    );
                    self.brain_aim(idx, away);
                    self.agents[idx].state = "flee";
                    g_speed * (1.32 - g_bold * 0.25)
                } else {
                    if self.agents[idx].state == "flee" { self.agents[idx].state = "wander"; }
                    self.brain_steer(idx, h);
                    cruise * 1.2
                }
            }
            // reproduce：只表达意图，生理条件（能量/冷却/配偶）仍由 update_*
            // 尾部的现有繁殖路径决定——脑不能绕过能量学。
            5 => {
                self.brain_steer(idx, h);
                cruise
            }
            // help：走现有协作路径（kind/年龄/能量/冷却全在 try_help 里）
            6 => {
                if self.agents[idx].state == "flee" { self.agents[idx].state = "graze"; }
                let ci = self.cell_index(self.agents[idx].x, self.agents[idx].z);
                if ci >= 0 {
                    self.try_help(idx, ci as usize);
                }
                self.agents[idx].state = "graze";
                0.0
            }
            // move_forward：保持当前目标全速前进
            _ => {
                if self.agents[idx].state == "flee" { self.agents[idx].state = "wander"; }
                self.brain_steer(idx, h);
                cruise
            }
        };
        self.agents[idx].desired_speed = desired;
    }

    pub fn kind_def(&self, id: SpeciesKind) -> &KindDef { self.kinds.def(id) }

    pub fn kind_name(&self, id: SpeciesKind) -> &'static str { self.kinds.name(id) }

    /// Register a new species at runtime (kind-forking in Phase 34). Appends
    /// per-kind history/fitness/speciation-gate slots so the id is always
    /// indexable.
    pub fn register_kind(&mut self, def: KindDef) -> SpeciesKind {
        let id = self.kinds.register(def);
        while self.evolution.len() <= id as usize { self.evolution.push(Vec::new()); }
        while self.generation_fitness.len() <= id as usize { self.generation_fitness.push(Vec::new()); }
        while self.last_spec_tick_by_kind.len() <= id as usize { self.last_spec_tick_by_kind.push(0); }
        // Phase 49: a freshly registered kind starts at its (cloned) global
        // cap; the fork path re-grants it from the parent's share immediately
        // after, so this is only the transient default.
        while self.kind_budgets.len() <= id as usize { self.kind_budgets.push(0); }
        self.kind_budgets[id as usize] = self.kind_def(id).global_cap;
        while self.kind_grace_until.len() <= id as usize { self.kind_grace_until.push(0); }
        id
    }

    /// Phase 34: the ancestor kind at the root of `kind`'s fork lineage
    /// (walks `KindDef.parent_kind` to the first `None` — one of the six
    /// legacy ancestors). Pure registry read; used to keep a daughter clade
    /// attached to its family's root.
    pub fn root_kind(&self, kind: SpeciesKind) -> SpeciesKind {
        let mut k = kind;
        for _ in 0..self.kinds.len() {
            match self.kind_def(k).parent_kind {
                Some(p) => k = p,
                None => return k,
            }
        }
        k
    }

    /// Phase 34: the registry kind id that births INTO `clade_id` should
    /// carry. Daughter clades fork a derived kind (`derived_kind`); root and
    /// pre-fork clades fall back to the caller's kind. Unknown clades
    /// (pre-first-detection sentinel 0) also fall back.
    pub fn clade_kind(&self, clade_id: u32, fallback: SpeciesKind) -> SpeciesKind {
        if let Some(c) = self.species_clades.iter().find(|c| c.id == clade_id) {
            // Phase 45 fix: id 0 doubles as the "unassigned" sentinel, so an
            // agent of another family that still carries 0 must NOT resolve
            // through a materialized clade 0 (e.g. a DECO root): without the
            // family check, every unassigned HERB/OMN/… birth inherited the
            // DECO kind and died on the DECO cap — the first natural split
            // silently froze all other kinds' reproduction.
            let family_ok = c.kind_id != u32::MAX
                && self.root_kind(c.kind_id as SpeciesKind) == self.root_kind(fallback);
            if let Some(dk) = c.derived_kind {
                if family_ok { return dk as SpeciesKind; }
            }
            if family_ok { return c.kind_id as SpeciesKind; }
        }
        fallback
    }

    /// Phase 34: the HERB lineage-bite instrument for the feeding/help paths.
    /// The retired preset names (迅聚/缓独) are gone; the modifiers key on
    /// the most recent HERB divergence event's derived labels (side "a" = the
    /// low-speed cluster, side "b" = the high-speed cluster). Any other
    /// lineage reads neutral 1.0.
    fn herb_lineage_bite(&self, lineage: Option<&'static str>) -> f64 {
        match (lineage, self.herb_lineage_a.as_deref(), self.herb_lineage_b.as_deref()) {
            (Some(l), Some(a), Some(_)) if l == a => 1.4,
            (Some(l), Some(a), Some(b)) if l == b => 0.7,
            _ => 1.0,
        }
    }

    /// Phase 34: the HERB lineage-drain instrument (update_herb total
    /// drain), same keying as `herb_lineage_bite`.
    fn herb_lineage_drain(&self, lineage: Option<&'static str>) -> f64 {
        match (lineage, self.herb_lineage_a.as_deref(), self.herb_lineage_b.as_deref()) {
            (Some(l), Some(a), Some(_)) if l == a => 1.15,
            (Some(l), Some(a), Some(b)) if l == b => 0.85,
            _ => 1.0,
        }
    }

    /// Phase 35: Landauer-style information cost. When `INFO_COST_ON`, every
    /// distinct perception sweep drains `INFO_COST_PER_READ * (sense/Base)^2`
    /// energy proportional to its sensory bandwidth. `false` → no-op, keeping
    /// pre-Phase-35 trajectory bit-exact.
    fn charge_info_cost(&mut self, idx: usize, reads: u64) {
        if !self.info_cost_on { return; }
        let sense = self.agents[idx].genome.sense;
        let cost = INFO_COST_PER_READ * (sense / 80.0).powi(2) * reads as f64;
        if cost > 0.0 {
            self.agents[idx].energy = (self.agents[idx].energy - cost).max(0.0);
            self.ledger.info_cost += cost;
            self.ledger.info_reads += reads;
        }
    }

    /// Phase 35: Kleiber body-mass metabolic scaling factor (size^0.25 under
    /// MTE_ON; identity when off).
    #[inline]
    fn mte_size_scale(&self, idx: usize) -> f64 {
        if MTE_ON { aether_libm::pow(self.agents[idx].genome.size, MTE_SIZE_EXPONENT) } else { 1.0 }
    }

    /// Phase 35: MTE body-mass reproduction fee — extra energy drawn from
    /// the parent proportional to `mass = size^3` per birth event.
    #[inline]
    fn mte_repro_fee(&self, idx: usize) -> f64 {
        if MTE_ON { MTE_REPRO_FEE * aether_libm::pow(self.agents[idx].genome.size, 3.0) } else { 0.0 }
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
        self.assign_initial_regions();
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

    /// Genome mutation. `mut_scale` multiplies every locus' mutation amplitude
    /// (1.0 = the Phase 36 trajectory exactly, since `x * 1.0` is bit-exact).
    /// Phase 39.1 feeds `1 + SOMATIC_MUTAGEN × parent_damage` here — the
    /// dirty-work mutagenesis channel. Public so research tools / acceptance
    /// tests can measure the amplitude directly; consumes the normal RNG
    /// streams (`rng`/`prng`/`fish_rng`) exactly like every birth.
    pub fn mutate(&mut self, g: &Genome, kind: SpeciesKind, mut_scale: f64) -> Genome {
        let mut out = g.clone();
        let mut ms = if mut_scale.is_finite() && mut_scale > 0.0 { mut_scale } else { 1.0 };
        if self.root_kind(kind) == DECO { ms *= self.deco_mut_scale; }
        let pop_size = self.count(kind);
        let (sp_lo, sp_hi) = self.kind_def(kind).speed_clamp;
        let mut_fin = self.kind_def(kind).mutates_fin_color;
        let mut_deco = self.kind_def(kind).mutates_deco_eff;
        let drift = if pop_size < 20 { 1.0 + (20.0 - pop_size as f64) / 20.0 } else { 1.0 };
        out.speed = (g.speed * (1.0 + (self.rng.next_f64() - 0.5) * 0.09 * drift * ms)).clamp(sp_lo, sp_hi);
        out.sense = (g.sense * (1.0 + (self.rng.next_f64() - 0.5) * 0.1 * drift * ms)).clamp(14.0, 130.0);
        out.size = (g.size * (1.0 + (self.rng.next_f64() - 0.5) * 0.07 * drift * ms)).clamp(0.58, 1.7);
        out.metabolism = (g.metabolism * (1.0 + (self.rng.next_f64() - 0.5) * 0.06 * drift * ms)).clamp(0.62, 1.6);
        out.lifespan = (g.lifespan * (1.0 + (self.rng.next_f64() - 0.5) * 0.08 * drift * ms)).clamp(180.0, 560.0);
        out.hue = (g.hue + (self.rng.next_f64() - 0.5) * 0.06 * drift * ms + 1.0) % 1.0;
        out.bold = (g.bold + (self.prng.next_f64() - 0.5) * 0.14 * drift * ms).clamp(0.0, 1.0);
        out.curious = (g.curious + (self.prng.next_f64() - 0.5) * 0.14 * drift * ms).clamp(0.0, 1.0);
        out.social = (g.social + (self.prng.next_f64() - 0.5) * 0.14 * drift * ms).clamp(0.0, 1.0);
        if mut_fin {
            out.fin_color = (g.fin_color + (self.fish_rng.next_f64() - 0.5) * 0.1 * drift * ms).clamp(0.0, 1.0);
        }
        if mut_deco {
            out.deco_efficiency = (g.deco_efficiency * (1.0 + (self.rng.next_f64() - 0.5) * 0.1 * drift * ms)).clamp(0.5, 1.5);
        }
        if let Some(h) = g.help_thr {
            out.help_thr = Some((h + (self.rng.next_f64() - 0.5) * 1.2 * drift * ms).clamp(-5.0, 5.0));
        }
        if let Some(p) = g.pref_sig {
            out.pref_sig = Some((p + (self.rng.next_f64() - 0.5) * 2.0 * SIG_MUT_PREF * drift * ms).clamp(0.58, 1.7));
        }
        if let Some(t) = g.tol_sig {
            out.tol_sig = Some((t + (self.rng.next_f64() - 0.5) * 2.0 * SIG_MUT_TOL * drift * ms).clamp(SIG_TOL_MIN, SIG_TOL_MAX));
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

    /// Phase 47: total living members of a root kind family (root + every
    /// derived fork). The family shares one carrying capacity, so a split
    /// changes the tree's shape but not the guild's biomass budget — without
    /// this, each derived kind would mint its own 80-slot cap and the
    /// population would grow with the clade count.
    pub fn count_family(&self, root: SpeciesKind) -> usize {
        self.agents.iter()
            .filter(|a| a.alive && self.root_kind(a.kind) == root)
            .count()
    }

    // ----- Phase 49: fork budget (independent resource share per species) ----

    /// Phase 49: the current carrying-capacity share of `kind`. Root kinds
    /// hold the family's budget (they start at `global_cap`); a derived kind
    /// holds the share carved out of its parent at fork time (0 once its
    /// lineage returned the share after extinction).
    pub fn kind_budget(&self, kind: SpeciesKind) -> usize {
        self.kind_budgets.get(kind as usize).copied()
            .unwrap_or_else(|| self.kind_def(kind).global_cap)
    }

    /// Phase 49: set a kind's share (pads the table for kinds registered
    /// after the last resize).
    fn set_kind_budget(&mut self, kind: SpeciesKind, value: usize) {
        while self.kind_budgets.len() <= kind as usize { self.kind_budgets.push(0); }
        self.kind_budgets[kind as usize] = value;
    }

    /// Phase 49: how much budget a fork carves out of `parent_budget`, or
    /// `None` when the parent cannot afford a fork. Grant = a quarter of the
    /// parent's share, clamped to `[FORK_BUDGET_MIN, FORK_BUDGET_GRANT]`; the
    /// parent must keep at least `FORK_BUDGET_MIN` (80 → 60+20 → 45+15 →
    /// 34+11 → 26+8, and below 2 × FORK_BUDGET_MIN no fork fires at all).
    /// Pure function — unit-tested (budget conservation + recursion floor).
    pub fn fork_budget_grant(parent_budget: usize) -> Option<usize> {
        Self::fork_budget_grant_with(parent_budget, FORK_BUDGET_MIN, FORK_BUDGET_MIN)
    }

    /// Phase 63: the Phase 49 grant rule with the two floors as parameters
    /// (candidates M/A). `min_parent` is the parent's post-grant remainder
    /// floor, `min_daughter` the daughter's grant floor. With both at
    /// `FORK_BUDGET_MIN` this is the Phase 49 function above, bit for bit.
    pub fn fork_budget_grant_with(
        parent_budget: usize,
        min_parent: usize,
        min_daughter: usize,
    ) -> Option<usize> {
        if parent_budget < min_parent + min_daughter { return None; }
        let g = (parent_budget / 4).clamp(min_daughter, FORK_BUDGET_GRANT);
        if parent_budget - g < min_parent { return None; }
        Some(g)
    }

    /// Phase 63: the grant this instance's arms currently allow.
    fn fork_grant_for(&self, parent_budget: usize) -> Option<usize> {
        Self::fork_budget_grant_with(parent_budget, self.fork_budget_min, self.fork_daughter_min)
    }

    /// Phase 63 candidate G arm: earn extra share from the parent by
    /// surviving. Arm-only until the 5-seed A/B decides.
    pub fn fork_budget_growth_on(&mut self) { self.budget_growth = true; }

    /// Phase 63 candidate G arm off (the Phase 49 fixed grant).
    pub fn fork_budget_growth_off(&mut self) { self.budget_growth = false; }

    /// Phase 63 candidate M arm: raise BOTH floors (grant and the parent's
    /// remainder) to `min`. A higher floor takes its extra slots out of the
    /// parent's remainder, so the guild total is unchanged.
    pub fn fork_budget_min_set(&mut self, min: usize) {
        self.fork_budget_min = min;
        self.fork_daughter_min = min;
    }

    /// Phase 63 candidate A arm: raise only the daughter's grant floor; the
    /// parent's remainder floor stays `FORK_BUDGET_MIN`.
    pub fn fork_budget_daughter_min_set(&mut self, min: usize) {
        self.fork_daughter_min = min;
    }

    /// Phase 63: restore the Phase 49 floors (arm reset).
    pub fn fork_budget_floors_reset(&mut self) {
        self.fork_budget_min = FORK_BUDGET_MIN;
        self.fork_daughter_min = FORK_BUDGET_MIN;
    }

    /// Phase 63 candidate G: how many slots `kind` has earned through growth
    /// (0 for roots and for kinds that never grew).
    pub fn kind_growth_granted(&self, kind: SpeciesKind) -> usize {
        self.kind_growth_grants.get(kind as usize).copied().unwrap_or(0)
    }

    // ----- Phase 66 R1/R2: derived-clade mate-radius floor ------------------

    /// Phase 66 R1 arm (moderate floor).
    pub fn mate_floor_r1(&mut self) { self.mate_radius_arm = 1; }

    /// Phase 66 R2 arm (wide floor; screen calibration only).
    pub fn mate_floor_r2(&mut self) { self.mate_radius_arm = 2; }

    /// Phase 67 R3 arm (near-neighbor floor, 80-120).
    pub fn mate_floor_r3(&mut self) { self.mate_radius_arm = 3; }

    /// Phase 66 arm reset: legacy Phase 42.1 radius.
    pub fn mate_floor_off(&mut self) { self.mate_radius_arm = 0; }

    /// Phase 66 arm readout.
    pub fn mate_radius_arm(&self) -> u8 { self.mate_radius_arm }

    /// Phase 66 D0: searches whose radius the floor raised.
    pub fn radius_boost_searches(&self) -> u64 { self.radius_boost_searches }

    /// Phase 66 D0: finds that only the floor's radius made possible.
    pub fn radius_only_finds(&self) -> u64 { self.radius_only_finds }

    /// Phase 67 D0: mean distance of the finds only the floor made possible.
    pub fn radius_only_find_dist_mean(&self) -> f64 {
        if self.radius_only_finds == 0 { 0.0 }
        else { self.radius_only_find_dist_sum / self.radius_only_finds as f64 }
    }

    /// Phase 66: the search radius for one mate attempt and whether the floor
    /// raised it. Pure geometry: `n` is the same-region (same tag) population
    /// of the searcher's kind, so a sparse deme gets a wider search. Roots keep
    /// the legacy radius: their four-quadrant deme structure is the Phase 47
    /// differentiation engine, and Phase 61's candidate a showed the cost of
    /// touching it. FISH keep their band radius.
    fn mate_radius_with_floor(&self, kind: SpeciesKind, region: u8, legacy: f64) -> (f64, bool) {
        if self.mate_radius_arm == 0 { return (legacy, false); }
        if kind == FISH || self.kind_def(kind).parent_kind.is_none() { return (legacy, false); }
        let n = self.region_count(kind, region).max(1) as f64;
        let (ref_dist, min_floor, cap) = if self.mate_radius_arm == 1 {
            (MATE_FLOOR_R1_REF, 0.0, MATE_FLOOR_R1_MAX)
        } else if self.mate_radius_arm == 2 {
            (MATE_FLOOR_R2_REF, 0.0, MATE_FLOOR_R2_MAX)
        } else {
            (MATE_FLOOR_R3_REF, MATE_FLOOR_R3_MIN, MATE_FLOOR_R3_MAX)
        };
        let mut floor = (ref_dist * (MATE_FLOOR_N_REF / n).sqrt()).min(cap);
        if floor < min_floor { floor = min_floor; }
        if floor > legacy { (floor, true) } else { (legacy, false) }
    }

    // ----- Phase 65 W1: fork grace -----------------------------------------

    /// Phase 65 W1 arm on. The window length is `grace_ticks`
    /// (`FORK_GRACE_TICKS` unless the arm overrides it).
    pub fn fork_grace_on(&mut self) { self.budget_grace = true; }

    /// Phase 65 W1 arm off: also drops any pending window (the default world
    /// never has one, so this is exact).
    pub fn fork_grace_off(&mut self) {
        self.budget_grace = false;
        self.kind_grace_until.clear();
    }

    /// Phase 65 W1 arm knob (parallel arms of the same mechanism use one
    /// window length; values below 1 tick are clamped so a window always runs
    /// at least until the next 60-tick settlement pass).
    pub fn fork_grace_ticks_set(&mut self, ticks: u32) {
        self.grace_ticks = ticks.max(1);
    }

    /// Phase 65 W1: is `kind` inside a fork window? Roots never are.
    fn in_fork_grace(&self, kind: SpeciesKind) -> bool {
        if !self.budget_grace { return false; }
        if self.kind_def(kind).parent_kind.is_none() { return false; }
        match self.kind_grace_until.get(kind as usize) {
            Some(&until) => until != 0 && self.tick_count < until,
            None => false,
        }
    }

    fn set_kind_grace_until(&mut self, kind: SpeciesKind, until: u32) {
        while self.kind_grace_until.len() <= kind as usize { self.kind_grace_until.push(0); }
        self.kind_grace_until[kind as usize] = until;
    }

    fn bump_grace_stats(&mut self, kind: SpeciesKind, over: usize) {
        while self.kind_grace_skips.len() <= kind as usize { self.kind_grace_skips.push(0); }
        while self.kind_grace_peak_over.len() <= kind as usize { self.kind_grace_peak_over.push(0); }
        self.kind_grace_skips[kind as usize] += 1;
        if over > self.kind_grace_peak_over[kind as usize] {
            self.kind_grace_peak_over[kind as usize] = over;
        }
    }

    fn add_grace_settled(&mut self, kind: SpeciesKind, slots: usize) {
        while self.kind_grace_settled.len() <= kind as usize { self.kind_grace_settled.push(0); }
        self.kind_grace_settled[kind as usize] += slots;
    }

    fn add_grace_unpaid(&mut self, kind: SpeciesKind, slots: usize) {
        while self.kind_grace_unpaid.len() <= kind as usize { self.kind_grace_unpaid.push(0); }
        self.kind_grace_unpaid[kind as usize] += slots;
    }

    /// Phase 65 W1: births that only the grace window admitted (the legacy
    /// gate would have refused them). The D0 arm criterion reads this sum.
    pub fn kind_grace_skips(&self, kind: SpeciesKind) -> u64 {
        self.kind_grace_skips.get(kind as usize).copied().unwrap_or(0)
    }

    /// Phase 65 W1: peak overdraft (`count − budget`) seen at an admitted
    /// grace birth.
    pub fn kind_grace_peak_over(&self, kind: SpeciesKind) -> usize {
        self.kind_grace_peak_over.get(kind as usize).copied().unwrap_or(0)
    }

    /// Phase 65 W1: slots moved parent→daughter at window close.
    pub fn kind_grace_settled(&self, kind: SpeciesKind) -> usize {
        self.kind_grace_settled.get(kind as usize).copied().unwrap_or(0)
    }

    /// Phase 65 W1: overdraft the parent could not pay (recorded, not minted).
    pub fn kind_grace_unpaid(&self, kind: SpeciesKind) -> usize {
        self.kind_grace_unpaid.get(kind as usize).copied().unwrap_or(0)
    }

    /// Phase 65 W1: pending window end (0 = none).
    pub fn kind_grace_until(&self, kind: SpeciesKind) -> u32 {
        self.kind_grace_until.get(kind as usize).copied().unwrap_or(0)
    }

    /// Phase 65 W1: total admitted overdraft births across every kind (the
    /// pre-registered D0 instrument).
    pub fn grace_total_skips(&self) -> u64 {
        self.kind_grace_skips.iter().sum()
    }

    /// Phase 65 W1 settlement, run from `check_speciation` after the extinct
    /// kinds have returned their shares. One-shot per kind: the window is
    /// cleared before any charge. `over` counts the derived kind's own
    /// members only (founders keep the parent kind and sit outside its grant);
    /// a viable clade charges the parent up to its whole share, a clade below
    /// `speciation_min_side` (or extinct) charges nothing and records the
    /// shortfall. No path mints slots, so every family total is unchanged.
    fn settle_fork_grace(&mut self, now: u32) {
        if !self.budget_grace { return; }
        for k in 0..self.kind_grace_until.len() {
            let until = self.kind_grace_until[k];
            if until == 0 || now < until { continue; }
            self.kind_grace_until[k] = 0;
            let kind = k as SpeciesKind;
            let budget = self.kind_budget(kind);
            let over = self.count(kind).saturating_sub(budget);
            if over == 0 { continue; }
            let min_side = self.speciation_min_side;
            let viable = self.species_clades.iter().any(|c| {
                c.derived_kind == Some(k as u32)
                    && c.death_tick.is_none()
                    && c.members.len() >= min_side
            });
            if !viable {
                self.add_grace_unpaid(kind, over);
                continue;
            }
            let parent = match self.kind_def(kind).parent_kind {
                Some(p) => p,
                None => { self.add_grace_unpaid(kind, over); continue; }
            };
            let parent_budget = self.kind_budget(parent);
            let pay = over.min(parent_budget);
            if pay > 0 {
                self.set_kind_budget(parent, parent_budget - pay);
                self.set_kind_budget(kind, budget + pay);
            }
            self.add_grace_settled(kind, pay);
            self.add_grace_unpaid(kind, over - pay);
        }
    }

    /// Phase 49: summed budget share of every kind in `root`'s guild. The
    /// invariant: this is conserved across forks and extinctions (grants move
    /// budget parent→child, returns move it back up), so a DECO family always
    /// sums to 80 — Phase 45's biomass-explosion lesson.
    pub fn family_budget_total(&self, root: SpeciesKind) -> usize {
        (0..self.kinds.len())
            .filter(|&k| self.root_kind(k as SpeciesKind) == root)
            .map(|k| self.kind_budget(k as SpeciesKind))
            .sum()
    }

    /// Phase 49 A/B arm switch: restore the full Phase 47 behavior — the
    /// family-shared carrying capacity (no per-kind budget split) and the
    /// legacy 2-member minimum side. With both set, the trajectory is
    /// bit-exact with the Phase 47 anchors (pinned in
    /// `phase49_budget_fork_test::budget_fork_off_is_bit_exact_phase47_anchor`).
    pub fn budget_fork_off(&mut self) {
        self.budget_fork = false;
        self.speciation_min_side = SPECIATION_MIN_SIDE_LEGACY;
    }

    /// Phase 51 A/B arm switch: restore the full Phase 50 foraging behaviour
    /// (niche_offset carries no resource preference). With the switch off the
    /// preference multiplier short-circuits to 1.0 on every intake path, so
    /// the trajectory is bit-exact with the Phase 50 anchors (pinned in
    /// `phase51_niche_partition_test::niche_partition_off_is_bit_exact_phase50_anchor`).
    pub fn niche_partition_off(&mut self) {
        self.niche_partition = false;
    }

    /// Phase 53 A/B arm switch: restore the Phase 52 niche projection
    /// (eff-gap only, no guaranteed generational minimum distance).
    pub fn niche_gen_accum_off(&mut self) {
        self.niche_gen_accum = false;
    }

    /// Phase 54 A/B arm switch: structures stop touching the environment
    /// (the multiplier field is reset to all-1.0 and frozen).
    pub fn structure_env_off(&mut self) {
        self.structure_env_on = false;
        for v in self.structure_env.iter_mut() { *v = 1.0; }
        self.structure_env_decay_active = false;
    }

    /// Phase 56 A/B arm switch: structures stop touching the decomposition
    /// field (the carcass decay multiplier is reset to all-1.0 and frozen, so
    /// the carcass pass consumes exactly the pre-56 constant rate).
    pub fn deco_resource_env_off(&mut self) {
        self.deco_env_on = false;
        for v in self.deco_env.iter_mut() { *v = 1.0; }
        self.deco_env_decay_active = false;
    }

    /// Phase 57 A/B arm switch: enable the sign-neutral placement reward
    /// (`gain = |grass_mod| × 10`). The compiled default is the Phase 56
    /// signed formula — see `culture::TOKEN_VALUE_SIGN_NEUTRAL` for the
    /// 200k A/B verdict that kept it off.
    pub fn token_value_sign_neutral_on(&mut self) {
        self.token_value_sign_neutral = true;
    }

    /// Phase 58 experiment arm: override the decomposition-multiplier bounds
    /// (`STRUCTURE_DECO_MIN/MAX` by default). Diagnostics only — the bounds
    /// are not persisted, so this never reaches a saved world.
    pub fn deco_env_clamp_on(&mut self, min: f32, max: f32) {
        self.deco_env_min = min;
        self.deco_env_max = max;
    }

    /// Phase 58 design b arm switch: placements pick the token with the
    /// highest emergent value instead of the first known token.
    pub fn token_placement_by_value_on(&mut self) {
        self.token_placement_by_value = true;
    }

    /// Phase 59 A/B arm: restore the Phase 58 fertility mapping
    /// (`0.96 × (2 − eff)`, sterile for `eff < 0.9583`). The default world is
    /// the saturated fix; this is the only configuration that replays the
    /// pre-Phase-59 anchors.
    pub fn repro_threshold_old_off(&mut self) {
        self.deco_repro_saturate = false;
    }

    /// Phase 60 A/B arm: regional DECO carrying capacity ON (the split-trigger
    /// candidate). Force a refresh at the first birth gate so a world switched
    /// on mid-run reads the current field, not a stale cache.
    pub fn deco_region_cap_on(&mut self) {
        self.deco_region_cap = true;
        self.deco_region_weight_tick = u32::MAX;
    }

    /// Phase 60 A/B arm: off = the Phase 59 world (global kind budget only).
    /// The gate is never reached, so no state is touched and the trajectory
    /// stays bit-exact with the pre-60 default.
    pub fn deco_region_cap_off(&mut self) {
        self.deco_region_cap = false;
    }

    /// Phase 61 A/B arm: renormalise the regional split over the occupied
    /// regions (a one-region clade may use its whole budget). Only meaningful
    /// while `deco_region_cap` is on; not persisted.
    pub fn deco_region_cap_renorm_on(&mut self) {
        self.deco_region_cap_renorm = true;
        self.deco_region_weight_tick = u32::MAX;
    }

    /// Phase 61 A/B arm: the Phase 60 split (normalise over all four regions).
    pub fn deco_region_cap_renorm_off(&mut self) {
        self.deco_region_cap_renorm = false;
    }

    /// Phase 61 candidate (b) arm: occupied regions keep at least
    /// `min(budget, speciation_min_side)` slots for the kind that holds them.
    pub fn deco_region_cap_floor_on(&mut self) {
        self.deco_region_cap_floor = true;
        self.deco_region_cap_floor_small = false;
        self.deco_region_weight_tick = u32::MAX;
    }

    /// Phase 61 candidate (b) arm off (the Phase 60 split).
    pub fn deco_region_cap_floor_off(&mut self) {
        self.deco_region_cap_floor = false;
        self.deco_region_cap_floor_small = false;
    }

    /// Phase 62 sensitivity arm (d): candidate (b)'s floor, but only for kinds
    /// whose budget is below `2 × speciation_min_side`. Implies the floor arm.
    pub fn deco_region_cap_floor_small_on(&mut self) {
        self.deco_region_cap_floor = true;
        self.deco_region_cap_floor_small = true;
        self.deco_region_weight_tick = u32::MAX;
    }

    /// Phase 61 candidate (c) arm: renormalise the derived kinds only.
    pub fn deco_region_cap_renorm_daughters_on(&mut self) {
        self.deco_region_cap_renorm_daughters = true;
        self.deco_region_weight_tick = u32::MAX;
    }

    /// Phase 61 candidate (c) arm off.
    pub fn deco_region_cap_renorm_daughters_off(&mut self) {
        self.deco_region_cap_renorm_daughters = false;
    }

    /// Phase 60: refresh the per-home-region capacity weights from the current
    /// decomposition field. Mean `deco_env` over habitable cells only; a region
    /// that rots carcasses faster retains less food, so its weight falls.
    /// Pure arithmetic + table lookups, zero RNG.
    fn refresh_deco_region_weights(&mut self) {
        let nr = REGION_COUNT as usize;
        let mut sum = [0.0f64; REGION_COUNT as usize];
        let mut cnt = [0usize; REGION_COUNT as usize];
        let cells = self.deco_env.len().min(self.deco_cell_region.len());
        for j in 0..cells {
            if self.suit[j] <= 0.02 { continue; }
            let r = self.deco_cell_region[j] as usize;
            if r >= nr { continue; }
            sum[r] += self.deco_env[j] as f64;
            cnt[r] += 1;
        }
        let mut wsum = 0.0f64;
        for r in 0..nr {
            let mean = if cnt[r] > 0 { sum[r] / cnt[r] as f64 } else { 1.0 };
            let w = (1.0 + DECO_REGION_CAP_GAIN * (1.0 - mean))
                .clamp(DECO_REGION_CAP_W_MIN, DECO_REGION_CAP_W_MAX);
            self.deco_region_weight[r] = w;
            wsum += w;
        }
        self.deco_region_weight_sum = wsum;
        self.deco_region_weight_tick = self.tick_count;
    }

    /// Phase 60: one kind's capacity share for one home region. The Phase-49
    /// budget is partitioned by the region weights, so `Σ_regions cap_r` is
    /// exactly `kind_budget(kind)` — the guild's total carrying capacity is
    /// conserved and each species gets its own regional share.
    ///
    /// Phase 61: with `deco_region_cap_renorm` the normaliser covers only the
    /// regions the kind occupies (plus the target region), so a clade that
    /// holds one quadrant may fill its whole budget there. The conservation
    /// identity moves with it: the occupied regions' caps still sum to the
    /// kind budget.
    fn deco_region_cap_limit(&self, kind: SpeciesKind, region: u8) -> f64 {
        let budget = self.kind_budget(kind) as f64;
        let r = (region as usize).min(REGION_COUNT as usize - 1);
        if self.deco_region_cap_renorm
            || (self.deco_region_cap_renorm_daughters && self.kind_def(kind).parent_kind.is_some()) {
            let mut wsum = 0.0f64;
            for rr in 0..REGION_COUNT as usize {
                if rr == r || self.deco_region_count(kind, rr as u8) > 0 {
                    wsum += self.deco_region_weight[rr];
                }
            }
            if wsum > 0.0 {
                return budget * self.deco_region_weight[r] / wsum;
            }
        }
        let natural = if self.deco_region_weight_sum > 0.0 {
            budget * self.deco_region_weight[r] / self.deco_region_weight_sum
        } else {
            budget / REGION_COUNT as f64
        };
        let floor_applies = !self.deco_region_cap_floor_small
            || (budget as usize) < 2 * self.speciation_min_side;
        if self.deco_region_cap_floor && floor_applies
            && self.deco_region_count(kind, r as u8) > 0
        {
            let floor = budget.min(self.speciation_min_side as f64);
            return natural.max(floor);
        }
        natural
    }

    /// Phase 60: living members of `kind` whose home quadrant is `region`.
    fn deco_region_count(&self, kind: SpeciesKind, region: u8) -> usize {
        self.agents.iter()
            .filter(|a| a.alive && a.kind == kind && a.region == region)
            .count()
    }

    /// Phase 60 diagnostic: the cached per-region capacity weights (the values
    /// the most recent birth gate used).
    pub fn deco_region_weights(&self) -> [f64; REGION_COUNT as usize] {
        self.deco_region_weight
    }

    /// Phase 60 diagnostic: the slot limit for `kind` in one home region —
    /// `kind_budget × w_r / Σw` (sums to the kind's budget over all regions).
    pub fn deco_region_cap_for(&self, kind: SpeciesKind, region: u8) -> f64 {
        self.deco_region_cap_limit(kind, region)
    }

    // ----- Phase 61: per-clade life telemetry ---------------------------------

    /// Phase 61: every clade's birth/death bookkeeping, in first-touch order.
    pub fn clade_life_stats(&self) -> &[CladeLifeStats] {
        &self.clade_life
    }

    /// Find (or create) the `(clade_id, root_kind)` telemetry slot.
    fn clade_life_slot(&mut self, clade_id: u32, root_kind: SpeciesKind) -> usize {
        if let Some(i) = self.clade_life.iter()
            .position(|s| s.clade_id == clade_id && s.root_kind == root_kind) {
            return i;
        }
        self.clade_life.push(CladeLifeStats {
            clade_id,
            root_kind,
            first_tick: self.tick_count,
            ..CladeLifeStats::default()
        });
        self.clade_life.len() - 1
    }

    /// A parented birth into `clade_id` (agent `kind` resolves the family).
    fn clade_life_birth(&mut self, clade_id: u32, kind: SpeciesKind) {
        let root = self.root_kind(kind);
        let now = self.tick_count;
        let i = self.clade_life_slot(clade_id, root);
        let s = &mut self.clade_life[i];
        s.births += 1;
        if s.first_birth_tick.is_none() { s.first_birth_tick = Some(now); }
        s.last_birth_tick = Some(now);
    }

    /// A death of agent `idx` (still readable: callers run this before the
    /// agent stops being counted, alive or not).
    fn clade_life_death(&mut self, idx: usize, cause: &'static str) {
        let (clade_id, kind, age, energy) = {
            let a = &self.agents[idx];
            (a.clade_id, a.kind, a.age, a.energy)
        };
        let off_region = Self::region_of(self.agents[idx].x, self.agents[idx].z) != self.agents[idx].region;
        let root = self.root_kind(kind);
        let i = self.clade_life_slot(clade_id, root);
        let s = &mut self.clade_life[i];
        if off_region { s.deaths_off_region += 1; }
        s.deaths += 1;
        match cause {
            "starved" => s.deaths_starved += 1,
            "aged" => s.deaths_aged += 1,
            "predation" => s.deaths_predation += 1,
            _ => s.deaths_other += 1,
        }
        s.death_age_sum += age;
        s.death_energy_sum += energy;
        s.last_death_energy = energy;
    }

    /// A refused birth, attributed to the mother's clade.
    fn clade_life_block(&mut self, tag: (u32, SpeciesKind), why: CladeBirthBlock) {
        let root = self.root_kind(tag.1);
        let i = self.clade_life_slot(tag.0, root);
        match why {
            CladeBirthBlock::RegionCap => self.clade_life[i].region_cap_blocks += 1,
            CladeBirthBlock::KindBudget => self.clade_life[i].kind_budget_blocks += 1,
        }
    }

    /// Phase 66 diagnostic: classify a miss by the constraint that binds it.
    /// Priority follows what the engine could change: a fertile same-region
    /// partner beyond the radius (`beyond_radius`) is fixed by a wider search; a
    /// same-region partner inside the radius that fails the age/cooldown/energy
    /// test (`filtered`) is fixed only by the fertility side; a conspecific that
    /// exists but lives in another region (`other_region`) needs recruitment;
    /// no conspecific at all (`no_kin`) needs a new deme.
    fn mate_miss_bucket(&self, idx: usize, radius: f64, min_e: f64) -> (MateMissBucket, usize, usize, Option<f64>) {
        let a = &self.agents[idx];
        let (a_region, kind) = (a.region, a.kind);
        let (a_x, a_z) = (a.x, a.z);
        let r2 = radius * radius;
        let mut kind_pop = 0usize;
        let mut region_pop = 0usize;
        let mut ready_far = false;
        let mut unready_near = false;
        let mut any_other_region = false;
        let mut ready_far_min = f64::INFINITY;
        for (j, b) in self.agents.iter().enumerate() {
            if j == idx || !b.alive || b.kind != kind { continue; }
            kind_pop += 1;
            if self.region_isolation && b.region != a_region { any_other_region = true; continue; }
            region_pop += 1;
            let min_e_b = if kind == DECO && self.deco_eff_choice_mode == save_state::DECO_CHOICE_SELECTIVE {
                Self::deco_repro_threshold_for(b.genome.deco_efficiency, self.deco_repro_saturate)
            } else {
                min_e
            };
            let ready = b.age > 30.0 && b.repro <= 0.0 && b.energy > b.max_energy * min_e_b;
            let dx = b.x - a_x;
            let dz = b.z - a_z;
            let d2 = dx * dx + dz * dz;
            if ready && d2 < r2 {
                return (MateMissBucket::Other, kind_pop + 1, region_pop + 1, None);
            }
            if ready {
                ready_far = true;
                if d2 < ready_far_min { ready_far_min = d2; }
            } else if d2 < r2 {
                unready_near = true;
            }
        }
        let bucket = if kind_pop == 0 {
            MateMissBucket::NoKin
        } else if region_pop == 0 && self.region_isolation {
            MateMissBucket::OtherRegion
        } else if ready_far {
            MateMissBucket::BeyondRadius
        } else if unready_near {
            MateMissBucket::Filtered
        } else {
            MateMissBucket::Other
        };
        let dist = if bucket == MateMissBucket::BeyondRadius { Some(ready_far_min.sqrt()) } else { None };
        (bucket, kind_pop + 1, region_pop + 1, dist)
    }

    /// Phase 66 diagnostic: a miss plus the classification of its cause.
    fn clade_life_mate_miss_detail(&mut self, clade_id: u32, kind: SpeciesKind, bucket: MateMissBucket, kind_pop: usize, region_pop: usize, ready_dist: Option<f64>) {
        let root = self.root_kind(kind);
        let young = self.clade_is_young(clade_id);
        let i = self.clade_life_slot(clade_id, root);
        let s = &mut self.clade_life[i];
        s.mate_misses += 1;
        if young { s.mate_miss_young += 1; }
        match bucket {
            MateMissBucket::NoKin => { s.mate_miss_no_kin += 1; if young { s.mate_miss_young_no_kin += 1; } }
            MateMissBucket::OtherRegion => { s.mate_miss_other_region += 1; if young { s.mate_miss_young_other_region += 1; } }
            MateMissBucket::BeyondRadius => { s.mate_miss_beyond_radius += 1; if young { s.mate_miss_young_beyond_radius += 1; } }
            MateMissBucket::Filtered => { s.mate_miss_filtered += 1; if young { s.mate_miss_young_filtered += 1; } }
            MateMissBucket::Other => { s.mate_miss_other += 1; if young { s.mate_miss_young_other += 1; } }
        }
        s.mate_miss_kind_pop_sum += kind_pop as u64;
        s.mate_miss_region_pop_sum += region_pop as u64;
        if let Some(d) = ready_dist {
            s.mate_miss_ready_dist_sum += d;
            if s.mate_miss_beyond_radius == 1 || d < s.mate_miss_ready_dist_min {
                s.mate_miss_ready_dist_min = d;
            }
        }
    }

    /// Phase 66 diagnostic: is this clade inside its founding window? The
    /// clade record's `birth_tick` is the fork tick; a clade without a record
    /// is the root (age = world age).
    fn clade_is_young(&self, clade_id: u32) -> bool {
        let birth = self.species_clades.iter()
            .find(|c| c.id == clade_id)
            .map(|c| c.birth_tick)
            .unwrap_or(0);
        self.tick_count.saturating_sub(birth) < YOUNG_CLADE_TICKS
    }

    /// Phase 66 diagnostic: the birth that fired natal dispersal belongs to
    /// this clade (the family left its home quadrant).
    fn clade_life_dispersal(&mut self, clade_id: u32, kind: SpeciesKind) {
        let root = self.root_kind(kind);
        let i = self.clade_life_slot(clade_id, root);
        self.clade_life[i].dispersal_out += 1;
    }

    /// Standing population sampled at the speciation-check cadence (the pass
    /// that rebuilds `SpeciesClade::members`; 60-tick granularity), with the
    /// per-quadrant split for the capacity evidence.
    fn clade_life_peak(&mut self, clade_id: u32, kind_id: u32, pop: usize, by_region: &[u32; REGION_COUNT as usize]) {
        if kind_id == u32::MAX { return; }
        let root = self.root_kind(kind_id as SpeciesKind);
        let now = self.tick_count;
        let i = self.clade_life_slot(clade_id, root);
        if pop as u32 > self.clade_life[i].peak_pop {
            self.clade_life[i].peak_pop = pop as u32;
            self.clade_life[i].peak_tick = now;
        }
        for r in 0..REGION_COUNT as usize {
            if by_region[r] > self.clade_life[i].region_peak[r] {
                self.clade_life[i].region_peak[r] = by_region[r];
            }
        }
    }

    /// The tick the engine logged this clade's extinction event.
    fn clade_life_extinct(&mut self, clade_id: u32, kind_id: u32) {
        if kind_id == u32::MAX { return; }
        let root = self.root_kind(kind_id as SpeciesKind);
        let now = self.tick_count;
        let i = self.clade_life_slot(clade_id, root);
        self.clade_life[i].extinction_tick = Some(now);
    }

    /// Phase 59: the DECO reproduction threshold for a given efficiency.
    /// Legacy `0.96 × (2 − eff)` up to `DECO_REPRO_SAT_KNEE` (so eff ≥ 1.0
    /// and the 0.48 easy-breed end are unchanged), then the excess is bent
    /// into `(KNEE, CEIL)` by `x/(1+x)`. Pure `+ - * /` arithmetic keeps the
    /// map bit-identical across native and wasm builds.
    pub fn deco_repro_threshold(&self, eff: f64) -> f64 {
        Self::deco_repro_threshold_for(eff, self.deco_repro_saturate)
    }

    pub fn deco_repro_threshold_for(eff: f64, saturate: bool) -> f64 {
        let legacy = DECO_REPRO_BASE * (2.0 - eff);
        if !saturate || legacy <= DECO_REPRO_SAT_KNEE {
            return legacy;
        }
        let span = DECO_REPRO_SAT_CEIL - DECO_REPRO_SAT_KNEE;
        let x = (legacy - DECO_REPRO_SAT_KNEE) / span;
        DECO_REPRO_SAT_KNEE + span * x / (1.0 + x)
    }

    /// Which of the agent's known tokens a placement stamps: the first one
    /// (inheritance order, Phase 56/57 world) or, in the design-b arm, the
    /// highest `use_gain` (ties → lowest id, so the pick is order-free).
    fn placement_token(&self, idx: usize) -> u32 {
        let toks = &self.agents[idx].known_tokens;
        if !self.token_placement_by_value || toks.len() == 1 { return toks[0]; }
        let mut best = toks[0];
        let mut best_v = self.token_registry.value_of(best);
        for &t in toks.iter().skip(1) {
            let v = self.token_registry.value_of(t);
            if v > best_v || (v == best_v && t < best) { best_v = v; best = t; }
        }
        best
    }

    /// Phase 55: apply one structure placement to the regrowth field as a
    /// net-zero dipole. Pass 1 boosts the disk (Chebyshev radius
    /// `STRUCTURE_ENV_RADIUS`) under the clamp and measures what the field
    /// actually absorbed; pass 2 subtracts exactly that from the surrounding
    /// ring (`STRUCTURE_ENV_RADIUS`..=`STRUCTURE_ENV_RING`). The transfer is
    /// exact while the clamps don't bind; where a ring cell sits at its
    /// bound the payout is only ever truncated, and the disk's own
    /// saturation (a disk pinned at MAX absorbs nothing) stops any runaway
    /// gain, so culture can choose WHERE regrowth happens but can never own
    /// the world's total the way the Phase 54 one-sided boost did.
    pub fn structure_env_dipole(&mut self, x: f64, z: f64, grass_mod: f64) {
        let ci0 = self.cell_index(x, z);
        if ci0 < 0 { return; }
        self.structure_env_decay_active = true;
        let boost = (grass_mod as f32) * STRUCTURE_ENV_BOOST;
        let c0 = ci0 as usize;
        let col = (c0 % GRID_N as usize) as i32;
        let row = (c0 / GRID_N as usize) as i32;
        let r = STRUCTURE_ENV_RADIUS;
        let ring = STRUCTURE_ENV_RING;
        let gn = GRID_N as i32;
        let mut absorbed = 0.0f32;
        let mut n_ring = 0usize;
        for dc in -ring..=ring {
            let cc = col + dc;
            if cc < 0 || cc >= gn { continue; }
            for dr in -ring..=ring {
                let rr = row + dr;
                if rr < 0 || rr >= gn { continue; }
                let cheb = dc.abs().max(dr.abs());
                let j = (rr as usize) * GRID_N as usize + cc as usize;
                if cheb <= r {
                    let m = (self.structure_env[j] + boost)
                        .clamp(STRUCTURE_ENV_MIN, STRUCTURE_ENV_MAX);
                    absorbed += m - self.structure_env[j];
                    self.structure_env[j] = m;
                } else {
                    n_ring += 1;
                }
            }
        }
        if absorbed != 0.0 && n_ring > 0 {
            let comp = -absorbed / n_ring as f32;
            for dc in -ring..=ring {
                let cc = col + dc;
                if cc < 0 || cc >= gn { continue; }
                for dr in -ring..=ring {
                    let rr = row + dr;
                    if rr < 0 || rr >= gn { continue; }
                    if dc.abs().max(dr.abs()) <= r { continue; }
                    let j = (rr as usize) * GRID_N as usize + cc as usize;
                    self.structure_env[j] = (self.structure_env[j] + comp)
                        .clamp(STRUCTURE_ENV_MIN, STRUCTURE_ENV_MAX);
                }
            }
        }
    }

    /// Phase 56: carcass decomposition multiplier at a world position — 1.0
    /// whenever the A/B is off or the position is off-grid, which is when the
    /// carcass pass is bit-identical to the pre-56 constant rate.
    pub fn deco_decay_mult_at(&self, x: f64, z: f64) -> f64 {
        if !self.deco_env_on { return 1.0; }
        let ci = self.cell_index(x, z);
        if ci < 0 { return 1.0; }
        self.deco_env[ci as usize] as f64
    }

    /// Phase 56: apply one structure placement to the decomposition field as a
    /// net-zero dipole. The direction comes from the token's `grass_mod` sign
    /// (its magnitude scales the imprint): an enrichment token slows
    /// decomposition on its disk (the soil around the structure holds carcass
    /// mass for its decomposers) and the ring pays back the absorbed larder by
    /// rotting faster; a depletion token does the mirror image. Culture
    /// therefore chooses WHERE carcass mass stays available, never how much of
    /// it the world keeps.
    ///
    /// Conservation is exact, not approximate. Phase 55's ring payout could
    /// only truncate at the clamp, which biases the field mean (measured
    /// −0.7 % at gain 10) — on a food-supply field that bias is a global
    /// change of the decomposition rate, the very thing the net-zero contract
    /// exists to forbid. So the ring's capacity in the payout direction is
    /// measured FIRST and the disk's whole imprint is scaled down to what the
    /// ring can actually absorb; the payout then spills over ring cells that
    /// clamp until the absorbed amount is fully returned.
    pub fn deco_env_dipole(&mut self, x: f64, z: f64, grass_mod: f64) {
        let ci0 = self.cell_index(x, z);
        if ci0 < 0 { return; }
        if grass_mod == 0.0 { return; }
        let boost = -(grass_mod as f32) * STRUCTURE_DECO_GAIN;
        let dir = if boost > 0.0 { 1.0f32 } else { -1.0f32 };
        let c0 = ci0 as usize;
        let col = (c0 % GRID_N as usize) as i32;
        let row = (c0 / GRID_N as usize) as i32;
        let r = STRUCTURE_DECO_RADIUS;
        let ring = STRUCTURE_DECO_RING;
        let gn = GRID_N as i32;
        let mut disk: Vec<usize> = Vec::with_capacity(49);
        let mut ring_cells: Vec<usize> = Vec::with_capacity(72);
        for dc in -ring..=ring {
            let cc = col + dc;
            if cc < 0 || cc >= gn { continue; }
            for dr in -ring..=ring {
                let rr = row + dr;
                if rr < 0 || rr >= gn { continue; }
                let j = (rr as usize) * GRID_N as usize + cc as usize;
                if dc.abs().max(dr.abs()) <= r { disk.push(j); } else { ring_cells.push(j); }
            }
        }
        if disk.is_empty() || ring_cells.is_empty() { return; }
        let (lo, hi) = (self.deco_env_min, self.deco_env_max);
        // Room a cell still has in direction `d` (+1 = upward).
        let room = |v: f32, d: f32| if d > 0.0 {
            hi - v
        } else {
            v - lo
        };
        let ring_room: f32 = ring_cells.iter().map(|&j| room(self.deco_env[j], -dir)).sum();
        let planned = boost.abs() * disk.len() as f32;
        let scale = if ring_room < planned { ring_room / planned } else { 1.0 };
        if scale <= 0.0 { return; }
        self.deco_env_decay_active = true;
        let delta = boost * scale;
        let mut absorbed = 0.0f32;
        for &j in &disk {
            let v = self.deco_env[j];
            let m = (v + delta).clamp(lo, hi);
            absorbed += (m - v).abs();
            self.deco_env[j] = m;
        }
        if absorbed == 0.0 { return; }
        // Payout: uniform share first, then the shortfall over ring cells that
        // still had room (the pre-measured capacity guarantees full payment).
        let mut unpaid = 0.0f32;
        let mut share = absorbed / ring_cells.len() as f32;
        for &j in &ring_cells {
            let room_j = room(self.deco_env[j], -dir);
            let pay = if room_j < share { room_j } else { share };
            self.deco_env[j] -= dir * pay;
            unpaid += share - pay;
        }
        for _ in 0..4 {
            if unpaid <= 1e-6 { break; }
            let mut open = 0usize;
            for &j in &ring_cells {
                if room(self.deco_env[j], -dir) > 1e-6 { open += 1; }
            }
            if open == 0 { break; }
            share = unpaid / open as f32;
            let mut still = 0.0f32;
            for &j in &ring_cells {
                let room_j = room(self.deco_env[j], -dir);
                if room_j <= 1e-6 { continue; }
                let pay = if room_j < share { room_j } else { share };
                self.deco_env[j] -= dir * pay;
                still += share - pay;
            }
            unpaid = still;
        }
    }

    /// Phase 49: the density-dependent crowding denominator for `kind` — its
    /// own budget share when the fork budget is active, the legacy global cap
    /// otherwise. Before any fork these are identical (a root kind's share
    /// starts at its global cap), so the pre-first-split trajectory is
    /// bit-exact; after a fork each species feels crowding relative to the
    /// resources it actually holds, which is the ecological consequence of
    /// splitting a share.
    fn crowd_cap(&self, kind: SpeciesKind) -> f64 {
        if self.budget_fork { self.kind_budget(kind).max(1) as f64 }
        else { self.kind_def(kind).global_cap as f64 }
    }

    /// Mendelian mix + mutation. `mut_scale` comes from the reproducing
    /// partner's somatic damage (Phase 39.1: dirty work raises the offspring
    /// mutation amplitude; 1.0 keeps the pre-39.1 trajectory bit-exact).
    fn mix_genome(&mut self, a: &Genome, b: &Genome, kind: SpeciesKind, mut_scale: f64) -> Genome {
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
        self.mutate(&g, kind, mut_scale)
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

    pub(crate) fn spawn(&mut self, kind: SpeciesKind, x: f64, z: f64, genome: Option<Genome>, generation: u32, age: f64) -> Option<&mut Agent> {
        self.spawn_tagged(kind, x, z, genome, generation, age, None)
    }

    /// Phase 61: `spawn` with a per-clade telemetry tag `(mother clade id,
    /// mother kind)`. The tag only feeds the Phase 61 counters — refusals and
    /// births — and changes nothing in the decision itself.
    pub(crate) fn spawn_tagged(&mut self, kind: SpeciesKind, x: f64, z: f64, genome: Option<Genome>, generation: u32, age: f64, tag: Option<(u32, SpeciesKind)>) -> Option<&mut Agent> {
        // Use biome-specific cap when available; otherwise fall back to global.
        let def_max_e = self.kind_def(kind).max_energy;
        let def_repro_cd = self.kind_def(kind).repro_cooldown;
        let (se0, se1) = self.kind_def(kind).start_energy;
        let cap = self.biome_max_for(kind, x, z);
        // Phase 65 W1: whether this birth is only admissible because of the
        // grace window (the legacy gate would have refused it). Recorded on
        // success only — a region-cap refusal leaves the trajectory identical.
        let mut grace_admitted = false;
        // Phase 49 A/B: with the fork budget active, each kind's births are
        // gated by its OWN carved-out share (min with the local biome cap):
        // a daughter's growth is decoupled from the root's occupancy. The
        // legacy arm keeps the Phase 47 family-shared check bit-exact.
        if self.budget_fork {
            let budget = self.kind_budget(kind);
            // Phase 65 W1: a newborn kind's births skip the budget gate for
            // one window (the regional cap below still applies). The family
            // table stays untouched until settlement.
            let grace = self.in_fork_grace(kind);
            let allowance = if grace {
                budget.saturating_add(FORK_GRACE_OVERDRAFT_MAX)
            } else {
                budget
            };
            let eff_cap = if allowance < cap { allowance } else { cap };
            let standing = self.count(kind);
            if standing >= eff_cap {
                if let Some(t) = tag { self.clade_life_block(t, CladeBirthBlock::KindBudget); }
                return None;
            }
            grace_admitted = grace && standing >= budget;
        } else {
            let root = self.root_kind(kind);
            let family_total = if root == kind {
                self.count(kind)
            } else {
                self.count_family(root)
            };
            if family_total >= cap {
                if let Some(t) = tag { self.clade_life_block(t, CladeBirthBlock::KindBudget); }
                return None;
            }
        }
        // Phase 60: regional carrying capacity for the DECO guild. The kind's
        // Phase-49 budget is spread over the four home quadrants by local food
        // retention (refresh at most once per tick, on demand). Refusing the
        // birth when the child's region holds its share is a slot capacity:
        // the mother keeps her energy, stays fertile, and the next birth
        // succeeds as soon as the region loses a member. With the decomposition
        // field switched off there is no structural food signal, so the
        // capacity instrument is off too — `deco_resource_env_off()` keeps the
        // pre-56 world bit-exact.
        if self.deco_region_cap && self.deco_env_on && self.root_kind(kind) == DECO {
            if self.deco_region_weight_tick != self.tick_count {
                self.refresh_deco_region_weights();
            }
            let r = Self::region_of(x, z);
            let limit = self.deco_region_cap_limit(kind, r);
            if self.deco_region_count(kind, r) as f64 >= limit {
                if let Some(t) = tag { self.clade_life_block(t, CladeBirthBlock::RegionCap); }
                return None;
            }
        }
        let g = match genome {
            Some(g) => g,
            None => {
                let rg = self.random_genome(kind);
                self.mutate(&rg, kind, 1.0)
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
            region: Self::region_of(x, z),
            offspring: 0,
            // Phase 36: seed the micro-brain from the kind's grandfather
            // reference network plus per-agent deterministic noise (zero main
            // RNG draws; the noise stream is agent-id derived).
            brain: self.initial_brain(kind, self.next_id - 1),
            // Phase 41: three-layer brain seeded from the kind's reference
            // genome (agents of the same kind still differ via id noise).
            triune: self.initial_triune(kind, self.next_id - 1),
            // Phase 42: 新生个体的行为权威从 FSM 开始——三脑必须用自己的一生
            // 再次赢回一致度门（脑与身体同步发育，不继承亲代的权威状态）。
            triune_equiv_streak: 0,
            triune_agree_bits: 0,
            triune_agree_n: 0,
            brain_authority: BRAIN_AUTHORITY_FSM,
            brain_equiv_streak: 0,
            brain_equiv_start: 0,
            brain_agree_window: 0,
            brain_window_len: 0,
            brain_last_fsm_action: 0,
            brain_last_brain_action: 0,
            // Phase 37: empty token set at birth — inherited via learning.
            known_tokens: Vec::new(),
            // Phase 39.1: newborns start solitary with no role and no damage.
            group_id: 0,
            role: crate::group::ROLE_NONE,
            somatic_damage: 0.0,
        };
        let idx = self.agents.len();
        self.agents.push(a);
        if grace_admitted {
            let over = self.count(kind).saturating_sub(self.kind_budget(kind));
            self.bump_grace_stats(kind, over);
        }
        if let Some(t) = tag { self.clade_life_birth(t.0, t.1); }
        Some(&mut self.agents[idx])
    }

    // ----- Phase 43: abiogenesis（生命再起源）---------------------------------

    /// 生命再起源：世界接近死寂时，热泉化学能自发萌发一个祖先个体。
    ///
    /// 候选格 = 矿化活跃（mineral ≥ ABIO_MINERAL_TH）或 水缘高湿×肥沃
    /// （moist > ABIO_MOIST_TH && suit > ABIO_SUIT_TH）；强度 = 两路化学能
    /// 代理各半（矿化度、湿度归一）。每 tick 以 ABIO_P_BASE × mean(强度)
    /// 概率触发；命中后均匀选一个候选格，按 PARENT_KIND == None 的注册表
    /// 序随机选一个与该格栖息地相容的祖先（水面格 → 鱼形，陆地格 → 非鱼形），
    /// spawn 1 个个体；初始能量入账 `ledger.chemo_in`（热泉化学能输入通道）。
    ///
    /// Draw 预算固定可审计：pop < ABIO_MIN_POP 且候选非空时每 tick ≤ 1 次触发
    /// roll，命中后 +1 选格 +1 选祖先；健康世界（pop ≥ 门限）零扫描零 draw。
    fn try_abiogenesis(&mut self) {
        if !self.abiogenesis_on { return; }
        // Phase 43.1: geologic cadence — abiogenesis is a rare-chemistry
        // event, so the liveness pre-check runs on a 50-tick rhythm instead
        // of every tick. The full-struct filter walks ~2 MB of agent state
        // per call on the wasm viewer workload, which alone ate ~7 ms/frame;
        // a 5-second (sim-time) sampling granularity costs nothing at this
        // timescale (the check itself keeps the per-check probability
        // compensated ×50 so the expected trigger rate is unchanged) and
        // only ever shifts a trigger by ≤50 ticks.
        if self.tick_count % 50 != 0 { return; }
        if self.agents.iter().filter(|a| a.alive).count() >= ABIO_MIN_POP { return; }
        let n = (self.grid_n * self.grid_n) as usize;
        let mut candidates: Vec<usize> = Vec::new();
        let mut intensity_sum = 0.0f64;
        for i in 0..n {
            let mineral = self.mineral_field[i] as f64;
            let moist = self.moist[i] as f64;
            let suit = self.suit[i] as f64;
            let mineral_ok = mineral >= ABIO_MINERAL_TH as f64;
            let wet_ok = moist > ABIO_MOIST_TH as f64 && suit > ABIO_SUIT_TH as f64;
            if !(mineral_ok || wet_ok) { continue; }
            candidates.push(i);
            intensity_sum += 0.5 * mineral + 0.5 * (moist / 0.45).min(1.0);
        }
        if candidates.is_empty() { return; }
        let p = (ABIO_P_BASE * 50.0 * (intensity_sum / candidates.len() as f64)).min(1.0);
        if self.abio_rng.next_f64() >= p { return; }
        let cell = candidates[(self.abio_rng.next_f64() * candidates.len() as f64) as usize % candidates.len()];
        let gi = (cell % self.grid_n as usize) as f64;
        let gj = (cell / self.grid_n as usize) as f64;
        let x = -GRID_SPAN / 2.0 + (gi + 0.5) * CELL;
        let z = -GRID_SPAN / 2.0 + (gj + 0.5) * CELL;
        let aquatic = terrain_height(x, z) < WATER_LEVEL;
        let ancestors: Vec<SpeciesKind> = self.kinds.defs().iter()
            .filter(|d| d.parent_kind.is_none())
            .filter(|d| (d.behavior == BehaviorTemplate::Fish) == aquatic)
            .map(|d| d.id)
            .collect();
        if ancestors.is_empty() { return; }
        let kind = ancestors[(self.abio_rng.next_f64() * ancestors.len() as f64) as usize % ancestors.len()];
        let spawned = self.spawn(kind, x, z, None, 0, 0.0).is_some();
        if std::env::var("ABIO_DEBUG").is_ok() && !spawned {
            eprintln!("[abio] spawn FAILED kind={} aquatic={} at ({:.0},{:.0}) terrain={:.2}", 
                self.kind_name(kind), aquatic, x, z, terrain_height(x, z));
        }
        if spawned {
            let idx = self.agents.len() - 1;
            self.ledger.chemo_in += self.agents[idx].energy;
            self.abiogenesis_events += 1;
            self.abiogenesis_ticks = self.tick_count;
        }
    }

    // ----- grass field helpers ----------------------------------------------

    pub(crate) fn cell_index(&self, x: f64, z: f64) -> isize {
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
        let lineage_bite = self.herb_lineage_bite(line_a);
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

    /// Phase 47: home-quadrant region id (0..3, cheap sign bits — no atan2
    /// in the mate-filter path). The four land arcs of the annulus are the
    /// model's geographic islands; region membership is a pure function of
    /// the home centre, so it needs no saved per-agent field.
    #[inline]
    fn region_of(x: f64, z: f64) -> u8 {
        let two_pi = std::f64::consts::PI * 2.0;
        let a = aether_libm::atan2(z, x) + std::f64::consts::PI;
        let sector = (a / two_pi * REGION_COUNT as f64).floor() as i32;
        sector.clamp(0, REGION_COUNT as i32 - 1) as u8
    }

    /// Phase 47: same-kind population inside one home region.
    fn region_count(&self, kind: SpeciesKind, region: u8) -> usize {
        self.agents.iter().filter(|a| a.alive && a.kind == kind && a.region == region).count()
    }

    /// Phase 47: split each kind's initial population into equal-sized
    /// angular sectors (balanced demes, so a sector never starts empty while
    /// another hogs the founders). Deterministic: angle, then agent id.
    fn assign_initial_regions(&mut self) {
        for k in 0..self.kinds.len() as u16 {
            let mut idxs: Vec<usize> = self.agents.iter().enumerate()
                .filter(|(_, a)| a.alive && a.kind == k)
                .map(|(i, _)| i)
                .collect();
            let n = idxs.len();
            if n == 0 { continue; }
            idxs.sort_by(|&i, &j| {
                let ai = aether_libm::atan2(self.agents[i].home_z, self.agents[i].home_x);
                let aj = aether_libm::atan2(self.agents[j].home_z, self.agents[j].home_x);
                ai.partial_cmp(&aj).unwrap_or(std::cmp::Ordering::Equal)
                    .then(self.agents[i].id.cmp(&self.agents[j].id))
            });
            for (rank, &i) in idxs.iter().enumerate() {
                self.agents[i].region = ((rank * REGION_COUNT as usize) / n) as u8;
            }
            if k == DECO {
                // Phase 47: the focal splitter starts as four small founder
                // clusters, one per region. The founder pool is sorted by
                // deco_efficiency and split into four quantiles, so the four
                // refugia receive different slices of the standing variation
                // (allopatric founder pools). A globally scattered founder
                // set leaves each region's 3-4 individuals hundreds of units
                // apart; they never meet and three of four demes die before
                // breeding. Clustering is the island model's initial
                // condition (each island receives propagules), not a
                // per-generation script.
                for r in 0..REGION_COUNT {
                    let members: Vec<usize> = idxs.iter().copied()
                        .filter(|&i| self.agents[i].region == r)
                        .collect();
                    if members.is_empty() { continue; }
                    let Some((cx, cz)) = self.region_land_center(r) else { continue; };
                    let m = members.len();
                    for (j, &i) in members.iter().enumerate() {
                        let ang = (j as f64) * std::f64::consts::PI * 2.0 / m as f64;
                        let px = cx + aether_libm::cos(ang) * 18.0;
                        let pz = cz + aether_libm::sin(ang) * 18.0;
                        let ag = &mut self.agents[i];
                        ag.x = px; ag.z = pz; ag.prev_x = px; ag.prev_z = pz;
                        ag.home_x = px; ag.home_z = pz;
                        ag.target_x = px; ag.target_z = pz;
                    }
                }
            }
        }
    }

    /// Phase 47: a land centre for one angular region (region bits:
    /// 0=x<0,z<0; 1=x>=0,z<0; 2=x<0,z>=0; 3=x>=0,z>=0).
    fn region_land_center(&self, region: u8) -> Option<(f64, f64)> {
        let ang = -std::f64::consts::PI
            + (region as f64 + 0.5) * std::f64::consts::PI * 2.0 / REGION_COUNT as f64;
        for r in [320.0, 280.0, 370.0, 240.0, 420.0, 200.0, 460.0] {
            let x = aether_libm::cos(ang) * r;
            let z = aether_libm::sin(ang) * r;
            if Self::region_of(x, z) != region { continue; }
            let h = terrain_height(x, z);
            if h >= WATER_LEVEL + 2.0 && h <= 82.0 && slope_at(x, z) <= 0.75 {
                return Some((x, z));
            }
        }
        None
    }

    /// Phase 47: a legal founding site inside an empty region (family
    /// colonisation). Draws angle + radius + two jitter values on the
    /// dedicated stream; habitat and diet-resource checks match the
    /// within-region join path.
    fn founding_site_in_region(&mut self, kind: SpeciesKind, region: u8) -> Option<(f64, f64)> {
        // Resource-tracking habitat selection: colonists follow a food cue
        // (carcass / grass cell / prey individual) inside the target region.
        // Uniform random points in a 200k-unit sector almost never contain
        // food, which is why the first colonisation attempts starved.
        let def = self.kind_def(kind);
        let mut cues: Vec<(f64, f64)> = Vec::new();
        if def.diet.eats_carcass {
            for c in self.carcasses.iter() {
                if Self::region_of(c.x, c.z) == region { cues.push((c.x, c.z)); }
            }
        }
        if def.diet.eats_plant {
            for i in (0..self.grass.len()).step_by(3) {
                if self.grass[i] <= 0.10 { continue; }
                let col = (i % GRID_N as usize) as f64;
                let row = (i / GRID_N as usize) as f64;
                let x = -GRID_SPAN / 2.0 + (col + 0.5) * CELL;
                let z = -GRID_SPAN / 2.0 + (row + 0.5) * CELL;
                if Self::region_of(x, z) == region { cues.push((x, z)); }
            }
        }
        for &pk in &def.diet.prey_kinds {
            for a in self.agents.iter() {
                if a.alive && a.kind == pk && a.region == region {
                    cues.push((a.x, a.z));
                }
            }
        }
        if !cues.is_empty() {
            let k = (self.dispersal_rng.next_f64() * cues.len() as f64) as usize;
            let (x, z) = cues[k.min(cues.len() - 1)];
            let jx = (self.dispersal_rng.next_f64() - 0.5) * 16.0;
            let jz = (self.dispersal_rng.next_f64() - 0.5) * 16.0;
            let px = x + jx;
            let pz = z + jz;
            if Self::region_of(px, pz) == region {
                return Some((px, pz));
            }
            return Some((x, z));
        }
        let centre = -std::f64::consts::PI
            + (region as f64 + 0.5) * std::f64::consts::PI * 2.0 / REGION_COUNT as f64;
        for _ in 0..DISPERSAL_TRIES {
            let ang = centre + (self.dispersal_rng.next_f64() - 0.5)
                * (std::f64::consts::PI * 2.0 / REGION_COUNT as f64);
            let r = 200.0 + self.dispersal_rng.next_f64() * 260.0;
            let x = aether_libm::cos(ang) * r;
            let z = aether_libm::sin(ang) * r;
            if Self::region_of(x, z) != region { continue; }
            let h = terrain_height(x, z);
            let habitat_ok = if kind == FISH {
                h >= WATER_LEVEL + 1.0 && h < WATER_LEVEL + 1.5 && slope_at(x, z) <= 0.8
            } else {
                h >= WATER_LEVEL + 1.5 && h <= 85.0 && slope_at(x, z) <= 0.8
            };
            if habitat_ok && self.landing_has_food(kind, x, z) {
                return Some((x, z));
            }
        }
        None
    }

    /// Phase 47: resolve a juvenile's founding site. `natal` is the legacy
    /// parents' midpoint + jitter. With probability `dispersal_p` the juvenile
    /// instead draws a direction and a distance in
    /// [DISPERSAL_MIN_DIST, DISPERSAL_MAX_DIST] from the natal site and is
    /// placed on the first habitat-legal landing cell that is also at least
    /// DISPERSAL_CLEARANCE away from every live conspecific (land kinds);
    /// after DISPERSAL_TRIES misses it stays natal. The dedicated stream is
    /// touched only when `dispersal_p > 0` and the trigger fires, so the p=0
    /// arm is a bit-exact pre-Phase-47 trajectory. Returns (x, z, dispersed).
    fn birth_site(&mut self, kind: SpeciesKind, natal_x: f64, natal_z: f64, natal_region: u8) -> (f64, f64, Option<u8>) {
        if self.dispersal_p <= 0.0 { return (natal_x, natal_z, None); }
        if self.dispersal_rng.next_f64() >= self.dispersal_p { return (natal_x, natal_z, None); }
        if self.region_isolation {
            if let Some((x, z, r)) = self.region_migrant_site(kind, natal_region) {
                return (x, z, Some(r));
            }
            return (natal_x, natal_z, None);
        }
        let max_r = SIM_RADIUS - 30.0;
        let clearance2 = DISPERSAL_CLEARANCE * DISPERSAL_CLEARANCE;
        for _ in 0..DISPERSAL_TRIES {
            let ang = self.dispersal_rng.next_f64() * std::f64::consts::PI * 2.0;
            let dist = DISPERSAL_MIN_DIST
                + self.dispersal_rng.next_f64() * (DISPERSAL_MAX_DIST - DISPERSAL_MIN_DIST);
            let x = natal_x + aether_libm::cos(ang) * dist;
            let z = natal_z + aether_libm::sin(ang) * dist;
            if x * x + z * z > max_r * max_r { continue; }
            let h = terrain_height(x, z);
            let habitat_ok = if kind == FISH {
                // Same shoreline band the initial FISH batch uses.
                h >= WATER_LEVEL + 1.0 && h < WATER_LEVEL + 1.5 && slope_at(x, z) <= 0.8
            } else {
                h >= WATER_LEVEL + 1.5 && h <= 85.0 && slope_at(x, z) <= 0.8
            };
            if !habitat_ok { continue; }
            if self.region_isolation && Self::region_of(x, z) == Self::region_of(natal_x, natal_z) {
                continue;
            }
            // Land demes must start outside the source cloud; FISH are pinned
            // to the narrow shoreline band and keep the plain habitat check.
            if kind != FISH {
                let mut clear = true;
                for a in self.agents.iter() {
                    if !a.alive || a.kind != kind { continue; }
                    let dx = a.x - x;
                    let dz = a.z - z;
                    if dx * dx + dz * dz <= clearance2 { clear = false; break; }
                }
                if !clear || !self.landing_has_food(kind, x, z) { continue; }
            }
            return (x, z, Some(Self::region_of(x, z)));
        }
        (natal_x, natal_z, None)
    }

    /// Phase 47: region-isolation dispersal. The migrant juvenile joins a
    /// random conspecific in a different home quadrant (conspecific
    /// attraction), which keeps it inside a resident deme instead of dropping
    /// it into an empty matrix where it would die before breeding. Draws:
    /// one region pick + one member pick on the dedicated stream.
    fn region_migrant_site(&mut self, kind: SpeciesKind, here: u8) -> Option<(f64, f64, u8)> {
        // Priority-effect colonisation: migrants only enter a region that has
        // no conspecific yet (a vacant island). Once a region is occupied it
        // receives no further immigration, so its deme is not continuously
        // homogenised by the source — the isolation that drift needs. This is
        // the island model's "empty niche" rule, not a per-species script.
        let mut empty: Vec<u8> = Vec::new();
        for r in 0..REGION_COUNT {
            if r != here && self.region_count(kind, r) == 0 { empty.push(r); }
        }
        if empty.is_empty() { return None; }
        let pick = (self.dispersal_rng.next_f64() * empty.len() as f64) as usize;
        let target = empty[pick.min(empty.len() - 1)];
        self.founding_site_in_region(kind, target).map(|(x, z)| (x, z, target))
    }

    /// Phase 47: habitat-selection check for a distant landing site. A
    /// founder family dropped into a resource desert starves before its
    /// first local birth (observed in the first family-dispersal probe: the
    /// remote daughter clade died within 1.6k ticks). Dispersers are therefore
    /// assumed to settle near a resource their diet can actually use — the
    /// standard habitat-selection assumption, not a per-kind script:
    /// plant-eaters need grass, carcass-eaters a carcass, predators prey.
    fn landing_has_food(&self, kind: SpeciesKind, x: f64, z: f64) -> bool {
        let def = self.kind_def(kind);
        let r2 = 120.0 * 120.0;
        if def.diet.eats_carcass {
            for c in self.carcasses.iter() {
                let dx = c.x - x;
                let dz = c.z - z;
                if dx * dx + dz * dz <= r2 { return true; }
            }
        }
        if def.diet.eats_plant {
            let ci = self.cell_index(x, z);
            if ci >= 0 {
                let i = ci as usize;
                if i < self.grass.len() && self.grass[i] > 0.05 { return true; }
            }
        }
        for &pk in &def.diet.prey_kinds {
            for a in self.agents.iter() {
                if !a.alive || a.kind != pk { continue; }
                let dx = a.x - x;
                let dz = a.z - z;
                if dx * dx + dz * dz <= r2 { return true; }
            }
        }
        false
    }

    /// Phase 47: a successful dispersal is a family founding event, not a
    /// lone juvenile drop. Both parents move with the child to the new home
    /// range (breeding dispersal), so the founder deme starts with a breeding
    /// pair and does not die in the Allee trap before its first local birth.
    /// Pure state assignment — zero RNG draws, zero energy cost.
    fn relocate_founders(&mut self, a: usize, b: Option<usize>, x: f64, z: f64) {
        let mut place = |ag: &mut Agent, px: f64, pz: f64| {
            ag.x = px; ag.z = pz; ag.prev_x = px; ag.prev_z = pz;
            ag.home_x = px; ag.home_z = pz; ag.target_x = px; ag.target_z = pz;
        };
        match b {
            Some(b) => {
                place(&mut self.agents[a], x - 1.5, z - 1.0);
                place(&mut self.agents[b], x + 1.5, z + 1.0);
            }
            None => place(&mut self.agents[a], x, z),
        }
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
        let a_home_x = self.agents[idx].home_x;
        let a_home_z = self.agents[idx].home_z;
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
        // Phase 40.1: never let the density brake reach the absorbing
        // zero-birth state (see REPRO_THRESHOLD_MAX). Applied after all
        // per-kind overrides except DECO's individualized selection mapping
        // (that one is a genuine fitness gradient, not a crowding brake).
        if a_kind != DECO { threshold = threshold.min(REPRO_THRESHOLD_MAX); }
        // FISH does not reproduce in winter
        if a_kind == FISH && self.season == 3 { return; }
        // Phase 7: DECO individualized reproduction threshold in selective mode.
        // eff=1.5 → 0.48 (easy breed), eff=1.0 → 0.96 (neutral, legacy), and
        // Phase 59 saturates everything below neutral into (0.96, 0.99) so no
        // legal efficiency is sterile by construction (see the fn docs).
        // The density brake above is replaced for DECO by this mapping: since eff=1.0 yields
        // exactly the legacy 0.96, the density factor is reapplied multiplicatively here.
        if a_kind == DECO && self.deco_eff_choice_mode == save_state::DECO_CHOICE_SELECTIVE {
            threshold = self.deco_repro_threshold(self.agents[idx].genome.deco_efficiency)
                * (1.0 + density_factor.max(0.0)).min(1.5);
        }
        if self.agents[idx].energy < self.agents[idx].max_energy * threshold { return; }
        // Find mate via nearest. FISH pair over a wider radius: they are
        // spread along the narrow shoreline band, so nearest-neighbor distance
        // is much larger than for land species.
        let mate_id = self.agents[idx].id; let mate_max_e = self.agents[idx].max_energy; let mate_kind = self.agents[idx].kind;
        // Phase 42: Allee 缓解——择偶搜索半径随稀疏程度扩大（密度越低，
        // 找到配偶的搜寻成本越高）。满员（density_factor == 1）时半径恰为
        // legacy 11.0，密集世界的动力学逐位不变；稀疏世界则不会因为
        // 「找不到 11 单位内的配偶」而落入零出生吸收态（42 诊断实测：
        // 满员时也只有 27% 的待育者在 11 单位内有配偶，稀疏后直接归零）。
        let a_region = self.agents[idx].region;
        let legacy_mate_radius = if mate_kind == FISH {
            60.0
        } else {
            11.0 * (1.0 + (1.0 - density_factor.clamp(0.0, 1.0)) * 3.0)
        };
        // Phase 66 R1/R2: a derived clade's search reaches across its regional
        // deme when the deme is too sparse for the legacy bound. The region
        // gate in the filter below is unchanged, so cross-deme gene flow stays
        // impossible by construction.
        let (mate_radius, radius_boosted) =
            self.mate_radius_with_floor(mate_kind, a_region, legacy_mate_radius);
        if radius_boosted { self.radius_boost_searches += 1; }
        let mate_min_e_const = if mate_kind == FISH { 0.55 } else { 0.68 };
        let selected_mode = self.deco_eff_choice_mode;
        let saturate = self.deco_repro_saturate;
        let region_isolation = self.region_isolation;
        let mate_filter = Some(move |b: &Agent| {
            if b.id != mate_id && b.age > 30.0 && b.repro <= 0.0
                && (!region_isolation || b.region == a_region) {
                // Phase 7: DECO in selective mode requires both parties to meet their individualized threshold.
                let min_e = if mate_kind == DECO && selected_mode == save_state::DECO_CHOICE_SELECTIVE && b.kind == DECO {
                    Ecology::deco_repro_threshold_for(b.genome.deco_efficiency, saturate)
                } else {
                    mate_min_e_const
                };
                b.energy > b.max_energy * min_e
            } else {
                false
            }
        });
            let mate_opt = self.nearest(idx, mate_kind, mate_radius, mate_filter);
        if mate_opt.is_none() {
            // Phase 61 telemetry: a ready adult (past cooldown, over her energy
            // threshold) found no mate inside the search radius. Phase 66 adds
            // the cause bucket (pure read of the same world state).
            let (cid, k) = (self.agents[idx].clade_id, self.agents[idx].kind);
            let (bucket, kind_pop, region_pop, ready_dist) =
                self.mate_miss_bucket(idx, mate_radius, mate_min_e_const);
            self.clade_life_mate_miss_detail(cid, k, bucket, kind_pop, region_pop, ready_dist);
        }
        if let Some((mate_j, mate_dist)) = mate_opt {
            if mate_dist > legacy_mate_radius {
                self.radius_only_finds += 1;
                self.radius_only_find_dist_sum += mate_dist;
            }
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
            let mut_scale = crate::group::mutation_scale(&self.agents[idx]);
            let g = self.mix_genome(&a_geno, &b_geno, a_kind, mut_scale);
            let (jx, jz) = if a_kind == FISH {
                ((self.fish_rng.next_f64() - 0.5) * 2.5, (self.fish_rng.next_f64() - 0.5) * 2.5)
            } else {
                ((self.rng.next_f64() - 0.5) * 2.5, (self.rng.next_f64() - 0.5) * 2.5)
            };
            // Phase 47 philopatry: the natal site is the mother's HOME plus
            // jitter, not the midpoint of the parents' current positions.
            // Current-position midpoints random-walk the home centre every
            // generation (both parents may be anywhere in their 150-unit
            // range when they meet), which is why Phase 46 had no stable
            // geographic structure for drift to act on.
            let natal_x = a_home_x + jx;
            let natal_z = a_home_z + jz;
            let (child_x, child_z, dispersed) = self.birth_site(a_kind, natal_x, natal_z, a_region);
            let max_gen_child = a_gen.max(b_gen) + 1;
            let parent_clade = self.agents[idx].clade_id;
            // Phase 34 kind-fork: births INTO a forked clade carry the clade's
            // derived kind; root-clade births keep the parent's kind.
            let child_kind = self.clade_kind(parent_clade, a_kind);
            // Phase 36: capture parent brain (immutable) before spawning so we
            // can mutate it for the child without conflicting with the
            // &mut Agent alias returned by spawn.
            let parent_brain = self.agents[idx].brain.clone();
            // Phase 42: 亲代行为权威类型（出生记账用，提前取以免 spawn 后借用冲突）。
            let parent_authority = self.agents[idx].brain_authority;
            // Phase 36: mutate the child's brain BEFORE spawning so we never
            // hold `&mut Agent` (from spawn) and `&mut self.brain_learning_rng`
            // simultaneously — they are both reachable through `self`.
            let mut child_brain = parent_brain;
            child_brain.mutate_structure(&mut self.brain_learning_rng, BRAIN_MUTATE_P_ADD, BRAIN_MUTATE_P_DEL);
            child_brain.perturb(&mut self.brain_learning_rng, BRAIN_PERTURB_SIGMA);
            let mut child_triune = self.agents[idx].triune.clone();
            child_triune.mutate(&mut self.brain_learning_rng, TRIUNE_MUTATE_SIGMA);
            child_triune.reset_transients();
            if let Some(child) = self.spawn_tagged(child_kind, child_x, child_z, Some(g), max_gen_child, 0.0, Some((parent_clade, a_kind))) {
                child.lineage = a_lineage.or(b_lineage);
                child.clade_id = parent_clade;
                child.region = dispersed.unwrap_or(a_region);
                child.brain = child_brain;
                child.triune = child_triune;
                // Phase 42: 按亲代的行为权威归类出生（适应性 A/B 的分母）。
                if parent_authority == BRAIN_AUTHORITY_BRAIN {
                    self.triune_stats.fitness.brain_parent_births += 1;
                } else {
                    self.triune_stats.fitness.fsm_parent_births += 1;
                }
                let (drain_herb, drain_pred) = self.kind_def(self.agents[idx].kind).repro_drain_pair;
                self.ledger.repro_loss += drain_herb + drain_pred;
                // Phase 35 MTE_ON: body-mass reproduction fee (size^3).
                #[cfg(any(test, feature = "phase35_mte"))]
                if MTE_ON {
                    let m = self.agents[idx].genome.size * self.agents[mate_j].genome.size;
                    self.ledger.repro_loss += MTE_REPRO_FEE * (m / 2.0).powi(3);
                }
                self.agents[idx].energy -= drain_herb;
                self.agents[mate_j].energy -= drain_pred;
                // Phase 43: 代际文化传承（cultural ratchet）。双亲各自教学：
                // 每位教师按自己的 known_tokens 顺序传承，能量付不起下一位即停
                // （贫穷约束，零 RNG）；保真度 roll 走专用 culture_rng，
                // 失败的 token 不传承。教学在出生扣费之后结算 —— 付不起
                // 繁殖代价的亲代自然也付不起教学。
                if self.culture_on && self.teach_on {
                    let parent_tokens = self.agents[idx].known_tokens.clone();
                    let mate_tokens = self.agents[mate_j].known_tokens.clone();
                    let mut parent_energy = self.agents[idx].energy;
                    let mut mate_energy = self.agents[mate_j].energy;
                    let child_idx = self.agents.len() - 1;
                    let mut child_tokens = std::mem::take(&mut self.agents[child_idx].known_tokens);
                    crate::culture::inherit_at_birth(
                        &mut child_tokens, &parent_tokens, &mut parent_energy,
                        &self.token_registry, self.culture_value_on,
                        &mut self.culture_rng, &mut self.culture_stats,
                    );
                    crate::culture::inherit_at_birth(
                        &mut child_tokens, &mate_tokens, &mut mate_energy,
                        &self.token_registry, self.culture_value_on,
                        &mut self.culture_rng, &mut self.culture_stats,
                    );
                    self.agents[child_idx].known_tokens = child_tokens;
                    self.agents[idx].energy = parent_energy;
                    self.agents[mate_j].energy = mate_energy;
                }
                if HELP_ENABLED && self.root_kind(self.agents[idx].kind) == HERB {
                    self.drain_e_h += 68.0;
                }
                self.agents[idx].repro = cooldown;
                self.agents[mate_j].repro = cooldown;
                self.agents[idx].offspring = self.agents[idx].offspring + 1;
                self.agents[mate_j].offspring = self.agents[mate_j].offspring + 1;
                self.births += 1;
                if dispersed.is_some() {
                    self.relocate_founders(idx, Some(mate_j), child_x, child_z);
                    self.dispersal_events += 1;
                    self.dispersal_last_tick = self.tick_count;
                    self.clade_life_dispersal(parent_clade, child_kind);
                }
                return;
            }
        }
        // Asexual reproduction
        // (Phase 40.1 note: a low-density rescue variant — dropping this bar
        // to 0.90 while live_count < 50 — was tried and reverted: it shifts
        // early population assembly and destabilizes dense seeds. The
        // Allee effect at sparse densities is recorded as a real ecological
        // feature, not a bug.)
        if self.agents[idx].age > 45.0 && self.agents[idx].energy > self.agents[idx].max_energy * 0.94 {
            let a_geno_a2 = self.agents[idx].genome.clone();
            let a_kind_a2 = self.agents[idx].kind;
            let a_gen_a2 = self.agents[idx].generation;
            let a_lineage_a2 = self.agents[idx].lineage;
            let mut_scale = crate::group::mutation_scale(&self.agents[idx]);
            let g = self.mutate(&a_geno_a2, a_kind_a2, mut_scale);
            let ang = if a_kind_a2 == FISH { self.fish_rng.next_f64() } else { self.rng.next_f64() } * std::f64::consts::PI * 2.0;
            let natal_x = self.agents[idx].home_x + aether_libm::cos(ang) * 3.0;
            let natal_z = self.agents[idx].home_z + aether_libm::sin(ang) * 3.0;
            let a_region_a2 = self.agents[idx].region;
            let (cx, cz, dispersed) = self.birth_site(a_kind_a2, natal_x, natal_z, a_region_a2);
            let parent_clade = self.agents[idx].clade_id;
            // Phase 34 kind-fork (asexual path): same clade-derived kind rule.
            let child_kind_a2 = self.clade_kind(parent_clade, a_kind_a2);
            let parent_brain_a2 = self.agents[idx].brain.clone();
            let mut child_brain_a2 = parent_brain_a2;
            child_brain_a2.mutate_structure(&mut self.brain_learning_rng, BRAIN_MUTATE_P_ADD, BRAIN_MUTATE_P_DEL);
            child_brain_a2.perturb(&mut self.brain_learning_rng, BRAIN_PERTURB_SIGMA);
            let mut child_triune_a2 = self.agents[idx].triune.clone();
            child_triune_a2.mutate(&mut self.brain_learning_rng, TRIUNE_MUTATE_SIGMA);
            child_triune_a2.reset_transients();
            if let Some(child) = self.spawn_tagged(child_kind_a2, cx, cz, Some(g), a_gen_a2 + 1, 0.0, Some((parent_clade, a_kind_a2))) {
                child.lineage = a_lineage_a2;
                child.clade_id = parent_clade;
                child.region = dispersed.unwrap_or(a_region_a2);
                // Phase 36: inherit parent brain via cloning + structural mutation.
                child.brain = child_brain_a2;
                child.triune = child_triune_a2;
                let drain = self.kind_def(self.agents[idx].kind).repro_drain_solo;
                self.ledger.repro_loss += drain;
                #[cfg(any(test, feature = "phase35_mte"))]
                if MTE_ON {
                    let m = self.agents[idx].genome.size;
                    self.ledger.repro_loss += MTE_REPRO_FEE * m.powi(3);
                }
                self.agents[idx].energy -= drain;
                // Phase 43: 代际文化传承（无性路径：单亲教学，规则同有性路径）。
                if self.culture_on && self.teach_on {
                    let parent_tokens = self.agents[idx].known_tokens.clone();
                    let mut parent_energy = self.agents[idx].energy;
                    let child_idx = self.agents.len() - 1;
                    let mut child_tokens = std::mem::take(&mut self.agents[child_idx].known_tokens);
                    crate::culture::inherit_at_birth(
                        &mut child_tokens, &parent_tokens, &mut parent_energy,
                        &self.token_registry, self.culture_value_on,
                        &mut self.culture_rng, &mut self.culture_stats,
                    );
                    self.agents[child_idx].known_tokens = child_tokens;
                    self.agents[idx].energy = parent_energy;
                }
                if HELP_ENABLED && self.root_kind(self.agents[idx].kind) == HERB {
                    self.drain_e_h += 52.0;
                }
                self.agents[idx].repro = cooldown * 1.4;
                self.agents[idx].offspring = self.agents[idx].offspring + 1;
                self.births += 1;
                if dispersed.is_some() {
                    self.relocate_founders(idx, None, cx, cz);
                    self.dispersal_events += 1;
                    self.dispersal_last_tick = self.tick_count;
                    self.clade_life_dispersal(parent_clade, child_kind_a2);
                }
            }
        }
    }

    pub(crate) fn die(&mut self, idx: usize, cause: &'static str) {
        if !self.agents[idx].alive { return; }
        // Phase 61 telemetry runs before the agent stops being counted, so the
        // counter still sees its clade, age and end-of-life energy.
        self.clade_life_death(idx, cause);
        self.agents[idx].alive = false;
        self.deaths += 1;
        if cause == "starved" { self.starved += 1; }
        else if cause == "aged" { self.aged += 1; }
        let offspring = self.agents[idx].offspring;
        // Phase 42: 按死者的行为权威类型记寿年与子代（接管 vs 未接管的适应性 A/B）。
        if self.agents[idx].brain_authority == BRAIN_AUTHORITY_BRAIN {
            self.triune_stats.fitness.brain_deaths += 1;
            self.triune_stats.fitness.brain_lifespan_sum += self.agents[idx].age;
            self.triune_stats.fitness.brain_offspring_sum += offspring as u64;
        } else {
            self.triune_stats.fitness.fsm_deaths += 1;
            self.triune_stats.fitness.fsm_lifespan_sum += self.agents[idx].age;
            self.triune_stats.fitness.fsm_offspring_sum += offspring as u64;
        }
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
        // Phase 35 ledger: dead agent biomass → carcass pool.
        self.ledger.offspring_invest += energy; // recorded here too since death creates a carcass with this energy
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
        let lineage_drain = self.herb_lineage_drain(lineage_a);
        let lineage_bite = self.herb_lineage_bite(lineage_a);

        self.agents[idx].fatigue = (self.agents[idx].fatigue + self.agents[idx].activity * 0.06 * h
            - if self.agents[idx].activity < 0.15 { 0.22 * h } else { 0.0 }).clamp(0.0, 1.0);
        self.agents[idx].curiosity = (self.agents[idx].curiosity + 0.02 * h).clamp(0.0, 1.0);

        // Phase 35 (INFO_COST_ON): charge for threat detection scan.
        self.charge_info_cost(idx, 1);
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
            self.ledger.metabolism_loss += drain_flee;
        } else if self.agents[idx].think <= 0.0 {
            self.agents[idx].think = 0.35 + self.rng.next_f64() * 0.4;
            let social_high = self.agents[idx].genome.social > 0.8;
            if social_high {
                let my_id = self.agents[idx].id;
                let mate_opt = self.nearest(idx, HERB, 15.0, Some(|b: &Agent| b.id != my_id));
                if let Some((mate_j, _)) = mate_opt {
                    // Phase 35 (INFO_COST_ON): social mate detection costs one read.
                    self.charge_info_cost(idx, 1);
                    self.agents[idx].target_x = self.agents[mate_j].x;
                    self.agents[idx].target_z = self.agents[mate_j].z;
                    self.agents[idx].state = "graze";
                    let dm = js_hypot(self.agents[mate_j].x - self.agents[idx].x, self.agents[mate_j].z - self.agents[idx].z);
                    desired = if dm > 3.0 { g_speed * 0.6 } else { 0.0 };
                } else if self.agents[idx].energy < self.agents[idx].max_energy * 0.84 {
                    // Phase 35 (INFO_COST_ON): grazing probe costs 1 read.
                    self.charge_info_cost(idx, 1);
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
                // Phase 35 (INFO_COST_ON): grazing probe costs 1 read.
                self.charge_info_cost(idx, 1);
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

        self.brain_bridge(idx, h);
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
        // Phase 35 (MTE_ON): Kleiber body-mass metabolic scaling.
        let total_drain = total_drain * self.mte_size_scale(idx);
        if HELP_ENABLED { self.drain_e_h += total_drain; }
        self.ledger.metabolism_loss += total_drain;
        self.agents[idx].energy -= total_drain;

        let ci_val = self.cell_index(self.agents[idx].x, self.agents[idx].z);
        if herb_eats_plant
            && ci_val >= 0 && (ci_val as usize) < self.grass.len() && (self.grass[ci_val as usize] as f64) > 0.1
            && self.agents[idx].bite <= 0.0 && self.agents[idx].speed < g_speed * 0.45 {
            let helped = self.try_help(idx, ci_val as usize);
            if !helped {
                let take = self.take_grass(idx, ci_val as usize, 0.2 * lineage_bite);
                        self.leaf_e_h += take * 62.0;
                self.ledger.grazed += take * 62.0;
                // Phase 51: niche resource-partition preference multiplier.
                let pref = self.niche_pref_mult(self.agents[idx].clade_id, ci_val as usize);
                self.agents[idx].energy = (self.agents[idx].energy + take * 62.0 * pref).min(self.agents[idx].max_energy);
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

        self.brain_shadow(idx, h);
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
        // Phase 33/34: diet gates — PRED hunts live prey (prey_kinds, the
        // legacy PRED scans [HERB, FISH] in that order) and scavenges
        // carcasses. Both read from the registry; the prey scan is
        // data-driven since Phase 34 (bit-equivalent under the default diet).
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
                // Phase 34: data-driven prey scan — the prey KIND LIST comes
                // from the registry (`KindDef.diet.prey_kinds`; legacy PRED =
                // [HERB, FISH], the same two `nearest` calls in the same
                // order, so the scan is bit-equivalent under the default
                // diet). Reachability stays a physical constraint: aquatic
                // prey (the `Fish` behavior template) are reachable only from
                // the water's edge. Selection: a reachable FISH candidate is
                // preferred (the legacy near-shore fish-specialist rule);
                // otherwise the first reachable candidate in scan order.
                let fish_reachable = my_ground < WATER_LEVEL + 3.0;
                let mut candidates: Vec<(usize, f64, SpeciesKind)> = Vec::new();
                for &pk in &self.diet_of(p_kind).prey_kinds {
                    if let Some(cand) = self.nearest(idx, pk, g_sense, None::<fn(&Agent) -> bool>) {
                        let aquatic = self.kind_def(pk).behavior == BehaviorTemplate::Fish;
                        if !aquatic || fish_reachable {
                            candidates.push((cand.0, cand.1, pk));
                        }
                    }
                }
                if let Some(fish) = candidates.iter().find(|c| c.2 == FISH) {
                    prey_opt = Some((fish.0, fish.1));
                } else if let Some(first) = candidates.first() {
                    prey_opt = Some((first.0, first.1));
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
        self.brain_bridge(idx, h);
        self.agents[idx].speed = self.agents[idx].speed + (self.agents[idx].desired_speed - self.agents[idx].speed) * (h * 5.0).min(1.0);
        let activity = self.agents[idx].speed / g_speed.max(1.0);
        self.agents[idx].activity = activity;
        let drain = g_metabolism * (0.46 + 0.78 * activity * activity + 0.14 * (g_speed / 10.5).powi(2));
        let thirst_p = if self.agents[idx].hydration < 0.3 { 1.5 } else { 1.0 };
        // Phase 35 (MTE_ON): Kleiner body-mass metabolic scaling on pred drain.
        let mte = self.mte_size_scale(idx);
        self.ledger.metabolism_loss += drain * thirst_p * self.season_meta * h * mte;
        self.agents[idx].energy -= drain * thirst_p * self.season_meta * h * mte;

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
                    // Phase 36 (TROPHIC_EFF LIVE): gate the live-prey → consumer
                    // biomass transfer with Lindeman 1942 efficiency (0.10).
                    // `trophic_eff_active == false` reproduces the Phase 35
                    // placeholder (the pre-Phase-36 bit-exact anchor) so A/B
                    // tests can isolate the re-baseline source.
                    let pre_eff_gain = if self.trophic_eff_active {
                        gain * 0.8 * TROPHIC_EFF
                    } else {
                        gain * 0.8
                    };
                    self.ledger.predation_gain += pre_eff_gain;
                    // Phase 51: niche resource-partition preference multiplier
                    // at the predator's foraging cell.
                    let pref = self.niche_pref_at(self.agents[idx].clade_id, self.agents[idx].x, self.agents[idx].z);
                    self.agents[idx].energy = (self.agents[idx].energy + pre_eff_gain * pref).min(self.agents[idx].max_energy);
                    self.agents[idx].hydration = (self.agents[idx].hydration + 0.4).min(1.0);
                    let bite_interval = PRED_BITE_INTERVAL + self.rng.next_f64() * 5.0;
                    self.agents[idx].bite = bite_interval;
                    self.kills += 1;
                    self.deaths += 1;
                    self.clade_life_death(pj, "predation");
                    self.agents[pj].alive = false;
                    // Carcass keeps the original 0.22 portion (bit-exact).
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
                    self.ledger.scavenged += c_energy;
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

        self.brain_shadow(idx, h);
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
        self.brain_bridge(idx, h);
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
                self.ledger.scavenged += take * 7.0;
                // Phase 51: niche resource-partition preference multiplier.
                let pref = self.niche_pref_at(self.agents[idx].clade_id, self.agents[idx].x, self.agents[idx].z);
                self.agents[idx].energy = (self.agents[idx].energy + take * 7.0 * pref).min(self.agents[idx].max_energy);
                self.agents[idx].bite = 1.5;
                if self.carcasses[ci].energy <= 0.2 { self.carcasses.remove(ci); }
            } else if self.agents[idx].energy < self.agents[idx].max_energy * 0.7 && self.agents[idx].bite <= 0.0 {
                let gi = self.cell_index(self.agents[idx].x, self.agents[idx].z);
                if gi >= 0 && (gi as usize) < self.grass.len() && (self.grass[gi as usize] as f64) > 0.1 {
                    let take = self.take_grass(idx, gi as usize, 0.15);
                    // Phase 51: niche resource-partition preference multiplier.
                    let pref = self.niche_pref_mult(self.agents[idx].clade_id, gi as usize);
                    self.ledger.grazed += take * 55.0;
                    self.agents[idx].energy = (self.agents[idx].energy + take * 55.0 * pref).min(self.agents[idx].max_energy);
                    self.agents[idx].bite = 2.0;
                }
            }
        } else if self.agents[idx].energy < self.agents[idx].max_energy * 0.7 && self.agents[idx].bite <= 0.0 {
            let gi = self.cell_index(self.agents[idx].x, self.agents[idx].z);
            if gi >= 0 && (gi as usize) < self.grass.len() && (self.grass[gi as usize] as f64) > 0.1 {
                let take = self.take_grass(idx, gi as usize, 0.15);
                self.ledger.grazed += take * 55.0;
                self.agents[idx].energy = (self.agents[idx].energy + take * 55.0).min(self.agents[idx].max_energy);
                self.agents[idx].bite = 2.0;
            }
        }
        self.ledger.metabolism_loss += g_metabolism * (0.18 + self.agents[idx].activity * 0.3) * self.season_meta * h * self.mte_size_scale(idx);
        self.agents[idx].energy -= g_metabolism * (0.18 + self.agents[idx].activity * 0.3) * self.season_meta * h * self.mte_size_scale(idx);
        self.brain_shadow(idx, h);
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
                // Phase 35 (INFO_COST_ON): grazing probe costs 1 read.
                self.charge_info_cost(idx, 1);
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
        self.brain_bridge(idx, h);
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
                // Phase 51: niche resource-partition preference multiplier.
                let pref = self.niche_pref_at(self.agents[idx].clade_id, self.agents[idx].x, self.agents[idx].z);
                self.agents[idx].energy = (self.agents[idx].energy + take * 8.0 * pref).min(self.agents[idx].max_energy);
                self.agents[idx].bite = 1.8;
                if self.carcasses[ci].energy <= 0.2 { self.carcasses.remove(ci); }
            }
        }
        if self.agents[idx].bite <= 0.0 && self.agents[idx].energy < self.agents[idx].max_energy * 0.9 {
            let gi = self.cell_index(self.agents[idx].x, self.agents[idx].z);
            if gi >= 0 && (gi as usize) < self.grass.len() && (self.grass[gi as usize] as f64) > 0.1 {
                let take = self.take_grass(idx, gi as usize, 0.18);
                // Phase 51: niche resource-partition preference multiplier.
                let pref = self.niche_pref_mult(self.agents[idx].clade_id, gi as usize);
                self.agents[idx].energy = (self.agents[idx].energy + take * 58.0 * pref).min(self.agents[idx].max_energy);
                self.agents[idx].bite = 2.2;
            }
        }
        self.ledger.metabolism_loss += g_metabolism * (0.2 + self.agents[idx].activity * 0.32) * self.season_meta * h * self.mte_size_scale(idx);
        self.agents[idx].energy -= g_metabolism * (0.2 + self.agents[idx].activity * 0.32) * self.season_meta * h * self.mte_size_scale(idx);
        self.brain_shadow(idx, h);
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
            self.ledger.evaporation += FISH_DRY_DEATH_RATE * (h / 0.1);
            self.agents[idx].energy -= FISH_DRY_DEATH_RATE * (h / 0.1);
        } else {
            self.ledger.filter_in += FISH_FILTER_GAIN * h;
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
                // Phase 35 (INFO_COST_ON): fish grazing probe costs 1 read.
                self.charge_info_cost(idx, 1);
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

        self.brain_bridge(idx, h);
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
        // Phase 35 (MTE_ON): fish metabolism scaled by size.
        self.ledger.metabolism_loss += total_drain * self.mte_size_scale(idx);
        self.agents[idx].energy -= total_drain * self.mte_size_scale(idx);

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
                // Phase 51: niche resource-partition preference multiplier.
                let pref = self.niche_pref_mult(self.agents[idx].clade_id, gi as usize);
                self.agents[idx].energy = (self.agents[idx].energy + take * 62.0 * 1.5 * pref).min(self.agents[idx].max_energy);
                self.agents[idx].bite = FISH_BITE_INTERVAL;
            }
        }

        self.brain_shadow(idx, h);
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

        self.brain_bridge(idx, h);
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
                // Phase 51: niche resource-partition preference multiplier.
                // For a DECO band member the multiplier keys off its own cell
                // — the soil it mineralizes — so the mineral-preferring side
                // of the axis feeds back into the very deposit it builds.
                let pref = self.niche_pref_at(self.agents[idx].clade_id, self.agents[idx].x, self.agents[idx].z);
                self.agents[idx].energy = (self.agents[idx].energy + take * 62.0 * deco_eff * pref).min(self.agents[idx].max_energy);
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
                    // Phase 35 ledger: carcass mass mineralized into soil.
                    self.ledger.mineralized += take * deco_eff;
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
        // Phase 35 (MTE_ON): DECO metabolism scaled by size.
        self.ledger.metabolism_loss += total_drain * self.mte_size_scale(idx);
        self.agents[idx].energy -= total_drain * self.mte_size_scale(idx);

        self.brain_shadow(idx, h);
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
            // Phase 45: patch disturbance roll (dedicated PATCH stream; zero
            // draw when patch_on is false, so the A/B arm keeps the exact
            // pre-Phase-45 trajectory).
            self.tick_patch();
            // Phase 54: structure regrowth multipliers decay toward 1.0
            // (cheap linear pass, skipped entirely while fully decayed).
            if self.structure_env_on && self.structure_env_decay_active {
                let mut any_off = false;
                for v in self.structure_env.iter_mut() {
                    if (*v - 1.0).abs() > 1e-6 {
                        *v = 1.0 + (*v - 1.0) * STRUCTURE_ENV_DECAY;
                        if (*v - 1.0).abs() <= 1e-4 { *v = 1.0; }
                        else { any_off = true; }
                    }
                }
                self.structure_env_decay_active = any_off;
            }
            // Phase 56: decomposition multipliers relax toward 1.0 (same
            // linear pass, skipped while the field is fully relaxed).
            if self.deco_env_on && self.deco_env_decay_active {
                let mut any_off = false;
                for v in self.deco_env.iter_mut() {
                    if (*v - 1.0).abs() > 1e-6 {
                        *v = 1.0 + (*v - 1.0) * STRUCTURE_DECO_DECAY;
                        if (*v - 1.0).abs() <= 1e-4 { *v = 1.0; }
                        else { any_off = true; }
                    }
                }
                self.deco_env_decay_active = any_off;
            }

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
            // Phase 35 ledger: record primary-production input from sunlight.
            let pre_biomass = self.grass.iter().map(|&g| g as f64).sum::<f64>();
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
                    * fert_mult * self.regrow_noise[i] as f64
                    // Phase 54: culture-built structures raise local regrowth
                    // (structure_env stays exactly 1.0 when the A/B is off).
                    * self.structure_env[i] as f64;
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
            // Phase 35 ledger: primary production (solar → plant biomass).
            let post_biomass = self.grass.iter().map(|&g| g as f64).sum::<f64>();
            self.ledger.light_in += (post_biomass - pre_biomass) * 62.0;

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

            // Phase 37: cultural dynamics — only when CULTURE_ON is true.
            // Zero-RNG discipline: all draws use the dedicated culture_rng stream.
            // Phase 40.2: the gate no longer requires a non-empty registry —
            // the de-novo invention path (try_invent_token) is the only legal
            // source of the first token, so the loop must run from tick zero.
            if self.culture_on {
                // Collect living agent indices first to avoid borrow checker issues
                let alive_indices: Vec<usize> = self.agents.iter().enumerate()
                    .filter(|(_, a)| a.alive)
                    .map(|(i, _)| i)
                    .collect();

                // Phase 40.2: de-novo invention for token-less agents (the
                // first-token pathway). Environment signature is sampled
                // before the mutable borrow of the agent.
                for &idx in &alive_indices {
                    if !self.agents[idx].known_tokens.is_empty() { continue; }
                    let env = self.sample_env_features(&self.agents[idx]);
                    if culture::try_invent_token(
                        &mut self.agents[idx], &mut self.token_registry, &env, &mut self.culture_rng,
                    ).is_some() {
                        self.culture_stats.innovation_count += 1;
                    }
                }
                
                // Try learning and innovation for each living agent with tokens
                for &idx in &alive_indices {
                    let has_tokens = !self.agents[idx].known_tokens.is_empty();
                    if has_tokens {
                        // Capture necessary data before mutable borrow
                        let agent_len = self.agents.len();
                        let n_tokens = self.agents[idx].known_tokens.len();
                        
                        // Try to learn a new token from the nearest token-bearing
                        // neighbour (Phase 44.1: real spatial proximity — culture
                        // spreads through contact, not through an id+1 roll that
                        // pairs agents on opposite sides of the world).
                        if agent_len > 1 && n_tokens > 0 {
                            let demoiser_idx = self.nearest(idx, self.agents[idx].kind, self.agents[idx].genome.sense, Some(|b: &Agent| !b.known_tokens.is_empty()))
                                .map(|(j, _)| j)
                                .unwrap_or_else(|| (self.agents[idx].id as usize + 1) % agent_len);
                            let target_tid = {
                                let dt = &self.agents[demoiser_idx].known_tokens;
                                // Phase 45: value-sorted learning — the learner
                                // targets the demoiser's highest-value token
                                // (ties → earliest in the demoiser's list).
                                if self.culture_value_on && dt.len() > 1 {
                                    let mut best_i = 0usize;
                                    for i in 1..dt.len() {
                                        let vb = self.token_registry.value_of(dt[i]);
                                        let va = self.token_registry.value_of(dt[best_i]);
                                        if vb > va { best_i = i; }
                                    }
                                    Some(dt[best_i])
                                } else {
                                    dt.first().copied()
                                }
                            };
                            if let Some(tid) = target_tid {
                                if !self.agents[idx].known_tokens.contains(&tid) {
                                    // Use split_at_mut to safely access both agents
                                    let (left, right) = self.agents.split_at_mut(idx.max(demoiser_idx));
                                    let (learner, demoiser) = if idx < demoiser_idx {
                                        (&mut left[idx], &right[0])
                                    } else {
                                        (&mut left[demoiser_idx], &right[0])
                                    };
                                    if culture::try_learn_token(learner, tid, demoiser, 
                                        self.culture_fidelity, &mut self.culture_rng) {
                                        self.culture_stats.learn_count += 1;
                                        self.culture_stats.learn_cost_total += culture::LEARNING_COST;
                                    }
                                }
                            }
                        }
                        // Try innovation (recombination) — costs energy, may produce new token
                        if culture::try_innovate_token(&mut self.agents[idx], &mut self.token_registry, &mut self.culture_rng).is_some() {
                            self.culture_stats.innovation_count += 1;
                        }
                    }
                }
                // Place structures based on learned tokens (simplified model)
                for &idx in &alive_indices {
                    if !self.agents[idx].known_tokens.is_empty() {
                        let first_token = self.placement_token(idx);
                        let x = self.agents[idx].x;
                        let z = self.agents[idx].z;
                        if let Some(token) = self.token_registry.get(first_token) {
                            let (grass_mod, _) = culture::token_to_env_mod(token);
                            // Small probability of placing a structure
                            if self.culture_rng.next_f64() < 0.001 {
                                if culture::try_place_structure(&mut self.agents[idx], &mut self.structure_registry, first_token, x, z) {
                                    self.culture_stats.structure_place_count += 1;
                                    // Phase 55: net-zero dipole — a
                                    // placement TRANSFERS regrowth
                                    // potential instead of creating it
                                    // (see `structure_env_dipole`; the
                                    // Phase 54 trophic cascade — value-
                                    // selected positive tokens raising the
                                    // world's total production — cannot recur
                                    // by construction).
                                    if self.structure_env_on {
                                        self.structure_env_dipole(x, z, grass_mod);
                                    }
                                    // Phase 56: the same placement imprints
                                    // the decomposition field — culture's
                                    // hand on the DECO's own food supply.
                                    if self.deco_env_on {
                                        self.deco_env_dipole(x, z, grass_mod);
                                    }
                                    // Apply immediate environmental effect
                                    // Phase 57: sign-neutral arm — the token's
                                    // environment direction decides the
                                    // effect, never the reward.
                                    let gain = if self.token_value_sign_neutral {
                                        grass_mod.abs() * 10.0
                                    } else {
                                        grass_mod * 10.0
                                    };
                                    self.agents[idx].energy += gain; // benefit feedback
                                    // Phase 45: the same feedback credits the
                                    // token's use_gain EMA (emergent value —
                                    // pure bookkeeping, zero extra RNG).
                                    self.token_registry.record_use(first_token, gain);
                                }
                            }
                        }
                    }
                }
                // Phase 45: uncredited knowledge fades (value EMA decay,
                // zero RNG, zero energy).
                self.token_registry.decay_use_gains();
                // Decay structures over time
                culture::decay_structures(&mut self.structure_registry, culture::STRUCTURE_DECAY);
            }

            // Phase 39.1: major-transition layer (group entities, task
            // switching, dirty-work mutagenesis). Zero side effects while
            // `group_on` is false or no group has formed: the scan is read-only
            // and consumes no RNG until a cohesion candidate passes the
            // persistence gate.
            crate::group::group_step(self, h);
            
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
                // Phase 56: the local decomposition multiplier. Exactly 1.0
                // when the A/B is off, so `1.4 * h * 1.0` is bit-identical to
                // the pre-56 constant rate.
                let decay_mult = self.deco_decay_mult_at(self.carcasses[i].x, self.carcasses[i].z);
                self.carcasses[i].energy -= 1.4 * h * decay_mult;
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

            // Phase 43: abiogenesis — life re-originates from hot-spring
            // chemistry when the world approaches death. No-op (zero RNG,
            // zero scan cost beyond one alive-count pass) while the world is
            // healthy; the trigger, cell and ancestor choices draw from the
            // dedicated ABIO stream, the spawn energy books into
            // `ledger.chemo_in`.
            self.try_abiogenesis();

            // Phase 34: the "hand of god" endangered-predator immigration is
            // RETIRED — extinction is now a real evolutionary outcome, and
            // the main RNG stream no longer draws here (re-baseline covered).
            // The experimenter injection interface is the public
            // `spawn_agent` method (off by default; nothing in the engine
            // calls it on its own).

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

                // HERB lineage divergence (Phase 34: the legacy script retired —
                // no `speciation_fired < 3` throttle, no 迅聚/缓独 preset names.
                // The two sides get deterministic derived labels
                // "<kind>-L<n><side>" (no semantic preset); the CV/social gates
                // and the 60-tick spacing remain static instruments.)
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
                        // Deterministic derived labels: event number + neutral
                        // side letter. Side "a" = the low-speed cluster, side
                        // "b" = the high-speed cluster (matches the marking
                        // below; no preset meaning in the letters).
                        let label_a = format!("{}-L{}a", self.kind_name(HERB), self.speciation_fired + 1);
                        let label_b = format!("{}-L{}b", self.kind_name(HERB), self.speciation_fired + 1);
                        let name_a = save_state::intern_str(&label_a);
                        let name_b = save_state::intern_str(&label_b);
                        let lineages = vec![
                            LineageInfo { id: "lineage-a", name: name_a, avg_speed: low_avg, count: count_fast },
                            LineageInfo { id: "lineage-b", name: name_b, avg_speed: high_avg, count: count_slow },
                        ];
                        self.speciation_event = Some(SpeciationEvent {
                            t: self.time, parent_kind: self.kind_name(HERB), lineages,
                            cv_speed: (cv * 1000.0).round() / 1000.0,
                            gap_idx,
                            name_suffix: format!("#{}", self.speciation_fired + 1),
                        });
                        // Phase 34: the `< 3` throttle is retired — the
                        // counter keeps counting (it still feeds the checksum
                        // stream), the time spacing stays the instrument.
                        if self.time - self.last_speciation > 60.0 {
                            self.last_speciation = self.time;
                            self.speciation_fired += 1;
                        }
                        // 谱系标记：每次 speciation event 都重打标；已隔离谱系
                        // 的个体不重划（保持隔离群成员稳定）。打标不消耗 RNG。
                        for (i, &idx) in sorted_idx.iter().enumerate() {
                            let cur = self.agents[idx].lineage.unwrap_or("");
                            if self.isolated_groups.contains_key(cur) { continue; }
                            self.agents[idx].lineage = Some(if i < gap_idx { name_a } else { name_b });
                        }
                        // 隔离登记：分化群之间生殖隔离；code 按 keys().length + 1 递增
                        for name in [name_a, name_b] {
                            if !self.isolated_groups.contains_key(name) {
                                let code = format!("隔离群-{}", self.isolated_groups.len() + 1);
                                self.isolated_groups.insert(name, IsolatedGroup { code, kind: self.kind_name(HERB), since: self.time });
                                self.isolated_order.push(name);
                            }
                        }
                        // The drain/bite instruments now key on the MOST RECENT
                        // event's derived labels (the retired script keyed the
                        // 迅聚/缓独 constants forever).
                        self.herb_lineage_a = Some(label_a);
                        self.herb_lineage_b = Some(label_b);
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

        // Phase 45: per-kind clade-isolation telemetry — the generalized
        // successor of the HERB-only lineage scan above (task 1.2). For every
        // kind with two or more living clades, the mean cross-clade trait
        // distance is the isolation signal around speciation events. Pure
        // bookkeeping (zero RNG), 600-tick cadence to keep the wasm frame
        // budget; the cross-clade pair scan is skipped while a kind holds a
        // single clade (the steady-state world pays nothing here).
        if self.tick_count % 600 == 0 {
            for k in 0..self.kinds.len() {
                let kind = k as SpeciesKind;
                let mut groups: Vec<(u32, Vec<usize>)> = Vec::new();
                for (j, a) in self.agents.iter().enumerate() {
                    if !a.alive || a.kind != kind { continue; }
                    match groups.iter_mut().find(|(cid, _)| *cid == a.clade_id) {
                        Some((_, m)) => m.push(j),
                        None => groups.push((a.clade_id, vec![j])),
                    }
                }
                if groups.len() < 2 { continue; }
                let mut sum = 0.0f64;
                let mut pairs = 0usize;
                for gi in 0..groups.len() {
                    for gj in (gi + 1)..groups.len() {
                        for &ai in &groups[gi].1 {
                            for &bi in &groups[gj].1 {
                                sum += self.pair_trait_distance(ai, bi);
                                pairs += 1;
                            }
                        }
                    }
                }
                if pairs == 0 { continue; }
                let pos = self.isolation_by_kind.iter().position(|(kid, _)| *kid == k as u32);
                let entry = match pos {
                    Some(p) => &mut self.isolation_by_kind[p].1,
                    None => {
                        self.isolation_by_kind.push((k as u32, Vec::new()));
                        let n = self.isolation_by_kind.len();
                        &mut self.isolation_by_kind[n - 1].1
                    }
                };
                entry.push(IsolationHistoryPoint { t: self.time, dist: (sum / pairs as f64 * 10000.0).round() / 10000.0, pairs });
                if entry.len() > 200 { entry.remove(0); }
            }

            // Phase 45: value-weighted bearer telemetry (task 2.3).
            // Σ_alive Σ_token max(0, use_gain): raw bearer counts weight all
            // knowledge equally; this weights held knowledge by its measured
            // energy yield, so the bottleneck report can watch whether the
            // ratchet concentrates VALUABLE knowledge.
            let mut vwb = 0.0f64;
            for a in self.agents.iter() {
                if !a.alive || a.known_tokens.is_empty() { continue; }
                for &tid in &a.known_tokens {
                    let v = self.token_registry.value_of(tid);
                    if v > 0.0 { vwb += v; }
                }
            }
            self.culture_stats.value_weighted_bearers = vwb;
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
            // Phase 36: dual-controller authority switch gate runs on the same
            // boundary to avoid per-tick overhead while keeping the shadow
            // trajectory strictly decoupled from the hot path.
            self.check_brain_authority_switch();
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

    /// Phase 34: weighted multi-axis trait distance between two agents — the
    /// MST edge metric replacing the 1-D |Δdeco_efficiency| distance. Core
    /// axes follow the `pair_trait_distance` normalization style
    /// (speed/6.4, sense/116, size/1.12, L1 mean); the ext-locus
    /// contribution is `EXT_LOCUS_WEIGHT × mean(|Δvalue|)` over id-paired
    /// loci (unit [0,1] band, symmetric, unpaired = 0); kinds whose genome
    /// mutates `deco_efficiency` add the raw |Δeff| (the [0.5, 1.5] band ⇒
    /// ≤ 1.0, same unit scale). Pure function of two genomes + the kind's
    /// `KindDef`: zero RNG, zero state — a detector instrument, not a script.
    fn pair_mst_distance(&self, ia: usize, ib: usize, kind: SpeciesKind) -> f64 {
        let ga = &self.agents[ia].genome;
        let gb = &self.agents[ib].genome;
        let core = ((ga.speed - gb.speed).abs() / 6.4
            + (ga.sense - gb.sense).abs() / 116.0
            + (ga.size - gb.size).abs() / 1.12) / 3.0;
        let mut d = core + Self::ext_locus_distance(&ga.ext, &gb.ext);
        if self.kind_def(kind).mutates_deco_eff {
            d += (ga.deco_efficiency - gb.deco_efficiency).abs();
        }
        d
    }

    /// Phase 34: the ext-locus contribution to the MST edge weight. Mean
    /// |Δvalue| over the id-paired loci (the lists are id-sorted — the Phase
    /// 32 invariant — so a two-pointer merge pairs them deterministically),
    /// scaled by the static `EXT_LOCUS_WEIGHT`. 0 when no locus is paired.
    fn ext_locus_distance(a: &[Locus], b: &[Locus]) -> f64 {
        if a.is_empty() || b.is_empty() { return 0.0; }
        let (mut i, mut j) = (0usize, 0usize);
        let mut sum = 0.0f64;
        let mut paired = 0usize;
        while i < a.len() && j < b.len() {
            match a[i].id.cmp(&b[j].id) {
                std::cmp::Ordering::Equal => {
                    sum += (a[i].value - b[j].value).abs();
                    paired += 1;
                    i += 1;
                    j += 1;
                }
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
            }
        }
        if paired == 0 { 0.0 } else { EXT_LOCUS_WEIGHT * sum / paired as f64 }
    }

    /// Prim's MST over the complete graph with the Phase 34 weighted
    /// multi-axis metric (edge weights computed on demand through `dist`).
    /// Returns n−1 edges (weight, from, to) with vertex = population
    /// position. Same deterministic tie-breaking as the retired 1-D variant:
    /// the lowest-index frontier vertex wins distance ties, and relaxation
    /// keeps the first (lowest-index) predecessor on equal weights.
    fn mst_edges_weighted(n: usize, dist: &dyn Fn(usize, usize) -> f64) -> Vec<(f64, usize, usize)> {
        if n < 2 { return Vec::new(); }
        let mut in_tree = vec![false; n];
        let mut best_w = vec![f64::INFINITY; n];
        let mut best_from = vec![usize::MAX; n];
        in_tree[0] = true;
        for k in 1..n {
            best_w[k] = dist(0, k);
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
                    let d = dist(pick, k);
                    if d < best_w[k] { best_w[k] = d; best_from[k] = pick; }
                }
            }
        }
        edges
    }

    /// Minimum Euclidean distance (world units) between two agent groups.
    fn majority_region(agents: &[Agent], pop_idx: &[usize], members: &[usize]) -> u8 {
        let mut tally = [0usize; REGION_COUNT as usize];
        for &p in members {
            let a = &agents[pop_idx[p]];
            tally[(a.region as usize).min(REGION_COUNT as usize - 1)] += 1;
        }
        let mut best = 0u8;
        for r in 1..REGION_COUNT as usize {
            if tally[r] > tally[best as usize] { best = r as u8; }
        }
        best
    }

    fn min_cross_dist(agents: &[Agent], pop_idx: &[usize], ga: &[usize], gb: &[usize]) -> f64 {
        let mut best = f64::INFINITY;
        for &p in ga {
            let a = &agents[pop_idx[p]];
            for &q in gb {
                let b = &agents[pop_idx[q]];
                let d = (a.x - b.x).hypot(a.z - b.z);
                if d < best { best = d; }
            }
        }
        best
    }

    /// Phase 63 candidate G: pay earned growth slots to derived kinds that
    /// have filled the share they already hold. Every
    /// `FORK_BUDGET_GROWTH_STEP_TICKS` of survival since the fork earns one
    /// slot (capped at `FORK_BUDGET_GROWTH_MAX_EXTRA`); the slot is a
    /// parent→child transfer, so `family_budget_total` is invariant — the
    /// answer to "where does the new budget come from" is "the parent's
    /// share, never a new pool". Zero RNG draws; runs on the 60-tick
    /// speciation boundary in registry order.
    ///
    fn grow_fork_budgets(&mut self) {
        if !self.budget_growth || !self.budget_fork { return; }
        let now = self.tick_count;
        // (kind, fork tick): one clade record per derived kind, kept as a
        // fossil after extinction, so the age clock is saved with the world.
        let mut plan: Vec<(SpeciesKind, u32)> = Vec::new();
        for (k, def) in self.kinds.defs().iter().enumerate() {
            if def.parent_kind.is_none() { continue; }
            if self.kind_budget(k as SpeciesKind) == 0 { continue; }
            if let Some(c) = self.species_clades.iter().find(|c| c.derived_kind == Some(k as u32)) {
                plan.push((k as SpeciesKind, c.birth_tick));
            }
        }
        for (kind, birth_tick) in plan {
            let target = (now.saturating_sub(birth_tick) / FORK_BUDGET_GROWTH_STEP_TICKS) as usize;
            let target = target.min(FORK_BUDGET_GROWTH_MAX_EXTRA);
            let earned = self.kind_growth_granted(kind);
            if target <= earned { continue; }
            let budget = self.kind_budget(kind);
            // Pay only a daughter that has filled what it already holds: the
            // Phase 62 failing daughters sit at >= 99.6 % birth-blocked, i.e.
            // exactly at their own share. A declining daughter gets nothing
            // (the parent keeps the slots).
            if self.count(kind) < budget { continue; }
            let Some(parent) = self.kind_def(kind).parent_kind else { continue };
            let parent_budget = self.kind_budget(parent);
            if parent_budget <= FORK_BUDGET_MIN { continue; }
            let pay = (target - earned).min(parent_budget - FORK_BUDGET_MIN);
            if pay == 0 { continue; }
            self.set_kind_budget(parent, parent_budget - pay);
            self.set_kind_budget(kind, budget + pay);
            while self.kind_growth_grants.len() <= kind as usize { self.kind_growth_grants.push(0); }
            self.kind_growth_grants[kind as usize] = earned + pay;
        }
    }

    /// Phase 34 generalized speciation detector: every kind's population is
    /// screened on the weighted multi-axis metric (Phase 15-33 ran DECO only).
    /// Fires every SPECIATION_CHECK_TICKS from sample(). Consumes no RNG
    /// draws; thresholds/gates are static instruments applied uniformly to
    /// all kinds (measuring devices, not per-species scripts).
    /// `#[inline(never)]`: inlining this (plus detect_splits) into the hot
    /// sample() monomorphizes multi-KB of engine into one wasm function and
    /// starves LLVM's shared-code passes — measured +11 KB on the binary.
    #[inline(never)]
    fn check_speciation(&mut self) {
        let now = self.tick_count;
        self.grow_fork_budgets();

        // (a) rebuild cached member lists + extinction bookkeeping (all
        // clades of all kinds, Phase 34). Family guard: an agent counts
        // toward a clade record only when that record belongs to the agent's
        // own kind family — unassigned sentinel-0 strays that transiently
        // point at another family's root never contaminate its cache.
        for clade in self.species_clades.iter_mut() { clade.members.clear(); }
        let ncl = self.species_clades.len();
        let mut region_counts: Vec<[u32; REGION_COUNT as usize]> =
            vec![[0u32; REGION_COUNT as usize]; ncl];
        for a in self.agents.iter() {
            if !a.alive { continue; }
            if self.clade_belongs_to_family(a.clade_id, a.kind) {
                let pos = self.species_clades.iter().position(|c| c.id == a.clade_id).unwrap();
                self.species_clades[pos].members.push(a.id);
                let r = (a.region as usize).min(REGION_COUNT as usize - 1);
                region_counts[pos][r] += 1;
            }
        }
        // Phase 61 telemetry: standing population per clade (with its
        // per-quadrant split), sampled at this pass — the same 60-tick cadence
        // that rebuilds `members`.
        let peaks: Vec<(u32, u32, usize, [u32; REGION_COUNT as usize])> = self.species_clades.iter()
            .enumerate()
            .filter(|(_, c)| !c.members.is_empty())
            .map(|(i, c)| (c.id, c.kind_id, c.members.len(), region_counts[i]))
            .collect();
        for (id, kid, n, by_region) in peaks { self.clade_life_peak(id, kid, n, &by_region); }
        // Phase 16: a clade whose member count hits zero freezes its record
        // AND logs an extinction event in the unified log. kind 2 = the clade
        // carried a competitive burden in the previous pass (Gause exclusion),
        // kind 1 = plain death (resource loss / old age). A root clade
        // emptying is a TRUE species extinction — no rescue exists anymore
        // (the Phase 34 神之手 retirement makes this the real record).
        let mut extinct: Vec<(u32, u8, u32, Option<u32>)> = Vec::new();
        for clade in self.species_clades.iter_mut() {
            if clade.members.is_empty() && clade.death_tick.is_none() {
                clade.death_tick = Some(now);
                let ek = if self.clade_burdens.iter().any(|(id, b)| *id == clade.id && *b > 0.0) {
                    EVENT_KIND_COMPETITIVE_EXCLUSION
                } else {
                    EVENT_KIND_EXTINCTION
                };
                extinct.push((clade.id, ek, clade.kind_id, clade.derived_kind));
            }
        }
        for (id, ek, kid, derived) in extinct {
            // Phase 61 telemetry: the extinction tick in the per-clade table.
            self.clade_life_extinct(id, kid);
            // Phase 49: an extinct derived kind returns its budget share up
            // the fork lineage — the guild total stays conserved across
            // speciation/turnover cycles (a permanently leaking share would
            // shrink the family one extinction at a time). Root kinds never
            // return (they hold the family budget by definition). The return
            // is idempotent: a second extinct clade of the same kind sees 0.
            let mut ret_budget = 0u32;
            if self.budget_fork {
                if let Some(dk) = derived {
                    let dk = dk as SpeciesKind;
                    let share = self.kind_budget(dk);
                    if share > 0 {
                        if let Some(pk) = self.kind_def(dk).parent_kind {
                            let parent_share = self.kind_budget(pk);
                            self.set_kind_budget(pk, parent_share + share);
                        }
                        self.set_kind_budget(dk, 0);
                        ret_budget = share as u32;
                    }
                }
            }
            self.speciation_events.push(CladeSpeciationEvent {
                tick: now,
                parent_clade_id: id,
                child_clade_ids: Vec::new(),
                genetic_distance: 0.0,
                event_kind: ek,
                kind_id: kid,
                parent_budget: 0,
                child_budget: ret_budget,
            });
        }

        // (a4) Phase 65 W1: windows that closed before this pass settle their
        // overdraft now. Runs after the extinct-kind returns above so a dead
        // daughter's share is already back with the parent (and its own
        // budget is 0, so it has no debt to settle).
        self.settle_fork_grace(now);

        // (b/c) per-kind: population → lazy root clade → repoint strays →
        // time-gated multi-axis split detection. Registry id order keeps the
        // pass deterministic.
        let n_kinds = self.kinds.len();
        while self.last_spec_tick_by_kind.len() < n_kinds {
            self.last_spec_tick_by_kind.push(0);
        }
        for k in 0..n_kinds {
            let kind = k as SpeciesKind;
            let idxs: Vec<usize> = self.agents.iter().enumerate()
                .filter(|(_, a)| a.alive && a.kind == kind)
                .map(|(i, _)| i)
                .collect();
            if idxs.is_empty() { continue; }
            // (b) Strays repoint to the FAMILY's root record when one exists
            // (the ancestor kind's root; a derived kind's lineage roots at its
            // ancestor's root). An agent with clade_id 0 (unassigned) is
            // repointed the same way. Kinds that never split have no root
            // record and their agents keep the 0 sentinel — the tree stays
            // minimal until a split materializes the root (Phase 34).
            if let Some(root_id) = self.kind_root_clade_id(kind) {
                for &i in &idxs {
                    let cid = self.agents[i].clade_id;
                    if !self.clade_belongs_to_family(cid, kind) {
                        self.agents[i].clade_id = root_id;
                    }
                }
            }
            // (c) split detection behind the per-kind time gate.
            if idxs.len() >= 2
                && now.saturating_sub(self.last_spec_tick_by_kind[k]) >= MIN_SPEC_TIME_TICKS
            {
                self.detect_splits(now, kind, &idxs);
            }
        }

        // (d) refresh cached members + centroids from live agents.
        self.refresh_clade_stats();

        // (e) Phase 16: niche re-derivation + competitive-exclusion burdens
        // (impl in niche.rs; pure bookkeeping — consumes no RNG draws).
        self.update_niches_and_competition();
    }

    /// Phase 34: the trait axes the detector measures for `kind` (recorded on
    /// every clade as the SaveState v8 `axes` field): the three core axes
    /// (always active, pair_trait_distance normalization) + the ext-locus
    /// contribution (always active; zero when no locus is carried) + the
    /// efficiency axis for kinds whose genome mutates `deco_efficiency`.
    fn detection_axes(&self, kind: SpeciesKind) -> Vec<&'static str> {
        let mut axes: Vec<&'static str> = vec!["speed", "sense", "size", "ext"];
        if self.kind_def(kind).mutates_deco_eff { axes.push("deco_eff"); }
        axes
    }

    /// Phase 34: the family's root clade record id — the ancestor kind's
    /// root (walking the fork lineage up). `None` when the family has no
    /// root record yet (no split has fired for it, and no derived kind can
    /// exist in that case).
    fn kind_root_clade_id(&self, kind: SpeciesKind) -> Option<u32> {
        let root_k = self.root_kind(kind);
        self.species_clades.iter()
            .find(|c| c.parent_id.is_none() && c.kind_id == root_k as u32)
            .map(|c| c.id)
    }

    /// Phase 34: is clade record `cid` a valid home for an agent of `kind`?
    /// Missing records are invalid; the family check walks the fork lineage
    /// (`root_kind`) so a daughter clade of this kind's family is kept and a
    /// clade of another kind's family is repointed. Pre-v8 saves carry
    /// `kind_id = u32::MAX` — those fall back to the recorded kind NAME.
    fn clade_belongs_to_family(&self, cid: u32, kind: SpeciesKind) -> bool {
        let Some(c) = self.species_clades.iter().find(|c| c.id == cid) else { return false; };
        if c.kind_id == u32::MAX {
            return c.kind == self.kind_name(kind);
        }
        self.root_kind(c.kind_id as SpeciesKind) == self.root_kind(kind)
    }

    /// MST gap detection + clade split + Phase 34 kind-fork. `idxs` holds
    /// the agent slots of all alive agents of `kind`; vertex positions
    /// below index into it. Deliberately HashMap/std-sort free: every
    /// generic sort or hash-map instantiation would monomorphize multi-KB
    /// machinery into the wasm binary (insertion sorts over tiny vecs are
    /// both smaller and fast).
    #[inline(never)]
    fn detect_splits(&mut self, now: u32, kind: SpeciesKind, idxs: &[usize]) {
        let n = idxs.len();
        // Phase 34: weighted multi-axis metric (core + ext + the DECO
        // family's efficiency axis). `pair_mst_distance` is a pure function
        // of two genomes — zero RNG, zero state.
        let dist = |pa: usize, pb: usize| self.pair_mst_distance(idxs[pa], idxs[pb], kind);
        let all_edges = Self::mst_edges_weighted(n, &dist);
        let cut: Vec<(f64, usize, usize)> = all_edges.iter()
            .filter(|(w, _, _)| *w >= SPECIATION_DISTANCE_THRESHOLD)
            .copied()
            .collect();
        if cut.is_empty() { return; }

        // Phase 34: materialize this lineage's root clade if the split fired
        // before any root record existed. A kind that never splits never
        // grows a root record (the tree stays minimal); the root is
        // "born" at tick 0 but MATERIALIZED at its first split, and every
        // unassigned member (clade_id 0 sentinel) is assigned to it here.
        if self.kind_root_clade_id(kind).is_none() {
            let refs: Vec<&Agent> = idxs.iter().map(|&i| &self.agents[i]).collect();
            let centroid = Self::mean_genome(&refs);
            let id = self.next_clade_id;
            self.next_clade_id += 1;
            self.species_clades.push(SpeciesClade {
                id,
                parent_id: None,
                kind: self.kind_name(kind),
                name: self.kind_name(kind).to_string(),
                birth_tick: 0,
                death_tick: None,
                centroid_genome: centroid,
                members: idxs.iter().map(|&i| self.agents[i].id).collect(),
                niche_offset: 0.0,
                kind_id: kind as u32,
                derived_kind: None,
                axes: self.detection_axes(kind),
            });
            for &i in idxs {
                if self.agents[i].clade_id == 0 {
                    self.agents[i].clade_id = id;
                }
            }
        }

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
                let cid = self.agents[idxs[p]].clade_id;
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
            if comp_members[child_side].len() < self.speciation_min_side
                || comp_members[parent_side].len() < self.speciation_min_side {
                continue;
            }
            let dist = Self::min_cross_dist(
                &self.agents, idxs,
                &comp_members[parent_side], &comp_members[child_side],
            );
            if dist <= GEO_ISOLATION_DIST {
                // Phase 47 region-island model: different home quadrants are
                // different geographic demes, so a cross-region cut is
                // isolated even where the two demes share a land boundary.
                if !self.region_isolation
                    || Self::majority_region(&self.agents, idxs, &comp_members[parent_side])
                        == Self::majority_region(&self.agents, idxs, &comp_members[child_side]) {
                    continue;
                }
            }
            used_as_child[child_side] = true;
            let min_id = comp_members[child_side].iter()
                .map(|&p| self.agents[idxs[p]].id).min().unwrap();
            splits.push((comp_clade[parent_side], min_id, comp_members[child_side].clone(), w));
        }
        if splits.is_empty() { return; }

        // Deterministic order: parent clade id, then smallest member agent id.
        insertion_sort_by(splits.as_mut_slice(), |a, b| (a.0, a.1).cmp(&(b.0, b.1)));
        let mut last_parent: Option<u32> = None;
        let mut event: Option<CladeSpeciationEvent> = None;
        for (parent, _min_id, members, w) in splits {
            // The forking species: the parent clade's own kind (root ancestor
            // or an earlier derived fork).
            let family_kind = self.clade_kind(parent, kind);
            // Phase 49: a fork carves the daughter's share out of the parent
            // kind's budget. When the parent cannot afford a fork (share below
            // 2 × FORK_BUDGET_MIN) the split does not fire at all — no
            // carrying capacity, no new species; the lineage stays one kind.
            let grant = if self.budget_fork {
                match self.fork_grant_for(self.kind_budget(family_kind)) {
                    Some(g) => {
                        self.set_kind_budget(family_kind, self.kind_budget(family_kind) - g);
                        Some(g)
                    }
                    None => continue,
                }
            } else {
                None
            };
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
                    kind_id: family_kind as u32,
                    // Telemetry: the parent's share before this fork and the
                    // first child's grant (multi-child events are rare; each
                    // child clade's share is also readable via kind_budget).
                    parent_budget: self.kind_budget(family_kind) as u32,
                    child_budget: 0,
                });
                last_parent = Some(parent);
            }
            let cid = self.next_clade_id;
            self.next_clade_id += 1;
            // Phase 34 kind-fork: the daughter clade mints a derived
            // KindDef — the parent's constants table verbatim (diet,
            // behavior, caps), a deterministic name
            // "<parent-name>-<clade-letter>", and the parent link. Pure data
            // event: existing members KEEP their old kind; births INTO this
            // clade carry the new kind id (reproduce resolves via
            // `clade_kind`). The six ancestors stay the tree roots.
            let label = clade_letter(cid);
            let derived_name = format!("{}-{}", self.kind_name(family_kind), label);
            let mut ddef = self.kind_def(family_kind).clone();
            ddef.name = Box::leak(derived_name.clone().into_boxed_str());
            ddef.parent_kind = Some(family_kind);
            let did = self.register_kind(ddef);
            // Phase 49: the daughter's share is the grant carved out of the
            // parent kind; the guild total is unchanged.
            if let Some(g) = grant {
                self.set_kind_budget(did, g);
                if let Some(ev) = event.as_mut() {
                    if ev.child_budget == 0 { ev.child_budget = g as u32; }
                }
            }
            // Phase 65 W1: the newborn kind may exceed its grant for one
            // window; the settlement pass charges the parent if it survives.
            if self.budget_grace {
                self.set_kind_grace_until(did, now + self.grace_ticks);
            }
            let derived_kind_name = self.kind_name(did);
            let refs: Vec<&Agent> = members.iter().map(|&p| &self.agents[idxs[p]]).collect();
            let centroid = Self::mean_genome(&refs);
            // Phase 16: birth-time niche = parent niche + centroid eff gap
            // (update_niches_and_competition re-derives it on the same pass;
            // this initial value keeps the record self-consistent). Non-DECO
            // families keep a 0-centered niche (competition stays DECO-only).
            let (parent_niche, parent_eff, parent_speed) = self.species_clades.iter()
                .find(|c| c.id == parent)
                .map(|p| (p.niche_offset, p.centroid_genome.deco_efficiency, p.centroid_genome.speed))
                .unwrap_or((0.0, centroid.deco_efficiency, centroid.speed));
            let niche0 = Self::generational_niche(
                parent_niche,
                (centroid.deco_efficiency - parent_eff) * NICHE_EFF_SCALE,
                centroid.speed - parent_speed,
                self.niche_gen_accum,
            );
            for &p in members.iter() {
                self.agents[idxs[p]].clade_id = cid;
            }
            self.species_clades.push(SpeciesClade {
                id: cid,
                parent_id: Some(parent),
                kind: derived_kind_name,
                name: derived_name,
                birth_tick: now,
                death_tick: None,
                centroid_genome: centroid,
                members: members.iter().map(|&p| self.agents[idxs[p]].id).collect(),
                niche_offset: niche0,
                kind_id: did as u32,
                derived_kind: Some(did as u32),
                axes: self.detection_axes(did),
            });
            let ev = event.as_mut().unwrap();
            ev.child_clade_ids.push(cid);
            if w > ev.genetic_distance { ev.genetic_distance = w; }
        }
        if let Some(ev) = event.take() {
            self.speciation_events.push(ev);
        }
        // Per-kind time gate (Phase 34); the legacy global tick is kept in
        // lockstep for the save contract.
        self.last_spec_tick_by_kind[kind as usize] = now;
        self.last_clade_spec_tick = now;
    }

    /// Rebuild `members` from live agents and recompute centroid genomes for
    /// living clades (extinct clades keep their last frozen centroid).
    /// Phase 34: all kinds, not just DECO.
    #[inline(never)]
    fn refresh_clade_stats(&mut self) {
        for clade in self.species_clades.iter_mut() { clade.members.clear(); }
        for a in self.agents.iter() {
            if !a.alive { continue; }
            if self.clade_belongs_to_family(a.clade_id, a.kind) {
                let pos = self.species_clades.iter().position(|c| c.id == a.clade_id).unwrap();
                self.species_clades[pos].members.push(a.id);
            }
        }
        let alive: Vec<&Agent> = self.agents.iter().filter(|a| a.alive).collect();
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
    /// Phase 55: read access for observatories/tests — the spatial-
    /// heterogeneity metrics of the structure-env A/B need the standing crop
    /// and the suitability mask (dead cells never grow and would add a
    /// constant floor to the variance).
    pub fn grass_field(&self) -> &[f32] { &self.grass }
    /// Phase 55: suitability mask accessor, see `grass_field`.
    pub fn suit_field(&self) -> &[f32] { &self.suit }
    /// Phase 56: per-cell carcass decay multiplier accessor (all-1.0 whenever
    /// no structure has imprinted it, so an A/B-fresh world reads flat).
    pub fn deco_env_field(&self) -> &[f32] { &self.deco_env }
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
    /// Historical v1 golden anchor (Phase 31-33): mixes the kind NAME's
    /// first byte (the value every legacy anchor, including
    /// `0x790184996f3acefd`, was pinned against). Retired as primary in
    /// Phase 34 — kept as a read-only historical accessor.
    pub fn checksum_v1(&self) -> String {
        self.checksum_with(|a| self.kind_name(a.kind).as_bytes()[0] as u64, false)
    }

    /// Phase 34 (Q2 ruling, promoted from the Phase 31 parallel export):
    /// the PRIMARY checksum. Same mix stream as v1 except the kind field is
    /// the registry id (u32) instead of the name's first byte, PLUS the
    /// per-agent ext-locus hash (id order, a fixed draw-free computation).
    /// Six legacy ids (0-5) never collide with the first bytes
    /// (102/104/111/112/115/100), so v1 and v2 differ by construction.
    pub fn checksum(&self) -> String {
        self.checksum_v2()
    }

    /// The v2 primary (alias kept so `main.rs`'s dual-output and existing
    /// call sites keep working; `checksum()` and `checksum_v2()` are equal).
    pub fn checksum_v2(&self) -> String {
        self.checksum_with(|a| a.kind as u32 as u64, true)
    }

    /// Shared mix stream. `kind_field` is the divergence point between v1
    /// (name byte) and v2 (registry id); `mix_ext` adds the Phase 34
    /// ext-locus hash. Keeping one source prevents the streams from
    /// drifting.
    fn checksum_with(&self, kind_field: impl Fn(&Agent) -> u64, mix_ext: bool) -> String {
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
            if mix_ext {
                // Phase 34: ext-locus hash. Each locus (id order) mixes its
                // id + the f64 value's 32-bit halves into the agent slot.
                // Pure function of state — zero RNG draws.
                for l in &a.genome.ext {
                    mix(l.id as u64);
                    let bits = l.value.to_bits();
                    mix((bits >> 32) as u64);
                    mix(bits as u64);
                }
            }
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
        if mix_ext {
            // Phase 39.1: group-layer state. Empty groups mix nothing, so a
            // group-free world keeps the pre-39.1 v2 value; once groups exist
            // the checksum reflects pool energy, membership and birth timing
            // (the group layer becomes part of the save/restore contract).
            // Mixed in v2 only — `checksum_v1` stays the historical stream.
            for g in &self.groups {
                mix(g.id as u64);
                mix(g.kind as u32 as u64);
                mix(g.members.len() as u64);
                mix((g.energy_pool * 100.0).floor() as i64 as u64);
                mix(g.birth_tick as u64);
            }
        }
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
    /// Phase 34: experimenter injection interface (the retired 神之手's
    /// replacement, opt-in only). Spawns one agent of `kind` at `(x, z)`
    /// through the FULL spawn path (biome cap check, fresh random genome,
    /// RNG draws, generation 0) and returns the spawned agent's id, or
    /// `None` when the kind's cap is already reached. Nothing in the engine
    /// calls this on its own — extinction is a real outcome; this is the
    /// controlled door for tests / the research CLI.
    pub fn spawn_agent(&mut self, kind: SpeciesKind, x: f64, z: f64) -> Option<u32> {
        let age = 30.0 + self.rng.next_f64() * 80.0;
        let g = self.random_genome(kind);
        self.spawn(kind, x, z, Some(g), 0, age).map(|a| a.id)
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

    /// Phase 39.1: number of in-flight cohesion trackers (research readout).
    pub fn group_pending_len(&self) -> usize { self.group_pending.len() }

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
        Region: a.region,
        offspring: a.offspring,
        ext_loci: a.genome.ext.iter().map(locus_to_save).collect(),
        // Phase 36: micro-brain roundtrip (compact flat layout).
        brain_weights: a.brain.weights.clone(),
        brain_n_in: a.brain.n_in as u16,
        brain_n_hid: a.brain.n_hid as u16,
        brain_n_out: a.brain.n_out as u16,
        brain_authority: a.brain_authority,
        brain_equiv_streak: a.brain_equiv_streak,
        brain_agree_window: a.brain_agree_window,
        brain_window_len: a.brain_window_len,
        brain_eligibility: a.brain.eligibility.clone(),
        brain_taxis_baseline: a.brain.taxis_baseline,
        brain_nodes_out: a.brain.nodes.iter().map(|n| n.output).collect(),
        brain_outputs: a.brain.outputs.clone(),
        brain_last_reward: a.brain.last_reward,
        // Phase 37: learned knowledge tokens (in learning order).
        known_tokens: a.known_tokens.clone(),
        // Phase 39.1: group membership, role and somatic damage.
        group_id: a.group_id,
        role: a.role,
        somatic_damage: a.somatic_damage,
        // Phase 41: three-layer brain as a compact JSON string (flat visitor
        // footprint, same trick as SavedClades).
        triune: serde_json::to_string(&a.triune).unwrap_or_default(),
        // Phase 42: triune agreement gate bookkeeping (authority switch state).
        triune_equiv_streak: a.triune_equiv_streak,
        triune_agree_bits: a.triune_agree_bits,
        triune_agree_n: a.triune_agree_n,
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

/// Phase 34: one `KindDef` as its save record (the runtime registry rides
/// `SaveState.kinds`; the six legacy entries are always present, derived
/// fork kinds appended in registration order).
fn kind_def_to_save(d: &KindDef) -> save_state::SaveKindDef {
    let behavior = match d.behavior {
        BehaviorTemplate::Herb => "Herb",
        BehaviorTemplate::Pred => "Pred",
        BehaviorTemplate::Scav => "Scav",
        BehaviorTemplate::Omn => "Omn",
        BehaviorTemplate::Fish => "Fish",
        BehaviorTemplate::Deco => "Deco",
    };
    let genome_template = match d.genome_template {
        GenomeTemplate::Herb => "Herb",
        GenomeTemplate::Fish => "Fish",
        GenomeTemplate::Deco => "Deco",
        GenomeTemplate::General => "General",
    };
    let biome_cap = match d.biome_cap {
        BiomeCap::Herb => "Herb",
        BiomeCap::Pred => "Pred",
        BiomeCap::Scav => "Scav",
        BiomeCap::Omn => "Omn",
        BiomeCap::Fish => "Fish",
        BiomeCap::Fallback => "Fallback",
    };
    save_state::SaveKindDef {
        name: d.name.to_string(),
        behavior: behavior.to_string(),
        genome_template: genome_template.to_string(),
        biome_cap: biome_cap.to_string(),
        global_cap: d.global_cap,
        max_energy: d.max_energy,
        repro_cooldown: d.repro_cooldown,
        range_base: d.range_base,
        mature_age: d.mature_age,
        repro_base_threshold: d.repro_base_threshold,
        repro_drain_pair: [d.repro_drain_pair.0, d.repro_drain_pair.1],
        repro_drain_solo: d.repro_drain_solo,
        start_energy: [d.start_energy.0, d.start_energy.1],
        speed_clamp: [d.speed_clamp.0, d.speed_clamp.1],
        mutates_fin_color: d.mutates_fin_color,
        mutates_deco_eff: d.mutates_deco_eff,
        eats_plant: d.diet.eats_plant,
        plant_types: d.diet.plant_types.clone(),
        eats_carcass: d.diet.eats_carcass,
        prey_kinds: d.diet.prey_kinds.iter().map(|k| *k as u32).collect(),
        parent_kind: d.parent_kind.map(|k| k as u32),
    }
}

/// Phase 34: the inverse of `kind_def_to_save` (load side). Enum labels
/// fall back to the conservative PRED/General/Fallback shapes when a
/// newer-engine label is unknown; `parent_kind` is a registry id and is
/// valid because the save list is id-ordered (parents precede children).
fn save_kind_def_to_kind(k: &save_state::SaveKindDef) -> KindDef {
    let behavior = match k.behavior.as_str() {
        "Herb" => BehaviorTemplate::Herb,
        "Pred" => BehaviorTemplate::Pred,
        "Scav" => BehaviorTemplate::Scav,
        "Omn" => BehaviorTemplate::Omn,
        "Fish" => BehaviorTemplate::Fish,
        "Deco" => BehaviorTemplate::Deco,
        _ => BehaviorTemplate::Pred,
    };
    let genome_template = match k.genome_template.as_str() {
        "Herb" => GenomeTemplate::Herb,
        "Fish" => GenomeTemplate::Fish,
        "Deco" => GenomeTemplate::Deco,
        _ => GenomeTemplate::General,
    };
    let biome_cap = match k.biome_cap.as_str() {
        "Herb" => BiomeCap::Herb,
        "Pred" => BiomeCap::Pred,
        "Scav" => BiomeCap::Scav,
        "Omn" => BiomeCap::Omn,
        "Fish" => BiomeCap::Fish,
        _ => BiomeCap::Fallback,
    };
    KindDef {
        id: 0, // `KindRegistry::register` assigns the real id.
        name: save_state::intern_str(&k.name),
        behavior,
        genome_template,
        biome_cap,
        global_cap: k.global_cap,
        max_energy: k.max_energy,
        repro_cooldown: k.repro_cooldown,
        range_base: k.range_base,
        mature_age: k.mature_age,
        repro_base_threshold: k.repro_base_threshold,
        repro_drain_pair: (k.repro_drain_pair[0], k.repro_drain_pair[1]),
        repro_drain_solo: k.repro_drain_solo,
        start_energy: (k.start_energy[0], k.start_energy[1]),
        speed_clamp: (k.speed_clamp[0], k.speed_clamp[1]),
        mutates_fin_color: k.mutates_fin_color,
        mutates_deco_eff: k.mutates_deco_eff,
        diet: Diet {
            eats_plant: k.eats_plant,
            plant_types: k.plant_types.clone(),
            eats_carcass: k.eats_carcass,
            prey_kinds: k.prey_kinds.iter().map(|k| *k as SpeciesKind).collect(),
        },
        parent_kind: k.parent_kind.map(|k| k as SpeciesKind),
    }
}

/// Compact camelCase JSON view of a SpeciesClade for the save-state string
/// fields (mirrored by from_save_state's JsonClade and the JS phylogeny
/// reader). Only centroid_eff survives the roundtrip of the centroid genome;
/// the remaining fields are recomputed from live members on the next pass.
/// Phase 34 additive keys: `kindId` (the clade's registry kind id),
/// `derivedKind` (the forked kind's NAME, null when the clade carries no
/// derived kind) and `axes` (the detection axes active for this clade).
/// Pre-Phase-34 JSON parses with kindId = u32::MAX / derivedKind = null /
/// axes = [] (all `get(...)`-style, never a hard requirement).
fn clade_to_json(c: &SpeciesClade) -> String {
    let parent = match c.parent_id {
        Some(p) => p.to_string(),
        None => "null".to_string(),
    };
    let death = match c.death_tick {
        Some(t) => t.to_string(),
        None => "null".to_string(),
    };
    let derived_kind = match c.derived_kind {
        Some(_) => format!("\"{}\"", c.kind), // the derived kind's name, JSON-quoted
        None => "null".to_string(),
    };
    let axes = c.axes.iter()
        .map(|a| format!("\"{}\"", a))
        .collect::<Vec<String>>()
        .join(",");
    format!(
        "{{\"id\":{},\"parentId\":{},\"kind\":\"{}\",\"name\":\"{}\",\"birthTick\":{},\"deathTick\":{},\"centroidEff\":{},\"aliveCount\":{},\"niche\":{},\"kindId\":{},\"derivedKind\":{},\"axes\":[{}]}}",
        c.id, parent, c.kind, c.name, c.birth_tick, death,
        c.centroid_genome.deco_efficiency, c.members.len(), c.niche_offset,
        c.kind_id, derived_kind, axes,
    )
}

/// Compact camelCase JSON view of a CladeSpeciationEvent (same contract as
/// clade_to_json). Phase 34 additive key: `kindId` (the affected clade's
/// registry kind id; u32::MAX sentinel serialized as 4294967295 on
/// pre-Phase-34 records, which parse back to the same value).
fn clade_event_to_json(e: &CladeSpeciationEvent) -> String {
    let children = e.child_clade_ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(",");
    // Phase 49 additive keys: the fork-budget telemetry (0 on extinction
    // records and pre-Phase-49 archives).
    format!(
        "{{\"tick\":{},\"parentCladeId\":{},\"childCladeIds\":[{}],\"geneticDistance\":{},\"kind\":{},\"kindId\":{},\"parentBudget\":{},\"childBudget\":{}}}",
        e.tick, e.parent_clade_id, children, e.genetic_distance, e.event_kind, e.kind_id,
        e.parent_budget, e.child_budget,
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
            // Phase 34: full runtime kind registry (legacy six + derived
            // fork kinds, id order) + the per-kind speciation time gate +
            // the retired HERB-script's latest lineage labels.
            Kinds: self.kinds.defs().iter().map(kind_def_to_save).collect(),
            LastSpecTickByKind: self.last_spec_tick_by_kind.clone(),
            HerbLineageA: self.herb_lineage_a.clone(),
            HerbLineageB: self.herb_lineage_b.clone(),
            // Phase 36: brain toggles + dedicated learning RNG state.
            BrainOn: self.brain_on,
            BrainUpkeepOn: self.brain_upkeep_on,
            InfoCostOn: self.info_cost_on,
            TrophicEffActive: self.trophic_eff_active,
            BrainSwitchEnabled: self.brain_switch_enabled,
            BrainLearningRngState: self.brain_learning_rng.state(),
            // Phase 41: triune-brain toggles.
            TriuneOn: self.triune_on,
            ReplayOn: self.replay_on,
            TriuneUpkeepOn: self.triune_upkeep_on,
            // Phase 37: culture layer serialization.
            TokenRegistry: self.token_registry.tokens.iter().map(|t| {
                save_state::SaveToken { id: t.id, vec: t.vec.clone(), use_gain: t.use_gain }
            }).collect(),
            StructureRegistry: self.structure_registry.structures.iter().map(|s| {
                save_state::SaveStructure {
                    id: s.id, token_id: s.token_id, x: s.x, z: s.z,
                    strength: s.strength, health: s.health, builder_id: s.builder_id,
                }
            }).collect(),
            CultureOn: self.culture_on,
            CultureRngState: self.culture_rng.state(),
            CultureFidelity: self.culture_fidelity,
            CultureBiasChannel: self.culture_bias_channel,
            CultureStats: save_state::SaveCultureStats {
                learn_count: self.culture_stats.learn_count,
                innovation_count: self.culture_stats.innovation_count,
                structure_place_count: self.culture_stats.structure_place_count,
                structure_maintenance_total: self.culture_stats.structure_maintenance_total,
                learn_cost_total: self.culture_stats.learn_cost_total,
                extinct_token_count: self.culture_stats.extinct_token_count,
                last_extinct_tick: self.culture_stats.last_extinct_tick,
                token_holder_count: self.culture_stats.token_holder_count,
                taught_count: self.culture_stats.taught_count,
                teach_cost_total: self.culture_stats.teach_cost_total,
                value_weighted_bearers: self.culture_stats.value_weighted_bearers,
            },
            // Phase 43: cultural ratchet + abiogenesis state.
            TeachOn: self.teach_on,
            AbiogenesisOn: self.abiogenesis_on,
            AbioRngState: self.abio_rng.state(),
            AbiogenesisEvents: self.abiogenesis_events,
            AbiogenesisTicks: self.abiogenesis_ticks,
            // Phase 45: patch disturbance + cultural value selection.
            PatchOn: self.patch_on,
            PatchP: self.patch_p,
            PatchRngState: self.patch_rng.state(),
            PatchEvents: self.patch_events,
            PatchLastTick: self.patch_last_tick,
            CultureValueOn: self.culture_value_on,
            // Phase 47: natal dispersal state.
            DispersalP: self.dispersal_p,
            DispersalRngState: self.dispersal_rng.state(),
            DispersalEvents: self.dispersal_events,
            DispersalLastTick: self.dispersal_last_tick,
            RegionIsolation: self.region_isolation,
            DecoMutScale: self.deco_mut_scale,
            // Phase 49: fork-budget state (independent per-kind shares).
            BudgetForkOn: self.budget_fork,
            KindBudgets: self.kind_budgets.clone(),
            BudgetGrowthOn: self.budget_growth,
            KindGrowthGrants: self.kind_growth_grants.clone(),
            // Phase 65: fork grace (the window end changes future births).
            ForkGraceOn: self.budget_grace,
            KindGraceUntil: self.kind_grace_until.clone(),
            SpeciationMinSide: self.speciation_min_side,
            NichePartitionOn: self.niche_partition,
            NicheGenAccum: self.niche_gen_accum,
            StructureEnv: self.structure_env.clone(),
            StructureEnvOn: self.structure_env_on,
            StructureEnvDecayActive: self.structure_env_decay_active,
            // Phase 56: decomposition field.
            DecoEnv: self.deco_env.clone(),
            DecoEnvOn: self.deco_env_on,
            DecoEnvDecayActive: self.deco_env_decay_active,
            // Phase 57: sign-neutral token value.
            TokenValueSignNeutral: self.token_value_sign_neutral,
            // Phase 39.1: group layer. `founder_genome` is serialized in full —
            // it is the propagation bottleneck, so it must survive the hop for
            // a restored sim to keep breeding from the same single genome.
            Groups: self.groups.iter().map(|g| save_state::SaveGroup {
                id: g.id,
                kind: g.kind as u32,
                members: g.members.clone(),
                founder_id: g.founder_id,
                energy_pool: g.energy_pool,
                birth_tick: g.birth_tick,
                parent_group: g.parent_group,
                roles: g.roles.clone(),
                clade_id: g.clade_id,
                founder_genome: genome_to_save(&g.founder_genome),
                founder_ext: g.founder_genome.ext.iter().map(locus_to_save).collect(),
                founder_lineage: g.founder_lineage.map(|s| s.to_string()),
                last_repro_tick: g.last_repro_tick,
                generation: g.generation,
            }).collect(),
            NextGroupId: self.next_group_id,
            GroupOn: self.group_on,
            GroupRngState: self.group_rng.state(),
            GroupPending: self.group_pending.iter().map(|t| save_state::SaveCohesion {
                anchor: t.anchor,
                kind: t.kind as u32,
                clade_id: t.clade_id,
                first_tick: t.first_tick,
            }).collect(),
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
        // Phase 34: when the save carries the full runtime registry
        // (`Kinds`, v8+), restore it verbatim — the six legacy entries plus
        // any derived fork kinds (names, constants, parent links). An empty
        // list (pre-v8) lands on the six legacy defaults.
        let mut kinds: KindRegistry = if state.Kinds.is_empty() {
            KindRegistry::legacy()
        } else {
            let mut reg = KindRegistry::legacy();
            // The save list always starts with the six legacy entries; any
            // entries from index 6 on are derived fork kinds restored via
            // register (appended in save order).
            for k in state.Kinds.iter().skip(LEGACY_KIND_COUNT) {
                let def = save_kind_def_to_kind(k);
                reg.register(def);
            }
            reg
        };
        let kind_ids: Vec<SpeciesKind> = state.agents.iter()
            .map(|a| kinds.resolve_or_register(&a.kind)).collect();
        let kinds_len = kinds.len();

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
        // Phase 34: the per-kind evolution/fitness slot lists are sized to the
        // kind registry (six legacy by default, + derived fork kinds in v8).
        // A shorter (legacy six) list pads with empty slots so every restored
        // kind id stays indexable; a longer one is kept as-is.
        assert!(state.evolution.len() >= LEGACY_KIND_COUNT,
            "evolution history slot count must cover the legacy six");
        assert!(state.generation_fitness.len() >= LEGACY_KIND_COUNT,
            "generation_fitness slot count must cover the legacy six");

        let grass_p = state.grass_by_species;
        let suit_eff_by = state.suit_eff_by_species;
        // Phase 33: per-plant scratch buffer sized to the plant registry
        // (a valid save already carries the right length; resize guards a
        // shorter list so `take_grass` never indexes out of bounds).
        let mut taken_buf = state.taken_buf;
        taken_buf.resize(pn, 0.0);

        let mut evolution: Vec<Vec<EvolutionSample>> = state.evolution.into_iter()
            .map(|hist| hist.into_iter().map(|e| EvolutionSample {
                t: e.t, gen: e.gen, count: e.count, speed: e.speed, sense: e.sense,
                size: e.size, metabolism: e.metabolism, bold: e.bold, curious: e.curious,
                social: e.social, help_thr: e.help_thr,
                deco_eff: e.DecoEffMean, deco_eff_std: e.DecoEffStd,
            }).collect::<Vec<_>>()).collect();
        let mut generation_fitness: Vec<Vec<FitnessRecord>> = state.generation_fitness.into_iter()
            .map(|recs| recs.into_iter().map(|r| FitnessRecord {
                gen: r.gen, age: r.age, offspring: r.offspring, fitness: r.fitness,
            }).collect::<Vec<_>>()).collect();
        // Phase 34: pad the per-kind slot lists to the restored registry so
        // every derived fork kind id stays indexable (`die()` indexes these).
        while evolution.len() < kinds_len { evolution.push(Vec::new()); }
        while generation_fitness.len() < kinds_len { generation_fitness.push(Vec::new()); }

        // Phase 47: pre-v16 saves carry Region=0xFF for every agent; assign
        // the balanced angular demes after the world is built.
        let needs_region_assignment = state.agents.iter().any(|a| a.Region == u8::MAX);
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
            // Phase 34 additive clade keys. Pre-v8 JSON has none of them:
            // kindId → u32::MAX (the "unresolved" sentinel, which
            // `clade_belongs_to_family` falls back to the kind NAME for),
            // derivedKind → null, axes → []. The registry is restored first,
            // so a v8 `derivedKind` name always resolves to a live id.
            let kind_id = c.get("kindId").and_then(|v| v.as_u64()).unwrap_or(u64::MAX) as u32;
            let derived_kind = c.get("derivedKind")
                .and_then(|v| v.as_str())
                .and_then(|n| kinds.id_of(n))
                .map(|k| k as u32);
            let axes: Vec<&'static str> = c.get("axes")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(save_state::intern_str)
                    .collect())
                .unwrap_or_default();
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
                kind_id,
                derived_kind,
                axes,
            }
        }).collect();
        let speciation_events = json_events.iter().map(|e| {
            // Phase 34 additive event key: kindId (the affected clade's
            // registry kind id). Pre-Phase-34 records parse to the
            // u32::MAX sentinel (the JSON stores it as 4294967295).
            let ev_kind_id = e.get("kindId").and_then(|v| v.as_u64()).unwrap_or(u64::MAX) as u32;
            CladeSpeciationEvent {
                tick: e["tick"].as_u64().unwrap_or(0) as u32,
                parent_clade_id: e["parentCladeId"].as_u64().unwrap_or(0) as u32,
                child_clade_ids: e.get("childCladeIds")
                    .and_then(|a| a.as_array())
                    .map(|arr| arr.iter().map(|v| v.as_u64().unwrap_or(0) as u32).collect())
                    .unwrap_or_default(),
                genetic_distance: e["geneticDistance"].as_f64().unwrap_or(0.0),
                // Phase 16: pre-v3 saves default to speciation (0).
                event_kind: e.get("kind").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
                kind_id: ev_kind_id,
                // Phase 49: pre-v17 saves carry no budget fields (0 = unknown).
                parent_budget: e.get("parentBudget").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                child_budget: e.get("childBudget").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            }
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

        // Phase 39.1: materialize the group table + id allocator before the
        // struct literal consumes the rest of the save state.
        let (restored_groups, restored_next_group_id) = {
            let max_id = state.Groups.iter().map(|g| g.id).max().unwrap_or(0);
            let next = state.NextGroupId.max(max_id + 1).max(1);
            let groups = state.Groups.into_iter().map(|sg| crate::group::Group {
                id: sg.id,
                kind: sg.kind.min(u16::MAX as u32) as SpeciesKind,
                members: sg.members,
                founder_id: sg.founder_id,
                energy_pool: sg.energy_pool,
                birth_tick: sg.birth_tick,
                parent_group: sg.parent_group,
                roles: sg.roles,
                clade_id: sg.clade_id,
                founder_genome: {
                    let mut gm = genome_from_save(sg.founder_genome);
                    gm.ext = sg.founder_ext.into_iter().map(locus_from_save).collect();
                    gm
                },
                founder_lineage: sg.founder_lineage.as_deref().map(save_state::intern_str),
                last_repro_tick: sg.last_repro_tick,
                generation: sg.generation,
            }).collect::<Vec<_>>();
            (groups, next)
        };

        // Phase 49: pre-v17 archives carry no budget table — rebuild the
        // root shares from the caps before `kinds` moves into the struct.
        let restored_kind_budgets: Vec<usize> = kinds.defs().iter().map(|d| d.global_cap).collect();

        let mut eco = Ecology {
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
                    region: a.Region,
                    offspring: a.offspring,
                    // Phase 36: backward-compatible brain restore.
                    // Pre-v9 saves carry empty weight/elgibility slices and
                    // zeroed auxiliary fields; the restored agent lands on
                    // Brain::zero(...) + FSM authority — behavior bit-identical
                    // to the pre-Phase-36 snapshot.
                    brain: {
                        let w = if a.brain_weights.is_empty() {
                            Vec::new()
                        } else { a.brain_weights.clone() };
                        let n_in = a.brain_n_in as usize;
                        let n_hid = a.brain_n_hid as usize;
                        let n_out = a.brain_n_out as usize;
                        let mut b = if w.is_empty() || n_in == 0 {
                            aether_brain::Brain::zero(n_in.max(1), n_hid.max(1), n_out.max(1))
                        } else {
                            aether_brain::Brain::from_weights(n_in, n_hid, n_out, &w)
                        };
                        if !a.brain_eligibility.is_empty() {
                            b.eligibility = a.brain_eligibility.clone();
                        }
                        b.taxis_baseline = a.brain_taxis_baseline;
                        // The recurrent layer reads `nodes[].output` as its
                        // previous-tick activations — restore them (and the
                        // logits) or the shadow trajectory silently diverges
                        // from the never-restored twin after the first tick.
                        if a.brain_nodes_out.len() == b.nodes.len() {
                            for (node, &out) in b.nodes.iter_mut().zip(a.brain_nodes_out.iter()) {
                                node.activation = out;
                                node.output = out;
                            }
                        }
                        if a.brain_outputs.len() == b.n_out {
                            b.outputs = a.brain_outputs.clone();
                        }
                        b.last_reward = a.brain_last_reward;
                        b
                    },
                    // Phase 41: triune brain. Pre-v12 archives carry an empty
                    // string and land on a deterministic reference brain.
                    triune: {
                        let mut t = if a.triune.is_empty() {
                            aether_brain::triune::TriuneBrain::reference(
                                kinds.name(kind_ids[i]), a.id,
                            )
                        } else {
                            serde_json::from_str::<aether_brain::triune::TriuneBrain>(&a.triune)
                                .unwrap_or_else(|_| aether_brain::triune::TriuneBrain::reference(
                                    kinds.name(kind_ids[i]), a.id,
                                ))
                        };
                        t.normalize_saved();
                        t
                    },
                    triune_equiv_streak: a.triune_equiv_streak,
                    triune_agree_bits: a.triune_agree_bits.min(
                        if T_EQUIV_K_TICKS >= 64 { u64::MAX } else { (1u64 << T_EQUIV_K_TICKS) - 1 },
                    ),
                    triune_agree_n: a.triune_agree_n.min(T_EQUIV_K_TICKS),
                    brain_authority: a.brain_authority,
                    brain_equiv_streak: a.brain_equiv_streak,
                    brain_equiv_start: 0,
                    brain_agree_window: a.brain_agree_window,
                    brain_window_len: a.brain_window_len,
                    brain_last_fsm_action: 0,
                    brain_last_brain_action: 0,
                    // Phase 37: learned knowledge tokens (empty on pre-v10 saves).
                    known_tokens: a.known_tokens,
                    // Phase 39.1: group membership / role / somatic damage
                    // (serde defaults make pre-v11 archives solitary & clean).
                    group_id: a.group_id,
                    role: a.role,
                    somatic_damage: if a.somatic_damage.is_finite() {
                        a.somatic_damage.clamp(0.0, crate::group::SOMATIC_MAX)
                    } else {
                        0.0
                    },
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
            // Phase 34: per-kind speciation time gate (zeroed on pre-v8
            // restores) + retired HERB-script lineage labels (None pre-v8 /
            // before the first event).
            last_spec_tick_by_kind: state.LastSpecTickByKind,
            herb_lineage_a: state.HerbLineageA,
            herb_lineage_b: state.HerbLineageB,
            ledger: EnergyLedger::new(),
            // Phase 36: backward-compatible defaults. Pre-v9 saves have no
            // brain fields — `#[serde(default)]` populates Brain::zero(...),
            // authority FSM, empty streaks; the deterministic shadow does not
            // alter any trajectory. Per-instance toggles inherit their default
            // constants (so old v8 saves run as a live-Phase-36 world on restore).
            brain_on: state.BrainOn,
            brain_upkeep_on: state.BrainUpkeepOn,
            info_cost_on: state.InfoCostOn,
            trophic_eff_active: state.TrophicEffActive,
            brain_switch_enabled: state.BrainSwitchEnabled,
            brain_learning: BrainLearningStats::default(),
            triune_on: state.TriuneOn,
            replay_on: state.ReplayOn,
            triune_upkeep_on: state.TriuneUpkeepOn,
            triune_stats: TriuneLearningStats::default(),
            brain_learning_rng: if state.BrainLearningRngState != 0 {
                Mulberry32::from_state(state.BrainLearningRngState)
            } else {
                // Fallback deterministic stream (pre-v9 / reset world).
                Mulberry32::new(state.seed ^ BRAIN_RNG_TAG)
            },
            // Phase 37: culture layer — backward-compatible defaults.
            token_registry: {
                let mut reg = crate::culture::CultureRegistry::new();
                for st in state.TokenRegistry {
                    let mut tk = crate::culture::KnowledgeToken::new(st.id, st.vec);
                    tk.use_gain = st.use_gain;
                    reg.register(tk);
                }
                reg.rebuild_value_cache();
                reg
            },
            structure_registry: {
                let mut reg = crate::culture::StructureRegistry::new();
                let structures = &state.StructureRegistry;
                for ss in structures {
                    reg.structures.push(crate::culture::Structure::new(
                        ss.id, ss.token_id, ss.x, ss.z, ss.strength, ss.health, ss.builder_id,
                    ));
                }
                reg.next_structure_id = structures.last().map(|s| s.id + 1).unwrap_or(1);
                reg
            },
            culture_on: state.CultureOn,
            culture_rng: if state.CultureRngState != 0 {
                Mulberry32::from_state(state.CultureRngState)
            } else {
                Mulberry32::new(state.seed ^ crate::culture::TOKEN_RNG_TAG)
            },
            culture_stats: crate::culture::CultureStats {
                learn_count: state.CultureStats.learn_count,
                innovation_count: state.CultureStats.innovation_count,
                structure_place_count: state.CultureStats.structure_place_count,
                structure_maintenance_total: state.CultureStats.structure_maintenance_total,
                learn_cost_total: state.CultureStats.learn_cost_total,
                extinct_token_count: state.CultureStats.extinct_token_count,
                last_extinct_tick: state.CultureStats.last_extinct_tick,
                token_holder_count: state.CultureStats.token_holder_count,
                taught_count: state.CultureStats.taught_count,
                teach_cost_total: state.CultureStats.teach_cost_total,
                value_weighted_bearers: state.CultureStats.value_weighted_bearers,
            },
            // Phase 43: cultural ratchet + abiogenesis. Pre-v14 archives carry
            // no fields → toggles ON (default world config), counters zeroed,
            // stream re-derived from seed ^ ABIO_RNG_TAG.
            teach_on: state.TeachOn,
            abiogenesis_on: state.AbiogenesisOn,
            abio_rng: if state.AbioRngState != 0 {
                Mulberry32::from_state(state.AbioRngState)
            } else {
                Mulberry32::new(state.seed ^ ABIO_RNG_TAG)
            },
            abiogenesis_events: state.AbiogenesisEvents,
            abiogenesis_ticks: state.AbiogenesisTicks,
            // Phase 45: patch disturbance + cultural value selection.
            // Pre-v15 archives carry no fields → patches ON at the current
            // calibration, stream re-derived from seed ^ PATCH_RNG_TAG,
            // value selection ON, zero counters.
            patch_on: state.PatchOn,
            patch_p: state.PatchP,
            patch_rng: if state.PatchRngState != 0 {
                Mulberry32::from_state(state.PatchRngState)
            } else {
                Mulberry32::new(state.seed ^ crate::patch::PATCH_RNG_TAG)
            },
            patch_events: state.PatchEvents,
            patch_last_tick: state.PatchLastTick,
            patch_last_x: 0.0,
            patch_last_z: 0.0,
            regrow_noise: crate::patch::generate_regrow_noise(),
            structure_env: if state.StructureEnv.len() == n {
                state.StructureEnv.clone()
            } else {
                vec![1.0f32; n]
            },
            structure_env_on: state.StructureEnvOn,
            structure_env_decay_active: state.StructureEnvDecayActive,
            deco_env: if state.DecoEnv.len() == n {
                state.DecoEnv.clone()
            } else {
                vec![1.0f32; n]
            },
            deco_env_on: state.DecoEnvOn,
            deco_env_decay_active: state.DecoEnvDecayActive,
            // Phase 58: the clamp is an experiment-arm override, never saved.
            deco_env_min: STRUCTURE_DECO_MIN,
            deco_env_max: STRUCTURE_DECO_MAX,
            token_value_sign_neutral: state.TokenValueSignNeutral,
            token_placement_by_value: false,
            deco_repro_saturate: true,
            deco_region_cap: DECO_REGION_CAP_ON,
            deco_region_weight: [1.0; REGION_COUNT as usize],
            deco_region_weight_sum: REGION_COUNT as f64,
            deco_region_weight_tick: u32::MAX,
            deco_cell_region: build_deco_cell_region_map(),
            deco_region_cap_renorm: DECO_REGION_CAP_RENORM_ON,
            deco_region_cap_floor: DECO_REGION_CAP_FLOOR_ON,
            deco_region_cap_floor_small: DECO_REGION_CAP_FLOOR_SMALL_ON,
            deco_region_cap_renorm_daughters: false,
            clade_life: Vec::new(),
            isolation_by_kind: Vec::new(),
            culture_value_on: state.CultureValueOn,
            // Phase 47: natal dispersal. Pre-v16 archives carry no fields →
            // current calibration, stream re-derived from
            // seed ^ DISPERSAL_RNG_TAG, zero counters.
            dispersal_p: state.DispersalP,
            dispersal_rng: if state.DispersalRngState != 0 {
                Mulberry32::from_state(state.DispersalRngState)
            } else {
                Mulberry32::new(state.seed ^ DISPERSAL_RNG_TAG)
            },
            dispersal_events: state.DispersalEvents,
            dispersal_last_tick: state.DispersalLastTick,
            region_isolation: state.RegionIsolation,
            deco_mut_scale: state.DecoMutScale,
            // Phase 49: fork budget. Pre-v17 archives carry an empty table →
            // rebuild root shares from the caps (derived kinds re-grant at
            // their next fork); the MIN_SIDE default keeps the Phase 47 arm
            // bit-exact for old archives.
            budget_fork: state.BudgetForkOn,
            kind_budgets: if state.KindBudgets.is_empty() {
                restored_kind_budgets
            } else {
                state.KindBudgets.clone()
            },
            // Phase 63: the growth arm's earned shares ride the budget table,
            // so the switch and the per-kind grant ledger are both restored.
            // Pre-v23 archives land on the compiled default (off during the
            // 5-seed A/B) with an empty ledger.
            budget_growth: state.BudgetGrowthOn,
            kind_growth_grants: state.KindGrowthGrants.clone(),
            // Phase 65: the grace switch and pending windows ride the archive;
            // the counters are telemetry and restart at zero.
            budget_grace: state.ForkGraceOn,
            kind_grace_until: state.KindGraceUntil.clone(),
            grace_ticks: FORK_GRACE_TICKS,
            kind_grace_skips: Vec::new(),
            kind_grace_peak_over: Vec::new(),
            kind_grace_settled: Vec::new(),
            kind_grace_unpaid: Vec::new(),
            fork_budget_min: FORK_BUDGET_MIN,
            fork_daughter_min: FORK_BUDGET_MIN,
            mate_radius_arm: if MATE_FLOOR_ON { 1 } else { 0 },
            radius_boost_searches: 0,
            radius_only_finds: 0,
            radius_only_find_dist_sum: 0.0,
            speciation_min_side: state.SpeciationMinSide,
            // Phase 51: niche resource partition. Pre-v18 archives carry no
            // flag → the current default world config (partition ON); the
            // O(1) niche cache is rebuilt from the restored clade table below.
            niche_partition: state.NichePartitionOn,
            niche_gen_accum: state.NicheGenAccum,
            niche_partition_share: crate::niche::NichePartitionShare::default(),
            niche_by_clade: Vec::new(),
            culture_fidelity: state.CultureFidelity,
            culture_bias_channel: state.CultureBiasChannel,
            // Phase 39.1: group layer. Pre-v11 archives carry no groups →
            // solitary world; the id allocator is clamped past every restored
            // group id so a post-restore formation can never reuse an id.
            groups: restored_groups,
            next_group_id: restored_next_group_id,
            group_on: state.GroupOn,
            group_rng: if state.GroupRngState != 0 {
                Mulberry32::from_state(state.GroupRngState)
            } else {
                Mulberry32::new(state.seed ^ crate::group::GROUP_RNG_TAG)
            },
            group_stats: crate::group::GroupStats::default(),
            group_pending: state.GroupPending.into_iter().map(|t| crate::group::CohesionTracker {
                anchor: t.anchor,
                kind: t.kind.min(u16::MAX as u32) as SpeciesKind,
                clade_id: t.clade_id,
                first_tick: t.first_tick,
            }).collect(),
        };
        if needs_region_assignment { eco.assign_initial_regions(); }
        // Phase 47: rebuild the clade member/centroid caches immediately, so a
        // save→restore→save hop is JSON-identical even when the restored world
        // has not yet reached its next detection pass.
        eco.refresh_clade_stats();
        // Phase 51: rebuild the O(1) niche cache the same way — a restored
        // world must not wait for the next detection pass to see its niches.
        eco.rebuild_niche_cache();
        eco
    }
}



#[cfg(test)]
mod phase63_fork_budget_tests {
    use super::*;
    use crate::save_state::{SaveState, SAVE_STATE_VERSION};

    /// A DECO-family daughter with `filled` live members and a synthetic
    /// share carved out of DECO (the fork path itself is covered by the
    /// real-world runs; here we test the growth transfer in isolation).
    fn growth_world(seed: u32, filled: usize, root_budget: usize) -> (Ecology, SpeciesKind) {
        let mut eco = Ecology::new(seed);
        eco.fork_budget_growth_on();
        // The regional share is a different gate (Phase 60/61/62); this test
        // isolates the kind-budget transfer, so keep all members in one place.
        eco.deco_region_cap_off();
        let d = {
            let mut def = eco.kind_def(DECO).clone();
            def.name = "deco-growth";
            def.parent_kind = Some(DECO);
            def.global_cap = 8;
            eco.register_kind(def)
        };
        eco.kind_budgets[DECO as usize] = root_budget;
        let cell = GRID_SPAN / GRID_N as f64;
        let j = (0..eco.suit.len()).find(|&j| eco.suit[j] > 0.5).expect("land");
        let x = -GRID_SPAN / 2.0 + ((j % GRID_N as usize) as f64 + 0.5) * cell;
        let z = -GRID_SPAN / 2.0 + ((j / GRID_N as usize) as f64 + 0.5) * cell;
        for i in 0..filled {
            assert!(eco.spawn_agent(d, x, z).is_some(), "spawn {i} within the daughter share");
        }
        let centroid = eco.agents[0].genome.clone();
        eco.species_clades.push(SpeciesClade {
            id: 100,
            parent_id: None,
            kind: "deco",
            name: "deco-growth".to_string(),
            birth_tick: 0,
            death_tick: None,
            centroid_genome: centroid,
            members: Vec::new(),
            niche_offset: 0.0,
            kind_id: DECO as u32,
            derived_kind: Some(d as u32),
            axes: Vec::new(),
        });
        (eco, d)
    }

    #[test]
    fn growth_pays_a_filled_daughter_from_the_parents_share() {
        let (mut eco, d) = growth_world(20261102, 8, 72);
        eco.tick_count = FORK_BUDGET_GROWTH_STEP_TICKS;
        eco.grow_fork_budgets();
        assert_eq!(eco.kind_budget(d), 9);
        assert_eq!(eco.kind_budget(DECO), 71);
        assert_eq!(eco.kind_growth_granted(d), 1);
        assert_eq!(eco.family_budget_total(DECO), 80, "parent→child transfer only");
    }

    #[test]
    fn growth_waits_for_the_daughter_to_fill_what_it_holds() {
        let (mut eco, d) = growth_world(20261102, 7, 72);
        eco.tick_count = FORK_BUDGET_GROWTH_STEP_TICKS;
        eco.grow_fork_budgets();
        assert_eq!(eco.kind_budget(d), 8, "a declining daughter earns nothing");
        assert_eq!(eco.kind_budget(DECO), 72);
        // Refill, then one more tier is paid at the next boundary.
        let (x, z) = (eco.agents[0].x, eco.agents[0].z);
        assert!(eco.spawn_agent(d, x, z).is_some());
        eco.grow_fork_budgets();
        assert_eq!(eco.kind_budget(d), 9);
        assert_eq!(eco.kind_budget(DECO), 71);
    }

    #[test]
    fn growth_stops_at_the_parent_floor_and_at_the_extra_cap() {
        let (mut eco, d) = growth_world(20261102, 8, FORK_BUDGET_MIN + 1);
        eco.tick_count = 100_000;
        eco.grow_fork_budgets();
        assert_eq!(eco.kind_budget(DECO), FORK_BUDGET_MIN, "parent keeps the floor");
        assert_eq!(eco.kind_budget(d), 9, "paid only the one slot the parent can spare");
        assert_eq!(eco.family_budget_total(DECO), (FORK_BUDGET_MIN + 1) + 8,
            "synthetic family total unchanged");

        let (mut eco, d) = growth_world(20261102, 8, 72);
        eco.tick_count = 1_000_000;
        eco.grow_fork_budgets();
        assert_eq!(eco.kind_budget(d), 8 + FORK_BUDGET_GROWTH_MAX_EXTRA);
        assert_eq!(eco.kind_budget(DECO), 72 - FORK_BUDGET_GROWTH_MAX_EXTRA);
        assert_eq!(eco.family_budget_total(DECO), 80);
    }

    #[test]
    fn growth_never_credits_a_root_kind() {
        let (mut eco, d) = growth_world(20261102, 8, 72);
        eco.tick_count = 1_000_000;
        eco.grow_fork_budgets();
        assert_eq!(eco.kind_growth_granted(DECO), 0, "roots hold their family share by definition");
        assert_eq!(eco.kind_budget(DECO), 64);
        assert_eq!(eco.kind_budget(d), 16);
        assert_eq!(eco.family_budget_total(DECO), 80);
    }

    #[test]
    fn growth_ledger_rides_the_archive_and_pre_v23_lands_off() {
        let (mut eco, d) = growth_world(20261102, 8, 72);
        eco.tick_count = FORK_BUDGET_GROWTH_STEP_TICKS;
        eco.grow_fork_budgets();
        let mut json = serde_json::to_value(eco.to_save_state()).unwrap();
        assert_eq!(json["Version"], SAVE_STATE_VERSION);
        assert_eq!(json["BudgetGrowthOn"], true);
        assert_eq!(json["KindGrowthGrants"][d as usize], 1);
        let restored = Ecology::from_save_state(serde_json::from_value::<SaveState>(json.clone()).unwrap());
        assert!(restored.budget_growth);
        assert_eq!(restored.kind_budget(d), 9);
        assert_eq!(restored.kind_growth_granted(d), 1);

        let obj = json.as_object_mut().unwrap();
        obj.remove("BudgetGrowthOn");
        obj.remove("KindGrowthGrants");
        json["Version"] = serde_json::json!(22);
        let legacy = Ecology::from_save_state(serde_json::from_value::<SaveState>(json).unwrap());
        assert!(!legacy.budget_growth, "pre-v23 archives land on the compiled default");
        assert_eq!(legacy.kind_growth_granted(d), 0);
    }

    #[test]
    fn phase49_grant_rule_is_bit_exact_and_the_floors_are_parameterised() {
        for parent in 0..=200usize {
            let old = if parent < 2 * FORK_BUDGET_MIN {
                None
            } else {
                let g = (parent / 4).clamp(FORK_BUDGET_MIN, FORK_BUDGET_GRANT);
                if parent - g < FORK_BUDGET_MIN { None } else { Some(g) }
            };
            assert_eq!(Ecology::fork_budget_grant(parent), old, "parent {parent}");
            let m12 = Ecology::fork_budget_grant_with(parent, 12, 12);
            if let Some(g) = m12 {
                assert!((12..=FORK_BUDGET_GRANT).contains(&g));
                assert!(parent - g >= 12);
            }
            let a12 = Ecology::fork_budget_grant_with(parent, FORK_BUDGET_MIN, 12);
            if let Some(g) = a12 {
                assert!((12..=FORK_BUDGET_GRANT).contains(&g));
                assert!(parent - g >= FORK_BUDGET_MIN);
            }
        }
        // The arms are distinguishable: 23 slots can fund a 12-slot daughter
        // while keeping the parent's own floor at 8, but not a symmetric
        // 12+12 (which needs 24).
        assert_eq!(Ecology::fork_budget_grant_with(23, FORK_BUDGET_MIN, 12), Some(12));
        assert_eq!(Ecology::fork_budget_grant_with(23, 12, 12), None);
    }

    #[test]
    fn symmetric_floor_shortens_the_fork_chain_without_minting_budget() {
        let chain = |min: usize| {
            let mut parent = 80usize;
            let mut forks = 0;
            while let Some(g) = Ecology::fork_budget_grant_with(parent, min, min) {
                parent -= g;
                forks += 1;
            }
            (forks, parent)
        };
        assert_eq!(chain(8), (6, 10), "Phase 49/53 chain from 80");
        assert_eq!(chain(FORK_BUDGET_MIN), (6, 10), "Phase 64 default (8)");
        assert_eq!(chain(10), (5, 14), "Phase 63's floor, now the M10 arm");
        assert_eq!(chain(12), (4, 21));
        assert_eq!(chain(16), (3, 28));
    }
}

#[cfg(test)]
mod phase65_fork_grace_tests {
    use super::*;
    use crate::save_state::SaveState;

    /// A DECO daughter with `grant` slots and `filled` live members, spawned
    /// through the real birth gate (the grace window is exercised, not
    /// bypassed). The clade record carries the spawned ids so the settlement's
    /// viability check sees the same population.
    fn grace_world(
        seed: u32,
        filled: usize,
        root_budget: usize,
        grant: usize,
        grace: bool,
    ) -> (Ecology, SpeciesKind) {
        let mut eco = Ecology::new(seed);
        if grace { eco.fork_grace_on(); } else { eco.fork_grace_off(); }
        // Isolate the kind-budget gate from the Phase 60 regional split.
        eco.deco_region_cap_off();
        let d = {
            let mut def = eco.kind_def(DECO).clone();
            def.name = "deco-grace";
            def.parent_kind = Some(DECO);
            eco.register_kind(def)
        };
        eco.kind_budgets[DECO as usize] = root_budget;
        eco.set_kind_budget(d, grant);
        if grace { eco.set_kind_grace_until(d, eco.grace_ticks); }
        let cell = GRID_SPAN / GRID_N as f64;
        let j = (0..eco.suit.len()).find(|&j| eco.suit[j] > 0.5).expect("land");
        let x = -GRID_SPAN / 2.0 + ((j % GRID_N as usize) as f64 + 0.5) * cell;
        let z = -GRID_SPAN / 2.0 + ((j / GRID_N as usize) as f64 + 0.5) * cell;
        let mut members = Vec::new();
        for i in 0..filled {
            let id = eco.spawn_agent(d, x, z);
            assert!(id.is_some(), "spawn {i} of {filled} (grant {grant})");
            members.push(id.unwrap());
        }
        let centroid = eco.agents[0].genome.clone();
        eco.species_clades.push(SpeciesClade {
            id: 100,
            parent_id: None,
            kind: "deco",
            name: "deco-grace".to_string(),
            birth_tick: 0,
            death_tick: None,
            centroid_genome: centroid,
            members,
            niche_offset: 0.0,
            kind_id: DECO as u32,
            derived_kind: Some(d as u32),
            axes: Vec::new(),
        });
        (eco, d)
    }

    #[test]
    fn without_grace_the_grant_is_a_hard_ceiling() {
        let (mut eco, d) = grace_world(20261116, FORK_BUDGET_MIN, 72, FORK_BUDGET_MIN, false);
        let (x, z) = (eco.agents[0].x, eco.agents[0].z);
        assert!(eco.spawn_agent(d, x, z).is_none(), "the legacy gate refuses at the grant");
        assert_eq!(eco.count(d), FORK_BUDGET_MIN);
        assert_eq!(eco.grace_total_skips(), 0);
        assert_eq!(eco.family_budget_total(DECO), 80);
    }

    #[test]
    fn grace_admits_at_most_one_extra_cohort() {
        let cap = FORK_BUDGET_MIN + FORK_GRACE_OVERDRAFT_MAX;
        let (mut eco, d) = grace_world(20261116, cap, 72, FORK_BUDGET_MIN, true);
        assert_eq!(eco.count(d), cap, "the whole overdraft is admissible");
        assert_eq!(eco.kind_grace_skips(d), FORK_GRACE_OVERDRAFT_MAX as u64);
        assert_eq!(eco.kind_grace_peak_over(d), FORK_GRACE_OVERDRAFT_MAX);
        let (x, z) = (eco.agents[0].x, eco.agents[0].z);
        assert!(eco.spawn_agent(d, x, z).is_none(), "grant + overdraft is a hard ceiling");
        assert_eq!(eco.kind_budget(d), FORK_BUDGET_MIN, "the window never moves slots");
        assert_eq!(eco.family_budget_total(DECO), 80);
    }

    #[test]
    fn settlement_moves_the_overdraft_out_of_the_parent_and_conserves_the_family() {
        let (mut eco, d) = grace_world(20261116, 12, 72, FORK_BUDGET_MIN, true);
        assert_eq!(eco.family_budget_total(DECO), 80);
        let now = eco.grace_ticks;
        eco.tick_count = now;
        eco.settle_fork_grace(now);
        assert_eq!(eco.kind_budget(DECO), 68, "the parent pays the four over-budget members");
        assert_eq!(eco.kind_budget(d), 12);
        assert_eq!(eco.kind_grace_settled(d), 4);
        assert_eq!(eco.kind_grace_unpaid(d), 0);
        assert_eq!(eco.family_budget_total(DECO), 80, "parent→daughter transfer only");
        assert_eq!(eco.kind_grace_until(d), 0, "the window is one-shot");
        // A second pass at the same tick must not charge again.
        eco.settle_fork_grace(now);
        assert_eq!(eco.kind_budget(DECO), 68);
        assert_eq!(eco.kind_budget(d), 12);
    }

    #[test]
    fn settlement_pays_at_most_the_parents_share_and_records_the_shortfall() {
        let (mut eco, d) = grace_world(20261116, 12, 3, FORK_BUDGET_MIN, true);
        assert_eq!(eco.family_budget_total(DECO), 11);
        let now = eco.grace_ticks;
        eco.tick_count = now;
        eco.settle_fork_grace(now);
        assert_eq!(eco.kind_budget(DECO), 0, "the parent is charged to zero, never below");
        assert_eq!(eco.kind_budget(d), 11);
        assert_eq!(eco.kind_grace_settled(d), 3);
        assert_eq!(eco.kind_grace_unpaid(d), 1, "the unpaid slot is recorded, not minted");
        assert_eq!(eco.family_budget_total(DECO), 11);
    }

    #[test]
    fn a_clade_below_min_side_charges_nothing() {
        // Contract test: with the real floor a derived kind cannot hold more
        // than its grant while its clade sits below MIN_SIDE, so the branch is
        // synthetic (budget 4 = below the floor).
        let (mut eco, d) = grace_world(20261116, 6, 72, FORK_BUDGET_MIN, true);
        eco.set_kind_budget(d, 4);
        eco.species_clades[0].members.truncate(6);
        let now = eco.grace_ticks;
        eco.tick_count = now;
        eco.settle_fork_grace(now);
        assert_eq!(eco.kind_budget(d), 4, "no transfer into a sub-viable clade");
        assert_eq!(eco.kind_budget(DECO), 72);
        assert_eq!(eco.kind_grace_settled(d), 0);
        assert_eq!(eco.kind_grace_unpaid(d), 2);
        assert_eq!(eco.family_budget_total(DECO), 76);
    }

    #[test]
    fn an_extinct_daughter_has_no_debt_after_its_share_returns() {
        let (mut eco, d) = grace_world(20261116, 12, 72, FORK_BUDGET_MIN, true);
        // The extinction path (check_speciation) removes the members, returns
        // the whole share to the parent and zeroes the daughter.
        for a in eco.agents.iter_mut() { a.alive = false; }
        eco.species_clades[0].members.clear();
        eco.species_clades[0].death_tick = Some(1);
        eco.set_kind_budget(DECO, 80);
        eco.set_kind_budget(d, 0);
        let now = eco.grace_ticks;
        eco.tick_count = now;
        eco.settle_fork_grace(now);
        assert_eq!(eco.kind_grace_settled(d), 0);
        assert_eq!(eco.kind_grace_unpaid(d), 0);
        assert_eq!(eco.family_budget_total(DECO), 80);
    }

    #[test]
    fn roots_never_enter_a_grace_window() {
        let (mut eco, _d) = grace_world(20261116, 10, 72, FORK_BUDGET_MIN, true);
        eco.set_kind_grace_until(DECO, eco.grace_ticks);
        assert!(!eco.in_fork_grace(DECO));
        assert_eq!(eco.kind_grace_until(DECO), 5_000);
    }

    #[test]
    fn grace_off_is_the_compiled_default_and_starts_clean() {
        let eco = Ecology::new(20261116);
        assert!(!eco.budget_grace, "W1 is arm-only until the 5-seed verdict");
        assert_eq!(eco.grace_ticks, FORK_GRACE_TICKS);
        assert_eq!(eco.grace_total_skips(), 0);
        assert!(eco.kind_grace_until.is_empty());
    }

    #[test]
    fn grace_window_rides_the_archive_and_pre_v24_lands_off() {
        let (mut eco, d) = grace_world(20261116, 12, 72, FORK_BUDGET_MIN, true);
        let mut json = serde_json::to_value(eco.to_save_state()).unwrap();
        assert_eq!(json["Version"], crate::save_state::SAVE_STATE_VERSION);
        assert_eq!(json["ForkGraceOn"], true);
        assert_eq!(json["KindGraceUntil"][d as usize], eco.grace_ticks);
        let restored = Ecology::from_save_state(
            serde_json::from_value::<SaveState>(json.clone()).unwrap());
        assert!(restored.budget_grace);
        assert_eq!(restored.kind_grace_until(d), eco.grace_ticks);
        assert_eq!(restored.family_budget_total(DECO), 80);

        // A v23 archive has neither key: lands OFF with no pending window.
        let obj = json.as_object_mut().unwrap();
        obj.remove("ForkGraceOn");
        obj.remove("KindGraceUntil");
        obj.insert("Version".to_string(), serde_json::json!(23));
        let old: SaveState = serde_json::from_value(json).unwrap();
        let restored = Ecology::from_save_state(old);
        assert!(!restored.budget_grace);
        assert_eq!(restored.kind_grace_until(d), 0);
        assert_eq!(restored.grace_total_skips(), 0);
    }
}

#[cfg(test)]
mod phase66_mate_miss_tests {
    use super::*;

    /// A DECO daughter with a controlled budget, minted the way a fork mints
    /// one, so the public birth gate accepts the fixture members.
    fn daughter(eco: &mut Ecology) -> SpeciesKind {
        eco.deco_region_cap_off();
        let mut def = eco.kind_def(DECO).clone();
        def.name = "deco-mate";
        def.parent_kind = Some(DECO);
        def.global_cap = 10;
        let d = eco.register_kind(def);
        eco.set_kind_budget(d, 10);
        d
    }

    /// Spawn one live member and pin the fields the mate search reads.
    fn plant(eco: &mut Ecology, d: SpeciesKind, x: f64, z: f64, region: u8, energy_frac: f64) -> usize {
        let id = eco.spawn_agent(d, x, z).expect("spawn");
        let i = eco.agents.iter().position(|a| a.id == id).unwrap();
        let a = &mut eco.agents[i];
        a.region = region;
        a.age = 100.0;
        a.repro = 0.0;
        a.energy = a.max_energy * energy_frac;
        i
    }

    #[test]
    fn no_conspecific_at_all_is_no_kin() {
        let mut eco = Ecology::new(20261123);
        let d = daughter(&mut eco);
        let i = plant(&mut eco, d, 0.0, 0.0, 0, 1.0);
        let (b, kp, rp, dist) = eco.mate_miss_bucket(i, 11.0, 0.68);
        assert_eq!(b, MateMissBucket::NoKin);
        assert_eq!((kp, rp), (1, 1), "population sums include the searcher");
        assert!(dist.is_none());
    }

    #[test]
    fn a_conspecific_in_another_region_is_other_region() {
        let mut eco = Ecology::new(20261123);
        assert!(eco.region_isolation);
        let d = daughter(&mut eco);
        let i = plant(&mut eco, d, 0.0, 0.0, 0, 1.0);
        plant(&mut eco, d, 4.0, 0.0, 1, 1.0);
        let (b, kp, rp, _) = eco.mate_miss_bucket(i, 11.0, 0.68);
        assert_eq!(b, MateMissBucket::OtherRegion);
        assert_eq!((kp, rp), (2, 1), "region pop excludes the other region");
    }

    #[test]
    fn a_fertile_conspecific_out_of_range_is_beyond_radius() {
        let mut eco = Ecology::new(20261123);
        let d = daughter(&mut eco);
        let i = plant(&mut eco, d, 0.0, 0.0, 0, 1.0);
        plant(&mut eco, d, 60.0, 0.0, 0, 1.0);
        let (b, kp, rp, dist) = eco.mate_miss_bucket(i, 11.0, 0.68);
        assert_eq!(b, MateMissBucket::BeyondRadius);
        assert_eq!((kp, rp), (2, 2));
        assert!((dist.unwrap() - 60.0).abs() < 1e-9, "reports the needed radius");
    }

    #[test]
    fn an_unready_conspecific_in_range_is_filtered() {
        let mut eco = Ecology::new(20261123);
        let d = daughter(&mut eco);
        let i = plant(&mut eco, d, 0.0, 0.0, 0, 1.0);
        let j = plant(&mut eco, d, 5.0, 0.0, 0, 0.10);
        let (b, _, _, dist) = eco.mate_miss_bucket(i, 11.0, 0.68);
        assert_eq!(b, MateMissBucket::Filtered);
        assert!(dist.is_none());
        // The same partner with energy above the bar is a find, not a miss.
        eco.agents[j].energy = eco.agents[j].max_energy;
        let (b2, _, _, _) = eco.mate_miss_bucket(i, 11.0, 0.68);
        assert_eq!(b2, MateMissBucket::Other);
    }

    fn many(eco: &mut Ecology, d: SpeciesKind, n: usize, region: u8) -> Vec<usize> {
        (0..n).map(|i| plant(eco, d, i as f64 * 3.0, 0.0, region, 1.0)).collect()
    }

    #[test]
    fn the_floor_is_off_by_default() {
        let mut eco = Ecology::new(20261123);
        assert_eq!(eco.mate_radius_arm(), 0);
        let d = daughter(&mut eco);
        assert_eq!(eco.mate_radius_with_floor(d, 0, 19.0), (19.0, false));
        assert_eq!(eco.radius_boost_searches(), 0);
    }

    #[test]
    fn r1_floor_follows_the_regional_deme_density() {
        let mut eco = Ecology::new(20261123);
        let d = daughter(&mut eco);
        many(&mut eco, d, 4, 0);
        eco.mate_floor_r1();
        let (r4, boosted) = eco.mate_radius_with_floor(d, 0, 19.0);
        assert!(boosted);
        assert!((r4 - 300.0).abs() < 1e-9, "4 members -> the reference floor");
        many(&mut eco, d, 4, 0);
        let (r8, _) = eco.mate_radius_with_floor(d, 0, 19.0);
        assert!((r8 - 300.0 * 0.5f64.sqrt()).abs() < 1e-9, "8 members -> half-density floor");
        assert!(r8 < r4, "denser deme searches closer");
    }

    #[test]
    fn the_floor_caps_and_never_shrinks_the_legacy_radius() {
        let mut eco = Ecology::new(20261123);
        let d = daughter(&mut eco);
        many(&mut eco, d, 1, 0);
        eco.mate_floor_r1();
        assert_eq!(eco.mate_radius_with_floor(d, 0, 19.0).0, MATE_FLOOR_R1_MAX);
        eco.mate_floor_r2();
        assert_eq!(eco.mate_radius_with_floor(d, 0, 19.0).0, MATE_FLOOR_R2_MAX);
        // Nine members: R2's floor is 280, so a wider legacy radius wins.
        many(&mut eco, d, 8, 0);
        let (r, boosted) = eco.mate_radius_with_floor(d, 0, 400.0);
        assert_eq!(r, 400.0);
        assert!(!boosted);
    }

    #[test]
    fn roots_and_fish_keep_the_legacy_radius() {
        let mut eco = Ecology::new(20261123);
        eco.mate_floor_r2();
        assert_eq!(eco.mate_radius_with_floor(DECO, 0, 19.0), (19.0, false));
        assert_eq!(eco.mate_radius_with_floor(FISH, 0, 60.0), (60.0, false));
    }

    /// A daughter pair 60 units apart: outside the legacy Allee radius (the
    /// density factor gives ~37 here) but inside R1's floor. The arm must
    /// convert a miss into a birth; the OFF arm must not.
    fn pair_world(arm: u8) -> (Ecology, usize) {
        let mut eco = Ecology::new(20261128);
        eco.deco_region_cap_off();
        eco.dispersal_p = 0.0;
        eco.gene_barrier_strength = 0.0;
        let d = daughter(&mut eco);
        // Keep the fixture density at 2/80 so the DECO selective threshold stays
        // below 1.0 (the arm must not be what makes the parents fertile).
        let mut def = eco.kind_def(DECO).clone();
        def.name = "deco-mate";
        def.parent_kind = Some(DECO);
        def.global_cap = 80;
        eco.kinds.defs[d as usize] = def;
        let i = plant(&mut eco, d, 0.0, 0.0, 0, 1.0);
        let j = plant(&mut eco, d, 60.0, 0.0, 0, 1.0);
        for k in [i, j] {
            eco.agents[k].genome.deco_efficiency = 1.0;
            eco.agents[k].genome.hue = 0.5;
            eco.agents[k].energy = eco.agents[k].max_energy;
            eco.agents[k].age = 100.0;
            eco.agents[k].repro = 0.0;
        }
        // Below the asexual fallback's age gate (45): a failed mate search must
        // not be masked by a solo birth.
        eco.agents[i].age = 40.0;
        if arm == 1 { eco.mate_floor_r1(); }
        (eco, i)
    }

    #[test]
    fn extended_search_converts_a_miss_into_a_birth() {
        let (mut off, i0) = pair_world(0);
        off.reproduce(i0, 0.1);
        assert_eq!(off.births, 0, "60 > the legacy radius: no mate");
        assert_eq!(off.radius_only_finds(), 0);

        let (mut on, i1) = pair_world(1);
        on.reproduce(i1, 0.1);
        assert_eq!(on.births, 1, "R1 reaches the partner at 60");
        assert_eq!(on.radius_only_finds(), 1);
        assert_eq!(on.radius_boost_searches(), 1);
    }

    #[test]
    fn off_region_deaths_and_dispersals_are_booked_per_clade() {
        let mut eco = Ecology::new(20261123);
        let d = daughter(&mut eco);
        // Position in region 2 (positive x) but tagged as region 1.
        let i = plant(&mut eco, d, 300.0, 0.0, 1, 1.0);
        assert_ne!(Ecology::region_of(eco.agents[i].x, eco.agents[i].z), eco.agents[i].region);
        eco.clade_life_dispersal(0, d);
        eco.die(i, "aged");
        let s = eco.clade_life_stats().iter().find(|s| s.clade_id == 0 && s.root_kind == DECO).unwrap();
        assert_eq!(s.dispersal_out, 1);
        assert_eq!(s.deaths_off_region, 1);
    }
}

#[cfg(test)]
mod phase67_small_floor_tests {
    use super::*;

    /// A DECO daughter with a controlled budget, minted the way a fork mints
    /// one, so the public birth gate accepts the fixture members.
    fn daughter(eco: &mut Ecology) -> SpeciesKind {
        eco.deco_region_cap_off();
        let mut def = eco.kind_def(DECO).clone();
        def.name = "deco-neighbor";
        def.parent_kind = Some(DECO);
        def.global_cap = 20;
        let d = eco.register_kind(def);
        eco.set_kind_budget(d, 20);
        d
    }

    /// Spawn one live member and pin the fields the mate search reads.
    fn plant(eco: &mut Ecology, d: SpeciesKind, x: f64, z: f64, region: u8, energy_frac: f64) -> usize {
        let id = eco.spawn_agent(d, x, z).expect("spawn");
        let i = eco.agents.iter().position(|a| a.id == id).unwrap();
        let a = &mut eco.agents[i];
        a.region = region;
        a.age = 100.0;
        a.repro = 0.0;
        a.energy = a.max_energy * energy_frac;
        i
    }

    #[test]
    fn r3_floor_spans_the_80_to_120_band() {
        let mut eco = Ecology::new(20261201);
        let d = daughter(&mut eco);
        eco.mate_floor_r3();
        // n = 1: 80 * sqrt(4) = 160, capped at 120. (Region 0 has no member
        // yet; the formula clamps the empty deme to one.)
        let (r1, b1) = eco.mate_radius_with_floor(d, 0, 19.0);
        assert!(b1);
        assert!((r1 - 120.0).abs() < 1e-9, "one member -> the 120 cap");
        plant(&mut eco, d, 3.0, 0.0, 0, 1.0);
        assert_eq!(eco.mate_radius_with_floor(d, 0, 19.0).0, 120.0);
        // n = 2: 113.14; n = 3: 92.38; both inside the band and above legacy.
        plant(&mut eco, d, 6.0, 0.0, 0, 1.0);
        let (r2, _) = eco.mate_radius_with_floor(d, 0, 19.0);
        assert!((r2 - 80.0 * 2.0f64.sqrt()).abs() < 1e-9, "two members -> 113.14");
        assert!(r2 > 80.0 && r2 < 120.0);
        plant(&mut eco, d, 9.0, 0.0, 0, 1.0);
        let (r3, _) = eco.mate_radius_with_floor(d, 0, 19.0);
        assert!((r3 - 80.0 * (4.0f64 / 3.0).sqrt()).abs() < 1e-9, "three members -> 92.38");
        // n = 4: 80 exactly; n >= 5: 80 by the floor, never below.
        plant(&mut eco, d, 12.0, 0.0, 0, 1.0);
        let (r4, _) = eco.mate_radius_with_floor(d, 0, 19.0);
        assert!((r4 - 80.0).abs() < 1e-9, "four members -> the 80 floor");
        for k in 5..=12 {
            plant(&mut eco, d, k as f64 * 3.0, 0.0, 0, 1.0);
        }
        let (r12, _) = eco.mate_radius_with_floor(d, 0, 19.0);
        assert!((r12 - 80.0).abs() < 1e-9, "twelve members -> still the 80 floor");
    }

    #[test]
    fn r3_never_exceeds_120_or_shrinks_a_wider_legacy_radius() {
        let mut eco = Ecology::new(20261201);
        let d = daughter(&mut eco);
        eco.mate_floor_r3();
        assert_eq!(eco.mate_radius_with_floor(d, 0, 44.0), (120.0, true));
        assert_eq!(eco.mate_radius_with_floor(d, 0, 150.0), (150.0, false),
            "a legacy bound wider than the cap wins and is not a boost");
    }

    #[test]
    fn r3_excludes_roots_and_fish() {
        let mut eco = Ecology::new(20261201);
        eco.mate_floor_r3();
        assert_eq!(eco.mate_radius_with_floor(DECO, 0, 19.0), (19.0, false),
            "the root guild is the differentiation engine and stays legacy");
        assert_eq!(eco.mate_radius_with_floor(FISH, 0, 60.0), (60.0, false));
    }

    /// A daughter pair `dist` apart: outside the legacy Allee radius (~43 for
    /// two members) but inside / outside R3's 80-120 band.
    fn pair_world(seed: u32, arm: u8, dist: f64) -> (Ecology, usize) {
        let mut eco = Ecology::new(seed);
        eco.deco_region_cap_off();
        eco.dispersal_p = 0.0;
        eco.gene_barrier_strength = 0.0;
        let d = daughter(&mut eco);
        let mut def = eco.kind_def(DECO).clone();
        def.name = "deco-neighbor";
        def.parent_kind = Some(DECO);
        def.global_cap = 80;
        eco.kinds.defs[d as usize] = def;
        let i = plant(&mut eco, d, 0.0, 0.0, 0, 1.0);
        let j = plant(&mut eco, d, dist, 0.0, 0, 1.0);
        for k in [i, j] {
            eco.agents[k].genome.deco_efficiency = 1.0;
            eco.agents[k].genome.hue = 0.5;
            eco.agents[k].energy = eco.agents[k].max_energy;
            eco.agents[k].age = 100.0;
            eco.agents[k].repro = 0.0;
        }
        // Below the asexual fallback's age gate (45): a failed mate search must
        // not be masked by a solo birth.
        eco.agents[i].age = 40.0;
        if arm == 3 { eco.mate_floor_r3(); }
        (eco, i)
    }

    #[test]
    fn r3_rescues_a_near_partner_that_legacy_cannot_reach() {
        let (mut off, i0) = pair_world(20261201, 0, 100.0);
        off.reproduce(i0, 0.1);
        assert_eq!(off.births, 0, "100 > the legacy radius: no mate");
        assert_eq!(off.radius_only_finds(), 0);

        let (mut on, i1) = pair_world(20261201, 3, 100.0);
        on.reproduce(i1, 0.1);
        assert_eq!(on.births, 1, "R3 reaches the partner at 100 inside the 120 cap");
        assert_eq!(on.radius_only_finds(), 1);
        assert_eq!(on.radius_boost_searches(), 1);
        assert!((on.radius_only_find_dist_mean() - 100.0).abs() < 1e-9,
            "the rescue distance never exceeds the cap");
    }

    #[test]
    fn r3_does_not_reach_the_far_tail() {
        let (mut on, i) = pair_world(20261201, 3, 200.0);
        on.reproduce(i, 0.1);
        assert_eq!(on.births, 0, "200 > the 120 cap: the long tail stays out");
        assert_eq!(on.radius_only_finds(), 0);
        assert_eq!(on.radius_boost_searches(), 1, "the search itself was widened");
    }

    #[test]
    fn p3_normalises_misses_by_clade_lifetime() {
        let mut s = CladeLifeStats::default();
        s.first_tick = 1_000;
        s.extinction_tick = Some(11_000);
        s.mate_misses = 500;
        s.mate_miss_beyond_radius = 300;
        s.mate_miss_filtered = 200;
        assert_eq!(s.life_ticks(200_000), 10_000);
        assert!((s.mate_miss_rate_per_tick(200_000) - 0.05).abs() < 1e-12);
        assert!((s.mate_miss_bucket_rate_per_tick(MateMissBucket::BeyondRadius, 200_000) - 0.03).abs() < 1e-12);
        assert!((s.mate_miss_bucket_rate_per_tick(MateMissBucket::Filtered, 200_000) - 0.02).abs() < 1e-12);
        // A live clade runs to the end of the window.
        s.extinction_tick = None;
        assert_eq!(s.life_ticks(200_000), 199_000);
        // An empty lifetime yields 0, never NaN/inf.
        let empty = CladeLifeStats::default();
        assert_eq!(empty.life_ticks(0), 0);
        assert_eq!(empty.mate_miss_rate_per_tick(0), 0.0);
        assert_eq!(empty.mate_miss_bucket_rate_per_tick(MateMissBucket::NoKin, 0), 0.0);
    }

    #[test]
    fn r3_is_off_by_default() {
        let eco = Ecology::new(20261201);
        assert_eq!(eco.mate_radius_arm(), 0);
        assert_eq!(eco.radius_only_finds(), 0);
    }
}
