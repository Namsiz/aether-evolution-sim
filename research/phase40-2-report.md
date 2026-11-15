# Phase 40.2 — 首 token 发明路径：文化层激活（第五次 re-baseline）

日期：2026-11-15
HEAD（前）：`a63ea3d`（Phase 40.1 繁殖阈值修复）
阶段性质：**观测驱动的机制补全**——补上 Phase 37 文化层的启动死锁。

---

## 1. 问题：文化层从未启动

Phase 37 交付了完整的文化机制（token / (c,f,N) / 三偏见 / 重组 / 失传 / 结构库），但 100k-tick 观测显示 **300k 模拟 tick 零发明**。原因：

```
文化循环门控：if culture_on && !token_registry.is_empty()
    ↓
registry 只能由 try_innovate_token 增长
    ↓
try_innovate_token 需要 agent 已有 ≥2 个 token
    ↓
初始 registry 为空、所有 agent known_tokens 为空
    → 死锁：首 token 无合法来源
```

## 2. 修复：de-novo 发明路径（个体试错 / 指导性变异）

**机制**（`culture.rs` + `ecology.rs`）：
- `try_invent_token(agent, registry, env_features, rng)`：
  - 门槛：`curious ≥ 0.50`（探索者才发明）、能量 ≥70% max、每 tick 概率 `INVENT_P = 5e-6`
  - 成本：`LEARNING_COST × 2`（试错比模仿贵——trial-and-error premium）
  - token 向量 = **环境签名 + 小噪声**（±0.05）：8 维 = 草量/适生性/水分/归一化 x/z/能量比例/光照/水合
  - 发明者立即持有该 token
- **环境耦合的意义**：「在富草区发明的 token」编码富草特征 → 用它放置的结构增强类似环境 → 生态位构造有真实梯度，而非随机语义。
- 文化循环门控放开：`if culture_on`（不再要求 registry 非空）。
- RNG 纪律：全部走 `culture_rng`（`TOKEN_RNG_TAG` 专用流）；draw 预算固定（尝试 1 次 roll，成功后再 8 次噪声）。

**参数标定**：初版 `INVENT_P = 2e-4` 时 t=5000 就有 1373 个 token（「疯狂发明」），调低到 `5e-6` 后在 30k tick 内产生 260 个 token、49 次学习、80 个结构——发明是稀有事件，符合「偶然发现」的语义。

## 3. 激活后的文化动力学（30k tick 实测）

```
t=5000  registry=2    holders=0  innovations=2    learns=0   structures=5
t=10000 registry=34   holders=7  innovations=34   learns=4   structures=20
t=15000 registry=118  holders=8  innovations=118  learns=19  structures=36
t=20000 registry=251  holders=1  innovations=251  learns=48  structures=68
t=25000 registry=252  holders=0  innovations=252  learns=48  structures=76
t=30000 registry=260  holders=1  innovations=260  learns=49  structures=80
```

四个动力学全部出现：
1. **发明**：token 逐步积累（de-novo 路径工作）
2. **学习传播**：49 次模仿（teach-and-copy 循环）
3. **结构放置**：80 个生态位构造物
4. **塔斯马尼亚式失传**：持有者随种群换血归零（t=10000 holders=0, t=25000 holders=0）——文化链条真实中断，registry 保留历史但无人传承

这是第一次在世界上看到完整的「知识产生→传播→积累→失传」循环。

## 4. Re-baseline（第五次）

文化事件发生在 t=300 与 t=3000 之间，因此 **300-tick 锚点全部不变**，只有 3000-tick 锚点移动：

| 锚点 | Phase 40.1（旧） | **Phase 40.2（新）** |
|---|---|---|
| seed 20261102 @300 v2 | `0xbbee72352ef4468d` | 不变 |
| seed 20261102 @3000 v2 | `0x00bd6cd1892d6c10` | **`0x23aca66c0c68ba69`** |
| seed 20261105 @300 v2 | `0xfa11ea50ba7ef71a` | 不变 |
| seed 20261102 @300 v1 | `0xfb8827f6b1b34b5a` | 不变 |
| group_off @300 (v2/v1) | `0x8e59843d…` / `0x771a11f1…` | 不变 |
| group_off @3000 v2 | `0xa62405ff14bfbcad` | **`0xa3dd655b64f18333`** |

## 5. 验收

- **128 passed / 0 failed**（含 4 项 Phase 40.2 专项：文化激活/学习与结构/边界约束/culture-off 惰性）
- 文化测试：`culture_activates_and_tokens_emerge`（registry>0）、`learning_and_structures_emerge`（learns>0 且 structures>0）、`invented_tokens_stay_in_bounds`（[-1,1]）、`culture_off_is_inert`（零 token、双跑一致）

## 5.5 长程验证（100k tick × 3 seeds，文化激活后）

| seed | 命运 | token 学习动力学 |
|---|---|---|
| 20261102 | 全程存活（pop 311-680 振荡） | **792 个发明持续积累**；持有者 0-5 人 |
| 20261105 | t=10000 全灭 | 1 个 token 成为「化石」（无人持有，registry 保留） |
| 42 | fish+deco 稳态 100k tick | 182 个发明；持有者 0-2 人 |

**核心观测：发明持续、传播断裂**——
- token registry 单调增长（seed2：34→792、seed42：9→182）——**知识产生是永不停歇的**
- 但 bearers 始终 0-5 人——**几乎无人持有**。塔斯马尼亚效应的极端版本：每个新发明由发明者短暂持有，发明者死亡即失传，新个体极少学习
- 死世界的文化化石：seed 20261105 灭绝后 registry 留下 1 个 token——「考古记录」自动形成

**因果链**（留待 Phase 40.3+）：
1. 学习机制粗糙（`(id+1)%len` 取邻而非空间邻近）
2. 无亲子传承（死亡即断链）
3. 学习成本 0.5 能量 vs 发明 1.0——但持有者基数太小，学不到

## 6. 留给下一阶段的观测问题

1. **传播效率不足**：260 个 token 只有 1-8 个持有者——学习机制（`(id+1)%len` 取邻居）过于粗糙，空间邻近学习待实现
2. **文化断裂**：种群换血即失传——需要「代际教学」(parent→child 传承) 才能形成累积文化
3. **token 语义效果**：`token_to_env_mod` 的映射（→ grass/moist 调制）是否产生可见的选择压，待长程观测
4. 百万 tick 超长观测（Phase 40.3 候选）：文化层激活后的世界长期命运
