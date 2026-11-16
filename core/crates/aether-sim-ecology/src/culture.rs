//! culture.rs — Phase 37 文化/技术层：知识 Token 与生态位构造。
//!
//! 核心思想：知识作为可传播可积累可失传的载体；技术 = 环境修改。
//! 不写任何具体技术（无火/轮子/文字内置）——世界若演化出它们，只能通过
//! token × 环境的效果路径自己走。
//!
//! # 确定性纪律
//! - 新 RNG 流全走专用 `TOKEN_RNG_TAG`（seed ^ TAG 派生），不与 main/prng/fish 混用
//! - `CULTURE_ON` 默认 true（本期是正式动力学阶段，预期 re-baseline）
//! - 关闭时（A/B 测试用）零额外 draw、零能量、轨迹回到 Phase 36

use aether_rng::Mulberry32;
use serde::{Serialize, Deserialize};
use crate::ecology::{Agent, Ecology};

/// 专用文化 RNG 流 tag：`seed ^ TOKEN_RNG_TAG` 派生专用流（审计路径标识）。
pub const TOKEN_RNG_TAG: u32 = 0x43_55_4C_54; // "CULT"
/// 文化层总开关。默认 true（本期是正式动力学阶段，预期 re-baseline）。
pub const CULTURE_ON: bool = true;
/// 知识向量的维度（固定 ~8，无预设语义）。
pub const TOKEN_DIM: usize = 8;
/// 习得成本（每次学习支付能量，进 ledger）。
pub const LEARNING_COST: f64 = 0.5;
/// 教学成本（亲代为每个传承 token 支付的能量）。标定：LEARNING_COST × 0.5 ——
/// 传授已掌握的知识比从零习得便宜（示范 vs 试错），但多 token 家庭每个孩子
/// 累计支付 TEACH_COST × len，亲代投入随知识存量线性增长（代际流动约束）。
pub const TEACH_COST: f64 = LEARNING_COST * 0.5;
/// 传播保真度 ∈ [0,1]（示范→模仿时 token 向量加噪，噪声幅度 ∝ (1-f)）。
pub const DEFAULT_FIDELITY: f64 = 0.9;
/// 重组创新概率（两个已知 token → 重组向量产生新 token，每持有者每 tick）。
/// Phase 37 的 0.01 标定于传播死锁期——holders 恒为个位数时从不出问题。
/// Phase 44.1 的空间邻近学习把 30k-tick 的 bearer 群抬到 144，0.01 立即
/// 变成信息爆炸（19839 个 token，其中 >97% 是重组）。创新必须是稀有事件：
/// 0.00002 让 500 持有者的世界每千 tick 约 10 次重组，与 de-novo 发明率
/// 同一量级。
pub const RECOMBINATION_P: f64 = 0.00002;
/// 重组变异 σ（crossover + mutation 时的噪声幅度）。
pub const RECOMBINATION_SIGMA: f64 = 0.05;
/// 结构放置成本（能量，进 ledger）。
pub const STRUCTURE_COST: f64 = 3.0;
/// 结构维护成本（每 tick 能量，进 ledger）。
pub const STRUCTURE_MAINTENANCE: f64 = 0.005;
/// 结构衰败率（无维护则衰减）。
pub const STRUCTURE_DECAY: f64 = 0.0005;
/// 结构影响范围（网格单元）。
pub const STRUCTURE_RADIUS: f64 = 3.0;
/// 结构影响强度（token 向量 → 局部场调制）。
pub const STRUCTURE_MOD_STRENGTH: f64 = 0.05;

/// Phase 45: token 价值 EMA 的新样本权重（记录一次结构放置能量反馈）。
pub const USE_GAIN_ALPHA: f64 = 0.01;
/// Phase 45: 无新样本时 token 价值的每 tick 衰减（0.999^1000 ≈ 0.37：
/// 停止产出价值的知识在千 tick 尺度淡出）。
pub const USE_GAIN_DECAY: f64 = 0.999;
/// Phase 45: 高价值（use_gain > 0）token 的传承保真度加成——好知识
/// 传得更准（涌现度量，价值完全来自能量学反馈，无预设语义）。
pub const VALUE_FIDELITY_BONUS: f64 = 0.05;

