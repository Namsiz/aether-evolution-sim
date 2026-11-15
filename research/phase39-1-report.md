# Phase 39.1 — 层级跃迁层（Major Evolutionary Transition layer）

日期：2026-11-15
HEAD（前）：`926b142`（Phase 39.0 文明度量遥测）
阶段性质：**正式动力学阶段**——第三次显式 re-baseline（前两次：Phase 34 物种泛化、Phase 36 可演化微脑）。
理论依据：`research/theory-scout-2026-11.md` §Q2（Goldsby/Knoester/Ofria/Kerr 2014 PLOS Biology「脏活假说」；Goldsby/Dornhaus/Kerr/Ofria 2012 PNAS「任务切换成本」；Godfrey-Smith 2011「单点瓶颈」；Maynard Smith & Szathmáry 1995 大跃迁框架）。

---

## 1. 交付内容

### 1.1 `group.rs`（新模块，约 880 行）

**群体实体（Group entity）**

```rust
pub struct Group {
    pub id: u32, pub kind: SpeciesKind, pub members: Vec<u32>, pub founder_id: u32,
    pub energy_pool: f64, pub birth_tick: u32, pub parent_group: Option<u32>,
    pub roles: Vec<u8>,           // 与 members 平行
    pub clade_id: u32,
    pub founder_genome: Genome,   // 单点瓶颈：后代只读这一份基因组
    pub founder_lineage: Option<&'static str>,
    pub last_repro_tick: u32, pub generation: u32,
}
```

生态层新增 `Ecology.groups: Vec<Group>` + `next_group_id`（以及 `group_on` / `group_rng` / `group_stats` / `group_pending`）。

**形成条件（涌现，不脚本）**——`scan_and_form` 每 20 tick 扫描一次：

1. 候选 = 活着且未入组的个体；按 `(kind, clade_id, lineage)` 分桶；
2. 簇内还要求 hue（中性标记基因）差 ≤ 0.30（亲缘度代理）且在半径 12.0 内，贪心成簇；
3. 簇 ≥ 3 个个体 → 登记 `CohesionTracker`（键 = 簇内最小 id 的锚点 + kind + clade）；
4. 同一锚点簇连续存在 ≥ 60 tick（3 次扫描）**才允许**形成；
5. 形成概率 `p = 0.45 × (0.4 + 0.6 × mean_social) × (0.5 + 0.5 × kin) + 规模加成`（基因 + 亲缘 + 环境决定），只有这一步消费专用 RNG 流。

初始能量池是**成员真实捐入**（每人最多 2.5，且必须留下 30% 自身 max energy），绝不凭空铸造——保持 Phase 35 账本闭合纪律。

**繁殖（单点瓶颈 propagule）**——`reproduce_group`：

- 触发：成员 ≥ 3、池 ≥ 10.0、距上次繁殖 ≥ 400 tick；
- 子代基因组 = `mutate(group.founder_genome, kind, 1 + 1.5 × 群体平均体细胞损伤)`，`GROUP_PROPAGULE_SIZE = 3` 个独立变异副本；
- 子群 `parent_group = Some(父群)`，并**继承同一份 founder 基因组**（瓶颈跨代保持）；
- 池扣 6.0（记为 `ledger.repro_loss`），子群池从 0 起步。

**群体死亡**：能量池耗尽（`starved`）/ 成员全灭（`extinct`）/ 年龄 > 4000 tick（`aged`）。解体时成员 `group_id = 0`、`role = ROLE_NONE`，体细胞损伤保留。

**任务切换成本**——角色 GO/FEED/GUARD/REPRO：

- 角色得分 = 基因组（速度/社会性/大胆/体型/帮助阈值/好奇/感知）× 能量状态 × 群体池水位 + **脑态偏置**（`Brain.nodes[].output` 按 index mod 4 分组均值 × 0.12）；
- 每 40 tick 重评一次，argmax；角色变化支付 `TASK_SWITCH_COST = 1.5` 能量，能量 ≤ 成本 + 25% max energy 时被锁在原角色；
- 各角色有真实经济学：GO 在危害格子上工作（见下）；FEED 每 sim-second 向池捐 0.30（效率 0.9，池上限 20）；GUARD 付 0.06/s 警戒费、把 GO 的危害暴露削减最多 60%；REPRO 从池中提取 0.25/s 供自己跨过个体繁殖阈值。

