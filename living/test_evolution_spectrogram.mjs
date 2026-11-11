// test_evolution_spectrogram.mjs — Phase 11 DECO hue 演化光谱条纹图数据链路验收。
//
// 与 Phase 10 的直方图测试同源，但关注「时间维度」：
//   1. Ring buffer 写入正确：pushFrame 后的 buffer 快照可重放重现同一分布
//      （与源头 counts 数组 bit-exact 对齐）。
//   2. 每 tick sum(counts) === 存活 DECO 数（守恒不变量）。
//   3. t=300（warmup 后）加权均值桶 ≈ 13.65（与 Phase 10 基线一致，±2 容差）。
//   4. t=3000 加权均值桶 ≈ 20.11（与 Phase 10 终态一致，±2 容差）。
//   5. 环形溢出安全：push 超过 720 帧后 oldest 被优雅丢弃，最新帧永远可读。
//
// Zero wasm changes：只消费既有 sim_deco_hue_histogram 导出，不改任何 .rs。
//
// Usage: node living/test_evolution_spectrogram.mjs [--wasm <path>] [--ticks 3000]

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const wasmArg = args.indexOf('--wasm');
const ticksArg = args.indexOf('--ticks');
const wasmPath = wasmArg >= 0 ? args[wasmArg + 1] : path.join(__dirname, 'ecology-sim.wasm');
const TOTAL_TICKS = Number(args[ticksArg + 1]) || 3000;
const WARMUP = 300;
const BUCKETS = 24;
const HISTORY_FRAMES = 720; // 环形 buffer 容量，与 renderer-evolution-histogram.mjs 同步

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

// 加权均值桶（0-based，与 Phase 10 口径一致）。
function histStats(counts) {
  const total = counts.reduce((a, b) => a + b, 0);
  if (total === 0) return null;
  const mean = counts.reduce((a, c, i) => a + c * i, 0) / total;
  return { total, mean };
}

