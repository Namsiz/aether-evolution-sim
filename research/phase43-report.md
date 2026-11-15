# Phase 43 — 代际文化传承（文化棘轮）+ Abiogenesis（生命再起源）

日期：2026-11-15
HEAD（前）：`2e47f2b`（Phase 42 三脑转正，第八次 re-baseline）
阶段性质：**第九次显式 re-baseline**。两个机制打包交付：出生传承（parent→child token inheritance，Tomasello 文化棘轮）与 Abiogenesis（热泉化学能→生命的自然再起源路径）。

---

## 0. 一句话结论

**文化棘轮**：新生儿出生时从亲代 `known_tokens` 继承——每位教师按自己的 token 序列逐个教学，每 token 支付 `TEACH_COST = 0.5 × LEARNING_COST = 0.25` 真实能量、以 `DEFAULT_FIDELITY = 0.9` 概率完整传承（失败的 token 不传承 = 教导失败/关键期错过）；能量付不起下一位即停课（贫穷约束，零 RNG）。有性繁殖双亲各自教学，群体繁殖 founder 教学所有 propagule 子代。「发明→死亡→清零」循环被打破：seed2 30k 处 bearer 从 Phase 42 的 0-3 升至 167，taught 事件 5557 次且加速累积。

**Abiogenesis**：全局存活 < 8 时，每 tick 以 `ABIO_P_BASE × mean(候选格强度)` 的概率从矿化活跃/水缘高湿格子自发萌发一个祖先个体（`PARENT_KIND == None` 注册表序均匀选择，栖息地相容过滤），初始能量入账 `ledger.chemo_in`（热泉化学能，与 light_in/filter_in 同级的开放输入通道）。健康世界零触发零 draw（生态饱和抑制）；灭绝世界数千~数万 tick 内自然重启。这是「接入现实规律」的正名：不是神之手复活，是米勒-尤里/热泉口化学演化。

---

## 1. 文化棘轮（cultural ratchet）

### 1.1 出生传承（`culture::inherit_at_birth`）

- **触发**：所有出生路径——有性（`reproduce` 配对分支）、无性（solo 分支）、群体（`group.rs::reproduce_group`）。
- **教学顺序**：教师按自己的 `known_tokens` 列表序逐 token 处理。每个 token：
  1. 教师能量 < `TEACH_COST` → **停课**（贫穷约束，零 RNG，后续 token 全部不传——真实的代际流动约束）；
  2. 子代已持有（另一位教师已传）→ 跳过（零 RNG、不重复付费）；
  3. `culture_rng` 一次保真度 roll（`< DEFAULT_FIDELITY = 0.9` 通过）→ 成功则支付 `TEACH_COST` 并完整传承 token id；失败不传承、不付费。
- **教学成本标定**：`TEACH_COST = LEARNING_COST × 0.5 = 0.25`。传授已掌握的知识比从零习得便宜（示范 vs 试错），但多 token 家庭每个孩子累计支付 `0.25 × len`——亲代投入随知识存量线性增长。教学在繁殖扣费（drain 58/48）**之后**结算：付不起繁殖代价的亲代自然也付不起教学。
- **有性路径双亲各自教学**（哺乳/鸟类双亲抚育），先 idx 后 mate；子代得两亲并集。
- **群体路径**：founder 是教学者。每个 propagule 子代独立学 founder 的完整课程（同一份课程教 n 个孩子）；founder 能量逐孩子递减，付不起即对后续孩子停课；founder 已死则无传承。
- **token 上限**：继承后子代 `known_tokens` 与现有语义一致（无额外上限约束——现有代码本就无 cap，注册表开放增长）。
- **RNG 纪律**：draw 预算固定可审计——≤ 1 draw/（教师, token）；贫穷 break 零 draw（测试断言 rng 状态不变）；走专用 `culture_rng` 流。

### 1.2 遥测与开关

- `culture_stats.taught_count`（传承 token 总数）、`culture_stats.teach_cost_total`（教学总成本）。
- `teach_on` 开关（默认 `TEACH_ON = true`，SaveState 持久化）——false 时出生教学是 bit-exact no-op（A/B 臂）。

---

## 2. Abiogenesis（生命再起源）

### 2.1 触发（涌现式，不脚本化）

`try_abiogenesis()` 在每 substep 的死亡清理之后调用：

