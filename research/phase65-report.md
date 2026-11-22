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

---

## 2. 实现（W1）

### 2.1 常量与状态

| 位置 | 内容 |
|---|---|
| `core/crates/aether-sim-ecology/src/ecology.rs:1094-1117` | `FORK_GRACE_ON = false`、`FORK_GRACE_TICKS = 5_000`、`FORK_GRACE_OVERDRAFT_MAX = 8`；注释记录「目标类的两道闸都为 0」这一机制事实 |
| `:2161-2179` | `budget_grace` / `kind_grace_until`（持久化）、`grace_ticks`（臂旋钮）、四个遥测向量（`skips/peak_over/settled/unpaid`，瞬态） |
| `:2403-2409`、`:2644-2650` | `new` / `new_with` 初始化（默认 OFF、空表） |
| `:3278` | `register_kind` 为 `kind_grace_until` 补零，保证索引连续 |

### 2.2 出生闸（`spawn_tagged`）

`core/crates/aether-sim-ecology/src/ecology.rs:4748-4770`：

- 旧闸 `eff_cap = min(budget, biome_cap)`；宽限期内改为 `min(budget + 8, biome_cap)`。
- 区域容量闸（`deco_region_cap`）与能量/择偶/成熟度闸完全不变。
- `grace_admitted` 只在**出生成功**后记账（`:4895`）：`kind_grace_skips[kind] += 1`、`kind_grace_peak_over = max(over)`。被区域闸拒绝的出生不记 skip——它的轨迹与旧闸拒绝逐位相同，不能算作「仪器咬到了」。

### 2.3 结算（`settle_fork_grace`，`:4085-4132`）

在 `check_speciation` 的 `(a3)` 灭绝归还循环**之后**调用（`:8094`）；每个 kind 一次性：

1. `until == 0 || now < until` → 跳过；否则先清零窗口。
2. `over = count(kind) − kind_budget(kind)`（只计派生 kind 自己的成员；创始人不计入 grant）。
3. `over == 0` → 无债务。
4. 支系存活且 `SpeciesClade::members.len() >= speciation_min_side` → `pay = min(over, parent_budget)`，`parent -= pay`、`daughter += pay`；差额记 `unpaid`。
5. 支系灭绝 / 成员不足 / 无父 kind → 不划转，全额记 `unpaid`。

边界语义与预注册 §1.3.3 逐条一致：父系可被扣到 0；父系已死则 `pay = 0`；不向祖父级追扣；`unpaid` 只记账不铸造。fork 钩子在 `:8369`（`grant` 落表之后设 `now + grace_ticks`）。

### 2.4 存档 v24

- `src/save_state.rs:140`：`SAVE_STATE_VERSION = 24`；`:1080` / `:1085`：`ForkGraceOn`、`KindGraceUntil`（均 `#[serde(default)]`）。
- 写入 `ecology.rs:9312-9313`，恢复 `:10065-10066`；计数器不持久化（与 `clade_life` 同契约）。`grace_ticks` 是臂旋钮（与 `fork_budget_min` 同契约，不入档）：窗口的绝对终点已随 `KindGraceUntil` 存档，读档后的新 fork 回到编译默认 5 000。
- v23 档案按 serde default 落 OFF + 空表（单元测试 `grace_window_rides_the_archive_and_pre_v24_lands_off` 覆盖）。

### 2.5 观测器

`tests/clade_fate_observatory.rs`：新增 `PHASE_FORK_GRACE=on|off` 与 `PHASE_FORK_GRACE_TICKS`，臂名进入四个 CSV 文件名；末尾新增 `[grace]` 逐 kind 行与 `[grace] total_skips=… windows_pending=…` 汇总行。`[budget]` / `[split]` / `[extinct]` 输出未改。

### 2.6 测试

- `ecology.rs` 内 `phase65_fork_grace_tests`（9 个）：无宽限时 grant 是硬顶；宽限最多放行一个额外 cohort（8）；结算把 4 个超编槽位从父系划走且 family total 80 不变；父系只有 3 槽时扣到 0、`unpaid = 1`；成员 < MIN_SIDE 的支系不划转（合成契约测试，注明真实世界不可达）；已灭绝女儿无债务；根 kind 永不进窗；编译默认 OFF；v24 存档往返 + v23 落 OFF。
- `tests/phase65_fork_grace_test.rs`（1 个）：Phase 34 的强制 PRED fork 夹具上验证真实 `detect_splits` 路径写入 `now + 5_000`，且开关在前叉之前零差异（两臂 fork tick 相同）。

