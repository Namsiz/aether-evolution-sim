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

---

## 2. 实现（Phase 69.2，已提交 `4a38acd`）

### 2.1 常量与开关

| 位置 | 内容 |
|---|---|
| `ecology.rs:1130` | `BUILD_WINDOW_ON = false`、`BUILD_WINDOW_TICKS = 10_000`、`BUILD_WINDOW_FLOOR = SPECIATION_MIN_SIDE_DEFAULT`（=8） |
| `ecology.rs:2288` | `Ecology.build_window: bool`；三个构造器（`new` / `with_biomes` / `from_save_state`）全部初始化 `BUILD_WINDOW_ON`，读档回编译默认 OFF |
| `ecology.rs:4518` | `build_window_treg_on/off`、`build_window_treg()` |
| `ecology.rs:4576` | `deco_region_cap_limit_with` = `deco_region_natural_cap` + T-region 分支 |
| `ecology.rs:5252` | `spawn_tagged` 区域闸：先 natural、再 protected，判拒；`protection_admitted` 只在 natural 拒绝且 protected 放行时置位 |
| `ecology.rs:5362` | 成功出生后按 `(clade_id, kind_id)` 记 `protection_admitted` |
| `ecology.rs:1727` | `CladeLifeStats.protection_admitted: u64` |
| `ecology.rs:11475` | `phase69_build_window_tests`（6 个测试） |

### 2.2 语义

- **作用对象**：派生 DECO（`root_kind == DECO ∧ parent_kind.is_some()`）；根 DECO、FISH 永不进入。
- **窗口起算**：`SpeciesClade.birth_tick`（fork tick），不是首次出生；`tick − birth_tick < 10_000` 有效，第 10 000 tick 到期回落。
- **区域 cap**：已占据象限 `max(natural, min(kind_budget, 8))`；空象限与到期后保持自然份额。
- **守恒**：区域 cap 只是空间份额；kind 预算仍是硬顶，`Σ kind_budget` 恒 80，不新增个体。
- **零 RNG / 不入档 / 不改搜索或出生顺序**；Phase 69.1 的 `would_region_blocks` 在 T 臂看到的是受保护 cap，因此 D0 可直接比较。
- **歧义处理**：仓库只有 `SPECIATION_MIN_SIDE_DEFAULT=8` 与每实例 `speciation_min_side`；按预注册「与分裂门一致、不调参」，floor 固定取 8，不随 legacy 2 漂移。

### 2.3 观测器

- `clade_fate_observatory.rs`：解析 `PHASE_BUILD_WINDOW=off|treg`，臂名 `bwinoff/bwinon` 进入输出文件名；`clade_life` CSV 末尾追加 `protection_admitted`；`[funnel]` 派生/根聚合加 `protection_admitted=`。

---

## 3. 执行

| 项 | 内容 |
|---|---|
| 实现提交 | `4a38acd`（默认 OFF） |
| 自检 | `20261102 / 200k` = 9 分化 / 4 灭绝 |
| 判定 | `20261215–19` × 200k × 2 臂（OFF / T） |
| 双跑 | `20261215 / T` 两次（`p69t15` / `p69tdbl15`） |
| 退化 | `20261216` 两臂均 `spec+ext=0`（无派生 DECO）；按 §1.4 用备用 `20261220` × OFF/T 替换 |
| 最终可测 | `20261215 / 17 / 18 / 19 / 20`（5 个） |
| 并发 | 4 进程/批；单臂 649–1 402 s |
| 运行脚本 | `/tmp/p69/run_ab.sh` + `/tmp/p69/run_backup.sh`；日志/CSV 在 `/tmp/p69/runs/` |

**小偏差**：OFF 臂实际传空环境变量（编译默认 `bwindflt`），预注册写的是 `PHASE_BUILD_WINDOW=off`；两者语义都是 OFF，不影响判定。

---

## 4. 结果

### 4.1 逐 seed（5 个可测 seed，派生 DECO 聚合）

| seed | 臂 | spec | ext | live/total | births/kt | deaths/kt | kind_block | region_block | would/kt | protect | deaths/birth |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 20261215 | OFF | 10 | 5 | 6 / 11 | 2.10 | 2.10 | 0.863 | 0.136 | 401.8 | 0 | 1.002 |
| 20261215 | T | 10 | 7 | 4 / 11 | 2.01 | 2.05 | 0.872 | 0.127 | 561.9 | 128 | 1.019 |
| 20261217 | OFF | 2 | 2 | 1 / 3 | 1.23 | 1.25 | 0.694 | 0.303 | 130.0 | 0 | 1.020 |
| 20261217 | T | 7 | 3 | 5 / 8 | 2.00 | 1.99 | 0.958 | 0.041 | 353.9 | 81 | 0.997 |
| 20261218 | OFF | 1 | 0 | 2 / 2 | 5.03 | 4.86 | 0.975 | 0.023 | 331.7 | 0 | 0.966 |
| 20261218 | T | 3 | 0 | 4 / 4 | 2.41 | 2.36 | 0.983 | 0.015 | 258.8 | 7 | 0.979 |
| 20261219 | OFF | 9 | 6 | 4 / 10 | 2.07 | 2.09 | 0.889 | 0.109 | 449.0 | 0 | 1.014 |
| 20261219 | T | 13 | 12 | 2 / 14 | 1.61 | 1.71 | 0.956 | 0.043 | 190.9 | 127 | 1.062 |
| 20261220 | OFF | 2 | 2 | 1 / 3 | 1.47 | 1.50 | 0.840 | 0.158 | 52.4 | 0 | 1.022 |
| 20261220 | T | 10 | 6 | 5 / 11 | 2.48 | 2.51 | 0.953 | 0.046 | 241.5 | 89 | 1.009 |

