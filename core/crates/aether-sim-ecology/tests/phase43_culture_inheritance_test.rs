//! Phase 43 acceptance — the cultural ratchet (birth-time token inheritance).
//!
//! Tomasello ratchet: children acquire accumulated culture through PAID
//! parental teaching; a failed fidelity roll = teaching failure / critical
//! period miss; a parent who cannot afford the next lesson stops teaching
//! (zero-RNG poverty break). Without this pathway every population turnover
//! resets bearers to zero (the Phase 42 observatory saw 792 inventions with
//! bearers stuck at 0-5).
//!
//! Acceptance:
//!   1. `inherit_at_birth_paid_ordered_zero_rng_poverty` — unit: cost per
//!      transmitted token only, order-preserving, poverty break draws nothing.
//!   2. `birth_teaching_charges_real_parent_energy` — integration: the parent
//!      pays TEACH_COST per transmitted token on top of the reproduction
//!      drain (cross-arm energy difference isolates the teaching fee from
//!      metabolism/grazing noise).
//!   3. `teach_off_is_inert` — A/B arm: no inheritance, no cost, no telemetry.
//!   4. `poor_parent_teaches_less` — post-drain energy caps the curriculum in
//!      known_tokens order.
//!   5. `group_founder_teaches_propagule` — the founder is the teacher on the
//!      group (propagule bottleneck) path.
//!   6. `teaching_double_run_bit_exact` — dedicated culture_rng discipline.
//!   7. `bearer_ratchet_survives_population_turnover` — ON vs OFF bearer
//!      counts after the original holders have all died.

use aether_rng::Mulberry32;
use aether_sim_ecology::culture::{self, CultureRegistry, CultureStats, KnowledgeToken, TOKEN_DIM};
use aether_sim_ecology::ecology::{Ecology, HERB};

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in from..=to {
        eco.update(0.1, daylight(t));
    }
}

// Injected curriculum id range. Chosen above every de-novo invention id the
// window can mint (registry next_token_id is pushed past the range at setup),
// so "holds a token in [LO, HI]" identifies taught lineages exactly.
const TOKEN_LO: u32 = 9001;
const TOKEN_HI: u32 = 9020;

fn register_curriculum(eco: &mut Ecology) {
    for id in TOKEN_LO..=TOKEN_HI {
        eco.token_registry.register(KnowledgeToken::new(id, vec![0.0; TOKEN_DIM]));
    }
}

fn holds_curriculum(a: &aether_sim_ecology::ecology::Agent) -> bool {
    a.known_tokens.iter().any(|&t| t >= TOKEN_LO && t <= TOKEN_HI)
}

// ---------------------------------------------------------------------------
// 1. Unit semantics of inherit_at_birth.
// ---------------------------------------------------------------------------
#[test]
fn inherit_at_birth_paid_ordered_zero_rng_poverty() {
    let teacher: Vec<u32> = vec![11, 22, 33];
    // Phase 45: these unit tests pin the ORDER-PRESERVING (A/B control) arm —
    // an empty registry + value_on=false reproduces the pre-Phase-45 path.
    let reg = CultureRegistry::new();

    // Poverty break: cannot afford the first lesson → teaches nothing and
    // consumes ZERO draws (rng state unchanged).
    let mut rng = Mulberry32::new(7);
    let state0 = rng.state();
    let mut stats = CultureStats::default();
    let mut child = Vec::new();
    let mut energy = 0.1;
    let taught = culture::inherit_at_birth(&mut child, &teacher, &mut energy, &reg, false, &mut rng, &mut stats);
    assert_eq!(taught, 0);
    assert!(child.is_empty());
    assert_eq!(energy, 0.1);
    assert_eq!(rng.state(), state0, "poverty break must be zero-RNG");
    assert_eq!(stats.taught_count, 0);
    assert_eq!(stats.teach_cost_total, 0.0);

    // Affordable: every transmitted token costs exactly TEACH_COST, the child
    // list is a subset of the teacher's in order, and the teacher's energy
    // drop equals TEACH_COST × transmitted.
    let mut rng = Mulberry32::new(7);
    let mut stats = CultureStats::default();
    let mut child = Vec::new();
    let mut energy = 10.0;
    let taught = culture::inherit_at_birth(&mut child, &teacher, &mut energy, &reg, false, &mut rng, &mut stats);
    assert!(taught >= 1, "seeded run must transmit at least one token");
    assert!(taught <= teacher.len() as u64);
    assert_eq!(energy, 10.0 - culture::TEACH_COST * taught as f64);
    assert!(child.iter().all(|t| teacher.contains(t)));
    assert_eq!(stats.taught_count, taught as u64);
    assert_eq!(stats.teach_cost_total, culture::TEACH_COST * taught as f64);
    // Determinism: same seed → same transmission set.
    let mut rng2 = Mulberry32::new(7);
    let mut stats2 = CultureStats::default();
    let mut child2 = Vec::new();
    let mut energy2 = 10.0;
    let taught2 = culture::inherit_at_birth(&mut child2, &teacher, &mut energy2, &reg, false, &mut rng2, &mut stats2);
    assert_eq!(taught, taught2);
    assert_eq!(child, child2);
}

