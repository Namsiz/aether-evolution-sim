# 「以太 · 自生世界」虚拟世界架构设计文档

> 版本：draft v0.2 · 起草：ds-architect · 校准：Chief 依据 scout 调研报告（`research/tech-scout-2026-11.md`）合并修订  
> 工作区基准：`~/workspace/world/living/`（源码在 `living/src/`，构建产物 `living/aether.html`）

---

## 1. 定位

**一句话愿景**：一个以达尔文演化为核心的离线可观测程序化生态——同 seed 在任何设备运行结果严格一致，玩家可以「走进」「观察」但**不被允许交互性干预**物种命运。

**明确不在范围**（本轮不动）：
- 游戏化机制（无背包/建造/任务/血量消耗——即便现有 `updateHunger/updateThreat/respawn/nearestTarget` 已半实现，本轮按「no 游戏化」原则剥离或删除）
- 3D 渲染写实升级（⑪ 视觉基因化 除外——那是基因→纹理映射，属 sim → renderer 数据流，不是着色器重写）
- WebGPU / Post-processing shader overhaul（移动端 WebGL 兼容性约束：Android WebView 支持弱 [待 scout 报告]）
- 多玩家/服务器（单进程确定性 sim 是硬需求，任何 worker 方案都禁止共享可变状态）

---

## 2. 现状痛点（基于源码证据）

| # | 问题 | 代码证据 |
|---|------|---------|
| P1 | **main.js 2127 行承担渲染初始化 + post chain + UI setup + 四种图表绘制 + HUD stats 更新 + 步行模式物理 + 环境天气 + 存档/回放控件 + 调试 expose** —— 所有功能耦合在单一 IIFE 入口 | `src/main.js:1` 起读至 `:2127` 结束；可见子函数：`setupRenderer(@L146)`、`setupPost(@L194)`、`setupUI(@L596)`、`drawChart/@L766`、`drawEvoChart/@L827`、`updateStats/@L875`、`updateEnvPanels/@L1503`、`toggleReplay/@L1551`、`setMode/@L1188`、`updatePlayer/@L1271`、`placeBuild/@L1829`（空壳）、`animate/@L1882`、`boot/@L2040`、`window.__aether/@L2076` |
| P2 | **ecology.js 1739 行混合草场、四物种行为、基因组突变、间接互惠、择偶偏好、基因壁垒、物种形成检测、世代统计、尸体循环** —— 十数个概念域被塞进同一个 class，无法单独替换任一子系统 | 方法散点：`_initGrass @L158`、`_seedLife @L219`、`_takeGrass @L451`、`_tryHelp @L514`、`_gutTransit @L582`、`_reproduce @L733`、`_pairCompatible @L798`、`_sigMatch @L827`、`_die @L852`、`_updateHerb @L868`、`_updateScav @L1039`、`_updateOmn @L1097`、`_updatePred @L1174`、`update @L1330`、`_sample @L1459`、`stats @L1641` |
| P3 | **确定性 rng 污染** —— `terrain.addBush` 使用非种子驱动的 `Math.random()` (@`src/terrain.js:841,843,847`)，破坏 headless 复现；`updateWeather` 同样依赖 `Math.random()` (@`src/main.js:1482-1483`) 并将随机雨/晴状态直接注入 `ecology.rain`，使同 seed 的生态步长不可预测 | `test_ecology.mjs` 仅导入 ecology 绕过了 terrain，因此测试仍然通过；生产构建暴露不一致 |
| P4 | **双时钟** —— 生态模拟用 `this.time`（累积 sim seconds，由 substep 0.1s 累加），昼夜由外层传入的 `daylight`（源于 `env.sunElevation` 经 `gameHours % 24` 换算）；两者仅在 `rain` 字段间接同步，且 `state.gameHours += raw / 48.0` (@`src/main.js:1915`) 与 ecology time 增长速率不同步（gameHours 是小时尺度模拟，ecology time 是秒） | `_updateHerb @L868` 读取 `this.daylight`，而该字段完全来自外部调用方 |
| P5 | **渲染层反向写 sim 状态** —— `refreshShade() @src/main.js:1128` 遍历渲染侧 `terrain.treeMeshes` 的实例几何矩阵，反算阴影格并写入 `ecology.shade`（@`src/main.js:1133`）；生态 sim 本应自包含地形阴影逻辑，却从渲染对象倒推 | 同时 `ecology.fires`（read 于 `src/ecology.js:904-913` H 警觉、`:1246-1255` P 避火）从未被主场景写入——dead field；walk 模式的 `structures[].userData.fire` 与 sim fires 完全割裂 |
| P6 | **stats() 在无缓冲下高频重算** —— `updateStats(@L875)` 每 ~380ms 调用一次 `ecology.stats()`，内部又调 `plantAmount()/plantAmountBy()/generationFitness/evolution[kind] 过滤/isolatedGroups/map/cargoTotal/deposit p50/p90`；`drawEvoChart/@L827` 每次单独再拉一次 stats；`updateEnvPanels/@L1503`、`updateEcoHud/@L1488`、调试接口 `window.__aether.info() @L2101-2102` 也各拉一次——同一时间窗内最多 4-6 次全量遍历 | 当前 650 agent × 3 个 species-filter × 几个 reduce 还勉强，规模扩大必崩 |
| P7 | **生态模拟步长耦合到帧率** —— `animate @L1882` 中 `raw = dtClamp(clock.getDelta())` 限制在 50ms，但 sim substep guard `while (acc >= step && guard < 26) @L1945` 有上限 26 步/帧；浏览器标签页失焦时会堆积 delta，导致单帧 26×0.1=2.6s tick 爆发——造成瞬时大量死亡/出生事件（guard 防止无限卡死，但丢失平滑度） | — |
| P8 | **Walk 模式残留游戏化骨架未清理** —— `invbar @index.html:159`（饱/命/世界三栏）、`updateHunger/@L1380`、`updateThreat/@L1389`（内部 `src/main.js:1408` 已是空操作「玩家为观察者，无伤害」的死分支）、`respawn/@L1422`、`nearestTarget` 中的 `storage/bed` 引用 @`src/main.js:1362-1375`、空函数 `placeBuild/@L1829` `cycleBuild/@L1830` —— 与「不破坏 no-game」诉求矛盾 | — |

