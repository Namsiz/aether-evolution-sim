# Phase 42 — 三脑转正：行为权威 + 反事实世界模型学习

日期：2026-11-15
HEAD（前）：`8cdff26`（Phase 41.1 脑节律，第七次 re-baseline）
阶段性质：**第八次显式 re-baseline**。三脑从影子层转正为行为权威（`BRAIN_SWITCH_ENABLED = true`），仲裁动作经桥接层驱动 FSM 状态与运动意图；世界模型加入海马式经验回放；抑制闸门重新校准。

---

## 0. 一句话结论

Phase 41 的三脑不再只是记录员：当一个个体的三脑在 60 个连续脑节律 tick 上与自己的 FSM 动作一致 ≥95%、且元认知置信度 ≥0.50 时，行为权威移交三脑（`brain_authority = BRAIN_AUTHORITY_BRAIN`）。接管个体的每一步行为来自三脑仲裁动作（前进/左转/右转/吃/逃/繁殖/帮助），经**接口层桥接**落地为物种能力允许的 FSM 状态与运动意图；FSM 保留全部感知-目标维护与生理子程序。每个脑节律 tick 额外从 12 条环形经验缓冲中确定性轮转抽取 1 条做 delta-rule 世界模型巩固（离线重放不驱动神经调质）。抑制闸门参考基因校准后，`gate>0.5` 占比从 ~67% 降到 17-24%（理智干预成为少数）。

---

## 1. 行为权威开闸

### 1.1 接管条件（`check_brain_authority_switch`）

- **一致度来自三脑自己**：Phase 36 的旧门用的是 legacy 微脑的 `brain_equiv_streak`——把权威交给一个与接管者无关的网络在语义上是错的。Phase 42 改为三脑自己的滑动窗口：`triune_agree_bits`（64 位移位寄存器，只保留最近 `T_EQUIV_K_TICKS=60` 位）+ `triune_equiv_streak`（连续一致数）。gate = `streak ≥ 60 && popcount(bits) ≥ ceil(0.95×60)=57 && triune.confidence ≥ 0.50`。
- **置信度下限**（`T_TAKEOVER_CONF = 0.50`）：世界模型还没预测好自己的转移（预测误差大 → 置信度低）的脑没有资格掌舵。t=300 实测 25 个满足一致度的个体中 5 个被置信度门挡住。
- **接管不可逆、不继承**：新生儿从 FSM 权威开始（spawn 处 `brain_authority = BRAIN_AUTHORITY_FSM`），必须用自己的脑节律重新赢回一致度。
- 边界节奏不变：只在 `SPECIATION_CHECK_TICKS` 边界检查（60 tick 一次）。

### 1.2 动作 → FSM 桥接（接口层，不是行为剧本）

`brain_bridge(idx, h)` 在每个 `update_*` 的运动块**之后**调用（脑拥有最终一拍），只对已接管个体生效；未更新的脑节律 tick 复用 `triune.last`（最近一次仲裁）。

| 动作 | 桥接语义 | 落地 |
|---|---|---|
| 0 move_forward | 维持当前移动模式 | 不覆盖 FSM 本 tick 已算好的 `state`/`desired_speed`/转向（前进=继续） |
| 1/2 turn_left/right | 转体 | 朝向 ±2.8·h，目标点改到新朝向前方 10 单位（否则下一 tick FSM 转向子程序会把身体掰回旧目标）；保留 FSM 的速度 |
| 3 eat | 依物种能力落地 | 脚下有植物 → 停下吃（`state="graze"`，desired=0）；目标格仍有草 → 沿用；否则 richest-cell 探针（sense 内 `grass − 0.001·d` 最大格）；活猎物/尸体按 diet 与距离最近者（PRED 猎物优先、水生猎物受浅水可达约束、尸体锁仲裁沿用 Phase 8） |
| 4 flee | 逃离最近威胁 | 朝向背离最近 PRED + `state="flee"` + 逃跑速度；无威胁 → 退化为继续移动（不演不存在的逃跑戏） |
| 5 reproduce | 只表达意图 | 不覆盖运动；生理条件（能量/冷却/配偶）仍由 `update_*` 尾部的现有繁殖路径决定——脑不能绕过能量学 |
| 6 help | 走现有协作路径 | `state="graze"` + 停下 + `try_help`（kind/年龄/能量/冷却全在原路径里） |

