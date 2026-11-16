//! Phase 31+ golden regression anchors — Phase 34 re-baselined, Phase 36
//! updated again (INFO_COST_ON + TROPHIC_EFF active changes the dynamics vs
//! the ledger-only Phase 35 skeleton), Phase 39.1 re-baselined a third time
//! (group entities + task-switching cost + dirty-work mutagenesis), and
//! **Phase 40.1 re-baselined a fourth time** — the 100k-tick observatory
//! found the density brake pushing the HERB reproduction threshold above
//! 1.0×max_energy at saturation (an absorbing zero-birth state → one-cohort
//! ageing → extinction). `REPRO_THRESHOLD_MAX = 0.90` caps the brake.
//!
//! PHASE 42 ANCHORS (previous; **eighth re-baseline** — the triune brain takes
//! behaviour authority once its own FSM-agreement gate + confidence floor is
//! met, its arbitration actions are bridged into FSM states/motion intent, and
//! offline replay consolidates the world model). Replay also shifts the shadow
//! arm, so even `brain_switch_enabled=false` moves off the 41.1 values.
//!   seed 20261102 @300  v2 0xdc98f7d34a7af777
//!   seed 20261102 @3000 v2 0xafa833641ebfd5a8
//!   seed 20261105 @300  v2 0x2e9539930abd8975
//!   seed 20261102 @300  v1 0xe6404a7bee633eb8
//!   (shadow-only @300 for the A/B arm: v2 0xf31c4501f764be29)
//! PHASE 45 ANCHORS (current; **tenth re-baseline** — patch disturbance +
//! static regrowth-noise heterogeneity + cultural value selection. All four
//! anchors move: the fixed-seed noise field changes grass growth from tick 1,
//! and the patch layer fires within the @3000 window on these seeds).
//!   seed 20261102 @300  v2 0xa68939c089f4207f
//!   seed 20261102 @3000 v2 0xda91fbd0e1d3d570
//!   seed 20261105 @300  v2 0x009ddda639c5c149
//!   seed 20261102 @300  v1 0xa774fb2033268005
//! PHASE 43 ANCHORS (previous; **ninth re-baseline** — the cultural ratchet:
//! birth-time token inheritance. @300 was UNCHANGED from Phase 42 — no de-novo
//! invention had fired by tick 300, so teaching was a bit-exact no-op there;
//! only the @3000 long-run anchor moved, where inventions already exist and
//! children now inherit them).
//!   seed 20261102 @300  v2 0xdc98f7d34a7af777  (= Phase 42)
//!   seed 20261102 @3000 v2 0x9af70d3dc96532f2
//!   seed 20261105 @300  v2 0x2e9539930abd8975  (= Phase 42)
//!   seed 20261102 @300  v1 0xe6404a7bee633eb8  (= Phase 42)
//! PHASE 41.1 ANCHORS (previous; Phase 41 triune brain + the 41.1 brain
//! rhythm: each agent runs a full triune update every other tick, staggered
//! by index parity, halving per-tick planning cost for the 16 ms wasm gate;
//! upkeep is still charged every tick):
//!   seed 20261102 @300  v2 0x1265fe37e680e828
//!   seed 20261102 @3000 v2 0x0d10cdeaf3e447c0
//!   seed 20261105 @300  v2 0x0c1a5b2254ef521e
//!   seed 20261102 @300  v1 0x490a0a05d97ba7d3
//! PHASE 40.2 ANCHORS (previous):
//!   seed 20261102 @300  v2 0xbbee72352ef4468d
//!   seed 20261102 @3000 v2 0x23aca66c0c68ba69
//!   seed 20261105 @300  v2 0xfa11ea50ba7ef71a
//!   seed 20261102 @300  v1 0xfb8827f6b1b34b5a
//! PHASE 40.1 ANCHORS (previous):
//!   seed 20261102 @3000 v2 0x00bd6cd1892d6c10
//! PHASE 39.1 ANCHORS (previous, kept for the record):
//!   seed 20261102 @300  v2 0x32c496b54efc9626
//!   seed 20261102 @3000 v2 0xb900f6366f38cc47
//!   seed 20261105 @300  v2 0x786979c7e9c84a93
//!   seed 20261102 @300  v1 0x5fa905f98eb6546e
//! PHASE 36 ANCHORS (previous, kept for the record):
//!   seed 20261102 @300  v2 0xa4d54e1aadbe83c6
//!   seed 20261102 @3000 v2 0xdd79b5ab74f3545d
//!   seed 20261105 @300  v2 0x777cd6c33bb3345c
//!   seed 20261102 @300  v1 0x3782a36aee6f1ff5
//! The Phase 36 values are still reachable bit-exactly by switching the group
//! layer off (`eco.group_on = false`) — pinned in
//! `phase39_1_transition_test::group_off_is_bit_exact_phase36_anchor`.
//!
//! Phase 31 pinned the v1 checksum (kind field = the kind NAME's first byte).
//! Phase 34 (Q2 ruling) promotes the v2 checksum to primary: the kind field
//! is the registry id (u32) and the per-agent ext-locus hash is mixed in.
//! `Ecology::checksum()` now returns the v2 value; `checksum_v1()` is kept as
//! the read-only historical accessor.
//!
//! RE-BASELINE (recorded in the Phase 36 report + commit): the approved
//! dynamics changes (INFO_COST_ON=true paying for perception sweeps +
//! TROPHIC_EFF=true gating live-prey → consumer biomass at Lindeman 0.10)
//! shift the long-run trajectory again, so the pinned anchors move to the
//! v2 values below. The OLD Phase 35 anchors are kept as comments:
//!   seed 20261102 @300  v2 0x785889bb1ec0737f  (Phase 35 / Phase 34 anchor)
//!   seed 20261102 @3000 v2 0x6dbb8f6fd216f353
//!   seed 20261105 @300  v2 0xf53762eefd0cf3d2
//!   seed 20261102 @300  v1 0x790184996f3acefd  (historical, pre-Phase-34)
//!
//! Phase 36 keeps the dual-controller gate OFF by default (BRAIN_SWITCH_ENABLED=false)
//! so behaviour is still FSM-driven, but the deterministic shadow network runs
//! every tick with zero extra RNG draws (the main/prng/fish streams stay
//! aligned). The net effect comes from two sources of energy change:
//! * INFO_COST_PER_READ × sense² per perception sweep (≈ 0.002 × (sense/80)²)
//! * TROPHIC_EFF ≈ 0.10 multiplier on live-prey conversion in PRED bite gains

