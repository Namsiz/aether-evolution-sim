//! aether-brain — Phase 36 智能层：可演化微脑（Evolvable Micro-Brain）。
//!
//! 极小循环神经网络（tiny recurrent network）。设计约束（见路线图 §2.3 /
//! §4 确定性纪律）：
//!
//! * **零额外 RNG**：`forward` / `hebbian_update` / 权重初始化全部是纯函数
//!   （输入 + 当前突触 → 新输出 / 新突触），不消耗任何 RNG 流。脑结构突变
//!   走外部传入的 RNG 流（由 ecology 的 `prng` 提供，不与 main/fish 混用）。
//! * **激活函数全走 `aether-libm`**（`libm_tanh` / `libm_sigmoid` 基于
//!   `aether_libm::exp` 构建，跨平台位一致，黄金样本制）。
//! * **权重向量按固定索引编址**（紧凑存档，无稀疏图）。
//! * **独立学习 prng 流 tag** `BRAIN_RNG_TAG`：供未来探索性变异使用；当前
//!   学习本身零 RNG（资格迹更新 = 纯状态函数），该 tag 仅作审计路径标识。
//!
//! 输入通道（sense 向量，固定 8 通道，由 ecology 在 FSM 决策后采集）：
//!   0  threat        是否有捕食者/威胁（0/1）
//!   1  hungry        能量低于阈值（0/1）
//!   2  food_near     感知范围内是否有食物（0/1）
//!   3  prey_near     感知范围内是否有活体猎物（0/1，PRED）
//!   4  social        社会行为激活（0/1）
//!   5  fatigue       疲劳度（0..1）
//!   6  daylight      昼夜光照（0..1）
//!   7  dist_norm     到目标的归一化距离（0..1）
//!
//! 输出通道（7 维 one-hot logit，argmax = 选中的 action）：
//!   0  move_forward  前移
//!   1  turn_left     左转
//!   2  turn_right    右转
//!   3  eat           进食
//!   4  flee          逃跑
//!   5  reproduce     繁殖
//!   6  help          帮扶

use aether_libm;

/// sense 输入通道数。
pub const INPUT_CHANNELS: usize = 8;
/// 输出 action 数（move_forward / turn_left / turn_right / eat / flee /
/// reproduce / help）。
pub const OUTPUT_ACTIONS: usize = 7;
/// 初始隐藏节点数（足够编码六祖先 FSM 行为）。
pub const INITIAL_NODES: usize = 4;
/// 节点数上限（Ohno 复制增长上界；工程上限 ≠ 机制上限，见路线图 R2）。
pub const MAX_NODES: usize = 16;
/// 权重向量硬上限：`n_in*n_hid + n_hid + n_hid*n_hid + n_hid*n_out + n_out`。
/// 在 `MAX_NODES` 下为 `8*16 + 16 + 256 + 112 + 7 = 519`。
pub const MAX_WEIGHTS: usize =
    INPUT_CHANNELS * MAX_NODES + MAX_NODES + MAX_NODES * MAX_NODES + MAX_NODES * OUTPUT_ACTIONS + OUTPUT_ACTIONS;

/// 独立学习 prng 流 tag（`seed ^ BRAIN_RNG_TAG` 派生专用流，审计路径标识）。
pub const BRAIN_RNG_TAG: u32 = 0x42_52_41_49;

/// Hebbian 学习率（每 spike 的权重增量系数）。
pub const HEBBIAN_ETA: f64 = 0.01;
/// 资格迹指数衰减时间常数（ticks）。
pub const ELIGIBILITY_TAU: f64 = 30.0;
/// 资格迹钳位上界（突触饱和，防止无界 Hebbian 正反馈爆炸）。
/// 数学上资格迹受 pre∈[0,1]×post∈[-1,1] 约束的平衡点 ≈ 1/(1-ε) ≈ 300，
/// 但输出层是线性的（无 tanh 饱和），W_out 的 post 无条件增长 →
/// eligibility 无界 → 权重无界 → Inf → NaN。钳位即生物学突触饱和。
pub const ELIGIBILITY_CLAMP: f64 = 50.0;
/// 单个突触权重钳位（‖w‖∞ 上界）。W_out 钳到 8 后输出 logits 有界
/// （4 隐藏 × 8 + b_out 8 = ±40），反馈增益不再发散。
pub const WEIGHT_CLAMP: f32 = 8.0;
/// 单位脑维护费（每节点每日扣能量，Herculano-Houzel 缩放）。
pub const BRAIN_PER_NEURON_COST: f64 = 0.01;
/// 每次脑决策（forward + 学习更新）的信息成本（进入 `ledger.info_cost`）。
pub const BRAIN_PER_DECISION_COST: f64 = 0.004;
/// 趋化式时间差分基线平滑系数（慢变内部变量 EMA）。
pub const TAXIS_EMA_ALPHA: f64 = 0.05;

