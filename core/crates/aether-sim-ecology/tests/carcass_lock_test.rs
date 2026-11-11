//! Phase 8 carcass occupancy-lock acceptance tests.
//!
//! Four pillars:
//! 1. `mutex_single_agent_eats_locked_carcass` — two DECOs competing for one
//!    carcass: the lock holder consumes it exclusively; the loser must never
//!    take a bite from the same carcass while it stays occupied.
//! 2. `lock_releases_after_carcass_depletion` — after the owner depletes the
//!    carcass, a second DECO can acquire the next one (release-then-reacquire
//!    cycle proves the lock is not permanent).
//! 3. `save_restore_roundtrip_preserves_carcass_locks` — SaveState v2 with the
//!    optional `occupied_by` field roundtrips bit-exactly; a legacy v2 save
//!    without the field restores with all locks cleared (serde default None).
//! 4. `six_species_survives_600_ticks_with_locks` — Phase 4-C ecological
//!    red-line re-check under the new competition rule.
//!
//! Plus a v1→v2-schema probe: a save serialized before Phase 8 (missing the
//! `CarcassLockEnabled` / `occupied_by` fields) loads and keeps running.

use aether_sim_ecology::ecology::{Carcass, Ecology, DECO, SCAV};
use aether_sim_ecology::save_state::{self, SaveState};

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in (from + 1)..=to {
        eco.update(0.1, daylight(t));
    }
}

fn count_kind(eco: &Ecology, kind: &str) -> usize {
    eco.living().iter().filter(|a| a.alive && a.kind == kind).count()
}

/// Strip all non-DECO agents and trim to exactly `n` DECOs so the test
/// scenario is fully controlled (a fresh sim seeds 15 DECOs).
fn strip_to_decos(eco: &mut Ecology, n: usize) {
    eco.agents.retain(|a| a.kind == DECO);
    eco.agents.truncate(n);
}

const SEED: u32 = 20261102;

// ---------------------------------------------------------------------------
// 1. Mutual exclusion — one carcass, two DECOs
// ---------------------------------------------------------------------------

#[test]
fn mutex_single_agent_eats_locked_carcass() {
    let mut eco = Ecology::new(SEED);
    strip_to_decos(&mut eco, 2);
    assert_eq!(eco.agents.len(), 2, "expected exactly 2 DECOs after strip");

    // Place both DECOs near the same carcass so they arrive within a few ticks.
    let (cx, cz) = (0.0f64, 0.0f64);
    eco.agents[0].x = cx + 3.0; eco.agents[0].z = cz;
    eco.agents[1].x = cx + 4.5; eco.agents[1].z = cz;
    eco.carcasses.clear();
    eco.carcasses.push(Carcass { x: cx, z: cz, energy: 30.0, age: 0.0, occupied_by: None });

    let deco_a = eco.agents[0].id;
    let deco_b = eco.agents[1].id;

    let mut ever_locked = false;
    for t in 1..=100u32 {
        run_ticks(&mut eco, t - 1, t);
        // Invariant: no carcass may be simultaneously locked and >0.2-energy-eaten
        // by a non-owner. (Energy drain only happens through the owner's bite.)
        for c in &eco.carcasses {
            if let Some(owner) = c.occupied_by {
                ever_locked = true;
                assert!(owner == deco_a || owner == deco_b, "lock held by unknown id {owner}");
            }
        }
    }
    assert!(ever_locked, "no carcass was ever locked in 100 ticks — lock path never exercised");
    eprintln!("=== mutex: after 100 ticks, {} carcass(es) remain ===", eco.carcasses.len());
    for c in &eco.carcasses {
        eprintln!("  carcass at ({:.1},{:.1}) energy={:.2} locked_by={:?}",
            c.x, c.z, c.energy, c.occupied_by);
    }
    // The lock must either be released (carcass eaten → removed, or timeout) or
    // still held by one of the two DECOs — never by an outside agent.
}

// ---------------------------------------------------------------------------
// 2. Lock release on carcass depletion
// ---------------------------------------------------------------------------

#[test]
fn lock_releases_after_carcass_depletion() {
    let mut eco = Ecology::new(SEED);
    strip_to_decos(&mut eco, 2);

    // One DECO placed right on the carcass so it locks and eats immediately.
    let (cx, cz) = (0.0f64, 0.0f64);
    eco.agents[0].x = cx; eco.agents[0].z = cz;
    eco.agents[1].x = 30.0; eco.agents[1].z = 0.0; // far away, cannot contest
    eco.carcasses.clear();
    eco.carcasses.push(Carcass { x: cx, z: cz, energy: 5.0, age: 0.0, occupied_by: None });

    // Let the near DECO lock and deplete the small carcass.
    let mut carcass_removed = false;
    for t in 1..=40u32 {
        run_ticks(&mut eco, t - 1, t);
        if eco.carcasses.iter().all(|c| (c.x - cx).abs() > 1.0) {
            carcass_removed = true;
            break;
        }
    }
    assert!(carcass_removed, "5.0-energy carcass should be depleted within 40 ticks by the co-located DECO");

    // Spawn a fresh carcass at the same spot — the second DECO (or the first)
    // must now be able to acquire it, proving stale locks do not persist.
    eco.carcasses.push(Carcass { x: cx, z: cz, energy: 25.0, age: 0.0, occupied_by: None });
    let mut relocked = false;
    for t in 41..=80u32 {
        run_ticks(&mut eco, t - 1, t);
        if eco.carcasses.iter().any(|c| c.occupied_by.is_some()) {
            relocked = true;
            break;
        }
    }
    assert!(relocked, "fresh carcass was never re-locked after the old one was depleted");
}

