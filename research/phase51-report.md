# Phase 51 — 生态位资源分区：niche_offset 从「竞争负担坐标」到「资源偏好坐标」

日期：2026-11-17
基线 HEAD：`89fd730`（Phase 50 观测）
性质：**机制变更**（本期允许 re-baseline，第十三次）

---

## 1. 机制设计

### 1.1 语义扩展

Phase 16–50 的 `niche_offset ∈ [-1,1]` 只是**竞争负担坐标**（重叠 → 败方 +15% 代谢负担）：
分化只改变「谁被罚」，不改变「谁去哪吃」——所有女儿仍与母亲争夺同一片资源面，所以
deco-A 的女儿们（孙代）陆续灭绝。

Phase 51 把它扩展为**资源偏好坐标**：

| niche 区间 | 语义 | 觅食增益 |
|---|---|---|
| `niche < -NICHE_PARTITION_BAND` | 矿物偏好侧 | 矿化场高的格 ×>1，草地主导的格 ×<1 |
| `|niche| ≤ NICHE_PARTITION_BAND` | 中性带（Phase 50 行为） | ×1（精确） |
| `niche > +NICHE_PARTITION_BAND` | 草偏好侧 | 草地主导的格 ×>1，矿化场高的格 ×<1 |

### 1.2 局部资源梯度（不是全局均值）

```
rel(cell) = (grass[cell] − mineral[cell]) / (grass[cell] + mineral[cell] + 1e-9)   ∈ [-1, 1]
strength  = min(1, (|niche| − band) / (1 − band))                                   ∈ [0, 1]
mult      = 1 + NICHE_PREF_GAIN × strength × (niche > 0 ? rel : −rel)               ∈ [1−gain, 1+gain]
```

- 偏好由**所在格的局部成分**决定 → 只有在资源异质的地方才显形（Phase 45 的 regrowth-noise
  场提供异质性；矿化场由 DECO 自身的矿化沉积积累，见 §4.3 的正反馈）。
- `strength` 从带边缘连续升到轴端，**没有硬开关** → 不会形成吸收态。
- 乘数作用在**采集增益**上（不是资源开关）：资源照样被取走，只是转换效率不同。

### 1.3 常量

```rust
pub const NICHE_PARTITION_BAND: f64 = 0.3;   // 中性带
pub const NICHE_PREF_GAIN: f64 = 0.15;      // ±15% 效率摆幅
pub const NICHE_PARTITION_ON: bool = true;  // 默认 ON
```

### 1.4 作用点（全部摄入路径，通用而非 DECO 专用）

| 路径 | 位置 | 资源 |
|---|---|---|
| HERB 啃草 | `ecology.rs:4830` | grass |
| PRED 捕食 | `ecology.rs:5039` | 活猎物 |
| SCAV 食尸 | `ecology.rs:5176` | carcass |
| SCAV 啃草 | `ecology.rs:5185` | grass |
| OMN 食尸 | `ecology.rs:5304` | carcass |
| OMN 啃草 | `ecology.rs:5315` | grass |
| FISH 滤食 | `ecology.rs:5491` | aquatic grass |
| DECO 食尸 | `ecology.rs:5639` | carcass |

机制对**任何带 niche_offset 的 clade** 生效（当前只有 DECO 家族有 MST 轴，机制本身通用）。

### 1.5 性能与确定性

- 偏好查表走 `niche_by_clade`（clade id → niche 的 O(1) 缓存，每个检测 pass 重建，
  载入时重建）→ 每 agent 每 tick **O(1)**，无网格扫描、无新 RNG 流。
- 全部为纯函数（状态 → 增益），确定性不变量保持。

### 1.6 遥测

`Ecology::niche_partition_share: NichePartitionShare { mineral, neutral, grass, partitioned }`
—— 各带成员占比，每个检测 pass（60 tick）重算，纯观测，不入存档。

### 1.7 A/B 开关与存档

- `Ecology::niche_partition_off()` → 乘数恒为 1.0 ⇒ 与 Phase 50 轨迹 **bit-exact**。
- 存档 **v18**：新增 `NichePartitionOn`（`#[serde(default = "default_true")]` ⇒ v17 及更早
  档位落在 ON，即 Phase 51 默认世界）；`niche_by_clade` 缓存与遥测不入存档（载入时重建）。

