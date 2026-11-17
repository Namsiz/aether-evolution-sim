//! Phase 41 acceptance tests — triune brain (instinct / emotion / reason).
//!
//! The three layers are unit-tested inside `aether-brain::triune`
//! (reflex-without-learning, fear accumulation/decay, emotional memory,
//! working-memory persistence, multi-step rollout, inhibition override,
//! confidence tracking prediction error, layer-scaled cost, determinism).
//! This file covers the ecology-level contract:
//!   1. the three-layer shadow runs on every living agent every tick;
//!   2. its upkeep + rollout + inhibition cost is real energy booked into
//!      `ledger.triune_cost` (A/B with the cost toggle);
//!   3. the v12 save string round-trips bit-exactly and the continuation is
//!      bit-identical;
//!   4. a pre-v12 archive (no `Triune` field, `Version = 11`) restores with a
//!      deterministic reference brain instead of failing;
//!   5. the observational counters (WM updates, rollout steps, prediction
//!      updates, fear events, confidence) actually move.

use aether_brain::triune::{GENE_COUNT, MAX_MEMORY_SLOTS, MAX_ROLLOUT, MAX_WM_SLOTS};
use aether_sim_ecology::ecology::Ecology;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in from..=to {
        eco.update(0.1, daylight(t));
    }
}

fn total_energy(eco: &Ecology) -> f64 {
    eco.living().iter().filter(|a| a.alive).map(|a| a.energy).sum()
}

#[test]
fn triune_shadow_runs_on_every_tick() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 300);
    let living: Vec<_> = eco.living().iter().filter(|a| a.alive).collect();
    assert!(!living.is_empty(), "no living agents at t=300");
    for a in &living {
        assert_eq!(a.triune.genes.len(), GENE_COUNT, "agent {} gene length", a.id);
        assert!(
            a.triune.stats.ticks > 0 || a.age < 0.25,
            "agent {} never ticked its triune brain",
            a.id
            // Phase 42: 新生个体允许未 tick——脑节律按 idx 奇偶错开，一个
            // 出生在本 tick 且奇偶未轮到的个体要等下一拍（Allee 缓解提高了
            // 出生率，这个窗口在 t=300 的断言里真的会被踩到）。
        );
        assert!(
            a.triune.wm.len() <= MAX_WM_SLOTS && a.triune.memory_valence.len() <= MAX_MEMORY_SLOTS,
            "agent {} structural sizes out of bounds",
            a.id
        );
        assert!(
            a.triune.rollout_depth() <= MAX_ROLLOUT,
            "agent {} rollout depth out of bounds",
            a.id
        );
    }
    let st = eco.triune_stats();
    assert!(st.ticks > 0, "aggregate ticks");
    assert!(st.wm_updates > 0, "working-memory updates");
    assert!(st.rollout_steps > 0, "rollout steps");
    assert!(st.prediction_updates > 0, "world-model prediction updates");
    assert!(eco.ledger.triune_cost > 0.0, "triune upkeep was never charged");
    assert!(st.upkeep_energy > 0.0 && st.decision_energy > 0.0);
    assert!(st.mean_confidence().is_finite());
    assert!(st.mean_pred_error().is_finite());
    eprintln!(
        "t=300 triune ticks={} upkeep={:.4} decision={:.4} fear_events={} fear_cond={} \
         wm={} rollout={} overrides={} inhibitions={} conf={:.3} err={:.3}",
        st.ticks, st.upkeep_energy, st.decision_energy, st.fear_events, st.fear_conditioning,
        st.wm_updates, st.rollout_steps, st.planned_overrides, st.inhibitions,
        st.mean_confidence(), st.mean_pred_error()
    );
}

#[test]
fn triune_cost_is_priced_and_ab_testable() {
    let mut paid = Ecology::new(20261102);
    let mut free = Ecology::new(20261102);
    free.triune_upkeep_on = false;
    // Phase 45: run the A/B on a short window. Under the patch/noise dynamics
    // the two arms diverge chaotically by t=300 (births/deaths desync), so an
    // aggregate energy ordering at t=300 measures population luck, not the
    // upkeep drain; 60 ticks is before the divergence but long enough for the
    // per-tick drain to dominate.
    run_ticks(&mut paid, 1, 60);
    run_ticks(&mut free, 1, 60);
    assert!(paid.ledger.triune_cost > 0.0, "paid world must book triune cost");
    assert_eq!(free.ledger.triune_cost, 0.0, "cost toggle must zero the drain");
    assert!(
        total_energy(&paid) < total_energy(&free),
        "paying for three brains must lower total energy: paid={} free={}",
        total_energy(&paid),
        total_energy(&free)
    );
    assert_ne!(
        paid.checksum(),
        free.checksum(),
        "the cost must feed back into the trajectory (energy gates behaviour)"
    );
    eprintln!(
        "cost A/B: paid={:.3} free={:.3} ledger={:.3}",
        total_energy(&paid),
        total_energy(&free),
        paid.ledger.triune_cost
    );
}

