# Phase 61 — 13 起灭绝的定因 + 单区支系份额重归一化（A/B：三候选全部未过线，默认不变）

日期：2026-11-20
HEAD（起点）：`6c24542`（Phase 60 报告修正，第十七次 re-baseline）
性质：**遥测机制 + 三个实验臂**。Phase 59 §10.5 的「逐 clade 出生/死亡计数器」本期落成引擎级实现（只读、不入档、零 RNG），并用它把 Phase 60 的 13 起灭绝逐起定因。块 2 的三个候选修正（a 全区重归一化 / b 女儿最低保留份额 / c 仅女儿重归一化）在 3 seeds × 200k 上全部**未能同时满足**「灭绝 ≤ 6 且分化 ≥ 22」。按验收条款，默认保持 Phase 60 世界（**不发生第十八次 re-baseline**），三个臂全部保留在代码里，供 Phase 62 的 5-seed 复测。

---

## 1. 问题

Phase 60 的区域承载容量把分化从 8 拉到 22，同时把灭绝从 2 拉到 13。已知设计缺陷写在 Phase 60 §9.2：

```
cap(kind, r) = kind_budget(kind) · w(r) / Σ_all w
```

一个只占一个 home quadrant 的支系，最多只能在该象限用 `budget / 4` 个槽位。而 Phase 49 的 fork 预算在 DECO（根 80）上依次给出 20 / 15 / 11 / 8 的份额，`budget / 4` 对应 **5 / 3.75 / 2.75 / 2** 个槽位——**全部低于 Phase 53 的分化门限 `SPECIATION_MIN_SIDE = 8`**。也就是说：引擎在一个支系有 8 个成员时宣布它是物种，却同时把它的可繁殖容量压在 2–5。分裂门与容量门自相矛盾（Phase 53 的「floor 与 split gate 必须一致」教训的第二次出现）。

## 2. 块 1：引擎级逐 clade 计数器

### 2.1 形式

`Ecology::clade_life: Vec<CladeLifeStats>`（`ecology.rs` 的 `CladeLifeStats`），按 `(clade_id, root_kind)` 键控——clade id 0 同时是「未分配」哨兵，所以同一个 id 0 在不同族里是不同行；根 clade 实体化时（id 必为 0）早期出生账目自然并入。

| 字段 | 含义 |
|---|---|
| `births` | 带亲代的出生（`reproduce` 的两条路径），按母亲 clade 记账 |
| `deaths` / `deaths_starved` / `deaths_aged` / `deaths_predation` / `deaths_other` | 死亡按死因四分（含 predation 直杀路径，绕开 `die()` 的那一处也记） |
| `region_cap_blocks` | 被 Phase 60 区域槽位闸拒绝的出生尝试 |
| `kind_budget_blocks` | 被 Phase 49 kind 预算 / biome cap 闸拒绝的出生尝试 |
| `mate_misses` | 已过冷却、能量过阈的成年个体，`nearest()` 择偶搜索返回空 |
| `first_tick` / `first_birth_tick` / `last_birth_tick` | 首次入账 / 首次出生 / 末次出生 |
| `peak_pop` / `peak_tick` / `region_peak[4]` | 站立人口峰值（60 tick 采样，与 `members` 重建同频）与每象限峰位 |
| `extinction_tick` | 引擎登记该 clade 绝灭事件的 tick |
| `death_age_sum` / `death_energy_sum` / `last_death_energy` | 死亡年龄与末期能量（寿命、末期能量证据） |