---

## 3. 80k 快筛（3 seed × 3 臂，只用于选臂）

`20261116 / 17 / 18`，编译默认 + 显式臂，同一 release 产物。

| seed | 臂 | 分化 | 灭绝 | DECO 灭绝 | 建支即失稳 | grace skips | settled | DECO family total | 人口@end | clades live/total |
|---|---|---|---|---|---|---|---|---|---|---|
| 20261116 | OFF | 6 | 3 | 3 | 0 | 0 | 0 | 80 | 675 | 5/8 |
| 20261116 | **W1** | **4** | **1** | 1 | 0 | 46 | 11 | 80 | 679 | 5/6 |
| 20261116 | W3 | 5 | 4 | 4 | 0 | 0 | 0 | 80 | 634 | 3/7 |
| 20261117 | OFF | 1 | 0 | 0 | 0 | 0 | 0 | 80 | 640 | 2/2 |
| 20261117 | **W1** | **3** | **0** | 0 | 0 | 35 | 7 | 80 | 690 | 4/4 |
| 20261117 | W3 | 1 | 0 | 0 | 0 | 0 | 0 | 80 | 640 | 2/2 |
| 20261118 | OFF | 5 | 1 | 1 | 0 | 0 | 0 | 80 | 640 | 5/6 |
| 20261118 | **W1** | **5** | **1** | 1 | 0 | 50 | 23 | 80 | 640 | 5/6 |
| 20261118 | W3 | 5 | 3 | 3 | 0 | 0 | 0 | 80 | 547 | 3/6 |
| **合计** | **OFF** | **12** | **4** | 4 | **0** | 0 | 0 | — | — | — |
| **合计** | **W1** | **12** | **2** | 2 | **0** | **131** | **41** | — | — | — |
| **合计** | **W3** | **11** | **7** | 7 | **0** | 0 | 0 | — | — | — |

快筛门：`Σ spec(W1) = 12 ≥ 0.90 × 12 = 10.8` ✅；`Σ ext(W1) = 2 ≤ 1.25 × 4 = 5` ✅。进入 200k。

**读法**：

1. **D0 不成立**：W1 在三个 seed 上共放行 131 个「旧闸会拒绝」的出生，仪器确实咬到了。这与预注册 §1.1 的悲观推论（「目标类闸计数为 0 ⇒ W1 无从作用」）不矛盾——W1 作用在**其它**女儿上，而不是两起已记录的目标实例。
2. **一半以上的超编在结算前就消散**：131 个 skips 只有 41 个在窗口结束时仍处于超编（settled），其余 90 个对应的个体已死亡，预算无需移动。这与「建支即失稳 = 招募不足」的读法一致：W1 能把出生放进来，但留不住。
3. **W3 在 80k 已经显出代价**（灭绝 7 vs 4），与 Phase 63 对 A12 的否定一致；它只作对照，不进入判定。

---

## 4. 200k 确证（5 新 seed × 2 臂）

| seed | 臂 | 分化 | 灭绝 | DECO 灭绝 | 建支即失稳 | grace skips | settled | unpaid | DECO family total | 人口@end | clades live/total |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 20261116 | OFF | 13 | 8 | 8 | 0 | 0 | 0 | 0 | 80 | 421 | 7/15 |
| 20261116 | **W1** | **6** | **4** | 4 | 0 | 79 | 29 | 0 | 80 | 380 | 4/8 |
| 20261117 | OFF | 9 | 5 | 5 | 0 | 0 | 0 | 0 | 80 | 412 | 5/10 |
| 20261117 | **W1** | **6** | **2** | 2 | 0 | 65 | 25 | 0 | 80 | 406 | 5/7 |
| 20261118 | OFF | 9 | 5 | 5 | 0 | 0 | 0 | 0 | 80 | 362 | 5/10 |
| 20261118 | **W1** | **7** | **2** | 2 | 0 | 68 | 26 | 0 | 80 | 371 | 6/8 |
| 20261119 | OFF | 6 | 1 | 1 | 0 | 0 | 0 | 0 | 80 | 638 | 6/7 |
| 20261119 | **W1** | **7** | **2** | 2 | 0 | 60 | 25 | 0 | 80 | 378 | 6/8 |
| 20261120 | OFF | 7 | 4 | 4 | 0 | 0 | 0 | 0 | 80 | 342 | 4/8 |
| 20261120 | **W1** | **5** | **3** | 3 | 0 | 39 | 15 | 0 | 80 | 390 | 3/6 |
| **合计** | **OFF** | **44** | **23** | **23** | **0** | 0 | 0 | 0 | — | — | — |
| **合计** | **W1** | **37** | **17** | **17** | **0** | **390** | **120** | **0** | — | — | — |

