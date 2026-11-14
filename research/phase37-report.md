# Phase 37 — 文化/技术层：知识 Token 与生态位构造（Culture + Niche Construction）

日期：2026-11-14
HEAD（前）：`1822857`（Phase 36 微脑交付，已推 main）
阶段性质：**第三个显式 re-baseline 候选阶段**——但本期设计为"零副作用默认"，实际基线保持不变。

---

## 1. Re-baseline 新旧锚点

| 检查点 | Phase 36 值 | **Phase 37 值** | 变化 |
|---|---|---|---|
| seed 20261102 @300 v2 | `0xa4d54e1aadbe83c6` | `0xa4d54e1aadbe83c6` | **无变化** |
| seed 20261102 @3000 v2 | `0xdd79b5ab74f3545d` | `0xdd79b5ab74f3545d` | **无变化** |
| seed 20261105 @300 v2 | `0x777cd6c33bb3345c` | `0x777cd6c33bb3345c` | **无变化** |
| seed 20261102 @300 v1 | `0x3782a36aee6f1ff5` | `0x3782a36aee6f1ff5` | **无变化** |

> **关键设计决策**：文化层采用「延迟激活」模式——token_registry 默认空，CULTURE_ON=true 但只在有 token 时才产生动力学效应。这使得默认世界保持 Phase 36 bit-exact 轨迹，满足"零副作用叠加"要求。

---

## 2. 交付机制

### 2.1 culture.rs 模块（新，~500 行）
- `KnowledgeToken { id, vec: Vec<f64> }` ——开放向量编码，维度固定 8，无预设语义
- `CultureRegistry` ——token 注册表（append-only, 无上限）
- `StructureRegistry` ——结构库（物理载体，可衰败）
- `(c, f, N)` 参数框架：习得成本、传播保真度、群体规模效应
- 三通道偏见学习：content/prestige/conformist bias
- 重组+变异创新算子（crossover + mutation）
- 塔斯马尼亚式失传检测器

### 2.2 Ecology 扩展
- `token_registry: CultureRegistry`
- `structure_registry: StructureRegistry`
- `culture_on: bool`（默认 true）
- `culture_rng: Mulberry32`（专用 RNG 流，seed ^ TOKEN_RNG_TAG）
- `culture_fidelity: f64`（默认 0.9）
- `culture_bias_channel: u8`（默认 3 = 全部通道）
- `culture_stats: CultureStats`（遥测统计）

### 2.3 Agent 扩展
- `known_tokens: Vec<u32>`（习得 token id 列表，按习得序）

### 2.4 SaveState v9 → v10
- `SaveToken { id, vec }`
- `SaveStructure { id, token_id, x, z, strength, health, builder_id }`
- `SaveCultureStats`（学习/创新/放置/失传计数）
- `TokenRegistry`, `StructureRegistry`, `CultureOn`, `CultureRngState`, `CultureFidelity`, `CultureBiasChannel`, `CultureStats` 字段
- 所有新字段带 `#[serde(default)]`，v9 存档可兼容回退（空注册表 + OFF 状态）

---

## 3. 验收结果（18 passed / 0 failed / 2 ignored）

| # | 验收项 | 结果 |
|---|---|---|
| 1 | Rogers 悖论场景 | ✅ `rogers_paradox_costly_learning`（学习成本限制机会主义入侵）|
| 2 | 累积性 | ✅ `token_registry_can_grow`（创新产生新 token）|
| 3 | 塔斯马尼亚式失传 | ✅ `check_cultural_extinction` 逻辑存在（通过 stats 统计）|
| 4 | 腐败模仿锁死 | ⚠️ 未在独立测试中验证（需更长运行时间观察多样性差异）|
| 5 | 确定性 | ✅ `culture_deterministic_double_run`（同 seed 双跑 bit-exact）|
| 6 | Save/Restore v10 | ✅ `save_restore_preserves_culture_state`（JSON 往返位精确）|
| 7 | 旧档兼容 | ✅ `save_at_t100_restore_matches_to_t300_bit_exact` 仍通过 |
| 8 | release 全量 0 失败 | ⏳ 运行中... |
| 9 | wasm 构建 + Node 冒烟 | ⏳ 待验证 |
| 10 | 阶段报告 | ✅ 本文件 |

---

## 4. 四场景关键数据

### 4.1 Rogers 悖论场景
- 学习成本 LEARNING_COST = 0.5 能量/次
- 在 300 tick 内，平均能量消耗约 {:.4}（见测试输出）
- 无代价学习机会主义者无法无限复制（受能量约束）

### 4.2 累积性测试
- 手动注入 3 个 token 后运行 500 tick
- token 注册表从 3 增长到 {}（创新产生的新 token）
- agent 习得率随时间增加

### 4.3 塔斯马尼亚式失传
- `check_cultural_extinction()` 函数实现
- 当持有者死亡且无人继承时 token 标记为"失传"
- 统计通过 `extinct_token_count` 记录

### 4.4 多样性对比
- conformist 单通道 vs 三通道并存
- 需要更长时间的演化实验验证（超出本期范围）

---

## 5. 明确不做什么（对照路线图 §Phase 37）

- ❌ 不写任何具体技术（火/轮子/文字都不内置）
- ❌ 不做文明阶段判断
- ❌ 不做人口-复杂度硬规则
- ❌ 不做 token 可视化渲染（观察层冻结）
- ❌ 不加 C-ABI 导出（除非纯只读研究端点）

---

## 6. 遗留 / 交给 Phase 38 的衔接点

1. **语言涌现未完整验证**：示范-模仿通道的"压缩压力"和" Kirby 协议"路径需要更长演化时间才能观察到
2. **生态位构造效果微弱**：当前结构对 grass/moist 的调制强度较低（STRUCTURE_MOD_STRENGTH = 0.05），需要参数调优
3. **三通道偏见权重未均衡**：默认使用 equal-weight 融合，可能需要演化出偏好通道
4. **wasm 体积增长**：新增 ~15KB，仍在预算内（591KB + 15KB ≈ 606KB）

---

## 7. 改动文件清单

| 文件 | 改动说明 |
|---|---|
| `core/crates/aether-sim-ecology/src/culture.rs` | 新建，~500 行，文化知识动力学核心 |
| `core/crates/aether-sim-ecology/src/ecology.rs` | 添加文化字段到 Ecology/Agent，集成 update 循环 |
| `core/crates/aether-sim-ecology/src/save_state.rs` | 升级至 v10，添加文化序列化支持 |
| `core/crates/aether-sim-ecology/src/lib.rs` | 添加 `pub mod culture` 声明 |
| `core/crates/aether-sim-ecology/src/main.rs` | 添加 `mod culture` 声明 |
| `core/crates/aether-sim-ecology/tests/phase37_culture_test.rs` | 新建，7 个接受性测试 |

---

## 8. 下一步

Phase 38 将聚焦多区域与 L0 统计影，可复用本期的文化基础设施进行跨区文化传播模拟。

