# Phase 68 — 三段漏斗遥测：成配 → 出生 → 容量拒绝 → 死亡（纯遥测，零默认变更）

日期：2026-11-23
HEAD（起点）：`7867548`（Phase 67：R3 默认 OFF，无 re-baseline）
性质：**纯遥测 / 诊断**。不新增机制、不改默认、不 re-baseline。目标是把 Phase 61-67 分散的计数器合成逐 clade 的可审计漏斗，回答「为什么更多成配反而更多灭绝」。

> 本文件的第一版只包含 §1，在第一个 Phase 68 观测进程启动之前单独提交。运行期间 §1 不做任何修改；§2 之后只追加。

---

## 1. 预注册（跑之前写死）

### 1.1 已知与未知

**已知（Phase 61-67 的定因链）**：

| 期 | 结论 | 证据 |
|---|---|---|
| 61 | 13 起灭绝：容量饥饿 10、竞争排除 3、配偶缺失 0 | 引擎级 per-clade 计数器 |
| 62 | 真正约束女儿支的是 Phase 49 kind 预算（8-20 槽），不是区域份额 | b/d 臂对照 |
| 63/64 | `FORK_BUDGET_MIN 8→10` 在 3-seed 过线，5 个全新 seed 不复现，回退 8 | out-of-sample 复测 |
| 65 | W1 建支宽限期 390 次宽限出生、69 % 消散；约束在招募，不在槽位闸 | 宽限臂 |
| 66 | R1/R2 大择偶半径把 miss 降 41-77 %，灭绝反升 23 % | 5-seed 200k |
| 67 | R3 小半径 31→44 分化、21→25 灭绝；派生 births/deaths +57 %，容量拒绝 +55-62 %，DECO 全族恒 80 | 5-seed 200k |

Phase 67 的关键读法是：R3 没有创造新生物量，只是在同一个天花板下把周转加速了；44.00 尖峰真实存在且被 R3 消掉，但存活没有变好。**约束在下游（容量闸），不在搜索半径。**

**未知（本期要回答的）**：

1. 上述结论是从**分散的计数器**推断的：`mate_misses`、`births`、`region_cap_blocks`、`kind_budget_blocks`、死因各自独立，没有按 clade 把「找到配偶 → 兼容通过 → 尝试出生 → 出生/被拒 → 死亡」串成一条漏斗。因此无法回答：R3 多出来的出生到底死在哪一段？
2. `clade_life` 的行键是 `(clade_id, root_kind)`：同一 clade 行里若混入根 kind agent，会污染派生支系的行。Phase 67 残余的 8/44 个 `44.00` 就是直接证据。
3. Phase 61 的 `region_cap_blocks` 被 `kind_budget_blocks` 遮蔽（闸门顺序：kind 预算先查，区域容量后查），所以「区域挤压」可能被系统性低估。

### 1.2 科学问题

- **Q1（主）**：对派生 DECO 支系，繁殖漏斗的损失主要发生在哪一段——成配、兼容、出生尝试、容量拒绝，还是出生后的死亡？
- **Q2**：R3 的额外出生是「在瓶颈前增加吞吐」还是「把更多个体推进瓶颈后死亡」？即 R3 是否移动了损失点？
- **Q3**：跨过出生段之后，死亡主要由饥饿、衰老还是捕食贡献？
- **Q4**：把 `kind_id` 加入行键后，Phase 67 的 `44.00` 残余是否消失，派生支系的作用域是否变纯？

### 1.3 插桩定义（写死）

`CladeLifeStats` 新增字段，全部为零 RNG、只读记账、不入档：

