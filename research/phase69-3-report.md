# Phase 69.3 — 资源流导出的 DECO 总承载容量（Stage 0 标定预注册）

日期：2026-11-24
HEAD（起点）：`cfe2e71`（Phase 69.2：T-region 默认 OFF，P2/D0 败，转 kind-only）
性质：**Stage 0 纯遥测标定**。不改默认、不引入机制、不 re-baseline；只测 DECO 的资源面（尸体能量）在世界中的尺度，为 Stage 1 的「资源→总容量」映射提供 `E_ref`。Stage 1 的 A/B 会在 Stage 0 数据出来后单独预注册。

> Phase 69.1/69.2 已把靶点钉死：派生 DECO 的 kind-only 拒绝 68.7%（T 臂 93.9%），region 侧三次否证。Phase 69.3 是唯一还没被否证的「增加总容量」方向：让 DECO 全族预算随尸体资源流变化，而不是固定 80。

---

## 1. Stage 0 预注册（跑之前写死）

### 1.1 资源信号

- **主信号**：`E(t) = Σ_{c ∈ carcasses} c.energy`，即当前尸体池的总能量（`Carcass.energy`，公开字段）。
- **辅助信号**：`N(t) = carcasses.len()`（尸体数），用于区分「少数大尸体」与「多数小尸体」。
- **为什么是尸体能量**：DECO 的唯一食物是尸体（`Carcass` 池）；尸体能量是 DECO 承载容量的直接资源面。它与 DECO 人口是负反馈（DECO 吃得越多，池越低），不会形成「人口→资源→人口」的正反馈爆炸。
- **采样**：沿用观测器 `SAMPLE_EVERY = 5_000` tick；只在 CSV 中记录，不参与任何动力学。
- **排除前 20k**：初始布景/abiogenesis 的 transient 不算入标定统计。

### 1.2 插桩（零副作用）

- `Ecology::carcass_energy_total() -> f64`：`carcasses.iter().map(|c| c.energy).sum()`；纯读、零 RNG。
- `Ecology::carcass_count() -> usize`：`carcasses.len()`；纯读。
- `clade_fate_observatory.rs` 的 `clade_fate` CSV 末尾追加 `carcass_energy,carcass_count` 两列；`[fate]` 行末尾追加 `carcass_e={:.1}`（便于 grep）。
- 不改任何 `spawn`/`die`/`update` 路径；默认世界必须与 Phase 69.2 的 `9/4` 和 `@60000` 锚点逐位一致。

### 1.3 运行

| 项 | 内容 |
|---|---|
| seed | `20261215 / 20261217 / 20261219`（Phase 69.2 的 OFF 臂已跑过 200k，但当时无 carcass 列；本期重跑） |
| tick | 200,000 |
| 臂 | 编译默认（OFF） |
| 并发 | 3 进程 |
| 自检 | `20261102 / 200k` = 9/4；`@60000` 锚点 `0x3272f0e3eaba9250` |
| 输出 | `/tmp/p69s0/*.log` 与 `clade_fate_*_p69s0*.csv` |

### 1.4 标定量

对每个 seed，取 `t ≥ 20_000` 的 36 个 5k 采样点：

- `E_mean`、`E_median`、`E_p10`、`E_p90`。
- `N_mean`、`N_median`。
- 跨 seed 的 `E_median` 中位数 → **`E_ref`**（Stage 1 的参考资源）。
- `E` 的逐 seed 变异系数（CV = std/mean），用于判断「资源流是否稳定到可以驱动总容量」。

### 1.5 判据（Stage 0 只验收仪器，不判定机制）

| # | 条件 | 失败处理 |
|---|---|---|
| D0-1 | 三个 seed 的 `carcass_energy` 列非空且有限 | 仪器错误；先修 |
| D0-2 | 默认 `20261102 / 200k` = 9/4 | 轨迹被扰动；停止 |
| D0-3 | `@60000` 锚点 `0x3272f0e3eaba9250` 位精确 | 同上 |
| D0-4 | 三 seed 的 `E_median` 同量级（CV ≤ 0.5） | 资源尺度 seed 依赖过强；Stage 1 需要 per-seed 归一化或更多 seed |

### 1.6 明确不做

- 不改 `MAX_DECO`、`kind_budgets`、`family_budget_total`。
- 不引入任何 cap 映射；Stage 1 的映射参数在 Stage 0 数据出来后单独预注册。
- 不改 region cap、择偶半径、W1、M10。
- 不做观察层/渲染功能；只做 headless CSV 遥测。

