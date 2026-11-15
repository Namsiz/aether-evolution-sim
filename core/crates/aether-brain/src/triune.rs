//! Phase 41 — 三脑架构（Triune Brain）：本能脑 / 情绪脑 / 理智脑。
//!
//! 现实对应（MacLean 三重脑 + 现代神经科学的最小工程同构）：
//! * **本能脑**（脑干-基底核）：固定拓扑密度的直接反射弧（sense → action），
//!   每个突触权重、偏置、增益、FAP 阈值都是可演化基因。没有「IF 威胁 THEN
//!   逃跑」的分支——行为由 56 个基因编码的反射矩阵计算出来，符号可突变。
//! * **情绪脑**（杏仁核-海马-伏隔核）：fear / desire / bond / arousal 四个
//!   连续状态变量（累积 + 衰减动力学），加一张有限槽位的 state→valence
//!   情绪记忆表（基因控制容量、学习率、提取增益）。情绪到动作的 28 个映射
//!   权重同样是基因，不存在写死的 fear→flee。
//! * **理智脑**（前额叶）：k 个持久工作记忆槽位、每个动作一套线性世界模型
//!   （state→next, expected_reward）、2-4 步 rollout 的模型式规划、抑制控制
//!   闸门（理智判断与本能/情绪冲动冲突时压制冲动）、元认知置信度（由预测
//!   误差 EMA 校准）。
//! * **神经调质**：dopamine（奖励预测误差）、norepinephrine（唤醒增益）、
//!   serotonin（情绪稳定）作为全局标量，敏感度是基因。
//!
//! 确定性纪律：
//! * `tick` / `forward` / 世界模型学习 / 情绪记忆更新全部零 RNG（纯状态函数）。
//! * 结构突变（基因扰动、工作记忆/记忆槽位增删、世界模型漂移）只发生在出生
//!   路径，消耗调用方注入的专用 RNG 流。
//! * 所有非线性走 `aether_libm`（本 crate 的 `libm_tanh` / `libm_sigmoid`）。
//! * 所有状态有界：NaN/Inf 一律回退到安全值，避免污染存档。
//!
//! 成本（延续「智能定价」）：
//! * 维护费 = `TRIUNE_PER_UNIT_COST × Σ 三层节点数`；理智脑的节点数包含
//!   `rollout_depth × OUTPUT_ACTIONS`（每个 rollout 步评估所有动作模型）。
//! * 决策费 = `TRIUNE_PER_ROLLOUT_COST × rollout_depth` + 抑制代价
//!   `inhibit_cost_gene × gate`，按秒计。

use serde::{Deserialize, Serialize};

use crate::{libm_sigmoid, libm_tanh, INPUT_CHANNELS, OUTPUT_ACTIONS};

pub const EMOTION_DIM: usize = 4;
pub const NEUROMOD_DIM: usize = 3;
pub const MEMORY_KEY_DIM: usize = 4;
pub const MIN_WM_SLOTS: usize = 2;
pub const MAX_WM_SLOTS: usize = 6;
pub const MIN_MEMORY_SLOTS: usize = 2;
pub const MAX_MEMORY_SLOTS: usize = 8;
pub const MIN_ROLLOUT: usize = 2;
pub const MAX_ROLLOUT: usize = 4;
pub const WORLD_STATE_DIM: usize = INPUT_CHANNELS;
pub const WORLD_PARAMS_PER_ACTION: usize =
    WORLD_STATE_DIM * WORLD_STATE_DIM + WORLD_STATE_DIM + WORLD_STATE_DIM + 1;
pub const WORLD_PARAMS: usize = OUTPUT_ACTIONS * WORLD_PARAMS_PER_ACTION;

/// 三层维护费单价（每节点每秒）。参考脑 59 节点 ≈ 0.012/s；这一标定来自
/// 实测：0.0008 / 0.0004 会把群体层 FEED 盈余压到 `GROUP_REPRO_POOL` 以下，
/// 使 Phase 39.1 的组繁殖从 t≈1200 推迟到 t≈1800 以上；0.0002 保住原组层
/// 动力学。`BRAIN_PER_NEURON_COST` 保持不变，三脑成本单独记入
/// `ledger.triune_cost`。
pub const TRIUNE_PER_UNIT_COST: f64 = 0.0002;
/// 每个 rollout 深度每秒的决策费（理智脑的边际计算成本）。
pub const TRIUNE_PER_ROLLOUT_COST: f64 = 0.0002;
/// 本能层节点数：8 个输入通道 + 7 个动作输出。
pub const INSTINCT_UNITS: usize = INPUT_CHANNELS + OUTPUT_ACTIONS;
/// 理智脑的非工作记忆皮层单元（状态编码/预测回路）。
pub const REASON_CORTEX_UNITS: usize = INPUT_CHANNELS;
/// 本能层权重数（56）与情绪→动作权重数（28）。
pub const INSTINCT_W_COUNT: usize = OUTPUT_ACTIONS * INPUT_CHANNELS;
pub const EMOTION_W_COUNT: usize = EMOTION_DIM * OUTPUT_ACTIONS;
/// 经验回放环形缓冲容量（Phase 42）。固定常量而非基因：回放缓冲是个体
/// 情节记忆，不是可遗传的脑结构；固定 12 条把每脑的额外内存锁在
/// 12 × 136 B ≈ 1.6 KB，且不给基因布局新增维度（148 基因保持不动）。
pub const REPLAY_CAP: usize = 12;
/// 抑制闸门「打开」判据（与 `Decision::inhibit_gate` 的 0.5 分界一致）。
pub const GATE_OPEN: f64 = 0.5;

// ---------------------------------------------------------------------------
// 基因布局（148 个连续 f64；结构基因存连续值、使用时四舍五入到整数）
// ---------------------------------------------------------------------------

pub const G_INSTINCT_W: usize = 0; // 0..56  (action × input)
pub const G_INSTINCT_B: usize = 56; // 56..63 (7 action biases)
pub const G_INSTINCT_GAIN: usize = 63;
pub const G_INSTINCT_THRESHOLD: usize = 64;
pub const G_FEAR_RISE: usize = 65;
pub const G_FEAR_DECAY: usize = 66;
pub const G_DESIRE_RISE: usize = 67;
pub const G_DESIRE_DECAY: usize = 68;
pub const G_BOND_RISE: usize = 69;
pub const G_BOND_DECAY: usize = 70;
pub const G_AROUSAL_GAIN: usize = 71;
pub const G_AROUSAL_DECAY: usize = 72;
pub const G_MEMORY_ETA: usize = 73;
pub const G_MEMORY_SLOTS: usize = 74;
pub const G_MEMORY_FEAR_GAIN: usize = 75;
pub const G_EMOTION_W: usize = 76; // 76..104 (emotion × action)
pub const G_MEMORY_W: usize = 104; // 104..111 (valence → action)
pub const G_WM_IN: usize = 111; // 111..119
pub const G_WM_BIAS: usize = 119; // 119..125
pub const G_WM_DECAY: usize = 125;
pub const G_WM_WRITE: usize = 126;
pub const G_WM_GAIN: usize = 127;
pub const G_WM_SLOTS: usize = 128;
pub const G_ROLLOUT: usize = 129;
pub const G_DISCOUNT: usize = 130;
pub const G_WORLD_LR: usize = 131;
pub const G_REWARD_LR: usize = 132;
pub const G_INHIBIT_GAIN: usize = 133;
pub const G_INHIBIT_THRESHOLD: usize = 134;
pub const G_INHIBIT_STRENGTH: usize = 135;
pub const G_INHIBIT_COST: usize = 136;
pub const G_CONFIDENCE_TAU: usize = 137;
pub const G_DA_SENS: usize = 138;
pub const G_NE_SENS: usize = 139;
pub const G_SER_SENS: usize = 140;
pub const G_NE_GAIN: usize = 141;
pub const G_SER_GAIN: usize = 142;
pub const G_FUSION_INSTINCT: usize = 143;
pub const G_FUSION_EMOTION: usize = 144;
pub const G_FUSION_REASON: usize = 145;
pub const G_PLAN_GAIN: usize = 146;
pub const G_REASON_CONF_FLOOR: usize = 147;
pub const GENE_COUNT: usize = 148;