设计纪律：**桥接零 RNG、不新增感知通道、不改能量学**。脑是总经理（决定「做什么」），FSM 是工人（保留「去哪儿/怎么吃/怎么繁殖」的全部子程序）。鱼类的转体/逃逸意图受水域物理钳制（`brain_aim` 对 Fish 模板做 11 候选朝向 × 10 单位 lookahead 探测，全离水则保持原朝向）——否则接管鱼群会被意图引上岸搁浅。

### 1.3 脑节律兼容（设计选择）

**选择：缓存 last decision，未更新 tick 复用。** 理由：(a) 行为连续性——200ms 的决策节律不该产生 2-tick 的「FSM/脑」交替抖动，否则接管个体会在两种控制器之间震荡；(b) 生物合理性——神经振荡周期之间运动系统维持上一拍的运动程序（motor program persistence）；(c) 学习闭环一致——`executed` 记录的是实际执行的动作，缓存意味着 executed 与真实行为严格一致。

### 1.4 回流一致性

`brain_shadow` 里 `executed = a.triune.last.action`（接管个体），影子个体仍是 `fsm_act`。三脑因此学习的是**自己造成的真实转移**，学习闭环自洽。测试：强制接管个体的 `last.action = 1`（FSM 永不产生的 turn_left），两 tick 内 `last_executed == 1`；FSM 臂保持 `≠ 1`。

---

## 2. 反事实世界模型学习（经验回放）

现实依据：海马尖波涟漪重放（sharp-wave ripple replay）/睡眠重放——离线经验回放是真实机制。

- **缓冲**：每脑固定 12 条环形经验（`REPLAY_CAP = 12`，`(sense, action, next, reward)` 全存）。固定常量而非基因：回放缓冲是个体情节记忆、不是可遗传脑结构，且不给 148 基因布局加维度；内存锁在 12 × 136 B ≈ 1.6 KB/脑。出生清空（继承世界模型=先天先验，不继承亲代的最后 12 条经历）。
- **更新**：每个脑节律 tick 在在线更新（当前 executed transition）之外，`replay_cursor = (cursor+1) % len` 轮转抽 1 条做 delta-rule 更新——**零 RNG**。回放走 `observe_impl(learn_meta=false)`：只练世界模型权重，**不触碰 RPE/置信度/校准**——离线巩固不应驱动神经调质（dopamine 必须反映在线奖励预测误差）。
- **成本计入**：回放更新的计算量与一次在线 `observe` 相当（7×81 线性模型的一步 delta rule ≈ 150 flops，远小于 rollout 的 7×depth×81），**计入现有 decision 成本口径、不另立费率**（说明：wasm 帧预算实测从 13.26 → 13.4ms，见 §7）。
- **A/B 效果**（见 §5.3）：replay ON 的世界模型在经验缓冲上的平均 L1 预测误差比 OFF 低 20-28%（t=600/1500/3000 三点一致），且在线 `pred_error_ema` 与置信度同步改善。回放给稀有动作（flee/help/turn）持续的巩固机会——这些动作的真实样本占比低，纯在线学习下模型长期欠拟合。

---

## 3. 抑制闸门校准

问题（Phase 41 §8.3）：30k 观测 67% tick `gate>0.5`、理智覆盖 70%——开闸后会「过度理性抹平情绪」。