### 1.7 执行协议

1. 提交本预注册（§1）。
2. 实现 §1.2 的两个 getter 与 CSV/`[fate]` 列；补一个只读单测（`carcass_energy_total == sum`、空池为 0）。
3. 默认 `@60000` 锚点 + 全量测试 0 失败。
4. 跑 3 seed × 200k；写 Stage 0 标定表与 `E_ref`。
5. Stage 1 的 A/B 预注册另行提交；本期不跑 A/B。

---

## 2. Stage 0 实现

- `ecology.rs:4684`：`carcass_energy_total()`（Σ `Carcass.energy`，纯读）。
- `ecology.rs:4689`：`carcass_count()`（`carcasses.len()`，纯读）。
- `clade_fate_observatory.rs`：`clade_fate` CSV 末尾追加 `carcass_energy,carcass_count`；`[fate]` 行末尾追加 `carcass_e={:.1}`。
- 单测 `phase69_3_stage0_tests::carcass_totals_are_a_pure_sum_of_the_pool`：空池为 0、两具尸体求和 6.75。
- 零 RNG、零动力学路径改动；默认世界与 Phase 69.2 的 9/4、`@60000` 锚点一致。

---

## 3. Stage 0 结果

3 个默认 seed × 200k，取 `t ≥ 20_000` 的 37 个 5k 采样点：

| seed | samples | E_mean | E_median | E_p10 | E_p90 | E_CV | N_mean | N_median |
|---|---|---|---|---|---|---|---|---|
| 20261215 | 37 | 186.12 | 194.70 | 92.70 | 250.50 | 0.367 | 15.57 | 17 |
| 20261217 | 37 | 219.90 | 202.50 | 110.10 | 353.00 | 0.459 | 18.08 | 17 |
| 20261219 | 37 | 173.74 | 173.10 | 98.50 | 273.60 | 0.430 | 14.30 | 14 |
| **合计** | **111** | **193.25** | **192.50** | **98.50** | **304.00** | — | — | — |

**`E_ref = 192.5`**（三 seed 的 `E_median` 中位数）。

读数：

- 三 seed 的 `E_median` 在 173-203 之间，跨 seed 中位数 CV ≈ 0.08，**D0-4 过**（≤0.5）。
- 逐 seed 内部 CV 0.37-0.46：尸体池是**噪声资源**，必须用 EMA 平滑，不能直接用瞬时 E 驱 cap。
- `E_p10=98.5`、`E_p90=304`：按 `80 × E / 192.5` 映射，cap 的典型范围约 **41-126**，正好落在预设 `[40,160]` 内，动态范围可用。
- `N_median=14-17`：尸体池由少量大尸体组成；总能量比数量更稳定。

---

## 4. Stage 1 候选映射（Stage 0 后写死，暂不预注册）

- `E_ref = 192.5`（Stage 0 三 seed 的 `E_median` 中位数）。
- `cap_total = clamp(round(80 × E_ema / 192.5), 40, 160)`；`E_p10/E_p90` 对应约 41/126，动态范围可用。
- `E_ema` 初始 `E_ref`；每 5,000 tick 更新：`E_ema ← 0.8 × E_ema + 0.2 × E`（零 RNG）。
- **预算重标定**：设当前 DECO 全族总额 `S = Σ kind_budget`；新 cap 为 `C`；逐 kind `new_k = round(old_k × C / S)`；四舍五入余量补到根 kind；若 `S=0`，根 = `C`。重标定后 `family_budget_total(DECO) = C`，fork 仍按 `clamp(parent/4, 8, 20)` 划拨，灭绝归还。
- **必须同时改的集成点**：
  1. `biome_cap_in` 对 DECO 的 `BiomeCap::Fallback` 现在返回 `def.global_cap=80`；Stage 1 必须改为 `deco_family_cap`，否则预算抬到 126 也会被 80 的 biome cap 卡住。
  2. `cap_for_kind` 的 crowding 分母 `MAX_DECO=80` 必须改用 `deco_family_cap`，否则超过 80 时过度惩罚。
  3. 安全阀 `total_cap` 仍用 `MAX_DECO=80`：cap 上限 160 时总上限 830 < total_cap+60=840，不会误杀；若未来上限 >160 需同步。
  4. `SaveState`：A/B 期 arm/cap 不入档；若 Stage 1 转正，必须加 v25 字段（`DecoFamilyCap` / `ResourceCapOn`），否则读一份 cap>80 的存档会被 OFF 默认解释成「预算总和 >80 但机制关」。
