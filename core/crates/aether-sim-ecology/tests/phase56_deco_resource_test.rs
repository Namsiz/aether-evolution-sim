//! Phase 56 acceptance tests — culture × the DECO resource face.
//!
//! Phases 54/55 proved the grass field is the wrong causal route: any structure
//! effect on primary production stabilises the DECO root (the lineage that
//! actually forks) and zeroes speciation. Phase 56 moves the target onto the
//! DECO's own food supply: `deco_env` is a per-cell multiplier on the carcass
//! decay rate, imprinted by structure placements as a net-zero dipole whose
//! direction is the sign of the token's `grass_mod`. Enrichment tokens slow
//! decomposition (a cultural larder), depletion tokens accelerate it (a
//! desert), and the ring pays back whatever the disk absorbed, so the field
//! mean stays at 1.0.
//!
//! Pinned here: the conservation/symmetry/saturation invariants of the dipole,
//! the carcass pass really reading the field, the A/B switch being a true
//! no-op when off (the Phase 54 OFF anchor is `phase54_culture_niche_test`'s),
//! and v21 archive compatibility. The 80k/200k A/B verdict lives in the
//! observatory data and the Phase 56 report.

use aether_sim_ecology::ecology::{
    Carcass, Ecology, DECO_RESOURCE_ENV_ON, STRUCTURE_DECO_MAX, STRUCTURE_DECO_MIN,
};
fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, to: u32) {
    for t in 1..=to {
        eco.update(0.1, daylight(t));
    }
}

fn field_sum(eco: &Ecology) -> f64 {
    eco.deco_env.iter().map(|&v| v as f64).sum()
}

/// World position → grid cell, mirroring `Ecology::cell_index` (which is
/// crate-private) with the public grid constants.
fn cell_of(x: f64, z: f64) -> usize {
    let span = aether_sim_ecology::ecology::GRID_SPAN;
    let n = aether_sim_ecology::ecology::GRID_N as usize;
    let cell = span / n as f64;
    let i = ((x + span / 2.0) / cell).floor() as usize;
    let j = ((z + span / 2.0) / cell).floor() as usize;
    j.min(n - 1) * n + i.min(n - 1)
}

#[test]
fn decomposition_dipole_preserves_the_field_sum_midfield_and_at_edges() {
    let mut eco = Ecology::new(20261102);
    let before = field_sum(&eco);
    // Positive grass_mod = enrichment token → slower decomposition on the
    // disk (multiplier below 1), faster on the ring.
    eco.deco_env_dipole(0.0, 0.0, 0.05);
    assert!((field_sum(&eco) - before).abs() < 1e-3,
        "enrichment dipole changed the field sum by {}", field_sum(&eco) - before);
    let below = eco.deco_env.iter().filter(|&&v| v < 1.0 - 1e-4).count();
    let above = eco.deco_env.iter().filter(|&&v| v > 1.0 + 1e-4).count();
    assert!(below > 0 && above > 0,
        "dipole must mark both sides: below={below} above={above}");
    // Edge clipping must not leak the absorption.
    eco.deco_env_dipole(-99999.0, -99999.0, 0.05);
    assert!((field_sum(&eco) - before).abs() < 1e-3,
        "edge-clipped dipole changed the field sum by {}", field_sum(&eco) - before);
    // The mirror: a depletion token walks the field back to all-1.0 on the
    // same cell — a placement only ever moves the larder, never creates it.
    // (Sub-clamp strengths only: once an imprint pins the disk at its bound
    // the clamp is a nonlinearity, and the reverse placement overshoots to
    // the other bound instead of retracing — that is the saturation case the
    // next test pins.)
    let mut eco2 = Ecology::new(20261102);
    eco2.deco_env_dipole(0.0, 0.0, 0.01);
    eco2.deco_env_dipole(0.0, 0.0, -0.01);
    let drift = eco2.deco_env.iter().map(|&v| (v - 1.0f32).abs() as f64)
        .fold(0.0f64, f64::max);
    assert!(drift < 1e-5, "opposite dipole on the same cell left residue {drift}");
    // The two signs really are opposite regimes (mean-conserved contrast).
    let mut eco3 = Ecology::new(20261102);
    eco3.deco_env_dipole(0.0, 0.0, -0.05);
    let center = cell_of(0.0, 0.0);
    assert!(eco3.deco_env[center] > 1.0 + 1e-4,
        "depletion token must speed up decomposition at its own cell");
    assert!(eco.deco_env[center] < 1.0 - 1e-4,
        "enrichment token must slow down decomposition at its own cell");
}

