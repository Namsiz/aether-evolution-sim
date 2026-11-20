//! Phase 60 — regional DECO carrying capacity (the split trigger that does not
//! ride on sterile individuals).
//!
//! Phase 59's fertility fix removed the old engine's hidden driver: the guild
//! used to sit below its 80-slot cap only because sterile low-efficiency
//! holders occupied slots without breeding. This arm spreads each DECO kind's
//! Phase-49 budget over the four home regions by local carcass-food retention
//! (`deco_env` mean), so a culture that speeds decomposition hollows out a
//! region's capacity instead of sterilising an individual.
//!
//! Pinned here: the weight partition (uniform field → even split; a field
//! imprint moves the weights), the slot gate (blocks at the regional share,
//! disappears with the arm off), the arm never being persisted, the OFF arm
//! replaying the Phase 59 default anchor, and the ON arm's double-run +
//! save/restore bit-exactness (the weight cache is re-derived from the restored
//! field, never saved).

use aether_sim_ecology::ecology::{Ecology, DECO, GRID_N, GRID_SPAN};

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

fn cell_center(j: usize) -> (f64, f64) {
    let cell = GRID_SPAN / GRID_N as f64;
    let col = (j % GRID_N as usize) as f64;
    let row = (j / GRID_N as usize) as f64;
    (
        -GRID_SPAN / 2.0 + (col + 0.5) * cell,
        -GRID_SPAN / 2.0 + (row + 0.5) * cell,
    )
}

/// A habitable cell whose whole `margin`-cell neighbourhood lies in `region`,
/// so a decomposition dipole there imprints only that region.
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

fn region_members(eco: &Ecology, region: u8) -> usize {
    eco.living().iter()
        .filter(|a| a.alive && a.kind == DECO && a.region == region)
        .count()
}

#[test]
fn uniform_field_partitions_the_kind_budget_evenly() {
    let eco = Ecology::new(20261102);
    assert_eq!(eco.deco_region_weights(), [1.0; 4],
        "a fresh field must start with the uniform weights");
    let total: f64 = (0..4).map(|r| eco.deco_region_cap_for(DECO, r)).sum();
    let budget = eco.kind_budget(DECO) as f64;
    assert!((total - budget).abs() < 1e-9,
        "regional caps must sum to the kind budget ({total} vs {budget})");
    for r in 0..4 {
        assert!((eco.deco_region_cap_for(DECO, r) - budget / 4.0).abs() < 1e-9,
            "region {r} must start at a quarter of the budget");
    }
}

#[test]
fn regional_weights_follow_the_decay_field() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_on();
    let (x, z) = interior_land_point(&eco, 0, 5);
    // Depletion token (negative grass_mod): its disk rots carcasses faster,
    // so the region retains less food and must lose capacity share.
    eco.deco_env_dipole(x, z, -0.05);
    // Any DECO spawn gate refreshes the cache from the current field; the
    // spawn itself may be refused, the refresh happens first.
    eco.spawn_agent(DECO, x, z);
    let w = eco.deco_region_weights();
    assert!(w.iter().all(|v| v.is_finite() && *v > 0.0), "weights: {w:?}");
    assert!(w[0] < 1.0,
        "the depleted region's weight must fall below 1.0, got {w:?}");
    let total: f64 = (0..4).map(|r| eco.deco_region_cap_for(DECO, r)).sum();
    assert!((total - eco.kind_budget(DECO) as f64).abs() < 1e-9,
        "the weights must still partition the budget exactly");
}

#[test]
fn the_slot_gate_binds_only_at_the_regional_share() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_on();
    let (x, z) = interior_land_point(&eco, 0, 5);
    let mut spawned = 0usize;
    while eco.spawn_agent(DECO, x, z).is_some() {
        spawned += 1;
        assert!(spawned < 200, "the slot gate never bound");
    }
    assert_eq!(region_members(&eco, 0), 20,
        "region 0 must hold exactly its uniform 20-slot share");
    assert!(spawned > 0, "the probe never spawned a DECO");
    // Off arm: the gate is gone, the guild budget (80) is the only cap.
    eco.deco_region_cap_off();
    assert!(eco.spawn_agent(DECO, x, z).is_some(),
        "with the arm off a regional share must not block a birth");
}

#[test]
fn the_arm_is_never_persisted() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_off();
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let restored = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    assert_eq!(restored.deco_region_cap,
        aether_sim_ecology::ecology::DECO_REGION_CAP_ON,
        "a restored world runs the compiled default, never the arm");
    assert_eq!(restored.deco_region_weights(), [1.0; 4],
        "restored weights start at the uniform cache and are refreshed from \
         the restored field at the first birth gate");
}

#[test]
#[ignore = "60k-tick arm in release — minutes; OFF arm must replay the Phase 59 default"]
fn off_arm_replays_the_phase59_default_anchor() {
    let mut eco = Ecology::new(20261102);
    eco.deco_region_cap_off();
    run_ticks(&mut eco, 60_000);
    assert_eq!(eco.checksum(), "0xde2c6292e0dfb9dc",
        "Phase 60 OFF: the Phase 59 default world at t=60000 must be bit-exact");
}

#[test]
#[ignore = "two 60k-tick arms in release — minutes; ON determinism + save/restore"]
fn on_arm_double_run_and_roundtrip_is_bit_exact() {
    let mut a = Ecology::new(20261102);
    let mut b = Ecology::new(20261102);
    run_ticks(&mut a, 60_000);
    run_ticks(&mut b, 60_000);
    assert_eq!(a.checksum(), b.checksum(), "ON-arm double run diverged");
    let json = serde_json::to_string(&a.to_save_state()).unwrap();
    let mut c = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    for t in 60_001..=61_200u32 {
        a.update(0.1, daylight(t));
        c.update(0.1, daylight(t));
    }
    assert_eq!(a.checksum(), c.checksum(),
        "ON-arm restored continuation diverged (weight cache not re-derived?)");
}

#[test]
fn the_fertility_map_is_untouched_by_the_capacity_arm() {
    // The decoupling contract: capacity is a slot instrument, never a second
    // fertility gate. The Phase 59 mapping must read the same in both arms.
    let mut eco = Ecology::new(20261102);
    let before: Vec<f64> = (0..=1000)
        .map(|i| eco.deco_repro_threshold(0.5 + i as f64 * 0.001))
        .collect();
    eco.deco_region_cap_on();
    let after: Vec<f64> = (0..=1000)
        .map(|i| eco.deco_repro_threshold(0.5 + i as f64 * 0.001))
        .collect();
    assert_eq!(before, after, "the capacity arm moved the fertility mapping");
    assert!(after.iter().all(|t| *t < 1.0),
        "some legal efficiency became sterile");
}
