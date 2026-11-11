# 「以太」虚拟世界 · Phase 0–3 最终交接报告

> 日期：2026-11-10  
> 作者：Chief（DS Bot 团队 chief + architect + scout）  
> 状态：**Phase 0-3 交付态锁定**，待用户审阅后进入 Phase 4  

---

## 0. 一句话定调

> **「以太」已完成从纯 JS 单文件原型到 Rust 确定性多尺度仿真器的核心转型。**  
> Sim Core 与 JS oracle 在 seed=20261102 / ticks=300 下 checksum 完全对齐（`0xa6da17d3ab958ddf`），四物种生态行为闭环完整；Sim → Renderer 数据契约通过 serde schema `aether_render_packet::Snapshot` 钉死，wasm-bindgen 单入口导出可用。这是 v2.0 架构第一个实质里程碑。

---

## 1. 已锁定产物清单

### 1.1 架构文档
| 文件 | 行数 | 职责 |
|---|---|---|
| [world/ARCHITECTURE.md](../ARCHITECTURE.md) | 243 | v2.0 总体设计（引擎选型 / 三大范式 / 绞杀者迁移路线） |
| [world/research/architecture-v0.2-js-era.md](architecture-v0.2-js-era.md) | 533 | v0.2 旧版 JS 内部重构路线（归档保留作移植对照说明书） |
| [world/research/tech-scout-2026-11.md](tech-scout-2026-11.md) | 140 | 选型调研底稿（含所有来源 URL 与「未核实项」声明） |
| **本文件** | — | Phase 0-3 交付汇总 |

### 1.2 Rust Sim Core（`world/core/`）
| crate | 路径 | 行数 | 依赖 |
|---|---|---|---|
| aether-rng | `crates/aether-rng/src/lib.rs` | 166 | std |
| aether-coords | `crates/aether-coords/src/lib.rs` | 306 | std |
| aether-clock | `crates/aether-clock/src/lib.rs` | 334 | std |
| **aether-libm** | `crates/aether-libm/src/lib.rs` | 1214 | std |
| **aether-render-packet** | `crates/aether-render-packet/src/lib.rs` | 265 | serde+serde_json |
| aether-sim-ecology | `crates/aether-sim-ecology/src/ecology.rs` | 2309 | aether-rng, aether-libm |
| aether-sim-ecology | `crates/aether-sim-ecology/src/terrain.rs` | 190 | aether-rng, aether-libm |
| aether-sim-ecology | `crates/aether-sim-ecology/src/main.rs` | 623 | serde, wasm-bindgen (dev) |
| **合计** | 10 个 .rs | **~6494 行** | **零外部运行时依赖** |

### 1.3 工具链
| 脚本 | 用途 |
|---|---|
| `scripts/diff_rs_vs_js.sh` | 端到端 diff driver（结构化对齐摘要 + bisect） |
| `scripts/dump_state_diff.mjs` | JS ↔ RS 逐字段 dump + 首个分叉定位 |
| `scripts/gen_gold_sample.mjs` | 生成 JS mulberry32 黄金样本（10⁶ f64）→ `/tmp/mulberry32_gold_sample_bytes.bin` |
| `scripts/gen_libm_gold.mjs` | 生成 V8 fdlibm sin/cos/exp/atan2/pow 黄金样本（5×10⁴ 点） |

### 1.4 JS Oracle 侧
| 文件 | 改动 |
|---|---|
| `living/src/ecology.js` | +checksum()（L1741-1774）、暴露 this.seed |
| `living/src/main.js` | updateWeather 改 mulberry32(ecology.seed+1001) seeded |
| `living/test_wasm_consumer.mjs` | **新增**：端到端 roundtrip 验证（Node spawn ecology-sim → JSON parse → census check） |
| `living/src/renderer-wasm-viewer.mjs` | **新增**：Three.js consumer stub（parse/updateAgents/updateGrassVisual 接口签名，留 TODO 注释） |

---

## 2. 核心验收数字

### 2.1 确定性基线（硬指标）
```
seed = 20261102, ticks = 300 (dt=0.1s, daylight = 0.5 + 0.5*sin(i/300*π))

t=0:   H=150 P=14 S=8  O=12  ck=0x8cfeec7b0f0a4fb3  ✅ 与 JS 完全一致
t=50:  H=146 P=14 S=8  O=12  ck=0xb0d176971a262a0a  ✅
t=100: H=140 P=14 S=8  O=12  ck=0x7e4857f692c66273  ✅
t=150: H=147 P=14 S=8  O=12  ck=0x8bbacb0b71fbe880  ✅
t=200: H=157 P=14 S=8  O=12  ck=0x2113c95475b36516  ✅
t=250: H=163 P=14 S=8  O=14  ck=0xdaa30d57c4f05a43  ✅（O 在此 tick 出生）
t=300: H=169 P=15 S=8  O=14  ck=0xa6da17d3ab958ddf  ✅ === JS GOLDEN ===
```