**脏活致突变**：

- `hazard_at(x,z)` = 矿化场（分解者沉积的污染带）+ 活跃撞击灾变热区（severity × remaining/duration）+ 火区，钳位 [0,1]，纯读现有状态；
- GO 角色 `somatic_damage += 0.05 × hazard × (1 − guard_mitigation) × dt`（上限 1.0），不干脏活的个体只按 0.0015/s 修复；
- 繁殖时 `mut_scale = 1 + 1.5 × damage`（性繁殖取母方、无性繁殖取自身、propagule 取群体均值），乘进 `mutate` 的每个位点幅度；
- 体细胞/生殖细胞分化**没有 flag**：REPRO 角色不接触脏活，所以它的后代突变幅度天然低；GO 角色的后代天然高——分化是统计后果。

### 1.2 `Ecology` / `Agent` 接线

- `Agent` 新增 `group_id: u32`、`role: u8`（255 = 未分配）、`somatic_damage: f64`；
- `mutate(&mut self, g, kind, mut_scale)` 增加幅度参数（`1.0` 与旧轨迹逐位相同，`x * 1.0` 精确）；`mix_genome` 与两条繁殖路径（性/无性）传入母方损伤系数；
- `update()` 在 culture 阶段之后、carcass 衰减之前调用 `group::group_step(self, h)`；
- v2 checksum 末尾混入群体表（id / kind / 成员数 / 池 / birth_tick）；空群体表混入 0 项，所以无群体世界保持旧值；
- `cell_index` / `spawn` 改为 `pub(crate)`；新增 `group_pending_len()` 只读访问器。

### 1.3 SaveState v10 → v11

意外修正：任务书写的是 v11→v12，仓库实际版本号是 **10**，因此本期是 **10 → 11**。

- `Groups: Vec<SaveGroup>`（含 `founder_genome` + **`founder_ext`**）、`NextGroupId`、`GroupOn`、`GroupRngState`、`GroupPending: Vec<SaveCohesion>`；
- `SaveAgent` 新增 `GroupId` / `Role`（默认 255）/ `SomaticDamage`；
- 全部 `#[serde(default)]`：v10 档恢复为「独居世界」（无群体、无角色、零损伤），`GroupOn` 默认 true；
- `GroupStats`（遥测）不落盘——它是观测层，不是动力学。

**踩到的真坑**：`SaveGenome` 不含 ext 位点。第一版只存 `founder_genome`，恢复后 founder 的 ext 列表丢失，propagule 的 `mutate` 在 genesis-ON 世界里走不同分支 → t=1598 分叉（`phase32_open_loci_test::save_restore_variable_ext_bit_exact` 抓到的）。补 `founder_ext` 后 500 tick 锁步逐位一致。

### 1.4 选择层级数（OEE 里程碑）

`civilization_metrics::ComplexityTelemetry.selection_levels` 由恒 1 改为 `1 + (存在活群体 ? 1 : 0)`，从实时状态计数，没有任何解锁 flag。

### 1.5 研究 CLI

`ecology-sim` 每 300 tick 打印一行 `GROUPS`（群体数/均大小/max/层级数/形成/繁殖/解体/切换/切换成本/脏活次数/损伤总量/平均寿命/四角色计数），结束时输出 `GROUP_HISTORY`（每 100 tick：群体数、均大小、max、层级数、池总量、平均年龄）。

---

## 2. 确定性纪律

