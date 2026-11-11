// renderer-evolution-histogram.mjs — DECO hue 直方图 overlay（Phase 10）。
//
// 一块独立 2D canvas 叠在 Three.js WebGL canvas 上（position:fixed +
// pointer-events:none），把 wasm 导出的 DECO hue 桶计数画成 24 根 bar：
// 选择推进时 bar 整体右移（toward magenta）且变窄（σ 收窄）——「自然选择 →
// 分布变化」的图形化证据链。零 Three.js 对象、零 shader 成本。
//
// 数据契约（与 wasm.rs sim_deco_hue_histogram 同步）：
//   counts[i] = 落在第 i 桶的存活 DECO 数；
//   bucket i 覆盖 [0.629 + i×span, 0.629 + (i+1)×span)，span = 0.242 / buckets；
//   hue 端点由 visual_hue_deco 决定：eff 0.5 → 0.63（深紫），1.5 → 0.87（品红）。

export const DECO_HUE_MIN = 0.629;
export const DECO_HUE_SPAN = 0.242; // hue ∈ [0.629, 0.871]
export const DECO_HUE_BUCKETS = 24; // 与 wasm.rs DECO_HUE_BUCKETS_CAP 对齐（Phase 10 直方图实际桶数）

// Phase 5 hsl→rgb（与 wasm.rs hsl_to_rgb_u32 同公式），返回 [r, g, b] ∈ [0, 255]。
export function hslToRgb(h, s, l) {
  h = ((h % 1) + 1) % 1;
  if (s <= 0) {
    const v = Math.round(l * 255);
    return [v, v, v];
  }
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
  const p = 2 * l - q;
  const hue2rgb = (t) => {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1 / 6) return p + (q - p) * 6 * t;
    if (t < 0.5) return q;
    if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
    return p;
  };
  const to8 = (v) => Math.round(v * 255);
  return [to8(hue2rgb(h + 1 / 3)), to8(hue2rgb(h)), to8(hue2rgb(h - 1 / 3))];
}

// bucket i 的中心 hue（bar 颜色 = 该 hue 的 DECO 物种色，深紫→品红沿 hue 轴自然渐变）。
export function bucketCenterHue(i, buckets) {
  return DECO_HUE_MIN + ((i + 0.5) / buckets) * DECO_HUE_SPAN;
}

// 手动圆角矩形路径：不依赖 ctx.roundRect（较新 API，部分 WebView 缺失）。
function panelPath(ctx, w, h, r) {
  ctx.beginPath();
  ctx.moveTo(r, 0.5);
  ctx.lineTo(w - r, 0.5);
  ctx.quadraticCurveTo(w - 0.5, 0.5, w - 0.5, r);
  ctx.lineTo(w - 0.5, h - r);
  ctx.quadraticCurveTo(w - 0.5, h - 0.5, w - r, h - 0.5);
  ctx.lineTo(r, h - 0.5);
  ctx.quadraticCurveTo(0.5, h - 0.5, 0.5, h - r);
  ctx.lineTo(0.5, r);
  ctx.quadraticCurveTo(0.5, 0.5, r, 0.5);
  ctx.closePath();
}

