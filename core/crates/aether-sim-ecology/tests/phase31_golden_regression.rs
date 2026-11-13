//! Phase 31 golden regression anchors.
//!
//! The Phase 31 generalization refactor (KindDef registry, `unreachable!()`
//! retirement, per-kind Vec stats, spatial index) must be a bit-exact no-op:
//! every checksum below is computed with the CURRENT engine at the pinned
//! (seed, tick-count) checkpoints and must not change as constants move from
//! `match` arms into `KindDef` data.
//!
//! `0x790184996f3acefd` (the Phase 16/17/18 historical anchor) — recipe
//! recovered during Phase 31 baseline capture: seed 20261102, 300 ticks,
//! diurnal daylight below. The long-run and alt-seed anchors re-pin the
//! contract at additional documented checkpoints. Values are printed on
//! failure so a re-baseline is a one-line edit with the new value recorded in
//! the phase report.

use aether_sim_ecology::ecology::Ecology;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, to: u32) {
    for t in 1..=to {
        eco.update(0.1, daylight(t));
    }
}

fn checksum_at(seed: u32, ticks: u32) -> String {
    let mut eco = Ecology::new(seed);
    run_ticks(&mut eco, ticks);
    eco.checksum()
}

#[test]
fn golden_seed_20261102_300_ticks() {
    let ck = checksum_at(20261102, 300);
    assert_eq!(ck, "0x790184996f3acefd",
        "300-tick checksum drifted (the Phase 16/17/18 historical anchor): {} — see file docs", ck);
}

#[test]
fn golden_seed_20261102_3000_ticks() {
    let ck = checksum_at(20261102, 3000);
    assert_eq!(ck, "0x58733db4ab0c3ca0",
        "3000-tick long-run checksum drifted: {} — if this change is an approved re-baseline, pin the new value here and record it in the phase report", ck);
}

#[test]
fn golden_seed_20261105_300_ticks() {
    let ck = checksum_at(20261105, 300);
    assert_eq!(ck, "0x48d66bf801a9f827",
        "alt-seed checksum drifted: {} — if this change is an approved re-baseline, pin the new value here and record it in the phase report", ck);
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

/// Phase 31 Part B: v2 checksum (kind field = registry id instead of the
/// name's first byte) is a parallel export — deterministic on its own and
/// distinct from v1 for a populated world. v1 stays the pinned anchor;
/// Phase 34 promotes v2 (plus locus hashes) to primary.
#[test]
fn checksum_v2_parallel_deterministic() {
    let mut a = Ecology::new(20261102);
    let mut b = Ecology::new(20261102);
    for t in 1..=300 { a.update(0.1, daylight(t)); b.update(0.1, daylight(t)); }
    assert_eq!(a.checksum_v2(), b.checksum_v2(), "v2 double run diverged");
    assert_ne!(a.checksum_v2(), a.checksum(), "v2 must differ from v1 on a populated world");
    // v1 stays pinned: the 300-tick anchor is unchanged by the parallel export.
    assert_eq!(a.checksum(), "0x790184996f3acefd");
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
