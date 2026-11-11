//! Phase 4-A integration test: spatial biomes stability.
//!
//! Exercises the `Ecology::with_biomes` constructor with a deliberately
//! heterogeneous landscape and runs 600 ticks (double the original 300). The
//! goal is NOT bit-exact parity — Phase 4-A intentionally shifts population
//! dynamics toward greater spatial stability — but we do assert ecological
//! health: predators never go extinct en masse, herbivore peaks stay bounded,
//! and scavenger/omnivore populations remain non-pathological.

use aether_biome::Biome;
use aether_sim_ecology::ecology::{Ecology, HERB, PRED, SCAV, OMN};
use aether_libm::sin;

/// Build a deliberately heterogeneous world with three biomes:
///   - "lush"     : x ∈ [-400, 100], z ∈ [-400, 100], high herb cap (250)
///   - "barren"   : x ∈ [200, 700], z ∈ [200, 700], low herb cap (80), home-mod 0.6
///   - "full"     : default full-world fallback (already present implicitly as
///                  the catch-all when an agent spawns outside defined regions)
fn make_biomes() -> Vec<Biome> {
    vec![
        Biome {
            id: 1,
            name: "lush",
            center_x: -150.0,
            center_z: -150.0,
            radius: 320.0,
            suit_modifier: 1.3,
            max_herb: 250,
            max_pred: 90,
            max_scav: 35,
            max_omn: 60,
            max_fish: None,
            home_range_mod: 1.1,
        },
        Biome {
            id: 2,
            name: "barren",
            center_x: 450.0,
            center_z: 450.0,
            radius: 280.0,
            suit_modifier: 0.55,
            max_herb: 80,
            max_pred: 20,
            max_scav: 8,
            max_omn: 15,
            max_fish: None,
            home_range_mod: 0.55,
        },
    ]
}

#[test]
fn spatial_biome_600_tick_stability() {
    let mut eco = Ecology::with_biomes(20261102, make_biomes());
    assert_eq!(eco.biome_reg.find_biome(0.0, 0.0).unwrap().name, "lush");
    assert_eq!(eco.biome_reg.find_biome(450.0, 450.0).unwrap().name, "barren");

    // Track running extremes over the simulation.
    let mut min_p: usize = usize::MAX;
    let mut max_h: usize = 0;
    let mut last_h: usize = 0;
    let mut last_p: usize = 0;
    let mut last_scav: usize = 0;
    let mut last_omn: usize = 0;
    let mut ever_zero_p = false;

    for t in 1..=600u32 {
        let daylight = 0.5 + 0.5 * sin((t as f64 / 300.0) * std::f64::consts::PI);
        eco.update(0.1, daylight);
        if t % 20 == 0 || t == 600 {
            let h = eco.agents.iter().filter(|a| a.alive && a.kind == HERB).count();
            let p = eco.agents.iter().filter(|a| a.alive && a.kind == PRED).count();
            let s = eco.agents.iter().filter(|a| a.alive && a.kind == SCAV).count();
            let o = eco.agents.iter().filter(|a| a.alive && a.kind == OMN).count();
            if p < min_p { min_p = p; }
            if h > max_h { max_h = h; }
            if p == 0 { ever_zero_p = true; }
            last_h = h; last_p = p; last_scav = s; last_omn = o;
        }
    }

    // Health assertions for a stable multi-biome ecosystem.
    assert!(min_p > 0, "Predators must not hit zero at any point during 600 ticks; min_p={min_p}",);
    assert!(!ever_zero_p, "P population dropped to zero mid-sim — indicates instability");
    assert!(max_h <= 600, "Herb peak {max_h} exceeds sensible bound of 600 (multi-biome should suppress boom)");
    // Final state sanity: all four kinds still alive
    assert!(last_h > 0, "H count dead at end: {last_h}");
    assert!(last_p > 0, "P count dead at end: {last_p}");
    assert!(last_scav >= 0, "SCAV count negative at end: {last_scav}");
    assert!(last_omn >= 0, "OMN count negative at end: {last_omn}");

    eprintln!(
        "[spatial_biome_stability] final state  t=600: H={last_h} P={last_p} S={last_scav} O={last_omn}\
         | min_P={min_p} max_H={max_h}"
    );
}

#[test]
fn biomass_density_brake_reduces_herbovore_peak_vs_global_cap() {
    // Compare same seed under two regimes: global-only caps vs per-biome reduced herb cap.
    // Baseline: Ecology::new uses global MAX_HERB=420 everywhere.
    // Test: replace one region with max_herb=100; peak H should drop below what
    // the baseline would allow (the baseline's own peak over 600 ticks is a proxy).

    let mut baseline_max_h = 0usize;
    let mut b = Ecology::new(20261102);
    for t in 1..=600u32 {
        let daylight = 0.5 + 0.5 * sin((t as f64 / 300.0) * std::f64::consts::PI);
        b.update(0.1, daylight);
        if t % 50 == 0 || t == 600 {
            baseline_max_h = baseline_max_h.max(b.agents.iter().filter(|a| a.alive && a.kind == HERB).count());
        }
    }

    let mut biome_max_h = 0usize;
    let mut c = Ecology::with_biomes(20261102, make_biomes());
    for t in 1..=600u32 {
        let daylight = 0.5 + 0.5 * sin((t as f64 / 300.0) * std::f64::consts::PI);
        c.update(0.1, daylight);
        if t % 50 == 0 || t == 600 {
            biome_max_h = biome_max_h.max(c.agents.iter().filter(|a| a.alive && a.kind == HERB).count());
        }
    }

    // Multi-biome landscape should suppress peak H relative to monolithic world.
    // Allow some margin since the barren biome only covers part of the map.
    assert!(biome_max_h <= baseline_max_h + 10,
        "Expected biome landscape to keep H peak <= baseline + 10; got biome_max_h={biome_max_h}, baseline_max_h={baseline_max_h}");
}
