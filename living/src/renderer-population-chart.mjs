// renderer-population-chart.mjs — Phase 25: population trend chart overlay.
//
// Draws a simple line chart of the last N frames per species kind.
// Data is pushed every few frames; rendered on an independent canvas
// overlaid on top of the Three.js scene (position:fixed, pointer-events:none).
//
// No WASM changes required. Pure JS canvas drawing.

const POP_HISTORY_LEN = 200; // frames to keep in history
const POP_COLORS = {
  herb: '#4ade80', // green
  pred: '#f87171', // red
  scav: '#fbbf24', // yellow
  omn:  '#60a5fa', // blue
  fish: '#38bdf8', // cyan
  deco: '#c084fc', // purple
};

export function createPopulationChartOverlay() {
  const width = 280;
  const height = 160;
  const right = 20;
  const bottom = 190;
  const PAD = { top: 20, bottom: 24, left: 40, right: 10 };

  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const canvas = document.createElement('canvas');
  canvas.id = 'population-chart-overlay';
  canvas.width = Math.round(width * dpr);
  canvas.height = Math.round(height * dpr);
  canvas.style.cssText = [
    'position:fixed',
    `right:${right}px`,
    `bottom:${bottom}px`,
    `width:${width}px`,
    `height:${height}px`,
    'z-index:10',
    'pointer-events:none',
    'display:none',
  ].join(';');
  document.body.appendChild(canvas);
  const ctx = canvas.getContext('2d');
  ctx.scale(dpr, dpr);

  const history = new Map(); // kind -> Uint16Array[POP_HISTORY_LEN]
  for (const k of ['herb','pred','scav','omn','fish','deco']) {
    history.set(k, new Uint16Array(POP_HISTORY_LEN));
  }
  let histIdx = 0;
  let popOn = true;

  function pushFrame(counts) {
    // counts: [herb, pred, scav, omn, fish, deco]
    for (let i = 0; i < 6; i++) {
      const kind = ['herb','pred','scav','omn','fish','deco'][i];
      const arr = history.get(kind);
      arr[histIdx % POP_HISTORY_LEN] = counts[i] | 0;
    }
    histIdx++;
  }

  function getPeak() {
    let peak = 0;
    for (const arr of history.values()) {
      for (let i = 0; i < arr.length; i++) if (arr[i] > peak) peak = arr[i];
    }
    return Math.max(1, peak);
  }

  function render(overlayData) {
    if (!popOn) return;
    const w = width, h = height;
    ctx.clearRect(0, 0, w, h);

    // Background panel
    ctx.fillStyle = 'rgba(10,16,18,0.7)';
    ctx.strokeStyle = 'rgba(210,232,220,0.16)';
    ctx.lineWidth = 1;
    roundRect(ctx, 0, 0, w, h, 8);
    ctx.fill();
    ctx.stroke();

    // Title
    ctx.font = 'bold 10px "SF Mono", ui-monospace, monospace';
    ctx.fillStyle = '#e9f2ec';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'alphabetic';
    ctx.fillText('POPULATION', PAD.left, 14);

    const pe = overlayData?.activitySummary;
    if (pe && +pe.pctBurst >= 50) {
      ctx.fillStyle = '#ffb347';
      ctx.fillText('· PE', PAD.left + 80, 14);
    }

    const peak = getPeak();
    const plotW = w - PAD.left - PAD.right;
    const plotH = h - PAD.top - PAD.bottom;
    const visibleLen = Math.min(POP_HISTORY_LEN, histIdx);
    const startX = PAD.left;
    const stepX = plotW / Math.max(1, POP_HISTORY_LEN - 1);

    // Draw axes
    ctx.strokeStyle = 'rgba(207,227,214,0.25)';
    ctx.beginPath();
    ctx.moveTo(PAD.left, PAD.top);
    ctx.lineTo(PAD.left, h - PAD.bottom);
    ctx.lineTo(w - PAD.right, h - PAD.bottom);
    ctx.stroke();

    // Y-axis labels
    ctx.font = '9px "SF Mono", ui-monospace';
    ctx.fillStyle = 'rgba(207,227,214,0.5)';
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';
    for (let pct = 0; pct <= 100; pct += 50) {
      const y = h - PAD.bottom - (pct / 100) * plotH;
      ctx.fillText(`${Math.round(peak * pct / 100)}`, PAD.left - 4, y);
    }

    // Draw lines
    const kinds = ['herb','pred','scav','omn','fish','deco'];
    for (const kind of kinds) {
      const arr = history.get(kind);
      if (arr.every(v => v === 0)) continue;
      ctx.strokeStyle = POP_COLORS[kind];
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      let started = false;
      const offset = POP_HISTORY_LEN - visibleLen;
      for (let i = 0; i < visibleLen; i++) {
        const val = arr[(offset + i) % POP_HISTORY_LEN];
        const x = startX + i * stepX;
        const y = h - PAD.bottom - (val / peak) * plotH;
        if (!started) { ctx.moveTo(x, y); started = true; }
        else ctx.lineTo(x, y);
      }
      ctx.stroke();
    }

    // Legend at bottom
    ctx.font = '9px "SF Mono", ui-monospace';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'alphabetic';
    let lx = PAD.left;
    for (const k of kinds) {
      const lastVal = arr?.[POP_HISTORY_LEN - 1] ?? 0;
      const arrK = history.get(k);
      const last = arrK[arrK.length - 1];
      if (last === 0) continue;
      ctx.fillStyle = POP_COLORS[k];
      ctx.fillRect(lx, h - 10, 8, 6);
      ctx.fillStyle = 'rgba(207,227,214,0.7)';
      ctx.fillText(`${k.slice(0,3)} ${last}`, lx + 11, h - 5);
      lx += ctx.measureText(`${k} 999`).width + 16;
    }
  }

  function setVisible(v) {
    popOn = v;
    canvas.style.display = v ? '' : 'none';
    if (v) render(null);
  }

  function roundRect(ctx, x, y, w, h, r) {
    ctx.beginPath();
    ctx.moveTo(x + r, y);
    ctx.lineTo(x + w - r, y);
    ctx.quadraticCurveTo(x + w, y, x + w, y + r);
    ctx.lineTo(x + w, y + h - r);
    ctx.quadraticCurveTo(x + w, y + h, x + w - r, y + h);
    ctx.lineTo(x + r, y + h);
    ctx.quadraticCurveTo(x, y + h, x, y + h - r);
    ctx.lineTo(x, y + r);
    ctx.quadraticCurveTo(x, y, x + r, y);
    ctx.closePath();
  }

  return { canvas, update: (counts) => { pushFrame(counts); render(null); }, setVisible, render, get visible() { return popOn; } };
}
