//! Phase 31 KindDef registry acceptance tests.
//!
//! 1. `registry_matches_legacy_match_arms` — every constant that moved from a
//!    `match kind` arm into `KindDef` data is asserted field-by-field against
//!    the values the old code branches carried (audited from the pre-Phase-31
//!    `match` sites: spawn/reproduce/die/crowding/mutate/biome_max_for).
//! 2. `ghost_kind_registration_is_inert` — registering a seventh kind must not
//!    panic (former `unreachable!()` sites now resolve through the registry),
//!    must not perturb the checksum, and must leave the simulation trajectory
//!    bit-identical to an unregistered twin (registry growth is inert).

use aether_sim_ecology::ecology::{
    BehaviorTemplate, BiomeCap, Diet, Ecology, GenomeTemplate, KindDef, KindRegistry, DECO, FISH,
    HERB, OMN, PRED, SCAV, SpeciesKind,
};

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn assert_def(eco: &Ecology, id: SpeciesKind, name: &str, cap: usize, max_e: f64, cd: f64,
              range: f64, mature: f64, thr: f64, pair: (f64, f64), solo: f64,
              se: (f64, f64), clamp: (f64, f64), biome: BiomeCap, behavior: BehaviorTemplate,
              genome: GenomeTemplate, fin: bool, deco: bool) {
    let d = eco.kind_def(id);
    assert_eq!(d.id, id, "{name}: id");
    assert_eq!(d.name, name, "{name}: name");
    assert_eq!(d.global_cap, cap, "{name}: global_cap");
    assert_eq!(d.max_energy, max_e, "{name}: max_energy");
    assert_eq!(d.repro_cooldown, cd, "{name}: repro_cooldown");
    assert_eq!(d.range_base, range, "{name}: range_base");
    assert_eq!(d.mature_age, mature, "{name}: mature_age");
    assert_eq!(d.repro_base_threshold, thr, "{name}: repro_base_threshold");
    assert_eq!(d.repro_drain_pair, pair, "{name}: repro_drain_pair");
    assert_eq!(d.repro_drain_solo, solo, "{name}: repro_drain_solo");
    assert_eq!(d.start_energy, se, "{name}: start_energy");
    assert_eq!(d.speed_clamp, clamp, "{name}: speed_clamp");
    assert_eq!(d.biome_cap, biome, "{name}: biome_cap");
    assert_eq!(d.behavior, behavior, "{name}: behavior");
    assert_eq!(d.genome_template, genome, "{name}: genome_template");
    assert_eq!(d.mutates_fin_color, fin, "{name}: mutates_fin_color");
    assert_eq!(d.mutates_deco_eff, deco, "{name}: mutates_deco_eff");
}

#[test]
fn registry_matches_legacy_match_arms() {
    let eco = Ecology::new(20261102);
    assert_def(&eco, HERB, "herb", 420, 150.0, 32.0, 170.0, 30.0, 0.86,
        (38.0, 30.0), 52.0, (78.0, 25.0), (2.0, 8.4),
        BiomeCap::Herb, BehaviorTemplate::Herb, GenomeTemplate::Herb, false, false);
    assert_def(&eco, PRED, "pred", 110, 190.0, 76.0, 280.0, 56.0, 0.96,
        (58.0, 48.0), 76.0, (92.0, 30.0), (3.4, 8.6),
        BiomeCap::Pred, BehaviorTemplate::Pred, GenomeTemplate::General, false, false);
    assert_def(&eco, SCAV, "scav", 40, 120.0, 40.0, 200.0, 56.0, 0.96,
        (58.0, 48.0), 76.0, (55.0, 20.0), (3.4, 8.6),
        BiomeCap::Scav, BehaviorTemplate::Scav, GenomeTemplate::General, false, false);
    assert_def(&eco, OMN, "omn", 80, 135.0, 38.0, 185.0, 56.0, 0.96,
        (58.0, 48.0), 76.0, (65.0, 22.0), (3.4, 8.6),
        BiomeCap::Omn, BehaviorTemplate::Omn, GenomeTemplate::General, false, false);
    assert_def(&eco, FISH, "fish", 60, 110.0, 45.0, 120.0, 25.0, 0.65,
        (20.0, 16.0), 46.0, (60.0, 18.0), (1.8, 6.8),
        BiomeCap::Fish, BehaviorTemplate::Fish, GenomeTemplate::Fish, true, false);
    assert_def(&eco, DECO, "deco", 80, 100.0, 50.0, 150.0, 30.0, 0.96,
        (58.0, 48.0), 45.0, (50.0, 20.0), (3.4, 8.6),
        BiomeCap::Fallback, BehaviorTemplate::Deco, GenomeTemplate::Deco, false, true);
    // Legacy ids keep their historical slot order (evolution/fitness/C-ABI
    // count export all rely on herb=0 … deco=5).
    assert_eq!(KindRegistry::legacy().len(), 6);
    assert_eq!(eco.kinds.defs().len(), 6);
}

#[test]
fn ghost_kind_registration_is_inert() {
    let ghost = KindDef {
        id: 0, // assigned by register_kind
        name: "ghost",
        behavior: BehaviorTemplate::Pred,
        genome_template: GenomeTemplate::General,
        biome_cap: BiomeCap::Fallback,
        global_cap: 64,
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
    };

    let mut with_ghost = Ecology::new(20261102);
    let before = with_ghost.checksum();
    let gid = with_ghost.register_kind(ghost);
    assert_eq!(gid, 6, "ghost kind appends after the legacy six");
    assert_eq!(with_ghost.kinds.defs().len(), 7);

    // Former `unreachable!()` sites must resolve through the registry instead
    // of panicking on a non-legacy kind.
    let _ = with_ghost.biome_max_for(gid, 100.0, 100.0);
    let _ = with_ghost.biome_max_for(gid, 99999.0, 99999.0);
    assert_eq!(with_ghost.count(gid), 0);
    assert_eq!(with_ghost.kind_name(gid), "ghost");

    // Registry growth alone must not perturb the checksum.
    assert_eq!(with_ghost.checksum(), before, "registration is inert");

    // And the trajectory stays bit-identical to an unregistered twin.
    let mut twin = Ecology::new(20261102);
    for t in 1..=50 {
        with_ghost.update(0.1, daylight(t));
        twin.update(0.1, daylight(t));
    }
    assert_eq!(with_ghost.checksum(), twin.checksum(),
        "a registered-but-unspawned kind must not alter the simulation");
}

#[test]
fn unknown_kind_id_resolves_conservatively() {
    // An id outside the registry (impossible via spawn; guards test/edge
    // paths) reads the PRED definition instead of panicking.
    let eco = Ecology::new(20261102);
    let d = eco.kind_def(999);
    assert_eq!(d.name, "pred");
    assert_eq!(d.behavior, BehaviorTemplate::Pred);
}