// ---------------------------------------------------------------------------
// libm 激活函数（全走 aether-libm::exp，跨平台位一致）
// ---------------------------------------------------------------------------

/// `tanh` 基于 `aether_libm::exp`：`tanh(x) = sign(x)·(e^{2|x|} - 1)/(e^{2|x|} + 1)`。
/// 对 `|x| > 20` 夹到 ±1（e^{2·20} ≈ 2.35e17，远未溢出），保证 wasm/native 位一致。
#[inline]
pub fn libm_tanh(x: f64) -> f64 {
    let ax = x.abs();
    if ax > 20.0 {
        if x >= 0.0 { 1.0 } else { -1.0 }
    } else {
        let t = aether_libm::exp(2.0 * ax);
        let r = (t - 1.0) / (t + 1.0);
        if x >= 0.0 { r } else { -r }
    }
}

/// `sigmoid(x) = 1/(1 + e^{-x})`。极端 x 安全：`x < -40` → 0，`x > 40` → 1
/// （`e^{-x}` 下溢/上溢前已被夹断）。
#[inline]
pub fn libm_sigmoid(x: f64) -> f64 {
    if x > 40.0 { 1.0 }
    else if x < -40.0 { 0.0 }
    else { 1.0 / (1.0 + aether_libm::exp(-x)) }
}

// ---------------------------------------------------------------------------
// Brain 结构（紧凑权重布局）
// ---------------------------------------------------------------------------

/// 一个隐藏节点：`bias` 是输入偏置，`activation`/`output` 是上一 tick 的
/// 激活与输出（循环依赖的存储）。
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub bias: f32,
    pub activation: f64,
    pub output: f64,
}

/// 极简循环网络。
///
/// 权重向量按固定索引编址（紧凑存档，无稀疏图）：
/// * `[0 .. n_in*n_hid)`                     = W_in  (input → hidden)
/// * `[n_in*n_hid .. +n_hid)`                = b_hid (hidden biases，冗余于 Node.bias)
/// * `[.. +n_hid*n_hid)`                     = W_rec (hidden → hidden，循环)
/// * `[.. +n_hid*n_out)`                     = W_out (hidden → output)
/// * `[.. +n_out)`                           = b_out (output biases)
///
/// `eligibility` 与 `weights` 同布局（Hebbian 资格迹）。`nodes` 长度 = `n_hid`，
/// 其 `output` 字段充当上一 tick 的循环状态。
#[derive(Clone, Debug, PartialEq)]
pub struct Brain {
    pub n_in: usize,
    pub n_out: usize,
    pub n_hid: usize,
    pub weights: Vec<f32>,
    pub nodes: Vec<Node>,
    /// 输出 logits（argmax = 选中的 action）。
    pub outputs: Vec<f64>,
    /// Hebbian 资格迹（同 `weights` 布局）。
    pub eligibility: Vec<f64>,
    /// 慢变内部变量：趋化式时间差分基线（能量 EMA）。
    pub taxis_baseline: f64,
    /// 上次奖励信号（能量变化率，TD(0) 同构）。
    pub last_reward: f64,
}

