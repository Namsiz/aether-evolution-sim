// evolution-export.mjs — Phase 13 光谱 ring buffer 导出（E 键）的纯逻辑部分。
//
// 编码：720 × 24 u32 → 69 120 B → base64 ≈ 92 KB（localStorage 5 MB 上限内宽裕）。
// schema version=1：行序 = 最老 → 最新；data 为行优先扁平 u32（LE）的 base64。
//
// decode 接口预留于 encodeEvolutionHistory 输出 JSON，解码端由 Phase 14 消费者自行实现。

export const EVOLUTION_HISTORY_KEY = 'aether-evolution-history';
export const EVOLUTION_HISTORY_VERSION = 1;

/** 把 u32 平面数组打包成 base64 字符串（浏览器 btoa / Node global）。 */
function u32ToBase64(u32) {
  const bytes = new Uint8Array(u32.buffer, u32.byteOffset, u32.byteLength);
  let bin = '';
  // String.fromCharCode.apply(null, array) 对长数组会触发栈溢出，分块拼接。
  for (let i = 0; i < bytes.length; i += 0x8000) {
    bin += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
  }
  return btoa(bin);
}

/**
 * @param {{ rows: ReadonlyArray<Uint32Array>, buckets: number, hueMin: number,
 *          hueSpan: number, capturedTick: number, capturedAt?: string }} opts
 * @returns {string} JSON 字符串
 */
export function encodeEvolutionHistory({ rows, buckets, hueMin, hueSpan, capturedTick, capturedAt }) {
  if (!Array.isArray(rows) || rows.length === 0) {
    throw new Error('encodeEvolutionHistory: rows 为空');
  }
  const width = rows[0].length;
  if (width !== buckets) {
    throw new Error(`encodeEvolutionHistory: 第一行宽度 ${width} ≠ 期望桶数 ${buckets}`);
  }
  const flat = new Uint32Array(rows.length * width);
  for (let r = 0; r < rows.length; r++) {
    if (rows[r].length !== width) {
      throw new Error(`encodeEvolutionHistory: 第 ${r} 行宽度 ${rows[r].length} ≠ ${width}`);
    }
    flat.set(rows[r], r * width);
  }
  return JSON.stringify({
    version: EVOLUTION_HISTORY_VERSION,
    capturedAt: capturedAt ?? `tick ${capturedTick}`,
    historyFrames: rows.length,
    buckets,
    hueMin,
    hueSpan,
    data: u32ToBase64(flat),
  });
}
