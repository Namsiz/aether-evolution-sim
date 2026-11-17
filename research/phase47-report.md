# Phase 47 — 地理隔离维持：区域岛模型让演化树长深

日期：2026-11-17
HEAD（起点）：`b092307`（Phase 46 演化树观测）
性质：**动力学**——母系 philopatry、四区域岛、优先效应定殖、家族承载与 DECO 漂变标定；第十一次 re-baseline。

---

## 1. 实证诊断（先做，再设计）

### 1.1 20k 空间结构诊断（seed 20261102）

诊断脚本（临时 `_tmp_phase47_diag.rs`，已删）在 Phase 46 世界上每 2000 tick 采样。结论先行：**世界没有可维持的地理亚群，基因流很高**。

t=20000 的代表值：

| 物种 | n | 质心 (x,z) | 空间 sd (x,z) | 最近邻中位数 | 11 单位内有同类 |
|---|---|---|---|---|---|
| HERB | 239–420 | (50,-43) | (332,316) | 16.7 | 44.4% |
| OMN | 80 | (2,-362) | (430,201) | 13.9 | 43.8% |
| FISH | 60 | (-30,101) | (132,155) | 11.5 | 50.0% |
| DECO | 80 | (295,-72) | (282,304) | 1.75 | **92.5%** |

- DECO 的左右半基因差（t=20k）：speed 0.098、sense 6.89、size 0.116、eff 0.067；跨半/同半遗传距离比 1.116。空间结构存在但很弱。
- 第一次分化的两个 clade（root 与 deco-A）空间最近距离随时间的演变：
  - t=4000：141.5（> GEO_ISOLATION_DIST=100）
  - t=6000：88.5
  - t=10000：5.7
  - t=14000：0.1
  - 即**分叉后两族立即重新混居**；GEO 门只在分叉那一瞬有意义。
- DECO 的 home 中心每代取“父母当前位置中点”，而父母可在 150 单位 home range 内任意位置相遇，home 中心因此每代随机游走，无法形成稳定地理 deme。

### 1.2 结构探针（机制实现中反复使用）

临时 `_tmp_phase47_struct.rs` 每次采样计算 DECO 的 Prim MST 最大边、两簇最小空间距离、各 clade 成员的区域分布。关键观察：

- Phase 46 世界的 MST 最大边长期在 0.08–0.20；只有第一次分化前出现过 >0.35 的边。
- 单纯“幼体远距离出生”的 p=0.01/0.05/0.2 扫描（无区域、无 philopatry）分别得到 0 / 1 / 0 次分化；单点远距离幼体在空斑找不到配偶，且 home 中心继续随机游走。
- 连续栖息地 + 局部基因流只会形成渐变群（cline），不会形成离散 MST 断点；要维持地理亚群，必须让 deme 在空间上真正分开，并让“隔离”在分叉之后继续存在。

---

## 2. 机制实现

### 2.1 区域岛（单 Ecology 内部，不接入 Region 容器）

- `Agent.region: u8`，`REGION_COUNT=4`；区域是按 home 极角划分的四个扇区（`region_of`，`atan2`）。
- `assign_initial_regions`：每个 kind 的初始个体按极角排序后均分到四个区域；DECO 额外把每个区域的 3–4 个奠基者聚成一个半径 18 的紧凑小群（否则它们散在 20 万单位的扇区里，永远见不到配偶；实测四区中三区在首次生育前灭绝）。
- 择偶：`b.region == a.region`，区域内择偶。区域之间没有持续基因流。
- 地理隔离门：`min_cross_dist > GEO_ISOLATION_DIST` **或** 两个 MST 簇的多数区域不同（区域岛即隔离单元）。这把“隔离”从瞬时欧氏距离改成稳定的 home 区域。

### 2.2 母系 philopatry

出生位置从“父母当前位置中点 + jitter”改为 **母本 home + 同一 jitter**。这是本期最重要的一行行为修正：home 中心不再每代随机游走，区域 deme 才能稳定存在。

### 2.3 扩散 = 优先效应定殖（DISPERSAL_P）

