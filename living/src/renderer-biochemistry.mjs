// renderer-biochemistry.mjs — Phase 18 biochemical strip + HUD readout.
//
// A thin 8px-wide colored bar positioned directly below the evolution
// spectrogram (the spectrogram's bottom edge is at top=910; this strips
// runs from there). The color encodes the fast nutrient pool level:
//   • baseline (fast = 0.5) → neutral mint (matches DECO hue ~0.75)
//   • depleted → desaturated grey (nutrient exhaustion state)
//   • enriched → warm amber/gold (tectonically active or high-efficiency DECO)
//
// The fertility readout line (`createFertilityHUDLine`) is a HUD span fed
// by wasm-viewer.mjs per frame; it just renders `fertile: X.XX`.

import { DECO_HUE_MIN, DECO_HUE_SPAN } from './renderer-evolution-histogram.mjs';

/// CSS-safe color lerp helper (avoid floating-point sRGB roundtrip drift).
function rgba(r, g, b, a = 1) {
  return `rgba(${Math.round(r)},${Math.round(g)},${Math.round(b)},${a})`;
}

/// Fast-pool concentration ∈ [0, 2] mapped to an 8px strip fill color.
/// Returns an RGBA string usable as canvas `fillStyle`.
export function fastPoolColor(fastPool) {
  const norm = Math.min(1, Math.max(0, (fastPool - 0.0) / 2.0));
  if (norm < 0.33) {
    // Depleted → muddy cyan-grey (nutrients locked up / never deposited).
    const t = norm / 0.33;
    return rgba(
      80 + t * 40,
      110 + t * 60,
      115 + t * 60,
      0.8,
    );
  } else if (norm < 0.67) {
    // Baseline region → the DECO visual hue band (standard violet).
    const t = (norm - 0.33) / 0.34;
    return rgba(
      140 + t * 30,
      160 + t * 20,
      175 + t * 10,
      0.85,
    );
  } else {
    // Enriched → golden-amber (high tectonic output, fast cycling).
    const t = (norm - 0.67) / 0.33;
    return rgba(
      170 + t * 85,
      180 - t * 20,
      185 - t * 85,
      0.85 + t * 0.1,
    );
  }
}

/// Build pure data for the strip: `{ fast, slow, depositRate, fertile }` so
/// tests can verify the mapping without hitting DOM.
export function buildNutrientView(fast, slow, depositRate) {
  const fert = Math.max(0, (fast - 0.5) * 1.0); // matches Rust's max(0.0) gate
  return { fast, slow, depositRate, fertile: fert.toFixed(4) };
}

/// Create an 8px-high strip canvas sitting just under the spectrogram.
/// `bottom` offset from the panel's bottom edge is passed in via opts.
export function createNutrientStrip(opts = {}) {
  const width = opts.width ?? 320;
  const height = 8;
  const left = opts.left ?? 20;
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const canvas = document.createElement('canvas');
  canvas.id = 'biochem-strip';
  canvas.width = Math.round(width * dpr);
  canvas.height = Math.round(height * dpr);
  canvas.style.cssText = [
    'position:fixed', `left:${left}px`, `bottom:${opts.bottom ?? 0}px`,
    `width:${width}px`, `height:${height}px`,
    'z-index:11', 'pointer-events:none', 'display:block',
  ].join(';');
  document.body.appendChild(canvas);
  const ctx = canvas.getContext('2d');
  ctx.scale(dpr, dpr);

  let view = null;
  function render() {
    if (!view) {
      ctx.clearRect(0, 0, width, height);
      return;
    }
    ctx.fillStyle = fastPoolColor(view.fast);
    ctx.fillRect(0, 0, width, height);
    // Thin baseline tick at x = (0.5/2.0)*width (where fast_pool = 0.5).
    ctx.strokeStyle = 'rgba(255,255,255,0.35)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(Math.round(width * 0.25), 0.5);
    ctx.lineTo(Math.round(width * 0.25), height - 0.5);
    ctx.stroke();
    ctx.font = '8px "SF Mono", ui-monospace, monospace';
    ctx.fillStyle = 'rgba(233,242,236,0.75)';
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';
    ctx.fillText(`fertile ${view.fertile}`, width - 2, height / 2);
  }

  return { canvas, update(v) { view = v; render(); }, render, get visible() { return !!view; } };
}

/// Produce a HUD text line (`fertile: 0.xx · fast=N.nn · slow=N.nn · dep=R.nn`).
/// Null when geoSupported is false or no data has been pulled yet.
export function createFertilityHUDLine(fast, slow, depositRate) {
  const f = Math.max(0, (fast - 0.5)).toFixed(2);
  return `<span style="color:#ffd166">fertile: ${f} · fast=${fast.toFixed(2)} · slow=${slow.toFixed(2)} · dep=${depositRate.toFixed(2)}</span><br>`;
}
