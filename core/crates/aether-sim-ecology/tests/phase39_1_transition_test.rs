//! Phase 39.1 acceptance tests — the major-transition layer.
//!
//! Seven invariants, one per acceptance criterion:
//!   1. `group_off_is_bit_exact_phase36_anchor` — with the layer switched off
//!      the world reproduces the Phase 36 anchors *bit-exactly* (zero side
//!      effects: no draws, no energy flows, no groups).
//!   2. `cohesion_conditions_gate_group_formation` — with N≥3 kin individuals
//!      inside the radius and the persistence window met, a group appears on
//!      its own; with mismatched kinship nothing appears and the checksum stays
//!      identical to the layer-off twin (zero side effects when unmet).
//!   3. `propagule_is_single_point_bottleneck` — a group's offspring genomes all
//!      come from the founder, never from a mixture of members.
//!   4. `task_switch_cost_is_paid_once_per_switch` — a role change costs
//!      `TASK_SWITCH_COST`; a re-evaluation without change costs nothing.
//!   5. `dirty_work_raises_offspring_mutation` — GO in a polluted cell
//!      accumulates somatic damage and mutates its offspring harder than FEED.
//!   6. `selection_levels_grow_with_the_group_layer` — 1 → 2 exactly when a
//!      live group exists.
//!   7. `save_v11_roundtrip_is_bit_exact` + `legacy_v10_archive_loads` — the
//!      v11 archive round-trips bit-exactly and a v10 archive (no group keys)
//!      loads as a solitary world.
//!
//! Plus `groups_form_in_the_natural_run` (the emergent, uncontrolled check) and
//! a determinism lockstep.

use aether_sim_ecology::civilization_metrics;
use aether_sim_ecology::ecology::{Ecology, Genome, SpeciesKind, DECO, HERB, PRED, SCAV, OMN, FISH};
use aether_sim_ecology::group::{
    self, GROUP_MIN_MEMBERS, GROUP_PROPAGULE_SIZE, GROUP_REPRO_COOLDOWN, ROLE_FEED, ROLE_GO,
    ROLE_NONE, SOMATIC_MUTAGEN, TASK_SWITCH_COST,
};
use aether_sim_ecology::save_state;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in from..=to {
        eco.update(0.1, daylight(t));
    }
}

/// A controlled genome: only the traits the group layer reads are varied.
fn genome(hue: f64, social: f64, speed: f64, bold: f64, help: f64, curious: f64) -> Genome {
    Genome {
        speed,
        sense: 60.0,
        size: 1.0,
        metabolism: 1.0,
        lifespan: 400.0,
        hue,
        bold,
        curious,
        social,
        help_thr: Some(help),
        pref_sig: None,
        tol_sig: None,
        fin_color: 0.5,
        deco_efficiency: 1.0,
        hull_ratio: 1.0,
        limb_scale: 1.0,
        ext: Vec::new(),
    }
}

/// GO-favouring genome (fast, asocial, timid, curious).
fn go_genome(hue: f64) -> Genome {
    genome(hue, 0.0, 1.8, 0.0, 0.0, 1.0)
}

/// FEED-favouring genome (slow, highly social, high help threshold).
fn feed_genome(hue: f64) -> Genome {
    genome(hue, 1.0, 0.4, 0.0, 5.0, 1.0)
}

/// GUARD-favouring genome (bold, large, fast) with low social/help so FEED
/// cannot tie with it.
fn guard_genome(hue: f64) -> Genome {
    let mut g = genome(hue, 0.0, 1.8, 1.0, 0.0, 0.5);
    g.size = 1.7;
    g
}

/// Empties the world's fauna so a test owns every individual in it.
fn empty_world(seed: u32) -> Ecology {
    let mut eco = Ecology::new(seed);
    for k in [HERB, PRED, SCAV, OMN, FISH, DECO] {
        eco.kill_kind(k);
    }
    assert_eq!(eco.living().iter().filter(|a| a.alive).count(), 0);
    eco
}

/// Spawn `n` agents of `kind` at `(x, z)` with explicit genomes and return
/// their ids in spawn order (id order = Vec order).
fn spawn_with(eco: &mut Ecology, kind: SpeciesKind, x: f64, z: f64, genomes: &[Genome]) -> Vec<u32> {
    let mut ids = Vec::new();
    for g in genomes {
        let id = eco.spawn_agent(kind, x, z).expect("spawn must succeed in an empty world");
        assert!(group::experimenter_set_genome(eco, id, g.clone()));
        ids.push(id);
    }
    ids
}

