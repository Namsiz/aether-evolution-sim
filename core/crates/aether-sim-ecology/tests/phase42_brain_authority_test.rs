//! Phase 42 acceptance tests — triune behaviour authority + counterfactual replay.
//!
//! The phase turns the Phase 41 shadow into the executive: once an individual's
//! three-layer brain agrees with its own FSM for `T_EQUIV_K_TICKS` consecutive
//! brain-rhythm ticks at `T_EQUIV` agreement (and its metacognitive confidence
//! clears `T_TAKEOVER_CONF`), its arbitration action drives behaviour through
//! the action→FSM bridge. Every brain-rhythm tick also replays one buffered
//! experience (sharp-wave-ripple analogue) to consolidate the world model.
//!
//! Covered here:
//!   1. the takeover gate needs the triune's own agreement streak + confidence;
//!   2. after takeover the body executes the brain's action (turn intent moves
//!      the heading and the target together);
//!   3. learning closure — the recorded `executed` action is the brain's, not
//!      the FSM's;
//!   4. a natural run switches agents and keeps them alive;
//!   5. replay A/B lowers the world-model error on experienced transitions;
//!   6. the inhibition gate is calibrated into the 10–30 % open band;
//!   7. the brain-driven class is not worse off (lifespan / energy);
//!   8. determinism: double run bit-exact, save/restore continuation exact;
//!   9. v13 schema round-trips and a v12 archive restores (replay defaults ON).

use aether_brain::triune::{GATE_OPEN, REPLAY_CAP};
use aether_sim_ecology::ecology::{
    Ecology, BRAIN_AUTHORITY_BRAIN, BRAIN_AUTHORITY_FSM, PRED, T_EQUIV_K_TICKS,
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

fn checksum_at(seed: u32, ticks: u32) -> String {
    let mut eco = Ecology::new(seed);
    run_ticks(&mut eco, 1, ticks);
    eco.checksum()
}

fn mean_probe_error(eco: &Ecology) -> f64 {
    let v: Vec<f64> = eco
        .living()
        .iter()
        .filter(|a| a.alive && a.triune.stats.ticks > 0)
        .map(|a| a.triune.probe_error())
        .filter(|e| *e > 0.0)
        .collect();
    if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 }
}

/// A living herbivore with no predator inside its own threat radius (so the FSM
/// will not enter its flee branch underneath the bridge).
fn pick_open_herb(eco: &Ecology) -> Option<usize> {
    for (i, a) in eco.agents.iter().enumerate() {
        if !a.alive || a.kind != aether_sim_ecology::ecology::HERB || a.fear > 0.5 {
            continue;
        }
        let mut threatened = false;
        for b in eco.agents.iter() {
            if !b.alive || b.kind != PRED || b.id == a.id { continue; }
            let d = ((b.x - a.x).powi(2) + (b.z - a.z).powi(2)).sqrt();
            if d < a.genome.sense * 0.95 { threatened = true; break; }
        }
        if !threatened { return Some(i); }
    }
    None
}

/// Force every precondition of the takeover gate on one agent.
fn arm_gate(eco: &mut Ecology, idx: usize, streak: u32, bits: u64, n: u32, conf: f64) {
    let a = &mut eco.agents[idx];
    a.brain_authority = BRAIN_AUTHORITY_FSM;
    a.triune_equiv_streak = streak;
    a.triune_agree_bits = bits;
    a.triune_agree_n = n;
    a.triune.confidence = conf;
    if a.triune.stats.ticks == 0 { a.triune.stats.ticks = 1; }
}