---

## 3. 目标总图（文字 ASCII 分层图）

```
┌─────────────────────────────────────────────────────────────┐
│  URL / index.html                                            │
│  └─ build.js (单入口 bootstrap：加载 core bundle + ui bundle)│
├──────────┬──────────┬──────────────┬───────────────┬─────────┤
│  Core    │ Renderer │      UI      │   Persist     │  Dev   │
│          │          │              │               │        │
│ world.js │ scene.ts │ dashboard.ts │ state-schema  │ tsconf │
│ ecology. │ camera.ts│ panels.ts    │ snapshot.json │ eslint │
│   grass  │ terrain.js│ inspect.ts  │ (planned)     │ prettier│
│   genome │ sky.js   │ tools.ts     │               │ vitest │
│   move   │ agents.js │ replay.ts    │               │        │
│   coop   │ landmarks.js│ walk-mode.ts│              │        │
│   repro  │          │ input.ts     │               │        │
│   spec   │          │              │               │        │
│  _tick(dt│          │              │               │        │
│  ,dusk)  │          │              │               │        │
├──────────┴──────────┴──────────────┴───────────────┴─────────┤
│  Three.js WebGL  ·  OrbitControls  ·  EffectComposer           │
│  UnrealBloomPass / SMAAPass / OutputPass                      │
├─────────────────────────────────────────────────────────────┤
│  Build pipeline (single IIFE bundle → aether.html)             │
└─────────────────────────────────────────────────────────────┘

数据流向：
  sim.core.tick(dt, daylight) → agents[].position/state/history/evolution[]
            ↑                              ↓
  env.sunElevation(gameHours%)         agents.update(alpha, elapsed)
  weather.rain (seeded from ecotime?)  ──► renderPass.render(scene, cam)
            ↓
  refreshShade(treeMeshes) ──→ ecology.shade[]  (renderer → core writeback)
```

**关键设计决策**：
1. **Core ↔ Renderer 单向接口**：core 只对外暴露 immutable snapshot（agents 数组，每 tick 更新）+ shade 字段；renderer 读取 shade 但不反向写 ecology 其他字段（`playerX/playerZ/torch` 改为核心主动 poll renderer state via 回调接口，见 §5）。
2. **daylight 来源统一**：由 `GameClock` 对象驱动（见 §8 tooling），核心不再依赖外部 daylight——改由 `sim.getTime()/getDaylight()` 函数对 `worldTime`（秒）查表。渲染侧传入 `state.gameHours` 只是 clock 的 display，不再双向耦合。
3. **无跨线程 sim**（详见 §9）。

---

## 4. Sim Core 详解

### 4.1 职责边界

Sim Core 是**纯计算**模块——输入给定时间步 `dt` 与 `daylight ∈ [0,1]`，输出确定性新状态。**零副作用**：不访问 DOM / 不触发 console / 不调 THREE.js。Headless Node 能跑的约束继续有效。

保留的模拟常量全部集中在 `constants.ts`（原 `ecology.js:16-55` 的 const 区）：`GRID_N=76, GRID_SPAN=1080, CELL, MAX_HERB/PRED/SCAV/OMN`、`HELP_*`、`SIG_*`、`WINTER_*`、`FRUIT_*`、`GUT_*`、`TRAIL_SLOTS` 等。命名统一为中性代号（遵循研究笔记风格）。

### 4.2 新增模块清单

```
living/src/core/
  constants.ts      ← ecology.js L16-55 搬出
  world-utils.ts    ← world.js（保持不变，仍是 sim 依赖）
  agent.ts          ← Agent 类型定义 + spawn() 工厂
  genome.ts         ← _randomGenome / _mutate / _mixGenome / _triangular
  behaviour.ts      ← _steer / _avoid / _clampHome / _nearest
  update_herb.ts    ← _updateHerb
  update_pred.ts    ← _updatePred
  update_scav.ts    ← _updateScav
  update_omn.ts     ← _updateOmn
  reproduction.ts   ← _reproduce + _pairCompatible + _sigMatch
  cooperation.ts    ← _tryHelp + _winterAggregationPull + _gutTransit
  speciation.ts     ← speciation 检测 + isolatedGroups 维护（从 _sample 提取）
  grassfield.ts     ← _initGrass + _takeGrass + 草地生长/扩散 loop
  sampling.ts       ← _sample + stats + evolution[herb/pred/scav/omn] 时序
  ecology.ts        ← Ecology class 薄包装（公开 API 不变）
```

**为什么这样切分**：`ecology.js` 内已有清晰的逻辑分区（cooperation 自含注释，speciation 自含，genome 方法群集中），只是放在同一个类里。拆成独立文件保持委托关系，接口签名 100% 向后兼容（tests 只需改 import 路径，不改调用方式）。

### 4.3 现有清晰度亮点（如实指出）

- `grassP[i]` / `suitEffBy[i]` / `moist[i]` / `shade[i]` 四个 5776 长度 Float32Array 的语义边界清晰——分别代表「分种草量」「湿度适应度」「水边湿润加成」「树荫遮蔽」。它们共同组成草地格子模型，不需要重组 [待 Chief 确认：是否保留三组而非合并成一个 grid？目前 OK，不动]。
- `_spreadBuf` 双缓冲避免偏置的做法（`src/ecology.js:1370-1391`）是经典的网格模拟防向量化偏差技巧，值得保留。
- `history` 滑动窗口限 240 条目（≈ 8 min at 30 ticks/min = 240 s × 0.5 s/sample? 实际 history push 间隔 20 tick ≈ 2s，窗口 240 ⇒ ~8 分钟历史）— 对于图表有用，对于存档无用 [与 persist 层解耦处理]。

### 4.4 新 sim 对外暴露的最小接口