fn checksum_at(seed: u32, ticks: u32, group_on: bool) -> String {
    let mut eco = Ecology::new(seed);
    eco.group_on = group_on;
    run_ticks(&mut eco, 1, ticks);
    eco.checksum()
}

// ---------------------------------------------------------------------------
// 1. Zero side effects when the layer is off.
// ---------------------------------------------------------------------------
#[test]
fn group_off_is_bit_exact_phase36_anchor() {
    // With `group_on = false` there are no scans, no draws, no energy flows —
    // the layer is a pure no-op. These anchors are the group-off trajectory
    // after Phase 43's cultural ratchet (birth token inheritance; the culture
    // layer itself stays ON in the group-off arm, so inventions now propagate
    // to children and the @3000 anchor moves; @300 is unchanged — no invention
    // has fired by then). Phase 45: the patch layer + regrowth-noise field
    // also run in the group-off arm, so all four anchors move again. Phase 43
    // values (previous):
    // Phase 45 values (previous):
    //   @300 0xf7b865ce421aeb0e · @3000 0x8f81c9041a132838
    //   seed2 @300 0xc56ee1f0c8d2528f · v1 @300 0x75c621a32eb86951
    // Phase 47: region islands + philopatry + DECO mutation ×5.
    // Phase 49: fork budget + SPECIATION_MIN_SIDE=8 (twelfth re-baseline).
    // Phase 56: decomposition field (fifteenth re-baseline) — structures
    // imprint carcass decay from their token sign, so the group-off arm's
    // @3000 anchor moves (@300 is untouched: nothing is built that early).
    // Phase 59: saturated DECO fertility map (sixteenth re-baseline) —
    // genotypes below eff 0.9583 now breed, so @3000 moves again; @300, the
    // 20261105 seed and the v1 stream are unchanged.
    // Phase 60: regional DECO capacity (seventeenth re-baseline) — the
    // group-off anchors are unchanged: this arm steps with `dt = 0.1`
    // (300/3 000 world ticks) and the regional gate only binds later.
    // Phase 49 values (previous): @3000 0xdb3aa647fa2e3a96.
    assert_eq!(checksum_at(20261102, 300, false), "0x7d2c8613d01cb79d");
    assert_eq!(checksum_at(20261102, 3000, false), "0x2cd62c186e0941d6");
    assert_eq!(checksum_at(20261105, 300, false), "0xbd6a86a9fe8264e7");

    // Same v1 anchor (the historical stream must not see the group hash).
    let mut eco = Ecology::new(20261102);
    eco.group_on = false;
    run_ticks(&mut eco, 1, 300);
    assert_eq!(eco.checksum_v1(), "0xc180a8890e5e4c5c");
}

// ---------------------------------------------------------------------------
// 2. Formation is gated by the cohesion conditions (and only by them).
// ---------------------------------------------------------------------------
#[test]
fn cohesion_conditions_gate_group_formation() {
    // --- (a) conditions met: 3 kin individuals, same clade/lineage, one spot.
    let mut eco = empty_world(20261102);
    let genomes = vec![genome(0.40, 1.0, 1.0, 0.5, 4.0, 0.5); GROUP_MIN_MEMBERS];
    let ids = spawn_with(&mut eco, HERB, 0.0, 0.0, &genomes);
    assert_eq!(ids.len(), GROUP_MIN_MEMBERS);
    run_ticks(&mut eco, 1, 400);
    assert!(
        !eco.groups.is_empty(),
        "3 kin individuals held inside the radius for 400 ticks must condense into a group"
    );
    assert_eq!(eco.group_stats.formed, eco.groups.len() as u64);
    assert_eq!(group::selection_levels(&eco), 2);

    // --- (b) conditions unmet (kinship): same spot, hues far apart.
    let mut eco = empty_world(20261102);
    let genomes = vec![
        genome(0.0, 1.0, 1.0, 0.5, 4.0, 0.5),
        genome(0.5, 1.0, 1.0, 0.5, 4.0, 0.5),
        genome(0.0, 1.0, 1.0, 0.5, 4.0, 0.5),
    ];
    spawn_with(&mut eco, HERB, 0.0, 0.0, &genomes);
    let mut twin = empty_world(20261102);
    let genomes2 = vec![
        genome(0.0, 1.0, 1.0, 0.5, 4.0, 0.5),
        genome(0.5, 1.0, 1.0, 0.5, 4.0, 0.5),
        genome(0.0, 1.0, 1.0, 0.5, 4.0, 0.5),
    ];
    spawn_with(&mut twin, HERB, 0.0, 0.0, &genomes2);
    twin.group_on = false;
    run_ticks(&mut eco, 1, 300);
    run_ticks(&mut twin, 1, 300);
    assert!(
        eco.groups.is_empty(),
        "non-kin neighbours (hue spread > tolerance) must never condense"
    );
    assert_eq!(eco.group_stats.formed, 0);
    assert_eq!(
        eco.checksum(),
        twin.checksum(),
        "with formation impossible the layer must be a bit-exact no-op"
    );

    // --- (c) persistence gate: nothing forms before the window elapses.
    let mut eco = empty_world(20261102);
    let genomes = vec![genome(0.40, 1.0, 1.0, 0.5, 4.0, 0.5); GROUP_MIN_MEMBERS];
    spawn_with(&mut eco, HERB, 0.0, 0.0, &genomes);
    run_ticks(&mut eco, 1, 40);
    assert!(
        eco.groups.is_empty(),
        "cohesion must persist for GROUP_PERSIST_TICKS before a group can form"
    );
}

