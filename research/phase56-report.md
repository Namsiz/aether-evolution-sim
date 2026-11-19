# Phase 56 — 文化直连 DECO 资源面：分解场让结构首次「换出」分化（默认 ON，第十五次 re-baseline）

日期：2026-11-19
HEAD（起点）：`dd27a60`（Phase 55 验收修正）
性质：**机制交付 + 标定通过**——结构效果改接 DECO 直接食物链（carcass 分解速率场）；默认 ON；SaveState v21；第十五次 re-baseline。

---

## 1. 调研结论：为什么是 carcass 分解，不是 mineral

DECO 的资源面有两条通路，逐条核查代码后：

| 通路 | 代码位置 | 结论 |
|---|---|---|
| **carcass 分解** | `ecology.rs` 的 carcass pass：`carcasses[i].energy -= 1.4 * h`（全局常数，无空间分辨率）；DECO 摄入 `update_deco` 三态机（search→approach→eat），`take = min(2.0, carcass.energy)` | **未被打过任何文化烙印**；尸体寿命 = DECO 的食物可得性；完全绕过 HERB 营养级联 |
| **mineral_field** | `mineral_field[g] += take × deco_eff × 0.008`；`MINERAL_DECAY=0.9995`；`cell_grow × (1 + mineral × 0.3)` | 矿化**唯一的下游是草生长**（Phase 54/55 已证伪的毒药路径）+ abiogenesis 热点判定；且 `niche_pref_mult` 用 `(grass−mineral)/(grass+mineral)` 定侧向，改 mineral 会顺带改草 |
| niche 偏好轴 | `niche.rs:niche_pref_mult`：`rel = (g−m)/(g+m)`，banded clade 在矿物富集格 +15%、草富集格 −15% | 是 DECO 适应度面，但**其资源就是 grass/mineral 场**，改它就等于改 Phase 54/55 的场 |

**选 carcass 分解场的理由**：

1. `1.4/tick` 是全局常数——**是全世界唯一一个还完全没有空间结构的资源速率**，加场是纯增量，零既有语义冲突；
2. 它是 DECO 的**直接食物链**（尸体在格子里腐烂，DECO 到得早才有肉），不经 HERB；
3. mineral 那条路的两个下游（草生长、矿物丰度侧向）都属于结构已经涉足的场域，重蹈覆辙的概率高；
4. 尸体的寿命是**离散的、可数的**（每具尸体约 15–40 tick），局部乘子 0.5/1.5 直接改变 DECO 的觅食窗口宽度，是一个干净的「觅食策略选择压力」探针。

## 2. 设计：结构印记的净零分解偶极

- `deco_env: Vec<f32>`（76×76，每格尸体腐烂速率乘子，初始 1.0）
- 放置结构时调用 `deco_env_dipole(x, z, grass_mod)`：
  - 方向来自 token 的 `grass_mod` 符号（文化身份），幅度乘 `STRUCTURE_DECO_GAIN`：
    - **富集 token（grass_mod>0）→ 盘面分解变慢**（结构化土壤把尸体留存给它的分解者 = 文化粮仓），**环带变快**（偿还）
    - **贫瘠 token → 盘面变快**（尸体迅速消失 = 文化荒漠），环带变慢
  - 盘面 Chebyshev 半径 3（49 格），补偿环 3..5（72 格），clamp `[0.5, 1.5]`，`STRUCTURE_DECO_DECAY=0.998`/tick（半衰期 ≈350）
- **净零是精确的**（比 Phase 55 更严）：先测补偿环在偿还方向的**剩余容量**，把盘面印记按「环带能支付的量」缩放，再偿还；环带被 clamp 截断的份额继续摊给还有余量的环格。Phase 55 的近似版本在 gain=10 时把场均拖到 0.99279（−0.7%），在食物供给场上这就是**全局分解速率变化**，正是净零契约要禁止的东西；本期的实测 `denv_mean ≡ 1.00000`（每 5k 采样，全部七个臂）。
- 关键设计问题的回答（「如何不重蹈可达性→稳定化」）：
  - 不追求「更多食物」。同一份尸体供给下，结构只决定**这份食物在该格能存活多久**；
  - 场是**双向撕裂**的：盘面与环带必然给出相反的生命史，一片区域对「快、感知远」的分解者有利，紧邻区域对「慢、代谢省」的有利；
  - 压力落在**检测器测量的轴上**（speed/sense/size/deco_eff），不是落在总量上；
  - 但实测（见 §3）表明真正的驱动不是符号分化，而是**对比度饱和**——这一点老实记录。

## 3. 标定（80k，seed 20261102；OFF 臂 = 2 次物种形成）

