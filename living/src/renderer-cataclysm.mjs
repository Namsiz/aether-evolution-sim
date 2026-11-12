// renderer-cataclysm.mjs — Phase 17 灾变可视化 helper（纯函数 + DOM overlay）。
//
// 数据流：wasm-viewer 通过 sim_cataclysm_count / sim_get_cataclysm 拉取 ABI
// 记录，渲染层负责画符号与 HUD。纯函数路径可被 Node smoke test 直接消费。
//
// 图标语义（canvas 绘制，不用 emoji 字符）：
//   ☄ impact — 红色彗星：实心圆 + 三道尾迹，暗示撞击方向
//   🌡 climate — 橙色温度计：竖杆 + 底部圆泡，暗示温度变化
//
// 光谱图时间轴是纵向的（y=0 = 最新帧），所以灾变划线在该面板上是横线；
// phylogeny 时间轴横向，划线为竖线。

export const CATACLYSM_IMPACT = 0;
export const CATACLYSM_CLIMATE = 1;
export const CATACLYSM_VERSION = 1;
/// 与 wasm-viewer.mjs 的光谱采样节奏同步（每 5 帧 × 2 ticks/frame = 10 ticks/帧）
const _HIST_FRAME_TICKS = 10;
/// spectrogram ring buffer 深度（与 renderer-evolution-histogram.mjs 对齐）
const _SPECTROGRAM_HISTORY_FRAMES = 720;

// ─── glyph 绘制（Node/DOM 均可）─────────────────────────────────────────────

/// 在 (x, y) 画一小段灾变类型图标（size px 级），用于面板标题区点缀。
export function drawCataclysmGlyph(ctx, kind, x, y, size = 5) {
  if (kind === CATACLYSM_IMPACT) {
    // 彗星：3 条左上方尾迹 + 实心圆头。
    ctx.strokeStyle = 'rgba(255,96,64,0.85)';
    ctx.lineWidth = 1.2;
    for (let k = 1; k <= 3; k++) {
      ctx.beginPath();
      ctx.moveTo(x - k * size * 0.8, y - k * size * 0.8);
      ctx.lineTo(x - k * size * 0.8 + size * 0.5, y - k * size * 0.8 + size * 0.5);
      ctx.stroke();
    }
    ctx.fillStyle = '#ff5a3c';
    ctx.beginPath();
    ctx.arc(x, y, size * 0.75, 0, Math.PI * 2);
    ctx.fill();
  } else {
    // 温度计：竖杆 + 底部圆泡。
    ctx.strokeStyle = '#ffb347';
    ctx.lineWidth = 1.6;
    ctx.beginPath();
    ctx.moveTo(x, y - size);
    ctx.lineTo(x, y + size * 0.4);
    ctx.stroke();
    ctx.fillStyle = '#ffb347';
    ctx.beginPath();
    ctx.arc(x, y + size * 0.75, size * 0.6, 0, Math.PI * 2);
    ctx.fill();
  }
}

/// 竖线标记（phylogeny：x 轴 = tick 时间轴）
export function drawCataclysmVline(ctx, kind, x, y0, y1) {
  ctx.strokeStyle = kind === CATACLYSM_IMPACT ? 'rgba(255,96,64,0.55)' : 'rgba(255,179,71,0.55)';
  ctx.lineWidth = 1;
  ctx.setLineDash([3, 3]);
  ctx.beginPath();
  ctx.moveTo(x + 0.5, y0);
  ctx.lineTo(x + 0.5, y1);
  ctx.stroke();
  ctx.setLineDash([]);
}

/// 横线标记（spectrogram：y 轴 = back × HIST_FRAME_TICKS ≈ tick）
export function drawCataclysmHline(ctx, kind, y, x0, x1) {
  ctx.strokeStyle = kind === CATACLYSM_IMPACT ? 'rgba(255,96,64,0.55)' : 'rgba(255,179,71,0.55)';
  ctx.lineWidth = 1;
  ctx.setLineDash([3, 3]);
  ctx.beginPath();
  ctx.moveTo(x0, y + 0.5);
  ctx.lineTo(x1, y + 0.5);
  ctx.stroke();
  ctx.setLineDash([]);
}

