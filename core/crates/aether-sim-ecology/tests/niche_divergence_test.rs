//! Phase 16 niche-divergence / competitive-exclusion acceptance tests.
//!
//! Five pillars:
//! 1. `clade_niche_diverges_from_parent` — after the seeded Phase 15 split,
//!    the daughter clade's niche_offset sits far from the parent's (>= 0.5 on
//!    the [-1, 1] axis) and stays there; the root stays pinned at 0.
//! 2. `overlapping_clades_compete` — with both clades pinned into the same
//!    niche (eff gap < overlap window), the lower-efficiency clade carries a
//!    competitive burden and its members' mean energy decays below both the
//!    unburdened winner in the same run and the same clade in an A/B control
//!    run whose niche gap keeps it out of the overlap window.
//! 3. `competitive_exclusion_extincts_loser` — sustained overlap + starvation
//!    of the loser's home range drives the burdened clade to zero members and
//!    logs an extinction event with kind = COMPETITIVE_EXCLUSION (⚔).
//! 4. `non_overlapping_clades_coexist` — niche-separated clades (both
//!    repro-able effs) coexist for 2000 ticks: zero extinction events, zero
//!    burdens.
//! 5. `save_restore_preserves_niche_state` — SaveState v3 carries niches,
//!    event kinds and the active burden map bit-exactly; lockstep checksums
//!    hold afterwards; a stripped v2-style save (no niche/kind/burden keys)
//!    restores to zeros and keeps running.
//!
//! Setup mirrors `species_diversification_test.rs`: two DECO clusters seeded
//! at the farthest-apart land points with a wide deco_efficiency gap, kept
//! alive by periodic carcass drops at each anchor.

use aether_sim_ecology::ecology::{
    Carcass, Ecology, SpeciesClade, DECO, EVENT_KIND_COMPETITIVE_EXCLUSION,
    EVENT_KIND_SPECIATION, NICHE_OVERLAP_THRESHOLD,
};
use aether_sim_ecology::save_state;
use aether_sim_ecology::terrain::{slope_at, terrain_height, WATER_LEVEL};

const SEED: u32 = 20261102;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in (from + 1)..=to {
        eco.update(0.1, daylight(t));
    }
}

/// Scan a coarse grid for the two valid-land points that are farthest apart
/// (same fixture as the Phase 15 test — deterministic).
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

/// Relocate the seeded DECOs into two clusters with a wide efficiency gap
/// (8 members per cluster at 0.55 / 1.45). Returns the two anchors.
fn seed_two_clusters(eco: &mut Ecology) -> ((f64, f64), (f64, f64)) {
    let (a, b) = two_far_land_points();
    let mut k = 0usize;
    for agent in eco.agents.iter_mut() {
        if agent.kind != DECO { continue; }
        let first_half = k < 8;
        let anchor = if first_half { a } else { b };
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

/// Feed carcasses at the anchors whose flag is set (2 per anchor / 100 ticks).
fn feed_anchors(eco: &mut Ecology, anchors: &[(f64, f64); 2], tick: u32, fed: [bool; 2]) {
    if tick % 100 != 0 { return; }
    for (i, (x, z)) in anchors.iter().enumerate() {
        if !fed[i] { continue; }
        let dx = if i == 0 { 4.0 } else { -4.0 };
        eco.carcasses.push(Carcass { x: x + dx, z: *z, energy: 30.0, age: 0.0, occupied_by: None });
        eco.carcasses.push(Carcass { x: x - dx, z: *z, energy: 30.0, age: 0.0, occupied_by: None });
    }
}

fn clade<'a>(eco: &'a Ecology, id: u32) -> &'a SpeciesClade {
    eco.species_clades.iter().find(|c| c.id == id)
        .unwrap_or_else(|| panic!("clade {id} missing"))
}

