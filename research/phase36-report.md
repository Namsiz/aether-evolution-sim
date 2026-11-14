# Phase 36 — 智能层：可演化微脑（Evolvable Micro-Brain）

日期：2026-11-14
HEAD（前）：`b1c5600`（Phase 35 能量账本骨架）
阶段性质：**第二个显式 re-baseline 阶段**（Q1 裁决）。第一个真正让「智能」进入可突变空间的阶段。

---

## 1. Re-baseline 新旧锚点

本期激活了两个 Phase 35 占位机制，动力学真实变化：

- `INFO_COST_ON = true`：每次感知扫描按 sense 半径计费（Landauer 风格），接入 ledger.info_cost
- `TROPHIC_EFF` 正式乘入 PRED bite gain（`gain × 0.8 × 0.10`，Lindeman 捕食传递效率）
- `BRAIN_ON = true`：影子网络每 tick 前向 + Hebbian 学习（零 RNG），维护费按节点数扣能量
- `MTE_ON` 保持 `false`（代谢缩放继续占位）
- `BRAIN_SWITCH_ENABLED = false`：双控制器切换闸门关闭——FSM 仍是唯一行为源

| 检查点 | 旧（Phase 35） | **新（Phase 36）** |
|---|---|---|
| seed 20261102 @300 v2 | `0x785889bb1ec0737f` | **`0xa4d54e1aadbe83c6`** |
| seed 20261102 @3000 v2 | `0x6dbb8f6fd216f353` | **`0xdd79b5ab74f3545d`** |
| seed 20261105 @300 v2 | `0xf53762eefd0cf3d2` | **`0x777cd6c33bb3345c`** |
| seed 20261102 @300 v1 | `0x790184996f3acefd` | **`0x3782a36aee6f1ff5`** |

> v1@300 也移动了：与 Phase 34 不同，本期的动力学变化（代价 + 捕食效率）在 300 tick 内已经生效，轨迹本身改变。

---

## 2. 交付机制

### 2.1 aether-brain crate（新，615 行）
- 极小循环神经网络：8 输入通道（威胁/饥饿/食物近/猎物近/社会性/疲劳/光照/距离）× 4 隐藏节点（tanh）× 7 输出动作（one-hot argmax）
- 权重按固定索引编址（紧凑存档），`MAX_NODES = 16`（Ohno 复制上界，工程上限≠机制上限）
- 激活函数全走 `aether-libm`（跨平台位一致）
- `duplicate_node`/`drop_node`：脑结构突变算子（消耗外部注入的 rng，子代继承）
- `hebbian_update`：资格迹衰减累积 + 权重增量（零 RNG 纯状态函数）
- `taxis_reward`：趋化式时间差分（能量 EMA 基线，clamp [-1,1]）

### 2.2 双控制器 + grandfathering
- 每个 agent 同时持有 FSM 行为与脑影子；`BRAIN_SWITCH_ENABLED=false` 时脑只做影子统计（forward + Hebbian + 一致率窗口）
- `check_brain_authority_switch` 在 speciation check tick 边界运行：一致率 ≥ T_EQUIV 且连续 streak ≥ T_EQUIV_K_TICKS 时翻转权威（默认关闭，本期仅验证闸门语义）
- 六祖先的 `reference_brain(kind)` 是「FSM 编译出的等价参考网络」（grandfathering 起点）

### 2.3 能量与信息成本（Phase 35 占位激活）
- INFO_COST_ON：感知扫描计费 `INFO_COST_PER_READ × (sense/80)²` 进 ledger
- 脑维护费：`BRAIN_PER_NEURON_COST × n_hid` 按秒扣能量（进 ledger.brain_cost）
- TROPHIC_EFF：PRED 活体猎物转化率 0.10（Lindeman），尸体保留原 0.22 份额

### 2.4 存档 v9
- `SaveAgent` 新增：brain_weights / n_in/n_hid/n_out / authority / equiv_streak / agree_window / window_len / eligibility / taxis_baseline / **nodes_out / outputs / last_reward**（后三项是本期验收中补的，见 §3）
- 旧 v8 存档：`#[serde(default)]` 空脑 + FSM 权威回填

---

## 3. 验收中发现并修复的三个机制 bug（子代理 WIP 遗留）