- **硬门**：全局存活个体 ≥ `ABIO_MIN_POP = 8` → 直接返回（零扫描零 draw）。生态饱和抑制是结构性的：健康世界（数百个体）永不触发。
- **候选格**（76×76 全格扫描，仅在世界濒死时付出成本）：
  - 矿化活跃：`mineral_field ≥ 0.15`（DECO 沉积场仍在工作的格子），或
  - 水缘高湿×肥沃：`moist > 0.3 && suit > 0.5`。
  - **阈值依据**（读现有场数据标定）：`moist` 由岸边 BFS 生成（`0.45 − 0.035·d`，值域上限 0.45）——任务建议的 `moist > 0.6` 超出值域、永不触发；`0.3` ⇔ 距水线 ~4 格内的湿润带，是值域内的等效「高湿」分位。`suit ∈ [0,1]`，草生长需 > 0.1，`> 0.5` 为肥沃土壤。`mineral_field ∈ [0,1]` 按活性分解位点沉积、每 tick ×`MINERAL_DECAY` 衰减，0.15 是衰减间隙中仍可辨认为活跃的下限。
- **强度与概率**：格子强度 = `0.5·mineral + 0.5·min(moist/0.45, 1)`（两路化学能代理各半）；每 tick 概率 `p = ABIO_P_BASE(6e-4) × mean(候选强度)`。标定：强度 ~0.4 → p_eff ≈ 2.4e-4/tick → 死世界首萌发期望 ~4k tick，落在「数千-数万 tick 自然重启」目标带。
- **产生什么**：候选格均匀选一（1 draw）→ 栖息地相容的祖先（`PARENT_KIND == None` 的注册表序：水面格 → FISH，陆地格 → HERB/PRED/SCAV/OMN/DECO）均匀选一（1 draw）→ `spawn(kind, x, z, None, 0, 0.0)`（随机基因组 + 主流出生 draw，与 seed_life 同路径）。候选集为空则零 draw 放弃。
- **能量学（热力学闭合）**：`ledger.chemo_in += 出生能量`；`total_in()` = light_in + filter_in + chemo_in。新能量经现有代谢/繁殖通道退出（metabolism_loss 等已在 total_loss 口径）。checksum 不含 ledger——abiogenesis 改变轨迹是经 agents/grass/births 反映的，属预期 re-baseline。
- **明确不做**：新物种形态（复用注册表）、前生命化学模拟（RAF 留待未来）、渲染。

### 2.2 RNG 流与 SaveState

- 专用流 `abio_rng = seed ^ ABIO_RNG_TAG`（0x4142494F "ABIO"）；draw 预算：pop < 8 且候选非空时每 tick ≤ 1 触发 roll，命中后 +1 选格 +1 选祖先。
- `abiogenesis_on` 开关（默认 `ABIOGENESIS_ON = true`，SaveState 持久化）。
- **SaveState v13 → v14**：`SaveCultureStats` + `TaughtCount`/`TeachCostTotal`；sim 级 + `TeachOn`/`AbiogenesisOn`/`AbioRngState`/`AbiogenesisEvents`/`AbiogenesisTicks`。全部 `#[serde(default)]`/`default_true` → v13 存档恢复为「两开关 ON、计数器 0、流按 `seed ^ TAG` 重派生」，续跑确定性。

---

## 3. re-baseline 第九次（新旧锚点）

@300 全部不变（tick 300 时世界尚无发明，传承为 bit-exact no-op）；仅 @3000 与 group_off @3000 漂移（发明已发生、子代开始继承）。

| 检查点 | Phase 42（旧） | **Phase 43（新）** |
|---|---|---|
| seed 20261102 @300 v2 | `0xdc98f7d34a7af777` | **`0xdc98f7d34a7af777`（不变）** |
| seed 20261102 @3000 v2 | `0xafa833641ebfd5a8` | **`0x9af70d3dc96532f2`** |
| seed 20261105 @300 v2 | `0x2e9539930abd8975` | **`0x2e9539930abd8975`（不变）** |
| seed 20261102 @300 v1 | `0xe6404a7bee633eb8` | **`0xe6404a7bee633eb8`（不变）** |
| group_off @300 v2 | `0xf5cc546cb068941f` | **`0xf5cc546cb068941f`（不变）** |
| group_off @3000 v2 | `0x615fbdaaa7fb0f4e` | **`0x1ec5282f975a354e`** |
| group_off alt @300 v2 | `0x7a3fe91e7165c234` | **`0x7a3fe91e7165c234`（不变）** |
| group_off @300 v1 | `0x5b514478b9799fa0` | **`0x5b514478b9799fa0`（不变）** |

漂移来源仅一处：出生传承改变了发明者的子代能量 → 行为分叉。abiogenesis 在锚点世界（健康种群）零触发、零贡献。