export function createEvolutionHistogramOverlay(opts = {}) {
  const width = opts.width ?? 320;
  const height = opts.height ?? 160;
  const left = opts.left ?? 20;
  const top = opts.top ?? 20;
  const decoSat = opts.saturation ?? 0.47; // hue_sl(DECO) 的 s/l，bar 色与场上个体一致
  const decoLight = opts.lightness ?? 0.70;

  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const canvas = document.createElement('canvas');
  canvas.id = 'deco-hist-overlay';
  canvas.width = Math.round(width * dpr);
  canvas.height = Math.round(height * dpr);
  canvas.style.cssText = [
    'position:fixed', `left:${left}px`, `top:${top}px`,
    `width:${width}px`, `height:${height}px`,
    'z-index:25', 'pointer-events:none', 'display:none',
  ].join(';');
  document.body.appendChild(canvas);
  const ctx = canvas.getContext('2d');
  ctx.scale(dpr, dpr);

  let visible = false;

  const setVisible = (v) => {
    visible = v;
    canvas.style.display = v ? '' : 'none';
  };

  const PAD = 12;
  const TITLE_H = 22;
  const AXIS_H = 14;
  const plotX = PAD;
  const plotY = TITLE_H;
  const plotW = width - PAD * 2;
  const plotH = height - TITLE_H - AXIS_H;
  const MONO = '10px "SF Mono", ui-monospace, monospace';

  // counts: 桶计数（u32 数组）；maxCount: y 轴自动缩放上限；tickCount: 右上角 tick 读数。
  function update(counts, maxCount, tickCount) {
    if (!visible) return;
    const n = counts.length;
    ctx.clearRect(0, 0, width, height);

    // 面板底（与 HUD 同色系）。
    ctx.fillStyle = 'rgba(10,16,18,0.55)';
    ctx.strokeStyle = 'rgba(210,232,220,0.16)';
    ctx.lineWidth = 1;
    panelPath(ctx, width, height, 10);
    ctx.fill();
    ctx.stroke();

    ctx.font = MONO;
    ctx.textBaseline = 'alphabetic';
    ctx.fillStyle = '#cfe3d6';
    ctx.textAlign = 'left';
    ctx.fillText('DECO hue distribution', PAD, 14);
    ctx.fillStyle = 'rgba(207,227,214,0.55)';
    ctx.textAlign = 'right';
    ctx.fillText(`t=${tickCount}`, width - PAD, 14);

    // x 轴基线。
    ctx.strokeStyle = 'rgba(207,227,214,0.25)';
    ctx.beginPath();
    ctx.moveTo(plotX, plotY + plotH + 0.5);
    ctx.lineTo(plotX + plotW, plotY + plotH + 0.5);
    ctx.stroke();

    ctx.font = '9px "SF Mono", ui-monospace, monospace';
    if (!(maxCount > 0)) {
      ctx.fillStyle = 'rgba(207,227,214,0.5)';
      ctx.textAlign = 'center';
      ctx.fillText('no DECO alive', width / 2, plotY + plotH / 2);
      ctx.textAlign = 'left';
      ctx.fillText('0.63', plotX, height - 3);
      ctx.textAlign = 'right';
      ctx.fillText('0.87', plotX + plotW, height - 3);
      return;
    }

    // bar：低效率端深紫 → 高效率端品红，颜色即数据（与场上 DECO 个体同色域）。
    const slot = plotW / n;
    const barW = Math.max(1, slot - 2);
    for (let i = 0; i < n; i++) {
      const c = counts[i];
      if (c <= 0) continue;
      const h = Math.max(1, Math.round((c / maxCount) * (plotH - 2)));
      const [r, g, b] = hslToRgb(bucketCenterHue(i, n), decoSat, decoLight);
      ctx.fillStyle = `rgb(${r},${g},${b})`;
      ctx.fillRect(plotX + i * slot + (slot - barW) / 2, plotY + plotH - h, barW, h);
    }

    // y 峰值 + x 端点 hue。
    ctx.fillStyle = 'rgba(207,227,214,0.55)';
    ctx.textAlign = 'left';
    ctx.fillText(String(maxCount), plotX + 2, plotY + 9);
    ctx.fillText('0.63', plotX, height - 3);
    ctx.textAlign = 'right';
    ctx.fillText('0.87', plotX + plotW, height - 3);
  }

  return {
    canvas,
    update,
    setVisible,
    get visible() { return visible; },
  };
}

// ─── Spectrogram overlay（Phase 11）────────────────────────────────────────────
// 竖向 heatmap：横向 = DECO hue bucket [0, 23]，纵向 = elapsed-time frames。
// 环形 buffer 持有最近 HISTORY_FRAMES 帧桶计数；顶部（y=0）为最新帧，底部
// 为最老帧。用 ImageData + putImageData 整块写入，避免逐 rect 开销。
//
// 颜色：每列 base hue 固定为 bucketCenterHue，饱和 s/l 来自 DECO 物种参数；
// 亮度由该列在本次全局 max 下的相对计数线性插值决定（暗 = 稀少，亮 = 富集）。
//
// 注意：与 createEvolutionHistogramOverlay 独立共存，互不干扰。
//
// Phase 12：随机访问回溯。buffer 除 pushFrame/render 外增加 getFrameAt(offset)
// 与 getEarliestReadable()：offset=0 最新帧、1 上一帧、负数向前（越早）；
// frameLen 记录累计 push 次数（ring 未满时读数不绕到脏数据）。