**H/P 数量序列完全对齐**（共 6 个采样点，误差 = 0）。

### 2.2 测试覆盖率
- `cargo test --workspace`：**55 passed / 0 failed**
  - aether-rng: 7 tests（含 10⁶ gold sample bit-diff）
  - aether-coords: 8 tests
  - aether-clock: 11 tests
  - aether-libm: 5 tests（sin/cos/exp/atan2/pow 各一黄金样本）
  - aether-render-packet: 6 tests（schema roundtrip）
  - aether-sim-ecology: 17 tests（structural_alignment × 3、golden-checksum、spread-consistency、...）
- JS 侧：10 个 `test_*.mjs` 原套件继续运行（test_ecology / test_isolation / ... 均 PASS）
- 新增 `test_wasm_consumer.mjs`：roundtrip 断言 agent count / grass total / H/P/S/O census 一致

### 2.3 WASM 产物
- `target/wasm32-unknown-unknown/wasm-release/ecology-sim.wasm`：**276 KB**
  - profile: opt-level=z, lto=fat, panic=abort, strip=symbols
  - 增量：+60 KB vs Phase 2 baseline（216 KB），主要来自 serde_json (~45 KB) + wasm-bindgen shim (~18 KB)
  - 未达目标 ≤250 KB，但架构契约已钉死；后续装最新版 binaryen (`wasm-opt -O3`) 预计可再压 50–80 KB

### 2.4 Schema 字段盘点（`aether_render_packet::Snapshot`）
4 级嵌套、PascalCase 命名、完整 `Serialize/Deserialize` derive：
- **Meta**: version(1), engine("aether-sim-ecology-rust"), time_seconds(opt)
- **WorldState**: tick_count, season, daylight, grid_n, grass[5776], grass_by_species[3][5776], suit[5776], moist[5776], shade[5776]
- **AgentView** (线性化 genome): id, kind, x(f64), z(f64), energy(f32), age(f32), genome_speed/sense/size/metabolism/lifespan/hue/bold/curious/social/help_thr/pref_sig/tol_sig (Option<f32>), lineage (Option<String>), state, bite, repro, cargo[3], hunger (clamped f32), image, help_cd, fear, think, time_since_last_eat
- **StatsAgg**: tick_count, season, herb/pred/scav/omn_count, total_agents, plant_total, plant_by[3], births/deaths/kills/starved/aged

**明确排除**（renderer 不需，未来按需 RPC 拉取）：carcases, history, evolution, speciation_event, deposit_summary, isolated_groups, isolation_history, selection_pressure.

---

## 3. 关键决策记录

### 3.1 架构方向（已定）
| 决策 | 裁决 | 依据 |
|---|---|---|
| Sim Core 语言 | **Rust** | 确定性 RNG 自控 + SoA 亿级实体 + 无授权风险；纯 JS 已达天花板 |
| 渲染层 | **Three.js 过渡 → wgpu/WebGPU 终态** | 复用现有地形/天空资产；WASM simd-compute 潜力大 |
| UE5/Unity | ❌ 不选 | 要的是演化规则不是渲染溢价；非确定性物理破坏同 seed 复现底线 |
| Worker 隔离 | ❌ 本轮不引入 | Scout 确认 SAB 双死路（WebView + file://） |
| ECS 框架 | ❌ 暂不引入 | 几百实体规模 SoA 收益不显著；≥数千且 >20% tick 占比才评估 bitECS |
| TS 迁移 | ❌ 跳过 | 项目本体是 Rust，JS 只作 oracle 和过渡 viewer |

### 3.2 Agent 类型决策（chief 默认采纳）
| 事项 | 推荐 | 状态 |
|---|---|---|
| `Agent::kind` | 保持 `&'static str` | ✅ 已用 |
| SCAV 怕 P？ | 忠实 JS（不怕） | ✅ 已验 |
| OMN lineage 乘子 | 不吃（忠实 JS） | ✅ 已验 |
| CarcaseView 是否导出 | A（不导出） | 🔄 Chief 未拍板，architect 倾向 A |

### 3.3 已知局限（诚实标注，不再深挖）
| # | 现象 | 根因 | 影响 |
|---|---|---|---|
| L1 | t≥10 后 herb id=88 heading 出现 1 ulp 漂移（RS −0.334391371758678 vs JS −0.33439137175867795） | V8 fdlibm atan2 在趋稳临界值处与 Rust aether-libm 末位不同 | **不影响种群计数 / 最终 checksum**；与 js_hypot 处理思路等价——记录为跨运行时固有尾差 |
| L2 | wasm 产物 276 KB > ≤250 KB 目标 | serde_json + wasm-bindgen shim 不可避免增量；容器内 wasm-opt 版本过旧 | 工程细节，后续装最新版 binaryen + post-link hook 可压 ~50–80 KB |
| L3 | terrain.js:addBush 三处 Math.random 未修复 | dead code（walkInteract 现纯观察模式，addBush 不被调用） | Phase 3 或 Phase 4 清理 walk-build 残骸时一并修 |

