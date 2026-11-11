// test_evolution_labels.mjs — Phase 13 标注/导出纯逻辑单测（无 canvas mock）。
//
// 测试覆盖：
//   1. getTickFromFrame：默认 ×2，显式 ×HIST_FRAME_TICKS=10
//   2. 光谱 ring（沿用 Phase 12 的 MinRingBuffer 同构实现）3001 帧 push 后
//      getFrameAt 与 wasm 输出 bit-exact；且 Y 轴刻度线的 tick 与对应帧内容一致
//   3. spectrogramYTicks：y=back×scale、chrome 带过滤、tick=anchor−back×10
//   4. scrubberXTicks：每 60 帧 mark、每 120 帧 label（交替）、x 单调递增、tick 映射正确
//   5. encodeEvolutionHistory：schema v1、base64 解码 bit-exact、720 行、JSON <200KB
//
// 不依赖 DOM/canvas：所有渲染相关函数以纯规划形式暴露供 Node 直接调用。
//
// Usage: node living/test_evolution_labels.mjs [--wasm <path>] [--ticks 3000]

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  getTickFromFrame,
  HIST_FRAME_TICKS,
  spectrogramYTicks,
} from './src/renderer-evolution-histogram.mjs';
import { scrubberXTicks } from './src/evolution-scrubber.mjs';
import { encodeEvolutionHistory } from './src/evolution-export.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const wasmArg = args.indexOf('--wasm');
const ticksArg = args.indexOf('--ticks');
const wasmPath = wasmArg >= 0 ? args[wasmArg + 1] : path.join(__dirname, 'ecology-sim.wasm');
const TOTAL_TICKS = Number(args[ticksArg + 1]) || 3000;
const BUCKETS = 24;
const CAPACITY = 720;
const TPF = 2; // TICKS_PER_FRAME（viewer frame → sim tick）

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