| 字段 | 定义 | 插入点 |
|---|---|---|
| `kind_id: u32` | 该行的真实 kind（根 kind 或派生 kind）；`u32::MAX` 仅作未知哨兵 | 行键 |
| `repro_ready` | 已过成熟年龄、冷却、能量阈值，即将执行配偶搜索的母亲 tick | `reproduce()` 能量闸之后、`nearest` 之前 |
| `mate_finds` | `nearest` 返回 `Some`（在 hue/barrier/sig 之前） | `mate_opt.is_some()` 分支入口 |
| `mate_reject_hue` | 找到配偶但 hue 兼容 roll 失败 | `hue_diff > 0.35` 分支的 `roll < 0.5` 早退 |
| `mate_reject_barrier` | 找到配偶但基因屏障拒绝 | `barrier_blocks += 1` 早退 |
| `mate_reject_sig` | 找到配偶但 sig 匹配失败 | `sig_blocks += 1` 早退 |
| `sexual_attempts` | 通过全部兼容检查，即将调用 `spawn_tagged` 的性繁殖尝试 | 性繁殖 `spawn_tagged` 之前 |
| `asexual_attempts` | 性繁殖路径失败后进入无性 fallback 的 spawn 尝试 | 无性 `spawn_tagged` 之前 |
| `birth_attempts` | **所有** spawn 尝试 = `sexual_attempts + asexual_attempts` | 两个 spawn 入口各加一次 |
| `sexual_births` | 性繁殖 spawn 成功 | 性繁殖 `spawn_tagged` 返回 `Some` 之后 |
| `asexual_births` | 无性 fallback spawn 成功 | 无性 spawn 成功之后 |

保留既有字段：`births`（总出生）、`kind_budget_blocks`、`region_cap_blocks`、`deaths_*`、`mate_misses`、`life_ticks`。

**行键变更**：`(clade_id, root_kind)` → `(clade_id, kind_id)`；`root_kind` 字段保留，用于家族显示与聚合。这样根 kind agent 不再混入派生 kind 行。

**恒等式（必须在每个 clade 行上成立）**：

1. `mate_finds ≤ repro_ready`
2. `sexual_attempts ≤ mate_finds`
3. `sexual_attempts + asexual_attempts = birth_attempts`
4. `sexual_births + asexual_births + kind_budget_blocks + region_cap_blocks = birth_attempts`
5. `births = sexual_births + asexual_births`
6. `asexual_births ≤ asexual_attempts`
7. `mate_reject_hue + mate_reject_barrier + mate_reject_sig ≤ mate_finds`

> **预注册修正（2026-11-23，5k 冒烟后、正式 200k 之前）**：冒烟发现 `spawn_tagged` 的容量拒绝同时覆盖有性与无性路径，而初版 `birth_attempts` 只数有性，导致 root DECO 出现 `region_blocks > attempts`。现将尝试数拆为 `sexual_attempts` / `asexual_attempts`，`birth_attempts` 定义为两者之和；判据、seed 池、作用域与指标不变。此修正在任何 200k 判定臂启动之前完成。

> 恒等式 3 依赖 `spawn_tagged` 只有两个失败出口（kind 预算、区域容量）；当前源码确实如此（见 Phase 68 实现时复核）。

**口径边界**：

- `repro_ready` 是「母亲 tick」计数，不是「唯一母亲」计数；同一母亲每 tick 最多一次，冷却期内不会重复。
- `mate_finds` 是搜索命中，不是成配成功；成配成功 = `birth_attempts` 之前的全部兼容检查通过。
- 无性 fallback 独立计数；它不经过配偶搜索，不能混入性繁殖漏斗。
- 所有计数按母亲的 `clade_id` 与 `kind_id` 归属；`spawn_tagged` 的失败按母亲的 tag 归属（与 Phase 61 一致）。
- `kind_id` 行只在实际有计数的 kind 上创建；`u32::MAX` 不建行。

### 1.4 作用域

| 作用域 | 定义 | 用途 |
|---|---|---|
| **派生 DECO** | `root_kind = DECO ∧ kind_def(kind_id).parent_kind.is_some()` | Q1/Q2 主作用域 |
| 根 DECO | `root_kind = DECO ∧ parent_kind.is_none()` | 对照；证明派生支系的漏斗与根不同 |
| 其它家族 | `root_kind != DECO` | 报告项；不参与主判定 |
| 年轻支系 | `tick − clade.birth_tick < YOUNG_CLADE_TICKS` | 报告项；Phase 66/67 的「新建支系」窗口 |

