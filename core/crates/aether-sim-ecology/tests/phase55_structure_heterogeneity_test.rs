//! Phase 55 acceptance tests — heterogeneity-first structure calibration
//! (net-zero dipole).
//!
//! Phase 54 measured the honest failure of a one-sided boost: value-selected
//! positive tokens raise total primary production, the DECO root stabilises,
//! drift slows, speciation goes to zero. Phase 55 reworks the boost as a
//! TRANSFER: the disk under a structure absorbs `grass_mod × BOOST` per cell
//! (clamped, measured) and the exact amount absorbed is subtracted from the
//! surrounding ring. Culture can choose where grass grows; it can no longer
//! raise the ceiling. The mechanism invariants are pinned here
//! (conservation, symmetry, saturation self-limit, archive compatibility);
//! the 80k A/B verdict lives in the observatory data: four ON calibrations
//! (BOOST 0.2/2/20 natural + 20 famine-sign) all scored zero speciation
//! events against the OFF arm's two, and the 200k probe of the held
//! calibration ran forkless while OFF forked four times — the dipole
//! concentrates regrowth where the culture lives, and the accessible
//! production (not the total) stabilises the DECO root. What the dipole
//! DOES deliver (and what these tests pin at 60k): a measurable multiplier
//! mosaic with the field mean locked at exactly 1.0, and a grass standing-
//! crop variance above the OFF arm.

use aether_sim_ecology::ecology::{
    Ecology, STRUCTURE_ENV_MAX, STRUCTURE_ENV_MIN,
};

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * ((t as f64 / 300.0) * std::f64::consts::PI).sin()
}

fn run_ticks(eco: &mut Ecology, to: u32) {
    for t in 1..=to {
        eco.update(0.1, daylight(t));
    }
}

fn field_sum(eco: &Ecology) -> f64 {
    eco.structure_env.iter().map(|&v| v as f64).sum()
}

#[test]
fn dipole_preserves_the_field_sum_midfield_and_at_edges() {
    // One placement from an all-1.0 field: the disk absorbs, the ring pays
    // back. No clamp binds (1.0 ± 0.1 is well inside [0.5, 1.5]), so the sum
    // must return to exactly n (f32 rounding aside).
    let n = (aether_sim_ecology::ecology::GRID_N as usize).pow(2);
    let mut eco = Ecology::new(20261102);
    let before = field_sum(&eco);
    // midfield placement, positive token
    eco.structure_env_dipole(0.0, 0.0, 0.05);
    assert!((field_sum(&eco) - before).abs() < 1e-3,
        "positive dipole changed the field sum by {}", field_sum(&eco) - before);
    // edge placement: in-bounds clipping must not leak the boost
    eco.structure_env_dipole(-99999.0, -99999.0, 0.05);
    assert!((field_sum(&eco) - before).abs() < 1e-3,
        "edge-clipped dipole changed the field sum by {}", field_sum(&eco) - before);
    // negative token mirrors the geometry (disk below 1, ring above);
    // (100,100) is ~7 cells away from the origin so the two dipoles overlap
    // only partially
    eco.structure_env_dipole(100.0, 100.0, -0.05);
    assert!((field_sum(&eco) - before).abs() < 1e-3,
        "negative dipole changed the field sum by {}", field_sum(&eco) - before);
    // the marks are really there, in both directions
    let above = eco.structure_env.iter().filter(|&&v| v > 1.0 + 1e-4).count();
    let below = eco.structure_env.iter().filter(|&&v| v < 1.0 - 1e-4).count();
    assert!(above > 0 && below > 0,
        "dipole must mark both sides: above={} below={}", above, below);

    // The purest form of the net-zero contract: an opposite dipole on the
    // SAME cell must walk the field back to all-1.0 (f32 rounding aside) —
    // a placement is reversible because it only ever moves potential.
    let mut eco2 = Ecology::new(20261102);
    eco2.structure_env_dipole(0.0, 0.0, 0.05);
    eco2.structure_env_dipole(0.0, 0.0, -0.05);
    let drift = eco2.structure_env.iter().map(|&v| (v - 1.0f32).abs() as f64)
        .fold(0.0f64, f64::max);
    assert!(drift < 1e-5,
        "opposite dipole on the same cell left residue {drift}");
    let _ = n;
}