| 臂 | GAIN | events | clades@80k | deco@50k | deco@80k | denv_var@50k | 判读 |
|---|---|---|---|---|---|---|---|
| OFF | — | **2**（34.68k/41.76k） | 3 | 60 | 60 | 0.00000 | 基线复现（与 Phase 55 数值逐 tick 一致） |
| g3 | 3 | 0 | 1 | 80 | 80 | 0.02334 | DECO 撞 80 上限 → 更稳 → 零分化 |
| g10 | 10 | 0 | 1 | 80 | 80 | 0.04625 | 同上（Phase 54/55 失败模式在食物寿命轴上重演） |
| g20inv | 20 反符号 | 0 | 1 | 80 | 80 | 0.06191 | 方向反了 → 回到「补贴 DECO」→ 零分化 |
| **g20** | **20** | **3**（23.70k/39.54k/76.38k） | **4** | 45 | 34 | 0.01039 | **通过**，DECO 掉出上限、区域化（r0 在 80k 归零） |

**剂量-反应是反向的**：温和镶嵌（g3/g10，全场 var 0.02–0.05）把 DECO 送到 80 槽上限，种群比 OFF 更稳；只有饱和对比（gain=20 → 每次放置把盘面直接钉在 0.5×/1.5× 边界）才让 DECO 从 80 掉到 45→34，并出现区域空缺。符号必须「自然方向」（富集=粮仓）；反符号的等价强度（g20inv）零分化。

## 4. 200k 探针

| 臂 | 物种形成事件 | 灭绝事件 | 终局 clades | 终局 deco | denv_mean | denv_var@end |
|---|---|---|---|---|---|---|
| OFF | **4**（34.68k/41.76k/125.94k/127.98k） | 0 | 5 | 34 | 1.00000 | 0.00000 |
| ON (g20) | **4**（23.70k/39.54k/76.38k/157.14k） | 1（t=82.0k，clade 3） | 4 | 45 | 1.00000 | 0.01011 |

- ON 臂的**首次分叉提前 11k tick**（23.7k vs 34.7k），第二次提前 2.2k，第三次提前 49.6k；
- ON 臂多出一次**灭绝事件**（全观测期内 OFF 从未产生）——结构压力下 clade 真的会死，这是 Phase 49 灭绝归还预算第一次被文化间接点燃；
- 终局 DECO 数量 ON 45 > OFF 34 但 clades 4 < 5：ON 的世界「更少种类、更多个体」，分化不是靠饥饿堆出来的。

## 5. 机制判断（诚实版）

1. **符号分化（设计 b）没有被证实是驱动**：80k 时站立结构的 token 符号分布是 pos=1809 / neg=81（Phase 45 的价值选择本就偏爱正 grass_mod token，因为放置即时能量 `gain = grass_mod × 10`）。所以盘面大多是「粮仓」、环带是「荒漠」；clade 之间的符号差异不足以解释分化。
2. **贫瘠斑块（设计 a）是真实成分**：72 格的环带比 49 格的盘面大，且环带互相重叠——世界里真正被撕成极端的是「环」，即大片快速腐烂区。DECO 在这些区间里觅食窗口被砍掉 1/3。
3. **真正被验证的是「对比度阈值」**：gain 3/10 的场 var 更大（0.023/0.046）但 DECO 反而更富（80 上限）；gain 20 的场 var 反而更小（0.010）却让 DECO 掉档。**决定命运的不是场的方差，而是乘子是否撞到基于生理的边界**（0.5×/1.5×）。这与 Phase 55 的「近惰性扰动也错过首分叉」互补：混沌敏感的是轨迹，生理边界敏感的是种群的承载力结构。
4. **违反直觉但重要**：Phase 54/55 的结论「可达性↑→稳定化」在本期被反向确认——可及食物的**寿命**被切碎，等价于可达性↓，而可达性↓换来的是分化。文化第一次「减少」而不是「增加」了世界的可居住性，世界反而活了。

## 6. 验收逐条

| # | 验收 | 结果 |
|---|---|---|
| 1 | 80k ON ≥ 1（OFF 的 2 的一半） | ✅ g20 = 3 |
| 1b | 200k ON ≥ 2 | ✅ g20 = 4（+1 灭绝） |
| 2 | 双跑 bit-exact / 存档往返 | ✅ phase56 慢测（ON 臂双跑 + 60k 存档→恢复锁步 1200 tick）；v21 存档 |
| 2b | A/B off 恢复 Phase 55 轨迹（全锚点不变） | ✅ @300 `0x651cf47e6d54ca64`、@3000 `0xc94fc584ddfd6eb3`、partition-off@60000 `0xf5bdea460cd4685a`、accum-off@60000 `0xbf91125e40589e1e`、phase54 OFF@60000 `0x3fff60ac98f9b8c5`（后三者现在由显式 `deco_resource_env_off()` 钉住） |
| 3 | release 全量 0 失败 | 见 §8 |
| 4 | wasm + Node <16ms 三次 | 见 §8 |
| 5 | 报告 | 本文件 |

## 7. 锚点记录（第十五次 re-baseline）

