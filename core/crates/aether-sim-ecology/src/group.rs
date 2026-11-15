//! group.rs — Phase 39.1 层级跃迁层（Major Evolutionary Transition layer）。
//!
//! 三个已验证的最小触发条件（research/theory-scout-2026-11.md §Q2，
//! Goldsby/Knoester/Ofria/Kerr 2014 PLOS Biology；Goldsby/Dornhaus/Kerr/Ofria
//! 2012 PNAS）在本模块落到现有能量系统上：
//!
//! 1. **群体实体（group entity）** —— 同一 clade + 同一 lineage 且基因型相近的
//!    N≥3 个个体在半径 R 内持续 T ticks 后，按基因/环境决定的概率自发凝聚成组。
//!    `Group` 有独立的能量池、生死与繁殖；繁殖走**单点瓶颈**（propagule 只携带
//!    founder 的一份基因组，Godfrey-Smith 2011：无瓶颈的群体选择太弱）。
//! 2. **任务切换成本（task-switching cost）** —— 成员角色 GO/FEED/GUARD/REPRO
//!    由基因组 + 脑态（hidden 输出）的得分 argmax 决定；换角色要付能量。
//! 3. **脏活致突变（dirty-work mutagenesis）** —— GO 角色在污染/灾变/火区
//!    工作时累积体细胞损伤（`Agent.somatic_damage`），损伤抬高其后代的突变
//!    幅度；不做 GO 的角色不积累损伤 —— 生殖分工/体细胞-生殖细胞分化由此
//!    统计性地自发出现，而不是由任何 flag 指定。
//!
//! # 确定性纪律
//! * 专用 RNG 流 `seed ^ GROUP_RNG_TAG`（"GRP1"），只用于形成概率 + propagule
//!   落点抖动；从不与 main/prng/fish/culture 流混用。
//! * 条件未满足时**零副作用**：无 draw、无能量变化、无组生成；`group_on=false`
//!   的 A/B 世界必须逐位复现 Phase 36 锚点（测试钉住）。
//! * 所有浮点只做 IEEE 精确运算（+ − × ÷、clamp、比较）；距离平方比较免开方，
//!   正弦等非线性函数仍走 aether-libm（本模块没有需要它们的公式）。
//! * 群体死亡原因（饿死/成员全灭/超龄）是数据，不是脚本：能量池由 FEED 成员
//!   真实捐入、由维护与 REPRO 提取真实耗尽。

use crate::ecology::{Agent, Ecology, Genome, SpeciesKind, CATACLYSM_IMPACT};

/// 专用群体 RNG 流 tag："GRP1"。与 GEO/BRAIN/CULT/REG1 各 tag 互不碰撞。
pub const GROUP_RNG_TAG: u32 = 0x47_52_50_31;
/// 群体层总开关。默认 true（正式动力学阶段，预期 re-baseline）。
pub const GROUP_ON: bool = true;

// ----- 形成条件（Goldsby 2012 最小触发条件之一：亲缘聚集） -------------------
/// 成组所需最小成员数。
pub const GROUP_MIN_MEMBERS: usize = 3;
/// 单组成员上限（超过不再吸收新成员）。
pub const GROUP_MAX_MEMBERS: usize = 12;
/// 凝聚半径（世界单位）。
pub const GROUP_RADIUS: f64 = 12.0;
/// 凝聚扫描周期（ticks）。
pub const GROUP_SCAN_TICKS: u32 = 20;
/// 持续时长门槛（ticks）：聚簇必须连续出现在扫描里这么久才允许成组。
pub const GROUP_PERSIST_TICKS: u32 = 60;
/// 亲缘判定：hue（中性标记基因）最大差。超过即视为非亲缘、不成组。
pub const GROUP_KIN_HUE_TOL: f64 = 0.30;
/// 基础成组概率；实际概率 = 基础 × 社会性因子 × 亲缘因子 + 规模加成。
pub const GROUP_FORM_BASE_P: f64 = 0.45;
/// 成组时每个成员最多捐入初始池的能量（真实转移，绝不凭空铸造 ——
/// 账本闭合 C10 的纪律：池的每一分钱都来自某个个体的能量）。
pub const GROUP_POOL_START: f64 = 2.5;
/// 成组捐入时个体必须留下的能量比例。
pub const GROUP_POOL_CONTRIB_FLOOR: f64 = 0.30;

// ----- 角色分工 -------------------------------------------------------------
/// 未分配角色的哨兵值（新成员/离群个体）。
pub const ROLE_NONE: u8 = 0xFF;
pub const ROLE_GO: u8 = 0;
pub const ROLE_FEED: u8 = 1;
pub const ROLE_GUARD: u8 = 2;
pub const ROLE_REPRO: u8 = 3;
/// 角色名（telemetry / 报告用）。
pub const ROLE_NAMES: [&str; 4] = ["go", "feed", "guard", "repro"];
/// 角色重评周期（ticks）。分工随状态漂移，不随 tick 硬切。
pub const GROUP_ROLE_TICKS: u32 = 40;
/// 任务切换成本（能量）。换角色 = 重塑行为程序，真实付价。
pub const TASK_SWITCH_COST: f64 = 1.5;
/// 切换所需最低能量比例（能量过低时个体被"锁"在原角色上，先吃饱再改行）。
pub const SWITCH_MIN_ENERGY_FRAC: f64 = 0.25;
/// 脑态对角色得分的偏置幅度（hidden 节点输出的角色分组均值）。
pub const NEURAL_ROLE_BIAS: f64 = 0.12;

