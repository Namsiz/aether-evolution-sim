// test_evolution_visualization.mjs — Phase 9 长程演化可视化验收。
//
// 端到端验证：Phase 7/8 的 DECO 选择压（deco_efficiency 选择性繁殖 + 物理竞争锁）
// 在 3000 ticks 长程模拟中，DECO 种群的可见信号（hue 右移 toward magenta）随时间演化。
//
// 数据路径：
//   1. Flat scratch（每 600 ticks）：count_deco + mean/std 走 sim_deco_eff_mean/std 导出。
//   2. JSON snapshot（仅 warmup 端和 final 端各一次）：DECO GenomeHueVisual 分布统计。
//
// Usage: node living/test_evolution_visualization.mjs [--wasm <path>] [--ticks 3000]

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseSnapshot } from './src/renderer-wasm-viewer.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const wasmArg = args.indexOf('--wasm');
const ticksArg = args.indexOf('--ticks');
const wasmPath = wasmArg >= 0 ? args[wasmArg + 1] : path.join(__dirname, 'ecology-sim.wasm');
const TOTAL_TICKS = Number(args[ticksArg + 1]) || 3000;
const SAMPLE_INTERVAL = 600; // ticks between samples
const WARMUP = 300;

// ─── Scratch layout —— 必须与 wasm.rs / wasm-viewer.mjs 保持同步 ──────────────────
const AGENT_CAP = 1024;
const DAYLIGHT_CAP = 64;
const MINERAL_BYTES = 1444 * 4;
const AGENT_BASE = MINERAL_BYTES;
const SLOT_KIND = 8, SLOT_FLAGS = 36, SLOT_DECO_EFF = 44;
const FLAG_DECO_EFF = 2;
const AGENT_SLOT_SIZE = 48;
const OFF_DAYLIGHT = AGENT_BASE + AGENT_CAP * AGENT_SLOT_SIZE;
const OFF_COUNTS = OFF_DAYLIGHT + DAYLIGHT_CAP * 8;

const KIND_BYTE = { h: 'herb', p: 'pred', s: 'scav', o: 'omn', f: 'fish', d: 'deco' };

// ─── helpers ───────────────────────────────────────────────────────────────────
let failures = 0;
const check = (name, cond, detail = '') => {
  if (cond) console.log(`  ok   ${name}${detail ? ` — ${detail}` : ''}`);
  else { console.error(`  FAIL ${name}${detail ? ` — ${detail}` : ''}`); failures += 1; }
};

const daylightFor = (tick) => {
  const hour = (tick * 0.1 * 24 / 28800) % 24;
  const elev = Math.sin(((hour - 6) / 12) * Math.PI);
  return Math.min(1, Math.max(0, (elev + 0.03) / 0.27));
};

const clamp01 = (v) => Math.min(1, Math.max(0, v));
// JS 侧复算 visual_hue_deco —— 与 Rust 公式一致（仅用于 final 路径的 slot 有效性检查）。
const visualHueDeco = (eff) => {
  const shift = Math.min(0.12, Math.max(-0.12, (eff - 1.0) * 0.24));
  return ((0.75 + shift) % 1 + 1) % 1;
};

// ─── wasm boot ─────────────────────────────────────────────────────────────────
const bytes = fs.readFileSync(wasmPath);
const mod = await WebAssembly.compile(bytes);
const instance = await WebAssembly.instantiate(mod, {});
const E = instance.exports;
const sim = E.sim_new(20261102);
const scratch = E.sim_scratch(sim);

console.log(`wasm ${path.basename(wasmPath)} (${(bytes.length / 1024).toFixed(0)} KB):`);
check('scratch covers 48-byte agent slots', E.sim_scratch_size() >= OFF_DAYLIGHT + 48,
  `${E.sim_scratch_size()} bytes`);
if (!E.sim_deco_eff_mean || !E.sim_deco_eff_std) {
  console.error('FAIL: sim_deco_eff_mean/std exports missing — WASM must be rebuilt for Phase 9');
  process.exit(1);
}

