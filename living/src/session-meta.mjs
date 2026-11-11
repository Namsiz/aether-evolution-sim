// session-meta.mjs — Phase 14 实验回路状态机：区分 live（实时推帧）与 loaded（已恢复快照）。
//
// 纯逻辑模块，零 DOM 依赖（Node 冒烟测试直接消费）。viewer 侧约定：
//   - loaded 模式下 spectrogram ring 冻结在快照上（不 pushFrame），快照内容稳定可研究；
//     sim 本身继续实时运行，恢复的数据只存在于 UI 层，不触碰任何 wasm 内部状态。
//   - loadFromJSON 解码 Phase 13 的 evolution history JSON（schema v1，见
//     evolution-export.mjs）：base64 u32 平面 → 每行 Uint32Array(buckets)，行序 = 最老 → 最新。
//   - capturedTick 从 capturedAt 的 "tick N" 字符串解析（v1 schema 未落独立数值字段，
//     解析失败按 0 处理，行 tick 全部钳到 ≥0）。

import { EVOLUTION_HISTORY_VERSION } from './evolution-export.mjs';
import { HIST_FRAME_TICKS } from './renderer-evolution-histogram.mjs';

const CAPTURED_AT_RE = /^tick (\d+)$/;

export function createSessionMeta() {
  let state = 'live';    // 'live' | 'loaded'
  let liveCount = 0;     // live 模式下实收的 pushLive 次数（独立计数器，load 不改写）
  let loadedRows = [];   // [{ frameIndex, tick, counts: Uint32Array }]，行序 = 最老 → 最新
  let loadedInfo = null; // { capturedAt, capturedTick, frames, buckets, hueMin, hueSpan }
  let onChange = null;

  const notify = () => { if (onChange) onChange(state); };

  // loaded 模式下 no-op：liveCount 冻结、loadedRows 不被覆盖。
  function pushLive(counts) {
    if (state !== 'live') return liveCount;
    liveCount += 1;
    return liveCount;
  }

  function loadFromJSON(jsonStr) {
    const saved = JSON.parse(jsonStr);
    if (!saved || saved.version !== EVOLUTION_HISTORY_VERSION) {
      throw new Error(`不支持的 evolution history schema version=${saved && saved.version}`);
    }
    const buckets = saved.buckets;
    if (!Number.isInteger(buckets) || buckets <= 0) {
      throw new Error(`buckets 非法: ${buckets}`);
    }
    const bin = atob(saved.data);
    const u8 = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) u8[i] = bin.charCodeAt(i);
    const flat = new Uint32Array(u8.buffer); // 非 4 字节对齐时此处直接 RangeError
    if (flat.length % buckets !== 0) {
      throw new Error(`data 解出 ${flat.length} 个 u32，不是 buckets=${buckets} 的整数倍`);
    }
    const rows = flat.length / buckets;
    if (Number.isInteger(saved.historyFrames) && saved.historyFrames !== rows) {
      throw new Error(`historyFrames=${saved.historyFrames} ≠ data 行数 ${rows}`);
    }
    const m = CAPTURED_AT_RE.exec(saved.capturedAt ?? '');
    const capturedTick = m ? Number(m[1]) : 0;
    loadedRows = [];
    for (let r = 0; r < rows; r++) {
      loadedRows.push({
        frameIndex: r,
        tick: Math.max(0, capturedTick - (rows - 1 - r) * HIST_FRAME_TICKS),
        counts: flat.slice(r * buckets, (r + 1) * buckets),
      });
    }
    loadedInfo = {
      capturedAt: saved.capturedAt ?? '',
      capturedTick,
      frames: rows,
      buckets,
      hueMin: saved.hueMin,
      hueSpan: saved.hueSpan,
    };
    state = 'loaded';
    notify();
    return loadedInfo;
  }

  function clear() {
    state = 'live';
    liveCount = 0;
    loadedRows = [];
    loadedInfo = null;
    notify();
  }

  return {
    pushLive,
    loadFromJSON,
    clear,
    getState: () => state,
    getLiveCount: () => liveCount,
    getLoadedRows: () => loadedRows,
    getLoadedInfo: () => loadedInfo,
    setOnStateChange: (fn) => { onChange = fn; },
  };
}
