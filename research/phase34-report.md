# Phase 34 — 分化引擎泛化：多轴 × 全物种 × kind 分叉

日期：2026-11-14
HEAD（前）：`ef495bb`（Phase 33 交付）
阶段性质：**本阶段是路线图里第一个显式 re-baseline 阶段**（Q1 裁决：34 起每阶段一次显式 re-baseline）。

---

## 1. Re-baseline 新旧锚点

`checksum()` 自 Phase 34 起返回 **v2** 值（kind 字段 = 注册表 id + 每 agent ext 座哈希）；
`checksum_v1()` 保留为历史锚点只读访问器（kind 字段 = kind 名字首字节，无 ext 哈希）。
三个黄金检查点（seed / tick）的新旧值：

| 检查点 | 旧 v1 锚点 | 新 v1（`checksum_v1()`） | **新 v2 主锚点（`checksum()`）** |
|---|---|---|---|
| seed 20261102 @300 | `0x790184996f3acefd` | `0x790184996f3acefd`（**未变**） | `0x785889bb1ec0737f` |
| seed 20261102 @3000 | `0x58733db4ab0c3ca0` | `0x6da134739d54ec96`（轨迹已重基线） | `0x6dbb8f6fd216f353` |
| seed 20261105 @300 | `0x48d66bf801a9f827` | `0xedf17a4e37b82ef6`（轨迹已重基线） | `0xf53762eefd0cf3d2` |

关键观察：

- **300 tick / seed 20261102 的 v1 轨迹逐位未变**（`0x790184996f3acefd`）——
  300 tick 内没有触发任何会改变主 RNG 流的机制（PRED 未跌破 5、HERB 脚本未触发、
  DECO 未分叉），因此 300 tick 的 v2 值与 v1 的差异**只来自校验和定义**（kind 字段 + ext 座哈希），
  而非动力学。该不变性已固化为测试 `golden_300_tick_trajectory_bit_exact_to_v1`。
- **3000 tick 与异 seed @300 的轨迹已重基线**，驱动因素：
  1. 神之手退役 —— 旧引擎在 `count(PRED) < 5` 时每个 substep 追加 1 个主 RNG draw
     （且 0.2% 概率整段 spawn 消费更多 draw），退役后主 RNG 流在这些时刻发生位移；
  2. legacy HERB 分化脚本退役 —— `speciation_fired` 不再被 `< 3` 限流（该计数仍进入校验和流），
     谱系修饰器改为按最近一次事件的确定性派生标签取键；
  3. kind 分叉的密度制动反馈 —— 派生 kind 的 `count(kind)` 参与繁殖门槛计算，
     分叉一旦发生，动力学与旧引擎分道。
- 双跑逐位一致：`golden_double_run_bit_identical`（500 tick）+ 完整测试套件中的
  lockstep 测试全部通过；re-baseline 只发生一次，此后所有测试钉新值。

---

## 2. 核心机制改动

### 2.1 C4 分化引擎泛化（`ecology.rs`）

- **多轴加权 MST**：边权从 1-D `|Δdeco_efficiency|` 推广为
  `core = (|Δspeed|/6.4 + |Δsense|/116 + |Δsize|/1.12)/3`
  `+ EXT_LOCUS_WEIGHT × mean(|Δvalue|)`（id 配对、单侧缺席不计、对称）
  `+ (DECO 家族) |Δdeco_efficiency|`。
  Prim 算法本体不变（`mst_edges_weighted`），平局规则与旧 1-D 版一致（最低索引胜出）。
- **全物种检测**：`check_speciation` 按注册表 id 序遍历每个 kind 的种群，各自跑
  MST 切割 + 地理隔离门 + 每-kind 时间门（`last_spec_tick_by_kind`）。
  阈值/隔离门/最小侧员数保持静态——文档中标注为**测量仪器**（`SPECIATION_DISTANCE_THRESHOLD`
  的 doc comment），对所有 kind 一视同仁，无 per-species 剧本。
- **kind 分叉（kind-fork）**：切割触发时，除注册新 clade 外，从父本 `KindDef`
  **逐字段拷贝**（饮食/行为/上限/能量常数全表），铸造派生 KindDef：
  - 名字确定性派生：`"<父名>-<clade_letter(clade_id)>"`（1→A, 2→B, …, 27→AA），无预设语义；
  - `parent_kind = Some(父 kind)`；六祖先 `parent_kind = None`，永远是树根；
  - **既有个体 kind 不变**（clade 记录树是身份源）；后续该 clade 出生的个体携带新 kind id
    （`reproduce` 经 `clade_kind` 解析）。
