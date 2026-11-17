//! Phase 47 acceptance tests — geographic isolation maintenance.
//!
//! The world is one Ecology sandbox, but every agent carries a home region
//! (one of four angular islands). Mating is region-local; dispersal is a
//! priority-effect colonisation of a region that has no conspecific yet. The
//! tests here pin the invariants that make the tree grow: balanced founding
//! demes, one shared family carrying capacity, deterministic dispersal,
//! v16 save roundtrip and v15 archive compatibility.

use aether_sim_ecology::ecology::{
    Ecology, SpeciesKind, DECO, DISPERSAL_P, REGION_COUNT,
};
use aether_sim_ecology::save_state;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in (from + 1)..=to {
        eco.update(0.1, daylight(t));
    }
}

#[test]
fn initial_regions_are_balanced_and_deco_founders_cluster() {
    let eco = Ecology::new(20261102);
    for k in 0..eco.kinds.len() as SpeciesKind {
        let n = eco.count(k);
        if n < REGION_COUNT as usize { continue; }
        let mut counts = [0usize; REGION_COUNT as usize];
        for a in eco.living().iter().filter(|a| a.alive && a.kind == k) {
            counts[(a.region as usize).min(REGION_COUNT as usize - 1)] += 1;
        }
        let lo = *counts.iter().min().unwrap();
        let hi = *counts.iter().max().unwrap();
        assert!(hi - lo <= 1,
            "kind {} has unbalanced regions {:?} (n={})", eco.kind_name(k), counts, n);
    }
    // DECO's four founder clusters must be spatially compact; otherwise the
    // 3-4 founders never meet and the island model loses three of four demes
    // before its first birth.
    for r in 0..REGION_COUNT {
        let members: Vec<&_> = eco.living().iter()
            .filter(|a| a.alive && a.kind == DECO && a.region == r)
            .collect();
        assert!(!members.is_empty(), "DECO region {r} starts empty");
        for i in 0..members.len() {
            for j in (i + 1)..members.len() {
                let d = ((members[i].home_x - members[j].home_x).powi(2)
                    + (members[i].home_z - members[j].home_z).powi(2)).sqrt();
                assert!(d <= 60.0, "DECO founders in region {r} are {d:.1} apart");
            }
        }
    }
}

#[test]
fn family_capacity_bounds_deco_total_across_splits() {
    let mut eco = Ecology::new(20261102);
    // Phase 47 A/B arm: pin the Phase-47 behaviour (family-shared cap,
    // legacy 2-member split gate) so this test keeps verifying exactly what
    // it was written for. The Phase 49 default (per-kind budgets, 8-member
    // gate) is covered by the Phase 49 tests.
    eco.budget_fork_off();
    run_ticks(&mut eco, 0, 4_000);
    let cap = eco.kind_def(DECO).global_cap;
    assert!(eco.count_family(DECO) <= cap,
        "DECO family exceeded its guild cap: {} > {cap}", eco.count_family(DECO));
    assert!(eco.species_clades().len() >= 2,
        "the region-island world must have split by t=4000");
}

#[test]
fn dispersal_off_is_zero_events_and_bit_exact() {
    let mut a = Ecology::new(20261102);
    let mut b = Ecology::new(20261102);
    a.dispersal_p = 0.0;
    b.dispersal_p = 0.0;
    run_ticks(&mut a, 0, 600);
    run_ticks(&mut b, 0, 600);
    assert_eq!(a.dispersal_events, 0);
    assert_eq!(a.checksum(), b.checksum(), "p=0 double run diverged");
}

#[test]
fn priority_colonisation_fires_and_regions_are_preserved() {
    let mut eco = Ecology::new(20261102);
    eco.dispersal_p = 1.0;
    run_ticks(&mut eco, 0, 1_500);
    assert!(eco.dispersal_events > 0, "p=1 must colonise a vacant region");
    for a in eco.living().iter().filter(|a| a.alive) {
        assert!((a.region as usize) < REGION_COUNT as usize,
            "region id {} out of range", a.region);
    }
}