---

## 4. 验收逐条（Chief 验收表）

1. **文化棘轮生效**：`bearer_ratchet_survives_population_turnover`（3000t 注入 10 持有者，ON 臂 taught ≥ 20 且 bearer > OFF 臂）+ 100k 观测 bearer 曲线持续上行（§6.1）——**通过**。
2. **教学成本真实**：`birth_teaching_charges_real_parent_energy`（跨臂能量差 = 0.25×传承数，1e-9 精度）；`poor_parent_teaches_less`（400 token 课程 × 2.5 能量 → ≤ 10 课，顺序截断）——**通过**。
3. **abiogenesis 重启**：`extinct_world_reboots_via_abiogenesis`（杀光 seed2 → 事件发生 → chemo_in > 0 → total_in 含 chemo 通道 → 40k 内 ≥ 2 次事件）；seed05 自然灭绝后的重启见 §6.2——**通过**。
4. **繁荣世界不乱冒生命**：`healthy_world_abiogenesis_suppressed`（seed2 3000t → 事件 0、chemo_in 0）——**通过**。
5. **确定性**：双跑 bit-exact（锚点测试 + `teaching_double_run_bit_exact` 注入世界）；v14 往返位精确（`abiogenesis_save_roundtrip_bit_exact` 在 ABIO 流活跃的死世界快照后续跑 1500t 位精确）；v13 兼容（`v13_archive_lands_on_phase43_defaults`：缺字段 → ON 默认 + 重派生流 + 双恢复锁步）；v10/v11/v12 旧档回归通过——**通过**。
6. **re-baseline 第九次**：锚点全表见 §3，旧值保留在 `phase31_golden_regression.rs` 头注释与本报告——**通过**。
7. **release 全量 0 失败**：**153 passed / 0 failed / 9 ignored**（aether-sim-ecology --tests；基线 143 + phase43 新增 10）+ aether-brain 22 passed——**通过**。
8. **wasm + Node < 16ms**：见 §7——**通过**。
9. **长程观测**（100k × 3 seeds + OFF 臂）：见 §6——**通过**。
10. **报告**：本文件自包含——**通过**。

---

## 5. 改动文件

- `core/crates/aether-sim-ecology/src/culture.rs` — `TEACH_COST`（标定注释）、`CultureStats.taught_count/teach_cost_total`、`inherit_at_birth`（贫穷零 RNG break / 保真度 roll / 去重跳过 / 遥测）。
- `core/crates/aether-sim-ecology/src/ecology.rs` — ABIO 常量组（`ABIO_RNG_TAG`/`ABIOGENESIS_ON`/`TEACH_ON`/`ABIO_MIN_POP`/`ABIO_P_BASE`/三阈值 + 标定依据注释）、`EnergyLedger.chemo_in` + `total_in()`、Ecology 字段（`teach_on`/`abiogenesis_on`/`abio_rng`/`abiogenesis_events`/`abiogenesis_ticks`）× 2 构造器、`try_abiogenesis`（update 子步死亡清理后挂载）、出生教学钩子（有性双亲/无性单亲）、to/from_save_state v14 接线。
- `core/crates/aether-sim-ecology/src/group.rs` — `reproduce_group` founder 教学路径（快照/逐 propagule 教学/结算写回）。
- `core/crates/aether-sim-ecology/src/save_state.rs` — v14 schema（版本历史注释 + SaveCultureStats 2 字段 + SaveState 5 字段，全 serde default）。
- `core/crates/aether-sim-ecology/tests/phase43_culture_inheritance_test.rs` — 6 项验收（单测语义/出生成本跨臂/贫穷截断/group founder/双跑/棘轮 A/B）。
- `core/crates/aether-sim-ecology/tests/phase43_abiogenesis_test.rs` — 4 项验收（灭绝重启+能量入账/繁荣抑制/存档往返/v13 兼容）。
- `core/crates/aether-sim-ecology/tests/phase43_longrun_observatory.rs` — 100k×3 seeds + OFF 臂探针（ignored；`PHASE43_ONLY` 选臂并行）。
- 既有测试锚点同步：`phase31_golden_regression.rs`（第九次锚点块）、`phase39_1_transition_test.rs`（group_off @3000 + v10/v11 版本断言）、`phase41_triune_test.rs`（v14 JSON 断言）、`phase42_brain_authority_test.rs`（v14 版本断言）。
- `living/ecology-sim.wasm`、`living/ecology-sim-v2.wasm` — wasm-release 重建产物（773 KB）。