- **DECO 家族判定**：niche 竞争排斥、分叉家族判定、成员缓存都从
  `kind == "deco"` 改为 `root_kind(kind_id) == DECO`（沿 `parent_kind` 链上溯），
  派生 DECO 分叉仍活在 deco_efficiency 生态位上。
- **根 clade 惰性物化**：根记录在**该家族第一次分叉时**才物化（`birth_tick = 0` 语义保留），
  未分叉的 kind 不产生 clade 记录 —— 树保持最小，且与既有 DECO 测试的
  "root + daughter = 2 clades" 期望相容。

### 2.2 legacy HERB 分化脚本退役

- 删除 `speciation_fired < 3` 限流（计数保留，仍进入 save/checksum 流）。
- 「迅聚/缓独」写死名替换为确定性派生标签 `"<kind>-L<事件序号><a|b>"`（a = 低速度簇，b = 高速度簇，
  字母无预设语义）；`LineageInfo.id` 改为中性的 `"lineage-a"/"lineage-b"`。
- 谱系 drain/bite 修饰器（原按写死名取键：a 侧 1.15/1.4，b 侧 0.85/0.7）改为按
  **最近一次事件**的派生标签取键（`herb_lineage_drain/bite`），机制保留、名字数据化。

### 2.3 神之手退役（C9，用户定夺）

- 删除 update 循环内的濒危捕食者移民段（PRED<5 时 p=0.002 从天而降，且每 substep 消耗主 RNG）。
- 新增公开注入接口 `spawn_agent(&mut self, kind, x, z) -> Option<u32>`：走完整 spawn 路径
  （cap 检查、随机基因组、RNG draw、generation 0），默认世界中**引擎自身从不调用**。
- 灭绝现在是真实结果：`extinction_records_death_tick` / 新
  `extinction_is_real_no_rescue` 共同钉住。

### 2.4 PRED 猎物扫描数据驱动

- 删除写死 `nearest(HERB)` + `nearest(FISH)` 二选一代码，改为按
  `KindDef.diet.prey_kinds` 顺序扫描（默认 PRED = `[HERB, FISH]`，与旧两次 nearest 调用
  **同序、同距离规则、同 tie-break**）：`nearest` 线性扫 agent Vec 序、严格 `<` 比较，
  同距离保留最低索引。
- 可达性仍是物理约束：`BehaviorTemplate::Fish` 的猎物仅在水边
  （`my_ground < WATER_LEVEL + 3.0`）可达；选择规则保留旧「近岸优先鱼」语义：
  有可达 FISH 候选则优先，否则取扫描序第一个可达候选。默认六物种饮食下与旧二选一逐位等价
  （300 tick v1 轨迹逐位不变即为整体证据；`pred_data_driven_preyscan_equivalent` 钉住
  饮食数据与两种地形 regime）。

### 2.5 checksum v2 提为主（Q2 裁决）

- `checksum_with(kind_field, mix_ext)` 单一混合流：
  - v1（历史）：kind 字段 = kind 名字首字节，`mix_ext = false`；
  - v2（主）：kind 字段 = 注册表 id，`mix_ext = true` —— 每 agent 的 ext 座按 id 序混入
    `locus.id` + f64 值的上下 32 位（固定、与 draw 无关的纯计算）。
- `checksum()` = v2；`checksum_v2()` 保留为别名；`checksum_v1()` 为历史锚点。
- `main.rs` 双输出保持：`SNAPSHOT` 行输出 v1、`SNAPSHOT_V2` 行输出 v2，
  `CHECKSUM_FINAL_V1` / `CHECKSUM_FINAL` / `CHECKSUM_FINAL_V2` 三行。

---

## 3. 契约

### 3.1 SaveState v8（additive）

- `Kinds: Vec<SaveKindDef>` —— 完整运行时注册表（六祖先 + 派生分叉，id 序）；
  旧档无此字段 → 回填六祖先默认。
- `LastSpecTickByKind: Vec<u32>` —— 每-kind 分化时间门；旧档 → 置零。
- `HerbLineageA/B: Option<String>` —— legacy HERB 最近一次分叉的派生标签。
- clade JSON 新增 `kindId` / `derivedKind`（派生物种名，null 表示无）/ `axes`
  （该 clade 活跃的检测轴：`["speed","sense","size","ext"]` + DECO 家族的 `"deco_eff"`）。
