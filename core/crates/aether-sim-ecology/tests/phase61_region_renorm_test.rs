//! Phase 61 — occupied-region renormalisation of the regional DECO share.
//!
//! Phase 60 spread each DECO kind's Phase-49 budget over all four home
//! quadrants, so a clade that lives in one quadrant could hold at most a
//! quarter of its budget there. A daughter forks off with ≥ 8 members; if that
//! cluster lands in one region it is born at (or above) the quarter share and
//! can never breed again — it dies by attrition while fully fertile.
//!
//! This arm renormalises the share over the regions a kind actually occupies,
//! so a one-region clade's cap is its whole budget. Pinned here: the formula
//! in both arms, the conservation identity over occupied regions, the slot
//! gate, the telemetry contract (counters never touch the trajectory and are
//! never persisted) and the pre-61 anchor replay.

use aether_sim_ecology::ecology::{
    Ecology, SpeciesKind, DECO, DECO_REGION_CAP_RENORM_ON, GRID_N, GRID_SPAN,
};

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, to: u32) {
    for t in 1..=to {
        eco.update(0.1, daylight(t));
    }
}

fn region_of(x: f64, z: f64) -> u8 {
    let two_pi = std::f64::consts::PI * 2.0;
    let a = aether_libm::atan2(z, x) + std::f64::consts::PI;
    let sector = (a / two_pi * 4.0).floor() as i32;
    sector.clamp(0, 3) as u8
}

/// A habitable cell whose whole `margin`-cell neighbourhood lies in `region`.
fn interior_land_point(eco: &Ecology, region: u8, margin: i32) -> (f64, f64) {
    let gn = GRID_N as i32;
    let cell = GRID_SPAN / GRID_N as f64;
    for row in margin..gn - margin {
        for col in margin..gn - margin {
            let x = -GRID_SPAN / 2.0 + (col as f64 + 0.5) * cell;
            let z = -GRID_SPAN / 2.0 + (row as f64 + 0.5) * cell;
            if region_of(x, z) != region { continue; }
            let mut inside = true;
            for dr in -margin..=margin {
                for dc in -margin..=margin {
                    let nx = -GRID_SPAN / 2.0 + (col + dc) as f64 * cell + 0.5 * cell;
                    let nz = -GRID_SPAN / 2.0 + (row + dr) as f64 * cell + 0.5 * cell;
                    if region_of(nx, nz) != region { inside = false; break; }
                }
                if !inside { break; }
            }
            if !inside { continue; }
            let j = row as usize * GRID_N as usize + col as usize;
            if eco.suit_field()[j] > 0.5 { return (x, z); }
        }
    }
    panic!("no interior habitable cell in region {region}");
}

/// A DECO-family kind with a controlled budget, registered exactly the way a
/// fork mints one (`parent_kind = DECO`, constants copied), so the arm can be
/// probed without touching the root guild.
fn deco_variant(eco: &mut Ecology, name: &'static str, budget: usize) -> SpeciesKind {
    let mut def = eco.kind_def(DECO).clone();
    def.name = name;
    def.parent_kind = Some(DECO);
    def.global_cap = budget;
    eco.register_kind(def)
}

fn members_in(eco: &Ecology, kind: SpeciesKind, region: u8) -> usize {
    eco.living().iter()
        .filter(|a| a.alive && a.kind == kind && a.region == region)
        .count()
}