逐 seed 方向：

| seed | 分化 OFF→W1 | 灭绝 OFF→W1 |
|---|---|---|
| 20261116 | 13 → 6（−7） | 8 → 4（−4） |
| 20261117 | 9 → 6（−3） | 5 → 2（−3） |
| 20261118 | 9 → 7（−2） | 5 → 2（−3） |
| 20261119 | 6 → 7（**+1**） | 1 → 2（**+1，唯一反向**） |
| 20261120 | 7 → 5（−2） | 4 → 3（−1） |
| **合计** | **44 → 37（−15.9 %）** | **23 → 17（−26.1 %）** |

预注册判据：

| 判据 | 门槛（§1.6） | 实测 | 结果 |
|---|---|---|---|
| **P1 分化** | `Σ_W1 ≥ 0.90 × 44` = **39.6** | **37**（R = 0.841） | ❌ |
| **P2 灭绝** | `Σ_W1 ≤ 0.75 × 23` = **17.25**，且反向 seed ≤ 2 | **17**（Q = 0.739），反向 **1/5**（20261119） | ✅ |
| **P3 目标** | `Σ_W1(建支即失稳) ≤ floor(0.5 × 0)` = **0** | **0**（两臂均为 0） | 空过 |
| **D0 作用力** | `Σ grace_skips > 0` | **390** | ✅（非 inert） |

**守恒**：11 个 200k 臂的 DECO `family total` 全部为 80；每一条 `[budget]` 行的 parent 链聚合后，其余五族（herb 420 / pred 110 / scav 40 / omn 80 / fish 60）也逐臂一致。`unpaid = 0`：所有存活到结算的超编都由父系付清，没有出现「父系已死/不足」。

**双跑 bit-exact**：`20261116 / W1` 两次（`p65_c_w1_16` 与 `p65_c_w1_16b`）的 `[split]`（10 行）/`[fate]`（4 行）/`[budget]`（13 行）/`[grace]`（8 行）逐行相同，`clade_fate` 与 `clade_life` CSV `cmp` 逐字节相同。

**读法**：

1. **W1 的净效应是「少生、少死」，与 M10 同形但幅度更大**：分化 −15.9 %、灭绝 −26.1 %。Phase 64 的 10 % 分化容差正是为拦住这类「用分化换灭绝」的臂而设——W1 越过了它。收益也不是普适的：20261119 两指标同时反向。
2. **目标类没有复现**：5 个新 seed 的 10 个 200k 臂里 `建支即失稳 = 0`（与 Phase 64 §11 #5 一致）。两起已记录实例来自 62/63 的旧池（42 / b 臂、102 / M10 臂），不能拿来给 W1 记账；因此 P3 是空过，W1 的**存在理由（救建支即失稳）在本期没有任何正向证据**。
3. **69 % 的超编在结算前就消散**：390 个 grace 出生只有 120 个在窗口结束时仍处于超编（settled），270 个对应的个体已经死亡。W1 能把出生放进来，但留不住——这与 Phase 62 定因表里的 `mate_misses` 与 6 出生/14 死亡一致：约束在**招募/留存**，不在槽位闸。
4. **代价的一条具体路径是根支出局 + 预算搁浅**：20261116 / W1 的 DECO 根支（clade 0，peak 80）在 200k 内灭绝，而它的 kind 仍持有 30 槽（`[budget] kind=5 budget=30`）——这些槽位没有再分配对象，正是 Phase 64 §11 #3 记录的问题。OFF 同 seed 的根支活到 200k（kind 5 只剩 15 槽）。
5. **非 DECO 族在本池没有连带灭绝**：两臂的 23 / 17 起灭绝全部是 DECO（`deco_ext == ext`）；五族预算末态逐臂相同。这与旧池的 05 scav 阿利事件不同，本池没有可测的跨族副作用。

