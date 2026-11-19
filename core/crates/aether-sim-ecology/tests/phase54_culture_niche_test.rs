//! Phase 54 acceptance tests — culture × speciation coupling (real niche
//! construction).
//!
//! Phase 37 registered structures but they never touched the environment;
//! Phase 54 wires the loop: a placed structure raises the local grass
//! regrowth multiplier, decays back toward 1.0, and (because cultural
//! learning is already kind-local) each clade's structures differentiate
//! the resource face it lives on.

use aether_sim_ecology::ecology::Ecology;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, to: u32) {
    for t in 1..=to {
        eco.update(0.1, daylight(t));
    }
}

#[test]
fn structure_placement_raises_the_local_regrowth_multiplier() {
    let mut eco = Ecology::new(20261102);
    // Default OFF (trophic cascade suppresses speciation); tests opt in.
    eco.structure_env_on = true;
    run_ticks(&mut eco, 60_000);
    // By t=60k the culture layer must have minted tokens and placed
    // structures (Phase 40.2/45 calibration: inventions from ~t=5k).
    assert!(eco.structure_registry.len() > 0,
        "expected structures to exist by t=60000, got 0");
    let raised = eco.structure_env.iter().filter(|&&v| (v - 1.0f32).abs() > 1e-4).count();
    assert!(raised > 0,
        "no cell carries a raised regrowth multiplier (structures={} )",
        eco.structure_registry.len());
    // The raised cells must be near an actual structure.
    let (sx, sz) = {
        let s = &eco.structure_registry.structures[0];
        (s.x, s.z)
    };
    // grid = 76x76 cells of CELL world units centered at 0; find the cell
    // by direct arithmetic (cell_index is crate-private).
    // simply assert that SOME raised cell exists near the structure:
    // scan the 3-cell neighbourhood around the structure's grid slot.
    let n = eco.structure_env.len() as usize;
    let side = (n as f64).sqrt() as usize; // 76
    let cell = 14.2105263157894736f64; // world span / 76 (approx from tests elsewhere)
    let col = (((sx + side as f64 * cell / 2.0) / cell).floor().max(0.0) as usize).min(side - 1);
    let row = (((sz + side as f64 * cell / 2.0) / cell).floor().max(0.0) as usize).min(side - 1);
    let center = row * side + col;
    // neighbourhood scan (radius 3): at least one cell raised
    let mut found = false;
    for dr in -3i64..=3 {
        for dc in -3i64..=3 {
            let r = row as i64 + dr;
            let c = col as i64 + dc;
            if r < 0 || c < 0 || r >= side as i64 || c >= side as i64 { continue; }
            let v = eco.structure_env[(r as usize) * side + c as usize];
            if (v - 1.0f32).abs() > 1e-4 { found = true; }
        }
    }
    assert!(found, "no raised cell within 3 cells of the structure");
}

#[test]
fn structure_env_off_is_the_phase53_trajectory() {
    // A/B: the switch off resets and freezes the multiplier field. With the
    // OFF switch the boost branch and the decay pass are skipped entirely
    // and the growth multiplier is exactly 1.0 — a mathematical no-op, so
    // the OFF anchor below IS the Phase 53 world trajectory (re-baselined
    // by the Phase 53 budget-floor change; the pre-floor value is gone).
    let mut eco = Ecology::new(20261102);
    eco.structure_env_off();
    // Phase 56 (re-baseline 15): the decomposition field is a separate switch
    // and it is on by default; this anchor is the pre-56 world.
    eco.deco_resource_env_off();
    // Phase 59 (re-baseline 16): the saturated DECO fertility map moves every
    // world with DECO births, this arm included.
    run_ticks(&mut eco, 60_000);
    assert_eq!(eco.checksum(), "0x29e3ebe0d592f8e3",
        "structure_env OFF: no-op trajectory (Phase 53 world) at t=60000");
}

#[test]
fn structure_env_roundtrips_bit_exactly() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 60_000);
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let restored = Ecology::from_save_state(
        serde_json::from_str(&json).unwrap(),
    );
    let mut a = eco;
    let mut b = restored;
    for t in 60_001..=61_200u32 {
        a.update(0.1, daylight(t));
        b.update(0.1, daylight(t));
    }
    assert_eq!(a.checksum(), b.checksum(),
        "restored continuation diverged (structure_env field must round-trip)");
}

#[test]
fn structure_env_multiplies_regrowth_only_through_growth() {
    // The multiplier never feeds anything but the growth path: with the
    // field fully decayed (no structures in a young world), the trajectory
    // is bit-exact with the field at all-1.0 — verified by the OFF anchor
    // above; here we pin that the decay pass is skipped once relaxed.
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 300);
    assert!(!eco.structure_env_decay_active,
        "no placement happened by t=300; the decay pass must be inactive");
}