/// 每个基因的 (min, max, mutation sigma)。结构基因单独处理。
pub fn gene_bounds(i: usize) -> (f64, f64, f64) {
    match i {
        G_INSTINCT_W..=G_INSTINCT_B_END => (-3.0, 3.0, 0.12),
        G_INSTINCT_GAIN => (0.2, 4.0, 0.12),
        G_INSTINCT_THRESHOLD => (-1.5, 1.5, 0.08),
        G_FEAR_RISE => (0.05, 4.0, 0.10),
        G_FEAR_DECAY => (0.02, 1.5, 0.06),
        G_DESIRE_RISE => (0.05, 4.0, 0.10),
        G_DESIRE_DECAY => (0.02, 1.5, 0.06),
        G_BOND_RISE => (0.02, 2.5, 0.08),
        G_BOND_DECAY => (0.01, 1.2, 0.05),
        G_AROUSAL_GAIN => (0.05, 4.0, 0.10),
        G_AROUSAL_DECAY => (0.02, 1.5, 0.06),
        G_MEMORY_ETA => (0.001, 0.25, 0.01),
        G_MEMORY_SLOTS => (MIN_MEMORY_SLOTS as f64, MAX_MEMORY_SLOTS as f64, 0.0),
        G_MEMORY_FEAR_GAIN => (0.0, 4.0, 0.10),
        G_EMOTION_W..=G_MEMORY_W_END => (-3.0, 3.0, 0.12),
        G_WM_IN..=G_WM_BIAS_END => (-1.5, 1.5, 0.08),
        G_WM_DECAY => (0.05, 0.99, 0.04),
        G_WM_WRITE => (0.05, 1.0, 0.06),
        G_WM_GAIN => (0.1, 3.0, 0.10),
        G_WM_SLOTS => (MIN_WM_SLOTS as f64, MAX_WM_SLOTS as f64, 0.0),
        G_ROLLOUT => (MIN_ROLLOUT as f64, MAX_ROLLOUT as f64, 0.0),
        G_DISCOUNT => (0.30, 0.99, 0.03),
        G_WORLD_LR => (0.0005, 0.50, 0.02),
        G_REWARD_LR => (0.0005, 0.50, 0.02),
        G_INHIBIT_GAIN => (0.0, 10.0, 0.20),
        G_INHIBIT_THRESHOLD => (-1.0, 2.0, 0.10),
        G_INHIBIT_STRENGTH => (0.0, 4.0, 0.12),
        G_INHIBIT_COST => (0.0, 0.01, 0.0005),
        G_CONFIDENCE_TAU => (1.0, 200.0, 5.0),
        G_DA_SENS => (0.0, 4.0, 0.10),
        G_NE_SENS => (0.0, 4.0, 0.10),
        G_SER_SENS => (0.0, 4.0, 0.10),
        G_NE_GAIN => (0.0, 2.0, 0.08),
        G_SER_GAIN => (0.0, 2.0, 0.08),
        G_FUSION_INSTINCT => (0.0, 3.0, 0.10),
        G_FUSION_EMOTION => (0.0, 3.0, 0.10),
        G_FUSION_REASON => (0.0, 3.0, 0.10),
        G_PLAN_GAIN => (0.0, 4.0, 0.10),
        G_REASON_CONF_FLOOR => (0.0, 1.0, 0.06),
        _ => (0.0, 1.0, 0.05),
    }
}

const G_INSTINCT_B_END: usize = G_INSTINCT_B + OUTPUT_ACTIONS - 1;
const G_MEMORY_W_END: usize = G_MEMORY_W + OUTPUT_ACTIONS - 1;
const G_WM_BIAS_END: usize = G_WM_BIAS + MAX_WM_SLOTS - 1;

#[inline]
fn argmax7(v: &[f64; OUTPUT_ACTIONS]) -> usize {
    let mut best = 0usize;
    let mut bv = v[0];
    for i in 1..OUTPUT_ACTIONS {
        if v[i] > bv {
            bv = v[i];
            best = i;
        }
    }
    best
}

#[inline]
fn argmax_state(v: &[f64; INPUT_CHANNELS]) -> usize {
    let mut best = 0usize;
    let mut bv = v[0];
    for i in 1..INPUT_CHANNELS {
        if v[i] > bv {
            bv = v[i];
            best = i;
        }
    }
    best
}

#[inline]
fn finite(x: f64, fallback: f64) -> f64 {
    if x.is_finite() { x } else { fallback }
}

#[inline]
fn clamp01(x: f64) -> f64 {
    finite(x, 0.0).clamp(0.0, 1.0)
}

#[inline]
fn round_slots(v: f64, lo: usize, hi: usize) -> usize {
    let r = finite(v, lo as f64).round() as i64;
    r.clamp(lo as i64, hi as i64) as usize
}

// ---------------------------------------------------------------------------
// 决策与单 tick 报告
// ---------------------------------------------------------------------------

/// 一次仲裁的完整轨迹（测试与遥测可读）。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    /// 最终动作（0..7，argmax 融合 logits）。
    pub action: u8,
    pub instinct_action: u8,
    pub emotion_action: u8,
    /// 理智脑的世界模型 rollout 首选。
    pub plan_action: u8,
    /// 本能 + 情绪融合后的冲动（抑制控制的对象）。
    pub urge_action: u8,
    /// 抑制闸门 ∈ [0,1]。
    pub inhibit_gate: f64,
    /// 元认知置信度 ∈ [0,1]。
    pub confidence: f64,
    /// 选中动作的 rollout 期望回报（贴现）。
    pub expected_value: f64,
    pub fused: [f64; OUTPUT_ACTIONS],
    pub reason: [f64; OUTPUT_ACTIONS],
}

impl Default for Decision {
    fn default() -> Self {
        Decision {
            action: 0,
            instinct_action: 0,
            emotion_action: 0,
            plan_action: 0,
            urge_action: 0,
            inhibit_gate: 0.0,
            confidence: 0.5,
            expected_value: 0.0,
            fused: [0.0; OUTPUT_ACTIONS],
            reason: [0.0; OUTPUT_ACTIONS],
        }
    }
}

/// 单 tick 的可观测增量（ecology 聚合用；不进动力学）。
#[derive(Clone, Debug, Default)]
pub struct TickOutcome {
    pub decision: Decision,
    pub fear: f64,
    pub confidence: f64,
    pub pred_error: f64,
    pub fear_event: bool,
    pub fear_conditioning: bool,
    pub memory_retrieved: bool,
    pub planned_override: bool,
    pub inhibition: bool,
    pub rollout_steps: u32,
    pub wm_updates: u32,
    pub prediction_updated: bool,
    /// 本 tick 决策费（每秒费率，未乘 dt）。
    pub decision_rate: f64,
}

/// 一条被回放的经验（Phase 42）：`(sense, action, next, reward)`。
/// 存完整的 next 而不是靠环形后继重建——最后写入的那条没有后继，
/// 而且重建会把两次不同动作之间的跳变当成转移。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplayEntry {
    pub sense: [f64; INPUT_CHANNELS],
    pub action: u8,
    pub next: [f64; INPUT_CHANNELS],
    pub reward: f64,
}

impl Default for ReplayEntry {
    fn default() -> Self {
        ReplayEntry {
            sense: [0.0; INPUT_CHANNELS],
            action: 0,
            next: [0.0; INPUT_CHANNELS],
            reward: 0.0,
        }
    }
}

/// 累计统计（可观测性；随三脑状态一起存档，不影响动力学）。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TriuneStats {
    pub ticks: u64,
    pub fear_events: u64,
    pub fear_conditioning: u64,
    pub memory_retrievals: u64,
    pub wm_updates: u64,
    pub rollout_steps: u64,
    pub planned_overrides: u64,
    pub inhibitions: u64,
    pub prediction_updates: u64,
    pub high_confidence_ticks: u64,
    pub max_fear: f64,
    pub fear_sum: f64,
    pub confidence_sum: f64,
    pub pred_error_sum: f64,
    pub action_hist: [u64; OUTPUT_ACTIONS],
    /// Phase 42: 离线回放次数。
    #[serde(default)]
    pub replay_updates: u64,
    /// Phase 42: 抑制闸门统计（校准指标）：打开次数与闸门值总和。
    #[serde(default)]
    pub gate_open_ticks: u64,
    #[serde(default)]
    pub gate_sum: f64,
}

impl TriuneStats {
    pub fn mean_fear(&self) -> f64 {
        if self.ticks == 0 { 0.0 } else { self.fear_sum / self.ticks as f64 }
    }
    pub fn mean_confidence(&self) -> f64 {
        if self.ticks == 0 { 0.0 } else { self.confidence_sum / self.ticks as f64 }
    }
    pub fn mean_pred_error(&self) -> f64 {
        if self.ticks == 0 { 0.0 } else { self.pred_error_sum / self.ticks as f64 }
    }
    /// Phase 42: gate > 0.5 的 tick 占比（抑制校准的主指标）。
    pub fn gate_open_ratio(&self) -> f64 {
        if self.ticks == 0 { 0.0 } else { self.gate_open_ticks as f64 / self.ticks as f64 }
    }
    pub fn mean_gate(&self) -> f64 {
        if self.ticks == 0 { 0.0 } else { self.gate_sum / self.ticks as f64 }
    }
}

// ---------------------------------------------------------------------------
// 三脑系统
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TriuneBrain {
    /// 148 个可演化基因（布局见模块顶部常量）。
    pub genes: Vec<f64>,
    // 情绪脑：连续状态变量
    pub fear: f64,
    pub desire: f64,
    pub bond: f64,
    pub arousal: f64,
    /// 情绪记忆：state→valence 关联表（长度 = memory_capacity）。
    pub memory_keys: Vec<[f64; MEMORY_KEY_DIM]>,
    pub memory_valence: Vec<f64>,
    pub memory_strength: Vec<f64>,
    pub memory_age: Vec<u32>,
    // 神经调质
    pub dopamine: f64,
    pub norepinephrine: f64,
    pub serotonin: f64,
    // 理智脑
    /// k 个持久工作记忆槽位。
    pub wm: Vec<f64>,
    /// 每个动作一套线性世界模型（布局见 `world_index`）。
    pub world: Vec<f64>,
    /// 上一 tick 的奖励预测（RPE 基线，可观测）。
    pub prev_reward_pred: f64,
    pub confidence: f64,
    pub pred_error_ema: f64,
    /// 元认知校准：1 - |置信度 - 预测准确率| 的 EMA。
    pub calibration: f64,
    pub last_rpe: f64,
    /// Phase 42: 经验回放环形缓冲（海马尖波涟漪重放的最小工程同构）。
    /// `replay[0..replay_len]` 是有效条目，`replay_head` 是下一个写入槽，
    /// `replay_cursor` 是确定性轮转读取位置。
    #[serde(default)]
    pub replay: Vec<ReplayEntry>,
    #[serde(default)]
    pub replay_head: usize,
    #[serde(default)]
    pub replay_len: usize,
    #[serde(default)]
    pub replay_cursor: usize,
    // 记账
    pub tick_count: u32,
    pub prev_fear: f64,
    pub last_sense: [f64; INPUT_CHANNELS],
    pub last_seen: bool,
    pub last_executed: u8,
    pub last: Decision,
    pub stats: TriuneStats,
}

