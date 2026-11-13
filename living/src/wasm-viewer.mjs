// wasm-viewer.mjs — Three.js viewer consuming Snapshot v3 from the Rust wasm sim.
//
// Data path (per frame, no JSON on the hot path):
//   sim_step(sim, n, daylightPtr, n)     — advance n × 0.1s substeps
//   sim_agents_flat(sim, cap)            — copy live agents into the wasm scratch
//   sim_mineral_resampled(sim)           — 38×38 block-averaged mineral field
//   sim_counts(sim)                      — per-kind live counts
// JS reads the results through views on WebAssembly.Memory.buffer at the
// scratch offsets mirrored from wasm.rs (SCRATCH_OFF_*).
//
// The full Snapshot v3 JSON (sim_snapshot_json) is only used for the one-shot
// version/HUD readout and debug — a t=300 packet is ~480 KB, far beyond the
// per-frame budget, which is exactly why the flat C-ABI path exists.

import * as THREE from 'three';
import { createViewerRenderer, createViewerWorld, updateSunFocus, VIEWER_QUALITY } from './render-utils.mjs';
import { createMineralHeatmapPlane, updateMineralTexture } from './renderer-mineral-mat.mjs';
import { createEvolutionHistogramOverlay, createSpectrogramOverlay, DECO_HUE_MIN, DECO_HUE_SPAN, DECO_HUE_BUCKETS, HIST_FRAME_TICKS, getTickFromFrame,
  SPECTROGRAM_HISTORY_FRAMES, SPECTROGRAM_WIDTH } from './renderer-evolution-histogram.mjs';
import { createPhylogenyOverlay, readPhylogenyState, buildPhylogenyGraph,
  annotatePunctuatedEquilibrium,
  CLADE_CAP, EVENT_CAP, CLADE_RECORD_SIZE, EVENT_RECORD_SIZE, CLADE_NAME_CAP,
} from './renderer-phylogeny.mjs';
import { createNicheOverlay, buildNicheCloud } from './renderer-niche.mjs';
import { createDustOverlay, drawCataclysmVline, drawCataclysmHline, drawCataclysmGlyph,
  formatCataclysmHud, CATACLYSM_IMPACT, CATACLYSM_CLIMATE,
} from './renderer-cataclysm.mjs';
import { createEvolutionScrubber } from './evolution-scrubber.mjs';
import { encodeEvolutionHistory } from './evolution-export.mjs';
import { createSessionMeta } from './session-meta.mjs';
import { terrainHeight, WATER_LEVEL, clamp } from './world.js';
import { deriveMorphology, buildSpeciesBaseGeometry, addAnimAttributes, updateAnimAttributes,
  applyHullRatioToScale, createMorphologyCompareMode, lodLevelForDist } from './morphology.mjs';
import { createMorphologyMaterial, updateMorphoTime, getLODLevel, setLODLevel } from './morphology-shader.mjs';
import { createMobileToolbar } from './mobile-controls.mjs';

// ─── Scratch layout — keep in sync with wasm.rs SCRATCH_* constants ──────────
const AGENT_CAP = 1024;
const DAYLIGHT_CAP = 64;
const MINERAL_BYTES = 1444 * 4; // 38² f32
const AGENT_BASE = MINERAL_BYTES;
const SLOT_X = 0, SLOT_Z = 4, SLOT_KIND = 8, SLOT_ENERGY = 12,
      SLOT_HUNGER = 16, SLOT_SIZE = 20, SLOT_HUE = 24,
      SLOT_COLOR = 28, SLOT_SCALE = 32, SLOT_FLAGS = 36, SLOT_ID = 40,
      SLOT_DECO_EFF = 44;
/// Phase 9: SLOT_FLAGS bit1 —— DeCO 有 deco_efficiency 可视值时置位（非 DECO 永远为 0）。
const FLAG_LINEAGE = 1;
const FLAG_DECO_EFF = 2;
const AGENT_SLOT_SIZE = 48; // next slot starts 48 bytes later
const OFF_DAYLIGHT = AGENT_BASE + AGENT_CAP * AGENT_SLOT_SIZE;
const OFF_COUNTS = OFF_DAYLIGHT + DAYLIGHT_CAP * 8;
/// Phase 10: DECO hue 直方图桶计数区（位于 counts 之后，与 wasm.rs SCRATCH_OFF_HIST 同步）。
const OFF_HIST = OFF_COUNTS + 6 * 4;
const HIST_EVERY = 5; // 每 5 帧读一次桶计数并重绘（2D canvas 绘制 24 bar 极轻量）

/// Phase 15: 系统发育数据区（clade/event 记录 + 名称缓冲），追加在 hist 区之后，
/// 与 wasm.rs SCRATCH_OFF_CLADES / SCRATCH_OFF_EVENTS / SCRATCH_OFF_CLADE_NAME 同步。
/// 注意 JS 侧的视图必须每次调用前重建（memory.grow 可能让旧 DataView 失效）。
const OFF_CLADES = OFF_HIST + 64 * 4;
const OFF_EVENTS = OFF_CLADES + CLADE_CAP * CLADE_RECORD_SIZE;
const OFF_CLADE_NAME = OFF_EVENTS + EVENT_CAP * EVENT_RECORD_SIZE;

const KIND_ORDER = ['herb', 'pred', 'scav', 'omn', 'fish', 'deco'];
// Keyed by the raw u8 kind byte the WASM writes (h=104, p=112, s=115, o=111, f=102, d=100),
// because the readers index with dv.getUint8() — numeric, not char.
const KIND_BYTE = { 104: 'herb', 112: 'pred', 115: 'scav', 111: 'omn', 102: 'fish', 100: 'deco' };

const SEED = 20261102;
const WARMUP_TICKS = 300; // 30 sim-seconds of history so the world opens alive
const TICKS_PER_FRAME = 2; // 12× real-time at 60 fps; sim dt is 0.1 s per tick
const DAY_TICKS = 28800;   // one in-game day = 48 real minutes (matches main.js)
const GRID_SPAN = 1080;

// 个体颜色不再由 JS 定义：每个体的最终色由 Rust 侧 genome 派生（SLOT_COLOR，
// 物种锚点 + hue 变异 + FISH fin 基因），材质基色保持白色让 instanceColor 直接生效。
const SPECIES = {
  herb: { cap: 420, yOff: (s) => 0.9 * s, geo: 'sphere' },
  pred: { cap: 110, yOff: (s) => 1.0 * s, geo: 'cone' },
  scav: { cap: 40,  yOff: (s) => 0.8 * s, geo: 'coneSmall' },
  omn:  { cap: 80,  yOff: (s) => 0.8 * s, geo: 'sphereSmall' },
  fish: { cap: 60,  yOff: () => -0.8,     geo: 'fish' },
  deco: { cap: 80,  yOff: (s) => 0.4 * s, geo: 'deco' },
};

// ─── wasm loading ─────────────────────────────────────────────────────────────

async function loadWasmModule() {
  // Must be loaded over http/https — fetch('./ecology-sim.wasm') resolves
  // relative to the HTML file. No file:// fallback: see README for deploy notes.
  const resp = await fetch(`./ecology-sim-v2.wasm`, { cache: 'no-store' });
  if (!resp.ok) throw new Error(`fetch ecology-sim.wasm: ${resp.status}`);
  if (resp.headers.get('content-type')?.includes('application/wasm') && 'instantiateStreaming' in WebAssembly) {
    return (await WebAssembly.instantiateStreaming(resp, {})).instance;
  }
  const bytes = new Uint8Array(await resp.arrayBuffer());
  return (await WebAssembly.instantiate(bytes, {})).instance;
}

