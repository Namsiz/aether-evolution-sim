# Phase 57 — 多 seed 验证 + 符号分化解耦（跨 seed 成立；解耦实验失败并回退）

日期：2026-11-19
HEAD（起点）：`eb2d6e3`（Phase 56，第十五次 re-baseline，默认 ON）
性质：**验证 + 失败实验记录**——Phase 56 的分解场分化发现跨 seed 成立；token 价值符号解耦被证实反而摧毁分化，默认关闭，**第二次没有 re-baseline**（默认轨迹与 Phase 56 bit-exact）。

---

## 1. 块 1：3 seeds（+1 补充）× 200k × 双臂

方法：`clade_fate_observatory`（`PHASE_SEED` / `PHASE_TICKS=200000` / `PHASE_DECO_ENV=on|off`），同一 release 二进制、6+2 进程并发，每臂独立进程。denv 统计每 5k 采样。

| seed | 臂 | 物种形成事件 | 灭绝事件 | 首分叉 | clades total/live | DECO@200k | denv_mean@end | denv_var@end | structs@end |
|---|---|---|---|---|---|---|---|---|---|
| 20261102 | OFF | **4**（34.68k/41.76k/125.94k/127.98k） | 0 | 34 680 | 5/5 | 34 | 1.00000 | 0.00000 | 3281 |
| 20261102 | ON | **4**（23.70k/39.54k/76.38k/157.14k） | 1（t=81.96k） | 23 700 | 5/4 | 45 | 1.00000 | 0.01011 | 2302 |
| 42 | OFF | **0** | 0 | — | 1/1 | 80（上限） | 1.00000 | 0.00000 | 2258 |
| 42 | ON | **2**（6.60k/33.12k） | 1（t=116.34k） | 6 600 | 3/2 | 60 | 1.00000 | 0.05990 | 2609 |
| 20261105 | OFF | 0 | 1（t=4.02k） | — | 1/0 | 0 | 1.00000 | 0.00000 | 3274 |
| 20261105 | ON | 0 | 1（t=4.02k） | — | 1/0 | 0 | 1.00000 | 0.03727 | 2417 |
| **20261109**（补充） | OFF | **1**（t=2.04k，其女支 t=3.90k 灭绝） | 1 | 2 040 | 2/1 | 80（上限） | 1.00000 | 0.00000 | 4417 |
| **20261109**（补充） | ON | **4**（27.84k/31.68k/150.30k/153.18k） | 0 | 27 840 | 5/5 | 34 | 1.00000 | 0.05526 | 3731 |

### 判据与结论

任务判据：「ON 臂在 ≥2/3 seeds 上 events ≥ OFF 臂且 denv_mean ≡ 1.00000」。

- 20261102：4 vs 4（平手，但 ON 首分叉提前 11k、多 1 次灭绝）→ 计入「≥」
- 42：2 vs 0 → 计入「≥」（且 OFF 是历史零分化 seed，ON 把它变成有分化的世界）
- 20261105：0 vs 0（**退化 seed**，不计入有效样本，见下）
- **结论：跨 seed 成立**，且强于「不劣于」——在两个 OFF 几乎不分化（42：0 次；09：1 次且其女支随即灭绝）的 seed 上，ON 分别造出 2 次和 4 次物种形成。

**denv_mean 在全部 8 个臂、全部 5k 采样上恒为 1.00000**：净零偶极的守恒契约在 200k 尺度、4 个 seed、两种 DECO 臂上都没有漂移。这是比 Phase 56 单 seed 更强的守恒证据。

### 20261105 为何退化（如实记录）

两臂都在 t=4020 出现一次 `event_kind=1`（灭绝）：DECO 根 clade 0 整体消亡（`clade_tree` 里 clade 0 的 death=4020，`deco` 列在 5k–195k 采样上恒为 0，200k 时 OFF 臂才由 abiogenesis 重新造出 52 个 DECO）。DECO 是唯一会分叉的支系，根死了就没有任何物种形成事件的载体，因此该 seed 对「分化」假设**不可测**，两臂都是 1 个事件（灭绝本身），无法区分。

