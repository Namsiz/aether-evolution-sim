// test_wasm_viewer.mjs — smoke test for the WASM viewer data path.
//
// Exercises the exact call sequence the browser viewer uses, without a DOM:
//   1. load living/ecology-sim.wasm (zero imports, bare C-ABI exports)
//   2. WasmSim warmup (300 ticks, sliding daylight window)
//   3. per-frame flat export: sim_step → sim_agents_flat → sim_counts
//      → sim_mineral_resampled
//   4. one-shot sim_snapshot_json → parseSnapshot (v3 contract check)
//
// Usage: node living/test_wasm_viewer.mjs [--wasm path] [--warmup 300]

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseSnapshot } from './src/renderer-wasm-viewer.mjs';

// Scratch layout — mirrored from wasm.rs / src/wasm-viewer.mjs.
const AGENT_CAP = 1024;
const DAYLIGHT_CAP = 64;
const AGENT_BASE = 1444 * 4;
const SLOT_X = 0, SLOT_Z = 4, SLOT_KIND = 8, SLOT_ENERGY = 12,
      SLOT_HUNGER = 16, SLOT_SIZE = 20, SLOT_HUE = 24,
      SLOT_COLOR = 28, SLOT_SCALE = 32, SLOT_FLAGS = 36, SLOT_ID = 40;
const AGENT_SLOT_SIZE = 48;
const FLAG_LINEAGE = 1;
const FLAG_DECO_EFF = 2;
const OFF_DAYLIGHT = AGENT_BASE + AGENT_CAP * AGENT_SLOT_SIZE;
const OFF_COUNTS = OFF_DAYLIGHT + DAYLIGHT_CAP * 8;
const KIND_BYTE = { 104: 'herb', 112: 'pred', 115: 'scav', 111: 'omn', 102: 'fish', 100: 'deco' };

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const wasmArg = args.indexOf('--wasm');
const warmupArg = args.indexOf('--warmup');
const wasmPath = wasmArg >= 0 ? args[wasmArg + 1] : path.join(__dirname, 'ecology-sim.wasm');
const WARMUP = warmupArg >= 0 ? Number(args[warmupArg + 1]) : 300;
const TICKS_PER_FRAME = 2;

let failures = 0;
const check = (name, cond, detail = '') => {
  if (cond) console.log(`  ok   ${name}${detail ? ` — ${detail}` : ''}`);
  else { console.error(`  FAIL ${name}${detail ? ` — ${detail}` : ''}`); failures += 1; }
};

const bytes = fs.readFileSync(wasmPath);
const mod = await WebAssembly.compile(bytes);
// Note: instantiate(Module, imports) resolves to the Instance itself, not {module, instance}.
const instance = await WebAssembly.instantiate(mod, {});
const E = instance.exports;
check('zero imports (bare C-ABI module)', WebAssembly.Module.imports(mod).length === 0,
  `${WebAssembly.Module.imports(mod).length} imports`);
const sim = E.sim_new(20261102);
const scratch = E.sim_scratch(sim);

console.log(`wasm ${path.basename(wasmPath)} (${(bytes.length / 1024).toFixed(0)} KB):`);
for (const name of ['sim_new', 'sim_free', 'sim_step', 'sim_agents_flat', 'sim_counts',
  'sim_mineral_resampled', 'sim_mineral_peak', 'sim_tick', 'sim_season', 'sim_daylight',
  'sim_snapshot_json', 'sim_free_json']) {
  check(`export sim_*:${name}`, typeof E[name] === 'function');
}
check('scratch size matches layout', E.sim_scratch_size() >= OFF_COUNTS + 24,
  `${E.sim_scratch_size()} bytes`);

// Warmup with the same sinusoidal daylight the viewer feeds per frame.
const t0 = performance.now();
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
check(`warmup ${WARMUP} ticks`, E.sim_tick(sim) === WARMUP,
  `${(performance.now() - t0).toFixed(0)} ms`);

// Per-frame path × 60 frames (what the rAF loop does).
const t1 = performance.now();
let lastCount = 0;
for (let f = 0; f < 60; f++) {
  const dl = new Float64Array(E.memory.buffer, scratch + OFF_DAYLIGHT, DAYLIGHT_CAP);
  for (let i = 0; i < TICKS_PER_FRAME; i++) {
    const tick = E.sim_tick(sim) + i;
    const hour = ((tick * 0.1 * 24) / 28800) % 24;
    const elev = Math.sin(((hour - 6) / 12) * Math.PI);
    dl[i] = Math.min(1, Math.max(0, (elev + 0.03) / 0.27));
  }
  E.sim_step(sim, TICKS_PER_FRAME, scratch + OFF_DAYLIGHT, TICKS_PER_FRAME);
  lastCount = E.sim_agents_flat(sim, AGENT_CAP);
  E.sim_counts(sim);
  E.sim_mineral_resampled(sim);
}
const frameMs = (performance.now() - t1) / 60;
check('60-frame sim+export loop under 16 ms/frame budget', frameMs < 16,
  `${frameMs.toFixed(2)} ms/frame avg`);