#[test]
fn decomposition_dipole_saturation_self_limits() {
    let mut eco = Ecology::new(20261102);
    for _ in 0..200 {
        eco.deco_env_dipole(0.0, 0.0, 0.05);
    }
    let saturated = field_sum(&eco);
    let pinned = eco.deco_env.iter().filter(|&&v| v <= STRUCTURE_DECO_MIN + 1e-4).count();
    assert!(pinned > 0, "disk must be pinned at MIN after 200 placements");
    for _ in 0..100 {
        eco.deco_env_dipole(0.0, 0.0, 0.05);
    }
    assert!((field_sum(&eco) - saturated).abs() < 1e-3,
        "sum kept moving after full saturation ({} → {})", saturated, field_sum(&eco));
    let mut eco2 = Ecology::new(20261102);
    for _ in 0..200 {
        eco2.deco_env_dipole(0.0, 0.0, -0.05);
    }
    let pinned_hi = eco2.deco_env.iter().filter(|&&v| v >= STRUCTURE_DECO_MAX - 1e-4).count();
    assert!(pinned_hi > 0, "depletion dipole must pin the disk at MAX");
}

#[test]
fn carcass_decay_follows_the_local_multiplier() {
    // The carcass pass must read the field at the carcass's own cell: two
    // identical carcasses, one in a larder (mult 0.5) and one in a desert
    // (mult 1.5), lose energy at 1.4/tick × their own multiplier on the same
    // tick. The spots are chosen away from every live agent so no bite can
    // confound the measurement.
    let mut eco = Ecology::new(20261102);
    let offsets = [(0.0f64, 0.0f64), (-400.0, -400.0), (-400.0, 400.0), (400.0, -400.0), (400.0, 400.0)];
    let clear = |eco: &Ecology, x: f64, z: f64| {
        eco.living().iter().all(|a| {
            let dx = a.x - x;
            let dz = a.z - z;
            dx * dx + dz * dz > 4.0
        })
    };
    let mut spots: Vec<(f64, f64)> = Vec::new();
    for &(x, z) in &offsets {
        if clear(&eco, x, z) { spots.push((x, z)); }
        if spots.len() == 2 { break; }
    }
    assert_eq!(spots.len(), 2, "need two agent-free spots for the decay probe");

    eco.deco_env_on = true;
    eco.carcasses.clear();
    eco.deco_env[cell_of(spots[0].0, spots[0].1)] = 0.5;
    eco.deco_env[cell_of(spots[1].0, spots[1].1)] = 1.5;
    for &(x, z) in &spots {
        eco.carcasses.push(Carcass { x, z, energy: 100.0, age: 0.0, occupied_by: None });
    }
    eco.update(0.1, daylight(1));
    // update() runs one 0.1-tick substep; decay = 1.4 × 0.1 × mult.
    let larder = 100.0 - eco.carcasses.iter().find(|c| c.x == spots[0].0).unwrap().energy;
    let desert = 100.0 - eco.carcasses.iter().find(|c| c.x == spots[1].0).unwrap().energy;
    assert!((larder - 1.4 * 0.1 * 0.5).abs() < 1e-9, "larder decay was {larder}");
    assert!((desert - 1.4 * 0.1 * 1.5).abs() < 1e-9, "desert decay was {desert}");
}

#[test]
fn a_switch_off_reads_exactly_the_flat_rate() {
    // OFF is the mathematical no-op the anchors rely on: multiplier 1.0 at
    // every position, independent of whatever the field holds.
    let mut eco = Ecology::new(20261102);
    eco.deco_env[cell_of(0.0, 0.0)] = 0.5;
    assert_eq!(eco.deco_env_on, DECO_RESOURCE_ENV_ON);
    eco.deco_env_on = false;
    assert_eq!(eco.deco_decay_mult_at(0.0, 0.0), 1.0);
    eco.deco_env_on = true;
    assert_eq!(eco.deco_decay_mult_at(0.0, 0.0), 0.5);
}

#[test]
fn deco_resource_env_off_resets_and_freezes_the_field() {
    let mut eco = Ecology::new(20261102);
    eco.deco_env_on = true;
    eco.deco_env_dipole(0.0, 0.0, 0.05);
    assert!(field_sum(&eco) != (eco.deco_env.len() as f64));
    eco.deco_resource_env_off();
    assert!(!eco.deco_env_on);
    assert!(eco.deco_env.iter().all(|&v| v == 1.0f32), "field must be flat after the arm switch");
    assert!(!eco.deco_env_decay_active);
}