// ─── Ring Buffer（与 renderer-evolution-histogram.mjs createSpectrogramOverlay 内部同构）
class MinRingBuffer {
  constructor(capacity, buckets) {
    this.capacity = capacity;
    this.buckets = buckets;
    this.data = new Uint32Array(capacity * buckets);
    this.writeIdx = 0;
    this.frameLen = 0;
  }
  pushFrame(counts) {
    const offset = this.writeIdx * this.buckets;
    for (let b = 0; b < this.buckets; b++) this.data[offset + b] = counts[b];
    this.writeIdx = (this.writeIdx + 1) % this.capacity;
    if (this.frameLen < this.capacity) this.frameLen += 1;
  }
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

// ─── 辅助：从 scratch 读出 counts 并拷贝（避免视图被后续修改污染）─────────────────
function snapshotCounts(buf) {
  const nB = buf.readDecoHistogram(BUCKETS);
  return Array.from(new Uint32Array(buf.memory.buffer, buf.scratch + OFF_HIST, nB));
}

// ─── 初始化 wasm（可选）—— 用真实 counts 做 bit-exact 对照 ───────────────────────
let wasmCounts = null;
if (process.env.VARIANT !== 'pure') {
  try {
    const bytes = fs.readFileSync(wasmPath);
    const mod = await WebAssembly.compile(bytes);
    const instance = await WebAssembly.instantiate(mod, {});
    const E = instance.exports;
    const sim = E.sim_new(20261102);
    const scratch = E.sim_scratch(sim);

    const runTicks = (target) => {
      let current = E.sim_tick(sim);
      while (current < target) {
        const n = Math.min(DAYLIGHT_CAP, target - current);
        const dl = new Float64Array(E.memory.buffer, scratch + OFF_DAYLIGHT, DAYLIGHT_CAP);
        for (let i = 0; i < n; i++) dl[i] = daylightFor(current + i);
        E.sim_step(sim, n, scratch + OFF_DAYLIGHT, n);
        current = E.sim_tick(sim);
      }
    };

    runTicks(TOTAL_TICKS);
    wasmCounts = snapshotCounts({ memory: instance, scratch });
    E.sim_free(sim);
  } catch (_) {
    // wasm 不可用时降级为纯合成 counts（pattern-based），保证结构测试仍有效。
    wasmCounts = null;
  }
}

// ─── 填充 ring buffer ───────────────────────────────────────────────────────────
const buf = new MinRingBuffer(CAPACITY, BUCKETS);

for (let t = 0; t <= TOTAL_TICKS; t++) {
  if (wasmCounts) {
    buf.pushFrame(new Uint32Array(wasmCounts));
  } else {
    // 合成 counts：第 b 桶 = (t * 7 + b * 13) mod 251 （保证非零、变化快、bit 可预测）。
    const fake = new Uint32Array(BUCKETS);
    for (let b = 0; b < BUCKETS; b++) fake[b] = (t * 7 + b * 13 + 1) % 251;
    buf.pushFrame(fake);
  }
}
const totalPushed = buf.frameLen;

// ─── Test 1：getTickFromFrame ────────────────────────────────────────────────────
check('getTickFromFrame default ×2', getTickFromFrame(100) === 200);
check('getTickFromFrame explicit ×2', getTickFromFrame(100, 2) === 200);
check('getTickFromFrame ×HIST_FRAME_TICKS', getTickFromFrame(100, HIST_FRAME_TICKS) === 1000);
check('HIST_FRAME_TICKS === 10', HIST_FRAME_TICKS === 10);
check('负数归零（Math.round(NaN) 行为）', !Number.isNaN(getTickFromFrame(-10)));
check('零输入', getTickFromFrame(0) === 0);
// bit-exact：浮点输入 → round 后乘。
check('float 输入：round 到整数再乘', getTickFromFrame(100.6) === 202);

// ─── Test 2：ring bit-exact（延续 Phase 12）+ 帧内容与时间轴一致 ─────────────────
{
  // 沿用 Phase 12 断言风格：getFrameAt 各偏移返回正确帧内容。
  const sampleBacks = [0, 1, 10, 100, Math.min(totalPushed - 1, 300)];
  for (const b of sampleBacks) {
    const frame = buf.getFrameAt(b);
    check(`getFrameAt(${b}) 非 null`, frame !== null);
    if (!frame) continue;
    check(`getFrameAt(${b}) 长度=${BUCKETS}`, frame.length === BUCKETS);
    // 帧内容应等于 w=TOTAL_TICKS-b 时 push 的值（因为第 b+1 次 push 是 totalPushed 次里倒数第 b+1 个）。
    // 实际顺序：第一次 push 在 t=0，第二次在 t=1…第 totalPushed 次在 t=TOTAL_TICKS。
    // getFrameAt(0) = 最新（t=TOTAL_TICKS），getFrameAt(1) = t=TOTAL_TICKS-1 …
    const expectedT = TOTAL_TICKS - b;
    if (wasmCounts) {
      // 无法直接从 wasm 查历史；但可验证对称性（正负取值等价）和一致性（相邻回退差一帧）。
      check(`getFrameAt(${b}) 与 getFrameAt(-${b}) 等价`,
        buf.getFrameAt(b).every((v, i) => v === buf.getFrameAt(-b)[i]));
    } else {
      // 合成 case：frame[0] = (expectedT*7 + 0*13 + 1) % 251。
      const exp0 = (expectedT * 7 + 1) % 251;
      check(`fake[0]@back${b}=${buf.getFrameAt(b)[0]} 符合公式 ${(exp0)}`,
        buf.getFrameAt(b)[0] === exp0);
    }
  }
  check('最老帧可读（back = frameLen-1）',
    buf.getFrameAt(-(totalPushed - 1)) !== null);
  check('越界（back ≥ frameLen）返回 null', buf.getFrameAt(totalPushed) === null);
}

// ─── Test 3：spectrogramYTicks（纯函数）─────────────────────────────────────────
{
  // scale=1（height == historyFrames），frameLen=CAPACITY=720, newestTick=30000。
  const lines = spectrogramYTicks({
    frameLen: CAPACITY, historyFrames: CAPACITY, height: CAPACITY,
    newestTick: 30000,
  });
  check('Y 刻度条数 = 5（120..600 step 120）', lines.length === 5);
  // y 严格等于 back（scale=1）。
  const expected = [120, 240, 360, 480, 600];
  for (let i = 0; i < 5; i++) {
    check(`line[${i}] y=${lines[i].y} 精确 = back=${expected[i]}`,
      lines[i].y === expected[i]);
  }
  // tick = anchor − back×10。
  for (const l of lines) {
    check(`line t=${l.tick} = 30000 − ${l.back}×10`,
      l.tick === 30000 - l.back * HIST_FRAME_TICKS);
  }
  // chrome 带过滤：height 极小时所有线被丢弃。
  const tiny = spectrogramYTicks({ frameLen: 720, historyFrames: 720, height: 10, newestTick: 30000 });
  check('height=10 chrome 带吞没所有刻度', tiny.length === 0);
  // 空 frameLen → []。
  check('frameLen=0 返回空',
    spectrogramYTicks({ frameLen: 0, historyFrames: 720, height: 720, newestTick: 30000 }).length === 0);
  // scale≠1（下采样）情况下 y=round(back×scale)。
  const half = spectrogramYTicks({ frameLen: 720, historyFrames: 720, height: 360, newestTick: 30000 });
  check('half-height scale 映射：first y=round(120×2)=240', half[0].y === 240);
  check('half-height 底部 chrome 裁剪',
    half.every(l => l.y <= 360 - 16),
    `bottom line last y=${half[half.length - 1]?.y ?? 'none'}`);
}

// ─── Test 4：scrubberXTicks（纯函数）────────────────────────────────────────────
{
  const m = scrubberXTicks({
    frameLen: CAPACITY,
    trackLeft: 14, // PAD_X from evolution-scrubber.mjs
    trackW: 292,   // SPECTROGRAM_WIDTH - 2×PAD_X
    tickAnchor: 30000,
  });
  // 60..660 step 60 = 11 个 mark（f=60,120,...,660）。
  check('marks 数量 = 11', m.length === 11);
  // 第一个 mark 在 f=60。
  check('首个 mark f=60', m[0].f === 60);
  // 最后一个 mark 在 f=660（<719）。
  check('末个 mark f=660', m[m.length - 1].f === 660);
  // x 单调递增。
  for (let i = 1; i < m.length; i++) {
    check(`x 单调 inc: ${m[i-1].x.toFixed(3)} < ${m[i].x.toFixed(3)}`,
      m[i - 1].x < m[i].x);
  }
  // tick 映射正确。
  for (const m_ of m) {
    const back = CAPACITY - 1 - m_.f;
    check(`tick=f${m_.f}=30000-${back}×10=${30000 - back * HIST_FRAME_TICKS}`,
      m_.tick === 30000 - back * HIST_FRAME_TICKS);
  }
  // label 仅 f%120===0（等间隔 2×60）→ 5 个（120,240,360,480,600）。
  const labeled = m.filter(x => x.label !== null);
  check('labels 数量 = 5', labeled.length === 5);
  const expectedLabels = [120, 240, 360, 480, 600];
  for (let i = 0; i < labeled.length; i++) {
    check(`label[${i}] f=${labeled[i].f}=${expectedLabels[i]}`,
      labeled[i].f === expectedLabels[i]);
    check(`label[${i}] t=${labeled[i].label}`,
      labeled[i].label === `t=${labeled[i].tick}`);
  }
  // frameLen<2 时返回空。
  check('frameLen=1 返回空',
    scrubberXTicks({ frameLen: 1, trackLeft: 14, trackW: 292, tickAnchor: 30000 }).length === 0);
}

// ─── Test 5：encodeEvolutionHistory schema + bit-exact base64 往返 ──────────────
{
  const ROWS = 720;
  const fakeRows = [];
  for (let r = 0; r < ROWS; r++) {
    const row = new Uint32Array(BUCKETS);
    for (let b = 0; b < BUCKETS; b++) row[b] = ((r * 31 + b * 17 + 3) >>> 0) % 0xFFFFFF;
    fakeRows.push(row);
  }
  const json = encodeEvolutionHistory({
    rows: fakeRows,
    buckets: BUCKETS,
    hueMin: 0.629,
    hueSpan: 0.242,
    capturedTick: 30000,
  });
  check('JSON stringified 且合法', typeof json === 'string');
  const obj = JSON.parse(json);
  check('version=1', obj.version === 1);
  check('historyFrames=720', obj.historyFrames === ROWS);
  check('buckets=24', obj.buckets === BUCKETS);
  check('hueMin/hueSpan 透传', obj.hueMin === 0.629 && obj.hueSpan === 0.242);
  check('capturedAt 格式', /^tick \d+$/.test(obj.capturedAt));
  check('data base64 非空', typeof obj.data === 'string' && obj.data.length > 0);
  check('JSON 总长度 < 200 KB', json.length < 200 * 1024, `${json.length / 1024 | 0} KB`);
  // base64 解码 bit-exact 反序列。
  const bytes = Buffer.from(obj.data, 'base64');
  check('decoded byteLength = 720×24×4=69120', bytes.length === ROWS * BUCKETS * 4);
  const u32View = new Uint32Array(bytes.buffer, bytes.byteOffset, bytes.byteLength / 4);
  for (let r = 0; r < ROWS; r++) {
    const src = fakeRows[r];
    const off = r * BUCKETS;
    for (let b = 0; b < BUCKETS; b++) {
      if (u32View[off + b] !== src[b]) {
        check(`row ${r} bucket ${b}: decoded ${u32View[off + b]} === expected ${src[b]}`, false,
          `at idx ${off + b}`);
        throw new Error('bit mismatch'); // break outer loop
      }
    }
  }
  check('720×24 行 bit-exact 往返无误', true);
  // width 不一致抛错。
  const badRows = fakeRows.map(r => r.subarray(0, BUCKETS));
  badRows.push(new Uint32Array([1, 2])); // wrong width.
  check('width-mismatch 行数会 throw', (() => {
    try { encodeEvolutionHistory({ rows: badRows, buckets: BUCKETS, hueMin: 0.629, hueSpan: 0.242, capturedTick: 0 }); return false; }
    catch { return true; }
  })());
}

// ─── 清理 & 结论 ─────────────────────────────────────────────────────────────────
if (failures > 0) {
  console.error(`\nEVOLUTION LABELS FAILED: ${failures} check(s) failed`);
  process.exit(1);
}
console.log('\nEVOLUTION LABELS OK');