- Stage 1 的 A/B：`20261222–26`（备用 `20261227–28`）× OFF/R × 200k；P1 分化非劣性、P2 births/kt ≥1.10×、P3 deaths/birth ≤1.05×、D0 cap 在 200k 内变化 ≥20%、D1 人口不爆炸/不塌缩（40-160）。

---

## 5. 验证

| 项 | 结果 |
|---|---|
| 默认自检 `20261102 / 200k` | **9 分化 / 4 灭绝**（1068.63 s） |
| 默认 `@60000` | `0x3272f0e3eaba9250` 位精确（334.01 s） |
| Stage 0 三 seed | `20261215 / 17 / 19` 全部 exit 0；CSV 列 `carcass_energy,carcass_count` 有值 |
| 单测 | `phase69_3_stage0_tests::carcass_totals_are_a_pure_sum_of_the_pool` 通过 |
| 全量测试 | 见 §5.1 |

### 5.1 全量测试

- 生态 crate：**50 二进制 / 318 passed / 0 failed / 28 ignored**。
- Stage 0 单测：1/1 通过。
- wasm：831 807 B，sha256 `1b936d5b7e8fe18ea230b0b8085ab7ed9f0ccd62fc22959c8596cb299458abb2`；`WASM VIEWER SMOKE OK`，10.09 ms/frame。

---

## 6. 改动文件

- `core/crates/aether-sim-ecology/src/ecology.rs`：`carcass_energy_total()` / `carcass_count()` + Stage 0 单测。
- `core/crates/aether-sim-ecology/tests/clade_fate_observatory.rs`：`clade_fate` CSV 两列 + `[fate]` 的 `carcass_e`。
- `research/phase69-3-report.md`：本文件。

---

## 7. 明确未做（Stage 0）

- 没有实现 Stage 1 的资源→cap 映射；没有改 `MAX_DECO`、`kind_budgets`、`biome_cap_in`、`cap_for_kind`。
- 没有跑 Stage 1 A/B；没有 re-baseline。
- 没有动 region cap / 择偶半径 / W1 / M10。

---

## 8. Stage 1 预注册（跑之前写死）

HEAD（起点）：`8f0659d`（Stage 0 交付；`E_ref=192.5`）。

### 8.1 机制

| 项 | 值 |
|---|---|
| 臂开关 | `PHASE_DECO_RESOURCE_CAP=on|off`；编译默认 **OFF** |
| 资源信号 | `E = carcass_energy_total()`（尸体池总能量） |
| 参考 | `E_ref = 192.5`（Stage 0 三 seed 中位） |
| EMA | 初始 `E_ema = E_ref`；每 5,000 tick `E_ema ← 0.8 × E_ema + 0.2 × E`；零 RNG |
| 总容量 | `cap_total = clamp(round(80 × E_ema / E_ref), 40, 160)` |
| 预算重标定 | 当前 DECO 全族 `S = Σ kind_budget`；逐 kind `new = round(old × cap_total / S)`；余量补根 kind；`S=0` 时根 = `cap_total`；`family_budget_total(DECO) = cap_total` |
| 区域 cap | 不变（仍按 `kind_budget` 与分解场权重分四区） |
| 集成点 | ① `biome_cap_in` 对 DECO 的 `Fallback` 在臂 ON 时返回 `deco_family_cap`；② `cap_for_kind` crowding 分母用 `deco_family_cap`；③ 安全阀 `total_cap` 仍 80（cap≤160 时总上限 830 < 840） |
| 存档 | A/B 期 arm/cap 不入档；若转正，加 v25 `DecoFamilyCap`/`ResourceCapOn`，读档默认 OFF |
| 确定性 | 零 RNG；所有重标定用整数运算 + 固定余量规则 |

> **实现偏差（2026-11-24，任何 Stage 1 200k 之前）**：§8.1 字面写「逐 kind `round(old × C / S)`，余量补根」。实现改为 **`floor(old × C / S)` + 余量补根**：独立 round 可能让 Σ 溢出或亏欠 `cap`，floor + 余量才能严格守恒；`assert_eq!(family_budget_total(DECO), cap)` 钉住。判据、seed 池、门槛、决策表不变。

### 8.2 seed 池

`20261222–26` 仓库命中 0；`20261227–28` 备用。Phase 69.2 的 `20261215–20` 不复用。

| 位 | seed | 用途 |
|---|---|---|
| 1–5 | 20261222 / 23 / 24 / 25 / 26 | 200k 判定 |
| 备 | 20261227 / 28 | 退化替换，按序最多 2 个 |