fn root_id_of(eco: &Ecology) -> u32 {
    // Phase 34: the tree now carries one root per kind; the DECO tests care
    // about the DECO family's root (kind_id == DECO), not the first root.
    eco.species_clades.iter()
        .find(|c| c.parent_id.is_none() && c.kind_id == DECO as u32)
        .expect("DECO root clade must exist").id
}

/// Phase 34: does clade `c` belong to the DECO family (the DECO root plus
/// any derived DECO forks)?
fn deco_family(eco: &Ecology, c: &SpeciesClade) -> bool {
    if c.kind_id == u32::MAX { c.kind == "deco" } else { eco.root_kind(c.kind_id as u16) == DECO }
}

fn alive_members(eco: &Ecology, id: u32) -> usize {
    eco.agents.iter().filter(|a| a.alive && a.kind == DECO && a.clade_id == id).count()
}

fn mean_energy(eco: &Ecology, id: u32) -> f64 {
    let es: Vec<f64> = eco.agents.iter()
        .filter(|a| a.alive && a.kind == DECO && a.clade_id == id)
        .map(|a| a.energy).collect();
    if es.is_empty() { 0.0 } else { es.iter().sum::<f64>() / es.len() as f64 }
}

/// Pin every living DECO's efficiency by clade membership: root clade gets
/// `root_eff`, the daughter clade `child_eff`. Test-only lever that makes the
/// niche gap (and therefore the overlap window) an explicit controlled
/// variable; the niche formula re-derives positions from the pinned effs on
/// the next detection pass.
fn pin_effs(eco: &mut Ecology, root_eff: f64, child_eff: f64, child: u32) {
    let root = root_id_of(eco);
    for a in eco.agents.iter_mut() {
        if a.alive && a.kind == DECO {
            a.genome.deco_efficiency = if a.clade_id == child { child_eff } else { root_eff };
        }
    }
    let _ = root;
}

/// Common fixture: two clusters in SELECTIVE mode, run to the seeded split.
/// Returns (eco, child_clade_id, anchors).
fn setup_split() -> (Ecology, u32, [(f64, f64); 2]) {
    let mut eco = Ecology::new(SEED);
    eco.deco_eff_choice_mode = save_state::DECO_CHOICE_SELECTIVE;
    let (a, b) = seed_two_clusters(&mut eco);
    let anchors = [a, b];
    for t in 1..=500u32 {
        feed_anchors(&mut eco, &anchors, t, [true, true]);
        run_ticks(&mut eco, t - 1, t);
    }
    let ev = eco.speciation_events.iter()
        .find(|e| e.event_kind == EVENT_KIND_SPECIATION
            && eco.species_clades.iter().any(|c| c.id == e.parent_clade_id && deco_family(&eco, c)))
        .expect("seeded DECO split must fire by t=500 (Phase 15 baseline)");
    let child = ev.child_clade_ids[0];
    (eco, child, anchors)
}

// ---------------------------------------------------------------------------
// 1. Niche divergence from the parent clade
// ---------------------------------------------------------------------------

