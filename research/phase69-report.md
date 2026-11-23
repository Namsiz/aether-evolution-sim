# Phase 69 — 建支窗口保护：`would_region_block` 真实份额 + 临时女儿区域份额保护（预注册）

日期：2026-11-23
HEAD（起点）：`aa39aff`（Phase 69.1：`would_region_block` 只读探针 + 2-seed 探针数据；Phase 68 为 `f78004f`）
性质：**先补纯遥测（Phase 69.1），再预注册单机制 A/B（Phase 69.2）**。Phase 69.1 已实现 `would_region_blocks` 只读探针；本文件 §1 在任何 Phase 69.2 的 200k 判定臂启动之前单独提交。

> Phase 68 的 §4.2 把区域容量份额写成 6.7 %，那是**下界**：`spawn_tagged` 先查 kind 预算，kind 失败时区域闸根本不被求值。Phase 69.1 的探针在 kind 失败时补一次只读区域判定，2 个 seed 的真实区域份额是 **31.2 %**。本文件以修正后的口径预注册 Phase 69.2。

---

## 1. 预注册（跑之前写死）

### 1.1 Phase 68 的遗留与 Phase 69.1 的修正

Phase 68 的 5-seed 200k 结论：

- 派生 DECO 的 spawn 尝试中，`kind_block = 93.2 %`、`region_block = 6.7 %`（下界）。
- `find = 0.535`、`compat = 0.829`；成配不是主瓶颈。
- `deaths/birth = 1.004`；死亡侧 `aged = 77.4 %`、`starved = 22.6 %`。
- R3 把 `find` 提高 7.2 pp，但 births/kt 只 +5.8 %；容量闸吸收额外成配。

Phase 69.1 在 `20261208 / 20261209` 两个默认 seed 上跑 200k 只读探针（轨迹与 Phase 68 的 7/5、5/1 逐事件一致）：

| 作用域 | seed | attempts | kind_blocks | region_blocks | would_region | **true region** | **kind-only** |
|---|---|---|---|---|---|---|---|
| 派生 DECO | 20261208 | 1 985 876 | 1 780 163 (89.6 %) | 203 705 (10.3 %) | 502 692 | **706 397 (35.6 %)** | **1 277 471 (64.3 %)** |
| 派生 DECO | 20261209 | 1 883 084 | 1 724 738 (91.6 %) | 156 713 (8.3 %) | 345 546 | **502 259 (26.7 %)** | **1 379 192 (73.2 %)** |
| **派生合计** | — | **3 868 960** | **3 504 901 (90.6 %)** | **360 418 (9.3 %)** | **848 238** | **1 208 656 (31.2 %)** | **2 656 663 (68.7 %)** |
| 根 DECO | 20261208 | 1 295 765 | 822 176 (63.4 %) | 472 351 (36.4 %) | 508 406 | **980 757 (75.7 %)** | **313 770 (24.2 %)** |
| 根 DECO | 20261209 | 3 500 645 | 3 443 747 (98.4 %) | 54 947 (1.6 %) | 95 190 | **150 137 (4.3 %)** | **3 348 557 (95.7 %)** |

**读法**：

- 派生 DECO 的真实区域闸是 **31.2 %**，不是 6.7 %；kind-only 仍有 **68.7 %**。区域闸是真实的二级约束，值得单独测一次。
- 根 DECO 的闸门组成跨 seed 极不稳定（08 上 region 75.7 %，09 上 kind-only 95.7 %）；任何「根族也如何」的结论都必须给逐 seed 分布。
- `would_region` 是 `kind_blocks` 的**子集**，不能加到总拒绝数上；总拒绝仍是 `kind_blocks + region_blocks`。

### 1.2 Phase 69.1 的插桩（已实现，纯遥测）