### 4.1 机制诊断（20261102 / `PHASE_FORK_MIN=10`，非判定）

| 臂 | 分化 | 灭绝 | 建支即失稳 | grace skips | settled | 根支（clade 0） | 人口@end |
|---|---|---|---|---|---|---|---|
| M10 / OFF | **12** | **7** | **1**（deco-I 98 400–107 160，peak 8，8 760 tick，113 mate-misses） | 0 | 0 | 活到 200k | 465 |
| M10 / W1 | 6 | 2 | 0 | 56 | 23 | **t=61 380 灭绝（peak 74）** | 663 |

- **旧臂可复现**：M10/OFF 精确复现 Phase 63 的 102 M10（12 分化 / 7 灭绝）与 §5 #11（deco-I，peak 8，两道闸 0，8 760 tick；Phase 63 报的 98 450 是 `first_tick`，tree 的 `birth` 是 98 400）。
- **W1 没有「救活 deco-I」**：两臂的前两次 fork 逐 tick 相同（18 180、22 560），第三次开始分岔（OFF 37 260 / W1 39 360），deco-I 这个支系在 W1 臂根本不存在。它的 2 起灭绝换来了 6 次分化（OFF 12），且根支在 61 380 出局——与确证池的「少生少死 + 根支代价」完全同形。
- 56 次宽限放行只有 23 槽在结算时仍超编（41 %），同 §4 读法 3。

---

## 5. 决策

**不通过 → W1 保持默认 OFF；不 re-baseline（锚点不变）。**

依据预注册 §1.7 决策表第 4 行（P1 败；同时 P3 空过，第 3 行的「无正向证据」结论也成立）：

- P1：`37 < 39.6`，分化代价 15.9 % 越过 10 % 容差；
- P2 虽然过线（17 ≤ 17.25，Q = 0.739，反向 1/5），但 P1/P2 不能分开记账——W1 的灭绝收益是用分化买来的，而分化是本项目的核心观测量；
- P3：目标类在两臂都是 0，W1 的设计目的（救建支即失稳）没有被检验到，更没有被证明；
- D0 成立（390 次放行），所以这不是「仪器没咬到」导致的空结果，而是「咬到了、但代价越界且目标类缺席」。

### 5.1 保持了默认世界

- `FORK_GRACE_ON = false` 未动；默认轨迹与 Phase 64 逐位一致（全 workspace 测试与锚点测试见 §7）。
- 第二十次 re-baseline **没有发生**：@300 / @3000 / @30000 / @60000 及全部臂锚点不变。
- 臂保留：`PHASE_FORK_GRACE=on` + `PHASE_FORK_GRACE_TICKS=5000` 可在任意 seed 上复跑；存档 v24 只增加两个 serde-default 字段（默认世界写 `false` + 空表）。

### 5.2 没有证明什么（诚实边界）

- **没有证明 W1 对所有 seed 都有害**：20261119 两项指标同时反向；其余四个 seed 的分化都下降（−1 到 −7），灭绝也都不升。符号一致，但幅度与 seed 方差同量级。
- **没有证明「建支即失稳 不可救」**：本期只证明「kind 预算闸不是它的约束」——目标实例的闸计数本来就是 0，W1 也救不了它们。真正的下一个杠杆是招募（择偶/扩散/奠基群聚集），见 §11。
- **没有测宽限窗长度与超编上限的敏感性**：5k / +8 是预注册的单点，不排除其它参数更优；但在 P1 已越界且目标类缺席的前提下，调参没有决策价值。
- **没有把 W1 与 M10 直接对打**：两者的效应同形（少生少死），M10 已被 out-of-sample 否定；W1 的幅度更大，不改变结论。

---

## 6. 锚点（没有 re-baseline）

**第二十次 re-baseline 没有发生。** W1 不通过，编译默认保持 `FORK_GRACE_ON = false`，默认世界逐位不变：

| 锚点 | 值 | 状态 |
|---|---|---|
| `@300` / `@3000` | Phase 31 黄金回归 | 不变（全测试通过，见 §7） |
| `@30000` | Phase 56/57/61 默认臂 | 不变（同上） |
| `@60000` | `0x3272f0e3eaba9250`（Phase 56/57/61 锚点测试） | 不变（定向慢测，见 §7） |
| 默认世界 200k 计数 | Phase 64 的 9 分化 / 4 灭绝（seed 20261102） | **复现 9 / 4**（`p65_self_02`，§4.1 的 M10 臂另复现 12 / 7） |