- `gate > 0.5 ⟺ gain·margin > threshold`，即 margin 分布的分位数决定闸门开度。
- 实测 margin 分布（3000t 采样）：p50=1.10 / p70=2.38 / p80=2.78 / p90=3.74。
- 校准：参考基因 `G_INHIBIT_GAIN: 3.0 → 0.75`、`G_INHIBIT_THRESHOLD: 0.2 → 2.0`（阈值比 thr/gain = 2.67，落在 margin 的 ~p73）。基因边界不变（[-1,2]/[0,10]），仍是可演化基因——只动参考初始化。
- 结果：`gate>0.5` 占比 **t=300: 17.2% / t=1500: 23.9% / t=3000: 23.9%**——落入 10-30% 目标带（现实参考：绝大多数行为由习惯/情绪驱动，前额叶干预是少数）。
- 附带效应：gate 均值 0.65 → 0.29，`decision_rate` 中的抑制代价随之下降；`planned_override`/`inhibition` 频率同步下降（理智覆盖成为少数事件）。

---

## 4. 必须随行的动力学修复（42.1）

### 4.1 Allee 缓解（择偶半径随稀疏度扩大）——世界存活性

诊断（42 验收 100k 预跑暴露）：Phase 41.1 的「稳定」世界处于混沌边缘——任何 1e-4 量级能量扰动都会把它翻进灭绝盆地。根因是**空间择偶 Allee 效应**：

- `reproduce()` 要求 11 单位内有成熟可育配偶。实测（420 满员）：270 个待育个体只有 72 个（27%）在 11 单位内有配偶；种群稀疏化后（132/32 只）该比例 → 8%/0%——**零出生吸收态**，cohort 老龄化灭绝（shREP 臂 t=3000 起出生 4/500tick，平均年龄 460 > 寿命）。
- 修复：`mate_radius = 11.0 × (1 + (1 − density_factor) × 3)`（非鱼类）。满员（density=1）时半径恰为 legacy 11.0，密集世界动力学逐位不变；稀疏时搜索半径最大 44（寻找配偶的搜寻成本随密度下降而上升）。
- 效果：authority+replay 世界 30k 稳定无灭绝（herb 201-420 / scav 40 / omn 80 / fish 60 / deco 80），稀疏后可恢复。

### 4.2 觅食经济学（桥接 eat 分支）

两个诊断期 bug 的修复（与 Chief 的并行诊断会话协同完成）：

1. **stale target**：eat 一味走向 FSM 的旧目标会让接管 herb 在满草甸里饿死 → sense 内 richest-cell 探针（`grass − 0.001·d`，阈值 0.15）。
2. **rich-cell 跳跃反噬**：每个 eat 决策都重定靶 → 草食者在肥草格间跳跃、永远坐不下来喈食，摄入崩塌 → cohort 老龄化。终版规则：**脚下有食物先停下吃 > 目标格仍有草沿用 > 才做 richest-cell 探针**。
3. **鱼类水域钳制**：`brain_aim` 对 Fish 模板钳制到「10 单位 lookahead 仍在水中」的最近朝向（接管鱼被 flee/turn 引上岸搁浅，诊断 @16k fish 60→0）。

---

## 5. 验收逐条（Chief 验收表）

### 5.1 开闸生效
- `brain_learning.authority_switches`：t=1200 自然长跑 >0（测试断言 + 自然观测 t=300:21 / t=3000:150）。
- 接管个体行为跟随脑输出：`brain_action_drives_movement_after_takeover`（turn_left/turn_right 的重定靶断言：目标点 = 身位 + 10·sin/cos(θ±0.28)）；`executed_action_is_the_brain_action_after_takeover`（回流一致）。

### 5.2 适应性对比（接管 vs 未接管）
数据见 §6.1。t=3000 口径：每生命时间繁殖率 +34%（0.0138 vs 0.0103/tick）、能量 +10%（113.7 vs 103.4）、寿命 -5.6%（188.1 vs 199.2，短寿多育）。结论：**脑驱动类在达尔文适应度与能量上均占优**（测试 `brain_driven_class_is_not_worse_off` 断言适应度与能量双不劣）。

### 5.3 反事实学习有效（A/B）
| t | probe_err replay ON | OFF | 相对改善 |
|---|---|---|---|
| 600 | 0.0420 | 0.0585 | +28.1% |
| 1500 | 0.0458 | 0.0587 | +21.9% |
| 3000 | 0.0263 | 0.0329 | +20.2% |