// ─── 纯函数：坐标映射（Node 可测）─────────────────────────────────────────────

/// 将 ABI 拉的 cataclysm 记录集合映射成光谱/系统发育面板各自的标记坐标。
/// phylogeny：nx ∈ [0,1]（同 buildPhylogenyGraph 的 nx = (tick − rootTick)/span）；
/// spectrogram：back = (newestTick − tick) / 10（距最新帧的帧数）。
export function cataclysmMarkPositions(records, opts = {}) {
  const rootTick = opts.rootTick ?? 0;
  const currentTick = opts.currentTick ?? 0;
  const newestTick = opts.newestTick ?? currentTick;
  const frameTicks = opts.frameTicks ?? _HIST_FRAME_TICKS;
  const historyFrames = opts.historyFrames ?? _SPECTROGRAM_HISTORY_FRAMES;
  const span = Math.max(1, currentTick - rootTick);
  const marks = [];
  for (const c of records) {
    if (c.tick < rootTick) continue;
    const phyloX = (c.tick - rootTick) / span;
    const back = (newestTick - c.tick) / frameTicks;
    marks.push({
      id: c.id, kind: c.kind, tick: c.tick,
      phyloX,
      specBack: back,
      inSpectrogram: back >= 0 && back < historyFrames,
    });
  }
  return { version: CATACLYSM_VERSION, marks };
}

// ─── HUD badge（纯 HTML 字符串，DOM 无关）────────────────────────────────────

/// 活跃徽标 + 最近 5 条历史（每条带 ☄ / 🌡 文本图标，与规范一致）。
/// records：由 ABI 拉取的 [{ id, kind, tick, remaining, active, ... }]，
/// dustMult：当前环境乘数（JS 从 sim_env_mods 读取）。
export function formatCataclysmHud(records, dustMult) {
  const active = records.filter((c) => c.active);
  let out = '';
  if (active.length > 0) {
    const a = active[0];
    const icon = a.kind === CATACLYSM_IMPACT ? '☄' : '🌡';
    out += `<span style="color:${a.kind === CATACLYSM_IMPACT ? '#ff8f6b' : '#ffb347'}">`
      + `${icon} t=${a.tick} dust=${(dustMult ?? 1).toFixed(2)} · 还剩 ${a.remaining}t</span><br>`;
  }
  if (records.length > 0) {
    const recent = records.slice(-5).map((c) =>
      `${c.kind === CATACLYSM_IMPACT ? '☄' : '🌡'}#${c.id}@t=${c.tick}${c.active ? '' : '†'}`).join(' · ');
    out += `<span style="opacity:0.72">灾变: ${recent}</span><br>`;
  }
  return out;
}

// ─── 全屏烟尘 overlay（DOM）──────────────────────────────────────────────────

/// 创建固定定位的全屏烟尘覆盖 div，opacity 由 setDust() 驱动（径向渐变模拟
/// 中央遮蔽感）。pointer-events:none → 不打断任何点击交互。
export function createDustOverlay(opts = {}) {
  const el = document.createElement('div');
  el.id = 'cataclysm-dust-overlay';
  el.style.cssText = [
    'position:fixed', 'inset:0', 'z-index:5', 'pointer-events:none',
    'background:radial-gradient(circle at 50% 40%, rgba(146,138,124,0.9), rgba(84,80,72,0.95))',
    'opacity:0', 'display:none', 'transition:opacity 0.4s',
  ].join(';');
  document.body.appendChild(el);
  return {
    el,
    setDust(dust) {
      const a = Math.max(0, Math.min(1, 1 - dust));
      if (a < 0.01) { el.style.display = 'none'; return; }
      el.style.display = '';
      el.style.opacity = (a * 0.5).toFixed(3);
    },
  };
}
