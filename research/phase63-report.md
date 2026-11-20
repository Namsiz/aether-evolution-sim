# Phase 63 — kind 预算营救新生儿（预注册 5-seed A/B）

日期：2026-11-20
HEAD（起点）：`4e0b823`（Phase 62：b 臂 5-seed 复测不通过，默认不变）
性质：**预注册 + 机制臂**。Phase 62 §3.3/§4 的定因是本期唯一出发点：真正约束女儿支的是 Phase 49 的 **kind 预算**（`FORK_BUDGET_MIN = 8`、`SPECIATION_MIN_SIDE = 8`、DECO 全族固定 80、grant = clamp(parent/4, 8, 20)），不是区域份额（a/b/c/d 四臂已在 Phase 61/62 全部否证）。本期直接动 kind 预算，三条候选都必须回答同一个问题：**新增预算从哪来**。三条的答案统一为「从父支系的既有份额里转移」——**不新增任何全局总额**，DECO 全族仍恒等于 80。

> 本文件的第一版只包含 §1 预注册，在第一个 Phase 63 实验进程启动之前单独提交。运行期间 §1 不做任何修改；后续各节只追加。

---

## 1. 预注册（跑之前写死）

### 1.1 三条候选的机制与「预算从哪来」

| 臂 | 规则 | 新增预算从哪来 | 开关 |
|---|---|---|---|
| **G** 年龄成长 | 派生 kind 每存活 5 000 tick 获得 1 个返还槽（tier 上限 8）；只在「女儿已填满当前预算」（`count(kind) ≥ kind_budget(kind)`）时支付；每次支付额 ≤ 父支系份额 − 8；累计 ≤ 8（女儿总预算 ≤ fork 时 grant + 8） | **父支系份额**（parent→child 转移，全族 80 不变） | `PHASE_BUDGET_GROWTH=on` |
| **M** 对称 floor | `FORK_BUDGET_MIN = X`（X = 10 / 12 / 16）：grant = clamp(parent/4, X, 20)，父支系剩余 ≥ X，`parent < 2X` 不分叉 | **父支系剩余**（总额固定，floor 越高，父方剩余越少、分叉链越浅） | `PHASE_FORK_MIN=10|12|16` |
| **A** 女儿准入 floor | grant = clamp(parent/4, 12, 20)，父支系剩余只需 ≥ 8（父 floor 不变） | **父支系剩余**（只抬高女儿侧 floor，不抬高父侧） | `PHASE_FORK_DAUGHTER_MIN=12` |

**G 的精确契约（写死，不可在跑后调参）**

1. 时钟：派生 kind 的存活时长取该 kind 对应 clade 的 `birth_tick`（已入档，无需新时钟）：`age = tick − birth_tick`。
2. `target = min(age / 5_000, 8)`；`earned = kind_growth_grants[kind]`；`d = target − earned`。5 000 而不是 10 000：Phase 62 §4 的灭绝生存区间集中在 21k–68k，10k 步长在危险期内只够 1–2 个槽；5k 让一个活过 20k 的女儿拿到 +4（8 → 12，恰好等于一个新物种的最小可行群体）。
3. 支付条件：该 kind 当前 `count(kind) ≥ kind_budget(kind)`（Phase 62 的灭绝支系满容占比 ≥ 99.6 %，这正是「成功」的可测形式）；父支系份额在支付后仍 ≥ 8（`FORK_BUDGET_MIN`）。
4. 支付额：`pay = min(d, parent_budget − 8)`；只把实际支付的槽数记入 `earned`，未支付部分保留到下一次检查（不丢 tier）。
5. 上限：`earned ≤ 8`。不设「parent 份额一半」的额外上限——父侧由第 3 条的地板保护，第 5 条已把单女儿的总量钉死。
6. 守恒：只做 `parent_budget −= pay; child_budget += pay`，`family_budget_total(DECO)` 恒为 80（单元测试断言）。
7. 灭绝归还：沿用 Phase 49——女儿支灭绝时把**含成长部分在内**的全部份额还给父支系。
8. 检查点：与 speciation 检测同一 60 tick 边界（`check_speciation` 开头，按 kind id 升序），零 RNG、零额外采样。

**M/A 的精确契约（写死）**

- 预检查：`parent_budget < fork_budget_min + fork_daughter_min` → 不分叉。
- `grant = clamp(parent_budget / 4, fork_daughter_min, FORK_BUDGET_GRANT=20)`。
- `parent_budget − grant < fork_budget_min` → 不分叉。
- 默认值 `fork_budget_min = fork_daughter_min = 8` 时与 Phase 49 的 `fork_budget_grant` **逐位等价**（单元测试对照）。
- M 臂把两个 floor 同时设为 X；A 臂只把 daughter floor 设为 12。两者可分辨：parent = 21 时 A12 允许分叉（grant 12 / 剩 9），M12 拒绝（21 < 24）。

**明确不做的事**：不碰 region cap 相关代码（Phase 61/62 已否证 a/b/c/d 四臂）；不做「fork 后 5k tick 内归还预算」的可逆分叉——撤回一次已登记的分叉事件需要回滚 clade id、成员归属与物种计数，且预算归零等价于宣判死刑，诚实的最小实现就是**在建支前拒绝注定失稳的分叉**（由 M/A 覆盖），该变体不进入本轮矩阵。