| 锚点 | 旧值（Phase 55） | 新值（Phase 56 默认 ON） |
|---|---|---|
| @300 (seed 20261102) | `0x651cf47e6d54ca64` | **不变** |
| @3000 (seed 20261102) | `0xc94fc584ddfd6eb3` | **不变**（结构在 3000 前尚无观测效果） |
| group-off @3000 | `0xdb3aa647fa2e3a96` | `0x08d4191f98770182`（group-off 臂更早产出 token，3000 前已有印记） |
| group-off @300 | `0x7d2c8613d01cb79d` | 不变 |
| partition-off @60000 | `0xf5bdea460cd4685a` | 不变（该臂显式关掉分解场） |
| accum-off @60000 | `0xbf91125e40589e1e` | 不变（同上） |
| phase54 OFF @60000 | `0x3fff60ac98f9b8c5` | 不变（同上） |
| **默认世界 @60000** | — | **`0x44285bfbed94865e`**（新锚点，2 次 clade 事件） |

## 8. 全量与性能

- **批 A（release，`cargo test --release --tests`）**：42 个测试二进制 **197 passed / 0 failed**（Phase 55 为 189，本期新增 phase56 的 8 个非 ignored 测试）。
- **慢测（`--ignored`）**：phase51/53/54/55/56 全绿——phase55 异质性 A/B 2/2（两个臂现在都显式关掉分解场，隔离回归场）、phase56 ON 臂双跑 + 60k 存档→恢复 1200 tick 锁步 2/2、OFF 臂 `0x3fff60ac98f9b8c5` 不变。
- **wasm**：`wasm-release` 816,375 B（Phase 55 为 813,054 B，+3.3 KB）。`node living/test_wasm_viewer.mjs` 连测三次 14.82 / 11.00 / 12.77 ms/frame（全部 <16），同机 HEAD 旧二进制对照 10.81 / 9.82 / 14.94 ms/frame——**同噪声带、无回归**。本机（PRoot/Android）测量噪声大：同一二进制在同一时段的采样跨 8.6–20.9 ms，因此以「新旧同龄交替测量 + 三次连续达标」为准。

## 9. 改动文件

- `core/crates/aether-sim-ecology/src/ecology.rs`
  - Phase 56 常量区（:1164-1242）：`STRUCTURE_DECO_RADIUS/RING/DECAY/GAIN/MIN/MAX`、`DECO_RESOURCE_ENV_ON`
  - 字段 `deco_env / deco_env_on / deco_env_decay_active`（:1811）+ 两个构造函数初始化（:2032/:2247）
  - `deco_resource_env_off()` A/B 臂开关（:3604）
  - `deco_decay_mult_at()`（:3671）：格点乘子读取，A/B 关闭时恒 1.0
  - `deco_env_dipole()`（:3695）：精确保守的净零偶极（先测环容量→缩放盘面→偿还+溢出再分配）
  - 衰减 pass（:6162）、放置点 hook（:6398）、carcass 衰减应用（:6447）
  - `deco_env_field()` 只读访问器（:7431）、存档写入/读回（:8286/:8972）
- `core/crates/aether-sim-ecology/src/save_state.rs`：`DecoEnv/DecoEnvOn/DecoEnvDecayActive`（:1041-1046）、`SAVE_STATE_VERSION = 21`（:126）、v20→v21 版本史（:115）
- `core/crates/aether-sim-ecology/tests/phase56_deco_resource_test.rs`（新，10 测：净零/钳位饱和/尸体衰减读场/A-B 关即平坦/符号印记/存档兼容/新默认锚点 + 2 慢测）
- `core/crates/aether-sim-ecology/tests/clade_fate_observatory.rs`：`PHASE_DECO_ENV=on|off` 臂 + `denv_mean/denv_var` 列（:96/:109/:139）
- re-baseline 相关测试：`phase39_1`（group-off @3000 新值 + v21 断言）、`phase41/42`（v21 断言）、`phase51/53/54`（锚点臂显式 `deco_resource_env_off()`）、`phase55`（两臂隔离分解场）
- `core/crates/aether-sim-ecology/tests/phase39_1_transition_test.rs:133` 等锚点更新
- `living/ecology-sim.wasm` / `living/ecology-sim-v2.wasm`（重建）
- `research/phase56-report.md`（本文件）

## 10. Phase 57 遗留

1. **符号分化真的没被检验**：本期 pos/neg = 1809/81，符号维度事实上退化了。要检验「clade 差异化」（设计 b），必须先把 token 价值函数从 `gain = grass_mod × 10` 解耦（现在它把正符号 token 垄断成文化默认），否则任何符号设计都会塌缩成正号单一文化。
2. **对比度阈值的普适性**：gain=20 是通过点，但是否存在「刚好撞边界」的更弱点（例如 clamp 收窄到 [0.6,1.4]）？若 0.6/1.4 也能通过，说明关键是「撞边界」而非「边界多深」。
3. **多 seed**：本期仍只有一个 seed，且 80k ON 的 3 次分叉里最大 dist=0.392（远高于 0.25 阈值），与 OFF 的 0.257/0.305 相比更「结实」，但仍未做 3 seeds × 200k。
4. **灭绝事件的因果**：t=82k 的 extinction 是 clade 3（deco-C，1 成员）自然消亡，还是结构压力的直接结果？需要把 demography 与场值做相关性分析。
5. **mineral 方向**（调研里被排除的那条）仍未被证伪，只是被判定为「高重蹈概率」。
