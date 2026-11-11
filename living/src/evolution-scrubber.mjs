// evolution-scrubber.mjs — DECO hue 演化光谱条下方的可交互时间轴拖拽条。
//
// 视觉设计：
//   - 深灰半透明圆角轨道（width ≈ spectrogram.width，高度 MIN_CANVAS_H = 58）
//   - 蓝色进度填充从左到右；当前位置用亮蓝竖线 + 小三角手柄表示
//   - 顶行：播放时标 start/now；scrub 时在指针旁显示蓝色 t= 绝对读数
//   - Phase 13 X 轴刻度：每 60 帧一个 mark（bar 下方 2px），每 120 帧标 t=（上下交错）
//   - hover 任意位置显示 frame/tick 气泡（canvas 绘制，无 DOM）
//
// 交互：
//   - mousedown → track movement → updatePointer(offset)
//   - mouseup → snap to nearest valid frame
//   - click on bar → jump immediately
//   - ← → 逐帧步进（-/+ FRAME_STEP），需要 watcher 监听后由 viewer 处理
//   - SPACE 切换 play/pause
//   - HOME / END 跳至最早 / 最新可用帧
//
// 与 Phase 11 spectrogram 配合：
//   - 播放模式：pointer 在末端（offset=0），spectrogram 调用 render() 实时更新
//   - Scrub 模式：pointer 在任意位置（offset < 0），spectrogram 调用 renderScrubbed(offset)
//     并在该帧画高亮行，histogram overlay 刷新为该帧的桶分布
//
// 零 DOM 依赖：只操作 canvas 2D context，挂于 document.body 绝对定位上。

import { HIST_FRAME_TICKS, getTickFromFrame } from './renderer-evolution-histogram.mjs';

const MIN_CANVAS_H = 58;
const MIN_CANVAS_W = 320; // 与 SPECTROGRAM_WIDTH 对齐
const HIGHLIGHT_COL = 'rgba(72,181,255,0.92)';
const TRACK_COL = 'rgba(30,40,60,0.78)';
const TRACK_BORDER = 'rgba(120,180,220,0.25)';
const HANDLE_H = 22;      // 指针总高（含上下三角延伸）
const HANDLE_W = 4;       // 指针竖线宽度
const PAD_X = 14;         // 水平内边距
const TRACK_TOP = 16;     // 轨道顶（上方留 16px 给 start/now 与蓝色 t= 读数）
const TRACK_H = 12;       // 轨道高

// X 轴刻度规划（纯函数，draw 消费、Node 可测）：每 every 帧一个 mark，
// x 沿 track 线性映射（f=0 最老 → f=frameLen−1 最新）；
// tick = tickAnchor − (frameLen−1−f)×HIST_FRAME_TICKS；
// label 仅在 f 为 2×every 整数倍时给出（60 帧间距 ≈24px，逐条标注会互相碰撞）。
export function scrubberXTicks({ frameLen, trackLeft, trackW, tickAnchor, every = 60 }) {
  const marks = [];
  if (frameLen < 2) return marks;
  for (let f = every; f < frameLen - 1; f += every) {
    const back = frameLen - 1 - f;
    const tick = tickAnchor - getTickFromFrame(back, HIST_FRAME_TICKS);
    marks.push({
      f,
      x: trackLeft + (f / (frameLen - 1)) * trackW,
      tick,
      label: (f / every) % 2 === 0 ? `t=${tick}` : null,
    });
  }
  return marks;
}

function roundRect(ctx, x, y, w, h, r) {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.lineTo(x + w - r, y);
  ctx.quadraticCurveTo(x + w, y, x + w, y + r);
  ctx.lineTo(x + w, h - r);
  ctx.quadraticCurveTo(x + w, h, x + w - r, h);
  ctx.lineTo(x + r, h);
  ctx.quadraticCurveTo(x, h, x, h - r);
  ctx.lineTo(x, y + r);
  ctx.quadraticCurveTo(x, y, x + r, y);
  ctx.closePath();
}