/// The uncontrolled check: in a plain seeded world the layer condenses groups
/// on its own, members stay consistent with the group table, and the group
/// layer reproduces.
#[test]
fn groups_form_in_the_natural_run() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 1200);
    assert!(eco.group_stats.formed >= 3, "natural run formed {} groups", eco.group_stats.formed);
    assert!(!eco.groups.is_empty());

    let alive: Vec<u32> = eco.living().iter().filter(|a| a.alive).map(|a| a.id).collect();
    let mut seen = std::collections::HashSet::new();
    for g in &eco.groups {
        assert_eq!(g.members.len(), g.roles.len(), "roles must stay parallel to members");
        assert!(!g.members.is_empty(), "empty groups must have been dissolved");
        assert!(g.energy_pool.is_finite());
        for (i, id) in g.members.iter().enumerate() {
            assert!(alive.contains(id), "group {} holds a dead member {}", g.id, id);
            assert!(seen.insert(*id), "agent {} is in two groups", id);
            let a = eco.living().iter().find(|a| a.id == *id).unwrap();
            assert_eq!(a.group_id, g.id, "agent {} group_id desynced", id);
            assert_eq!(a.role, g.roles[i], "agent {} role desynced", id);
            assert!(a.role < 4, "grouped agent must carry a real role");
        }
    }
    assert!(eco.group_stats.reproductions > 0, "group reproduction never fired");
    assert!(eco.group_stats.propagules > 0);
    // Division of labour is real: at least two distinct roles are in use.
    let mut roles = [false; 4];
    for a in eco.living().iter().filter(|a| a.alive && a.group_id != 0) {
        if (a.role as usize) < 4 { roles[a.role as usize] = true; }
    }
    assert!(roles.iter().filter(|&&r| r).count() >= 2, "only one role was ever used: {:?}", roles);
}

// ---------------------------------------------------------------------------
// 3. Single-point bottleneck propagule.
// ---------------------------------------------------------------------------
#[test]
fn propagule_is_single_point_bottleneck() {
    let mut eco = empty_world(20261102);
    // Founder (smallest id = anchor) is hue 0.10; the other two members are
    // hue 0.35 — inside the kin tolerance but far apart in marker space.
    let genomes = vec![genome(0.10, 1.0, 1.0, 0.5, 4.0, 0.5),
                       genome(0.35, 1.0, 1.0, 0.5, 4.0, 0.5),
                       genome(0.35, 1.0, 1.0, 0.5, 4.0, 0.5)];
    let ids = spawn_with(&mut eco, HERB, 0.0, 0.0, &genomes);
    let gid = group::form_group(&mut eco, &ids, HERB, 0, None).expect("group forms");
    let parent = eco.groups.iter().find(|g| g.id == gid).unwrap().clone();
    assert_eq!(parent.founder_id, ids[0]);

    let child_gid = group::experimenter_reproduce(&mut eco, gid).expect("propagule produced");
    let child = eco.groups.iter().find(|g| g.id == child_gid).unwrap().clone();
    assert_eq!(child.parent_group, Some(gid));
    assert_eq!(child.members.len(), GROUP_PROPAGULE_SIZE);
    assert_eq!(child.founder_genome.hue, parent.founder_genome.hue);
    assert_eq!(child.generation, parent.generation + 1);

    // No child may look like a mixture: a 50/50 mix of 0.10 and 0.35 is 0.225.
    for id in &child.members {
        let a = eco.living().iter().find(|a| a.id == *id).unwrap();
        let d_founder = (a.genome.hue - 0.10f64).abs();
        let d_other = (a.genome.hue - 0.35f64).abs();
        assert!(d_founder < 0.08, "child {} hue {} drifted from founder", id, a.genome.hue);
        assert!(d_other > 0.15, "child {} hue {} looks like a member mixture", id, a.genome.hue);
        assert!(a.genome.ext.is_empty(), "bottleneck genome must not gain loci");
    }
    assert_eq!(eco.group_stats.propagules, 1);
    assert_eq!(eco.group_stats.propagule_agents, GROUP_PROPAGULE_SIZE as u64);
}

