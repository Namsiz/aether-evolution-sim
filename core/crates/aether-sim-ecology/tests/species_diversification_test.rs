//! Phase 15 speciation-engine acceptance tests.
//!
//! Five pillars:
//! 1. `manual_split_two_isolated_groups` — two DECO clusters seeded 0.9 apart
//!    on deco_efficiency and >100 m apart geographically must produce a
//!    speciation event (root "deco" → child "deco-A") within 500 ticks.
//! 2. `no_speciation_without_geographic_isolation` — same genomes, interleaved
//!    at one location: the geographic gate must veto the split (no events,
//!    root-only clade tree).
//! 3. `clade_tree_grows_monotonically` — long run: total clade count never
//!    decreases, ids increase with birth order, parent_id < id, events stay
//!    consistent with the clade list.
//! 4. `extinction_records_death_tick` — killing every member of a daughter
//!    clade freezes its record with a death_tick (fossil) while the parent
//!    stays alive.
//! 5. `save_restore_roundtrip_preserves_clades` — SaveState v2 carries the
//!    clade tree + event log through the JSON hop bit-exactly and both sims
//!    stay checksum-locked afterwards; a pre-Phase-15 v2 save (fields
//!    stripped) loads into single-clade mode and keeps running.
//!
//! Setup notes: tests run in legacy DECO mode (efficiency is fitness-neutral)
//! so the seeded gap is pure drift-stable signal, and inject carcasses at the
//! cluster anchors so the decomposers never starve mid-experiment.

use aether_sim_ecology::ecology::{Carcass, Ecology, DECO};
use aether_sim_ecology::save_state;
use aether_sim_ecology::terrain::{slope_at, terrain_height, WATER_LEVEL};

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in (from + 1)..=to {
        eco.update(0.1, daylight(t));
    }
}

const SEED: u32 = 20261102;

/// Scan a coarse grid for the two valid-land points that are farthest apart.
/// Deterministic; both points satisfy the seed_life placement gates.
fn two_far_land_points() -> ((f64, f64), (f64, f64)) {
    let mut pts: Vec<(f64, f64)> = Vec::new();
    let mut x = -480.0f64;
    while x <= 480.0 {
        let mut z = -480.0f64;
        while z <= 480.0 {
            let h = terrain_height(x, z);
            if h > WATER_LEVEL + 2.0 && h < 80.0 && slope_at(x, z) < 0.7 {
                pts.push((x, z));
            }
            z += 60.0;
        }
        x += 60.0;
    }
    let mut best_d = 0.0f64;
    let mut best_pair = (pts[0], pts[pts.len() - 1]);
    for i in 0..pts.len() {
        for j in (i + 1)..pts.len() {
            let d = (pts[i].0 - pts[j].0).hypot(pts[i].1 - pts[j].1);
            if d > best_d {
                best_d = d;
                best_pair = (pts[i], pts[j]);
            }
        }
    }
    assert!(best_d > 400.0, "expected two land points >400 apart, got {best_d}");
    best_pair
}

/// Small deterministic jitter pattern so cluster members don't stack exactly.
const JITTER: [(f64, f64); 8] = [
    (0.0, 0.0), (6.0, 2.0), (-5.0, 5.0), (3.0, -6.0),
    (-7.0, -3.0), (8.0, 7.0), (-2.0, 8.0), (5.0, -8.0),
];

/// Relocate the seeded DECOs into two clusters with a wide efficiency gap.
/// `co_located` interleaves both clusters around one anchor (control group).
/// Returns the two anchors.
fn seed_two_clusters(eco: &mut Ecology, co_located: bool) -> ((f64, f64), (f64, f64)) {
    eco.deco_eff_choice_mode = save_state::DECO_CHOICE_LEGACY;
    let (a, b) = if co_located {
        let p = two_far_land_points().0;
        (p, p)
    } else {
        two_far_land_points()
    };
    let mut k = 0usize;
    for agent in eco.agents.iter_mut() {
        if agent.kind != DECO { continue; }
        let first_half = k < 8;
        let anchor = if co_located || first_half { a } else { b };
        let j = JITTER[k % JITTER.len()];
        let eff = if first_half { 0.55 } else { 1.45 };
        agent.x = anchor.0 + j.0;
        agent.z = anchor.1 + j.1;
        agent.prev_x = agent.x;
        agent.prev_z = agent.z;
        agent.home_x = anchor.0;
        agent.home_z = anchor.1;
        agent.genome.deco_efficiency = eff;
        k += 1;
    }
    (a, b)
}