// ─── sim handle: scratch views are rebuilt whenever memory grows ─────────────

function createSimHandle(instance) {
  const E = instance.exports;
  const sim = E.sim_new(SEED);
  const scratchPtr = E.sim_scratch(sim);
  let views = null;
  let bufferLen = 0;

  function ensureViews() {
    const buf = E.memory.buffer;
    if (views && buf.byteLength === bufferLen) return views;
    bufferLen = buf.byteLength;
    views = {
      daylight: new Float64Array(buf, scratchPtr + OFF_DAYLIGHT, DAYLIGHT_CAP),
      counts: new Uint32Array(buf, scratchPtr + OFF_COUNTS, 6),
      hist: new Uint32Array(buf, scratchPtr + OFF_HIST, DECO_HUE_BUCKETS),
      mineral: new Float32Array(buf, scratchPtr, 1444),
      agents: new DataView(buf, scratchPtr + AGENT_BASE, AGENT_CAP * AGENT_SLOT_SIZE),
    };
    return views;
  }

  return {
    exports: E,
    views: ensureViews,
    step(n, daylightLen) { E.sim_step(sim, n, scratchPtr + OFF_DAYLIGHT, daylightLen); },
    readAgents() { return E.sim_agents_flat(sim, AGENT_CAP); },
    agentCount() { return E.sim_agent_count ? E.sim_agent_count(sim) : -1; },
    readMineral() { E.sim_mineral_resampled(sim); },
    readCounts() { E.sim_counts(sim); },
    tick() { return E.sim_tick(sim); },
    season() { return E.sim_season(sim); },
    mineralPeak() { return E.sim_mineral_peak(sim); },
    // Phase 9: DECO 种群效率统计（无 DECO 时 mean/std 均为 0）。
    decoEffMean() { return E.sim_deco_eff_mean(sim); },
    decoEffStd() { return E.sim_deco_eff_std(sim); },
    // Phase 10: DECO hue 直方图 —— 把桶计数写进 scratch 的 OFF_HIST 区，返回桶数。
    // 旧 wasm（Phase 10 之前）无此导出，调用方需先判 histSupported。
    readDecoHistogram(buckets = DECO_HUE_BUCKETS) {
      return E.sim_deco_hue_histogram(sim, buckets, scratchPtr + OFF_HIST);
    },
    // Phase 15: 系统发育拉取 —— clade/event 记录写入 scratch 的新增区域，
    // 返回 plain-object 快照（旧 wasm 无导出时返回空数组，调用方需先判 phyloSupported）。
    readPhylogeny() {
      return readPhylogenyState(E, sim,
        scratchPtr + OFF_CLADES, scratchPtr + OFF_EVENTS, scratchPtr + OFF_CLADE_NAME, CLADE_NAME_CAP);
    },
    // Phase 17: cataclysm 注入与读出（旧 wasm 无导出时为 undefined，调用方需先判
    // catSupported）。out 指针由调用方提供（boot 期追加的 1 页内存）。
    triggerCataclysm(kind, x, z, radius, severity, duration) {
      return E.sim_trigger_cataclysm(sim, kind, x, z, radius, severity, duration) | 0;
    },
    cataclysmCount() { return E.sim_cataclysm_count(sim) | 0; },
    getCataclysm(i, outPtr) { return E.sim_get_cataclysm(sim, i, outPtr) | 0; },
    envMods(outPtr) { return E.sim_env_mods(sim, outPtr) | 0; },
    // Diagnostic helpers
    getBufferLen() { return E.memory.buffer.byteLength; },
    rawCountsBytes() {
      const b = new Uint8Array(E.memory.buffer, scratchPtr + OFF_COUNTS, 24);
      return Array.from(b).map(x => x.toString(16).padStart(2, '0')).join(' ');
    },
    dispose() { E.sim_free(sim); },
  };
}

// ─── agent meshes ─────────────────────────────────────────────────────────────

function makeSpeciesGeometry(kind, lod = 0) {
  return buildSpeciesBaseGeometry(kind, lod, THREE);
}

function createSpeciesMeshes(scene) {
  const meshes = {};
  for (const kind of KIND_ORDER) {
    const spec = SPECIES[kind];
    // Phase 19: LOD — build 3 detail levels, default to LOD-0
    const lodGeos = [makeSpeciesGeometry(kind, 0), makeSpeciesGeometry(kind, 1), makeSpeciesGeometry(kind, 2)];
    const geo = lodGeos[0];
    const mat = createMorphologyMaterial(THREE, { color: 0xffffff });
    const mesh = new THREE.InstancedMesh(geo, mat, spec.cap);
    mesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    // Per-instance genome color（r150+）：直接写 attribute，避免 setColorAt 的对象分配。
    mesh.instanceColor = new THREE.InstancedBufferAttribute(new Float32Array(spec.cap * 3), 3);
    mesh.instanceColor.setUsage(THREE.DynamicDrawUsage);
    mesh.frustumCulled = false;
    mesh.count = 0;
    mesh.castShadow = kind !== 'fish';
    // Phase 19: per-instance animation attributes + LOD geometries
    addAnimAttributes(mesh, spec.cap, THREE);
    mesh.userData.lodGeos = lodGeos;
    mesh.userData.currentLOD = 0;
    scene.add(mesh);
    meshes[kind] = mesh;
  }
  return meshes;
}

const _matrix = new THREE.Matrix4();
const _pos = new THREE.Vector3();
const _quat = new THREE.Quaternion();
const _scale = new THREE.Vector3();

function updateInstancedMeshes(meshes, handle, agentCount) {
  const v = handle.views();
  const dv = v.agents;
  const perKind = { herb: 0, pred: 0, scav: 0, omn: 0, fish: 0, deco: 0 };
  for (const kind of KIND_ORDER) {
    meshes[kind].count = 0;
    // Phase 19: track agent IDs per mesh for animation attribute updates
    if (!meshes[kind].userData.agentIds) meshes[kind].userData.agentIds = [];
    meshes[kind].userData.agentIds.length = 0;
  }

  for (let i = 0; i < agentCount; i++) {
    const base = i * AGENT_SLOT_SIZE;
    const kindChar = dv.getUint8(base + SLOT_KIND);
    const kind = KIND_BYTE[kindChar];
    if (!kind) continue;
    const mesh = meshes[kind];
    if (mesh.count >= SPECIES[kind].cap) continue;
    const x = dv.getFloat32(base + SLOT_X, true);
    const z = dv.getFloat32(base + SLOT_Z, true);
    const scale = dv.getFloat32(base + SLOT_SCALE, true);
    const argb = dv.getUint32(base + SLOT_COLOR, true);
    const flags = dv.getUint32(base + SLOT_FLAGS, true);
    // Phase 19: read agent ID for morphology derivation
    const agentId = dv.getUint32(base + SLOT_ID, true);
    const spec = SPECIES[kind];
    const y = kind === 'fish'
      ? WATER_LEVEL - 0.8 + Math.sin(i * 2.1) * 0.3
      : terrainHeight(x, z) + spec.yOff(scale);
    _pos.set(x, y, z);
    // Phase 19: anisotropic scale from hull_ratio (volume-preserving)
    const { hull_ratio } = deriveMorphology(agentId);
    applyHullRatioToScale(_scale, scale, hull_ratio);
    _matrix.compose(_pos, _quat, _scale);
    mesh.setMatrixAt(mesh.count, _matrix);
    // Phase 19: track ID for animation attribute update
    mesh.userData.agentIds.push(agentId);
    const j = mesh.count * 3;
    const ca = mesh.instanceColor.array;
    ca[j] = ((argb >>> 16) & 0xff) / 255;
    ca[j + 1] = ((argb >>> 8) & 0xff) / 255;
    ca[j + 2] = (argb & 0xff) / 255;
    if (flags & FLAG_LINEAGE) {
      // lineage 成员向白偏移 22%：无 per-instance emissive 通道时的最廉价弱发光。
      ca[j] += (1 - ca[j]) * 0.22;
      ca[j + 1] += (1 - ca[j + 1]) * 0.22;
      ca[j + 2] += (1 - ca[j + 2]) * 0.22;
    }
    mesh.count += 1;
    perKind[kind] += 1;
  }
  for (const kind of KIND_ORDER) {
    const mesh = meshes[kind];
    mesh.instanceMatrix.needsUpdate = true;
    mesh.instanceColor.needsUpdate = true;
    // Phase 19: update per-instance animation attributes (aPhase, aSwayAmp)
    if (mesh.userData.agentIds.length > 0) {
      updateAnimAttributes(mesh, mesh.userData.agentIds, THREE);
    }
  }
  return perKind;
}

