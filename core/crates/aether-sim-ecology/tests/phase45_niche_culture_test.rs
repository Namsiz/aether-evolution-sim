//! Phase 45 acceptance tests — niche dynamics (patch disturbance + resource
//! heterogeneity) and the cultural bottleneck (value-tracked inheritance).
//!
//! Task acceptance rejected here (default suite): patch disturbance visibly
//! wipes grass / pulses mineral; the static regrowth field is fixed-seed and
//! bounded; value selection prefers high-value tokens deterministically; the
//! v15 save roundtrips bit-exactly and a v14 archive loads with defaults.
//! The long-horizon claims (speciation > 0 in 100k, the PATCH_P sweep) live
//! in `phase45_patch_sweep.rs` behind `#[ignore]`.

use aether_rng::Mulberry32;
use aether_sim_ecology::ecology::{
    Ecology, CELL, GRID_N, GRID_SPAN,
};
use aether_sim_ecology::culture::{
    inherit_at_birth, CultureRegistry, CultureStats, KnowledgeToken, TEACH_COST, TOKEN_DIM,
    VALUE_FIDELITY_BONUS, DEFAULT_FIDELITY,
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

fn cell_center(col: u32, row: u32) -> (f64, f64) {
    let half = GRID_SPAN / 2.0;
    (
        -half + (col as f64 + 0.5) * CELL,
        -half + (row as f64 + 0.5) * CELL,
    )
}

/// Row-major grid index of the cell whose center is nearest (x, z).
fn grid_index_of(x: f64, z: f64) -> usize {
    let half = GRID_SPAN / 2.0;
    let col = ((x + half) / CELL).floor().max(0.0).min((GRID_N - 1) as f64) as usize;
    let row = ((z + half) / CELL).floor().max(0.0).min((GRID_N - 1) as f64) as usize;
    row * GRID_N as usize + col
}

// ---------------------------------------------------------------------------
// 1. Patch disturbance: grass wipe + mineral pulse, exactly on the cells the
//    event covers (deterministic direct invocation — no RNG involved).
// ---------------------------------------------------------------------------

#[test]
fn patch_wipes_grass_and_pulses_mineral() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 0, 100);

    // Pick the grassiest land cell as the patch center so the wipe is visible.
    let grass_before = eco.grass().to_vec();
    let mineral_before = eco.mineral_field().to_vec();
    let (best_idx, best_grass) = grass_before
        .iter()
        .enumerate()
        .map(|(i, &g)| (i, g))
        .fold((0usize, 0.0f32), |acc, cur| if cur.1 > acc.1 { cur } else { acc });
    assert!(best_grass > 0.1, "world must carry grass at t=100 (got {best_grass})");
    let col = (best_idx % GRID_N as usize) as u32;
    let row = (best_idx / GRID_N as usize) as u32;
    let (cx, cz) = cell_center(col, row);

    let events_before = eco.patch_event_count();
    eco.fire_patch(col, row);
    assert_eq!(eco.patch_event_count(), events_before + 1, "event counter must bump");
    assert!((eco.patch_last_x - cx).abs() < 1e-9 && (eco.patch_last_z - cz).abs() < 1e-9,
        "patch center telemetry must match the fired cell");

    // Every cell inside the radius: grass wiped to ~0, mineral pulsed up.
    let r = aether_sim_ecology::patch::PATCH_RADIUS_CELLS * CELL;
    let pulse = aether_sim_ecology::patch::PATCH_MINERAL_PULSE as f32;
    let mut covered = 0usize;
    let mut wiped_grassy = 0usize;
    for idx in 0..(GRID_N * GRID_N) as usize {
        let c = (idx % GRID_N as usize) as u32;
        let w = (idx / GRID_N as usize) as u32;
        let (x, z) = cell_center(c, w);
        if (x - cx).hypot(z - cz) > r { continue; }
        covered += 1;
        assert_eq!(eco.grass()[idx], 0.0, "grass cell {idx} inside the patch must be wiped");
        let m = eco.mineral_field()[idx];
        let expect = (mineral_before[idx] as f64 + pulse as f64).min(1.0) as f32;
        assert!((m - expect).abs() < 1e-6, "mineral cell {idx}: {m} vs expected {expect}");
        if grass_before[idx] > 0.05 { wiped_grassy += 1; }
    }
    assert!(covered > 20, "patch must cover a meaningful local area (got {covered} cells)");
    assert!(wiped_grassy > 0, "the chosen center must cover grassy cells");

    // A cell well outside the radius is untouched.
    let far = { let mut f = (best_idx + 2000) % ((GRID_N * GRID_N) as usize); while (f as i64 - best_idx as i64).abs() < 15 { f = (f + 37) % (GRID_N * GRID_N) as usize; } f };
    assert_eq!(eco.grass()[far], grass_before[far], "outside cell grass must be untouched");
    assert_eq!(eco.mineral_field()[far], mineral_before[far], "outside cell mineral must be untouched");
}