// ----- 群体能量学 -----------------------------------------------------------
/// 每成员每 sim-second 的池维护损耗。
pub const GROUP_POOL_MAINT: f64 = 0.004;
/// FEED 角色每 sim-second 向池捐出的能量。
pub const FEED_RATE: f64 = 0.30;
/// 捐献转化效率（其余为搬运损耗，记账意义上的"税"）。
pub const FEED_EFFICIENCY: f64 = 0.90;
/// FEED 成员不得把自己吃到这个能量比例以下。
pub const FEED_ENERGY_FLOOR: f64 = 0.30;
/// GUARD 角色每 sim-second 的警戒代谢费。
pub const GUARD_DRAIN: f64 = 0.06;
/// REPRO 角色每 sim-second 从池中提取的能量。
pub const REPRO_DRAW: f64 = 0.25;
/// 触发群体繁殖所需的最低池量。
pub const GROUP_REPRO_POOL: f64 = 10.0;
/// 池容量上限（倍数于繁殖门槛）：池满则 FEED 停止捐入，余粮留在觅食者身上
/// —— 否则池会变成无底能量黑洞，把个体饿死来供养死钱。
pub const GROUP_POOL_CAP_MULT: f64 = 2.0;
/// 群体繁殖一次性从池中扣除的成本。
pub const GROUP_REPRO_COST: f64 = 6.0;
/// 群体繁殖冷却（ticks）。
pub const GROUP_REPRO_COOLDOWN: u32 = 400;
/// propagule 大小（子群个体数，全部来自 founder 单点瓶颈）。
pub const GROUP_PROPAGULE_SIZE: usize = 3;
/// propagule 出生年龄。
pub const GROUP_PROPAGULE_AGE: f64 = 6.0;
/// 群体寿命上限（ticks）。
pub const GROUP_MAX_AGE: u32 = 4000;

// ----- 脏活致突变 -----------------------------------------------------------
/// GO 角色在单位危害下每 sim-second 累积的体细胞损伤。
pub const DIRTY_WORK_RATE: f64 = 0.05;
/// 体细胞损伤上限。
pub const SOMATIC_MAX: f64 = 1.0;
/// 体细胞损伤的自然修复速率（每 sim-second）。
pub const SOMATIC_DECAY: f64 = 0.0015;
/// 损伤 → 后代突变幅度的放大系数（scale = 1 + SOMATIC_MUTAGEN × damage）。
pub const SOMATIC_MUTAGEN: f64 = 1.5;
/// GUARD 对 GO 危害暴露的最大削减比例。
pub const GUARD_MITIGATION: f64 = 0.6;
/// 矿化场（分解者沉积 = 污染带）的单位危害权重。
pub const HAZARD_MINERAL_GAIN: f64 = 1.0;
/// 火区危害权重与半径。
pub const HAZARD_FIRE_GAIN: f64 = 0.5;
pub const HAZARD_FIRE_RADIUS: f64 = 8.0;

/// 一个群体实体：行为层聚合（不是多细胞发育模型）。
///
/// * `members` 是 agent id 列表（与 `roles` 平行，按加入顺序）。
/// * `founder_genome` 是繁殖瓶颈：不管哪个成员还活着，群体后代只来自这一份
///   基因组 + 少量变异（无混合基因组）。
/// * `energy_pool` 是群体级代谢池：FEED 捐入、维护/REPRO 支出、繁殖扣费。
#[derive(Clone, Debug)]
pub struct Group {
    pub id: u32,
    pub kind: SpeciesKind,
    pub members: Vec<u32>,
    pub founder_id: u32,
    pub energy_pool: f64,
    pub birth_tick: u32,
    pub parent_group: Option<u32>,
    /// 与 `members` 平行的角色表（`ROLE_*`）。
    pub roles: Vec<u8>,
    pub clade_id: u32,
    /// 单点瓶颈：群体繁殖只读取这一份基因组。
    pub founder_genome: Genome,
    pub founder_lineage: Option<&'static str>,
    /// 上次群体繁殖的 tick（冷却计时）。
    pub last_repro_tick: u32,
    /// propagule 代数（成组时 = founder.generation；子群 +1）。
    pub generation: u32,
}

impl Group {
    pub fn size(&self) -> usize { self.members.len() }

    pub fn role_counts(&self) -> [usize; 4] {
        let mut c = [0usize; 4];
        for &r in &self.roles {
            if (r as usize) < 4 { c[r as usize] += 1; }
        }
        c
    }
}

/// 一个持续存在的聚簇候选（跨扫描记忆）。只有同一个锚点个体（簇内最小 id）
/// 在连续扫描里一直出现，才把 `first_tick` 记住 —— 这是"持续 T ticks"的载体。
#[derive(Clone, Debug)]
pub struct CohesionTracker {
    pub anchor: u32,
    pub kind: SpeciesKind,
    pub clade_id: u32,
    pub first_tick: u32,
}