`protection_admitted` 原始计数：15=128、17=81、18=7、19=127、20=89，合计 **432**。

### 4.2 聚合与判据（5 个可测 seed）

| 指标 | OFF | T | 比值（T/OFF） |
|---|---|---|---|
| 分化 `Σ spec` | 24 | 43 | **1.792** |
| 灭绝 `Σ ext` | 15 | 28 | 1.867 |
| 派生 births/kt | 2.021 | 2.049 | **1.014** |
| 派生 deaths/birth | 1.0058 | 1.0160 | **1.010** |
| `kind_block` 份额 | 0.874 | 0.939 | 1.074 |
| `region_block` 份额 | 0.125 | 0.060 | 0.481 |
| `would_region` / ktick | 357.6 | 331.1 | 0.926 |
| `would_region` 原始总量 | 940 540 | 1 339 671 | **1.424** |
| `protection_admitted` | 0 | **432** | — |

| 判据 | 门槛 | 结果 | 判定 |
|---|---|---|---|
| P1 分化 | `Σ_T spec ≥ 0.90 × Σ_OFF` | 43 ≥ 21.6（1.792×） | **过** |
| P2 出生率 | `births/kt_T ≥ 1.10 × OFF` | 1.014× | **败** |
| P3 死亡/出生 | `deaths/birth_T ≤ 1.05 × OFF` | 1.010× | **过** |
| D0 作用力 | `protection_admitted > 0` 且 `Σ_T would ≤ 0.80 × Σ_OFF` | 432 > 0；原始 **1.424×**，归一化 **0.926×** | **败** |

**D0 的口径陷阱**：预注册 §1.6 写的是原始总量之比，实测 1.424×；按 derived clade-ticks 归一化后是 0.926×。两者都 >0.80，所以 D0 的结论不依赖口径。原始总量反向上升的原因是 T 臂 clade-ticks 更长、派生 clade 更多（spec 43 vs 24），`would_region` 被更多支系寿命放大；按 tick 密度它只降了 7.4%。

### 4.3 机制读数

- **T-region 把 region 闸换成了 kind 闸**：`region_block` 0.125→0.060，`kind_block` 0.874→0.939；合计拒绝仍约 99.9%。这与 Phase 61 b 臂 / Phase 62 d 变体的「floor 只是换闸门」完全一致。
- **保护确实咬到，但很小**：`protection_admitted = 432`，相对 5 个 T seed 的千万级 spawn 尝试是极小份额；P2 只 +1.4%，远低于 1.10 门。
- **P1 的 +79% 是周转放大器，不是存活改善**：T 的 spec 43、ext 28、live 20；OFF spec 24、ext 15、live 14。T 同时多生支系、多灭支系，`deaths/birth` 还略差（1.016 vs 1.006）。
- **空间集中（D1，报告不判定）**：T 臂出现明显单象限集中（seed15 r0 83%、seed19 r2 71%），但没有 100% 单象限退化；`DECO@200k` 分布：15 `40/2/1/5`、17 `11/0/1/14`、18 `20/19/1/5`、19 `2/3/45/13`。

### 4.4 自检 / 双跑 / 身份式

- 自检 `20261102` = **9 分化 / 4 灭绝**。
- 双跑 `p69t15` vs `p69tdbl15`：`clade_life` / `clade_fate` / `clade_tree` CSV **逐字节相同**。
- 14 个 run（12 个主跑 + 2 个备用） × derived/root 两个作用域，共 28 条 `[funnel] identity ... ok=true`，`ok=false` 为 0。
- `20261216` 两臂的 life/fate/tree CSV 逐字节相同（无派生 DECO，T-region 无作用对象），确认退化不是机制差异。

---

## 5. 决策

**默认 OFF，不 re-baseline（第二十次 re-baseline 未发生）。** 按 §1.7：

1. **P2 败**（1.014× < 1.10×）→ 第 4 行：区域闸不是出生率杠杆；
2. **D0 败**（原始 1.424×、归一化 0.926×，均 >0.80）→ 第 1 行：作用力门槛未过；
3. P1 过（1.792×）与 P3 过（1.010×）不能补偿 P2/D0；P1 是周转放大器的伴生现象。