#[test]
fn patch_tick_fires_and_changes_fields() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 0, 100);
    let mineral_before: f64 = eco.mineral_field().iter().map(|&m| m as f64).sum();
    let grass_before: f64 = eco.grass().iter().map(|&g| g as f64).sum();
    let events_before = eco.patch_event_count();

    // Force-fire: patch_p = 1 guarantees the tick-101 roll triggers.
    eco.patch_p = 1.0;
    eco.update(0.1, daylight(101));
    assert_eq!(eco.patch_event_count(), events_before + 1, "patch_p=1 must fire exactly one event");
    let mineral_after: f64 = eco.mineral_field().iter().map(|&m| m as f64).sum();
    assert!(mineral_after > mineral_before,
        "mineral sum must rise after a pulse ({mineral_before} -> {mineral_after})");
    let grass_after: f64 = eco.grass().iter().map(|&g| g as f64).sum();
    // The wipe removes grass far faster than one tick of regrowth can add it.
    assert!(grass_after < grass_before,
        "grass sum must drop after a wipe ({grass_before} -> {grass_after})");
    assert_eq!(eco.patch_last_tick, eco.tick_count(), "last-event tick telemetry");
}

#[test]
fn patch_off_is_zero_event() {
    let mut eco = Ecology::new(20261105);
    eco.patch_on = false;
    eco.patch_p = 1.0; // would fire every tick if the layer were on
    run_ticks(&mut eco, 0, 500);
    assert_eq!(eco.patch_event_count(), 0, "patch_on=false must never fire");
}

// ---------------------------------------------------------------------------
// 2. Static regrowth-noise field (resource heterogeneity, task 1.3).
// ---------------------------------------------------------------------------

#[test]
fn regrow_noise_field_is_static_bounded_and_shared() {
    let a = Ecology::new(20261102);
    let b = Ecology::new(20261105);
    assert_eq!(a.regrow_noise.len(), (GRID_N * GRID_N) as usize);
    assert_eq!(a.regrow_noise, b.regrow_noise, "fixed-seed field must be identical across worlds");
    let lo = a.regrow_noise.iter().cloned().fold(f32::INFINITY, f32::min);
    let hi = a.regrow_noise.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let amp = aether_sim_ecology::patch::REGROW_NOISE_AMP as f32;
    assert!(lo >= 1.0 - amp - 1e-6 && hi <= 1.0 + amp + 1e-6,
        "field must stay in [1-amp, 1+amp] (got [{lo}, {hi}])");
    assert!(hi - lo > 0.1, "field must be spatially non-uniform (got span {})", hi - lo);

    // Restore re-derives the identical field (it is not persisted).
    let state = a.to_save_state();
    let restored = Ecology::from_save_state(state);
    assert_eq!(restored.regrow_noise, a.regrow_noise, "restore must re-derive the same field");
}

// ---------------------------------------------------------------------------
// 3. Cultural bottleneck: value-tracked tokens + biased inheritance.
// ---------------------------------------------------------------------------

fn token_with_value(id: u32, use_gain: f64) -> KnowledgeToken {
    let mut t = KnowledgeToken::new(id, vec![0.1; TOKEN_DIM]);
    t.use_gain = use_gain;
    t
}