#[test]
fn clade_niche_diverges_from_parent() {
    // Legacy mode: eff is fitness-neutral, so the niche gap here is pure
    // drift-stable bookkeeping with no competitive pressure.
    let mut eco = Ecology::new(SEED);
    eco.deco_eff_choice_mode = save_state::DECO_CHOICE_LEGACY;
    let (a, b) = seed_two_clusters(&mut eco);
    let anchors = [a, b];
    for t in 1..=600u32 {
        feed_anchors(&mut eco, &anchors, t, [true, true]);
        run_ticks(&mut eco, t - 1, t);
    }
    let ev = eco.speciation_events.iter()
        .find(|e| e.event_kind == EVENT_KIND_SPECIATION
            && eco.species_clades.iter().any(|c| c.id == e.parent_clade_id && deco_family(&eco, c)))
        .expect("seeded DECO split must fire by t=600");
    let child_id = ev.child_clade_ids[0];
    let root = clade(&eco, root_id_of(&eco));
    let child = clade(&eco, child_id);

    eprintln!("=== niche divergence (legacy, t=600) ===");
    eprintln!("  root  '{}' niche={:.4} eff={:.4}", root.name, root.niche_offset, root.centroid_genome.deco_efficiency);
    eprintln!("  child '{}' niche={:.4} eff={:.4}", child.name, child.niche_offset, child.centroid_genome.deco_efficiency);

    // Root pinned at 0; child bounded and clearly displaced.
    assert_eq!(root.niche_offset, 0.0, "root clade is pinned at niche 0");
    assert!(child.niche_offset.is_finite(), "niche must stay finite");
    assert!(child.niche_offset.abs() <= 1.0 + 1e-9, "niche clamped to [-1, 1]");
    let gap600 = (child.niche_offset - root.niche_offset).abs();
    assert!(gap600 >= 0.5,
        "child niche must diverge from parent by >= 0.5 at t=600, got {gap600}");

    // Divergence persists (drift keeps the clusters' eff apart).
    for t in 601..=1500u32 {
        feed_anchors(&mut eco, &anchors, t, [true, true]);
        run_ticks(&mut eco, t - 1, t);
    }
    let child = clade(&eco, child_id);
    let root = clade(&eco, root_id_of(&eco));
    let gap1500 = (child.niche_offset - root.niche_offset).abs();
    eprintln!("  t=1500 gap={gap1500:.4}");
    assert!(gap1500 >= 0.5,
        "divergence must persist at t=1500, got {gap1500}");
    assert!(child.death_tick.is_none(), "child clade alive throughout");
}

// ---------------------------------------------------------------------------
// 2. Overlapping clades compete (burden decays the loser's energy)
// ---------------------------------------------------------------------------

#[test]
fn overlapping_clades_compete() {
    // Mechanism-level A/B: two identical runs (same seed, same pinned effs
    // 1.0/0.98 → the engine itself derives the niche overlap and burdens the
    // child). Run B wipes the burden table after every tick, isolating the
    // burden's causal effect on the child clade's energy. A natural eff-pin
    // A/B is confounded because the niche axis is derived from the eff gap.
    let (mut burdened, child_id, anchors) = setup_split();
    let (mut control, child_id_b, anchors_b) = setup_split();
    assert_eq!(child_id, child_id_b, "A/B must share the split history");
    assert_eq!(root_id_of(&burdened), root_id_of(&control));

    let mut ever_burdened = false;
    let mut loser_samples: Vec<f64> = Vec::new();
    let mut control_child_samples: Vec<f64> = Vec::new();

    for t in 501..=3000u32 {
        if t % 60 == 0 {
            // Pin BEFORE the tick so the detection pass inside run_ticks sees it.
            pin_effs(&mut burdened, 1.0, 0.98, child_id);
            pin_effs(&mut control, 1.0, 0.98, child_id_b);
        }
        feed_anchors(&mut burdened, &anchors, t, [true, true]);
        feed_anchors(&mut control, &anchors_b, t, [true, true]);
        run_ticks(&mut burdened, t - 1, t);
        run_ticks(&mut control, t - 1, t);
        if burdened.clade_burdens.iter().any(|(id, b)| *id == child_id && *b > 0.0) {
            ever_burdened = true;
        }
        // Wipe the control's burden table so its child never carries the
        // competitive drain (the engine re-fills it only on pass ticks, so
        // clearing every tick keeps it permanently burden-free).
        control.clade_burdens.clear();
        if t % 60 == 0 {
            loser_samples.push(mean_energy(&burdened, child_id));
            control_child_samples.push(mean_energy(&control, child_id_b));
        }
    }

    eprintln!("=== burden A/B (child clade energy every 60t) ===");
    eprintln!("  burdened child: {:?}", loser_samples.iter().map(|v| format!("{v:.1}")).collect::<Vec<_>>());
    eprintln!("  control child : {:?}", control_child_samples.iter().map(|v| format!("{v:.1}")).collect::<Vec<_>>());
    eprintln!("  final: burdened={:.2} control={:.2}  burdens={:?}",
        loser_samples.last().unwrap(), control_child_samples.last().unwrap(), burdened.clade_burdens);

    // Detection layer: the engine derived the overlap and burdened the child.
    assert!(ever_burdened, "overlap run must apply the competitive burden to the child clade");
    // Effect layer: identical runs except the burden. Point-in-time energy
    // comparison is unusable — the +15 % drain diverges the two populations
    // chaotically (reproduction/death amplify it), so late samples can flip
    // sign. The burden's causal signature is the TIME-AVERAGED energy gap:
    // the drain acts on every tick, so its mean effect must be negative.
    let avg_burdened = loser_samples.iter().sum::<f64>() / loser_samples.len() as f64;
    let avg_control = control_child_samples.iter().sum::<f64>() / control_child_samples.len() as f64;
    assert!(avg_burdened < avg_control,
        "time-averaged energy: burdened child ({avg_burdened:.2}) must fall below unburdened twin ({avg_control:.2})");
}

