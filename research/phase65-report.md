# Phase 65 — 建支宽限期（W1）：预注册与 5 新 seed A/B

日期：2026-11-22
HEAD（起点）：`a89b678`（Phase 64：M10 回退 `FORK_BUDGET_MIN = 8`，第十九次 re-baseline）
性质：**预注册的单机制 A/B**。目标失败模式是 Phase 62 §10.2 定义的「建支即失稳」：fork 通过 `MIN_SIDE = 8` 后，`peak ≤ 12 ∧ region_cap_blocks = 0 ∧ kind_budget_blocks = 0 ∧ 生存 < 10 000` 即耗散。已记录两起：Phase 62 §4 #15（42 / b 臂，t=21 540 建支、3 960 tick 耗散、6 出生 / 14 死亡）与 Phase 63 §5 #11（102 / M10 臂，t=98 450、peak 8、8 710 tick）。

> 本文件的第一版只包含 §1 预注册，在第一个 Phase 65 实验进程启动之前单独提交。运行期间 §1 不做任何修改；后续各节只追加。§1.1 的机制读法与 §1.7 的「无作用力判据」同样是预注册的一部分：本期的核心问题不是「W1 能不能把灭绝刷低」，而是「建支即失稳 是否由 kind 预算不足造成」。

---

## 1. 预注册（跑之前写死）

### 1.1 机制读法（为什么本期必须先问「闸是否真的拒绝过」）

Phase 62/63 的定因表已经把目标类钉死：两起案例的 `kind_budget_blocks = 0`、`region_cap_blocks = 0`。这不是巧合，而是定义的直接后果——「建支即失稳」之所以成立，就是因为**两道容量闸都没有拦过它**。据此可以写出一个代码级事实：

- W1 只在一个出生被 kind 预算闸拒绝时改变世界（`spawn_tagged` 的 `count(kind) >= eff_cap` 分支）。
- 目标类的 `kind_budget_blocks` 恒为 0，即该闸从未拒绝过；因此 W1 **在原理上救不了任何一起已记录的建支即失稳**。
- W1 仍可能通过两种次要渠道产生净效应：(i) 挽救「本来会被预算闸拒绝、随后会死」的其它女儿（它们目前的死因记为容量饥饿，不是建支即失稳）；(ii) 结算把槽位从父系划走后，父系更早满容，改变整条世界线。

所以本期把「W1 是否真的产生超编出生」作为**一等判据**（§1.7 D0），而不是事后解释。若全池 `grace_skips = 0`，则「建支即失稳 = 预算不足」这一假设被否证，默认保持 OFF，结论写成「约束在招募（择偶/扩散），不在容量」——这是本期的有效结论，不是失败运行。

### 1.2 seed 池（机械派生，全新池）

**生成器**：日期编码 `2026_DD` = `20261100 + DD`，DD 从 `16` 起、到本期执行日 `2026-11-22` 的 `22` 止，升序枚举。

**过滤（按序）**：

1. Phase 64 池（`20261110/11/12/14/15`）**不得复用**——它虽只被用于否定 M10，但已被写入判定记录；
2. Phase 57–63 判定池（`20261102 / 42 / 20261109 / 20261105 / 20261107`）不得复用；
3. 仓库内任何文件命中即剔除（`grep -rIl <seed> --exclude-dir={node_modules,target,.git}`）；
4. 取前 5 个。

**预注册时实测（6 个全部 0 命中）**：

| 位 | seed | 命中文件数 | 结论 |
|---|---|---|---|
| 1 | 20261116 | 0 | ✅ 判定 |
| 2 | 20261117 | 0 | ✅ 判定 |
| 3 | 20261118 | 0 | ✅ 判定 |
| 4 | 20261119 | 0 | ✅ 判定 |
| 5 | 20261120 | 0 | ✅ 判定 |
| 备 | 20261121 | 0 | 退化替换第 1 顺位 |
| 备 | 20261122 | 0 | 退化替换第 2 顺位 |

这 5 个 seed 在仓库中从未出现过（含全部历史报告与测试），因此对本期的任何臂都是 out-of-sample。**禁止在看到结果后追加、替换或删除 seed**；唯一例外是 §1.8 的退化替换规则。

### 1.3 W1 设计（语义写死，含边界情况）

#### 1.3.1 状态与开关