- event JSON 新增 `kindId`（受影响 clade 的注册表 kind id；旧记录缺省为 `u32::MAX` 哨兵）。
- 旧 v7 档兼容：`Kinds` 缺省 → 六祖先；clade 无 `kindId` → `u32::MAX`，
  家族判定回退到 kind 名；`evolution`/`generation_fitness` 槽位按注册表长度补齐。

### 3.2 C-ABI（additive，旧 JS 读旧偏移不破）

- clade 记录 40 → 44 字节：`[40..44] = kind_id u32`（clade 的注册表 kind id）。
- event 记录 56 → 60 字节：`[56..60] = kind_id u32`（受影响 clade 的 kind id）。
- 所有既有偏移不变；scratch arena 随记录尺寸增长 512 字节（61,896 → 62,408），
  JS 侧常量同步更新。

### 3.3 RenderPacket / JS

- `AgentView.kind` 本就是字符串，派生 kind 输出派生名（"pred-A" 等），六祖先名不变。
- 扁平 ABI（`sim_agents_flat`）的 `SLOT_KIND` 单字节无法承载任意名字：派生 kind 写
  `b'?'`（0x3F，未知 kind 字节）。
- `living/src/wasm-viewer.mjs`：新增 `KIND_BYTE_FALLBACK = 'herb'`，两处读 kind 字节的
  位置改为 `KIND_BYTE[byte] ?? KIND_BYTE_FALLBACK`（不再静默丢弃未知 kind 的个体；
  派生 kind 以默认几何渲染、保留自身基因组颜色）。
- `living/src/renderer-phylogeny.mjs`：`CLADE_RECORD_SIZE` 40→44、`EVENT_RECORD_SIZE` 56→60，
  读者与引擎 stride 对齐。
- **JS 侧需要同步的改动仅限于这两处映射表/常量**（无新渲染功能）；已一并完成。

---

## 4. 验收结果

| # | 验收项 | 结果 |
|---|---|---|
| 1 | re-baseline 记录完整（commit message + 报告含旧→新锚点），黄金测试钉新值且双跑 bit-exact | ✅ 见 §1；`phase31_golden_regression` 6 passed（含双跑、v1 300-tick 不变性、v2 主/确定性） |
| 2 | `cargo test --release --tests` 全量 0 失败 | ✅ 全 workspace 通过（ecology 92 项 + rng/libm 等） |
| 3 | 强制分叉：非 DECO kind（PRED）人为双峰 + 地理隔离 → 注册新 clade + 派生 KindDef（数据事件） | ✅ `forced_fork_mints_derived_kind`：t≈240 分叉，派生 kind `pred-A`（id=6，parent=PRED），六祖先仍为根，分叉 clade 的新生个体携带派生 kind |
| 4 | 物种可真实灭绝统计测试（神之手确已退役，无救援） | ✅ `extinction_is_real_no_rescue`：t=300 清空 PRED → 再跑 600 tick 仍为 0；`spawn_agent` 可注入 1 只 |
| 5 | PRED 数据驱动扫描等价性 | ✅ `pred_data_driven_preyscan_equivalent`（饮食序 `[HERB, FISH]`、内陆不选 FISH、水边可达性 regime）；整体等价由 300 tick v1 逐位不变背书 |
| 6 | 存档 v8 往返（含派生 kind 注册项 + 旧 v7 档兼容） | ✅ `save_v8_roundtrip_includes_derived_kinds`：Kinds/clade kindId/derivedKind/axes 往返；剥离 v8 字段的 v7 档回填六祖先 |
| 7 | wasm 构建通过 + Node 冒烟（kind 名字 fallback） | ✅ `wasm-release` 构建通过（568,540 B）；`test_phylodynamics.mjs`、`test_wasm_viewer.mjs`、新 `test_phase34_kind_fallback.mjs` 全过（快照 JSON 携带 "deco-A"、扁平 ABI 写 `b'?'`、JS 两处 fallback） |
| 8 | 阶段报告 `research/phase34-report.md` | ✅ 本文件 |

### 4.1 灭绝观测数据

- `extinction_is_real_no_rescue`（seed 20261102）：t=300 时 `kill_kind(PRED)`，
  `count(PRED)` 从 >0 归零；t=300→900 连续 600 tick 保持 0（旧引擎在该窗口会以
  p≈0.002/substep 的濒危移民把 PRED 拉回 ≥1）；`spawn_agent(PRED, x, z)` 成功注入 1 只。
