// renderer-event-log.mjs — Phase 30: persistent evolution event log overlay.
//
// Collects speciation / extinction / cataclysm events from the sim and renders
// them as a scrollable text log in a small fixed-position canvas overlay.
// Events are time-stamped with sim tick and grouped by type.
//
// No WASM changes required. Pure JS canvas drawing.

const MAX_LOG_ENTRIES = 200;

const EVENT_COLORS = {
  speciation: '#8fd3ff',   // light blue
  extinction: '#f87171',   // red
  cataclysm:  '#ffb347',   // amber
  info:       'rgba(207,227,214,0.6)',
};

export function createEventLogOverlay() {
  const width = 260;
  const height = 180;
  const left = 10;
  const bottom = 10;
  const PAD = { top: 18, bottom: 6, left: 8, right: 8 };
  const LINE_H = 13;

  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const canvas = document.createElement('canvas');
  canvas.id = 'event-log-overlay';
  canvas.width = Math.round(width * dpr);
  canvas.height = Math.round(height * dpr);
  canvas.style.cssText = [
    'position:fixed',
    `left:${left}px`,
    `bottom:${bottom}px`,
    `width:${width}px`,
    `height:${height}px`,
    'z-index:10',
    'pointer-events:auto',
    'display:none',
    'cursor:pointer',
  ].join(';');
  document.body.appendChild(canvas);
  const ctx = canvas.getContext('2d');
  ctx.scale(dpr, dpr);

  const entries = []; // { tick, type, text }
  let visible = false;
  let scrollOffset = 0; // lines to skip from bottom
  let isDragging = false;
  let dragStartY = 0;
  let dragStartScroll = 0;

  function push(tick, type, text) {
    entries.push({ tick, type, text });
    if (entries.length > MAX_LOG_ENTRIES) entries.shift();
    // Auto-scroll to bottom unless user is scrolled up
    if (!isDragging) scrollOffset = 0;
  }

  function getVisibleEntries() {
    const maxLines = Math.floor((height - PAD.top - PAD.bottom) / LINE_H);
    const start = Math.max(0, entries.length - maxLines - scrollOffset);
    return entries.slice(start);
  }

  function render() {
    if (!visible) return;
    const w = width, h = height;
    ctx.clearRect(0, 0, w, h);

    // Background
    ctx.fillStyle = 'rgba(10,16,18,0.85)';
    ctx.strokeStyle = 'rgba(210,232,220,0.16)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(8, 0);
    ctx.lineTo(w - 8, 0);
    ctx.quadraticCurveTo(w, 0, w, 8);
    ctx.lineTo(w, h - 8);
    ctx.quadraticCurveTo(w, h, w - 8, h);
    ctx.lineTo(8, h);
    ctx.quadraticCurveTo(0, h, 0, h - 8);
    ctx.lineTo(0, 8);
    ctx.quadraticCurveTo(0, 0, 8, 0);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();

    // Title
    ctx.font = 'bold 10px "SF Mono", ui-monospace, monospace';
    ctx.fillStyle = '#e9f2ec';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'alphabetic';
    ctx.fillText(`EVENT LOG (${entries.length})`, PAD.left, 13);

    const vis = getVisibleEntries();
    ctx.font = '9px "SF Mono", ui-monospace';
    ctx.textBaseline = 'alphabetic';

    for (let i = 0; i < vis.length; i++) {
      const e = vis[i];
      const y = PAD.top + 2 + i * LINE_H;
      const color = EVENT_COLORS[e.type] ?? EVENT_COLORS.info;
      ctx.fillStyle = color;
      const prefix = e.type === 'speciation' ? '🧬' : e.type === 'extinction' ? '⚰' : '💥';
      ctx.fillText(`${prefix} t=${e.tick} ${e.text}`, PAD.left, y);
    }

    // Scroll hint
    if (scrollOffset > 0) {
      ctx.fillStyle = 'rgba(207,227,214,0.5)';
      ctx.textAlign = 'right';
      ctx.fillText(`↑ ${scrollOffset} older`, w - PAD.right, h - 4);
    }
  }

  // Mouse interaction for scrolling
  canvas.addEventListener('mousedown', (ev) => {
    isDragging = true;
    dragStartY = ev.clientY;
    dragStartScroll = scrollOffset;
    ev.preventDefault();
  });
  window.addEventListener('mousemove', (ev) => {
    if (!isDragging) return;
    const dy = dragStartY - ev.clientY;
    scrollOffset = Math.max(0, Math.min(entries.length - 5, dragStartScroll + dy / LINE_H));
    render();
  });
  window.addEventListener('mouseup', () => { isDragging = false; });

  canvas.addEventListener('wheel', (ev) => {
    ev.preventDefault();
    scrollOffset = Math.max(0, Math.min(entries.length - 5, scrollOffset + ev.deltaY > 0 ? 3 : -3));
    render();
  }, { passive: false });

  function setVisible(v) {
    visible = v;
    canvas.style.display = v ? '' : 'none';
    if (v) render();
  }

  return { canvas, push, setVisible, render, get visible() { return visible; } };
}