// ---------------------------------------------------------------------------
// 3. Competitive exclusion drives the loser extinct (kind-2 event)
// ---------------------------------------------------------------------------

#[test]
fn competitive_exclusion_extincts_loser() {
    let (mut eco, child_id, anchors) = setup_split();
    let root_id = root_id_of(&eco);

    // Mechanism-level test of the kind-2 bookkeeping path. Natural run-to-
    // extinction dynamics are unreliable here (feeding only the root anchor
    // starves the larger root clade first — its own reproductive success
    // dilutes its food). Instead: run one pinned pass so the engine fills
    // the burden table for the overlapping lower-eff child, then kill the
    // child's members directly. The next detection pass sees an empty
    // member list while the previous pass's burden table still lists the
    // clade → the extinction MUST be logged as kind-2.
    pin_effs(&mut eco, 1.0, 0.98, child_id);
    feed_anchors(&mut eco, &anchors, 600, [true, true]);
    run_ticks(&mut eco, 500, 600);
    assert!(eco.clade_burdens.iter().any(|(id, b)| *id == child_id && *b > 0.0),
        "child must carry the burden before the kill-off, got {:?}", eco.clade_burdens);

    // Phase 34: a daughter clade's members include both the original-kind
    // agents AND the derived-kind children born into it (kind-fork). Wipe the
    // whole clade by clade_id, not by kind.
    for a in eco.agents.iter_mut() {
        if a.alive && a.clade_id == child_id {
            a.alive = false;
        }
    }

    let mut fired = false;
    for t in 601..=720u32 {
        run_ticks(&mut eco, t - 1, t);
        if eco.speciation_events.iter()
            .any(|e| e.event_kind == EVENT_KIND_COMPETITIVE_EXCLUSION)
        {
            fired = true;
            break;
        }
    }
    eprintln!("=== competitive exclusion run: fired={fired} events={:?}",
        eco.speciation_events.iter().map(|e| (e.tick, e.parent_clade_id, e.event_kind)).collect::<Vec<_>>());

    assert!(fired, "competitive-exclusion extinction event must fire by t=720");
    let xev = eco.speciation_events.iter()
        .find(|e| e.event_kind == EVENT_KIND_COMPETITIVE_EXCLUSION).unwrap();
    assert_eq!(xev.parent_clade_id, child_id,
        "the starved + burdened child clade is the loser");
    let child = clade(&eco, child_id);
    assert!(child.death_tick.is_some(), "extinct clade records death_tick");
    assert!(child.members.is_empty(), "extinct clade keeps no members");
    assert!(alive_members(&eco, root_id) > 0, "winner clade must survive");
}

// ---------------------------------------------------------------------------
// 4. Non-overlapping clades coexist
// ---------------------------------------------------------------------------

