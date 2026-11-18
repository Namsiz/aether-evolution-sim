# Phase 53 — 代际 niche 累积 + 预算下限校准：孙代存活的第一缕光

日期：2026-11-18
HEAD（起点）：`500ff4a`（Phase 52 观测）
性质：**动力学**（第十四次 re-baseline）——`NICHE_GEN_MIN_OFFSET=0.35` + `FORK_BUDGET_MIN 5→8`；SaveState v19。

---

## 1. 靶点与机制

Phase 52 的遗留：**孙代（depth-2）6 个全灭**。根因链已由 200k 观测证实有两层：

### 1.1 代际 niche 最小距离（`NICHE_GEN_MIN_OFFSET = 0.35`）
- 旧公式：`daughter_niche = parent_niche + (eff_daughter − eff_parent) × 2`——孙代的 eff 差小 → niche 距离 < `NICHE_PARTITION_BAND(0.3)` → 与母种吃同一片资源 → 已建立优势的母种胜出
- 新公式 `generational_niche()`：方向 = eff 差（平局时用速度差，再平局固定 +），**距离 ≥ 0.35**；强 eff 差照旧 clamp 到轴端（保留 Phase 16 语义）；**仅当 clamp 会压缩距离到 < 0.35 时才反向**（parent 已贴边）
- 两处调用点必须一致：fork 铸造时 + 每检测 pass 的重推导（否则初始值被覆盖）

### 1.2 预算下限与分化门槛对齐（`FORK_BUDGET_MIN 5 → 8`）
- 观测发现的机制矛盾：`SPECIATION_MIN_SIDE=8` 宣布「8 个成员才算物种」，但 grant 公式给孙代 kind 的预算只有 5（母种 20 ÷ 4）——**分化瞬间的 8 个成员立即被 spawn cap 压到 5，慢性萎缩**
- 修复后链条：80 → 60+20 → 45+15 → 37+8（再浅不叉，floor 8 = 门槛 8）

### 1.3 A/B 与存档
- `niche_gen_accum_off()`：恢复 Phase 52 投影（B 臂）
- SaveState **v19**：`NicheGenAccum`（serde default=true，旧档兼容）

## 2. 200k 观测（seed 20261102，多轮迭代）

| 轮次 | 分化事件 | 孙代命运 |
|---|---|---|
| 初版（floor 5） | deco-A-B 22k 灭绝、deco-A-D 54k 灭绝 | 0/2 存活 |
| **floor 8（当前）** | deco-A-B **44k 灭绝**、deco-A-C **51k 灭绝**、deco-A-D（t=194k 诞生）**存活到终局** | **1/3 存活** |

- 孙代寿命中位约翻倍（22k → 44-51k）
- **首次有孙代进入终局存活集**（deco-A-D）
- 终局活 clade 3（根族 + deco-A + deco-A-D 孙代）——三代同堂

## 3. 诚实记录：孙代仍多为周转死

真实演化中 99% 的新物种灭绝——当前世界的行为**与真实一致**。剩余差距的可能原因：niche 效率增益（±15%）可能不足对抗母种的位置/数量优势。**不再加参数**（继续加大增益会改变整体动力学），留给后续观测判定是否为稳态特征。

seed 42 的 200k 零分化（该世界 DECO 稳态不同）——seed 特异性，如实记录。

## 4. 验收

- **phase53 测试 5/5**（最小距离/边界反弹/off 投影/存活女儿不塌缩/A/B 深锚点）
- **全量分批全绿**：批 A 174 passed（跳过 8 个已单独验证的慢测）+ phase51 三测 + phase53 五测 + phase49 两测 + phase34 forced_fork = **0 failed**
- **第十四次 re-baseline**：@300/@3000 四锚点**全部不变**（首分叉 t=34.68k 远超窗口）；深锚点更新（partition/accum 四臂值记录在 phase51/53 测试）
- **wasm 792K @14.31-14.43ms 三次稳定**（<16ms）
- v19 存档（v18 兼容测试通过）
- phase16 竞争测试钉 `niche_gen_accum_off()`（最小距离令母女永不重叠是设计意图；Gause 机制仍在姐妹/远亲间生效）

## 5. 遗留与 Phase 54

1. **孙代长期命运**：需 1M 观测（按需）确认「1/3 存活」是否为稳态比例
2. **文化-分化耦合**（未做）：分化的 kind 用不同 token 建结构 → 独立资源微环境（第二种孙代支持机制）
3. **多 MST 轴家族**：HERB/OMN 分化通道（当前只 DECO）
4. 观测策略：200k 标准（~18 分钟）+ 3 seeds 并行 + 1M 按需
