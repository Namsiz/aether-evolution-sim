//! Phase 57 acceptance tests — token-value sign decoupling.
//!
//! Phase 56 handed structure placements the decomposition field, whose
//! direction is the sign of the token's `grass_mod`. But the placement reward
//! was still `gain = grass_mod × 10`, so a positive-sign token was worth one
//! energy more per placement than its mirror image: value selection
//! (`culture_value_on`) sorted the negative-sign tokens out of the repertoire
//! and the culture collapsed into a single (enrichment) sign — Phase 56
//! measured pos/neg = 1809/81 standing structures.
//!
//! Phase 57 decouples reward from direction: `token_value_sign_neutral` makes
//! the placement gain read `|grass_mod| × 10`, so a token's sign only decides
//! the environmental effect. The A/B arm is `token_value_sign_neutral_on()`.
//!
//! The 200k verdict kept the switch OFF (the signed Phase 56 reward stays the
//! default world): neutralising the reward did not balance the signs — the
//! invention prior (`vec[0] = 2·grass − 1`, and standing grass sits below the
//! 0.5 midpoint almost everywhere) then owns the sign outright, every clade
//! converged on the negative pole (pos=0/neg=4416 on seed 20261102), and
//! speciation vanished on both seeds (0 events vs the signed arm's 5 and 3).
//! The tests below pin the two things that survive that verdict: the reward
//! really is decoupled on the arm, and the arm is a true no-op when off.

use aether_sim_ecology::culture::token_to_env_mod;
use aether_sim_ecology::ecology::Ecology;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, to: u32) {
    for t in 1..=to {
        eco.update(0.1, daylight(t));
    }
}

/// Standing structures split by the sign of their token's `grass_mod`.
///
/// De-novo inventions seed `vec[0]` from the local grass signature
/// (`2·grass − 1`), so the sign mix at any tick is an environmental
/// statistic, not a value-selection one; the A/B contrast lives in the
/// observatory data (Phase 57 report), not in a short-run assertion.
fn structure_signs(eco: &Ecology) -> (usize, usize) {
    let mut pos = 0usize;
    let mut neg = 0usize;
    for s in &eco.structure_registry.structures {
        let gm = eco.token_registry.get(s.token_id)
            .map(|tok| token_to_env_mod(tok).0)
            .unwrap_or(0.0);
        if gm > 0.0 { pos += 1; } else if gm < 0.0 { neg += 1; }
    }
    (pos, neg)
}

#[test]
fn default_world_keeps_the_signed_reward_and_the_arm_survives_a_roundtrip() {
    let mut eco = Ecology::new(20261102);
    assert!(!eco.token_value_sign_neutral,
        "Phase 57 default world keeps the signed Phase 56 reward");
    eco.token_value_sign_neutral_on();
    assert!(eco.token_value_sign_neutral);
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let back = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    assert!(back.token_value_sign_neutral, "v22 archive must carry the A/B arm");
    assert!(json.contains("\"TokenValueSignNeutral\":true"));
}

#[test]
fn pre_v22_archive_lands_on_the_signed_default_and_continues_deterministically() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 400);
    let mut v: serde_json::Value = serde_json::to_value(&eco.to_save_state()).unwrap();
    let obj = v.as_object_mut().unwrap();
    obj.remove("TokenValueSignNeutral");
    obj.insert("Version".into(), serde_json::json!(21));
    let legacy = serde_json::to_string(&v).unwrap();
    let mut a = Ecology::from_save_state(serde_json::from_str(&legacy).unwrap());
    assert!(!a.token_value_sign_neutral,
        "pre-v22 archive must land on the signed default (the Phase 56 world)");
    let mut b = Ecology::from_save_state(serde_json::from_str(&legacy).unwrap());
    for t in 401..=1_400u32 {
        a.update(0.1, daylight(t));
        b.update(0.1, daylight(t));
    }
    assert_eq!(a.checksum(), b.checksum(), "two restores of the same v21 archive diverged");
}

#[test]
fn value_ema_sign_follows_the_placement_gain_sign_in_both_arms() {
    // Invariant, not a statistic: every placement of a token records
    // `gain = grass_mod × 10` (signed arm) or `|grass_mod| × 10` (neutral
    // arm) into that token's `use_gain` EMA, whose only other operation is a
    // positive multiplicative decay. So on the signed arm a negative-sign
    // token can never acquire a positive value, and on the neutral arm no
    // token can ever be negative — that asymmetry IS the selection pressure
    // Phase 57 removes.
    let mut signed = Ecology::new(20261102);
    run_ticks(&mut signed, 12_000);
    let mut neutral = Ecology::new(20261102);
    neutral.token_value_sign_neutral_on();
    run_ticks(&mut neutral, 12_000);

    let mut signed_neg_valued = 0usize;
    for t in signed.token_registry.tokens.iter() {
        let gm = token_to_env_mod(t).0;
        if gm < 0.0 {
            assert!(signed.token_registry.value_of(t.id) <= 0.0,
                "signed arm: negative-sign token {} has positive value {}", t.id, signed.token_registry.value_of(t.id));
            signed_neg_valued += 1;
        }
    }
    let mut neutral_neg_positive_value = 0usize;
    let mut neutral_neg = 0usize;
    for t in neutral.token_registry.tokens.iter() {
        let gm = token_to_env_mod(t).0;
        assert!(neutral.token_registry.value_of(t.id) >= 0.0,
            "neutral arm: token {} has negative value {}", t.id, neutral.token_registry.value_of(t.id));
        if gm < 0.0 {
            neutral_neg += 1;
            if neutral.token_registry.value_of(t.id) > 0.0 { neutral_neg_positive_value += 1; }
        }
    }
    assert!(signed_neg_valued > 0 && neutral_neg > 0,
        "both arms must have invented negative-sign tokens (signed={signed_neg_valued} neutral={neutral_neg})");
    assert!(neutral_neg_positive_value > 0,
        "neutral arm must credit negative-sign tokens (0 of {neutral_neg} placed)");
}

#[test]
#[ignore = "three 60k-tick arms in release — minutes; signed-arm anchor + neutral-arm determinism"]
fn signed_arm_keeps_the_phase56_anchor_and_the_neutral_arm_is_deterministic() {
    // The arm lost the A/B, so it is off by default and the default world is
    // the Phase 56 one — until Phase 59's sixteenth re-baseline moved the
    // default trajectory (saturated DECO fertility map).
    let mut signed = Ecology::new(20261102);
    run_ticks(&mut signed, 60_000);
    assert_eq!(signed.checksum(), "0xde2c6292e0dfb9dc",
        "default (signed) world at t=60000 (Phase 59 re-baseline)");

    let mut a = Ecology::new(20261102);
    a.token_value_sign_neutral_on();
    run_ticks(&mut a, 60_000);
    let mut b = Ecology::new(20261102);
    b.token_value_sign_neutral_on();
    run_ticks(&mut b, 60_000);
    assert_eq!(a.checksum(), b.checksum(), "neutral-arm double run diverged");

    let (sp, sn) = structure_signs(&signed);
    let (np, nn) = structure_signs(&a);
    eprintln!("[phase57] @60000 signed pos={sp} neg={sn} · neutral pos={np} neg={nn} \
        checksum={}", a.checksum());
    assert_eq!(a.checksum(), NEUTRAL_ARM_ANCHOR_60000, "sign-neutral arm at t=60000");
}

// Phase 59 re-baseline: the fertility fix moved the neutral arm too.
const NEUTRAL_ARM_ANCHOR_60000: &str = "0xd802b5b07aadfe4a";
