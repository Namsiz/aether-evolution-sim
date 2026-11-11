# 「以太 · 自生世界」架构重设计选型调研报告

> scout ｜ 2026-11-09 ｜ 基于 web 检索，检索日期即数据日期；查不到的项已明确标注「未查到/未核实」。
> 本文件由 Chief 从 scout 会话保存，作为 ARCHITECTURE.md 技术选型章节的证据底稿。

## 1. 构建工具：esbuild vs Vite vs Rolldown

**现状背景（关键事实）**：Vite 8 已于 2026-03-12 发布稳定版，用 Rust 写的 Rolldown 统一取代了 esbuild（dev 预构建）+ Rollup（生产打包），不再需要 opt-in 的 `rolldown-vite`。官方 19,000 模块基准：生产构建从 Rollup 的 40.1s 降到 1.61s；中型应用（Vite 7 → 8）实报约 11.4s → 6s 量级，小项目提升没那么夸张。esbuild 本身仍活跃（基准里最新 0.28.2），但角色退化为「独立快速打包器」。

| | esbuild（现用） | Vite 8（Rolldown） | Rolldown 单用 |
|---|---|---|---|
| 多入口 ESM / code splitting | 手工配置，弱 | 一等公民（且支持手动 chunk 控制） | 同 Vite |
| Web Worker 打包 | `--worker` 可用但简陋 | `import Worker from './x?worker'`，支持 `?worker&inline` 产 base64 内联 worker | 支持 |
| HMR | 无 | 有，成熟 | WIP（还在开发中） |
| 单文件产物 | 现有 build.py 手工内联即可 | `vite-plugin-singlefile` 2.3.4（2026-10-17 发布，支持 Vite 5–8，周下载 124 万，维护中） | 需自配 |

**针对本项目的关键点**：
- **单文件离线分发不会丢**：vite-plugin-singlefile 把 JS/CSS 全部内联回 dist/index.html，与现有 build.py 产物形态等价。注意其限制：**不内联 public/ 静态资源和 SVG**；CSP meta 会拦内联脚本（会构建期警告）。worker 用 `?worker&inline` 可内联为 base64。
- Rolldown 的 HMR 还是 WIP，但 Vite 的 HMR 是 Vite 自己的 dev server 能力，已成熟多年，不依赖 Rolldown 的 HMR。
- 在 PRoot 容器里跑：Rolldown/Oxc 是原生二进制（napi），arm64 Linux 有官方预编译，风险低但**未在本容器实测**。

**推荐**：**迁移到 Vite 8**。理由：恰好需要「多入口 + worker 打包 + HMR + 可选单文件产物」四件事的全集，esbuild 单独都做得到但每件都要手工拼；Vite 8 是 2026 年事实标准，生态全兼容（Rollup 插件 API）。esbuild 作为兜底方案完全可行——若容器里原生二进制出问题，回退 esbuild 的成本近乎为零（现有 build.py 就是它）。不建议 Rolldown 单独使用，它就是为 Vite 设计的。

**来源**：
- https://vite.dev/guide/migration （Vite 7→8，Rolldown 集成）
- http://rolldown.rs/guide/introduction
- https://newstack.dev/devops/build-tools （2026-08，Vite 8 生态综述）
- https://github.com/rstackjs/build-tools-performance （2026-11 基准数据）
- https://www.npmjs.com/package/vite-plugin-singlefile

## 2. TypeScript 迁移路径

**候选对比**（共识度极高，多份 2025–2026 指南结论一致）：

| 方案 | 对「现有 JS 不重写」友好度 | 说明 |
|---|---|---|
| JSDoc + checkJs | 最高，零改动起步 | `allowJs` + `checkJs` 后，`.js` 里的 JSDoc（`@param/@typedef/@type`）被当真实类型检查；可按文件用 `// @ts-check` 局部开启、`// @ts-nocheck` 局部豁免 |
| 全量 .ts 迁移 | 最差 | 大爆炸重命名 → 数百上千错误，公认反模式 |
| 混合（推荐路径本身） | — | 事实标准做法：先 allowJs/checkJs + JSDoc 打底 → 从依赖图叶子（工具函数）开始逐文件改后缀 → strict 标志逐个拧（noImplicitAny → strictNullChecks → strict） |

**针对本项目**：6400 行、ecology.js 是仿真心且已有 10 个无头测试兜底——非常适合作第一个改 `.ts` 的目标（测试是现成的回归护栏）。main.js（UI+渲染接线）可长期留在 JSDoc 状态，不阻塞。