#[inline]
pub fn world_index(action: usize, row: usize, col: usize) -> usize {
    action * WORLD_PARAMS_PER_ACTION + row * WORLD_STATE_DIM + col
}

#[inline]
fn world_bias_index(action: usize, row: usize) -> usize {
    action * WORLD_PARAMS_PER_ACTION + WORLD_STATE_DIM * WORLD_STATE_DIM + row
}

#[inline]
fn world_reward_w_index(action: usize, col: usize) -> usize {
    action * WORLD_PARAMS_PER_ACTION + WORLD_STATE_DIM * WORLD_STATE_DIM + WORLD_STATE_DIM + col
}

#[inline]
fn world_reward_b_index(action: usize) -> usize {
    action * WORLD_PARAMS_PER_ACTION + WORLD_PARAMS_PER_ACTION - 1
}

impl TriuneBrain {
    /// 参考基因组：六祖先的初始值（可演化，不是行为剧本）。`kind` 只影响
    /// 初始尺寸基因（工作记忆槽位 / rollout 深度 / 情绪记忆容量），与
    /// `reference_brain(kind)` 的 grandfathering 语义一致。
    pub fn reference_genes(kind: &str) -> Vec<f64> {
        let mut g = vec![0.0f64; GENE_COUNT];
        // 本能反射矩阵：每行一个动作，每列一个输入通道。
        // 行为方向由基因符号决定，后续突变可以翻转。
        let rows: [[f64; INPUT_CHANNELS]; OUTPUT_ACTIONS] = [
            // forward: 食物/猎物/距离驱动，疲劳与威胁抑制
            [-0.6, 0.0, 0.5, 0.6, 0.2, -1.0, 0.3, 0.4],
            // turn_left / turn_right：轻微趋近 + 探索
            [0.2, 0.0, 0.1, 0.1, 0.0, -0.1, 0.0, 0.1],
            [0.2, 0.0, 0.1, 0.1, 0.0, -0.1, 0.0, 0.1],
            // eat: 饥饿 + 食物
            [-0.5, 1.2, 1.5, 0.2, 0.0, -0.2, 0.0, 0.3],
            // flee: 威胁
            [2.0, -0.2, -0.2, -0.2, 0.0, -0.4, -0.1, 0.0],
            // reproduce: 能量充足（hungry 低）+ 光照
            [-0.3, -0.8, 0.0, 0.0, 0.3, -0.3, 0.2, -0.2],
            // help: 社会性
            [-0.5, -0.1, 0.0, 0.0, 1.5, 0.0, 0.0, 0.0],
        ];
        for a in 0..OUTPUT_ACTIONS {
            for j in 0..INPUT_CHANNELS {
                g[G_INSTINCT_W + a * INPUT_CHANNELS + j] = rows[a][j];
            }
        }
        let biases = [0.5, -0.2, 0.2, 0.0, 0.0, 0.0, 0.0];
        for a in 0..OUTPUT_ACTIONS {
            g[G_INSTINCT_B + a] = biases[a];
        }
        g[G_INSTINCT_GAIN] = 1.0;
        g[G_INSTINCT_THRESHOLD] = 0.0;

        // 情绪动力学
        g[G_FEAR_RISE] = 1.5;
        g[G_FEAR_DECAY] = 0.5;
        g[G_DESIRE_RISE] = 1.2;
        g[G_DESIRE_DECAY] = 0.4;
        g[G_BOND_RISE] = 0.8;
        g[G_BOND_DECAY] = 0.2;
        g[G_AROUSAL_GAIN] = 1.0;
        g[G_AROUSAL_DECAY] = 0.3;
        g[G_MEMORY_ETA] = 0.05;
        g[G_MEMORY_SLOTS] = 4.0;
        g[G_MEMORY_FEAR_GAIN] = 1.0;

        // 情绪 → 动作（fear/desire/bond/arousal × 7 actions）
        let emo: [[f64; OUTPUT_ACTIONS]; EMOTION_DIM] = [
            // fear: flee 高，forward 抑制
            [ -1.5, 0.2, 0.2, -0.2, 2.0, -0.3, 0.0],
            // desire: eat / approach
            [0.5, 0.0, 0.0, 2.0, 0.0, 0.3, 0.0],
            // bond: help
            [0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0],
            // arousal: 唤醒放大前移/逃跑
            [0.8, 0.0, 0.0, 0.0, 0.5, 0.0, 0.0],
        ];
        for e in 0..EMOTION_DIM {
            for a in 0..OUTPUT_ACTIONS {
                g[G_EMOTION_W + e * OUTPUT_ACTIONS + a] = emo[e][a];
            }
        }
        // 情绪记忆 valence → 动作：负价 → flee，正价 → eat/forward
        let mem = [0.2, 0.0, 0.0, 0.8, 0.8, 0.0, 0.0];
        for a in 0..OUTPUT_ACTIONS {
            g[G_MEMORY_W + a] = mem[a];
        }

        // 工作记忆
        for j in 0..INPUT_CHANNELS {
            g[G_WM_IN + j] = 0.5;
        }
        let wm_bias = [0.3, -0.1, 0.2, -0.2, 0.1, 0.0];
        for i in 0..MAX_WM_SLOTS {
            g[G_WM_BIAS + i] = wm_bias[i];
        }
        g[G_WM_DECAY] = 0.7;
        g[G_WM_WRITE] = 0.5;
        g[G_WM_GAIN] = 1.0;

        // 理智脑默认尺寸：PRED 规划更深，DECO/FISH 更浅。
        let (wm_slots, mem_slots, depth) = match kind {
            "pred" => (5.0, 5.0, 4.0),
            "herb" => (4.0, 4.0, 3.0),
            "scav" => (4.0, 4.0, 3.0),
            "omn" => (4.0, 5.0, 3.0),
            "fish" => (3.0, 3.0, 2.0),
            "deco" => (2.0, 3.0, 2.0),
            _ => (3.0, 4.0, 2.0),
        };
        g[G_WM_SLOTS] = wm_slots;
        g[G_MEMORY_SLOTS] = mem_slots;
        g[G_ROLLOUT] = depth;

        g[G_DISCOUNT] = 0.90;
        g[G_WORLD_LR] = 0.02;
        g[G_REWARD_LR] = 0.05;
        g[G_INHIBIT_GAIN] = 0.75;
        g[G_INHIBIT_THRESHOLD] = 2.0;
        g[G_INHIBIT_STRENGTH] = 2.0;
        g[G_INHIBIT_COST] = 0.002;
        g[G_CONFIDENCE_TAU] = 20.0;

        // 神经调质
        g[G_DA_SENS] = 1.0;
        g[G_NE_SENS] = 1.0;
        g[G_SER_SENS] = 1.0;
        g[G_NE_GAIN] = 1.0;
        g[G_SER_GAIN] = 1.0;

        // 层间融合
        g[G_FUSION_INSTINCT] = 1.2;
        g[G_FUSION_EMOTION] = 1.0;
        g[G_FUSION_REASON] = 1.0;
        g[G_PLAN_GAIN] = 1.0;
        g[G_REASON_CONF_FLOOR] = 0.3;
        g
    }

    /// 确定性参考脑（`id` 派生初始化噪声，不消耗主 RNG 流）。
    pub fn reference(kind: &str, id: u32) -> Self {
        let mut genes = Self::reference_genes(kind);
        let mut rng = aether_rng::Mulberry32::new(
            id.wrapping_mul(0x9E37_79B9).wrapping_add(crate::BRAIN_RNG_TAG),
        );
        for i in 0..GENE_COUNT {
            let (lo, hi, sigma) = gene_bounds(i);
            if sigma == 0.0 {
                genes[i] = genes[i].clamp(lo, hi);
                continue;
            }
            let v = genes[i] + (rng.next_f64() - 0.5) * 0.1 * sigma.max(0.05);
            genes[i] = v.clamp(lo, hi);
        }
        Self::from_genes(genes)
    }

    /// 从基因向量构造（状态清零，结构按基因同步）。
    pub fn from_genes(genes: Vec<f64>) -> Self {
        let mut b = TriuneBrain {
            genes,
            fear: 0.0,
            desire: 0.0,
            bond: 0.0,
            arousal: 0.0,
            memory_keys: Vec::new(),
            memory_valence: Vec::new(),
            memory_strength: Vec::new(),
            memory_age: Vec::new(),
            dopamine: 0.0,
            norepinephrine: 0.0,
            serotonin: 0.0,
            wm: Vec::new(),
            world: vec![0.0; WORLD_PARAMS],
            prev_reward_pred: 0.0,
            confidence: 0.5,
            pred_error_ema: 0.5,
            calibration: 0.5,
            last_rpe: 0.0,
            replay: Vec::new(),
            replay_head: 0,
            replay_len: 0,
            replay_cursor: 0,
            tick_count: 0,
            prev_fear: 0.0,
            last_sense: [0.0; INPUT_CHANNELS],
            last_seen: false,
            last_executed: 0,
            last: Decision::default(),
            stats: TriuneStats::default(),
        };
        b.normalize_genes();
        b.sync_structure();
        b
    }

    #[inline]
    fn g(&self, i: usize) -> f64 {
        finite(self.genes[i], 0.0)
    }

    pub fn wm_slots(&self) -> usize {
        round_slots(self.g(G_WM_SLOTS), MIN_WM_SLOTS, MAX_WM_SLOTS)
    }

    pub fn memory_capacity(&self) -> usize {
        round_slots(self.g(G_MEMORY_SLOTS), MIN_MEMORY_SLOTS, MAX_MEMORY_SLOTS)
    }

    pub fn rollout_depth(&self) -> usize {
        round_slots(self.g(G_ROLLOUT), MIN_ROLLOUT, MAX_ROLLOUT)
    }

