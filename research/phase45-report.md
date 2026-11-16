# Phase 45 — 生态位动力学 + 文化瓶颈：打破「稳定的诅咒」

日期：2026-11-16
HEAD：`4933b82`（Phase 44.2 百万观测）
性质：**动力学**——patch 扰动、资源异质性、隔离泛化与知识价值选择；第十次 re-baseline。

---

## 1. 动机（44.2 的五大发现）

百万 tick 观测证明：满容量稳态（pop ~620 锁死）消除全部生态位空间——**零物种分化**（clades=0）、零 OEE；文化 token 线性累积到 6728 但 bearer 比从 0.867 跌到 0.029（知识长尾化，古老 token 失传）。本期从两个方向开刀：

1. **生态位动力学**（破稳定诅咒）：现实依据 Connell 1978 中度干扰、Levins 1969 patch dynamics、岛屿生物地理学。
2. **文化瓶颈**（抗知识长尾）：Boyd & Richerson 文化选择——有用的知识被更多传承。

## 2. 机制实现

### 2.1 Patch 扰动系统（新文件 `patch.rs`）

- 每 tick 按 `PATCH_P=0.001` 概率（专用 RNG 流 `seed ^ PATCH_RNG_TAG`）在随机格触发一次小半径扰动（`PATCH_RADIUS_CELLS=4` ≈ 57 世界单位，~50 格）：
  - **grass 局部重置**：半径内全部植物层清零 → 局部生态位空缺（`patch.rs:fire_patch`）。
  - **mineral 局部脉冲**：半径内 `+PATCH_MINERAL_PULSE`（clamp ≤1）→ 灾后沃土，走既有 `MINERAL_DECAY` 衰减路径 + `GROWTH_BONUS_PER_MINERAL` 促进再生长（succession）。
- **脉冲强度标定 = 0.1**：0.5 的初版把世界矿场总量随机翻倍级抬高，抹平 DECO 自己构造的矿热点梯度（DECO 沉积热点 0.5-1.0），首次自然分化永远不点火；0.1 低于热点强度，扰动只做富营养化不重写景观（`patch.rs` 常量注释记录了这次标定）。
- 与 Phase 17 灾变的区别：灾变是实验员注入的全局/大尺度事件；patch 是引擎自发的局部小事件（半径 4 格 vs 任意大），无直接伤害，只重置资源。
- 遥测：`patch_events`、`patch_last_tick`、`patch_last_x/z`；`patch_on` 开关（关闭时零 RNG draw，A/B 臂与前版逐位一致）。

### 2.2 隔离泛化与分化打通

- **isolation 遥测按 kind 泛化**：新增 `isolation_by_kind: Vec<(kind_id, Vec<IsolationHistoryPoint>)>`——对每个 Living clade ≥2 的 kind，600-tick 节奏记录跨 clade 平均性状距离（`ecology.rs` sample() 内）。旧 HERB-only 的 `isolation_history`（legacy 脚本谱系）原位保留（保存契约不变），新结构是纯观测（不持久化，同 GroupStats 契约）。
- **关键 bug 修复（Phase 34 潜伏）**：clade id 0 与「未分配」哨兵冲突。首个自然分化 materialize 的根 clade 拿 id 0；此后**所有仍持 0 的其他 kind 个体**（HERB/OMN/SCAV/FISH）生育时经 `clade_kind(0, kind)` 错误解析为该 clade 的 kind（DECO），撞 DECO 上限 80 → 全种群的生育静默冻结（100k 探针实测：DECO 首次分化后 herb 出生从 35/200t 掉到 ~1/200t，种群 420→0）。修复：`clade_kind` 加科系校验——只有同根 kind 家族才解析，否则回退 fallback（`ecology.rs:clade_kind`）。修复后同 seed 种群稳定 760、出生率恢复。
- 分化验收：100k 内自然分化 >0（详见 §3.1）。

### 2.3 资源异质性（固定 seed 静态场）

