//! Phase 59 — saturated DECO fertility threshold (the birth-cliff hard gate).
//!
//! Phase 58 §2.1: the DECO reproduction threshold is
//! `0.96 × (2 − deco_efficiency)`, while energy is clamped at `max_energy`.
//! Any `eff < 0.9583` therefore asks for more than 100 % of the tank — an
//! absorbing sterile state. `DECO_MUT_SCALE = 5` speciation births generate
//! exactly those genomes, so a daughter clade can be born sterile (two Phase
//! 58 extinctions). Phase 59 saturates the excess into `(0.96, 0.99)`.
//!
//! Pinned here: the boundary values (1.5 → 0.48 unchanged, 1.0 → 0.96
//! unchanged), whole-band fertility, monotonicity + continuity, the old-arm
//! switch restoring the Phase 58 line bit-exactly, and the arm not being
//! persisted.

use aether_sim_ecology::ecology::Ecology;

fn legacy(eff: f64) -> f64 { 0.96 * (2.0 - eff) }

#[test]
fn saturation_keeps_the_easy_end_and_the_neutral_point() {
    let eco = Ecology::new(20261102);
    assert!((eco.deco_repro_threshold(1.5) - 0.48).abs() < 1e-12,
        "eff=1.5 must stay at the Phase 7 0.48");
    assert!((eco.deco_repro_threshold(1.0) - 0.96).abs() < 1e-12,
        "eff=1.0 must stay at the legacy 0.96");
    // The whole easy half is the legacy line, not merely close to it.
    for i in 0..=50 {
        let eff = 1.0 + i as f64 * 0.01;
        assert_eq!(eco.deco_repro_threshold(eff), legacy(eff),
            "eff={eff} left the untouched half of the mapping");
    }
}

#[test]
fn every_legal_efficiency_is_fertile_monotone_and_continuous() {
    let eco = Ecology::new(20261102);
    // eff = 0.5 is the genome clamp's low end: 98.8 % of max_energy, hard but
    // reachable; the legacy line asked for 144 %.
    assert!(eco.deco_repro_threshold(0.5) < 1.0);
    assert!(eco.deco_repro_threshold(0.5) * 100.0 < 100.0);
    let mut prev: Option<f64> = None;
    for i in 0..=1000 {
        let eff = 0.5 + i as f64 * 0.001;
        let t = eco.deco_repro_threshold(eff);
        assert!(t < 1.0, "eff={eff} is sterile: threshold {t} of max_energy");
        if let Some(p) = prev {
            // Lower efficiency must never be easier than a higher one.
            assert!(t <= p + 1e-12, "not monotone at eff={eff}: {t} > {p}");
            // A 0.001 efficiency step can never jump (the legacy slope is
            // 0.96/unit and the saturated slope is smaller).
            assert!((p - t).abs() < 1e-3, "discontinuity at eff={eff}: {p} -> {t}");
        }
        prev = Some(t);
    }
}

#[test]
fn old_arm_restores_the_phase58_line_and_its_sterile_wedge() {
    let mut eco = Ecology::new(20261102);
    eco.repro_threshold_old_off();
    for i in 0..=1000 {
        let eff = 0.5 + i as f64 * 0.001;
        assert_eq!(eco.deco_repro_threshold(eff), legacy(eff),
            "old arm must be the Phase 58 line at eff={eff}");
    }
    assert!(eco.deco_repro_threshold(0.9) > 1.0,
        "the old arm's sterile wedge is the reference this test exists for");
}

#[test]
fn the_arm_is_never_persisted() {
    let mut eco = Ecology::new(20261102);
    eco.repro_threshold_old_off();
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let restored = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    assert!(restored.deco_repro_saturate,
        "a restored world runs the compiled default (saturated) mapping");
}

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

#[test]
#[ignore = "60k-tick arm in release — minutes; Phase 56 anchor replay on the old mapping"]
fn old_arm_reproduces_the_phase56_default_anchor() {
    let mut eco = Ecology::new(20261102);
    eco.repro_threshold_old_off();
    for t in 1..=60_000u32 {
        eco.update(0.1, daylight(t));
    }
    assert_eq!(eco.checksum(), "0x44285bfbed94865e",
        "old mapping: the Phase 56 default world at t=60000 must replay bit-exactly");
}