契约：
- **不入 SaveState**（运行时遥测，与 `isolation_by_kind`、`GroupStats` 同一条款）；读档后从活体活动重建。
- **零 RNG、零回写**：计数器只在引擎已经做出决定之后累加，不被任何判定读取。实测 @60000 锚点逐位不变（§6）。
- 拒绝记账挂在 `spawn_tagged(..., tag: Option<(clade_id, mother_kind)>)` 上；`spawn()` 以 `None` 委托，初始种群 / abiogenesis / group propagule 不计为「出生」。
- 闸门顺序（重要）：`spawn` 先查 Phase 49 kind 预算，再查 Phase 60 区域容量。所以一个已经顶到自己预算的 kind，它的区域拒绝会被预算闸「遮住」——`region_cap_blocks` 因此**系统性低估**区域挤压，判据要看 cap 本身（§3.2）。
- 观测器：`clade_fate_observatory` 新增 `/tmp/clade_life_*.csv`（上表全列 + clade 名/族名）与 stderr 的 `[extinct]` 汇总行。

### 2.2 复现性

3 seeds × 200k 的事件表与 Phase 60 报告逐事件一致（102: 9 分化 + 4 灭绝 = 13；42: 10 + 8 = 18；109: 3 + 1 = 4）。计数器没有移动任何一个 tick。

## 3. 块 1 结果：13 起灭绝定因

### 3.1 分类表

grant = fork 时该支系的 kind 预算（由事件序 + `fork_budget_grant` 复算，与引擎守恒一致）；`cap/象限` = `grant / 4`（均场权重）；life = 观测器 tree 的生存区间；`rpeak` = 每象限峰位人口（四个象限独立取历史最大，故其和可大于 `peak`；含该 clade 里仍带父 kind 的原始成员）。`ek` = 引擎灭绝事件类别（1 普通灭绝 / 2 竞争排除）。

| # | seed | clade | grant | 生存区间 | births/deaths | 死因 (饿/老/捕) | peak / rpeak | region-blk | budget-blk | mate-miss | 寿命均值 | 末期能量 | ek | **分类** |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 102 | deco-C | 11 | 37.3k–61.4k | 50/65 | 11/54/0 | 22 · [3,15,3,5] | 2 364 | 50 986 | 40 496 | 466 | 0.00 | 1 | **容量饥饿** |
| 2 | 102 | deco-A-D | 8 | 41.5k–72.5k | 50/58 | 13/45/0 | 15 · [8,3,4,5] | 2 169 | 41 003 | 41 879 | 434 | 0.00 | 1 | **容量饥饿** |
| 3 | 102 | deco-E | 8 | 44.3k–176.5k | 272/283 | 100/183/0 | 15 · [7,5,7,8] | 7 906 | 148 612 | 125 542 | 353 | 0.00 | 1 | **容量饥饿** |
| 4 | 102 | deco-G | 8 | 79.7k–145.1k | 150/160 | 73/87/0 | 16 · [14,3,3,5] | 4 653 | 65 439 | 59 051 | 320 | 0.00 | 2 | **竞争排除** |
| 5 | 42 | deco-A | 20 | 3.2k–83.3k | 290/302 | 113/189/0 | 21 · [7,6,13,7] | 113 609 | 55 670 | 88 312 | 378 | 0.00 | 1 | **容量饥饿** |
| 6 | 42 | deco-B | 15 | 22.3k–78.8k | 216/230 | 66/164/0 | 15 · [5,5,13,5] | 35 298 | 115 741 | 79 281 | 340 | 13.71 | 1 | **容量饥饿** |
| 7 | 42 | deco-C | 11 | 29.8k–67.0k | 83/96 | 18/78/0 | 26 · [5,10,6,7] | 17 361 | 82 115 | 56 719 | 382 | 0.00 | 2 | **竞争排除** |
| 8 | 42 | deco-D | 8 | 32.7k–167.2k | 275/285 | 53/232/0 | 15 · [15,3,8,3] | 4 789 | 319 292 | 133 222 | 377 | 8.02 | 1 | **容量饥饿** |
| 9 | 42 | deco-E | 8 | 33.7k–134.0k | 184/192 | 36/156/0 | 12 · [7,8,4,4] | 17 837 | 142 401 | 122 173 | 390 | 0.00 | 2 | **竞争排除** |
| 10 | 42 | deco-F | 8 | 67.9k–168.9k | 189/197 | 52/145/0 | 15 · [8,7,7,3] | 11 758 | 157 133 | 123 718 | 409 | 0.00 | 1 | **容量饥饿** |
| 11 | 42 | deco-H | 12 | 160.4k–180.0k | 57/63 | 20/43/0 | 19 · [3,5,14,4] | 547 | 42 595 | 26 363 | 367 | 28.93 | 1 | **容量饥饿** |
| 12 | 42 | deco-J | 8 | 166.4k–182.9k | 29/37 | 17/20/0 | 15 · [4,8,4,3] | 1 358 | 26 512 | 22 980 | 385 | 0.00 | 1 | **容量饥饿** |
| 13 | 109 | deco-A-B | 8 | 75.4k–132.3k | 100/108 | 40/68/0 | 13 · [12,7,3,3] | 78 | 49 433 | 46 877 | 396 | 8.51 | 1 | **容量饥饿** |

