# Phase 62 — b 臂（女儿最低保留份额）5-seed 预注册复测

日期：2026-11-20
HEAD（起点）：`53fda73`（Phase 61：三个修正臂全部默认 OFF，无 re-baseline）
性质：**预注册复测**。Phase 61 §9.1 的遗留是「灭绝 13 → 4 超额达标，但分化 22 → 20（−9 %）」。本期把判据、seed 池、退化替换规则与决策优先级**在跑之前写死**，然后用 5 个可测 seed × 200k × 双臂判定这 −9 % 是盆地噪声还是真降付。

> 本文件的第一版只包含 §1 预注册，在第一个 `PHASE_TICKS=200000` 进程启动之前单独提交（`git log --format=%ci` 的提交时间早于任何 §2 数据产物的 mtime）。运行期间 §1 未做任何修改。

---

## 1. 预注册（跑之前写死）

### 1.1 seed 池

| 位 | seed | 来源 | 已知性质 |
|---|---|---|---|
| 1 | 20261102 | Phase 57–61 主 seed | OFF 臂 9 分化 / 4 灭绝（Phase 61 §4.2） |
| 2 | 42 | Phase 57–61 | 10 / 8 |
| 3 | 20261109 | Phase 57 补充 seed（Phase 59 起固定） | 3 / 1 |
| 4 | 20261105 | 项目既有 5-seed 池（`tests/phase32_open_loci_test.rs:98`） | **已知退化**：Phase 57 §2 记录该 seed 在两臂上 t=4020 DECO 根 clade 整体灭绝（此后 abiogenesis 再起源），对分化假设不可测 |
| 5 | 20261107 | 项目既有 5-seed 池（同测试） | 未做过 200k 观测；预注册前只跑过 2 000 tick 冒烟（t=1980 首次分叉，不退化） |
| 备用 | 20261113 | 项目既有 5-seed 池（同测试） | 未做过 200k 观测 |

**退化判定（写死，只用 DECO 根支的生存遥测，不看分化/灭绝合计）**：某 seed 判为退化，当且仅当 DECO 根（`clade_id = 0`、`root_kind = DECO`）的 `extinction_tick < 10 000`，**且**该 seed 的 OFF 臂与 b 臂都如此。

**替换规则（写死，按顺序执行）**：
1. 20261105 退化 → 备用位 20261113 顶替，判据在 `{20261102, 42, 20261109, 20261107, 20261113}` 上计算。
2. 若 20261113 也退化，或替换后可测 seed 少于 5 个 → 判 **inconclusive**：默认保持 OFF，不 re-baseline，不据此否定机制（不用 4 个 seed 硬凑结论）。
3. 为了不让「看到结果再挑 seed」有任何空间，备用位 20261113 与主池**同时跑满 12 臂**；替换只在表格层面做，不补跑、不二次挑选。
4. 20261103 不在项目既有 seed 池里，故不采用；备用位取既有池中的 20261113。

### 1.2 指标定义（预注册）

| 指标 | 定义 | 来源 |
|---|---|---|
| 分化 | `event_kind = 0` 的事件数（DECO 分叉） | stderr `[split]` |
| 灭绝 | `event_kind ∈ {1, 2}` 的事件数（全族；2 = 引擎判定的竞争排除） | stderr `[split]` |
| DECO@200k | t=200 000 的 `deco` 列 | `clade_fate_*.csv` |
| 每象限 DECO | t=200 000 的 `deco_r0..deco_r3` | 同上 |
| denv_var | t=200 000 的 `denv_var` | 同上 / stderr `[fate]` |
| clades live/total | t=200 000 | stderr `[fate]` |
| 灭绝定因 | `budget_blocks / (budget_blocks + births)`、`peak` vs `grant/4`、死因四分、mate_misses | `clade_life_*.csv` |
| grant | 由事件序 + `Ecology::fork_budget_grant` 复算（Phase 61 §3.1 同法） | `/tmp/p62/analyze.py` |

### 1.3 判据（预注册，两条并列，全部满足才算通过）

- **P1 分化不显著下降**：`Σ_b(spec) ≥ 0.90 × Σ_off(spec)`（允许 10 % 盆地噪声）
- **P2 灭绝减半**：`Σ_b(ext) ≤ 0.50 × Σ_off(ext)`

记 `R = Σ_b(spec) / Σ_off(spec)`，`Q = Σ_b(ext) / Σ_off(ext)`。依据：Phase 59 §10.4 确立「分化率判据至少 5 seed」；Phase 61 §9.1 要求「判据、seed 池、验收线必须先写死再跑」。

### 1.4 决策规则（优先级从上到下，写死）

1. **通过**：P1 且 P2 → b 臂默认 ON（`DECO_REGION_CAP_FLOOR_ON = true`）+ **第十八次 re-baseline**（全锚点更新），并给 b 臂补双跑 / 存档往返测试与新锚点测试。
2. **噪声带**：P2 成立但 `0.85 ≤ R < 0.90` → 默认保持 OFF（未达提升门槛，不翻默认，不 re-baseline）；结论写「差距 < 15 %，落在 5-seed 分辨率内的盆地噪声带，**不能拒绝 b 机制**」，Phase 63 用一组此前未用过的 ≥ 5 个 seed 独立复测后再定。
3. **真降**：`R < 0.85`（≥ 15 %）且方向一致（b 在 ≥ 3/5 个可测 seed 上分化 ≤ OFF）→ 判「真降，**拒绝 b 作为默认**」；默认 OFF，臂保留在代码，不 re-baseline。
4. **P2 失败**（`Q > 0.50`）→ 直接判不通过（默认 OFF），无论 R——b 的唯一卖点就是拿分化换灭绝减半，交换不成立则无提升理由。
5. **inconclusive**：可测 seed < 5 → 默认 OFF，不 re-baseline，不据此否定机制。

### 1.5 敏感性分析（次要，不参与默认决策）

条件执行：**若主判据不通过**，在同一组可测 seed 上跑变体 d——floor 只施加于 `budget < 2 × speciation_min_side = 16` 的 kind（Phase 61 §9.2 的提案，开关 `PHASE_CAP_FLOOR=small`），用于回答「floor 的高度」这一未标定量。变体 d 的数据**只作机制诊断**，不参与 §1.4 的任何判定，也不改默认。若主判据通过，b 转正，变体 d 留作 Phase 63 候选（不跑）。

### 1.6 执行协议

- 12 臂 = 6 seed × 2 臂（`PHASE_CAP_FLOOR=off|on`），`PHASE_TICKS=200000`，release 二进制，单变量（其余全部编译默认 = Phase 60 机制栈：DECO 分解场 + 区域承载容量 + fork budget + 代际 niche + 分区 + patch + 三脑 + 文化 + 生育饱和映射）。
- 每臂独立进程，`PHASE_ARM_TAG=p62_off` / `p62_on`，并发 6（8 核），产物落 `/tmp/p62/*.log` 与 `/tmp/clade_{fate,tree,culture,life}_*_p62_{off,on}.csv`。
- **确定性自检**：OFF 臂必须逐事件复现 Phase 61 的 3 seed 计数（102：9/4；42：10/8；109：3/1）。任何偏差说明二进制或臂开关状态不对，先查再算。
- 报告成文后删除临时文件（`/tmp/p62/`、`/tmp/clade_*_p62_*.csv`）。
