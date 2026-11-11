# 「以太」虚拟世界 · 总体架构设计 v2.0

> 版本：v2.0 · 2026-11-09
> 取代：`research/architecture-v0.2-js-era.md`（v0.2 的 JS 内部重构路线 —— 因目标尺度从「山谷生态」跃升为「宇宙 ↔ 原子」而废弃其主路线；其中对现有 JS 代码的痛点分析与源码行号引用仍然有效，作为移植时的 oracle 说明书保留）
> 决策依据：引擎能力调研（Three.js 实例化上限 / UE5 Nanite+MassEntity+双精度 / Unity DOTS 确定性 / Bevy ECS 并行与 wasm）+ 跨尺度坐标精度问题调研（f32 抖动、参考系树、floating origin）

---

## 0. 一句话定调

**「以太」不是一个 3D 场景项目，而是一个多尺度确定性宇宙仿真器；渲染只是它的观景窗。**

当前 `world/living/`（6426 行 JS）是这条路上一次成功的**小尺度原型**：它证明了确定性生态规则可以在浏览器里跑出「无剧本演化」。但它的架构上限是「山谷 × 650 实体 × 单一尺度」。目标尺度是**约 40 个数量级**的空间跨度（10⁻¹⁰ m 的分子 → 10²⁶ m 的可观测宇宙）与同样悬殊的时间跨度（飞秒化学反应 → 亿年恒星演化）。

**在这个尺度上，没有任何现成引擎「开箱即用」**——UE5 有世界分区但没有分层仿真语义，Three.js 有渲染弹性但没有双精度坐标与并行仿真。正确的做法是：**仿真心自建（Rust + ECS），渲染层借力（先 Three.js 后 wgpu/WebGPU），用三个架构范式把「无限尺度」变成工程问题。**

---

## 1. 引擎与语言选型（已裁决）

### 1.1 决策矩阵

| 候选 | 仿真确定性 | 并行能力 | 跨尺度坐标 | 实体规模上限 | 生态/授权 | 裁决 |
|---|---|---|---|---|---|---|
| **Rust + Bevy ECS（自建 sim core）** | ✅ 天然友好（确定性 RNG 自控、fixed timestep 自控） | ✅ 无锁并行调度，原生多线程 | ✅ 自己实现参考系树（f64 锚点） | 亿级实体（SoA 缓存友好） | MIT 开源，零授权风险 | **✅ Sim Core 选定** |
| **Three.js（渲染层）** | 不适用（只渲染） | 不适用 | ❌ 无大世界支持（f32 单精度，无 origin 重定基） | 实例化 ~10⁵、非实例化 ~10⁴ | MIT | **✅ 过渡期渲染器** |
| **原生 wgpu / WebGPU** | 不适用 | ✅ compute shader | 同上自建 | 10⁷ 粒子级（vertex pulling） | MIT | **✅ 渲染终态候选**（Phase 4+） |
| UE5 | ❌ Chaos 物理非确定性；自建确定性仿真要在 C++ 里对抗引擎 | ✅ MassEntity | ✅ 内建 LWC 双精度 | 极高（Nanite） | 免费→$1M 后 5% 抽成；C++ 工具链重 | ❌ 渲染溢价买不到本项目要的仿真语义；Web 不可达 |
| Unity DOTS | ⚠️ Unity Physics 确定性可行 | ✅ Burst+Jobs | ⚠️ 双精度支持长期在路上 | 高 | 闭源黑盒；$2,310/座/年 | ❌ 闭源 + C# 天花板 + 路线受制于人 |
| 继续 pure JS | ⚠️ 可维持（现有测试已证明） | ❌ 单线程（Worker 无法共享仿真状态，见 scout 报告 §3 SAB 双死路） | ❌ 需自建且无引擎支持 | 650 实体已到舒适区上限，10⁴ 后帧率崩 | — | ❌ 原型已完成使命 |

### 1.2 裁决理由的三句话版本

1. **仿真是这个项目的本体**。「同 seed 同结果」的确定性 + 亿级实体的数据导向迭代 + 原生多线程，交集里只有 Rust ECS 一条路（Bevy 是开源标杆，也可只用其 ECS 思想自建轻量 core——见 §4.3）。
2. **渲染是可替换的观景窗**。Sim 产出「渲染包」（不可变快照），任何渲染器（Three.js / wgpu / 未来whatever）都是它的消费者。过渡期 Three.js 让 Web 端零成本存活，终期原生 wgpu 让桌面端吃满 GPU。
3. **UE5/Unity 的溢价买错了东西**。它们卖的是「画面与资产管线」，本项目要的是「规则与涌现」——为用不到的 Nanite 付出非确定性物理与闭源锁定，是负资产。