use aether_sim_ecology::ecology::Ecology;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, to: u32) {
    for t in 1..=to {
        eco.update(0.1, daylight(t));
    }
}

/// Primary (v2) checksum — the Phase 34 re-baseline anchor.
fn checksum_at(seed: u32, ticks: u32) -> String {
    let mut eco = Ecology::new(seed);
    run_ticks(&mut eco, ticks);
    eco.checksum()
}

#[test]
fn golden_seed_20261102_300_ticks() {
    // Phase 45 re-baseline (patch disturbance + regrowth-noise field + value selection).
    let ck = checksum_at(20261102, 300);
    assert_eq!(ck, "0xa68939c089f4207f",
        "300-tick v2 checksum drifted: {} — see file docs + Phase 45 report", ck);
}

#[test]
fn golden_seed_20261102_3000_ticks() {
    // Phase 45 re-baseline (patch events fire inside this window; the noise
    // field shifts grass growth from tick 1).
    let ck = checksum_at(20261102, 3000);
    assert_eq!(ck, "0xda91fbd0e1d3d570",
        "3000-tick v2 long-run checksum drifted: {} — if this change is an approved re-baseline, pin the new value here and record it in the phase report", ck);
}

#[test]
fn golden_seed_20261105_300_ticks() {
    // Phase 45 re-baseline.
    let ck = checksum_at(20261105, 300);
    assert_eq!(ck, "0x009ddda639c5c149",
        "alt-seed v2 checksum drifted: {} — if this change is an approved re-baseline, pin the new value here and record it in the phase report", ck);
}

/// Phase 40.1: the reproduction-threshold cap shifts the v1 stream too
/// (the brake sits in `reproduce`, which runs on the main RNG path).
#[test]
fn golden_300_tick_trajectory_bit_exact_to_v1() {
    let mut eco = Ecology::new(20261102);
    for t in 1..=300 { eco.update(0.1, daylight(t)); }
    assert_eq!(eco.checksum_v1(), "0xa774fb2033268005",
        "the 300-tick v1 trajectory must stay at the pinned Phase 45 anchor");
}

/// Determinism guard: same seed + same ticks must be bit-identical across
/// two fresh instances (protects against RNG-stream or iteration-order
/// regressions the checksum anchors alone might miss at a single point).
#[test]
fn golden_double_run_bit_identical() {
    let a = checksum_at(20261102, 500);
    let b = checksum_at(20261102, 500);
    assert_eq!(a, b, "double run diverged");
}

/// Phase 34 (Q2 ruling): v2 is now PRIMARY. `checksum()` and
/// `checksum_v2()` are the same value; v2 is deterministic on its own and
/// differs from the historical v1 (registry-id kind field + ext hash vs the
/// name-byte kind field).
#[test]
fn checksum_v2_is_primary_and_deterministic() {
    let mut a = Ecology::new(20261102);
    let mut b = Ecology::new(20261102);
    for t in 1..=300 { a.update(0.1, daylight(t)); b.update(0.1, daylight(t)); }
    assert_eq!(a.checksum_v2(), b.checksum_v2(), "v2 double run diverged");
    // v2 promoted to primary: checksum() == checksum_v2().
    assert_eq!(a.checksum(), a.checksum_v2(), "checksum() must equal checksum_v2() (v2 is primary)");
    // v2 differs from the historical v1 on a populated world.
    assert_ne!(a.checksum_v2(), a.checksum_v1(), "v2 must differ from v1 on a populated world");
}

/// Phase 31 Part B: tick cost baseline for the Phase 38 measurement gate.
/// Run with `--include-ignored` to record the number; machine-dependent, so
/// it only prints (no assert).
#[test]
#[ignore]
fn tick_cost_baseline() {
    let mut eco = Ecology::new(20261102);
    // warmup (JIT/alloc) — not measured
    for t in 1..=100 { eco.update(0.1, daylight(t)); }
    let n_ticks = 300u32;
    let start = eco.tick_count();
    let t0 = std::time::Instant::now();
    for _ in 0..n_ticks { eco.update(0.1, 1.0); }
    let elapsed = t0.elapsed();
    let per_tick_us = elapsed.as_nanos() as f64 / n_ticks as f64 / 1e3;
    let agents = eco.agents.len();
    eprintln!(
        "tick_cost_baseline seed=20261102 from_tick={} to_tick={} agents={} per_tick={:.1}us",
        start, eco.tick_count(), agents, per_tick_us
    );
}