### 1.2 seed 池（沿用 Phase 62，写死）

| 位 | seed | 已知性质 |
|---|---|---|
| 1 | 20261102 | Phase 57–62 主 seed；OFF 9 分化 / 4 灭绝 |
| 2 | 42 | 10 / 8 |
| 3 | 20261109 | 3 / 1 |
| 4 | 20261105 | 2 / 3（Phase 62 未退化：DECO 根 clade 0 两臂存活到 200k） |
| 5 | 20261107 | 7 / 1 |
| 备用 | 20261113 | 非判定；Phase 62 已跑满两臂 |

**退化判定（写死，只用 DECO 根支生存遥测）**：某 seed 判为退化，当且仅当 DECO 根（`clade_id = 0`、`root_kind = DECO`）的 `extinction_tick < 10 000`，**且**该 seed 的 OFF 臂与候选臂都如此。

**替换规则（写死）**：第 4 位退化 → 备用 20261113 顶替；若备用也退化，或替换后可测 seed < 5 → 判 **inconclusive**：默认保持 OFF、不 re-baseline、不据此否定机制。

### 1.3 指标定义

| 指标 | 定义 | 来源 |
|---|---|---|
| 分化 | `event_kind = 0` 的事件数（DECO 分叉） | stderr `[split]` |
| 灭绝 | `event_kind ∈ {1, 2}`（全族；2 = 引擎判定的竞争排除） | stderr `[split]` |
| DECO@200k | t=200 000 的 `deco` 列与 `deco_r0..r3` | `clade_fate_*.csv` |
| clades live/total | t=200 000 | stderr `[fate]` |
| 灭绝定因 | `budget_blocks / (budget_blocks + births)`、`region_cap_blocks`、peak vs budget、死因四分、mate_misses | `clade_life_*.csv` |
| **建支即失稳（新档）** | `peak ≤ 12 ∧ region_cap_blocks = 0 ∧ kind_budget_blocks = 0 ∧ 生存 < 10 000` | `clade_life_*.csv`（Phase 62 §10.2） |
| 预算守恒 | `family_budget_total(DECO) = 80`（每个采样点） | 单元测试 + 观测器末态 |

### 1.4 第一段：80k 快筛（只用于选臂，不参与判定）

- 矩阵：3 seed（20261102 / 42 / 20261109）× 6 臂（OFF、G、M10、M12、M16、A12）× 80 000 tick = 18 个进程。
- **选臂规则（写死）**：
  1. 候选臂须过 P1-screen：`Σ spec(cand@80k) ≥ 0.90 × Σ spec(OFF@80k)`。
  2. 在过线者中取 `Σ ext` 最低者；并列取 `Σ spec` 更高者；再并列按固定顺序 G → A12 → M10 → M12 → M16。
  3. 仅当胜出臂的 `Σ ext < Σ ext(OFF@80k)` 才进入 200k 确证；否则三条候选全部判「无信号、拒绝」，不跑确证。
- 快筛数字只作选臂依据；**不因快筛结果修改 §1.1 的任何参数**，不追加臂。

### 1.5 第二段：200k 确证（唯一判定）

- 矩阵：5 seed × 2 臂（OFF + 快筛胜出臂）× 200 000 tick。
- **P1 分化**：`Σ_cand(spec) ≥ 0.90 × Σ_off(spec)`。按 Phase 62 的 OFF 合计 31 计，门槛 27.9 → 需 ≥ 28。
- **P2 灭绝（绝对区间）**：`Σ_cand(ext) ≤ 12`（Phase 62 OFF 为 17）**且**逐 seed 符号反转 ≤ 1 个（反转 = `ext_cand(seed) > ext_off(seed)`）。Phase 62 §10.4 的教训：不再用相对比例，不用「减半」这类从 3-seed 拟合出来的门槛。
- **决策优先级（写死）**：
  1. P1 且 P2 → 候选默认 ON（`FORK_BUDGET_GROWTH_ON = true` 或 floor 常量改为该值）+ **第十八次 re-baseline**（全锚点更新）+ 双跑/存档往返/新锚点测试。
  2. P1 过、P2 未过 → 默认保持 OFF，臂保留在代码，不 re-baseline；报告写清差多少、哪个 seed 反向。
  3. P1 败 → 拒绝该候选作为默认（默认 OFF，不 re-baseline）。
  4. 可测 seed < 5 → inconclusive：默认 OFF，不 re-baseline，不据此否定机制。

### 1.6 执行协议

- release 二进制；每臂独立进程；`PHASE_ARM_TAG` 区分产物；并发 6（8 核）；其余开关全部编译默认（Phase 60 机制栈）。
- **确定性自检**：200k OFF 臂必须逐事件复现 Phase 62 的 3 seed 计数（102：9/4；42：10/8；109：3/1）。任何偏差先查二进制与开关状态，不得直接算结论。
- 快筛 80k、确证 200k 使用同一二进制；快筛与确证之间不重新编译（只改环境变量）。
- 产物落 `/tmp/p63/*.log` 与 `/tmp/clade_{fate,tree,culture,life}_*_p63_*.csv`；报告成文后删除全部临时文件。
- 报告须含：预注册内容、三条候选的「预算从哪来」推导、快筛表、确证表（逐 seed）、定因表、决策、锚点/确定性/wasm、改动文件（路径:行号）、Phase 64 遗留。