```ts
// living/src/core/ecology.ts（导出名不变，tests 零修改）
export class Ecology {
  constructor(seed?: number);
  // 读（不创建/不变异）
  get agents(): ReadonlyArray<Agent>;
  get alive(): Array<Agent>;       // 别名，tests 用
  get time(): number;
  get season(): 0|1|2|3;
  get daylight(): number;
  get history(): HistoryPoint[];
  get evolution(): EvolHistoryMap;
  // 写入（外部驱动，仅此三个入口；其余字段一律只读）
  setRain(v: number): void;       // 原 ecology.rain 直接赋值（改由 GameClock/weather 驱动）
  setShade(s: Float32Array): void; // 原 main.refreshShade 反向写的入口
  setProbe(p: RenderProbe): void;  // renderer 只读探针（player pos / fires / torch）
  // 步长
  update(dt: number, daylight: number): void;
  hunt(agent: Agent): boolean;
  living(): Array<Agent>;         // stats/chart 用
  stats(): EcosystemStats;
}
```

**为什么不引入 interface separation**：现有 tests 直接 `new Ecology(seed)` 跑；增加抽象只添麻烦，且 sim 核心小不需要面向接口的松耦合。

---

## 5. Renderer 层

### 5.1 现状（main.js 承担的渲染职责）

| 职责 | 当前位置 | 大小（约） |
|------|---------|----------|
| Renderer init + WebGL context | `setupRenderer @L146` | 50 行 |
| Post-processing chain（Bloom+SMAA+Output+Grade） | `setupPost @L194` | 60 行 |
| Camera control (Orbit + iso + walk) | `updateIsoCamera @L254`, `onResize @L274`, `moveCamera @L1965`, `updateWalkCamera @L1837` | 100 行 |
| Terrain mesh & instanced grass/tree/rock/bush | `src/terrain.js:createTerrain @L215` | 整个文件 1209 行 |
| Sky/water/stars/moon/cloud shader | `src/sky.js:createEnvironment @L243` | 整个文件 498 行 |
| Agent instanced meshes + interpolation | `src/agents.js:createAgentRenderer @L136` | 整个文件 232 行 |
| Landmark geometry (cabin/pier/mill/playground/lighthouse/boats) | `src/landmarks.js:createLandmarks @L346` | 整个文件 454 行 |
| Shade compute（从 treeMeshes 反推写回 sim）| `refreshShade @L1128` in main.js | 23 行 |
| Grass visual update（接收 sim.grass/grassP） | `src/terrain.js:updateGrassVisual @L388` | 45 行 |

### 5.2 新增模块清单

```
living/src/renderer/
  init.ts       ← 合并 setupRenderer + setupPost（当前分散 main.js 前半部）
  camera.ts     ← orbit/iso/walk 相机切换 + updateWalkCamera/moveCamera
  terrain.ts    ← 现状 src/terrain.js 不动（保持 export createTerrain）
  sky.ts        ← 现状 src/sky.js 不动
  agents.ts     ← 现状 src/agents.js 不动
  landmarks.ts  ← 现状 src/landmarks.js 不动
  grass.ts      ← 从 terrain.ts 抽出 updateGrassVisual 成为独立 function（接受 sim.grass/grassP/trample 参数）
  shade.ts      ← 从 main.js refreshShade 抽出的 pure 函数（接收 treeMeshes，返回 Float32Array）
  boot.ts       ← animate loop + ecology accumulator + guard + render call（原 animate @L1882）
```

### 5.3 关键接口变更

**移除生态↔渲染直接耦合**：

现在 main.js 写 `ecology.playerX = player.x; ecology.playerZ = player.z; ecology.torch = null; @L1894-1896`——这违反单向契约。改成 **poller 接口**：

```ts
// renderer 向外暴露一个只读的 probe：
interface RenderProbe {
  getPlayerPos(): { x: number; z: number } | null;
  getFirePositions(): Array<{x:number,z:number}>;
  getTorchPos(): {x:number;z:number} | null;
}
// ecology 构造时接受 probe，内部用闭包读取；渲染层实现 probe
ecology = new Ecology(seed, { probe: renderProbe });
```

这样 ecology 永远不知道 THREE，也不再接收外部写它的字段。`this.playerX !== undefined` 守卫（`src/ecology.js:891` H 警觉、`:1234` P 回避）改为 `if (probe) { const p = probe.getPlayerPos(); ... }`。

### 5.4 树木阴影职责

`refreshShade @L1128` 现在由 `shade.ts` 封装为一个函数 `computeShadeFromTrees(treeMeshes, GRID_N, CELL)` 返回 shade grid。动画循环里每次 grid refresh 调用此函数并把结果交给 `ecology.setShade(shade)`。树木生长逻辑（`terrain.updateTrees`）留在 terrain 模块，不与 sim 合并。

### 5.5 render loop 如何接受来自 worker 或同步 tick 的结果

当前 `animate @L1882` 是单线程 RAF + 帧内 accumulation。新架构保持相同形态，仅将 accumulation/guard 细节收入 `boot.ts`。若将来真的上 worker：

- worker 导出 `tick(stateSnapshot, dt, daylight) → newStateSnapshot` 纯函数
- 主线程每帧 `postMessage(snapshot)` → 收回 `postMessage(newSnapshot)` → `agents.update(alpha)`
- 这种方案不在这版文档推进（§9 否决），但接口留好口子即可（Ecology 构造函数可接受可选 `workerURL` 参数——暂不支持）

---

## 6. UI 层

### 6.1 职责

UI 层只做三件事：**① 事件派发**（pointer/keyboard/slider）；**② DOM/HUD 读写**（chart canvas、stats div、inspect panel、tool buttons）；**③ 把模拟数据翻译成图表**（canvas 2d draw）。**绝对禁止 UI 直写 sim 状态**。

### 6.2 拆分后的目录

