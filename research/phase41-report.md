# Phase 41 — 三脑架构（Triune Brain）：本能脑 / 情绪脑 / 理智脑

日期：2026-11-15
HEAD（前）：`d7d75b5`（Phase 40.2 文化激活，第五次 re-baseline）
阶段性质：**第六次显式 re-baseline**。三脑的维护费、rollout 决策费与抑制代价是正式动力学，直接扣能量并进入 `ledger.triune_cost`。

---

## 0. 一句话结论

把 Phase 36 的单层 4 节点微脑升级为 **148 个可演化基因编码的三层系统**：
本能层是 56 个可突变符号的反射矩阵；情绪层是 fear/desire/bond/arousal 连续动力学 + 有限槽位的 state→valence 情绪记忆；理智层是 k 槽工作记忆 + 每动作一套线性世界模型 + 2–4 步 rollout + 抑制闸门 + 元认知置信度。三层每 tick 作为零 RNG 影子决策层运行，仲裁结果被记录；脑规模/规划深度/抑制强度全部按节点数×深度计费并扣真实能量，长程观测到明显的“小脑化”选择压。

---

## 1. 架构

### 1.1 本能脑（Instinct）——脑干-基底核反射弧

- 直接 `sense[8] → action[7]` 线性反射：**56 个突触权重 + 7 个动作偏置 + 增益 + FAP 阈值**，全部是基因。
- 反射方向由基因符号决定，可突变翻转（没有“IF 威胁 THEN 逃跑”的剧本）。
- FAP 发放门：`fap = sigmoid(4·(max_raw − threshold))`，阈值基因控制“胆大/胆小”；norepinephrine 放大增益。
- 零学习、零 RNG、最小计算量 → 最快、最便宜的一层。

### 1.2 情绪脑（Emotion）——杏仁核-海马-伏隔核

连续状态变量（不是 0/1 标签）：

```
fear    += dt·(fear_rise·threat·(1−fear) − fear_decay·fear·(1+ser·gain/2))
desire  += dt·(desire_rise·hunger·(0.3+food)·(1−desire) − desire_decay·desire)
bond    += dt·(bond_rise·social·(1−bond) − bond_decay·bond)
arousal += dt·(arousal_gain·(0.7·threat+0.3·hunger+0.5·novelty)·(1−arousal) − arousal_decay·arousal)
```

- **情绪记忆**：`capacity = gene ∈ [2,8]` 个槽位，键为 `[threat, food, social, fatigue]`；相似度匹配后按 delta rule 更新 valence/strength，空槽优先、否则替换最弱槽。威胁 + 负回报触发 fear-conditioning（负价快速绑定）；记忆提取值反向抬高 fear（`memory_fear_gain`）。
- **情绪→动作**：4×7 = 28 个基因权重。默认 fear→flee、desire→eat、bond→help，但全部可突变，不存在硬编码映射。

### 1.3 理智脑（Reason）——前额叶

- **工作记忆**：`k = gene ∈ [2,6]` 个持久循环槽位，`wm_i ← tanh(decay·wm_i + write·tanh(gain·(bias_i + Σ wm_in_j·sense_j)))`；槽位作为规划状态的上下文（`plan_sense = sense + 0.15·tanh(wm[j%k])`）。
- **世界模型**：每个动作一套线性模型 `next = tanh(W_a·s + b_a)`、`reward = w_r,a·s + b_r,a`（7×81 参数），用实际 transition 的 delta rule 在线学习，学习率受 dopamine 调制。
- **规划**：对 7 个动作各做 2–4 步 rollout，累加贴现奖励，得每个动作的期望回报。
- **抑制控制**：`urge = argmax(fusion(instinct, emotion))`，`plan = argmax(reason)`；`gate = sigmoid(inhibit_gain·(reason[plan]−reason[urge]) − threshold)`，当 gate > 0 时压制 urge、抬升 plan。抑制有能量代价 `inhibit_cost·gate`。
- **元认知**：预测误差 EMA → `confidence = 1/(1+τ·error)`；`calibration = EMA(1−|confidence−accuracy|)`；置信度进入理智层融合权重（低置信时理智权重下降）。

