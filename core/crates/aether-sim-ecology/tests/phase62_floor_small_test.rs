//! Phase 62 — the b arm (occupied-region floor) under 5 seeds, plus the
//! sensitivity arm d.
//!
//! Phase 61 measured candidate (b) on three seeds (extinctions 13 → 4,
//! speciation 22 → 20) and left the default off. Phase 62 re-ran it on five
//! pre-registered seeds (`research/phase62-report.md`): speciation did NOT fall
//! (31 → 33), and extinctions did not halve (17 → 15; the two new seeds flip
//! the sign), so the default stays off and this file pins the arm's contract:
//!
//! * `floor` lifts an OCCUPIED quadrant to `min(budget, speciation_min_side)`
//!   and never touches a vacant one (Phase 61 §4);
//! * arm d (`floor_small`) applies that lift only to kinds with a budget below
//!   `2 × speciation_min_side`, i.e. it leaves the 20-budget first-generation
//!   daughters on the plain Phase 60 share;
//! * both switches are arm-only: never persisted, a restored world runs the
//!   compiled default, and the b arm's trajectory is deterministic (double run
//!   + save/restore 60k, ignored — minutes in release).

use aether_sim_ecology::ecology::{
    Ecology, SpeciesKind, DECO, DECO_REGION_CAP_FLOOR_ON, DECO_REGION_CAP_FLOOR_SMALL_ON,
    GRID_N, GRID_SPAN, SPECIATION_MIN_SIDE_DEFAULT,
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

/// A DECO-family kind with a controlled budget, minted the way a fork does it.
fn deco_variant(eco: &mut Ecology, name: &'static str, budget: usize) -> SpeciesKind {
    let mut def = eco.kind_def(DECO).clone();
    def.name = name;
    def.parent_kind = Some(DECO);
    def.global_cap = budget;
    eco.register_kind(def)
}

#[test]
fn the_small_arm_spares_a_first_generation_daughter() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_on();
    // 20 = the first-generation grant; 8 = the deepest legal grandchild.
    let first_gen = deco_variant(&mut eco, "deco-d-first-gen", 20);
    let grandchild = deco_variant(&mut eco, "deco-d-grandchild", 8);
    let (x0, z0) = interior_land_point(&eco, 0, 5);
    eco.deco_region_cap_floor_small_on();
    assert!(eco.spawn_agent(first_gen, x0, z0).is_some());
    assert!(eco.spawn_agent(grandchild, x0, z0).is_some());
    assert_eq!(eco.deco_region_cap_for(first_gen, 0), 5.0,
        "arm d must leave a 20-budget kind on the plain Phase 60 quarter share");
    assert_eq!(eco.deco_region_cap_for(grandchild, 0), 8.0,
        "arm d still lifts a budget below 2 × min_side to the split gate");
    // The plain b arm lifts both: every derived kind has ≤ 20 budget, whose
    // quarter share (≤ 5) always sits below the 8-slot split minimum — which
    // is why b disables the regional gate for daughters.
    eco.deco_region_cap_floor_on();
    assert_eq!(eco.deco_region_cap_for(first_gen, 0), 8.0,
        "the plain floor lifts the first-generation daughter as well");
    assert_eq!(eco.deco_region_cap_for(grandchild, 0), 8.0);
    // A vacant quadrant keeps the plain share in both variants.
    assert_eq!(eco.deco_region_cap_for(first_gen, 1), 5.0);
    assert_eq!(eco.deco_region_cap_for(grandchild, 1), 2.0);
}

#[test]
fn arm_d_is_a_strict_subset_of_the_floor_arm() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_on();
    eco.deco_region_cap_floor_small_on();
    assert!(eco.deco_region_cap_floor,
        "arm d implies the floor arm (it is a restriction of it)");
    assert_eq!(2 * SPECIATION_MIN_SIDE_DEFAULT, 16,
        "the budget cut-off is 2 × the split gate");
    eco.deco_region_cap_floor_off();
    assert!(!eco.deco_region_cap_floor && !eco.deco_region_cap_floor_small);
}

#[test]
fn the_small_arm_is_never_persisted() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_floor_small_on();
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let restored = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    assert_eq!(restored.deco_region_cap_floor, DECO_REGION_CAP_FLOOR_ON);
    assert_eq!(restored.deco_region_cap_floor_small, DECO_REGION_CAP_FLOOR_SMALL_ON);
    assert!(!restored.deco_region_cap_floor && !restored.deco_region_cap_floor_small,
        "a restored world runs the compiled default, never the arm");
}

#[test]
#[ignore = "two 60k-tick arms in release — minutes; b-arm determinism + save/restore"]
fn floor_arm_double_run_and_roundtrip_is_bit_exact() {
    let mut a = Ecology::new(20261102);
    let mut b = Ecology::new(20261102);
    a.deco_region_cap_floor_on();
    b.deco_region_cap_floor_on();
    run_ticks(&mut a, 60_000);
    run_ticks(&mut b, 60_000);
    assert_eq!(a.checksum(), b.checksum(), "b-arm double run diverged");
    let json = serde_json::to_string(&a.to_save_state()).unwrap();
    let mut c = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    c.deco_region_cap_floor_on();
    for t in 60_001..=61_200u32 {
        a.update(0.1, daylight(t));
        c.update(0.1, daylight(t));
    }
    assert_eq!(a.checksum(), c.checksum(), "b-arm restored continuation diverged");
}