/// 群体层遥测（只读，不进 checksum、不进存档：observability 不是动力学）。
#[derive(Clone, Debug, Default)]
pub struct GroupStats {
    /// 自组织形成的群体数（de-novo）。
    pub formed: u64,
    /// 由群体繁殖产生的子群数（propagule）。
    pub propagules: u64,
    /// 累计解体的群体数 + 寿命累计（平均群体寿命 = lifetime_sum / lifetime_n）。
    pub dissolved: u64,
    pub lifetime_sum: f64,
    pub lifetime_n: u64,
    /// 解体原因分解。
    pub starved: u64,
    pub extinct: u64,
    pub aged: u64,
    /// 群体繁殖次数。
    pub reproductions: u64,
    /// propagule 个体总数。
    pub propagule_agents: u64,
    /// 角色切换次数 + 累计支付的能量。
    pub switches: u64,
    pub switch_cost_total: f64,
    /// 脏活事件：GO 成员在危害 > 0 的格子上的累积损伤。
    pub dirty_work_events: u64,
    pub somatic_total: f64,
    /// 池的收支（FEED 捐入 / REPRO+繁殖支出）。
    pub pool_in: f64,
    pub pool_out: f64,
    /// 时间序列（每 `GROUP_SAMPLE_TICKS` 一采样）。
    pub history: Vec<GroupSample>,
}

/// 群体层时间序列采样。
#[derive(Clone, Debug, Default)]
pub struct GroupSample {
    pub t: u32,
    pub groups: usize,
    pub mean_size: f64,
    pub max_size: usize,
    pub selection_levels: usize,
    pub pool_total: f64,
    pub mean_group_age: f64,
}

/// 采样周期（ticks）。
pub const GROUP_SAMPLE_TICKS: u32 = 100;

/// 选择层级数：个体层恒在（1），有活群体时再加 1（群体层）。
pub fn selection_levels(eco: &Ecology) -> usize {
    1 + usize::from(!eco.groups.is_empty())
}

/// 角色得分：基因组（速度/社会性/大胆/体型/帮助阈值/好奇/感知）+ 当前能量
/// 状态 + 群体池水位 + 脑态偏置。纯函数、零 RNG、零状态写。
pub fn role_scores(a: &Agent, pool_frac: f64) -> [f64; 4] {
    let speed_n = ((a.genome.speed - 0.4) / 1.6).clamp(0.0, 1.0);
    let sense_n = ((a.genome.sense - 14.0) / 116.0).clamp(0.0, 1.0);
    let size_n = ((a.genome.size - 0.58) / 1.12).clamp(0.0, 1.0);
    let help_n = a.genome.help_thr
        .map(|h| ((h + 5.0) / 10.0).clamp(0.0, 1.0))
        .unwrap_or(0.5);
    let e_frac = (a.energy / a.max_energy.max(1e-9)).clamp(0.0, 1.0);
    let mut s = [
        // GO：跑得快、不太社交、不太大胆、饿 → 出门干活。
        0.45 * speed_n + 0.25 * (1.0 - a.genome.social) + 0.20 * (1.0 - a.genome.bold)
            + 0.10 * (1.0 - e_frac),
        // FEED：社交 + 帮助阈值高 → 把食物搬回池子。
        0.45 * a.genome.social + 0.35 * help_n + 0.20 * (1.0 - speed_n),
        // GUARD：大胆 + 体型 + 有余粮 → 站岗（吃自己的能量换群体安全）。
        0.45 * a.genome.bold + 0.30 * size_n + 0.25 * e_frac,
        // REPRO：感知好、不瞎逛、饱、池子水位高 → 走生殖路线。
        0.30 * sense_n + 0.30 * (1.0 - a.genome.curious) + 0.25 * e_frac + 0.15 * pool_frac,
    ];
    // 脑态：hidden 节点输出按 index mod 4 分组，作为角色的神经偏置。
    // 这是"角色由脑态决定"的真实通路，不是随机数。
    if !a.brain.nodes.is_empty() && a.brain.n_hid > 0 {
        let mut acc = [0.0f64; 4];
        let mut cnt = [0.0f64; 4];
        for (i, node) in a.brain.nodes.iter().enumerate() {
            let r = i % 4;
            acc[r] += node.output;
            cnt[r] += 1.0;
        }
        for r in 0..4 {
            if cnt[r] > 0.0 {
                s[r] += NEURAL_ROLE_BIAS * (acc[r] / cnt[r]);
            }
        }
    }
    s
}

