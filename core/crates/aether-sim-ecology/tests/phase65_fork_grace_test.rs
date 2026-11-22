//! Phase 65 W1 — fork-grace wiring on the real fork path.
//!
//! The gate/settlement contracts live in `ecology.rs`'s
//! `phase65_fork_grace_tests`. This file only proves the arm is wired through
//! a *real* `detect_splits` fork (not just the synthetic-kind fixture): a
//! derived kind must receive `now + FORK_GRACE_TICKS` with the arm on, never
//! with it off, and the family total must stay conserved either way.
//!
//! Fixture: the Phase 34 forced-fork setup (two geographically isolated PRED
//! clusters with a wide multi-axis gap), pinned to the legacy 2-member gate
//! because the clusters hold 7 members each — the mint path is under test here,
//! not the production `MIN_SIDE = 8`.

use aether_sim_ecology::ecology::{
    Carcass, Ecology, SpeciesKind, FORK_GRACE_TICKS, PRED, SPECIATION_MIN_SIDE_LEGACY,
};
use aether_sim_ecology::terrain::{slope_at, terrain_height, WATER_LEVEL};

const SEED: u32 = 20261116;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in (from + 1)..=to {
        eco.update(0.1, daylight(t));
    }
}

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
    let mut best = (pts[0], pts[pts.len() - 1]);
    for i in 0..pts.len() {
        for j in (i + 1)..pts.len() {
            let d = (pts[i].0 - pts[j].0).hypot(pts[i].1 - pts[j].1);
            if d > best_d {
                best_d = d;
                best = (pts[i], pts[j]);
            }
        }
    }
    best
}

fn seed_two_pred_clusters(eco: &mut Ecology) -> ((f64, f64), (f64, f64)) {
    let (a, b) = two_far_land_points();
    let mut k = 0usize;
    let total = eco.agents.iter().filter(|a| a.alive && a.kind == PRED).count();
    for agent in eco.agents.iter_mut() {
        if agent.kind != PRED || !agent.alive {
            continue;
        }
        let first_half = k < total / 2;
        let anchor = if first_half { a } else { b };
        agent.x = anchor.0;
        agent.z = anchor.1;
        agent.prev_x = anchor.0;
        agent.prev_z = anchor.1;
        agent.home_x = anchor.0;
        agent.home_z = anchor.1;
        agent.genome.speed = if first_half { 3.6 } else { 8.4 };
        agent.genome.sense = if first_half { 20.0 } else { 100.0 };
        agent.genome.size = if first_half { 0.8 } else { 1.6 };
        k += 1;
    }
    (a, b)
}

/// Run the fixture to its first fork. Returns the world, the derived kind id
/// and the fork tick.
fn forced_fork(grace: bool) -> (Ecology, SpeciesKind, u32) {
    let mut eco = Ecology::new(SEED);
    eco.speciation_min_side = SPECIATION_MIN_SIDE_LEGACY;
    if grace { eco.fork_grace_on(); } else { eco.fork_grace_off(); }
    let (a, b) = seed_two_pred_clusters(&mut eco);
    let anchors = [a, b];
    for t in 1..=500u32 {
        if t % 100 == 0 {
            for &(x, z) in &anchors {
                eco.carcasses.push(Carcass { x: x + 4.0, z, energy: 30.0, age: 0.0, occupied_by: None });
            }
        }
        run_ticks(&mut eco, t - 1, t);
    }
    let fork = eco.speciation_events.iter()
        .find(|e| e.event_kind == 0)
        .cloned()
        .expect("the forced PRED fixture must fork");
    let child_id = fork.child_clade_ids[0];
    let derived = eco.species_clades.iter()
        .find(|c| c.id == child_id)
        .and_then(|c| c.derived_kind)
        .expect("daughter must carry a derived kind");
    (eco, derived as SpeciesKind, fork.tick)
}

#[test]
fn a_real_fork_opens_the_window_only_with_the_arm_on() {
    let (eco_on, d_on, fork_tick) = forced_fork(true);
    let (eco_off, d_off, fork_tick_off) = forced_fork(false);

    // The arm changes nothing before the first fork (no derived kind exists),
    // so both worlds fork at the same tick.
    assert_eq!(fork_tick, fork_tick_off, "the arm is inert before the first fork");
    assert_eq!(eco_on.kind_grace_until(d_on), fork_tick + FORK_GRACE_TICKS);
    assert_eq!(eco_off.kind_grace_until(d_off), 0, "no window without the arm");
    assert_eq!(eco_on.family_budget_total(PRED), eco_off.family_budget_total(PRED),
        "the fork transfer conserves the family total");
    assert_eq!(eco_on.grace_total_skips(), 0,
        "the fixture's daughter has not exceeded its grant yet");
    assert_eq!(eco_off.grace_total_skips(), 0);
}