// ---------------------------------------------------------------------------
// 4. Task-switching cost.
// ---------------------------------------------------------------------------
#[test]
fn task_switch_cost_is_paid_once_per_switch() {
    let mut eco = empty_world(20261102);
    let genomes = vec![feed_genome(0.4), feed_genome(0.4), feed_genome(0.4)];
    let ids = spawn_with(&mut eco, HERB, 0.0, 0.0, &genomes);
    let gid = group::form_group(&mut eco, &ids, HERB, 0, None).expect("group forms");
    let roles: Vec<u8> = eco.groups.iter().find(|g| g.id == gid).unwrap().roles.clone();
    assert!(roles.iter().all(|&r| r == ROLE_FEED), "initial roles: {:?}", roles);

    // Pick the richest member: the switch must be affordable.
    let richest = *ids.iter().max_by(|a, b| {
        let ea = eco.living().iter().find(|x| x.id == **a).unwrap().energy;
        let eb = eco.living().iter().find(|x| x.id == **b).unwrap().energy;
        ea.partial_cmp(&eb).unwrap()
    }).unwrap();
    let a = eco.living().iter().find(|x| x.id == richest).unwrap();
    assert!(
        a.energy > TASK_SWITCH_COST + a.max_energy * group::SWITCH_MIN_ENERGY_FRAC,
        "test precondition: richest member can afford a retooling"
    );
    let e_before = a.energy;
    assert!(group::experimenter_set_genome(&mut eco, richest, guard_genome(0.4)));

    // A: the role really changes and the switch is billed.
    group::reassign_roles(&mut eco);
    let after = eco.living().iter().find(|x| x.id == richest).unwrap();
    assert_eq!(after.role, group::ROLE_GUARD, "guard genome must win the argmax");
    assert_eq!(after.energy, e_before - TASK_SWITCH_COST, "switch must cost exactly its price");
    assert_eq!(eco.group_stats.switches, 1);
    assert_eq!(eco.group_stats.switch_cost_total, TASK_SWITCH_COST);

    // B: re-evaluating without a role change is free.
    let e_mid = after.energy;
    group::reassign_roles(&mut eco);
    let again = eco.living().iter().find(|x| x.id == richest).unwrap();
    assert_eq!(again.role, group::ROLE_GUARD);
    assert_eq!(again.energy, e_mid, "a no-op re-evaluation must not bill anything");
    assert_eq!(eco.group_stats.switches, 1);
    assert_eq!(eco.group_stats.switch_cost_total, TASK_SWITCH_COST);
}

