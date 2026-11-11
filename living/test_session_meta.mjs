// test_session_meta.mjs — Phase 14 实验回路状态机（SessionMeta）Node 冒烟测试。
//
// 测试覆盖：
//   1. 初始状态 'live'，liveCount=0，loadedRows 空
//   2. live 模式 pushLive ×5 → liveCount=5
//   3. 用 Phase 13 的 encodeEvolutionHistory 生成 fake JSON → loadFromJSON：
//      state 切换 'loaded'、loadedRows 行数正确、内容 bit-exact、行 tick 映射正确
//   4. loaded 模式下再 pushLive → 状态保持 'loaded'（liveCount 冻结、旧行不覆盖）
//   5. clear() → 恢复 'live'，liveCount 归零、loadedRows 清空
//   6. setOnStateChange 回调收到 live→loaded→live 转移
//   7. schema 校验：version≠1 / data 非 4 字节对齐 / historyFrames 不一致 → 抛错且状态不变
//
// 纯逻辑测试：零 DOM / 零 canvas / 零 wasm。
// Usage: node living/test_session_meta.mjs

import { createSessionMeta } from './src/session-meta.mjs';
import { encodeEvolutionHistory } from './src/evolution-export.mjs';
import { HIST_FRAME_TICKS } from './src/renderer-evolution-histogram.mjs';

let failures = 0;
const check = (name, cond, detail = '') => {
  if (cond) console.log(`  ok   ${name}${detail ? ` — ${detail}` : ''}`);
  else { console.error(`  FAIL ${name}${detail ? ` — ${detail}` : ''}`); failures += 1; }
};

const BUCKETS = 24;
const FRAMES = 50;

// ─── fake history：用 Phase 13 编码器生成可辨识的非平凡数据 ─────────────────────
const srcRows = [];
for (let r = 0; r < FRAMES; r++) {
  const row = new Uint32Array(BUCKETS);
  for (let b = 0; b < BUCKETS; b++) row[b] = (r * BUCKETS + b) % 7;
  srcRows.push(row);
}
const CAPTURED_TICK = 500;
const json = encodeEvolutionHistory({
  rows: srcRows, buckets: BUCKETS, hueMin: 0.629, hueSpan: 0.242, capturedTick: CAPTURED_TICK,
});

const meta = createSessionMeta();
const counts = new Uint32Array(BUCKETS);

// 1. 初始状态
check('initial state is live', meta.getState() === 'live');
check('initial liveCount is 0', meta.getLiveCount() === 0, `got ${meta.getLiveCount()}`);
check('initial loadedRows empty', meta.getLoadedRows().length === 0);

// 2. live 模式 pushLive ×5
for (let i = 0; i < 5; i++) meta.pushLive(counts);
check('pushLive ×5 → liveCount=5', meta.getLiveCount() === 5, `got ${meta.getLiveCount()}`);
check('still live after pushLive', meta.getState() === 'live');

// 3. loadFromJSON：fake JSON → loaded
const transitions = [];
meta.setOnStateChange((s) => transitions.push(s));
const info = meta.loadFromJSON(json);
check('state switches to loaded', meta.getState() === 'loaded');
check('loadedRows row count', meta.getLoadedRows().length === FRAMES, `got ${meta.getLoadedRows().length}`);
check('info.frames matches', info.frames === FRAMES);
check('info.capturedTick parsed from capturedAt "tick N"', info.capturedTick === CAPTURED_TICK, `got ${info.capturedTick}`);
check('info.buckets passthrough', info.buckets === BUCKETS);
check('liveCount independent of load', meta.getLiveCount() === 5, `got ${meta.getLiveCount()}`);
const rows = meta.getLoadedRows();
let bitExact = true;
for (const idx of [0, Math.floor(FRAMES / 2), FRAMES - 1]) {
  for (let b = 0; b < BUCKETS; b++) {
    if (rows[idx].counts[b] !== srcRows[idx][b]) { bitExact = false; break; }
  }
}
check('loaded rows bit-exact vs source', bitExact);
check('newest row tick = capturedTick', rows[FRAMES - 1].tick === CAPTURED_TICK, `got ${rows[FRAMES - 1].tick}`);
check('oldest row tick = capturedTick − (N−1)×HIST_FRAME_TICKS',
  rows[0].tick === CAPTURED_TICK - (FRAMES - 1) * HIST_FRAME_TICKS, `got ${rows[0].tick} want ${CAPTURED_TICK - (FRAMES - 1) * HIST_FRAME_TICKS}`);
check('frameIndex sequence 0..N−1', rows[0].frameIndex === 0 && rows[FRAMES - 1].frameIndex === FRAMES - 1);

// 4. loaded 模式 pushLive：状态保持，liveCount 冻结，旧行不覆盖
const rowSnapshot = Array.from(rows[0].counts);
for (let i = 0; i < 3; i++) meta.pushLive(counts);
check('loaded: state stays loaded', meta.getState() === 'loaded');
check('loaded: liveCount frozen', meta.getLiveCount() === 5, `got ${meta.getLiveCount()}`);
check('loaded: rows untouched', Array.from(meta.getLoadedRows()[0].counts).every((v, i) => v === rowSnapshot[i]));

// 5. clear()：恢复 live，全部归零
meta.clear();
check('clear: back to live', meta.getState() === 'live');
check('clear: liveCount reset to 0', meta.getLiveCount() === 0, `got ${meta.getLiveCount()}`);
check('clear: loadedRows emptied', meta.getLoadedRows().length === 0);
check('clear: loadedInfo nulled', meta.getLoadedInfo() === null);

// 6. 状态转移回调：live → loaded → live
check('state transitions notified', transitions.length === 2 && transitions[0] === 'loaded' && transitions[1] === 'live',
  JSON.stringify(transitions));

// 7. schema 校验：抛错且状态不被污染
let threw = false;
try { meta.loadFromJSON(JSON.stringify({ version: 99, data: '' })); } catch { threw = true; }
check('rejects wrong schema version', threw);

threw = false;
const truncated = new Uint8Array(6); // 6 B → 无法按 u32 对齐
let bin = '';
for (const b of truncated) bin += String.fromCharCode(b);
try {
  meta.loadFromJSON(JSON.stringify({
    version: 1, capturedAt: 'tick 1', historyFrames: 1, buckets: BUCKETS,
    hueMin: 0.629, hueSpan: 0.242, data: btoa(bin),
  }));
} catch { threw = true; }
check('rejects truncated (non-u32-aligned) data', threw);

threw = false;
const tampered = JSON.parse(encodeEvolutionHistory({
  rows: [new Uint32Array(BUCKETS)], buckets: BUCKETS, hueMin: 0.629, hueSpan: 0.242, capturedTick: 1,
}));
tampered.historyFrames = 5;
try { meta.loadFromJSON(JSON.stringify(tampered)); } catch { threw = true; }
check('rejects historyFrames mismatch', threw);

check('failed loads leave state untouched', meta.getState() === 'live' && meta.getLoadedRows().length === 0);

// 8. clear 后可再次 load（实验回路可重复往返）
const info2 = meta.loadFromJSON(json);
check('reload after clear works', info2.frames === FRAMES && meta.getState() === 'loaded' && meta.getLoadedRows().length === FRAMES);

console.log(failures === 0 ? '\nSESSION META SMOKE OK' : `\nSMOKE FAILED: ${failures} check(s) failed`);
process.exit(failures === 0 ? 0 : 1);