/// 知识 Token（开放向量编码，维度固定 ~8，无预设语义）。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeToken {
    pub id: u32,
    pub vec: Vec<f64>,
    /// Phase 45: 涌现价值 EMA——持有者通过该 token 的结构放置获得的能量
    /// 反馈的加权平均。纯观测记账：不消耗 RNG、不改能量路径，但
    /// inherit_at_birth/try_learn_token 的候选排序读它（文化选择）。
    #[serde(default)]
    pub use_gain: f64,
}

impl KnowledgeToken {
    pub fn new(id: u32, vec: Vec<f64>) -> Self {
        assert!(vec.len() == TOKEN_DIM, "token vector must have fixed dim {}", TOKEN_DIM);
        KnowledgeToken { id, vec, use_gain: 0.0 }
    }

    /// 生成随机初始 token（用于"发明"——de-novo 新 token）。
    pub fn random(id: u32, rng: &mut Mulberry32) -> Self {
        let mut vec = Vec::with_capacity(TOKEN_DIM);
        for _ in 0..TOKEN_DIM {
            vec.push(rng.next_f64() * 2.0 - 1.0);
        }
        KnowledgeToken::new(id, vec)
    }

    /// 与另一个 token 的欧氏距离（用于 content bias 相似度）。
    pub fn distance(&self, other: &Self) -> f64 {
        self.vec.iter()
            .zip(other.vec.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f64>()
            .sqrt()
    }

    /// 与另一向量（如基因组）的距离。
    pub fn distance_to_vec(&self, other: &[f64]) -> f64 {
        self.vec.iter()
            .zip(other.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f64>()
            .sqrt()
    }
}

/// 文化注册表（token 注册表，可增长——由个体发明产生，无上限）。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CultureRegistry {
    pub tokens: Vec<KnowledgeToken>,
    pub next_token_id: u32,
    /// Phase 45: id 索引的价值 EMA 镜像（O(1) 查值）。热路径每 tick 需要
    /// 几十次查值（学习目标选择/传承排序），线性扫 registry 在兆 tick
    /// token 量下会撑爆帧预算；镜像非持久化（serde skip），从 tokens
    /// 重建。不占 SaveState 空间。
    #[serde(skip)]
    value_cache: Vec<f64>,
}

impl CultureRegistry {
    pub fn new() -> Self {
        CultureRegistry { tokens: Vec::new(), next_token_id: 1, value_cache: Vec::new() }
    }

    pub fn len(&self) -> usize { self.tokens.len() }

    pub fn is_empty(&self) -> bool { self.tokens.is_empty() }

    /// 注册新 token，返回其 id。
    pub fn register(&mut self, token: KnowledgeToken) -> u32 {
        let id = token.id;
        let i = id as usize;
        if self.value_cache.len() <= i { self.value_cache.resize(i + 1, 0.0); }
        self.value_cache[i] = token.use_gain;
        self.tokens.push(token);
        self.next_token_id = self.next_token_id.max(id + 1);
        id
    }

    /// 获取指定 id 的 token。
    pub fn get(&self, id: u32) -> Option<&KnowledgeToken> {
        self.tokens.iter().find(|t| t.id == id)
    }

    /// 从 token 列表重建镜像（反序列化/恢复路径）。
    pub fn rebuild_value_cache(&mut self) {
        self.value_cache.clear();
        self.value_cache.resize(self.next_token_id as usize, 0.0);
        for t in &self.tokens {
            let i = t.id as usize;
            if i < self.value_cache.len() { self.value_cache[i] = t.use_gain; }
        }
    }

    /// Phase 45: 涌现价值查询（id → value EMA；未知 token 得 0）。O(1)。
    pub fn value_of(&self, id: u32) -> f64 {
        self.value_cache.get(id as usize).copied().unwrap_or(0.0)
    }

    /// Phase 45: 记录一次能量反馈到 token 的价值 EMA（结构放置路径）。
    pub fn record_use(&mut self, id: u32, gain: f64) {
        if let Some(t) = self.tokens.iter_mut().find(|t| t.id == id) {
            t.use_gain += USE_GAIN_ALPHA * (gain - t.use_gain);
            let i = id as usize;
            if i < self.value_cache.len() { self.value_cache[i] = t.use_gain; }
        }
    }