---

## 2. A/B 数据（seed 20261102，200k tick）

### 2.1 终局树

```
B 臂（Phase 50，partition OFF）
clade 0  deco       birth=0       members=60  活        niche=0.000
├─ clade 1 deco-A    birth=34680   members=11  活        niche=+0.074
│  ├─ clade 2 deco-A-B 41760→59520   寿命 17760
│  ├─ clade 3 deco-A-C 78900→130320  寿命 51420  ⚔竞争排除
│  └─ clade 4 deco-A-D birth=196860 members=9  活      niche=−0.859
总 5 clade / 4 次分化 / 终局活 3

A 臂（partition ON，默认）
clade 0  deco       birth=0       members=34  活        niche=0.000
├─ clade 1 deco-A    birth=34680   members=20  活        niche=−0.894
│  └─ clade 2 deco-A-B 41760→72900   寿命 31140  ⚔竞争排除
├─ clade 3 deco-C    birth=60120   members=15  活        niche=−0.604
├─ clade 4 deco-D    birth=113040  members=11  活        niche=+0.202
└─ clade 5 deco-E    122460→140100  寿命 17640
总 6 clade / 5 次分化 / 终局活 4
```

### 2.2 指标对照

| 指标 | B 臂（Phase 50） | A 臂（Phase 51） | 变化 |
|---|---|---|---|
| 终局活 clade | 3 | **4** | +1 |
| 总 clade / 分化次数 | 5 / 4 | 6 / 5 | +1 / +1 |
| **deco-A-B（首个孙代）寿命** | 17760 | **31140** | **+75%** |
| 根族二次辐射（deco-A 之外的根族女儿） | 0 | **3**（deco-C/D/E） | 新增第二辐射面 |
| deco 家族终局总员 | 80 | 80 | 守恒（Phase 49 预算不变量） |
| pop 采样最低 / 最高 | 246 / 680 | **297 / 680** | 谷底抬升 +21% |
| 分区成员占比峰值（遥测） | —（机制不存在） | 0.6875 @t=115k（均值 0.272） | 新增 |

### 2.3 遥测轨迹（A 臂）

`t=35k` 起 `share_partitioned` 从 0 起跳（deco-A 首次越带）：0.14 → 0.26 → 0.42 → **0.69 @115k**
→ 0.44 @200k。`share_grass` 全程 ≈0——A 臂里唯一越过正带的谱系（deco-A-B）寿命太短，
偏好侧由矿物侧主导。

---

## 3. 机制强度：两个实测（同起点受控对照 + 受控小口咬）

### 3.1 DECO 的能量上限稀释效应（结构性约束）

DECO 一口 bite 的能量是 `take × 62 × deco_eff`：`take = 2.0`、`deco_eff ≈ 1` ⇒ **≈124 能量**，
而 DECO 的 `max_energy = 100` ⇒ **满口咬的增益恒被 clamp**，偏好乘数在能量侧被吞掉。

真正能显形的区间只有两种：

1. **小口**（尸体剩余 < ~1.3 能量，即每具尸体的收尾一口）；
2. **低效谱系**：deco-A 的质心 `deco_eff ≈ 0.66` ⇒ 满口增益 ≈ 81，低于上限，
   饥饿（energy < 29）时偏好可见——**恰是分化出来的那一支**。

后果（canonical 世界，seed 20261102）：ON/OFF 两臂从 t=34680 分化起点后
**到 t≈48300 才首次出现能量差**（首次可见的偏好咬需要等小口/低效区间），
首次 checksum 分叉落在 40k–60k 之间。

### 3.2 同起点受控对照（把机制直接效应与轨迹混沌分开）

在 t=34800（首分化后 120 tick，两臂尚未分叉）快照，从同一状态拷贝出 ON/OFF 两份继续跑
（`_tmp_p51_forkab`，已删）：

