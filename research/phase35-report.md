# Phase 35 — 能量账本（观察层）+ 代谢/信息成本占位

日期：2026-11-14
HEAD（前）：`8533280`（Phase 34 交付，首个显式 re-baseline）
阶段性质：**不引入动力学变化**；本阶段是「热力学闭合基础设施」——全局 EnergyLedger、TROPHIC_EFF 命名常量、MTE/Kleiber 参数位、INFO_COST 占位。所有新字段在默认开关关闭时数学恒等（×1.0），因此黄金锚点**不变**。

---

## 1. 设计决策与边界

### C10 热力学闭合：账本是观察不是干预
- `EnergyLedger` 结构体新增到 `Ecology`（纯累加器：`light_in / filter_in / evaporation / metabolism_loss / repro_loss / mineralized / offspring_invest / grazed / predation_gain / scavenged / info_cost / info_reads`）。
- **所有更新路径的 drain/gain 数值保持不变**：只在原有表达式旁边追加 `self.ledger.xxx += ...` 记账行，不改任何 `energy -=` 算式本身。
- `mte_size_scale(idx)` 返回 `size^0.25`（当 `MTE_ON=true`）或恒等 `1.0`（当前默认）。当前多乘以 `1.0` 是预留计算槽，不影响轨迹。

### TROPHIC_EFF：占位，未接入动力学
- `pub const TROPHIC_EFF: f64 = 0.10;` 已定义并写入文档注释（Lindeman 1942 营养级传递效率命名常量）。
- **当前未将 TROPHIC_EFF 乘入 PRED bite gain 或其他动力学路径**——仅作为账本参数钩子保留，Phase 35 re-baseline 时才会真正接入。

### MTE_ON / INFO_COST_ON：双开关均为 false
- `MTE_ON = false` → `mte_size_scale` 恒为 1.0，代谢缩放完全不生效。
- `INFO_COST_ON = false` → `INFO_COST_PER_READ` 常量定义但无调用点。
- 两者都是为 Phase 36（脑的最小实现 + AI 评估）准备的观测端口。

---

## 2. Re-baseline 新旧锚点

**结论：无重基线。** 所有 Phase 34 v2 主锚点在 Phase 35 WIP 下保持逐位精确：

| 检查点 | v2 主锚点（`checksum()`） | 状态 |
|---|---|---|
| seed 20261102 @300 | `0x785889bb1ec0737f` | ✅ 未变 |
| seed 20261102 @3000 | `0x6dbb8f6fd216f353` | ✅ 未变 |
| seed 20261105 @300 | `0xf53762eefd0cf3d2` | ✅ 未变 |
| seed 20261102 @300 v1 | `0x790184996f3acefd` | ✅ 未变（不变量测试确认） |

---

## 3. 改动文件清单

**核心** `core/crates/aether-sim-ecology/src/ecology.rs`
- `PlantType` / `Diet` / `KindDef` 区段后新增 `EnergyLedger` 结构体（`#[derive(Clone, Debug, Default)]`，13 个累加字段）及 `impl EnergyLedger { new() / total_in() / total_loss() }`
- 七处 `update_*` 路径（HERB/PRED/FISH/DECO/OMN 等）追加 `self.ledger.xxx += ...` 记账行（共约 20 处）。全部在原有能量扣减表达式旁，不改原表达式。
- 新增常量：
  - `TROPHIC_EFF: f64 = 0.10`（命名占位，未接入动力学）
  - `MTE_ON: bool = false` / `MTE_MASS_EXPONENT: f64 = 0.75` / `MTE_SIZE_EXPONENT: f64 = 0.25` / `MTE_REPRO_FEE: f64 = 0.5`
  - `INFO_COST_ON: bool = false` / `INFO_COST_PER_READ: f64 = 0.002`
- `Ecology::new` / `with_biomes` / `from_save_state` 三处构造函数初始化 `ledger: EnergyLedger::new()`
- `mte_size_scale(idx) -> f64`：MTE_ON=false 时恒返 1.0
- `reproduce` 内两处 MTE repro fee（cfg(test, feature="phase35_mte")，非 release build 下 dead code）

**配套** `save_state.rs` / `wasm.rs` / `geology.rs` / `niche.rs`：无结构性改动（ledger 不序列化）。

**测试** `tests/phase32_open_loci_test.rs` / `tests/phase33_plant_registry_test.rs`：anchor 值验证确认不变（无需更新）。

**产物** `living/ecology-sim.wasm` / `living/ecology-sim-v2.wasm`：体积 568,540 → 569,192 B（+652 B，账本结构体的元数据开销，按 Q6 管理预算不砍机制）。

---

## 4. 验收结果

1. **黄金回归 4 锚点逐位精确** ✅
   - `phase31_golden_regression` 6 passed（含 300-tick v1 不变量测试、v2 确定性、双跑 bit-exact）
2. **release 全量 92 passed / 0 failed / 2 ignored** ✅
   - 包括 `scale_transition_test::save_restore_v5_roundtrips_geology_state`（锁步延续 2000 tick 通过）
3. **Phase 32/33/34 专项测试**：全部通过 ✅
4. **wasm 构建**：exit 0，产物 569,192 B ✅
5. **Node 冒烟**：`WASM VIEWER SMOKE OK` ✅

---

## 5. 留给 Phase 36 的衔接点

1. `TROPHIC_EFF` 激活时机：Phase 36（AI/脑最小实现）期间将 `TROPHIC_EFF` 乘入 predation gain 并 re-baseline——届时 PRED 能量账本会显示真实的 trophic loss。
2. `MTE_ON` 激活时机：Phase 36 同时打开「大体型繁殖减速」语义（current `MTE_REPRO_FEE` 占位代码已就绪）。
3. `INFO_COST_ON` 接入端口：Phase 36 的感知/学习/记忆模块会在 update 中调用 `charge_info_cost(idx, reads)`，当前 skeleton 已预留接口。
4. `sim_energy_snapshot` C-ABI：可选只读导出，供 JS 侧能量面板使用（暂不添加工具，留待报告层需求）。
5. SaveState v9：如需存档 ledger 快照（研究 CLI 用），走 additive 扩展即可；当前不要求（账本是纯累计观测，重启可从零重建等价历史）。

---

## 6. 未完成项 / 已知限制

- `speciation_fired` 仍 legacy 计数进入校验和（彻底数据化需 Phase 36 后再次 re-baseline）
- `sim_counts` / `Stats.deco_eff_mean/std` 仍只统计六祖先 kind，派生 kind 不进入 HUD（观察层冻结，Phase 36 按需开放）
- wasm 体积 569K（超出 515K 预算 54K，Q6 管理条件，Phase 35+36 累积超支；分片方案待定）