    /// Phase 45: 每 tick 全注册表价值衰减（无新样本的知识淡出）。
    /// 纯乘法、零 RNG——O(tokens) 每 tick，兆 tick 下 tokens ~7k 仍可忽略。
    pub fn decay_use_gains(&mut self) {
        for t in &mut self.tokens {
            t.use_gain *= USE_GAIN_DECAY;
        }
        for v in &mut self.value_cache {
            *v *= USE_GAIN_DECAY;
        }
    }

    /// Phase 45: 传承/学习候选的价值排序：价值降序，同价值按 id 升序
    /// （确定性，与注册表顺序无关）。
    pub fn sorted_by_value(ids: &[u32], registry: &Self) -> Vec<u32> {
        let mut out = ids.to_vec();
        out.sort_by(|&a, &b| {
            let va = registry.value_of(a);
            let vb = registry.value_of(b);
            vb.partial_cmp(&va).unwrap_or(std::cmp::Ordering::Equal).then(a.cmp(&b))
        });
        out
    }

    /// 重组+变异创新算子：两个已知 token → 重组向量产生新 token。
    /// 消耗专用 RNG 流。返回新 token 的 id（或 None 如果注册表为空）。
    pub fn recombine(&mut self, a_id: u32, b_id: u32, rng: &mut Mulberry32) -> Option<u32> {
        let a = self.get(a_id)?;
        let b = self.get(b_id)?;
        // 单点 crossover（随机切点）
        let cut = (rng.next_f64() * TOKEN_DIM as f64) as usize % TOKEN_DIM;
        let mut vec = Vec::with_capacity(TOKEN_DIM);
        for i in 0..TOKEN_DIM {
            let parent = if i < cut { a } else { b };
            let mut v = parent.vec[i];
            // mutation：噪声幅度 ∝ RECOMBINATION_SIGMA
            v += (rng.next_f64() * 2.0 - 1.0) * RECOMBINATION_SIGMA;
            // 保持在 [-1, 1] 内
            vec.push(v.clamp(-1.0, 1.0));
        }
        let id = self.next_token_id;
        self.next_token_id += 1;
        let token = KnowledgeToken::new(id, vec);
        self.register(token);
        Some(id)
    }

    /// 塔斯马尼亚式失传：检查哪些 token 不再被任何持有者持有，
    /// 从持有状态中移除（但保留在注册表中作为历史）。
    pub fn check_extinct(&self, holders: &[Vec<u32>]) -> Vec<u32> {
        let mut held = std::collections::HashSet::new();
        for h in holders {
            for &t in h {
                held.insert(t);
            }
        }
        self.tokens.iter()
            .filter(|t| !held.contains(&t.id))
            .map(|t| t.id)
            .collect()
    }
}

/// 结构（生态位构造的物理载体）——agent 可消耗能量在环境中放置/维护。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Structure {
    pub id: u32,
    pub token_id: u32,
    pub x: f64,
    pub z: f64,
    pub strength: f64,
    pub health: f64,
    pub builder_id: u32,
}

impl Structure {
    pub fn new(id: u32, token_id: u32, x: f64, z: f64, strength: f64, health: f64, builder_id: u32) -> Self {
        Structure { id, token_id, x, z, strength, health, builder_id }
    }
}

/// 结构库（物理载体注册表）。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StructureRegistry {
    pub structures: Vec<Structure>,
    pub next_structure_id: u32,
}

impl StructureRegistry {
    pub fn new() -> Self {
        StructureRegistry { structures: Vec::new(), next_structure_id: 1 }
    }

    pub fn len(&self) -> usize { self.structures.len() }

    pub fn is_empty(&self) -> bool { self.structures.is_empty() }

    /// 放置新结构。
    pub fn place(&mut self, token_id: u32, x: f64, z: f64, strength: f64, builder_id: u32) -> u32 {
        let id = self.next_structure_id;
        self.next_structure_id += 1;
        let s = Structure::new(id, token_id, x, z, strength, 1.0, builder_id);
        self.structures.push(s);
        id
    }

    /// 衰减/衰败：无维护则 health 衰减。
    pub fn decay(&mut self, rate: f64) {
        for s in &mut self.structures {
            s.health -= s.health * rate;
            if s.health < 0.01 {
                s.health = 0.0;
            }
        }
        // 移除完全衰败的结构
        self.structures.retain(|s| s.health > 0.0);
    }