export const SPECTROGRAM_HISTORY_FRAMES = 720;
export const SPECTROGRAM_WIDTH = 320;
export const SPECTROGRAM_HEIGHT = SPECTROGRAM_HISTORY_FRAMES; // 1 px/frame，完整展示全部历史

// Phase 13 标注换算：1 张光谱帧 = HIST_EVERY(5) × TICKS_PER_FRAME(2) = 10 sim tick
//（采样粒度，与 wasm-viewer.mjs 的 HIST_EVERY / TICKS_PER_FRAME 同步）。
export const HIST_FRAME_TICKS = 10;

// 帧号 → sim tick 线性换算。默认 ticksPerFrame=2（视口帧粒度）；光谱帧传 HIST_FRAME_TICKS。
export function getTickFromFrame(frame, ticksPerFrame = 2) {
  return Math.round(frame) * ticksPerFrame;
}

// Y 轴刻度规划（纯函数，drawYTicks 消费、Node 可测）：每 every 帧一条，
// y = back×scale（back = 距最新帧步数），tick = newestTick − back×HIST_FRAME_TICKS。
// 落在顶部标题带（y<20）或底部 chrome 带（y>height−16）的刻度跳过。
export function spectrogramYTicks({ frameLen, historyFrames, height, newestTick, every = 120 }) {
  const lines = [];
  if (!(frameLen > 0) || !(height > 0)) return lines;
  const scale = historyFrames / height;
  for (let back = every; back < frameLen; back += every) {
    const y = Math.round(back * scale);
    if (y < 20 || y > height - 16) continue;
    lines.push({ back, y, tick: newestTick - getTickFromFrame(back, HIST_FRAME_TICKS) });
  }
  return lines;
}

