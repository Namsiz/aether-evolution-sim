//! Phase 6 save/restore acceptance tests.
//!
//! The core guarantee: saving at any tick N and restoring must continue the
//! simulation bit-exactly — checksums, censuses, grass/mineral fields, and
//! the full internal save-state JSON all stay identical to the never-restored
//! sim. Without this, players cannot trust a restored world.
//!
//! Daylight convention matches the other ecology tests (deco_cycle_test,
//! structural_alignment): tick t receives `0.5 + 0.5*sin(t/300 * PI)` with
//! t counted from 1.

use aether_sim_ecology::ecology::{DECO, Ecology, FISH, HERB, OMN, PRED, SCAV};
use aether_sim_ecology::save_state;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in (from + 1)..=to {
        eco.update(0.1, daylight(t));
    }
}

fn census(eco: &Ecology) -> [usize; 6] {
    [
        eco.living().iter().filter(|a| a.alive && a.kind == HERB).count(),
        eco.living().iter().filter(|a| a.alive && a.kind == PRED).count(),
        eco.living().iter().filter(|a| a.alive && a.kind == SCAV).count(),
        eco.living().iter().filter(|a| a.alive && a.kind == OMN).count(),
        eco.living().iter().filter(|a| a.alive && a.kind == FISH).count(),
        eco.living().iter().filter(|a| a.alive && a.kind == DECO).count(),
    ]
}

/// Save via real JSON (serialize → deserialize) so the serde roundtrip itself
/// is part of the bit-exact guarantee, not just the in-memory conversion.
fn save_through_json(eco: &Ecology) -> (save_state::SaveState, String) {
    let state = eco.to_save_state();
    let json = serde_json::to_string(&state).expect("serialize SaveState");
    let parsed: save_state::SaveState = serde_json::from_str(&json).expect("deserialize SaveState");
    (parsed, json)
}

#[test]
fn save_at_t100_restore_matches_to_t300_bit_exact() {
    const SEED: u32 = 20261102;
    const SAVE_AT: u32 = 100;

    let mut eco = Ecology::new(SEED);
    run_ticks(&mut eco, 0, SAVE_AT);

    let (state, _json) = save_through_json(&eco);
    assert_eq!(state.version, save_state::SAVE_STATE_VERSION);
    assert_eq!(state.schema_name, save_state::SAVE_STATE_SCHEMA_NAME);
    assert_eq!(state.tick_count, SAVE_AT);

    let mut eco2 = Ecology::from_save_state(state);

    // Lockstep to t=200 and t=300: checksum + census + save-state JSON must be
    // identical at every checkpoint. The save-state JSON covers RNG streams,
    // isolated groups, evolution/fitness history and every private counter —
    // stronger than the checksum alone.
    let mut checksums_a = Vec::new();
    let mut checksums_b = Vec::new();
    let mut census_a = Vec::new();
    let mut census_b = Vec::new();
    for target in [200u32, 300u32] {
        run_ticks(&mut eco, SAVE_AT, target);
        run_ticks(&mut eco2, SAVE_AT, target);
        let ca = eco.checksum();
        let cb = eco2.checksum();
        checksums_a.push(ca.clone());
        checksums_b.push(cb.clone());
        assert_eq!(ca, cb, "checksum diverged at t={}", target);

        let na = census(&eco);
        let nb = census(&eco2);
        census_a.push(na);
        census_b.push(nb);
        assert_eq!(na, nb, "six-species census diverged at t={}", target);

        let ja = serde_json::to_string(&eco.to_save_state()).unwrap();
        let jb = serde_json::to_string(&eco2.to_save_state()).unwrap();
        assert_eq!(ja, jb, "full save-state JSON diverged at t={}", target);
    }

    // Direct field checks at t=300: pools the checksum does not see.
    assert_eq!(eco.mineral_field(), eco2.mineral_field(), "mineral_field");
    assert_eq!(eco.carcasses.len(), eco2.carcasses.len(), "carcass count");
    for (ca, cb) in eco.carcasses.iter().zip(eco2.carcasses.iter()) {
        assert_eq!(ca.x.to_bits(), cb.x.to_bits());
        assert_eq!(ca.z.to_bits(), cb.z.to_bits());
        assert_eq!(ca.energy.to_bits(), cb.energy.to_bits());
        assert_eq!(ca.age.to_bits(), cb.age.to_bits());
    }
    assert_eq!(eco.grass(), eco2.grass(), "grass");
    for sp in 0..3 {
        assert_eq!(eco.grass_p(sp), eco2.grass_p(sp), "grass_p[{sp}]");
        assert_eq!(eco.suit_eff_by(sp), eco2.suit_eff_by(sp), "suit_eff_by[{sp}]");
    }
    assert_eq!(eco.suit(), eco2.suit(), "suit");
    assert_eq!(eco.moist(), eco2.moist(), "moist");
    assert_eq!(eco.shade(), eco2.shade(), "shade");

    eprintln!("checksums t=200/t=300: {:?}", checksums_a);
    eprintln!("censuses t=200/t=300 (restored): {:?}", census_b);
    // The restored sim must keep running stably — all six species alive at t=300.
    assert!(census_b.last().unwrap().iter().all(|&n| n > 0), "species extinct after restore");
}