### 1.5 指标（写死）

**段间存活率（每个 seed、每个作用域）**：

- `find_rate = mate_finds / repro_ready`
- `compat_rate = sexual_attempts / mate_finds`
- `birth_rate = sexual_births / sexual_attempts`
- `asexual_attempt_rate = asexual_attempts / (sexual_attempts + asexual_attempts)`
- `kind_block_rate = kind_budget_blocks / birth_attempts`
- `region_block_rate = region_cap_blocks / birth_attempts`
- `sexual_share = sexual_births / births`

**按 clade-tick 归一（主指标）**：

- `births_per_ktick = births / life_ticks × 1000`
- `deaths_per_ktick = deaths / life_ticks × 1000`
- `net_per_ktick = (births − deaths) / life_ticks × 1000`

**按出生归一（报告项）**：

- `deaths_per_birth = deaths / births`
- `starved_share = deaths_starved / deaths`
- `aged_share = deaths_aged / deaths`
- `predation_share = deaths_predation / deaths`

**漏斗损失表**：对派生 DECO，逐段给出绝对计数、占 `repro_ready` 的份额、以及每 clade-tick 的损失率。主读法是「绝对损失最大的段」，不是「比率最低的段」。

### 1.6 运行设计（写死）

| 项 | 内容 |
|---|---|
| 主观测 | 5 个全新 seed `20261208 / 09 / 10 / 11 / 12` × 200k，编译默认（不设任何臂开关） |
| R3 描述性对照 | `20261208 / 09` × 200k，`PHASE_MATE_FLOOR=r3`；只比较漏斗形状，不重新裁定 R3 |
| 备用 seed | `20261213 / 14`，仅在主 seed 退化时按序替换，最多 2 个 |
| 退化定义 | `spec + ext = 0`，或 DECO 根 clade 在 `t < 10 000` 灭绝 |
| 默认自检 | `20261102` / 200k 必须复现 **9 分化 / 4 灭绝** |
| 双跑 | `20261208` 默认臂跑两次（不同 `PHASE_ARM_TAG`），`clade_life` 与 `clade_fate` CSV 逐字节相同 |
| 快筛 | 不设 80k 门；不因中间结果停臂；不追加 seed |
| 并发 | 每批 ≤ 4 进程 |

### 1.7 数据质量门（D0）

| # | 条件 | 失败处理 |
|---|---|---|
| D0-1 | 派生 DECO 作用域内至少一个 seed 满足 `repro_ready > 0 ∧ mate_finds > 0 ∧ birth_attempts > 0 ∧ sexual_births > 0` | 仪器未咬到；报告 inconclusive，不据此做 Phase 69 决策 |
| D0-2 | §1.3 的 6 条恒等式在每个 clade 行成立（测试断言 + CSV 复算） | 实现错误；先修仪器，不跑判定 |
| D0-3 | `kind_id` 行分离：seed `20261201` 的 DECO 根 id=1 场景下，根行与派生行不再混记；Phase 67 的 `44.00` 残余归零 | 作用域仍有污染；报告标注，P3 类指标降级为报告项 |
| D0-4 | 默认 `@60000` 锚点 `0x3272f0e3eaba9250` 位精确不变 | 默认轨迹被扰动；停止，回滚 |
| D0-5 | `20261208` 双跑 bit-exact | 确定性破坏；停止，定位 |
| D0-6 | wasm 60 帧三轮均 < 16 ms | 性能回归；优化或回滚插桩 |

### 1.8 Phase 69 决策表（写死）