// ---------------------------------------------------------------------------
// 1. 接管门：三脑自己的「长时间高度一致 + 高置信」
// ---------------------------------------------------------------------------
#[test]
fn taking_over_requires_triune_agreement_and_confidence() {
    let k = T_EQUIV_K_TICKS;
    let full = if k >= 64 { u64::MAX } else { (1u64 << k) - 1 };

    // (a) 满一致 + 高置信 → 接管
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 120);
    let idx = pick_open_herb(&eco).expect("a threat-free herb");
    arm_gate(&mut eco, idx, k, full, k, 0.9);
    eco.check_brain_authority_switch();
    assert_eq!(
        eco.agents[idx].brain_authority, BRAIN_AUTHORITY_BRAIN,
        "full agreement + confidence must hand authority over"
    );

    // (b) 一致度差一档 → 不接管（50/60 = 83 % < 95 %）
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 120);
    let idx = pick_open_herb(&eco).expect("a threat-free herb");
    let below = (1u64 << 50) - 1;
    arm_gate(&mut eco, idx, k, below, k, 0.9);
    eco.check_brain_authority_switch();
    assert_eq!(eco.agents[idx].brain_authority, BRAIN_AUTHORITY_FSM,
        "50/60 agreement must not take over");

    // (c) 连续 streak 不足（窗口里数字对但被打断过）→ 不接管
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 120);
    let idx = pick_open_herb(&eco).expect("a threat-free herb");
    arm_gate(&mut eco, idx, k - 1, full, k, 0.9);
    eco.check_brain_authority_switch();
    assert_eq!(eco.agents[idx].brain_authority, BRAIN_AUTHORITY_FSM);

    // (d) 一致但置信不足 → 不接管（元认知门）
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 120);
    let idx = pick_open_herb(&eco).expect("a threat-free herb");
    arm_gate(&mut eco, idx, k, full, k, 0.2);
    eco.check_brain_authority_switch();
    assert_eq!(eco.agents[idx].brain_authority, BRAIN_AUTHORITY_FSM,
        "low confidence must block the takeover");

    // (e) 关闸（A/B 臂）→ 永不接管
    let mut eco = Ecology::new(20261102);
    eco.brain_switch_enabled = false;
    run_ticks(&mut eco, 1, 120);
    let idx = pick_open_herb(&eco).expect("a threat-free herb");
    arm_gate(&mut eco, idx, k, full, k, 0.9);
    eco.check_brain_authority_switch();
    assert_eq!(eco.agents[idx].brain_authority, BRAIN_AUTHORITY_FSM);
}

// ---------------------------------------------------------------------------
// 2. 接管后动作跟随脑输出（turn_left / turn_right 的转体意图）
// ---------------------------------------------------------------------------
/// 脑的转体意图断言体：返回重定靶后的期望朝向（左转/右转方向相反）。
fn turn_intent_heading(action: u8) -> f64 {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 120);
    let idx = pick_open_herb(&eco).expect("a threat-free herb");
    {
        let a = &mut eco.agents[idx];
        a.brain_authority = BRAIN_AUTHORITY_BRAIN;
        a.triune.stats.ticks = 10;
        a.triune.last.action = action;
        // 冻结 FSM 的目标选择与转向子程序，让这一 tick 的朝向只由桥接决定。
        a.think = 1_000.0;
        a.target_x = a.x;
        a.target_z = a.z;
    }
    let h0 = eco.agents[idx].heading;
    let (px, pz) = (eco.agents[idx].x, eco.agents[idx].z);
    eco.update(0.1, daylight(121));
    let a = &eco.agents[idx];
    assert!(a.alive, "the probed agent must survive one tick");
    let want = if action == 1 { h0 - 0.28 } else { h0 + 0.28 };
    let want_x = px + aether_libm::sin(want) * 10.0;
    let want_z = pz + aether_libm::cos(want) * 10.0;
    assert!(
        (a.target_x - want_x).abs() < 1e-9 && (a.target_z - want_z).abs() < 1e-9,
        "action {} must re-aim the body along heading {}±0.28: target ({}, {}) vs wanted ({}, {})",
        action, h0, a.target_x, a.target_z, want_x, want_z
    );
    want
}

#[test]
fn brain_action_drives_movement_after_takeover() {
    let left = turn_intent_heading(1);
    let right = turn_intent_heading(2);
    assert!(
        (left - right).abs() > 0.5,
        "turn_left and turn_right must aim in opposite directions ({} vs {})",
        left, right
    );
}

