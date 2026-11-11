// test_evolution_histogram.mjs — Phase 10 DECO hue 直方图数据链路验收。
//
// 端到端验证 sim_deco_hue_histogram 导出与 Phase 9 演化叙事的一致性：
//   1. 每个采样点：counts 全部为非负整数，且 sum(counts) === 存活 DECO 数
//      （直方图不丢个体、不重复计数 —— 与 sim_counts 交叉对账）。
//   2. 3000 ticks 后加权均值桶位右移（选择 → 分布向品红漂移）。
//   3. 3000 ticks 后加权标准差收窄（选择 → 分布变窄）。
//   4. 与 Phase 9 的 mean_efficiency 读数互证：eff→hue→bucket 的期望桶位与
//      直方图观测桶位一致（同一迁移，两条独立路径）。
//
// Usage: node living/test_evolution_histogram.mjs [--wasm <path>] [--ticks 3000]

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const wasmArg = args.indexOf('--wasm');
const ticksArg = args.indexOf('--ticks');
const wasmPath = wasmArg >= 0 ? args[wasmArg + 1] : path.join(__dirname, 'ecology-sim.wasm');
const TOTAL_TICKS = Number(args[ticksArg + 1]) || 3000;
const SAMPLE_INTERVAL = 600;
const WARMUP = 300;
const BUCKETS = 24;
const DECO_HUE_MIN = 0.629;
const DECO_HUE_SPAN = 0.242;

// ─── Scratch layout —— 必须与 wasm.rs SCRATCH_* 保持同步 ──────────────────────────
const AGENT_CAP = 1024;
const DAYLIGHT_CAP = 64;
const MINERAL_BYTES = 1444 * 4;
const AGENT_BASE = MINERAL_BYTES;
const OFF_DAYLIGHT = AGENT_BASE + AGENT_CAP * 48;
const OFF_COUNTS = OFF_DAYLIGHT + DAYLIGHT_CAP * 8;
const OFF_HIST = OFF_COUNTS + 6 * 4;

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

// eff → hue（与 wasm.rs visual_hue_deco 同式）→ 期望桶位（连续值，非 floor）。
function hueOfEff(eff) {
  const shift = Math.min(0.12, Math.max(-0.12, (eff - 1.0) * 0.24));
  return ((0.75 + shift) % 1 + 1) % 1;
}
const bucketOfHue = (hue) => (hue - DECO_HUE_MIN) / (DECO_HUE_SPAN / BUCKETS);

// 直方图加权统计：均值桶位与标准差（桶为单位，0-based）。
function histStats(counts) {
  const total = counts.reduce((a, b) => a + b, 0);
  if (total === 0) return null;
  const mean = counts.reduce((a, c, i) => a + c * i, 0) / total;
  const variance = counts.reduce((a, c, i) => a + c * (i - mean) ** 2, 0) / total;
  return { total, mean, std: Math.sqrt(variance) };
}

// ─── wasm boot ─────────────────────────────────────────────────────────────────
const bytes = fs.readFileSync(wasmPath);
const mod = await WebAssembly.compile(bytes);
const instance = await WebAssembly.instantiate(mod, {});
const E = instance.exports;
const sim = E.sim_new(20261102);
const scratch = E.sim_scratch(sim);

console.log(`wasm ${path.basename(wasmPath)} (${(bytes.length / 1024).toFixed(0)} KB):`);
if (!E.sim_deco_hue_histogram) {
  console.error('FAIL: sim_deco_hue_histogram export missing — WASM must be rebuilt for Phase 10');
  process.exit(1);
}
check('export sim_deco_hue_histogram present', typeof E.sim_deco_hue_histogram === 'function');
check('scratch covers histogram region',
  E.sim_scratch_size() >= OFF_HIST + BUCKETS * 4, `${E.sim_scratch_size()} bytes`);

// 桶数上限截断：>64 的请求被安全钳到 DECO_HUE_BUCKETS_CAP（scratch 区容量）。
{
  const nCap = E.sim_deco_hue_histogram(sim, 100, scratch + OFF_HIST);
  check('buckets > cap clamps to 64', nCap === 64, `returned ${nCap}`);
}

// 懒视图：调用后立即读，避免 memory.grow 使旧视图失效。
function readHistogram() {
  const nB = E.sim_deco_hue_histogram(sim, BUCKETS, scratch + OFF_HIST);
  return Array.from(new Uint32Array(E.memory.buffer, scratch + OFF_HIST, nB));
}
function decoAlive() {
  E.sim_counts(sim);
  return new Uint32Array(E.memory.buffer, scratch + OFF_COUNTS, 6)[5];
}

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