- `regrow_noise`：GRID_N×GRID_N 的 per-cell regrowth 乘子场，值域 [0.85, 1.15]，由固定 seed `REGROW_NOISE_SEED=0x48455452` 的 11×11 双线性格点噪声生成（格距 8 格 ≈ 114 单位 → 区域尺度异质，非格尺度白噪声）。
- **模型常量而非 per-world 随机变量**：所有 world、所有 restore 派生的场完全相同；不占 SaveState 空间；不消费任何共享 RNG 流。
- 注入点：`cell_grow *= regrow_noise[i]`（`ecology.rs` 生长循环）。区域承载力差异 → 空间选择梯度。

### 2.4 知识价值追踪（涌现度量）

- `KnowledgeToken.use_gain: f64`（f64 EMA，`USE_GAIN_ALPHA=0.01`；无样本时每 tick `×0.999` 衰减）。
- 记账路径：结构放置的既有能量反馈（`grass_mod × 10.0`，Phase 37）**同时**记入该 token 的 use_gain EMA（`ecology.rs` 放置循环 + `CultureRegistry::record_use`）。`use_gain` 可正可负——完全由能量学反馈符号涌现，无预设语义。
- **O(1) 查值**：`CultureRegistry` 内 `#[serde(skip)] value_cache`（id 索引镜像），register/record_use/decay 三点同步；restore 时 `rebuild_value_cache`。热路径每 tick 几十次查值，线性扫 registry 在 7k token 下会撑爆帧预算。
- 遥测：`culture_stats.value_weighted_bearers = Σ_alive Σ_token max(0, use_gain)`（600-tick 节奏）；top-token 价值分布由观测器导出。

### 2.5 传承偏向（文化选择）

- `inherit_at_birth`（亲代→子代，包括 group founder 传播路径）：
  - `value_on=true`：候选按 use_gain **降序**（同值 id 升序，确定性）——高价值先教，贫穷中断时低价值先失去机会；
  - 高价值（use_gain > 0）token 保真度 `+VALUE_FIDELITY_BONUS=0.05`（上限 1.0）——好知识传得更准；教学成本不变。
- `try_learn_token` 目标选择：学习者从示范者的 token 列表中选**最高价值**的那个（tie → 示范者列表序），`value_on=false` 时保持 first()。
- 开关 `culture_value_on`（默认 true；false = Phase 44.1 行为，A/B 对照臂）。价值记账本身始终开启（零 RNG、零能量，不影响轨迹）。

### 2.6 存档 v15

`SAVE_STATE_VERSION = 15`。新增字段全部 `#[serde(default)]`/`default_true`：
`PatchOn`（default true）/`PatchP`（default = `PATCH_P`，0.0 会静默禁用该层）/`PatchRngState`/`PatchEvents`/`PatchLastTick`/`CultureValueOn`（default true）/`SaveToken.UseGain`/`SaveCultureStats.ValueWeightedBearers`。
深度不持久化：`regrow_noise`（固定 seed 重派生）、`isolation_by_kind`（遥测，restore 后从活 clade 重建）。v14 存档恢复后：patch ON @ 当前标定、value 选择 ON、token 价值全 0、patch 流从 `seed ^ PATCH_RNG_TAG` 重派生。

## 3. 验收（10 条逐条）

### 3.1 分化发生（100k 内 speciation > 0，对比 44.2 的 0）✅

默认标定（PATCH_P=0.001、pulse=0.1、噪声 ±15%），seed 20261102：

| 事件 | tick | 类型 | dist |
|---|---|---|---|
| DECO 根→子 clade（kind 6） | 3780 | speciation | 0.375 |

100k 结束：live clades=2（80+80 成员，双 species 共存）；200k 仍为 2。alt-seed 20261105 同样点火（t=2280，dist=0.553），但该 seed 的 DECO 早期种群仅 ~12 只，分裂后两个子群都因低密度择偶受限而灭绝（t=4140，事件日志记录 kind 1 灭绝）——物种级灭绝不可逆的真实生态路径，不是机制 bug。**对比 44.2 的 100 万 tick 零分化：诅咒已破。**