**推荐**：**JSDoc + checkJs 起步的混合渐进方案**。另注意：TypeScript 7（Go 重写版 tsgo）已发布，对 checked JS 的 JSDoc 解析有 rework，个别旧 JSDoc 写法不再按老方式解析——迁移时锁版本、逐版本升级。

**来源**：
- https://codezup.com/migrating-from-javascript-to-typescript-guide （含 TS7/JSDoc 变更提示）
- https://www.irurueta.com/web/typescript/javascript-interop.html （checkJs/JSDoc 机制细节）
- https://devfieldguide.com/tutorials/migrate-javascript-to-typescript-incrementally

## 3. Sim 核心进 Web Worker

**三条路线的现状（本次调研里最硬的约束发现）**：

- **SharedArrayBuffer**：桌面 Chrome 和 Android Chrome 在 COOP/COEP 下可用；**Android WebView 明确不可用**——Chromium 官方 issue 40914606 已被关闭（"working as intended"），根因是 WebView 无进程隔离/站点隔离；Chrome 团队对 Document-Isolation-Policy（新的 SAB 解锁途径，桌面 M137 起）明确表态「可预见的未来不会进 Android WebView」。另外 **file:// 协议下根本无法建立 crossOriginIsolated**（无 HTTP 头可发）。结论：本项目（file:// 分发 + Android WebView）**SAB 路线被双重封死**。
- **MessageChannel + 结构化克隆**：无任何前置条件，WebView/file:// 全可用。性能上结构化克隆是大对象场景下最快的平台内建复制方式（V8 内建，IndexedDB/postMessage 同一算法）；`Float32Array` 等 TypedArray 属结构化克隆支持类型，快照传输是零依赖可序列化的。配合 Transferable（ArrayBuffer transfer）可达近零拷贝。
- **OffscreenCanvas**：BCD/caniwebview 数据称 Android WebView 自 69 起支持（含 `transferControlToOffscreen`），但 Chromium tracker 有多年历史 issue 记录 WebView 中 `transferControlToOffscreen`（WebGL 路径）长期坏掉（canonical issue 40617844，2024 年仍有用户报告）。**2D worker 内 OffscreenCanvas 基本可靠；WebGL offmain 路径在 WebView 上需真机验证，未查到明确的「已修复」时间点**。

**推荐协议模式（确定性 tick + 每帧渲染同步）**：
1. Sim 跑固定时间步长 tick，每个（或每 N 个）tick 产出**不可变快照**，主线程只读渲染——这是保证确定性最简单的模式（sim 线程永不读渲染状态，无共享可变状态）。
2. 传输用 MessageChannel postMessage + **transferable ArrayBuffer**（Float32Array.buffer 移交所有权），避免克隆开销；草场这类大 TypedArray 走 transfer，agent 列表这类小对象走普通结构化克隆。
3. 不建议 SAB + Atomics 自旋（平台封锁如上）；不建议 OffscreenCanvas 作为 sim 协议的一部分——它解决的是渲染主线程卸载，与 sim 无关，留作未来可选优化。

**来源**：
- https://issues.chromium.org/issues/40914606 （WebView SAB 不可用，closed as intended）
- https://issues.chromium.org/issues/404645676 （WebView COI 请求）
- https://groups.google.com/a/chromium.org/g/blink-dev/c/cFuPRXcpc84 （Document-Isolation-Policy：不做 WebView，Android M146 起）
- https://web.dev/articles/structured-clone （克隆算法与性能特征）
- https://caniwebview.com/features/mdn-offscreencanvas ＋ https://issues.chromium.org/issues/40623066 （OffscreenCanvas/WebView 争议史）

## 4. Three.js 2026 现状（只摆证据）