```
living/src/ui/
  dashboard.ts    ← updateStats/@L875 + drawChart/@L766 + drawEvoChart/@L827 + drawArmsRace/@L1001 + drawGeneDist/@L1086 + drawFitnessLandscape/@L1046
  panels.ts       ← updateClock/@L753 + updateEcoHud/@L1488 + updateEnvPanels/@L1503 + updateInvUI/@L1640 + toggleReplay/@L1551 + drawReplay/@L1568
  inspect.ts      ← inspectAt/@L305 + computeEcoDetail/@L335 + updateInspectPanel/@L365
  tools.ts        ← tryPlace/@L530 + ringColorForTool/@L393 + hideRing/@L402 + toast/@L497
  input.ts        ← setupPointer/@L436 + setupUI/@L596 + keyboard handlers
  walk-mode.ts    ← setMode/@L1188 + updatePlayer/@L1271 + nearestTarget/@L1335 + updateHunger/@L1380 + updateThreat/@L1389 + respawn/@L1422 + walkInteract/@L1430 + updatePrompt/@L1447 + updateFarmRegrow/@L1608 + updateMiles/@L1620 + makeCampfire/@L1656 + makeFence/@L1677 + makeBridge/@L1695 + makeHouse/@L1742 + makeFarm/@L1817 + placeBuild/@L1829 (stub) + cycleBuild/@L1830 (stub) + updateBuildBtn/@L1832
```

### 6.3 移除/隔离的游戏化残骸

按用户「不破坏现有确定性 + headless node 可用性质」和「no 游戏化」两点硬性要求：

| 位置 | 内容 | 处理方式 |
|------|------|---------|
| `updateHunger(@L1380)` + `updateThreat(@L1389)` + `respawn(@L1422)` | 饥饿消耗 + 夜间威胁 + 重生（`updateThreat` 的伤害分支已是空操作，`src/main.js:1408`） | **移除**，连同 `invbar` HTML div（`index.html:159`）/ `updateInvUI(@L1640)` |
| `makeCampfire/@L1656` + `structures` 相关 walk 道具 | 生存庇护所 | **移除**（除非 ⑩ 分解者土壤肥力 后续作为 sim 侧 landform 出现） |
| `placeBuild/@L1829`、`cycleBuild/@L1830`、`updateBuildBtn/@L1832` | 建筑菜单空桩 | **整体删除** |
| `nearestTarget` 里的 `storage/bed` 分支 | @`src/main.js:1362-1375` | 随 walk-mode.ts 一起删（makeHouse @L1742 内部的储物箱/床 mesh 同步删除） |
| `updateWeather/@L1479` 中的 `Math.random() @L1482-1483` | 天气随机源 | 改为 seeded RNG，或用模拟时间的伪噪声（例如 `mulberry32(Math.floor(time / 60))`）。这是 P3 的直接修复。 |

**理由**：游戏化功能不仅违反「not games」原则，还贡献了额外的非确定性（`state.weather` 变化影响 `ecology.rain` @L1484）。清理掉之后 sim 完全自洽。

### 6.4 stats() 缓存化解法

给 Ecology 加 `private statsCache: EcosystemStats | null; private cacheTick: number;` —— 当 `stats()` 在同一 substep 被多次调用时直接返回 cache；只在 `lastSample` 推进或显式刷新时重新计算。`updateStats/@L875` 调用频率 ~2.6 Hz 降为 ~0.5 Hz（只在 `dashboard.ts` 主动刷时）。

---

## 7. Persistence 层（schema 定义，暂不实现 IndexedDB）

### 7.1 协议格式

```json
{
  "$schema": "aether-state-v1",
  "version": 1,
  "engine": "sim-core v0.1",
  "metadata": {
    "seed": 20261102,
    "createdISO": "2026-11-08T20:01:00Z",
    "playbackDurationSeconds": 600
  },
  "sim": {
    "time": 600.0,
    "tickCount": 6000,
    "season": 1,
    "daylight": 0.78,
    "agentCount": 537,
    "maxId": 538,
    "births": 4832,
    "deaths": 1205,
    "kills": 1205,
    "isolationGroups": { /* sparse */ },
    "speciationEvent": null
  },
  "agents": [
    {
      "id": 1,
      "kind": "herb",
      "x": 281.3, "z": 249.8, "y": 14.2,
      "heading": 0.62,
      "energy": 112.4,
      "age": 87.3,
      "generation": 12,
      "genome": {
        "speed": 5.82, "sense": 42, "size": 1.04,
        "metabolism": 0.91, "lifespan": 388,
        "hue": 0.73, "bold": 0.34, "curious": 0.67, "social": 0.82,
        "helpThr": 0.12, "prefSig": 1.08, "tolSig": 0.18
      },
      "lineage": "缓独"
    }
    // ...
  ],
  "grassField": {
    "n": 76,
    "span": 1080,
    "total": [0.42, 0.68, ...],
    "bySpecies": [[0.21, ...], [0.11, ...], [0.10, ...]]
  },
  "evolution": {
    "herb": [{ "t": ..., "gen": ..., "count": ..., "speed": ..., "sense": ..., ... }],
    "pred": [...],
    "scav": [...],
    "omn": [...]
  },
  "historyTail": [/* 最后 240 points of Ecology.history */],
  "isolationHistory": [...],
  "depositStats": { "count": N, "meanDistCells": X, "p50": Y, ... }
}
```

### 7.2 序列化策略对比

| 格式 | 优点 | 缺点 | 适用阶段 |
|------|------|------|---------|
| **JSON Text** | 人类可读、debug 方便、IndexedDB 原生支持、跨平台分享 | 体积大（5 MB+ 典型会话），解析慢 | 初期存档加载（#12）首选 |
| Binary TypedArray | 快、体积小（~500 KB）、确定性字节序 | 不可读、需手工编解码、跨引擎不通用 | ⑧ 生物群区大规模 sim 阶段考虑 |
| **Protocol Buffers** | 强 schema、紧凑、向前兼容、跨语言 | 需 .proto 定义、运行时解码库（~50 KB gzipped）、工具链重 | 长期方案（若未来多端） |

**本轮决策**：保留 JSON 为默认（简单可验证），但 schema 锁定在 `living/src/persist/schema.json`；后续迁移至 protobuf 需要时通过 wrapper 透明替换。

### 7.3 存档触发时机（规划）

