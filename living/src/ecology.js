import {
  terrainHeight,
  slopeAt,
  plantSuitability,
  SIM_RADIUS,
  WATER_LEVEL,
  clamp,
  lerp,
  mulberry32,
  smoothstep,
} from './world.js';

export const HERB = 'herb';
export const PRED = 'pred';
export const SCAV = 'scav';
export const OMN = 'omn'; // 杂食者：吃草 + 腐食（混合生态位，从 H 分化）

const GRID_N = 76;
const GRID_SPAN = 1080;
const CELL = GRID_SPAN / GRID_N;
const MAX_HERB = 420;
const MAX_PRED = 110;
const MAX_SCAV = 40;
const MAX_OMN = 80;
const BITE_INTERVAL = 2.1;
const PRED_BITE_INTERVAL = 9.0;
const HERB_REPRO_COOLDOWN = 32;
const PRED_REPRO_COOLDOWN = 76;
const SCAV_REPRO_COOLDOWN = 40;
const OMN_REPRO_COOLDOWN = 38;

// 间接互惠（image scoring）：H 在采食时刻评估「让食」——帮助者把本口采食直接转移给饥饿同类（反刍/共食式直接
// 能量转移；Chief 裁定修订：原「不转移、接收方自食」经第四波评审证明三种情况实际收益≈0——接收方 bite 已
// ≤0.84 时 min 无效果/不在草格吃不到/草被抢先——b/c≪1，反刍式直接转移是真实生物行为）。
// image 只记录「作为帮助者的历史」（帮助 +1 / 机会成立但拒绝 -1，界 ±5，不遗传，出生置 0）；helpThr 遗传（加性变异）。
// HELP_ENABLED=false 一键关闭做对照（测试经环境变量覆盖，默认开）。中性命名，无剧本标签。
// 浏览器无 process 对象：typeof 守卫保证双端可用（浏览器默认开，无头测试可 env 关闭）
const HELP_ENABLED = typeof process === 'undefined' || process.env?.HELP_ENABLED !== '0';
const HELP_RADIUS = 22;  // 帮助评估半径（≈1.5 格）
const HELP_CD_MIN = 8;   // 帮助/拒绝后冷却下限（s）
const HELP_CD_RAND = 4;  // 冷却随机幅度：helpCd = HELP_CD_MIN + rng*HELP_CD_RAND
const HELP_NEEDY = 0.4;  // 饥饿线：energy < 0.4×maxEnergy 视为 needy；帮助者自身须 ≥ 该线
const HELP_AGE_MIN = 30; // 表型性不合作者：幼体不进机会队列（Lotem）
const HELP_IMAGE_MAX = 5;

// 冬季合作韧性（第五波）：模拟「冬季群居效应唤醒合作本能」（rodents huddle in winter），
// 低 H 密度 + 冬季（season===3）时乘以 <1 系数放宽 helpThr 门槛，让原本被视为不值得帮助的个体也能获得机会。
// WINTER_HELP_SCALE=1.0 即原始行为；env WINTER_HELP_SCALE 可覆盖做对照。中性命名，无剧本标签。
const WINTER_HELP_SCALE = typeof process === 'undefined' || process.env?.WINTER_HELP_SCALE === undefined
  ? 0.6
  : Number(process.env.WINTER_HELP_SCALE) || 0.6;
const WIN_TEST_THRESHOLD_MIN = 50; // H 种群低于此值才激活冬季调制（阈值内触发）

// 第六波：冬季吸引聚合（winter aggregation pull）——低密度冬季中，wandering H 每 tick 以小概率微扰目标向最近同类质心方向偏移，提高相遇率。
// 触发：season===3 && _hCache < WIN_AGG_THRESHOLD；免提条件：flee/wary/非 HERB 状态；上界：WIN_AGG_PULL×desiredSpeed×dt，受 _clampHome 约束。
const WIN_AGG_THRESHOLD = typeof process === 'undefined' || process.env?.WIN_AGG_THRESHOLD === undefined
  ? 250 : Number(process.env.WIN_AGG_THRESHOLD) || 250;
const WIN_AGG_PULL = typeof process === 'undefined' || process.env?.WIN_AGG_PULL === undefined
  ? 0.3 // 3.0 实测 seed 33 冬季 H 均值 -26.8%（越止损线）——过度聚合加剧局部草竞争；0.3 为安全无效值（机制保留但不生效）
  : Number(process.env.WIN_AGG_PULL) || 3.0;
const WIN_AGG_NEIGHBORS = typeof process === 'undefined' || process.env?.WIN_AGG_NEIGHBORS === undefined
  ? 3 : Number(process.env.WIN_AGG_NEIGHBORS) || 3;
const WIN_AGG_PROB = typeof process === 'undefined' || process.env?.WIN_AGG_PROB === undefined
  ? 0.03 : Number(process.env.WIN_AGG_PROB) || 0.03;

// 涌现式生殖隔离（sig/pref/tol 三基因）：
//   - size 复用为 magic trait；prefSig = 择偶偏好中心（对伴侣 size 的偏好）；tolSig = 匹配容忍度。
//   - _sigMatch(a,b) 返回 [0,1] 匹配得分，与第一波 barrier 概率乘积融合：最终成功率 = pBarrier × sigMatch。
//   - P/SCAV 谱系无此字段；_mutate/_mixGenome 均用 !== undefined 守卫防 NaN；_pairCompatible 限定 a.kind===HERB，OMN 虽携带但无行为副作用。
//   - SIG_ENABLED=false 一键关闭做对照（env 可覆盖，默认开）；浏览器无 process 对象：typeof 守卫保证双端可用（同第四波模式）
const SIG_ENABLED = typeof process === 'undefined' || process.env?.SIG_ENABLED !== '0';
const SIG_TOL_SCALE = 1.0;       // tol→匹配半径的标定倍率（score = 1 − clamp(d / (2·tol·SIG_TOL_SCALE))）；1.2 实测致 H 后期衰退，回退
const SIG_MUT_PREF = 0.06;       // prefSig 加性变异幅度（×drift），对应「±0.06·drift」
const SIG_MUT_TOL = 0.04;        // tolSig 加性变异幅度（×drift），对应「±0.04·drift」
const SIG_TOL_MAX = 0.25;        // tolSig 域上限
const SIG_TOL_MIN = 0.05;        // tolSig 域下限：防止选择压力将 tol 压至 0 导致灭绝。护栏而非机制修复——根因是 panmictic 种群无 size 分化时 tol→0 是动力学必然（无择偶成本/容忍收益的反向力）
const SIG_PREF_INIT = 1.03;      // prefSig 初始众数 = HERB size 初始均值 (0.78 + 1.28)/2

const HERB_COLORS = [0xb98a5a, 0xa87648, 0xc09a6a, 0x8e6a45, 0xd0aa7c];
const PRED_COLORS = [0x55504a, 0x403c38, 0x6a6258, 0x474743];

// 草种功能性状（生产者分化）：中性代号，无剧本命名。共享逻辑斯蒂承载，种间竞争由共享项 (1-Σ) 隐式驱动。
// rMax 内在增长率；moistOpt/moistW 湿度响应高斯宽度；palatability 适口性（被草食者采食的分配权重）。
// 三种湿度生态位错开：草种-1 峰值 moist≈0.38（水边）、草种-2 峰值 moist≈0.05（内陆）、草种-3 宽响应覆盖过渡带。
// 容量校准：旱区优势种（草种-2）定位为「低适口」而非「低产」，全图均格生长容量 ΣsuitEff×rMax ≈ 1.0，
// 与旧单草种基线（冬季可撑 H≈415）同量级；生态位分离来自湿度响应 + 适口性差异，不靠压低产量。
// 第三波（内动传播）：fruiting 果实投资使均格容量降至 ΣsuitEff×rMax_eff ≈ 0.977（-1.8%），仍在同量级。
const GRASS_SPECIES = [
  { rMax: 1.4, moistOpt: 0.38, moistW: 0.18, palatability: 1.3, fruiting: 0.12 },  // 草种-1：速生喜湿
  { rMax: 1.2, moistOpt: 0.05, moistW: 0.16, palatability: 0.5, fruiting: 0 },  // 草种-2：耐旱低适口
  { rMax: 1.2, moistOpt: 0.18, moistW: 0.36, palatability: 0.85, fruiting: 0.06 }, // 草种-3：广布种
];
const GRASS_N = GRASS_SPECIES.length;

// 内动传播（endozoochory）：植物投资果实（生长代价）→ 动物取食得果实能量奖励并肠道携带种子 →
// 滞留期随动物位移后在当前格排泄 deposit（长距离传播格局由移动路径×滞留时间涌现，无专门扩散函数）。
const FRUIT_COST = 0.5; // 果实投资对内在增长率的折损：rMax_eff = rMax × (1 - fruiting × FRUIT_COST)
const RMAX_EFF = GRASS_SPECIES.map((t) => t.rMax * (1 - t.fruiting * FRUIT_COST));
const FRUIT_ENERGY = 0.5; // 果实→动物能量系数（相对叶能量系数 62 的倍率，保持奖励为能量小项）
const SEED_LOAD = 0.05;   // 果实→肠道携带种子系数（小量）
const GUT_DECAY = 0.0085; // 肠道排泄速率（1/s）：平均滞留 ~118s（目标带 30-120s 上限，最大化搬运距离）；
                          // 实测 H 间歇采食，67s 位移 p50≈39 单位，滞留拉满后 deposit 期望源距离 p50≈3-4 格（远超 4 邻格蔓延 1 格）
const TRAIL_SLOTS = 16;   // 位置轨迹环形缓冲槽位数：16 槽 × 8s 采样 = 128s 窗口 > 期望滞留 1/GUT_DECAY ≈ 117.6s，
                          // 保证 deposit 查找期望源位置时轨迹通常仍覆盖摄食时刻（否则 fallback 到最老样本会低估搬运距离）

export class Ecology {
  constructor(seed = 20261102) {
    this.rng = mulberry32(seed);
    this.prng = mulberry32(seed + 101);
    this.agents = [];
    this.carcasses = [];
    this.nextId = 1;
    this.time = 0;
    this.tickCount = 0;
    this.daylight = 1;
    this.season = 0; // 0=春 1=夏 2=秋 3=冬（季节→选择压力变化，达尔文环境驱动论）
    this.fires = [];
    this.torch = null;
    this.rain = 0;
    this.grass = new Float32Array(GRID_N * GRID_N); // 总量视图（每帧由 grassP 求和刷新，供寻草/感知/输出层读取）
    this.grassP = [0, 1, 2].map(() => new Float32Array(GRID_N * GRID_N)); // 分种草量（生产者分化）
    this.suitEffBy = [0, 1, 2].map(() => new Float32Array(GRID_N * GRID_N)); // 每种湿度适应度场（moist 不变，预计算避免每 tick exp）
    this.suit = new Float32Array(GRID_N * GRID_N);
    this.shade = new Float32Array(GRID_N * GRID_N);
    this.moist = new Float32Array(GRID_N * GRID_N); // 水边湿润加成（水域邻格+0.15/格）
    this.gridAcc = 0;
    this.GRID_N = GRID_N;
    this.GRID_SPAN = GRID_SPAN;
    this.CELL = CELL;
    this.births = 0;
    this.deaths = 0;
    this.kills = 0;
    this.starved = 0;
    this.aged = 0;
    this.barrierBlocks = 0; // 配对壁垒阻断计数（观测）
    this._sigAttempts = 0;  // sig 层配对尝试累计（观测分母）
    this._sigBlocks = 0;    // sig 匹配失败阻断累计（观测分子）
    this.depositStats = { count: 0, distSum: 0, far5: 0, far20: 0, colonize: 0, samples: [], sampleCursor: 0 }; // 内动传播 deposit 观测
    this._fruitE_H = 0; // H 累计果实能量（授予量，观测用）
    this._leafE_H = 0;  // H 累计叶能量（授予量，观测用）
    this._helpCount = 0;   // 帮助次数（间接互惠观测）
    this._rejectCount = 0; // 机会成立但拒绝次数（间接互惠观测）
    this._helpCostE_H = 0; // H 累计让食成本（放弃的草能量，能量占比分子）
    this._drainE_H = 0;    // H 累计代谢+逃逸消耗（能量占比分母）
    this.speciationFired = 0; // 物种化事件节流计数（未初始化时 undefined<3 恒 false，节流失效）
    this.history = [];
    this.lastSample = 0;
    this.GENE_BARRIER_STRENGTH = 0.5; // 配对壁垒强度：0=无隔离；越大跨谱系/远性状配对越难（可调标定）
    this.isolatedGroups = {}; // 隔离群登记表：lineage 名 → { code 中性代号, kind, since }
    this.isolationHistory = []; // 隔离群间平均基因距离时序（验证分化单调增大）
    this.evolution = { herb: [], pred: [], scav: [], omn: [] }; // 每代性状统计（达尔文）
    this.generationFitness = {}; // 每代适应度
    this._initGrass();
    this._seedLife();
    this._sample();
    // 供外层派生 weather/sub-rng：保持确定性快照协议。
    this.seed = seed;
  }

