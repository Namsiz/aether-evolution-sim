// test_gene_visualization.mjs — Phase 5 视觉基因化验收。
//
// 端到端验证 Rust 预计算的 genome→视觉映射（wasm.rs visual_fields）：
//   flat 路径：render_scale ∈ [0.6, 1.4] 且与原始 size 基因逐 agent 一致；
//             ARGB alpha = 0xFF；id 严格递增；lineage flag ∈ {0,1}；
//             种群内尺度差异可辨（max−min > 0.05）。
//   JSON 路径：kind 级 hue 带（fish 蓝→青→绿 / pred 红褐 / deco 紫 / herb 金 /
//             omn 绿 / scav 棕）；GenomeHueVisual 可由 JS 侧公式逐 agent 复算；
//             FinColorVisual 仅 FISH 携带。
//
// Usage: node living/test_gene_visualization.mjs [--wasm path] [--warmup 300]

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseSnapshot } from './src/renderer-wasm-viewer.mjs';

// Scratch layout — mirrored from wasm.rs / src/wasm-viewer.mjs（保持同步）。
const AGENT_CAP = 1024;
const DAYLIGHT_CAP = 64;
const AGENT_BASE = 1444 * 4;
const SLOT_KIND = 8, SLOT_SIZE = 20,
      SLOT_COLOR = 28, SLOT_SCALE = 32, SLOT_FLAGS = 36, SLOT_ID = 40;
const AGENT_SLOT_SIZE = 48;
const FLAG_LINEAGE = 1;
const FLAG_DECO_EFF = 2;
const OFF_DAYLIGHT = AGENT_BASE + AGENT_CAP * AGENT_SLOT_SIZE;
const KIND_BYTE = { 104: 'herb', 112: 'pred', 115: 'scav', 111: 'omn', 102: 'fish', 100: 'deco' };

// Phase 5 映射常量（与 wasm.rs 一致；JS 侧重算用于逐 agent 交叉验证）。
const SIZE_GENE_MIN = 0.78, SIZE_GENE_MAX = 1.28;
const VIS_SCALE_MIN = 0.6, VIS_SCALE_MAX = 1.4;
const HUE_SPREAD = 0.30;
const HUE_ANCHOR = { herb: 0.11, pred: 0.04, scav: 0.08, omn: 0.30, fish: 0.56, deco: 0.75 };
const HUE_SHIFT = { herb: 0.0, pred: -0.02, scav: -0.04, omn: 0.04, fish: 0.0, deco: 0.06 };
const wrap01 = (h) => ((h % 1) + 1) % 1;
const expectedScale = (size) =>
  VIS_SCALE_MIN +
  Math.min(1, Math.max(0, (size - SIZE_GENE_MIN) / (SIZE_GENE_MAX - SIZE_GENE_MIN))) *
  (VIS_SCALE_MAX - VIS_SCALE_MIN);
const expectedHue = (kind, hue, fin) =>
  wrap01(HUE_ANCHOR[kind] + (hue - 0.5) * HUE_SPREAD + (kind === 'fish' ? fin * 0.3 : HUE_SHIFT[kind]));

// kind 级 hue 允许带（含 u16 量化余量）。派生自 anchor + (hue−0.5)×0.30 + shift。
const HUE_BANDS = {
  herb: [[0.959, 1.001], [0.0, 0.261]],
  pred: [[0.869, 1.001], [0.0, 0.171]],
  scav: [[0.889, 1.001], [0.0, 0.191]],
  omn:  [[0.189, 0.491]],
  fish: [[0.409, 1.001], [0.0, 0.011]],
  deco: [[0.629, 0.871]], // Phase 9: visual_hue_deco → [0.75-0.12, 0.75+0.12] = [0.63, 0.87]
};
const inBand = (h, bands) => bands.some(([lo, hi]) => h >= lo && h <= hi);

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const wasmArg = args.indexOf('--wasm');
const warmupArg = args.indexOf('--warmup');
const wasmPath = wasmArg >= 0 ? args[wasmArg + 1] : path.join(__dirname, 'ecology-sim.wasm');
const WARMUP = warmupArg >= 0 ? Number(args[warmupArg + 1]) : 300;