---

## 6. 长程观测（100k × 3 seeds，`phase43_longrun_observatory`）

### 6.1 文化棘轮：seed 20261102 ON vs OFF（bearer 曲线）

| t | ON bearer | ON pop | ON taught | OFF bearer | OFF taught |
|---|---|---|---|---|---|
| 5k | 10 | 628 | 12 | 0 | 0 |
| 10k | 17 | 680 | 41 | 3 | 0 |
| 20k | 69 | 465 | 1088 | 3 | 0 |
| 30k | **167** | 481 | 5557 | 1 | 0 |
| 50k | **380** | 574 | 31971 | 0 | 0 |
| 75k | **522** | 661 | 67587 | 1 | 0 |
| 85k | **526**（pop 的 77%） | 680 | 99814 | 1 | 0 |
| 100k | **333** | 482 | 138645 | **0** | 0 |

- **棘轮生效**：bearer 从 Phase 42 的「长期 0-3」变为持续爬升（峰值 526/680）；taught 138,645 次传输、teach_cost_total 34,661.2 = 0.25 × taught（精确自洽）。
- **OFF 臂 = Phase 42 世界逐位重现**：OFF 臂 100k 终点 tokens=323 —— 与 Phase 42 报告的「323 @100k」完全一致，bearer 终点 0（Phase 42 观测「bearer 长期 0-3」）。传承开关的 A/B 差异即机制效应：**ON 333 bearer vs OFF 0**。
- 曲线在 17.5k（24→78）与 75k（239→522）两处跳升 —— 教学链的 founder 效应放大（多 token 个体成功繁殖的爆发）。
- 92.5k 处种群低谷（449）同步压缩 bearer（289）—— 棘轮跟随种群而非归零，恢复后重新爬升。

### 6.2 Abiogenesis：seed 20261105（继承性灭绝 → 反复再起源）

Phase 42 基线：灭绝 @9.6k 后死寂 91k tick（100k 终点 pop=0）。Phase 43：

- 灭绝 @9.6k 仍发生（继承性命运保留——文化层无法拯救基因层的灭绝）；
- 随后 **13 次灭绝、25 次萌发事件**，世界在 0↔1↔8 个体间震荡：萌发容易、立足难（单个体面临 Allee 择偶/无性阈值），但化学火花持续重燃，最终段 pop 攀至 8（ABIO_MIN_POP 门槛），**100k 终点 alive（pop=8）**；
- `chemo_in` 累计 1893.8（25 次萌发的初始能量入账）；tick 间隔符合标定（首萌发 ~40k 死寂后，后续 ~600-3000 tick 一次，取决于 pop<8 门限开合）。
- seed05 taught=0：种群始终太稀疏（无出生即无教学）——文化与 abiogenesis 的耦合在下一阶段值得观察（萌发个体繁殖起来后，token 才有宿主）。

### 6.3 seed 42：文化层整体灭绝（诚实记录）

alive pop=135，但 bearer=0、taught=0、tokens 冻结在 39（发明在 ~7.5k 后停止，幸存基因池 curious 漂到 INVENT_MIN_CURIOUS=0.5 以下，且 token 持有者在 ~10k 前全部死亡）。棘轮机制本身无缺陷（seed2 同机制爆发），但**持有者谱系死亡即文化终结**——这正是棘轮要解决、但只能在持有者成功繁殖时解决的问题。

### 6.4 健康世界 abiogenesis 全程抑制

seed2 ON/OFF、seed42 三臂 100k 全程 `abio_ev=0`、`chemo_in=0.0` —— 生态饱和抑制在长程下稳定成立。

---

## 7. wasm + Node 冒烟

- 冒烟全过（census/snapshot v3/矿物采样/FinColor 等契约全部 OK），**15.77 ms/frame**（60 帧 sim+export，预算 16 ms）。
- **同机同时刻 A/B 归因**：HEAD（Phase 42）旧 wasm 三跑 15.79/15.93/15.79 ms vs Phase 43 新 wasm 15.71/15.77/15.77 ms —— 噪声内无差异，本期改动对帧预算代价 ≈ 0（新增路径：每 tick 一次 O(n) 存活扫描，且仅 pop<8 时才做全格扫描）。
- 说明：Phase 42 报告的 6.53 ms 与本容器当前环境不可比（41.1 基线 13.26 ms 同样慢于此环境 ~2.4×）；余量在本容器上偏薄（<2%），建议在交付环境复测。
- wasm 产物 773 KB（767 → 773，+6 KB）。

---

## 8. 关键统计汇总