// ─── daylight curve (mirrors the JS viewer: one day per 48 real minutes) ─────

// 与 JS 路径一致（sky.js createEnvironment 的 startHour 默认值）：世界从午后开始。
// 若不偏移，tick 0 对应 0 点 = 午夜，开局长时间黑夜。
const START_HOUR = 16.4;

function hourFor(tick) {
  return (START_HOUR + tick * 0.1 * 24 / DAY_TICKS) % 24;
}

function daylightFor(tick) {
  const hour = hourFor(tick);
  const elev = Math.sin(((hour - 6) / 12) * Math.PI);
  return clamp((elev + 0.03) / 0.27, 0, 1);
}

// ─── HUD ──────────────────────────────────────────────────────────────────────

function createHud() {
  const el = document.createElement('div');
  el.id = 'wasm-hud';
  el.style.cssText = [
    'position:fixed', 'top:max(18px, env(safe-area-inset-top))', 'right:max(20px, env(safe-area-inset-right))',
    'z-index:30', 'pointer-events:none', 'text-align:right',
    'font:11px/1.7 "SF Mono", ui-monospace, monospace',
    'color:#cfe3d6', 'text-shadow:0 1px 3px rgba(0,0,0,0.8)',
    'background:rgba(10,16,18,0.5)', 'padding:10px 14px', 'border-radius:10px',
    'border:1px solid rgba(210,232,220,0.14)',
  ].join(';');
  document.body.appendChild(el);
  return el;
}

function createToast() {
  const el = document.createElement('div');
  el.id = 'wasm-toast';
  el.style.cssText = [
    'position:fixed', 'left:50%', 'bottom:96px', 'transform:translateX(-50%)',
    'z-index:40', 'pointer-events:none', 'font:12px/1.5 inherit',
    'color:#e9eee9', 'background:rgba(10,16,18,0.78)', 'padding:8px 18px',
    'border-radius:999px', 'border:1px solid rgba(210,232,220,0.2)',
    'opacity:0', 'transition:opacity 0.25s', 'white-space:nowrap',
  ].join(';');
  document.body.appendChild(el);
  let timer = 0;
  return (text) => {
    el.textContent = text;
    el.style.opacity = '1';
    clearTimeout(timer);
    timer = setTimeout(() => { el.style.opacity = '0'; }, 2200);
  };
}

// ─── boot ─────────────────────────────────────────────────────────────────────