`[budget]` 的 `family_total=80` 在默认世界与 W1 臂上都成立（11 个确证臂 + 9 个快筛臂 + 3 个诊断/自检臂）。

## 7. 确定性 / 测试

| 项 | 内容 | 结果 |
|---|---|---|
| 外部双跑（200k） | seed 20261116 / W1 两次：`[split]`/`[fate]`/`[budget]`/`[grace]` 逐行相同，`clade_fate` 与 `clade_life` CSV 逐字节相同 | ✅ |
| 默认世界 200k 自检 | seed 20261102：9 分化 / 4 灭绝 | ✅（Phase 64 基线） |
| M10 旧臂复现 | seed 20261102 / `PHASE_FORK_MIN=10`：12 分化 / 7 灭绝，deco-I 目标实例 | ✅（Phase 63 基线） |
| 新增单元测试 | `phase65_fork_grace_tests` 9 个 | ✅ |
| 新增集成测试 | `phase65_fork_grace_test` 1 个（真实 fork 路径） | ✅ |
| 全 workspace 测试 | 生态 crate `cargo test --release --tests` | **50 二进制 / 260 passed / 0 failed / 28 ignored** |
| 全 workspace 其余 crate | `cargo test --release --workspace --exclude aether-sim-ecology --tests` | **12 二进制 / 75 passed / 0 failed / 0 ignored** |
| 定向慢测（7 个二进制的 `#[ignore]`） | phase63/62/60/61/56/57/59 | **10 tests 全过**：1168.7 s / 1008.8 s / 1174.3 s / 1158.5 s / 1084.6 s / 1453.9 s / 601.2 s |

合计 **62 个测试二进制 / 335 passed / 0 failed / 28 ignored**（含本期新增 10 个）。

## 8. wasm

| 项 | 值 |
|---|---|
| rustc | 1.99.1 (58cf8f9c3 2026-10-13) |
| 产物 | `living/ecology-sim.wasm` / `ecology-sim-v2.wasm`，**828 723 B** |
| sha256 | `12f4ce1f15c19775815569ea9cebf7a0f51698b3f638fef7f1725729d00a043a` |
| 旧基线 | 826 127 B，`d1fb0d95…`（Phase 64） |
| 增量 | +2 596 B（W1 状态/闸/结算/观测器） |

60 帧 sim+export 三轮同会话交错 A/B（每轮先新后旧；空闲机器）：

| 轮 | 新产物 | 旧产物（Phase 64） |
|---|---|---|
| 1 | 9.89 ms/frame | 9.92 ms/frame |
| 2 | 9.95 ms/frame | 9.86 ms/frame |
| 3 | 9.96 ms/frame | 9.95 ms/frame |

三轮全部 < 16 ms；新旧在噪声内相当（差 ≤ 0.1 ms）。两次 `build.py` 重建的 sha 逐位相同（`12f4ce1f…`）。

## 9. 验收逐条

| # | 验收 | 结果 |
|---|---|---|
| 1 | 预注册（seed 池 + 判据 + 决策），跑前单独 commit | ✅ §1，提交 `9d7fad5`；第一个实验进程在其后启动，运行期间 §1 未改 |
| 2 | W1 实现 + 结算语义 + 单元测试（守恒/超编上限/结算） | ✅ §2；9 个单元 + 1 个集成 |
| 3 | 200k 5-seed A/B 数据表（分化/灭绝/建支即失稳/守恒/逐 seed） | ✅ §4 |
| 4 | 决策（通过 → 第二十次 re-baseline；不通过 → OFF） | ✅ §5：P1 败 → OFF，不 re-baseline |
| 5 | 双跑 bit-exact；全 workspace 0 失败（含定向慢测）；wasm < 16 ms 三次 | ✅ §7 / §8 |
| 6 | 报告 `research/phase65-report.md` | ✅ 本文件 |

## 10. 改动文件

### 核心
- `core/crates/aether-sim-ecology/src/ecology.rs`
  - `:1094-1117`：W1 常量与机制注释
  - `:2161-2179`：`budget_grace` / `kind_grace_until` / `grace_ticks` + 四个遥测向量
  - `:3998-4132`：开关、`in_fork_grace`、窗口写入、`settle_fork_grace`
  - `:4748-4770`、`:4895`：出生闸的宽限与放行记账
  - `:8094`：结算 pass 的调用点（灭绝归还之后）
  - `:8369`：fork 时写窗口
  - `:9312-9313`、`:10065-10066`：v24 写入/恢复
  - `:10290-10479`：`phase65_fork_grace_tests`（9 个）