- 自然长跑（Node 冒烟 telemetry）：默认 seed 跑 3000 tick，DECO 在 t≈2461 自然分叉
  1 次（2 clades / 1 event），t≈2761 出现第 2 个事件；无任何"救援"痕迹。

### 4.2 OEE 条件③：分化速率对生态位构造的响应

`oee3_diversification_rate_responds_to_niche_construction`（seed 20261102，2500 tick，
导出观测 = `clade_events().len()` / `species_clades.len()`）：

| 世界 | events | clades |
|---|---|---|
| baseline（默认 3 植物） | 1 | 2 |
| + 注册第 4 植物层 | 1 | 2 |
| + t=0 注入 impact 灾变（r=180, severity=0.8, 240t） | 0 | 1 |

- 灾变注入使分化观测从 1 event/2 clades 变为 0/1（响应成立）；
- 单纯注册第 4 植物层在该 horizon 内未移动事件数（植物层改变资源但未改变该窗口的分化计数）——
  记录为观测事实，不做方向性断言。验收只要求"有响应"，由灾变分支满足。

### 4.3 PRED 扫描等价性说明

1. **数据**：默认 PRED 的 `diet.prey_kinds == [HERB, FISH]`（旧代码两次 `nearest` 的顺序）。
2. **调用序**：数据驱动循环按 `prey_kinds` 顺序逐个调 `nearest` —— 与旧代码的调用序一致，
   且 `nearest` 无 RNG draw、线性扫描 + 严格 `<`，同距离 tie-break 一致。
3. **选择规则**：可达 FISH 候选优先（旧「近岸鱼专家」），否则扫描序第一个可达候选；
   在 `[HERB, FISH]` 两 kind 下与旧四分支 match 逐例等价（推导见 PRED 数据驱动实现注释）。
4. **整体证据**：300 tick / seed 20261102 的 v1 轨迹逐位不变（`0x790184996f3acefd`），
   期间 PRED 正常捕食/巡逻，任何扫描行为差异都会位移 RNG 流。

---

## 5. 已知遗留 / 交接 35/36

- **kind 分叉的统计观测粒度**：`Stats.deco_eff_mean/std`、`Stats` 的 h/p/scav/omn 计数仍按
  六祖先 kind id 统计；派生 kind 的个体不进入这些 HUD 读数（观察层冻结，未改）。
  若后续需要"家族级"读数，应新增导出而非改语义。
- **`evolution`/`generation_fitness`** 按 kind id 分槽，派生 kind 的槽位存在但
  `sample()` 只采样六祖先（每 600 tick 的进化统计不含派生 kind）。
- **`sim_counts`**（6×u32 scratch 计数）仍只统计六祖先；派生 kind 个体在扁平渲染中
  以 `b'?'` + 默认几何呈现，不进入 per-kind 计数。
- **带派生 kind 的家族级 niche 竞争**：`update_niches_and_competition` 的竞争窗口已按
  DECO 家族判定，但 niche 重推导公式仍以 deco_efficiency 为轴（DECO 家族专用）；
  非 DECO 家族 clade 的 niche 恒为父值（无害，竞争/DECO 行为不读取它们）。
- **`speciation_fired`** 仍是 legacy HERB 脚本的计数并进入校验和；Phase 35+ 若要把它
  彻底数据化，需再次 re-baseline。
- **wasm 体积**：545,019 → 568,540 B（+23.5 KB），来自 KindDef 存档/派生 kind 路径；
  若 Phase 35 有体积门槛需关注。
- **`isolation_history` 仍只统计 HERB 隔离群**（legacy 机制，未泛化）。
- 神之手的**实验者注入接口**已保留为 `spawn_agent`（默认世界无自动调用）；
  Phase 35+ 的研究 CLI/测试可直接使用。

---

## 6. 破坏性操作与边界

- 未删除任何文件（临时捕获/调试测试已删除）。
- 未改动 geology / biochem / cataclysm 层动力学（`cataclysm` 仅作为 OEE③ 测试的扰动源）。
- 未新增物种模板库、未给分叉方向任何偏置（对称突变，派生 KindDef 全字段拷贝父本）。
- 未做系统发生渲染的新功能（仅同步 ABI stride 与 kind 名字 fallback）。
- Phase 31/32/33 已验收机制本体（注册表/ext 算子/植物表）未改语义，只在其上泛化。