/// 当前环境危害 ∈ [0,1]：矿化场（分解者沉积的污染带）+ 活跃撞击灾变的热区
/// + 火区。纯读现有状态，零 RNG。
pub fn hazard_at(eco: &Ecology, x: f64, z: f64) -> f64 {
    let mut hz = 0.0f64;
    let ci = eco.cell_index(x, z);
    if ci >= 0 {
        if let Some(&v) = eco.mineral_field().get(ci as usize) {
            hz += v as f64 * HAZARD_MINERAL_GAIN;
        }
    }
    for c in &eco.cataclysms {
        if !c.active || c.kind != CATACLYSM_IMPACT { continue; }
        let dx = x - c.x;
        let dz = z - c.z;
        if dx * dx + dz * dz <= c.radius * c.radius {
            let frac = if c.dust_recovery_ticks > 0 {
                c.remaining as f64 / c.dust_recovery_ticks as f64
            } else {
                0.0
            };
            hz += c.severity * frac;
        }
    }
    for &(fx, fz) in eco.fires.iter() {
        let dx = x - fx;
        let dz = z - fz;
        if dx * dx + dz * dz <= HAZARD_FIRE_RADIUS * HAZARD_FIRE_RADIUS {
            hz += HAZARD_FIRE_GAIN;
        }
    }
    hz.clamp(0.0, 1.0)
}

/// 个体的后代突变幅度系数：体细胞损伤越高，后代突变越大。
pub fn mutation_scale(a: &Agent) -> f64 {
    1.0 + SOMATIC_MUTAGEN * a.somatic_damage.clamp(0.0, SOMATIC_MAX)
}

/// 每个 substep 调用一次。`group_on == false` → 完全 no-op。
pub fn group_step(eco: &mut Ecology, h: f64) {
    if !eco.group_on { return; }
    if eco.tick_count() % GROUP_SCAN_TICKS == 0 {
        scan_and_form(eco);
    }
    if !eco.groups.is_empty() {
        if eco.tick_count() % GROUP_ROLE_TICKS == 0 {
            reassign_roles(eco);
        }

        let idx_of = index_by_id(eco);
        let n_groups = eco.groups.len();
        let mut dissolve: Vec<(u32, u8)> = Vec::new(); // (group id, reason code)

        for gi in 0..n_groups {
            let gid = eco.groups[gi].id;
            // --- 1. 成员存活同步（死者出列，离群者出列） ---
            let mut keep_members: Vec<u32> = Vec::new();
            let mut keep_roles: Vec<u8> = Vec::new();
            for k in 0..eco.groups[gi].members.len() {
                let id = eco.groups[gi].members[k];
                let alive_member = match idx_of.get(&id) {
                    Some(&idx) => eco.agents[idx].alive && eco.agents[idx].group_id == gid,
                    None => false,
                };
                if alive_member {
                    keep_members.push(id);
                    keep_roles.push(eco.groups[gi].roles[k]);
                } else if let Some(&idx) = idx_of.get(&id) {
                    if eco.agents[idx].group_id == gid {
                        eco.agents[idx].group_id = 0;
                        eco.agents[idx].role = ROLE_NONE;
                    }
                }
            }
            if keep_members.len() != eco.groups[gi].members.len() {
                eco.groups[gi].members = keep_members;
                eco.groups[gi].roles = keep_roles;
            }
            if eco.groups[gi].members.is_empty() {
                dissolve.push((gid, REASON_EXTINCT));
                continue;
            }

            // --- 2. 池维护 ---
            let n = eco.groups[gi].members.len() as f64;
            eco.groups[gi].energy_pool -= GROUP_POOL_MAINT * n * h;

            // --- 3. 分工的即时经济学 ---
            let counts = eco.groups[gi].role_counts();
            let go_n = counts[ROLE_GO as usize] as f64;
            let mitigation = if go_n > 0.0 {
                (counts[ROLE_GUARD as usize] as f64 / go_n).min(1.0) * GUARD_MITIGATION
            } else {
                0.0
            };
            let member_ids: Vec<u32> = eco.groups[gi].members.clone();
            let member_roles: Vec<u8> = eco.groups[gi].roles.clone();
            for (k, &id) in member_ids.iter().enumerate() {
                let Some(&idx) = idx_of.get(&id) else { continue };
                if !eco.agents[idx].alive { continue; }
                match member_roles[k] {
                    ROLE_FEED => {
                        let cap = GROUP_REPRO_POOL * GROUP_POOL_CAP_MULT;
                        let room = (cap - eco.groups[gi].energy_pool).max(0.0);
                        let a = &mut eco.agents[idx];
                        let floor = a.max_energy * FEED_ENERGY_FLOOR;
                        let give = (FEED_RATE * h).min((a.energy - floor).max(0.0)).min(room);
                        if give > 0.0 {
                            a.energy -= give;
                            let into_pool = give * FEED_EFFICIENCY;
                            eco.groups[gi].energy_pool += into_pool;
                            eco.group_stats.pool_in += into_pool;
                        }
                    }
                    ROLE_GUARD => {
                        eco.agents[idx].energy -= GUARD_DRAIN * h;
                    }
                    ROLE_REPRO => {
                        let draw = (REPRO_DRAW * h).min(eco.groups[gi].energy_pool.max(0.0));
                        if draw > 0.0 {
                            eco.groups[gi].energy_pool -= draw;
                            eco.agents[idx].energy += draw;
                            eco.group_stats.pool_out += draw;
                        }
                    }
                    ROLE_GO => {
                        let hz = hazard_at(eco, eco.agents[idx].x, eco.agents[idx].z);
                        let dmg = DIRTY_WORK_RATE * hz * (1.0 - mitigation) * h;
                        if dmg > 0.0 {
                            let a = &mut eco.agents[idx];
                            a.somatic_damage = (a.somatic_damage + dmg).clamp(0.0, SOMATIC_MAX);
                            eco.group_stats.dirty_work_events += 1;
                            eco.group_stats.somatic_total += dmg;
                        }
                    }
                    _ => {}
                }
            }

            // --- 4. 群体死亡（饿死 / 超龄 / 成员全灭） ---
            let age = eco.tick_count().saturating_sub(eco.groups[gi].birth_tick);
            if eco.groups[gi].energy_pool <= 0.0 {
                dissolve.push((gid, REASON_STARVED));
                continue;
            }
            if age > GROUP_MAX_AGE {
                dissolve.push((gid, REASON_AGED));
                continue;
            }

            // --- 5. 群体繁殖（单点瓶颈 propagule） ---
            if eco.groups[gi].members.len() >= GROUP_MIN_MEMBERS
                && eco.groups[gi].energy_pool >= GROUP_REPRO_POOL
                && eco.tick_count().saturating_sub(eco.groups[gi].last_repro_tick) >= GROUP_REPRO_COOLDOWN
            {
                reproduce_group(eco, gi, &idx_of);
            }
        }

        for (gid, reason) in dissolve {
            dissolve_group(eco, gid, reason);
        }
        }

    // 体细胞损伤的自然修复（所有个体每 substep 一次，顺序固定 = agents 顺序）。
    for a in eco.agents.iter_mut() {
        if a.somatic_damage > 0.0 {
            a.somatic_damage = (a.somatic_damage - SOMATIC_DECAY * h).max(0.0);
        }
    }

    // 时间序列采样（纯遥测）。
    if eco.tick_count() % GROUP_SAMPLE_TICKS == 0 {
        let groups = eco.groups.len();
        let (mean_size, max_size, pool_total, mean_age) = if groups == 0 {
            (0.0, 0, 0.0, 0.0)
        } else {
            let mut sum = 0usize;
            let mut maxs = 0usize;
            let mut pool = 0.0f64;
            let mut age_sum = 0.0f64;
            for g in &eco.groups {
                sum += g.size();
                maxs = maxs.max(g.size());
                pool += g.energy_pool;
                age_sum += eco.tick_count().saturating_sub(g.birth_tick) as f64;
            }
            (
                sum as f64 / groups as f64,
                maxs,
                pool,
                age_sum / groups as f64,
            )
        };
        eco.group_stats.history.push(GroupSample {
            t: eco.tick_count(),
            groups,
            mean_size,
            max_size,
            selection_levels: selection_levels(eco),
            pool_total,
            mean_group_age: mean_age,
        });
        if eco.group_stats.history.len() > 240 {
            eco.group_stats.history.remove(0);
        }
    }
}