let failures = 0;
const check = (name, cond, detail = '') => {
  if (cond) console.log(`  ok   ${name}${detail ? ` — ${detail}` : ''}`);
  else { console.error(`  FAIL ${name}${detail ? ` — ${detail}` : ''}`); failures += 1; }
};

const bytes = fs.readFileSync(wasmPath);
const mod = await WebAssembly.compile(bytes);
const instance = await WebAssembly.instantiate(mod, {});
const E = instance.exports;
const sim = E.sim_new(20261102);
const scratch = E.sim_scratch(sim);

console.log(`wasm ${path.basename(wasmPath)} (${(bytes.length / 1024).toFixed(0)} KB):`);
check('scratch size covers 48-byte agent slots', E.sim_scratch_size() >= OFF_DAYLIGHT + 48,
  `${E.sim_scratch_size()} bytes`);

// Warmup（与 viewer/test_wasm_viewer 相同的正弦昼光曲线）。
for (let done = 0; done < WARMUP; done += DAYLIGHT_CAP) {
  const n = Math.min(DAYLIGHT_CAP, WARMUP - done);
  const dl = new Float64Array(E.memory.buffer, scratch + OFF_DAYLIGHT, DAYLIGHT_CAP);
  for (let i = 0; i < n; i++) {
    const hour = (((done + i) * 0.1 * 24) / 28800) % 24;
    const elev = Math.sin(((hour - 6) / 12) * Math.PI);
    dl[i] = Math.min(1, Math.max(0, (elev + 0.03) / 0.27));
  }
  E.sim_step(sim, n, scratch + OFF_DAYLIGHT, n);
}
const n = E.sim_agents_flat(sim, AGENT_CAP);
check(`warmup ${WARMUP} ticks with live agents`, n > 0, `${n} agents @ tick ${E.sim_tick(sim)}`);

// ─── flat 路径：逐 agent 交叉验证 scale 映射与颜色打包 ──────────────────────────
const dv = new DataView(E.memory.buffer, scratch + AGENT_BASE, n * AGENT_SLOT_SIZE);
let scaleMatchesFormula = true, alphaOk = true, flagsOk = true, idsAscending = true;
let scaleMin = Infinity, scaleMax = -Infinity;
let prevId = 0;
for (let i = 0; i < n; i++) {
  const base = i * AGENT_SLOT_SIZE;
  const size = dv.getFloat32(base + SLOT_SIZE, true);
  const scale = dv.getFloat32(base + SLOT_SCALE, true);
  const argb = dv.getUint32(base + SLOT_COLOR, true);
  const id = dv.getUint32(base + SLOT_ID, true);
  if (Math.abs(scale - expectedScale(size)) > 1e-6) scaleMatchesFormula = false;
  if (((argb >>> 24) & 0xff) !== 0xff) alphaOk = false;
  if ((dv.getUint32(base + SLOT_FLAGS, true) & ~(FLAG_LINEAGE | FLAG_DECO_EFF)) !== 0) flagsOk = false;
  if (id <= prevId) idsAscending = false;
  prevId = id;
  if (scale < scaleMin) scaleMin = scale;
  if (scale > scaleMax) scaleMax = scale;
}
check('flat render_scale == 0.6 + clamp((size−0.78)/0.50) × 0.8 per agent', scaleMatchesFormula);
check('flat ARGB alpha byte is 0xFF', alphaOk);
check('flat flags only use known bits', flagsOk);
check('agent ids strictly ascending', idsAscending);
check('body-size diversity visible in one frame', scaleMax - scaleMin > 0.05,
  `scale ${scaleMin.toFixed(3)}..${scaleMax.toFixed(3)}`);

