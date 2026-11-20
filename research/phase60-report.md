# Phase 60 — 分化触发与生育修复解耦：区域承载容量（第十七次 re-baseline）

日期：2026-11-20
HEAD（起点）：`cdc127c`（Phase 59，第十六次 re-baseline）
性质：**新机制 + 默认 ON + 第十七次 re-baseline**。Phase 59 修好生育硬闸后，旧分化引擎失去隐性驱动（半绝育人口把 DECO 压在容量顶以下），seed 20261102 的分化归零。本期把「掉档」从**个体绝育**搬到**区域承载容量**：文化改写的分解场决定每个 home quadrant 能承载多少分解者，DECO 被迫区域化，物种形成在 3 seed × 200k 上从 8 → **22** 次，且所有低 eff 女儿支在灭绝前都保持可育（`deco_ready > 0`）。

---

## 1. 缺陷：旧分化引擎吃的是生育 bug 的副作用

Phase 59 §6 的结论原样成立：

- 旧硬闸 `0.96 × (2 − eff)` 在 `eff < 0.9583` 时要求 > 100 % `max_energy`，是**不可达**的绝育态；
- 这些个体占着容量却永不繁殖，把 DECO 的**有效繁殖群体**砍掉一截，DECO 长期停在容量顶以下（seed 20261102 旧世界 34–45）；
- Phase 56 的分解场再把某个支系推过物种距离，于是「分化」发生。

Phase 59 把硬闸改成饱和映射后，低 eff 个体全部可育，DECO 恒定满编 80（seed 20261102），首分叉的前提消失：**0 次物种形成**。能量精确截顶（`min(energy, max_energy)`）又让任何 `阈值 ≤ 1` 在满能量时立刻满足，所以**不能再用能量阈值做难度梯度**。Phase 60 需要一个不依赖个体绝育的掉档机制。

## 2. 机制：区域承载容量（regional DECO carrying capacity）

### 2.1 形式

`deco_env` 是 Phase 56 的每格尸体腐烂乘子（`1.4/tick × deco_env`）。值越高，尸体腐烂越快、能留下的食物越少。对每个 home quadrant `r`：

```
mean_env(r) = mean{ deco_env[cell] : cell 宜居 (suit > 0.02)，cell ∈ r }
w(r)        = clamp(1 + GAIN · (1 − mean_env(r)), W_MIN, W_MAX)
cap(kind, r)= kind_budget(kind) · w(r) / Σ_r w(r)
```

- `GAIN = 10`，`W_MIN = 0.25`，`W_MAX = 4.0`，`REGION_COUNT = 4`；
- 每格 → quadrant 的映射 `deco_cell_region` 是纯几何表（构造/读档时重建，不入档）；
- 权重在**每个 tick 的第一次 DECO 出生闸门**按当前 `deco_env` 重算（纯函数、零 RNG、表查找 5 776 次），所以不需要存档，读档后从恢复的场重导出一致。
- 出生闸门：`spawn()` 在原有 kind budget 检查之后，若 `deco_region_cap && deco_env_on`，用**孩子 home quadrant** 的 `count(kind, r) >= cap(kind, r)` 拒绝出生。拒绝不扣能量、不设冷却：母亲仍可育，区域一空出槽位就立刻能生。

### 2.2 为什么它不依赖人口副作用

1. **不是能量闸**：`deco_repro_threshold` 一行未改（`phase60_region_capacity_test::the_fertility_map_is_untouched_by_the_capacity_arm` 在 1 001 点网格上逐位比较两臂）。不存在「阈值 > max_energy」的不可达态。
2. **不是个体绝育**：被拒的母亲保留能量与 `repro` 冷却状态，仍是可育个体。观测器 `deco_ready` 在 200k 末对低 eff 女儿支逐支 > 0（§5），灭绝支系的生命周期普查也全部 > 0（最低 0.055，109 的 deco-A-B）。
3. **掉档来自空间容量**：每个 kind 的 Phase-49 budget 按区域食物保留量分配到四个 home quadrant，`Σ_r cap(kind,r) = kind_budget`。一个只占一个区域的支系只能用到该区域的份额——这是「区域容量」的直接后果，不是「个体不能生」。
4. **它确实改变了种群结构**（如实记录）：seed 20261102 的 DECO 从「69 只挤在 region 3 的超级 deme」变成四区约 20 只的岛模型，80k 总数为 28（OFF 80）。掉档是功能，不是副作用。

