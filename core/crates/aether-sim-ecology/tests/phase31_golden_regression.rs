//! Phase 31+ golden regression anchors — Phase 34 re-baselined.
//!
//! Phase 31 pinned the v1 checksum (kind field = the kind NAME's first byte).
//! Phase 34 (Q2 ruling) promotes the v2 checksum to primary: the kind field
//! is the registry id (u32) and the per-agent ext-locus hash is mixed in.
//! `Ecology::checksum()` now returns the v2 value; `checksum_v1()` is kept as
//! the read-only historical accessor.
//!
//! RE-BASELINE (recorded in the Phase 34 report + commit): the approved
//! dynamics changes (神之手 retirement, the legacy HERB script retirement,
//! kind-forking's density-brake feedback) shift the long-run trajectory, so
//! the pinned anchors move to the v2 values below. The OLD v1 anchors are
//! kept in the comments as the historical record:
//!   seed 20261102 @300  v1 0x790184996f3acefd  (Phase 16/17/18 anchor)
//!   seed 20261102 @3000 v1 0x58733db4ab0c3ca0
//!   seed 20261105 @300  v1 0x48d66bf801a9f827
//!
//! A key Phase 34 invariant the 300-tick run pins: the *trajectory* up to
//! tick 300 is bit-exact to the pre-Phase-34 engine (v1 unchanged), so the
//! v2 value there differs from v1 only by the kind-field + ext-hash
//! definition, not by dynamics. The 3000-tick run is where the re-baseline
//! (a genuinely different trajectory) shows up.
//!
//! Phase 35: TROPHIC_EFF is a placeholder (parameter hook, NOT yet active
//! in dynamics — see `TROPHIC_EFF` in ecology.rs). The energy ledger is
//! pure bookkeeping, and `mte_size_scale` is a ×1.0 identity when `MTE_ON`
//! is false, so the Phase 34 anchors remain exact.

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
    // Phase 34 v2 anchor (unchanged: Phase 35 ledger is bookkeeping-only).
    let ck = checksum_at(20261102, 300);
    assert_eq!(ck, "0x785889bb1ec0737f",
        "300-tick v2 checksum drifted: {} — see file docs + Phase 34 report", ck);
}

#[test]
fn golden_seed_20261102_3000_ticks() {
    // Phase 34 v2 anchor (unchanged: Phase 35 is bookkeeping-only).
    let ck = checksum_at(20261102, 3000);
    assert_eq!(ck, "0x6dbb8f6fd216f353",
        "3000-tick v2 long-run checksum drifted: {} — if this change is an approved re-baseline, pin the new value here and record it in the phase report", ck);
}

#[test]
fn golden_seed_20261105_300_ticks() {
    // Phase 34 v2 anchor (unchanged: Phase 35 is bookkeeping-only).
    let ck = checksum_at(20261105, 300);
    assert_eq!(ck, "0xf53762eefd0cf3d2",
        "alt-seed v2 checksum drifted: {} — if this change is an approved re-baseline, pin the new value here and record it in the phase report", ck);
}

/// Phase 34 invariant: the 300-tick TRAJECTORY is bit-exact to the
/// pre-Phase-34 engine. The v1 checksum (name-byte kind field, no ext hash)
/// still reads the historical anchor — proving the 300-tick dynamics are
/// unchanged and only the checksum DEFINITION (v1→v2) moved the 300-tick
/// value. Phase 35 keeps this (ledger is bookkeeping-only).
#[test]
fn golden_300_tick_trajectory_bit_exact_to_v1() {
    let mut eco = Ecology::new(20261102);
    for t in 1..=300 { eco.update(0.1, daylight(t)); }
    assert_eq!(eco.checksum_v1(), "0x790184996f3acefd",
        "the 300-tick v1 trajectory must remain bit-exact to the historical anchor");
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