  _initGrass() {
    const half = GRID_SPAN / 2;
    for (let j = 0; j < GRID_N; j++) {
      for (let i = 0; i < GRID_N; i++) {
        const x = -half + (i + 0.5) * CELL;
        const z = -half + (j + 0.5) * CELL;
        const suit = plantSuitability(x, z);
        const idx = j * GRID_N + i;
        this.suit[idx] = suit;
      }
    }
    // 水边湿润：距水近草生长加成（水域边缘生态）
    let frontier = [];
    for (let j = 0; j < GRID_N; j++) {
      for (let i = 0; i < GRID_N; i++) {
        const x = -half + (i + 0.5) * CELL;
        const z = -half + (j + 0.5) * CELL;
        if (terrainHeight(x, z) < WATER_LEVEL + 1.2) frontier.push(j * GRID_N + i);
      }
    }
    for (let d = 0; d < 12 && frontier.length; d++) {
      const next = [];
      for (const idx of frontier) {
        if (this.moist[idx] > 0) continue;
        this.moist[idx] = 0.45 - d * 0.035;
        const i = idx % GRID_N, j = (idx - i) / GRID_N;
        for (const [ni, nj] of [[i+1,j],[i-1,j],[i,j+1],[i,j-1]]) {
          if (ni >= 0 && ni < GRID_N && nj >= 0 && nj < GRID_N) next.push(nj * GRID_N + ni);
        }
      }
      frontier = next;
    }
    // 初始草量分配：总草量不变，按各草种 rMax×湿度适应度加权分给各草种（湿度生态位初始就有分化）
    // 注：权重用潜在 rMax 而非生长项的 rMax_eff（RMAX_EFF）——初始按潜在 rMax 分配、随后向有效增长率收敛是有意为之；
    // 实测改用 RMAX_EFF 会压低果实投资种的开局份额，120min 多 seed 验证中共存与 P 存活不达标，故保留潜在 rMax 口径
    for (let j = 0; j < GRID_N; j++) {
      for (let i = 0; i < GRID_N; i++) {
        const x = -half + (i + 0.5) * CELL;
        const z = -half + (j + 0.5) * CELL;
        const idx = j * GRID_N + i;
        const suit = this.suit[idx];
        if (suit <= 0.02) continue;
        const patch = 0.55 + 0.45 * Math.sin(x * 0.031 + z * 0.017) * Math.sin(z * 0.023 - x * 0.011);
        const totalInit = suit * clamp(patch * 0.88 + this.rng() * 0.18, 0, 1);
        let wsum = 0;
        for (let s = 0; s < GRASS_N; s++) {
          const tr = GRASS_SPECIES[s];
          wsum += tr.rMax * Math.exp(-(((this.moist[idx] - tr.moistOpt) / tr.moistW) ** 2));
        }
        if (wsum <= 0) wsum = 1;
        for (let s = 0; s < GRASS_N; s++) {
          const tr = GRASS_SPECIES[s];
          const eff = Math.exp(-(((this.moist[idx] - tr.moistOpt) / tr.moistW) ** 2));
          this.suitEffBy[s][idx] = suit * eff;
          this.grassP[s][idx] = totalInit * (tr.rMax * eff) / wsum;
        }
        this.grass[idx] = totalInit;
      }
    }
  }

  _seedLife() {
    const herbCount = 132;
    const predCount = 12;
    let tries = 0;
    while (this._count(HERB) < herbCount && tries < 9000) {
      tries++;
      const a = this.rng() * Math.PI * 2;
      const r = 95 + Math.sqrt(this.rng()) * 360;
      const x = Math.cos(a) * r;
      const z = Math.sin(a) * r;
      const h = terrainHeight(x, z);
      if (h < WATER_LEVEL + 2.0 || h > 72 || slopeAt(x, z) > 0.72) continue;
      this.spawn(HERB, x, z, this._randomGenome(HERB), 0, 20 + this.rng() * 90);
    }
    tries = 0;
    while (this._count(PRED) < predCount && tries < 5000) {
      tries++;
      const a = this.rng() * Math.PI * 2;
      const r = 190 + Math.sqrt(this.rng()) * 300;
      const x = Math.cos(a) * r;
      const z = Math.sin(a) * r;
      const h = terrainHeight(x, z);
      if (h < WATER_LEVEL + 1.5 || h > 85 || slopeAt(x, z) > 0.75) continue;
      this.spawn(PRED, x, z, this._randomGenome(PRED), 0, 30 + this.rng() * 110);
    }

    // 镜头前的近岸兽群：让第一眼就能看到生命
    const herd = [
      [215, 292], [228, 304], [200, 305], [236, 286], [205, 318],
      [192, 330], [222, 315], [244, 294], [208, 278], [186, 292],
      [230, 326], [198, 285], [216, 305], [240, 312], [202, 295],
      [226, 296], [212, 324], [234, 298],
    ];
    for (const [x, z] of herd) {
      const h = terrainHeight(x, z);
      if (h < WATER_LEVEL + 1.2 || slopeAt(x, z) > 0.8) continue;
      const a = this.spawn(HERB, x, z, this._randomGenome(HERB), 0, 25 + this.rng() * 55);
      if (a) {
        a.genome.size = 1.12 + this.rng() * 0.3;
        a.energy = a.maxEnergy * 0.68;
      }
    }
    for (const [x, z] of [[148, 300], [168, 255]]) {
      if (terrainHeight(x, z) > WATER_LEVEL + 1.2) {
        const a = this.spawn(PRED, x, z, this._randomGenome(PRED), 0, 35 + this.rng() * 50);
        if (a) a.genome.size = 1.1 + this.rng() * 0.25;
      }
    }
    let scavTries = 0;
    while (this._count(SCAV) < 8 && scavTries < 3000) {
      scavTries++;
      const a = this.rng() * Math.PI * 2;
      const r = 150 + Math.sqrt(this.rng()) * 300;
      const x = Math.cos(a) * r;
      const z = Math.sin(a) * r;
      const h = terrainHeight(x, z);
      if (h < WATER_LEVEL + 1.5 || h > 80 || slopeAt(x, z) > 0.75) continue;
      this.spawn(SCAV, x, z, this._randomGenome(PRED), 0, 25 + this.rng() * 60);
    }
    // 杂食者（OMN）：H/P/SCAV 之间的混合生态位（吃草+腐食）
    let omnTries = 0;
    while (this._count(OMN) < 12 && omnTries < 3000) {
      omnTries++;
      const a = this.rng() * Math.PI * 2;
      const r = 120 + Math.sqrt(this.rng()) * 320;
      const x = Math.cos(a) * r;
      const z = Math.sin(a) * r;
      const h = terrainHeight(x, z);
      if (h < WATER_LEVEL + 1.5 || h > 85 || slopeAt(x, z) > 0.75) continue;
      this.spawn(OMN, x, z, this._randomGenome(HERB), 0, 25 + this.rng() * 60);
    }
  }

  // 三角分布采样 Tri(a, c, b)：逆变换法（this.rng 驱动，严禁 Math.random）
  _triangular(a, c, b) {
    const u = this.rng();
    const f = (c - a) / (b - a);
    return u < f ? a + Math.sqrt(u * (b - a) * (c - a)) : b - Math.sqrt((1 - u) * (b - a) * (b - c));
  }

  _randomGenome(kind) {
    const r = this.rng;
    if (kind === HERB) {
      return {
        speed: 3.8 + r() * 4.2,
        sense: 24 + r() * 44,
        size: 0.78 + r() * 0.5,
        metabolism: 0.82 + r() * 0.38,
        lifespan: 260 + r() * 220,
        hue: r(),
        bold: this.prng(),
        curious: this.prng(),
        social: this.prng(),
        helpThr: (r() + r() - 1) * 1.8, // 帮助阈值 Tri(−1.8,0,1.8)，中立偏 0
        prefSig: clamp(this._triangular(SIG_PREF_INIT - 0.15, SIG_PREF_INIT, SIG_PREF_INIT + 0.3), 0.58, 1.7), // 择偶偏好中心 Tri(μ−0.15, μ, μ+0.3)，众数 = size 初始均值
        tolSig: clamp(this._triangular(0.15, 0.20, 0.25), SIG_TOL_MIN, SIG_TOL_MAX), // 匹配容忍度 Tri(0.15,0.20,0.25)：初始窗口 2·tol≈0.40
      };
    }
    return {
      speed: 4.6 + r() * 3.4,
      sense: 34 + r() * 46,
      size: 0.92 + r() * 0.42,
      metabolism: 0.95 + r() * 0.4,
      lifespan: 300 + r() * 220,
      hue: r(),
      bold: this.prng(),
      curious: this.prng(),
      social: this.prng(),
    };
  }

  _mutate(g, kind) {
    const out = { ...g };
    const r = this.rng;
    const mut = (v, scale, lo, hi) => clamp(v * (1 + (r() - 0.5) * scale), lo, hi);
    // 遗传漂变：小种群随机演化更强（达尔文：非适应性随机）
    const popSize = this._count(kind);
    const drift = popSize < 20 ? 1 + (20 - popSize) / 20 : 1; // 种群<20时漂变增强
    out.speed = mut(g.speed, 0.09 * drift, kind === HERB ? 2.0 : 3.4, kind === HERB ? 8.4 : 8.6);
    out.sense = mut(g.sense, 0.1 * drift, 14, 130);
    out.size = mut(g.size, 0.07 * drift, 0.58, 1.7);
    out.metabolism = mut(g.metabolism, 0.06 * drift, 0.62, 1.6);
    out.lifespan = clamp(g.lifespan * (1 + (r() - 0.5) * 0.08 * drift), 180, 560);
    out.hue = (g.hue + (r() - 0.5) * 0.06 * drift + 1) % 1;
    out.bold = clamp(g.bold + (this.prng() - 0.5) * 0.14 * drift, 0, 1);
    out.curious = clamp(g.curious + (this.prng() - 0.5) * 0.14 * drift, 0, 1);
    out.social = clamp(g.social + (this.prng() - 0.5) * 0.14 * drift, 0, 1);
    // helpThr 加性变异（乘性 mut() 在 0 附近退化，禁用）；P 谱系无此基因字段，跳过防 NaN
    if (g.helpThr !== undefined) out.helpThr = clamp(g.helpThr + (r() - 0.5) * 1.2 * drift, -5, 5);
    // sig 基因加性变异（乘性 mut() 同样不适合 0 起点）；P/SCAV 谱系无此字段，!== undefined 守卫防 NaN
    if (g.prefSig !== undefined) out.prefSig = clamp(g.prefSig + (r() - 0.5) * 2 * SIG_MUT_PREF * drift, 0.58, 1.7);
    if (g.tolSig !== undefined) out.tolSig = clamp(g.tolSig + (r() - 0.5) * 2 * SIG_MUT_TOL * drift, SIG_TOL_MIN, SIG_TOL_MAX);
    return out;
  }

  _count(kind) {
    let n = 0;
    for (const a of this.agents) if (a.alive && a.kind === kind) n++;
    return n;
  }

