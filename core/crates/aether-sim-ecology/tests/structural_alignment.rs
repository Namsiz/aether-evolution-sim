use aether_sim_ecology::ecology::{Ecology, DECO, FISH, HERB, PRED, SCAV, OMN};

// FNV-1a 64 over floor(grass[i] * 1000) for every grid cell, matching the
// checksum() mix style. Used as a full-grid grass regression fingerprint.
fn grass_fingerprint(eco: &Ecology) -> u64 {
    let mut h: u64 = 14695981039346656033;
    for &v in eco.grass() {
        let f = (v as f64 * 1000.0).floor() as i64 as u64;
        h = (h ^ (f & 0xFFFF_FFFF)).wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

#[test]
fn grass_spread_consistency_at_tick_40() {
    // Spread triggers at tickCount % 40 == 0: each species double-buffers grass_p,
    // transfers 1% of its own stock to in-grid neighbors with suit > 0.02, and
    // refreshes the aggregate grass view. This locks the t=40 post-spread state
    // (seed=20261102) as a regression guard: any change to the spread path,
    // the gut-transit deposit feed-in, or the grow feed-in that alters a single
    // f32 grass cell will shift this fingerprint.
    // JS oracle: node -e '...mix floor(eco.grass[i]*1000) after 40 ticks...' → 0x42ad7909a6ef600e
    // Phase 4-C rebase: DECO mineral deposits boost grass growth, shifting the
    // fingerprint (bit-exact golden waived for RS-only DECO dynamics).
    let mut eco = Ecology::new(20261102);
    for t in 1..=40u32 {
        let daylight = 0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI);
        eco.update(0.1, daylight);
    }
    assert_eq!(eco.tick_count(), 40, "tick_count must be exactly 40");
    assert_eq!(grass_fingerprint(&eco), 7835318463236313578, "full-grid grass fingerprint at t=40 (post-spread, Phase 4-C: deco mineral boost shifts the field)");
}

#[test]
fn init_structure_matches_js_oracle() {
    // With Phase 4-B (FISH) + Phase 4-C (DECO): herb=150, pred=14, scav=8, omn=12, fish=30, deco=15 (total 229)
    let eco = Ecology::new(20261102);
    let mut counts = std::collections::HashMap::new();
    for a in &eco.agents {
        assert!(a.alive, "no agent should be dead at init");
        *counts.entry(a.kind).or_insert(0usize) += 1;
    }
    assert_eq!(*counts.get(&HERB).unwrap_or(&0), 150, "herb count");
    assert_eq!(*counts.get(&PRED).unwrap_or(&0), 14, "pred count");
    assert_eq!(*counts.get(&SCAV).unwrap_or(&0), 8, "scav count");
    assert_eq!(*counts.get(&OMN).unwrap_or(&0), 12, "omn count");
    assert_eq!(*counts.get(&FISH).unwrap_or(&0), 30, "fish count (Phase 4-B)");
    assert_eq!(*counts.get(&DECO).unwrap_or(&0), 15, "deco count (Phase 4-C)");
    assert_eq!(eco.agents.len(), 229, "total agent count (with fish + deco)");
}

#[test]
fn checksum_covers_all_six_kinds() {
    // checksum() must hash a.kind first byte for herb/pred/scav/omn/fish/deco
    let eco = Ecology::new(20261102);
    let kinds: std::collections::HashSet<&'static str> =
        eco.agents.iter().map(|a| eco.kind_name(a.kind)).collect();
    assert_eq!(kinds.len(), 6, "all six kinds present at init (Phase 4-C adds deco)");
    let ck = eco.checksum();
    assert!(ck.starts_with("0x"), "checksum format 0x…: {}", ck);
    assert_eq!(ck.len(), 18, "checksum is 16 hex digits: {}", ck);
}

#[test]
fn deterministic_same_seed_same_checksum() {
    let a = Ecology::new(20261102).checksum();
    let b = Ecology::new(20261102).checksum();
    assert_eq!(a, b, "same seed must produce identical checksum");
}

#[test]
fn scav_omn_ecology_loop_active_and_counts_match_js_oracle() {
    // Phase 3 P1 acceptance: SCAV/OMN must be functionally active in the sim loop.
    // JS oracle golden (seed=20261102, ticks=300): scav stays 8 throughout;
    // omn stays 12 until a reproduction event lands it at 14 by t=250.
    // The behavioral fingerprints (scavenge / flee states) prove the
    // update_scav/update_omn paths actually execute — a static NPC would
    // never reach either state.
    use aether_libm::sin;
    use std::collections::HashSet;

    let mut eco = Ecology::new(20261102);
    // Phase 8: carcass locks alter SCAV/OMN food competition, drifting the
    // exact population oracle. This test locks pre-Phase-8 JS counts, so the
    // new mechanism is disabled here; carcass-lock behavior is covered by
    // carcass_lock_test.rs.
    eco.carcass_lock_enabled = false;
    let mut next_sample = 50u32;
    let mut states_scav: HashSet<&'static str> = HashSet::new();
    let mut states_omn: HashSet<&'static str> = HashSet::new();
    for t in 1..=300u32 {
        let daylight = 0.5 + 0.5 * sin((t as f64 / 300.0) * std::f64::consts::PI);
        eco.update(0.1, daylight);
        if t == next_sample {
            let s = eco.agents.iter().filter(|a| a.alive && a.kind == SCAV).count();
            let o = eco.agents.iter().filter(|a| a.alive && a.kind == OMN).count();
            assert_eq!(s, 8, "scav count at t={}", t);
            let o_gold = if t == 300 { 14usize } else if t >= 250 { 13 } else { 12 };
            assert_eq!(o, o_gold, "omn count at t={}", t);
            next_sample += 50;
        }
        for a in &eco.agents {
            if !a.alive { continue; }
            if a.kind == SCAV { states_scav.insert(a.state); }
            if a.kind == OMN { states_omn.insert(a.state); }
        }
    }
    assert!(
        states_scav.contains("scavenge"),
        "SCAV must enter 'scavenge' state near carcasses: {:?}",
        states_scav
    );
    assert!(
        states_omn.contains("flee"),
        "OMN must enter 'flee' state when a predator is within sense*0.9: {:?}",
        states_omn
    );
}