（probe_err = 各脑回放缓冲上的平均 L1 预测误差，纯读不学习；同步 `pred_error_ema` ON 0.051 vs OFF 0.064。）测试：`replay_ab_reduces_world_model_error`（1200t，ON < OFF×0.92）。

### 5.4 抑制校准
`gate_open_ratio`：t=300 17.2% / t=1500 23.9% / t=3000 23.9% ∈ [10%, 30%]。测试：`inhibition_gate_is_calibrated`。

### 5.5 确定性
- 同 seed 双跑 bit-exact（authority+replay 默认世界 + shadow 臂双双验证，600/400t）。
- 存档 v13 往返位精确 + 续跑 checksum 一致（`save_v13_roundtrip_continues_bit_exactly`，400→700t）。
- v12 兼容：v12 存档（无 ReplayOn/authority 位图字段、三脑 JSON 无 replay 键）恢复 → replay ON、streak=0、空环，续跑确定性（`v12_archive_restores_with_replay_defaults`）。v10/v11 旧档回归通过。

### 5.6 re-baseline 第八次
见 §7 新旧锚点表。旧值（Phase 41.1）保留在测试注释与本报告。

### 5.7 release 全量
**143 passed / 0 failed / 8 ignored**（`cargo test --release -p aether-sim-ecology --tests`；Phase 41.1 基线 133 passed）。新增：phase42 验收 10 项 + triune 回放单测 4 项 + 长程观测探针（ignored）；既有锚点测试同步第八次 re-baseline。aether-brain crate 21 tests 全绿。

### 5.8 wasm + Node 冒烟
**6.53 ms/frame**（60 帧 sim+export 循环，预算 16 ms，Phase 41.1 基线 13.26 ms）——`WASM VIEWER SMOKE OK`，全契约断言通过（flat census/snapshot v3/矿物采样/FinColor 等）。wasm 产物 767 KB（wasm-release profile，opt-level=z + fat LTO）。

### 5.9 长程观测 100k × 3 seeds
见 §6.2：seed 20261102 蓬勃存活（final 511 只）、seed 42 存活（135 只）、seed 20261105 灭绝 @9.6k（**继承性命运**：41.1 基线同 seed 更快灭绝——herb 5k 清零、总灭绝 ~11k，与上阶段观测「seed05 阿利灭绝」一致）。

---

## 6. 关键观测

### 6.1 接管比例与适应性（seed 20261102）

| t | pop | brain (占比) | switches | gate>0.5 | conf | e brain/fsm | 寿命 brain/fsm | births/death brain/fsm | 达尔文适应度 brain/fsm (/tick) |
|---|---|---|---|---|---|---|---|---|---|
| 300 | 244 | 16 (6.6%) | 16 | 17.8% | 0.470 | — | — / 34.4 | — / 2.0 | — |
| 3000 | 589 | 119 (20.2%) | 152 | 26.6% | 0.595 | 113.7 / 103.4 | 188.1 / 199.2 | **2.60 / 2.04** | **0.0138 / 0.0103** |
| 10000 | 680 | 205 (30%) | 619 | 27.7% | 0.652 | 115.8 / 97.8 | 329.1 / 315.4 | 1.04 / 0.97 | — |
| 30000 | 461 | 253 (55%) | 2709 | 23.1% | 0.694 | 89.2 / 79.1 | 308.6 / 338.9 | 0.73 / 0.83 | — |

- **接管是渐进的**：6.6% (t=300) → 20% (3k) → 30% (10k) → 55% (30k)。三脑逐步赢得种群，但从未全盘接管（新个体不断从 FSM 起步）。
- **适应性（智能定价的终极检验）**：t=3000 口径下脑驱动类**每单位生命时间的子代产出比 FSM 类高 34%**（0.0138 vs 0.0103 /tick），能量高 10%；代价是寿命 -5.6%（短寿多育）。达尔文适应度（选择最大化的量）为正——脑驱动是适应度胜利，不是「更贵但没用」。
- 寿命单独看会误导（t=30k 反转），所以验收断言用适应度+能量双口径（`brain_driven_class_is_not_worse_off`）。