- `CladeLifeStats.would_region_blocks`：kind 预算失败时，若 Phase 60 区域闸**也会**拒绝该 spawn，则 +1。
- 探针是只读的：权重缓存过期时计算**局部**权重数组，不写 `deco_region_weight` / `deco_region_weight_sum` / `deco_region_weight_tick`；无 RNG；不入档。
- 单测：`kind_budget_fail_probes_region_cap_without_touching_the_cache`、`the_probe_is_false_when_the_region_cap_is_off`。
- 默认 `@60000` 锚点 `0x3272f0e3eaba9250` 位精确（339.65 s）；两个 200k 探针复现 Phase 68 的 7/5、5/1。
- CSV 末尾加 `would_region_blocks`；`[funnel]` 行加 `would_region_blocks=`。

### 1.3 Phase 69.2 机制：临时女儿区域份额保护（T-region）

**目标**：只保护派生 DECO 的**建支窗口**，不永久改变根族的象限分割。

| 项 | 值 |
|---|---|
| 作用对象 | `root_kind == DECO ∧ kind_def(kind).parent_kind.is_some()`；根 DECO、FISH 不进入 |
| 窗口 | fork 后 `PROTECT_TICKS = 10_000`（沿用 `YOUNG_CLADE_TICKS`，与 Phase 62 #15 的 3 960 tick 耗散窗口同量级） |
| 保护规则 | 已占据象限：`cap = max(natural_cap, min(budget, SPECIATION_MIN_SIDE))`；空象限保持自然份额 |
| 常量 | `SPECIATION_MIN_SIDE = 8`（与分裂门一致，不调参） |
| 到期 | `tick − clade.birth_tick ≥ 10_000` 后回到自然 cap；无永久 floor |
| 守恒 | 区域 cap 是空间份额，不是全局预算；`Σ kind_budget` 仍恒等于 DECO 全族 80，不新增总额 |
| RNG | 零 RNG；只改 cap 的返回值，不改搜索/出生顺序 |
| 存档 | 开关与窗口不入档；读档回编译默认 OFF |
| D0 | `protection_admitted > 0`（因保护而通过的出生）；T 臂的 `would_region_blocks` 比 OFF 下降 ≥20 % |

**与 Phase 61 b 臂的区别**：b 是**永久** floor，会把女儿的 region 闸整体失效、把矛盾全部推到 kind 预算（Phase 62 §3.3）；T-region 只在建支窗口生效，窗口结束后女儿必须按自然份额生存。它回答的是「建支期的区域挤压是否是可救的」，不是「永久给女儿保留份额」。

**与 Phase 65 W1 的区别**：W1 是**出生闸宽限**（跳过 kind 预算），T-region 只动**区域 cap**，不动 kind 预算；两者作用在不同闸门。

**已知风险（写死）**：

- 区域闸只占真实拒绝的 31.2 %；即使 T-region 完全消除区域拒绝，kind-only 的 68.7 % 仍在，`births/kt` 可能到不了 1.10×。这是本期的主要失败模式，失败后 Phase 69.3 应转向 kind-only。
- 窗口内多个象限同时触发 floor 会让区域份额之和超过 budget；这不新增个体（kind 预算仍是硬顶），但会把空间分布推向「单象限 deme」。报告必须给 `DECO@200k` 的四象限分布。
- `clade.birth_tick` 是 fork tick；如果某 clade 的派生 kind 在 fork 后很久才首次出生，窗口从 fork 起算（不是首次出生），与 Phase 66/67 的 young 口径一致。

### 1.4 seed 池（机械派生，全新池）

`20261215–19` 在仓库内命中 0；`20261220–21` 为备用。Phase 68 的 `20261208–12`、Phase 67 的 `20261201–05` 不复用。

| 位 | seed | 用途 |
|---|---|---|
| 1–5 | 20261215 / 16 / 17 / 18 / 19 | 200k 判定 |
| 备 | 20261220 / 21 | 退化替换，按序最多 2 个 |

退化：`spec + ext = 0`，或 DECO 根 clade 在 `t < 10 000` 灭绝。禁止看到结果后追加/替换 seed。

