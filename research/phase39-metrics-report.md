# Phase 39.0 — 文明度量导出（Civilization Telemetry）

日期：2026-11-14
HEAD（前）：`ab65a44`（Phase 38 多区域）
阶段性质：**纯只读遥测**——零动力学影响（黄金锚点逐位不变）。对应路线图 Phase 39+ 的「文明度量导出」条目。

---

## 1. 为什么先做这个

Phase 38 交付后按路线图要求跑 5 seed × 5000 tick 基准观测，发现关键演化事实：

- **PRED 系统性灭绝**（全 seed 在 t≈2000-3000 归零）——Phase 36 的 `TROPHIC_EFF=0.10` + 脑维护费双重压力下的真实结果
- **DECO 饱和**（~80 满容量）——矿化通道是唯一稳定能量来源
- **clade 分化极少**（0-2 个）

路线图 §Phase 39+ 明确「这些依赖 31-38 的实际演化结果重新规划」——在决定下一个动力学扩展前，先建立**不改变世界的望远镜**（度量纪律 §2.5：metrology is a telescope, not a whip）。

---

## 2. 交付内容

### civilization_metrics.rs（新，256 行，纯 read-only）

四个参考系（全部来自现有状态，零 RNG、零状态写）：

1. **Kardashev 能量通量**（EnergyFlux）
   - `primary_input`：账本 light_in 累计初级生产
   - `dissipation_ratio`：总损耗 / 初级输入（稳态趋近 1.0）
   - `trophic_flux_share`：predation_gain / primary（第二营养级能量占比）
   - `cognition_share`：brain_cost / primary（**智能定价的直接度量**）
   - `information_share`：info_cost / primary（感知成本占比）
2. **Adami 信息存量**（InformationStock）
   - 总座数 = Σ(17 + ext.len())（每座携带的环境信息）
   - 存活 clade 数 + token 注册数 + token 携带者数（文化信息）
3. **Tainter 边际斜率**（ComplexityTelemetry）
   - `tainter_slope` = (吞吐 − 维护成本) / 存活 clade 数——**负值即崩溃信号**
   - `selection_levels`：当前选择层级数（1 = 仅个体层；群体实体出现后才会 >1）
   - `niche_occupancy`：有生态位偏移的 clade 数
4. **OEE 活动度**（OeeActivity）
   - 分化速率 / 1000 ticks（Bedau 式新增类型速率）
   - 发明数（token 注册）+ 谱系计数

外加 `format_line()` 研究 CLI 一行输出。

---

## 3. 观测结果（望远镜第一次校准）

5 seed × 5000 tick 实测：

```
seed=20261102 CIV t=2000 pop=626 clades=1 diss=0.90 trophic=0.002 cog=0.037 tainter=5241.6
seed=20261105 CIV t=4000 pop=105 clades=1 diss=0.42 trophic=0.002 cog=0.025 tainter=-660.7
seed=20261105 CIV t=5000 pop=80  clades=1 diss=0.39 trophic=0.001 cog=0.023 tainter=-4093.5
```

- **Tainter 斜率转负与人口崩溃完全吻合**（seed 20261105：626→80 人的崩塌窗口里斜率从 +1600 跌到 −4093）——崩溃信号真实涌现，非脚本
- **trophic flux 0.001-0.006**：PRED 灭绝后捕食通道几乎关闭
- **cognition share 2-5%**：大脑成本占初级生产 2-5%（对比人脑 ~20%——还有演化空间）
- **tokens=0**：文化层等待演化触发（发明算子的长程验证留给后续观测）

---

## 4. 验收结果

| # | 验收项 | 结果 |
|---|---|---|
| 1 | 遥测纯只读 | ✅ `telemetry_is_read_only`（测量前后 checksum 逐位相同） |
| 2 | Kardashev 通量与账本一致 | ✅ `kardashev_flux_reflects_ledger`（三个 share 精确等于账本比值） |
| 3 | 信息存量与存活状态一致 | ✅ `information_stock_counts_loci_and_tokens` |
| 4 | OEE 活动度与事件日志一致 | ✅ `oee_activity_tracks_clade_events` |
| 5 | 黄金锚点不变（零动力学） | ✅ 全 v2 锚点逐位保留 |
| 6 | release 全量 | ✅ **114 passed / 0 failed** |
| 7 | wasm + Node 冒烟 | ✅ 636K（零增量——新模块纯 Rust headless，不参与 wasm 构建也可以；实测产物尺寸不变） |

---

## 5. 决策记录：PRED 灭绝不调参

**决定：保持 `TROPHIC_EFF=0.10` 不变。** PRED 在现有参数下灭绝是热力学闭合的真实后果——10% 营养级传递效率 + 大脑维护费压垮了顶级捕食者。这本身是「智能定价」实验的有效数据点，不是 bug。后续层级跃迁（Phase 39.1+）从幸存物种（DECO/Omn/Fish）出发，让世界自己找到新的组织形式。

---

## 6. 交接点

1. **Phase 39.1 层级跃迁层**：群体实体（单点瓶颈繁殖 propagule）+ 任务切换成本 + 脏活致突变——scout 报告 §Q2 的三个最小触发条件
2. `selection_levels` 字段已预留——群体实体出现后从 1 开始增长（OEE 里程碑）
3. 度量接研究 CLI（headless 导出）而非 wasm 产物——保持「观察层冻结」下的例外纪律
4. token 长程观测：需要数千 tick 才能看到发明/传播/失传事件，留给 Phase 39.1 的演化回灌