**结论：T-region 不是出生率杠杆，而是「把 region 拒绝换成 kind 拒绝」的闸门搬运；建支窗口的区域份额方向关闭。**

### 5.1 Phase 69.3 方向

Phase 69.1 已经给出下一步的靶点：派生 DECO 的真实拒绝里 **kind-only 68.7%**、region 31.2%；T-region 又把 region 压到 6.0%、kind 抬到 93.9%。**Phase 69.3 应直接针对 kind-only**，候选：

1. **资源流导出的总承载容量**：让 DECO 全族预算随分解场/初级生产力变化，而不是固定 80；这是唯一还没被否证的「增加总额」方向，但必须保留根 kind 的 deme 分割。
2. **kind 预算的建支窗口**：fork 后前 N tick 允许 daughter 预算临时抬到 ≥16，从父系未用槽位借、到期归还；与 W1 的区别是「预算」而非「出生闸」，但 W1 的 69% 消散是强反例，优先级低于 1。
3. **目标重定义**：若继续只数 spec，T/ R3 都会「通过」；若以 live clades / 支系持续期为主，需要单独预注册。

**不再做**：region cap 的永久/临时 floor（61/62/69.2 三次否证）；择偶半径；`FORK_BUDGET_MIN` 微调；W1。

### 5.2 预注册口径教训

- D0 的「原始总量」门槛对 clade-ticks 敏感：T 臂支系更多、活得更久时，原始 `would_region` 会反向上升。下期主指标应固定为 **per-clade-tick**，原始总量只作报告项。
- `protection_admitted` 是「被保护放行的出生」而非「唯一被救母亲」；下期若做个体级漏斗，加「首次拒绝 / 窗口」计数。

---

## 6. 锚点 / 测试 / wasm

| 项 | 结果 |
|---|---|
| 默认 `@60000` | `0x3272f0e3eaba9250` 位精确（phase61 `renorm_off…`，333.00 s） |
| 默认 200k | seed 20261102 = 9 分化 / 4 灭绝 |
| 生态 crate 全测试 | **50 二进制 / 316 passed / 0 failed / 28 ignored** |
| Phase 69 单测 | **8 passed / 0 failed**（新增 6 + Phase 69.1 的 2） |
| 双跑 | `p69t15` vs `p69tdbl15`：life/fate/tree CSV 逐字节相同 |
| wasm | **831 807 B**，sha256 `901eab33502df49b6820fad7ba9510479a9aa6bc9a0f2c98bb3fbdfd19de719b`（Phase 68 旧 830 787 B；+1 020 B） |
| 帧预算 | 三轮新旧交错：old 10.59 / 11.64 / 12.35 ms；new 10.80 / 11.85 / 12.60 ms；全部 <16 ms |
| wasm 冒烟 | `WASM VIEWER SMOKE OK`（空闲态 10.62 ms/frame） |

---

## 7. 改动文件

- `core/crates/aether-sim-ecology/src/ecology.rs`：常量、`build_window` 字段/开关、`deco_region_natural_cap`、T-region cap 分支、`protection_admitted`、6 个单测。
- `core/crates/aether-sim-ecology/tests/clade_fate_observatory.rs`：`PHASE_BUILD_WINDOW` 解析、文件名臂标记、CSV/`[funnel]` 新列。
- `living/ecology-sim.wasm`、`living/ecology-sim-v2.wasm`：831 807 B / `901eab33…`。

---

## 8. 遗留

1. **SpeciesClade 缺失即无窗口**：按 `kind_id` 查 `species_clades` 取 `birth_tick`；未物化或 pre-v8 的 kind 不进入窗口；默认 OFF 不受影响。
2. **kind_id 唯一性依赖**：同 kind_id 多记录时 `find` 取第一条；现引擎每 kind 只 mint 一次，风险低。
3. **空间份额可超 budget**：窗口内多象限同时 floor 时区域 cap 之和可临时超过 kind 预算；不新增个体，但报告必须给 `DECO@200k` 四象限分布。
4. **T-region 已否证**：P2 1.014×、D0 原始 1.424× / 归一化 0.926×；Phase 69.3 直接针对 kind-only（OFF 68.7 %，T 臂 93.9 %），不要再动 region cap。
5. **D0 原始总量门槛被 clade-ticks 放大**：T 臂支系更多、活得更久时 `would_region` 原始总量会反向上升；下期主指标固定 per-clade-tick，原始总量只作报告项。
6. **seed 池教训**：`20261216` 两臂均 `spec+ext=0`，按预注册用 `20261220` 补位；下一期 seed 池应先用 80k 快筛剔除「无派生 DECO」的退化 seed，不要等到 200k 才发现。
7. **P1 的 +79 % 不是存活改善**：T 的 spec 43 / ext 28 / live 20，OFF 24 / 15 / 14；它是周转放大器，不能作为 Phase 69.3 的正面证据。
8. **尝试加权**：`would_region_blocks` / `protection_admitted` 仍是母亲-tick 计数；Phase 69.3 若要个体级漏斗，需加「首次拒绝/窗口」计数。