### 1.4 层间融合与神经调质

```
lower = fusion_instinct·instinct + fusion_emotion·emotion
final = lower + fusion_reason·(floor+(1−floor)·confidence)·reason
if gate>0: final[urge] -= strength·gate·(1+lower[urge]); final[plan] += ...
action = argmax(final)
```

- dopamine = `tanh(da_sensitivity·RPE)` 的慢 EMA；norepinephrine = `ne_sensitivity·arousal`；serotonin = `ser_sensitivity·(1−fear)` 的 EMA。
- 兴奋度放大本能增益；5-HT 加速恐惧消退；DA 放大世界模型/情绪记忆学习率。

### 1.5 基因布局（148 个连续 f64，结构基因取整）

| 区段 | 数量 | 含义 |
|---|---|---|
| `G_INSTINCT_W` 0..56 | 56 | 7 动作 × 8 输入的反射权重（符号可突变） |
| `G_INSTINCT_B` 56..63 | 7 | 动作偏置 |
| 63..65 | 2 | 反射增益、FAP 阈值 |
| 65..76 | 11 | 情绪 rise/decay、记忆学习率/容量/恐惧提取增益 |
| `G_EMOTION_W` 76..104 | 28 | 4 情绪 × 7 动作 |
| `G_MEMORY_W` 104..111 | 7 | 情绪记忆 valence → 动作 |
| 111..129 | 18 | WM 输入/偏置/decay/write/gain/槽位数 |
| 129..138 | 9 | rollout 深度、贴现、世界模型/奖励学习率、抑制 4 参数、置信度 τ |
| 138..143 | 5 | DA/NE/5-HT 敏感度与增益 |
| 143..148 | 5 | 三层融合权重 ×3、规划增益、理智置信下限 |

### 1.6 成本模型（智能定价）

```
维护费/秒 = TRIUNE_PER_UNIT_COST × [15(本能) + (4+mem_slots+3)(情绪) + (8+wm_slots+depth×7)(理智)]
决策费/秒 = TRIUNE_PER_ROLLOUT_COST × depth + inhibit_cost_gene × gate
```

- `TRIUNE_PER_UNIT_COST = 0.0002`，`TRIUNE_PER_ROLLOUT_COST = 0.0002`。
- 参考脑 59 units ≈ 0.012/s；理智层占约 56%（33/59 units），符合“理智最贵”。
- 维护费与决策费都进入 `EnergyLedger::triune_cost` 并计入 `total_loss`；扣的是真实 energy。

**成本标定记录（实测）**：`0.0008` / `0.0004` 会把群体层 FEED 盈余压到 `GROUP_REPRO_POOL=10` 以下，使 Phase 39.1 的组繁殖从 t≈1200 推迟到 t≈1800 以后；`0.0002` 保住原组层动力学（t=1200 已出现 2 次组繁殖）。这是本期唯一一次参数标定，记录在此以免被误读为“写死行为”。

---

## 2. 集成与确定性

- `Agent.triune: TriuneBrain`，每 tick 在 `brain_shadow` 中运行（`sample_sense_vector` 后、FSM 决策完成后）。`executed` 传 FSM 当前动作，三脑学习的是真实 transition。
- **零 RNG**：`tick` / `forward` / `observe` / 情绪记忆更新全部纯状态函数；只有出生路径的 `mutate`（基因扰动、WM/记忆槽位 ±1、rollout ±1、世界模型漂移）消耗专用 `brain_learning_rng`（`BRAIN_RNG_TAG` 流）。
- **行为权威不变**：`BRAIN_SWITCH_ENABLED=false` 仍然是 FSM 主控；三脑的仲裁动作、plan、gate 被记录但不写回 FSM。三脑通过**能量成本**与既有 sense 通道进入正式动力学（与 Phase 36 双控制器设计一致）。
- 出生继承：子代克隆父代三脑（基因/情绪记忆/世界模型），再做基因+结构突变，清空瞬时情绪/调质/置信度。
- `fear_events` 统计 fear 上穿 0.6 的次数；`fear_conditioning` 统计威胁-负回报绑定；`planned_overrides` / `inhibitions` 统计理智覆盖冲动的 tick。