| # | 数据形态 | Phase 69 方向 |
|---|---|---|
| 1 | `kind_block_rate` 是出生段最大损失，且 `births_per_ktick` 被封顶 | **容量/kind 预算侧**：优先测「女儿区域份额的临时保护」或「承载容量由分解场/资源流导出」；不再单独动择偶半径 |
| 2 | `compat_rate` 低，`mate_reject_*` 占主导 | **兼容侧**：hue/barrier/sig；半径不是主因 |
| 3 | `birth_rate` 高，但 `deaths_per_birth` 高且 `starved_share` 高 | **出生后能量/存活侧**：幼体能量、区域资源、死亡前能量 |
| 4 | `find_rate` 低，`mate_misses` 占主导 | **招募侧**：才回到择偶，且必须与容量侧同时设计 |
| 5 | 无单段损失 > 50 % | **多段**：Phase 69 做两段联合实验，不做单旋钮 |
| 6 | R3 的额外出生被 kind/region 拒绝吸收，死亡同比上升 | **关闭半径线**：R3 是吞吐放大器，不是招募修复；后续不再为半径立项 |
| 7 | D0 失败 | **不决策**：先修仪器 |

### 1.9 明确不做

- 不新增机制、不改默认、不 re-baseline。
- 不重跑 Phase 61/63/65/66/67 的已否证臂（a/b/c、M10、W1、R1/R2/R3 转正）。
- 不把 2-seed R3 当作 R3 的重新裁定；它只是漏斗形状的描述性对照。
- 不因中间结果改指标、加 seed、去掉 R3 对照。
- 不做观察层/渲染功能；只做 headless 遥测。
- 不在本期修任何「下游约束」；诊断与修复分离。

### 1.10 执行协议

1. 实现 §1.3 计数器与 `kind_id` 行键；补 §1.3 的恒等式单元测试。
2. 跑默认自检与 `@60000` 锚点；全 workspace 测试 0 失败。
3. 编译一次 release 测试二进制；所有臂复用，只改环境变量。
4. 跑 5 个主 seed × 200k + 2 个 R3 对照 + 1 个双跑；每批 ≤ 4 进程。
5. 重建 wasm；三次同会话交错 A/B，60 帧平均 < 16 ms。
6. 写 `research/phase68-report.md` §2 之后：逐 seed 漏斗表、D0、锚点、测试、wasm、Phase 69 决策。

---

## 2. 实现

### 2.1 行键与计数器

- `CladeLifeStats` 增加 `kind_id: u32`；行键从 `(clade_id, root_kind)` 改为 `(clade_id, kind_id)`。`root_kind` 保留用于家族显示。
- 新增漏斗字段：`repro_ready`、`mate_finds`、`mate_reject_hue`、`mate_reject_barrier`、`mate_reject_sig`、`birth_attempts`、`sexual_attempts`、`sexual_births`、`asexual_attempts`、`asexual_births`。
- 插桩点：`reproduce()` 的能量闸之后、`nearest` 之前（`repro_ready`）；`nearest` 返回 `Some`（`mate_finds`）；hue / 硬基因屏障 / sig 早退（三个 reject）；性/无性 `spawn_tagged` 之前（attempts）；spawn 成功之后（births）。
- `spawn_tagged` 的两个失败出口（kind 预算、区域容量）沿用既有 `clade_life_block`，现在按 `(clade_id, kind_id)` 归属。
- 全部为零 RNG、只读记账、不入档；`clade_life` 仍是瞬态遥测。

### 2.2 恒等式与偏差声明

**成立（每个 clade 行）**：

1. `mate_finds ≤ repro_ready`
2. `sexual_attempts ≤ mate_finds`
3. `sexual_attempts + asexual_attempts = birth_attempts`
4. `sexual_births + asexual_births + kind_budget_blocks + region_cap_blocks = birth_attempts`
5. `births = sexual_births + asexual_births`
6. `asexual_births ≤ asexual_attempts`
7. `mate_reject_hue + mate_reject_barrier + mate_reject_sig ≤ mate_finds`

9 个 run（自检 1 + 主 seed 5 + R3 2 + 双跑 1）的 `[funnel] identity` 全部 `ok=true`，`clade_life` CSV 复算也一致。