#[test]
fn save_v16_roundtrip_preserves_regions_and_dispersal_stream() {
    let mut eco = Ecology::new(20261102);
    eco.dispersal_p = 0.2;
    run_ticks(&mut eco, 0, 400);

    let state = eco.to_save_state();
    assert_eq!(state.version, save_state::SAVE_STATE_VERSION);
    assert!(state.RegionIsolation);
    let json = serde_json::to_string(&state).unwrap();
    let parsed: save_state::SaveState = serde_json::from_str(&json).unwrap();
    let mut restored = Ecology::from_save_state(parsed);

    assert_eq!(restored.dispersal_p, eco.dispersal_p);
    assert_eq!(restored.dispersal_events, eco.dispersal_events);
    assert_eq!(restored.region_isolation, eco.region_isolation);

    for target in [450u32, 500u32] {
        run_ticks(&mut eco, 400, target);
        run_ticks(&mut restored, 400, target);
        assert_eq!(eco.checksum(), restored.checksum(), "checksum diverged at t={target}");
        assert_eq!(
            serde_json::to_string(&eco.to_save_state()).unwrap(),
            serde_json::to_string(&restored.to_save_state()).unwrap(),
            "serialized state diverged at t={target}"
        );
    }
}

#[test]
fn v15_archive_loads_with_region_defaults() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 0, 100);
    let mut val = serde_json::to_value(eco.to_save_state()).unwrap();
    {
        let obj = val.as_object_mut().unwrap();
        obj.insert("Version".into(), serde_json::json!(15));
        for k in ["DispersalP", "DispersalRngState", "DispersalEvents",
                  "DispersalLastTick", "RegionIsolation", "DecoMutScale"] {
            obj.remove(k);
        }
        if let Some(agents) = obj.get_mut("Agents").and_then(|v| v.as_array_mut()) {
            for a in agents {
                a.as_object_mut().unwrap().remove("Region");
            }
        }
    }
    let v15: save_state::SaveState = serde_json::from_value(val).expect("v15 archive must deserialize");
    let mut restored = Ecology::from_save_state(v15);
    assert!((restored.dispersal_p - DISPERSAL_P).abs() < 1e-12,
        "pre-v16 archives land on the current dispersal calibration");
    assert!(restored.region_isolation, "pre-v16 archives land on the island model");
    assert_eq!(restored.dispersal_events, 0);
    for a in restored.living().iter().filter(|a| a.alive) {
        assert!((a.region as usize) < REGION_COUNT as usize);
    }
    // Balanced assignment must also happen for restored legacy agents.
    for k in 0..restored.kinds.len() as SpeciesKind {
        let n = restored.count(k);
        if n < REGION_COUNT as usize { continue; }
        let mut counts = [0usize; REGION_COUNT as usize];
        for a in restored.living().iter().filter(|a| a.alive && a.kind == k) {
            counts[a.region as usize] += 1;
        }
        let lo = *counts.iter().min().unwrap();
        let hi = *counts.iter().max().unwrap();
        assert!(hi - lo <= 1, "legacy restore region imbalance {:?}", counts);
    }
    run_ticks(&mut restored, 100, 160);
    assert!(restored.checksum().starts_with("0x"));
}

#[test]
fn dispersal_world_double_run_bit_exact() {
    let mut a = Ecology::new(20261102);
    let mut b = Ecology::new(20261102);
    a.dispersal_p = 0.2;
    b.dispersal_p = 0.2;
    run_ticks(&mut a, 0, 3_000);
    run_ticks(&mut b, 0, 3_000);
    assert!(a.dispersal_events > 0);
    assert_eq!(a.checksum(), b.checksum());
    assert_eq!(a.dispersal_events, b.dispersal_events);
    assert_eq!(a.species_clades().len(), b.species_clades().len());
}