/// 权重索引（固定编址）。`kind`：0=W_in, 1=b_hid, 2=W_rec, 3=W_out, 4=b_out。
#[inline]
pub fn w_index(kind: u8, a: usize, b: usize, n_in: usize, n_hid: usize, n_out: usize) -> usize {
    match kind {
        0 => a * n_hid + b,                                            // W_in[a][b]: input a → hidden b
        1 => n_in * n_hid + b,                                         // b_hid[b]
        2 => n_in * n_hid + n_hid + a * n_hid + b,                     // W_rec[a][b]: hidden a → hidden b
        3 => n_in * n_hid + n_hid + n_hid * n_hid + a * n_out + b,     // W_out[a][b]: hidden a → output b
        4 => n_in * n_hid + n_hid + n_hid * n_hid + n_hid * n_out + b, // b_out[b]
        _ => panic!("bad weight kind"),
    }
}

/// 给定布局所需权重槽位数。
pub fn weight_slots(n_in: usize, n_hid: usize, n_out: usize) -> usize {
    n_in * n_hid + n_hid + n_hid * n_hid + n_hid * n_out + n_out
}

/// 单个突触的 Hebbian 更新（资格迹衰减累积 + 权重增量 + 双钳位）。
/// 钳位是机制的一部分：线性输出层使 W_out 的 post 无界，无钳位时
/// 正反馈会导致权重指数爆炸 → Inf → NaN 污染存档。
#[inline]
fn hebbian_step(e: &mut f64, w: &mut f32, pre_post: f64, decay: f64, eta: f64) {
    let prod = if pre_post.is_finite() { pre_post } else { 0.0 };
    let mut next_e = decay * *e + prod;
    if !next_e.is_finite() { next_e = 0.0; }
    *e = next_e.clamp(-ELIGIBILITY_CLAMP, ELIGIBILITY_CLAMP);
    let delta = (eta * *e) as f32;
    let next_w = *w + if delta.is_finite() { delta } else { 0.0 };
    *w = next_w.clamp(-WEIGHT_CLAMP, WEIGHT_CLAMP);
}

impl Brain {
    /// 创建全零权重/偏置网络（`n_hid` 个隐藏节点）。
    pub fn zero(n_in: usize, n_hid: usize, n_out: usize) -> Self {
        let slots = weight_slots(n_in, n_hid, n_out);
        Brain {
            n_in,
            n_out,
            n_hid,
            weights: vec![0.0f32; slots],
            nodes: vec![Node { bias: 0.0, activation: 0.0, output: 0.0 }; n_hid],
            outputs: vec![0.0; n_out],
            eligibility: vec![0.0; slots],
            taxis_baseline: 0.0,
            last_reward: 0.0,
        }
    }

    /// 从权重/偏置切片重建网络（用于存档回填）。`weights` 长度必须等于
    /// `weight_slots(n_in, n_hid, n_out)`。
    pub fn from_weights(n_in: usize, n_hid: usize, n_out: usize, weights: &[f32]) -> Self {
        let slots = weight_slots(n_in, n_hid, n_out);
        assert_eq!(weights.len(), slots, "brain weight slice length");
        let mut nodes: Vec<Node> = (0..n_hid)
            .map(|b| Node {
                bias: weights[w_index(1, 0, b, n_in, n_hid, n_out)],
                activation: 0.0,
                output: 0.0,
            })
            .collect();
        // 让初始 output 与 bias 一致（tanh(bias)），避免全零输入下循环层静默。
        for b in 0..n_hid {
            nodes[b].activation = libm_tanh(nodes[b].bias as f64);
            nodes[b].output = nodes[b].activation;
        }
        Brain {
            n_in,
            n_out,
            n_hid,
            weights: weights.to_vec(),
            nodes,
            outputs: vec![0.0; n_out],
            eligibility: vec![0.0; slots],
            taxis_baseline: 0.0,
            last_reward: 0.0,
        }
    }