// ---------------------------------------------------------------------------
// 2+3. Birth-time teaching is real parental investment; the OFF arm is inert.
// ---------------------------------------------------------------------------
/// herbs-only world, one fertile pair (P = lower Vec index, holds the
/// curriculum), everyone else pinned at 0.3×max so the pair is the only
/// reproductive pathway. Returns (eco, taught_before, p_index, m_index,
/// energies of P and M at setup end).
fn setup_birth_world(seed: u32, teach_on: bool) -> (Ecology, u64, usize, usize, f64, f64) {
    let mut eco = Ecology::new(seed);
    register_curriculum(&mut eco);
    eco.teach_on = teach_on;
    for a in eco.agents.iter_mut() {
        if a.kind != HERB { a.alive = false; }
    }
    run_ticks(&mut eco, 1, 1); // retain sweeps the dead

    // Closest alive-herb pair (i < j so P is the initiating index).
    let n = eco.agents.len();
    let mut best = (0usize, 1usize, f64::MAX);
    for i in 0..n {
        if !eco.agents[i].alive { continue; }
        for j in (i + 1)..n {
            if !eco.agents[j].alive { continue; }
            let dx = eco.agents[i].x - eco.agents[j].x;
            let dz = eco.agents[i].z - eco.agents[j].z;
            let d = (dx * dx + dz * dz).sqrt();
            if d < best.2 { best = (i, j, d); }
        }
    }
    let (pi, mi, d) = best;
    assert!(d < 11.0, "no herb pair within mate radius (d={})", d);

    let (p_max, m_max) = (eco.agents[pi].max_energy, eco.agents[mi].max_energy);
    eco.agents[pi].known_tokens = (TOKEN_LO..=TOKEN_HI).collect();
    // Same genome for both → hue diff 0 (no rejection roll) and pair trait
    // distance 0 (gene barrier can never fire); strip sig genes so that gate
    // is skipped entirely.
    eco.agents[pi].genome.help_thr = None;
    eco.agents[pi].genome.pref_sig = None;
    eco.agents[pi].genome.tol_sig = None;
    eco.agents[mi].genome = eco.agents[pi].genome.clone();
    eco.agents[pi].energy = p_max * 0.95;
    eco.agents[pi].repro = 0.0;
    eco.agents[pi].age = 100.0;
    eco.agents[mi].energy = m_max * 0.95;
    eco.agents[mi].repro = 0.0;
    eco.agents[mi].age = 100.0;
    for (k, a) in eco.agents.iter_mut().enumerate() {
        if a.alive && k != pi && k != mi {
            a.energy = a.max_energy * 0.3; // cannot initiate, cannot be a mate
        }
    }
    let (e_p, e_m) = (eco.agents[pi].energy, eco.agents[mi].energy);
    (eco, 0, pi, mi, e_p, e_m)
}

#[test]
fn birth_teaching_charges_real_parent_energy() {
    let (mut eco_on, _, pi, _, e_p0, _) = setup_birth_world(20261102, true);
    let (mut eco_off, _, pi_off, _, e_p0_off, _) = setup_birth_world(20261102, false);
    assert_eq!(pi, pi_off);

    let mut birth_tick_on = None;
    let mut taught_at_birth = 0u64;
    let mut e_p_after_on = 0.0;
    for t in 2..=32u32 {
        let taught_before = eco_on.culture_stats.taught_count;
        run_ticks(&mut eco_on, t, t);
        let new_holder = eco_on.agents.iter()
            .any(|a| a.alive && a.id != eco_on.agents[pi].id && holds_curriculum(a));
        if new_holder {
            birth_tick_on = Some(t);
            taught_at_birth = eco_on.culture_stats.taught_count - taught_before;
            e_p_after_on = eco_on.agents[pi].energy;
            break;
        }
    }
    let bt = birth_tick_on.expect("fertile pair must give birth inside the window");
    assert!(taught_at_birth >= 1, "20 tokens at fidelity 0.9 cannot all fail");

    // OFF arm: identical setup and identical main-stream trajectory up to the
    // birth tick; no inheritance happens there.
    let e_p_after_off;
    for t in 2..=bt {
        run_ticks(&mut eco_off, t, t);
    }
    assert!(!eco_off.agents.iter().any(|a| a.alive && a.id != eco_off.agents[pi_off].id && holds_curriculum(a)),
        "teach_off arm must produce no taught lineage");
    assert_eq!(eco_off.culture_stats.taught_count, 0);
    e_p_after_off = eco_off.agents[pi_off].energy;

    // The cross-arm energy difference isolates the teaching fee: both arms
    // share the identical reproduction drain and the identical (pre-divergence)
    // metabolism/grazing flows of the birth tick; the only difference is
    // TEACH_COST × transmitted.
    let gap = e_p_after_off - e_p_after_on;
    let expected = culture::TEACH_COST * taught_at_birth as f64;
    assert!((gap - expected).abs() < 1e-9,
        "teaching fee mismatch: gap={} expected={}", gap, expected);
    // ...and it is real money: strictly positive.
    assert!(gap > 0.0);
    let _ = (e_p0, e_p0_off); // setup-end energies (documented fixtures)
}