    /// 基因向量长度/数值规范化；结构基因来自存档时四舍五入。
    fn normalize_genes(&mut self) {
        if self.genes.len() != GENE_COUNT {
            let mut base = Self::reference_genes("herb");
            for i in 0..self.genes.len().min(GENE_COUNT) {
                base[i] = self.genes[i];
            }
            self.genes = base;
        }
        for i in 0..GENE_COUNT {
            let (lo, hi, _) = gene_bounds(i);
            self.genes[i] = finite(self.genes[i], 0.0).clamp(lo, hi);
        }
        self.genes[G_WM_SLOTS] = self.wm_slots() as f64;
        self.genes[G_MEMORY_SLOTS] = self.memory_capacity() as f64;
        self.genes[G_ROLLOUT] = self.rollout_depth() as f64;
    }

    /// 工作记忆/情绪记忆容量与基因对齐（增删槽位）。
    pub fn sync_structure(&mut self) {
        let k = self.wm_slots();
        self.wm.resize(k, 0.0);
        for v in self.wm.iter_mut() {
            if !v.is_finite() { *v = 0.0; }
            *v = v.clamp(-1.0, 1.0);
        }
        let m = self.memory_capacity();
        self.memory_keys.resize(m, [0.0; MEMORY_KEY_DIM]);
        self.memory_valence.resize(m, 0.0);
        self.memory_strength.resize(m, 0.0);
        self.memory_age.resize(m, 0);
        for k in self.memory_keys.iter_mut() {
            for v in k.iter_mut() { if !v.is_finite() { *v = 0.0; } }
        }
        for v in self.memory_valence.iter_mut() {
            *v = finite(*v, 0.0).clamp(-1.0, 1.0);
        }
        for v in self.memory_strength.iter_mut() {
            *v = finite(*v, 0.0).clamp(0.0, 1.0);
        }
        for w in self.world.iter_mut() { *w = finite(*w, 0.0).clamp(-8.0, 8.0); }
    }

    /// 从存档恢复后的结构一致性：若保存的向量长度合法，以保存值为准
    /// （否则基因与向量长度矛盾会破坏位精确恢复）。
    pub fn normalize_saved(&mut self) {
        self.normalize_genes();
        if self.wm.len() >= MIN_WM_SLOTS && self.wm.len() <= MAX_WM_SLOTS {
            self.genes[G_WM_SLOTS] = self.wm.len() as f64;
        }
        if self.memory_keys.len() >= MIN_MEMORY_SLOTS
            && self.memory_keys.len() <= MAX_MEMORY_SLOTS
            && self.memory_valence.len() == self.memory_keys.len()
            && self.memory_strength.len() == self.memory_keys.len()
            && self.memory_age.len() == self.memory_keys.len()
        {
            self.genes[G_MEMORY_SLOTS] = self.memory_keys.len() as f64;
        } else {
            self.sync_structure();
        }
        self.sync_replay();
        self.sync_structure();
    }

    /// Phase 42: 回放缓冲的结构/数值规范化（v12 存档无此字段 → 空缓冲）。
    fn sync_replay(&mut self) {
        if self.replay.len() > REPLAY_CAP {
            self.replay.truncate(REPLAY_CAP);
        }
        for e in self.replay.iter_mut() {
            for v in e.sense.iter_mut().chain(e.next.iter_mut()) {
                *v = finite(*v, 0.0).clamp(-4.0, 4.0);
            }
            e.action %= OUTPUT_ACTIONS as u8;
            e.reward = finite(e.reward, 0.0).clamp(-4.0, 4.0);
        }
        self.replay_len = self.replay_len.min(self.replay.len());
        self.replay_head = if self.replay.is_empty() { 0 } else { self.replay_head % self.replay.len() };
        self.replay_cursor = if self.replay_len == 0 { 0 } else { self.replay_cursor % self.replay_len };
    }

    // ----- 成本 -----------------------------------------------------------

    pub fn layer_units(&self) -> (usize, usize, usize) {
        let instinct = INSTINCT_UNITS;
        let emotion = EMOTION_DIM + self.memory_capacity() + NEUROMOD_DIM;
        let reason = REASON_CORTEX_UNITS + self.wm_slots() + self.rollout_depth() * OUTPUT_ACTIONS;
        (instinct, emotion, reason)
    }

    /// 维护费（每秒）。
    pub fn upkeep(&self) -> f64 {
        let (i, e, r) = self.layer_units();
        TRIUNE_PER_UNIT_COST * (i + e + r) as f64
    }

    /// 决策费（每秒，未乘 dt）。理智脑的 rollout 深度与抑制闸门决定边际成本。
    pub fn decision_rate(&self) -> f64 {
        TRIUNE_PER_ROLLOUT_COST * self.rollout_depth() as f64
            + self.g(G_INHIBIT_COST) * self.last.inhibit_gate
    }

    // ----- 世界模型 -------------------------------------------------------

    /// 预测 `sense` 下执行 `action` 的 next state 与期望奖励。
    pub fn world_predict(&self, sense: &[f64; INPUT_CHANNELS], action: usize) -> ([f64; INPUT_CHANNELS], f64) {
        let a = action % OUTPUT_ACTIONS;
        let mut next = [0.0f64; INPUT_CHANNELS];
        for i in 0..INPUT_CHANNELS {
            let mut acc = self.world[world_bias_index(a, i)];
            for j in 0..INPUT_CHANNELS {
                acc += self.world[world_index(a, i, j)] * sense[j];
            }
            next[i] = libm_tanh(acc);
        }
        let mut r = self.world[world_reward_b_index(a)];
        for j in 0..INPUT_CHANNELS {
            r += self.world[world_reward_w_index(a, j)] * sense[j];
        }
        (next, finite(r, 0.0).clamp(-4.0, 4.0))
    }

    /// 单条经验更新世界模型（delta rule，受 dopamine 增益调制）。零 RNG。
    pub fn observe(&mut self, prev: &[f64; INPUT_CHANNELS], action: usize, next: &[f64; INPUT_CHANNELS], reward: f64) {
        self.observe_impl(prev, action, next, reward, true);
    }

    /// `observe` 的实现体。`learn_meta=false` 是 Phase 42 的离线回放路径：
    /// 只练世界模型权重，不触碰 RPE / 置信度 / 校准——离线回放是模型巩固，
    /// 不应驱动神经调质（dopamine 必须反映在线奖励预测误差）。
    fn observe_impl(
        &mut self,
        prev: &[f64; INPUT_CHANNELS],
        action: usize,
        next: &[f64; INPUT_CHANNELS],
        reward: f64,
        learn_meta: bool,
    ) {
        let a = action % OUTPUT_ACTIONS;
        let (pred, rpred) = self.world_predict(prev, a);
        let mut err = [0.0f64; INPUT_CHANNELS];
        let mut abs_sum = 0.0;
        for i in 0..INPUT_CHANNELS {
            err[i] = (next[i] - pred[i]).clamp(-2.0, 2.0);
            abs_sum += err[i].abs();
        }
        let mean_err = abs_sum / INPUT_CHANNELS as f64;
        let rpe = reward - rpred;
        if learn_meta {
            let tau = self.g(G_CONFIDENCE_TAU).max(0.1);
            let conf = 1.0 / (1.0 + tau * mean_err);
            self.confidence = clamp01(0.9 * self.confidence + 0.1 * conf);
            self.pred_error_ema = clamp01(0.9 * self.pred_error_ema + 0.1 * mean_err.min(1.0));
            let accurate = if argmax_state(next) == argmax_state(&pred) { 1.0 } else { 0.0 };
            self.calibration = clamp01(
                0.9 * self.calibration + 0.1 * (1.0 - (self.confidence - accurate).abs()),
            );
            self.last_rpe = finite(rpe, 0.0).clamp(-8.0, 8.0);
            self.prev_reward_pred = rpred;
        }

        let da = self.dopamine.max(0.0);
        let lr = self.g(G_WORLD_LR) * (1.0 + self.g(G_DA_SENS) * da);
        for i in 0..INPUT_CHANNELS {
            let bi = world_bias_index(a, i);
            self.world[bi] = (self.world[bi] + lr * err[i]).clamp(-8.0, 8.0);
            for j in 0..INPUT_CHANNELS {
                let idx = world_index(a, i, j);
                self.world[idx] = (self.world[idx] + lr * err[i] * prev[j]).clamp(-8.0, 8.0);
            }
        }
        let rlr = self.g(G_REWARD_LR) * (1.0 + self.g(G_DA_SENS) * da);
        for j in 0..INPUT_CHANNELS {
            let idx = world_reward_w_index(a, j);
            self.world[idx] = (self.world[idx] + rlr * rpe * prev[j]).clamp(-8.0, 8.0);
        }
        let bi = world_reward_b_index(a);
        self.world[bi] = (self.world[bi] + rlr * rpe).clamp(-8.0, 8.0);
    }

    /// 把一条经验写入回放环形缓冲（超出容量时覆盖最旧）。容量固定，零 RNG。
    /// 公开入口供离线巩固实验直接喂样本；在线路径由 `learn_transition` 调用。
    pub fn record(&mut self, sense: &[f64; INPUT_CHANNELS], action: usize, next: &[f64; INPUT_CHANNELS], reward: f64) {
        self.push_replay(sense, action, next, reward);
    }

    fn push_replay(&mut self, sense: &[f64; INPUT_CHANNELS], action: usize, next: &[f64; INPUT_CHANNELS], reward: f64) {
        if self.replay.len() < REPLAY_CAP {
            self.replay.push(ReplayEntry::default());
        }
        if self.replay.is_empty() { return; }
        let cap = self.replay.len();
        let slot = self.replay_head % cap;
        let e = &mut self.replay[slot];
        e.sense = *sense;
        e.action = (action % OUTPUT_ACTIONS) as u8;
        e.next = *next;
        e.reward = finite(reward, 0.0).clamp(-4.0, 4.0);
        self.replay_head = (slot + 1) % cap;
        self.replay_len = (self.replay_len + 1).min(self.replay.len());
    }

