//! Phase 32 open-locus (variable-length genome) acceptance tests.
//!
//! The contract this phase pins:
//!   1. `genesis_on == false` (default) is a bit-exact no-op — covered by the
//!      Phase 31 golden regression anchors (0x790184996f3acefd @300,
//!      0x58733db4ab0c3ca0 @3000, alt-seed, double-run). The sentinel test
//!      below additionally guards that ext stays empty with genesis off.
//!   2. `genesis_on == true` is deterministic: same seed, two fresh
//!      instances, identical ext locus totals and duplication-event counts.
//!   3. OEE condition ①: a new heritable trait can appear (a de-novo locus),
//!      so some fraction of agents end up carrying a non-empty ext.
//!   4. Expression upkeep is an ECONOMIC cap: over a long genesis-ON run the
//!      per-agent ext length stays bounded (does not run away), and we record
//!      the typical maximum.
//!   5. Variable-length ext survives a save/restore JSON roundtrip
//!      bit-exactly (checksums + per-locus lists identical).
//!
//! Daylight convention matches the other ecology tests: tick t receives
//! `0.5 + 0.5*sin(t/300 * PI)` with t counted from 1.

use aether_sim_ecology::ecology::{Ecology, Locus};
use aether_sim_ecology::save_state;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in (from + 1)..=to {
        eco.update(0.1, daylight(t));
    }
}

/// (agent id, ext locus list) for every alive agent, in a stable id-sorted
/// order — used to compare per-locus ext across two sims.
fn ext_by_id(eco: &Ecology) -> Vec<(u32, Vec<Locus>)> {
    let mut v: Vec<(u32, Vec<Locus>)> = eco
        .living()
        .iter()
        .filter(|a| a.alive)
        .map(|a| (a.id, a.genome.ext.clone()))
        .collect();
    v.sort_by_key(|(id, _)| *id);
    v
}

/// Sentinel for acceptance #1: with genesis OFF (the default) the open-locus
/// operator never fires — zero ext slots, zero duplication events, and the
/// 300-tick checksum is still the pinned golden anchor. Phase 34: `checksum()`
/// is the v2 primary; the 300-tick trajectory is bit-exact so the anchor is
/// the Phase 34 v2 value (the historical v1 value 0x790184996f3acefd is
/// recorded in phase31_golden_regression).
#[test]
fn genesis_off_is_bit_exact_noop() {
    let mut eco = Ecology::new(20261102);
    // default genesis_on == false; assert the whole run keeps ext empty.
    run_ticks(&mut eco, 0, 300);
    assert!(eco.genesis_on == false);
    assert_eq!(eco.ext_dup_events, 0, "genesis off must never fire a dup event");
    assert_eq!(eco.ext_total_loci(), 0, "genesis off must keep ext empty");
    // The pinned golden anchor (Phase 35 v2 primary) reflects TROPHIC_EFF
    // wiring into predation gains; the trajectory itself shifted from
    // Phase 34.
    assert_eq!(eco.checksum(), "0xa4d54e1aadbe83c6");
}

/// Acceptance #3: determinism under genesis ON. Two fresh instances with the
/// same seed + set_genesis(true) must produce identical ext totals and
/// duplication-event counts after ~500 ticks.
#[test]
fn genesis_on_double_run_bit_exact() {
    const SEED: u32 = 20261102;
    const TICKS: u32 = 500;

    let mut a = Ecology::new(SEED);
    a.set_genesis(true);
    let mut b = Ecology::new(SEED);
    b.set_genesis(true);
    run_ticks(&mut a, 0, TICKS);
    run_ticks(&mut b, 0, TICKS);

    assert_eq!(a.ext_dup_events, b.ext_dup_events, "dup-event count diverged");
    assert_eq!(a.ext_total_loci(), b.ext_total_loci(), "ext locus total diverged");
    // The full per-agent ext map must agree too (stronger than the totals).
    assert_eq!(ext_by_id(&a), ext_by_id(&b), "per-agent ext lists diverged");
    // And the checksum itself is deterministic under genesis ON.
    assert_eq!(a.checksum(), b.checksum(), "genesis-on checksum diverged");
    // Sanity: with genesis ON and enough births, at least something happened.
    assert!(a.ext_dup_events > 0, "expected some duplication events by t={}", TICKS);
}

/// Acceptance #4 (OEE condition ①): a new heritable trait must be able to
/// appear. With genesis ON over a long run, at least one seed yields a
/// nonzero fraction of agents carrying a non-empty ext list.
#[test]
fn oee_condition1_new_trait_emerges() {
    const TICKS: u32 = 2000;
    let seeds = [20261102u32, 20261105u32, 20261107u32, 20261109u32, 20261113u32];
    let mut any_emerged = false;
    for &seed in &seeds {
        let mut eco = Ecology::new(seed);
        eco.set_genesis(true);
        run_ticks(&mut eco, 0, TICKS);
        let share = eco.ext_bearer_share();
        eprintln!(
            "OEE① seed={} t={} bearer_share={:.4} total_loci={} dup_events={}",
            seed, TICKS, share, eco.ext_total_loci(), eco.ext_dup_events
        );
        if share > 0.0 {
            any_emerged = true;
        }
    }
    assert!(
        any_emerged,
        "OEE condition ① failed: no seed produced any agent with a new heritable ext locus"
    );
}