- 两臂统计到 t=47800 仍完全一致 → 此前的差异全部归因于偏好机制启动；
- 首次分叉 t=48300：deco-A 均能 ON 85.302 vs OFF 85.284；
- 其后（混沌放大）：t≈59800 时 ON 臂根族 312 只 / OFF 臂 227 只——
  与 200k 观测的「A 臂 pop 谷底 297 vs B 臂 246」同向（A 臂更稳）。

### 3.3 受控小口咬（单元测试实测）

同一个 staged agent 同一具 staged 尸体（take=1.0，能量起点 5），唯一变量是所在格的资源构成：

- 带内 clade（deco-A，矿物偏好侧）：草密格 **+69.30** vs 矿物格 **+75.07**（摆幅 +8.3%，
  与 strength×NICHE_PREF_GAIN×2×梯度 的理论值一致）
- 中性根族对照：两臂 **+92.986 / +92.986 bit-exact**（乘数恰为 1.0 的直接验证）

### 3.4 legacy 中立契约

偏好乘数与 Phase 16 竞争负担同样**仅限 SELECTIVE 模式**：niche 轴由 eff 差推导，
若在中立模式下生效就会把 eff 驱动的选择偷运进 `legacy_mode_shows_no_selection` 对照组
（该测试 t≤3000 无分化、测不到，但契约语义必须守住）。

---

## 4. A/B 深锚点表（seed 20261102）

| tick | B 臂（partition OFF = Phase 50） | A 臂（partition ON） |
|---|---|---|
| 300 | `0x651cf47e6d54ca64` | 同（首次分化前 no-op） |
| 3 000 | `0xc94fc584ddfd6eb3` | 同 |
| 30 000 | `0x516d99450971ff76` | 同 |
| 60 000 | `0x0ab6c03f2c29eec5` | `0x105ab3f835ff4248` |
| 100 000 | `0x57d485377da39f21` | `0x0c721a418b62cacb` |
| 200 000 | `0x83e35de52855d64d` | `0x2362175169152ceb` |
| 400 000 | （Phase 50 未采样） | `0xfb622c2a9b893e5f` |
| 1 000 000 | （Phase 50 未采样） | `0xa1910a863088e57f` |

黄金锚点（`phase31_golden_regression.rs`，第十三次 re-baseline）：**四个钉定锚点全部不变**
（@300 / @3000 / @20261105@300 / v1@300 都在首次分化 t=34680 之前，机制在钉定窗口内是
bit-exact no-op）；深锚点新增钉在 `phase51_niche_partition_test`（OFF@30000/@60000 =
Phase 50 值，ON@60000 ≠ Phase 50 值）。

## 5. 长程观测（1M，seed 20261102）

### 5.1 A 臂终局树（partition ON）

```
clade 0  deco       0→500280            ✝（根族灭绝）
├─ clade 1 deco-A    34680→             活（20 员）niche=−0.863  孙代辐射中心
│  ├─ deco-A-B  41760→72900    31140
│  ├─ deco-A-I  407940→421320  13380
│  ├─ deco-A-L  519420→585000  65580
│  ├─ deco-A-M  826140→883020  56880
│  ├─ deco-A-N  909180→928620  19440
│  └─ deco-A-O  934140→984780  50640
├─ clade 3 deco-C    60120→             活（15 员）niche=+0.515
├─ clade 4 deco-D   113040→             活（11 员）niche=−0.527
├─ deco-E  122460→140100  17640
├─ deco-F  281580→530760  249180
├─ deco-G  335880→445140  109260
├─ deco-H  386580→410580  24000
├─ deco-J  449820→490920  41100
└─ deco-K  489600→651180  161580
总 16 clade / 28 事件 / 终局活 3 / 活 clade 峰值 8（t=410k）
```

### 5.2 同口径 A/B 对比（1M）