| 项 | 值 | 说明 |
|---|---|---|
| `FORK_GRACE_ON` | `false` | 编译默认 OFF；转正前不改变默认世界 |
| `FORK_GRACE_TICKS` | `5_000` | 固定常量，不做基因化：基因化会新增可遗传状态与存档字段，且 5-seed A/B 无法标定其适应度 |
| `FORK_GRACE_OVERDRAFT_MAX` | `8` | 超编上限 = 一个最小可育群体（`SPECIATION_MIN_SIDE`）；父系 fork 后剩余必 ≥ `FORK_BUDGET_MIN = 8`，故无后续 fork 时必然付得起；与 Phase 63 G 臂的 `grant + 8` 上界一致 |
| 状态字段 | `kind_grace_until: Vec<u32>` | 按 kind id；0 = 无宽限。fork 时对**派生 kind** 写 `now + 5_000` |
| 作用域 | 全族 | 与 `fork_budget_min` 一致（全局）；报告单列非 DECO 支系的连带变化（Phase 64 §11 #7） |

#### 1.3.2 出生闸（宽限期内）

`spawn_tagged` 的 kind 预算闸从

```
eff_cap = min(budget, biome_cap)
```

改为（仅当该 kind 宽限期未结束、且它是派生 kind）

```
eff_cap = min(budget + 8, biome_cap)
```

- **区域容量闸不变**（DECO 仍按 `budget × w_r / Σw` 逐象限限制）；
- **能量/择偶/成熟度闸不变**；
- 宽限期内每放行一个「旧闸会拒绝」的出生，累加 `kind_grace_skips[kind]`；
- 出生成功后若 `count(kind) > budget`，更新 `kind_grace_peak_over[kind] = max(...)`；
- 超过 `budget + 8` 或 biome cap 的出生仍按 `KindBudget` 记账并拒绝（诊断口径与 Phase 61 一致）。

#### 1.3.3 结算（宽限期结束后的第一个 60 tick 检查 pass）

在 `check_speciation` 的「灭绝归还预算」循环**之后**执行（保证已灭绝的女儿先归还、预算归 0）：

1. 对每个 `kind_grace_until[k] != 0 && now >= until` 的 kind：先清零 `grace_until[k]`（一次性）。
2. `over = count(kind).saturating_sub(kind_budget(kind))`。
   - `over` 只计**新 kind 的成员**。创始人 fork 后保留父 kind，不计入 grant（现有 `detect_splits` 语义），因此「超编」= 新 kind 人口 − grant，不是「支系总人数 − grant」。
3. `over == 0` → 无债务，直接结束。
4. 支系存活且**支系成员数 ≥ `speciation_min_side`**（成员含创始人，取本 pass 刚重建的 `SpeciesClade::members`）→
   `pay = min(over, kind_budget(parent_kind))`；`parent -= pay`、`daughter += pay`；
   `kind_grace_settled[k] += pay`，`kind_grace_unpaid[k] += over − pay`。
5. 支系已灭绝或成员 < `speciation_min_side` → 不划转，`kind_grace_unpaid[k] += over`。
   已灭绝的支系在步骤 1 之前已由灭绝路径把整份预算归还父系（`kind_budget = 0`、`count = 0`），因此 `over = 0`，自然无债务。

**边界情况（写死）**：

| 情况 | 处理 |
|---|---|
| 父系在宽限期内又 fork，剩余份额变小 | `pay` 以父系当前份额为上限，可扣到 0 |
| 父系已灭绝（其份额已沿链归还） | `kind_budget(parent) = 0` → `pay = 0`，全额记 `unpaid`；**不向祖父级追扣** |
| 父系份额 < `over` | 扣到 0，差额记 `unpaid`；女儿人口 > 预算，出生闸恢复为硬顶，直到死亡把人口降到预算以下 |
| 女儿存活但成员 < 8 | 不划转（半耗散状态），记 `unpaid`；预算不增长，也不会凭空铸造 |
| 同一 kind 多次 fork | 每个派生 kind 各自独立宽限；父系的宽限窗不会传给孙女 |
| 根 kind | 永远没有宽限（fork 时才设置，且只有派生 kind 被设置） |