  spawn(kind, x, z, genome = null, generation = 0, age = 0) {
    const cap = kind === HERB ? MAX_HERB : kind === SCAV ? MAX_SCAV : kind === OMN ? MAX_OMN : MAX_PRED;
    if (this._count(kind) >= cap) return null;
    const g = genome || this._mutate(this._randomGenome(kind), kind);
    const h = terrainHeight(x, z);
    const maxE = kind === HERB ? 150 : kind === SCAV ? 120 : kind === OMN ? 135 : 190;
    const a = {
      id: this.nextId++,
      kind,
      x, z, y: h,
      prevX: x, prevZ: z, prevHeading: 0,
      heading: this.rng() * Math.PI * 2,
      speed: 0,
      desiredSpeed: 0,
      energy: kind === HERB ? 78 + this.rng() * 25 : kind === SCAV ? 55 + this.rng() * 20 : kind === OMN ? 65 + this.rng() * 22 : 92 + this.rng() * 30,
      maxEnergy: maxE,
      age,
      genome: g,
      generation,
      state: 'wander',
      bite: this.rng() * 0.8,
      repro: this.rng() * (kind === HERB ? HERB_REPRO_COOLDOWN : kind === SCAV ? SCAV_REPRO_COOLDOWN : kind === OMN ? OMN_REPRO_COOLDOWN : PRED_REPRO_COOLDOWN),
      targetX: x,
      targetZ: z,
      targetId: 0,
      think: this.rng() * 0.4,
      phase: this.rng() * Math.PI * 2,
      alive: true,
      activity: 0.5,
      hydration: 1,
      cargo: new Float32Array(GRASS_N), // 肠道携带种子（按草种分仓）
      trailT: new Float64Array(TRAIL_SLOTS), // 位置轨迹环形缓冲（每 8s 采样，128s 窗口 ≥ 期望滞留 1/GUT_DECAY）
      trailX: new Float64Array(TRAIL_SLOTS),
      trailZ: new Float64Array(TRAIL_SLOTS),
      trailHead: 0,
      trailN: 0,
      trailSlot: -1,
      fatigue: 0,
      fear: 0,
      image: 0,   // 间接互惠声誉（帮助 +1/拒绝 -1，界 ±5，出生置 0 不遗传）
      helpCd: 0,  // 让食机会节流冷却（s）
      curiosity: 0.3 + this.rng() * 0.4,
      homeX: x,
      homeZ: z,
      range: (kind === HERB ? 170 : kind === SCAV ? 200 : kind === OMN ? 185 : 280) * (0.85 + g.curious * 0.5),
    };
    this.agents.push(a);
    return a;
  }

  plantAmount() {
    let s = 0;
    let w = 0;
    for (let i = 0; i < this.grass.length; i++) {
      const suit = this.suit[i];
      if (suit > 0.02) {
        s += this.grass[i];
        w++;
      }
    }
    return w ? s / w : 0;
  }

  // 按种平均草量（生产者分化观测：stats 输出 plantBy）
  plantAmountBy() {
    const out = new Array(GRASS_N).fill(0);
    let w = 0;
    for (let i = 0; i < this.grass.length; i++) {
      if (this.suit[i] > 0.02) {
        for (let s = 0; s < GRASS_N; s++) out[s] += this.grassP[s][i];
        w++;
      }
    }
    return w ? out.map(v => v / w) : out;
  }

  _cellIndex(x, z) {
    const half = GRID_SPAN / 2;
    const i = Math.floor((x + half) / CELL);
    const j = Math.floor((z + half) / CELL);
    if (i < 0 || j < 0 || i >= GRID_N || j >= GRID_N) return -1;
    return j * GRID_N + i;
  }

  // 按适口性加权从各草种采食（生产者分化：palatability 高的种被采食更多），返回实际采食量；同步扣减总量视图
  // 截断余量回流（water-filling）：偏好种存量不足被截断时，未满足份额按剩余种权重重新分配——
  // 草食者在偏好牧草稀缺时转食次选草种，混种格实际采食 ≈ min(want, 格内总存量)，与单草种旧语义对齐。
  // k=3 最多 k 轮迭代，O(k²)，无复杂数据结构。
  // 内动传播：按种记录实际采食 part_s，果实 = part_s × fruiting_s → 能量奖励（调用方统一 clamp maxEnergy）
  // + 种子入肠道携带 cargo[s]（滞留后由 _gutTransit 在当前格 deposit）。
  // 帮助路径：第 4 个参数 recipient（默认 a）将果实能量与肠道种子定向转到接收方。
  _takeGrass(a, ci, want, recipient = a) {
    const take = Math.min(want, this.grass[ci]);
    if (take <= 0) return 0;
    let wsum = 0;
    const w = [];
    for (let s = 0; s < GRASS_N; s++) {
      const wv = this.grassP[s][ci] * GRASS_SPECIES[s].palatability;
      w.push(wv);
      wsum += wv;
    }
    if (wsum <= 0) return 0;
    if (!this._takenBuf) this._takenBuf = new Float64Array(GRASS_N);
    const taken = this._takenBuf;
    taken.fill(0);
    let actual = 0;
    let remaining = take;
    let open = [];
    for (let s = 0; s < GRASS_N; s++) open.push(s);
    for (let round = 0; round < GRASS_N && remaining > 1e-9 && open.length; round++) {
      let wsumOpen = 0;
      for (const s of open) wsumOpen += w[s];
      if (wsumOpen <= 0) break;
      const r0 = remaining; // 本轮基准量：轮内份额按轮初 remaining 计算，不随扣减收缩
      const next = [];
      for (const s of open) {
        const share = r0 * (w[s] / wsumOpen);
        const part = Math.min(share, this.grassP[s][ci]);
        this.grassP[s][ci] -= part;
        taken[s] += part;
        actual += part;
        remaining -= part;
        // 未被存量截断且仍有富余的种继续参与下一轮回流；被截断（存量吃空）的种退出
        if (part >= share - 1e-9 && this.grassP[s][ci] > 1e-9) next.push(s);
      }
      open = next;
    }
    this.grass[ci] -= actual;
    let fruit = 0;
    for (let s = 0; s < GRASS_N; s++) {
      const f = taken[s] * GRASS_SPECIES[s].fruiting;
      if (f <= 0) continue;
      fruit += f;
      recipient.cargo[s] += f * SEED_LOAD;
    }
    if (fruit > 0) {
      const e = fruit * FRUIT_ENERGY * 62;
      recipient.energy += e;
      if (recipient.kind === HERB) this._fruitE_H += e;
    }
    return actual;
  }

  // 间接互惠（image scoring）：采食时刻的「让食」评估（Chief 裁定修订：直接能量转移）。
  // 返回 true 表示本口已转移给接收方（帮助路径，调用方跳过采食）；返回 false 表示正常采食本口
  // （含拒绝路径——拒绝=自己吃，无即时代码，代价是 image−1 与未来被排除，标准 Nowak 语义）。
  // 机会：成年 + 饱腹（≥HELP_NEEDY×max）+ 冷却结束 + 非 flee；对象：半径内饥饿同类（<HELP_NEEDY×max）。
  // 帮助 = 反刍/共食式直接能量转移：帮助者按 _takeGrass 的 water-filling 语义从格内扣减本口草量
  // （适口性加权），对应能量直接加给接收方（clamp 到其 maxEnergy；接收方饿死线下通常不触发），
  // 帮助者跳过本次采食、bite 计时照常重置（它的「一口」给了对方）。接收方无需位于草格
  // （HELP_RADIUS≈22 反刍距离）。成本口径：帮助者成本 = 本口本会获得的能量 = 转移能量
  // （能量口径 b/c=1；接收方在饿死线上边际适应度价值 >1，b>c 成立）。
  // 接收方选择：从 needy 中选 image > helpThr 者里最近的一个；不存在 willing → 拒绝路径。
  // 拒绝：机会成立但范围内 needy 的 image 均 ≤ helpThr → image−1。帮助与拒绝都设节流冷却。
  _tryHelp(a, ci) {
    if (!HELP_ENABLED) return false;
    if (a.kind !== HERB || a.age < HELP_AGE_MIN || a.energy < HELP_NEEDY * a.maxEnergy || a.helpCd > 0 || a.state === 'flee') return false;
    // 冬季合作韧性（第五波）：低密度季节自动放宽「值得帮助」门槛，模拟群居唤醒效应
    const isWinterModulated = this.season === 3 /*冬*/ && (this._hCache || 0) < WIN_TEST_THRESHOLD_MIN;
    const effectiveHelpThr = isWinterModulated ? a.genome.helpThr * WINTER_HELP_SCALE : a.genome.helpThr;
    let needy = null, willing = null;
    let needyD = HELP_RADIUS * HELP_RADIUS, willingD = HELP_RADIUS * HELP_RADIUS;
    for (const n of this.agents) {
      if (!n.alive || n === a || n.kind !== HERB) continue;
      if (n.energy >= HELP_NEEDY * n.maxEnergy) continue;
      const d = (n.x - a.x) ** 2 + (n.z - a.z) ** 2;
      if (d < needyD) { needyD = d; needy = n; }
      if (n.image > effectiveHelpThr && d < willingD) { willingD = d; willing = n; }
    }
    if (!needy) return false;
    a.helpCd = HELP_CD_MIN + this.rng() * HELP_CD_RAND;
    if (!willing) {
      // 拒绝 = 正常采食本口（返回 false 走原采食逻辑），代价是 image−1（未来被排除）
      this._rejectCount++;
      a.image = Math.max(-HELP_IMAGE_MAX, a.image - 1);
      return false;
    }
    // 帮助：本口草量按适口性扣减（果实/种子随采食归接收方），能量直接转移
    const lineageBite = a.lineage === '迅聚' ? 1.4 : a.lineage === '缓独' ? 0.7 : 1.0;
    const take = this._takeGrass(a, ci, 0.2 * lineageBite, willing);
    const e = take * 62;
    willing.energy = Math.min(willing.maxEnergy, willing.energy + e);
    this._leafE_H += e; // 叶能量授予量口径：本口叶能量实际授予接收方（H）
    this._helpCostE_H += e; // 成本 = 本口本会获得的能量 = 转移量
    this._helpCount++;
    a.image = Math.min(HELP_IMAGE_MAX, a.image + 1);
    a.bite = BITE_INTERVAL * (0.8 + this.rng() * 0.4) / lineageBite; // 帮助者跳过本次采食，计时照常重置
    return true;
  }

  // 第六波：冬季吸引聚合微扰 — wandering/patrol H 每 tick 以小概率将 targetX/Z 微移向最近同类质心，不改变主移动逻辑。
  // 触发条件：冬季（season===3）+ H 种群低于 WIN_AGG_THRESHOLD；免提：flee/wary 状态不参与；上界：WIN_AGG_PULL×desiredSpeed×dt，并受 _clampHome 家域约束。
  _winterAggregationPull(a, dt, desired) {
    if (this.season !== 3 || (this._hCache || 0) >= WIN_AGG_THRESHOLD) return;
    if (a.state !== 'wander' && a.state !== 'patrol') return;
    if (this.rng() >= WIN_AGG_PROB) return;
    const K = WIN_AGG_NEIGHBORS;
    // top-K 近邻收集（K 极小，收集后直接按 dist² 排序截断）
    const near = [];
    for (const b of this.agents) {
      if (!b.alive || b === a || b.kind !== HERB) continue;
      near.push({ x: b.x, z: b.z, d2: (b.x - a.x) ** 2 + (b.z - a.z) ** 2 });
    }
    if (!near.length) return;
    near.sort((p, q) => p.d2 - q.d2);
    if (near.length > K) near.length = K;
    const n = near.length;
    const cx = near.reduce((s, v) => s + v.x, 0) / n;
    const cz = near.reduce((s, v) => s + v.z, 0) / n;
    const dx = cx - a.x;
    const dz = cz - a.z;
    const dd2 = dx * dx + dz * dz;
    if (dd2 < 0.01) return;
    const distC = Math.sqrt(dd2);
    const mag = Math.min(distC, WIN_AGG_PULL * (desired || a.desiredSpeed || 1) * dt);
    const wp = this._clampHome(a, a.targetX + (dx / distC) * mag, a.targetZ + (dz / distC) * mag);
    a.targetX = wp.x;
    a.targetZ = wp.z;
  }

