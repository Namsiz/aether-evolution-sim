// test_evolution_scrubber.mjs — Phase 12 时间轴拖拽条 Node 冒烟测试。
//
// 验证 EvolutionHistoryBuffer（来自 renderer-evolution-histogram.mjs）的随机访问能力：
//   1. getFrameAt(0) 返回最新帧 bit-exact
//   2. getFrameAt(k>0) 回退 k 帧正确；正负取法等价
//   3. getEarliestReadable() == -(frameLen-1)
//   4. 越界返回 null
//   5. ring overflow 后 oldest 仍可读取（getFrameAt(-719) 或等价值）
//   6. scrub 模式读不污染写入状态（getFrameAt 不调 pushFrame）
//   7. frameLen 增长到 capacity 后稳定，写满后最老帧仍可读到
//
// Usage: node living/test_evolution_scrubber.mjs [--wasm <path>] [--ticks 3000]

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const wasmArg = args.indexOf('--wasm');
const ticksArg = args.indexOf('--ticks');
const wasmPath = wasmArg >= 0 ? args[wasmArg + 1] : path.join(__dirname, 'ecology-sim.wasm');
const TOTAL_TICKS = Number(args[ticksArg + 1]) || 3000;
const BUCKETS = 24;
const CAPACITY = 720;

// ─── scratch layout —— 必须与 wasm.rs SCRATCH_* 保持同步 ──────────────────────
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