export function createSpectrogramOverlay(opts = {}) {
  const width = opts.width ?? SPECTROGRAM_WIDTH;
  const height = opts.height ?? SPECTROGRAM_HEIGHT;
  const left = opts.left ?? 20;
  const top = opts.top ?? 190; // 叠在 Phase 10 histogram 下方
  const decoSat = opts.saturation ?? 0.47;
  const decoLightHigh = opts.lightness ?? 0.70;
  const historyFrames = opts.historyFrames ?? SPECTROGRAM_HISTORY_FRAMES;
  const histBuckets = DECO_HUE_BUCKETS;
  const PAD = 12;
  const GUTTER_W = 46; // 右侧 Y 轴刻度带（8px 刻度线 + tick 数字），不占热图宽度
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const canvas = document.createElement('canvas');
  canvas.id = 'deco-spectrogram-overlay';
  canvas.width = Math.round((width + GUTTER_W) * dpr);
  canvas.height = Math.round(height * dpr);
  canvas.style.cssText = [
    'position:fixed', `left:${left}px`, `top:${top}px`,
    `width:${width + GUTTER_W}px`, `height:${height}px`,
    'z-index:25', 'pointer-events:none', 'display:none',
  ].join(';');
  document.body.appendChild(canvas);
  const ctx = canvas.getContext('2d');
  ctx.scale(dpr, dpr);

  let visible = false;
  const setVisible = (v) => {
    visible = v;
    canvas.style.display = v ? '' : 'none';
  };

  // Ring buffer: flat Uint32Array，writeIdx 指向下一帧写入位置。
  // 布局：buffer[row * histBuckets + col]
  const buffer = new Uint32Array(historyFrames * histBuckets);
  let writeIdx = 0;
  let frameLen = 0; // 已 push 的总帧数（ring 未满时 < historyFrames）

  function pushFrame(counts) {
    const offset = writeIdx * histBuckets;
    for (let b = 0; b < histBuckets; b++) buffer[offset + b] = counts[b];
    writeIdx = (writeIdx + 1) % historyFrames;
    if (frameLen < historyFrames) frameLen += 1;
  }

  // 随机访问（时间轴约定）：offset=0 最新帧，-k / +k 往前 k 帧。
  // 使用 writeIdx 直接算物理槽位，与 ring 是否满无关。
  // 返回该帧 24 桶的拷贝（slice，不共享底层 buffer）。
  function getFrameAt(offset) {
    if (frameLen === 0) return null;
    const back = Math.abs(offset); // 距离最新帧多少步
    if (back >= frameLen) return null; // 超出可用历史
    // 最新帧的物理槽 = (writeIdx - 1 + cap) % cap
    // 往回 back 步 = 槽位减 back，再 mod cap
    const newestSlot = ((writeIdx - 1) % historyFrames + historyFrames) % historyFrames;
    const slot = ((newestSlot - back) % historyFrames + historyFrames) % historyFrames;
    const start = slot * histBuckets;
    return buffer.slice(start, start + histBuckets);
  }

  // 最早可读帧相对最新帧的偏移（负数或 0）：-(frameLen-1)；空 buffer 返回 0。
  // 与 getFrameAt 的约定一致：earliest ≤ offset ≤ 0 都可读。
  function getEarliestReadable() {
    if (frameLen === 0) return 0;
    return -(frameLen - 1);
  }

  // 预计算每列的 base RGB（基于 bucket center hue，固定不变）。
  const colRgb = new Array(histBuckets);
  for (let b = 0; b < histBuckets; b++) {
    const h = bucketCenterHue(b, histBuckets);
    colRgb[b] = hslToRgb(h, decoSat, decoLightHigh);
  }
  // 备用的暗沉底色（count=0 时的填充色）。
  const darkBg = hslToRgb(0.75, decoSat, 0.18);

  // 把「ring buffer → canvas ImageData」的热图像素批量生成。scale = historyFrames / height
  //（每像素代表的 buffer 行数）；newestAbs = writeIdx - 1（最新帧的物理偏移，mod 已好）。
  // 列 → 桶映射：画布宽 width=320 ≠ 桶数 24，每桶占 width/histBuckets ≈ 13px 条带；
  // 直接用列号索引 buffer/colRgb 会越界（Phase 11 遗留缺陷，浏览器 boot 即崩）。
  function makeImage(newestAbs, scale, norm) {
    const imgData = ctx.createImageData(width, height);
    const px = imgData.data;
    for (let r = 0; r < height; r++) {
      const back = Math.round(r * scale);
      const bufRow = ((newestAbs - back) % historyFrames + historyFrames) % historyFrames;
      const bOff = bufRow * histBuckets;
      for (let c = 0; c < width; c++) {
        const b = Math.min(histBuckets - 1, Math.floor((c * histBuckets) / width));
        const cnt = buffer[bOff + b];
        const brightness = cnt > 0 ? Math.min(1, cnt * norm) : 0;
        const k = r * width + c;
        px[k * 4]     = Math.round(darkBg[0] * (1 - brightness) + colRgb[b][0] * brightness);
        px[k * 4 + 1] = Math.round(darkBg[1] * (1 - brightness) + colRgb[b][1] * brightness);
        px[k * 4 + 2] = Math.round(darkBg[2] * (1 - brightness) + colRgb[b][2] * brightness);
        px[k * 4 + 3] = 255;
      }
    }
    return imgData;
  }

  // 绘制面板 chrome（标题、刻度等，不触发热图重绘）——在热图 putImageData 之后叠加。
  function drawChrome(tickLabel, showAxes = true) {
    const w = width;
    const h = height;
    ctx.font = '10px "SF Mono", ui-monospace, monospace';
    ctx.textBaseline = 'alphabetic';
    ctx.textAlign = 'left';
    ctx.fillStyle = '#cfe3d6';
    ctx.fillText('DECO hue evolution', PAD, 14);
    ctx.textAlign = 'right';
    ctx.fillStyle = 'rgba(207,227,214,0.55)';
    ctx.fillText(tickLabel, w - PAD, 14);
    if (!showAxes) return;
    ctx.font = '9px "SF Mono", ui-monospace, monospace';
    ctx.textAlign = 'left';
    ctx.fillStyle = 'rgba(207,227,214,0.45)';
    ctx.fillText('0.63', PAD, h - 3);
    ctx.textAlign = 'right';
    ctx.fillText('0.87', w - PAD, h - 3);
    ctx.textAlign = 'left';
    ctx.fillText('↑ newest', PAD, h - 14);
    ctx.textAlign = 'right';
    ctx.fillText('oldest ↓', w - PAD, h - 14);
  }

  // Phase 13：右侧 gutter 画 Y 轴刻度（每 120 帧一条细白线 + tick 数字，热图区不受影响）。
  function drawYTicks(newestTick) {
    const lines = spectrogramYTicks({ frameLen, historyFrames, height, newestTick });
    if (lines.length === 0) return;
    ctx.font = '10px "SF Mono", ui-monospace, monospace';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'middle';
    ctx.lineWidth = 1;
    for (const { y, tick } of lines) {
      ctx.strokeStyle = 'rgba(255,255,255,0.35)';
      ctx.beginPath();
      ctx.moveTo(width + 0.5, y + 0.5);
      ctx.lineTo(width + 8.5, y + 0.5);
      ctx.stroke();
      ctx.fillStyle = 'rgba(255,255,255,0.75)';
      ctx.fillText(String(tick), width + 11, y + 0.5);
    }
    ctx.textBaseline = 'alphabetic';
  }

  // Phase 14: 清空 ring buffer 并重绘空面板（R 键清除快照、回到 live 模式时调用）。
  function clear() {
    buffer.fill(0);
    writeIdx = 0;
    frameLen = 0;
    if (!visible) return;
    const fullW = width + GUTTER_W;
    ctx.clearRect(0, 0, fullW, height);
    ctx.fillStyle = 'rgba(10,16,18,0.55)';
    ctx.strokeStyle = 'rgba(210,232,220,0.16)';
    ctx.lineWidth = 1;
    panelPath(ctx, fullW, height, 10);
    ctx.fill();
    ctx.stroke();
    drawChrome('t=—', false);
    ctx.font = '10px "SF Mono", ui-monospace, monospace';
    ctx.fillStyle = 'rgba(207,227,214,0.5)';
    ctx.textAlign = 'center';
    ctx.fillText('cleared · waiting for live frames', fullW / 2, height / 2);
    ctx.textAlign = 'left';
  }

  // live 模式：直接读 ring buffer，画完整光谱图。
  function render(maxCount, tickCount) {
    if (!visible || maxCount <= 0) return;
    const fullW = width + GUTTER_W;
    const h = height;
    ctx.clearRect(0, 0, fullW, h);
    ctx.fillStyle = 'rgba(10,16,18,0.55)';
    ctx.strokeStyle = 'rgba(210,232,220,0.16)';
    ctx.lineWidth = 1;
    panelPath(ctx, fullW, h, 10);
    ctx.fill();
    ctx.stroke();
    const scale = historyFrames / h;
    const norm = 1 / maxCount;
    ctx.putImageData(makeImage(writeIdx - 1, scale, norm), 0, 0);
    drawChrome(`t=${tickCount}`);
    drawYTicks(tickCount);
  }

  // scrub 模式：同 live 模式热图，额外在选中帧处覆盖红色指示线。
  // frameOffset 允许传负 back-offset（viewer 直传 scrubber.offset），内部 abs 归一。
  function renderScrubbed(frameOffset, maxCount, tickCount) {
    if (!visible || frameLen === 0) return;
    const fullW = width + GUTTER_W;
    const h = height;
    ctx.clearRect(0, 0, fullW, h);
    ctx.fillStyle = 'rgba(10,16,18,0.55)';
    ctx.strokeStyle = 'rgba(210,232,220,0.16)';
    ctx.lineWidth = 1;
    panelPath(ctx, fullW, h, 10);
    ctx.fill();
    ctx.stroke();
    const scale = historyFrames / h;
    const norm = 1 / maxCount;
    ctx.putImageData(makeImage(writeIdx - 1, scale, norm), 0, 0);
    // 高亮行（back=0 → y=0）。scale 下采样时线性映射像素位置。
    const back = Math.abs(frameOffset);
    const y = Math.min(h - 1, Math.round(back * scale));
    ctx.fillStyle = 'rgba(255,96,96,0.9)';
    ctx.fillRect(PAD, y, width - PAD * 2, 2);
    ctx.fillStyle = 'rgba(255,255,255,1.0)';
    ctx.fillRect(PAD, y - 1, 10, 4);
    drawChrome(back > 0 ? `t=${tickCount} · frame -${back}` : `t=${tickCount}`);
    drawYTicks(tickCount);
  }

  return {
    canvas,
    pushFrame,
    clear,
    getFrameAt,
    getEarliestReadable,
    render,
    renderScrubbed,
    setVisible,
    get visible() { return visible; },
    get writeIdx() { return writeIdx; },
    get frameLen() { return frameLen; },
    get bufferSize() { return historyFrames; },
  };
}