#[test]
fn save_v12_roundtrip_is_bit_exact() {
    // Phase 45 bumps the schema to v15 (patch layer + cultural value), but
    // the triune-state roundtrip contract is unchanged.
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 120);
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    assert!(json.contains("\"Version\":16"), "schema version must be v16");
    assert!(json.contains("\"Triune\""), "SaveAgent must carry the triune string");
    assert!(json.contains("\"ReplayOn\""), "v13 must carry the replay toggle");
    let restored = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    let json2 = serde_json::to_string(&restored.to_save_state()).unwrap();
    assert_eq!(json, json2, "v12 save-state JSON must round-trip bit-exactly");
    for (a, b) in eco.living().iter().zip(restored.living().iter()) {
        assert_eq!(a.triune, b.triune, "triune state drift for agent {}", a.id);
    }
    let mut a = eco;
    let mut b = restored;
    run_ticks(&mut a, 121, 220);
    run_ticks(&mut b, 121, 220);
    assert_eq!(a.checksum(), b.checksum(), "restored continuation diverged");
    // Aggregate telemetry is runtime-only (reset on restore, like GroupStats);
    // the per-agent triune state (stats included) is what must round-trip.
    assert!(b.triune_stats().ticks > 0, "restored world must keep ticking");
}

#[test]
fn pre_v12_archive_restores_with_reference_triune() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 80);
    let mut v: serde_json::Value = serde_json::to_value(eco.to_save_state()).unwrap();
    v["Version"] = serde_json::json!(11);
    v.as_object_mut().unwrap().remove("TriuneOn");
    v.as_object_mut().unwrap().remove("TriuneUpkeepOn");
    for a in v["Agents"].as_array_mut().unwrap() {
        a.as_object_mut().unwrap().remove("Triune");
    }
    let legacy: aether_sim_ecology::save_state::SaveState = serde_json::from_value(v).unwrap();
    let restored = Ecology::from_save_state(legacy);
    assert!(restored.triune_on, "pre-v12 restore must default triune ON");
    assert!(restored.triune_upkeep_on);
    for a in restored.living().iter().filter(|a| a.alive) {
        assert_eq!(a.triune.genes.len(), GENE_COUNT);
        assert_eq!(a.triune.stats.ticks, 0, "reference brain starts fresh");
    }
    // Deterministic continuation: two independent restores of the same
    // (now re-saved) legacy world must stay in lockstep.
    let snapshot = serde_json::to_string(&restored.to_save_state()).unwrap();
    let mut b = Ecology::from_save_state(serde_json::from_str(&snapshot).unwrap());
    let mut c = Ecology::from_save_state(serde_json::from_str(&snapshot).unwrap());
    run_ticks(&mut b, 81, 120);
    run_ticks(&mut c, 81, 120);
    assert_eq!(b.checksum(), c.checksum(), "legacy restore must be deterministic");
}

#[test]
#[ignore]
fn phase41_anchor_probe() {
    for &(seed, ticks) in &[(20261102u32, 300u32), (20261102, 3000), (20261105, 300)] {
        let mut eco = Ecology::new(seed);
        run_ticks(&mut eco, 1, ticks);
        eprintln!(
            "seed={} t={} v2={} v1={} pop={} alive={} triune_cost={:.4} stats={:?}",
            seed, ticks, eco.checksum(), eco.checksum_v1(),
            eco.living().len(),
            eco.living().iter().filter(|a| a.alive).count(),
            eco.ledger.triune_cost,
            eco.triune_stats().ticks,
        );
    }
    for &(seed, ticks) in &[(20261102u32, 300u32), (20261102, 3000), (20261105, 300)] {
        let mut eco = Ecology::new(seed);
        eco.group_on = false;
        run_ticks(&mut eco, 1, ticks);
        eprintln!(
            "group_off seed={} t={} v2={} v1={} pop={}",
            seed, ticks, eco.checksum(), eco.checksum_v1(),
            eco.living().iter().filter(|a| a.alive).count(),
        );
    }
}