**偏差 1（软基因屏障未单列）**：`reproduce()` 里还有一条软屏障早退——`barrier_roll >= (1 − dist × gene_barrier_strength)` 时直接 `return`，不打 `barrier_blocks`。本期新增的 `mate_reject_barrier` 只覆盖硬屏障（lineage 隔离）分支。残差 `mate_finds − sexual_attempts − hue − hard_barrier − sig` 在各 seed 为 **50 841–112 830**，占 `mate_finds` 的 **6.8–7.9 %**。报告把它列为 `compat_unclassified`，不参与判定；Phase 69 若要补齐，应在软屏障早退处加一个独立计数器。

**偏差 2（漏斗是尝试加权，不是个体加权）**：被容量闸拒绝的母亲下一 tick 仍会重试，因此 `birth_attempts`/`blocks` 是母亲-tick 计数，不是唯一母亲计数。`birth = sexual_births / sexual_attempts = 0.001` 不能读成「99.9 % 的母亲绝育」；应读成「每次尝试几乎都被容量闸拒绝」。主指标因此固定为每 clade-tick 的出生/死亡率与每次尝试的拒绝份额，而不是原始成功率。

**偏差 3（闸门顺序）**：`spawn_tagged` 先查 kind 预算，再查区域容量；kind 预算失败时区域容量根本不会被求值。因此 **kind_block 份额是上界，region_block 份额是下界**。这与 Phase 61 §9.4 的盲区一致，Phase 69 若要拆分，必须在 kind 预算失败时也做一次只读的区域容量判定。

---

## 3. 执行

| 项 | 内容 |
|---|---|
| 编译 | 一次 release 测试二进制；所有 run 复用，只改环境变量 |
| 默认自检 | `20261102 / 200k` = **9 分化 / 4 灭绝**，与 Phase 64/65/66/67 基线一致 |
| 主 seed | `20261208 / 09 / 10 / 11 / 12` × 200k，编译默认 |
| R3 描述性对照 | `20261208 / 09` × 200k，`PHASE_MATE_FLOOR=r3` |
| 双跑 | `20261208` 默认臂两次（`p68d1` / `p68dbl`），`clade_life` / `clade_fate` / `clade_tree` CSV 逐字节相同；`[funnel]` / `[split]` / `[fate]` 日志行逐行相同 |
| 退化替换 | 未触发；`20261212`（d5）虽然终局 `live_clades=0`，但 `spec+ext=21` 且 DECO 根未在 10k 前灭绝，按 §1.6 保留 |
| 并发 | 4 进程/批；单臂 970–1 563 s |

**d5 是质量离群点**：`10 分化 / 11 灭绝`，终局 0 live clades（11 total）。它是真实的集群灭绝样本，不是仪器失败；排除它后容量结论不变（见 §4.2）。

---

## 4. 结果

### 4.1 分化与灭绝（200k）

| seed | 分化 | 灭绝 | live/total clades |
|---|---|---|---|
| 20261102（自检） | 9 | 4 | 6 / 10 |
| 20261208 | 7 | 5 | 3 / 8 |
| 20261209 | 5 | 1 | 6 / 7 |
| 20261210 | 6 | 2 | 6 / 8 |
| 20261211 | 8 | 3 | 7 / 10 |
| 20261212 | 10 | 11 | 0 / 11 |
| **5-seed 合计** | **36** | **22** | **22 / 44** |

### 4.2 派生 DECO 漏斗（主作用域）

每 seed 的派生 DECO 聚合；`births/kt`、`deaths/kt` 的分母是该 seed 所有派生行的 `life_ticks` 之和。