**守恒不变量**：宽限期内 `KindBudgets` 不动（人口可以超编，槽位不铸）；结算只做 parent→daughter 的槽位移动。因此**每个族的 `family total` 在整个过程中不变**，这与 Phase 49 以来的 ADR 不变量一致。`unpaid` 是账面差额，不是新槽位。

#### 1.3.4 存档

`SaveState` v23 → **v24**：新增 `ForkGraceOn: bool`（`#[serde(default)]` → 旧档落 OFF）与 `KindGraceUntil: Vec<u32>`（`#[serde(default)]` → 旧档空）。计数器（`skips/settled/unpaid/peak_over`）是遥测，不持久化（与 `clade_life` 同契约）。理由与 Phase 63 的 growth 臂相同：`grace_until` 会改变未来的出生决策，必须随档走，否则中途存档恢复会丢失宽限窗。

### 1.4 臂定义

| 臂 | 环境变量 | 语义 |
|---|---|---|
| **OFF** | `PHASE_FORK_GRACE=off` | Phase 64 默认世界（`FORK_BUDGET_MIN = 8`，无宽限） |
| **W1** | `PHASE_FORK_GRACE=on`（+ `PHASE_FORK_GRACE_TICKS=5000`） | 本文 §1.3 |
| **W3** | `PHASE_FORK_DAUGHTER_MIN=12`（对照臂） | 「建支最小群体强制」的可实现形式：女儿 grant floor = 12（= 1.5 × MIN_SIDE），父方剩余 floor 仍为 8。与 Phase 63 的 A12 逐位相同；字面上的「fork 时强制 12 名成员」需要改 `MIN_SIDE`，已被 Phase 63/64 否证，故不实现 |

`PHASE_FORK_GRACE` 未设时走编译默认 OFF；OFF 与 W1 两臂都显式设置，避免「默认 = 哪一臂」的歧义。W2（能量补贴）**不实现**：它引入新能量源，破坏热力学闭合（repro_loss / ledger 记账），且与「不新增全局总额」纪律冲突。

### 1.5 指标定义

- 分化 `spec` = `event_kind = 0` 的事件数（DECO 分叉，含所有族时单列）；
- 灭绝 `ext` = `event_kind ∈ {1, 2}` 的事件数（全族）；DECO 族口径另列（排除 05 scav 这类跨族固定事件）；
- **建支即失稳** = 派生 DECO 支系满足 `peak ≤ 12 ∧ region_cap_blocks = 0 ∧ kind_budget_blocks = 0 ∧ (extinction_tick − birth_tick) < 10 000`（Phase 62 §10.2；生存时长由 `clade_tree.birth` 与 `clade_life.extinction_tick` 关联）；
- 守恒 = 末态每个族的 `Σ kind_budget`（由 `[budget]` 行的 `parent` 链聚合）；
- **作用力证据** = W1 臂全池 `Σ kind_grace_skips`（§1.7 D0）。

### 1.6 判据（唯一判定，200k 5 seed × 2 臂）

| 判据 | 门槛（写死） | 说明 |
|---|---|---|
| **P1 分化不显著下降** | `Σ_W1(spec) ≥ 0.90 × Σ_OFF(spec)` | 与 Phase 64 同一条线 |
| **P2 灭绝** | `Σ_W1(ext) ≤ 0.75 × Σ_OFF(ext)` **且** 反向 seed ≤ **2 / 5** | 与 Phase 64 相同的相对线；反向 seed = `ext_W1(seed) > ext_OFF(seed)` |
| **P3 目标指标** | `Σ_W1(建支即失稳) ≤ floor(0.5 × Σ_OFF(建支即失稳))` | 当 OFF ≥ 1 时即「至少减半」；OFF = 0 时必须为 0（**空过**，见 D2） |

三条判据必须同时满足才算通过。P3 是本期的存在理由：P1/P2 只是「不伤害」的护栏，不能单独构成转正依据。

### 1.7 无作用力判据与决策表（写死）

**D0（inert 判据）**：W1 臂 5 seed 合计 `grace_skips = 0` → 仪器从未越过旧闸。此时无论 P1/P2 的计数如何（世界线可能因结算之外的路径完全不变），结论都是「W1 在本池上无从作用」，默认 OFF、不 re-baseline。

