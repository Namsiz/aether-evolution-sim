//! Phase 63 — kind-budget rescue arms.
//!
//! Phase 62 (§3.3/§4) showed the daughter clades die on their own Phase 49
//! share (grant 8–20, born at `SPECIATION_MIN_SIDE = 8` members, ≥ 99.6 % of
//! births blocked by the kind budget), not on the regional share. This file
//! pins the three candidate contracts (`research/phase63-report.md`):
//!
//! * G — earned growth: a filled daughter moves one slot per 5 000 survival
//!   ticks out of its parent's share (family total invariant). The state
//!   lives in `KindBudgets`, so the switch and the ledger ride the archive;
//! * M — both fork floors raised (grant and the parent's remainder);
//! * A — only the daughter's grant floor raised.
//!
//! The default world stays on the Phase 49 rule; the arms are per instance.

use aether_sim_ecology::ecology::{
    Ecology, DECO, FORK_BUDGET_GRANT, FORK_BUDGET_GROWTH_ON, FORK_BUDGET_MIN, FORK_GRACE_ON,
    SPECIATION_MIN_SIDE_DEFAULT,
};
use aether_sim_ecology::save_state::{SaveState, SAVE_STATE_VERSION};

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in from..=to {
        eco.update(0.1, daylight(t));
    }
}

#[test]
fn the_default_world_keeps_the_phase49_grant_rule() {
    // Phase 64 reverted the floor to the Phase 49/53 chain after the
    // out-of-sample run (`research/phase64-report.md`): from a DECO root,
    // 80 → 20 + 60 → 15 + 45 → 11 + 34 → 8 + 26 → 8 + 18 → 8 + 10.
    assert_eq!(Ecology::fork_budget_grant(80), Some(20));
    assert_eq!(Ecology::fork_budget_grant(60), Some(15));
    assert_eq!(Ecology::fork_budget_grant(45), Some(11));
    assert_eq!(Ecology::fork_budget_grant(34), Some(8));
    assert_eq!(Ecology::fork_budget_grant(24), Some(8));
    assert_eq!(Ecology::fork_budget_grant(18), Some(8));
    assert_eq!(Ecology::fork_budget_grant(15), None, "below 2 × FORK_BUDGET_MIN");
    let eco = Ecology::new(20261102);
    assert_eq!(eco.fork_budget_min, FORK_BUDGET_MIN);
    assert_eq!(eco.fork_daughter_min, FORK_BUDGET_MIN);
    assert_eq!(eco.speciation_min_side, SPECIATION_MIN_SIDE_DEFAULT);
    assert_eq!(eco.budget_growth, FORK_BUDGET_GROWTH_ON);
    assert_eq!(eco.budget_grace, FORK_GRACE_ON);
    assert_eq!(SAVE_STATE_VERSION, 24);
}

#[test]
fn floor_arms_are_per_instance_and_not_archived() {
    let mut eco = Ecology::new(20261102);
    eco.fork_budget_min_set(12);
    assert_eq!((eco.fork_budget_min, eco.fork_daughter_min), (12, 12));
    assert_eq!(Ecology::fork_budget_grant_with(23, eco.fork_budget_min, eco.fork_daughter_min), None);
    eco.fork_budget_min_set(FORK_BUDGET_MIN);
    eco.fork_budget_daughter_min_set(12);
    assert_eq!(
        Ecology::fork_budget_grant_with(23, eco.fork_budget_min, eco.fork_daughter_min),
        Some(12),
        "daughter-only floor lets a 23-slot parent mint a 12-slot daughter and keep 11"
    );
    eco.fork_budget_min_set(12);
    assert_eq!(Ecology::fork_budget_grant_with(23, 12, 12), None, "the symmetric 12+12 floor refuses");
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let restored = Ecology::from_save_state(serde_json::from_str::<SaveState>(&json).unwrap());
    assert_eq!(restored.fork_budget_min, FORK_BUDGET_MIN, "floors are arm switches, not archive state");
    assert_eq!(restored.fork_daughter_min, FORK_BUDGET_MIN);
    eco.fork_budget_floors_reset();
    assert_eq!(eco.fork_budget_min, FORK_BUDGET_MIN);
    assert_eq!(eco.kind_budget(DECO), 80);
    assert_eq!(FORK_BUDGET_GRANT, 20);
}

#[test]
fn growth_state_rides_the_archive() {
    let mut eco = Ecology::new(20261102);
    eco.fork_budget_growth_on();
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    assert!(json.contains("\"BudgetGrowthOn\":true"));
    let restored = Ecology::from_save_state(serde_json::from_str::<SaveState>(&json).unwrap());
    assert!(restored.budget_growth, "a restored world keeps growing its daughters");
    assert!(restored.kind_growth_grants.is_empty());

    let mut legacy: serde_json::Value = serde_json::from_str(&json).unwrap();
    legacy.as_object_mut().unwrap().remove("BudgetGrowthOn");
    legacy.as_object_mut().unwrap().remove("KindGrowthGrants");
    legacy["Version"] = serde_json::json!(22);
    let legacy = Ecology::from_save_state(serde_json::from_value::<SaveState>(legacy).unwrap());
    assert!(!legacy.budget_growth, "pre-v23 archives land on the compiled default");
}

/// The growth pass is a pure bookkeeping no-op before the first fork: with no
/// derived kind to pay, an armed world must be bit-identical to the default
/// (no RNG draws, no budget moves).
#[test]
fn the_growth_pass_is_a_no_op_before_the_first_fork() {
    let mut a = Ecology::new(20261102);
    let mut b = Ecology::new(20261102);
    b.fork_budget_growth_on();
    run_ticks(&mut a, 1, 600);
    run_ticks(&mut b, 1, 600);
    assert_eq!(a.checksum(), b.checksum());
    assert_eq!(b.kind_budget(DECO), 80);
}

/// Phase 63 promoted M10, Phase 64's out-of-sample run reverted it (floor 8).
/// A double run and a save/restore hop at 60 000 ticks must be bit-exact, and
/// the guild total must stay 80 through both. Ignored — minutes in release.
#[test]
#[ignore = "double 60k run + save/restore lockstep — minutes in release"]
fn default_world_double_run_and_roundtrip_is_bit_exact() {
    let mut a = Ecology::new(20261102);
    let mut b = Ecology::new(20261102);
    run_ticks(&mut a, 1, 60_000);
    run_ticks(&mut b, 1, 60_000);
    assert_eq!(a.checksum(), b.checksum(), "double run");
    assert_eq!(a.family_budget_total(DECO), 80);
    let json = serde_json::to_string(&a.to_save_state()).unwrap();
    let mut restored = Ecology::from_save_state(serde_json::from_str::<SaveState>(&json).unwrap());
    run_ticks(&mut a, 60_001, 61_200);
    run_ticks(&mut restored, 60_001, 61_200);
    assert_eq!(a.checksum(), restored.checksum(), "archive roundtrip lockstep");
    assert_eq!(a.family_budget_total(DECO), 80);
}