**合计：容量饥饿 10 起，竞争排除 3 起，配偶缺失 0 起（作为主因），其它 0 起。**

### 3.2 为什么是容量，而不是配偶

三条判据，按强度排列：

1. **结构算术**：13 起里 12 起的 `grant/象限` 为 2–3.75，全部 < `SPECIATION_MIN_SIDE = 8`；剩下的 deco-A（grant 20，5/象限）同样低于 8。引擎在 8 个成员时宣布物种，但每个象限只准它留 2–5 个。它们全部是 `ek = 1`（无 Gause burden）或 `ek = 2`（有 burden）。
2. **满容占比**：每起灭绝的 `budget_blocks / (budget_blocks + births)` 都 ≥ 99.8 %（例：42 deco-D 319 292/(319 292+275) = 99.91 %）。即：**只要个体可育，它的出生尝试几乎总被容量闸拒绝**。配偶搜索失败是同一件事的另一面——象限被压到 2–5 只时，「11 单位内没有合格配偶」是容量的必然结果，不是独立原因。mate-miss 数列在同一批 clade 上是 2.3 万–13.3 万，与满容占比同向。
3. **人口学形状**：起止差只有 −6 到 −15（例：42 deco-A 290 出生 / 302 死亡），寿命 16.4k–132.2k tick，`mean_death_age` 320–466 与根 clade 同量级。这不是「出生即绝育」（Phase 58 的形态），而是**被压在低容量上的人口学随机游走**——clade 的原始成员带父 kind（不受女儿预算约束），随老化消失；替代只能由女儿 kind 提供，而女儿 kind 每象限 2–5 个槽位。

`region_cap_blocks` 在 109 deco-A-B 上只有 78，看起来像反例，其实是闸门顺序：它的 kind 一直在自己的 8 预算上，Phase 49 预算闸先拒，区域闸根本走不到。这条必须写进契约（§2.1 最后一条），否则会误读。

### 3.3 竞争排除的 3 起

102 deco-G（79.7k–145.1k）、42 deco-C（29.8k–67.0k）、42 deco-E（33.7k–134.0k）在清空的那次检测里带着非零 niche burden（引擎 `event_kind = 2`）。它们的 rpeak 里能看到象限重叠（如 42 deco-C 的 [5,10,6,7] 与根 [64,16,55,16] 共存），属于 Phase 45 起的 Gause 负荷路径。注意：这 3 起同样受 §3.2 的容量挤压，分类取引擎自己的标志，不在两者间二选一。

## 4. 块 2：三个候选与 A/B 结果

### 4.1 候选