### 2.3 开关与存档契约

- `Ecology::deco_region_cap_on()` / `deco_region_cap_off()`；编译默认 `DECO_REGION_CAP_ON = true`（本期通过后翻 ON）。
- **不入存档**（与 Phase 58 clamp / Phase 59 生育映射同契约）：`from_save_state` 一律回到编译默认；权重与 `deco_cell_region` 不入档，由恢复的场重导出。SaveState 仍是 **v22**。
- `deco_env_on = false` 时区域容量一并失效（没有结构食物信号就没有容量信号），因此 `deco_resource_env_off()` 仍逐位复现 Phase 54/55/56 的旧世界锚点。
- 观测器新臂：`PHASE_DECO_REGION_CAP=on|off`（文件名分量 `regcapon` / `regcapoff` / `regcapdflt`）。

## 3. 块 1：80k 快速筛（3 seeds，单变量 A/B）

同一 release 二进制，`PHASE_DECO_REGION_CAP` 单变量。事件只数 `event_kind == 0`（物种形成）；灭绝另列。

| seed | 臂 | 物种形成 | 灭绝 | DECO@80k | home 区域分布 r0/r1/r2/r3 |
|---|---|---|---|---|---|
| 20261102 | OFF | **0** | 0 | 80 | 7 / 1 / 3 / 69 |
| 20261102 | **ON** | **7**（18.18k…79.68k） | 2 | **28** | 12 / 5 / 3 / 8 |
| 42 | OFF | **2** | 0 | 60 | 42 / 1 / 13 / 4 |
| 42 | **ON** | **6** | 2 | 36 | 26 / 3 / 6 / 1 |
| 20261109 | OFF | **3** | 0 | 34 | 8 / 2 / 24 / 11 |
| 20261109 | **ON** | **2** | 0 | 60 | 33 / 1 / 16 / 10 |
| **合计** | OFF 5 / **ON 15** | | | | |

读法：seed 20261102 的超级 deme（69 只）被打散成四区，首分叉 18.18k；seed 42 的 6 次里有 4 次是 22–34k 的新增；seed 20261109 在 80k 窗口略降（2 vs 3），但 200k 只差 1 次（§4）。有信号，上 200k。

## 4. 块 2：3 seeds × 200k × 双臂

| seed | 臂 | 物种形成 | 灭绝 | clades total/live | DECO@200k | structs | denv_var@end |
|---|---|---|---|---|---|---|---|
| 20261102 | OFF（Phase 59） | **0** | 0 | 2 / 2 | 80 | 2 029 | 0.04056 |
| 20261102 | **ON** | **9**（18.2/22.6/37.3/41.5/44.3/52.8/79.7/144.5/146.2k） | 4 | 10 / 6 | **29** | 3 729 | 0.01042 |
| 42 | OFF | **4**（3.2/9.5/160.3/176.5k） | 2 | 5 / 3 | 60 | 2 793 | 0.05435 |
| 42 | **ON** | **10**（3.2/22.3/29.8/32.7/33.7/67.9/154.3/160.4/162.7/166.4k） | 8 | 11 / 3 | 55 | 3 099 | 0.05653 |
| 20261109 | OFF | **4**（14.2/21.0/37.0/84.2k） | 0 | 5 / 5 | 34 | 3 560 | 0.06511 |
| 20261109 | **ON** | **3**（64.3/75.4/152.0k） | 1 | 4 / 3 | 60 | 3 051 | 0.06614 |
| **合计** | OFF **8** / **ON 22** | OFF 2 / ON 13 | | | | | |