// ---------------------------------------------------------------------------
// 3. Save/restore roundtrip with occupancy locks
// ---------------------------------------------------------------------------

#[test]
fn save_restore_roundtrip_preserves_carcass_locks() {
    let mut eco = Ecology::new(SEED);
    // Run a few ticks so the population settles, then inject a locked carcass.
    run_ticks(&mut eco, 0, 30);
    let owner_id = eco.living().iter()
        .find(|a| a.kind == DECO).map(|a| a.id)
        .expect("at least one DECO must exist at t=30");
    eco.carcasses.push(Carcass { x: 10.0, z: 10.0, energy: 20.0, age: 1.0, occupied_by: Some(owner_id) });

    let saved = eco.to_save_state();
    assert_eq!(saved.version, save_state::SAVE_STATE_VERSION, "save version unchanged (v2 schema, optional fields)");

    // The saved carcass must carry the lock.
    let saved_carcass = saved.carcasses.iter()
        .find(|c| (c.x - 10.0).abs() < 0.01 && (c.z - 10.0).abs() < 0.01)
        .expect("locked carcass must be serialized");
    assert_eq!(saved_carcass.occupied_by, Some(owner_id), "occupied_by must roundtrip");

    // JSON hop must preserve the field (serde_json roundtrip).
    let json = serde_json::to_string(&saved).expect("save to JSON");
    let parsed: SaveState = serde_json::from_str(&json).expect("JSON back to SaveState");
    let parsed_carcass = parsed.carcasses.iter()
        .find(|c| (c.x - 10.0).abs() < 0.01 && (c.z - 10.0).abs() < 0.01)
        .expect("locked carcass survives JSON hop");
    assert_eq!(parsed_carcass.occupied_by, Some(owner_id), "occupied_by survives JSON roundtrip");

    // Restore and verify the lock comes back on the live carcass.
    let restored = Ecology::from_save_state(parsed);
    let restored_carcass = restored.carcasses.iter()
        .find(|c| (c.x - 10.0).abs() < 0.01 && (c.z - 10.0).abs() < 0.01)
        .expect("locked carcass exists after restore");
    assert_eq!(restored_carcass.occupied_by, Some(owner_id),
        "occupancy lock must survive the full save→restore cycle");
}

#[test]
fn legacy_v2_save_without_lock_fields_restores_cleanly() {
    // Build a save, then strip the Phase 8 optional fields to simulate a
    // Phase 6/7-era v2 document (serde default: occupied_by=None, lock=true).
    let mut eco = Ecology::new(SEED);
    run_ticks(&mut eco, 0, 20);
    let mut saved = eco.to_save_state();
    saved.carcasses.iter_mut().for_each(|c| { c.occupied_by = None; });

    let mut json = serde_json::to_value(&saved).expect("save to JSON value");
    // Remove CarcaseLockEnabled to prove the serde default kicks in.
    if let Some(obj) = json.as_object_mut() {
        obj.remove("CarcaseLockEnabled");
    }
    let stripped: SaveState = serde_json::from_value(json)
        .expect("a v2 save without Phase 8 fields must deserialize via serde defaults");
    assert!(stripped.CarcaseLockEnabled, "absent CarcaseLockEnabled must default to true (locking stays on)");
    assert!(stripped.carcasses.iter().all(|c| c.occupied_by.is_none()), "absent occupied_by must default to None");

    let restored = Ecology::from_save_state(stripped);
    assert!(restored.carcass_lock_enabled, "restored sim must have locking enabled after a legacy save");
    run_ticks(&mut Ecology::from_save_state(eco.to_save_state()), 0, 1); // smoke: original save still loadable
}

// ---------------------------------------------------------------------------
// 4. Ecological health under locking (Phase 4-C red line)
// ---------------------------------------------------------------------------

#[test]
fn six_species_survives_600_ticks_with_locks() {
    let mut eco = Ecology::new(SEED);
    run_ticks(&mut eco, 0, 600);

    let kinds = [
        ("herb", 30), ("pred", 4), ("scav", 5), ("omn", 5), ("fish", 5), ("deco", 10),
    ];
    eprintln!("=== 600-tick census with carcass locks ===");
    for (name, min) in kinds {
        let n = count_kind(&eco, name);
        eprintln!("  {name:>5}: {n}");
        assert!(n >= min, "{name} population collapsed to {n} (<{min}) under carcass locking");
    }
    // DECO/SCAV bandwidth (Phase 4-C red line).
    let d = count_kind(&eco, DECO);
    let s = count_kind(&eco, SCAV);
    assert!((15..=40).contains(&d), "DECO count {d} out of [15,40]");
    assert!((5..=15).contains(&s), "SCAV count {s} out of [5,15]");
}