#[test]
fn token_value_ema_tracks_feedback_and_decays() {
    let mut reg = CultureRegistry::new();
    reg.register(token_with_value(1, 0.0));
    assert_eq!(reg.value_of(1), 0.0);
    // Positive feedback raises the EMA toward the sample.
    reg.record_use(1, 0.5);
    let v1 = reg.value_of(1);
    assert!(v1 > 0.0 && v1 < 0.5, "EMA must move toward the sample, not jump (got {v1})");
    assert_eq!(v1, reg.tokens[0].use_gain, "cache and token storage must agree");
    // Decay pulls it down again.
    for _ in 0..1000 { reg.decay_use_gains(); }
    assert!(reg.value_of(1) < v1 * 0.5, "1000 ticks of decay must fade the value");
    // Negative feedback drives the EMA negative (harmful knowledge).
    for _ in 0..3000 { reg.record_use(1, -0.5); }
    assert!(reg.value_of(1) < 0.0, "harmful outcomes must yield negative value");
}

#[test]
fn inheritance_prefers_high_value_token_under_poverty() {
    let mut reg = CultureRegistry::new();
    reg.register(token_with_value(1, 0.5));   // high value
    reg.register(token_with_value(2, -0.3));  // harmful
    reg.register(token_with_value(3, 0.0));   // neutral
    let teacher_tokens = vec![2u32, 3, 1];    // insertion order: low value first

    // Teacher can afford exactly one lesson: the value-sorted arm must pick
    // token 1 (highest value), the random-order arm must pick token 2.
    // Seed choice: both arms share the same first roll, which must pass the
    // BASE fidelity — then the only difference left is candidate order.
    let mut seed = 0u32;
    while Mulberry32::new(seed).next_f64() >= DEFAULT_FIDELITY { seed += 1; }
    let mut stats = CultureStats::default();
    let mut energy = TEACH_COST;
    let mut child = Vec::new();
    let mut rng = Mulberry32::new(seed);
    inherit_at_birth(&mut child, &teacher_tokens, &mut energy, &reg, true, &mut rng, &mut stats);
    assert_eq!(child, vec![1], "value-sorted inheritance must teach the highest-value token first");

    let mut stats2 = CultureStats::default();
    let mut energy2 = TEACH_COST;
    let mut child2 = Vec::new();
    let mut rng2 = Mulberry32::new(seed);
    inherit_at_birth(&mut child2, &teacher_tokens, &mut energy2, &reg, false, &mut rng2, &mut stats2);
    assert_eq!(child2, vec![2], "value_on=false must keep the insertion order (A/B arm)");
}

#[test]
fn value_bonus_fidelity_is_strictly_higher_for_positive_tokens() {
    // Paired-roll proof: same RNG seed → same roll. The +0.05 fidelity bonus
    // can only convert failures into successes, never the reverse.
    let mut reg = CultureRegistry::new();
    reg.register(token_with_value(1, 0.5));
    reg.register(token_with_value(2, -0.3));
    let mut high = 0usize;
    let mut low = 0usize;
    for seed in 0..1000u32 {
        let mut stats = CultureStats::default();
        let mut child = Vec::new();
        let mut e = TEACH_COST * 2.0;
        inherit_at_birth(&mut child, &[1], &mut e, &reg, true, &mut Mulberry32::new(seed), &mut stats);
        high += child.len();
        let mut child2 = Vec::new();
        let mut e2 = TEACH_COST * 2.0;
        inherit_at_birth(&mut child2, &[2], &mut e2, &reg, true, &mut Mulberry32::new(seed), &mut stats);
        low += child2.len();
    }
    assert!(high >= low, "bonus must never reduce success ({high} vs {low})");
    assert!(high > low,
        "with {}+0.05 fidelity some rolls must flip to success ({high} vs {low})",
        DEFAULT_FIDELITY);
    // Sanity: the bonus stays within [0,1].
    assert!(DEFAULT_FIDELITY + VALUE_FIDELITY_BONUS <= 1.0);
}