---

## 3. 第六次 re-baseline（新旧锚点）

| 检查点 | Phase 40.2（旧） | **Phase 41（新）** |
|---|---|---|
| seed 20261102 @300 v2 | `0xbbee72352ef4468d` | **`0xe82f6b44f4871bf7`** |
| seed 20261102 @3000 v2 | `0x23aca66c0c68ba69` | **`0x1142f4862637a383`** |
| seed 20261105 @300 v2 | `0xfa11ea50ba7ef71a` | **`0x9d060128d2ff9a87`** |
| seed 20261102 @300 v1 | `0xfb8827f6b1b34b5a` | **`0x44b3fda76ab6c427`** |
| group_off @300 v2 | `0x8e59843d4f009045` | **`0xf0d9487bf2497095`** |
| group_off @3000 v2 | `0xa3dd655b64f18333` | **`0xec820a13793f9780`** |
| group_off seed2 @300 v2 | `0x8f981cd0b42a58d7` | **`0x27af4bd0b8bb57c5`** |
| group_off @300 v1 | `0x771a11f1bd8ce316` | **`0x6addfa3f6fff573e`** |

锚点已钉在 `phase31_golden_regression.rs`（旧值保留在文件头注释）、`phase32_open_loci_test.rs`、`phase33_plant_registry_test.rs`、`phase39_1_transition_test.rs`。

---

## 4. 观测统计（seed 20261102）

### 4.1 三脑运行遥测

| t | alive | mean units | upkeep/s | wm | depth | mem | fear 事件 | 恐惧条件化 | 抑制触发 | 理智覆盖 | WM 更新 | rollout 步 | 预测更新 | 高置信 tick | mean fear | mean conf | mean err |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1500 | 506 | 57.9 | 0.0116 | 3.8 | 2.9 | 4.0 | 204 | 13347 | 285586 | 291294 | 1.83M | 9.67M | 476k | 95k | 0.039 | 0.568 | 0.056 |
| 3000 | 607 | 57.7 | 0.0115 | 3.8 | 2.8 | 4.0 | 204 | 13347 | 942019 | 971208 | 5.29M | 27.7M | 1.38M | 437k | 0.013 | 0.617 | 0.044 |
| 4500 | 393 | 57.4 | 0.0115 | 3.8 | 2.8 | 4.0 | 204 | 13347 | 1.52M | 1.58M | 8.24M | 43.0M | 2.15M | 849k | 0.009 | 0.645 | 0.039 |
| 6000 | 181 | 54.8 | 0.0110 | 3.5 | 2.5 | 3.7 | 204 | 13347 | 1.81M | 1.89M | 9.80M | 51.0M | 2.57M | 1.10M | 0.007 | 0.656 | 0.037 |
| 9000 | 135 | 51.2 | 0.0102 | 3.1 | 2.1 | 3.2 | 204 | 13347 | 2.03M | 2.13M | 11.26M | 58.2M | 3.01M | 1.39M | 0.006 | 0.666 | 0.035 |

（1500 tick 之后 fear 事件/条件化计数不再增长：早期 PRED 威胁事件结束后，fear 不再上穿 0.6；max_fear 仍达到 1.0。高置信 tick 占比 20%→46%。）

动作分布（t=3000，7 维 `[forward,left,right,eat,flee,reproduce,help]`）：
`[312795, 104011, 61032, 399435, 128034, 257333, 114798]` — 取食与繁殖占主导，逃跑 9.3%，help 8.3%。

### 4.2 30k tick 长程

