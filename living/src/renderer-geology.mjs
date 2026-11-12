// renderer-geology.mjs — Phase 18 geological mini-map overlay.
//
// 1 px = 1 grid cell (GRID_N × GRID_N, same layout as the mineral texture
// but re-used here so no extra resampling pass is needed). Ocean cells are
// tinted with a transparent blue; plates are opaque circles colored by type
// with velocity arrows. The overlay itself sits at bottom-right with epoch
// label + sea-level delta above it. G-key toggle is wired through
// wasm-viewer.mjs.
//
// All drawing functions take a CanvasRenderingContext2D directly; the
// build* helpers return pure data views (Node smoke-testable). Pure-data
// path avoids DOM coupling during wasm import smoke tests.

import { terrainHeight, WATER_LEVEL } from './world.js';

export const GEOMETRY_GRID_N = 76;
/// Plate fill color: oceanic convergent (dark basalt).
export const PLATE_COLOR_OCEANIC = '#c9b8a1';
/// Plate fill color: continental core (warm granite).
export const PLATE_COLOR_CONTINENTAL = '#d4a76a';
/// Plate fill color: rift zone (fresh volcanic glass — deep cyan).
export const PLATE_COLOR_RIFT = '#6adbc2';
/// Submerged ocean-cell tint (RGBA float).
export const OCEAN_TINT = [0.15, 0.3, 0.45, 0.6];

/// Parse a plate record (48B raw, as written by sim_get_plate into a buffer
/// passed by wasm-viewer); mirrors wasm.rs PLATE_RECORD_SIZE layout.
export function parsePlate(buf, offset = 0) {
  const dv = new DataView(buf, offset, 48);
  return {
    id: dv.getUint32(0, true),
    x: dv.getFloat64(4, true),
    z: dv.getFloat64(12, true),
    vx: dv.getFloat64(20, true),
    vz: dv.getFloat64(28, true),
    angle: dv.getFloat32(36, true),
    type_: dv.getUint32(40, true),
    seismicIndex: dv.getFloat32(44, true),
  };
}

/// Build the pure data view consumed by drawGeologyMap / updateGeologyMap.
/// Returns { epoch, seaLevel, cells: Uint8Array(GRID_N²), plates: [...] }.
/// cells[i] = 0 land / 1 ocean (based on base terrain + sea_level).
export function buildGeologyView(plates, seaLevel) {
  const cells = new Uint8Array(GEOMETRY_GRID_N * GEOMETRY_GRID_N);
  const half = 540;
  for (let j = 0; j < GEOMETRY_GRID_N; ++j) {
    for (let i = 0; i < GEOMETRY_GRID_N; ++i) {
      const x = -half + (i + 0.5) * (1080 / GEOMETRY_GRID_N);
      const z = -half + (j + 0.5) * (1080 / GEOMETRY_GRID_N);
      cells[j * GEOMETRY_GRID_N + i] = terrainHeight(x, z) < (WATER_LEVEL + seaLevel) ? 1 : 0;
    }
  }
  return {
    epoch: '',       // populated externally from sim_epoch_name
    seaLevel,
    cells,
    plates: plates.map((p) => ({ ...p })),
  };
}

/// Draw the view onto an already-scaled 2D context (`ctx.scale(dpr, dpr)`
/// has been applied by the caller). `w/h` are the CSS-sized canvas dims;
/// each grid cell = w/GRID_N physical px. Returns nothing.
export function drawGeologyMap(ctx, w, h, view) {
  if (!view) return;
  const n = GEOMETRY_GRID_N;
  const cw = w / n;
  const ch = h / n;
  // Cell-level ocean/land.
  for (let j = 0; j < n; ++j) {
    for (let i = 0; i < n; ++i) {
      const idx = j * n + i;
      const x = i * cw;
      const y = j * ch;
      if (view.cells[idx]) {
        ctx.fillStyle = `rgba(${Math.round(OCEAN_TINT[0]*255)},${Math.round(OCEAN_TINT[1]*255)},${Math.round(OCEAN_TINT[2]*255)},${OCEAN_TINT[3]})`;
      } else {
        ctx.fillStyle = 'rgba(180,195,185,0.18)';
      }
      ctx.fillRect(Math.floor(x), Math.floor(y), Math.ceil(cw), Math.ceil(ch));
    }
  }
  // Plate overlays: filled circles + velocity arrow.
  for (const p of view.plates) {
    const px = (p.x + 540) / 1080 * w;
    const py = (p.z + 540) / 1080 * h;
    const r = (p.type_ === 0 ? 240 : p.type_ === 1 ? 170 : 150) / 1080 * w;
    const fill = p.type_ === 0
      ? PLATE_COLOR_CONTINENTAL
      : p.type_ === 1
      ? PLATE_COLOR_OCEANIC
      : PLATE_COLOR_RIFT;
    ctx.fillStyle = fill;
    ctx.globalAlpha = 0.55;
    ctx.beginPath();
    ctx.arc(px, py, r, 0, Math.PI * 2);
    ctx.fill();
    ctx.globalAlpha = 1;
    // Arrow: direction vector proportional to |velocity|, max length r.
    const speed = Math.hypot(p.vx, p.vz);
    if (speed > 0.001) {
      const ax = (p.vx / speed) * r * 0.9;
      const ay = (p.vz / speed) * r * 0.9;
      ctx.strokeStyle = 'rgba(255,255,255,0.85)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(px - ax * 0.5, py - ay * 0.5);
      ctx.lineTo(px + ax, py + ay);
      ctx.stroke();
      const tipLen = 2.5;
      const perpX = -ay / speed;
      const perpY = ax / speed;
      ctx.beginPath();
      ctx.moveTo(px + ax, py + ay);
      ctx.lineTo(px + ax - tipLen * ax/speed + tipLen * perpX, py + ay - tipLen * ay/speed + tipLen * perpY);
      ctx.moveTo(px + ax, py + ay);
      ctx.lineTo(px + ax - tipLen * ax/speed - tipLen * perpX, py + ay - tipLen * ay/speed - tipLen * perpY);
      ctx.stroke();
    }
  }
  ctx.globalAlpha = 1;
}