| seed | 派生行 | clade-ticks | births/kt | deaths/kt | find | compat | kind_block | region_block | deaths/birth | starved |
|---|---|---|---|---|---|---|---|---|---|---|
| 20261208 | 9 | 983 078 | 2.04 | 2.04 | 0.487 | 0.823 | 0.896 | 0.103 | 0.997 | 0.251 |
| 20261209 | 6 | 616 899 | 2.65 | 2.64 | 0.575 | 0.826 | 0.916 | 0.083 | 0.998 | 0.198 |
| 20261210 | 6 | 473 020 | 3.22 | 3.19 | 0.539 | 0.848 | 0.931 | 0.068 | 0.993 | 0.212 |
| 20261211 | 9 | 979 199 | 2.38 | 2.38 | 0.579 | 0.818 | 0.959 | 0.040 | 1.000 | 0.239 |
| 20261212 | 11 | 817 682 | 1.92 | 1.99 | 0.469 | 0.837 | 0.950 | 0.049 | 1.039 | 0.217 |
| **合计** | **41** | **3 869 878** | **2.341** | **2.351** | **0.535** | **0.829** | **0.932** | **0.067** | **1.004** | **0.226** |

**读数**：

- **出生段几乎全被容量闸吸收**：`kind_block` **93.2 %**、`region_block` **6.7 %**，两者合计 99.9 % 的 spawn 尝试被拒绝。
- **成配不是主瓶颈**：`find` 53.5 %、`compat` 82.9 %；即使把 `find` 做到 100 %，尝试数最多增加约 0.87 倍，而尝试的 99.9 % 仍会被容量闸挡住。
- **出生与死亡基本持平**：`deaths/birth = 1.004`，`births/kt ≈ deaths/kt ≈ 2.34`。派生支系是「被容量闸限流的稳态周转」，不是「出生后大量饿死」。
- **死亡侧以衰老为主**：`aged` 77.4 %、`starved` 22.6 %、`predation` 0 %。Phase 61 的「容量饥饿」是**出生侧**的拒绝，不是**死亡侧**的饥饿。
- **排除 d5 后结论不变**：4-seed 派生 `kind_block = 92.9 %`、`births/kt = 2.455`、`deaths/birth = 0.997`。

### 4.3 根 DECO 对照

| 作用域 | clade-ticks | births/kt | deaths/kt | find | compat | kind_block | region_block | deaths/birth | starved |
|---|---|---|---|---|---|---|---|---|---|
| 派生 DECO（5 seed） | 3 869 878 | 2.341 | 2.351 | 0.535 | 0.829 | 0.932 | 0.067 | 1.004 | 0.226 |
| 根 DECO（5 seed） | 5 018 024 | 1.442 | 1.413 | 0.694 | 0.815 | 0.881 | 0.118 | 0.980 | 0.213 |

根 DECO 的成配率更高（0.694 vs 0.535），但出生率更低（1.44 vs 2.34/kt）；根族同样主要被 kind 预算挡住（88.1 %），区域容量份额更高（11.8 %）。**派生支系不是「成配更差」的支系，而是「出生被更紧地限流」的支系**。

### 4.4 R3 对照（2 seed，描述性，不重新裁定 R3）

同一 2 个 seed（20261208/09）的 OFF 与 R3：

| 指标 | OFF | R3 | 变化 |
|---|---|---|---|
| 分化 / 灭绝 | 12 / 6 | 13 / 3 | 2-seed，方向混合，不作判定 |
| 派生 births/kt | 2.276 | 2.408 | +5.8 % |
| 派生 deaths/kt | 2.269 | 2.389 | +5.3 % |
| `find` | 0.528 | 0.600 | **+7.2 pp** |
| `compat` | 0.825 | 0.827 | +0.2 pp |
| `kind_block` | 0.906 | 0.889 | −1.7 pp |
| `region_block` | 0.093 | 0.110 | +1.7 pp |
| `deaths/birth` | 0.997 | 0.992 | −0.5 pp |
| `starved` 份额 | 0.227 | 0.254 | +2.7 pp |

R3 确实提高了成配率（+7.2 pp），但 births/kt 只 +5.8 %、deaths/kt +5.3 %；容量拒绝份额几乎不变（kind 89 % + region 11 %）。**R3 的额外成配被容量闸吸收，没有转化为净存活**。这与 Phase 67 的「吞吐放大器」读法一致，且本期给出了段级证据。

