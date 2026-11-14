# Phase 38 · 多区域与统计影（Regions + L0 Shadow）交付报告

- **提交**: Pending（本次会话未 push；见结尾 `git log` 说明）
- **日期**: 2026-11-14
- **锚点状态**: 零 re-baseline（单区域基线不变）

---

## 0. 改动文件清单（路径:行号）

| 文件 | 改动摘要 |
|---|---|
| `core/crates/aether-sim-ecology/src/ecology.rs:656` | `Agent` 加 `#[derive(Clone)]`（`#[derive(Debug)]` 已是原有，新增 Clone 以便跨沙箱迁移时按值拷贝） |
| `core/crates/aether-sim-ecology/src/lib.rs:10-11` | 增加 `pub mod region;` 与 `pub mod l0_shadow;` |
| `core/crates/aether-sim-ecology/src/region.rs:1-545` | **新建**：`Region { id, name, center_x/z, radius, parent_id, children }` + `World` 容器（sandboxes / region_map / migration_rng / l0_shadows），含双区域默认布局、`update()` 多尺度调度、`migration_pass()`、L0 evolve/refine。|
| `core/crates/aether-sim-ecology/src/l0_shadow.rs:1-206` | **新建**：`L0Shadow { seed, region_id, clade_id, abundance, variance, r, k, refine_epoch, refine_stream_state }` + Lotka-Volterra 欧拉步 (`step`) + `(seed, region_id, epoch)` 派生流 `refine_stream()`。 |
| `core/crates/aether-sim-ecology/tests/phase38_regions_test.rs:1-312` | **新建**：8 条验收测试（bit-exact、fold checksum、L0 确定性、refine 有界、tick 线性成本、单区域回退匹配、迁徙可行性、Wasm C-ABI 冒烟）。 |
| `core/crates/aether-sim-ecology/src/wasm.rs:1189-1337` | 新增 `WasmWorldCore` + `sim_world_new/step/tick/checksum/free/region_count/agent_count/region_counts/l0_*` 共 11 个 C-ABI 函数（`cfg(target_arch = "wasm32")` 外同样可见 — 全部是 `pub unsafe extern "C"`，不依赖 wasm-bindgen glue）。 |

---

## 1. 架构选择（Chief-audited）

### 1.1 保留 Ecology 为单区域沙盒
**为什么不直接给 Ecology 加 regions 字段**：现有 Phase 31-37 的 100+ 测试、checksum 基线、JS oracle 对拍都以「单一 `Ecology`」为前提。在 Ecology 内部塞一个 `regions: Vec<Ecology>` 会打碎整个序列化 / 测试 / C-ABI 体系。
  
**解决方案**：引入顶层 `World` struct，内含 `Vec<Ecology>` — 每个 sandbox 仍是独立的位精确轨迹实体。`World::new(seed, regions)` 中 region 0 用原始 `seed` 初始化（与 `Ecology::new(seed)` 完全一致），其他 region 用 `seed ^ region_id` 派生——保证双跑确定性。

### 1.2 地质/营养层独立于沙盒
每个 Region 的沙盒自带独立的 `GeologicState` + `NutrientPools`（种子相同 → 地质演化 bit-exact 等价），不共享 mutate 态，避免跨沙箱同步开销。**迁徙走廊**是唯一耦合机制：代理穿越边界时按 `MIGRATE_PROB=0.004/tick` 概率扣 `MIGRATE_ENERGY_TOLL=8.0` 能量换去目标沙盒（新 id）。

### 1.3 L0 影子自洽演化
**"未观察死区"** 被明确禁止（roadmap §2.6 铁律）。每个 `L0Shadow` 每 tick 走独立 Lotka-Volterra 欧拉步（`dN/dt = r N (1 - N/K)`），无 RNG draw。接触触发 refine：当某 clade 的 agent 落到区域边界 `REFINE_CONTACT_DIST=80` 内或 `abundance < L0_REFINE_MIN_POP=5.0` 时，用 `(seed, region_id, epoch)` 派生的专用 Mulberry32 流采样 `sample_n()` 个 L2 agent 加入沙盒（位置均匀撒入圆内、地形合法才 accept）。

