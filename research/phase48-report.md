# Phase 48 — 百万 tick 演化树命运：分叉-灭绝永动机（周转而非多样化）

日期：2026-11-17
HEAD：`1dc18be`（Phase 47 地理隔离维持）
性质：**纯观测**——1,000,000 tick 追踪 Phase 47 区域岛世界的新演化树。

---

## 1. 终局树结构

```
clades total 79（deco, deco-A … deco-BZ，78 个后缀）
alive 1（根 clade 'deco'，80 成员）
extinct 78
max depth 2（Phase 47 的 200k 快照曾达 6 层——那是瞬时结构）
```

**女儿 clade 寿命**：min 660 / median 3840 / mean 4022 / max 11280 tick（38-112 模拟秒）。

**分化时间分布**（每 100k tick）：6 / 3 / **22 / 9 / 24** / 1 / 3 / 7 / 1 / 2 —— 200k-500k 最密集。

## 2. 核心发现：这是物种周转，不是物种多样化

事件序列清晰重复：

```
[split] t=482700  parent=0  children=[62]  dist=0.258  kind=5   ← 分化出新 kind/clade
[split] t=489120  parent=62 children=[]    dist=0.000  kind=67  ← 女儿 clade 灭绝（event_kind=1）
[split] t=497520  parent=0  children=[64]  dist=0.428  kind=5   ← 根再分叉
```

**每次分化产生一个新 kind（deco-X）+ 一个 2 成员左右的小 clade；几千 tick 后小 clade 灭绝；根 clade 永生。** 154 次事件、79 个 clade、最终生态功能仍然只有一个（deco 在跑）。

### 2.1 三重机制根因
1. **`SPECIATION_MIN_SIDE = 2`**——2 个成员的「新物种」本质是采样噪声（Phase 34 注释自承 one-agent 是 noise）
2. **family cap（每根族 80）**（Phase 47 加）——女儿 clade 与根共享预算，无法扩张
3. **资源全局共享**——niche_offset 只改变竞争负担（COMPETITION_BURDEN），不提供独立资源池。真实物种形成需要独立生态位（海岛/山脉/宿主），我们的 grass/mineral 是全局的 → 任何分化必然在资源竞争中消亡

### 2.2 但这是现实的镜像
真实谱系树也是「不断分叉 + 99% 分支灭绝」，活种数远少于历史种数。我们复现了**高周转生态系统**（热泉/珊瑚礁型）——只是周转的是名义物种而非生态功能。

**关键区分**：
- ✅ 分化机制工作了（遗传距离检测、隔离门、niche 分离全部在跑）
- ❌ 分化没有生态后果（功能冗余：一个 deco 家族占据所有角色）

## 3. 其他长期模式

| 指标 | 数据 |
|---|---|
| 种群 | 370-680 波动（patch 扰动 + 分化扰动叠加） |
| 文化 | tokens 8346（线性），bearers 274-623（比 44.2 稳定） |
| 认知份额 | cog 0.088（稳定高位——智能持续被选择） |
| OEE 速率 | 0.15-0.25（持续分化中） |
| Tainter | 负值累计（口径瑕疵，仍未修） |

## 4. Phase 49 方向：让分化产生生态后果（多样化而非周转）

候选机制（按杠杆排序）：
1. **资源份额随 kind 分配**：新 kind 诞生时获得独立资源配额（根族预算 80 → 60 + 20），总预算不变——女儿 clade 有独立生态位可活（同时避免 Phase 45 的预算爆炸）
2. **提高 SPECIATION_MIN_SIDE**（2 → 8-12）：真实门槛——小群体不算物种
3. **文化-分化耦合**：分化的 clade 若能用不同 token 建结构 → 创造自己的资源微环境（Phase 37 的生态位构造 × Phase 47 的分化耦合）
4. 多 kind 并存的 guild 分配

验收目标：**活 clade 数 > 2 稳定存在 + 生态功能多样性（多个 kind 同时承担不同 diet/角色）+ 周转率不塌缩**。

## 5. 产物

- `tests/clade_fate_observatory.rs`（Phase 46 工具，用于 1M 观测）
- CSV：`/tmp/clade_fate_20261102.csv`（200 行采样）、`/tmp/clade_tree_20261102.csv`（79 clade 完整树）
- 分析脚本：`/tmp/analyze_tree.py`