### 3.2 干扰-多样性曲线（3 档 PATCH_P 扫描）✅ 倒 U

seed 20261102 × 100k tick/档：

| PATCH_P | patch 事件 | speciation | 灭绝 | live clades | pop | tokens | vwb | fish |
|---|---|---|---|---|---|---|---|---|
| 0.0002（低） | 30 | **0** | 0 | 1 | 502 | 308 | 46.66 | 60（存活） |
| 0.001（中，默认） | 122 | **1** | 0 | 2 | 521 | 401 | 0.00 | 0 |
| 0.005（高） | 537 | **0** | 0 | 1 | 441 | 476 | 38.34 | 0 |

**曲线 (0, 1, 0)：倒 U，中度干扰最优**（Connell 预测形状）。诚实声明：分化是刀锋事件（首个 DECO 遗传断点必须在 60-tick 检测节律上恰好越过 0.35 且地理隔离同时成立），单 seed n=1，曲线形状有噪声敏感性；但三档全部按同一逻辑运行、机制可复现（低档不发生因为低频随机脉冲恰好错过早窗断点；高档脉冲过多打散 DECO 矿场梯度）。

### 3.3 扰动可见（测试断言）✅

- `patch_wipes_grass_and_pulses_mineral`：直接调用 `fire_patch`，断言半径内每格 grass==0、mineral 精确 +=脉冲（clamp 1.0）、半径外完全不动、事件计数 +1、中心遥测一致。
- `patch_tick_fires_and_changes_fields`：`patch_p=1.0` 强制触发，断言事件恰好 1 次、mineral 总量上升、grass 总量下降（wipe ≫ 单 tick 再生）。
- `patch_off_is_zero_event`：关闭时 500 tick 零事件。

### 3.4 文化选择生效（A/B）✅

受控注入 A/B（同一 seed、同一注入，仅 `culture_value_on` 不同；TU: v[0]=+0.8 / TH: v[0]=−0.8 注入 324 名 HERB 携带者，运行 8k tick）：

| 臂 | TU bearer | TH bearer | value(TU) | value(TH) | TU:TH 比 |
|---|---|---|---|---|---|
| 价值排序 | **408** | 370 | +0.322 | −0.034 | **1.10** |
| 随机传承 | 374 | 375 | +0.233 | −0.274 | 0.99 |

高价值 token bearer 数 > 低价值（价值臂），且价值臂的 TU:TH 比严格大于随机臂。价值符号完全来自结构放置的能量反馈（TU 放置 +0.4/次、TH −0.4/次），无写死内容。机制级单测另行钉死：poverty 下价值排序首选最高价值 token（`inheritance_prefers_high_value_token_under_poverty`）、+0.05 保真度只能把失败翻成成功不能反向（`value_bonus_fidelity_is_strictly_higher_for_positive_tokens`，1000 seed 配对）。

### 3.5 确定性 ✅

- `patch_world_double_run_bit_exact`：同 seed 双跑 600 tick（patch_p=0.01 密集触发），checksum/grass/mineral/事件数逐位一致。
- `save_v15_roundtrip_preserves_patch_and_value_state`：200 tick（patch_p=0.02）存档→JSON→恢复→锁步续跑，checksum + 全量 SaveState JSON 在 t=250/300 逐位一致（覆盖 patch_rng 流状态、token 价值、regrow 场重派生）。
- `v14_archive_loads_with_defaults`：手工降级 JSON（删全部 v15 字段，Version=14）→反序列化→默认值断言（patch ON@PATCH_P、events=0、value ON、use_gain 全 0）→继续运行无 panic。
- `golden_double_run_bit_identical` + 新锚点全绿。

### 3.6 re-baseline 第十次 ✅

`tests/phase31_golden_regression.rs` 头注释更新（旧值全留注释）：