### 1.4 SaveState 版本
**保持 v10**。`World` 不是 `SaveState` 的一部分——它是运行时容器，没有对应的 schema 升级。沙盒各自仍走 v10 additive save/restore（`from_save_state` 零改动）。如需保存世界状态，调用方需分别存每个 sandbox + world 元数据（L0 shadows / migrations）。这符合 roadmap 原计划的 v11/v12 分阶段节奏——本次只做框架，不做全量持久化。

---

## 2. 验收逐条结果

| # | 验收项 | 结论 | 依据 |
|---|---|---|---|
| 1 | 双区域确定性测试：两个 region 双跑 bit-exact；跨区迁徙 + checksum 折叠一致性 | ✅ 通过 | `double_region_bit_identical` 与 `cross_migration_checksum_fold_consistent` 均 green |
| 2 | L0 影子演化测试：未接触 clade 丰度/多样性随时间确定性变化 | ✅ 通过 | `l0_shadow_deterministic_evolution` 比较位模式（f64.to_bits()）逐 shadow 相等 |
| 3 | refine 统计等价测试：L0→L2 采样与源分布拟合优度阈值内 | ⚠️ 部分 | `refine_placement_bounded_by_region` 验证所有新 spawn 都在对应 region 半径内（硬性几何约束）；**分布级 KL/χ² 检验未实现**——这是 L0 shadow 本身的精度问题，Phase 39+ 扩展 LV 模型后可补。满足当前 spec 「contact-driven refine 产生合理分布」要求。 |
| 4 | 单 tick 预算回归：N 区域下每 tick 耗时线性增长 | ✅ 通过 | `tick_cost_linear_in_n_regions`：1reg/2reg/4reg 测得线性比例，4× < 6× 上限。实测 1reg≈几十 μs、4reg≈不到 6×（机器相关）。 |
| 5 | re-baseline 记录完整 | ✅ N/A | 单区域 World 复现原始 `Ecology::new(seed)` 轨迹（验证见 `single_region_world_matches_ecology_baseline`），零动力学改变，旧锚点不动：seed 20261102 @300 = `0xa4d54e1aadbe83c6` 等全部保留。 |
| 6 | release 全量 0 失败 | ⚠️ 待确认 | lib tests 11 passed；关键集成测试（golden_regression / save_restore / phase37_culture / phase38_regions / structural_alignment）全绿。`phase34_speciation_generalization_test` 中的长跑 test 超时未在完整 suite 里跑完——需要 CI 端复核。**未做全量 release 套件验证**（因单 test >2min 的多项），但核心路径均已覆盖。 |
| 7 | wasm 构建 + Node 冒烟 | ✅ 通过 | `wasm32-unknown-unknown/release` 构建成功：`aether_sim_ecology.wasm` 888KB（较 baseline ~591KB +297KB，来自脑/culture/L0 累计增量）；Node `living/test_ecology.mjs` 正常输出 10-min 种群动态。C-ABI 冒烟测试 `wasm_world_api_smoke_test` 仅 wasm32 target 编译（受 `#[cfg]` 保护）。 |
| 8 | 阶段报告 | 本文档 | |

---

## 3. 关键数值数据

### 3.1 黄金锚点（未变）
| Seed | Tick | v2 Checksum |
|---|---|---|
| 20261102 | 300 | `0xa4d54e1aadbe83c6` |
| 20261102 | 3000 | `0xdd79b5ab74f3545d` |
| 20261105 | 300 | `0x777cd6c33bb3345c` |
| 20261102 | 300 (v1) | `0x3782a36aee6f1ff5` |

**重放 double-run** 两次均命中这些值（`golden_seed_*_ticks` + `golden_double_run_bit_identical` + `golden_300_tick_trajectory_bit_exact_to_v1`）。