/// Keep both clusters fed: two fresh carcasses per anchor every 100 ticks.
fn feed(eco: &mut Ecology, anchors: &[(f64, f64); 2], tick: u32) {
    if tick % 100 != 0 { return; }
    for (i, (x, z)) in anchors.iter().enumerate() {
        let dx = if i == 0 { 4.0 } else { -4.0 };
        eco.carcasses.push(Carcass { x: x + dx, z: *z, energy: 30.0, age: 0.0, occupied_by: None });
    }
}

fn deco_pop(eco: &Ecology) -> usize {
    eco.agents.iter().filter(|a| a.alive && a.kind == DECO).count()
}

// ---------------------------------------------------------------------------
// 1. Manual split with geographic isolation
// ---------------------------------------------------------------------------

#[test]
fn manual_split_two_isolated_groups() {
    let mut eco = Ecology::new(SEED);
    let (a, b) = seed_two_clusters(&mut eco, false);
    let anchors = [a, b];
    assert!((a.0 - b.0).hypot(a.1 - b.1) > 400.0, "clusters must start far apart");

    for t in 1..=500u32 {
        feed(&mut eco, &anchors, t);
        run_ticks(&mut eco, t - 1, t);
    }

    eprintln!("=== split run: deco pop={} clades={} events={}",
        deco_pop(&eco), eco.species_clades.len(), eco.speciation_events.len());
    for c in &eco.species_clades {
        eprintln!("  clade {} '{}' parent={:?} birth={} death={:?} members={}",
            c.id, c.name, c.parent_id, c.birth_tick, c.death_tick, c.members.len());
    }
    for e in &eco.speciation_events {
        eprintln!("  event t={} parent={} children={:?} d={:.3}",
            e.tick, e.parent_clade_id, e.child_clade_ids, e.genetic_distance);
    }

    // Root clade exists from the first detection pass.
    let root = eco.species_clades.iter().find(|c| c.parent_id.is_none())
        .expect("root clade must exist");
    assert_eq!(root.kind, "deco");
    assert_eq!(root.name, "deco");

    // Exactly one split, root → deco-A.
    assert_eq!(eco.speciation_events.len(), 1, "expected exactly one speciation event");
    let ev = &eco.speciation_events[0];
    assert_eq!(ev.parent_clade_id, root.id);
    assert_eq!(ev.child_clade_ids.len(), 1);
    assert!(ev.genetic_distance >= 0.35, "split distance {} below threshold", ev.genetic_distance);
    assert!(ev.tick >= 200, "time gate forbids splits before tick 200");

    let child_id = ev.child_clade_ids[0];
    let child = eco.species_clades.iter().find(|c| c.id == child_id).expect("child clade recorded");
    assert_eq!(child.parent_id, Some(root.id));
    assert_eq!(child.name, "deco-A");
    assert!(child.death_tick.is_none(), "child clade must be alive at t=500");

    // Clade assignment is exclusive: every living DECO carries root or child.
    let bad = eco.agents.iter()
        .filter(|a| a.alive && a.kind == DECO && a.clade_id != root.id && a.clade_id != child_id)
        .count();
    assert_eq!(bad, 0, "DECOs with unknown clade id found");
    // The split actually partitioned the population.
    assert!(eco.agents.iter().any(|a| a.alive && a.kind == DECO && a.clade_id == root.id));
    assert!(eco.agents.iter().any(|a| a.alive && a.kind == DECO && a.clade_id == child_id));
    assert_eq!(eco.species_clades.len(), 2, "exactly root + one daughter clade");
}

// ---------------------------------------------------------------------------
// 2. Control group: no geographic isolation → no split
// ---------------------------------------------------------------------------

#[test]
fn no_speciation_without_geographic_isolation() {
    let mut eco = Ecology::new(SEED);
    let (a, b) = seed_two_clusters(&mut eco, true);
    assert_eq!((a.0 - b.0).hypot(a.1 - b.1), 0.0, "control clusters share one anchor");
    let anchors = [a, b];

    for t in 1..=500u32 {
        feed(&mut eco, &anchors, t);
        run_ticks(&mut eco, t - 1, t);
    }

    eprintln!("=== control run: deco pop={} clades={} events={}",
        deco_pop(&eco), eco.species_clades.len(), eco.speciation_events.len());
    assert!(eco.speciation_events.is_empty(),
        "geographic gate must veto co-located splits, got {:?}", eco.speciation_events);
    assert_eq!(eco.species_clades.len(), 1, "root-only clade tree in the control group");
}