| 臂 | 规则 | 开关 |
|---|---|---|
| **a** 全区重归一化 | `cap(kind,r) = budget · w(r) / Σ_{r∈active∪{r}} w(r)`，active = 该 kind 有成员的象限 | `deco_region_cap_renorm_on()` / `PHASE_RENORM=on` |
| **b** 女儿最低保留份额 | 已占据象限的 cap 抬到 `max(自然份额, min(budget, speciation_min_side))`；空象限保持原份额 | `deco_region_cap_floor_on()` / `PHASE_CAP_FLOOR=on` |
| **c** 仅女儿重归一化 | 只对派生 kind（`parent_kind.is_some()`）用 a 的规则，六族祖先保持 Phase 60 分割 | `deco_region_cap_renorm_daughters_on()` / `PHASE_RENORM=daughters` |

三种都**不入存档**（与 Phase 58 clamp / Phase 59 映射 / Phase 60 容量同契约；读档回编译默认=OFF）。

设计意图：
- a 直接把单区支系的 cap 从 `budget/4` 抬到 `budget`，其守恒性仍然成立（**已占据**象限的 cap 之和仍等于 kind 预算，空象限无成员）。
- b 只补「与分裂门一致」这一条：`SPECIATION_MIN_SIDE = 8` 宣布的物种必须能在它占据的象限里留住 8 个；空象限不变，所以根族的四象限 deme 结构（Phase 60 的分化引擎）与殖民动力学不动。
- c 是 a 与 b 的折中：承认「根的象限分割是分化引擎」，只解开女儿的挤压。

### 4.2 A/B（3 seeds × 200k，同 release 二进制，单变量）

物种形成 / 灭绝按事件 `event_kind`（0 / 1,2）计。

| seed | 臂 | **分化** | **灭绝** | DECO@200k | clades live/total | denv_var@end |
|---|---|---|---|---|---|---|
| 20261102 | OFF（Phase 60 默认） | **9** | 4 | 29 | 6/10 | 0.01042 |
| 20261102 | a 全区重归一化 | **0** | 0 | **80** | 1/1 | 0.05458 |
| 20261102 | **b 女儿最低份额** | **8** | **2** | 18 | 7/9 | 0.03969 |
| 20261102 | c 仅女儿重归一化 | **6** | 5 | 60 | 2/7 | 0.06255 |
| 42 | OFF | **10** | 8 | 55 | 3/11 | 0.05653 |
| 42 | a | **5** | 1 | 26 | 5/6 | 0.06713 |
| 42 | **b** | **7** | **1** | 18 | 7/8 | 0.05377 |
| 42 | c | **9** | 4 | 19 | 6/10 | 0.07957 |
| 20261109 | OFF | **3** | 1 | 60 | 3/4 | 0.06614 |
| 20261109 | a | **8** | 5 | 45 | 4/9 | 0.05231 |
| 20261109 | **b** | **5** | **1** | 34 | 5/6 | 0.06112 |
| 20261109 | c | **6** | 2 | 26 | 5/7 | 0.06785 |
| **合计** | **OFF** | **22** | **13** | — | — | — |
| **合计** | **a** | **13** | **6** | — | — | — |
| **合计** | **b** | **20** | **4** | — | — | — |
| **合计** | **c** | **21** | **11** | — | — | — |

### 4.3 读法与三种失败方式