- `core/crates/aether-sim-ecology/src/save_state.rs`
  - `:131-136`：v23 → v24 文档
  - `:140`：`SAVE_STATE_VERSION = 24`
  - `:1076-1086`：`ForkGraceOn` / `KindGraceUntil`（serde default）
- `core/crates/aether-sim-ecology/tests/clade_fate_observatory.rs`
  - `:182-187`、`:192-195`：`PHASE_FORK_GRACE` / `PHASE_FORK_GRACE_TICKS` 与文件臂名
  - `:227-231`：臂设置
  - `:484-505`：`[grace]` 诊断输出
- `core/crates/aether-sim-ecology/tests/phase65_fork_grace_test.rs`：真实 fork 路径的窗口断言（新文件）

### 版本 pin（v23 → v24）
- `tests/phase39_1_transition_test.rs:429`、`tests/phase41_triune_test.rs:121`、`tests/phase42_brain_authority_test.rs:367`、`tests/phase63_kind_budget_test.rs:50`：断言更新为 24（并新增 `budget_grace` 默认值断言）

### 产物与文档
- `living/ecology-sim.wasm`、`living/ecology-sim-v2.wasm`：828 723 B（sha `12f4ce1f…`）
- `research/phase65-report.md`（本文件；§1 单独提交 `9d7fad5`）

### 临时（不入仓，报告成文后删除）
- `/tmp/p65/{run.sh,resume.sh,slow.sh,analyze.py,manifest_*.txt,logs/,slow_*.log,*.wasm}`、`/tmp/clade_*_p65_*.csv`、`/tmp/p65_ft*.log`

## 11. Phase 66 遗留

1. **「建支即失稳」的真约束是招募，不是容量**：目标实例的两道闸计数为 0；W1 放行的 390 次超编里 69 % 在窗口结束前消散；确证池的两臂目标类都是 0。Phase 66 应直接测招募类机制——宽限期内对新生支系豁免 `region_isolation`（允许跨象限择偶）、fork 时把女儿成员收拢到一个 deme（奠基群聚集）、或给新生 kind 一个择偶半径下限。三条都不新增全局总额，但都改变「谁能遇到谁」，必须单独预注册。
2. **「加厚女儿」类提案的判据应 lexicographic**：W1（−15.9 % 分化 / −26.1 % 灭绝）与 M10 同形，都是父系支付换女儿存活。P1/P2 并列会反复制造「灭绝大幅改善但分化越界」的诱惑；建议 Phase 66 直接把「分化不降」设为硬门，灭绝只作为次级指标。
3. **根支出局 + 预算搁浅是独立 bug**（Phase 64 §11 #3）：本期 seed 20261116/W1 与 20261102/M10-W1 都出现根支灭绝而 kind 仍持槽（30 / 2 槽）。这与 W1 无关，但每次「父系支付」类机制都会放大它。建议单列一期做「根灭绝时预算回到族池/按存活份额重分配」，并给出守恒不变量。
4. **v24 是为默认 OFF 的臂付的成本**：两个 serde-default 字段；默认世界写 `false` + 空表。若 Phase 66 不用 grace 臂，可规划回 v23（破坏性 schema 变更，需单独迁移计划）。
5. **5 seed 的判定力**：本期 OFF 44 次分化 / 23 起灭绝，W1 效应 −7 / −6，已超过 seed 方差但仍有 1 个反向 seed。Phase 64 §11 #2 的建议仍有效：预算/容量类 A/B 用 ≥8 seed 或成对 bootstrap，并在预注册里写最小可检测效应。
6. **非 DECO 族在本池无连带**：23 / 17 起灭绝全部是 DECO；若 Phase 66 换池，仍需单列非目标族的灭绝变化（Phase 64 §11 #7）。
7. **快筛的 3 seed 与确证池重叠**：`20261116/17/18` 既进了 80k 快筛又进了 200k 判定。Phase 63 有同样做法，但严格说它们对确证不再是「完全未触碰」。Phase 66 若要最严格，应把快筛 seed 与判定 seed 完全分离（代价是多 3 个 seed 的机时）。
