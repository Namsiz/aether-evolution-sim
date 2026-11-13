//! Phase 33 — open plant registry + data-driven diets (封顶点 C5/C6 数据化).
//!
//! The hard red line: the default 3-plant world's trajectory and golden
//! regression anchors (0x790184996f3acefd @300, 0x58733db4ab0c3ca0 @3000)
//! are UNCHANGED. This file verifies that:
//!   1. Diet data precisely expresses the six legacy recipes
//!      (legacy recipe == data-table-driven diet, per kind).
//!   2. The PRED two-nearest prey-scan order is pinned by the data
//!      (`prey_kinds == [HERB, FISH]`, the legacy scan order).
//!   3. Registering a 4th plant (new era) grows, feeds, seeds cargo and is
//!      deterministic — while the default world stays bit-exact.
//!   4. FISH/DECO open-locus upkeep is a default-3 no-op (ext empty → ×1.0).
//!   5. SaveState v7 roundtrips the variable-length plant layer + cargo and
//!      backfills a v6 document to three legacy plants.
//!
//! Daylight convention matches the other ecology tests: tick t receives
//! `0.5 + 0.5*sin(t/300 * PI)` with t counted from 1.

use aether_sim_ecology::ecology::{
    Ecology, PlantType, Diet, HERB, PRED, SCAV, OMN, FISH, DECO, SpeciesKind,
};
use aether_sim_ecology::save_state;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in (from + 1)..=to {
        eco.update(0.1, daylight(t));
    }
}

/// A distinct 4th plant (mechanism-verification only — not a real species):
/// faster r_max, wetter moisture optimum, tastier, fruitier than any legacy
/// type. Registered kinds only ever ADD a layer, so a default-3 world never
/// touches this value.
fn fourth_plant() -> PlantType {
    PlantType { r_max: 1.8, moist_opt: 0.55, moist_w: 0.22, palatability: 1.6, fruiting: 0.20 }
}

/// The legacy recipes, transcribed verbatim from the former hardcoded
/// `update_*` "who-eats-whom" branches (C6). This is the source of truth the
/// diet data table must match exactly.
fn legacy_recipe(kind: SpeciesKind) -> (bool, bool, Vec<SpeciesKind>) {
    match kind {
        HERB => (true, false, vec![]),
        PRED => (false, true, vec![HERB, FISH]),
        SCAV => (true, true, vec![]),
        OMN => (true, true, vec![]),
        FISH => (true, false, vec![]),
        DECO => (false, true, vec![]),
        _ => panic!("legacy recipe only defined for the six core kinds"),
    }
}

#[test]
fn diet_data_matches_legacy_recipes() {
    let eco = Ecology::new(20261102);
    for kind in [HERB, PRED, SCAV, OMN, FISH, DECO] {
        let d = eco.diet_of(kind);
        let (lp, lc, prey) = legacy_recipe(kind);
        assert_eq!(d.eats_plant, lp, "{kind}: eats_plant drifted from legacy");
        assert_eq!(d.eats_carcass, lc, "{kind}: eats_carcass drifted from legacy");
        assert_eq!(d.prey_kinds, prey, "{kind}: prey_kinds drifted from legacy");
        // Legacy grass-eaters eat EVERY registered layer (no per-type
        // whitelist); non-grass-eaters list none.
        if lp {
            assert!(d.plant_types.is_empty(),
                "{kind}: legacy grass-eater must be layer-agnostic (empty whitelist)");
        } else {
            assert!(d.plant_types.is_empty(), "{kind}: must not whitelist plants");
        }
    }
}

#[test]
fn pred_prey_scan_order_is_pinned() {
    // The legacy PRED prey scan calls nearest(HERB) then nearest(FISH) with the
    // same distance rule + tie-break; the data order must mirror it so a future
    // matrix-scan re-baseline can switch without changing the default trajectory.
    let eco = Ecology::new(20261102);
    assert_eq!(eco.diet_of(PRED).prey_kinds, vec![HERB, FISH],
        "PRED prey_kinds order must equal the legacy two-nearest scan order");
}

#[test]
fn unknown_kind_diet_is_conservative_not_panic() {
    // The conservative diet (used for unknown / newly-registered kinds) =
    // eat carcass + plants, no live prey — never a panic.
    let d = Diet::conservative();
    assert!(d.eats_carcass && d.eats_plant && d.prey_kinds.is_empty(),
        "conservative diet = eat carcass + grass, no live prey");
    // An out-of-registry id resolves through the registry to the PRED def
    // (the Phase 31 contract) instead of panicking — its diet is still
    // well-defined.
    let eco = Ecology::new(20261102);
    let far = eco.diet_of(999); // out-of-range id, must not panic
    assert!(far.eats_carcass, "out-of-range id falls back to PRED's diet");
}