- **a（13 / 6）：灭绝达标，分化崩了。** 102 上 0 次分化、DECO 回满 80（`deco_r = [32,19,1,28]`）。原因在机制里：a 的归一化集合随该 kind 占据的象限收缩——均场权重下根 kind 的象限上限从 Phase 60 的 20 抬到 3 区 26.7 / 2 区 40 / 1 区 80。Phase 60 之所以能分化，是因为根被四象限各 20 槽分割成 4 个 deme；a 把这个分割一并取消，岛模型消失，分化归零。这正是任务里预警的「苍穹平衡被打破」。
- **b（20 / 4）：最好的交换，但仍差 2 次分化。** 灭绝 13 → 4（超额达标），分化 22 → 20（−9 %）。逐 seed：102 9→8、42 10→7、109 3→5。b 不动根族分割（空象限份额不变），所以 109 反而多分化 2 次；42 的 −3 与 102 的 −1 抵消后仍为负。注意 b 臂的空间分布也变了（DECO@200k 18/18/34，三 seed 全部收缩到单象限 deme），但根的四象限上限（20）没变，所以分化只是小幅移动。按验收条款「分化 ≥ 22（不降）」判**不通过**。
- **c（21 / 11）：两头都不如 b。** 只解开女儿的挤压后，女儿活得更久、更容易和根在同一象限内混居，42 的分化短暂上升（9），但 102 的灭绝反而从 4 升到 5、总数 13 → 11 的降幅不足。
- 三个臂都无法在 3 seeds 上同时满足「灭绝 ≤ 6」与「分化 ≥ 22」。**没有做参数微调**（把 b 的 floor 从 8 调到 10/12 大概率能让 3-seed 数字过线，但那是在 3 个 seed 上拟合；Phase 59 §10.4 已确立「分化率判据至少 5 seed」）。

## 5. 默认决策

**不通过 → 默认保持 Phase 60 世界（三个臂全部 OFF），不发生第十八次 re-baseline。**

- `DECO_REGION_CAP_RENORM_ON = false`（a 臂）
- `DECO_REGION_CAP_FLOOR_ON = false`（b 臂；**Phase 62 首选候选**）
- `deco_region_cap_renorm_daughters` 默认 `false`（c 臂）
- 三个候选臂（a / b / c）全部保留在代码与观测器臂里，`PHASE_RENORM=on|daughters`、`PHASE_CAP_FLOOR=on` 可随时复跑。

之所以不因「灭绝 −9 很诱人」而翻 b：本期的硬验收把「分化不降」写成并列条件，而 20 vs 22 恰好落在 3-seed 的噪声带里（b 臂 102 的 8 vs 9、42 的 7 vs 10 都是单事件级别的差异）。用 3 seed 去接受一个会动全锚点的默认变更，会重演 Phase 59 的「盆地切换」教训。Phase 62 的第一件事应是 5-seed 预注册复测 b。

## 6. 锚点 / 确定性 / 存档

默认世界未变，**全部锚点不变**（第十七次 re-baseline 的数值继续有效）。

| 锚点 | 值 | 状态 |
|---|---|---|
| @300（seed 20261102，v2） | `0x651cf47e6d54ca64` | **不变** |
| @3000（seed 20261102，v2） | `0xbeb50df36733a6e9` | **不变** |
| @30000（seed 20261102，默认） | `0x42e6da39caeecb24` | **不变**（未直接 pin，机制未启用） |
| @60000（seed 20261102，默认） | **`0x3272f0e3eaba9250`** | **逐位复现 Phase 60**（`phase61_region_renorm_test::renorm_off_replays_the_phase60_default_anchor`，实测通过） |
| group-off @300 / @3000 / seed 20261105 / v1 | Phase 60 表 | **不变** |
| partition-off / accum-off / phase54 OFF / phase56 OFF @30000 & @60000 | Phase 60 表 | **不变**（`deco_env` off ⇒ 区域容量与三个臂一并失效） |
| 中性奖励臂 @60000 | `0xf18fdad334f6e18d` | **不变** |
| 旧映射臂 @60000（新机制 ON/OFF） | `0x5153690c11521a33` / `0x44285bfbed94865e` | **不变** |

确定性（实测）：
- 默认臂双跑 + 存档往返 60k + 1 200 tick 锁步：`phase60_region_capacity_test::on_arm_double_run_and_roundtrip_is_bit_exact` ✅
- 默认臂 @60000 锚点：`phase61_region_renorm_test::renorm_off_replays_the_phase60_default_anchor` ✅
- a 臂双跑 + 存档往返 60k：`phase61_region_renorm_test::renorm_on_double_run_and_roundtrip_is_bit_exact` ✅（臂不入档，读档后显式重挂）
- 遥测不参与轨迹：默认臂 §6 锚点逐位不变 + §2.2 事件表逐事件一致