### 1.5 臂与运行

| 臂 | 环境变量 | 语义 |
|---|---|---|
| OFF | `PHASE_BUILD_WINDOW=off` | Phase 68 默认世界 |
| T | `PHASE_BUILD_WINDOW=treg` | §1.3，10k 窗口 + `min(budget, 8)` 区域 floor |

- 5 seed × 2 臂 × 200k = 10 个判定 run；4 进程/批。
- 默认自检：`20261102 / 200k` = 9/4。
- 双跑：`20261215 / T` 两次，`clade_life` / `clade_fate` / `clade_tree` CSV 逐字节相同。
- wasm：重建；三次同会话交错 A/B，60 帧 <16 ms。

### 1.6 判据（写死）

| 判据 | 门槛 | 说明 |
|---|---|---|
| **P1 分化** | `Σ_T(spec) ≥ 0.90 × Σ_OFF(spec)` | 非劣性；先于 P2 |
| **P2 出生率** | `Σ_T(births) / Σ_T(derived_clade_ticks) ≥ 1.10 ×` OFF 同口径 | 本期主目标 |
| **P3 死亡/出生** | `Σ_T(deaths) / Σ_T(births) ≤ 1.05 ×` OFF | 非劣性；防止「多生多死」 |
| **D0 作用力** | `protection_admitted > 0` **且** `Σ_T(would_region_blocks) ≤ 0.80 × Σ_OFF(would_region_blocks)` | 防「机制没咬到」 |
| **D1 空间** | 报告 `DECO@200k` 四象限分布；不判定 | 检验是否退化为单象限 |
| **D2 确定性** | 默认 @60000 锚点不变；T 双跑 bit-exact | 必要条件 |

P1、P2、P3 必须同时满足；D0 是必要条件。P2 用**出生率**而不是灭绝数，避免 Phase 61-67 的「灭绝相对比例」拟合陷阱。

### 1.7 决策表（写死）

| # | 结果 | 决策 |
|---|---|---|
| 1 | D0 不成立 | 默认 OFF，不 re-baseline；结论「保护没咬到，本池无信息」 |
| 2 | D0 ∧ P1 ∧ P2 ∧ P3 | 默认 ON；**第二十次 re-baseline**；保留 `PHASE_BUILD_WINDOW=off` 回放臂 |
| 3 | P1 败 | 默认 OFF；分化代价越界 |
| 4 | P1 过、P2 败 | 默认 OFF；结论「区域闸不是出生率杠杆」，Phase 69.3 转向 kind-only |
| 5 | P1/P2 过、P3 败 | 默认 OFF；结论「多生多死，净存活无改善」 |
| 6 | 可测 seed < 5 | inconclusive；默认 OFF，不据此否定机制 |

### 1.8 明确不做

- 不做择偶半径（R1/R2/R3 已两期否证）。
- 不做永久区域 floor（Phase 61 b / 62 d 已否证）。
- 不做 `FORK_BUDGET_MIN` 微调（Phase 63/64 已否证）。
- 不做 W1 出生宽限（Phase 65 已否证）。
- 不改 DECO 全族 80 总额；不新增全局预算。
- 不在本期修 kind-only 的 68.7 %；T-region 失败后再单独预注册 Phase 69.3。
- 不做观察层/渲染功能；只做 headless 遥测。

### 1.9 执行协议

1. 先提交本预注册（§1）。
2. 实现 §1.3 的 T-region 臂与 `protection_admitted` 只读计数；补单测（窗口边界、到期回落、空象限、根/FISH 排除、守恒）。
3. 默认 @60000 锚点 + 全 workspace 测试 0 失败。
4. 编译一次 release 测试二进制；10 个判定 run 复用，只改环境变量。
5. 跑 5 seed × 2 臂 + 双跑；记录逐 seed 与聚合。
6. 重建 wasm；三次 <16 ms。
7. 写报告 §2 之后：逐 seed、D0-D2、锚点、测试、wasm、决策。