/**
 * @param {{ historyBuf: import('./renderer-evolution-histogram.mjs').SpectrogramOverlay,
 *           width?: number, height?: number }} opts
 */
export function createEvolutionScrubber(opts) {
  const historyBuf = opts.historyBuf;
  if (!historyBuf || typeof historyBuf.frameLen !== 'number') {
    throw new TypeError('createEvolutionScrubber requires a valid spectrogram history buffer');
  }

  const W = Math.max(MIN_CANVAS_W, opts.width ?? 320);
  const H = opts.height ?? MIN_CANVAS_H;
  const dpr = Math.min(window.devicePixelRatio || 1, 2);

  const canvas = document.createElement('canvas');
  canvas.id = 'deco-scrubber-overlay';
  canvas.style.cssText = [
    'position:fixed',
    'left:20px',               // 与 spectrogram overlay 左对齐
    'top:910px',               // 初始占位；wasm-viewer 会在创建时修正为 specOverlay 正下方
    `width:${W}px`,
    `height:${H}px`,
    'z-index:25',
    'pointer-events:auto',     // 需要鼠标交互
    'display:none',
  ].join(';');
  document.body.appendChild(canvas);
  const ctx = canvas.getContext('2d');
  ctx.scale(dpr, dpr);

  // ── state ────────────────────────────────────────────────────────────────────
  let visible = false;
  let playing = true;        // 默认 play；进入 scrub 模式时由 wasm-viewer 置 false
  let offset = 0;            // back offset（0 = 最新帧，负值往回）
  let dirty = true;          // 标记是否需要重绘
  let dragging = false;
  let tickAnchor = 0;        // 最新帧的 sim tick（由 viewer 在每次 push 后同步）
  let hoverX = null;         // hover 预览位置（canvas 局部 x）；null = 无 hover

  /** 外部注册：状态变更回调（viewer 用来切 render/renderScrubbed）。 */
  let _onChange = null;
  const setOnChange = (fn) => { _onChange = fn; };

  const setVisible = (v) => {
    visible = v;
    canvas.style.display = v ? '' : 'none';
    dirty = true;
    if (v) bindEvents(); else cleanupEvents();
  };

  const snapOffset = (raw) => {
    const earliest = historyBuf.getEarliestReadable(); // ≤ 0，或 0 当空
    return Math.max(earliest, Math.min(0, raw));
  };

  /** 由 clientX 推算 offset 并通知外部（用于拖拽） */
  function updatePointer(clientX) {
    const rect = canvas.getBoundingClientRect();
    const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / W));
    const earliest = historyBuf.getEarliestReadable();
    // ratio=0（左边缘）→ 最老；ratio=1（右边缘）→ newest(0)
    offset = snapOffset(Math.round(earliest * (1 - ratio)));
    dirty = true;
    if (_onChange) _onChange(!playing && offset !== 0, offset);
  }

  /** 由外部分配绝对帧序号（0-based 从最老开始），转换为 back-offset 内部表示 */
  function updatePointerFromFrame(frameIdx) {
    const total = historyBuf.frameLen;
    if (total === 0) { offset = 0; dirty = true; return; }
    offset = -(total - 1 - frameIdx);
    offset = snapOffset(offset);
    dirty = true;
    if (_onChange) _onChange(false, offset);
  }

  function isPlaying() { return playing; }
  function play() {
    if (playing) return;
    playing = true;
    offset = 0;
    dirty = true;
    if (_onChange) _onChange(true, 0);
  }
  function pause() {
    if (!playing) return;
    playing = false;
    dirty = true;
    if (_onChange) _onChange(false, offset);
  }

  /** viewer 在每次光谱 pushFrame 后同步最新帧的 sim tick（刻度标注的时间锚点）。 */
  const setTickAnchor = (t) => {
    if (t !== tickAnchor) {
      tickAnchor = t;
      dirty = true;
    }
  };

  // ── 渲染 ─────────────────────────────────────────────────────────────────────
  function draw() {
    if (!visible || !dirty) return;
    dirty = false;
    ctx.clearRect(0, 0, W, H);

    const available = historyBuf.frameLen;
    if (available === 0) {
      ctx.fillStyle = 'rgba(207,227,214,0.4)';
      ctx.font = '10px "SF Mono", ui-monospace, monospace';
      ctx.textAlign = 'center';
      ctx.fillText('push frames first', W / 2, H / 2 + 3);
      return;
    }

    const earliest = historyBuf.getEarliestReadable(); // ≤ 0
    const trackTop = TRACK_TOP;
    const trackBot = TRACK_TOP + TRACK_H;
    const trackH = TRACK_H;
    const trackLeft = PAD_X;
    const trackRight = W - PAD_X;
    const trackW = trackRight - trackLeft;

    // 轨道底。
    ctx.fillStyle = TRACK_COL;
    roundRect(ctx, trackLeft, trackTop, trackW, trackH, 3);
    ctx.fill();
    ctx.strokeStyle = TRACK_BORDER;
    ctx.lineWidth = 0.8;
    ctx.stroke();

    // 已填充段（从最左延伸到当前指针）。
    // fillRatio ∈ [0, 1]；offset=0（now）→ 满；offset=earliest → 最小非零
    const fillRatio = available <= 1 ? 1 : Math.abs(offset) / (available - 1);
    const fillW = Math.max(fillRatio * trackW, 3);
    const grad = ctx.createLinearGradient(trackLeft, 0, trackLeft + fillW, 0);
    grad.addColorStop(0, 'rgba(42,130,210,0.78)');
    grad.addColorStop(1, HIGHLIGHT_COL);
    ctx.fillStyle = grad;
    roundRect(ctx, trackLeft, trackTop, fillW, trackH, 3);
    ctx.fill();

    // 滑块指针（竖线 + 上下三角）。
    const ptrX = trackLeft + fillW;
    const ptrMidY = trackTop + trackH / 2;
    // 竖线（贯穿整个轨道）。
    ctx.fillStyle = HIGHLIGHT_COL;
    ctx.fillRect(ptrX - HANDLE_W / 2, trackTop, HANDLE_W, trackH);
    // 上方三角（y < trackTop）。
    ctx.beginPath();
    ctx.moveTo(ptrX, trackTop - 5);
    ctx.lineTo(ptrX - 4, trackTop);
    ctx.lineTo(ptrX + 4, trackTop);
    ctx.closePath();
    ctx.fill();
    // 下方三角（y > trackBot）。
    ctx.beginPath();
    ctx.moveTo(ptrX, trackBot + 5);
    ctx.lineTo(ptrX - 4, trackBot);
    ctx.lineTo(ptrX + 4, trackBot);
    ctx.closePath();
    ctx.fill();

    // 顶部信息行：播放时标 start/now；scrub 时在指针旁显示蓝色 t= 绝对读数。
    ctx.font = '9px "SF Mono", ui-monospace, monospace';
    if (!playing && offset < 0) {
      const tick = tickAnchor - getTickFromFrame(-offset, HIST_FRAME_TICKS);
      ctx.font = 'bold 12px "SF Mono", ui-monospace, monospace';
      ctx.fillStyle = 'rgba(72,181,255,0.95)';
      ctx.textAlign = 'center';
      ctx.fillText(`t=${tick}`, Math.max(30, Math.min(W - 30, ptrX)), 12);
    } else {
      ctx.fillStyle = 'rgba(207,227,214,0.55)';
      ctx.textAlign = 'left';
      ctx.fillText('start', trackLeft, 12);
      ctx.textAlign = 'right';
      ctx.fillText('now', trackRight, 12);
    }

    // Phase 13：X 轴刻度——每 60 帧一个 mark，每 120 帧标 t=（上下交错避免碰撞）。
    const marks = scrubberXTicks({ frameLen: available, trackLeft, trackW, tickAnchor });
    ctx.fillStyle = 'rgba(255,255,255,0.35)';
    for (const m of marks) ctx.fillRect(Math.round(m.x), trackBot + 2, 1, 2);
    ctx.font = '8px "SF Mono", ui-monospace, monospace';
    ctx.textAlign = 'center';
    for (const m of marks) {
      if (m.label === null) continue;
      ctx.fillStyle = 'rgba(207,227,214,0.55)';
      ctx.fillText(m.label, m.x, ((m.f / 60) % 2 === 0) ? trackBot + 14 : trackBot + 24);
    }

    // hover 气泡：任意位置的 frame/tick 预览（不移动指针）。
    if (hoverX !== null && !dragging) {
      const ratio = Math.max(0, Math.min(1, hoverX / W));
      const f = Math.round((available - 1) * ratio);
      const back = available - 1 - f;
      const text = `frame ${f} · t=${tickAnchor - getTickFromFrame(back, HIST_FRAME_TICKS)}`;
      ctx.font = '8px "SF Mono", ui-monospace, monospace';
      const bw = ctx.measureText(text).width + 12;
      const bx = Math.max(2, Math.min(W - bw - 2, hoverX - bw / 2));
      ctx.fillStyle = 'rgba(10,16,18,0.85)';
      roundRect(ctx, bx, 1, bw, 14, 4);
      ctx.fill();
      ctx.strokeStyle = 'rgba(120,180,220,0.35)';
      ctx.lineWidth = 0.8;
      ctx.stroke();
      ctx.fillStyle = 'rgba(207,227,214,0.9)';
      ctx.textAlign = 'center';
      ctx.fillText(text, bx + bw / 2, 11);
    }
  }

  // ── 事件绑定 ─────────────────────────────────────────────────────────────────
  let _onMouseMove = null;
  let _onMouseUp = null;
  let _onCanvasMove = null;
  let _onCanvasLeave = null;

  function cleanupEvents() {
    if (_onMouseMove) { window.removeEventListener('mousemove', _onMouseMove); _onMouseMove = null; }
    if (_onMouseUp)   { window.removeEventListener('mouseup', _onMouseUp); _onMouseUp = null; }
    if (_onCanvasMove)  { canvas.removeEventListener('mousemove', _onCanvasMove); _onCanvasMove = null; }
    if (_onCanvasLeave) { canvas.removeEventListener('mouseleave', _onCanvasLeave); _onCanvasLeave = null; }
  }

  function bindEvents() {
    cleanupEvents();
    // Phase 13：hover 预览（不拖拽时跟随鼠标显示 frame/tick 气泡）。
    _onCanvasMove = (e) => {
      if (dragging) return;
      hoverX = e.clientX - canvas.getBoundingClientRect().left;
      dirty = true;
    };
    _onCanvasLeave = () => {
      hoverX = null;
      dirty = true;
    };
    canvas.addEventListener('mousemove', _onCanvasMove);
    canvas.addEventListener('mouseleave', _onCanvasLeave);
    canvas.addEventListener('mousedown', (e) => {
      e.preventDefault();
      dragging = true;
      updatePointer(e.clientX);
      _onMouseMove = (ev) => { if (dragging) updatePointer(ev.clientX); };
      _onMouseUp = () => {
        dragging = false;
        offset = snapOffset(offset);
        dirty = true;
        window.removeEventListener('mousemove', _onMouseMove);
        window.removeEventListener('mouseup', _onMouseUp);
        _onMouseMove = null;
        _onMouseUp = null;
      };
      window.addEventListener('mousemove', _onMouseMove);
      window.addEventListener('mouseup', _onMouseUp);
    });
    canvas.addEventListener('click', (e) => {
      if (!dragging) updatePointer(e.clientX);
    });
  }

  return {
    canvas,
    updatePointer,
    updatePointerFromFrame,
    isPlaying,
    play,
    pause,
    setTickAnchor,
    setVisible,
    setOnChange,
    get visible() { return visible; },
    get offset() { return offset; },
    get playing() { return playing; },
    // 每帧由 wasm-viewer 调用一次，内部做脏标记判断避免无谓重绘。
    draw,
    getDirty() { return dirty; },
    clearDirty() { dirty = false; },
    _bindEvents: bindEvents,
    _cleanupEvents: cleanupEvents,
  };
}