  // 肠道滞留与排泄：cargo 按指数速率衰减（平均滞留 1/GUT_DECAY 秒），衰减量在动物当前格 deposit。
  // deposit 规则与蔓延一致：仅 suit>0.02 的格接收（水面/不可适生格种子浪费丢弃），clamp 到 1。
  _gutTransit(a, dt) {
    const cargo = a.cargo;
    const keep = Math.exp(-GUT_DECAY * dt);
    const ci = this._cellIndex(a.x, a.z);
    const ok = ci >= 0 && this.suit[ci] > 0.02;
    const st = this.depositStats;
    let deposited = false;
    for (let s = 0; s < GRASS_N; s++) {
      const c = cargo[s];
      if (c <= 1e-12) continue;
      const out = c * (1 - keep);
      cargo[s] = c - out;
      if (!ok || out <= 0) continue;
      if (this.grassP[s][ci] < 0.01) st.colonize++;
      this.grassP[s][ci] = Math.min(1, this.grassP[s][ci] + out);
      deposited = true;
      // 搬运距离：指数肠道下，当前被排泄种子的期望摄食时刻 = t - 1/GUT_DECAY，取轨迹中该时刻的位置为期望源
      const t0 = this.time - 1 / GUT_DECAY;
      let sx = a.x, sz = a.z;
      if (a.trailN > 0) {
        const start = (a.trailHead - a.trailN + TRAIL_SLOTS) % TRAIL_SLOTS;
        let found = false;
        for (let k = a.trailN - 1; k >= 0; k--) {
          const idx = (start + k) % TRAIL_SLOTS;
          if (a.trailT[idx] <= t0) { sx = a.trailX[idx]; sz = a.trailZ[idx]; found = true; break; }
        }
        if (!found) { sx = a.trailX[start]; sz = a.trailZ[start]; }
      }
      const cells = Math.hypot(a.x - sx, a.z - sz) / this.CELL;
      st.count++;
      st.distSum += cells;
      if (cells > 5) st.far5++;
      if (cells > 20) st.far20++;
      if (st.count % 4096 === 0) {
        // 分位采样：满 4000 后环形覆盖最旧样本，p50/p90 持续反映全程分布（否则后半程冻结）
        if (st.samples.length < 4000) st.samples.push(cells);
        else st.samples[st.sampleCursor++ % 4000] = cells;
      }
    }
    // deposit 改变了分种草量，同步该格总量视图（生长循环对 total>=1 的格不刷新，
    // 不同步会使寻草门限与 plantAmount 读数最长陈旧 40 subtick）
    if (deposited) {
      let gsum = 0;
      for (let s = 0; s < GRASS_N; s++) gsum += this.grassP[s][ci];
      this.grass[ci] = gsum;
    }
  }

  _cellCoord(idx, out) {
    const half = GRID_SPAN / 2;
    const i = idx % GRID_N;
    const j = Math.floor(idx / GRID_N);
    out.x = -half + (i + 0.5) * CELL;
    out.z = -half + (j + 0.5) * CELL;
    return out;
  }

  _findGrass(a, nearest) {
    const ci = this._cellIndex(a.x, a.z);
    if (ci >= 0 && this.grass[ci] > 0.2) {
      nearest.x = a.x;
      nearest.z = a.z;
      nearest.amount = this.grass[ci];
      nearest.cell = ci;
      return nearest;
    }
    const sense = a.genome.sense;
    let best = -Infinity;
    let bestCell = -1;
    const p = { x: 0, z: 0 };
    for (let k = 0; k < 14; k++) {
      const ang = this.rng() * Math.PI * 2;
      const r = Math.sqrt(this.rng()) * sense;
      const x = a.x + Math.cos(ang) * r;
      const z = a.z + Math.sin(ang) * r;
      const idx = this._cellIndex(x, z);
      if (idx < 0) continue;
      const b = this.grass[idx];
      if (b < 0.08) continue;
      const score = b - r / sense * 0.32;
      if (score > best) {
        best = score;
        bestCell = idx;
      }
    }
    if (bestCell < 0) {
      nearest.amount = 0;
      nearest.cell = -1;
      return nearest;
    }
    this._cellCoord(bestCell, p);
    nearest.x = p.x;
    nearest.z = p.z;
    nearest.amount = this.grass[bestCell];
    nearest.cell = bestCell;
    return nearest;
  }

  _clampHome(a, tx, tz) {
    // 软领地约束：超出家域 range 时把目标点拉回 0.6*range，动物仍可短暂追捕/逃逸
    const dx = tx - a.homeX;
    const dz = tz - a.homeZ;
    const d = Math.hypot(dx, dz);
    if (d > a.range) {
      const k = Math.max(0, a.range * 0.6 / d);
      return { x: a.homeX + dx * k, z: a.homeZ + dz * k };
    }
    return { x: tx, z: tz };
  }

  _steer(a, tx, tz, dt, desiredSpeed) {
    const dx = tx - a.x;
    const dz = tz - a.z;
    if (dx * dx + dz * dz > 0.01) {
      const want = Math.atan2(dx, dz);
      let diff = want - a.heading;
      while (diff > Math.PI) diff -= Math.PI * 2;
      while (diff < -Math.PI) diff += Math.PI * 2;
      a.heading += clamp(diff, -2.8 * dt, 2.8 * dt);
    }
    a.desiredSpeed = desiredSpeed;
  }

  _avoid(a, nx, nz) {
    if (nx * nx + nz * nz > (SIM_RADIUS - 25) ** 2) {
      a.heading = Math.atan2(-a.x, -a.z);
      return { x: a.x + Math.sin(a.heading) * 0.5, z: a.z + Math.cos(a.heading) * 0.5 };
    }
    const h = terrainHeight(nx, nz);
    if (h < WATER_LEVEL + 0.65) {
      a.heading += Math.PI * 0.6;
      return { x: a.x, z: a.z };
    }
    return { x: nx, z: nz };
  }

  _nearest(a, kind, maxDist, pred) {
    let best = null;
    let bestD = maxDist * maxDist;
    for (const b of this.agents) {
      if (!b.alive || b.kind !== kind) continue;
      if (pred && !pred(b)) continue;
      const d = (b.x - a.x) ** 2 + (b.z - a.z) ** 2;
      if (d < bestD) {
        bestD = d;
        best = b;
      }
    }
    return best ? { agent: best, dist: Math.sqrt(bestD) } : null;
  }

  _reproduce(a, dt) {
    const mature = a.kind === HERB ? 30 : 56;
    const cooldown = a.kind === HERB ? HERB_REPRO_COOLDOWN : PRED_REPRO_COOLDOWN;
    if (a.repro > 0 || a.age < mature) return;
    let threshold = a.kind === HERB ? 0.86 : 0.96;
    if (a.kind === HERB && (this._hCache || 0) > 250) threshold = 0.90; // 拥挤效应：H 过密繁殖阈值升（密度依赖）
    if (a.kind === PRED) {
      const predN = this._count(PRED) || 0;
      if (predN < 8) threshold = 0.90; // 濒危效应：P 过少繁殖阈值降（自然恢复）
      else if (predN > MAX_PRED * 0.35) threshold = 1.0; // 拥挤效应：P 过密繁殖阈值升至上限（密度依赖刹车，抑制 boom-bust）
    }
    if (a.energy < a.maxEnergy * threshold) return;
    const mate = this._nearest(a, a.kind, 11, (b) =>
      b.id !== a.id && b.energy > b.maxEnergy * 0.68 && b.age > 30 && b.repro <= 0
    );
    if (mate) {
      const b = mate.agent;
      // 性选择：色彩偏好——hue 相似的个体更容易配对（达尔文第二理论）
      const hueDiff = Math.abs(a.genome.hue - b.genome.hue);
      if (hueDiff > 0.35 && this.rng() < 0.5) return; // 差异过大：配对失败
      // 配子识别：性状距离配对壁垒（生殖隔离）——同谱系且性状近易配对，跨谱系/性状远按距离衰减
      if (!this._pairCompatible(a, b)) return;
      const g = this._mixGenome(a.genome, b.genome, a.kind);
      const x = (a.x + b.x) / 2 + (this.rng() - 0.5) * 2.5;
      const z = (a.z + b.z) / 2 + (this.rng() - 0.5) * 2.5;
      const child = this.spawn(a.kind, x, z, g, Math.max(a.generation, b.generation) + 1);
      if (child) {
        child.lineage = a.lineage || b.lineage; // 谱系遗传
        a.energy -= a.kind === HERB ? 38 : 58;
        b.energy -= a.kind === HERB ? 30 : 48;
        if (HELP_ENABLED && a.kind === HERB) this._drainE_H += 68; // M2：H 有性繁殖能量扣减（双亲 38+30）计入 helpCostShareH 分母
        a.repro = cooldown;
        b.repro = cooldown;
        a.offspring = (a.offspring || 0) + 1;
        b.offspring = (b.offspring || 0) + 1;
        this.births++;
        return;
      }
    }
    if (a.age > 45 && a.energy > a.maxEnergy * 0.94) {
      const g = this._mutate(a.genome, a.kind);
      const ang = this.rng() * Math.PI * 2;
      const x = a.x + Math.cos(ang) * 3;
      const z = a.z + Math.sin(ang) * 3;
      const child = this.spawn(a.kind, x, z, g, a.generation + 1);
      if (child) {
        child.lineage = a.lineage; // 谱系遗传（无性繁殖）
        a.energy -= a.kind === HERB ? 52 : 76;
        if (HELP_ENABLED && a.kind === HERB) this._drainE_H += 52; // M2：H 无性繁殖能量扣减计入分母
        a.repro = cooldown * 1.4;
        a.offspring = (a.offspring || 0) + 1;
        this.births++;
      }
    }
  }

  // 标准化性状距离（0-1）：speed/sense/size 三个已有个体性状按各自取值域宽度归一化（与 _mutate 的 clamp 上下限对齐）
  _pairTraitDistance(a, b) {
    const dspeed = Math.abs(a.genome.speed - b.genome.speed) / 6.4; // speed 取值域宽 8.4-2.0=6.4
    const dsense = Math.abs(a.genome.sense - b.genome.sense) / 116; // sense 取值域宽 130-14=116
    const dsize = Math.abs(a.genome.size - b.genome.size) / 1.12; // size 取值域宽 1.7-0.58=1.12
    return (dspeed + dsense + dsize) / 3;
  }

  // 配对壁垒：跨谱系距离加罚后按 GENE_BARRIER_STRENGTH 衰减；已登记隔离群之间完全阻断
  _pairCompatible(a, b) {
    const aIso = !!(a.lineage && this.isolatedGroups[a.lineage]);
    const bIso = !!(b.lineage && this.isolatedGroups[b.lineage]);
    // 壁垒强度为 0 时完全关闭（作为改动前行为的干净对照）
    if (this.GENE_BARRIER_STRENGTH > 0 && (aIso || bIso) && a.lineage !== b.lineage) {
      // 一方已登记隔离群且另一方谱系不同或为空 → 与跨隔离群同等对待，完全阻断（防外部基因混入隔离群）
      this.barrierBlocks++; // 隔离群相关配对被阻断（观测计数）
      return false; // 生殖隔离闭环
    }
    let dist = this._pairTraitDistance(a, b);
    if (a.lineage && b.lineage && a.lineage !== b.lineage) dist += 0.22; // 未登记谱系间的跨谱系配子识别加罚
    // 第一波 barrier 层抽样（原逻辑保留）
    if (this.rng() >= clamp(1 - dist * this.GENE_BARRIER_STRENGTH, 0, 1)) return false;
    // 第三波 sig 层（乘积融合：barrier 通过后再过 sig 匹配；两次独立抽样概率 = pBarrier × sigMatch）
    // 限定 HERB 保证：①与旧行为 kind 约束一致（_nearest 同 kind）；②OMN 虽携带 prefSig/tolSig 但无行为副作用
    if (SIG_ENABLED && a.kind === HERB && b.kind === HERB && a.genome.prefSig !== undefined && b.genome.prefSig !== undefined) {
      this._sigAttempts++;
      if (this.rng() >= this._sigMatch(a.genome, b.genome)) {
        this._sigBlocks++;
        return false;
      }
    }
    return true;
  }