    /// 前向传播（单步循环更新，零 RNG）。输入 `input` 长度 ≥ `n_in`。
    /// 返回 7 维输出 logits（one-hot = argmax）。
    ///
    /// 隐藏层偏置的唯一数据源是权重槽 `b_hid`（`weights[w_index(1, …)]`）：
    /// Hebbian 只更新这个槽，不再同步冗余的 `Node.bias`（后者仅作初始值
    /// 供 `reference_brain` 使用）。读冗余槽会让存档恢复路径（从 weights
    /// 重建 bias）与原始路径（未更新的 Node.bias）在一 tick 后产生 1-ulp
    /// 级分叉。
    pub fn forward(&mut self, input: &[f64]) -> Vec<f64> {
        let n_in = self.n_in;
        let n_hid = self.n_hid;
        let n_out = self.n_out;
        let mut new_act = vec![0.0f64; n_hid];
        for b in 0..n_hid {
            let mut acc = self.weights[w_index(1, 0, b, n_in, n_hid, n_out)] as f64;
            for j in 0..n_in {
                let w = self.weights[w_index(0, j, b, n_in, n_hid, n_out)] as f64;
                acc += w * input[j.min(input.len() - 1)];
            }
            for a in 0..n_hid {
                let w = self.weights[w_index(2, a, b, n_in, n_hid, n_out)] as f64;
                acc += w * self.nodes[a].output;
            }
            new_act[b] = libm_tanh(acc);
        }
        for b in 0..n_hid {
            self.nodes[b].activation = new_act[b];
            self.nodes[b].output = new_act[b];
        }
        let mut out = vec![0.0f64; n_out];
        for o in 0..n_out {
            let mut acc = self.weights[w_index(4, 0, o, n_in, n_hid, n_out)] as f64;
            for a in 0..n_hid {
                let w = self.weights[w_index(3, a, o, n_in, n_hid, n_out)] as f64;
                acc += w * self.nodes[a].output;
            }
            out[o] = acc;
        }
        self.outputs = out.clone();
        out
    }

    /// 取 one-hot 动作索引（argmax；平局取最低索引，确定性）。
    pub fn argmax_action(&self) -> usize {
        let mut best = 0usize;
        let mut bv = self.outputs[0];
        for (i, &v) in self.outputs.iter().enumerate().skip(1) {
            if v > bv { bv = v; best = i; }
        }
        best
    }

    /// 趋化式时间差分奖励：`reward = clamp(energy - baseline, -1, 1)`，
    /// 然后更新 `baseline += alpha * (energy - baseline)`（慢变 EMA）。
    /// 零 RNG，纯状态函数。
    pub fn taxis_reward(&mut self, energy: f64) -> f64 {
        let delta = energy - self.taxis_baseline;
        let r = delta.clamp(-1.0, 1.0);
        self.taxis_baseline += TAXIS_EMA_ALPHA * delta;
        r
    }

    /// Hebbian 资格迹更新（零 RNG，纯函数）：
    /// * 每个权重资格迹 `e_ij = exp(-dt/τ)·e_ij + pre_i·post_j`
    /// * 权重更新 `w_ij += η · e_ij · reward`
    /// `pre`/`post` 分别为上一 tick 的输入激活与当前隐藏激活（含 1.0 偏置
    /// 常值通道以覆盖偏置学习）。奖励 = 趋化式时间差分。
    pub fn hebbian_update(&mut self, input: &[f64], reward: f64, dt: f64) {
        let n_in = self.n_in;
        let n_hid = self.n_hid;
        let n_out = self.n_out;
        let decay = aether_libm::exp(-dt / ELIGIBILITY_TAU);
        let eta = HEBBIAN_ETA * reward;
        if !eta.is_finite() { return; }
        // 构造 pre 向量：输入通道 + 隐藏输出 + 常值 1.0（偏置通道）。
        let mut pre = vec![0.0f64; n_in + n_hid + 1];
        for j in 0..n_in { pre[j] = input[j.min(input.len() - 1)]; }
        for a in 0..n_hid { pre[n_in + a] = self.nodes[a].output; }
        pre[n_in + n_hid] = 1.0;
        // 资格迹衰减 + 累积（W_in / b_hid / W_rec 通道由 pre × post_hid 驱动）。
        for b in 0..n_hid {
            let post = self.nodes[b].output;
            for j in 0..n_in {
                let idx = w_index(0, j, b, n_in, n_hid, n_out);
                hebbian_step(&mut self.eligibility[idx], &mut self.weights[idx],
                    pre[j] * post, decay, eta);
            }
            // b_hid（偏置通道 = 常值 1.0 输入）
            let idx = w_index(1, 0, b, n_in, n_hid, n_out);
            hebbian_step(&mut self.eligibility[idx], &mut self.weights[idx],
                pre[n_in + n_hid] * post, decay, eta);
        }
        // W_rec：隐藏 → 隐藏（pre 用隐藏输出，post 用隐藏输出）
        for a in 0..n_hid {
            for b in 0..n_hid {
                let idx = w_index(2, a, b, n_in, n_hid, n_out);
                let pre_a = self.nodes[a].output;
                let post_b = self.nodes[b].output;
                hebbian_step(&mut self.eligibility[idx], &mut self.weights[idx],
                    pre_a * post_b, decay, eta);
            }
        }
        // W_out：隐藏 → 输出
        for a in 0..n_hid {
            for o in 0..n_out {
                let idx = w_index(3, a, o, n_in, n_hid, n_out);
                let post_o = self.outputs[o];
                hebbian_step(&mut self.eligibility[idx], &mut self.weights[idx],
                    self.nodes[a].output * post_o, decay, eta);
            }
        }
        // b_out（输出偏置通道）
        for o in 0..n_out {
            let idx = w_index(4, 0, o, n_in, n_hid, n_out);
            let post_o = self.outputs[o];
            hebbian_step(&mut self.eligibility[idx], &mut self.weights[idx],
                pre[n_in + n_hid] * post_o, decay, eta);
        }
        self.last_reward = reward;
    }