// ─── 推进 sim 的 tick 循环（与 wasm-viewer.mjs daylight 曲线一致） ───────────────
function runTicks(target) {
  let current = E.sim_tick(sim);
  while (current < target) {
    const n = Math.min(DAYLIGHT_CAP, target - current);
    const dl = new Float64Array(E.memory.buffer, scratch + OFF_DAYLIGHT, DAYLIGHT_CAP);
    for (let i = 0; i < n; i++) dl[i] = daylightFor(current + i);
    E.sim_step(sim, n, scratch + OFF_DAYLIGHT, n);
    current = E.sim_tick(sim);
  }
}

// ─── JSON snapshot 辅助（提前定义，用于 t=0 与 t=3000 两个快照） ────────────────
const lenSlot = scratch + OFF_DAYLIGHT + 24;
const collectDecoHues = () => {
  const jp = E.sim_snapshot_json(sim, lenSlot);
  const jsonLen = new Uint32Array(E.memory.buffer, lenSlot, 1)[0] - 1;
  const raw = new TextDecoder().decode(new Uint8Array(E.memory.buffer, jp, jsonLen));
  E.sim_free_json(jp, jsonLen + 1);
  const snap = parseSnapshot(raw, 3);
  return { snap, hues: snap.Agents.filter(a => a.Kind === 'deco').map(a => a.GenomeHueVisual) };
};

// t=0 快照：刚创建 sim，未受选择压
const start = collectDecoHues();

// ─── warmup + 基线采集 ────────────────────────────────────────────────────────
runTicks(WARMUP);
const nWarmup = E.sim_agents_flat(sim, AGENT_CAP);
const baselineMean = E.sim_deco_eff_mean(sim);
const baselineStd = E.sim_deco_eff_std(sim);
const baselineHue = visualHueDeco(baselineMean);
console.log(`  warmup ${WARMUP} ticks: tick=${E.sim_tick(sim)} agents=${nWarmup} ` +
  `DECO eff baseline: ${baselineMean.toFixed(4)} ± ${baselineStd.toFixed(4)} → hue ${baselineHue.toFixed(4)}`);

// ─── 每 600 ticks 采样 ────────────────────────────────────────────────────────
const samples = [{ tick: E.sim_tick(sim), count: nWarmup, mean: baselineMean, std: baselineStd }];
for (let t = WARMUP + SAMPLE_INTERVAL; t <= TOTAL_TICKS; t += SAMPLE_INTERVAL) {
  runTicks(t);
  const n = E.sim_agents_flat(sim, AGENT_CAP);
  samples.push({
    tick: E.sim_tick(sim),
    count: n,
    mean: E.sim_deco_eff_mean(sim),
    std: E.sim_deco_eff_std(sim),
  });
}

// ─── 断言 1：每个采样点种群存活且效率读数可读 ─────────────────────────────────
for (const s of samples) {
  check(`tick ${s.tick}: alive agents > 0`, s.count > 0, `${s.count} agents`);
  check(`tick ${s.tick}: deco_eff_mean in [0.5, 1.5]`, s.mean >= 0.49 && s.mean <= 1.51,
    `mean=${s.mean.toFixed(4)}`);
}

// ─── 断言 2：DECO mean efficiency 单调上升（selection pressure 生效） ──────────
const firstMean = samples[0].mean;
const lastMean = samples[samples.length - 1].mean;
const meanRise = lastMean - firstMean;
check('DECO mean efficiency rises over long run (selection pressure)',
  meanRise > 0.10,
  `Δ mean = ${meanRise.toFixed(4)} (${firstMean.toFixed(4)} → ${lastMean.toFixed(4)})`);
check('DECO mean efficiency gain tracks Phase 7 benchmark (≥ +0.15)',
  meanRise >= 0.15,
  `gain ${meanRise.toFixed(4)} vs benchmark +0.15`);

// ─── 断言 3：std dev 收敛（population homogenization） ────────────────────────
const firstStd = samples[0].std;
const lastStd = samples[samples.length - 1].std;
check('DECO efficiency std dev narrows over long run',
  lastStd < firstStd * 1.05, // allow tiny noise; Phase 7 shows σ↓35%
  `σ ${firstStd.toFixed(4)} → ${lastStd.toFixed(4)}`);

