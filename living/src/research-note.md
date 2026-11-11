# 研究笔记：间接互惠合作机制（wave 4 候选）

## 0. 设计理解与歧义声明

理解：为 H（herb）引入 Nowak-Sigmund 式 image scoring，合作动作嵌入采食场景，阈值遗传，Lotem 表型性不合作者保稳定，不写剧本。歧义：任务提到「被帮助 +1」，与 image scoring 标准定义（只有行动者得分）冲突，下文按标准版处理（被帮助者不得分），理由见 §3。范围：第一波只对 HERB 开放；OMN 由 `_randomGenome(HERB)`（ecology.js:239）生成会自动携带新基因字段（无行为副作用），暂不参与。

## 1. 动作语义：「让食」（yield-bite）

**触发环节**：`_updateHerb` 的采食时刻（ecology.js:830 `if (ci>=0 && grass[ci]>0.1 && a.bite<=0 && a.speed<g.speed*0.45)`）。帮助者在即将咬下的一刻评估是否让出本次采食机会。

- **机会成立**（全部满足）：帮助者 `a.kind===HERB ∧ a.age≥30 ∧ a.energy≥0.4·a.maxEnergy ∧ a.helpCd≤0`，且 ∃ 同类 n：`n.energy < 0.4·n.maxEnergy`（饥饿线，H 即 <60）且 `dist(a,n) < HELP_RADIUS=22`（≈1.5 格，CELL≈14.2，ecology.js:17-18）。低能量（<0.4max）与幼体（age<30）自动不进机会队列 = Lotem 表型性不合作者。
- **帮助动作（Chief 裁定修订：直接能量转移）**：原设计「帮助者不转移、接收方自食」经第四波评审证明三种情况实际收益≈0（①接收方 bite 已 ≤0.84 时 `min(bite, 0.4·BITE_INTERVAL)` 无效果；②接收方不在草格，bite 到 0 也吃不到；③草被其他 H/OMN 抢先吃掉），帮助者承担整口代价（8–17 能量）而 b/c≪1，实测 H 均值 −11%/−17% 即结构性浪费，不可合入。修订后为**反刍/共食式直接能量转移**（真实生物行为，非游戏化）：帮助者按 `_takeGrass` 的 water-filling 语义从格内扣减本口草量（适口性加权），对应能量直接加给接收方（clamp 到其 maxEnergy；接收方饿死线下通常不触发），帮助者跳过本次采食、bite 计时照常重置（它的「一口」给了对方）；果实能量与肠道种子随采食归接收方（`_takeGrass` 第 4 参 recipient）。接收方无需位于草格（HELP_RADIUS=22≈反刍距离）。
- **成本 c**：本口本会获得的能量 = 转移给接收方的能量（`take·62`，满草格约 8–17 energy，另损失 0.18 hydration）。能量口径 b/c=1；接收方处于 `energy<0.4max` 饥饿拐点，能量边际适应度价值 >1（跨过饿死线），帮助者处于饱腹区边际≈0 → **适应度口径 b>c 严格成立**（Nowak 机制成立）。
- **收益 b**：即转移能量本身（饿死线上边际适应度价值 >1，见上）。
- **接收方选择**：从 needy（energy<0.4max、dist<22）中选 **image > helpThr 者里最近的一个**；不存在 willing → 拒绝路径。
- **拒绝路径（M1 修订）**：拒绝 = **正常采食本口**（`_tryHelp` 返回 false 走原采食逻辑）+ image−1 + helpCd。语义干净：帮助=把一口给对方（有真实成本），拒绝=自己吃（无即时代码，代价是未来被排除——标准 Nowak 模型）。helpCd 防止每 tick 重复评估。
- **image 更新**：帮助者 `image←min(5, image+1)`；机会成立但拒绝（`n.image ≤ a.genome.helpThr`）→ `image←max(−5, image−1)`；被帮助者不变（image scoring 标准版：image 只记录「作为帮助者的历史」）。image 为个体状态，出生置 0，不遗传。
- **节流**：`helpCd←8+rng·4` s（帮助与拒绝都设），帮助频率天然被咬食节奏与 needy 空间稀疏度双重上限。