const REASON_STARVED: u8 = 0;
const REASON_EXTINCT: u8 = 1;
const REASON_AGED: u8 = 2;

fn index_by_id(eco: &Ecology) -> std::collections::HashMap<u32, usize> {
    let mut m = std::collections::HashMap::with_capacity(eco.agents.len());
    for (i, a) in eco.agents.iter().enumerate() {
        m.insert(a.id, i);
    }
    m
}

/// 扫描凝聚候选并尝试成组。零 RNG 直到某个聚簇通过"持续 T ticks"门槛。
pub fn scan_and_form(eco: &mut Ecology) {
    if !eco.group_on { return; }
    let tick = eco.tick_count();

    // 候选：活着的、尚未入组的个体，按 agents 顺序（= id 升序）。
    struct Cand {
        id: u32,
        kind: SpeciesKind,
        clade: u32,
        lineage: Option<&'static str>,
        x: f64,
        z: f64,
        hue: f64,
        social: f64,
    }
    let mut cands: Vec<Cand> = Vec::new();
    for a in &eco.agents {
        if !a.alive || a.group_id != 0 { continue; }
        cands.push(Cand {
            id: a.id,
            kind: a.kind,
            clade: a.clade_id,
            lineage: a.lineage,
            x: a.x,
            z: a.z,
            hue: a.genome.hue,
            social: a.genome.social,
        });
    }

    let mut seen = vec![false; eco.group_pending.len()];
    let mut used = vec![false; cands.len()];
    let mut forms: Vec<(Vec<u32>, SpeciesKind, u32, Option<&'static str>)> = Vec::new();

    for i in 0..cands.len() {
        if used[i] { continue; }
        // 贪心聚簇：同 kind + 同 clade + 同 lineage + 亲缘（hue 差）+ 半径内。
        let mut cluster: Vec<usize> = vec![i];
        for j in (i + 1)..cands.len() {
            if used[j] || cluster.len() >= GROUP_MAX_MEMBERS { continue; }
            if cands[j].kind != cands[i].kind || cands[j].clade != cands[i].clade { continue; }
            if cands[j].lineage != cands[i].lineage { continue; }
            if (cands[j].hue - cands[i].hue).abs() > GROUP_KIN_HUE_TOL { continue; }
            let dx = cands[j].x - cands[i].x;
            let dz = cands[j].z - cands[i].z;
            if dx * dx + dz * dz > GROUP_RADIUS * GROUP_RADIUS { continue; }
            cluster.push(j);
        }
        if cluster.len() < GROUP_MIN_MEMBERS { continue; }
        for &c in &cluster { used[c] = true; }
        let anchor = cands[cluster[0]].id; // 候选按 id 升序 → 簇内最小 id
        let kind = cands[cluster[0]].kind;
        let clade = cands[cluster[0]].clade;

        let pos = eco.group_pending.iter().position(|t| {
            t.anchor == anchor && t.kind == kind && t.clade_id == clade
        });
        match pos {
            Some(p) => {
                seen[p] = true;
                if tick.saturating_sub(eco.group_pending[p].first_tick) >= GROUP_PERSIST_TICKS {
                    // 通过持续门槛 → 才首次消费专用 RNG 流。
                    let n = cluster.len() as f64;
                    let mean_social = cluster.iter().map(|&c| cands[c].social).sum::<f64>() / n;
                    let mut lo = f64::INFINITY;
                    let mut hi = f64::NEG_INFINITY;
                    for &c in &cluster {
                        lo = lo.min(cands[c].hue);
                        hi = hi.max(cands[c].hue);
                    }
                    let spread = (hi - lo).clamp(0.0, 1.0);
                    let kin = (1.0 - spread / (GROUP_KIN_HUE_TOL * 2.0)).clamp(0.0, 1.0);
                    let size_bonus = (((cluster.len() - GROUP_MIN_MEMBERS) as f64)
                        / ((GROUP_MAX_MEMBERS - GROUP_MIN_MEMBERS) as f64)
                        * 0.5)
                        .clamp(0.0, 0.5);
                    let p = (GROUP_FORM_BASE_P * (0.4 + 0.6 * mean_social) * (0.5 + 0.5 * kin)
                        + size_bonus)
                        .clamp(0.0, 1.0);
                    let roll = eco.group_rng.next_f64();
                    if roll < p {
                        let ids: Vec<u32> = cluster.iter().map(|&c| cands[c].id).collect();
                        forms.push((ids, kind, clade, cands[cluster[0]].lineage));
                    }
                    // 未成组：保留 tracker，下一个扫描继续判定（first_tick 不回拨
                    // → 门槛只算一次，形成概率每次扫描重掷）。
                }
            }
            None => {
                eco.group_pending.push(CohesionTracker {
                    anchor,
                    kind,
                    clade_id: clade,
                    first_tick: tick,
                });
            }
        }
    }

    // 清理本轮没再出现的 tracker（聚簇散掉 = 持续性归零）。
    let mut kept: Vec<CohesionTracker> = Vec::with_capacity(eco.group_pending.len());
    for (i, t) in eco.group_pending.drain(..).enumerate() {
        if i < seen.len() {
            if seen[i] { kept.push(t); }
        } else {
            kept.push(t); // 本轮新登记
        }
    }
    eco.group_pending = kept;

    for (ids, kind, clade, lineage) in forms {
        form_group(eco, &ids, kind, clade, lineage);
    }
}

/// 用给定的成员 id 建立群体（实验者入口/形成内部路径共用）。
/// 返回新群体 id；成员不足或已被占用时返回 `None`。
pub fn form_group(
    eco: &mut Ecology,
    ids: &[u32],
    kind: SpeciesKind,
    clade_id: u32,
    lineage: Option<&'static str>,
) -> Option<u32> {
    if ids.len() < GROUP_MIN_MEMBERS { return None; }
    let idx_of = index_by_id(eco);
    let mut members = Vec::with_capacity(ids.len());
    for &id in ids {
        let idx = *idx_of.get(&id)?;
        let a = &eco.agents[idx];
        if !a.alive || a.group_id != 0 || a.kind != kind || a.clade_id != clade_id {
            return None;
        }
        members.push(id);
    }
    // 单点瓶颈的载体：锚点（最小 id）的基因组。
    let founder_idx = *idx_of.get(&members[0])?;
    let founder_genome = eco.agents[founder_idx].genome.clone();
    let founder_id = eco.agents[founder_idx].id;
    let generation = eco.agents[founder_idx].generation;

    // 初始池由成员真实捐入（feed the pool, don't mint it）：低于保留线的
    // 个体捐不出东西，池从 0 开始也是合法状态。
    let mut pool = 0.0f64;
    for &id in &members {
        if let Some(&idx) = idx_of.get(&id) {
            let floor = eco.agents[idx].max_energy * GROUP_POOL_CONTRIB_FLOOR;
            let give = GROUP_POOL_START.min((eco.agents[idx].energy - floor).max(0.0));
            eco.agents[idx].energy -= give;
            pool += give;
        }
    }
    let gid = eco.next_group_id.max(1);
    eco.next_group_id = gid + 1;
    let n = members.len();
    eco.groups.push(Group {
        id: gid,
        kind,
        members: members.clone(),
        founder_id,
        energy_pool: pool,
        birth_tick: eco.tick_count(),
        parent_group: None,
        roles: vec![ROLE_NONE; n],
        clade_id,
        founder_genome,
        founder_lineage: lineage,
        last_repro_tick: eco.tick_count(),
        generation,
    });
    let gi = eco.groups.len() - 1;
    for &id in &members {
        if let Some(&idx) = idx_of.get(&id) {
            eco.agents[idx].group_id = gid;
            eco.agents[idx].role = ROLE_NONE;
        }
    }
    eco.group_stats.formed += 1;
    assign_roles_for_group(eco, gi, &idx_of);
    Some(gid)
}

/// 实验者入口：替换某个体的基因组（受控 A/B 测试 / 研究 CLI 用；引擎内部
/// 从不调用）。角色得分、亲缘判定、繁殖瓶颈全部读这份基因组。
pub fn experimenter_set_genome(eco: &mut Ecology, id: u32, genome: Genome) -> bool {
    match eco.agents.iter().position(|a| a.id == id) {
        Some(idx) => {
            eco.agents[idx].genome = genome;
            true
        }
        None => false,
    }
}

/// 实验者入口：立即执行一次群体繁殖，跳过池量/冷却门槛（propagule 路径本身
/// 不变：仍然只读 `founder_genome`）。引擎内部从不调用。
pub fn experimenter_reproduce(eco: &mut Ecology, gid: u32) -> Option<u32> {
    let gi = eco.groups.iter().position(|g| g.id == gid)?;
    let idx_of = index_by_id(eco);
    let before = eco.groups.len();
    reproduce_group(eco, gi, &idx_of);
    if eco.groups.len() > before {
        eco.groups.last().map(|g| g.id)
    } else {
        None
    }
}

/// 角色重评（每 `GROUP_ROLE_TICKS`）。切换付费，付不起就留在原角色。
pub fn reassign_roles(eco: &mut Ecology) {
    let idx_of = index_by_id(eco);
    for gi in 0..eco.groups.len() {
        assign_roles_for_group(eco, gi, &idx_of);
    }
}

fn assign_roles_for_group(
    eco: &mut Ecology,
    gi: usize,
    idx_of: &std::collections::HashMap<u32, usize>,
) {
    let pool_frac = (eco.groups[gi].energy_pool / GROUP_REPRO_POOL).clamp(0.0, 1.0);
    let members = eco.groups[gi].members.clone();
    for (k, &id) in members.iter().enumerate() {
        let Some(&idx) = idx_of.get(&id) else { continue };
        if !eco.agents[idx].alive { continue; }
        let scores = role_scores(&eco.agents[idx], pool_frac);
        let mut best = 0usize;
        let mut best_v = scores[0];
        for r in 1..4 {
            if scores[r] > best_v {
                best_v = scores[r];
                best = r;
            }
        }
        let current = eco.groups[gi].roles.get(k).copied().unwrap_or(ROLE_NONE);
        if current == ROLE_NONE {
            eco.groups[gi].roles[k] = best as u8;
            eco.agents[idx].role = best as u8;
            continue;
        }
        if current as usize == best { continue; }
        // 换角色要付 task-switch cost；能量不足以重塑行为程序时被锁在原角色。
        let cost = TASK_SWITCH_COST;
        if eco.agents[idx].energy <= cost + eco.agents[idx].max_energy * SWITCH_MIN_ENERGY_FRAC {
            continue;
        }
        eco.agents[idx].energy -= cost;
        eco.agents[idx].role = best as u8;
        eco.groups[gi].roles[k] = best as u8;
        eco.group_stats.switches += 1;
        eco.group_stats.switch_cost_total += cost;
    }
}

/// 群体繁殖：单点瓶颈 propagule。
///
/// 所有子代基因组都来自 `group.founder_genome`（一份），各自独立变异；
/// 不存在成员基因组的混合。子群以 `parent_group = Some(父群 id)` 继承
/// 同一份 founder 基因组（瓶颈在世代间保持，而不是每代重抽一个成员）。
fn reproduce_group(
    eco: &mut Ecology,
    gi: usize,
    idx_of: &std::collections::HashMap<u32, usize>,
) {
    let kind = eco.groups[gi].kind;
    let clade_id = eco.groups[gi].clade_id;
    let founder_genome = eco.groups[gi].founder_genome.clone();
    let founder_id = eco.groups[gi].founder_id;
    let founder_lineage = eco.groups[gi].founder_lineage;
    let parent_id = eco.groups[gi].id;
    let generation = eco.groups[gi].generation + 1;
    let tick = eco.tick_count();

    // 群体级体细胞负荷 → 后代的突变幅度（脏活的跨代后果）。
    let mut dmg_sum = 0.0f64;
    let mut dmg_n = 0usize;
    let mut cx = 0.0f64;
    let mut cz = 0.0f64;
    for &id in &eco.groups[gi].members {
        if let Some(&idx) = idx_of.get(&id) {
            dmg_sum += eco.agents[idx].somatic_damage;
            dmg_n += 1;
            cx += eco.agents[idx].x;
            cz += eco.agents[idx].z;
        }
    }
    if dmg_n == 0 { return; }
    let mut_scale = 1.0 + SOMATIC_MUTAGEN * (dmg_sum / dmg_n as f64).clamp(0.0, SOMATIC_MAX);
    cx /= dmg_n as f64;
    cz /= dmg_n as f64;

    // Phase 43: 代际文化传承 —— founder 是教学者。存活的 founder 对每个
    // propagule 子代按自己的 known_tokens 顺序教学（每位孩子都学同一份
    // 课程；founder 能量逐孩子递减，付不起即对后续孩子停课）。founder
    // 已死/不在场则无传承（无教师）。
    let founder_teaching: Option<(Vec<u32>, f64)> = idx_of.get(&founder_id).and_then(|&fi| {
        if eco.agents[fi].alive && !eco.agents[fi].known_tokens.is_empty() {
            Some((eco.agents[fi].known_tokens.clone(), eco.agents[fi].energy))
        } else {
            None
        }
    });
    let (founder_tokens, mut founder_energy) = match founder_teaching {
        Some(x) => (x.0, x.1),
        None => (Vec::new(), 0.0),
    };
    let founder_teaches = eco.culture_on && eco.teach_on && !founder_tokens.is_empty();

    let mut ids: Vec<u32> = Vec::with_capacity(GROUP_PROPAGULE_SIZE);
    for _ in 0..GROUP_PROPAGULE_SIZE {
        let g = eco.mutate(&founder_genome, kind, mut_scale);
        let jx = (eco.group_rng.next_f64() - 0.5) * 4.0;
        let jz = (eco.group_rng.next_f64() - 0.5) * 4.0;
        let spawned = eco
            .spawn(kind, cx + jx, cz + jz, Some(g), generation, GROUP_PROPAGULE_AGE)
            .map(|a| a.id);
        if let Some(id) = spawned {
            if let Some(ai) = index_of_id(eco, id) {
                eco.agents[ai].clade_id = clade_id;
                eco.agents[ai].lineage = founder_lineage;
            }
            if founder_teaches {
                let child_idx = eco.agents.len() - 1;
                let mut child_tokens = std::mem::take(&mut eco.agents[child_idx].known_tokens);
                crate::culture::inherit_at_birth(
                    &mut child_tokens, &founder_tokens, &mut founder_energy,
                    &mut eco.culture_rng, &mut eco.culture_stats,
                );
                eco.agents[child_idx].known_tokens = child_tokens;
            }
            ids.push(id);
        }
    }
    // founder 教学支出结算回本体。
    if founder_teaches {
        if let Some(&fi) = idx_of.get(&founder_id) {
            if eco.agents[fi].alive {
                eco.agents[fi].energy = founder_energy;
            }
        }
    }
    if ids.is_empty() { return; }

    let gid = eco.next_group_id.max(1);
    eco.next_group_id = gid + 1;
    let n = ids.len();
    eco.groups.push(Group {
        id: gid,
        kind,
        members: ids.clone(),
        founder_id,
        // 子群的池从 0 起步：propagule 的生存要靠自己（以及它们的 FEED 角色）。
        energy_pool: 0.0,
        birth_tick: tick,
        parent_group: Some(parent_id),
        roles: vec![ROLE_NONE; n],
        clade_id,
        founder_genome,
        founder_lineage,
        last_repro_tick: tick,
        generation,
    });
    let child_gi = eco.groups.len() - 1;
    let idx_of_new = index_by_id(eco);
    for &id in &ids {
        if let Some(&idx) = idx_of_new.get(&id) {
            eco.agents[idx].group_id = gid;
            eco.agents[idx].role = ROLE_NONE;
        }
    }
    // 父群扣费 + 冷却复位。
    let paid = GROUP_REPRO_COST.min(eco.groups[gi].energy_pool).max(0.0);
    eco.groups[gi].energy_pool -= paid;
    // 池 → propagule 生物量：记进账本的繁殖支出一侧（池本身是内部转移）。
    eco.ledger.repro_loss += paid;
    eco.groups[gi].last_repro_tick = tick;
    eco.group_stats.reproductions += 1;
    eco.group_stats.propagules += 1;
    eco.group_stats.propagule_agents += n as u64;
    eco.group_stats.pool_out += GROUP_REPRO_COST;
    assign_roles_for_group(eco, child_gi, &idx_of_new);
}

fn index_of_id(eco: &Ecology, id: u32) -> Option<usize> {
    eco.agents.iter().position(|a| a.id == id)
}

fn dissolve_group(eco: &mut Ecology, gid: u32, reason: u8) {
    let Some(pos) = eco.groups.iter().position(|g| g.id == gid) else { return };
    let members = eco.groups[pos].members.clone();
    for id in members {
        if let Some(idx) = eco.agents.iter().position(|a| a.id == id) {
            if eco.agents[idx].group_id == gid {
                eco.agents[idx].group_id = 0;
                eco.agents[idx].role = ROLE_NONE;
            }
        }
    }
    let age = eco.tick_count().saturating_sub(eco.groups[pos].birth_tick) as f64;
    eco.groups.remove(pos);
    eco.group_stats.dissolved += 1;
    eco.group_stats.lifetime_sum += age;
    eco.group_stats.lifetime_n += 1;
    match reason {
        REASON_STARVED => eco.group_stats.starved += 1,
        REASON_EXTINCT => eco.group_stats.extinct += 1,
        _ => eco.group_stats.aged += 1,
    }
}