#[test]
fn save_at_t0_restore_matches_never_saved_run_300_ticks() {
    const SEED: u32 = 20261102;

    let mut eco = Ecology::new(SEED);
    let (state, _json) = save_through_json(&eco);
    let mut eco2 = Ecology::from_save_state(state);

    run_ticks(&mut eco, 0, 300);
    run_ticks(&mut eco2, 0, 300);

    let ca = eco.checksum();
    let cb = eco2.checksum();
    assert_eq!(ca, cb, "t=0 restore diverged from the never-saved sim");
    assert_eq!(census(&eco), census(&eco2));
    eprintln!("t=0-restore t=300 checksum: {} census: {:?}", ca, census(&eco));
}

#[test]
fn restore_rebuilds_internal_pools_and_history() {
    const SEED: u32 = 20261102;

    let mut eco = Ecology::new(SEED);
    run_ticks(&mut eco, 0, 200);

    let (state, _json) = save_through_json(&eco);
    // t=200 with DECO active: carcass pool and mineral field should be populated.
    let mineral_sum: f64 = state.mineral_field.iter().map(|&v| v as f64).sum();
    eprintln!("t=200 pre-restore mineral sum = {:.4}, carcasses = {}", mineral_sum, state.carcasses.len());
    let mut eco2 = Ecology::from_save_state(state);

    assert_eq!(eco.carcasses.len(), eco2.carcasses.len(), "carcass count mismatch at t=200");
    assert_eq!(eco.mineral_field(), eco2.mineral_field(), "mineral_field mismatch at t=200");
    assert_eq!(eco.grass(), eco2.grass(), "grass mismatch at t=200");
    assert_eq!(eco.tick_count(), eco2.tick_count());
    assert_eq!(eco.births, eco2.births);
    assert_eq!(eco.deaths, eco2.deaths);
    assert_eq!(eco.living().len(), eco2.living().len());
    // RNG streams restored: one more tick must produce identical draws.
    run_ticks(&mut eco, 200, 201);
    run_ticks(&mut eco2, 200, 201);
    assert_eq!(eco.checksum(), eco2.checksum(), "single-tick divergence after restore");

    let groups = eco.to_save_state().isolated_groups.len();
    assert_eq!(groups, eco2.to_save_state().isolated_groups.len(), "isolated_groups post-restore");
}