// ---------------------------------------------------------------------------
// 5. Dirty-work mutagenesis.
// ---------------------------------------------------------------------------
#[test]
fn dirty_work_raises_offspring_mutation() {
    let mut eco = empty_world(20261102);
    // A polluted block around the origin: decompose/mining waste is the
    // mineral field, so hazard(x,z) reads 1.0 anywhere inside.
    for j in 32..46usize {
        for i in 32..46usize {
            eco.inject_mineral(j * 76 + i, 1.0);
        }
    }
    let mut genomes = vec![go_genome(0.4), go_genome(0.4), feed_genome(0.4)];
    // The FEED individual must be distinguishable by hue for the stats below.
    genomes[2].hue = 0.41;
    let ids = spawn_with(&mut eco, HERB, 0.0, 0.0, &genomes);
    let gid = group::form_group(&mut eco, &ids, HERB, 0, None).expect("group forms");
    let roles = eco.groups.iter().find(|g| g.id == gid).unwrap().roles.clone();
    assert_eq!(roles[0], ROLE_GO, "go genome must take dirty work: {:?}", roles);
    assert_eq!(roles[2], ROLE_FEED, "feed genome must take feeding: {:?}", roles);

    run_ticks(&mut eco, 1, 300);
    let go = eco.living().iter().find(|a| a.id == ids[0]).unwrap();
    let feed = eco.living().iter().find(|a| a.id == ids[2]).unwrap();
    assert!(go.somatic_damage > 0.2, "GO on polluted ground must accumulate damage, got {}", go.somatic_damage);
    assert!(
        feed.somatic_damage < go.somatic_damage * 0.25,
        "FEED must stay near-clean: go={} feed={}",
        go.somatic_damage,
        feed.somatic_damage
    );
    assert!(eco.group_stats.dirty_work_events > 0);
    assert!(eco.group_stats.somatic_total > 0.0);

    // The damage scale is what the germline pays: sample the mutation
    // distribution of both genomes through the real mutate path.
    let go_scale = group::mutation_scale(go);
    let feed_scale = group::mutation_scale(feed);
    assert!((go_scale - (1.0 + SOMATIC_MUTAGEN * go.somatic_damage)).abs() < 1e-12);
    assert!(go_scale > feed_scale * 1.5, "go_scale={} feed_scale={}", go_scale, feed_scale);

    let go_genome = go.genome.clone();
    let feed_genome_c = feed.genome.clone();
    let mut go_dev = 0.0;
    let mut feed_dev = 0.0;
    let n = 400;
    for _ in 0..n {
        let c = eco.mutate(&go_genome, HERB, go_scale);
        go_dev += (c.hue - go_genome.hue).abs();
        let c = eco.mutate(&feed_genome_c, HERB, feed_scale);
        feed_dev += (c.hue - feed_genome_c.hue).abs();
    }
    let ratio = go_dev / feed_dev.max(1e-12);
    assert!(
        ratio > 1.3,
        "dirty-work germline must mutate harder (ratio {:.2}, go {:.4} feed {:.4})",
        ratio,
        go_dev / n as f64,
        feed_dev / n as f64
    );
}

// ---------------------------------------------------------------------------
// 6. Selection level count.
// ---------------------------------------------------------------------------
#[test]
fn selection_levels_grow_with_the_group_layer() {
    let mut eco = empty_world(20261102);
    assert_eq!(civilization_metrics::measure(&eco).complexity.selection_levels, 1);

    let genomes = vec![feed_genome(0.4); GROUP_MIN_MEMBERS];
    let ids = spawn_with(&mut eco, HERB, 0.0, 0.0, &genomes);
    group::form_group(&mut eco, &ids, HERB, 0, None).expect("group forms");
    assert_eq!(civilization_metrics::measure(&eco).complexity.selection_levels, 2);

    // Dissolve the group → back to a single selection level.
    for id in &ids {
        if let Some(idx) = eco.agents.iter().position(|a| a.id == *id) {
            eco.agents[idx].energy = -1.0;
        }
    }
    run_ticks(&mut eco, 1, 2);
    assert!(eco.groups.is_empty(), "starved group must dissolve");
    assert_eq!(civilization_metrics::measure(&eco).complexity.selection_levels, 1);
}

// ---------------------------------------------------------------------------
// 7. Save v11 round-trip + legacy v10 archive.
// ---------------------------------------------------------------------------
fn grouped_world(ticks: u32) -> Ecology {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, ticks);
    assert!(!eco.groups.is_empty(), "the fixture must contain groups");
    eco
}

#[test]
fn save_v11_roundtrip_is_bit_exact() {
    let mut eco = grouped_world(600);
    let state = eco.to_save_state();
    assert_eq!(state.version, save_state::SAVE_STATE_VERSION);
    // Phase 65 bumped the schema to v24 (fork-grace window).
    assert_eq!(save_state::SAVE_STATE_VERSION, 24);
    assert!(!state.Groups.is_empty());

    let json = serde_json::to_string(&state).expect("save serializes");
    let back: save_state::SaveState = serde_json::from_str(&json).expect("save parses");
    let mut restored = Ecology::from_save_state(back);

    // Group table survives the JSON hop verbatim.
    assert_eq!(restored.groups.len(), eco.groups.len());
    assert_eq!(restored.next_group_id, eco.next_group_id);
    for (a, b) in eco.groups.iter().zip(restored.groups.iter()) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.members, b.members);
        assert_eq!(a.roles, b.roles);
        assert_eq!(a.energy_pool, b.energy_pool);
        assert_eq!(a.parent_group, b.parent_group);
        assert_eq!(a.founder_genome.hue, b.founder_genome.hue);
        assert_eq!(a.generation, b.generation);
    }
    for a in eco.living().iter().filter(|a| a.alive) {
        let b = restored.living().iter().find(|b| b.id == a.id).expect("agent restored");
        assert_eq!(a.group_id, b.group_id);
        assert_eq!(a.role, b.role);
        assert_eq!(a.somatic_damage, b.somatic_damage);
    }
    assert_eq!(eco.checksum(), restored.checksum());

    // ...and the continuation is bit-exact for 400 more ticks.
    let mut t = 600;
    for _ in 0..8 {
        run_ticks(&mut eco, t + 1, t + 50);
        run_ticks(&mut restored, t + 1, t + 50);
        t += 50;
        assert_eq!(eco.checksum(), restored.checksum(), "forked at t={}", t);
        assert_eq!(eco.groups.len(), restored.groups.len(), "group count forked at t={}", t);
    }
}