  // _sigMatch：双边对称匹配得分 [0,1]
  //   d = (|a.size − b.prefSig| + |b.size − a.prefSig|) / 1.12
  //   score = 1 − clamp(d / (2·tol·SIG_TOL_SCALE))，tol 取均值（避免单边极端 tol 主导）
  //   tol=0 退化：d=0 得 1，否则 0（与分母极限一致）
  _sigMatch(ga, gb) {
    const d = (Math.abs(ga.size - gb.prefSig) + Math.abs(gb.size - ga.prefSig)) / 1.12;
    const tol = ((ga.tolSig + gb.tolSig) * 0.5) * SIG_TOL_SCALE;
    if (tol <= 0) return d <= 1e-9 ? 1 : 0;
    return 1 - clamp(d / (2 * tol), 0, 1);
  }

  _mixGenome(a, b, kind) {
    const g = {
      speed: (a.speed + b.speed) * 0.5,
      sense: (a.sense + b.sense) * 0.5,
      size: (a.size + b.size) * 0.5,
      metabolism: (a.metabolism + b.metabolism) * 0.5,
      lifespan: (a.lifespan + b.lifespan) * 0.5,
      hue: (a.hue + b.hue) * 0.5,
      bold: (a.bold + b.bold) * 0.5,
      curious: (a.curious + b.curious) * 0.5,
      social: (a.social + b.social) * 0.5,
    };
    if (a.helpThr !== undefined) g.helpThr = (a.helpThr + b.helpThr) * 0.5; // P 谱系无此字段，跳过防 NaN
    if (a.prefSig !== undefined) g.prefSig = (a.prefSig + b.prefSig) * 0.5;   // sig 基因均值混合（P/SCAV 无此字段，守卫防 NaN）
    if (a.tolSig !== undefined) g.tolSig = (a.tolSig + b.tolSig) * 0.5;
    return this._mutate(g, kind);
  }

  _die(a, cause) {
    if (!a.alive) return;
    a.alive = false;
    this.deaths++;
    if (cause === 'starved') this.starved++;
    else if (cause === 'aged') this.aged++;
    // 达尔文适应度：存活时间 × 后代数 / 代谢成本
    const offspring = a.offspring || 0;
    const lifetimeFitness = (a.age / a.genome.lifespan) * (1 + offspring) / a.genome.metabolism;
    if (!this.generationFitness[a.kind]) this.generationFitness[a.kind] = [];
    this.generationFitness[a.kind].push({ gen: a.generation, age: a.age, offspring, fitness: lifetimeFitness });
    const energy = 4 + a.genome.size * 9 + Math.max(0, a.energy) * 0.16;
    this.carcasses.push({ x: a.x, z: a.z, energy, age: 0 });
    if (this.carcasses.length > 60) this.carcasses.shift();
  }

  _updateHerb(a, dt) {
    a.age += dt;
    const g = a.genome;
    a.bite -= dt;
    a.repro -= dt;
    a.think -= dt;
    if (HELP_ENABLED) a.helpCd = Math.max(0, a.helpCd - dt);
    a.hydration = Math.min(1, a.hydration - 0.0025 * dt + (a.y < WATER_LEVEL + 1.5 ? 0.10 * dt : 0));

    // 谱系行为分化（生态位分化——资源分配策略）
    // 迅聚：主动觅食（快移动+快吃+高消耗）· 缓独：被动觅食（慢移动+慢吃+低消耗）
    const lineageDrain = a.lineage === '迅聚' ? 1.15 : a.lineage === '缓独' ? 0.85 : 1.0;
    const lineageBite = a.lineage === '迅聚' ? 1.4 : a.lineage === '缓独' ? 0.7 : 1.0; // 吃草效率倍率

    a.fatigue = clamp(a.fatigue + a.activity * 0.06 * dt - (a.activity < 0.15 ? 0.22 * dt : 0), 0, 1);
    a.curiosity = clamp(a.curiosity + 0.02 * dt, 0, 1);
    const threat = this._nearest(a, PRED, g.sense * 0.95);
    const resting = this.daylight < 0.12 && !threat;
    let desired = 0.4 + g.speed * 0.18;
    if (threat) a.fear = clamp(a.fear + (1 - g.bold * 0.6) * 1.5 * dt, 0, 1);
    else a.fear = clamp(a.fear - 0.5 * dt, 0, 1);

    // 玩家警觉（动物怕人）：玩家走近且白天可见，缓慢走开（家域内）
    if (!threat && this.playerX !== undefined && this.daylight > 0.2) {
      const pd = Math.hypot(this.playerX - a.x, this.playerZ - a.z);
      if (pd < 9) {
        a.state = 'wary';
        const away = Math.atan2(a.x - this.playerX, a.z - this.playerZ);
        const wp = this._clampHome(a, a.x + Math.cos(away) * 12, a.z + Math.sin(away) * 12);
        a.targetX = wp.x;
        a.targetZ = wp.z;
        desired = g.speed * 0.5;
      }
    }

    // 篝火警觉：夜间火光附近走开（火危险）
    if (!threat && this.fires && this.fires.length && this.daylight < 0.3) {
      for (const f of this.fires) {
        const fd = Math.hypot(f.x - a.x, f.z - a.z);
        if (fd < 8) {
          a.state = 'wary';
          const away = Math.atan2(a.x - f.x, a.z - f.z);
          const wp = this._clampHome(a, a.x + Math.cos(away) * 10, a.z + Math.sin(away) * 10);
          a.targetX = wp.x;
          a.targetZ = wp.z;
          desired = g.speed * 0.55;
          break;
        }
      }
    }

    if (threat) {
      a.state = 'flee';
      const away = Math.atan2(a.x - threat.agent.x, a.z - threat.agent.z);
      a.heading = away;
      desired = g.speed * (1.32 - g.bold * 0.25);
      a.energy -= Math.max(0, 0.16 * dt * g.metabolism);
      if (HELP_ENABLED) this._drainE_H += Math.max(0, 0.16 * dt * g.metabolism); // 逃逸消耗计入能量占比分母
    } else if (a.think <= 0) {
      a.think = 0.35 + this.rng() * 0.4;

      if (g.social > 0.8) {
        // 聚群：社交基因高 → 跟随同类（群体防御）
        const mate = this._nearest(a, HERB, 15, (b) => b.id !== a.id);
        if (mate) {
          a.targetX = mate.agent.x;
          a.targetZ = mate.agent.z;
          a.state = 'graze';
          const dm = Math.hypot(mate.agent.x - a.x, mate.agent.z - a.z);
          desired = dm > 3 ? g.speed * 0.6 : 0;
        } else if (a.energy < a.maxEnergy * 0.84) {
          const p = this._findGrass(a, { x: a.x, z: a.z, amount: 0, cell: -1 });
          if (p.cell >= 0) {
            a.targetX = p.x;
            a.targetZ = p.z;
            a.state = 'graze';
            const d = Math.hypot(p.x - a.x, p.z - a.z);
            desired = d > 2 ? g.speed * (0.55 + (1 - a.energy / a.maxEnergy) * 0.45) : 0;
          } else {
            a.state = 'wander';
            const wa = this.rng() * Math.PI * 2;
            const wr = resting ? 14 : 35;
            const wp = this._clampHome(a, a.x + Math.cos(wa) * wr, a.z + Math.sin(wa) * wr);
            a.targetX = wp.x;
            a.targetZ = wp.z;
            desired = g.speed * 0.55;
          }
        }
      } else if (a.energy < a.maxEnergy * 0.84) {
        const p = this._findGrass(a, { x: a.x, z: a.z, amount: 0, cell: -1 });
        if (p.cell >= 0) {
          a.targetX = p.x;
          a.targetZ = p.z;
          a.state = 'graze';
          const d = Math.hypot(p.x - a.x, p.z - a.z);
          desired = d > 2 ? g.speed * (0.55 + (1 - a.energy / a.maxEnergy) * 0.45) : 0;
        } else {
          a.state = 'wander';
          const wa = this.rng() * Math.PI * 2;
          const wr = resting ? 14 : 35;
          const wp = this._clampHome(a, a.x + Math.cos(wa) * wr, a.z + Math.sin(wa) * wr);
          a.targetX = wp.x;
          a.targetZ = wp.z;
          desired = g.speed * 0.55;
        }
      } else {
        a.state = 'wander';
        const wp = this._clampHome(a, a.x + (this.rng() - 0.5) * (resting ? 18 : 45), a.z + (this.rng() - 0.5) * (resting ? 18 : 45));
        a.targetX = wp.x;
        a.targetZ = wp.z;
        desired = g.speed * (resting ? 0.12 : 0.3);
      }
    }

    // 第六波：冬季吸引聚合微扰（在状态机选定目标之后、移动决策之前，对 wander/patrol H 微移 target 向同类质心）
    this._winterAggregationPull(a, dt, desired);

    if (a.state !== 'flee') {
      const d = Math.hypot(a.targetX - a.x, a.targetZ - a.z);
      if (d < 1.6) {
        a.state = 'graze';
        desired = 0;
      } else {
        this._steer(a, a.targetX, a.targetZ, dt, desired);
      }
    }

    a.speed = lerp(a.speed, a.desiredSpeed, Math.min(1, dt * 5));
    const activity = a.speed / Math.max(1, g.speed);
    a.activity = activity;
    // 基因上位性（epistasis）：速度×体型、感知×代谢——基因交互影响（达尔文适应度景观）
    const epistasis = 1 + (g.speed / 8) * (g.size - 1) * 0.15 + (g.sense / 100) * (g.metabolism - 1) * 0.1;
    const drain = g.metabolism * (0.34 + 0.62 * activity * activity + 0.12 * (g.speed / 8) ** 2 + 0.08 * (g.sense / 130)) * epistasis;
    // 表型可塑性：环境诱导代谢微调（同一基因型，不同环境表现不同性状——达尔文不确定性）
    const plasticity = 1 + (this.daylight < 0.3 ? -0.06 : 0) + (a.hydration < 0.4 ? 0.04 : 0);
    // 种内竞争：同种密度高时 energy 消耗增（领地竞争压力）
    const localCrowd = this._count(a.kind) / (a.kind === HERB ? MAX_HERB : a.kind === PRED ? MAX_PRED : a.kind === OMN ? MAX_OMN : MAX_SCAV);
    const crowdPenalty = localCrowd > 0.4 ? 1 + (localCrowd - 0.4) * 0.5 : 1;
    const thirstH = a.hydration < 0.3 ? 1.5 : 1;
    if (HELP_ENABLED) this._drainE_H += drain * thirstH * crowdPenalty * plasticity * (this.seasonMeta || 1) * lineageDrain * dt; // 代谢消耗计入分母
    a.energy -= drain * thirstH * crowdPenalty * plasticity * (this.seasonMeta || 1) * lineageDrain * dt;

    const ci = this._cellIndex(a.x, a.z);
    if (ci >= 0 && this.grass[ci] > 0.1 && a.bite <= 0 && a.speed < g.speed * 0.45) {
      // 间接互惠：先评估是否让食；返回 true 表示本口已转移给接收方（跳过采食），false 正常采食本口（含拒绝=自己吃）
      if (!this._tryHelp(a, ci)) {
        const take = this._takeGrass(a, ci, 0.2 * lineageBite);
        this._leafE_H += take * 62;
        a.energy = Math.min(a.maxEnergy, a.energy + take * 62);
        a.hydration = Math.min(1, a.hydration + 0.18);
        a.bite = BITE_INTERVAL * (0.8 + this.rng() * 0.4) / lineageBite;
        if (this.grass[ci] < 0.03 && this.rng() < 0.1) a.think = 0;
      }
    }

    const nx = a.x + Math.sin(a.heading) * a.speed * dt;
    const nz = a.z + Math.cos(a.heading) * a.speed * dt;
    const moved = this._avoid(a, nx, nz);
    a.x = moved.x;
    a.z = moved.z;
    a.y = terrainHeight(a.x, a.z);

    if (a.energy <= 0) {
      this._die(a, 'starved');
    } else if (a.age > g.lifespan) {
      this._die(a, 'aged');
    } else if (a.repro <= 0 && a.energy > a.maxEnergy * 0.72) {
      this._reproduce(a, dt);
    }
  }