- 自动保存：每 120 秒保存一次 `historyTail + agents snapshot`（不含完整 grassP，重建成本太高；下次启动重新 initGrass + seek through ticks 回溯到快照时刻——代价巨大，所以短期不建议做时间旅行）
- 手动下载：HUD 按钮「导出世界」生成 `.json` 文件，让用户存盘
- 加载：`?seed=...&load=filename` query param 或拖拽本地 json

---

## 8. Tooling 层

### 8.1 构建管线演进路线

**目标**：同时满足「多入口 ESM / Worker 隔离 / HMR 开发体验 / 可选单文件离线产物」。这四项 esbuild 单独每项都能做到但都要手工拼——**唯一覆盖全集的是 Vite 8（Rolldown 内核）** [scout 报告第 1 节]。

```
living/package.json          ← 补登 three ^0.170.0 依赖（现状 npm install 过但未登记，重装会丢）
living/tsconfig.json         ← allowJs + checkJs 起步（渐进 TS，见 §10.2）
living/vite.config.ts        ← 入口配置 + worker 内联插件 + singlefile 插件
living/build.ts              ← 可选：基于 Vite bundle-only 的 CI 管道（含 source map）
living/build.py              ← 保留作兜底（ESM → esbuild IIFE，~毫秒级）— 若 Rolldown 在 PRoot arm64 异常，回退成本为零
living/.vscode/settings.json ← biome type-check 集成
```

**关于单文件离线的关键实现点**：`vite-plugin-singlefile@2.3.4`（2026-10 发布，周下载 124 万）会把 JS/CSS 全部内联回 `dist/index.html`，形态等价于现 `build.py` 产物。注意限制：**不内联 `public/` 静态资源/SVG**；CSP meta 会拦内联脚本（构建期警告需配 CSP 豁免）。Worker 用 Vite 内置 `import MainWorker from './sim?worker&inline'` 产出 base64 内联 worker，**绕开 `file://` 下的同源策略失败** [scout 额外发现 §2]。

**不选的理由**：
- Rolldown 单独使用不合适 —— 它是为 Vite 设计的 engine，单跑没有配套生态 [scout]
- esbuild 纯单入口方案保留作 fallback（当前 build.py 即它），不强制替换
- 暂不加 WebGPU/TSL 重写 —— WebGLRenderer 官方仍在维护，Android WebView 兼容性仍是硬约束

**TODO-立刻**：第一步工程化动作是把 `three` 登记进 `package.json`（当前缺这条），否则 `npm install --save` 以外的任何重建都会丢失渲染层。

### 8.2 linter/formatter 推荐

首选组合：`biome`（原名 Rome fork）—— 一行命令集成格式化 + lint + type-check（TS enabled）+ fix。比 ESLint + Prettier 单 binary 轻量，迁移成本小。备选 `eslint-plugin-import` + `prettier`（更成熟生态）。

### 8.3 测试 runner

| Runner | 与当前 mjs 脚本兼容性 | TypeScript 支持 | Watch mode | 并行 | 备注 |
|--------|-------------------|----------------|-----------|------|-----|
| Vitest | ✅ import `{ Ecology }` | ✅ 原生 | ✅ | ✅ worker | 推荐 |
| jest | ⚠️ 需 babel/jest config | ⚠️ 需 ts-jest | ✅ | ✅ | 太重 |
| node:test（Node v22 内置）| ✅ 可直接跑 | ✅ via `--experimental-transform-types` 或用 TS runner shim | ❌（需 wrap） | ✅ | 轻量、无依赖 |
| 手写（现状） | — | ❌ | ❌ | ❌ | 不可持续 |

**本轮决策**：**Vitest**（默认选用）或 `node:test`（若想零依赖）。后者在当前 Node 22 环境下开箱即用，适合早期；Vit 后期换不痛。

### 8.4 CI 建议（可选）

`.github/workflows/test.yml`：push 时跑 `npm run test -- --run`，覆盖生态 headless 测试 + 单元测试。PR 合并前必须绿。

---

## 9. Worker 隔离策略

### 9.1 结论：**本轮仍不建议引入 Worker**，但原因与原文不同

#### 硬约束（而非工程权衡）

Scout 调研发现了两条平台封锁，彻底关闭了更优雅的 SharedArrayBuffer + Atomics 方案：
- **Android WebView 明确不支持 COOP/COEP**（Chromium issue 40914606 已 closed as intended），根因是 WebView 无进程隔离；Chrome 团队对新的 Document-Isolation-Policy 也表态「可预见的未来不会进 Android WebView」。
- **`file://` 协议下无论如何都无法建立 crossOriginIsolated**（无 HTTP 头可发）。
- 结果：Sim 线程若想与主线程共享可变状态（SAB），两条路径都被双重封死。**MessageChannel + 结构化克隆是唯一可行方案** [scout 第 3 节]。

#### 工程权衡（保留原结论）

即便平台允许，以下原因使「本轮不进 worker」合理：
1. **计算量**：Node v22 实测单次 tick ~0.3 ms（1000 ticks = 1 分钟 sim ≈ 0.3 s CPU）；speed=4 满 agent 650 时帧预算仍远低于 16 ms。跨线程通信成本 > 计算节省。
2. **数据结构不友好 transfer**：agent.cargo 与 trailX/Y/Z 每条都持多个 typed array，共 650 条 → 内存压力已不小；每次 tick transfer 全量 agents（>20 MB）不现实，增量 diff 又破坏确定性 [scout 额外发现 §1]。
3. **O(N²) 扫描**：_nearest/@L718、_tryHelp/@L514 遍历 agents 找最近个体；speciation detection 在 _sample/@L1459 再次排序全体 alive。多线程需串行化或额外锁，复杂度陡增。
4. **确定性风险**：Worker 消息顺序理论上 FIFO 但在部分浏览器/WebView 存在抖动；保序需显式序列号 ack，反而增加失效面。同 seed 同结果的硬约束被打破的风险不可控。
5. **file:// 分发 + Worker 的内联矛盾**：单文件离线模式下无法用相对 URL 加载 Worker（Chrome 同源策略会在 file:// 下拒绝），必须借助 Vite 的 `?worker&inline` base64 内联——这是「保留离线分发」与「Worker 隔离」交汇处的翻车高发点，本轮不值得冒这个险 [scout 额外发现 §2]。