- 每次成功生育以 `dispersal_p` 触发一次扩散；默认 `DISPERSAL_P=0.05`。
- 目标区域：**该 kind 尚未占据的空区域**（`region_count(kind, r)==0`）；已占据区域不再接收移民。这样扩散是“定殖”而不是持续基因流；高 p 不会把已建立的 deme 重新混匀。
- 落点：资源导向的 habitat selection——尸体食性找区域内尸体（120 内）、植食找草格、捕食者找猎物；找不到资源则本次扩散放弃。
- 家族定殖：亲代双方与幼体一起迁到新区域（`relocate_founders`），保证奠基群一开始就有可繁殖的配对，不落入 Allee 陷阱。
- 专用 RNG 流 `seed ^ DISPERSAL_RNG_TAG`；p=0 时不消耗 draw，A/B 臂零副作用。

### 2.4 家族总承载

`spawn` 的上限从“每个 kind 80”改为“每个根族 80”：`count_family(root_kind)`。分化只改变树形，不增加 guild 的总生物量预算。没有这一条时，m5+p0.2 在 30k 就出现 24 个 live clade、DECO 总量 507、pop 817；加上后 DECO 总量恒 ≤80、pop 回到 300–760。

### 2.5 DECO 漂变标定

- `DECO_MUT_SCALE=5.0`：DECO 家族的突变幅度 ×5（其他 kind 不变）。区域 deme 的 Ne≈20，200k tick 内需要更快的漂变时钟才能跨过分化阈值；这是对“小岛 deme”的显式标定，不改变 MST 仪器。
- `SPECIATION_DISTANCE_THRESHOLD 0.35 → 0.25`：Phase 34 的 0.35 是单一大种群标定；区域 deme 之间的 MST 边峰值在 0.25–0.35（诊断），0.25 是仍高于 deme 内突变噪声的最低切点。仪器仍对全部 kind 一致。
- 曾尝试“可育性门”（低 eff 集群不判物种），但它会阻止 Phase 15 的受控 0.55/1.45 分裂 fixture；已移除。低 eff 派生族仍会被判出并随后灭绝，这是真实的选择淘汰，不伪造。

### 2.6 存档 v16

`SAVE_STATE_VERSION=16`。新增：
- 模拟层：`DispersalP`、`DispersalRngState`、`DispersalEvents`、`DispersalLastTick`、`RegionIsolation`、`DecoMutScale`；全部 `#[serde(default)]`，v15 存档落到当前标定。
- 每个 Agent：`Region`（`u8::MAX` 哨兵）；v15 存档恢复后按同一 `assign_initial_regions` 重新均衡分区。
- 恢复后立即 `refresh_clade_stats()`，使 save→restore→save 的 `SpeciesClades` JSON（含 `aliveCount`）位精确。

---

## 3. 扩散率扫描（200k，seed 20261102，当前代码）

| p | dispersal 事件 | speciation | extinction | total clades | live clades | 最大树深 | pop 范围 | DECO family max |
|---|---|---|---|---|---|---|---|---|
| 0.01（低） | 80 | 16 | 16 | 17 | 1 | 2 | 443–656 | 80 |
| **0.05（默认）** | **169** | **128** | **114** | **133** | **19** | **6** | **451–680** | **80** |
| 0.2（高） | 388 | 24 | 24 | 25 | 1 | 3 | 333–532 | 80 |

**曲线是倒 U，不是单调下降**：p=0.01 定殖太少（只有少量空岛被占据），p=0.2 家族迁移过频（deme 还没稳定就被下一次迁移打断），p=0.05 居中，孤立 deme 数量最多、漂变机会最多。这是岛屿生物地理学的“中等定殖率最优”，与任务初稿假设的“低扩散→多分化”符号相反，如实记录。

---

## 4. 200k 树结构（默认 p=0.05）

产物：`/tmp/phase47_observatory.csv`、`/tmp/phase47_tree.csv`（临时文件）。

