//! Phase 43 acceptance — abiogenesis (hot-spring life re-origin).
//!
//! Miller-Urey / hydrothermal-vent chemistry: life re-originates from inorganic
//! chemistry when the world approaches death (pop < ABIO_MIN_POP). This is the
//! natural-path successor to the retired hand-of-god immigration — not a
//! resurrection mechanism. Trigger is emergent: per-tick probability
//! ABIO_P_BASE × mean(candidate-cell intensity), candidates = mineral-active
//! OR wet+fertile cells; the spawn's start energy books into
//! `ledger.chemo_in` (a primary input channel beside light_in/filter_in).
//!
//! Acceptance:
//!   1. `extinct_world_reboots_via_abiogenesis` — a killed world spawns
//!      ancestors from chemistry; chemo_in > 0 and enters total_in().
//!   2. `healthy_world_abiogenesis_suppressed` — a flourishing world fires
//!      zero events (ecological saturation suppression).
//!   3. `abiogenesis_save_roundtrip_bit_exact` — the ABIO stream survives the
//!      save/restore hop and the restored world stays bit-exact.
//!   4. `v13_archive_lands_on_phase43_defaults` — pre-v14 archives restore
//!      with both toggles ON, zero counters, and a re-derived stream.

use aether_sim_ecology::ecology::Ecology;
use aether_sim_ecology::save_state;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in from..=to {
        eco.update(0.1, daylight(t));
    }
}

fn alive_count(eco: &Ecology) -> usize {
    eco.agents.iter().filter(|a| a.alive).count()
}

/// Build a dead world: run a healthy head-start, then kill everything
/// (the manual extinction event; the engine itself has no hand of god).
fn dead_world(seed: u32) -> Ecology {
    let mut eco = Ecology::new(seed);
    run_ticks(&mut eco, 1, 50);
    for a in eco.agents.iter_mut() {
        a.alive = false;
    }
    run_ticks(&mut eco, 51, 51); // retain sweeps the dead → pop 0
    assert_eq!(alive_count(&eco), 0);
    eco
}

// ---------------------------------------------------------------------------
// 1. Extinction → chemistry → life again.
// ---------------------------------------------------------------------------
#[test]
fn extinct_world_reboots_via_abiogenesis() {
    let mut eco = dead_world(20261102);
    let first_event = {
        let mut found = None;
        for t in 52..=60000u32 {
            if eco.abiogenesis_events > 0 { found = Some(t); break; }
            run_ticks(&mut eco, t, t);
        }
        found.expect("a dead world must re-originate life within the window")
    };
    assert!(alive_count(&eco) >= 1, "the re-originated individual must be alive");
    assert!(eco.abiogenesis_ticks >= 52, "event tick must be recorded");
    // Energetics: the spawn's start energy came from hot-spring chemistry.
    assert!(eco.ledger.chemo_in > 0.0);
    assert!(eco.ledger.light_in > 0.0);
    // total_in() includes the new channel (light + filter + chemo).
    let expected_total = eco.ledger.light_in + eco.ledger.filter_in + eco.ledger.chemo_in;
    assert!((eco.ledger.total_in() - expected_total).abs() < 1e-9);
    // Keep running: the bubbling continues while pop < ABIO_MIN_POP, and the
    // world climbs out of the absorbing state (more events, population grows).
    run_ticks(&mut eco, first_event + 1, first_event + 40000);
    assert!(eco.abiogenesis_events >= 2,
        "a still-near-dead world must keep re-originating (events={})", eco.abiogenesis_events);
    assert!(alive_count(&eco) >= 1);
}

// ---------------------------------------------------------------------------
// 2. Saturation suppression: a healthy world never fires.
// ---------------------------------------------------------------------------
#[test]
fn healthy_world_abiogenesis_suppressed() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 3000);
    assert!(alive_count(&eco) > 100, "sanity: the world must be flourishing");
    assert_eq!(eco.abiogenesis_events, 0, "a healthy world must not bubble life");
    assert_eq!(eco.abiogenesis_ticks, 0);
    assert_eq!(eco.ledger.chemo_in, 0.0);
}

// ---------------------------------------------------------------------------
// 3. The ABIO stream rides the save/restore contract.
// ---------------------------------------------------------------------------
#[test]
fn abiogenesis_save_roundtrip_bit_exact() {
    let mut eco = dead_world(20261102);
    // Run until at least one event has fired (state to snapshot).
    let mut save_at = None;
    for t in 52..=60000u32 {
        if eco.abiogenesis_events >= 2 { save_at = Some(t); break; }
        run_ticks(&mut eco, t, t);
    }
    let t_save = save_at.expect("dead world must re-originate before the snapshot");
    let state = eco.to_save_state();
    assert_eq!(state.version, save_state::SAVE_STATE_VERSION);
    assert!(state.AbiogenesisEvents >= 2);
    assert_ne!(state.AbioRngState, 0, "the ABIO stream must be non-default by now");
    let json = serde_json::to_string(&state).unwrap();
    let mut restored = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    assert_eq!(restored.abiogenesis_events, eco.abiogenesis_events);
    // Both worlds keep running from the snapshot — bit-exact continuation
    // (the ABIO stream keeps drawing while pop < ABIO_MIN_POP, so any stream
    // restore defect forks the trajectory immediately).
    run_ticks(&mut eco, t_save + 1, t_save + 1500);
    run_ticks(&mut restored, t_save + 1, t_save + 1500);
    assert_eq!(eco.checksum(), restored.checksum(),
        "restored dead world (abiogenesis live) must be bit-exact");
}

// ---------------------------------------------------------------------------
// 4. Pre-v14 archives land on the Phase 43 defaults and keep running.
// ---------------------------------------------------------------------------
#[test]
fn v13_archive_lands_on_phase43_defaults() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 120);
    let mut v: serde_json::Value = serde_json::to_value(eco.to_save_state()).unwrap();
    // Degrade to v13: strip every Phase 43 field (exactly what an older
    // archive looks like to this engine).
    v["Version"] = serde_json::json!(13);
    let obj = v.as_object_mut().unwrap();
    for k in ["TeachOn", "AbiogenesisOn", "AbioRngState", "AbiogenesisEvents", "AbiogenesisTicks"] {
        obj.remove(k);
    }
    let cs = obj["CultureStats"].as_object_mut().unwrap();
    cs.remove("TaughtCount");
    cs.remove("TeachCostTotal");
    let legacy: save_state::SaveState = serde_json::from_value(v).expect("v13 archive parses");
    let restored = Ecology::from_save_state(legacy);
    assert!(restored.teach_on, "pre-v14 archives land on inheritance ON");
    assert!(restored.abiogenesis_on, "pre-v14 archives land on abiogenesis ON");
    assert_eq!(restored.abiogenesis_events, 0);
    assert_eq!(restored.abiogenesis_ticks, 0);
    assert_eq!(restored.culture_stats.taught_count, 0);
    assert_eq!(restored.culture_stats.teach_cost_total, 0.0);
    // Two independent restores run in lockstep (deterministic continuation
    // with the re-derived `seed ^ ABIO_RNG_TAG` stream).
    let snapshot = serde_json::to_string(&restored.to_save_state()).unwrap();
    let mut b = Ecology::from_save_state(serde_json::from_str(&snapshot).unwrap());
    let mut c = Ecology::from_save_state(serde_json::from_str(&snapshot).unwrap());
    run_ticks(&mut b, 121, 400);
    run_ticks(&mut c, 121, 400);
    assert_eq!(b.checksum(), c.checksum(), "v13 restore must be deterministic");
}