### 9.2 何时应该回头引入 Worker

当后续波次⑧「空间生物群区」上线后，世界划分为多个独立区块（每个区块 ~500 agent），区块间几乎无交叉——这时每个 block 一个 WebWorker 跑自己的 sim tick、主线程汇总全局 stats/history 才真正划算。
届时只做两件事：把 `Ecology.update(simState, dt, daylight) → nextState` 包成顶层纯函数（当前已接近），再把区块分发逻辑插入 boot loop 即可。接口预留完好，无需重来。

### 9.3 当前可做的轻量优化（不跨线程）

- sim 内部 agent 列表改用**原地交换删除**代替每次 `filter` 重建（当前 `src/ecology.js:1433-1435` 每 substep 都在 sweep）—— 预计 20% 提升
- 网格加速近邻搜索：_nearest @L718 是全表扫；替换为 spatial hash（CELL=14.2 天然分格）—— 下一波性能优化的合理点，与架构解耦
- 若将来真要 worker，`Float32Array.buffer` 走 `transfer` 可实现近零拷贝传输（比结构化克隆快一个数量级）

---

## 10. 技术选型总结表

### 10.1 维度：构建工具

**Scout 报告结论**（证据见 `research/tech-scout-2026-11.md` 第 1 节）：
- **Vite 8（Rolldown 内核）** 已于 2026-03-12 发布稳定版，统一取代 esbuild（dev 预构建）+ Rollup（生产打包）；官方 19,000 模块基准生产构建 40.1s → 1.61s
- **esbuild** 仍活跃（基准最新 0.28.2），但角色退化为「独立快速打包器」
- **vite-plugin-singlefile@2.3.4**（2026-10，周下载 124 万）支持 Vite 5–8，保持单文件离线产物形态
- **Vite 原生 `?worker&inline`** 可产 base64 内联 worker，绕开 `file://` 下相对 URL Worker 加载失败的同源问题

| 候选 | 评估维度 | 结论 |
|------|---------|------|
| **Vite 8** | 多入口/code split 一等公民；worker 打包一行配置；HMR 成熟；singlefile 插件保单文件形态；Rolldown arm64 napi 有官方预编译（PRoot 未实测，保留风险） | **选定，主构建通道** |
| esbuild（现用） | 毫秒级；配置极简；IIFE 输出匹配现有单文件交付；但 worker/多入口/HMR 需手工拼 | **保留作兜底**（build.py 即它） |
| Rolldown 单独使用 | 为 Vite 设计的 engine，单跑无生态配套 | 不选 |
| webpack | 配置复杂度高；bundle 优化收益有限（THREE 占大头） | 不选 |

**当前决议**：Phase 1c 引入 Vite 8 + vite-plugin-singlefile；`build.py` 保留作 fallback；首步先把 `three ^0.170.0` 补进 package.json。

### 10.2 维度：TypeScript 迁移

**Scout 报告结论**（证据见 research 报告第 2 节）：
- 共识路径：`allowJs + checkJs` + JSDoc 打底 → 从依赖图叶子逐文件改后缀 → strict 标志逐个拧（noImplicitAny → strictNullChecks → strict）
- **ecology.js 是第一个转 `.ts` 的最佳目标**：已有 10 个无头测试作现成回归护栏
- main.js（UI+渲染接线）可长期留在 JSDoc 状态不阻塞
- 注意：TypeScript 7（Go 重写版 tsgo）已发布，对 checked JS 的 JSDoc 解析有 rework——迁移时锁版本逐版本升级

| 路线 | 评估 | 结论 |
|------|------|------|
| **渐进式 TS（.js → .ts 改名）** | 最小侵入；JSDoc 标注类型声明；tsc --noEmit 检查；可保持 ES Module syntax 不动 | 推荐起步 |
| 全文转 TS + strict | 重构量大，风险高；break 现有测试 | 不推荐首轮 |
| 跳过 TS，用 JSDoc + biome check | 纯 JS 风格但获得类型提示与 lint | 备选（若团队不熟悉 TS） |
| TypeScript with declaration emit | 标准做法；配合 `tsconfig.json` `moduleResolution: node16` | 最终落地形态 |

**当前决议**：Phase 1 用 JSDoc 注解 + tsc --noEmit 逐步覆盖；Phase 2 切 TS 源。详细里程碑待 Chief 审批。

### 10.3 维度：测试框架

**Scout 报告结论**（证据见 research 报告第 6 节）：
- **Vitest 5.1**（2026-11-02）要求 Node ≥ 22.12（本机 v22.22.1，满足）
- 关键特性：Vitest 4.1 起提供 `experimental.viteModuleRunner: false` —— **禁用 Vite 沙箱、用原生 Node import 跑测试**，正好匹配「sim 核心必须在真实 node 语义下确定性复现」的约束（Vite 沙箱 transform 行为与原生 node 有差异，会影响确定性）
- **node:test**（Node 24 起 TS 原生剥离、零依赖）是体面备选，但 watch/并行能力弱于 Vitest
- 性能基准：无开箱即用的「tick 耗时统计」工具，建议自写 `node:perf_hooks` P50/P99 脚本，不引依赖

| 候选 | 评估 | 结论 |
|------|------|------|
| **Vitest 5 + `viteModuleRunner: false`** | ESM 原生；保留真实 node 语义；watch/snapshot/并行；与 Vite 8 同源 | **选定** |
| node:test | 零依赖；与现有 `process.exit(0/1)` 风格相近；但无 watch | 备选（过渡） |
| Jest | 重型；需 babel transformer | 不推荐 |

**当前决议**：Phase 1c 引入 Vitest 5，`viteModuleRunner: false` 配置项必须显式写入 vitest.config.ts；现有 10 个 test_*.mjs 加一层薄适配（import 改为 vitest）后纳入。