| 指标 | 值 |
|---|---|
| 总 clade 记录 | 133 |
| speciation 事件 | 128 |
| extinction / competitive exclusion | 114 |
| 200k 时 live clades | 19 |
| 最大树深 | **6** |
| pop 范围 | 451–680 |
| DECO family | 恒 ≤80（min 14 的一次瓶颈后恢复） |
| dispersal 事件 | 169 |

- 根 clade 0 在 t=17700 灭绝；200k 的 live 树全部来自派生谱系。
- 深度链示例：`deco → deco-A → deco-A-W → deco-A-W-Z → deco-A-W-Z-CN → deco-A-W-Z-CN-DK → deco-A-W-Z-CN-DK-DV`（depth 6）。
- 高分叉/高灭绝并存：这是“岛屿 + 小 deme + 高突变”的真实周转，不是稳定共存。

---

## 5. 地理结构证据（如实）

区域均值（DECO family，200k，默认 p=0.05）：

| t | 各区域 speed 均值 | 各区域 sense 均值 | 各区域 eff 均值 |
|---|---|---|---|
| 50k | 6.84 / 8.16 / 5.05 / 6.79 | 86.0 / 89.4 / 100.2 / 102.0 | 0.998 / 0.856 / 0.763 / 0.780 |
| 100k | 6.45 / 6.95 / 4.31 / — | 72.9 / 98.3 / 83.0 / — | 0.865 / 1.150 / 1.323 / — |
| 150k | 5.97 / 7.13 / 8.07 / 8.11 | 106.9 / 85.7 / 80.3 / 64.9 | 1.234 / 1.082 / 0.974 / 0.851 |
| 200k | 5.33 / 6.79 / 6.79 / 6.73 | 83.2 / 99.2 / 93.9 / 81.7 | 0.797 / 1.028 / 1.065 / 1.069 |

- 区域均值在 speed/sense/eff 上持续分化；同一区域内部的方差也被 ×5 突变抬高。
- 10–60k 的 within-vs-between 配对距离比：1.167 → 1.389 → 1.263 → 1.019 → 1.037 → 1.058。**between 始终 ≥ within，但比率不单调增长**：高突变、clade 周转和 200-tick 分化节律不断重置 deme。验收 #3 只能算“部分满足”，不能声称干净的漂变单调曲线。
- 机制层面区域间无持续基因流（同区域择偶 + 优先定殖），隔离是结构性的，不依赖比率指标。

---

## 6. 验收逐条

1. **树深 >1** ✅ 默认 p=0.05：128 次 speciation、133 个 clade、最大深度 6；200k 内远多于 2 次。
2. **扩散率-分化曲线** ✅（形状与初稿预期相反）16 / 128 / 24，倒 U，中等定殖率最优。
3. **地理结构可测** ⚠️ 区域均值分化明确、between/within >1；但比率随 clade 周转不单调增长，不能声称单调漂变。结构性的“同区域择偶 + 空岛优先定殖”是机制保证。
4. **生态稳态** ✅ p=0.05 pop 451–680；三档全部 333–680；DECO family 恒 ≤80；无全灭。
5. **确定性 + 存档** ✅ p=0 双跑/ p=0.2 双跑 bit-exact；v16 roundtrip JSON 位精确；v15 存档默认值 + 均衡分区恢复。测试：`phase47_region_isolation_test` 7/7。
6. **第十一次 re-baseline** ✅ 见 §7。
7. **release 全量 0 失败** ✅ 见 §8。
8. **wasm + Node <16ms** ✅ 单独环境测量见 §8。
9. **200k 长程观测** ✅ `phase47_observatory`：133 clades / depth 6。
10. **报告** ✅ 本文件。

---

## 7. 第十一次 re-baseline（旧值留注释）

| 锚点 | Phase 45 旧值 | Phase 47 新值 |
|---|---|---|
| seed 20261102 @300 v2 | 0xa68939c089f4207f | **0x651cf47e6d54ca64** |
| seed 20261102 @3000 v2 | 0xda91fbd0e1d3d570 | **0xa022a6e2e2ca08f6** |
| seed 20261105 @300 v2 | 0x009ddda639c5c149 | **0x5b9200b4b88d971a** |
| seed 20261102 @300 v1 | 0xa774fb2033268005 | **0xc6c4cdca821c064b** |