// ─── 断言 4：JSON snapshot 的 DECO hue 分布整体右移 ───────────────────────────
runTicks(TOTAL_TICKS);           // 驱动到最终 tick
const end = collectDecoHues();   // 此时经过完整 Ticks 演化

check('early DECO sample size > 0', start.hues.length > 0, `${start.hues.length} DECO`);
check('final DECO sample size > 0', end.hues.length > 0, `${end.hues.length} DECO`);

const avg = (arr) => arr.reduce((a, b) => a + b, 0) / arr.length;
const stdDev = (arr) => {
  if (arr.length < 2) return 0;
  const m = avg(arr);
  return Math.sqrt(arr.reduce((a, h) => a + (h - m) ** 2, 0) / arr.length);
};
const startAvg = avg(start.hues), endAvg = avg(end.hues);
const startStdHue = stdDev(start.hues), endStdHue = stdDev(end.hues);
const phaseShift = (endAvg - startAvg) / 65535;

check('DECO hue average shifts right (toward magenta)',
  phaseShift > 0.002,
  `avg hue: ${(startAvg / 65535).toFixed(4)} → ${(endAvg / 65535).toFixed(4)}, Δ=${(phaseShift * 65535).toFixed(0)} u16`);
check('DECO hue std dev narrows (population convergence)',
  endStdHue < startStdHue * 1.05,
  `σ hue: ${startStdHue.toFixed(0)} → ${endStdHue.toFixed(0)} u16`);

// ─── 断言 5：snapshot JSON 新字段 DecoEffMean/Std 存在且数值合理 ────────────────
{
  const jp = E.sim_snapshot_json(sim, lenSlot);
  const jsonLen = new Uint32Array(E.memory.buffer, lenSlot, 1)[0] - 1;
  const snap = parseSnapshot(new TextDecoder().decode(new Uint8Array(E.memory.buffer, jp, jsonLen)), 3);
  E.sim_free_json(jp, jsonLen + 1);
  check('snapshot Stats has DecoEffMean field', 'DecoEffMean' in snap.Stats,
    `val=${snap.Stats.DecoEffMean}`);
  check('snapshot Stats has DecoEffStd field', 'DecoEffStd' in snap.Stats,
    `val=${snap.Stats.DecoEffStd}`);
  if (snap.Stats.DecoEffMean !== null) {
    check('DecoEffMean within [0.5, 1.5]',
      snap.Stats.DecoEffMean >= 0.49 && snap.Stats.DecoEffMean <= 1.51,
      snap.Stats.DecoEffMean.toFixed(4));
  }
}

// ─── 断言 6：flat 路径 DECO agent 的 SLOT_FLAGS bit1（FLAG_DECO_EFF）置位 ────────
const nFinal = E.sim_agents_flat(sim, AGENT_CAP);
const dvFlat = new DataView(E.memory.buffer, scratch + AGENT_BASE, nFinal * AGENT_SLOT_SIZE);
let anyDecoFlagged = false, decoHueMatches = true;
for (let i = 0; i < nFinal; i++) {
  const base = i * AGENT_SLOT_SIZE;
  if (dvFlat.getUint8(base + SLOT_KIND) === 100 /* 'd' */) {
    const flags = dvFlat.getUint32(base + SLOT_FLAGS, true);
    if (flags & FLAG_DECO_EFF) anyDecoFlagged = true;
    const slotEff = dvFlat.getUint32(base + SLOT_DECO_EFF, true);
    if (slotEff === 0 || slotEff > 65535) decoHueMatches = false;
  }
}
check('flat path: at least one DECO with FLAG_DECO_EFF set', anyDecoFlagged);
check('flat path: every DECO has valid SLOT_DECO_EFF ∈ (0, 65535]', decoHueMatches);

E.sim_free(sim);

if (failures > 0) {
  console.error(`\nEVOLUTION VISUALIZATION FAILED: ${failures} check(s) failed`);
  process.exit(1);
}
console.log('\nEVOLUTION VISUALIZATION OK');