export async function bootWasmViewer(canvasId = 'stage') {
  const canvas = document.getElementById(canvasId);
  const toast = createToast();
  const hud = createHud();
  const status = document.getElementById('load-status');
  const setStatus = (t) => { if (status) status.textContent = t; };

  setStatus('加载 WASM 生态内核…');
  const instance = await loadWasmModule();
  // Phase 17: cataclysm ABI 出参缓冲 —— 追加 1 页 wasm 内存（旧 wasm 无这些导出
  // 时跳过；memory.grow 会使旧视图失效，ensureViews() 按 byteLength 自动重建）。
  const catSupported = typeof instance.exports.sim_trigger_cataclysm === 'function';
  let catBufPtr = 0;
  if (catSupported) {
    const pagesBefore = instance.exports.memory.buffer.byteLength / 65536;
    instance.exports.memory.grow(1);
    catBufPtr = pagesBefore * 65536;
  }
  const handle = createSimHandle(instance);
  const v = handle.views();

  setStatus('预热生态（300 ticks）…');
  for (let done = 0; done < WARMUP_TICKS; done += DAYLIGHT_CAP) {
    const n = Math.min(DAYLIGHT_CAP, WARMUP_TICKS - done);
    for (let i = 0; i < n; i++) v.daylight[i] = daylightFor(handle.tick() + i);
    handle.step(n, n);
  }

  // Phase 9: DECO 效率演化基线 —— 以预热后为 t0，后续 HUD 显示相对增量。
  const decoBaselineMean = handle.decoEffMean();
  const decoBaselineStd = handle.decoEffStd();

  setStatus('生成地形与天空…');
  const { renderer, scene, camera, controls } = createViewerRenderer(canvas);
  const { env, landmarks } = createViewerWorld(scene, renderer);

  const meshes = createSpeciesMeshes(scene);
  const { mesh: mineralPlane, texture: mineralTexture } = createMineralHeatmapPlane(GRID_SPAN, 38);
  mineralPlane.visible = false;
  scene.add(mineralPlane);

  setStatus('就绪');
  const enter = document.getElementById('enter');
  const loading = document.getElementById('loading');

  // Mobile: add touch button bar + auto-hide all overlays by default (no-op on desktop)
  const mobileBar = createMobileToolbar();
  if (mobileBar) {
    // On mobile: start with all overlays OFF so the 3D view dominates.
    // User taps buttons to toggle them on.
    const hideOverlay = (el) => { if (el) el.style.display = 'none'; };
    // The overlays are created later in the boot sequence; mark them off on next frame.
    setTimeout(() => {
      document.querySelectorAll('#hist-overlay, #spec-overlay, #phy-overlay, #cat-overlay, #niche-overlay, #geo-overlay, #bio-overlay, .overlay-panel').forEach(el => {
        el.dataset.wasShown = '1';
        el.style.display = 'none';
      });
    }, 50);
  }
  if (enter) {
    enter.disabled = false;
    const label = document.getElementById('enter-label');
    if (label) label.textContent = '进入世界';
    enter.addEventListener('click', () => {
      if (enter.disabled) return;
      if (loading) {
        loading.classList.add('hidden');
        setTimeout(() => loading.remove(), 1400);
      }
    });
  }

  // mineral heatmap auto-gain: raw values stay ≪1 early on, so scale by the
  // current peak (render-side only; the v3 data contract is untouched).
  const mineralNorm = new Float32Array(1444);

  // Census from the flat agent buffer directly — independent of sim_counts's
  // own scratch layout, so it works even if memory grew between calls.
  function censusFromFlat(n) {
    const dv = handle.views().agents;
    const c = [0, 0, 0, 0, 0, 0];
    for (let i = 0; i < n; i++) {
      const k = KIND_BYTE[dv.getUint8(i * AGENT_SLOT_SIZE + SLOT_KIND)];
      if (k) c[KIND_ORDER.indexOf(k)] += 1;
    }
    return c;
  }

  let mineralOn = false;
  let paused = false;
  let histFrame = 0;
  let lastHud = 0;
  let fpsAcc = 0, fpsN = 0, fps = 0;

  // Phase 10: DECO hue 直方图 overlay（旧 wasm 无 sim_deco_hue_histogram 时自动降级隐藏）。
  const histSupported = typeof instance.exports.sim_deco_hue_histogram === 'function';
  const histOverlay = histSupported ? createEvolutionHistogramOverlay() : null;
  const isMobile = /Android|iPhone/i.test(navigator.userAgent) || (navigator.maxTouchPoints > 0 && window.innerWidth < 900);
  let histOn = !isMobile;
  const histCounts = new Uint32Array(DECO_HUE_BUCKETS);
  let histTotal = 0, histPeak = 0, histTick = 0;

  // Phase 11: DECO hue 演化光谱条纹图 overlay（旧 wasm 无导出时降级隐藏）。
  const specOverlay = histSupported ? createSpectrogramOverlay() : null;
  let specOn = !isMobile; // 默认 ON，与 histogram 独立（移动端默认关）
  let specMax = 0;

  // Phase 13：spectral 时间锚点与累计帧计数器（HUD / 刻度标注用）。
  let specNewestTick = 0;   // 最近一次 pushFrame 时的 sim tick
  let specTotalPushed = 0;  // 开机以来总 push 次数（不随 ring 封顶），用于 HUD frame N/M
  // Phase 14: 实验回路状态机 —— 区分 live 推帧与 R 键恢复的快照；liveCount 独立于
  // loadedRows（不受 load 影响），clear() 后归零重新计数。
  const sessionMeta = createSessionMeta();

  // Phase 15: 物种系统发育 overlay（旧 wasm 无 sim_clade_count 时自动降级隐藏）。
  const phyloSupported = typeof instance.exports.sim_clade_count === 'function';
  const phyloOverlay = phyloSupported ? createPhylogenyOverlay() : null;
  let phyloOn = !isMobile;   // T 键切换（移动端默认关）
  let fossilOn = false; // F 键切换化石高亮，默认 OFF
  let phyloClades = [];
  let phyloEvents = [];
  const seenCladeIds = new Set();
  const seenDeadIds = new Set();
  let lastPhyloPull = -Infinity;

  // Phase 16: 生态位空间 overlay（U 键切换；复用 phylogeny 的 60-tick 拉取节奏）。
  const nicheOverlay = phyloSupported ? createNicheOverlay() : null;
  let nicheOn = !isMobile;

  // Phase 17: 灾变系统 —— 注入（C/X 键）、烟尘 overlay、spectrogram/phylogeny
  // 时间标记与 HUD 历史（旧 wasm 无导出时整体降级隐藏）。
  const dustOverlay = catSupported ? createDustOverlay() : null;
  let catHistory = [];      // sim_get_cataclysm 拉取快照（旧 → 新）
  let lastCatPull = -Infinity;

  function pullCataclysms(force = false) {
    if (!catSupported) return;
    const t = handle.tick();
    if (!force && t - lastCatPull < 60) return;
    lastCatPull = t;
    const count = Math.min(handle.cataclysmCount(), 64);
    const list = [];
    for (let i = 0; i < count; i++) {
      if (handle.getCataclysm(i, catBufPtr) === 0) break;
      const dv = new DataView(instance.exports.memory.buffer, catBufPtr, 40);
      list.push({
        id: dv.getUint32(0, true),
        kind: dv.getUint32(4, true),
        tick: dv.getUint32(8, true),
        endsAt: dv.getUint32(12, true),
        remaining: dv.getUint32(16, true),
        x: dv.getFloat32(20, true),
        z: dv.getFloat32(24, true),
        radius: dv.getFloat32(28, true),
        severity: dv.getFloat32(32, true),
        active: dv.getUint32(36, true) === 1,
      });
    }
    catHistory = list;
  }

  // 灾变时间标记：phylogeny（时间轴横向 → 竖线 + 图标）与 spectrogram（时间轴
  // 纵向 → 横线 + 图标）。overlay 自身的 render 会整面重绘并抹掉标记，因此这里
  // 每帧在其后补画（几条虚线 + 小图标，开销可忽略）。
  function drawCataclysmMarks() {
    if (!catSupported || catHistory.length === 0) return;
    if (phyloOverlay && phyloOverlay.visible && phyloOverlay.graph) {
      const g = phyloOverlay.graph;
      const w = 360, h = 280; // 与 createPhylogenyOverlay 默认几何同步
      const PAD = { top: 26, bottom: 20, left: 84, right: 30 };
      const ctx = phyloOverlay.canvas.getContext('2d');
      const span = Math.max(1, g.currentTick - g.rootTick);
      for (const c of catHistory) {
        if (c.tick < g.rootTick) continue;
        const x = PAD.left + ((c.tick - g.rootTick) / span) * (w - PAD.left - PAD.right);
        drawCataclysmVline(ctx, c.kind, x, PAD.top, h - PAD.bottom);
        drawCataclysmGlyph(ctx, c.kind, x, PAD.top + 12);
      }
    }
    if (specOverlay && specOverlay.visible && specNewestTick > 0) {
      const ctx = specOverlay.canvas.getContext('2d');
      for (const c of catHistory) {
        const back = (specNewestTick - c.tick) / HIST_FRAME_TICKS;
        if (back < 0 || back >= SPECTROGRAM_HISTORY_FRAMES) continue;
        const y = Math.round(back);
        drawCataclysmHline(ctx, c.kind, y, 12, SPECTROGRAM_WIDTH - 12);
        drawCataclysmGlyph(ctx, c.kind, SPECTROGRAM_WIDTH - 22, Math.max(9, y - 7));
      }
    }
  }

  function renderPhylo() {
    if (!phyloOverlay) return;
    phyloOverlay.update(annotatePunctuatedEquilibrium(buildPhylogenyGraph(
      phyloClades, phyloEvents, handle.tick(), { fossilHighlight: fossilOn })));
  }

  // Phase 16: niche 云图渲染（与 cladogram 共享同一份 clade 快照）。
  function renderNiche() {
    if (!nicheOverlay) return;
    nicheOverlay.update(buildNicheCloud(phyloClades, handle.tick()));
  }

  function pullPhylogeny(initial = false) {
    if (!phyloOverlay) return;
    const t = handle.tick();
    if (!initial && t - lastPhyloPull < 60) return; // 与生态采样节奏对齐（每 60 tick）
    lastPhyloPull = t;
    const state = handle.readPhylogeny();
    for (const c of state.clades) {
      if (!seenCladeIds.has(c.id)) {
        seenCladeIds.add(c.id);
        if (!initial && c.parentId !== null) {
          const ev = state.events.find((e) => e.childCladeIds.includes(c.id));
          toast(`🧬 ${c.name} born @ t=${c.birthTick}`
            + (ev ? ` · d=${ev.geneticDistance.toFixed(2)} vs parent` : ''));
        }
      }
      if (c.extinct && !seenDeadIds.has(c.id)) {
        seenDeadIds.add(c.id);
        if (!initial) {
          // Phase 16: 竞争排斥灭绝（event kind 2）用 ⚔ 标记，区分普通死亡 ⚰。
          const xev = state.events.find((e) => e.parentCladeId === c.id && (e.kind ?? 0) >= 1);
          toast(xev && xev.kind === 2
            ? `⚔ ${c.name} 竞争排斥灭绝 @ t=${c.deathTick}`
            : `⚰ ${c.name} extinct @ t=${c.deathTick}`);
        }
      }
    }
    phyloClades = state.clades;
    phyloEvents = state.events;
    renderPhylo();
    renderNiche();
  }
  let pngReadyUntil = -Infinity; // S 键下载提示行的消失时刻（clock.elapsedTime 域）
  let pngReadyTick = 0;
  // clock 上移至此：keydown 处理程序（含 S 键）在 boot 早期就可能触发，必须避免 TDZ。
  const clock = new THREE.Clock();

  // Phase 12: 时间轴拖拽条（挂在 spectrogram 正下方，仅当 spec 可用时创建）。
  const scrubber = specOverlay ? createEvolutionScrubber({ historyBuf: specOverlay }) : null;
  if (scrubber) {
    // 修正到 spectrogram 正下方 8px 间隙（specOverlay top=190, height=720 → bottom=910）。
    scrubber.canvas.style.top = `${190 + 720 + 8}px`;
    scrubber.setVisible(true);
    scrubber.draw(); // 立即渲染初始指针
  }

  function drawHist() {
    if (!histOverlay || !histSupported) return;
    const nB = handle.readDecoHistogram();
    const hv = handle.views().hist;
    histCounts.set(hv.subarray(0, nB));
    histTotal = 0; histPeak = 0;
    for (let i = 0; i < nB; i++) {
      histTotal += histCounts[i];
      if (histCounts[i] > histCounts[histPeak]) histPeak = i;
    }
    histTick = handle.tick();
    // Phase 14: loaded 模式 —— ring 冻结在恢复的快照上，直方图/光谱只渲染快照内容；
    // live 桶计数仅喂 sessionMeta（不进 ring、不覆盖旧行），R 再次按下即清除回 live。
    if (sessionMeta.getState() === 'loaded') {
      sessionMeta.pushLive(histCounts);
      if (!specOverlay || !specOn) return;
      const scrubbing = scrubber && !scrubber.isPlaying();
      const back = scrubbing ? Math.abs(scrubber.offset) : 0;
      const frameCounts = specOverlay.getFrameAt(scrubbing ? scrubber.offset : 0);
      if (!frameCounts) return;
      let smax = 0;
      for (let i = 0; i < frameCounts.length; i++) if (frameCounts[i] > smax) smax = frameCounts[i];
      const frameTick = specNewestTick - getTickFromFrame(back, HIST_FRAME_TICKS);
      histOverlay.update(frameCounts, smax, frameTick);
      if (scrubbing) specOverlay.renderScrubbed(scrubber.offset, smax, frameTick);
      else specOverlay.render(specMax, specNewestTick);
      return;
    }
    histOverlay.update(histCounts, histTotal > 0 ? histCounts[histPeak] : 0, histTick);
    // Phase 11/12: spectrogram 逻辑——播放模式实时 pushFrame+render；scrub 模式只读 buffer 不调 push。
    if (!specOverlay || !specOn || histTotal <= 0) return;
    if (scrubber && !scrubber.isPlaying()) {
      // Scrub 模式：读取选中帧，直接渲染（不污染 ring buffer）。
      // 同时刷新直方图显示该帧分布。
      const scrubbedCounts = specOverlay.getFrameAt(scrubber.offset);
      if (scrubbedCounts) {
        let smax = 0;
        for (let i = 0; i < scrubbedCounts.length; i++) if (scrubbedCounts[i] > smax) smax = scrubbedCounts[i];
        // 选中帧的绝对 tick：anchor − back×HIST_FRAME_TICKS（histTick 在 scrub 期间继续前进，不能用作帧时刻）。
        const frameTick = specNewestTick - getTickFromFrame(Math.abs(scrubber.offset), HIST_FRAME_TICKS);
        histOverlay.update(scrubbedCounts, smax, frameTick); // histogram overlay 同步到该帧
        specOverlay.renderScrubbed(scrubber.offset, smax, frameTick);
      }
    } else {
      // Play 模式：pushFrame 并渲染最新状态
      specOverlay.pushFrame(histCounts);
      sessionMeta.pushLive(histCounts);
      specMax = Math.max(specMax, histCounts[histPeak]);
      specNewestTick = histTick;
      specTotalPushed += 1;
      if (scrubber) {
        scrubber.setTickAnchor(specNewestTick);
        scrubber.draw();
      }
      specOverlay.render(specMax, specNewestTick);
    }
  }
  // 把 scrubber offset 变化与 viewer 的渲染路径打通（外部注册回调）。
  if (scrubber) {
    scrubber.setOnChange((isPlayingState, newOffset) => {
      // 重绘一次以立即反映新状态。
      if (specOverlay && specOn && (histTotal > 0 || sessionMeta.getState() === 'loaded')) drawHist();
      if (scrubber) scrubber.draw();
    });
    // 把初始可见性交给 drawHist 首次调用后决定（warmup 阶段已经先画了），这里保持 true。
  }
  if (histOverlay) {
    histOverlay.setVisible(histOn);
    if (specOverlay) specOverlay.setVisible(specOn);
    drawHist(); // 预热阶段已产生 15 只 DECO，立即让面板有内容
  }
  if (phyloOverlay) {
    phyloOverlay.setVisible(phyloOn);
    pullPhylogeny(true); // 预热后首次拉取（root clade 已在 t≈60 诞生）
  }
  if (nicheOverlay) {
    nicheOverlay.setVisible(nicheOn);
    renderNiche();
  }
  if (catSupported) {
    pullCataclysms(true); // 预热阶段无灾变，仅建立空基线
    dustOverlay.setDust(1);
  }

  // Phase 19: morphology compare mode (V key)
  const morphoCompare = createMorphologyCompareMode(scene, THREE);
  let morphoCompareAgents = []; // populated each frame from agent IDs

  window.addEventListener('keydown', (e) => {
    if (e.code === 'KeyM') {
      mineralOn = !mineralOn;
      mineralPlane.visible = mineralOn;
      toast(`矿化热力层 ${mineralOn ? 'ON' : 'OFF'}（M 切换）`);
  } else if (e.code === 'Space') {
      e.preventDefault();
      // SPACE：优先控制 scrub 模式（pause/play），仅当不在 scrub 时切换生态暂停。
      if (scrubber) {
        if (scrubber.isPlaying()) {
          scrubber.pause();
          toast('生态时间已暂停');
        } else {
          scrubber.play();
          toast('生态继续演化');
        }
        drawHist();
        scrubber.draw();
      } else {
        paused = !paused;
        toast(paused ? '生态时间已暂停' : '生态继续演化');
      }
    } else if (e.code === 'KeyN' && histOverlay) {
      histOn = !histOn;
      histOverlay.setVisible(histOn);
      if (histOn) drawHist(); // 重新显示时立即重绘，避免空白面板等下一帧
      toast(`DECO hue 直方图 ${histOn ? 'ON' : 'OFF'}（N 切换）`);
    } else if (e.code === 'KeyP' && specOverlay) {
      specOn = !specOn;
      specOverlay.setVisible(specOn);
      if (specOn) drawHist(); // 恢复显示时立即补帧
      toast(`DECO hue 演化光谱 ${specOn ? 'ON' : 'OFF'}（P 切换）`);
    } else if (scrubber && e.code === 'ArrowLeft') {
      e.preventDefault();
      const step = -10;
      const total = specOverlay.frameLen;
      if (total > 0) {
        scrubber.pause();
        const newOffset = Math.max(specOverlay.getEarliestReadable(), scrubber.offset + step);
        const frameIdx = total - 1 + newOffset;
        scrubber.updatePointerFromFrame(Math.max(0, frameIdx));
        scrubber.draw();
        drawHist();
      }
      toast('← 回退 10 帧');
    } else if (scrubber && e.code === 'ArrowRight') {
      e.preventDefault();
      const step = 10;
      const total = specOverlay.frameLen;
      if (total > 0) {
        const newOffset = Math.min(0, scrubber.offset + step);
        const frameIdx = total - 1 + newOffset;
        scrubber.updatePointerFromFrame(Math.min(total - 1, frameIdx));
        scrubber.draw();
        drawHist();
      }
      toast('→ 前进 10 帧');
    } else if (scrubber && e.code === 'Home') {
      e.preventDefault();
      const total = specOverlay.frameLen;
      if (total > 0) {
        scrubber.pause();
        scrubber.updatePointerFromFrame(0); // oldest frame
        scrubber.draw();
        drawHist();
      }
      toast('跳到最早帧');
    } else if (scrubber && e.code === 'End') {
      e.preventDefault();
      const total = specOverlay.frameLen;
      if (total > 0) {
        scrubber.play();
        scrubber.draw();
        drawHist();
      }
      toast('跳到最新帧');
    } else if (e.code === 'KeyU' && nicheOverlay) {
      // Phase 16: 生态位空间开关。
      nicheOn = !nicheOn;
      nicheOverlay.setVisible(nicheOn);
      if (nicheOn) renderNiche();
      toast(`niche space ${nicheOn ? 'ON' : 'OFF'}（U 切换）`);
    } else if (e.code === 'KeyT' && phyloOverlay) {
      phyloOn = !phyloOn;
      phyloOverlay.setVisible(phyloOn);
      if (phyloOn) renderPhylo();
      toast(`species phylogeny ${phyloOn ? 'ON' : 'OFF'}（T 切换）`);
    } else if (e.code === 'KeyF' && phyloOverlay) {
      fossilOn = !fossilOn;
      renderPhylo();
      toast(`fossil highlight ${fossilOn ? 'ON' : 'OFF'}（F 切换）`);
    } else if (e.code === 'KeyC' && catSupported) {
      // Phase 17: 演示小行星 —— 固定参数（地图中心偏移），实验者手动注入。
      const id = handle.triggerCataclysm(CATACLYSM_IMPACT, 140, -90, 170, 0.9, 600);
      if (id > 0) {
        pullCataclysms(true);
        toast(`☄ 小行星撞击已注入 (#${id}) · 尘幕 600t`);
      } else {
        toast('注入失败（参数无效）');
      }
    } else if (e.code === 'KeyX' && catSupported) {
      // Phase 17: 演示气候突变 —— 全图草地生长 ×0.5，持续 1200t。
      const id = handle.triggerCataclysm(CATACLYSM_CLIMATE, 0, 0, 0, 0.5, 1200);
      if (id > 0) {
        pullCataclysms(true);
        toast(`🌡 气候突变已注入 (#${id}) · 生长 ×0.5 · 1200t`);
      } else {
        toast('注入失败（参数无效）');
      }
    } else if (e.code === 'KeyS') {
      // S 键保存 snapshot：截取当前 WebGL canvas + overlay canvases 合并为一张 PNG，存 localStorage。
      try {
        const snapCanvas = document.createElement('canvas');
        snapCanvas.width = canvas.width;
        snapCanvas.height = canvas.height;
        const sctx = snapCanvas.getContext('2d');
        sctx.drawImage(canvas, 0, 0);
        // 叠加 overlay canvas（仅可见的）。
        for (const ov of [histOverlay, specOverlay, phyloOverlay, nicheOverlay]) {
          if (ov && ov.visible) sctx.drawImage(ov.canvas, 0, 0);
        }
        // Phase 13：左上角信息条（canvas 2D 直绘，不依赖 DOM）。
        const scrubbing = scrubber && !scrubber.isPlaying();
        const back = scrubbing ? Math.abs(scrubber.offset) : 0;
        const frameTick = specNewestTick - getTickFromFrame(back, HIST_FRAME_TICKS);
        const frameCounts = scrubbing && specOverlay ? specOverlay.getFrameAt(back) : histCounts;
        let bucketMean = 0;
        if (frameCounts) {
          let sum = 0;
          for (let i = 0; i < frameCounts.length; i++) sum += frameCounts[i];
          bucketMean = sum / DECO_HUE_BUCKETS;
        }
        const info = `tick=${frameTick} · frame=${specTotalPushed - back}/${specTotalPushed}`
          + ` · DECO eff=${handle.decoEffMean().toFixed(2)} ± ${handle.decoEffStd().toFixed(2)}${scrubbing ? ' (live)' : ''}`
          + ` · mean bucket=${bucketMean.toFixed(2)}/${DECO_HUE_BUCKETS}`;
        const px = snapCanvas.width / Math.max(1, canvas.clientWidth || snapCanvas.width);
        sctx.font = `bold ${Math.round(12 * px)}px "SF Mono", ui-monospace, monospace`;
        const tw = sctx.measureText(info).width;
        const bx = 10 * px, by = 10 * px, bh = 22 * px;
        sctx.fillStyle = 'rgba(10,16,18,0.72)';
        sctx.fillRect(bx, by, tw + 16 * px, bh);
        sctx.fillStyle = '#e9f2ec';
        sctx.textBaseline = 'middle';
        sctx.fillText(info, bx + 8 * px, by + bh / 2);
        // Phase 14 B：右下角下载就绪图标（canvas 直绘，随 PNG 一并保存）。
        const cx = snapCanvas.width - 26 * px, cy = snapCanvas.height - 26 * px;
        sctx.fillStyle = 'rgba(10,16,18,0.72)';
        sctx.beginPath();
        sctx.arc(cx, cy, 14 * px, 0, Math.PI * 2);
        sctx.fill();
        sctx.strokeStyle = '#e9f2ec';
        sctx.lineWidth = Math.max(1.5, 2 * px);
        sctx.lineCap = 'round';
        sctx.lineJoin = 'round';
        sctx.beginPath();
        sctx.moveTo(cx, cy - 6 * px);
        sctx.lineTo(cx, cy + 2 * px);
        sctx.moveTo(cx - 4 * px, cy - 1 * px);
        sctx.lineTo(cx, cy + 3 * px);
        sctx.lineTo(cx + 4 * px, cy - 1 * px);
        sctx.moveTo(cx - 5 * px, cy + 6 * px);
        sctx.lineTo(cx + 5 * px, cy + 6 * px);
        sctx.stroke();
        const dataUrl = snapCanvas.toDataURL('image/png');
        localStorage.setItem(`deco-snapshot-${Date.now()}`, dataUrl);
        // Phase 14 B：一键下载（Blob URL + 临时 <a download>；URL 延迟回收保证点击生效）。
        snapCanvas.toBlob((blob) => {
          if (!blob) return;
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = `aether-snapshot-t${frameTick}.png`;
          a.click();
          setTimeout(() => URL.revokeObjectURL(url), 3000);
        }, 'image/png');
        pngReadyUntil = clock.elapsedTime + 3;
        pngReadyTick = frameTick;
        toast(`快照已保存 + 已触发下载（t=${frameTick}）`);
      } catch (err) {
        toast(`保存失败: ${err.message}`);
      }
    } else if (e.code === 'KeyE' && specOverlay) {
      // Phase 13：E 键把整个 ring buffer 导出为 base64 JSON 存 localStorage（离线分析）。
      try {
        if (specOverlay.frameLen === 0) {
          toast('光谱 buffer 为空，无可导出数据');
        } else {
          const rows = [];
          for (let back = specOverlay.frameLen - 1; back >= 0; back--) rows.push(specOverlay.getFrameAt(back));
          const json = encodeEvolutionHistory({
            rows,
            buckets: DECO_HUE_BUCKETS,
            hueMin: DECO_HUE_MIN,
            hueSpan: DECO_HUE_SPAN,
            capturedTick: handle.tick(),
          });
          localStorage.setItem('aether-evolution-history', json);
          toast(`✓ saved ${specOverlay.frameLen}×${DECO_HUE_BUCKETS} evolution history`);
        }
      } catch (err) {
        toast(`导出失败: ${err.message}`);
      }
    } else if (e.code === 'KeyR' && specOverlay) {
      // Phase 14 A：R 键在「恢复 localStorage 快照」与「清除回 live」之间切换。
      if (sessionMeta.getState() === 'loaded') {
        clearLoadedHistory();
        toast('已清空加载的历史，回到 live 模式');
      } else {
        const res = tryLoadSavedHistory();
        toast(res.ok
          ? `Loaded ${res.loaded}×${res.buckets} frames from session ${res.capturedAt}（再按 R 清除）`
          : res.err);
      }
    } else if (e.code === 'KeyV') {
      // Phase 19: V 键切换「形态对比模式」（同 clade 两代并排显示）
      const on = morphoCompare.toggle();
      if (on && morphoCompareAgents.length >= 2) {
        morphoCompare.show(morphoCompareAgents[0], morphoCompareAgents[1]);
        toast('形态对比 ON（V 关闭）');
      } else if (!on) {
        toast('形态对比 OFF');
      } else {
        toast('形态对比：需要 ≥2 个个体（等待 sim 产出数据）');
      }
    }
  });

  // Phase 14 A：从 localStorage 恢复 Phase 13 导出的 evolution history 到 spectrogram
  // ring buffer。只叠加在 UI 层，sim 内部状态零触碰，生态继续实时运行。
  function tryLoadSavedHistory() {
    const raw = localStorage.getItem('aether-evolution-history');
    if (!raw) return { ok: false, err: 'No saved evolution history（先按 E 键导出一份）' };
    let info;
    try {
      info = sessionMeta.loadFromJSON(raw); // 先校验解码，失败时不碰 ring buffer
    } catch (err) {
      return { ok: false, err: `加载失败：${err.message}` };
    }
    const rows = sessionMeta.getLoadedRows();
    specOverlay.clear();
    let peak = 0;
    for (const row of rows) {
      for (let b = 0; b < row.counts.length; b++) if (row.counts[b] > peak) peak = row.counts[b];
    }
    specMax = Math.max(specMax, peak);
    for (const row of rows) specOverlay.pushFrame(row.counts);
    specNewestTick = info.capturedTick;
    specTotalPushed = rows.length;
    if (scrubber) {
      scrubber.play(); // 退出可能残留的 scrub，指针钉回最新帧
      scrubber.setTickAnchor(specNewestTick);
      scrubber.draw();
    }
    const newest = specOverlay.getFrameAt(0);
    if (newest) {
      let smax = 0;
      for (let i = 0; i < newest.length; i++) if (newest[i] > smax) smax = newest[i];
      histOverlay.update(newest, smax, specNewestTick);
    }
    specOverlay.render(specMax, specNewestTick); // 立即渲染恢复后的光谱，不等下一采样帧
    return { ok: true, loaded: rows.length, buckets: info.buckets, capturedAt: info.capturedAt };
  }

  // R 键二次按下 / HUD 徽标点击共用：清空快照回到 live 模式。
  function clearLoadedHistory() {
    specOverlay.clear();
    sessionMeta.clear();
    specTotalPushed = 0;
    specMax = 0;
    specNewestTick = 0;
    if (scrubber) {
      scrubber.play(); // 若原在 scrub 态，onChange → drawHist 会立即推一帧 live 并同步 anchor
      scrubber.draw();
    }
    drawHist();
  }

  // Phase 14: HUD 徽标点击 = 清除快照回 live（与 R 键二次按下等效）。
  hud.addEventListener('click', (e) => {
    if (e.target && e.target.id === 'loaded-badge' && sessionMeta.getState() === 'loaded') {
      clearLoadedHistory();
      toast('已清空加载的历史，回到 live 模式');
    }
  });

  // ─── 死亡脉冲（Phase 5）：个体消失当帧在原位白闪 0.3s，强化「死亡滋养土地」叙事。
  // 检测依赖 SLOT_ID 跨帧对比：上帧在场、本帧不在场的 id 视为死亡（id 单调不复用）。
  const GHOST_CAP = 96;
  const GHOST_TTL = 0.3;
  const ghostMat = new THREE.MeshBasicMaterial({
    color: 0xffffff,
    transparent: true,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  const ghostMesh = new THREE.InstancedMesh(new THREE.SphereGeometry(1.0, 8, 6), ghostMat, GHOST_CAP);
  ghostMesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
  ghostMesh.instanceColor = new THREE.InstancedBufferAttribute(new Float32Array(GHOST_CAP * 3), 3);
  ghostMesh.frustumCulled = false;
  ghostMesh.count = 0;
  scene.add(ghostMesh);
  const ghosts = []; // { x, y, z, born }
  let prevAgents = new Map(); // id → { x, z, kind }（上一帧快照）

  function trackDeaths(agentCount) {
    const dv = handle.views().agents;
    const current = new Map();
    for (let i = 0; i < agentCount; i++) {
      const base = i * AGENT_SLOT_SIZE;
      const kind = KIND_BYTE[dv.getUint8(base + SLOT_KIND)];
      if (!kind) continue;
      current.set(dv.getUint32(base + SLOT_ID, true), {
        x: dv.getFloat32(base + SLOT_X, true),
        z: dv.getFloat32(base + SLOT_Z, true),
        kind,
      });
    }
    for (const [id, prev] of prevAgents) {
      if (current.has(id)) continue;
      if (ghosts.length >= GHOST_CAP) ghosts.shift();
      ghosts.push({
        x: prev.x,
        y: prev.kind === 'fish' ? WATER_LEVEL - 0.6 : terrainHeight(prev.x, prev.z) + 0.9,
        z: prev.z,
        born: clock.elapsedTime,
      });
    }
    prevAgents = current;
  }

  function updateGhosts(now) {
    for (let g = ghosts.length - 1; g >= 0; g--) {
      if (now - ghosts[g].born > GHOST_TTL) ghosts.splice(g, 1);
    }
    let w = 0;
    for (const gh of ghosts) {
      const t = (now - gh.born) / GHOST_TTL;
      _pos.set(gh.x, gh.y, gh.z);
      _scale.setScalar(0.8 + t * 1.4);
      _matrix.compose(_pos, _quat, _scale);
      ghostMesh.setMatrixAt(w, _matrix);
      const ca = ghostMesh.instanceColor.array;
      const fade = (1 - t) * (1 - t);
      ca[w * 3] = ca[w * 3 + 1] = ca[w * 3 + 2] = fade;
      w += 1;
    }
    ghostMesh.count = w;
    ghostMesh.instanceMatrix.needsUpdate = true;
    ghostMesh.instanceColor.needsUpdate = true;
  }

  function tick() {
    requestAnimationFrame(tick);
    const raw = Math.min(0.05, Math.max(0, clock.getDelta()));
    fpsAcc += raw; fpsN += 1;

    const elapsed = clock.elapsedTime;
    const hour = hourFor(handle.tick());
    env.setTime(hour);
    updateSunFocus(env, controls.target);
    env.update(raw, elapsed);
    landmarks.update(raw, elapsed);
    controls.update();

    if (!paused) {
      for (let i = 0; i < TICKS_PER_FRAME; i++) {
        v.daylight[i] = daylightFor(handle.tick() + i);
      }
      handle.step(TICKS_PER_FRAME, TICKS_PER_FRAME);
      const n = handle.readAgents();
      if (n === 0 && handle.tick() > 100) {
        console.warn('[agent-debug] readAgents()=0 at tick', handle.tick(),
          'agentCount:', handle.agentCount(), 'decoEff:', handle.decoEffMean().toFixed(4));
      }      updateInstancedMeshes(meshes, handle, n);
      trackDeaths(n);
      // Phase 19: update uTime uniform + LOD switch + compare mode data
      for (const kind of KIND_ORDER) {
        updateMorphoTime(meshes[kind].material, elapsed);
        const lodLevel = getLODLevel(camera.position.length());
        if (lodLevel !== meshes[kind].userData.currentLOD) {
          setLODLevel(meshes[kind], lodLevel, THREE);
          meshes[kind].userData.currentLOD = lodLevel;
        }
      }
      // Populate morphoCompareAgents from the first species mesh with data
      morphoCompareAgents = (meshes.herb.userData.agentIds || [])
        .map((id, idx) => ({ id, kind: 'herb' }));
      if (morphoCompareAgents.length < 2 && meshes.pred.userData.agentIds?.length > 0) {
        morphoCompareAgents = morphoCompareAgents.concat(
          meshes.pred.userData.agentIds.map((id) => ({ id, kind: 'pred' }))
        );
      }
      if (mineralOn) {
        handle.readMineral();
        let peak = 0;
        for (let i = 0; i < 1444; i++) if (v.mineral[i] > peak) peak = v.mineral[i];
        const gain = 1 / Math.max(peak, 0.03);
        for (let i = 0; i < 1444; i++) mineralNorm[i] = Math.min(1, v.mineral[i] * gain);
        updateMineralTexture(mineralTexture, mineralNorm);
      }
      if (histOn && histFrame % HIST_EVERY === 0) drawHist();
      histFrame += 1;
      // Phase 15: 每 60 sim-tick 拉一次 clade/event（内部有节奏门控）。
      if (phyloOverlay) pullPhylogeny();
      // Phase 17: 灾变拉取（60-tick 门控）+ 每帧烟尘浓度读数。
      if (catSupported) {
        pullCataclysms();
        handle.envMods(catBufPtr);
        dustOverlay.setDust(new DataView(instance.exports.memory.buffer, catBufPtr, 16).getFloat64(0, true));
      }
    }

    updateGhosts(clock.elapsedTime);
    if (catSupported) drawCataclysmMarks(); // overlay 自绘之后补画灾变标记
    renderer.render(scene, camera);

    if (elapsed - lastHud > 0.25) {
      lastHud = elapsed;
      fps = fpsN / Math.max(fpsAcc, 1e-6); fpsAcc = 0; fpsN = 0;
      const c = censusFromFlat(handle.readAgents());
      const hh = Math.floor(hourFor(handle.tick()));
      // Phase 9: DECO 效率演化读数（相对 warmup 基线的 delta）。
      const effMean = handle.decoEffMean();
      const effStd = handle.decoEffStd();
      const decoLine = effMean > 0
        ? `DECO eff: ${effMean.toFixed(2)} ± ${effStd.toFixed(2)}`
          + ` (${effMean >= decoBaselineMean ? '↑' : '↓'} ${(effMean - decoBaselineMean).toFixed(2)}`
          + ` / σ${effStd <= decoBaselineStd ? '↓' : '↑'} ${Math.abs((effStd - decoBaselineStd) / Math.max(decoBaselineStd, 1e-9) * 100).toFixed(0)}%)<br>`
        : '';
      // Phase 15: 物种谱系行 —— clade 名单（≤5 个直接列名，否则计数）+ 事件数。
      let cladeLine = '';
      if (phyloOverlay && phyloClades.length > 0) {
        const alive = phyloClades.filter((c) => !c.extinct).length;
        const names = phyloClades
          .map((c) => (c.extinct ? `†${c.name}` : c.name));
        const list = names.length <= 5 ? names.join(' · ') : `${alive} alive / ${names.length} total`;
        cladeLine = `<span style="color:#c9a0ff">clades: ${list} · ${phyloEvents.length} event${phyloEvents.length === 1 ? '' : 's'}</span><br>`;
      }
      // Phase 17: 活跃灾变徽标 + 最近 5 条历史。
      let catLine = '';
      if (catSupported && catHistory.length > 0) {
        handle.envMods(catBufPtr);
        const dust = new DataView(instance.exports.memory.buffer, catBufPtr, 16).getFloat64(0, true);
        catLine = formatCataclysmHud(catHistory, dust);
      }
      // Phase 13: 当前帧绝对读数（播放=最新帧；scrub=选中帧，红标）。
      let frameLine = '';
      if (specOverlay && specTotalPushed > 0) {
        const scrubbing = scrubber && !scrubber.isPlaying();
        const back = scrubbing ? Math.abs(scrubber.offset) : 0;
        const frameTick = specNewestTick - getTickFromFrame(back, HIST_FRAME_TICKS);
        frameLine = `frame: ${specTotalPushed - back} / ${specTotalPushed} (tick ${frameTick})`
          + (scrubbing ? ' <span style="color:#ff6b6b;font-weight:bold">[scrub]</span>' : '') + '<br>';
      }
      // Phase 10: 峰值桶位置 + 桶宽（与 overlay 联动，histogram 关闭时不占行）。
      const peakHue = DECO_HUE_MIN + ((histPeak + 0.5) / DECO_HUE_BUCKETS) * DECO_HUE_SPAN;
      const histLine = histOverlay && histOn && histTotal > 0
        ? `<span style="opacity:0.72">hue peak bucket ${histPeak + 1}/${DECO_HUE_BUCKETS} · hue≈${peakHue.toFixed(2)} · width ${(DECO_HUE_SPAN / DECO_HUE_BUCKETS).toFixed(4)} · t=${histTick}</span><br>`
        : '';
      // Phase 14: loaded 快照徽标（点击清除；HUD 本体 pointer-events:none，
      // 徽标 span 自带 pointer-events:auto 才能收到点击）与 S 键下载提示行。
      let loadedLine = '';
      if (sessionMeta.getState() === 'loaded') {
        const k = sessionMeta.getLoadedRows().length;
        loadedLine = `<span id="loaded-badge" style="pointer-events:auto;cursor:pointer;color:#8fd3ff;text-decoration:underline dotted">⏎ loaded: ${k}/${specOverlay.bufferSize} frames · 点击清除</span><br>`;
      }
      const pngLine = elapsed < pngReadyUntil
        ? `<span style="color:#8fd3ff">↓ PNG ready (t=${pngReadyTick})</span><br>`
        : '';
      const agentTotal = handle.agentCount();
      const speciesTotal = c[0]+c[1]+c[2]+c[3]+c[4]+c[5];
      const countLine = agentTotal > 0
        ? `agent count=${agentTotal} ${speciesTotal===0 ? '· ⚠ flat=0' : ''}<br>`
        : '';
      hud.innerHTML =
        `<b>WASM SNAPSHOT v3</b> · tick ${handle.tick()} · D+${hh}h<br>` +
        `牧 ${c[0]} · 掠 ${c[1]} · 腐 ${c[2]} · 杂 ${c[3]} · 鱼 ${c[4]} · 分解 ${c[5]}<br>` +
        countLine +
        decoLine +
        cladeLine +
        catLine +
        frameLine +
        loadedLine +
        histLine +
        `mineral peak ${handle.mineralPeak().toFixed(3)} · M 热力层${mineralOn ? ' ON' : ' OFF'}<br>` +
        `${fps.toFixed(0)} fps · mem ${(handle.getBufferLen()/1024).toFixed(0)}KB · ${paused ? 'PAUSED' : `${TICKS_PER_FRAME} ticks/frame`}` +
        pngLine;
    }
  }
  tick();
  return { handle, meshes, mineralPlane };
}

// Auto-boot when the canvas is present (module-level fire-and-forget).
if (typeof document !== 'undefined' && document.getElementById('stage')) {
  bootWasmViewer('stage').catch((e) => {
    console.error('[wasm-viewer] boot failed:', e);
    const el = document.getElementById('load-status');
    if (el) { el.textContent = '加载失败 · ' + e.message; el.style.color = '#e8a07a'; }
  });
}