  _updateScav(a, dt) {
    a.age += dt;
    const g = a.genome;
    a.bite -= dt;
    a.repro -= dt;
    a.think -= dt;
    a.fear = clamp(a.fear - 0.5 * dt, 0, 1);
    if (a.think <= 0) {
      a.think = 0.3 + this.rng() * 0.3;
      let best = null;
      let bd = (g.sense * 1.2) ** 2;
      for (const c of this.carcasses) {
        const d = (c.x - a.x) ** 2 + (c.z - a.z) ** 2;
        if (d < bd) { bd = d; best = c; }
      }
      if (best) {
        a.targetX = best.x;
        a.targetZ = best.z;
        a.state = 'scavenge';
      } else {
        a.state = 'wander';
        const wa = this.rng() * Math.PI * 2;
        const wp = this._clampHome(a, a.x + Math.cos(wa) * 30, a.z + Math.sin(wa) * 30);
        a.targetX = wp.x;
        a.targetZ = wp.z;
      }
    }
    if (a.state !== 'flee') {
      const d = Math.hypot(a.targetX - a.x, a.targetZ - a.z);
      if (d < 1.6) a.desiredSpeed = 0;
      else this._steer(a, a.targetX, a.targetZ, dt, g.speed * 0.55);
    }
    a.speed = lerp(a.speed, a.desiredSpeed, Math.min(1, dt * 5));
    a.activity = a.speed / Math.max(1, g.speed);
    a.x += Math.sin(a.heading) * a.speed * dt;
    a.z += Math.cos(a.heading) * a.speed * dt;
    a.y = terrainHeight(a.x, a.z);
    const ci = this.carcasses.findIndex((c) => Math.hypot(c.x - a.x, c.z - a.z) < 1.8 && c.energy > 0.2);
    if (ci >= 0 && a.bite <= 0) {
      const take = Math.min(4, this.carcasses[ci].energy);
      this.carcasses[ci].energy -= take;
      a.energy = Math.min(a.maxEnergy, a.energy + take * 7);
      a.bite = 1.5;
      if (this.carcasses[ci].energy <= 0.2) this.carcasses.splice(ci, 1);
    } else if (a.energy < a.maxEnergy * 0.7 && a.bite <= 0) {
      const gi = this._cellIndex(a.x, a.z);
      if (gi >= 0 && this.grass[gi] > 0.1) {
        const take = this._takeGrass(a, gi, 0.15);
        a.energy = Math.min(a.maxEnergy, a.energy + take * 55);
        a.bite = 2.0;
      }
    }
    a.energy -= g.metabolism * (0.18 + a.activity * 0.3) * (this.seasonMeta || 1) * dt;
    if (a.energy <= 0) this._die(a, 'starved');
    else if (a.age > g.lifespan) this._die(a, 'aged');
    else if (a.repro <= 0 && a.energy > a.maxEnergy * 0.78) this._reproduce(a, dt);
  }

  _updateOmn(a, dt) {
    // 杂食者：吃草 + 腐食混合生态位（从 H 分化，遇尸体吃尸体、否则吃草）
    a.age += dt;
    const g = a.genome;
    a.bite -= dt;
    a.repro -= dt;
    a.think -= dt;
    a.fear = clamp(a.fear - 0.5 * dt, 0, 1);
    const threat = this._nearest(a, PRED, g.sense * 0.9);
    if (threat) {
      a.state = 'flee';
      const away = Math.atan2(a.x - threat.agent.x, a.z - threat.agent.z);
      a.heading = away;
      a.desiredSpeed = g.speed * 1.25;
    } else if (a.think <= 0) {
      a.think = 0.3 + this.rng() * 0.3;
      // 优先腐食（能量高），否则吃草
      let best = null;
      let bd = (g.sense * 1.2) ** 2;
      for (const c of this.carcasses) {
        const d = (c.x - a.x) ** 2 + (c.z - a.z) ** 2;
        if (d < bd) { bd = d; best = c; }
      }
      const needEnergy = a.energy < a.maxEnergy * 0.7;
      if (best && needEnergy) {
        a.targetX = best.x;
        a.targetZ = best.z;
        a.state = 'scavenge';
      } else {
        const p = this._findGrass(a, { x: a.x, z: a.z, amount: 0, cell: -1 });
        if (p.cell >= 0) {
          a.targetX = p.x;
          a.targetZ = p.z;
          a.state = 'graze';
        } else {
          a.state = 'wander';
          const wa = this.rng() * Math.PI * 2;
          const wp = this._clampHome(a, a.x + Math.cos(wa) * 30, a.z + Math.sin(wa) * 30);
          a.targetX = wp.x;
          a.targetZ = wp.z;
        }
      }
    }
    if (a.state !== 'flee') {
      const d = Math.hypot(a.targetX - a.x, a.targetZ - a.z);
      if (d < 1.6) a.desiredSpeed = 0;
      else this._steer(a, a.targetX, a.targetZ, dt, g.speed * 0.55);
    }
    a.speed = lerp(a.speed, a.desiredSpeed, Math.min(1, dt * 5));
    a.activity = a.speed / Math.max(1, g.speed);
    a.x += Math.sin(a.heading) * a.speed * dt;
    a.z += Math.cos(a.heading) * a.speed * dt;
    a.y = terrainHeight(a.x, a.z);
    // 腐食：近尸体吃（杂食者的尸体来源）
    const ci = this.carcasses.findIndex((c) => Math.hypot(c.x - a.x, c.z - a.z) < 1.8 && c.energy > 0.2);
    if (ci >= 0 && a.bite <= 0) {
      const take = Math.min(3, this.carcasses[ci].energy);
      this.carcasses[ci].energy -= take;
      a.energy = Math.min(a.maxEnergy, a.energy + take * 8);
      a.bite = 1.8;
      if (this.carcasses[ci].energy <= 0.2) this.carcasses.splice(ci, 1);
    }
    // 吃草
    if (a.bite <= 0 && a.energy < a.maxEnergy * 0.9) {
      const gi = this._cellIndex(a.x, a.z);
      if (gi >= 0 && this.grass[gi] > 0.1) {
        const take = this._takeGrass(a, gi, 0.18);
        a.energy = Math.min(a.maxEnergy, a.energy + take * 58);
        a.bite = 2.2;
      }
    }
    a.energy -= g.metabolism * (0.2 + a.activity * 0.32) * (this.seasonMeta || 1) * dt;
    if (a.energy <= 0) this._die(a, 'starved');
    else if (a.age > g.lifespan) this._die(a, 'aged');
    else if (a.repro <= 0 && a.energy > a.maxEnergy * 0.8) this._reproduce(a, dt);
  }

  _updatePred(a, dt) {
    a.age += dt;
    const g = a.genome;
    a.bite -= dt;
    a.repro -= dt;
    a.think -= dt;
    a.hydration = Math.min(1, a.hydration - 0.002 * dt + (a.y < WATER_LEVEL + 1.5 ? 0.10 * dt : 0));
    a.fatigue = clamp(a.fatigue + a.activity * 0.05 * dt - (a.activity < 0.2 ? 0.2 * dt : 0), 0, 1);
    a.curiosity = clamp(a.curiosity + 0.015 * dt, 0, 1);
    a.fear = clamp(a.fear - 0.3 * dt, 0, 1);
    let desired = 0.3 + g.speed * 0.16;
    const hungry = a.energy < a.maxEnergy * 0.9;

    if (a.think <= 0 || !a.targetId) {
      a.think = 0.22 + this.rng() * 0.28;
      const prey = hungry ? this._nearest(a, HERB, g.sense) : null;
      let best = null;
      let bd = (g.sense * 1.2) ** 2;
      for (const c of this.carcasses) {
        const d = (c.x - a.x) ** 2 + (c.z - a.z) ** 2;
        if (d < bd) { bd = d; best = c; }
      }
      if (best) {
        a.targetX = best.x;
        a.targetZ = best.z;
        a.state = 'scavenge';
        a.targetId = 0;
      } else if (prey) {
        a.targetId = prey.agent.id;
        a.targetX = prey.agent.x;
        a.targetZ = prey.agent.z;
        a.state = 'hunt';
      } else {
        a.targetId = 0;
        const wp = this._clampHome(a, a.x + (this.rng() - 0.5) * 95, a.z + (this.rng() - 0.5) * 95);
        a.targetX = wp.x;
        a.targetZ = wp.z;
        a.state = 'patrol';
      }
    }

    let prey = null;
    if (a.targetId) {
      prey = this.agents.find((b) => b.id === a.targetId && b.alive);
      if (prey) {
        a.targetX = prey.x;
        a.targetZ = prey.z;
        a.state = 'hunt';
      } else {
        a.targetId = 0;
        a.think = 0;
      }
    }

    if (a.state === 'hunt' || a.state === 'scavenge') {
      desired = g.speed * 1.1;
    } else {
      desired = g.speed * 0.32;
    }
    // 捕食者白天回避玩家（避免冲突；夜间攻击由威胁系统处理）
    if (this.playerX !== undefined && this.daylight > 0.4) {
      const pd = Math.hypot(this.playerX - a.x, this.playerZ - a.z);
      if (pd < 12) {
        a.state = 'avoid';
        const away = Math.atan2(a.x - this.playerX, a.z - this.playerZ);
        const wp = this._clampHome(a, a.x + Math.cos(away) * 14, a.z + Math.sin(away) * 14);
        a.targetX = wp.x;
        a.targetZ = wp.z;
        desired = g.speed * 0.7;
      }
    }
    // 篝火避火：夜间火光附近走开（避免夜袭篝火区）
    if (this.fires && this.fires.length && this.daylight < 0.3) {
      for (const f of this.fires) {
        const fd = Math.hypot(f.x - a.x, f.z - a.z);
        if (fd < 8) {
          a.state = 'avoid';
          const away = Math.atan2(a.x - f.x, a.z - f.z);
          const wp = this._clampHome(a, a.x + Math.cos(away) * 12, a.z + Math.sin(away) * 12);
          a.targetX = wp.x;
          a.targetZ = wp.z;
          desired = g.speed * 0.6;
          break;
        }
      }
    }
    // 火把：捕食者避玩家火光（夜间火把防夜袭）
    if (this.torch && this.daylight < 0.3) {
      const td = Math.hypot(this.torch.x - a.x, this.torch.z - a.z);
      if (td < 8) {
        a.state = 'avoid';
        const away = Math.atan2(a.x - this.torch.x, a.z - this.torch.z);
        const wp = this._clampHome(a, a.x + Math.cos(away) * 12, a.z + Math.sin(away) * 12);
        a.targetX = wp.x;
        a.targetZ = wp.z;
        desired = g.speed * 0.6;
      }
    }
    this._steer(a, a.targetX, a.targetZ, dt, desired);
    a.speed = lerp(a.speed, a.desiredSpeed, Math.min(1, dt * 5));
    a.activity = a.speed / Math.max(1, g.speed);
    const drain = g.metabolism * (0.46 + 0.78 * a.activity * a.activity + 0.14 * (g.speed / 10.5) ** 2);
    const thirstP = a.hydration < 0.3 ? 1.5 : 1;
    a.energy -= drain * thirstP * (this.seasonMeta || 1) * dt;

    const nx = a.x + Math.sin(a.heading) * a.speed * dt;
    const nz = a.z + Math.cos(a.heading) * a.speed * dt;
    const moved = this._avoid(a, nx, nz);
    a.x = moved.x;
    a.z = moved.z;
    a.y = terrainHeight(a.x, a.z);

    if (prey && a.bite <= 0) {
      const d = Math.hypot(prey.x - a.x, prey.z - a.z);
      const reach = 1.9 + g.size * 0.8;
      if (d < reach && hungry) {
        const gain = 18 + prey.energy * 0.22 + prey.genome.size * 11;
        a.energy = Math.min(a.maxEnergy, a.energy + gain * 0.8);
        a.hydration = Math.min(1, a.hydration + 0.4);
        a.bite = PRED_BITE_INTERVAL + this.rng() * 5.0;
        this.kills++;
        this.deaths++;
        prey.alive = false;
        this.carcasses.push({ x: prey.x, z: prey.z, energy: gain * 0.22, age: 0 });
        if (this.carcasses.length > 60) this.carcasses.shift();
        a.targetId = 0;
        a.think = 0;
        a.state = 'feed';
      }
    }

    if (a.state === 'scavenge' && a.bite <= 0) {
      for (let i = 0; i < this.carcasses.length; i++) {
        const c = this.carcasses[i];
        if (Math.hypot(c.x - a.x, c.z - a.z) < 2.2) {
          a.energy = Math.min(a.maxEnergy, a.energy + c.energy);
          a.hydration = Math.min(1, a.hydration + 0.15);
          this.carcasses.splice(i, 1);
          a.bite = 1.0;
          a.state = 'feed';
          a.targetId = 0;
          a.think = 0;
          break;
        }
      }
    }

    if (a.energy <= 0) {
      this._die(a, 'starved');
    } else if (a.age > g.lifespan) {
      this._die(a, 'aged');
    } else if (a.repro <= 0 && a.energy > a.maxEnergy * 0.85) {
      this._reproduce(a, dt);
    }
  }