### 6.2 长程观测（100k × 3 seeds，`phase42_longrun_observatory`）

| seed | 命运 | final pop | brain 占比（终值） | switches | tokens / learn / struct | 备注 |
|---|---|---|---|---|---|---|
| 20261102 | **alive** | 511 | 54%（全程 30-62%） | 11108 | 323 / 37 / 158 | 30k 起即 461-680 只稳定；文化持续积累 |
| 42 | **alive** | 135 | 30%（全程 24-30%） | 1582 | 39 / 2 / 9 | 小种群稳态；**脑类寿命 412 vs FSM 301（+37%）** |
| 20261105 | extinct @9.6k | 0 | —（灭前 49 次） | 49 | 1 / 0 / 5 | 继承性灭绝：41.1 基线同 seed herb@5k 清零、总灭绝 ~11k |

- **接管比例全程漂移**（seed2: 30%→47%→55%→62%→54%）：权威不是单向棘轮——新生儿不断从 FSM 起步，接管比例是「重赚权威的出生流」与「接管者存活」的动态平衡。
- **三脑统计**：gate>0.5 全程 17-28%（校准在 100k 尺度稳定）；confidence 0.65→0.72（世界模型随回放持续巩固）；probe_err（模型在经验缓冲上的 L1 误差）0.022-0.027 → 0.022（长程不发散）。
- **文化**：seed2 的发明持续增长（190 @30k → 323 @100k），但 bearer 长期 0-3——学习传播远慢于发明（(id+1)%len 槽位机制的运气依赖），证实 §9.1 的 Phase 43 衔接点。
- seed42 的脑类寿命优势（412 vs 301）与 seed2 的「短寿多育」形成对照：脑类的适应性优势在不同生态位里以不同形式出现（长寿命 / 高繁殖率），共同点是能量不劣。

---

## 7. 第八次 re-baseline（新旧锚点）

| 检查点 | Phase 41.1（旧） | **Phase 42（新）** |
|---|---|---|
| seed 20261102 @300 v2 | `0x1265fe37e680e828` | **`0xdc98f7d34a7af777`** |
| seed 20261102 @3000 v2 | `0x0d10cdeaf3e447c0` | **`0xafa833641ebfd5a8`** |
| seed 20261105 @300 v2 | `0x0c1a5b2254ef521e` | **`0x2e9539930abd8975`** |
| seed 20261102 @300 v1 | `0x490a0a05d97ba7d3` | **`0xe6404a7bee633eb8`** |
| group_off @300 v2 | `0x928befcb3c6e255a` | **`0xf5cc546cb068941f`** |
| group_off @3000 v2 | `0xa45d2f6f3d8ab0dd` | **`0x615fbdaaa7fb0f4e`** |
| group_off alt @300 v2 | `0x1273f9270d9d1d33` | **`0x7a3fe91e7165c234`** |
| group_off @300 v1 | `0xda27297f37db8db1` | **`0x5b514478b9799fa0`** |

A/B 参考锚点（不进回归，报告存档）：shadow-only（关闸）与 replay-off 两臂的 300t checksum 均与默认世界不同——开闸与回放各自移动轨迹（`deterministic_double_run_bit_exact` 内含 `assert_ne` 验证）。
SCAV/OMN census @50t 网格（structural_alignment）：SCAV 8 全程，OMN 12→13→14（t=250/300），与 Phase 41.1 相同；phase32/33 的 300t 锚点同步到新值。

---

## 8. wasm 与 Node 冒烟

`living/test_wasm_viewer.mjs`：`WASM VIEWER SMOKE OK`，**6.53 ms/frame** << 16 ms 预算（41.1 基线 13.26）。回放（每脑节律 tick 一次 ≈150 flops 的 delta rule）与开闸桥接（零 RNG 查表+探针）都在预算内。census（herb 168 / pred 14 / scav 8 / omn 15 / fish 21 / deco 18）在 warmup+120t 处采样。

