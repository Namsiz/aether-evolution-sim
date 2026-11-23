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