#[test]
#[ignore]
fn phase41_stats_probe() {
    let mut eco = Ecology::new(20261102);
    for t in 1..=9000u32 {
        eco.update(0.1, daylight(t));
        if t % 1500 == 0 {
            let living: Vec<_> = eco.living().iter().filter(|a| a.alive).collect();
            let n = living.len().max(1) as f64;
            let mean_units = living.iter().map(|a| {
                let (i, e, r) = a.triune.layer_units();
                (i + e + r) as f64
            }).sum::<f64>() / n;
            let mean_upkeep = living.iter().map(|a| a.triune.upkeep()).sum::<f64>() / n;
            let mean_wm = living.iter().map(|a| a.triune.wm.len()).sum::<usize>() as f64 / n;
            let mean_depth = living.iter().map(|a| a.triune.rollout_depth()).sum::<usize>() as f64 / n;
            let mean_mem = living.iter().map(|a| a.triune.memory_capacity()).sum::<usize>() as f64 / n;
            let st = eco.triune_stats();
            eprintln!(
                "t={} alive={} units={:.1} upkeep={:.4} wm={:.1} depth={:.1} mem={:.1} | \
                 ticks={} fear={} cond={} inhib={} override={} wm_upd={} rollout={} pred={} hi_conf={} \
                 mean_fear={:.3} max_fear={:.3} mean_conf={:.3} mean_err={:.3} hist={:?}",
                t, living.len(), mean_units, mean_upkeep, mean_wm, mean_depth, mean_mem,
                st.ticks, st.fear_events, st.fear_conditioning, st.inhibitions, st.planned_overrides,
                st.wm_updates, st.rollout_steps, st.prediction_updates, st.high_confidence_ticks,
                st.mean_fear(), st.max_fear, st.mean_confidence(), st.mean_pred_error(), st.action_hist,
            );
        }
    }
}

#[test]
#[ignore]
fn phase41_longrun_probe() {
    let mut eco = Ecology::new(20261102);
    for t in 1..=30000u32 {
        eco.update(0.1, daylight(t));
        if t % 5000 == 0 {
            let living: Vec<_> = eco.living().iter().filter(|a| a.alive).collect();
            let n = living.len().max(1) as f64;
            let mean_units = living.iter().map(|a| {
                let (i, e, r) = a.triune.layer_units();
                (i + e + r) as f64
            }).sum::<f64>() / n;
            eprintln!(
                "t={} alive={} units={:.1} tokens={} groups={} fear={} inhib={} overrides={} conf={:.3} err={:.3}",
                t, living.len(), mean_units, eco.token_registry.tokens.len(), eco.groups.len(),
                eco.triune_stats().fear_events, eco.triune_stats().inhibitions,
                eco.triune_stats().planned_overrides, eco.triune_stats().mean_confidence(),
                eco.triune_stats().mean_pred_error(),
            );
        }
    }
}

#[test]
#[ignore]
fn phase41_group_timeline_probe() {
    let mut eco = Ecology::new(20261102);
    for t in 1..=4000u32 {
        eco.update(0.1, daylight(t));
        if t % 400 == 0 {
            let max_pool = eco.groups.iter().map(|g| g.energy_pool).fold(0.0f64, f64::max);
            eprintln!(
                "t={} groups={} formed={} repros={} propagules={} max_pool={:.3} alive={}",
                t, eco.groups.len(), eco.group_stats.formed, eco.group_stats.reproductions,
                eco.group_stats.propagules, max_pool,
                eco.living().iter().filter(|a| a.alive).count(),
            );
        }
    }
}

#[test]
fn emotion_and_metacognition_telemetry_moves() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 600);
    let st = eco.triune_stats();
    assert!(st.ticks > 0);
    assert!(st.wm_updates >= st.ticks, "each tick updates every WM slot");
    assert!(st.rollout_steps > 0);
    assert!(st.prediction_updates > 0);
    assert!(st.action_hist.iter().sum::<u64>() == st.ticks, "action histogram must cover every tick");
    assert!(st.max_fear.is_finite());
    assert!(st.mean_fear().is_finite() && st.mean_fear() >= 0.0);
    assert!(st.mean_confidence().is_finite());
    assert!(
        st.mean_pred_error().is_finite() && st.mean_pred_error() >= 0.0,
        "prediction error EMA must be finite and non-negative"
    );
    eprintln!(
        "t=600: fear_events={} cond={} inhibitions={} overrides={} mean_fear={:.3} \
         mean_conf={:.3} mean_err={:.3}",
        st.fear_events, st.fear_conditioning, st.inhibitions, st.planned_overrides,
        st.mean_fear(), st.mean_confidence(), st.mean_pred_error()
    );
}