/// Acceptance #5 (expression-cost cap): over a long genesis-ON run the
/// per-agent ext length must stay bounded (suppressed by the metabolic
/// upkeep), not run away toward unbounded growth. Records the typical max.
#[test]
fn upkeep_caps_ext_growth() {
    const SEED: u32 = 20261102;
    const TICKS: u32 = 3000;
    // Guard cap: observed peak per-agent ext over 3000 ticks is ~2 (duplication
    // is a rare per-birth event, P_DUP=0.06 vs P_DEL=0.02). A runaway (e.g. a
    // broken deletion branch or removed upkeep) would push the per-agent max
    // well past 20; the expression upkeep keeps it firmly bounded.
    const MAX_EXT_CAP: usize = 20;

    let mut eco = Ecology::new(SEED);
    eco.set_genesis(true);
    let mut peak = 0usize;
    // Sample the population max every 200 ticks to observe the trajectory.
    for t in (0..=TICKS).step_by(200) {
        if t > 0 {
            run_ticks(&mut eco, t - 200, t);
        }
        let m = eco.ext_max_len();
        peak = peak.max(m);
        eprintln!("upkeep t={} ext_max={} total={}", t, m, eco.ext_total_loci());
    }
    eprintln!("upkeep peak ext len over t={}: {}", TICKS, peak);
    assert!(
        peak <= MAX_EXT_CAP,
        "ext growth ran away (peak {} > cap {}): the expression upkeep did not suppress it",
        peak, MAX_EXT_CAP
    );
    // Also confirm the cap is on a per-agent basis, not the total.
    assert!(
        eco.ext_max_len() <= MAX_EXT_CAP,
        "terminal per-agent ext len {} exceeds cap {}",
        eco.ext_max_len(), MAX_EXT_CAP
    );
}

/// Acceptance #6: variable-length ext survives a save/restore JSON roundtrip
/// bit-exactly. Run genesis ON until some agents carry ext, save at that tick,
/// restore, and continue both in lockstep: checksums agree AND every agent's
/// ext list is identical per locus.
#[test]
fn save_restore_variable_ext_bit_exact() {
    const SEED: u32 = 20261102;
    const SAVE_AT: u32 = 1200;

    let mut eco = Ecology::new(SEED);
    eco.set_genesis(true);
    run_ticks(&mut eco, 0, SAVE_AT);
    // Require that the save actually carries some ext loci (otherwise this
    // test is vacuous). If the chosen seed/tick has none, extend the run.
    let mut extra = 0u32;
    while eco.ext_total_loci() == 0 && extra < 2000 {
        run_ticks(&mut eco, SAVE_AT + extra, SAVE_AT + extra + 1);
        extra += 1;
    }
    let save_tick = SAVE_AT + extra;
    let loci_at_save = eco.ext_total_loci();
    eprintln!(
        "save/restore t={} total_loci={} dup_events={}",
        save_tick, loci_at_save, eco.ext_dup_events
    );
    assert!(loci_at_save > 0, "need some ext loci present at the save tick to test roundtrip");

    // Save through real JSON so the serde roundtrip is part of the guarantee.
    let state = eco.to_save_state();
    let json = serde_json::to_string(&state).expect("serialize SaveState v6");
    assert!(json.contains("\"ExtLoci\""), "v6 save must carry the per-agent ExtLoci array");
    let parsed: save_state::SaveState = serde_json::from_str(&json).expect("deserialize SaveState v6");
    assert!(parsed.GenesisOn, "GenesisOn must survive the roundtrip");
    assert_eq!(parsed.ExtDupEvents, eco.ext_dup_events, "ExtDupEvents must survive the roundtrip");

    let mut eco2 = Ecology::from_save_state(parsed);
    assert!(eco2.genesis_on, "restored sim must keep genesis ON");
    assert_eq!(eco2.ext_dup_events, eco.ext_dup_events, "dup counter must restore");

    // Immediate per-locus comparison at the save tick.
    assert_eq!(ext_by_id(&eco), ext_by_id(&eco2), "ext lists diverged immediately after restore");
    assert_eq!(eco.checksum(), eco2.checksum(), "checksum diverged at restore");

    // Continue both in lockstep; everything must stay identical.
    for target in [save_tick + 200u32, save_tick + 500u32] {
        run_ticks(&mut eco, save_tick, target);
        run_ticks(&mut eco2, save_tick, target);
        assert_eq!(eco.checksum(), eco2.checksum(), "checksum diverged at t={}", target);
        assert_eq!(ext_by_id(&eco), ext_by_id(&eco2), "ext lists diverged at t={}", target);
    }
}

/// C-ABI observability helper (research-only readout): the per-agent ext
/// locus count is exposed and reads 0 with genesis OFF.
#[test]
fn genesis_off_default_ext_empty_sentinel() {
    let eco = Ecology::new(20261102);
    assert!(!eco.genesis_on);
    assert_eq!(eco.ext_total_loci(), 0);
    assert_eq!(eco.ext_max_len(), 0);
    assert_eq!(eco.ext_bearer_share(), 0.0);
}