    /// 获取影响某位置的结构的局部调制值（token 向量 → 局部场调制）。
    /// 返回 (token_id, strength) 列表，由调用方根据 token 向量计算具体效果。
    pub fn modifiers_at(&self, x: f64, z: f64, radius: f64) -> Vec<(u32, f64)> {
        self.structures.iter()
            .filter(|s| {
                let dx = s.x - x;
                let dz = s.z - z;
                (dx * dx + dz * dz).sqrt() <= radius
            })
            .map(|s| (s.token_id, s.strength * s.health))
            .collect()
    }
}

/// 文化学习事件记录（用于测试/遥测）。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CultureStats {
    /// 总学习次数。
    pub learn_count: u64,
    /// 总创新事件数。
    pub innovation_count: u64,
    /// 总结构放置数。
    pub structure_place_count: u64,
    /// 结构维护总成本。
    pub structure_maintenance_total: f64,
    /// 学习总成本。
    pub learn_cost_total: f64,
    /// 失传 token 数。
    pub extinct_token_count: u64,
    /// 最近一次失传事件的 tick。
    pub last_extinct_tick: u32,
    /// 当前持有 token 的 agent 数（遥测）。
    pub token_holder_count: u64,
    /// Phase 43: 传承 token 总数（出生教学成功传递的 token 次数）。
    pub taught_count: u64,
    /// Phase 43: 教学总成本（亲代支付的 TEACH_COST 累计）。
    pub teach_cost_total: f64,
    /// Phase 45: 价值加权持有者数——Σ_alive Σ_token max(0, use_gain)。
    /// 文化健康的涌现度量：知识总量相同下，价值权重越高文化越「有效」。
    pub value_weighted_bearers: f64,
}

/// 三通道偏见枚举。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BiasChannel {
    /// content bias：内容偏好（token 向量与自身基因组相似度）。
    Content,
    /// prestige bias：声望（持有率高的个体被模仿概率高）。
    Prestige,
    /// conformist bias：从众（多数持有 token 被优先学）。
    Conformist,
}

/// 学习请求（一个 agent 尝试学习一个 token）。
/// 返回 (成功, 支付的能量)。
pub fn attempt_learn(
    learner_energy: &mut f64,
    token: &KnowledgeToken,
    genome_vec: &[f64],
    _fidelity: f64,
    _rng: &mut Mulberry32,
) -> bool {
    // (c, f) 框架：学习需要支付 c 能量。
    if *learner_energy < LEARNING_COST {
        return false;
    }
    *learner_energy -= LEARNING_COST;
    // 保真度影响学习效果：这里简化为学习成功概率（高保真更可能学对）。
    // 实际实现中，噪声在复制时加入，不在学习时。
    true
}

/// 从示范者复制 token 向量（保真度噪声）。
/// 噪声幅度 ∝ (1-f)，走专用 RNG 流。
pub fn copy_with_fidelity(
    source: &[f64],
    fidelity: f64,
    rng: &mut Mulberry32,
) -> Vec<f64> {
    let noise_amp = (1.0 - fidelity).max(0.0);
    source.iter().map(|&v| {
        let n = (rng.next_f64() * 2.0 - 1.0) * noise_amp;
        (v + n).clamp(-1.0, 1.0)
    }).collect()
}

/// 计算 content bias 偏好权重：token 与基因组相似度越高，越容易被学。
pub fn content_bias_weight(token: &KnowledgeToken, genome_vec: &[f64]) -> f64 {
    let dist = token.distance_to_vec(genome_vec);
    1.0 / (1.0 + dist) // 距离越小 → 权重越大
}

/// 计算 prestige bias 偏好权重：持有率高的个体被模仿概率高。
pub fn prestige_bias_weight(hold_rate: f64) -> f64 {
    0.5 + hold_rate
}

/// 计算 conformist bias 偏好权重：多数持有 token 被优先学。
pub fn conformist_bias_weight(pop_share: f64) -> f64 {
    if pop_share > 0.5 { 1.0 } else { 0.0 }
}

