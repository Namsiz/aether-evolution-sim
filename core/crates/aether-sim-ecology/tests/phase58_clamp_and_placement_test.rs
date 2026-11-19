//! Phase 58 acceptance tests — clamp sweep + the design-b placement arm.
//!
//! Phase 56 explained its speciation result by "the multiplier hits a
//! physiological bound": gain 20 pins every placement disk at 0.5×/1.5×, and
//! that saturation — not the field variance — is what costs the DECO guild
//! population and fragments it. Phase 58 asks whether the bound itself is the
//! mechanism (a wider/narrower clamp still forging speciation) or whether the
//! result is a coincidence of gain 20 × the [0.5, 1.5] window.
//!
//! Both instruments here are experiment arms, never defaults:
//! `deco_env_clamp_on(min, max)` overrides `STRUCTURE_DECO_MIN/MAX` for a run
//! (not persisted), and `token_placement_by_value_on()` makes placements stamp
//! the highest-value known token instead of `known_tokens[0]`. The sweep
//! verdicts live in the Phase 58 report; these tests pin the plumbing: the
//! default world keeps the compiled bounds and the inheritance-order pick, the
//! override really moves the saturation bound while the net-zero contract
//! survives, and neither arm leaks into a save/restore.

use aether_sim_ecology::ecology::{
    Ecology, STRUCTURE_DECO_MAX, STRUCTURE_DECO_MIN,
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

#[test]
fn the_default_world_keeps_the_compiled_bounds_and_the_inheritance_order_pick() {
    let eco = Ecology::new(20261102);
    assert_eq!(eco.deco_env_min, STRUCTURE_DECO_MIN);
    assert_eq!(eco.deco_env_max, STRUCTURE_DECO_MAX);
    assert!(!eco.token_placement_by_value,
        "design-b placement arm must stay off in the default world");
}

#[test]
fn the_clamp_arm_moves_the_saturation_bound_and_the_net_zero_contract_survives() {
    // Narrowing: an enrichment dipole pins its disk at the new floor.
    let mut narrow = Ecology::new(20261102);
    let before = field_sum(&narrow);
    narrow.deco_env_clamp_on(0.6, 1.4);
    for _ in 0..200 {
        narrow.deco_env_dipole(0.0, 0.0, 0.05);
    }
    let lo = narrow.deco_env.iter().cloned().fold(f32::INFINITY, f32::min);
    assert!((lo - 0.6).abs() < 1e-4, "narrow arm must pin the disk at 0.6, got {lo}");
    assert!((field_sum(&narrow) - before).abs() < 1e-3,
        "narrow clamp broke the net-zero contract: {}", field_sum(&narrow) - before);

    // Widening: a depletion dipole pins at the new ceiling.
    let mut wide = Ecology::new(20261102);
    wide.deco_env_clamp_on(0.4, 1.6);
    for _ in 0..200 {
        wide.deco_env_dipole(0.0, 0.0, -0.05);
    }
    let hi = wide.deco_env.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!((hi - 1.6).abs() < 1e-4, "wide arm must pin the disk at 1.6, got {hi}");

    // Setting the override to the compiled bounds must be a pure no-op.
    let mut same = Ecology::new(20261102);
    same.deco_env_clamp_on(STRUCTURE_DECO_MIN, STRUCTURE_DECO_MAX);
    run_ticks(&mut same, 300);
    let mut plain = Ecology::new(20261102);
    run_ticks(&mut plain, 300);
    assert_eq!(same.checksum(), plain.checksum(),
        "an override equal to the constants must not disturb the trajectory");
}

#[test]
fn the_narrow_clamp_clips_the_live_field_above_the_control_floor() {
    // By t≈900 the first placements have imprinted the field; the 0.6 floor
    // is a different carcass lifetime for every DISK cell the control pins
    // at 0.5, which is where the two worlds part (the agent-level checksum
    // only moves once a DECO actually feeds on a clipped cell — that is the
    // 80k observatory's job, not a 2k assertion).
    let mut plain = Ecology::new(20261102);
    run_ticks(&mut plain, 2_000);
    let mut narrow = Ecology::new(20261102);
    narrow.deco_env_clamp_on(0.6, 1.4);
    run_ticks(&mut narrow, 2_000);
    let pmin = plain.deco_env.iter().cloned().fold(f32::INFINITY, f32::min);
    let nmin = narrow.deco_env.iter().cloned().fold(f32::INFINITY, f32::min);
    assert!(nmin > pmin + 1e-3,
        "plain_min={pmin} narrow_min={nmin}: the 0.6 floor must clip the disk imprint above the control's 0.5 floor");
}

#[test]
fn both_experiment_arms_are_volatile_but_deterministic() {
    let mut a = Ecology::new(20261102);
    a.deco_env_clamp_on(0.6, 1.4);
    a.token_placement_by_value_on();
    run_ticks(&mut a, 1_500);
    let mut b = Ecology::new(20261102);
    b.deco_env_clamp_on(0.6, 1.4);
    b.token_placement_by_value_on();
    run_ticks(&mut b, 1_500);
    assert_eq!(a.checksum(), b.checksum(), "arm double run diverged");

    // Neither arm is persisted: a restore lands on the compiled defaults.
    let json = serde_json::to_string(&a.to_save_state()).unwrap();
    let back = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    assert_eq!(back.deco_env_min, STRUCTURE_DECO_MIN);
    assert_eq!(back.deco_env_max, STRUCTURE_DECO_MAX);
    assert!(!back.token_placement_by_value);
}