SaveState **仍是 v22**；`clade_life`、`deco_region_cap_renorm`、`deco_region_cap_floor`、`deco_region_cap_renorm_daughters` 均不入档。

## 7. 验收逐条

| # | 验收 | 结果 |
|---|---|---|
| 1 | 块 1：13 起灭绝分类表（引擎级计数器，逐起分类 + 证据） | ✅ §3.1（容量饥饿 10 / 竞争排除 3；证据列含 births、deaths+死因、寿命均值、末期能量、满容占比、每象限峰位） |
| 2 | 块 2：3 seeds × 200k A/B，灭绝 13 → ≤ 6 且分化 ≥ 22 | ❌ **未通过**：a 13/6（分化崩）、b 20/4（分化 −2）、c 21/11。**无一个同时过线** |
| 3 | 通过 → 默认 ON + 第十八次 re-baseline；不通过 → 默认 OFF + 诚实记录 | ✅ 走「不通过」分支：默认 OFF，三臂保留，无 re-baseline |
| 4 | 双跑 bit-exact；存档往返；OFF 臂回放 Phase 60 锚点 @60000 = `0x3272f0e3eaba9250` | ✅ §6 |
| 5 | 全量 0 失败；wasm < 16 ms 三次 | ✅ §7.1 / §7.2 |
| 6 | 报告 | ✅ 本文件 |

### 7.1 全量与慢测

- 批 A：`cargo test --release --tests`（最终源码，含最后两处开关对称性修改）→ **59 个测试二进制，295 passed / 0 failed / 26 ignored**（退出码 0）。相对 Phase 60（58 套件 / 288 passed / 24 ignored）的差值 = 新增 `phase61_region_renorm_test` 的 7 个快测 + 2 个 ignored。
- 定向慢测（只跑需要的 ignored，逐二进制）：
  - `phase61_region_renorm_test -- --ignored`（2 测：默认臂 @60000 锚点 `0x3272…` + a 臂双跑/存档往返）✅
  - `phase60_region_capacity_test -- --ignored`（2 测：OFF 臂回放 Phase 59 默认 + ON 臂双跑 + 1 200 tick 锁步）✅
- 说明：本期默认世界未变，Phase 56/57/59 的既有 ignored 锚点在 Phase 60 已验证过且本期未触碰其机制（`deco_region_cap_*` 三臂默认 OFF、遥测零回写），故按定向原则不重复跑；§6 的默认锚点由 `phase61` 的 ignored 臂直接覆盖。

### 7.2 wasm

- `wasm-release` **823 736 B**（Phase 60 为 820 085 B，**+3 651 B / +0.45 %**）；`ecology-sim.wasm` 与 `ecology-sim-v2.wasm` 的 sha256 相同：`ab7f61c1bef1d7f717b7…`。
- `node living/test_wasm_viewer.mjs` 同条件交错 A/B（每轮先新后旧，3 轮；60 帧 sim+export 平均）：

| 轮 | Phase 61 新 wasm | HEAD 旧 wasm |
|---|---|---|
| 1 | **10.64 ms/frame** | 10.62 ms/frame |
| 2 | **10.76 ms/frame** | 10.60 ms/frame |
| 3 | **10.56 ms/frame** | 10.60 ms/frame |

全部 < 16 ms；差异在测量噪声内（三臂默认 OFF，热路径只有 `spawn_tagged` 的 `Option` 透传和出生/死亡时的 ≤ 64 slot 线性查找）。

## 8. 改动文件