---

## 2. 三大架构范式（本设计的灵魂）

这三个范式共同把「宇宙到原子」从哲学概念变成可实现的工程结构。

### 2.1 范式一：参考系树（Reference Frame Tree）

**问题**：f32 浮点在离原点 ~10 km 处就开始视觉抖动（1 m 的坐标在 10⁴ m 尺度下精度只剩 ~1 mm，在 10⁷ m 下只剩 ~1 m）。跨 40 个数量级的项目，单一世界坐标系必然崩溃。

**解法**（业界成熟模式：Foresight Transform64 / Babylon floatingOriginMode / 开源 cosmos 项目的 scale-frame tree）：

```
universe（f64 锚点，单位：秒差距）
 └─ galaxy（f64，单位：光年）
     └─ star_system（f64，单位：AU）
         └─ planet（f64，单位：米）
             └─ region（f64，单位：米，即现在的山谷）
                 └─ organism（f64，单位：毫米）
                     └─ organ（f64，单位：微米）
                         └─ cell（f64，单位：微米）
                             └─ molecule（f64，单位：埃）
```

- 每个节点 = 一个**仿真上下文（ScaleFrame）**：自己的原点锚（f64，相对父系）、自己的单位比例尺、自己的物理规则集、自己的时间步。
- 任何实体位置 = `UniversePosition { frame_id, local: [f64;3] }`。跨系换算沿父链在 **f64 域完成后再降级 f32**，渲染只拿「相机所在 frame 的相对坐标」。
- **Floating origin**：渲染帧开始时，把相机所在 frame 的锚重定基（rebase），保证 GPU 侧永远只处理小数值 f32。换算纯函数、原子化、每帧一次。

### 2.2 范式二：观察驱动细节化（Observation-Driven Materialization）

**问题**：不可能同时模拟 10²¹ 颗恒星的完整生态 + 每颗行星的全部物种 + 每个生物的全部细胞。算力是有限的，「无限细节」必须变成「按需细节」。

**解法**——这直接继承自项目旧主线（cascade.py 的「观察者坍缩」）的哲学，把它从隐喻变成调度机制：

- 每个实体有**细节档位（LOD-sim，不是渲染 LOD）**：
  - `L0 统计影`：只维护聚合量（如「这颗行星有 ~10⁶ 物种，多样性指数 X」）。星系视角下的一颗行星就是 L0。更新成本 ≈ O(1)。
  - `L1 代理群`：几百个聚合代理（类似当前 JS 生态的 650 agent，但可能带简化规则）。区域视角。
  - `L2 全体仿真`：每个个体独立实体、完整行为/基因/交互。当前山谷的原型就是 L2。
  - `L3 物理/化学细节`：分子动力学、代谢通路。显微镜视角，局部小空间内激活。
- **提升/降级规则（refine/coarsen）是显式的一等公民**：
  - refine（L0→L2）：用 L0 的统计量按分布采样生成 L2 个体（基因分布、年龄结构、空间分布来自聚合参数）。**不要求可逆精确，只要求统计一致**。
  - coarsen（L2→L0）：把个体状态聚合回统计量（总数、基因均值/方差、空间质心）。离开观察范围即触发。
- **确定性约束**：refine/coarsen 的采样只从 `SimRng` 取随机数，且调用顺序由「观察事件日志」的确定顺序驱动 → 同 seed 同观察路径 = 同结果。观察路径本身成为存档的一部分。

**这就是「无限可能性」的工程化**：世界在任何时刻只精确模拟玩家「看着」的那一小块，其余以统计影子存在——而统计影子的演化规则同样是确定性的、真实的（物种会在未观察时兴衰，只是以 L0 粒度）。

### 2.3 范式三：分层异步时间（Hierarchical Asynchronous Time）

**问题**：分子的化学事件在飞秒尺度，生态演化在世代尺度，恒星演化在百万年尺度。统一时钟下，要么分子等死生态（浪费 10¹⁸ 个 tick），要么生态一夜跳过恒星（细节丢失）。

**解法**：
- 每个 ScaleFrame 有自己的 `dt` 与「节拍预算」：`molecule: dt=10⁻¹⁵s`、`cell: dt=ms`、`organism/ecology: dt=0.1s`（沿用现原型）、`planet/orbit: dt=分钟`、`galaxy: dt=年`。
- 层间不逐 tick 同步，而以**锚点事件（anchor events）**交换：如「organism 层每 100 tick 向 region 层上报一次聚合」「planet 层每当地表季节切换，向 region 层广播光照/温度场更新」。
- 跨层因果通过**版本戳（lamport 风格逻辑时钟）**保证：任何跨层读取都读到「不晚于本地时间」的状态，避免时间悖论。