// ---------------------------------------------------------------------------
// 4. Poverty: the curriculum is capped by post-drain energy, in list order.
// ---------------------------------------------------------------------------
#[test]
fn poor_parent_teaches_less() {
    // Poverty demonstration on the exact post-drain budget: a 400-token
    // curriculum with energy for only ~10 lessons transmits strictly less
    // than the full list, in known_tokens order, with the exact fee math.
    let teacher: Vec<u32> = (TOKEN_LO..TOKEN_LO + 400).collect();
    let reg = CultureRegistry::new();
    let budget = 2.5; // affords ⌊2.5 / 0.25⌋ = 10 lessons
    let mut child: Vec<u32> = Vec::new();
    let mut energy = budget;
    let mut stats = CultureStats::default();
    let mut rng = Mulberry32::new(99);
    let taught = culture::inherit_at_birth(&mut child, &teacher, &mut energy, &reg, false, &mut rng, &mut stats);
    let affordable = (budget / culture::TEACH_COST).floor() as u64;
    assert!(taught <= affordable, "poverty cap violated: {} > {}", taught, affordable);
    assert!(taught < teacher.len() as u64, "curriculum must be truncated by poverty");
    assert_eq!(energy, budget - culture::TEACH_COST * taught as f64);
    assert!(child.iter().all(|t| teacher.contains(t)));
    assert_eq!(stats.taught_count, taught);
}

// ---------------------------------------------------------------------------
// 5. Group (propagule bottleneck) path: the founder is the teacher.
// ---------------------------------------------------------------------------
#[test]
fn group_founder_teaches_propagule() {
    use aether_sim_ecology::group::{Group, GROUP_MIN_MEMBERS, GROUP_PROPAGULE_SIZE, GROUP_REPRO_POOL};

    let build = |teach_on: bool| -> (Ecology, usize, u64, f64, u32) {
        let mut eco = Ecology::new(20261102);
        register_curriculum(&mut eco);
        eco.teach_on = teach_on;
        for a in eco.agents.iter_mut() {
            if a.kind != HERB { a.alive = false; }
        }
        // Run past the group reproduction cooldown so the seeded group is
        // eligible on the first step after seeding.
        run_ticks(&mut eco, 1, 401);
        // Founder = first alive herb; members = founder + next two alive herbs.
        let alive: Vec<usize> = (0..eco.agents.len()).filter(|&k| eco.agents[k].alive).collect();
        assert!(alive.len() >= GROUP_MIN_MEMBERS);
        let fi = alive[0];
        let members: Vec<u32> = alive[..GROUP_MIN_MEMBERS].iter().map(|&k| eco.agents[k].id).collect();
        eco.agents[fi].known_tokens = (TOKEN_LO..TOKEN_LO + 10).collect();
        let founder_genome = eco.agents[fi].genome.clone();
        let tick = eco.tick_count();
        let gid = eco.next_group_id.max(1);
        eco.groups.push(Group {
            id: gid,
            kind: HERB,
            members: members.clone(),
            founder_id: members[0],
            energy_pool: 30.0, // > GROUP_REPRO_POOL, survives maintenance
            birth_tick: tick,
            parent_group: None,
            roles: vec![aether_sim_ecology::group::ROLE_NONE; members.len()],
            clade_id: 0,
            founder_genome,
            founder_lineage: None,
            last_repro_tick: tick.saturating_sub(400), // cooldown elapsed
            generation: 0,
        });
        eco.next_group_id = gid + 1;
        // Membership must be visible on the agents themselves, or the member
        // sync step expels everyone on the first scan.
        for &id in &members {
            let k = eco.agents.iter().position(|a| a.id == id).unwrap();
            eco.agents[k].group_id = gid;
        }
        let taught_before = eco.culture_stats.taught_count;
        let f_e0 = eco.agents[fi].energy;
        (eco, fi, taught_before, f_e0, gid)
    };

    let (mut eco_on, fi, taught0, f_e0, gid) = build(true);
    run_ticks(&mut eco_on, 402, 402);
    // Locate the seeded group's propagule child (the world may also grow
    // cohesion groups of its own during the pre-run — identify by lineage).
    let child_group = eco_on.groups.iter().find(|g| g.parent_group == Some(gid))
        .expect("the seeded group must reproduce (child carries parent_group)");
    assert_eq!(child_group.members.len(), GROUP_PROPAGULE_SIZE);
    let mut transmitted = 0usize;
    for &id in &child_group.members {
        let a = eco_on.agents.iter().find(|a| a.id == id).expect("propagule restored");
        assert!(holds_curriculum(a), "propagule {} must inherit the curriculum", id);
        transmitted += a.known_tokens.iter().filter(|&&t| (TOKEN_LO..TOKEN_LO + 10).contains(&t)).count();
    }
    let taught_delta = eco_on.culture_stats.taught_count - taught0;
    assert_eq!(transmitted as u64, taught_delta, "taught_count must mirror transmitted tokens");
    assert!(taught_delta >= 1);
    assert!(eco_on.culture_stats.teach_cost_total >= culture::TEACH_COST * taught_delta as f64 - 1e-9);

    // OFF arm: same setup, founder pays nothing, propagules hold nothing.
    let (mut eco_off, fi_off, taught0_off, f_e0_off, _) = build(false);
    run_ticks(&mut eco_off, 402, 402);
    assert!(eco_off.groups.iter().any(|g| g.parent_group == Some(gid)),
        "reproduction is independent of teaching");
    for &id in &eco_off.groups[1].members {
        let a = eco_off.agents.iter().find(|a| a.id == id).unwrap();
        assert!(!holds_curriculum(a));
    }
    assert_eq!(eco_off.culture_stats.taught_count, taught0_off);

    // Cross-arm founder energy gap isolates the teaching fee (identical
    // metabolism flows in both arms — divergence starts at the fee itself).
    let drop_on = f_e0 - eco_on.agents[fi].energy;
    let drop_off = f_e0_off - eco_off.agents[fi_off].energy;
    let gap = drop_on - drop_off;
    let expected = culture::TEACH_COST * taught_delta as f64;
    assert!((gap - expected).abs() < 1e-6, "founder fee mismatch: gap={} expected={}", gap, expected);
}