- **物种形成 8 → 22**（旧 Phase 58 基线为 10）；三 seed 全部 ≥ 0，无归零。
- 灭绝 2 → 13：**不是生育断崖**。逐 clade 生命周期普查（§5）显示每个灭绝支系都有可育成年个体，寿命 16.4k–176.5k tick；Phase 58 的断崖灭绝是「出生即 `ready ≡ 0`，~5 980 tick 老龄灭绝」。
- seed 20261109 的物种形成 4 → 3、且推迟到 64k：这是该 seed 的盆地切换，如实记录，不用总量掩盖。

## 5. 个体级证据：低 eff 女儿支全程可育

### 5.1 200k 末存活支系（观测器 `deco_ready` 列，按 DECO 家系判定）

| seed | clade | 成员 | mean_eff | **deco_ready** |
|---|---|---|---|---|
| 20261102 | deco-A | 12 | 0.6918 | **3** |
| 20261102 | deco-B | 15 | 0.8401 | **2** |
| 20261102 | deco-F | 8 | 1.0525 | **1** |
| 20261102 | deco-H | 8 | 0.6752 | 0（末快照瞬时；生命周期普查 `ready_frac = 0.203`） |
| 20261102 | deco-A-I | 8 | 0.7233 | **1** |
| 42 | deco-G | 16 | 1.0258 | **4** |
| 42 | deco-I | 9 | 0.6488 | **3** |
| 20261109 | deco-A | 12 | 0.8064 | **2** |
| 20261109 | deco-A-C | 8 | 0.5636 | **1** |

`eff 0.564–0.723` 的支系仍开闸——这在 Phase 58 的旧线里是 `阈值 > 1` 的绝育区。

### 5.2 灭绝支系的生命周期普查（临时诊断器，每 600 tick 采样一次，已删除）

`ready_frac = 满足（成年 + 冷却完 + 能量 ≥ 个体阈值）的个体·样本 / 全部个体·样本`。

| seed | clade | 存活区间 | mean_eff | **ready_frac** |
|---|---|---|---|---|
| 20261102 | deco-C | 37.3k–61.4k | 0.888 | **0.160** |
| 20261102 | deco-A-D | 41.5k–72.5k | 0.803 | **0.093** |
| 20261102 | deco-E | 44.3k–176.5k | 0.717 | **0.078** |
| 20261102 | deco-G | 79.7k–145.1k | 0.813 | **0.114** |
| 42 | deco-A | 3.2k–83.3k | 0.764 | **0.068** |
| 42 | deco-B | 22.3k–78.8k | 1.042 | **0.186** |
| 42 | deco-C | 29.8k–67.0k | 1.186 | **0.385** |
| 42 | deco-D | 32.7k–167.2k | 0.930 | **0.226** |
| 42 | deco-E | 33.7k–134.0k | 0.905 | **0.189** |
| 42 | deco-F | 67.9k–168.9k | 0.737 | **0.106** |
| 42 | deco-H | 160.4k–180.0k | 1.214 | **0.313** |
| 42 | deco-J | 166.4k–182.9k | 1.200 | **0.393** |
| 20261109 | deco-A-B | 75.4k–132.3k | 0.753 | **0.055** |

灭绝原因是区域容量/竞争（寿命 16k+，全程有可育个体），不是「出生即绝育」。这是本期必须说清楚的地方：**我们消除了生育断崖，但没有消除灭绝**；容量驱动的支系消失是生态结果，13 起的绝对数偏高，列入 Phase 61。

## 6. 锚点全表（第十七次 re-baseline）

`deco_region_cap_off()` 逐位复现 Phase 59 默认世界：@60000 = `0xde2c6292e0dfb9dc`（`phase60_region_capacity_test` 的 ignored 臂，实测通过）。旧映射 + 新机制 off 复现 Phase 56 世界：@60000 = `0x44285bfbed94865e`（`phase59_repro_saturation_test`，实测通过）。