    /// Ohno 式节点复制（脑结构突变，消耗 `rng`）：随机选一个已有隐藏节点
    /// 复制到末尾（新节点 = 副本 + 小扰动），重新编址并补齐权重/资格迹。
    /// 返回是否发生复制。
    pub fn duplicate_node(&mut self, rng: &mut aether_rng::Mulberry32) -> bool {
        if self.n_hid >= MAX_NODES { return false; }
        let old_n = self.n_hid;
        let new_n = old_n + 1;
        let n_in = self.n_in;
        let n_out = self.n_out;
        let src = (rng.next_f64() * old_n as f64) as usize % old_n;
        // 新布局：复制 src 的 bias + 相关权重，新连接加小扰动。
        let mut new_weights = vec![0.0f32; weight_slots(n_in, new_n, n_out)];
        let mut new_nodes: Vec<Node> = Vec::with_capacity(new_n);
        // 复制旧隐藏节点
        for b in 0..old_n { new_nodes.push(self.nodes[b].clone()); }
        // 新节点 = src 副本 + 扰动（bias 的权威源是 weights 的 b_hid 槽）
        let src_bias = self.weights[w_index(1, 0, src, n_in, old_n, n_out)] as f64;
        new_nodes.push(Node {
            bias: (src_bias + (rng.next_f64() - 0.5) * 0.6).clamp(-3.0, 3.0) as f32,
            activation: 0.0,
            output: 0.0,
        });
        // W_in（新行）
        for j in 0..n_in {
            let w = self.weights[w_index(0, j, src, n_in, old_n, n_out)] as f64;
            new_weights[w_index(0, j, new_n - 1, n_in, new_n, n_out)] =
                (w + (rng.next_f64() - 0.5) * 0.4).clamp(-3.0, 3.0) as f32;
        }
        // b_hid
        new_weights[w_index(1, 0, new_n - 1, n_in, new_n, n_out)] =
            (src_bias + (rng.next_f64() - 0.5) * 0.4).clamp(-3.0, 3.0) as f32;
        // W_rec 旧块
        for a in 0..old_n {
            for b in 0..old_n {
                new_weights[w_index(2, a, b, n_in, new_n, n_out)] =
                    self.weights[w_index(2, a, b, n_in, old_n, n_out)];
            }
        }
        // W_rec 新行/新列
        for a in 0..old_n {
            new_weights[w_index(2, a, new_n - 1, n_in, new_n, n_out)] =
                (self.weights[w_index(2, a, src, n_in, old_n, n_out)] as f64 * 0.5 + (rng.next_f64() - 0.5) * 0.4) as f32;
            new_weights[w_index(2, new_n - 1, a, n_in, new_n, n_out)] =
                (self.weights[w_index(2, src, a, n_in, old_n, n_out)] as f64 * 0.5 + (rng.next_f64() - 0.5) * 0.4) as f32;
        }
        new_weights[w_index(2, new_n - 1, new_n - 1, n_in, new_n, n_out)] =
            (self.weights[w_index(2, src, src, n_in, old_n, n_out)] as f64 + (rng.next_f64() - 0.5) * 0.4) as f32;
        // W_out 新行
        for o in 0..n_out {
            let w = self.weights[w_index(3, src, o, n_in, old_n, n_out)] as f64;
            new_weights[w_index(3, new_n - 1, o, n_in, new_n, n_out)] =
                (w + (rng.next_f64() - 0.5) * 0.4).clamp(-3.0, 3.0) as f32;
        }
        // b_out（不变）
        for o in 0..n_out {
            new_weights[w_index(4, 0, o, n_in, new_n, n_out)] =
                self.weights[w_index(4, 0, o, n_in, old_n, n_out)];
        }
        self.n_hid = new_n;
        self.weights = new_weights;
        self.nodes = new_nodes;
        self.eligibility = vec![0.0; weight_slots(n_in, new_n, n_out)];
        true
    }

