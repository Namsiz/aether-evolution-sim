# Phase 69.3 — 资源流导出的 DECO 总承载容量（Stage 0 标定预注册）

日期：2026-11-24
HEAD（起点）：`cfe2e71`（Phase 69.2：T-region 默认 OFF，P2/D0 败，转 kind-only）
性质：**Stage 0 纯遥测标定**。不改默认、不引入机制、不 re-baseline；只测 DECO 的资源面（尸体能量）在世界中的尺度，为 Stage 1 的「资源→总容量」映射提供 `E_ref`。Stage 1 的 A/B 会在 Stage 0 数据出来后单独预注册。

> Phase 69.1/69.2 已把靶点钉死：派生 DECO 的 kind-only 拒绝 68.7%（T 臂 93.9%），region 侧三次否证。Phase 69.3 是唯一还没被否证的「增加总容量」方向：让 DECO 全族预算随尸体资源流变化，而不是固定 80。

---

## 1. Stage 0 预注册（跑之前写死）

### 1.1 资源信号

- **主信号**：`E(t) = Σ_{c ∈ carcasses} c.energy`，即当前尸体池的总能量（`Carcass.energy`，公开字段）。
- **辅助信号**：`N(t) = carcasses.len()`（尸体数），用于区分「少数大尸体」与「多数小尸体」。
- **为什么是尸体能量**：DECO 的唯一食物是尸体（`Carcass` 池）；尸体能量是 DECO 承载容量的直接资源面。它与 DECO 人口是负反馈（DECO 吃得越多，池越低），不会形成「人口→资源→人口」的正反馈爆炸。
- **采样**：沿用观测器 `SAMPLE_EVERY = 5_000` tick；只在 CSV 中记录，不参与任何动力学。
- **排除前 20k**：初始布景/abiogenesis 的 transient 不算入标定统计。

### 1.2 插桩（零副作用）

- `Ecology::carcass_energy_total() -> f64`：`carcasses.iter().map(|c| c.energy).sum()`；纯读、零 RNG。
- `Ecology::carcass_count() -> usize`：`carcasses.len()`；纯读。
- `clade_fate_observatory.rs` 的 `clade_fate` CSV 末尾追加 `carcass_energy,carcass_count` 两列；`[fate]` 行末尾追加 `carcass_e={:.1}`（便于 grep）。
- 不改任何 `spawn`/`die`/`update` 路径；默认世界必须与 Phase 69.2 的 `9/4` 和 `@60000` 锚点逐位一致。

### 1.3 运行

| 项 | 内容 |
|---|---|
| seed | `20261215 / 20261217 / 20261219`（Phase 69.2 的 OFF 臂已跑过 200k，但当时无 carcass 列；本期重跑） |
| tick | 200,000 |
| 臂 | 编译默认（OFF） |
| 并发 | 3 进程 |
| 自检 | `20261102 / 200k` = 9/4；`@60000` 锚点 `0x3272f0e3eaba9250` |
| 输出 | `/tmp/p69s0/*.log` 与 `clade_fate_*_p69s0*.csv` |

### 1.4 标定量

对每个 seed，取 `t ≥ 20_000` 的 36 个 5k 采样点：

- `E_mean`、`E_median`、`E_p10`、`E_p90`。
- `N_mean`、`N_median`。
- 跨 seed 的 `E_median` 中位数 → **`E_ref`**（Stage 1 的参考资源）。
- `E` 的逐 seed 变异系数（CV = std/mean），用于判断「资源流是否稳定到可以驱动总容量」。

### 1.5 判据（Stage 0 只验收仪器，不判定机制）

| # | 条件 | 失败处理 |
|---|---|---|
| D0-1 | 三个 seed 的 `carcass_energy` 列非空且有限 | 仪器错误；先修 |
| D0-2 | 默认 `20261102 / 200k` = 9/4 | 轨迹被扰动；停止 |
| D0-3 | `@60000` 锚点 `0x3272f0e3eaba9250` 位精确 | 同上 |
| D0-4 | 三 seed 的 `E_median` 同量级（CV ≤ 0.5） | 资源尺度 seed 依赖过强；Stage 1 需要 per-seed 归一化或更多 seed |

### 1.6 明确不做

- 不改 `MAX_DECO`、`kind_budgets`、`family_budget_total`。
- 不引入任何 cap 映射；Stage 1 的映射参数在 Stage 0 数据出来后单独预注册。
- 不改 region cap、择偶半径、W1、M10。
- 不做观察层/渲染功能；只做 headless CSV 遥测。

### 1.7 执行协议

1. 提交本预注册（§1）。
2. 实现 §1.2 的两个 getter 与 CSV/`[fate]` 列；补一个只读单测（`carcass_energy_total == sum`、空池为 0）。
3. 默认 `@60000` 锚点 + 全量测试 0 失败。
4. 跑 3 seed × 200k；写 Stage 0 标定表与 `E_ref`。
5. Stage 1 的 A/B 预注册另行提交；本期不跑 A/B。

---

## 2. Stage 0 实现

（待实现后补入。）

---

## 3. Stage 0 结果

（待 3 seed 200k 完成后补入：逐 seed 的 E/N 分布、E_ref、CV。）

---

## 4. Stage 1 候选映射（Stage 0 后写死，暂不预注册）

- `cap_total = clamp(round(MAX_DECO × E_ema / E_ref), MIN_CAP, MAX_CAP)`。
- `E_ema` 每 5,000 tick 更新：`E_ema ← 0.8 × E_ema + 0.2 × E`（零 RNG）。
- 初始 `MIN_CAP=40`、`MAX_CAP=160`；`E_ref` 取 Stage 0 的三 seed 中位数。
- 根 DECO 的 `kind_budget` 调整为 `cap_total`；fork 仍按 `clamp(parent/4, 8, 20)` 划拨；灭绝归还。`family_budget_total(DECO)` 从恒 80 变为恒 `cap_total`。
- Stage 1 的 A/B 判据：P1 分化非劣性、P2 births/kt ≥1.10×、P3 deaths/birth ≤1.05×、D0 cap 在 200k 内变化 ≥20%、D1 人口不爆炸/不塌缩（40-160）。