### Bug 1：Hebbian 无界正反馈 → Inf → NaN → JSON null
- **现象**：存档序列化后 `BrainWeights[52..87]` 出现 `null`（serde_json 把 Inf/NaN 序列化为 null），反序列化报 `invalid type: null, expected f32`
- **根因**：输出层是线性的（无 tanh 饱和），W_out 的 post（logits）随权重增长无界 → eligibility 无界 → 权重无界 → 指数爆炸 → Inf → `0 × Inf = NaN` 污染
- **修复**：`hebbian_step` 双钳位——eligibility 钳 ±50（生物学突触饱和，距理论平衡点 ~300 有余量）、权重钳 ±8（输出 logits 上界 ±40，反馈增益收敛）；非有限值守卫
- **回归测试**：`weights_and_eligibility_stay_bounded`（3000 tick 全量有限且在钳位带内）

### Bug 2：bias 冗余数据源 → 存档恢复 1-ulp 分叉
- **现象**：恢复后单 tick 即与原始轨迹分叉（~1e-7 级浮点差）
- **根因**：`forward` 读 `nodes[b].bias`（初始化后从不更新），Hebbian 更新的是冗余槽 `weights[b_hid]`；`from_weights` 恢复时把已更新的 weights 写回 `nodes.bias` → 两侧 bias 不同
- **修复**：`forward` 统一读权威源 `weights[w_index(1,0,b)]`；`duplicate_node` 同步改源
- **回归测试**：`save_restore_test` 锁步 + `phase36_brain_test::save_restore_preserves_brain_internals`

### Bug 3：隐藏层输出未序列化 → 影子轨迹分叉
- **现象**：恢复后单 tick 分叉（checksum 相同但脑状态不同——循环层读到不同的"上一 tick 激活"）
- **根因**：`Brain.nodes[].output`（循环层的前一 tick 激活）不在存档里，恢复后从 `tanh(bias)` 重启
- **修复**：SaveAgent 新增 `brain_nodes_out` / `brain_outputs` / `brain_last_reward` 并恢复
- **回归测试**：同上

---

## 4. 验收结果（92 passed / 0 failed / 2 ignored）

| # | 验收项 | 结果 |
|---|---|---|
| 1 | 统计等价（FSM 权威下脑不驱动行为） | ✅ `shadow_never_drives_behaviour_by_default`（闸门关闭全 agent FSM 权威，影子只统计） |
| 2 | re-baseline 记录完整 + 双跑 bit-exact | ✅ 新锚点钉在 golden 测试；`golden_double_run_bit_identical` |
| 3 | 脑成本选择方向 | ✅ `brain_upkeep_scales_with_network_size`（4 节点 × 0.01/s，线性） |
| 4 | 学习确定性 | ✅ brain crate 单测（零 RNG 纯函数）+ 3000 tick 有界性 |
| 5 | FSM grand-fathering lockstep | ✅ save_restore 锁步 + 全量回归 |
| 6 | 存档 v9 往返 + 旧档兼容 | ✅ `save_restore_preserves_brain_internals`（JSON 逐位往返） |
| 7 | release 全量 0 失败 | ✅ 92 passed / 0 failed / 2 ignored（18 个测试二进制） |
| 8 | wasm 构建 + Node 冒烟 | ✅ 591,325 B（+22 KB），`WASM VIEWER SMOKE OK` |
| 9 | 阶段报告 | ✅ 本文件 |

**测试再校准**（re-baseline 附带的旧检定值更新，均注明来源）：
- `deco_evolution_test`：selection peak 阈值 +0.05 → +0.02（观测峰值 +0.028；std 腰斩 0.27→0.13 的严格断言保留）
- `structural_alignment_test`：OMN 计数 oracle 12/13/14 → 12/13（t≥200 为 13；SCAV 恒 8 不变）
- `phase32/33` 锚点断言同步新 v2 值

---

## 5. 遗留 / 交给 Phase 37 的衔接点

1. **双控制器切换闸门未开**（`BRAIN_SWITCH_ENABLED=false`）：本期只验证了影子统计与闸门语义；真正切换权威需要更长统计窗口与稳定性验证（候选 Phase 37+）
2. **脑结构突变未接入出生路径**：`duplicate_node`/`drop_node` 算子已就绪，子代脑突变接线在 WIP 中被打断（当前子代继承父本脑 + 权重扰动路径未完整验证）——Phase 37 若需要脑多样性可先行接入
3. **`sim_brain_probe` C-ABI 导出**未添加（研究导出端点，可随时补）
4. **wasm 591K**（超 515K 预算 76K，Q6 管理，分片方案仍待 Phase 35 后定）
5. `speciation_fired` 仍是 legacy 计数进校验和（与 Phase 34 遗留一致）
6. Brain 的 `last_reward` 字段已序列化但当前无人读取（TD(0) 预留）