| t | alive | mean units | tokens | groups | 抑制触发 | 理智覆盖 | mean conf | mean err |
|---|---|---|---|---|---|---|---|---|
| 5000 | 319 | 56.8 | 9 | 46 | 1.65M | 1.72M | 0.650 | 0.038 |
| 10000 | 140 | 50.5 | 17 | 36 | 2.10M | 2.20M | 0.668 | 0.035 |
| 15000 | 140 | 50.2 | 17 | 43 | 2.44M | 2.56M | 0.682 | 0.033 |
| 20000 | 140 | 50.2 | 17 | 44 | 2.73M | 2.87M | 0.691 | 0.031 |
| 25000 | 139 | 50.2 | 17 | 54 | 3.07M | 3.24M | 0.698 | 0.030 |
| 30000 | 139 | 50.2 | 17 | 55 | 3.51M | 3.71M | 0.702 | 0.030 |

**关键观测：成本选择压有效**——三脑规模从 57.9 units 降到 50.2（WM 3.8→3.1、rollout 2.9→2.1、记忆 4.0→3.2），并在 50.2 附近稳定；置信度随世界模型收敛升到 0.702，预测误差降到 0.030。种群在 t=10000 后稳定于 139-140，未灭绝；文化 registry 17 个 token（种群比 Phase 40.2 小，文化积累速度相应下降）。

---

## 5. 验收逐条

| # | 验收项 | 结果 | 证据 |
|---|---|---|---|
| 1 | 三层独立可测 | ✅ | `instinct_reflex_without_learning`（威胁反射直接响应，零学习）；`fear_accumulates_on_threat_and_decays` + `emotional_memory_conditioning_and_recall`；`planner_needs_multistep_rollout`（depth-2 取即时小奖励，depth-3 看到延迟大奖励） |
| 2 | 不写死 | ✅ | 148 个基因覆盖反射矩阵/情绪映射/融合/尺寸；`bold_and_timid_genotypes_diverge` 同一场景因勇气阈值/反射权重不同而动作不同；代码中无场景 IF-THEN 分支 |
| 3 | 抑制控制有效 | ✅ | `inhibition_overrides_fear_urge`：fear=1 时弱抑制基因型 final=flee，强抑制基因型 plan=eat 且 gate>0.5、final=eat |
| 4 | 元认知/工作记忆真实 | ✅ | `working_memory_updates_and_persists`（输入改变槽位、无输入保留衰减痕迹）；`confidence_tracks_prediction_error`（低误差脑 confidence 更高）；30k 观测 conf 0.702 / err 0.030 |
| 5 | 成本定价 | ✅ | `cost_scales_with_layer_size`（WM/记忆/深度↑ ⇒ units、upkeep、decision_rate 严格↑）；`triune_cost_is_priced_and_ab_testable`（paid ledger>0、free=0、paid 总能量更低、轨迹不同）；30k 规模收缩 |
| 6 | 确定性 | ✅ | `long_run_stays_finite_and_deterministic`、`serde_roundtrip_is_exact`、`save_v12_roundtrip_is_bit_exact`、`pre_v12_archive_restores_with_reference_triune`、golden 双跑 |
| 7 | 第六次 re-baseline | ✅ | §3 表 + 测试注释 |
| 8 | release 全量 0 失败 | ✅ | 204 passed / 0 failed / 7 ignored（见 §6） |
| 9 | wasm 构建 + Node 冒烟 | ✅ | 751,069 B；`WASM VIEWER SMOKE OK`（见 §7） |
| 10 | 阶段报告 | ✅ | 本文件 |

### 5.1 关于存档版本号

任务书写“存档 v13 / v12 兼容”。实际 HEAD 的 `SAVE_STATE_VERSION = 11`（Phase 40.1/40.2 均为加法字段、未 bump）。本期按序推进为 **v12**：
- `SaveAgent.triune`（紧凑 JSON 字符串，同 `SavedClades` 手法）
- `SaveState.TriuneOn` / `TriuneUpkeepOn`（`#[serde(default = true)]`）
- v11 及更早存档：缺 `Triune` → 从 `kind + id` 重建参考三脑；测试 `pre_v12_archive_restores_with_reference_triune` 与既有 `legacy_v10_archive_loads_as_solitary_world` 覆盖。