function sampleAt() {
  return {
    tick: E.sim_tick(sim),
    counts: readHistogram(),
    deco: decoAlive(),
    eff: E.sim_deco_eff_mean(sim),
    effStd: E.sim_deco_eff_std(sim),
  };
}

// t=0（刚创建、未受选择压）+ warmup 后基线 + 每 600 ticks 采样到 3000。
const t0 = sampleAt();
runTicks(WARMUP);
const samples = [sampleAt()];
for (let t = WARMUP + SAMPLE_INTERVAL; t <= TOTAL_TICKS; t += SAMPLE_INTERVAL) {
  runTicks(t);
  samples.push(sampleAt());
}
if (samples[samples.length - 1].tick !== TOTAL_TICKS) {
  runTicks(TOTAL_TICKS);
  samples.push(sampleAt());
}
const allSamples = [t0, ...samples];

// 直方图速览（诊断输出，不计入断言）。
console.log('\n  DECO hue histogram (bucket counts, 0.63 → 0.87):');
for (const s of allSamples) {
  const st = histStats(s.counts);
  const stat = st ? `mean=${st.mean.toFixed(1)} σ=${st.std.toFixed(1)}` : 'empty';
  console.log(`  t=${String(s.tick).padStart(4)}  ${s.counts.map((c) => String(c).padStart(2)).join(' ')}  | ${stat} | eff=${s.eff.toFixed(3)}`);
}
console.log('');

// ─── 断言 1+2：counts 有效性 & 守恒 ────────────────────────────────────────────
for (const s of allSamples) {
  check(`tick ${s.tick}: counts 全部为非负整数且 ≤ DECO 总数`,
    s.counts.every((c) => Number.isInteger(c) && c >= 0 && c <= s.deco),
    `[${s.counts.join(',')}] vs deco=${s.deco}`);
  check(`tick ${s.tick}: sum(counts) === 存活 DECO 数`,
    s.counts.reduce((a, b) => a + b, 0) === s.deco);
  check(`tick ${s.tick}: DECO 种群存活`, s.deco > 0, `${s.deco}`);
}

// ─── 断言 3：均值桶位右移（t=warmup → t=3000）──────────────────────────────────
const first = samples[0];
const last = samples[samples.length - 1];
const st0 = histStats(first.counts);
const st1 = histStats(last.counts);
check('直方图两端均有存活 DECO', st0 !== null && st1 !== null,
  `t=${first.tick}: ${st0?.total}, t=${last.tick}: ${st1?.total}`);
if (st0 && st1) {
  check('均值桶位右移（选择 → 分布向品红漂移）',
    st1.mean > st0.mean + 1.0,
    `mean bucket ${st0.mean.toFixed(2)} → ${st1.mean.toFixed(2)} (+${(st1.mean - st0.mean).toFixed(2)})`);

  // ─── 断言 4：分布收窄 ────────────────────────────────────────────────────────
  check('分布收窄（选择 → σ 变小）',
    st1.std < st0.std * 0.95,
    `σ bucket ${st0.std.toFixed(2)} → ${st1.std.toFixed(2)} (${((st1.std / st0.std - 1) * 100).toFixed(0)}%)`);

  // ─── 断言 5：与 Phase 9 mean_efficiency 互证 ─────────────────────────────────
  const expFirst = bucketOfHue(hueOfEff(first.eff));
  const expLast = bucketOfHue(hueOfEff(last.eff));
  check(`t=${first.tick}: 观测均值桶 ≈ eff→hue 期望桶（±2）`,
    Math.abs(st0.mean - expFirst) <= 2.0,
    `observed ${st0.mean.toFixed(2)} vs expected ${expFirst.toFixed(2)}`);
  check(`t=${last.tick}: 观测均值桶 ≈ eff→hue 期望桶（±2）`,
    Math.abs(st1.mean - expLast) <= 2.0,
    `observed ${st1.mean.toFixed(2)} vs expected ${expLast.toFixed(2)}`);
  const obsShift = st1.mean - st0.mean;
  const expShift = expLast - expFirst;
  check('bucket 迁移量与 Δhue（Phase 9 口径）一致（±2.5）',
    Math.abs(obsShift - expShift) <= 2.5,
    `Δobs ${obsShift.toFixed(2)} vs Δexp ${expShift.toFixed(2)}`);
}

E.sim_free(sim);

if (failures > 0) {
  console.error(`\nEVOLUTION HISTOGRAM FAILED: ${failures} check(s) failed`);
  process.exit(1);
}
console.log('\nEVOLUTION HISTOGRAM OK');