/// 计算三通道融合权重。
pub fn combined_bias_weight(
    token: &KnowledgeToken,
    genome_vec: &[f64],
    hold_rate: f64,
    pop_share: f64,
    mode: BiasChannel,
) -> f64 {
    match mode {
        BiasChannel::Content => content_bias_weight(token, genome_vec),
        BiasChannel::Prestige => prestige_bias_weight(hold_rate),
        BiasChannel::Conformist => conformist_bias_weight(pop_share),
    }
}

/// 把 token 向量映射为局部生态参数调制（通用 modifier，纯数据）。
/// token 向量的前两维映射到 (grass_growth_mod, moisture_mod)。
/// 语义由 token 向量决定，不硬编码类型。
pub fn token_to_env_mod(token: &KnowledgeToken) -> (f64, f64) {
    let v = &token.vec;
    let grass_mod = v.get(0).copied().unwrap_or(0.0) * STRUCTURE_MOD_STRENGTH;
    let moist_mod = v.get(1).copied().unwrap_or(0.0) * STRUCTURE_MOD_STRENGTH;
    (grass_mod, moist_mod)
}

/// 把 agent 基因组投影到 token 相似度空间（content bias 用）。
pub fn genome_to_vec(agent: &Agent) -> Vec<f64> {
    // 使用基因组核心性状的一个子集（速度/感知/体型/代谢/社交等）
    vec![
        agent.genome.speed,
        agent.genome.sense,
        agent.genome.size,
        agent.genome.metabolism,
        agent.genome.lifespan,
        agent.genome.bold,
        agent.genome.curious,
        agent.genome.social,
    ]
}

// ----- Phase 37: Culture Dynamics Engine -----

/// Agent 尝试从一个示范者学习 token。
/// 返回 (成功, 支付的能量)。
pub fn try_learn_token(
    agent: &mut Agent,
    target_id: u32,
    demoiser: &Agent,
    fidelity: f64,
    rng: &mut Mulberry32,
) -> bool {
    if !agent.alive || !demoiser.alive { return false; }
    if agent.known_tokens.contains(&target_id) { return false; }
    if agent.energy < LEARNING_COST { return false; }
    
    // 支付习得成本
    agent.energy -= LEARNING_COST;
    
    // 保真度噪声：复制时加噪，幅度 ∝ (1-f)
    let source_token = demoiser.known_tokens.iter()
        .find(|&&id| id == target_id);
    
    if source_token.is_some() {
        // 找到目标token在示范者身上，标记学习成功
        agent.known_tokens.push(target_id);
        true
    } else {
        // 示范者没有该token，但agent仍然支付了成本（试图学习）
        false
    }
}

/// Agent 进行重组创新：两个已知 token → 新 token。
/// 消耗专用 RNG 流。
pub fn try_innovate_token(
    agent: &mut Agent,
    registry: &mut crate::culture::CultureRegistry,
    rng: &mut Mulberry32,
) -> Option<u32> {
    if !agent.alive { return None; }
    if agent.known_tokens.len() < 2 { return None; }
    if rng.next_f64() > RECOMBINATION_P { return None; }
    if agent.energy < LEARNING_COST { return None; }
    
    // 支付创新成本
    agent.energy -= LEARNING_COST;
    
    // 随机选择两个已知 token
    let i1 = rng.next_f64() * agent.known_tokens.len() as f64;
    let i2 = rng.next_f64() * agent.known_tokens.len() as f64;
    let id1 = agent.known_tokens[i1 as usize];
    let id2 = agent.known_tokens[i2 as usize];
    
    // 重组 + 变异
    registry.recombine(id1, id2, rng)
}

/// Agent 放置一个物理结构（生态位构造）。
/// 返回是否成功。
pub fn try_place_structure(
    agent: &mut Agent,
    registry: &mut crate::culture::StructureRegistry,
    token_id: u32,
    x: f64,
    z: f64,
) -> bool {
    if !agent.alive { return false; }
    if agent.energy < STRUCTURE_COST { return false; }
    
    // 检查 agent 是否持有该 token
    if !agent.known_tokens.contains(&token_id) { return false; }
    
    // 支付放置成本
    agent.energy -= STRUCTURE_COST;
    
    // 从 token 向量提取强度参数（暂时使用默认值）
    let strength = 0.5;
    
    // 放置结构
    registry.place(token_id, x, z, strength, agent.id);
    true
}