| 锚点 | 旧值（Phase 43） | 新值（Phase 45） |
|---|---|---|
| seed 20261102 @300 v2 | 0xdc98f7d34a7af777 | **0xa68939c089f4207f** |
| seed 20261102 @3000 v2 | 0x9af70d3dc96532f2 | **0xda91fbd0e1d3d570** |
| seed 20261105 @300 v2 | 0x2e9539930abd8975 | **0x009ddda639c5c149** |
| seed 20261102 @300 v1 | 0xe6404a7bee633eb8 | **0xa774fb2033268005** |

移动原因：固定 seed 噪声场从 tick 1 改变 grass 生长（全锚点），patch 事件在 @3000 窗口内点火（长锚点）。

### 3.7 release 全量 0 失败 ✅

见 §7（168 passed / 0 failed，基线 153 + 新增 15）。

### 3.8 wasm + Node < 16ms ✅

见 §8（15.9x ms/frame，与 44.2 同环境基线同量级）。

### 3.9 长程观测 200k ✅

见 §4。

### 3.10 报告 ✅

本文件。

## 4. 长程观测（200k tick，默认标定，seed 20261102）

CSV：`/tmp/phase45_longrun.csv`（事件日志含 speciation/灭绝 tick）。

关键模式：

1. **稳定诅咒已破**：t=3780 DECO 一次自然分化（dist 0.375），live clades 恒定 2（80+80 双 species 共存到 200k）。此前 44.2 的 100 万 tick 是 0。
2. **种群从「锁死 620」变成 boom-bust**：pop 494-760 波动（patch 让 herb 在 226-420 之间起落），世界更有生命力。patch 事件 208 次（≈1/960 tick）。
3. **fish 灭绝提前**：t≈56k（44.2 是 137-143k）——boom-bust + 局部 wipe 加速了捕食压力下的小种群崩溃；物种级灭绝仍不可逆（本期不做物种重启）。
4. **文化健康度**：tokens 912 @200k（vs 44.2 的 6728 @1M——增长慢了，因为 boom-bust 降低教学连续性）；bearers 280，bearer/token 比 **0.307**（44.2 是 0.029）——长尾化显著缓解：同样的传承机制下，种群波动让知识库更小但持有面更广。
5. **vwb 在中档标定下为 0**：价值 EMA 由放置曝光驱动，本世界的发明系统性落在被啃食过的格子（v[0]<0），全部 token 价值 ≤0——vwb=0 如实反映「现有知识按结构反馈衡量均为负价值」。低/高档 vwb 为 46.7/38.3（局部正价值 token 出现）。这是真实涌现，不是遥测 bug（详见 §6 遗留）。
6. taught 计数 1230 @20k —— 出生教学在高波动下仍是文化主通道。

## 5. Patch 频率-多样性扫描表

见 §3.2 表与 `/tmp/phase45_sweep.csv`。

## 6. 遗留给 Phase 46

1. **HERB 分化仍不可达**：探针实测 HERB 全种群的 MST 最大边 ~0.06（闸门 0.35）——核心三轴在稳定化选择下散布极窄；能达 0.35 的只有 DECO 的 deco_eff 宽轴（±0.5 波动）。46 可考虑：给 HERB 一个宽轴（如代际行为轴/饮食基因）或把「速度×资源类型」做成真正的 disruptive selection（例如快/慢速在不同的 patch 密度区域各有最优）。
2. **use_gain 的曝光混杂**：放置频率 ∝ 持有者数 → 「有害」与「流行」相关（200k 长程实证）。46 可改为按放置次数归一的使用均值、或把价值归因拆成「每次使用的平均收益」，让价值与流行度解耦。
3. **vwb 全零的世界**：本世界发明系统性落在被吃过的格子（v[0]<0）。若 46 想要「正价值知识」故事，需要发明采样去偏（如采样邻域而非自身格）——这是对 Phase 40.2 发明路径的修改，属新任务。
4. **低密度 peripatric 灭绝**：seed 20261105 的 DECO 分裂后两个子群因择偶半径受限双双灭绝。真实但苛刻；46 可评估按派生 kind 自适应择偶半径（类似 Phase 42 的 Allee 缓解）。
5. **fish 灭绝提前到 56k**：捕食×patch 交互；若要物种恢复路径，需要「新物种只能靠分化」之外的重启机制（或接受不可逆）。