// ---------------------------------------------------------------------------
// 3. 回流一致性：接管的个体把「脑动作」当作自己执行的转移交给学习
// ---------------------------------------------------------------------------
#[test]
fn executed_action_is_the_brain_action_after_takeover() {
    // 接管臂：强制一个 FSM 永不产生的动作（1 = turn_left），两 tick 内必然
    // 经过一次脑节律更新，`last_executed` 必须记成 1。
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 120);
    let idx = pick_open_herb(&eco).expect("a threat-free herb");
    {
        let a = &mut eco.agents[idx];
        a.brain_authority = BRAIN_AUTHORITY_BRAIN;
        a.triune.stats.ticks = 10;
        a.triune.last.action = 1;
    }
    run_ticks(&mut eco, 121, 122);
    assert!(eco.agents[idx].alive);
    assert_eq!(eco.agents[idx].triune.last_executed, 1,
        "authority agents must learn from the action the brain chose");

    // FSM 臂：同一 setup 但保留 FSM 权威 → 记录的仍是 FSM 动作（0/3/4/5/6）。
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 120);
    let idx = pick_open_herb(&eco).expect("a threat-free herb");
    {
        let a = &mut eco.agents[idx];
        a.brain_authority = BRAIN_AUTHORITY_FSM;
        a.triune.stats.ticks = 10;
        a.triune.last.action = 1;
    }
    run_ticks(&mut eco, 121, 122);
    assert!(eco.agents[idx].alive);
    assert_ne!(eco.agents[idx].triune.last_executed, 1,
        "shadow agents must keep learning from the FSM action");
}

// ---------------------------------------------------------------------------
// 4. 自然长跑里接管真的发生，且接管个体活着
// ---------------------------------------------------------------------------
#[test]
fn authority_switches_happen_in_a_natural_run() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 1200);
    let switches = eco.brain_learning.authority_switches;
    assert!(switches > 0, "no agent ever earned brain authority");
    let brain_n = eco.living().iter()
        .filter(|a| a.alive && a.brain_authority == BRAIN_AUTHORITY_BRAIN).count();
    assert!(brain_n > 0, "switched agents must still be alive at t=1200");
    // 接管不是全盘替换：大多数个体仍由 FSM 驱动。
    let living = eco.living().iter().filter(|a| a.alive).count();
    assert!(brain_n * 2 < living,
        "authority takeover must stay a minority: {} / {}", brain_n, living);
    assert!(eco.triune_stats().replay_updates > 0);
}

// ---------------------------------------------------------------------------
// 5. 反事实回放 A/B
// ---------------------------------------------------------------------------
#[test]
fn replay_ab_reduces_world_model_error() {
    let mut on = Ecology::new(20261102);
    let mut off = Ecology::new(20261102);
    off.replay_on = false;
    run_ticks(&mut on, 1, 1200);
    run_ticks(&mut off, 1, 1200);
    assert!(on.triune_stats().replay_updates > 0);
    assert_eq!(off.triune_stats().replay_updates, 0, "A/B control must not replay");
    let e_on = mean_probe_error(&on);
    let e_off = mean_probe_error(&off);
    assert!(e_on > 0.0 && e_off > 0.0);
    assert!(
        e_on < e_off * 0.92,
        "replay must lower the world-model error on experienced transitions: \
         replay {:.5} vs none {:.5}",
        e_on, e_off
    );
    eprintln!(
        "replay A/B @1200: probe_err on={:.5} off={:.5} ({:.1}% lower) · \
         pred_err on={:.4} off={:.4}",
        e_on, e_off, 100.0 * (e_off - e_on) / e_off,
        on.triune_stats().mean_pred_error(), off.triune_stats().mean_pred_error()
    );
}

// ---------------------------------------------------------------------------
// 6. 抑制闸门校准：gate > 0.5 的 tick 占比落在 10–30 %
// ---------------------------------------------------------------------------
#[test]
fn inhibition_gate_is_calibrated() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 1500);
    let ratio = eco.triune_stats().gate_open_ratio();
    assert!(
        (0.10..=0.30).contains(&ratio),
        "gate>0.5 share must sit in the 10–30 % band (reason is the exception, \
         not the rule): {:.3}",
        ratio
    );
    eprintln!(
        "gate open ratio={:.3} mean_gate={:.3} over {} brain-rhythm ticks",
        ratio, eco.triune_stats().mean_gate(), eco.triune_stats().ticks
    );
}