### 4.5 `44.00` 残差与 `kind_id` 修复

| 臂 | 派生行中 `beyond_radius` 且 `min_ready_dist = 44.00` | `min_ready_dist < 80` |
|---|---|---|
| OFF（5 seed） | 6 / 5 / 5 / 7 / 9 行 | 同左（OFF 派生常用 legacy 44 半径，44.00 是真实的） |
| R3（2 seed） | **0 / 0 行** | **0 / 0 行** |

Phase 67 的残余污染（R3 下派生行仍记 44.00）在 `kind_id` 分维后**归零**：R3 的派生 kind 搜索半径恒 ≥ 80，不可能产生 44.00；OFF 下的 44.00 是 legacy 半径的真实边界，不是污染。**作用域修复得到直接验证。**

---

## 5. 决策

**按 §1.8 决策表：第 1 行（容量/kind 预算侧）+ 第 6 行（关闭半径线）。默认世界不变，不 re-baseline。**

依据：

1. `kind_block_rate = 0.932` 是出生段最大损失；`region_block_rate = 0.067` 是下界（被闸门顺序遮蔽）。
2. `births/kt` 被封顶在 2.34；`deaths/birth ≈ 1.00`，不是出生后存活崩溃。
3. `find = 0.535`、`compat = 0.829`：成配/兼容不是主约束；R3 把 find 提高 7.2 pp，仍被容量闸吸收。
4. 死亡侧 `aged = 77.4 %`、`starved = 22.6 %`：出生侧的容量拒绝与死亡侧的饥饿是两件事，不能再用「容量饥饿」混称。

### 5.1 Phase 69 建议（按预注册决策表，不提前拍板机制）

**优先方向：容量/kind 预算侧。** 候选按风险从低到高：

1. **先补闸门顺序盲区（纯遥测）**：在 kind 预算失败时也做一次只读的区域容量判定，记录 `would_region_block`，把 6.7 % 的下界变成真实份额。不改轨迹。
2. **女儿区域份额的临时保护**：fork 后前 N tick，女儿在其占据象限的 region cap 取 `max(自然份额, min(budget, SPECIATION_MIN_SIDE))`，到期解除。与 Phase 61 b 臂（永久 floor）不同：只保护建支窗口，不永久压缩根族分割。
3. **资源流导出的承载容量**：让 region cap 跟随分解场/局部尸体供给，而不是固定 kind 预算份额；前提是保留根 kind 的 deme 分割，否则会重演 Phase 61 a 臂（分化归零）。

**不再做**：择偶半径（R1/R2/R3 三种剂量、两期已否证）；区域份额永久重归一化（61/62）；`FORK_BUDGET_MIN` 地板微调（63/64）；W1 出生宽限（65）。

**判据建议**：Phase 69 的 5-seed 200k A/B 应同时预注册「分化非劣性」（如 ≥0.90 × OFF）与「出生率提升」（如 births/kt ≥1.10 × OFF），并把 `deaths/birth` 作为非劣性门；不要再用单独的灭绝相对比例。

---

## 6. 锚点 / 测试 / wasm

| 项 | 结果 |
|---|---|
| 默认 `@60000` | `0x3272f0e3eaba9250` 位精确（phase61 `renorm_off…`，281.72 s） |
| 默认 200k | seed 20261102 = 9 分化 / 4 灭绝 |
| 双跑 | `20261208` 默认臂两次：`clade_life` / `clade_fate` / `clade_tree` CSV 逐字节相同；关键日志行逐行相同 |
| 生态 crate 全测试 | 见 §6.1 |
| wasm | **830 787 B**，sha256 `c8d1e813d0a7c0bae0ddf89faecd78a61e013bf99ad07ebcfb6d73643254c7bf`（Phase 67 旧 830 464 B / `a0dc57f4…`；+323 B） |
| 帧预算 | 三轮同会话交错：新 14.65 / 14.50 / 14.50 ms；旧 14.37 / 14.55 / 14.48 ms；全部 <16 ms |
| wasm 冒烟 | `WASM VIEWER SMOKE OK`（零导入、flat census、snapshot v3、FinColor 契约全过） |