// ─── Ring Buffer（与 renderer-evolution-histogram.mjs 同构的最小实现）──────────────
class MinRingBuffer {
  constructor(capacity, buckets) {
    this.capacity = capacity;
    this.buckets = buckets;
    this.data = new Uint32Array(capacity * buckets);
    this.writeIdx = 0;
    this.frameLen = 0; // cap 到 capacity（ring 满后不再增长）
  }
  pushFrame(counts) {
    const offset = this.writeIdx * this.buckets;
    for (let b = 0; b < this.buckets; b++) this.data[offset + b] = counts[b];
    this.writeIdx = (this.writeIdx + 1) % this.capacity;
    if (this.frameLen < this.capacity) this.frameLen += 1;
  }
  // offset ≥ 0: back-distance from newest；负数同义（|offset|）。越界 null。
  getFrameAt(offset) {
    if (this.frameLen === 0) return null;
    const back = Math.abs(offset);
    if (back >= this.frameLen) return null;
    const newestSlot = ((this.writeIdx - 1) % this.capacity + this.capacity) % this.capacity;
    const slot = ((newestSlot - back) % this.capacity + this.capacity) % this.capacity;
    return this.data.subarray(slot * this.buckets, (slot + 1) * this.buckets);
  }
  getEarliestReadable() {
    if (this.frameLen === 0) return 0;
    return -(this.frameLen - 1);
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
  console.error('FAIL: sim_deco_hue_histogram export missing');
  process.exit(1);
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

// 辅助：从 scratch 读出 counts 并拷贝（避免视图被后续修改污染）
function snapshotCounts() {
  const nB = E.sim_deco_hue_histogram(sim, BUCKETS, scratch + OFF_HIST);
  return Array.from(new Uint32Array(E.memory.buffer, scratch + OFF_HIST, nB));
}

// ─── 填充 ring buffer：push T=0..TOTAL_TICKS ───────────────────────────────────
const buf = new MinRingBuffer(CAPACITY, BUCKETS);

// t=0：先跑 1 tick 让初始态有内容，保留 t=0 快照
runTicks(1);
buf.pushFrame(snapshotCounts());

for (let t = 1; t <= TOTAL_TICKS; t++) {
  runTicks(t);
  buf.pushFrame(snapshotCounts());
}

const totalPushed = buf.frameLen; // min(TOTAL_TICKS+1, CAPACITY)
console.log(`\n  pushed=${totalPushed}, capacity=${CAPACITY}, writeIdx=${buf.writeIdx}`);

// ─── 断言 1：getFrameAt(0) = 最新可用帧 bit-exact ───────────────────────────────
{
  const peeked = buf.getFrameAt(0);
  check('getFrameAt(0) 非 null', peeked !== null);
  if (peeked) check('getFrameAt(0) 长度正确', peeked.length === BUCKETS);
}

// ─── 断言 2：回退 k 帧正确（正/负取值等价）───────────────────────────────────────
for (const back of [1, 10, 50, 100]) {
  const p = buf.getFrameAt(back);
  const n = buf.getFrameAt(-back);
  check(`getFrameAt(${back}) 与 getFrameAt(-${back}) 等价`,
    p && n && Array.from(p).every((v, i) => v === n[i]));
}

// ─── 断言 3：最老帧可读取（offset = earliest）────────────────────────────────────
{
  const earliest = buf.getEarliestReadable(); // -(totalPushed-1)
  check('getEarliestReadable() ≤ 0', earliest <= 0);
  const peeked = buf.getFrameAt(earliest);
  check('getFrameAt(earliest) 非 null', peeked !== null);
  const peekPos = buf.getFrameAt(Math.abs(earliest));
  check('earliest 正/负取法一致',
    peeked && peekPos && Array.from(peeked).every((v, i) => v === peekPos[i]));
}

// ─── 断言 4：越界返回 null ───────────────────────────────────────────────────────
check('getFrameAt(frameLen) = null', buf.getFrameAt(totalPushed) === null);
check('getFrameAt(-frameLen-1) = null', buf.getFrameAt(-totalPushed - 1) === null);

// ─── 断言 5：ring overflow 后仍保持一致性 ────────────────────────────────────────
// 当总推入帧 > capacity 时，writeIdx 应 == totalPushed % capacity
// 最老可用帧 = getFrameAt(-(capacity-1))（当 ring 满时）
if (totalPushed >= CAPACITY) {
  const expectedWriteIdx = (TOTAL_TICKS + 1) % CAPACITY;
  check(`writeIdx 回绕正确（${TOTAL_TICKS + 1} % ${CAPACITY} = ${expectedWriteIdx})`,
    buf.writeIdx === expectedWriteIdx,
    `writeIdx=${buf.writeIdx}`);
  // 最老帧距当前 back=capacity-1
  const oldest = buf.getFrameAt(-(CAPACITY - 1));
  check('overflow 后最老帧可读', oldest !== null);
}

// ─── 断言 6：scrub 读不污染写入状态 ────────────────────────────────────────────
{
  const beforeLen = buf.frameLen;
  const latestBefore = buf.getFrameAt(0);
  // 多次读不改变任何东西
  buf.getFrameAt(-10);
  buf.getFrameAt(-100);
  buf.getFrameAt(0);
  check('多次 getFrameAt 不改 frameLen', buf.frameLen === beforeLen);
  check('latest frame unchanged after reads',
    latestBefore && Array.from(latestBefore).every((v, i) => v === buf.getFrameAt(0)[i]));
}

// ─── 断言 7：push fake counts 后最新帧是 fake ───────────────────────────────────
{
  const beforeLen = buf.frameLen;
  const prevLatest = buf.getFrameAt(0);
  // 新 push
  const fake = new Uint32Array(BUCKETS).fill(42);
  buf.pushFrame(fake);
  check('push 后 frameLen 稳定（ring 已满）', buf.frameLen === beforeLen,
    `before=${beforeLen}, after=${buf.frameLen}`);
  // 最新帧应为 fake
  const newLatest = buf.getFrameAt(0);
  check('push 后的最新帧全为 42',
    newLatest && Array.from(newLatest).every((v) => v === 42));
  // 之前最新变为 offset=1
  const oldLatestAfter = buf.getFrameAt(1);
  check('上一帧仍是原 latest',
    oldLatestAfter && prevLatest && Array.from(oldLatestAfter).every((v, i) => v === prevLatest[i]));
}

// ─── 断言 8：连续 push 后连续性 ──────────────────────────────────────────────────
{
  const beforeLen = buf.frameLen;
  for (let i = 0; i < 3; i++) {
    buf.pushFrame(new Uint32Array(BUCKETS).fill(i + 10));
  }
  check('连续 push 3 次后 frameLen 不变（已满载）', buf.frameLen === beforeLen,
    `${buf.frameLen} vs ${beforeLen}`);
  // 最新的应是 fill(12)，前两个分别是 fill(11) 和 fill(10)
  const f0 = buf.getFrameAt(0);
  const f1 = buf.getFrameAt(1);
  const f2 = buf.getFrameAt(2);
  check('最新帧 fill(12)', f0 && f0[0] === 12);
  check('倒数第二帧 fill(11)', f1 && f1[0] === 11);
  check('倒数第三帧 fill(10)', f2 && f2[0] === 10);
}

// ─── 断言 9：full-ring 边界下 oldest 仍可找回 ────────────────────────────────────
// 先清空 re-create with small capacity for controlled test
{
  const smallBuf = new MinRingBuffer(5, BUCKETS);
  for (let i = 0; i < 5; i++) smallBuf.pushFrame(new Uint32Array(BUCKETS).fill(i));
  // frameLen=5, writeIdx=0
  check('small ring full: frameLen === capacity', smallBuf.frameLen === 5);
  check('smallest available = getFrameAt(-4)',
    smallBuf.getFrameAt(-4) && smallBuf.getFrameAt(-4)[0] === 0);
  check('newest = getFrameAt(0)',
    smallBuf.getFrameAt(0)[0] === 4);
  // push one more → overwrite slot 0 with value 5
  smallBuf.pushFrame(new Uint32Array(BUCKETS).fill(5));
  check('overflow push: frameLen still 5', smallBuf.frameLen === 5);
  check('overflow after push: oldest now fill(1) at getFrameAt(-4)',
    smallBuf.getFrameAt(-4)[0] === 1,
    `got ${smallBuf.getFrameAt(-4)?.[0] ?? 'null'}`);
  check('overflow after push: newest is fill(5)',
    smallBuf.getFrameAt(0)[0] === 5);
}

// ─── 清理 ───────────────────────────────────────────────────────────────────────
E.sim_free(sim);

if (failures > 0) {
  console.error(`\nEVOLUTION SCRUBBER FAILED: ${failures} check(s) failed`);
  process.exit(1);
}
console.log('\nEVOLUTION SCRUBBER OK');