// ---------------------------------------------------------------------------
// 4. Save v15: full roundtrip (patch stream + token values) and v14 compat.
// ---------------------------------------------------------------------------

#[test]
fn save_v15_roundtrip_preserves_patch_and_value_state() {
    let mut eco = Ecology::new(20261102);
    eco.patch_p = 0.02; // dense enough that patch state is non-trivial by t=200
    run_ticks(&mut eco, 0, 200);
    assert!(eco.patch_event_count() > 0, "dense patch arm must have fired by t=200");

    let state = eco.to_save_state();
    assert_eq!(state.version, save_state::SAVE_STATE_VERSION);
    let json = serde_json::to_string(&state).unwrap();
    let parsed: save_state::SaveState = serde_json::from_str(&json).unwrap();
    let mut restored = Ecology::from_save_state(parsed);

    assert_eq!(restored.patch_events, eco.patch_events);
    assert_eq!(restored.patch_p, eco.patch_p);
    assert_eq!(restored.patch_on, eco.patch_on);
    assert_eq!(restored.culture_value_on, eco.culture_value_on);

    // Lockstep continuation: identical checksums + identical serialized state.
    for target in [250u32, 300u32] {
        run_ticks(&mut eco, 200, target);
        run_ticks(&mut restored, 200, target);
        assert_eq!(eco.checksum(), restored.checksum(), "checksum diverged at t={target}");
        let ja = serde_json::to_string(&eco.to_save_state()).unwrap();
        let jb = serde_json::to_string(&restored.to_save_state()).unwrap();
        assert_eq!(ja, jb, "serialized state diverged at t={target}");
    }
}

#[test]
fn v14_archive_loads_with_defaults() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 0, 100);
    let mut val = serde_json::to_value(eco.to_save_state()).unwrap();
    {
        let obj = val.as_object_mut().unwrap();
        // Downgrade: strip every v15 field, as a real v14 archive lacks them.
        obj.insert("Version".into(), serde_json::json!(14));
        for k in ["PatchOn", "PatchP", "PatchRngState", "PatchEvents", "PatchLastTick",
                  "CultureValueOn"] {
            obj.remove(k);
        }
        if let Some(stats) = obj.get_mut("CultureStats").and_then(|v| v.as_object_mut()) {
            stats.remove("ValueWeightedBearers");
        }
        if let Some(tokens) = obj.get_mut("TokenRegistry").and_then(|v| v.as_array_mut()) {
            for t in tokens {
                t.as_object_mut().unwrap().remove("UseGain");
            }
        }
    }
    let v14: save_state::SaveState = serde_json::from_value(val).expect("v14 archive must deserialize");
    let mut restored = Ecology::from_save_state(v14);
    assert!(restored.patch_on, "pre-v15 archives land on patches ON");
    assert_eq!(restored.patch_p, aether_sim_ecology::patch::PATCH_P);
    assert_eq!(restored.patch_events, 0);
    assert!(restored.culture_value_on, "pre-v15 archives land on value selection ON");
    assert!(restored.token_registry.tokens.iter().all(|t| t.use_gain == 0.0));
    // Continue running without panics; the patch stream re-derives from seed.
    run_ticks(&mut restored, 100, 160);
    assert!(restored.checksum().starts_with("0x"));
}

// ---------------------------------------------------------------------------
// 5. Determinism with the patch layer active.
// ---------------------------------------------------------------------------

#[test]
fn patch_world_double_run_bit_exact() {
    let mut a = Ecology::new(20261102);
    let mut b = Ecology::new(20261102);
    a.patch_p = 0.01;
    b.patch_p = 0.01;
    for t in 1..=600u32 {
        a.update(0.1, daylight(t));
        b.update(0.1, daylight(t));
    }
    assert!(a.patch_event_count() > 0, "dense arm must have fired");
    assert_eq!(a.checksum(), b.checksum(), "patch-layer double run diverged");
    assert_eq!(a.patch_events, b.patch_events);
    assert_eq!(a.grass(), b.grass());
    assert_eq!(a.mineral_field(), b.mineral_field());
}