### 6.1 测试计数

| 范围 | 结果 |
|---|---|
| 生态 crate（`cargo test -p aether-sim-ecology --release --tests`） | **50 二进制 / 300 passed / 0 failed / 28 ignored** |
| 其余 workspace（`--workspace --exclude aether-sim-ecology --tests`） | **12 二进制 / 75 passed / 0 failed / 0 ignored** |
| 定向慢测（`#[ignore]`） | `@60000` 默认锚点 281.72 s ✅；Phase 56 / 60 / 63 三个默认锚点慢测见 §6.2 |

### 6.2 定向慢测

4 个默认锚点慢测全部通过（0 failed）：

| 测试 | 耗时 |
|---|---|
| phase61 `renorm_off_replays_the_phase60_default_anchor`（@60000） | 281.72 s |
| phase56 `off_arm_restores_the_pre_phase56_anchor` | 298.88 s |
| phase60 `off_arm_replays_the_phase59_default_anchor` | 355.60 s |
| phase63 `default_world_double_run_and_roundtrip_is_bit_exact` | 624.35 s |

---

## 7. 改动文件

### 核心
- `core/crates/aether-sim-ecology/src/ecology.rs`
  - `CladeLifeStats`：`kind_id` + 9 个漏斗字段
  - `clade_life_slot`：行键改为 `(clade_id, kind_id)`
  - `clade_life_funnel` + `FunnelStage`
  - `reproduce()`：`repro_ready` / `mate_finds` / 三个 reject / sexual & asexual attempt/birth
- `core/crates/aether-sim-ecology/tests/clade_fate_observatory.rs`
  - `clade_life` CSV 加 `kind_id,kind` 与 9 个漏斗列
  - `is_derived_deco` 改为读行自己的 `kind_id`，不再按 clade 反查
  - `[funnel]` 派生/根聚合与身份式
- `core/crates/aether-sim-ecology/tests/phase61_region_renorm_test.rs`
  - 旧「booked an event」断言纳入漏斗计数
- `living/ecology-sim.wasm`、`living/ecology-sim-v2.wasm`：830 787 B

### 文档
- `research/phase68-report.md`（本文件；§1 与预注册修正分别单独提交）

### 临时（不入仓）
- `/tmp/p68/*`（日志、CSV、parser、旧 wasm）

---

## 8. Phase 69 遗留

1. **闸门顺序盲区**：kind 预算失败时 region cap 不被求值；Phase 69 先加只读 `would_region_block`，再决定是否动女儿份额。
2. **尝试加权高估**：容量拒绝按母亲-tick 计，一个被挡母亲会重复计数。Phase 69 应加「首次拒绝 / 冷却窗口」计数，避免把重试当成独立事件。
3. **软基因屏障未单列**：`compat_unclassified` 占 finds 的 6.8–7.9 %；Phase 69 若继续看成配，补一个 `mate_reject_soft_barrier`。
4. **d5 集群灭绝**：`20261212` 终局 0 live clades；它是真实样本，但任何「存活率」结论都应给逐 seed 分布。
5. **R3 只做 2-seed 描述**：不构成对 Phase 67 判定的重新裁定；若未来要做多样性目标，需单独预注册 5-seed 与持续期主指标。
6. **默认 5-seed 基线**：本期 `36 分化 / 22 灭绝`（`20261208–12`）与 Phase 67 的 `31/21`（`20261201–05`）不是同一 seed 池，不能直接比较。
7. **wasm 增量 323 B**：漏斗字段与 `kind_id` 没有破坏帧预算；下期若加 `would_region_block` 与首次拒绝计数，预计再增几十到几百 B，仍需三次 <16 ms。