---

## 3. 目标总架构图

```
┌──────────────────────────────────────────────────────────────────┐
│                       渲染 / 交互层（可替换）                       │
│   Phase 2-3: Three.js/WebGL2 viewer（Web + 安卓 WebView）          │
│   Phase 4+:  wgpu 原生 viewer（桌面，WebGPU compute 粒子/体素）      │
│   消费：RenderPacket（不可变快照）  上行：ObservationIntent（观察意图）│
├──────────────────────────────────────────────────────────────────┤
│                        Sim Core（Rust）                           │
│                                                                  │
│  ┌────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │ 坐标系 core     │  │ 实体 core (ECS)   │  │ 时间 core         │  │
│  │ FrameTree      │  │ Bevy ECS 或       │  │ HierarchicalClock│  │
│  │ UniversePos    │  │ 自研轻量 ECS      │  │ anchor events    │  │
│  │ floating origin│  │ SoA 存储          │  │ lamport 版本戳   │  │
│  └────────────────┘  └──────────────────┘  └──────────────────┘  │
│  ┌────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │ 规则模块（每层） │  │ 细节化调度器       │  │ 确定性基座        │  │
│  │ chemistry/     │  │ materialize/     │  │ SimRng           │  │
│  │ biology/       │  │ dematerialize/   │  │ (mulberry32 对齐) │  │
│  │ ecology/       │  │ LOD-sim 档位     │  │ 事件顺序日志      │  │
│  │ planetary/     │  │ 观察路径回放      │  │ 固定 timestep    │  │
│  │ stellar/       │  │                  │  │                  │  │
│  └────────────────┘  └──────────────────┘  └──────────────────┘  │
├──────────────────────────────────────────────────────────────────┤
│   持久化：世界快照（分层，frame 粒度）+ 观察路径日志 + 规则版本号      │
└──────────────────────────────────────────────────────────────────┘
```

**数据流是严格的单向环**：
`ObservationIntent`（观察者意图：相机位置、焦点实体）→ 细节化调度器（refine/coarsen 决策）→ Sim Core 推进 → `RenderPacket`（含：相机 frame 的 f32 相对坐标、实例化批次、可选的聚合统计）→ 渲染层画出来。渲染层**永远不写** sim 状态（吸取 v0.2 痛点 P3/P5 的教训，从第一天就切断）。

---

## 4. Sim Core 设计细节

### 4.1 目录结构（Rust workspace）

```
world/core/                     # 新建 Rust workspace
  Cargo.toml
  crates/
    aether-coords/              # 范式一：参考系树
      src/lib.rs                #   FrameTree, UniversePos, rebase()
    aether-clock/               # 范式三：分层时钟
      src/lib.rs                #   HierarchicalClock, AnchorEvent
    aether-rng/                 # 确定性基座
      src/lib.rs                #   SimRng（mulberry32 语义对齐 JS 版，用于对拍）
    aether-ecs/                 # 实体存储
      src/lib.rs                #   初期直接用 bevy_ecs crate；若 wasm/嵌入场景受限则换自研 SoA
    aether-sim-ecology/         # Phase 2 移植目标：现 JS 生态规则
      src/                      #   grassfield / genome / species / reproduction / cooperation ...
    aether-sim-chem/            # Phase 4+：分子层（L3）
    aether-sim-stellar/         # Phase 4+：恒星/行星层（L0 统计影起步）
    aether-materialize/         # 范式二：refine/coarsen 调度器
    aether-render-packet/       # RenderPacket 结构定义（serde 序列化）
    aether-headless/            # node/python 可调用的 headless 驱动（wasm-bindgen / FFI）
world/living/                   # 现有 JS 项目：冻结为 oracle + 过渡期 viewer（见 §6）
world/research/                 # 调研与决策存档
```

### 4.2 ECS 选型：先用 bevy_ecs，保留自研退路

- **Phase 1-2 直接依赖 `bevy_ecs`**（只取 ECS，不取 Bevy 的 App/renderer——避免被整引擎绑架）：查询并行、Change Detection、组件即 struct，开发效率最高。
- **退路**：若后续遇到 wasm 体积 / 嵌入式 / 参考系树深度耦合等摩擦，`aether-ecs` crate 的公共 trait 保证可替换为自研 SoA 存储（bitECS 的 TypedArray 序列化思路的 Rust 版）。规则模块只依赖 trait 不依赖具体 ECS 实现。