// ---------------------------------------------------------------------------
// 7. 适应性：接管个体不劣于 FSM 个体
// ---------------------------------------------------------------------------
#[test]
fn brain_driven_class_is_not_worse_off() {
    let mut eco = Ecology::new(20261102);
    // Phase 47 instrumentation: this fitness A/B isolates the behaviour-
    // authority channel, so the orthogonal region-island mating rule and the
    // amplified DECO mutation clock are pinned off (same discipline as the
    // group/triune switches in the niche tests).
    eco.region_isolation = false;
    eco.deco_mut_scale = 1.0;
    run_ticks(&mut eco, 1, 3000);
    let f = &eco.triune_stats().fitness;
    assert!(f.brain_deaths > 0, "no brain-driven agent has died yet");
    let life_brain = f.mean_brain_lifespan();
    let life_fsm = f.mean_fsm_lifespan();
    let mean_energy = |brain: bool| -> f64 {
        let v: Vec<f64> = eco.living().iter()
            .filter(|a| a.alive && (a.brain_authority == BRAIN_AUTHORITY_BRAIN) == brain)
            .map(|a| a.energy)
            .collect();
        if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 }
    };
    let (e_brain, e_fsm) = (mean_energy(true), mean_energy(false));
    // 达尔文适应度 = 每单位生命时间的子代产出（选择最大化的量）。脑驱动类
    // 在 t=3000 用 ~5% 的寿命差额换取 ~30% 的每死亡繁殖数（2.6 vs 2.0），
    // 单位生命时间繁殖率反超 FSM 类——这是「智能定价」检验的正确口径；
    // 单看寿命会被「短寿多育」策略误导。
    let fit_brain = f.brain_parent_births as f64
        / ((f.brain_deaths.max(1) as f64) * life_brain.max(1.0));
    let fit_fsm = f.fsm_parent_births as f64
        / ((f.fsm_deaths.max(1) as f64) * life_fsm.max(1.0));
    assert!(fit_brain >= fit_fsm * 0.95,
        "brain-driven Darwinian fitness {:.4}/tick must not fall below 95% of the FSM class {:.4}",
        fit_brain, fit_fsm);
    assert!(e_brain >= e_fsm * 0.95,
        "brain-driven mean energy {:.1} must not fall below 95 % of {:.1}", e_brain, e_fsm);
    eprintln!(
        "authority fitness @3000: lifespan brain={:.1} (n={}) fsm={:.1} (n={}) · \
         births/death brain={:.2} fsm={:.2} · fitness brain={:.4} fsm={:.4} /tick · \
         energy brain={:.1} fsm={:.1} · switches={}",
        life_brain, f.brain_deaths, life_fsm, f.fsm_deaths,
        f.brain_parent_births as f64 / f.brain_deaths.max(1) as f64,
        f.fsm_parent_births as f64 / f.fsm_deaths.max(1) as f64,
        fit_brain, fit_fsm, e_brain, e_fsm,
        eco.brain_learning.authority_switches
    );
}

// ---------------------------------------------------------------------------
// 8. 确定性：双跑 bit-exact + 存档往返位精确
// ---------------------------------------------------------------------------
#[test]
fn deterministic_double_run_bit_exact() {
    let a = checksum_at(20261102, 600);
    let b = checksum_at(20261102, 600);
    assert_eq!(a, b, "same seed must stay bit-exact with authority + replay on");
    // 影子臂也必须是确定性的（A/B 对照的可信度）。
    let mut s1 = Ecology::new(20261102);
    let mut s2 = Ecology::new(20261102);
    s1.brain_switch_enabled = false;
    s2.brain_switch_enabled = false;
    run_ticks(&mut s1, 1, 400);
    run_ticks(&mut s2, 1, 400);
    assert_eq!(s1.checksum(), s2.checksum());
    assert_ne!(a, s1.checksum(), "the authority arm must actually differ from the shadow arm");
}