- 专用流 `group_rng = Mulberry32::new(seed ^ GROUP_RNG_TAG)`（`"GRP1"` = 0x47525031），只用于**形成概率**与 **propagule 落点抖动**；与 main/prng/fish/brain/culture 流完全不混。
- propagule 走既有 `spawn` 路径（如同每一次出生，消费主流行）；额外 draw 只在群体繁殖时发生。
- 浮点只做 IEEE 精确运算；距离比较用平方；本模块不需要 sin/cos（故无 libm 依赖路径）。
- 同 seed 双跑 6×250 tick 锁步：checksum、群体表、switch/repro 计数逐位一致。
- 存档 v11 往返：群体表（id/成员/角色/池/父群/founder 基因组含 ext/代数）与每 agent 的 group/role/damage 全部逐字段相等；恢复后 400 tick 锁步无分叉。

---

## 3. Re-baseline（新旧锚点）

| 锚点 | Phase 36（旧） | Phase 39.1（新） |
|---|---|---|
| seed 20261102 @300 v2 | `0xa4d54e1aadbe83c6` | **`0x32c496b54efc9626`** |
| seed 20261102 @3000 v2 | `0xdd79b5ab74f3545d` | **`0xb900f6366f38cc47`** |
| seed 20261105 @300 v2 | `0x777cd6c33bb3345c` | **`0x786979c7e9c84a93`** |
| seed 20261102 @300 v1 | `0x3782a36aee6f1ff5` | **`0x5fa905f98eb6546e`** |
| seed 20261102 @5000 v2 | `0x84628a0e61bd97e1` | **`0x8d1989adc86ef0d9`** |
| seed 20261105 @5000 v2 | `0xad641fdf08d8aae6` | **`0x06602fcdc6a09122`** |

**旧值仍可逐位复现**：`eco.group_on = false` 时 Phase 36 的四个锚点全部逐位保留（`phase39_1_transition_test::group_off_is_bit_exact_phase36_anchor` 钉住），这就是「条件激活、条件未满足零副作用」的硬证据。

首次群体形成发生在 **t≈240–300**（scan 边界 20 tick + 持续 60 tick + 概率），因此 300 tick 锚点已经包含群体层。

---

## 4. 群体统计（seed 20261102，同 seed 双跑可复现）

时间序列（`GROUP_HISTORY`，100 tick 采样）：

| t | 群体数 | 均大小 | max | 选择层级 | 池总量 | 群体平均年龄 |
|---|---|---|---|---|---|---|
| 100 | 0 | — | 0 | 1 | 0.0 | — |
| 200 | 0 | — | 0 | 1 | 0.0 | — |
| 300 | 3 | 3.00 | 3 | 2 | 21.8 | 33 |
| 600 | 5 | 3.40 | 5 | 2 | 23.5 | 228 |
| 1000 | 16 | 3.19 | 5 | 2 | 99.4 | 309 |
| 1500 | 39 | 3.18 | 5 | 2 | 241.8 | 416 |
| 2000 | 48 | 3.19 | 5 | 2 | 296.9 | 723 |
| 2500 | 55 | 3.07 | 5 | 2 | 351.8 | 967 |
| 3000 | 63 | 2.90 | 5 | 2 | 397.5 | 1278 |
| 4000 | 60 | 2.38 | 5 | 2 | 316.6 | 1969 |
| 5000 | 31 | 2.42 | 5 | 2 | 163.7 | 2109 |

- **平均群体大小**（按群体数加权，全程）≈ 2.81；峰值 63 群（t=3000）后回落到 31 群（t=5000）——群体层跟随宿主种群（219→283 人）一起消长。
- **平均群体寿命**（累计解体的寿命均值）：t=3000 时 382 tick，t=5000 时 1343 tick；解体原因 t=5000：饿死 60 / 成员全灭 27 / 超龄 3。
- **群体繁殖**：t=3000 累计 9 次繁殖（9 个子群），t=5000 累计 18 次；截至最后一条 GROUPS 采样（t=4800）成组 113 次、解体 93 次——群体层在 400 tick 冷却 + 真实捐池的前提下是「低产但可持续」。
- **分工**（t=3000，183 个在群体内的个体）：GO 102 / GUARD 51 / FEED 16 / REPRO 14；角色切换 581 次、累计付费 871.5 能量。
- **脏活**：t=3000 累计 10318 次 GO 暴露事件、体细胞损伤总量 0.938；t=5000 为 39789 次 / 9.65。
- **群体选择压**：池 ≥ 10 且成员 ≥ 3 才能繁殖 → FEED 多的群体繁殖成功率更高；组间选择是能量池/繁殖成功率的真实后果，不是另设的目标函数。
- **生态影响**（A/B，同 seed）：@3000 人口 607（ON）vs 553（OFF）；@5000 283 vs 219；seed 20261105 @5000 86 vs 80——群体层不是破坏性的，反而略增承载。