#[test]
fn a_one_region_clade_may_use_its_whole_budget() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_on();
    let kind = deco_variant(&mut eco, "deco-renorm-one", 16);
    let (x0, z0) = interior_land_point(&eco, 0, 5);

    // Phase 60 arm: a single occupied quadrant is capped at budget / 4.
    eco.deco_region_cap_renorm_off();
    assert!((eco.deco_region_cap_for(kind, 0) - 4.0).abs() < 1e-9,
        "the phase-60 quarter share must be 16 / 4 = 4");
    let mut spawned = 0usize;
    while eco.spawn_agent(kind, x0, z0).is_some() {
        spawned += 1;
        assert!(spawned < 100, "the quarter share never bound");
    }
    assert_eq!(members_in(&eco, kind, 0), 4,
        "the phase-60 arm must stop at the quarter share");

    // Phase 61 arm: the occupied set is {0}, so the cap is the whole budget.
    eco.deco_region_cap_renorm_on();
    assert!((eco.deco_region_cap_for(kind, 0) - 16.0).abs() < 1e-9,
        "renormalised onto one region the cap must be the whole budget");
    let mut extra = 0usize;
    while eco.spawn_agent(kind, x0, z0).is_some() {
        extra += 1;
        assert!(extra < 100, "the renormalised share never bound");
    }
    assert_eq!(members_in(&eco, kind, 0), 16,
        "the renormalised arm must fill the whole budget in one region");
}

#[test]
fn occupied_regions_still_conserve_the_kind_budget() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_on();
    eco.deco_region_cap_renorm_on();
    let kind = deco_variant(&mut eco, "deco-renorm-two", 16);
    let (x0, z0) = interior_land_point(&eco, 0, 5);
    let (x1, z1) = interior_land_point(&eco, 1, 5);
    for _ in 0..4 {
        assert!(eco.spawn_agent(kind, x0, z0).is_some());
        assert!(eco.spawn_agent(kind, x1, z1).is_some());
    }
    assert_eq!(members_in(&eco, kind, 0), 4);
    assert_eq!(members_in(&eco, kind, 1), 4);
    // Two occupied regions share the budget; the empty ones are not counted.
    let caps: f64 = (0..4).filter(|&r| r == 0 || r == 1)
        .map(|r| eco.deco_region_cap_for(kind, r))
        .sum();
    assert!((caps - eco.kind_budget(kind) as f64).abs() < 1e-9,
        "the occupied regions' caps must sum to the kind budget ({caps})");
    for r in [2u8, 3] {
        assert!(eco.deco_region_cap_for(kind, r) > 0.0,
            "a vacant quadrant keeps a colonisation share");
    }
}

#[test]
fn the_arm_is_never_persisted_and_telemetry_starts_empty() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_renorm_on();
    run_ticks(&mut eco, 200);
    assert!(!eco.clade_life_stats().is_empty(),
        "the telemetry must book at least the period's deaths");
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    assert!(!json.contains("clade_life"),
        "the runtime telemetry must never enter the save state");
    let restored = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    assert_eq!(restored.deco_region_cap_renorm, DECO_REGION_CAP_RENORM_ON,
        "a restored world runs the compiled default, never the arm");
    assert!(restored.clade_life_stats().is_empty(),
        "the telemetry is rebuilt from live activity, never restored");
}

#[test]
fn telemetry_books_every_death_exactly_once() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 600);
    let total: u64 = eco.clade_life_stats().iter().map(|s| s.deaths).sum();
    assert_eq!(total, eco.deaths as u64,
        "the per-clade death counters must add up to the engine's death total");
    for s in eco.clade_life_stats() {
        assert_eq!(s.deaths, s.deaths_starved + s.deaths_aged + s.deaths_predation + s.deaths_other,
            "the death causes must partition the total");
        assert!(s.births + s.deaths + s.region_cap_blocks + s.kind_budget_blocks + s.mate_misses > 0
            || s.peak_pop > 0,
            "a booked clade must have booked an event");
    }
}

#[test]
#[ignore = "60k-tick arm in release — minutes; with the arm off the Phase 60 default must replay"]
fn renorm_off_replays_the_phase60_default_anchor() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_renorm_off();
    run_ticks(&mut eco, 60_000);
    assert_eq!(eco.checksum(), "0x3272f0e3eaba9250",
        "Phase 61 OFF: the Phase 60 default world at t=60000 must be bit-exact \
         (telemetry must not touch the trajectory)");
}