    /// Ohno 式节点删除（脑结构收缩）：删除最后一个隐藏节点。返回是否发生。
    pub fn drop_node(&mut self) -> bool {
        if self.n_hid <= 2 { return false; }
        let old_n = self.n_hid;
        let new_n = old_n - 1;
        let n_in = self.n_in;
        let n_out = self.n_out;
        let mut new_weights = vec![0.0f32; weight_slots(n_in, new_n, n_out)];
        for j in 0..n_in {
            for b in 0..new_n {
                new_weights[w_index(0, j, b, n_in, new_n, n_out)] =
                    self.weights[w_index(0, j, b, n_in, old_n, n_out)];
            }
        }
        for b in 0..new_n {
            new_weights[w_index(1, 0, b, n_in, new_n, n_out)] =
                self.weights[w_index(1, 0, b, n_in, old_n, n_out)];
        }
        for a in 0..new_n {
            for b in 0..new_n {
                new_weights[w_index(2, a, b, n_in, new_n, n_out)] =
                    self.weights[w_index(2, a, b, n_in, old_n, n_out)];
            }
        }
        for a in 0..new_n {
            for o in 0..n_out {
                new_weights[w_index(3, a, o, n_in, new_n, n_out)] =
                    self.weights[w_index(3, a, o, n_in, old_n, n_out)];
            }
        }
        for o in 0..n_out {
            new_weights[w_index(4, 0, o, n_in, new_n, n_out)] =
                self.weights[w_index(4, 0, o, n_in, old_n, n_out)];
        }
        self.n_hid = new_n;
        self.weights = new_weights;
        self.nodes.truncate(new_n);
        self.eligibility = vec![0.0; weight_slots(n_in, new_n, n_out)];
        true
    }

    /// 脑结构突变入口：以固定概率 `p_add`/`p_del` 决定是否复制/删除节点
    /// （消耗 `rng`，确定性流由调用方保证）。每次调用最多一个结构事件。
    pub fn mutate_structure(&mut self, rng: &mut aether_rng::Mulberry32, p_add: f64, p_del: f64) {
        let roll = rng.next_f64();
        if roll < p_add {
            self.duplicate_node(rng);
        } else if roll < p_add + p_del {
            self.drop_node();
        }
    }

    /// 权重扰动（脑学习之外的遗传漂移，消耗 `rng`）。用于子代初始化前
    /// 给祖父网络加微噪声。
    pub fn perturb(&mut self, rng: &mut aether_rng::Mulberry32, sigma: f64) {
        for w in self.weights.iter_mut() {
            let v = (*w as f64) + (rng.next_f64() - 0.5) * sigma;
            *w = v.clamp(-6.0, 6.0) as f32;
        }
    }

    /// 每 tick 脑维护成本（`BRAIN_PER_NEURON_COST × 节点数`）。零 RNG。
    pub fn upkeep(&self) -> f64 {
        BRAIN_PER_NEURON_COST * self.n_hid as f64
    }
}

// ---------------------------------------------------------------------------
// 参考网络（grandfathering）：六祖先 FSM 编译出的等价权重表
// ---------------------------------------------------------------------------