/// Agent 维护其拥有的结构（每 tick 一次）。
pub fn maintain_structures(
    agent: &mut Agent,
    registry: &mut crate::culture::StructureRegistry,
    stats: &mut crate::culture::CultureStats,
) {
    if !agent.alive { return; }
    
    // 找出 agent 拥有/能维护的结构（简单起见：所有结构都收费）
    let n_structures = registry.structures.len();
    if n_structures == 0 { return; }
    
    // 每个结构的维护成本（按强度比例）
    let cost_per_struct = STRUCTURE_MAINTENANCE * n_structures as f64;
    if agent.energy < cost_per_struct {
        return; // 无法维护
    }
    
    agent.energy -= cost_per_struct;
    stats.structure_maintenance_total += cost_per_struct;
}

/// 结构自然衰败（无维护时衰减更快）。
pub fn decay_structures(registry: &mut crate::culture::StructureRegistry, rate: f64) {
    registry.decay(rate);
}

/// 应用结构对局部环境的调制效果。
/// 返回 (grass_mod_sum, moist_mod_sum) —— 累加到环境场。
pub fn apply_structure_effects(
    registry: &crate::culture::StructureRegistry,
    token_registry: &crate::culture::CultureRegistry,
    x: f64,
    z: f64,
) -> (f64, f64) {
    let modifiers = registry.modifiers_at(x, z, STRUCTURE_RADIUS);
    let mut grass_mod = 0.0;
    let mut moist_mod = 0.0;
    
    for (token_id, strength) in modifiers {
        if let Some(token) = token_registry.get(token_id) {
            let (gm, mm) = token_to_env_mod(token);
            grass_mod += gm * strength;
            moist_mod += mm * strength;
        }
    }
    
    (grass_mod, moist_mod)
}

/// 塔斯马尼亚式失传检查：找出不再被任何 agent 持有的 token。
pub fn check_cultural_extinction(
    agents: &[Agent],
    registry: &crate::culture::CultureRegistry,
    stats: &mut crate::culture::CultureStats,
) -> Vec<u32> {
    let holders: Vec<Vec<u32>> = agents.iter()
        .map(|a| a.known_tokens.clone())
        .collect();
    
    let extinct = registry.check_extinct(&holders);
    stats.extinct_token_count += extinct.len() as u64;
    stats.last_extinct_tick = 0; // could track this
    
    extinct
}

/// 计算当前持有 token 的 agent 数量（遥测）。
pub fn count_token_holders(agents: &[Agent]) -> u64 {
    agents.iter().filter(|a| !a.known_tokens.is_empty() && a.alive).count() as u64
}

// ---------------------------------------------------------------------------
// Phase 40.2: de-novo invention (the first-token pathway)
// ---------------------------------------------------------------------------
//
// Phase 37 shipped `try_innovate_token`, which only RECOMBINES two known
// tokens. With an empty registry there is no way to produce the first token —
// a deadlock the 100k-tick observatory confirmed (zero inventions in 300k
// simulated ticks). This adds the missing pathway: guided variation / trial
// and error. An explorer (high `curious` gene) with spare energy occasionally
// invents a token that encodes its current environment experience. It costs
// twice a learned token (trial-and-error is dearer than imitation) and the
// token vector is the environment signature plus small noise — so invented
// knowledge is coupled to where it was invented, giving niche construction a
// real gradient to work with. Zero preset semantics: the token means whatever
// its vector's effect on the environment turns out to be.

/// Per-agent per-tick probability of a de-novo invention attempt (before the
/// energy / curiosity gates). Sized so a 400-agent world sees a handful of
/// inventions per thousand ticks.
pub const INVENT_P: f64 = 0.000005;
/// Minimum energy fraction (of max) required to attempt invention.
pub const INVENT_MIN_ENERGY_FRAC: f64 = 0.70;
/// Minimum `curious` gene value for invention — explorers invent, settled
/// individuals imitate.
pub const INVENT_MIN_CURIOUS: f64 = 0.50;
/// Invention costs this multiple of LEARNING_COST (trial-and-error premium).
pub const INVENT_COST_MULT: f64 = 2.0;