## 2. 数学公式（现有字段表达）

```
机会:  helpable(a) = (a.kind==HERB) ∧ (a.age≥30) ∧ (a.energy ≥ 0.4·a.maxEnergy) ∧ (a.helpCd≤0) ∧ (a.state!=='flee')
       needy(n)    = (n.kind==HERB) ∧ n.alive ∧ (n.id≠a.id) ∧ (n.energy < 0.4·n.maxEnergy) ∧ (hypot(n.x−a.x,n.z−a.z) < 22)
帮助:  help ⇔ ∃n: needy(n) ∧ (n.image > a.genome.helpThr)    // needy 中选 image>thr 者里最近者（N2）
执行:  take = _takeGrass(a, ci, 0.2·lineageBite, recipient=n)  // water-filling 扣减，果实/种子归 n
       n.energy = min(n.maxEnergy, n.energy + take·62)           // 直接能量转移
       a.image = min(5, a.image+1);  a.bite = BITE_INTERVAL·(0.8+rng·0.4)/lineageBite  // 跳过采食，计时照常重置
       _helpCostE_H += take·62                                    // 成本 = 本口本会获得的能量 = 转移量
       a.helpCd = 8+rng·4
拒绝:  (∃n: needy(n) ∧ n.image ≤ a.genome.helpThr) → a.image = max(−5, a.image−1); a.helpCd = 8+rng·4; 正常采食本口（返回 false）
遗传:  _randomGenome(HERB) 增加 helpThr = (rng()+rng()−1)·1.8          // Tri(−1.8,0,1.8)，中立偏 0
       _mutate: helpThr += (rng()−0.5)·1.2·drift, clamp [−5,5]         // 加性变异；乘性 mut() 在 0 附近退化，禁用
       _mixGenome: helpThr = (a.helpThr+b.helpThr)/2
```

## 3. 生态风险预判

- **Axelrod 二阶搭便车**：拒绝者 image 跌至 −5 后，自身饥饿时只有 helpThr<−5 者才会帮它——间接惩罚内生于系统，无需额外惩罚机制。Nowak & Sigmund 1998 证明 discriminator 群体抵御 defector 入侵的条件是 b/c 足够大；本设计 c 小（一口草）而 b 大（饿死线上的存活），满足。真正的风险是 drift：H<60 时相遇稀疏，合作等位基因可能被随机淹没（呼应旧结论「自私会赢」）。
- **空间结构**：4 邻格草蔓延 + H 家域 `range≈170`（≈12 格）+ `genome.social>0.8` 聚群（ecology.js:758-765）提供足够粘性与重复相遇；帮助半径 22 单位 ≈1.5 格在 H≈200–400 密度下（420 上限/5776 格≈0.073/格，needy 占比~15% → 单次机会概率 ~0.25）可达有效频率。H 密度长期 <60 则不足以支撑——验收时重点看低密度段。
- **能量预算**：单次 c≈8–17 energy，帮助者上限 0.125 次/s（helpCd≥8s），期望频率远低于此。预算约束：**Σ帮助成本 / Σ(H 代谢+逃逸消耗) < 5%**，预期实测 1–3%。超 5% 视为破坏性，触发止损。
- **捕食者伪装**：不需要防滥用机制。机会触发要求双方 `kind===HERB`，P 无 image 字段、无帮助队列；帮助评估只在帮助者自身 graze 采食时刻发生，且 `state==='flee'` 时整体跳过——P 靠近引发的恰是 flee，不存在「被解读为帮助」的通道。
- **image scoring vs standing（Leimar）**：本实现感知为精确读取（无观测噪声），标准 image scoring 已够；表型性不合作者不进机会队列、不积累负分，天然提供了 Leimar 建议的容错。不引入 standing 的二级状态机。

## 4. 实现草图（ecology.js 改动点）