| 指标 | B 臂（Phase 50） | A 臂（Phase 51） | 变化 |
|---|---|---|---|
| **deco-A 的女儿（孙代）寿命中位** | **22380** | **40890** | **+82.7%** |
| 孙代寿命均值 | 27471 | 39510 | +43.8% |
| 孙代样本数 | 14 | 6 | 样本更少但每个活得更久 |
| deco-A 本尊寿命 | 752820（死于 787500） | 965320+（活到 1M） | 不再死于辐射后 |
| 根族辐射子女数 | 2（deco-F/N） | **8**（deco-C…deco-K） | 根族成为第二辐射中心 |
| 终局活 clade | 2 | **3** | +1 |
| 总 clade / 事件 | 18 / 33 | 16 / 28 | 略降（周转更慢） |
| 活 clade 峰值 | 3 | **8** | +5 |
| pop 采样最低 / 最高 | 246 / 680 | 297 / 680 | 谷底抬升 +21% |
| 分区成员占比（遥测） | — | 均值 0.63，峰值 1.00 @525k | 新增 |

注：Phase 50 报告里的「女儿中位寿命 33120」口径不同（含删失与分母差异），本表用
**同一算法**：deco-A 的全部已灭绝女儿谱系的完整寿命（death−birth）取中位/均值，
B 臂数据取自 `/tmp/clade_tree_20261102.csv`（Phase 50 观测原树）。

### 5.3 套件

**177 passed / 0 failed**（基线 174 + 本期 3 个新测试），release 全量。

---

## 6. 验收（11 条）

| # | 验收项 | 结果 |
|---|---|---|
| 1 | 孙代寿命上升（具体数字） | ✅ **1M 同口径：deco-A 女儿寿命中位 22380 → 40890（+82.7%）**，均值 27471 → 39510；200k 的 deco-A-B 17760 → 31140（+75%） |
| 2 | 活 clade > 2 或第二辐射祖先 | ✅ 1M 终局活 **3**（B 臂 2），活 clade 峰值 **8**；根族在 A 臂辐射 **8 支**（B 臂 2 支）= 第二辐射中心 |
| 3 | 偏好生效可测（高 niche 族草格增益 > 低 niche 族） | ✅ 受控小口咬：矿物偏好族 草密格 **+69.30** vs 矿物格 **+75.07**（+8.3%）；中性对照两臂 bit-exact（+92.986/+92.986） |
| 4 | niche_partition_off 回到 Phase 50 轨迹（锚点） | ✅ OFF@30000 / @60000 / @200000 全部 bit-exact = Phase 50（0x516d9945… / 0x0ab6c03f… / 0x83e35de5…） |
| 5 | 200k 内 pop 不塌缩 | ✅ 1M 全程采样最低 **297**（200 个采样点仅 1 个 <300，@130000；B 臂最低 246） |
| 6 | 确定性：双跑 bit-exact；v18 往返 + v17 兼容 | ✅ 双跑/lockstep bit-exact；v18 往返后 1.5k tick 锁步 bit-exact；v17 档位（无 NichePartitionOn 键）载入为 ON 且与 v18 同世界续跑 bit-exact |
| 7 | re-baseline 第十三次锚点全表 | ✅ 见 §4（四个黄金钉定锚点**不变**；深锚点新增并入 phase51 测试） |
| 8 | release 全量 0 失败 | ✅ **177 passed / 0 failed**（39 个测试二进制） |
| 9 | wasm + Node < 16ms（三次稳定） | ✅ **14.26 / 14.42 / 14.44 ms/frame**（另一次连续批 15.22 / 14.78 / 14.58），wasm 791 KB；与 Phase 50 原版 wasm 同机交错对照 14.26 / 14.41 / 13.81 → 机制零成本 |
| 10 | 长程观测（200k 必须 + 1M） | ✅ 200k A/B 见 §2；1M A/B 见 §5 |
| 11 | 报告 | ✅ 本文 |


1. **偏好被能量上限吞掉**：DECO 满口咬恒饱和。Phase 52 可选：把偏好接入**净能量收支**
   （代谢 drain 侧，与 Phase 7 的 `eff_metab_mod` 同款通道）或按规格允许的「吸取率乘数」
   改成 bite 速率乘数。
2. **正带（草偏好侧）几乎空置**：A 臂 `share_grass ≈ 0`。需要一个能产生正 niche 的谱系
   （高 eff 分支）或把偏好方向与 eff 的耦合显式化。