/// Register the 4th plant, run a fixed number of ticks, report the key data
/// points. Used to assert growth/feeding/cargo/determinism.
fn run_with_4th_plant(seed: u32) -> (String, f64, f64, bool, usize) {
    let mut eco = Ecology::new(seed);
    let idx = eco.register_plant(fourth_plant());
    run_ticks(&mut eco, 0, 400);
    let layer3_mass: f64 = eco.grass_p(3).iter().map(|&v| v as f64).sum();
    let cargo3_total: f64 = eco.living()
        .iter()
        .filter(|a| a.alive && a.cargo.len() > 3)
        .map(|a| a.cargo[3] as f64)
        .sum();
    let all_cargo_sized = eco.living()
        .iter()
        .filter(|a| a.alive)
        .all(|a| a.cargo.len() == 4);
    let plant_count = eco.plant_count();
    assert_eq!(idx, 3, "4th plant appends at index 3");
    (eco.checksum(), layer3_mass, cargo3_total, all_cargo_sized, plant_count)
}

#[test]
fn fourth_plant_grows_feeds_seeds_and_is_deterministic() {
    let a = run_with_4th_plant(20261102);
    let b = run_with_4th_plant(20261102);
    assert_eq!(a.0, b.0, "determinism: same seed + same registration → same checksum");
    assert_eq!(a.4, 4, "plant_count must be 4 after registration");
    assert!(a.1 > 0.0, "4th plant layer must grow to positive mass, got {}", a.1);
    assert!(a.3, "every live agent's cargo must be sized to the plant count (4)");
    assert!(a.2 > 0.0,
        "some agent must carry 4th-plant fruit seed in cargo[3] (got {})", a.2);
}

#[test]
fn default_3_plant_checksum_unchanged_by_registry() {
    // Registering nothing: the default world must reproduce the golden anchor
    // (guards that the Vec-based layers are bit-identical to the former arrays).
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 0, 300);
    assert_eq!(eco.plant_count(), 3);
    assert_eq!(eco.checksum(), "0x790184996f3acefd");
}

#[test]
fn fish_deco_upkeep_default_is_noop() {
    // Genesis OFF (default) → every agent's ext is empty → the FISH/DECO
    // upkeep factor is ×1.0 → the 300-tick golden anchor is untouched.
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 0, 300);
    assert!(eco.living().iter().filter(|a| a.alive).all(|a| a.genome.ext.is_empty()),
        "default genesis-OFF world must keep every ext list empty");
    assert_eq!(eco.checksum(), "0x790184996f3acefd");
}

#[test]
fn save_v7_default_roundtrip_bit_exact() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 0, 100);
    let state = eco.to_save_state();
    assert_eq!(state.version, save_state::SAVE_STATE_VERSION);
    assert_eq!(state.plants.len(), 3, "default world saves exactly 3 plants");
    assert_eq!(state.grass_by_species.len(), 3);
    assert_eq!(state.suit_eff_by_species.len(), 3);

    let mut eco2 = Ecology::from_save_state(state);
    assert_eq!(eco2.plant_count(), 3);
    run_ticks(&mut eco, 100, 300);
    run_ticks(&mut eco2, 100, 300);
    assert_eq!(eco.checksum(), eco2.checksum(), "default-3 save/restore must be lockstep");
}

#[test]
fn save_v7_variable_plant_roundtrip() {
    let mut eco = Ecology::new(20261102);
    eco.register_plant(fourth_plant());
    run_ticks(&mut eco, 0, 50);
    let state = eco.to_save_state();
    assert_eq!(state.plants.len(), 4, "4-plant world saves 4 plants");
    assert_eq!(state.grass_by_species.len(), 4);
    assert_eq!(state.taken_buf.len(), 4);

    let mut eco2 = Ecology::from_save_state(state);
    assert_eq!(eco2.plant_count(), 4, "restored world keeps the 4th plant");
    assert_eq!(eco2.grass_p(3).len(), eco.grass_p(3).len());
    run_ticks(&mut eco, 50, 120);
    run_ticks(&mut eco2, 50, 120);
    assert_eq!(eco.checksum(), eco2.checksum(),
        "variable-length plant layer + cargo must survive a save/restore");
}

#[test]
fn save_v6_backfills_three_legacy_plants() {
    // Simulate an older v6 document: the plant list is absent (serde default →
    // empty), the per-plant layers are the fixed 3, and cargo is 3. The loader
    // must backfill to the three legacy plants and continue bit-exactly.
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 0, 50);
    let mut state = eco.to_save_state();
    state.version = 6;
    state.plants = Vec::new(); // v6 had no plant list
    state.taken_buf = vec![0.0; 3]; // v6's fixed [f64; 3] buffer

    let mut eco2 = Ecology::from_save_state(state);
    assert_eq!(eco2.plant_count(), 3, "v6 load backfills to three legacy plants");
    assert!(eco2.living().iter().filter(|a| a.alive).all(|a| a.cargo.len() == 3),
        "v6 load must keep 3-slot cargo");
    run_ticks(&mut eco, 50, 150);
    run_ticks(&mut eco2, 50, 150);
    assert_eq!(eco.checksum(), eco2.checksum(),
        "a v6 backfilled world must continue bit-exactly");
}
