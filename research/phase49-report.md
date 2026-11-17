# Phase 49 — 分化生态后果：独立预算让女儿谱系能活（第十二次 re-baseline）

日期：2026-11-17
HEAD（起点）：`bb8141e`（Phase 48 百万观测）
性质：**动力学**——fork budget（独立资源份额）+ SPECIATION_MIN_SIDE 2→8；v17 存档。

---

## 1. 机制（针对 Phase 48 的三重根因）

### 1.1 Fork budget（针对根因②③：family cap 挤出 + 无独立资源池）
- 每个 kind 持有自己的承载份额（`kind_budgets`，root kind 从 `global_cap` 起）
- **分叉划拨**：`fork_budget_grant(parent) = clamp(parent/4, 5, 20)`，父份额扣减划给女儿 kind；递归下限 5（80 → 60+20 → 45+15 → 15+5，再浅不叉）
- **灭绝归还**：kind 灭绝时份额沿谱系归还父 kind——**guild 总量守恒**（Phase 45 生物量爆炸教训：80 恒定）
- spawn cap 从 family 共享改为 kind 自己的份额
- A/B：`budget_fork_off()` 恢复 Phase 47 family-shared-cap 轨迹（测试钉住）

### 1.2 SPECIATION_MIN_SIDE 2 → 8（针对根因①：采样噪声当物种）
- 2 个成员的「物种」中位寿命 3840 tick（Phase 48 实证）——是噪声不是物种
- 8 成员是最小独立演化群体；实例字段 `speciation_min_side`（默认 8，legacy 2 可钉）

## 2. A/B 验证（60k tick，seed 20261102）

| 臂 | 活 clade | 活女儿 | 总 clade | 事件 | 女儿寿命中位 |
|---|---|---|---|---|---|
| fork_on（本期） | **2** | **1** | 3 | 3 | **17760** |
| fork_off（Phase 47） | 1 | 0 | 7 | 12 | 1740 |

- **女儿寿命 10.2 倍提升**（1740 → 17760）
- **首次出现存活女儿 clade**（fork_off 全灭）
- 噪声分化消失（12 → 3 事件，全部是 ≥8 成员的真分化）
- 张力记录：总分化率下降（门槛提高的代价）——持续多样化速率待长程观测验证

## 3. 存档与确定性

- **v17**：`BudgetForkOn` / `KindBudgets`（空表 → 从 root caps 重建）/ `SpeciationMinSide`（default = legacy 2，旧档位精确回 Phase 47 臂）
- v16 及更早：全 serde-default 兼容；roundtrip bit-exact；双跑 bit-exact
- 第十二次 re-baseline：@300 锚点不变（分化未在 300 tick 内发生），@3000 移动（seed2 `0xa022a6e2 → 0xc94fc584`、group_off @3000 `0x88a4b980 → 0xdb3aa647`）

## 4. 验收

- **170 passed / 0 failed**（基线 170；phase34 mint 测试钉 legacy 门槛、phase47 cap 测试钉 A/B 臂——两者语义即测旧机制）
- wasm 792K @ **12.06-12.11ms** 三次稳定达标
- A/B 数据见 §2

## 5. 遗留与 Phase 50

1. **持续多样化速率待验**：60k 只有 3 次分化——MIN_SIDE=8 让分化变稀有。200k/500k 长程观测树结构（活女儿数是否随时间增长）
2. **文化与分化的耦合**（未做）：分化的 kind 若用不同 token 建结构 → 创造自己的资源微环境
3. fish 灭绝死因专项（Phase 44.2 遗留）