| 锚点 | Phase 59 | **Phase 60** | 状态 |
|---|---|---|---|
| @300（seed 20261102，v2） | `0x651cf47e6d54ca64` | **不变** | 容量闸未触发 |
| @3000（seed 20261102，v2） | `0xbeb50df36733a6e9` | **不变** | 同上 |
| @30000（seed 20261102，默认） | — | **`0x42e6da39caeecb24`** | 新 pin |
| @60000（seed 20261102，默认） | `0xde2c6292e0dfb9dc` | **`0x3272f0e3eaba9250`** | — |
| group-off @300 | `0x7d2c8613d01cb79d` | **不变** | 容量闸在此窗口未触发 |
| group-off @3000 | `0x2cd62c186e0941d6` | **不变** | 同上 |
| group-off seed 20261105 @300 | `0xbd6a86a9fe8264e7` | **不变** | 同上 |
| group-off @300 v1 | `0xc180a8890e5e4c5c` | **不变** | 同上 |
| partition-off @30000 | `0xd601816f29373fbd` | **不变** | deco_env off ⇒ 区域容量也 off |
| partition-off @60000 | `0x52c26d39e2f08629` | **不变** | 同上 |
| accum-off @30000 | `0xd601816f29373fbd` | **不变** | 同上 |
| accum-off @60000 | `0x29e3ebe0d592f8e3` | **不变** | 同上 |
| phase54 OFF（structure off + deco off）@60000 | `0x29e3ebe0d592f8e3` | **不变** | 同上 |
| phase56 OFF（deco off）@60000 | `0x29e3ebe0d592f8e3` | **不变** | 同上 |
| 中性奖励臂 @60000 | `0xd802b5b07aadfe4a` | **`0xf18fdad334f6e18d`** | — |
| 旧映射臂 @60000（新机制 ON） | — | **`0x5153690c11521a33`** | — |
| 旧映射臂 @60000（新机制 OFF，新 pin） | — | **`0x44285bfbed94865e`** | Phase 56/58 精确回放 |
| **新机制 OFF @60000（新 pin）** | — | **`0xde2c6292e0dfb9dc`** | Phase 59 默认精确回放 |

SaveState **仍是 v22**。`deco_region_cap` 与权重/格区表刻意不入档；恢复世界回到编译默认（ON），并从恢复的 `deco_env` 重导出权重（ON 双跑 + 存档锁步 1 200 tick 实测通过）。

## 7. 验收逐条

| # | 验收 | 结果 |
|---|---|---|
| 1 | 3 seeds × 200k：物种形成 ≥ 旧基线 10；生育断崖灭绝消失 | ✅ **22 ≥ 10**（102: 0→9，42: 4→10，109: 4→3；OFF 合计 8）。断崖灭绝 ✅：§5.2 全部 13 起灭绝支系 `ready_frac` 0.055–0.393，寿命 16.4k–176.5k；§5.1 低 eff 存活支系 `deco_ready > 0` |
| 2 | 双跑 bit-exact；存档往返；OFF 臂回放 | ✅ `phase60_region_capacity_test::on_arm_double_run_and_roundtrip_is_bit_exact`（60k 双跑 + 1 200 tick 锁步）；`off_arm_replays_the_phase59_default_anchor` = `0xde2c…`；旧映射 + OFF = `0x4428…` |
| 3 | 全量 0 失败（批 A + 慢测定向） | ✅ 见 §7.1 |
| 4 | wasm <16 ms 三次（同会话交错 A/B） | ✅ 见 §7.2 |
| 5 | 报告 | ✅ 本文件 |
| 6 | 默认决策 | **通过 → 默认 ON（第十七次 re-baseline）** |

### 7.1 全量与慢测

- 批 A：`cargo test --release --tests`（临时诊断器已删）→ **58 个套件，288 passed / 0 failed / 24 ignored**。
- 定向慢测：`phase60_region_capacity_test -- --ignored`（2 测）✅；`phase56_deco_resource_test -- --ignored`（2 测）✅；`phase57_token_sign_test -- --ignored`（signed + neutral + 双跑）✅；`phase59_repro_saturation_test -- --ignored`（60k 旧映射 + 新机制 OFF）✅。

### 7.2 wasm