// ─── JSON 路径：kind 级 hue 带 + 公式复算 + fin 基因契约 ────────────────────────
const lenSlot = scratch + OFF_DAYLIGHT + 24;
const jp = E.sim_snapshot_json(sim, lenSlot);
const jsonLen = new Uint32Array(E.memory.buffer, lenSlot, 1)[0] - 1;
const json = new TextDecoder().decode(new Uint8Array(E.memory.buffer, jp, jsonLen));
let snap = null;
try { snap = parseSnapshot(json, 3); check('sim_snapshot_json parses as v3', true, `${(jsonLen / 1024).toFixed(0)} KB`); }
catch (e) { check('sim_snapshot_json parses as v3', false, e.message); }

if (snap) {
  const perKind = {};
  let hueFormulaOk = true, hueBandOk = true, scaleRangeOk = true;
  for (const a of snap.Agents) {
    perKind[a.Kind] = (perKind[a.Kind] || 0) + 1;
    const hue = a.GenomeHueVisual / 65535;
    const fin = a.FinColorVisual ?? 0;
    // Phase 9: DECO uses visual_hue_deco (efficiency-driven), not Phase 5 formula.
    const isDeco = a.Kind === 'deco';
    if (!isDeco && Math.abs(hue - expectedHue(a.Kind, a.GenomeHue, fin)) > 1e-4) hueFormulaOk = false;
    // Phase 9 DECO 专属公式：hue = wrap01(0.75 + (decoEfficiency-1)*0.24)
    if (isDeco) {
      const eff = a.DecoEffVisual != null ? a.DecoEffVisual / 65535 * 1.0 + 0.5 : 1.0;
      const shift = Math.min(0.12, Math.max(-0.12, (eff - 1.0) * 0.24));
      const phase9Hue = ((0.75 + shift) % 1 + 1) % 1;
      if (Math.abs(hue - phase9Hue) > 1e-4) hueFormulaOk = false;
    }
    if (!inBand(hue, HUE_BANDS[a.Kind] || [[0, 1]])) hueBandOk = false;
    if (a.GenomeSizeVisual < VIS_SCALE_MIN - 1e-4 || a.GenomeSizeVisual > VIS_SCALE_MAX + 1e-4) scaleRangeOk = false;
  }
  check('every kind present at warmup end', Object.keys(perKind).length === 6, JSON.stringify(perKind));
  check('GenomeSizeVisual within [0.6, 1.4] for all agents', scaleRangeOk);
  check('JS recompute of hue formula matches GenomeHueVisual (u16)', hueFormulaOk);
  check('per-kind hue bands: fish 蓝→绿 / pred 红 / deco 紫 / herb 金 / omn 绿 / scav 棕', hueBandOk);

  // fish fin 基因表达：种群内色相跨度可观（fin ∈ [0,1] → +0.3 hue 漂移）。
  const fishHues = snap.Agents.filter(a => a.Kind === 'fish').map(a => a.GenomeHueVisual / 65535);
  const fishSpread = Math.max(...fishHues) - Math.min(...fishHues);
  check('fish hue spread visible (fin gene expression)', fishHues.length > 0 && fishSpread > 0.05,
    `spread ${fishSpread.toFixed(3)} over ${fishHues.length} fish`);

  const fish = snap.Agents.filter(a => a.Kind === 'fish');
  const finOk = fish.every(a => a.FinColorVisual !== null) &&
    snap.Agents.filter(a => a.Kind !== 'fish').every(a => a.FinColorVisual === null);
  check('FinColorVisual only on FISH', finOk);
}

E.sim_free_json(jp, jsonLen + 1);
E.sim_free(sim);

if (failures > 0) {
  console.error(`\nGENE-VIS FAILED: ${failures} check(s) failed`);
  process.exit(1);
}
console.log('\nGENE VISUALIZATION OK');