/// 按 kind 名返回一个参考网络（grandfather network）。六祖先 FSM 的行为
/// 被编译为固定权重表（线性阈值网络 + 一个隐藏层），使得初始脑输出与
/// FSM 决策在统计上同分布（T_EQUIV 阈值由 ecology 维护）。
///
/// `kind_name` ∈ {"herb","pred","scav","omn","fish","deco"}；未知 kind 返回
/// `Brain::zero(INPUT_CHANNELS, INITIAL_NODES, OUTPUT_ACTIONS)`（保守默认）。
pub fn reference_brain(kind_name: &str) -> Brain {
    let n_in = INPUT_CHANNELS;
    let n_hid = INITIAL_NODES;
    let n_out = OUTPUT_ACTIONS;
    let mut b = Brain::zero(n_in, n_hid, n_out);
    // 隐藏节点偏置（使 tanh 工作在线性区附近）。
    let hid_biases = [0.0, 0.2, -0.2, 0.0];
    for (i, &v) in hid_biases.iter().enumerate() {
        b.nodes[i].bias = v as f32;
        b.weights[w_index(1, 0, i, n_in, n_hid, n_out)] = v as f32;
        b.nodes[i].activation = libm_tanh(v);
        b.nodes[i].output = libm_tanh(v);
    }
    // 输入 → 隐藏：通道 0 threat、1 hungry、2 food_near、3 prey_near、
    // 4 social、5 fatigue、6 daylight、7 dist_norm
    let w_in = |b: &mut Brain, ch: usize, node: usize, w: f64| {
        b.weights[w_index(0, ch, node, n_in, n_hid, n_out)] = w as f32;
    };
    let w_out = |b: &mut Brain, node: usize, act: usize, w: f64| {
        b.weights[w_index(3, node, act, n_in, n_hid, n_out)] = w as f32;
    };
    match kind_name {
        "herb" => {
            // 威胁 → flee；饥饿+食物 → eat；基础 → move_forward
            w_in(&mut b, 0, 1, 2.0);   // threat → node1 (→ flee)
            w_out(&mut b, 1, 4, 3.0);  // node1 → flee
            w_in(&mut b, 1, 2, 1.5);   // hungry → node2
            w_in(&mut b, 2, 2, 1.5);   // food_near → node2
            w_out(&mut b, 2, 3, 3.0);  // node2 → eat
            w_in(&mut b, 7, 0, 1.0);   // dist_norm → node0
            w_out(&mut b, 0, 0, 1.5);  // node0 → move_forward
            // 基础前进（bias 侧）
            b.weights[w_index(4, 0, 0, n_in, n_hid, n_out)] = 0.5;
            b.weights[w_index(4, 0, 1, n_in, n_hid, n_out)] = -0.2;
            b.weights[w_index(4, 0, 2, n_in, n_hid, n_out)] = 0.2;
        }
        "pred" => {
            // 猎物存在 → hunt/move；尸体存在 → scavenge/move
            w_in(&mut b, 3, 1, 2.0);   // prey_near → node1
            w_out(&mut b, 1, 0, 3.0);  // node1 → move_forward
            w_in(&mut b, 2, 2, 1.5);   // food_near (carcass) → node2
            w_out(&mut b, 2, 3, 2.5);  // node2 → eat
            w_in(&mut b, 1, 3, 1.5);   // hungry → node3
            w_out(&mut b, 3, 0, 2.0);  // node3 → move_forward
            b.weights[w_index(4, 0, 0, n_in, n_hid, n_out)] = 0.6;
        }
        "scav" | "omn" => {
            // 尸体/食物 → scavenge/move + eat；威胁 → flee
            w_in(&mut b, 2, 1, 2.0);   // food_near → node1
            w_out(&mut b, 1, 0, 3.0);  // node1 → move_forward
            w_in(&mut b, 1, 2, 1.5);   // hungry → node2
            w_out(&mut b, 2, 3, 2.5);  // node2 → eat
            w_in(&mut b, 0, 3, 2.5);   // threat → node3
            w_out(&mut b, 3, 4, 3.0);  // node3 → flee
            b.weights[w_index(4, 0, 0, n_in, n_hid, n_out)] = 0.4;
        }
        "fish" => {
            // 社会 → school；食物 → graze
            w_in(&mut b, 4, 1, 2.0);   // social → node1
            w_out(&mut b, 1, 0, 3.0);  // node1 → move_forward
            w_in(&mut b, 2, 2, 1.5);   // food_near → node2
            w_out(&mut b, 2, 3, 2.5);  // node2 → eat
            b.weights[w_index(4, 0, 0, n_in, n_hid, n_out)] = 0.5;
        }
        "deco" => {
            // 尸体 → approach/eat；else wander
            w_in(&mut b, 2, 1, 2.0);   // food_near → node1
            w_out(&mut b, 1, 0, 3.0);  // node1 → move_forward
            w_in(&mut b, 1, 2, 1.5);   // hungry → node2
            w_out(&mut b, 2, 3, 2.5);  // node2 → eat
            b.weights[w_index(4, 0, 0, n_in, n_hid, n_out)] = 0.35;
        }
        _ => {
            // 保守默认：全零 → 输出全 0 → argmax = move_forward
        }
    }
    b
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tanh_known_values() {
        assert!((libm_tanh(0.0) - 0.0).abs() < 1e-12);
        assert!((libm_tanh(1.0) - 0.7615941559557649).abs() < 1e-12);
        assert!((libm_tanh(-1.0) + 0.7615941559557649).abs() < 1e-12);
        assert_eq!(libm_tanh(100.0), 1.0);
        assert_eq!(libm_tanh(-100.0), -1.0);
    }

    #[test]
    fn sigmoid_known_values() {
        assert!((libm_sigmoid(0.0) - 0.5).abs() < 1e-12);
        assert!((libm_sigmoid(1.0) - 0.7310585786300049).abs() < 1e-12);
        assert_eq!(libm_sigmoid(100.0), 1.0);
        assert_eq!(libm_sigmoid(-100.0), 0.0);
    }

    #[test]
    fn reference_networks_have_expected_weights() {
        for name in ["herb", "pred", "scav", "omn", "fish", "deco"] {
            let b = reference_brain(name);
            assert_eq!(b.n_hid, INITIAL_NODES);
            assert_eq!(b.weights.len(), weight_slots(b.n_in, b.n_hid, b.n_out));
            assert_eq!(b.n_out, OUTPUT_ACTIONS);
        }
    }

    #[test]
    fn forward_is_deterministic_zero_rng() {
        let mut b = reference_brain("herb");
        let inp = [0.0f64; INPUT_CHANNELS];
        let o1 = b.forward(&inp);
        let o2 = b.forward(&inp);
        assert_eq!(o1, o2, "same-state forward must be bit-identical");
        // 不同输入产生不同输出
        let mut inp2 = inp;
        inp2[0] = 1.0; // threat
        let o3 = b.forward(&inp2);
        assert_ne!(o1, o3, "threat channel must move the output");
    }

    #[test]
    fn hebbian_updates_weights() {
        let mut b = reference_brain("herb");
        let inp = [0.0f64; INPUT_CHANNELS];
        b.forward(&inp);
        let before = b.weights.clone();
        let r = b.taxis_reward(0.5);
        b.hebbian_update(&inp, r, 0.1);
        assert_ne!(before, b.weights, "hebbian update must move weights");
        // 双跑 bit-exact
        let mut b2 = reference_brain("herb");
        b2.forward(&inp);
        let r2 = b2.taxis_reward(0.5);
        b2.hebbian_update(&inp, r2, 0.1);
        assert_eq!(b.weights, b2.weights, "hebbian is deterministic");
    }

    #[test]
    fn duplicate_and_drop_node_preserve_layout() {
        let mut b = reference_brain("pred");
        let mut rng = aether_rng::Mulberry32::new(42);
        let orig = b.n_hid;
        assert!(b.duplicate_node(&mut rng));
        assert_eq!(b.n_hid, orig + 1);
        assert_eq!(b.weights.len(), weight_slots(b.n_in, b.n_hid, b.n_out));
        assert!(b.drop_node());
        assert_eq!(b.n_hid, orig);
        assert_eq!(b.weights.len(), weight_slots(b.n_in, b.n_hid, b.n_out));
    }

    #[test]
    fn weight_slots_cap_within_bounds() {
        assert!(weight_slots(INPUT_CHANNELS, MAX_NODES, OUTPUT_ACTIONS) <= MAX_WEIGHTS);
    }
}