    /// Phase 42: 离线回放一步（海马尖波涟漪重放）：按确定性游标轮转抽取
    /// 一条历史经验做 delta-rule 世界模型更新。零 RNG、不驱动神经调质。
    /// 返回本次回放前的预测误差（L1 均值），供 A/B 遥测。
    pub fn replay_step(&mut self) -> Option<f64> {
        let n = self.replay_len.min(self.replay.len());
        if n == 0 { return None; }
        let idx = self.replay_cursor % n;
        let e = self.replay[idx].clone();
        let before = self.entry_error(&e);
        self.replay_cursor = (idx + 1) % n;
        self.observe_impl(&e.sense, e.action as usize, &e.next, e.reward, false);
        self.stats.replay_updates += 1;
        Some(before)
    }

    #[inline]
    fn entry_error(&self, e: &ReplayEntry) -> f64 {
        let (pred, _) = self.world_predict(&e.sense, e.action as usize);
        let mut s = 0.0;
        for i in 0..INPUT_CHANNELS {
            s += (e.next[i] - pred[i]).abs();
        }
        s / INPUT_CHANNELS as f64
    }

    /// Phase 42: 回放缓冲上的平均预测误差（纯读，不学习）——A/B 指标。
    /// 这是模型对「亲身经历过的转移」的拟合质量：回放给稀有动作更多
    /// 巩固机会，因此有回放的模型应该在同一个缓冲上误差更低。
    pub fn probe_error(&self) -> f64 {
        let n = self.replay_len.min(self.replay.len());
        if n == 0 { return 0.0; }
        let mut sum = 0.0;
        for e in self.replay.iter().take(n) {
            sum += self.entry_error(e);
        }
        sum / n as f64
    }

    /// 单一动作在回放缓冲上的误差（缓冲里没有该动作时返回 None）。
    pub fn probe_error_for(&self, action: usize) -> Option<f64> {
        let a = (action % OUTPUT_ACTIONS) as u8;
        let mut sum = 0.0;
        let mut n = 0usize;
        for e in self.replay.iter().take(self.replay_len.min(self.replay.len())) {
            if e.action == a {
                sum += self.entry_error(e);
                n += 1;
            }
        }
        if n == 0 { None } else { Some(sum / n as f64) }
    }

    fn learn_transition(&mut self, sense: &[f64; INPUT_CHANNELS], reward: f64) {
        if !self.last_seen { return; }
        let prev = self.last_sense;
        let a = self.last_executed as usize % OUTPUT_ACTIONS;
        self.observe(&prev, a, sense, reward);
        // Phase 42: 同一条在线转移写入回放缓冲（供后续脑节律 tick 离线巩固）。
        self.push_replay(&prev, a, sense, reward);
    }

    // ----- 情绪脑 ---------------------------------------------------------

    fn memory_key(sense: &[f64; INPUT_CHANNELS]) -> [f64; MEMORY_KEY_DIM] {
        [
            clamp01(sense[0]),
            clamp01(sense[2].max(sense[3])),
            clamp01(sense[4]),
            clamp01(sense[5]),
        ]
    }

    fn similarity(a: &[f64; MEMORY_KEY_DIM], b: &[f64; MEMORY_KEY_DIM]) -> f64 {
        let mut d = 0.0;
        for i in 0..MEMORY_KEY_DIM {
            d += (a[i] - b[i]).abs();
        }
        (1.0 - d / MEMORY_KEY_DIM as f64).clamp(0.0, 1.0)
    }

    /// 情绪记忆提取：相似度 × 强度加权的 valence 期望。
    pub fn retrieve_valence(&self, sense: &[f64; INPUT_CHANNELS]) -> f64 {
        let key = Self::memory_key(sense);
        let mut num = 0.0;
        let mut den = 0.0;
        for i in 0..self.memory_keys.len() {
            if self.memory_strength[i] <= 0.0 { continue; }
            let sim = Self::similarity(&key, &self.memory_keys[i]);
            if sim > 0.3 {
                let w = sim * self.memory_strength[i];
                num += w * self.memory_valence[i];
                den += w;
            }
        }
        if den > 0.0 { (num / den).clamp(-1.0, 1.0) } else { 0.0 }
    }

    /// 情绪记忆关联学习（杏仁核-海马式快速绑定）。零 RNG。
    fn update_memory(&mut self, sense: &[f64; INPUT_CHANNELS], reward: f64, _dt: f64) -> (bool, bool) {
        let key = Self::memory_key(sense);
        let eta = self.g(G_MEMORY_ETA)
            * (1.0 + self.g(G_DA_SENS) * self.dopamine.max(0.0));
        let mut best = usize::MAX;
        let mut best_sim = 0.0;
        let mut weakest = 0usize;
        let mut weakest_score = f64::INFINITY;
        for i in 0..self.memory_keys.len() {
            let sim = Self::similarity(&key, &self.memory_keys[i]);
            if sim > best_sim {
                best_sim = sim;
                best = i;
            }
            let score = self.memory_strength[i] - (self.tick_count.saturating_sub(self.memory_age[i]) as f64) * 1e-6;
            if score < weakest_score {
                weakest_score = score;
                weakest = i;
            }
        }
        let mut conditioned = false;
        let threat = clamp01(sense[0]);
        let slot = if best_sim >= 0.6 && best != usize::MAX {
            best
        } else {
            // 空槽优先，否则替换最弱（强度 + 久未触及）。
            let mut empty = usize::MAX;
            for i in 0..self.memory_keys.len() {
                if self.memory_strength[i] <= 0.0 {
                    empty = i;
                    break;
                }
            }
            let s = if empty != usize::MAX { empty } else { weakest };
            self.memory_keys[s] = key;
            self.memory_valence[s] = 0.0;
            self.memory_strength[s] = eta.min(0.5);
            self.memory_age[s] = self.tick_count;
            s
        };
        if threat > 0.5 && reward < 0.0 {
            let v = self.memory_valence[slot];
            self.memory_valence[slot] = (v + eta * (-1.0 - v)).clamp(-1.0, 1.0);
            self.memory_strength[slot] = (self.memory_strength[slot] + eta * threat).min(1.0);
            conditioned = true;
            self.stats.fear_conditioning += 1;
        } else {
            let v = self.memory_valence[slot];
            let target = reward.clamp(-1.0, 1.0);
            self.memory_valence[slot] = (v + eta * (target - v)).clamp(-1.0, 1.0);
            self.memory_strength[slot] = (self.memory_strength[slot] + eta * 0.5).min(1.0);
        }
        self.memory_age[slot] = self.tick_count;
        let retrieved = best_sim >= 0.6;
        if retrieved {
            self.stats.memory_retrievals += 1;
        }
        (conditioned, retrieved)
    }

    /// 情绪变量积分 + 神经调质更新。
    fn update_emotion(&mut self, sense: &[f64; INPUT_CHANNELS], dt: f64) {
        let threat = clamp01(sense[0]);
        let hunger = clamp01(sense[1]);
        let food = clamp01(sense[2].max(sense[3]));
        let social = clamp01(sense[4]);
        let mut novelty = 0.0;
        if self.last_seen {
            for j in 0..INPUT_CHANNELS {
                novelty += (sense[j] - self.last_sense[j]).abs();
            }
            novelty /= INPUT_CHANNELS as f64;
        }
        let ser_stab = 1.0 + self.g(G_SER_GAIN) * self.serotonin * 0.5;
        self.fear += dt * (
            self.g(G_FEAR_RISE) * threat * (1.0 - self.fear)
                - self.g(G_FEAR_DECAY) * self.fear * ser_stab
        );
        self.desire += dt * (
            self.g(G_DESIRE_RISE) * hunger * (0.3 + food) * (1.0 - self.desire)
                - self.g(G_DESIRE_DECAY) * self.desire
        );
        self.bond += dt * (
            self.g(G_BOND_RISE) * social * (1.0 - self.bond)
                - self.g(G_BOND_DECAY) * self.bond
        );
        self.arousal += dt * (
            self.g(G_AROUSAL_GAIN) * (threat * 0.7 + hunger * 0.3 + novelty * 0.5) * (1.0 - self.arousal)
                - self.g(G_AROUSAL_DECAY) * self.arousal
        );
        let recalled = self.retrieve_valence(sense);
        if recalled < 0.0 {
            self.fear += dt * self.g(G_MEMORY_FEAR_GAIN) * (-recalled) * threat;
        }
        self.fear = clamp01(self.fear);
        self.desire = clamp01(self.desire);
        self.bond = clamp01(self.bond);
        self.arousal = clamp01(self.arousal);

        let da = self.g(G_DA_SENS) * self.last_rpe;
        self.dopamine = (0.9 * self.dopamine + 0.1 * libm_tanh(da)).clamp(-1.0, 1.0);
        self.norepinephrine = (self.g(G_NE_SENS) * self.arousal).clamp(0.0, 2.0);
        let ser_target = (self.g(G_SER_SENS) * (1.0 - self.fear)).clamp(0.0, 2.0);
        self.serotonin = (0.9 * self.serotonin + 0.1 * ser_target).clamp(0.0, 2.0);
    }

    // ----- 三层前向 + 仲裁 ------------------------------------------------