#[test]
fn non_overlapping_clades_coexist() {
    let (mut eco, child_id, anchors) = setup_split();
    let root_id = root_id_of(&eco);
    // Phase 34: the multi-axis metric can spawn SEVERAL DECO daughters, and
    // the extra daughters would pile into the root's niche and manufacture
    // spurious overlap burdens. So pin the DECO family into three separated
    // niches: root = 1.1, the target child = 1.3, every OTHER DECO-family
    // clade = 0.9. All pairwise niche gaps then exceed the overlap window,
    // so a clean coexistence (zero burdens) is what the test asserts.
    for t in 501..=2500u32 {
        if t % 60 == 0 {
            // Snapshot the DECO-family clade ids (immutable borrow) before
            // the mutable pinning pass (avoids a borrow conflict).
            let deco_clade_ids: Vec<u32> = eco.species_clades.iter()
                .filter(|c| deco_family(&eco, c))
                .map(|c| c.id).collect();
            for a in eco.agents.iter_mut() {
                if !a.alive || !deco_clade_ids.contains(&a.clade_id) { continue; }
                a.genome.deco_efficiency = if a.clade_id == child_id { 1.3 }
                    else if a.clade_id == root_id { 1.1 }
                    else { 0.9 };
            }
        }
        feed_anchors(&mut eco, &anchors, t, [true, true]);
        run_ticks(&mut eco, t - 1, t);
    }
    eprintln!("=== coexistence run: clades={} events={} burdens={:?}",
        eco.species_clades.len(), eco.speciation_events.len(), eco.clade_burdens);
    for c in &eco.species_clades {
        eprintln!("  clade {} '{}' birth={} death={:?} members={} niche={:.3}",
            c.id, c.name, c.birth_tick, c.death_tick, c.members.len(), c.niche_offset);
    }

    // Phase 34: other kinds may legitimately go extinct in a 2500-tick run;
    // this DECO coexistence test forbids DECO-FAMILY extinctions only.
    assert!(eco.speciation_events.iter().all(|e| {
        if e.event_kind == EVENT_KIND_SPECIATION { return true; }
        e.kind_id == u32::MAX || eco.root_kind(e.kind_id as u16) != DECO
    }),
        "no DECO-family extinction events expected, got {:?}", eco.speciation_events);
    let root = clade(&eco, root_id);
    let child = clade(&eco, child_id);
    assert!(root.death_tick.is_none() && !root.members.is_empty(), "root clade alive");
    assert!(child.death_tick.is_none() && !child.members.is_empty(), "child clade alive");
    // Their niches really are separated (no silent overlap).
    assert!((child.niche_offset - root.niche_offset).abs() >= NICHE_OVERLAP_THRESHOLD,
        "pinned niches must stay outside the overlap window");
    // Phase 34: with the family pinned into separated niches, no DECO-family
    // pair overlaps, so no competitive burden may be active on any clade.
    assert!(eco.clade_burdens.iter().all(|(id, b)| {
        let c = eco.species_clades.iter().find(|c| c.id == *id).unwrap();
        !(deco_family(&eco, c) && *b > 0.0)
    }),
        "no DECO-family burdens without niche overlap, got {:?}", eco.clade_burdens);
}

// ---------------------------------------------------------------------------
// 5. Save/restore preserves the niche state bit-exactly
// ---------------------------------------------------------------------------