- **版本线**：项目在 0.170.0（即 r170，2024-10）。最新已到 **r187（2026-10）**，r188 开发中。r170→r187 之间 breaking change 见官方 Migration Guide（如 r171 起 WebGPURenderer/TSL 改从 `three/webgpu`、`three/tsl` 入口导入；WebGL1 支持早已移除）。
- **WebGPURenderer 成熟度**：官方在 r171（2025-09）将 WebGPURenderer 定为生产可用，内建 WebGL2 自动 fallback；r183 加了 node 化后处理管线 RenderPipeline；r187 期间仍有大量 WebGPURenderer/TSL 修复 PR 在合入（活跃开发中，但已是主干而非实验分支）。
- **浏览器覆盖**：WebGPU 已默认启用于 Chrome、Edge、Safari 26、Firefox 141+（Windows）。**Firefox on Linux/Android 到 2026 年中仍无稳定 WebGPU**。社区独立基准显示部分场景（大量非实例化 mesh）WebGLRenderer 反而更快。
- **迁移成本**：`ShaderMaterial`/`onBeforeCompile` 的自定义 GLSL 在 WebGPURenderer 下完全不工作，必须重写为 TSL；EffectComposer 生态部分 pass 不兼容。
- **Android WebView 上的 WebGPU 支持状态：未查到可靠结论，未核实**——这是远期写实渲染的关键前置条件，建议列为后续真机验证项。

**只摆证据的要点**：WebGLRenderer 官方仍在维护，官方手册明说它是「纯 WebGL 需求的推荐选择」；WebGPURenderer 是新项目低风险默认（有 fallback），但对已有项目只有在命中具体瓶颈时才值得投入。

**来源**：
- https://threejs.org/manual/pages/webgpurenderer.html （官方手册，含迁移警告）
- https://github.com/mrdoob/three.js/wiki/Migration-Guide （r170→r187 变更）
- https://zko.world/blog/webgpu-threejs-production-ready-quick-take ＋ https://bitsoulhosting.com/marketplace/blog/threejs-webgpurenderer-production-ready-what-breaks

## 5. ECS 框架

**候选现状**：
- **bitECS**：0.4.0 于 2025-12-06 正式发 npm（此前长期只能 GitHub 安装），TS 完全重写，~5kb minzipped，零依赖，自带 SoA 序列化模块，被 Third Room/Hubs/Phaser 4 采用。是 JS ECS 里迭代基准的标杆（noctjs 套件 packed_5 ≈ 335k ops/s，SoA 类第一梯队）。
- **miniplex**：2.x，对象数组（AoS）+ archetype 查询 + React 绑定，ergonomics 最好但迭代性能最低梯队（同一基准 ≈ 109k ops/s；ecsia 基准里 per-entity 成本 ≈ 12ns，是 bitECS 的 9 倍）。
- **koota**（pmndrs）：EliCS 2026 基准中中游（如 add/remove 47ms vs bitECS 25ms）。活跃度未深查，未核实其最新 release。

**对「几百个 agent + 字段级变异」是否值得引入——判断依据**：
- SoA vs AoS 的实际收益来自缓存局部性与向量化，Javelin 的 500 万实体基准：AoS 53ms / 混合 26ms / SoA 12ms——**实体规模上千、热循环字段访问为主时 SoA 收益才显著**。几百个 agent 时，V8 对小对象数组的 hidden-class 优化已经很好，差距在这个量级不构成瓶颈。
- 更硬的约束是：已有「对象数组 + 直接字段访问」的 ecology.js，且**测试依赖确定性复现**。换 ECS 意味着全量重写仿真心 + 重验证全部确定性——成本远大于几百实体规模下的性能收益。
- bitECS 的真正诱惑点其实是它的 **TypedArray 序列化**（快照即连续字节）和 query 模型——这两点可以借鉴其模式而不引入库。

**推荐**：**不引入 ECS 框架，保持现有结构**；当且仅当后续波次把 agent 推到数千以上、且 profiler 证明 agent 迭代是热点时，再评估 bitECS 0.4（SoA + 自带序列化，与 Worker transfer 协议天然契合）。判断门槛：单 tick 的 agent 迭代耗时占比 > 20% 才值得动。

**来源**：
- https://github.com/NateTheGreatt/bitECS/issues/183 （0.4.0 发布时间线）
- https://github.com/noctjs/ecs-benchmark ＋ https://github.com/ddmills/js-ecs-benchmarks
- https://grzegorzotto.dev/blog/archetype-ecs-typescript （SoA/AoS/Map 三存储模型实测取舍）
- https://javelin.hashnode.dev/ecs-in-js-storage-mechanisms （500 万实体 AoS/hybrid/SoA 数据）
- https://elixr-games.github.io/elics/benchmarks.html （bitECS/koota/becsy 对比）

## 6. 测试与工具链