    pub fn instinct_logits(&self, sense: &[f64; INPUT_CHANNELS]) -> [f64; OUTPUT_ACTIONS] {
        let mut out = [0.0f64; OUTPUT_ACTIONS];
        let mut max_raw = -f64::INFINITY;
        for a in 0..OUTPUT_ACTIONS {
            let mut acc = self.g(G_INSTINCT_B + a);
            for j in 0..INPUT_CHANNELS {
                acc += self.g(G_INSTINCT_W + a * INPUT_CHANNELS + j) * sense[j];
            }
            out[a] = acc;
            if acc > max_raw { max_raw = acc; }
        }
        // FAP 阈值：不是 IF-THEN，而是一个基因控制的反射弧发放门。
        let fap = libm_sigmoid(4.0 * (max_raw - self.g(G_INSTINCT_THRESHOLD)));
        let gain = self.g(G_INSTINCT_GAIN) * (0.2 + 0.8 * fap)
            * (1.0 + 0.3 * self.norepinephrine);
        for a in 0..OUTPUT_ACTIONS {
            out[a] = finite(out[a] * gain, 0.0);
        }
        out
    }

    pub fn emotion_logits(&self, sense: &[f64; INPUT_CHANNELS]) -> [f64; OUTPUT_ACTIONS] {
        let emo = [self.fear, self.desire, self.bond, self.arousal];
        let recalled = self.retrieve_valence(sense);
        let mut out = [0.0f64; OUTPUT_ACTIONS];
        for a in 0..OUTPUT_ACTIONS {
            let mut acc = 0.0;
            for e in 0..EMOTION_DIM {
                acc += emo[e] * self.g(G_EMOTION_W + e * OUTPUT_ACTIONS + a);
            }
            acc += recalled * self.g(G_MEMORY_W + a);
            out[a] = finite(acc * (1.0 + 0.3 * self.norepinephrine), 0.0);
        }
        out
    }

    fn plan_state(&self, sense: &[f64; INPUT_CHANNELS]) -> [f64; INPUT_CHANNELS] {
        let k = self.wm.len().max(1);
        let mut s = [0.0f64; INPUT_CHANNELS];
        for j in 0..INPUT_CHANNELS {
            s[j] = (sense[j] + 0.15 * libm_tanh(self.wm[j % k])).clamp(-2.0, 2.0);
        }
        s
    }

    /// 工作记忆更新（持久循环槽位）。返回写入的槽位是否发生变化。
    fn update_wm(&mut self, sense: &[f64; INPUT_CHANNELS]) {
        let k = self.wm.len();
        if k == 0 { return; }
        let write = self.g(G_WM_WRITE);
        let decay = self.g(G_WM_DECAY);
        let gain = self.g(G_WM_GAIN);
        for i in 0..k {
            let mut input = self.g(G_WM_BIAS + i % MAX_WM_SLOTS);
            for j in 0..INPUT_CHANNELS {
                input += self.g(G_WM_IN + j) * sense[j];
            }
            let update = libm_tanh(gain * input);
            let v = libm_tanh(decay * self.wm[i] + write * update);
            self.wm[i] = finite(v, 0.0).clamp(-1.0, 1.0);
        }
    }

    /// 模型式 rollout：对每个动作用世界模型向前推 `rollout_depth` 步，
    /// 累加贴现奖励。
    fn reason_logits(&mut self, sense: &[f64; INPUT_CHANNELS]) -> [f64; OUTPUT_ACTIONS] {
        self.update_wm(sense);
        let depth = self.rollout_depth();
        let start = self.plan_state(sense);
        let discount_gene = self.g(G_DISCOUNT);
        let gain = self.g(G_PLAN_GAIN);
        let mut out = [0.0f64; OUTPUT_ACTIONS];
        for a in 0..OUTPUT_ACTIONS {
            let mut s = start;
            let mut value = 0.0;
            let mut discount = 1.0;
            for _ in 0..depth {
                let (next, r) = self.world_predict(&s, a);
                value += discount * r;
                discount *= discount_gene;
                s = next;
            }
            out[a] = finite(gain * value, 0.0).clamp(-32.0, 32.0);
        }
        out
    }

    /// 单步决策（更新情绪、工作记忆与理智规划，不学习世界模型）。
    pub fn forward(&mut self, sense: &[f64], dt: f64) -> Decision {
        let mut s = [0.0f64; INPUT_CHANNELS];
        for j in 0..INPUT_CHANNELS {
            s[j] = finite(sense.get(j).copied().unwrap_or(0.0), 0.0).clamp(-4.0, 4.0);
        }
        self.update_emotion(&s, dt.max(1e-6));
        self.decide(&s)
    }

    fn decide(&mut self, sense: &[f64; INPUT_CHANNELS]) -> Decision {
        let instinct = self.instinct_logits(sense);
        let emotion = self.emotion_logits(sense);
        let reason = self.reason_logits(sense);
        let w_i = self.g(G_FUSION_INSTINCT);
        let w_e = self.g(G_FUSION_EMOTION);
        let conf_floor = self.g(G_REASON_CONF_FLOOR).clamp(0.0, 1.0);
        let w_r = self.g(G_FUSION_REASON) * (conf_floor + (1.0 - conf_floor) * self.confidence);

        let mut fused = [0.0f64; OUTPUT_ACTIONS];
        for a in 0..OUTPUT_ACTIONS {
            fused[a] = w_i * instinct[a] + w_e * emotion[a];
        }
        let urge = argmax7(&fused);
        let plan = argmax7(&reason);
        let margin = reason[plan] - reason[urge];
        let mut gate = libm_sigmoid(
            self.g(G_INHIBIT_GAIN) * margin - self.g(G_INHIBIT_THRESHOLD),
        );
        if margin <= 0.0 { gate = 0.0; }
        gate = clamp01(gate);

        let mut final_logits = fused;
        for a in 0..OUTPUT_ACTIONS {
            final_logits[a] += w_r * reason[a];
        }
        if gate > 0.0 {
            let strength = self.g(G_INHIBIT_STRENGTH) * gate;
            final_logits[urge] -= strength * (1.0 + fused[urge].max(0.0));
            final_logits[plan] += strength * (1.0 + reason[plan].max(0.0));
        }
        let action = argmax7(&final_logits);
        let d = Decision {
            action: action as u8,
            instinct_action: argmax7(&instinct) as u8,
            emotion_action: argmax7(&emotion) as u8,
            plan_action: plan as u8,
            urge_action: urge as u8,
            inhibit_gate: gate,
            confidence: clamp01(self.confidence),
            expected_value: reason[plan],
            fused: final_logits,
            reason,
        };
        self.last = d.clone();
        d
    }

    /// 完整 tick：先用上一 tick 的经验学习世界模型，再更新情绪记忆与三脑
    /// 状态，最后仲裁出动作。`executed` 是本 tick 实际执行的动作（影子模式
    /// 下是 FSM 动作），供下一 tick 的 transition 学习使用。
    pub fn tick(
        &mut self,
        sense: &[f64],
        reward: f64,
        dt: f64,
        executed: u8,
    ) -> TickOutcome {
        self.tick_count = self.tick_count.wrapping_add(1);
        let mut s = [0.0f64; INPUT_CHANNELS];
        for j in 0..INPUT_CHANNELS {
            s[j] = finite(sense.get(j).copied().unwrap_or(0.0), 0.0).clamp(-4.0, 4.0);
        }
        let prediction_updated = self.last_seen;
        if prediction_updated {
            self.learn_transition(&s, reward);
        }
        let (conditioned, memory_retrieved) = self.update_memory(&s, reward, dt.max(1e-6));
        let d = self.forward(&s, dt);
        let depth = self.rollout_depth();
        let fear_event = self.fear > 0.6 && self.prev_fear <= 0.6;
        let planned_override = d.action != d.urge_action && d.action == d.plan_action;
        let inhibition = d.inhibit_gate > 0.5 && d.action != d.urge_action && d.action == d.plan_action;

        self.stats.ticks += 1;
        self.stats.fear_events += fear_event as u64;
        self.stats.wm_updates += self.wm.len() as u64;
        self.stats.rollout_steps += (OUTPUT_ACTIONS * depth) as u64;
        self.stats.planned_overrides += planned_override as u64;
        self.stats.inhibitions += inhibition as u64;
        self.stats.prediction_updates += prediction_updated as u64;
        self.stats.high_confidence_ticks += (self.confidence > 0.7) as u64;
        self.stats.gate_open_ticks += (d.inhibit_gate > GATE_OPEN) as u64;
        self.stats.gate_sum += d.inhibit_gate;
        self.stats.max_fear = self.stats.max_fear.max(self.fear);
        self.stats.fear_sum += self.fear;
        self.stats.confidence_sum += self.confidence;
        self.stats.pred_error_sum += self.pred_error_ema;
        self.stats.action_hist[d.action as usize] += 1;
        self.prev_fear = self.fear;
        self.last_sense = s;
        self.last_seen = true;
        self.last_executed = executed % OUTPUT_ACTIONS as u8;

        let decision_rate = TRIUNE_PER_ROLLOUT_COST * depth as f64
            + self.g(G_INHIBIT_COST) * d.inhibit_gate;
        TickOutcome {
            decision: d,
            fear: self.fear,
            confidence: self.confidence,
            pred_error: self.pred_error_ema,
            fear_event,
            fear_conditioning: conditioned,
            memory_retrieved,
            planned_override,
            inhibition,
            rollout_steps: (OUTPUT_ACTIONS * depth) as u32,
            wm_updates: self.wm.len() as u32,
            prediction_updated,
            decision_rate,
        }
    }

    // ----- 可演化性 -------------------------------------------------------