补充 seed 20261109（`phase32_open_loci_test` 已在用的 5-seed 池中的一个，此前未做过观测）取代它的第三证人位。选它而不是随意造一个 seed，是为了让补充样本仍在项目既有 seed 池内、可被其它测试复现。

**没有 1/3 降级的必要**：两个有效 seed 都指向同一方向，且第三有效 seed（补充）方向更强。

---

## 2. 块 2：符号分化解耦（实验，判定失败，默认关闭）

### 2.1 改动

- `culture::TOKEN_VALUE_SIGN_NEUTRAL`（默认 **false**）+ `Ecology::token_value_sign_neutral` 字段
- 放置反馈：签名臂 `gain = grass_mod × 10`（Phase 56），中性臂 `gain = |grass_mod| × 10`（`ecology.rs` 放置点）
- A/B 臂：`Ecology::token_value_sign_neutral_on()`；观测器 `PHASE_TOKEN_SIGN=neutral|signed`
- SaveState v22：`TokenValueSignNeutral`（`#[serde(default)]` → false，即 Phase 56 世界）

解耦的动机成立：旧公式给正号 token 每次放置 +1 能量的净优势（放置成本 3.0，正号返还 +0.5、负号 −0.5，差 1.0），价值选择因此把负号文化逐出。

### 2.2 A/B（200k，DECO 场 ON，seed 20261102 / 42）

| seed | 臂 | 物种形成 | 灭绝 | clades total/live | DECO@200k | denv_var@end | structs | 结构符号 pos/neg |
|---|---|---|---|---|---|---|---|---|
| 20261102 | 签名（默认） | **4** | 1 | 5/4 | 45 | 0.01011 | 2302 | 2187 / 115 |
| 20261102 | 中性 | **0** | 0 | 1/1 | 80（上限） | 0.07925 | 4416 | **0 / 4416** |
| 42 | 签名（默认） | **2** | 1 | 3/2 | 60 | 0.05990 | 2609 | 2605 / 4 |
| 42 | 中性 | **0** | 0 | 1/1 | 80（上限） | 0.08594 | 4542 | **0 / 4542** |

per-clade 文化符号（`clade_culture_*.csv`，`frac_grass_positive` = 持有 token 中 grass_mod>0 的比例）：

| seed | 臂 | per-clade frac_positive | 判读 |
|---|---|---|---|
| 20261102 | 签名 | 0.115 / 0.489 / 0.214 / 0.165 / 0.174 | clade 间有 4 倍落差（DECO 根 0.489 vs 其它 ~0.17） |
| 20261102 | 中性 | 0.000 / 0.000 / 0.000 / 0.000 | 全部归零 |
| 42 | 签名 | 0.498 / 0.399 / 0.255 / 0.354 / 0.534 | 有落差 |
| 42 | 中性 | 0.053 / 0.000 / 0.183 / 0.000 / 0.000 | 基本归零 |

### 2.3 结论：解耦既没有平衡符号，也摧毁了分化

1. **符号没有趋向 50/50，反而被发明先验锁死在负号单一**。`try_invent_token` 用 `vec[0] = (grass×2−1)`（局部立草量）作为发明种子；立草全场绝大多数格子在 0.5 中点以下，所以新发明的 token 天然是负号。签名公式下价值选择用 +1 能量的净优势把这股先验扳成正号垄断（Phase 56：pos 2187/neg 115）；把收益解耦后，**没有任何力量再对抗发明先验**，文化直接塌缩到先验那一极（pos 0/neg 4416，`frac_positive` 全 0）。两种臂都不是「平衡」，而是两种不同方向的单一化——符号单一化的根源在发明先验 + 场异质性，不在奖励差。
2. **分化彻底消失**：2/2 seeds、200k 内 0 次物种形成（签名臂 4/2 次）。机制上与 Phase 54/55 的失败模式完全同构——中性臂 DECO 在两端都卡回 80 上限（签名臂 45/60），区域空缺消失（`deco_r*` 分布均匀），即「饱和度 → 稳定化」。
3. **反直觉但重要**：中性臂的场方差反而更大（0.079/0.086 vs 0.010/0.060），但分化更少。这复证了 Phase 56 的判读——**决定命运的不是场的方差，而是乘子是否撞到生理边界并把 DECO 从 80 上限打下来**。中性臂虽然「更不均匀」，但所有结构的符号一致（全负号），等于全场朝同一方向推，没有任何区域被推到阈值之外；签名臂的单一正号（粮仓）反而在每次放置时把盘面钉到 1.5× 边界、环带钉到 0.5×，制造出真正的区域级撕裂。
4. **机制含义**：Phase 56 的「符号垄断」不是偏差，是分化机制的必要成分——文化必须先收敛到一个**相干方向**（正号粮仓），才能把空间对比推到生理边界。符号多样化（设计 b）在「奖励解耦」这一路径上被证伪；若仍要检验设计 b，必须换一条不依赖「符号自由选择」的路径（见 §6）。