// Agent census from the flat buffer (stride = AGENT_SLOT_SIZE).
const kinds = new Uint8Array(E.memory.buffer, scratch + AGENT_BASE + SLOT_KIND, lastCount * AGENT_SLOT_SIZE);
const xs = new Float32Array(E.memory.buffer, scratch + AGENT_BASE + SLOT_X, lastCount * AGENT_SLOT_SIZE / 4);
const census = {};
let coordsSane = true;
for (let i = 0; i < lastCount; i++) {
  const k = KIND_BYTE[kinds[i * AGENT_SLOT_SIZE]];
  census[k] = (census[k] || 0) + 1;
  const x = xs[i * AGENT_SLOT_SIZE / 4];
  if (!Number.isFinite(x) || Math.abs(x) > 2000) coordsSane = false;
}
check('flat census sums to agent count', Object.values(census).reduce((a, b) => a + b, 0) === lastCount,
  JSON.stringify(census));
check('agent X coords finite and within world span', coordsSane);

// Phase 5 visual-genomics：flat 路径颜色/缩放/标志位/id 不变量。
const agentDv = new DataView(E.memory.buffer, scratch + AGENT_BASE, lastCount * AGENT_SLOT_SIZE);
let visSane = true, idsAscending = true, prevId = 0;
let scaleMin = Infinity, scaleMax = -Infinity;
for (let i = 0; i < lastCount; i++) {
  const base = i * AGENT_SLOT_SIZE;
  const scale = agentDv.getFloat32(base + SLOT_SCALE, true);
  const argb = agentDv.getUint32(base + SLOT_COLOR, true);
  const id = agentDv.getUint32(base + SLOT_ID, true);
  if (!Number.isFinite(scale) || scale < 0.6 - 1e-4 || scale > 1.4 + 1e-4) visSane = false;
  if (((argb >>> 24) & 0xff) !== 0xff) visSane = false;
  if ((agentDv.getUint32(base + SLOT_FLAGS, true) & ~(FLAG_LINEAGE | FLAG_DECO_EFF)) !== 0) visSane = false;
  if (id <= prevId) idsAscending = false;
  prevId = id;
  if (scale < scaleMin) scaleMin = scale;
  if (scale > scaleMax) scaleMax = scale;
}
check('per-agent render scale ∈ [0.6, 1.4] with ARGB alpha 0xFF', visSane,
  `scale ${scaleMin.toFixed(2)}..${scaleMax.toFixed(2)}`);
check('agent ids strictly ascending (stable death tracking)', idsAscending);

const counts = new Uint32Array(E.memory.buffer, scratch + OFF_COUNTS, 6);
check('sim_counts agrees with flat census',
  counts[0] === (census.herb || 0) && counts[1] === (census.pred || 0) && counts[5] === (census.deco || 0),
  `h${counts[0]} p${counts[1]} s${counts[2]} o${counts[3]} f${counts[4]} d${counts[5]}`);

// Mineral field: 38×38, finite, peak matches sim_mineral_peak.
const mineral = new Float32Array(E.memory.buffer, scratch, 1444);
check('mineral resampled length 1444', mineral.length === 1444);
let mPeak = 0;
for (const v of mineral) { if (!Number.isFinite(v)) { mPeak = -1; break; } if (v > mPeak) mPeak = v; }
check('mineral values finite', mPeak >= 0);
check('mineral resampled peak ≤ full-res peak',
  mPeak <= E.sim_mineral_peak(sim) + 1e-6,
  `resampled ${mPeak.toFixed(4)} vs full ${E.sim_mineral_peak(sim).toFixed(4)}`);

// One-shot v3 JSON snapshot → parseSnapshot contract check.
const lenSlot = scratch + OFF_COUNTS + 24; // spare 4 bytes past the 6 u32 counts
const jp = E.sim_snapshot_json(sim, lenSlot);
const jsonLen = new Uint32Array(E.memory.buffer, lenSlot, 1)[0] - 1;
const json = new TextDecoder().decode(new Uint8Array(E.memory.buffer, jp, jsonLen));
let snap;
try { snap = parseSnapshot(json, 3); check('sim_snapshot_json parses as v3', true, `${(jsonLen / 1024).toFixed(0)} KB`); }
catch (e) { check('sim_snapshot_json parses as v3', false, e.message); }
if (snap) {
  check('JSON agent count matches flat count', snap.Agents.length === lastCount,
    `${snap.Agents.length} vs ${lastCount}`);
  check('JSON mineral resampled length 1444', snap.World.MineralFieldResampled.length === 1444);
  // Phase 5：JSON 路径新增视觉字段的契约。
  const visOk = snap.Agents.every(a =>
    Number.isFinite(a.GenomeSizeVisual) && a.GenomeSizeVisual >= 0.6 - 1e-4 && a.GenomeSizeVisual <= 1.4 + 1e-4 &&
    Number.isInteger(a.GenomeHueVisual) && a.GenomeHueVisual >= 0 && a.GenomeHueVisual <= 65535 &&
    Number.isFinite(a.GenomeColorHex) && a.GenomeColorHex >= 0);
  check('JSON GenomeSizeVisual/HueVisual/ColorHex contract', visOk);
  const fish = snap.Agents.filter(a => a.Kind === 'fish');
  const finOk = fish.length > 0 &&
    fish.every(a => a.FinColorVisual !== null) &&
    snap.Agents.filter(a => a.Kind !== 'fish').every(a => a.FinColorVisual === null);
  check('FISH carries FinColorVisual, other kinds null', finOk, `${fish.length} fish`);
}
E.sim_free_json(jp, jsonLen + 1);
E.sim_free(sim);

if (failures > 0) {
  console.error(`\nSMOKE FAILED: ${failures} check(s) failed`);
  process.exit(1);
}
console.log('\nWASM VIEWER SMOKE OK');