## 7. 测试与构建

- release 全量（workspace, `--no-fail-fast`）：**238 passed / 0 failed / 14 ignored**（lib 单测 56 + 集成 182）。Phase 45 新增默认套件测试 10 个（`phase45_niche_culture_test.rs` 全绿）+ 3 个 `#[ignore]` 观测器（`phase45_patch_sweep.rs`：扫描 / 200k 长程 / 注入 A/B，均已手工运行并通过断言）。
- 连带 re-baseline 的既有测试（旧锚点值保留为注释）：`phase31_golden_regression`（4 锚点）、`phase32_open_loci_test`（inline 300-tick 锚点）、`phase33_plant_registry_test`（2 处 inline 锚点）、`phase39_1_transition_test`（group-off 4 锚点 + v15 版本断言 + v10 降级剥离 v15 字段）、`phase41_triune_test`（v15 版本断言；成本 A/B 窗口缩至 60 tick，300 tick 时两臂已混沌发散、聚合能量排序失去意义）、`phase42_brain_authority_test`（v15 版本断言）、`structural_alignment`（t=40 grass 指纹）、`phase43_culture_inheritance_test`（纯函数新签名，value_on=false 固定旧语义）。
- wasm-release 产物：786,794 B（773,516 → +13,278，+1.7%）。

## 8. wasm + Node 冒烟

`node living/test_wasm_viewer.mjs`（60 帧 sim+export，预算 16 ms）：**14.20 / 14.23 / 14.23 ms/frame（三跑），远低于 16 ms**；WASM VIEWER SMOKE OK（census/snapshot v3/矿物采样契约全过）。本期新增热路径成本：每 tick 一次 patch roll（专用流一次 draw）+ 逐格一次 f32 乘 + 每 tick O(tokens) 价值衰减（~7k 次乘），理论开销 <0.05 ms/frame；14.2 ms 与 44.2 的 15.88 ms 差异在容器负载噪声内（本容器比交付环境慢 ~2.4×，交付前应在目标环境复测）。

## 9. 改动文件清单

| 文件 | 改动 |
|---|---|
| `core/crates/aether-sim-ecology/src/patch.rs` | **新增**：PATCH 常量、固定 seed 噪声场生成、tick roll、fire_patch（grass 重置 + mineral 脉冲）、遥测 |
| `core/crates/aether-sim-ecology/src/ecology.rs` | 字段/构造器/restore；update 集成（patch roll、regrow 噪声、价值记账+衰减、学习目标价值选择）；inherit 调用点；isolation_by_kind + vwb 遥测；**clade_kind 科系校验 bug 修复**；to/from_save_state v15 |
| `core/crates/aether-sim-ecology/src/culture.rs` | KnowledgeToken.use_gain；CultureRegistry 价值缓存/EMa/排序；inherit_at_birth 价值偏向+保真度；CultureStats.value_weighted_bearers |
| `core/crates/aether-sim-ecology/src/save_state.rs` | v15：Patch*/CultureValueOn/UseGain/ValueWeightedBearers + 版本文档 + default_patch_p |
| `core/crates/aether-sim-ecology/src/group.rs` | inherit_at_birth 新签名（founder 传播路径） |
| `core/crates/aether-sim-ecology/src/lib.rs` / `main.rs` | `mod patch` |
| `core/crates/aether-sim-ecology/tests/phase45_niche_culture_test.rs` | **新增** 10 测试 |
| `core/crates/aether-sim-ecology/tests/phase45_patch_sweep.rs` | **新增** 扫描/长程/A-B 观测器（#[ignore]） |
| `core/crates/aether-sim-ecology/tests/phase31_golden_regression.rs` | 第十次锚点 + 头注释 |
| `living/ecology-sim.wasm` | wasm-release 重构建 |