/// Create a fixed-position 64×64 overlay canvas. The panel shows the
/// geology minimap in its interior, with epoch label + sea-level readout
/// above. Toggle visibility via setVisible().
export function createGeologyMinimap(opts = {}) {
  const width = opts.width ?? 64;
  const height = opts.height ?? 64;
  const right = opts.right ?? 400;
  const bottom = opts.bottom ?? 40;
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const canvas = document.createElement('canvas');
  canvas.id = 'geology-overlay';
  canvas.width = Math.round(width * dpr);
  canvas.height = Math.round(height * dpr);
  canvas.style.cssText = [
    'position:fixed', `right:${right}px`, `bottom:${bottom}px`,
    `width:${width}px`, `height:${height}px`,
    'z-index:10', 'pointer-events:auto', 'display:none',
  ].join(';');
  document.body.appendChild(canvas);
  const ctx = canvas.getContext('2d');
  ctx.scale(dpr, dpr);

  let visible = false;
  let view = null;

  function render() {
    if (!visible) return;
    ctx.clearRect(0, 0, width, height);
    // Panel background (same style as phylogeny/niche overlays).
    ctx.fillStyle = 'rgba(10,16,18,0.65)';
    ctx.strokeStyle = 'rgba(210,232,220,0.16)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0.5, 0.5); ctx.lineTo(width - 0.5, 0.5);
    ctx.quadraticCurveTo(width - 0.5, 0.5, width - 0.5, 0.5);
    ctx.lineTo(width - 0.5, height - 0.5);
    ctx.quadraticCurveTo(width - 0.5, height - 0.5, width - 0.5, height - 0.5);
    ctx.lineTo(0.5, height - 0.5);
    ctx.quadraticCurveTo(0.5, height - 0.5, 0.5, height - 0.5);
    ctx.lineTo(0.5, 0.5);
    ctx.quadraticCurveTo(0.5, 0.5, 0.5, 0.5);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();

    // Mini-map content fills most of the panel.
    const PAD = 10;
    const mw = width - PAD * 2;
    const mh = height - PAD * 2 - 10; // title row
    const padCtx = ctx.createImageData(mw, mh);
    // We don't have a backing store for the raw cells — redraw directly
    // with rects instead (fast enough at 64×64).
    ctx.save();
    ctx.translate(PAD, PAD + 10);
    drawGeologyMap(ctx, mw, mh, view);
    ctx.restore();

    // Epoch + sea-level HUD (top row inside the border).
    ctx.font = 'bold 9px "SF Mono", ui-monospace, monospace';
    ctx.fillStyle = '#e9f2ec';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'alphabetic';
    const info = `${view ? (view.epoch || '?') : '—'} · t=${view ? (view.tick ?? '—') : '—'}`;
    const seaStr = view && view.seaLevel !== undefined
      ? (view.seaLevel > 0 ? '+' : '') + view.seaLevel.toFixed(0) + 'm'
      : '—';
    ctx.fillText(info, 2, 8);
    ctx.fillStyle = 'rgba(207,227,214,0.65)';
    ctx.textAlign = 'right';
    ctx.fillText(seaStr, width - 2, 8);
  }

  return {
    canvas,
    update(v) { view = v; render(); },
    setVisible(flag) { visible = !!flag; canvas.style.display = flag ? '' : 'none'; if (flag) render(); },
    render,
    get visible() { return visible; },
    get epoch() { return view?.epoch ?? ''; },
    get seaLevel() { return view?.seaLevel ?? 0; },
  };
}