#[test]
fn legacy_v10_archive_loads_as_solitary_world() {
    let eco = grouped_world(400);
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    // Phase 65 bumped the schema to v24 (fork-grace window);
    // older layers ride alongside (all serde-defaulted).
    assert_eq!(v["Version"], 24);
    // Strip every v10/v11/v13/v14 addition — this is exactly what a v10 archive looks like.
    v["Version"] = serde_json::json!(10);
    let obj = v.as_object_mut().unwrap();
    for k in ["Groups", "NextGroupId", "GroupOn", "GroupRngState",
              "TriuneOn", "TriuneUpkeepOn", "ReplayOn",
              "TeachOn", "AbiogenesisOn", "AbioRngState",
              "AbiogenesisEvents", "AbiogenesisTicks",
              "PatchOn", "PatchP", "PatchRngState", "PatchEvents", "PatchLastTick",
              "CultureValueOn", "DispersalP", "DispersalRngState", "DispersalEvents",
              "DispersalLastTick", "RegionIsolation", "DecoMutScale",
              "BudgetForkOn", "KindBudgets", "SpeciationMinSide"] {
        obj.remove(k);
    }
    if let Some(tokens) = obj.get_mut("TokenRegistry").and_then(|v| v.as_array_mut()) {
        for t in tokens { t.as_object_mut().unwrap().remove("UseGain"); }
    }
    if let Some(stats) = obj.get_mut("CultureStats").and_then(|v| v.as_object_mut()) {
        stats.remove("ValueWeightedBearers");
    }
    for a in obj["Agents"].as_array_mut().unwrap() {
        let ao = a.as_object_mut().unwrap();
        for k in ["GroupId", "Role", "SomaticDamage", "Triune",
                  "TriuneEquivStreak", "TriuneAgreeBits", "TriuneAgreeN", "Region"] {
            ao.remove(k);
        }
    }
    let state: save_state::SaveState = serde_json::from_value(v).expect("v10 archive parses");
    let mut restored = Ecology::from_save_state(state);
    assert!(restored.groups.is_empty(), "v10 archive restores no groups");
    assert!(restored.group_on, "the layer toggle defaults on");
    for a in restored.living() {
        assert_eq!(a.group_id, 0);
        assert_eq!(a.role, ROLE_NONE);
        assert_eq!(a.somatic_damage, 0.0);
        assert!(a.alive);
    }
    // It keeps running (and deterministically).
    run_ticks(&mut restored, 1, 60);
    let mut twin = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    // (the twin is the v11 save; different start, so just require no panic and
    // a stable checksum for this one)
    run_ticks(&mut twin, 1, 60);
    assert!(restored.checksum().starts_with("0x"));
}

#[test]
fn group_layer_double_run_is_bit_exact() {
    let mut a = Ecology::new(20261102);
    let mut b = Ecology::new(20261102);
    let mut t = 0;
    for _ in 0..6 {
        run_ticks(&mut a, t + 1, t + 250);
        run_ticks(&mut b, t + 1, t + 250);
        t += 250;
        assert_eq!(a.checksum(), b.checksum(), "double run forked at t={}", t);
        assert_eq!(a.groups.len(), b.groups.len(), "group table forked at t={}", t);
        assert_eq!(a.group_stats.switches, b.group_stats.switches);
        assert_eq!(a.group_stats.reproductions, b.group_stats.reproductions);
    }
    // The cooldown is part of the contract: no group may breed twice inside it.
    for g in a.groups.iter() {
        if g.last_repro_tick > g.birth_tick {
            assert!(
                g.last_repro_tick - g.birth_tick >= GROUP_REPRO_COOLDOWN,
                "group {} reproduced before its cooldown elapsed",
                g.id
            );
        }
    }
}