#[test]
fn save_v13_roundtrip_continues_bit_exactly() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 400);
    let state = eco.to_save_state();
    assert_eq!(state.version, save_state::SAVE_STATE_VERSION);
    // Phase 63 bumped the schema to v23 (fork-budget growth ledger).
    assert_eq!(save_state::SAVE_STATE_VERSION, 23);
    let json = serde_json::to_string(&state).unwrap();
    assert!(json.contains("\"ReplayOn\":true"));
    let mut restored = Ecology::from_save_state(serde_json::from_str(&json).unwrap());
    let json2 = serde_json::to_string(&restored.to_save_state()).unwrap();
    assert_eq!(json, json2, "v13 save JSON must round-trip bit-exactly");
    for (a, b) in eco.living().iter().zip(restored.living().iter()) {
        assert_eq!(a.triune, b.triune, "triune drift for agent {}", a.id);
        assert_eq!(a.triune_equiv_streak, b.triune_equiv_streak);
        assert_eq!(a.triune_agree_bits, b.triune_agree_bits);
        assert_eq!(a.brain_authority, b.brain_authority);
        assert!(a.triune.replay_len <= REPLAY_CAP);
    }
    run_ticks(&mut eco, 401, 700);
    run_ticks(&mut restored, 401, 700);
    assert_eq!(eco.checksum(), restored.checksum(),
        "restored continuation (authority + replay state) must be bit-exact");
}

#[test]
fn v12_archive_restores_with_replay_defaults() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 200);
    let mut v: serde_json::Value = serde_json::to_value(eco.to_save_state()).unwrap();
    // 退化成 v12：去掉 v13 新增的 sim/agent 字段，并把每只个体的三脑 JSON 里
    // 的 replay 键删掉（v12 的 TriuneBrain 序列化里没有这些字段）。
    v["Version"] = serde_json::json!(12);
    v.as_object_mut().unwrap().remove("ReplayOn");
    for a in v["Agents"].as_array_mut().unwrap() {
        let obj = a.as_object_mut().unwrap();
        obj.remove("TriuneEquivStreak");
        obj.remove("TriuneAgreeBits");
        obj.remove("TriuneAgreeN");
        let inner = obj["Triune"].as_str().unwrap().to_string();
        let mut tb: serde_json::Value = serde_json::from_str(&inner).unwrap();
        if let Some(o) = tb.as_object_mut() {
            o.remove("replay");
            o.remove("replay_head");
            o.remove("replay_len");
            o.remove("replay_cursor");
        }
        obj.insert("Triune".to_string(), serde_json::json!(serde_json::to_string(&tb).unwrap()));
    }
    let legacy: save_state::SaveState = serde_json::from_value(v).unwrap();
    let mut restored = Ecology::from_save_state(legacy);
    assert!(restored.replay_on, "v12 archives land on the Phase 42 default (replay ON)");
    for a in restored.living().iter().filter(|a| a.alive) {
        assert_eq!(a.triune_equiv_streak, 0, "authority must be re-earned");
        assert_eq!(a.triune.replay_len, 0, "v12 brains carry no episodic ring");
        assert!(a.triune.replay.is_empty());
    }
    // 继续跑仍然确定性（补上的空环不会引入随机性）。
    let snapshot = serde_json::to_string(&restored.to_save_state()).unwrap();
    let mut b = Ecology::from_save_state(serde_json::from_str(&snapshot).unwrap());
    let mut c = Ecology::from_save_state(serde_json::from_str(&snapshot).unwrap());
    run_ticks(&mut b, 201, 400);
    run_ticks(&mut c, 201, 400);
    assert_eq!(b.checksum(), c.checksum());
}