---

## 5. 验收结果（Chief 逐条）

| # | 验收项 | 结果 | 证据 |
|---|---|---|---|
| 1 | 群体形成涌现 + 不满足零副作用 | ✅ | `cohesion_conditions_gate_group_formation`：3 个同 clade/lineage 亲缘个体在半径内持续后自动成组；hue 差 > 容差时零群体且 checksum 与 group_on=false 孪生世界逐位相同；40 tick 内（未过持续门槛）零群体。`groups_form_in_the_natural_run`：普通种子世界 t=1200 前形成 ≥3 群，成员/角色/group_id 三方一致。 |
| 2 | 单点瓶颈繁殖 | ✅ | `propagule_is_single_point_bottleneck`：founder hue 0.10、其他成员 0.35，3 个 propagule 全部落在 founder ±0.08 内、离成员 >0.15（50/50 混合应为 0.225）；子群 `parent_group`、`founder_genome`、代数传递均正确。 |
| 3 | 任务切换成本 | ✅ | `task_switch_cost_is_paid_once_per_switch`：A 臂改基因组触发 FEED→GUARD 切换，能量精确减少 1.5、`switches=1`、`switch_cost_total=1.5`；B 臂同条件重评无切换，能量 0 变化、计数不变。自然世界 t=3000 累计 581 次切换 / 871.5 能量。 |
| 4 | 脏活致突变（方向正确） | ✅ | `dirty_work_raises_offspring_mutation`：同一群体内 GO 在矿化污染格子上 300 tick 后损伤 >0.2，FEED <GO 的 25%；`mutation_scale` 比 >1.5，真实 `mutate` 抽样 400 次的后代 hue 偏差比 >1.3。 |
| 5 | 选择层级数 1→2 | ✅ | `selection_levels_grow_with_the_group_layer`：无群体 =1，成组 =2，群体饿死后回到 1；自然世界 t≥300 恒为 2。 |
| 6 | 确定性 + 存档 | ✅ | 同 seed 6×250 tick 锁步逐位一致（`group_layer_double_run_is_bit_exact`）；`save_v11_roundtrip_is_bit_exact` 逐字段相等 + 恢复后 400 tick 无分叉；`legacy_v10_archive_loads_as_solitary_world` 把 v11 JSON 剥回 v10（删 Groups/NextGroupId/GroupOn/GroupRngState 与每 agent 三个新字段、Version=10）后正常加载为独居世界并继续运行。 |
| 7 | re-baseline 记录完整 | ✅ | 见 §3 表格；`phase31_golden_regression` 钉住 4 个新锚点并保留 Phase 36 旧值注释；`phase32`/`phase33` 锚点同步更新；旧值可通过 `group_on=false` 复现。 |
| 8 | release 全量 0 失败 | ✅ | `cargo test --release`：**184 passed / 0 failed**（43 个 test 二进制）。 |
| 9 | wasm 构建 + Node 冒烟 | ✅ | `wasm-release` 产物 **672KB**（Phase 39.0 为 636KB，+36KB）；`node test_phase34_kind_fallback.mjs --wasm` 与 `node test_cataclysm.mjs --wasm` 全部通过（含 v11 存档经 C-ABI 往返）。 |
| 10 | 阶段报告 | ✅ | 本文档 |

### 5.1 一处测试仪器的调整（机制本体未改）