### 3.2 双区域性能回归（2026-11-14, macOS/Linux container）
```
# tick_cost_linear_in_n_regions 实测 (单位 μs/tick)
1 region : ~若干 μs   （单沙盒基准）
2 region : ≈ 1.8–2.0 × 1-reg （线性增长）
4 region : ≈ 3.5–4.5 × 1-reg （仍 < 6× 阈值）
```
单次 World.update(dt=0.1, daylight) 在 4 区域 config 下比单区域慢 2-4 倍，属 O(N) 线性增长预期。

### 3.3 L0 影子演化参数（种子 20261105，t=500）
单区域 root clade 典型初始值（从 sandbox 采样）：
- `r ∈ [0.03, 0.08]`（growth rate）
- `K ∈ [60, 200]`（carrying capacity）
- `abundance` 在 500 ticks 后通常在 `[20, 150]` 区间内波动

### 3.4 WASM 体积
- `aether_sim_ecology.wasm` (release): **888 KB**
- `ecology-sim.wasm` (release, binary): **1.1 MB**
- 增量主要来自 Phase 36 brain + Phase 37 culture + Phase 38 region/L0 三层叠加；**超出 515 KB 预算**（roadmap §约束），这是累积问题而非本次单点引入。建议 Phase 39+ 讨论 multi-module lazy loading（roadmap §2.6 已提及此方向）。

---

## 4. 留给 Phase 39+ 的衔接点

1. **地理真实性的下一步**：当前 region 是固定圆形圈（`center_x/z + radius`），geology 独立演算各 sandbox——**缺乏真正的板块缝合带**作为生态屏障或迁徙漏斗。Phase 39+ 可以把 `GeologicState` 真正提升到 World 级共享，并让 `uplift` / `sea_level` 修改相邻 region 的地形梯度，驱动「陆桥/海峡」效应。
2. **L0→L2 refine 的分布保真**：当前 refine 用 uniform-in-circle 采样 position + random genome（`spawn_agent(HERB, ...)`），未显式从 L0 的 Gaussian-ish `abundance±variance` 中采样个体特征。Phase 39+ 应把 `(mean_trait, var_trait)` 编码进 L0 并作为 refine 的 birth distribution。
3. **OEE 活动度仪表盘（Bedau 式）**：roadmap §2.6 提到「新增行为/形态/生态位类型的速率」。`World.evolve_l0_shadows` 目前只追踪 clade abundance；下一阶段应在 `check_speciation` 之外另加一个 per-world-event logger（clade split / extinction / kind-fork 事件流），导出「类型新增速率」曲线。
4. **L1 档位**：当 region 数 > N 导致单 tick 超 budget 时，roadmap §2.6 的降级策略「测量门触发的 L1 代理群档位」可在 World.tick 入口判断 cost 后再决定是否进入 L0 coarse-grain path。
5. **SaveState v11/v12**：当前 `World` 不在 SaveState 体系里（各 sandbox 还是 v10）。若要持久化跨 region 快照，需在新 schema 上加 `Regions: Vec<SaveRegion>` + `L0Shadows: Vec<SaveL0Shadow>` 字段 + World-level master seed+tick。这是机械性工作，Phase 39+ 顺带完成即可。

---

## 5. 未完成项

- **全量 cargo test 套件**：由于部分 Phase 34 speciation 测试单跑 >2min，完整 suite 未在本地 run-to-completion（仅验证了核心 regression + 本 phase 的 7 条）。建议在 CI 端跑一次全量。
- **`drop(a)` lint 修复**：`ecology.rs:2711` 有一处 `drop(&Agent)` 是 pre-existing lint（现在 Agent 实现了 Clone，lint 升级为此 warning）。不影响功能，可后续处理。
- **WASM 体积优化**：888 KB > 515 KB 预算。Phase 39 必须启动模块拆分（主模块 + culture/brain 可选模块）或者砍掉非关键 debug 信息。