- `wasm-release` **820 085 B**（Phase 59 为 818 675 B，**+1 410 B / +0.17 %**）；`ecology-sim.wasm` 与 `ecology-sim-v2.wasm` 的 sha256 相同：`100f31e449aa1163302a7f663823a2ce2c1fdc2355c7a89dfe9c875f1006952a`。
- `node living/test_wasm_viewer.mjs` 同条件交错 A/B（每轮先旧后新，3 轮）：

| 轮 | HEAD 旧 wasm | Phase 60 新 wasm |
|---|---|---|
| 1 | 12.26 ms/frame | **12.19 ms/frame** |
| 2 | 12.28 ms/frame | **12.23 ms/frame** |
| 3 | 12.22 ms/frame | **12.22 ms/frame** |

全部 < 16 ms；无回归（差异在测量噪声内）。

## 8. 改动文件

### 核心
- `core/crates/aether-sim-ecology/src/ecology.rs`
  - `:1255-1318`：Phase 60 机制文档、`DECO_REGION_CAP_ON/GAIN/W_MIN/W_MAX`、`build_deco_cell_region_map`
  - `:1918-1935`：`deco_region_cap` / `deco_region_weight[4]` / `deco_region_weight_sum` / `deco_region_weight_tick` / `deco_cell_region`
  - `:2155` / `:2380` / `:9290`：两个构造器 + 恢复路径的初始化
  - `:3770-3840`：`deco_region_cap_on/off`、`refresh_deco_region_weights`、`deco_region_cap_limit`、`deco_region_count`、诊断访问器
  - `:4156`：`spawn()` 区域容量闸门

### 观测器与测试
- `tests/clade_fate_observatory.rs`：`PHASE_DECO_REGION_CAP=on|off` 臂 + 文件名分量
- `tests/phase60_region_capacity_test.rs`（新，7 测：均匀分配、场权重、槽位闸门、不入档、OFF 锚点、ON 双跑+存档、生育映射不受影响）
- `tests/phase31_golden_regression.rs`（Phase 60 文档块；@300/@3000/v1 不变）
- `tests/phase39_1_transition_test.rs`（Phase 60 注释：group-off 锚点不变）
- `tests/phase51_niche_partition_test.rs`（旧映射 fixture 补 `deco_region_cap_off()`，三处）
- `tests/phase53_niche_accum_test.rs`（旧映射 fixture 补 `deco_region_cap_off()`）
- `tests/phase56_deco_resource_test.rs`（默认 @60000 → `0x3272…`）
- `tests/phase57_token_sign_test.rs`（signed → `0x3272…`，neutral → `0xf18f…`）
- `tests/phase59_repro_saturation_test.rs`（旧映射锚点补 `deco_region_cap_off()`，仍是 `0x4428…`）

### 产物
- `living/ecology-sim.wasm`、`living/ecology-sim-v2.wasm`（重建）

## 9. Phase 61 遗留

1. **灭绝率上升（2 → 13）需要定因**：本期证明了都不是出生即绝育，但 13 起里有多少是容量饥饿、多少是竞争排除，需要用引擎级逐 clade 出生/死亡计数器区分（Phase 59 §10.5 的建议仍未落地）。
2. **per-kind 区域份额对「只占一个区」的支系偏严**：`cap = budget/Σw · w_r` 意味着单区支系最多只能用到 budget/4。可以试「按已占据区域重归一化」或「family 级区域 cap + 女儿最低保留份额」，看能否在保住分化率的同时降低灭绝。
3. **GAIN/边界标定**：GAIN=10、[0.25, 4.0] 是首轮工作点；3 seed 不够（Phase 59 §10.4 要求 ≥5 seed），且 42/109 方向不一致。
4. **niche 构造反馈**：Phase 54 的 `structure_env` 仍是 OFF；它和 `deco_env` 的区域容量可以合并成一条「文化 → 承载力」通道，但本期不做。
5. **wasm 成本**：每 tick 首次 DECO 出生闸门做 5 776 次表查找（无 atan2）。§7.2 已测；若未来把权重改成逐 tick 全量，会进入 wasm 预算。