### 10.4 维度：持久化封装

**Scout 报告结论**（证据见 research 报告第 7 节）：
- **idb**（Jake Archibald）仍是 2026 年标准推荐；纯 promise 封装；不改变 IDB 语义
- **idb-keyval** 适合「单 key 整存」场景（本项目的单文件快照恰是这种形态）
- **localforage** 注意其 localStorage 回退路径会退化为 JSON 序列化（Android WebView WebSQL 已废弃，可能被迫走退化路径）——**不推荐**
- **IndexedDB 的 value 本身走结构化克隆**：`Float32Array`/`TypedArray`/`Map`/`Set`/`Date` 都能原生无损存取，**不需要先 JSON.stringify**（stringify 反而是有损且更慢的降级）

| 候选 | 评估 | 结论 |
|------|------|------|
| **idb / idb-keyval** | 轻量；纯 promise；结构化克隆友好 | **选定** |
| 原生 IDB API | 零依赖；但 callback 风格易错 | 短期可用 |
| Dexie.js | 链式查询优雅；~30 KB dep | 对本项目单快照场景过重，不选 |
| LocalForage | 兼容旧浏览器 | 不选（Android WebView 有退化风险） |

**当前决议**：Phase 3 存档接入时使用 `idb-keyval`（单 key 整存 + 原生 TypedArray 支持刚好匹配草场 Float32Array 需求）；`grassField.bySpecies` 等 TypedArray 走原生结构化克隆，不要 stringify。

### 10.5 综合评分卡

| 维度 | 选定方案 | 备注 |
|------|---------|------|
| 构建 | **Vite 8（主）** + esbuild build.py（兜底） | 首步补登 three 依赖 |
| TS 迁移 | 渐进式（JSDoc + checkJs → .ts 改名） | 注意 TS7（tsgo）对 JSDoc 解析的 rework，锁版本逐版本升 |
| 测试 | **Vitest 5（`viteModuleRunner: false`）** | 保留原生 node 语义，护住确定性 |
| DB | **idb-keyval**（原生结构化克隆 TypedArray） | 不要 stringify |
| Lint/Format | Biome | 单 binary 集成 lint+format+type-check；生态成熟度待实测后定案（scout 未深查） |
| ECS | **不引入** | 几百实体规模下 SoA 收益不显著；换 ECS = 重写仿真心 + 重验证确定性。当 agent > 数千且单 tick 的 agent 迭代耗时占比 > 20% 时再评估 bitECS 0.4（SoA + 自带序列化与 worker transfer 天然契合） [scout 第 5 节] |
| Three.js | 留在 WebGLRenderer + r170 | WebGPURenderer r171 起生产可用但有 fallback；但自定义 GLSL shader（天空/云/水）迁移到 TSL 成本高；Android WebView 的 WebGPU 支持状态未核实。远期写实渲染阶段再评估，非本轮事项 [scout 第 4 节] |
| Worker | 本轮不引入；MessageChannel + Transferable 预留 | SAB 双死路（WebView + file://），OffscreenCanvas WebGL 路径 WebView 长期不可靠 [scout 第 3 节] |

---

## 11. 风险与未知

| # | 风险 | 缓解措施 |
|---|------|---------|
| R1 | **确定性强破坏**：重构过程中误动 rng 调用顺序，导致 `test_isolation.mjs` 等多 seed 回归失败 | 每一步提交前先跑全套 seed diff；`ecology.js` 所有 `this.rng()` / `this.prng()` 调用顺序不得变；拆分只移动方法体、不改变调用链 |
| R2 | **双时钟混乱**：`gameHours` 与 `ecology.time` 在动画循环中被分别修改，拆分 UI 层后容易两边各自推进 | 统一 `GameClock` 单例：所有模块只能访问 `clock.getTime()` / `clock.setGameHours(h)`；生态 daylight 由 clock 查表给出 |
| R3 | **walk 模式生存要素意外保留**：若遗漏某个 health/hunger 引用，播放器 UI 会出现空白或断言崩溃 | 静态分析 grep `inv-hunger|inv-life|state\.hunger|state\.life` 清一遍；加 CI lint rule 禁止 import 除 core/renderer/ui 外的符号 |
| R4 | **speciation 时序偏移**：`speciationFired < 3` 计数在拆分后可能被重置（如 `constructor` 漏调 `_sample`）| 确保 `Ecology.constructor` 末尾仍然顺序执行 `_initGrass → _seedLife → _sample`，保持 seed 不变 |
| R5 | **tree seed 非确定性**：`addBush @src/terrain.js:841` 用 Math.random()；移除 walk-build 模块后会消失，但将来恢复时需注意 | 将 `addBush` 改为使用传入 rng 或移除（植物生长由 ecology 自行管理） |
| R6 | ~~[待 scout 报告] 构建工具/TypeScript/测试框架具体版本锁定~~ **已合并**（见 §10） | 剩余未核实项：Rolldown napi 在 PRoot arm64 容器实测；Android WebView 的 WebGPU 支持状态；Biome 生态成熟度。均不阻塞 Phase 1a/1b，Phase 1c 首日可实测 Rolldown |
| R7 | **[待 Chief 确认]** 是否保留 `window.__aether` 全量 expose（调试价值 vs 攻击面） | 评估：保留但改名 `window.__ae_debugApi`，并在 prod 模式下有条件剔除 |
| R8 | **TS7（tsgo）JSDoc 解析 rework** [scout 第 2 节]：部分旧 JSDoc 写法不再按老方式解析，Phase 1c 若盲目升级 TypeScript 大版本可能引发类型误报 | 锁定 TS 小版本（如 5.x 最新），逐版本升级；每次升级重跑全套 test_*.mjs 验证 |
| R9 | **确定性 × 未来 Worker 的隐藏约束** [scout 额外发现 §1]：postMessage 结构化克隆本身是确定性的，但若未来把 `performance.now()` 或随机数混进 tick，快照协议本身会破坏可复现性 | 当前架构已约定 tick 由消息驱动而非时间驱动；Sim Core 内部禁止直接调 `Date.now()/performance.now()/Math.random()`（lint 规则强制） |