### 4.3 确定性协议（对拍 JS oracle 的根基）

1. `SimRng = mulberry32` 的 Rust 逐位对齐实现（含 `prng` 第二流），单元测试用 JS 版生成 10⁶ 个数做黄金样本比对。
2. 所有规则代码**禁止** `rand::thread_rng` / `std::time` / 浮点跨平台分歧操作（`powf/sin/cos` 需走自有的 `compat` 模块，与 JS `Math.*` 逐位对齐——这是对拍能成立的前提）。
3. 每个 sim tick 的输入 = `(tick, [anchor events])`，输出 = `RenderPacket + 状态哈希`。**对拍协议**：同 seed 下，JS `Ecology` 与 Rust `sim-ecology` 各跑 N tick，比较：agent 数量序列、每 tick 状态哈希（JS 侧需给 ecology.js 加一个 `checksum()` 方法——纯读遍历，不改逻辑，约 20 行）。哈希不一致 = 移植 bug，逐字段二分定位。
4. 现有 10 个 `test_*.mjs` 的**验收标准原样翻译**成 Rust 集成测试（`test_ecology` / `test_isolation` / `test_cooperation` / `test_grassSpeciation`...），先在 JS 上跑通、再在 Rust 上跑通、最后交叉对拍。

### 4.4 Headless 双端运行

- **原生**：`cargo run -p aether-headless -- --seed 20261102 --ticks 36000`（快，CI 用）
- **浏览器**：同一 crate 编译 wasm32（wasm-bindgen），供过渡期 Three.js viewer 调用——sim 在 wasm 里跑，JS 只做渲染与 UI，与 scout 报告的「MessageChannel+Transferable」协议天然契合（wasm 线性内存可直接 transfer）
- 注意 wasm 下多线程不可用的 scout 结论：**wasm 仿真跑单线程版本，原生跑并行版本，两者确定性必须一致**（并行系统的确定性调度：system 执行顺序固定表 + RNG 消耗顺序与执行顺序绑定——这是 Bevy ECS 的已知难点，Phase 2 首个攻坚点，详见 §7 风险 R2）

---

## 5. 渲染层策略

### 5.1 过渡期（Phase 2-3）：Three.js viewer

- 现有 `living/` 的地形/天空/水面/地标资产与 post 管线**原样复用**，但改造为「RenderPacket 消费者」：agents 不再由 JS 生态驱动，而由 wasm sim 的快照驱动。
- 抽掉 `living/` 里与 JS 生态的双向耦合（v0.2 痛点 P3/P5 对应的 `Math.random` 天气、refreshShade 反向写回、playerX/torch 直写）——shade/fires 等字段改由 sim 在 Rust 侧计算并放进 RenderPacket。
- 单文件离线产物能力保留（`vite-plugin-singlefile` + wasm inline）。

### 5.2 终态（Phase 4+）：wgpu 原生 viewer

- 触发条件（满足其一开始）：
  1. 单帧实例数稳定超过 ~2×10⁵（Three.js 实测舒适上限）；
  2. 需要 GPU compute 做渲染侧加速（粒子系统 10⁷ 级、体素地形、raymarch 云）;
  3. 产品形态转向桌面优先。
- wgpu 直接对接 Sim Core 的 RenderPacket，与 Three.js viewer 长期并存（web 用轻量版，desktop 用完整版）——**渲染器是插件，不是宿主**。

---

## 6. 现有 JS 项目的处置（绞杀者时间表）

| 阶段 | `world/living/`（JS） | `world/core/`（Rust） |
|---|---|---|
| **Phase 0（本周）** | 冻结：不再加新特性；补 `ecology.checksum()`；10 个测试继续作 oracle | 空仓库搭好 workspace 骨架 |
| **Phase 1（1-2 周）** | 只修确定性 bug（P3 的两处 Math.random） | coords / clock / rng / ecs 四个基座 crate + 单元测试 |
| **Phase 2（2-4 周）** | oracle：同 seed 对拍基准；发现移植歧义时以 JS 行为为准并记录差异 | sim-ecology 移植（草场→基因组→四物种→繁殖→合作→隔离，逐模块对拍） |
| **Phase 3（2-3 周）** | 改造为纯 viewer：删 JS 生态调用，接 wasm sim 快照；单文件产物照常发布 | render-packet + wasm 导出 + headless CLI |
| **Phase 4（按节奏）** | 逐步退役（viewer 功能并入新渲染路径后归档） | materialize 调度器 + 分层时间 + 化学/恒星层起步 —— **「宇宙到原子」从这里开始** |