#[test]
fn placements_imprint_the_field_by_token_sign() {
    // Behavioral leg: with the A/B on, t=60k structures must have marked the
    // decomposition field in both directions (the token repertoire carries
    // both signs), and the field mean must stay at 1.0 (net-zero contract).
    let mut eco = Ecology::new(20261102);
    eco.deco_env_on = true;
    run_ticks(&mut eco, 60_000);
    assert!(eco.structure_registry.len() > 0, "expected structures by t=60000");
    let below = eco.deco_env.iter().filter(|&&v| v < 1.0 - 1e-4).count();
    let above = eco.deco_env.iter().filter(|&&v| v > 1.0 + 1e-4).count();
    let mean = field_sum(&eco) / eco.deco_env.len() as f64;
    eprintln!("[phase56] ON t=60000 structs={} below={below} above={above} mean={mean:.5}",
        eco.structure_registry.len());
    assert!(below > 0, "no cell slows decomposition");
    assert!(above > 0, "no cell speeds decomposition");
    assert!((mean - 1.0).abs() < 1e-4, "field mean moved to {mean} (net-zero broken)");
}

#[test]
fn pre_v21_archive_restores_with_a_flat_field_and_continues_deterministically() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 400);
    let mut v: serde_json::Value = serde_json::to_value(&eco.to_save_state()).unwrap();
    let obj = v.as_object_mut().unwrap();
    obj.remove("DecoEnv");
    obj.remove("DecoEnvOn");
    obj.remove("DecoEnvDecayActive");
    obj.insert("Version".into(), serde_json::json!(20));
    let legacy = serde_json::to_string(&v).unwrap();
    let mut a = Ecology::from_save_state(serde_json::from_str(&legacy).unwrap());
    assert!(a.deco_env.iter().all(|&x| x == 1.0f32),
        "pre-v21 archive must restore a flat decomposition field");
    assert!(!a.deco_env_on, "pre-v21 archive must land on the OFF arm");
    let mut b = Ecology::from_save_state(serde_json::from_str(&legacy).unwrap());
    for t in 401..=1_400u32 {
        a.update(0.1, daylight(t));
        b.update(0.1, daylight(t));
    }
    assert_eq!(a.checksum(), b.checksum(), "two restores of the same v20 archive diverged");
}

#[test]
#[ignore = "two 60k-tick arms in release — minutes; ON-arm determinism + save/restore"]
fn on_arm_double_run_and_roundtrip_is_bit_exact() {
    let mut a = Ecology::new(20261102);
    a.deco_env_on = true;
    run_ticks(&mut a, 60_000);
    let mut b = Ecology::new(20261102);
    b.deco_env_on = true;
    run_ticks(&mut b, 60_000);
    assert_eq!(a.checksum(), b.checksum(), "ON-arm double run diverged");
    let json = serde_json::to_string(&a.to_save_state()).unwrap();
    let mut c = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    for t in 60_001..=61_200u32 {
        a.update(0.1, daylight(t));
        c.update(0.1, daylight(t));
    }
    assert_eq!(a.checksum(), c.checksum(), "ON-arm restored continuation diverged");
}

#[test]
#[ignore = "60k-tick arm in release — minutes; A/B arm switch restores the Phase 55 trajectory"]
fn off_arm_restores_the_pre_phase56_anchor() {
    // Acceptance #2: with the arm switched off the world must be the Phase 54/
    // 55 world — the Phase 54 OFF checksum at t=60000. The Phase 59
    // re-baseline moved that world too (the fertility map is a separate
    // default); this arm pins the post-fix pre-56 trajectory.
    let mut eco = Ecology::new(20261102);
    eco.deco_resource_env_off();
    run_ticks(&mut eco, 60_000);
    assert_eq!(eco.checksum(), "0x29e3ebe0d592f8e3",
        "deco resource env OFF: pre-Phase-56 trajectory at t=60000");
}

#[test]
fn default_arm_anchor_at_60000_is_the_phase56_trajectory() {
    // Re-baseline 15: the field is ON by default now, so the default world's
    // deep anchor moves. The pre-structure anchors (@300/@3000) are untouched
    // because no token has been invented yet, so this is the first tick block
    // the new default actually redefines.
    //
    // Phase 59 (re-baseline 16): the DECO fertility map saturates below
    // eff 0.9583, so the default world moves again. The Phase 56 value
    // 0x44285bfbed94865e is now the `repro_threshold_old_off()` anchor
    // (phase59_repro_saturation_test).
    //
    // Phase 60 (re-baseline 17): the regional DECO capacity arm is on by
    // default, so the default world moves again (0xde2c… is the
    // `deco_region_cap_off()` arm, pinned in phase60_region_capacity_test).
    //
    // Phase 63 (re-baseline 18) raised FORK_BUDGET_MIN 8 → 10, and Phase 64
    // (re-baseline 19) reverted it after the out-of-sample run failed, so the
    // default world is the Phase 60/61/62 trajectory again.
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 60_000);
    assert_eq!(eco.checksum(), "0x3272f0e3eaba9250",
        "default arm (Phase 64 = Phase 62 world) at t=60000");
}