  update(dt, daylight = 1) {
    if (dt <= 0) return;
    this.daylight = daylight;
    const rng = this.rng;
    const steps = Math.max(1, Math.round(dt / 0.1));
    const h = dt / steps;
    for (let s = 0; s < steps; s++) {
      this.time += h;
      this.tickCount++;

      // 季节：每 1800 秒（30 分钟真实时间）一季，四季循环——环境变化驱动选择
      this.season = Math.floor(this.time / 1800) % 4;
      const seasonGrass = [1.3, 0.6, 0.8, 0.25][this.season]; // 春丰/夏旱/秋中/冬荒
      const seasonMeta = [1.0, 1.15, 0.9, 0.75][this.season]; // 春正常/夏高耗/秋略低/冬节能
      this.seasonMeta = seasonMeta;

      // 共享逻辑斯蒂承载：各草种共争同一格容量 (1-Σ)，种间竞争由共享项隐式驱动；湿度适应度 suitEff 按种错开
      const grow = 0.0056 * seasonGrass * (0.18 + 0.82 * daylight) * (1 + (this.rain || 0) * 0.02);
      for (let i = 0; i < this.grass.length; i++) {
        const suit = this.suit[i];
        if (suit <= 0.02) continue;
        let total = 0;
        for (let s = 0; s < GRASS_N; s++) total += this.grassP[s][i];
        if (total >= 1) continue;
        const shadeF = 1 - this.shade[i] * 0.55;
        for (let sp = 0; sp < GRASS_N; sp++) {
          const b = this.grassP[sp][i];
          // (0.42+b) 保底项：b≈0 时仍有少量种子雨，避免格子被啃食至零后永久荒漠化；
          // rMax 用 rMax_eff（果实投资折损）：投资果实的种以增长率换传播
          const nb = b + this.suitEffBy[sp][i] * grow * RMAX_EFF[sp] * (1 - total) * (0.42 + b) * shadeF * h;
          this.grassP[sp][i] = nb > 1 ? 1 : nb;
        }
        let gsum = 0;
        for (let s = 0; s < GRASS_N; s++) gsum += this.grassP[s][i];
        this.grass[i] = gsum;
      }

      // 草扩散：旺盛草格向邻格自主蔓延（元素自主——草自发生长/扩散），逐物种独立跑。
      // 双缓冲：读旧数组（gs）写新数组（ngs），避免原位更新带来的右下方向偏置
      if (this.tickCount % 40 === 0) {
        if (!this._spreadBuf) this._spreadBuf = new Float32Array(GRID_N * GRID_N);
        for (let sp = 0; sp < GRASS_N; sp++) {
          const gs = this.grassP[sp];
          const ngs = this._spreadBuf;
          ngs.set(gs);
          for (let j = 1; j < GRID_N - 1; j++) {
            for (let i = 1; i < GRID_N - 1; i++) {
              const idx = j * GRID_N + i;
              const gv = gs[idx];
              if (gv < 0.55 || this.suit[idx] < 0.1) continue;
              const spread = gv * 0.01; // 每次每格向各邻格转移 1% 自身存量
              ngs[idx] -= spread * 4;
              // 接收端检查：suit≤0.02 的格不接收，避免成为永久小草斑
              if (this.suit[idx + 1] > 0.02) ngs[idx + 1] = Math.min(1, ngs[idx + 1] + spread);
              if (this.suit[idx - 1] > 0.02) ngs[idx - 1] = Math.min(1, ngs[idx - 1] + spread);
              if (this.suit[idx + GRID_N] > 0.02) ngs[idx + GRID_N] = Math.min(1, ngs[idx + GRID_N] + spread);
              if (this.suit[idx - GRID_N] > 0.02) ngs[idx - GRID_N] = Math.min(1, ngs[idx - GRID_N] + spread);
            }
          }
          this.grassP[sp] = ngs;
          this._spreadBuf = gs; // 旧数组回笼，作为下一物种的写缓冲
        }
        // 蔓延后同步总量视图（保证寻草/感知/输出层读到准确值）
        for (let i = 0; i < this.grass.length; i++) {
          let gsum = 0;
          for (let s = 0; s < GRASS_N; s++) gsum += this.grassP[s][i];
          this.grass[i] = gsum;
        }
      }

      if (this.tickCount % 100 === 0) this._hCache = this._count(HERB);

      for (const a of this.agents) {
        if (!a.alive) continue;
        a.prevX = a.x;
        a.prevZ = a.z;
        a.prevHeading = a.heading;
        if (a.cargo) {
          // 位置轨迹采样（每 8s 一点，供搬运距离观测）
          const slot = Math.floor(this.time / 8);
          if (slot !== a.trailSlot) {
            a.trailSlot = slot;
            a.trailT[a.trailHead] = this.time;
            a.trailX[a.trailHead] = a.x;
            a.trailZ[a.trailHead] = a.z;
            a.trailHead = (a.trailHead + 1) % TRAIL_SLOTS;
            if (a.trailN < TRAIL_SLOTS) a.trailN++;
          }
          this._gutTransit(a, h);
        }
        if (a.kind === HERB) this._updateHerb(a, h);
        else if (a.kind === SCAV) this._updateScav(a, h);
        else if (a.kind === OMN) this._updateOmn(a, h);
        else this._updatePred(a, h);
      }

      for (const c of this.carcasses) c.age += h;
      for (let i = this.carcasses.length - 1; i >= 0; i--) {
        const c = this.carcasses[i];
        c.energy -= 1.4 * h;
        if (c.energy <= 0.2) this.carcasses.splice(i, 1);
      }

      if (this.agents.length > 0 && this.agents.some((a) => !a.alive)) {
        this.agents = this.agents.filter((a) => a.alive);
      }
      // 濒危移民：P 过少时随机迁入（自然扩散机制；用播种 rng 保证同 seed 可复现）
      if (this._count(PRED) < 5 && this.rng() < 0.002) {
        const a = this.rng() * Math.PI * 2;
        const r = 350 + Math.sqrt(this.rng()) * 200;
        const x = Math.cos(a) * r, z = Math.sin(a) * r;
        const h = terrainHeight(x, z);
        if (h > WATER_LEVEL + 1.5 && h < 80 && slopeAt(x, z) < 0.7) {
          this.spawn(PRED, x, z, this._randomGenome(PRED), 0, 30 + this.rng() * 80);
        }
      }
      if (this.agents.length > MAX_HERB + MAX_PRED + MAX_SCAV + MAX_OMN + 60) {
        // 安全阀：极端繁殖时去掉最老的个体
        this.agents.sort((a, b) => b.age - a.age);
        this.agents.splice(MAX_HERB + MAX_PRED + MAX_SCAV + MAX_OMN, this.agents.length - MAX_HERB - MAX_PRED - MAX_SCAV - MAX_OMN);
      }

      if (this.tickCount - this.lastSample >= 20) {
        this.lastSample = this.tickCount;
        this._sample();
      }
    }
  }