    /// 出生突变：连续基因扰动 + 低概率结构漂移（WM/记忆槽位、rollout 深度）
    /// + 世界模型权重的表型漂移。消耗调用方的专用 RNG 流。
    pub fn mutate(&mut self, rng: &mut aether_rng::Mulberry32, sigma: f64) {
        for i in 0..GENE_COUNT {
            let (lo, hi, s) = gene_bounds(i);
            if s == 0.0 { continue; }
            let step = (rng.next_f64() - 0.5) * 2.0 * sigma * s;
            self.genes[i] = (self.genes[i] + step).clamp(lo, hi);
        }
        for &(idx, lo, hi) in &[
            (G_WM_SLOTS, MIN_WM_SLOTS, MAX_WM_SLOTS),
            (G_MEMORY_SLOTS, MIN_MEMORY_SLOTS, MAX_MEMORY_SLOTS),
            (G_ROLLOUT, MIN_ROLLOUT, MAX_ROLLOUT),
        ] {
            if rng.next_f64() < 0.05 {
                let cur = self.genes[idx].round();
                let delta = if rng.next_f64() < 0.5 { -1.0 } else { 1.0 };
                self.genes[idx] = (cur + delta).clamp(lo as f64, hi as f64);
            }
        }
        for w in self.world.iter_mut() {
            *w = (*w + (rng.next_f64() - 0.5) * sigma * 0.1).clamp(-8.0, 8.0);
        }
        self.normalize_genes();
        self.sync_structure();
    }

    /// 出生时清空瞬时情绪/调质状态（情绪记忆与世界模型作为亲代遗产保留）。
    /// Phase 42: 回放缓冲是**个体情节记忆**，出生时清空——继承世界模型
    /// （先天先验）但不继承亲代的最后 12 条经历（那些不是这个身体见过的）。
    pub fn reset_transients(&mut self) {
        self.fear = 0.0;
        self.desire = 0.0;
        self.bond = 0.0;
        self.arousal = 0.0;
        self.dopamine = 0.0;
        self.norepinephrine = 0.0;
        self.serotonin = 0.0;
        self.prev_fear = 0.0;
        self.last_rpe = 0.0;
        self.confidence = 0.5;
        self.pred_error_ema = 0.5;
        self.calibration = 0.5;
        self.last_seen = false;
        self.last_sense = [0.0; INPUT_CHANNELS];
        self.last_executed = 0;
        self.last = Decision::default();
        self.replay.clear();
        self.replay_head = 0;
        self.replay_len = 0;
        self.replay_cursor = 0;
        self.stats = TriuneStats::default();
    }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sense(vals: [f64; INPUT_CHANNELS]) -> [f64; INPUT_CHANNELS] { vals }

    #[test]
    fn instinct_reflex_without_learning() {
        let b = TriuneBrain::reference("herb", 7);
        let out = b.instinct_logits(&sense([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 0.5]));
        assert_eq!(argmax7(&out), 4, "threat reflex should favour flee: {:?}", out);
        // 无学习：连续两次相同输入输出完全一致
        let out2 = b.instinct_logits(&sense([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 0.5]));
        assert_eq!(out, out2);
    }

    #[test]
    fn bold_and_timid_genotypes_diverge() {
        let s = sense([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5]);
        let timid = {
            let mut b = TriuneBrain::reference("herb", 1);
            b.genes[G_INSTINCT_THRESHOLD] = -0.8;
            b.genes[G_INSTINCT_W + 4 * INPUT_CHANNELS] = 3.0; // threat → flee
            b
        };
        let bold = {
            let mut b = TriuneBrain::reference("herb", 1);
            b.genes[G_INSTINCT_THRESHOLD] = 0.9; // FAP 难触发
            b.genes[G_INSTINCT_W + 4 * INPUT_CHANNELS] = 0.4;
            b.genes[G_INSTINCT_B + 0] = 1.2; // forward 偏置高
            b
        };
        let timid_flee = timid.instinct_logits(&s)[4];
        let bold_flee = bold.instinct_logits(&s)[4];
        assert!(timid_flee > bold_flee, "timid flee {} should exceed bold {}", timid_flee, bold_flee);
        assert_eq!(argmax7(&timid.instinct_logits(&s)), 4);
        assert_ne!(argmax7(&bold.instinct_logits(&s)), 4);
    }

    #[test]
    fn fear_accumulates_on_threat_and_decays() {
        let mut b = TriuneBrain::reference("herb", 3);
        let threat = sense([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5]);
        let safe = sense([0.0, 0.0, 0.3, 0.0, 0.0, 0.0, 0.5, 0.5]);
        for _ in 0..40 {
            b.tick(&threat, -0.1, 0.1, 4);
        }
        let fear_high = b.fear;
        assert!(fear_high > 0.5, "fear should accumulate, got {}", fear_high);
        for _ in 0..80 {
            b.tick(&safe, 0.05, 0.1, 0);
        }
        assert!(b.fear < fear_high * 0.5, "fear should decay: {} -> {}", fear_high, b.fear);
        assert!(b.stats.fear_events > 0);
    }

    #[test]
    fn emotional_memory_conditioning_and_recall() {
        let mut b = TriuneBrain::reference("herb", 5);
        let threat = sense([1.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.5, 0.5]);
        for _ in 0..60 {
            b.tick(&threat, -1.0, 0.1, 4);
        }
        assert!(b.stats.fear_conditioning > 0);
        let recalled = b.retrieve_valence(&threat);
        assert!(recalled < -0.2, "threat context should recall negative valence, got {}", recalled);
        // 安全/富食情境获得正价
        let food = sense([0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.5, 0.5]);
        for _ in 0..60 {
            b.tick(&food, 0.8, 0.1, 3);
        }
        assert!(b.retrieve_valence(&food) > 0.2, "food context should acquire positive valence");
    }

    #[test]
    fn working_memory_updates_and_persists() {
        let mut b = TriuneBrain::reference("herb", 9);
        let a = sense([0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.5, 0.5]);
        let c = sense([1.0, 0.0, 0.0, 1.0, 1.0, 0.8, 0.1, 0.9]);
        b.forward(&a, 0.1);
        let wm_a = b.wm.clone();
        b.forward(&c, 0.1);
        assert_ne!(wm_a, b.wm, "working memory must change with input");
        let after = b.wm.clone();
        // 无输入时槽位保留衰减后的痕迹（不是瞬间清零）
        b.forward(&[0.0; INPUT_CHANNELS], 0.1);
        assert!(b.wm.iter().zip(after.iter()).any(|(x, y)| (x - y).abs() > 1e-6));
        assert!(b.wm.iter().all(|v| v.is_finite() && v.abs() <= 1.0));
    }

    #[test]
    fn planner_needs_multistep_rollout() {
        // 构造一个需要 3 步才能拿到奖励的世界：
        // forward 每步 0.1 即时奖励；eat 需要 A→B→C 三步才 +1。
        let a = sense([0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.5, 0.5]);
        let bstate = sense([0.0, 0.8, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5]);
        let cstate = sense([0.0, 0.6, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5]);

        let mut shallow = TriuneBrain::reference("herb", 11);
        shallow.genes[G_ROLLOUT] = 2.0;
        shallow.genes[G_WORLD_LR] = 0.5;
        shallow.genes[G_REWARD_LR] = 0.5;
        let mut deep = shallow.clone();
        deep.genes[G_ROLLOUT] = 3.0;
        for br in [&mut shallow, &mut deep] {
            for _ in 0..200 {
                br.observe(&a, 0, &a, 0.1);
                br.observe(&a, 3, &bstate, 0.0);
                br.observe(&bstate, 3, &cstate, 0.0);
                br.observe(&cstate, 3, &cstate, 1.0);
            }
            br.tick(&a, 0.0, 0.1, 0);
        }
        assert_eq!(shallow.last.plan_action, 0, "depth-2 planner should take the immediate small reward: {:?}", shallow.last.reason);
        assert_eq!(deep.last.plan_action, 3, "depth-3 planner should see the delayed reward: {:?}", deep.last.reason);
    }

    #[test]
    fn inhibition_overrides_fear_urge() {
        let threat = sense([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5]);
        // 威胁情境下：flee 负回报，eat 正回报（理智知道该留下取食）。
        let make = |inhibit_strength: f64| {
            let mut b = TriuneBrain::reference("herb", 13);
            b.genes[G_FEAR_RISE] = 4.0;
            b.genes[G_FEAR_DECAY] = 0.05;
            b.genes[G_WORLD_LR] = 0.5;
            b.genes[G_REWARD_LR] = 0.5;
            b.genes[G_INHIBIT_GAIN] = 5.0;
            b.genes[G_INHIBIT_STRENGTH] = inhibit_strength;
            for _ in 0..300 {
                b.observe(&threat, 4, &threat, -1.0);
                b.observe(&threat, 3, &threat, 0.8);
            }
            b.fear = 1.0;
            b.serotonin = 0.0;
            b
        };
        let mut weak = make(0.0);
        let mut strong = make(4.0);
        let dw = weak.forward(&threat, 0.1);
        let ds = strong.forward(&threat, 0.1);
        assert_eq!(dw.urge_action, 4, "high fear should urge flee");
        assert_eq!(dw.action, 4, "without inhibition the urge wins");
        assert_eq!(ds.plan_action, 3, "reason should plan to stay and eat");
        assert_eq!(ds.action, 3, "inhibition must override the fear urge");
        assert!(ds.inhibit_gate > 0.5, "gate should open, got {}", ds.inhibit_gate);
    }

    #[test]
    fn confidence_tracks_prediction_error() {
        let mut low = TriuneBrain::reference("herb", 17);
        let mut high = TriuneBrain::reference("herb", 17);
        low.genes[G_CONFIDENCE_TAU] = 40.0;
        high.genes[G_CONFIDENCE_TAU] = 40.0;
        let x = sense([0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.5, 0.5]);
        let y = sense([1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.1, 0.9]);
        for _ in 0..200 {
            low.observe(&x, 0, &x, 0.5); // 一致转移 → 低误差
            high.observe(&x, 0, &y, 0.5); // 矛盾转移 → 高误差
        }
        assert!(low.pred_error_ema < high.pred_error_ema);
        assert!(low.confidence > high.confidence, "confidence {} should exceed {}", low.confidence, high.confidence);
    }