退化：`spec + ext = 0`，或 DECO 根 clade 在 `t < 10 000` 灭绝。

### 8.3 臂与运行

- OFF：编译默认；R：`PHASE_DECO_RESOURCE_CAP=on`。
- 5 seed × 2 臂 × 200k = 10 个判定 run；4 进程/批。
- 默认自检 `20261102 / 200k` = 9/4；R 双跑 `20261222` 两次；wasm 重建三次 <16 ms。

### 8.4 判据

| 判据 | 门槛 | 说明 |
|---|---|---|
| **P1 分化** | `Σ_R spec ≥ 0.90 × Σ_OFF` | 非劣性；先于 P2 |
| **P2 出生率** | `births/kt_R ≥ 1.10 ×` OFF 同口径 | 主目标 |
| **P3 死亡/出生** | `deaths/birth_R ≤ 1.05 ×` OFF | 非劣性 |
| **D0 资源驱动** | 200k 内 `deco_family_cap` 至少 3 个不同取值，且 `max−min ≥ 16`（0.20×80） | 防「cap 没动」 |
| **D1 人口** | 200k 末 DECO 人口 ∈ [40, 160] | 防爆炸/塌缩 |
| **D2 确定性** | OFF 默认 @60000 锚点不变；R 双跑 bit-exact | 必要条件 |

P1/P2/P3 必须同时满足；D0/D1/D2 是必要条件。

### 8.5 决策表

| # | 结果 | 决策 |
|---|---|---|
| 1 | D0 不成立 | 默认 OFF；结论「cap 未动，本池无信息」 |
| 2 | D0 ∧ D1 ∧ P1 ∧ P2 ∧ P3 | 默认 ON；**第二十次 re-baseline**；保留 `PHASE_DECO_RESOURCE_CAP=off` 回放臂 |
| 3 | P1 败 | 默认 OFF；分化代价越界 |
| 4 | P1 过、P2 败 | 默认 OFF；资源总量不是出生率杠杆 |
| 5 | P1/P2 过、P3 败 | 默认 OFF；多生多死 |
| 6 | D1 败 | 默认 OFF；人口爆炸/塌缩 |
| 7 | 可测 seed < 5 | inconclusive；默认 OFF |

### 8.6 明确不做

- 不做 region cap 永久/临时 floor、择偶半径、W1、M10。
- 不做手工 cap 调参；`E_ref`、`40/160`、`0.8/0.2` 全部写死，不按结果回调。
- 不新增全局预算上限；cap 只在 [40,160]。
- 不做 per-seed 自适应；同一映射跑所有 seed。
- 不改 region 预算分四区的语义；只改全族总额。

### 8.7 执行协议

1. 提交本预注册（§8）。
2. 实现 §8.1 的 arm、`deco_family_cap`、EMA、预算重标定、`biome_cap_in`/`cap_for_kind` 集成；补单测（默认 OFF 位精确、cap 边界 40/160、重标定守恒、EMA 确定性、存档默认 OFF）。
3. OFF 默认 @60000 锚点 + 全 workspace 测试 0 失败。
4. 编译一次 release 测试二进制；10 个判定 run 复用。
5. 跑 5 seed × 2 臂 × 200k + 双跑；记录逐 seed、P1-P3、D0-D2。
6. 重建 wasm；三次 <16 ms。
7. 写报告 §9 之后：结果、决策、锚点、测试、wasm、Phase 70 遗留。

### 8.8 实现验证（Stage 1，A/B 之前）

- 新增单测：`phase69_3_stage1_tests` 8 个（默认 OFF、cap clamp、重标定守恒、空族、EMA 确定性、arm 路由、存档契约）；`--lib phase69_3` 共 **9 passed / 0 failed**。
- 全量生态 crate：**50 二进制 / 334 passed / 0 failed / 28 ignored**。
- 默认 `@60000`：`0x3272f0e3eaba9250`（398.38 s）。
- 20k ON 冒烟（20261222）：`deco_family_cap` 80→99→101→106→105；DECO 人口 106 > 80，证明 biome cap 真放开；`family_total=105`。
- wasm：**832 800 B**，sha256 `33726e1424bdc371a4f79a16da0998aea726e202481df46d5bdd92c4ddb5b206`；smoke OK，10.04 ms/frame；新旧三轮 9.90-9.94 ms，全部 <16 ms。
- 已知不一致：`kind_budgets` 入档而 arm/cap 不入档；A/B 不读档，转正需 v25。