  _sample() {
    let h = 0;
    let p = 0;
    let speedH = 0;
    let senseH = 0;
    let sizeH = 0;
    let speedP = 0;
    let senseP = 0;
    let scav = 0;
    let omn = 0;
    let gen = 0;
    let coopH = 0;      // image>0 的 H（正声誉占比，中性代号）
    let helpThrSum = 0; // helpThr 均值累加
    // sig 观测（仅 HERB）
    let prefSumH = 0, tolSumH = 0, size2H = 0;
    for (const a of this.agents) {
      if (!a.alive) continue;
      gen = Math.max(gen, a.generation);
      if (a.kind === HERB) {
        h++;
        speedH += a.genome.speed;
        senseH += a.genome.sense;
        sizeH += a.genome.size;
        if (a.image > 0) coopH++;
        helpThrSum += a.genome.helpThr || 0;
        if (a.genome.prefSig !== undefined) { prefSumH += a.genome.prefSig; tolSumH += a.genome.tolSig || 0; }
        size2H += a.genome.size ** 2;
      } else if (a.kind === SCAV) {
        scav++;
      } else if (a.kind === OMN) {
        omn++;
      } else {
        p++;
        speedP += a.genome.speed;
        senseP += a.genome.sense;
      }
    }
    this.history.push({
      t: this.time,
      season: this.season, // 季节标签（供 stats() 按季分段统计 winterCoopShareH）
      h, p, scav, omn,
      plant: this.plantAmount(),
      plantBy: this.plantAmountBy(), // 按种草量（生产者分化观测）
      speedH: h ? speedH / h : 0,
      senseH: h ? senseH / h : 0,
      sizeH: h ? sizeH / h : 0,
      speedP: p ? speedP / p : 0,
      senseP: p ? senseP / p : 0,
      gapHP: (h && p) ? (speedH / h - speedP / p) : 0, // 军备竞赛差距
      coopShareH: h ? coopH / h : 0, // 间接互惠观测：正声誉 H 占比
      helpThrH: h ? helpThrSum / h : 0, // 间接互惠观测：帮助阈值均值
      // sig 观测（T3：中性命名）
      sigMeanH: h ? sizeH / h : 0,       // HERB size 均值（magic trait 中心）
      sigVarH: h ? (size2H / h - (sizeH / h) ** 2) : 0, // HERB size 方差（涌现隔离监测）
      prefMeanH: h ? prefSumH / h : 0,   // HERB prefSig 均值（择偶偏好中心漂移）
      tolMeanH: h ? tolSumH / h : 0,     // HERB tolSig 均值（匹配容忍度演化）
      gen,
    });
    if (this.history.length > 240) this.history.shift();

    // 达尔文世代统计：每代性状均值 + 适应度均值（fitness）
    if (this.tickCount % 600 === 0) {
      for (const kind of [HERB, PRED, SCAV, OMN]) {
        const alive = this.agents.filter(a => a.alive && a.kind === kind);
        if (!alive.length) continue;
        const avg = {
          t: this.time,
          gen: Math.max(...alive.map(a => a.generation)),
          count: alive.length,
          speed: alive.reduce((s, a) => s + a.genome.speed, 0) / alive.length,
          sense: alive.reduce((s, a) => s + a.genome.sense, 0) / alive.length,
          size: alive.reduce((s, a) => s + a.genome.size, 0) / alive.length,
          metabolism: alive.reduce((s, a) => s + a.genome.metabolism, 0) / alive.length,
          bold: alive.reduce((s, a) => s + a.genome.bold, 0) / alive.length,
          curious: alive.reduce((s, a) => s + a.genome.curious, 0) / alive.length,
          social: alive.reduce((s, a) => s + a.genome.social, 0) / alive.length,
          helpThr: alive.reduce((s, a) => s + (a.genome.helpThr || 0), 0) / alive.length,
        };
        const kindKey = kind === HERB ? 'herb' : kind === PRED ? 'pred' : kind === SCAV ? 'scav' : 'omn';
        if (!this.evolution[kindKey]) this.evolution[kindKey] = [];
        this.evolution[kindKey].push(avg);
        if (this.evolution[kindKey].length > 100) this.evolution[kindKey].shift();

        // 物种形成检测（达尔文终点）：同种内基因距离聚类 + 生态位分化
        if (kind === HERB && alive.length >= 30) {
          const speeds = alive.map(a => a.genome.speed);
          const ranges = alive.map(a => a.range / 170); // 归一化家域
          const socials = alive.map(a => a.genome.social);
          const avgSpeed = speeds.reduce((s,v) => s+v, 0) / speeds.length;
          const cv = Math.sqrt(speeds.reduce((s,v) => s+(v-avgSpeed)**2, 0) / speeds.length) / avgSpeed;
          // 高变异（CV>0.15）+ 行为分化（社交/独居并存且各>15%）→ 生态位分化
          const socialHigh = socials.filter(s => s > 0.6).length / socials.length;
          const socialLow = socials.filter(s => s < 0.4).length / socials.length;
          if (cv > 0.12 && socialHigh > 0.15 && socialLow > 0.15) {
            // 谱系标记前先按速度排序，在排序后的序列上找相邻最大速度差作为切分点
            // （避免在未排序数组上取索引后错位映射到排序序列）
            const sortedAgents = alive.slice().sort((a, b) => a.genome.speed - b.genome.speed);
            const sortedSpeeds = sortedAgents.map((a) => a.genome.speed);
            let maxGap = 0, gapIdx = 0;
            for (let i = 1; i < sortedSpeeds.length; i++) {
              if (sortedSpeeds[i] - sortedSpeeds[i-1] > maxGap) { maxGap = sortedSpeeds[i] - sortedSpeeds[i-1]; gapIdx = i; }
            }
            const lowAvg = sortedSpeeds.slice(0, gapIdx).reduce((s, v) => s + v, 0) / (gapIdx || 1);
            const highAvg = sortedSpeeds.slice(gapIdx).reduce((s, v) => s + v, 0) / (sortedSpeeds.length - gapIdx || 1);
            // 谱系命名：根据行为分化特征命名
            const lineages = [
              { id: 'social_fast', name: '迅聚', avgSpeed: lowAvg, count: Math.round(socialHigh * speeds.length) },
              { id: 'solitary_slow', name: '缓独', avgSpeed: highAvg, count: Math.round(socialLow * speeds.length) },
            ];
            this.speciationEvent = {
              t: this.time,
              parentKind: 'herb',
              lineages,
              cvSpeed: +cv.toFixed(3),
              gapIdx,
              nameSuffix: `#${this.speciationFired + 1}`,
            };
            if (this.speciationFired < 3 && this.time - (this.lastSpeciation || 0) > 60) {
              this.lastSpeciation = this.time;
              this.speciationFired++;
            }
            // 谱系标记：给分化群个体加 lineage 标签（达尔文物种形成标记）
            // 已隔离谱系的个体不重划（保持隔离群成员稳定，否则隔离闭环被洗牌破坏）
            for (let i = 0; i < sortedAgents.length; i++) {
              if (this.isolatedGroups[sortedAgents[i].lineage]) continue;
              sortedAgents[i].lineage = i < gapIdx ? lineages[0].name : lineages[1].name;
            }
            // 隔离登记：分化群之间生殖隔离（观测中以中性代号呈现，不预设剧本式命名）
            for (const lin of lineages) {
              if (!this.isolatedGroups[lin.name]) {
                this.isolatedGroups[lin.name] = {
                  code: `隔离群-${Object.keys(this.isolatedGroups).length + 1}`,
                  kind: 'herb',
                  since: this.time,
                };
              }
            }
          }
        }
      }

      // 隔离群间平均基因距离时序：验证分化是否随世代增大（每 600 tick 采样一次）
      const isoNames = new Set(Object.keys(this.isolatedGroups)); // 一次构建，采样入口 O(1) 查询
      if (isoNames.size >= 2) {
        const isoClusters = {};
        for (const a of this.agents) {
          if (a.alive && a.kind === HERB && a.lineage && isoNames.has(a.lineage)) {
            (isoClusters[a.lineage] = isoClusters[a.lineage] || []).push(a);
          }
        }
        const isoKeys = Object.keys(isoClusters).filter((k) => isoClusters[k].length > 0);
        if (isoKeys.length >= 2) {
          let sum = 0, pairs = 0;
          for (let i = 0; i < isoKeys.length; i++) {
            for (let j = i + 1; j < isoKeys.length; j++) {
              for (const ga of isoClusters[isoKeys[i]]) {
                for (const gb of isoClusters[isoKeys[j]]) {
                  sum += this._pairTraitDistance(ga, gb);
                  pairs++;
                }
              }
            }
          }
          if (pairs > 0) {
            this.isolationHistory.push({ t: this.time, dist: +(sum / pairs).toFixed(4), pairs });
            if (this.isolationHistory.length > 200) this.isolationHistory.shift();
          }
        }
      }
    }

    // 选择压力（达尔文）：当前环境 → 选择方向
    const plant = this.plantAmount();
    const predCount = this._count(PRED);
    const herbCount = this._count(HERB);
    this.selectionPressure = {
      plant: plant, // 草量（资源压力）
      predRatio: herbCount ? predCount / herbCount : 0, // 捕食强度
      crowding: (herbCount + predCount) / (MAX_HERB + MAX_PRED), // 密度
    };
  }

  stats() {
    const last = this.history[this.history.length - 1] || {
      h: 0, p: 0, scav: 0, omn: 0, plant: 0, plantBy: [0, 0, 0], speedH: 0, senseH: 0, speedP: 0, senseP: 0, coopShareH: 0, helpThrH: 0, sigMeanH: 0, sigVarH: 0, prefMeanH: 0, tolMeanH: 0, gen: 0,
    };
    const first = this.history[0] || last;
    // 达尔文适应度：当前种群平均 fitness vs 历史
    let curFitH = 0, nFitH = 0, curFitP = 0, nFitP = 0;
    for (const a of this.agents) {
      if (!a.alive) continue;
      const off = a.offspring || 0;
      const fit = (a.age / a.genome.lifespan) * (1 + off) / a.genome.metabolism;
      if (a.kind === HERB) { curFitH += fit; nFitH++; }
      else if (a.kind === PRED) { curFitP += fit; nFitP++; }
    }
    // 内动传播观测：全群携带种子量 + 果实能量占比（H，授予量口径）+ deposit 距离分位数
    let cargoTotal = 0;
    for (const a of this.agents) {
      if (!a.alive || !a.cargo) continue;
      for (let s = 0; s < GRASS_N; s++) cargoTotal += a.cargo[s];
    }
    const st = this.depositStats;
    let depositSummary = null;
    if (st.count > 0) {
      const sorted = [...st.samples].sort((x, y) => x - y);
      const q = (p) => sorted.length ? +sorted[Math.min(sorted.length - 1, Math.floor(p * sorted.length))].toFixed(1) : 0;
      depositSummary = {
        count: st.count,
        meanDistCells: +(st.distSum / st.count).toFixed(1),
        p50: q(0.5), p90: q(0.9),
        far5: st.far5, far20: st.far20,
        colonize: st.colonize,
      };
    }
    const fruitE = this._fruitE_H, leafE = this._leafE_H;
    return {
      ...last,
      time: this.time,
      season: this.season,
      births: this.births,
      deaths: this.deaths,
      kills: this.kills,
      starved: this.starved,
      aged: this.aged,
      trendH: first.speedH ? last.speedH - first.speedH : 0,
      trendP: first.speedP ? last.speedP - first.speedP : 0,
      // 达尔文进化数据
      selectionPressure: this.selectionPressure || {},
      curFitH: nFitH ? curFitH / nFitH : 0,
      curFitP: nFitP ? curFitP / nFitP : 0,
      evHistH: this.evolution.herb,
      evHistP: this.evolution.pred,
      evHistScav: this.evolution.scav,
      evHistOmn: this.evolution.omn,
      speciationEvent: this.speciationEvent || null,
      // 生殖隔离观测：隔离群（中性代号）+ 群间平均基因距离时序
      geneBarrier: this.GENE_BARRIER_STRENGTH,
      barrierBlocks: this.barrierBlocks,
      isolatedGroups: Object.entries(this.isolatedGroups).map(([name, g]) => ({
        name: g.code,
        kind: g.kind,
        since: +g.since.toFixed(1),
        count: this.agents.filter((a) => a.alive && a.lineage === name).length,
      })),
      isolationHistory: this.isolationHistory,
      // 生产者分化观测：按种草量 + 草种性状（中性代号）
      plantBy: last.plantBy || [],
      grassSpecies: GRASS_SPECIES.map(t => ({ ...t })), // 浅拷贝，防消费方改写全局性状表
      // 内动传播观测：cargoTotal 全群携带种子总量；deposit 含搬运距离分布（格，源=最近摄果位置）
      cargoTotal: +cargoTotal.toFixed(4),
      fruitEnergyShareH: fruitE + leafE > 0 ? +(fruitE / (fruitE + leafE)).toFixed(4) : 0,
      deposit: depositSummary,
      // 间接互惠观测（中性代号）：helpCount/rejectCount 帮助与拒绝次数；helpCostShareH = 让食成本 / H 总消耗
      helpCount: this._helpCount,
      rejectCount: this._rejectCount,
      helpCostShareH: this._helpCostE_H + this._drainE_H > 0 ? +(this._helpCostE_H / (this._helpCostE_H + this._drainE_H)).toFixed(4) : 0,
      // 第五波：冬季时段（season===3）coopShareH 均值，供验收对照（history 无冬样本时为 0）
      winterCoopShareH: (() => {
        const w = this.history.filter((s) => s.season === 3);
        return w.length ? w.reduce((acc, s) => acc + (s.coopShareH || 0), 0) / w.length : 0;
      })(),
      // sig 观测（T3）：sigPairFailRate = sig 匹配失败阻断 / sig 层配对尝试（SIG 关闭时恒 0）
      sigPairFailRate: this._sigAttempts > 0 ? +(this._sigBlocks / this._sigAttempts).toFixed(4) : 0,
      sigPairAttempts: this._sigAttempts,
      sigPairFails: this._sigBlocks,
    };
  }

  // 供渲染层读取，不分配新数组
  living() {
    return this.agents;
  }

  // 外部狩猎：杀死个体并留腐肉（供腐食循环），返回是否成功
  hunt(a) {
    if (!a || !a.alive) return false;
    this._die(a, 'hunted');
    return true;
  }

  // Oracle checksum：纯读遍历所有可观测状态，返回确定性 hex 字符串。
  // 用途：Rust sim 移植阶段的对拍黄金样本；不改任何字段、不调任何改变 RNG 的方法。
  // 只覆盖「会影响后续 tick 行为」的状态子集（agents 位置/能量/kind/alice 标记、grass grid、
  // time/season、births/deaths 累计、sign 等演化元数据）。非状态字段（如 isolatedGroups.code 等
  // 纯展示标签）不参与哈希，保持轻量。
  checksum() {
    const MOD = 0x100000001b3; // FNV-1a 64 prime
    let h = 14695981039346656033n;
    const mix = (v) => {
      h ^= BigInt(v) & 0xffffffffn;
      h = (h * BigInt(MOD)) & 0xffffffffffffffffn;
    };
    // agents：每活体按 kind→id→x/z→energy→age 序列化，顺序稳定（array 遍历序）
    for (const a of this.agents) {
      if (!a.alive) continue;
      mix(a.kind.charCodeAt(0));
      mix(a.id);
      mix(Math.floor(a.x * 100));
      mix(Math.floor(a.z * 100));
      mix(Math.floor(a.energy * 100));
      mix(Math.floor(a.age * 10));
    }
    // grass 总量场（只取第一个通道的 sum，足以编码草分布）
    for (let i = 0; i < this.grass.length; i += 7) mix(Math.floor(this.grass[i] * 1000));
    // 关键标量
    mix(Math.floor(this.time * 10));
    mix(this.season);
    mix(this.rain);
    mix(this.births);
    mix(this.deaths);
    mix(this.kills);
    mix(this.speciationFired);
    return '0x' + h.toString(16).padStart(16, '0');
  }
}