**明确不做**（否则又是无限工程）：不把 v0.2 的「JS 内部三步重构」做下去——Phase 0 只修两处确定性 bug，其余痛点（P1/P2/P6/P8）随 JS 退役自然消失，不再投入。

---

## 7. 风险与未决

| # | 风险 | 缓解 |
|---|---|---|
| R1 | **移植对拍失败**：JS/Rust 浮点行为差异（`sin/pow` 实现、f64→f32 收缩时机）导致哈希永不一致 | `compat` 模块逐函数对齐 + 黄金样本测试先行；若个别函数无法逐位对齐，允许「统计等价」降级验收（分布检验代替逐 tick 哈希），并在 oracle 文档记录偏差 |
| R2 | **并行确定性**：Bevy 多线程 system 调度顺序不定 → RNG 消耗顺序不定 → 原生与 wasm 结果分叉 | Phase 2 期间 sim-ecology 先全串行（确定性铁律优先）；并行化推迟到 Phase 3 后，用「固定 system 顺序表 + 每 system 独立 RNG 子流（按实体 id 派生）」方案，天然与调度顺序解耦 |
| R3 | **范围失控**：「宇宙到原子」容易变成永远做不完的银弹 | 纪律：Phase 4 之前不写一行化学/恒星代码；每个 Phase 有硬验收（§6 表格）；参考系树和 materialize 调度器在 Phase 1-2 就以最小可用形态落地（哪怕只有 region 一层），保证范式被真实验证而非纸面设计 |
| R4 | **单人工程量**：Rust + ECS + 多尺度是三人年级别的领域 | 善用 LLM 协作开发（本项目本身就是人机协作产物）；bevy_ecs 只取不绑；每一步都有可运行的中间产物（对拍绿 = 前进一步） |
| R5 | **现 viewer 资产沉没**：地形/天空 shader 是手工调出来的 | Phase 3 保留 Three.js viewer 路线，资产零损失；wgpu viewer 起步时也只复刻必要通道 |
| R6 | **wasm 体积与加载**：sim core 编译 wasm 可能数 MB | `wee_alloc` + `opt-level="z"` + 特性裁剪；headless 对拍全走原生，wasm 只承载过渡期 viewer 所需的 ecology 层 |

---

## 8. 术语表

| 术语 | 含义 |
|---|---|
| **ScaleFrame** | 参考系树的一个节点：f64 锚点 + 单位比例尺 + 物理规则集 + dt |
| **UniversePosition** | `{ frame_id, local: [f64;3] }`，实体唯一真位置 |
| **Floating origin / rebase** | 每帧把相机所在 frame 锚点平移回近旁，使渲染坐标保持 f32 安全 |
| **LOD-sim** | 仿真细节档位（L0 统计影 / L1 代理群 / L2 全体仿真 / L3 物理细节），与渲染 LOD 正交 |
| **refine / coarsen** | 细节档位的提升/降级变换；refine 从统计采样生成个体，coarsen 聚合回统计量；采样只走 SimRng |
| **ObservationIntent** | 渲染层上行给 sim 的观察意图（相机 frame 位置、焦点、期望档位） |
| **RenderPacket** | Sim 下行给渲染器的不可变快照（f32 相对坐标批、聚合统计、事件流） |
| **Anchor event** | 分层时钟之间的同步事件（如季节切换广播） |
| **Oracle** | 冻结的 JS 生态实现：同 seed 行为基准，Rust 移植的对照物 |
| **对拍（differential testing）** | 同 seed 同 tick 下 JS 与 Rust 状态哈希逐点比对，定位移植偏差 |

---

## 9. 下一步（获批准后立即执行）

1. **Phase 0**：给 `living/src/ecology.js` 加 `checksum()`（~20 行纯读遍历）；修 `terrain.js:841` 与 `main.js:1482` 两处 `Math.random` 确定性污染（改用 seeded rng）；跑通 10 测试确认零回归 → oracle 冻结
2. **Phase 1**：`cargo init` `world/core/` workspace；实现 `aether-rng`（黄金样本对拍 JS）与 `aether-coords` 最小版（先只两层：planet/region）；`aether-clock` 单层版
3. **Phase 2 启动前**：将本文档的 §4.3 对拍协议与 §6 时间表同步给 DS Bot 团队（architect 出 Rust 模块规格，scout 补 bevy_ecs 并行确定性的现状调研）