// ---------------------------------------------------------------------------
// 3. Long-run tree invariants
// ---------------------------------------------------------------------------

#[test]
fn clade_tree_grows_monotonically() {
    let mut eco = Ecology::new(SEED);
    let (a, b) = seed_two_clusters(&mut eco, false);
    let anchors = [a, b];

    let mut snapshots: Vec<(u32, usize, u32, usize)> = Vec::new(); // (tick, total, next_id, events)
    for t in 1..=1500u32 {
        feed(&mut eco, &anchors, t);
        run_ticks(&mut eco, t - 1, t);
        if t % 300 == 0 {
            snapshots.push((t, eco.species_clades.len(), eco.next_clade_id, eco.speciation_events.len()));
        }
    }
    eprintln!("=== monotonic run snapshots (tick, total_clades, next_id, events): {:?}", snapshots);

    // Total clade count (incl. fossils) never decreases.
    for w in snapshots.windows(2) {
        assert!(w[1].1 >= w[0].1, "clade count regressed between {:?} and {:?}", w[0], w[1]);
        assert!(w[1].2 >= w[0].2, "next_clade_id regressed");
        assert!(w[1].3 >= w[0].3, "event log regressed");
    }
    assert!(snapshots.last().unwrap().1 >= 2, "seeded split must persist in the tree");

    // Structural invariants over the whole tree.
    let mut last_birth = 0u32;
    for c in &eco.species_clades {
        match c.parent_id {
            Some(p) => assert!(p < c.id, "parent id {} must precede child id {}", p, c.id),
            None => assert_eq!(c.birth_tick, 0, "root clade is born at t=0"),
        }
        assert!(c.birth_tick >= last_birth, "birth ticks must be non-decreasing in vec order");
        last_birth = c.birth_tick;
        if c.death_tick.is_some() {
            assert!(c.death_tick.unwrap() >= c.birth_tick, "death before birth");
        }
    }
    // Every event references existing clades.
    for e in &eco.speciation_events {
        assert!(eco.species_clades.iter().any(|c| c.id == e.parent_clade_id), "event parent missing");
        for &cid in &e.child_clade_ids {
            let child = eco.species_clades.iter().find(|c| c.id == cid).expect("event child missing");
            assert_eq!(child.parent_id, Some(e.parent_clade_id), "event child parent mismatch");
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Extinction → fossil record
// ---------------------------------------------------------------------------

#[test]
fn extinction_records_death_tick() {
    let mut eco = Ecology::new(SEED);
    let (a, b) = seed_two_clusters(&mut eco, false);
    let anchors = [a, b];
    for t in 1..=500u32 {
        feed(&mut eco, &anchors, t);
        run_ticks(&mut eco, t - 1, t);
    }
    let child_id = eco.speciation_events[0].child_clade_ids[0];
    assert!(eco.species_clades.iter().any(|c| c.id == child_id && c.death_tick.is_none()));

    // Wipe the daughter clade (direct kill; carcass/bookkeeping side effects
    // are irrelevant to the clade log under test).
    for agent in eco.agents.iter_mut() {
        if agent.alive && agent.kind == DECO && agent.clade_id == child_id {
            agent.alive = false;
        }
    }
    eco.agents.retain(|a| a.alive);

    for t in 501..=620u32 {
        feed(&mut eco, &anchors, t);
        run_ticks(&mut eco, t - 1, t);
    }

    let child = eco.species_clades.iter().find(|c| c.id == child_id).expect("child clade kept");
    let death = child.death_tick.expect("extinct clade must record death_tick");
    assert!(death > 500 && death <= 620, "death_tick {} outside the post-kill detection window", death);
    let root = eco.species_clades.iter().find(|c| c.parent_id.is_none()).unwrap();
    assert!(root.death_tick.is_none(), "parent clade must survive its daughter's extinction");
    // Fossil is permanent: no revival, members stay empty.
    assert!(child.members.is_empty(), "extinct clade keeps no members");
}

// ---------------------------------------------------------------------------
// 5. Save/restore roundtrip (v2 schema, clade fields as compact JSON strings)
// ---------------------------------------------------------------------------

#[test]
fn save_restore_roundtrip_preserves_clades() {
    let mut eco = Ecology::new(SEED);
    let (a, b) = seed_two_clusters(&mut eco, false);
    let anchors = [a, b];
    for t in 1..=600u32 {
        feed(&mut eco, &anchors, t);
        run_ticks(&mut eco, t - 1, t);
    }
    assert!(!eco.speciation_events.is_empty(), "speciation must precede the save");
    let ck600 = eco.checksum();

    // --- JSON hop ---
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let restored = Ecology::from_save_state(serde_json::from_str(&json).unwrap());

    assert_eq!(restored.species_clades.len(), eco.species_clades.len(), "clade count");
    for (o, r) in eco.species_clades.iter().zip(restored.species_clades.iter()) {
        assert_eq!(o.id, r.id);
        assert_eq!(o.parent_id, r.parent_id);
        assert_eq!(o.name, r.name);
        assert_eq!(o.birth_tick, r.birth_tick);
        assert_eq!(o.death_tick, r.death_tick);
        assert!((o.centroid_genome.deco_efficiency - r.centroid_genome.deco_efficiency).abs() < 1e-9,
            "centroid eff drift for clade {}", o.id);
    }
    assert_eq!(restored.speciation_events.len(), eco.speciation_events.len(), "event count");
    for (o, r) in eco.speciation_events.iter().zip(restored.speciation_events.iter()) {
        assert_eq!(o.tick, r.tick);
        assert_eq!(o.parent_clade_id, r.parent_clade_id);
        assert_eq!(o.child_clade_ids, r.child_clade_ids);
        assert!((o.genetic_distance - r.genetic_distance).abs() < 1e-9);
    }
    assert_eq!(restored.next_clade_id, eco.next_clade_id, "next_clade_id");
    assert_eq!(restored.tick_count(), eco.tick_count(), "tick count");
    let restored_agents_clades: Vec<u32> = restored.agents.iter().map(|a| a.clade_id).collect();
    let original_agents_clades: Vec<u32> = eco.agents.iter().map(|a| a.clade_id).collect();
    assert_eq!(restored_agents_clades, original_agents_clades, "per-agent clade ids");

    // --- lockstep continuation: both sims must stay checksum-identical ---
    let mut a_sim = eco;
    let mut b_sim = restored;
    for t in 601..=900u32 {
        feed(&mut a_sim, &anchors, t);
        feed(&mut b_sim, &anchors, t);
        run_ticks(&mut a_sim, t - 1, t);
        run_ticks(&mut b_sim, t - 1, t);
        if t % 100 == 0 {
            assert_eq!(a_sim.checksum(), b_sim.checksum(), "lockstep diverged at t={t}");
        }
    }
    assert_eq!(a_sim.checksum(), b_sim.checksum());
    let _ = ck600; // (kept for debuggability; the lockstep check supersedes it)

    // --- pre-Phase-15 v2 probe: strip the four new keys → single-clade mode ---
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    {
        let obj = v.as_object_mut().unwrap();
        for key in ["SpeciesClades", "NextCladeId", "SpeciationEvents", "LastCladeSpecTick"] {
            assert!(obj.remove(key).is_some(), "key {key} missing from save");
        }
    }
    let legacy: save_state::SaveState = serde_json::from_value(v).unwrap();
    let legacy_eco = Ecology::from_save_state(legacy);
    assert!(legacy_eco.species_clades.is_empty(), "legacy save restores with no clades");
    assert_eq!(legacy_eco.next_clade_id, 0);
    // The save keeps running; the first detection pass recreates the root
    // clade from agent clade_ids. Note the stripped save loses the split
    // history but not the divergence itself — the population still carries a
    // ≥0.35 efficiency gap across a >100 m gap, so the engine legitimately
    // re-detects it (a fresh event, not a resurrection of the old fossil).
    let mut le = legacy_eco;
    run_ticks(&mut le, 600, 700);
    assert!(le.species_clades.len() >= 1, "root clade recreated after load");
    assert_eq!(le.species_clades[0].name, "deco");
    assert!(le.species_clades[0].parent_id.is_none());
    assert!(le.tick_count() == 700, "legacy save keeps simulating");
}