    #[test]
    fn cost_scales_with_layer_size() {
        let mut small = TriuneBrain::reference("herb", 19);
        small.genes[G_WM_SLOTS] = 2.0;
        small.genes[G_MEMORY_SLOTS] = 2.0;
        small.genes[G_ROLLOUT] = 2.0;
        small.normalize_genes();
        small.sync_structure();
        let mut large = small.clone();
        large.genes[G_WM_SLOTS] = 6.0;
        large.genes[G_MEMORY_SLOTS] = 8.0;
        large.genes[G_ROLLOUT] = 4.0;
        large.normalize_genes();
        large.sync_structure();
        let (si, se, sr) = small.layer_units();
        let (li, le, lr) = large.layer_units();
        assert_eq!(si, li);
        assert!(le > se && lr > sr);
        assert!(large.upkeep() > small.upkeep());
        small.last.inhibit_gate = 1.0;
        large.last.inhibit_gate = 1.0;
        assert!(large.decision_rate() > small.decision_rate());
    }

    #[test]
    fn long_run_stays_finite_and_deterministic() {
        let mut a = TriuneBrain::reference("pred", 23);
        let mut b = a.clone();
        let mut fear_hits = 0;
        for t in 0..400 {
            let phase = (t % 37) as f64 / 37.0;
            let s = sense([
                if phase > 0.6 { 1.0 } else { 0.0 },
                if phase > 0.3 { 1.0 } else { 0.0 },
                phase,
                1.0 - phase,
                if phase > 0.5 { 1.0 } else { 0.0 },
                phase * 0.5,
                0.5,
                phase,
            ]);
            let out_a = a.tick(&s, (phase - 0.5) * 0.6, 0.1, (t % 7) as u8);
            let out_b = b.tick(&s, (phase - 0.5) * 0.6, 0.1, (t % 7) as u8);
            assert_eq!(out_a.decision.action, out_b.decision.action);
            if out_a.fear_event { fear_hits += 1; }
        }
        assert_eq!(a, b, "tick must be deterministic");
        assert!(a.genes.iter().all(|v| v.is_finite()));
        assert!(a.world.iter().all(|v| v.is_finite()));
        assert!(a.wm.iter().all(|v| v.is_finite()));
        assert!(a.memory_valence.iter().all(|v| v.is_finite()));
        assert!(a.fear.is_finite() && a.confidence.is_finite());
        assert!(a.upkeep().is_finite());
        let _ = fear_hits;
    }

    #[test]
    fn serde_roundtrip_is_exact() {
        let mut b = TriuneBrain::reference("scav", 29);
        for t in 0..30 {
            let s = sense([(t % 3) as f64 / 3.0, 1.0, 0.5, 0.0, 0.2, 0.3, 0.5, 0.4]);
            b.tick(&s, 0.1, 0.1, (t % 7) as u8);
        }
        let json = serde_json::to_string(&b).unwrap();
        let mut c: TriuneBrain = serde_json::from_str(&json).unwrap();
        c.normalize_saved();
        assert_eq!(b, c);
        let j2 = serde_json::to_string(&c).unwrap();
        assert_eq!(json, j2);
        // 继续走一步仍然 bit-exact
        let s = sense([0.3, 0.2, 0.7, 0.1, 0.4, 0.5, 0.6, 0.7]);
        let da = b.tick(&s, 0.2, 0.1, 2).decision;
        let dc = c.tick(&s, 0.2, 0.1, 2).decision;
        assert_eq!(da, dc);
    }

    // ----- Phase 42: 经验回放 -------------------------------------------------

    #[test]
    fn replay_buffer_is_bounded_and_deterministic() {
        let mut a = TriuneBrain::reference("herb", 31);
        let mut b = a.clone();
        for t in 0..60 {
            let s = sense([(t % 5) as f64 / 5.0, 1.0, 0.4, 0.1, 0.2, 0.3, 0.5, 0.6]);
            let oa = a.tick(&s, 0.1, 0.1, (t % 7) as u8);
            let ob = b.tick(&s, 0.1, 0.1, (t % 7) as u8);
            assert_eq!(oa.decision.action, ob.decision.action);
            a.replay_step();
            b.replay_step();
        }
        assert_eq!(a, b, "tick + replay must stay deterministic");
        assert_eq!(a.replay.len(), REPLAY_CAP, "ring grows to the fixed cap");
        assert_eq!(a.replay_len, REPLAY_CAP);
        // 60 tick 里只有 59 条在线转移（首次 tick 没有上一条经验），
        // 因此也只有 59 次回放更新。
        assert_eq!(a.stats.replay_updates, 59);
        assert!(a.replay.iter().all(|e| e.action < OUTPUT_ACTIONS as u8));
        assert!(a.replay.iter().all(|e| e.next.iter().all(|v| v.is_finite())));
    }

    #[test]
    fn replay_does_not_move_neuromodulators() {
        let mut b = TriuneBrain::reference("pred", 37);
        for t in 0..24 {
            let s = sense([0.2, 0.9, 0.1, 0.7, 0.0, 0.4, 0.5, 0.3]);
            b.tick(&s, (t % 3) as f64 - 1.0, 0.1, (t % 7) as u8);
        }
        let rpe = b.last_rpe;
        let conf = b.confidence;
        let err = b.pred_error_ema;
        let cal = b.calibration;
        assert!(b.replay_step().is_some());
        assert_eq!(b.last_rpe, rpe, "离线重放不得改写 RPE（DA 必须反映在线误差）");
        assert_eq!(b.confidence, conf);
        assert_eq!(b.pred_error_ema, err);
        assert_eq!(b.calibration, cal);
    }

    #[test]
    fn replay_refines_rare_action_world_model() {
        // 真转移：所有动作都是同一个固定映射 next = tanh(0.4·s + 0.2)，奖励 0。
        // 主行为几乎只执行动作 0，动作 3 每 20 tick 才执行一次。
        // 只做在线学习的脑对动作 3 只能拿到 1/20 的样本；有回放的脑在每次
        // 节律 tick 额外巩固一条历史经验 → 动作 3 的预测误差应明显更低。
        let truth = |s: &[f64; INPUT_CHANNELS], action: usize| -> ([f64; INPUT_CHANNELS], f64) {
            let mut n = [0.0; INPUT_CHANNELS];
            for i in 0..INPUT_CHANNELS {
                n[i] = libm_tanh(0.4 * s[i] + 0.2) + 0.05 * action as f64 * s[i];
            }
            (n, 0.0)
        };
        let make = || {
            let mut b = TriuneBrain::reference("herb", 41);
            b.genes[G_WORLD_LR] = 0.15;
            b
        };
        let mut online = make();
        let mut replayed = make();
        for t in 0..400 {
            let s = sense([
                (t % 7) as f64 / 7.0,
                ((t * 3) % 5) as f64 / 5.0,
                ((t * 5) % 11) as f64 / 11.0,
                0.5,
                0.2,
                0.3,
                0.4,
                0.6,
            ]);
            let action = if t % 20 == 0 { 3usize } else { 0usize };
            let (next, reward) = truth(&s, action);
            // tick 用 executed 记录上一条转移；这里直接显式喂一条转移
            // （等价于 tick 的 online 路径，但不受节律/情绪影响）。
            online.observe(&s, action, &next, reward);
            online.record(&s, action, &next, reward);
            replayed.observe(&s, action, &next, reward);
            replayed.record(&s, action, &next, reward);
            replayed.replay_step();
        }
        // 再各自把同一批稀有动作经验写入缓冲，用于同一把尺子度量。
        for t in 0..40 {
            let s = sense([
                ((t * 11) % 13) as f64 / 13.0,
                0.8,
                0.2,
                0.4,
                0.1,
                0.5,
                0.6,
                0.3,
            ]);
            let (next, reward) = truth(&s, 3);
            online.observe(&s, 3, &next, reward);
            online.record(&s, 3, &next, reward);
            replayed.observe(&s, 3, &next, reward);
            replayed.record(&s, 3, &next, reward);
        }
        // 两边缓冲内容一致（只是回放次数不同），尺子相同。
        assert_eq!(online.replay, replayed.replay);
        let e_online = online.probe_error_for(3).unwrap();
        let e_replay = replayed.probe_error_for(3).unwrap();
        assert!(
            e_replay < e_online * 0.95,
            "replay should refine the rare action model: replay {:.4} vs online {:.4}",
            e_replay, e_online
        );
        assert!(replayed.stats.replay_updates == 400);
    }

    #[test]
    fn probe_error_decreases_with_learning() {
        let mut b = TriuneBrain::reference("omn", 43);
        b.genes[G_WORLD_LR] = 0.2;
        let s = sense([0.1, 0.8, 0.3, 0.0, 0.4, 0.2, 0.5, 0.6]);
        let next = [0.5, 0.2, 0.6, 0.1, 0.3, 0.4, 0.7, 0.2];
        for _ in 0..200 {
            b.observe(&s, 2, &next, 0.3);
        }
        // 手动写入一条历史经验，测 probe 是否反映当前拟合。
        b.replay.clear();
        b.replay.push(ReplayEntry { sense: s, action: 2, next, reward: 0.3 });
        b.replay_len = 1;
        let fitted = b.probe_error();
        assert!(fitted < 0.1, "fitted model error should be small, got {}", fitted);
        b.replay[0].next = [0.9; INPUT_CHANNELS];
        assert!(b.probe_error() > fitted, "probe must respond to a mismatched target");
    }
}