#[test]
fn save_state_schema_independent_of_render_packet() {
    let eco = Ecology::new(20261102);
    let state = eco.to_save_state();
    assert_eq!(state.version, save_state::SAVE_STATE_VERSION, "save-state version starts at {}", save_state::SAVE_STATE_VERSION);
    assert_eq!(state.schema_name, "aether-sim-save-state");
    assert!(!state.schema_name.contains("render"), "must not collide with render-packet naming");
    let json = serde_json::to_string(&state).unwrap();
    assert!(!json.contains("\"Meta\""), "save-state JSON must not embed render-packet Meta");
    assert!(json.contains("\"SchemaName\":\"aether-sim-save-state\""));
    // Phase 7: version is now 2; new fields present and sensible.
    assert!(json.contains(&format!("\"Version\":{}", save_state::SAVE_STATE_VERSION)));
    assert!(json.contains("\"DecoEffChoiceMode\":1"), "DECO selective mode should default to 1 (selective)");
}

// ---------------------------------------------------------------------------
// Diagnostic probe — disabled by default so it does not break acceptance.
// Enable with `cargo test -- --include-ignored` to locate first-divergence tick.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "diagnostic probe: do not count towards acceptance"]
fn _diag_dump_first_divergence_tick() {
    let seed = 20261102u32;
    let mut eco_a = Ecology::new(seed);
    for t in 1..=200 { eco_a.update(0.1, daylight(t)); }
    let ca_200 = eco_a.checksum();

    let mut eco_b = Ecology::new(seed);
    for t in 1..=100 { eco_b.update(0.1, daylight(t)); }
    let st = eco_b.to_save_state();
    let mut eco_c = Ecology::from_save_state(st);
    for t in 101..=200 { eco_c.update(0.1, daylight(t)); }
    let cc_200 = eco_c.checksum();

    eprintln!("=== POST-RUN checksums ===");
    eprintln!("A t=200: {}", ca_200);
    eprintln!("C t=200: {}", cc_200);
    if ca_200 == cc_200 { return; }

    let mut eco_d = Ecology::new(seed);
    for t in 1..=100 { eco_d.update(0.1, daylight(t)); }
    let st0 = eco_d.to_save_state();
    eprintln!("\n=== t=100 pre-save ===");
    eprintln!("agents={} herb={} fish={} carcasses={} rng={} prng={} fishrng={}",
        eco_d.living().len(),
        eco_d.living().iter().filter(|a| a.kind==HERB).count(),
        eco_d.living().iter().filter(|a| a.kind==FISH).count(),
        st0.carcasses.len(),
        st0.rng_state.state, st0.prng_state.state, st0.fish_rng_state.state);
    eprintln!("births={} deaths={} spec_fired={}", st0.births, st0.deaths, st0.speciation_fired);

    let mut eco_e = Ecology::from_save_state(st0);
    for tick in 101..=200 {
        eco_d.update(0.1, daylight(tick));
        eco_e.update(0.1, daylight(tick));
        let jd = serde_json::to_string(&eco_d.to_save_state()).unwrap();
        let je = serde_json::to_string(&eco_e.to_save_state()).unwrap();
        if jd != je {
            let vj: serde_json::Value = serde_json::from_str(&jd).unwrap();
            let ve: serde_json::Value = serde_json::from_str(&je).unwrap();
            eprintln!("\n!!! FIRST DIVERGENCE AT TICK {} !!!", tick);
            eprintln!("  A ck={} C ck={}", eco_d.checksum(), eco_e.checksum());
            let oj = vj.as_object().unwrap();
            let oe = ve.as_object().unwrap();
            for k in oj.keys() {
                let av = oj.get(k).unwrap();
                let ev = oe.get(k).unwrap_or(&serde_json::Value::Null);
                if av != ev {
                    fn snip(v: &serde_json::Value, n: usize) -> String {
                        match v {
                            serde_json::Value::Array(arr) => format!("[{:?}]", arr.iter().take(n).collect::<Vec<_>>()),
                            other => format!("{}", other),
                        }
                    }
                    eprintln!("  DIFF '{}': A={}", k, snip(av, 3));
                    eprintln!("       '{}': C={}", k, snip(ev, 3));
                }
            }
            break;
        }
    }
}