---

## 12. 附录

### A. 术语表

| 术语 | 含义 |
|------|------|
| **Sim Core** | 纯计算生态仿真内核，不依赖任何图形/DOM 库 |
| **Renderer** | Three.js 驱动的 3D 可视化层（地理/草木/动物/天际线/水面/云） |
| **UI 层** | DOM chart/HUD/inspect/tool 控制逻辑，把 sim 数据翻译成屏幕像素 |
| **Pulse** | 放置实体时的视觉脉冲特效（环状 RingGeometry，叠加 AdditiveBlending） |
| **Lineage** | 谱系标记（'迅聚'/'缓独'），在 speciation event 触发时给 agent 打 tag |
| **IsolatedGroup** | 隔离群的登记名（中性代号 '隔离群-N'），由 `_pairCompatible` 阻断触发 |
| **Magic Trait** | `prefSig`/`tolSig` 构成的择偶性状，驱动 speciation barrier |
| **Seed Life** | `Ecology._seedLife`：初始批量 spawn 420 H / 12 P / 8 S / 12 Omn |
| **Water-filling** | `_takeGrass` 的采食语义：按适口性加权多草种分配，存量不足则回流剩余 |
| **Image Scoring** | 间接互惠声誉系统：image ∈ [-5,+5]，帮助+1 / 拒绝-1，不遗传 |
| **GUT_DECAY** | 肠道种子滞留指数衰减率，约 0.0085 s⁻¹，期望滞留时间 ~118s |
| **Growth Season Meta** | `seasonMeta ∈ [0.75, 1.15]` 四季代谢乘子，冬季最低 0.75 |

### B. 主要文件改动预估

| 阶段 | 动作 | 预期影响行数 |
|------|------|-------------|
| Phase 1 | 新增 `src/core/` 10 个文件，把 ecology.js 方法逐块搬运进对应 ts，`Ecology` 类体重塑为薄聚合 | +2200 / -1739（净增 ~460 行） |
| Phase 1 | 抽取 `src/renderer/init.ts`、`camera.ts`、`shade.ts`、`boot.ts`，其余 terrain/sky/agents/landmarks 暂时不动 | +400 / -350 |
| Phase 1 | 拆分 `src/ui/dashboard.ts`、`panels.ts`、`tools.ts`、`input.ts`、`walk-mode.ts`；删除饥饿/生命/空桩 build 函数 | +1800 / -1700 |
| Phase 2 | 新增 `tsconfig.json`、`vitest.config.ts`、`biome.json`、`build.ts`；升级 package.json scripts | +200 |
| Phase 3 | 持久化 `src/persist/` schema 定义 + JSON serializer/deserializer（仅草稿，暂不连 IndexedDB） | +250 |
| 总计 | 新增 ~5850 行，删除 ~3439 行，净增 ~2400 行（主要由 UI/core 拆分带来） | 最终项目约 9K LOC，分布均匀 |

### B'. 迁移三步走（每步可独立发布，不破坏在线版本）

**Phase 1a（Week 1）—— 核心拆分，外观不动**
- 新建 `living/src/core/**`，把 ecology.js 各子域逐文件迁入；保留原 `src/ecology.js` 作为 barrel（`export * from './core/ecology.js'`），tests 与 build 入口零改动
- 同时新建 `src/ui/**` barrel；`main.js` 只改 import 语句
- 每次提交前跑全套 seed diff（`test_ecology.mjs` / `test_isolation.mjs` 等 10 个脚本）确认零回归
- 发版：同一 URL 同一 `aether.html`，用户无感知

**Phase 1b（Week 2）—— 渲染/UI 清理 + 游戏化残骸移除**
- renderer setup/post/camera 从 main.js 抽到 `src/renderer/{init,camera,boot}.ts`；UI 抽到 `src/ui/{dashboard,panels,inspect,tools,input,walk-mode}.ts`
- 删除 `updateHunger/updateThreat/respawn/placeBuild/cycleBuild` 与 `updateWeather` 的 `Math.random`；`invbar` DOM 整段移除
- 引入 `RenderProbe` + `ecology.setProbe()/setShade()/setRain()`，切断双向字段写入
- 发版：URL 不变，产物因死代码删除小幅变小

**Phase 1c（Week 3）—— TS + tooling + persist 打底**
- 引入 **Vite 8** + `vite.config.ts`（`vite-plugin-singlefile` 保单文件产物，`?worker&inline` 预留）+ `tsconfig.json`（allowJs/checkJs 起步）+ Vitest 5（`viteModuleRunner: false`）+ Biome
- `package.json` 补登 `three ^0.170.0`；scripts 升级（dev/build/test/lint/format）
- Phase 1a/b 产出的文件逐步标注 JSDoc → 转 TS（ecology 优先，main.js 长期 JSDoc）
- 新增 `src/persist/schema.json` + JSON serializer（TypedArray 走原生结构化克隆，不 stringify）
- `build.py` 保留作 fallback（Rolldown 在 PRoot arm64 未实测，异常时零成本回退）
- 发版：同 URL 同产物结构，开启 source map

**Phase 2+**（⑧ 生物群区 / ⑫ 存档加载触发时）：IndexedDB 接入、必要时 worker pool 边界化。

### C. 参考链接

- 项目现状文档：[`living/src/research-note.md`](living/src/research-note.md)（第四波合作机制研究，与本架构无关但保留参考）
- 三次迭代测试样例：`living/test_ecology.mjs` / `test_isolation.mjs` / `test_cooperation.mjs`
- 产出 HTML 基座：`living/index.html`（199 行），包含所有 HUD 元素
- 成品尺寸：`living/aether.html` 当前 786 KB（minified IIFE bundle inline）

---

*v0.2 收尾：scout 报告已全部合并进 §8.1 / §9 / §10 / §11；剩余待 Chief 决策项仅 R7（调试 expose 是否剔除）与 §7.3 的存档触发时机细节。下一步：用户审阅本架构，确认后按 Phase 1a（core 拆分）启动实施。*