---

## 6. 全量测试

`cargo test --release --workspace`：**204 passed / 0 failed / 7 ignored**（最终代码状态）。其中 `aether-brain` 18 项（含 10 项三脑单测）、`aether-sim-ecology` 的 24 个测试二进制全部通过；新增 `phase41_triune_test` 5 项验收 + 3 个 ignored 探针。基线 128 passed 保持全绿。

受 re-baseline 影响的既有断言（均记录原因）：
- `phase31_golden_regression`：四个锚点更新。
- `phase32_open_loci_test` / `phase33_plant_registry_test`：@300 v2 锚点更新。
- `phase39_1_transition_test`：group_off 三锚点 + v1 更新；v10 归档测试同步剥离 Triune 字段。
- `structural_alignment::scav_omn...`：SCAV 8（旧 9）、OMN 12→13→14（旧 12→13→14→16）。

---

## 7. wasm 与 Node 冒烟

- 构建：`cargo build --profile wasm-release --target wasm32-unknown-unknown -p aether-sim-ecology`
- 产物：`living/ecology-sim.wasm` / `living/ecology-sim-v2.wasm`，**751,069 B**（Phase 36 为 591,325 B；+160 KB 主要来自三脑状态的 serde 序列化代码与 567 参数世界模型）。
- Node 冒烟：`node living/test_wasm_viewer.mjs` → **`WASM VIEWER SMOKE OK`**；247 agents，warmup 300 tick 1871 ms，60 帧 sim+export 平均 **14.49 ms/frame**（<16 ms 预算，三脑计算已计入）。

---

## 8. 留给 Phase 42 的衔接点

1. **行为权威仍未交给三脑**（`BRAIN_SWITCH_ENABLED=false`）：本期三脑的仲裁动作只记录不驱动。下一步需要把 7 动作映射回 FSM 状态（flee/eat/approach/…），并在高置信/高唤醒条件下开闸；这会带来一次行为级 re-baseline。
2. **世界模型只学习 FSM 执行过的动作**：影子模式下另外 6 个动作的模型保持零初始化，规划质量受限。Phase 42 可加经验回放/反事实更新（专用 RNG 不必要，仍零 RNG）。
3. **抑制闸门偏敏感**：30k 内约 67% tick gate>0.5，理智覆盖率 70%。因为不驱动行为所以无直接后果；开闸前应校准 `inhibit_threshold`/置信度下限，避免“过度理性”抹平情绪反应。
4. **情绪记忆/世界模型为亲代遗产**（Lamarckian）：出生清空瞬时情绪但保留记忆与世界模型。是否改为基因门控的“先天恐惧”需要 42 决定。
5. **生态稳态变小**：seed 20261102 t=10000 后稳定 139-140，低于 Phase 40.2 的 311-680；文化 token 17 vs 792@100k。若要保留大种群/文化积累，需要重新标定维护费或让 `TROPHIC_EFF` 随脑成本补偿（会再 re-baseline）。
6. **感知通道仍 8 维、观察层冻结**（按要求未做）。
7. **可视化**：三脑只有聚合/个体统计字段，未进 render packet；观察层留给后续阶段。

---

## 9. 改动文件

- `core/crates/aether-brain/src/triune.rs`（新增，三层实现 + 10 单测）
- `core/crates/aether-brain/src/lib.rs`（`pub mod triune`）
- `core/crates/aether-brain/Cargo.toml`（serde / serde_json dev-dep）
- `core/crates/aether-sim-ecology/src/ecology.rs`（Agent.triune、Ecology 开关/遥测、`initial_triune`、`brain_shadow` 集成、出生/繁殖突变、账本、存档接线）
- `core/crates/aether-sim-ecology/src/save_state.rs`（v12 schema + v11 兼容）
- `core/crates/aether-sim-ecology/tests/phase41_triune_test.rs`（新增 5 项 + 3 个 ignored 探针）
- 既有测试：phase31/32/33/39/structural_alignment 锚点与计数更新