按任务约定：**实验失败 → 默认保持签名（Phase 56）轨迹，不 re-baseline**。中性命中值仅作为实验臂锚点保留在测试里。

---

## 3. 验收逐条

| # | 验收 | 结果 |
|---|---|---|
| 1 | 块 1：3 seeds × 200k 双臂数据表（6 臂） | ✅ §1（另加 20261109 补充 2 臂，共 8 臂） |
| 2 | 块 2：A/B 数据 + 符号分布 + per-clade 文化符号差异 | ✅ §2.2/§2.3（结论：解耦失败，默认回退） |
| 3 | 双跑 bit-exact | ✅ `phase57` 慢测：签名默认臂 @60000 = Phase 56 锚点 `0x44285bfbed94865e`；中性臂双跑同一 checksum；v21→v22 存档恢复两跑一致 |
| 3b | 全量 0 失败 | ✅ 见 §5 |
| 4 | wasm <16ms 三次 | ✅ 见 §5 |
| 5 | 报告 | 本文件 |
| 6 | 任何 re-baseline 必须全锚点更新记录 | ✅ **本期无 re-baseline**：默认轨迹与 Phase 56 bit-exact，全部锚点不变（§4） |

---

## 4. 锚点记录（无 re-baseline）

| 锚点 | Phase 56 | Phase 57 | 说明 |
|---|---|---|---|
| @300 (seed 20261102) | `0x651cf47e6d54ca64` | **不变** | 默认臂未改 |
| @3000 (seed 20261102) | `0xc94fc584ddfd6eb3` | **不变** | 同上 |
| group-off @300 | `0x7d2c8613d01cb79d` | **不变** | 同上 |
| group-off @3000 | `0x08d4191f98770182` | **不变** | 同上 |
| partition-off @60000 | `0xf5bdea460cd4685a` | **不变** | 同上 |
| accum-off @60000 | `0xbf91125e40589e1e` | **不变** | 同上 |
| phase54 OFF @60000 | `0x3fff60ac98f9b8c5` | **不变** | 同上 |
| 默认世界 @60000 | `0x44285bfbed94865e` | **不变** | 默认 = 签名臂 |
| 中性臂 @60000（实验臂，新） | — | `0x47acbccd66a5fdab` | 仅作实验臂参考，不影响默认轨迹 |

SaveState：v21 → **v22**（新增 `TokenValueSignNeutral`，`#[serde(default)]` → false；旧档加载后落在 Phase 56 世界）。

---

## 5. 全量与性能

- **批 A（release，`cargo test --release --tests`）**：43 个测试二进制 **200 passed / 0 failed / 21 ignored**（Phase 56 为 197 passed；本期新增 phase57 的 3 个非 ignored 测试）。
- **慢测**（定向 `--ignored`，phase56 报告的慢测集合 + 本期新增）：
  - phase55 异质性 A/B 2/2、phase56 ON 臂双跑 + 存档恢复 2/2、phase57 签名锚点 + 中性臂双跑 1/1（**917s**，中性臂 @60000 = `0x47acbccd66a5fdab`，签名默认臂 = Phase 56 锚点 `0x44285bfbed94865e`）
  - phase41 探针 4/4、phase42 长程 2/2、phase31 tick-cost 1/1、save_restore 诊断 1/1
  - phase51/53/54 无 ignored 测试（其慢测就是批 A 里的 60k 锚点，已全绿）
  - **注**：`cargo test --tests -- --ignored` 会连带拉起 `clade_fate_observation` / `longmill_observatory`（默认 1M tick / 约 70 分钟）等观测器，故慢测按测试二进制定向执行，不跑全量 `--ignored`