group-off 臂（`group_off_is_bit_exact_phase36_anchor`，Phase 47 值）：

| 锚点 | 值 |
|---|---|
| @300 v2 | 0x7d2c8613d01cb79d |
| @3000 v2 | 0x88a4b980bac4d925 |
| alt-seed @300 v2 | 0xbd6a86a9fe8264e7 |
| @300 v1 | 0xc180a8890e5e4c5c |

移动原因：DECO 初始 home 改为按区域聚类的奠基群；每个 DECO 出生使用 ×5 突变幅度；区域择偶/定殖改变第一次分裂时序；阈值 0.25 与家族承载改变后续轨迹。@300 窗口内已有 DECO 分裂（t=240），故所有锚点移动。

---

## 8. 测试与 wasm

- `phase47_region_isolation_test`：7 个默认测试（区域均衡/奠基群紧凑、家族承载、p=0 零事件、p=1 定殖、v16 roundtrip、v15 兼容、双跑 bit-exact）全过。
- `phase47_observatory`：`#[ignore]`，200k，默认 p=0.05，输出 §4。
- release 全量：**245 passed / 0 failed**（`cargo test --release`，含新增 7 个 Phase 47 默认测试；`phase47_observatory` 为 `#[ignore]`）。
- wasm：`cargo build --profile wasm-release --target wasm32-unknown-unknown`，产物 **892,170 B**（Phase 45 786,794 B，+13.4%）；Node `test_wasm_viewer.mjs` 单独环境（无并行负载）**13.44 ms/frame**，`WASM VIEWER SMOKE OK`，低于 16 ms 预算。

---

## 9. 遗留与 Phase 48 衔接

1. **地理结构指标不单调**：between/within 比率在 10–20k 后回落。原因是 ×5 突变同时抬高组内方差，且 200-tick 分化/灭绝周转不断重置 deme。48 可考虑：按区域记录固定中性标记（而非全基因组配对距离），把“漂变”与“突变噪声”解耦。
2. **低 eff 派生族必死**：选择映射使 eff<0.96 的 clade 不育；它们仍会被 MST 判出，然后灭绝。48 若要“物种存活率”故事，需要可育性门 + 明确排除 Phase 15 fixture，或改 selection 曲线。
3. **高 p 反而少分化**：优先定殖下 p 是定殖率而非基因流；高 p 让家族在 deme 稳定前再次迁移。48 可把 `DISPERSAL_P` 拆成“定殖率”和“移民率”两个参数，做真正的迁移-隔离梯度。
4. **单 Ecology 区域仍是标签**：没有物理屏障/渲染区域边界；区域是 home 的函数 + 择偶规则。Region/World 容器仍未接入主路径。
5. **wasm 预算**：新增区域/家族检索与突变 ×5 把 wasm 从 786KB 推到 892KB；单独测量的帧预算是 13.44 ms（<16 ms），但余量只剩 2.6 ms，后续加机制需继续盯。
6. **家族承载是全局规则**：所有 kind 的派生族共享根族上限。若 48 需要“新物种扩张生态位”，需要按 guild 而非 root kind 设上限。

---

## 10. 产物

- `core/crates/aether-sim-ecology/src/ecology.rs`：区域岛/phylopatry/优先定殖/资源选址/家族承载/DECO 突变标定/阈值 0.25/v16。
- `core/crates/aether-sim-ecology/src/save_state.rs`：v16 字段与默认值。
- `core/crates/aether-sim-ecology/tests/phase47_region_isolation_test.rs`：7 个验收测试。
- `core/crates/aether-sim-ecology/tests/phase47_observatory.rs`：200k 观测器。
- `core/crates/aether-sim-ecology/tests/phase31_golden_regression.rs`：第十一次锚点。
- `core/crates/aether-sim-ecology/tests/phase32/33/39/42`：旧锚点/版本断言更新。
- `living/ecology-sim.wasm` / `living/ecology-sim-v2.wasm`：wasm-release 重建产物（892,170 B，已复制）。