#[test]
fn dipole_saturation_self_limits() {
    // Repeated placements on the same spot pin the disk at MAX; once pinned
    // a placement absorbs nothing and compensates nothing, so the field sum
    // stops moving upward even after many further placements.
    let mut eco = Ecology::new(20261102);
    for _ in 0..200 {
        eco.structure_env_dipole(0.0, 0.0, 0.05);
    }
    let after_saturation = field_sum(&eco);
    let disk_max = eco.structure_env.iter().filter(|&&v| v >= STRUCTURE_ENV_MAX - 1e-4).count();
    assert!(disk_max > 0, "disk must be pinned at MAX after 200 placements");
    for _ in 0..100 {
        eco.structure_env_dipole(0.0, 0.0, 0.05);
    }
    assert!((field_sum(&eco) - after_saturation).abs() < 1e-3,
        "sum kept moving after full saturation ({} → {})",
        after_saturation, field_sum(&eco));
    // and the mirror: a negative token pins the disk at MIN symmetrically
    let mut eco2 = Ecology::new(20261102);
    for _ in 0..200 {
        eco2.structure_env_dipole(0.0, 0.0, -0.05);
    }
    let disk_min = eco2.structure_env.iter().filter(|&&v| v <= STRUCTURE_ENV_MIN + 1e-4).count();
    assert!(disk_min > 0, "negative dipole must pin the disk at MIN");
}

#[test]
#[ignore = "two 60k-tick arms in release — minutes; the ON/OFF heterogeneity A/B"]
fn heterogeneity_on_exceeds_off_at_60k() {
    // Acceptance #2: the dipole must measurably heterogenise the standing
    // grass field. Same seed, both arms to t=60k, spatial variance of the
    // crop over live (suit>0.02) cells.
    let variance = |eco: &Ecology| -> f64 {
        let mut nn = 0.0f64;
        let mut sum = 0.0f64;
        for i in 0..eco.grass_field().len() {
            if eco.suit_field()[i] <= 0.02 { continue; }
            nn += 1.0;
            sum += eco.grass_field()[i] as f64;
        }
        let mean = sum / nn;
        let mut var = 0.0f64;
        for i in 0..eco.grass_field().len() {
            if eco.suit_field()[i] <= 0.02 { continue; }
            let d = eco.grass_field()[i] as f64 - mean;
            var += d * d;
        }
        var / nn
    };
    let mut on = Ecology::new(20261102);
    on.structure_env_on = true;
    run_ticks(&mut on, 60_000);
    let var_on = variance(&on);
    let env_var_on = {
        let n = on.structure_env.len() as f64;
        let mean = on.structure_env.iter().map(|&v| v as f64).sum::<f64>() / n;
        on.structure_env.iter().map(|&v| { let d = v as f64 - mean; d * d }).sum::<f64>() / n
    };
    eprintln!("[phase55] ON  t=60000 grass_var={var_on:.5} env_var={env_var_on:.5} structs={}",
        on.structure_registry.len());
    let mut off = Ecology::new(20261102);
    off.structure_env_off();
    run_ticks(&mut off, 60_000);
    let var_off = variance(&off);
    eprintln!("[phase55] OFF t=60000 grass_var={var_off:.5} env_var=0 structs={}",
        off.structure_registry.len());
    assert!(var_on > var_off,
        "ON arm grass variance ({var_on:.5}) must exceed OFF ({var_off:.5})");
    assert!(env_var_on > 0.0,
        "ON arm structure_env field is flat — dipole never fired");
}

#[test]
#[ignore = "two 60k-tick arms in release — minutes; ON-arm determinism + save/restore"]
fn on_arm_double_run_and_roundtrip_is_bit_exact() {
    // Determinism on the ON path: two independent worlds replay tick-for-tick
    // identically, and a save/restore at t=60k continues in lockstep (the
    // dipole writes only the structure_env field, which v20 already
    // round-trips — this pins the whole path).
    let mut a = Ecology::new(20261102);
    a.structure_env_on = true;
    run_ticks(&mut a, 60_000);
    let mut b = Ecology::new(20261102);
    b.structure_env_on = true;
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
fn pre_v20_archive_restores_with_a_flat_field_and_continues_deterministically() {
    // Old-archive contract: a v19 archive has no StructureEnv keys. It must
    // land with the field rebuilt as all-1.0 (``structure_env.len() != n`` →
    // fresh 1.0s) and continue deterministically — the dipole never needs
    // state the archive lacks, so the schema stays v20.
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 400);
    let mut v: serde_json::Value = serde_json::to_value(&eco.to_save_state()).unwrap();
    let obj = v.as_object_mut().unwrap();
    obj.remove("StructureEnv");
    obj.remove("StructureEnvOn");
    obj.remove("StructureEnvDecayActive");
    obj.insert("Version".into(), serde_json::json!(19));
    let legacy = serde_json::to_string(&v).unwrap();
    let mut a = Ecology::from_save_state(serde_json::from_str(&legacy).unwrap());
    assert!(a.structure_env.iter().all(|&x| x == 1.0f32),
        "pre-v20 archive must restore a flat regrowth field");
    let mut b = Ecology::from_save_state(serde_json::from_str(&legacy).unwrap());
    for t in 401..=1_400u32 {
        a.update(0.1, daylight(t));
        b.update(0.1, daylight(t));
    }
    assert_eq!(a.checksum(), b.checksum(),
        "two restores of the same v19 archive diverged");
}