3. **单种子、单世界**：全部结论来自 seed 20261102。跨种子重复是 Phase 52 的统计任务
   （每世界 200k ≈ 20 分钟，需要并行批跑）。
4. **辐射中心仍单一**：A 臂里根族成了第二辐射面（deco-C/D/E），但 deco-A 本尊只生了 1 个
   女儿——「多辐射中心」目前是根族的分裂面，不是多个祖先各自辐射。
5. **HERB 家族的 niche 轴**：只有 DECO 有 eff 突变轴，HERB/OMN 无分化轴 → 草偏好侧无载体。
   给非 DECO 家族开一条突变轴是更大的改动。

---

## 7. 性能（wasm + Node，三次测量）

### 7.1 测量

`living/test_wasm_viewer.mjs`（60 帧 sim+export 循环平均），wasm 用 `wasm-release` profile 重建：

| 二进制 | 三次测量（ms/frame） | 结论 |
|---|---|---|
| Phase 51（本期，791 KB） | **14.26 / 14.42 / 14.44**（另一批 15.22 / 14.78 / 14.58） | ✅ < 16ms |
| Phase 50（HEAD 原版，789 KB，对照） | 14.26 / 14.41 / 13.81 | 同量级 |

### 7.2 一次假警报及其排查（值得记入方法论）

中途测量曾出现 20–24 ms/frame 三次稳定超标。排查发现：**第一次被超时强杀的 cargo test
留下了孤儿进程**（cargo 被杀、子测试二进制未被杀），持续占 CPU；叠加 1M 长程观测后，
Node 侧被压到 20ms 以上。挂起孤儿与长程后复测即回到 14–15ms。
**教训**：本仓的 wasm 门限是「单次不可信」的硬要求，超标时先查宿主 CPU 占用与孤儿进程，
并用**上一版 wasm 同机交错对照**分离「环境节流」与「本期机制成本」。

机制本身是 O(1) 的（clade-id 查表 + 两个格点读数），对照测量也证实成本在噪声内。

---

## 8. 遗留与 Phase 52 衔接

1. **偏好被能量上限吞掉（主因）**：DECO 满口咬恒饱和（124 vs 上限 100），canonical 世界
   ON/OFF 到 t≈48300 才首次分叉。Phase 52 首选：把偏好接入**净能量收支**（代谢 drain 侧，
   与 Phase 7 `eff_metab_mod` 同款通道）或按规格备选「吸取率乘数」改成 bite 速率乘数
   （`bite = DECO_BITE_INTERVAL / pref`，无 clamp、无新 RNG）。
2. **正带（草偏好侧）几乎空置**：A 臂 200k `share_grass ≈ 0`——唯一越过正带的 deco-A-B
   短命。需要高 eff 分支存活（或 HERB/OMN 开分化轴）才能激活另一侧。
3. **单种子单世界**：全部结论来自 seed 20261102；跨种子批跑（每世界 200k ≈ 20 分钟）
   是 Phase 52 的统计任务。
4. **孙代样本量仍小**：A 臂 1M 里 deco-A 生了 6 个女儿（B 臂 14 个）——A 臂的根族分走了
   更多生态位，deco-A 的辐射机会相应减少（总 clade 16 vs 18）。「孙代中位 40890 > 22380」
   建立在 A 臂 6 个 / B 臂 14 个已完成寿命的样本上，需要跨种子重复才能定性。
5. **HERB/OMN 无分化轴**：机制通用但当前只有 DECO 有 MST 轴——草偏好侧无载体。

---

## 9. 产物

- 测试：`core/crates/aether-sim-ecology/tests/phase51_niche_partition_test.rs`
- CSV：`/tmp/p51_on_fate.csv`（A 臂 200k）、`/tmp/p51_base_fate.csv`（B 臂 200k）、
  `/tmp/p51_on_1m_fate.csv`（A 臂 1M）
- 树：`/tmp/p51_on_tree.csv`、`/tmp/p51_base_tree.csv`、`/tmp/p51_on_1m_tree.csv`
- 受控对照：`_tmp_p51_forkab`（已删；数据在本文 §3.2）