- **wasm**：`wasm-release` 816,741 B（Phase 56 为 816,375 B，+366 B）。`node living/test_wasm_viewer.mjs` 连测三次 **12.20 / 12.20 / 12.18 ms/frame**（全部 <16），机器空载时段测量；`node living/test_wasm_consumer.mjs` ABI 冒烟正常（grass total 1161.91）。

---

## 6. 改动文件

### 核心
- `core/crates/aether-sim-ecology/src/culture.rs:20-28`：`TOKEN_VALUE_SIGN_NEUTRAL`（默认 false）+ 失败判决记录
- `core/crates/aether-sim-ecology/src/ecology.rs:1814-1820`：`token_value_sign_neutral` 字段；`:2040`/`:2256` 两个构造器初始化
- `core/crates/aether-sim-ecology/src/ecology.rs:3616-3623`：`token_value_sign_neutral_on()`（实验臂开关）
- `core/crates/aether-sim-ecology/src/ecology.rs:6415-6423`：放置收益分支（签名 `grass_mod × 10` / 中性 `|grass_mod| × 10`）
- `core/crates/aether-sim-ecology/src/ecology.rs:8312`/`:9003`：存档写入/读回
- `core/crates/aether-sim-ecology/src/save_state.rs:122-124`（v21→v22 版本史）、`:129`（`SAVE_STATE_VERSION = 22`）、`:1049-1054`（`TokenValueSignNeutral`）

### 测试
- `core/crates/aether-sim-ecology/tests/phase57_token_sign_test.rs`（新，4 测：默认签名 + A/B 往返、v21→v22 存档落点、价值 EMA 符号不变式、慢测锚点）
- `core/crates/aether-sim-ecology/tests/clade_fate_observatory.rs:103-110`（`PHASE_TOKEN_SIGN` 臂）、`:113-115`（文件名加臂后缀）、`:124`（臂开关）
- v22 断言更新：`tests/phase39_1_transition_test.rs:423`/`:468`、`tests/phase41_triune_test.rs:121`、`tests/phase42_brain_authority_test.rs:367`

### 产物
- `living/ecology-sim.wasm`、`living/ecology-sim-v2.wasm`（重建）
- `research/phase57-report.md`（本文件）

---

## 7. Phase 58 遗留

1. **设计 b 未死透，但需要换路径**：本期只证伪了「奖励解耦 → 符号自由选择」这一条。剩下的可能：(a) 让结构放置使用**价值最高的 token** 而不是 `known_tokens[0]`（当前放置只用首 token，符号多样性被继承顺序压死）；(b) 让放置成本与 |grass_mod| 脱钩、只保留环境效果差；(c) 直接对「符号」加频变选择（同一 clade 内负号稀有时负号得溢价）——但这是预置语义，与「无预设语义」的项目原则冲突，慎用。
2. **符号在发明先验上被草量锁死**：`vec[0] = 2·grass−1` 与立草分布（几乎处处 <0.5）耦合，意味着「负号 = 常见」是模型常量而非涌现。若设计 b 要做，先要问：这个先验是想要的吗？改成「以 0.5 为中心化立草」会同时改变 4 个 seed 的符号分布。
3. **灭绝事件的因果**仍是 Phase 56 遗留：本期的 3 次灭绝（02 ON t=81.96k、42 ON t=116.34k、09 OFF t=3.90k）都没做 demography × 场值的相关性分析。
4. **对比度阈值的普适性**（0.6/1.4 clamp）未测。
5. **20261105 的 DECO 根灭绝**（t=4020，两臂一致）本身是独立现象：它让该 seed 完全失去分叉载体，值得单独查一次（是溃败还是结构压力？）。