/// Agent 进行 de-novo 发明（个体试错 / 指导性变异）：从其当前环境体验
/// 编码出一个新 token。这是首个 token 的唯一合法来源。
/// 消耗专用 RNG 流；`env_features` 由调用方从 agent 位置采样。
pub fn try_invent_token(
    agent: &mut Agent,
    registry: &mut CultureRegistry,
    env_features: &[f64; TOKEN_DIM],
    rng: &mut Mulberry32,
) -> Option<u32> {
    if !agent.alive { return None; }
    if agent.genome.curious < INVENT_MIN_CURIOUS { return None; }
    if agent.energy < agent.max_energy * INVENT_MIN_ENERGY_FRAC { return None; }
    // Fixed per-attempt draw budget: one roll for the attempt, then (only on
    // success) TOKEN_DIM draw for the noise.
    if rng.next_f64() >= INVENT_P { return None; }
    let cost = LEARNING_COST * INVENT_COST_MULT;
    if agent.energy < cost { return None; }
    agent.energy -= cost;
    let mut vec = Vec::with_capacity(TOKEN_DIM);
    for i in 0..TOKEN_DIM {
        let v = env_features[i] + (rng.next_f64() * 2.0 - 1.0) * 0.05;
        vec.push(v.clamp(-1.0, 1.0));
    }
    let id = registry.next_token_id;
    registry.next_token_id += 1;
    registry.register(KnowledgeToken::new(id, vec));
    agent.known_tokens.push(id);
    Some(id)
}

// ---------------------------------------------------------------------------
// Phase 43: cultural ratchet (parent → child token inheritance)
// ---------------------------------------------------------------------------
//
// Tomasello ratchet: children acquire accumulated culture through parental
// teaching; teaching is paid parental investment, and inheritance is an
// acquisition (DEFAULT_FIDELITY per-token roll: a failed roll = teaching
// failure / critical-period miss), not a copy. Without this pathway every
// population turnover resets culture to zero (the Phase 42 observatory saw
// bearers stuck at 0-5 despite 792 inventions).
//
// Determinism discipline: at most ONE culture_rng draw per (teacher, token)
// pair, scanned in the teacher's known_tokens order; the poverty break (can't
// afford the next lesson → teaching window closes) is zero-RNG. Duplicate
// tokens (already known by the child via the other parent) are skipped before
// any roll — no draw, no payment.

/// 出生传承：教师按自己的 known_tokens 顺序向 child 传授。
/// 每个 token：能量付不起 TEACH_COST → 停（贫穷约束，零 RNG）；
/// 已被孩子从另一位教师处习得 → 跳过（零 RNG，不重复付费）；
/// 否则 culture_rng 一次保真度 roll，成功则支付 TEACH_COST 并完整传承。
/// 返回本次传承的 token 数。
///
/// Phase 45 文化瓶颈：`value_on` 为 true 时候选按涌现价值降序排列
/// （高价值优先传承；同价值按 id 升序，确定性），且 use_gain > 0 的
/// token 传承保真度 +VALUE_FIDELITY_BONUS（上限 1.0）——好知识传得更早
/// 更准。教学成本不变；价值完全来自能量学的涌现反馈。
pub fn inherit_at_birth(
    child_tokens: &mut Vec<u32>,
    teacher_tokens: &[u32],
    teacher_energy: &mut f64,
    registry: &CultureRegistry,
    value_on: bool,
    rng: &mut Mulberry32,
    stats: &mut CultureStats,
) -> u64 {
    if teacher_tokens.is_empty() { return 0; }
    let order: Vec<u32> = if value_on {
        CultureRegistry::sorted_by_value(teacher_tokens, registry)
    } else {
        teacher_tokens.to_vec()
    };
    let mut taught = 0u64;
    for &tid in &order {
        if *teacher_energy < TEACH_COST { break; }
        if child_tokens.contains(&tid) { continue; }
        let fidelity = if value_on {
            (DEFAULT_FIDELITY
                + (registry.value_of(tid) > 0.0) as u8 as f64 * VALUE_FIDELITY_BONUS)
                .min(1.0)
        } else {
            DEFAULT_FIDELITY
        };
        if rng.next_f64() >= fidelity { continue; }
        *teacher_energy -= TEACH_COST;
        child_tokens.push(tid);
        taught += 1;
    }
    stats.taught_count += taught;
    stats.teach_cost_total += TEACH_COST * taught as f64;
    taught
}