| # | 结果 | 决策 |
|---|---|---|
| 1 | **D0 成立**（`Σ grace_skips = 0`） | **默认 OFF**，不 re-baseline；结论：「建支即失稳 的约束不在 kind 预算闸，W1 在原理上救不了已记录的实例」 |
| 2 | D0 不成立，P1 ∧ P2 ∧ P3 全过，且 `Σ_OFF(建支即失稳) ≥ 1` | **W1 转正（默认 ON）**；**第二十次 re-baseline**，全锚点更新；w1 保留 `PHASE_FORK_GRACE=off` 回放臂 |
| 3 | D0 不成立，P1 ∧ P2 过，但 `Σ_OFF(建支即失稳) = 0`（P3 空过） | **默认 OFF**，不 re-baseline；结论：「W1 可作用但目标类样本为 0，无正向证据」 |
| 4 | D0 不成立，P1 或 P2 败 | **默认 OFF**，不 re-baseline；结论写「代价越界」并把逐 seed 表留档 |
| 5 | 可测 seed < 5 | inconclusive：默认 OFF、不 re-baseline、不据此否定机制 |

### 1.8 退化替换规则（写死）

某个 seed 判为**不可测**，当且仅当 OFF 与 W1 两臂在 200k 内 `spec + ext = 0`。替换：按 §1.2 生成器继续升序取 `20261121`、`20261122`（已核 0 命中），**最多替换 2 个**；替换 seed 与原 seed 都要在报告中列出。若可测 seed < 5 → 决策表第 5 行。

### 1.9 执行协议

1. **编译一次**：`cargo test --release --test clade_fate_observatory --no-run`，全部臂复用同一 release 产物，只改环境变量。
2. **80k 快筛**（只用于选臂/排除有害设计，不参与判定）：3 seed（20261116/17/18）× 3 臂（OFF / W1 / W3）。快筛门：
   - `Σ spec(W1) ≥ 0.90 × Σ spec(OFF)`（3 seed 合计）；
   - `Σ ext(W1) ≤ 1.25 × Σ ext(OFF)`。
   任一失败 → 设计在快筛被否，默认 OFF，**不进入 200k**（W3 不设门，只报数）。
3. **200k 确证**（唯一判定）：5 seed（§1.2）× 2 臂（OFF / W1）。并发 6，每臂一个进程。
4. **机制诊断**（非判定）：seed `20261102` + `PHASE_FORK_MIN=10`（复现 Phase 63 §5 #11 的已知实例）× 2 臂（OFF / W1），200k。用途有二：(i) 核对 `20261102/M10/OFF` 是否复现 Phase 63 的 12 分化 / 7 灭绝；(ii) 在该已知实例上直接读 `grace_skips`，检验「W1 救不了它」的机制命题。该臂**不改变默认值、不参与判定**。
5. **确定性自检（停止规则）**：
   - `20261102` / 200k / 无臂（编译默认）必须复现 Phase 64 的 OFF 计数 **9 分化 / 4 灭绝**；不成立则停止，先修环境；
   - 双跑 bit-exact：`20261116 / W1 / 200k` 跑两次（不同 `PHASE_ARM_TAG`），`[split]`/`[fate]`/`[budget]`/`[grace]` stderr 行 `diff` 为空，`clade_fate` 与 `clade_life` CSV `cmp` 逐字节相同；
   - W1 开关关闭时默认世界必须与 Phase 64 锚点逐位一致（@300/@3000/@30000/@60000），由全 workspace 测试与锚点测试守住。
6. **产物**：`/tmp/p65/*.log` 与 `/tmp/clade_*_p65_*.csv`；报告成文后删除。
7. **wasm**：三次同会话交错 A/B（W1-off 产物 vs HEAD 旧产物），每次 60 帧 sim+export 平均，要求 < 16 ms；记录 rustc 版本与产物大小/sha。

### 1.10 明确不做的事

- 不重试已否证方向：region cap 参数微调（Phase 61/62）、fork floor 8→10/12/16（Phase 63/64）、G 年龄成长（Phase 63）、符号解耦（Phase 57）；
- 不实现 W2（能量补贴）；
- 不把 W3 当候选：它只是对照臂，Phase 63 已测得 A12 在 80k 上 P1 差 0.5 个事件；
- 不在看到 200k 结果后追加 seed、改判据或改结算语义；
- 不为了让 D0「有作用力」而放宽宽限窗或超编上限——那会把「仪器没咬到」伪装成「机制无效/有效」。