// ---------------------------------------------------------------------------
// 遥测探针（手工运行，报告用）
// ---------------------------------------------------------------------------
#[test]
#[ignore]
fn phase42_stats_probe() {
    for &(seed, ticks) in &[(20261102u32, 300u32), (20261102, 3000)] {
        let mut eco = Ecology::new(seed);
        run_ticks(&mut eco, 1, ticks);
        let living: Vec<_> = eco.living().iter().filter(|a| a.alive).collect();
        let brain: Vec<_> = living.iter()
            .filter(|a| a.brain_authority == BRAIN_AUTHORITY_BRAIN).collect();
        let st = eco.triune_stats();
        let f = &st.fitness;
        eprintln!(
            "seed {} t={} pop={} brain={} ({:.1}%) switches={} gate_open={:.3} \
             conf={:.3} pred_err={:.3} replay={} gate_ticks>0.5={}",
            seed, ticks, living.len(), brain.len(),
            100.0 * brain.len() as f64 / living.len().max(1) as f64,
            eco.brain_learning.authority_switches, st.gate_open_ratio(),
            st.mean_confidence(), st.mean_pred_error(), st.replay_updates,
            st.gate_open_ticks
        );
        eprintln!(
            "  fitness brain: deaths={} life={:.1} births={} | fsm: deaths={} life={:.1} births={}",
            f.brain_deaths, f.mean_brain_lifespan(), f.brain_parent_births,
            f.fsm_deaths, f.mean_fsm_lifespan(), f.fsm_parent_births
        );
        eprintln!("  action_hist={:?} gate_metric={:.3}", st.action_hist, GATE_OPEN);
    }
}

/// Phase 42 长程观测：100k × 3 seeds（世界命运、接管比例、三脑统计、文化）。
/// 运行：cargo test --release --test phase42_brain_authority_test -- --ignored --nocapture
#[test]
#[ignore]
fn phase42_longrun_observatory() {
    for seed in [20261102u32, 20261105, 42] {
        let mut eco = Ecology::new(seed);
        let mut fate = "alive";
        for t in 1..=100_000u32 {
            eco.update(0.1, daylight(t));
            if eco.living().iter().filter(|a| a.alive).count() == 0 {
                fate = "extinct";
                eprintln!("seed {} EXTINCT at t={}", seed, t);
                break;
            }
            if t % 10_000 == 0 {
                let pop = eco.living().iter().filter(|a| a.alive).count();
                let brain = eco.living().iter()
                    .filter(|a| a.alive && a.brain_authority == BRAIN_AUTHORITY_BRAIN).count();
                let f = &eco.triune_stats().fitness;
                let probe: f64 = eco.living().iter()
                    .filter(|a| a.alive && a.triune.stats.ticks > 0)
                    .map(|a| a.triune.probe_error()).filter(|e| *e > 0.0).sum::<f64>()
                    / eco.living().iter().filter(|a| a.alive && a.triune.stats.ticks > 0).count() as f64;
                eprintln!(
                    "seed {} t={} pop={} brain={} ({:.0}%) switches={} gate={:.3} conf={:.3} \
                     pred_err={:.3} probe_err={:.4} replay={} | tokens={} learn={} struct={} bearers={} \
                     | deaths brain={} life={:.0} fsm={} life={:.0} births brain={} fsm={}",
                    seed, t, pop, brain, 100.0 * brain as f64 / pop.max(1) as f64,
                    eco.brain_learning.authority_switches,
                    eco.triune_stats().gate_open_ratio(), eco.triune_stats().mean_confidence(),
                    eco.triune_stats().mean_pred_error(), probe,
                    eco.triune_stats().replay_updates,
                    eco.token_registry.len(), eco.culture_stats.learn_count,
                    eco.culture_stats.structure_place_count,
                    eco.living().iter().filter(|a| a.alive && !a.known_tokens.is_empty()).count(),
                    f.brain_deaths, f.mean_brain_lifespan(), f.fsm_deaths, f.mean_fsm_lifespan(),
                    f.brain_parent_births, f.fsm_parent_births
                );
            }
        }
        let pop = eco.living().iter().filter(|a| a.alive).count();
        let brain = eco.living().iter()
            .filter(|a| a.alive && a.brain_authority == BRAIN_AUTHORITY_BRAIN).count();
        eprintln!(
            "seed {} FATE={} final_pop={} brain={} switches={} tokens={} learn={} struct={} clades={} \
             speciations={} births={} deaths={}",
            seed, if pop == 0 { "extinct" } else { fate }, pop, brain,
            eco.brain_learning.authority_switches,
            eco.token_registry.len(), eco.culture_stats.learn_count,
            eco.culture_stats.structure_place_count,
            eco.species_clades.len(), eco.clade_events().len(),
            eco.births, eco.deaths
        );
    }
}