// ─── Ring buffer（与 renderer-evolution-histogram.mjs 同构的最小实现）─────────────
class RingBuffer {
  constructor(historyFrames, buckets) {
    this.capacity = historyFrames;
    this.buckets = buckets;
    this.data = new Uint32Array(historyFrames * buckets);
    this.writeIdx = 0;
    this.pushed = 0;
  }
  pushFrame(counts) {
    const offset = this.writeIdx * this.buckets;
    for (let b = 0; b < this.buckets; b++) this.data[offset + b] = counts[b];
    this.writeIdx = (this.writeIdx + 1) % this.capacity;
    this.pushed += 1;
  }
  // 读取最近第 k 帧（k=0 最新，k=capacity-1 最老）。溢出后 oldest 已被覆盖。
  peek(k) {
    const idx = ((this.writeIdx - 1 - k) % this.capacity + this.capacity) % this.capacity;
    return this.data.subarray(idx * this.buckets, (idx + 1) * this.buckets);
  }
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

// ─── 主循环：逐 tick pushFrame 到环形 buffer ────────────────────────────────────
const ring = new RingBuffer(HISTORY_FRAMES, BUCKETS);
const snapshots = new Map(); // tick → { counts, deco }（用于重放对账）
const sumCheckTicks = new Set([0, 1, 150, 300, 1500, 3000]);

runTicks(1); // 先推 1 tick 让 t=1 有实际演化，同时保留 t=0 空环初始帧
const pushedFrames = [];

// t=0 快照（刚创建）。
{
  const counts = readHistogram();
  const deco = decoAlive();
  ring.pushFrame(counts);
  pushedFrames.push({ tick: 0, counts, deco });
  snapshots.set(0, { counts, deco });
}

for (let t = 1; t <= TOTAL_TICKS; t++) {
  runTicks(t);
  const counts = readHistogram();
  const deco = decoAlive();
  ring.pushFrame(counts);
  pushedFrames.push({ tick: t, counts, deco });
  if (sumCheckTicks.has(t)) snapshots.set(t, { counts, deco });
}

console.log(`\n  ring buffer: pushed=${ring.pushed}, capacity=${ring.capacity}, writeIdx=${ring.writeIdx}`);

// ─── 断言 1：pushFrame 次数与 expected ────────────────────────────────────────
check(`pushFrame 恰好执行 ${TOTAL_TICKS + 1} 次`, ring.pushed === TOTAL_TICKS + 1, `pushed=${ring.pushed}`);

// ─── 断言 2：buffer 内容与源头 counts 数组 bit-exact（重放对账）────────────────
// 最后一帧（最新）应与当前 wasm 读数一致。
{
  const last = pushedFrames[pushedFrames.length - 1];
  const latest = ring.peek(0);
  check('最新帧 bit-exact 对齐源头 counts',
    latest.length === last.counts.length && Array.from(latest).every((v, i) => v === last.counts[i]),
    `[${Array.from(latest).join(',')}] vs tick=${last.tick}`);
}
// 中段抽查 3 帧：ring.peek(k) 应与 push 时的记录一致。
for (const back of [1, 100, 359]) {
  const idx = ring.pushed - 1 - back;
  if (idx < 0) continue;
  const recorded = pushedFrames[idx];
  const peeked = ring.peek(back);
  check(`回溯 ${back} 帧 bit-exact 对齐（t=${recorded.tick}）`,
    Array.from(peeked).every((v, i) => v === recorded.counts[i]));
}

// ─── 断言 3：每 tick sum(counts) === 存活 DECO 数 ─────────────────────────────
for (const [tick, s] of snapshots) {
  const sum = s.counts.reduce((a, b) => a + b, 0);
  check(`t=${tick}: sum(counts) === 存活 DECO 数`, sum === s.deco, `sum=${sum} vs deco=${s.deco}`);
}

// ─── 断言 4：t=300 mean bucket ≈ 13.65 ────────────────────────────────────────
{
  const s = snapshots.get(300);
  const st = s ? histStats(s.counts) : null;
  check('t=300: 加权均值桶 ≈ 13.65（±2，与 Phase 10 基线一致）',
    st !== null && Math.abs(st.mean - 13.65) <= 2.0,
    st ? `observed ${st.mean.toFixed(2)}` : 'empty');
}

// ─── 断言 5：t=3000 mean bucket ≈ 20.11 ───────────────────────────────────────
{
  const s = snapshots.get(TOTAL_TICKS);
  const st = s ? histStats(s.counts) : null;
  check(`t=${TOTAL_TICKS}: 加权均值桶 ≈ 20.11（±2，与 Phase 10 终态一致）`,
    st !== null && Math.abs(st.mean - 20.11) <= 2.0,
    st ? `observed ${st.mean.toFixed(2)}` : 'empty');
}

// ─── 断言 6：环形溢出安全 ──────────────────────────────────────────────────────
// 3001 帧 push 进 720 容量 ring：pushed=3001 > capacity=720。
// writeIdx 应恰好回绕到 (3001 % 720) = 121。
check(`writeIdx 回绕正确（3001 % ${HISTORY_FRAMES} = ${3001 % HISTORY_FRAMES}）`,
  ring.writeIdx === (TOTAL_TICKS + 1) % HISTORY_FRAMES,
  `writeIdx=${ring.writeIdx}`);
// 最老帧（back = capacity-1）仍可读且与 push 时的对应帧一致。
{
  const back = HISTORY_FRAMES - 1;
  const idx = ring.pushed - 1 - back; // 3001-1-719 = 2281
  const recorded = pushedFrames[idx];
  const peeked = ring.peek(back);
  check(`最老帧（back=${back}）可读且与 push 时一致（t=${recorded.tick}）`,
    Array.from(peeked).every((v, i) => v === recorded.counts[i]),
    `t=${recorded.tick} counts=[${Array.from(peeked).join(',')}]`);
}
// 再抽查回绕边界附近 2 帧。
for (const back of [120, 121, 122]) {
  const idx = ring.pushed - 1 - back;
  if (idx < 0) continue;
  const recorded = pushedFrames[idx];
  const peeked = ring.peek(back);
  check(`回绕边界 back=${back}（t=${recorded.tick}）bit-exact`,
    Array.from(peeked).every((v, i) => v === recorded.counts[i]));
}

// ─── 断言 7：全时段演化趋势（与 Phase 10 叙事互证）──────────────────────────────
{
  const s300 = histStats(snapshots.get(300).counts);
  const s3000 = histStats(snapshots.get(TOTAL_TICKS).counts);
  check('均值桶右移（t=300 → t=3000，选择 → 分布向品红漂移）',
    s3000.mean > s300.mean + 1.0,
    `${s300.mean.toFixed(2)} → ${s3000.mean.toFixed(2)} (+${(s3000.mean - s300.mean).toFixed(2)})`);
}

E.sim_free(sim);

if (failures > 0) {
  console.error(`\nEVOLUTION SPECTROGRAM FAILED: ${failures} check(s) failed`);
  process.exit(1);
}
console.log('\nEVOLUTION SPECTROGRAM OK');