| 指标 | Phase 42（基线） | **Phase 43** |
|---|---|---|
| seed2 bearer @30k | 0-3 | **167** |
| seed2 bearer @100k | 0 | **333**（峰值 526） |
| seed2 taught @100k | —（无机制） | **138,645**（成本 34,661 能量） |
| seed2 tokens @100k | 323 | 247,060（OFF 臂重现 323） |
| seed05 命运 | 灭绝 @9.6k + 死寂 91k | 灭绝 @9.6k → **25 次再起源**，alive pop=8 |
| 健康世界 abio 事件 | — | **0**（3 臂 × 100k） |
| 测试 | 143 passed | **153 passed / 0 failed**（+brain 22） |
| SaveState | v13 | **v14**（v13/v12/v11/v10 兼容） |
| wasm | 767 KB | 773 KB（帧时序同机 A/B 无差异） |

---

## 9. 遗留与 Phase 44 衔接点

1. **保真度仍是常量**（0.9）：文化保真度基因化（per-lineage fidelity 进化）是天然的 Phase 44 候选——棘轮已建立，保真度的选择压首次可测量（taught 遥测 + bearer 曲线给了适应度梯度）。
2. **teach 成本未入独立 ledger 通道**：教学能量走 agent 能量 → 代谢路径隐式退出（与 LEARNING_COST 同等口径）；若 Phase 44 要做教育投入的演化经济学（亲代投入 vs 存活权衡），建议给 `ledger.teach_cost` 独立通道（会 re-baseline）。
3. **abiogenesis 的祖先选择是均匀随机**：不偏好「最原始」表型（任务即此定义）；若未来引入化学前生命层（RAF），起点选择应改为化学适宜度加权。
4. **发明速率未调**：INVENT_P=5e-6 下 token 注册表 100k 到 ~3 万（远超 bearer 承载），「发明:传播」比例失衡依旧——棘轮解决了传播慢，但失传检查（check_extinct）只做遥测未执行淘汰；下一阶段候选：注册表压缩/失传执行。
5. **观察层冻结、无渲染、无前生命化学**（按任务要求未做）。
6. **abiogenesis 立足率低**（seed05 观测）：萌发个体在 Allee 择偶/无性阈值前存活率高但繁殖难，世界在 0↔1 震荡；候选：萌发时额外给一次能量充足铺底（化学能富集）或允许短程同类聚集——需 Phase 44 拍板，会改标定。
7. **文化-abiogenesis 耦合未观测到**（seed05 太稀疏）：待世界稳定重启后，token 才有宿主；Phase 44 若做保真度基因化，可把「灭绝-重启循环中的文化重播」作为观测点。

---

## 10. 提交

- commit：phase43（本报告 + 上述文件）
- 测试：`cargo test --release -p aether-sim-ecology --tests` 153 passed / 0 failed；`cargo test --release -p aether-brain` 22 passed；Node wasm 冒烟 OK。


---

## Phase 43.1 — abiogenesis 地质节律 + 概率补偿 + 测试语义修正（Chief 补丁）

### 性能
wasm 冒烟在 Phase 43 交付后超预算（16.99-20.11 ms/frame > 16）。对照实验定位：`try_abiogenesis` 每 tick 的 alive-filter 每次触碰 ~2 MB agent 状态（250 agents 的大结构体），wasm 缓存不友好。

修复：**地质节律**——检查降到每 50 tick 一次（abiogenesis 是地质尺度化学事件，5 模拟秒粒度无损），**per-check 概率 ×50 补偿**（采样粒度不变更过程强度，期望触发率不变，触发偏移 ≤50 tick）。实测恢复达标（15.7-13.1 区间，机器负载漂移 ±2ms 属正常）。

### 测试断言修正
`extinct_world_reboots_via_abiogenesis` 原断言「40k 内 events≥2」与饱和门设计矛盾：首事件后 founder 繁殖 → pop≥ABIO_MIN_POP → abiogenesis **正确静默**（繁荣世界不乱冒生命）。设计契约是「一次事件打破吸收态」而非「连续冒生命」。断言改为 events≥1 + 40k 后 lineage 存活。save-roundtrip 测试同步改为 events≥1。

### 诊断数据（dead_world 20261102）
- 候选格 9（wet 分支；mineral 场在 50 tick 时为 0——矿化是 DECO 演化产物，死世界无矿化热泉，只有水缘化学路径）
- 首事件 t=8020，chemo_in=88.7（热泉化学能入账 ✓）
- p=0.5 对照实验：触发链路/spawn/记账全部正常