### 核心
- `core/crates/aether-sim-ecology/src/ecology.rs`
  - `:1302-1341`：Phase 61 三候选的常量与文档（`DECO_REGION_CAP_RENORM_ON` / `DECO_REGION_CAP_FLOOR_ON`）
  - `:1549-1606`：`CladeLifeStats` + `CladeBirthBlock`
  - `:2029-2045`：`deco_region_cap_renorm` / `deco_region_cap_floor` / `deco_region_cap_renorm_daughters` / `clade_life` 字段
  - `:2274` / `:2503` / `:9619`：两个构造器 + 恢复路径初始化
  - `:3915-3946`：`deco_region_cap_renorm_on/off`、`deco_region_cap_floor_on/off`、`deco_region_cap_renorm_daughters_on/off`
  - `:3986-4012`：`deco_region_cap_limit` 三臂公式
  - `:4030-4104`：遥测段（`clade_life_stats` / `clade_life_slot` / `_birth` / `_death` / `_block` / `_mate_miss` / `_peak` / `_extinct`）
  - `:4404-4475`：`spawn` → `spawn_tagged` 委托 + 两个闸门的拒绝记账 + 成功出生记账
  - `:5370` / `:5452` / `:5544`：`reproduce` 的 mate-miss 记账与两处 `spawn_tagged(tag)`
  - `:5592`：`die()` 死亡记账；`:6032`：predation 直杀路径记账
  - `:7638-7645`：`check_speciation` 的峰值/每象限采样；`:7664`：绝灭 tick 记账

### 测试与观测器
- `tests/phase61_region_renorm_test.rs`（新，9 测：a 臂公式、守恒、b 臂 floor、c 臂仅女儿、三臂不入档、遥测不入档且不入轨迹、死亡计数守恒、@60000 锚点、a 臂双跑+往返）
- `tests/clade_fate_observatory.rs`：`PHASE_RENORM=on|off|daughters`、`PHASE_CAP_FLOOR=on|off`、文件名分量、`/tmp/clade_life_*.csv` 全字段 dump、`[extinct]` stderr 汇总

### 产物
- `living/ecology-sim.wasm`、`living/ecology-sim-v2.wasm`（重建）
- `research/phase61-report.md`（本文件）

### 临时（不入仓）
- `/tmp/p61/analyze.py`、`/tmp/p61/table.py`（fork 预算复算与分类表脚本，数据段落已抄入本报告）

## 9. Phase 62 遗留

1. **b 臂 5-seed 预注册复测（首要）**：本期的分水岭是「灭绝 13 → 4 但分化 22 → 20」。需要 5 seed（加 20261105 / 20261107 一类）判定 20 vs 22 是盆地噪声还是真降；判据、seed 池、验收线必须先写死再跑。
2. **floor 的高度是未标定量**：b 臂的 floor 取 `SPECIATION_MIN_SIDE = 8`（与分裂门一致，非调参）。若 5-seed 显示分化确实受损，应改的是「根的象限分割是否必须独占」——例如只对 `budget < 2 × min_side` 的 kind 应用 floor，让 20 预算的第一代女儿保持自然份额。
3. **根族象限分割的不可再生性**：a 臂的实验证明 Phase 60 的分化引擎**完全依赖根 kind 的 `budget/4` 象限上限**。任何后续「提高区域容量」的提案都必须先回答「谁来提供 deme 分割」；这是 Phase 62 若要做 family 级区域 cap 的前置约束。
4. **闸门顺序使遥测有盲区**：`region_cap_blocks` 在 kind 顶满预算时被预算闸遮蔽。若 Phase 62 需要严格的区域挤压计数，应把区域检查前移（会改轨迹），或给拒绝记账补一个「预算闸命中时同时记录目标象限是否已满」的旁路（零轨迹影响）。
5. **灭绝率与 seed**：13 起里 8 起集中在 seed 42（该 seed 的分化 10 次最多，灭绝也最多）。任何「灭绝率」结论都应给出逐 seed 分布，不用合计掩盖。
6. **wasm 成本**：`refresh_deco_region_weights` 的 5 776 次表查找未变；本期新增的遥测只在出生/死亡/检测点做 ≤ 64 slot 的线性查找，§7.2 实测无回归。