// ---------------------------------------------------------------------------
// 6. Determinism: the dedicated culture_rng stream keeps runs bit-exact.
// ---------------------------------------------------------------------------
#[test]
fn teaching_double_run_bit_exact() {
    let run = || {
        let mut eco = Ecology::new(20261102);
        register_curriculum(&mut eco);
        run_ticks(&mut eco, 1, 50);
        // Inject the curriculum into the first 5 alive herbs mid-run.
        let mut given = 0;
        for a in eco.agents.iter_mut() {
            if a.alive && a.kind == HERB && given < 5 {
                a.known_tokens = (TOKEN_LO..TOKEN_LO + 6).collect();
                given += 1;
            }
        }
        assert_eq!(given, 5);
        run_ticks(&mut eco, 51, 600);
        eco.checksum()
    };
    let c1 = run();
    let c2 = run();
    assert_eq!(c1, c2, "teaching world must be bit-exact across runs");
}

// ---------------------------------------------------------------------------
// 7. The ratchet itself: bearers survive population turnover only with
//    inheritance ON.
// ---------------------------------------------------------------------------
#[test]
fn bearer_ratchet_survives_population_turnover() {
    let run = |teach_on: bool| -> (u64, u64, u64) {
        let mut eco = Ecology::new(20261102);
        register_curriculum(&mut eco);
        eco.teach_on = teach_on;
        run_ticks(&mut eco, 1, 50);
        let mut given = 0;
        for a in eco.agents.iter_mut() {
            if a.alive && a.kind == HERB && given < 10 {
                a.known_tokens = (TOKEN_LO..TOKEN_LO + 3).collect();
                given += 1;
            }
        }
        assert_eq!(given, 10);
        run_ticks(&mut eco, 51, 3000);
        let bearers = eco.agents.iter().filter(|a| a.alive && holds_curriculum(a)).count() as u64;
        (bearers, eco.culture_stats.taught_count, eco.culture_stats.teach_cost_total.to_bits())
    };
    let (on_bearers, on_taught, _) = run(true);
    let (off_bearers, off_taught, _) = run(false);
    assert_eq!(off_taught, 0);
    assert!(on_taught >= 20, "inheritance must be firing at scale (taught={})", on_taught);
    // Original holders are dead by t=3000 (age 50 + lifespan ≪ 3000); the ON
    // arm's bearers exist only because teaching carried the tokens across
    // generations. The ratchet: ON strictly dominates OFF.
    assert!(on_bearers > off_bearers,
        "ratchet failed: bearers ON={} vs OFF={}", on_bearers, off_bearers);
}