- **Vitest**：当前 5.1（2026-11-02），要求 Node ≥ 22.12（本机 v22.22.1，满足）。对裸 node ESM 脚本迁移极轻。Vitest 4.1 起提供 `experimental.viteModuleRunner: false`——**禁用 Vite 沙箱、用原生 Node import 跑测试**，匹配「sim 核心必须在真实 node 语义下确定性复现」的约束（Vite 沙箱的 transform 行为与原生 node 有差异）。
- **node:test**：Node 24 起 TS 原生剥离、零依赖，2026 年对比基准里纯 ESM 启动最快（`--test-isolation=none` 时 0.5s）。若决定全程不引测试框架，它也是体面选项。
- **性能基准**：未查到「仿真 tick 耗时统计」的专用现成工具；实用方案是 `node:perf_hooks` + 简单自写统计（P50/P99），或用 tinybench。**没有开箱即用的答案，如实说明**。

**推荐**：引入 Vitest 5（`viteModuleRunner: false`），现有 10 个测试包一层薄适配即纳入；tick 性能统计自写 perf_hooks 脚本，不引依赖。

**来源**：
- https://vitest.dev/blog/vitest-4-1 （viteModuleRunner: false）
- https://www.infoq.com/news/2026/11/vite-5-release ＋ https://vitest.dev/guide/migration （5.x 要求 Node 22.12+）
- https://dev.to/fabwaseem/vitest-vs-jest-vs-nodetest-in-2026-benchmarked-on-a-real-typescript-project-4bj5

## 7. 存档 / 持久化

- **IndexedDB 封装**：**idb**（Jake Archibald）仍是 2026 年的标准推荐（多个 2026 综述均列为首选），体积小、纯 promise 封装、不改变 IndexedDB 语义。idb-keyval 适合单 key 整存场景（单文件快照恰好是这种形态）。localforage 注意其 localStorage 回退路径会退化为 JSON 序列化。
- **关键机制事实**：IndexedDB 的 value 本身就走结构化克隆——**`Float32Array`、TypedArray、Map、Set、Date 都能原生无损存取，不需要先 JSON.stringify**（stringify 反而是有损且更慢的降级）。
- **单文件快照格式**：
  - **JSON**：可读、可 diff、跨版本兼容好；但对 TypedArray 要逐元素遍历（慢且体积爆炸），几百 agent 的对象图也不适合。
  - **二进制 TypedArray 序列化**：草场 Float32Array 直接存 buffer，agent 数组可用 SoA 化的 TypedArray 或 v8-serialize/结构化克隆格式；体积和速度都最优，代价是需要**版本头 + 显式 schema 演进**（字段加了怎么办）。
  - 推荐混合：文件头（schema 版本、seed、tick 计数）用 JSON，大块数值状态用 TypedArray 原始 buffer 拼接。**存档必须与确定性绑定：seed + tick + 版本号一起存，读档即从此三元组恢复确定性**。

**来源**：
- https://diragb.dev/blog/indexeddb-vs-localstorage-vs-cookies （idb/idb-keyval 推荐、localforage 陷阱）
- https://www.browser-storage.com/browser-storage-fundamentals-quotas/data-serialization-deserialization/structured-clone-vs-json-stringify-for-indexeddb （IndexedDB 原生结构化克隆，stringify 有损）
- https://web.dev/articles/structured-clone

## 额外发现

1. **确定性 × Worker 的隐藏约束**：postMessage 结构化克隆是确定性的，但**不能把 `performance.now()`/随机数混进 tick**——sim worker 的 tick 计数应由消息驱动而非时间驱动，否则快照协议本身会破坏可复现性。
2. **file:// 下 Worker 的坑**：Chrome 系从 `file://` 加载页面时，用相对 URL new Worker 会因同源策略失败——单文件分发模式下 worker 必须走 Vite 的 `?worker&inline`（base64 内联）或 Blob URL 构造。这是「sim 进 worker」与「单文件离线分发」两个目标交汇处最容易翻车的点，架构设计时先定这条。
3. **Three.js 0.170 未登记依赖**：既然要走 npm 工程化，第一步把 three 写进 package.json（现状是 npm 安装了但未登记，重装环境会丢）。
4. **Vite 8 + 单文件 + Worker 的组合已被验证可行**：vite-plugin-singlefile 2.3.x 支持到 Vite 8，且 Vite 原生提供 `?worker&inline`，三者没有已知的组合性冲突（未在本项目实测，标为「未验证」）。
5. **查不到/未核实的项**（如实声明）：koota 最新 release 与维护活跃度；OffscreenCanvas `transferControlToOffscreen` 在 WebView 的确切修复版本；Android WebView 的 WebGPU 支持状态；tinybench 之外的 tick 统计现成工具。