1. **常量区**（L16-30 附近）：`HELP_RADIUS=22, HELP_CD_MIN=8, HELP_CD_RAND=4, HELP_NEEDY=0.4, HELP_ENABLED=true`。
2. **`_randomGenome` HERB 分支**（L239-251）：加 `helpThr: (r()+r()-1)*1.8`。
3. **`_mutate`**（L267-288）：加 `out.helpThr = clamp(g.helpThr + (r()-0.5)*1.2*drift, -5, 5)`（drift 已有小种群漂变增强，复用）。
4. **`_mixGenome`**（L662-674）：加 `helpThr: (a.helpThr+b.helpThr)*0.5`。
5. **`spawn` 字段表**（L292-352）：加 `image: 0, helpCd: 0`（spawn 时初始化，满足铁律）。
6. **新增 `_tryHelp(a, ci)`** 方法（放 `_takeGrass` 后）：返回 true 表示本口已转移（帮助，调用方跳过采食），false 表示正常采食（含拒绝路径）；帮助路径复用 `_takeGrass(a, ci, want, recipient)` 的 water-filling 扣减并把 `take·62` 直接加给接收方（clamp 其 maxEnergy），实现 §2 公式。
7. **`_updateHerb` 采食块**（L830-837）：改为 `if (!this._tryHelp(a, ci)) { ...原采食逻辑不动... }`（拒绝路径返回 false 自然落入原采食）。
7b. **M2 分母补全**：`_reproduce` H 分支把繁殖能量扣减累加到 `_drainE_H`（有性双亲 38+30=68 / 无性 52），保证 helpCostShareH 分母完整。
8. **观测**：`_sample`（L1310-1325）统计 `coopShareH`（image>0 的 H 占比，中性代号）、`helpThrH`（均值）→ `history.push`（L1336-1344）与 `stats()`（L1436 起）透传；evolution.herb avg（L1328-1335）加 helpThr。stats 命名只用 helper/coop 中性词，无剧本标签。

## 5. 标定建议与验收线

- **先跑参数**：HELP_RADIUS=22；helpThr 初始 Tri(−1.8,0,1.8)；变异步长 1.2·drift；helpCd 8–12s；needy 线 0.4max；不做 c/b 显式缩放（成本由既有采食参数内生决定）。
- **验收**（仿 `test_ecology.mjs` 写 `test_cooperation.mjs`，≥3 seed × ≥600s，仅用 this.rng）：
  - H/P/S/O 全程存活，H 数量不低于无合作基线的 90%，无物种灭绝；
  - `coopShareH` 在 t>200s 后 >0.05 且 mean helpThr 无单调冲顶（>4 为警号）；
  - 能量占比 <5%（需临时埋点：帮助次数×单次 c 估算 ÷ H 总消耗）；
  - 无 NaN、energy 无负值、helpCd/image 恒在界内。

## 6. 止损点

失败模式与处置（任一触发即回退，不追加机制）：
- (a) 多 seed 600s 后 coopShareH≈0 且 mean helpThr>3 —— 合作被漂变/选择淘汰，说明空间相遇不足，属环境约束，不通过加惩罚机制硬凑；
- (b) 任一物种灭绝或数量偏离基线 ±25% —— 能量预算失衡；
- (c) NaN 或数值越界。
处置：`HELP_ENABLED=false` 一键关闭（所有新路径包在开关内），报告失败模式数据，等待下一轮设计。**明确不做**：共享觅食格、声誉二级状态、跨物种帮助、对 OMN 开放。（Chief 裁定修订：原禁止项「显式资源转移」已删除——第四波评审证明「不转移」路径帮助实际收益≈0、b/c≪1，反刍/共食式直接能量转移是真实生物行为而非游戏化，故改为直接转移语义并从禁止项移除。）

---

## 给主会话的建议与风险

1. **关键取舍已定**：合作动作选「让食」而非「警戒/护幼」——警戒会改变 P 的击杀率，与捕食动态强耦合，风险更高；让食只动 H 内部能量流，blast radius 最小。
2. **最大风险**：低 H 密度时段（冬荒季、捕食高压后）相遇率不足，合作可能起不来——这是「自私会赢」教训的空间版，验收时须分密度段看数据，避免误判为机制失败。
3. **实现注意**：`_tryHelp` 内所有随机走 `this.rng`；`image/helpCd` 必须在 spawn 字面量初始化；勿用乘性变异处理 helpThr。
4. **未完成项**：标定数字（helpCd、初始方差）为理论推导值，需无头验证后回调。