#[test]
#[ignore = "two 60k-tick arms in release — minutes; ON determinism + save/restore"]
fn renorm_on_double_run_and_roundtrip_is_bit_exact() {
    let mut a = Ecology::new(20261102);
    let mut b = Ecology::new(20261102);
    a.deco_region_cap_renorm_on();
    b.deco_region_cap_renorm_on();
    run_ticks(&mut a, 60_000);
    run_ticks(&mut b, 60_000);
    assert_eq!(a.checksum(), b.checksum(), "ON-arm double run diverged");
    let json = serde_json::to_string(&a.to_save_state()).unwrap();
    let mut c = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    c.deco_region_cap_renorm_on();
    for t in 60_001..=61_200u32 {
        a.update(0.1, daylight(t));
        c.update(0.1, daylight(t));
    }
    assert_eq!(a.checksum(), c.checksum(),
        "ON-arm restored continuation diverged");
}
#[test]
fn the_floor_keeps_an_occupied_region_at_the_split_minimum() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_on();
    let kind = deco_variant(&mut eco, "deco-floor-eight", 8);
    let (x0, z0) = interior_land_point(&eco, 0, 5);
    let (x1, z1) = interior_land_point(&eco, 1, 5);
    // Phase 60: an 8-budget kind gets 8 / 4 = 2 slots in the quadrant.
    eco.deco_region_cap_floor_off();
    assert!(eco.spawn_agent(kind, x0, z0).is_some());
    assert!(eco.spawn_agent(kind, x0, z0).is_some());
    assert_eq!(eco.deco_region_cap_for(kind, 0), 2.0,
        "the plain share of an 8-budget kind is 2 slots per quadrant");
    // Candidate (b): an OCCUPIED quadrant must be able to hold the split
    // gate's minimum side (8) — but never more than the kind's own budget.
    eco.deco_region_cap_floor_on();
    assert_eq!(eco.deco_region_cap_for(kind, 0), 8.0,
        "the floor must lift the occupied quadrant to min(budget, min_side)");
    let mut extra = 0usize;
    while eco.spawn_agent(kind, x0, z0).is_some() {
        extra += 1;
        assert!(extra < 100, "the floor never bound");
    }
    assert_eq!(members_in(&eco, kind, 0), 8,
        "the occupied quadrant must hold the kind's whole budget, not more");
    // A vacant quadrant keeps the plain Phase 60 share, so colonisation and
    // the root's four-deme structure are untouched. (The kind itself sits at
    // its 8-slot budget by now, so the global budget gate — not the quadrant
    // — is what would refuse a migrant.)
    assert_eq!(eco.deco_region_cap_for(kind, 1), 2.0,
        "a vacant quadrant must keep the plain share");
}

#[test]
fn the_floor_arm_is_never_persisted() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_floor_on();
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let restored = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    assert_eq!(restored.deco_region_cap_floor,
        aether_sim_ecology::ecology::DECO_REGION_CAP_FLOOR_ON,
        "a restored world runs the compiled default, never the arm");
}

#[test]
fn daughter_only_renorm_leaves_the_root_partition_alone() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_on();
    let kind = deco_variant(&mut eco, "deco-c-var", 8);
    let (x0, z0) = interior_land_point(&eco, 0, 5);
    eco.deco_region_cap_renorm_daughters_on();
    assert_eq!(members_in(&eco, kind, 0), 0);
    assert!(eco.spawn_agent(kind, x0, z0).is_some());
    // Derived kind, one occupied quadrant: its whole budget fits there.
    assert_eq!(eco.deco_region_cap_for(kind, 0), 8.0,
        "a derived kind must be renormalised onto the quadrants it occupies");
    // The root DECO keeps the Phase 60 four-quadrant split (the speciation
    // engine's deme partition).
    let budget = eco.kind_budget(DECO) as f64;
    assert_eq!(eco.deco_region_cap_for(DECO, 0), budget / 4.0,
        "the root kind must keep the plain Phase 60 share");
}