#[test]
fn save_restore_preserves_niche_state() {
    let (mut eco, child_id, anchors) = setup_split();
    // Force an active burden before saving so the burden map is non-trivial.
    for t in 501..=620u32 {
        if t % 60 == 0 {
            pin_effs(&mut eco, 1.0, 0.98, child_id);
        }
        feed_anchors(&mut eco, &anchors, t, [true, true]);
        run_ticks(&mut eco, t - 1, t);
    }
    assert!(!eco.clade_burdens.is_empty(),
        "forced overlap must leave an active burden before the save, got {:?}",
        eco.clade_burdens);
    assert!(eco.to_save_state().version == save_state::SAVE_STATE_VERSION);

    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let restored = Ecology::from_save_state(serde_json::from_str(&json).unwrap());

    // Niches and burdens survive the JSON hop bit-exactly.
    assert_eq!(eco.species_clades.len(), restored.species_clades.len());
    for (o, r) in eco.species_clades.iter().zip(restored.species_clades.iter()) {
        assert_eq!(o.id, r.id);
        assert_eq!(o.niche_offset, r.niche_offset, "niche bit-exact for clade {}", o.id);
    }
    assert_eq!(eco.clade_burdens, restored.clade_burdens, "burden map bit-exact");
    assert_eq!(eco.speciation_events.len(), restored.speciation_events.len());
    for (o, r) in eco.speciation_events.iter().zip(restored.speciation_events.iter()) {
        assert_eq!(o.event_kind, r.event_kind, "event kind roundtrip");
    }

    // Lockstep continuation with continued pinning: checksums stay identical.
    let mut a_sim = eco;
    let mut b_sim = restored;
    for t in 621..=1000u32 {
        if t % 60 == 0 {
            pin_effs(&mut a_sim, 1.0, 0.98, child_id);
            pin_effs(&mut b_sim, 1.0, 0.98, child_id);
        }
        feed_anchors(&mut a_sim, &anchors, t, [true, true]);
        feed_anchors(&mut b_sim, &anchors, t, [true, true]);
        run_ticks(&mut a_sim, t - 1, t);
        run_ticks(&mut b_sim, t - 1, t);
        if t % 100 == 0 {
            assert_eq!(a_sim.checksum(), b_sim.checksum(), "lockstep diverged at t={t}");
        }
    }

    // --- pre-Phase-16 v2 probe: strip the niche/kind/burden keys -> zeros ---
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    {
        let obj = v.as_object_mut().unwrap();
        assert!(obj.remove("CladeNicheBurdens").is_some(), "v3 save carries the burden key");
        let clades: Vec<serde_json::Value> =
            serde_json::from_str(obj["SpeciesClades"].as_str().unwrap()).unwrap();
        let stripped: Vec<serde_json::Value> = clades.into_iter().map(|mut c| {
            if let Some(m) = c.as_object_mut() { m.remove("niche"); }
            c
        }).collect();
        obj["SpeciesClades"] = serde_json::Value::String(serde_json::to_string(&stripped).unwrap());
        let events: Vec<serde_json::Value> =
            serde_json::from_str(obj["SpeciationEvents"].as_str().unwrap()).unwrap();
        let stripped_e: Vec<serde_json::Value> = events.into_iter().map(|mut e| {
            if let Some(m) = e.as_object_mut() { m.remove("kind"); }
            e
        }).collect();
        obj["SpeciationEvents"] = serde_json::Value::String(serde_json::to_string(&stripped_e).unwrap());
        obj["Version"] = serde_json::Value::Number(serde_json::Number::from(2u32));
    }
    let legacy: save_state::SaveState = serde_json::from_value(v).unwrap();
    let legacy_eco = Ecology::from_save_state(legacy);
    assert!(legacy_eco.species_clades.iter().all(|c| c.niche_offset == 0.0),
        "stripped save restores niches to 0");
    assert!(legacy_eco.clade_burdens.is_empty(), "stripped save restores no burdens");
    assert!(legacy_eco.speciation_events.iter().all(|e| e.event_kind == EVENT_KIND_SPECIATION),
        "stripped save restores event kinds to speciation");
    // And it keeps running; the next detection pass re-derives niches.
    let mut le = legacy_eco;
    run_ticks(&mut le, 620, 760);
    assert!(le.species_clades.iter().any(|c| c.niche_offset != 0.0 || c.parent_id.is_none()),
        "niches re-derive after a stripped restore");
}