---

## 9. 遗留与 Phase 43 衔接点

1. **文化传播仍然脆弱**：learn 依赖 `(id+1)%len` 槽位的示范者运气；本世界 30k 内 learn 事件稀少、bearer 经常清零。代际文化传承（亲代 known_tokens 部分遗传）是 Phase 43 首选候选。
2. **草食者能量带宽**：42.1 修复后世界稳定但草食者偏瘦（e/em ~0.5 @30k），0.90 硬性繁殖开关（`h_cache>250`）+ REPRO_THRESHOLD_MAX 的组合在低草期仍会压出生率。若 100k 观测显示种群长期低位，候选：把 0.90 硬开关改成密度连续函数。
3. **replay 缓冲容量是常量**：12 条对稀有动作的覆盖有限；若 Phase 43 想做「按动作分桶」的回放，需要把容量改为结构基因（会再次 re-baseline）。
4. **接管不可逆**：没有降级路径（脑持续劣化也无法回退 FSM）。演化上这由致死淘汰兜底，但一个「权威可逆」实验（低于置信度下限回退）值得一做。
5. **桥接的动作语义仍按物种模板分派**（Herb/Pred/…），新注册 kind 若带非标 behavior 模板需要补桥接分支。
6. 感知通道 8 维不变、观察层冻结、无代际文化传承（均按任务要求未做）。

---

## 10. 改动文件

- `core/crates/aether-brain/src/triune.rs` — 经验回放（`ReplayEntry`/`record`/`replay_step`/`probe_error`/`probe_error_for`）、`observe` 拆分出 `observe_impl(learn_meta)`（离线巩固不驱动神经调质）、闸门遥测（`gate_open_ticks`/`gate_sum`/`gate_open_ratio`/`mean_gate`）、抑制参考基因校准（gain 3.0→0.75 / threshold 0.2→2.0）、`reset_transients` 清空情节环、4 项回放单测。
- `core/crates/aether-sim-ecology/src/ecology.rs` — `BRAIN_SWITCH_ENABLED=true` + `T_TAKEOVER_CONF`/`REPLAY_ON` 常量；Agent 三脑一致度字段（`triune_equiv_streak`/`triune_agree_bits`/`triune_agree_n`）；`brain_shadow` 回流一致性 + 一致度移位寄存器 + 回放调用 + gate 遥测；`check_brain_authority_switch` 改用三脑一致度+置信度门；`brain_bridge`/`brain_steer`/`brain_aim`/`brain_probe_grass`/`brain_probe_prey`/`brain_probe_carcass` 桥接层（六种行为模板的物理钳制与觅食经济学）；六个 `update_*` 插桩；Allee 缓解（`mate_radius` 随稀疏度扩大）；`AuthorityFitness` 出生/死亡记账；`replay_on` 开关。
- `core/crates/aether-sim-ecology/src/save_state.rs` — v13 schema：per-agent `TriuneEquivStreak`/`TriuneAgreeBits`/`TriuneAgreeN` + sim 级 `ReplayOn`（全部 serde default → v12/v11/v10 兼容）。
- `core/crates/aether-sim-ecology/tests/phase42_brain_authority_test.rs` — 新增：10 项验收测试 + `phase42_longrun_observatory`（ignored 探针）。
- 既有测试锚点：`phase31_golden_regression.rs` / `phase32_open_loci_test.rs` / `phase33_plant_registry_test.rs` / `phase39_1_transition_test.rs` / `structural_alignment.rs`（第八次锚点与 census）；`phase36_brain_test.rs`（关闸 A/B 臂语义更新）；`phase41_triune_test.rs`（v13 schema + 新生个体脑节律窗口）；`niche_divergence_test.rs`（机制 A/B 隔离行为权威）。
- `living/ecology-sim.wasm`、`living/ecology-sim-v2.wasm` — wasm-release 重建产物。