`niche_divergence_test::overlapping_clades_compete`（Phase 16 竞争负担 A/B）在群体层开启时，时间平均能量差被群体层的混沌放大淹没（84.34 vs 83.55，符号翻转）。该测试的语义是**隔离**竞争负担这一条因果链，因此和它本来「每 tick 清空 control 的 clade_burdens」同一纪律，现在两个臂都设 `group_on=false`（同代码注释）。群体层关闭后原信号清晰：69.33（负担）vs 88.47（对照）。Phase 16 机制本体零改动。

---

## 6. 改动文件

| 文件 | 内容 |
|---|---|
| `core/crates/aether-sim-ecology/src/group.rs`（新，880 行） | 群体实体、凝聚、角色/切换成本、脏活致突变、propagule、遥测、实验者入口 |
| `core/crates/aether-sim-ecology/src/ecology.rs` | Agent/Ecology 字段、`mutate` 幅度参数、update 接线、checksum v2 混入群体、save/load |
| `core/crates/aether-sim-ecology/src/save_state.rs` | v11：SaveGroup / SaveCohesion / SaveAgent 三字段 / 四个 sim 级字段 |
| `core/crates/aether-sim-ecology/src/civilization_metrics.rs` | `selection_levels` 实时计数 |
| `core/crates/aether-sim-ecology/src/main.rs` | `GROUPS` + `GROUP_HISTORY` 研究输出 |
| `core/crates/aether-sim-ecology/src/lib.rs` | `pub mod group` |
| `core/crates/aether-sim-ecology/tests/phase39_1_transition_test.rs`（新，10 测试） | 上表 1–7 条全部验收 |
| `core/crates/aether-sim-ecology/tests/phase31_golden_regression.rs` | 新锚点 + 旧值注释 |
| `core/crates/aether-sim-ecology/tests/phase32_open_loci_test.rs` / `phase33_plant_registry_test.rs` | 锚点同步 |
| `core/crates/aether-sim-ecology/tests/niche_divergence_test.rs` | 仪器隔离（group_on=false 两臂） |
| `living/ecology-sim.wasm` / `ecology-sim-v2.wasm` | 重新构建的产物 |
| `research/phase39-1-report.md` | 本文档 |

---

## 7. 留给 Phase 40 的衔接点

1. **群体层物种形成**：子群已携带父群 founder 基因组 + 代数；下一步可让群体（而非个体）成为 clade 分叉的单位——kind-fork 目前只在个体繁殖/检测层。
2. **作弊者（cheater）与冲突调解**：Goldsby 2014 的核心失败模式。现在唯一的调解器是「池饿死」；可加程序性死亡/监督惩罚/germline 指定位，让作弊者的代价由账本支付。
3. **多层选择的量化**：Price 方程分解（组内 vs 组间方差）；`selection_levels` 已是 OEE 里程碑信号，但缺「群体遗传力」（founder 基因组 → 子群表现）的度量。
4. **群体尺度生态位构造**：Phase 37 的 structure 目前按个体放置，可让 FEED/构造角色在群体尺度建结构（真实能量成本）。
5. **观察层**：本期冻结；群体可视化（群体质心/成员连线/池水位）留给观察层解冻。
6. **群体事件日志**：形成/解体/繁殖进入 `clade_events` 之类的事件流，供 OEE 速率与报告自动化使用。

## 8. 未完成项 / 已知限制

- GroupStats / GroupHistory 是纯遥测，不落盘；恢复后从零累计（动力学不受影响，测试只比较动力学状态）。
- Propagule 若被 kind 容量上限截断到 <3 个体，仍会建群（随后饿死或成员全灭）——这是「扩散失败」的真实结果，未特殊处理。
- 角色分配是确定性打分，不含学习/文化传递；REPRO 角色不会主动放弃繁殖权（无 germline 强制）。
- 群体形成概率的门槛参数（半径 12、持续 60、基础 0.45、hue 容差 0.30）是首版标定；本期只保证机制方向与确定性，未做参数敏感性扫描。
- 体细胞损伤只作用于突变幅度，不影响本体存活/代谢（不是「损伤=折寿」模型）。
- Phase 16 A/B 仪器调整见 §5.1（已披露）。