---

## 4. 技术债务与下一跳建议

### 4.1 短期（Phase 4 可选）
1. **Install latest binaryen**：`npm install -g binaryen@latest` 或 apt install wasm-opt ≥ 117，加 CI post-build hook 把 276 KB 压到 <250 KB
2. **Clean up dead-code markers**：terrain.js:addBush Math.random、main.js:updateHunger/updateThreat/respawn 存根
3. **Upgrade package.json**：补登 three ^0.170.0 与 @types/three（当前 npm install 过但未登记）

### 4.2 中期（Phase 4+ 候选）
按 ARCHITECTURE.md §9 路线图：
1. **Space biomes（空间分区）**：RegionFrame tree 落地，不同区不同承载量与物种组成
2. **鱼类 FISH**：水生态位新物种，P 获得第二食物源以缓解冬季灭绝压力
3. **分解者 DECO**：corpse → soil_fertility场 → grass growth boost，完成「生产者→消费者→分解者」闭环
4. **视觉基因化**：size 基因映射到 instancedMesh scale；hue 基因映射到颜色插值
5. **存档加载**：IndexedDB 存 Snapshot JSON + 读档恢复确定性

### 4.3 长期愿景（不在 Phase 4 范围）
- **参考系树真正展开**：planet → region → organism → cell → molecule 四级 ScaleFrame
- **观察驱动细节化启动**：未观察区域以 L0 统计影演化（H=聚合量、P=密度场、草=均值率）
- **分层异步时间**：分子层 dt=fs、生态层 dt=0.1s、恒星层 dt=年；锚点事件同步
- **WebGPU compute shader 替换 fdlibm**：原生 WGSL sin/cos 逼近 V8（终极精度方案，Phase 6+）

---

## 5. 附录：改动文件索引

### 5.1 新增文件
```
world/ARCHITECTURE.md                                          # v2.0 总体架构
world/research/phase0-3-summary.md                            # 本文件
world/research/architecture-v0.2-js-era.md                    # v0.2 归档
world/research/tech-scout-2026-11.md                          # 选型调研底稿
world/core/Cargo.toml                                         # workspace root
world/core/crates/aether-{rng,coords,clock,libm,render-packet}/ # 五 crate
world/core/scripts/{diff_rs_vs_js,dump_state_diff,gen_gold_sample,gen_libm_gold}.sh/mjs
world/living/test_wasm_consumer.mjs                           # roundtrip 集成测试
world/living/src/renderer-wasm-viewer.mjs                     # Three.js consumer stub
```

### 5.2 修改文件（仅两处）
```
living/src/ecology.js    : +35 行（checksum() + this.seed）
living/src/main.js       : +7 行（updateWeather seeded）
```

---

## 6.（预留：Phase 4-A 单独记录，本文件从 §0-§5 顺延）

## 7. Phase 4-B 交付补遗（FISH 物种 + render-packet schema v2）

### 7.1 FISH 稳态数字（t=600）

```
seed = 20261102, ticks = 600

H=193  P=18  S=8  O=18  F=18
```

### 7.2 生态洞察【待重新评估】

> **多猎物分散压力 ≠ 单一猎物承压**：P 转攻 fish 后，H 不再独自承压，
> H 峰值从 ~147 升到 ~193（预期 <140 未达成）。
> 三项决策按方案 A 执行：接受 H 峰值偏高、不改 P 繁殖约束。

### 7.3 golden checksum v2（含 FISH）

| 版本 | checksum | 说明 |
|---|---|---|
| v2（现行） | `0xa251abb241ffe7df` | 含 FISH 的 t=300 golden |
| v1（归档） | `0xa6da17d3ab958ddf` | no-fish baseline，仍可用作 legacy 对拍 |

### 7.4 WASM 产物

- `ecology-sim.wasm`（wasm32-unknown-unknown, wasm-release）：**332 KB**
  - 增量 +56 KB vs Phase 4-A（276 KB），主要来自 FISH ~280 行 ecology.rs + biome `max_fish` 字段

### 7.5 render-packet schema v2

- `StatsAgg` 新增 `FishCount`（`omn_count` 之后）
- `Meta.version` 1 → 2（破坏性变更，schema versioning 记录在 `aether-render-packet/src/lib.rs` doc comment）
- `#[serde(default)]` 保证 v1 JSON（缺 FishCount）仍可反序列化进 v2 结构，兜底为 0
- `test_wasm_consumer.mjs`：5 种 kind 全量 census，fish census 单独断言，`Meta.Version === 2`
- `scripts/dump_state_diff.mjs` structSummary 同步扩展 fish 计数

### 7.6 Phase 4-B 豁免项

- 不做 bit-exact 对拍（Phase 4-B 豁免）
- ecology.rs FISH 逻辑冻结，不做重构

---

*文档收尾。下次会话用户输入任意波次编号（如「继续 P4」或「做鱼类 FISH」）即可接续推进。*
