// renderer-niche.mjs — Phase 16 生态位空间 overlay（1-D niche 轴云图）。
//
// 数据流：wasm-viewer 复用 phylogeny 的每 60-tick C-ABI 拉取（readPhylogenyState
// 已带 nicheOffset 字段，sim_get_clade offset 36 的 f32）→ buildNicheCloud 生成
// 纯数据视图（Node 可测）→ update() 绘制。横轴 = niche ∈ [-1, 1]；每个 clade
// 一个散点，半径 ∝ sqrt(成员数)，颜色按 clade hue；灭绝 clade 灰色 + 虚线圈；
// 存活 clade 间 |Δniche| < 阈值（0.15，与 Rust 侧同步）画琥珀色重叠连线。
// U 键 toggle（wasm-viewer.mjs 注册），与 cladogram（T 键）并排互补：
// cladogram 回答「物种如何诞生」，niche 云图回答「新物种占据什么生态位」。
//
// readPhylogenyState / buildNicheCloud 是纯函数（不触 DOM），Node 冒烟测试
// 直接消费；createNicheOverlay 才依赖 document。

export const NICHE_VERSION = 1;
export const NICHE_MIN = -1;
export const NICHE_MAX = 1;
/// 与 ecology.rs NICHE_OVERLAP_THRESHOLD 同步（1-D Gause 重叠窗口）。
export const NICHE_OVERLAP_THRESHOLD = 0.15;

// ─── 云图构建（纯函数，Node 可测）─────────────────────────────────────────────
//
// 输入 clades 来自 readPhylogenyState（或测试夹具）。输出：
//   { version, currentTick, threshold, points[], overlaps[] }
//   point: { id, name, niche, members, extinct, color, x, y }  x/y ∈ [0,1]
//   overlap: { a, b, dist }  a/b 为 clade id
export function buildNicheCloud(clades, currentTick, opts = {}) {
  const overlapThreshold = opts.overlapThreshold ?? NICHE_OVERLAP_THRESHOLD;
  const nicheOf = (c) => {
    const raw = c.nicheOffset ?? 0;
    if (!Number.isFinite(raw)) return 0;
    return Math.max(NICHE_MIN, Math.min(NICHE_MAX, raw));
  };

  // 活 clade 按出生顺序分行（y 轴），避免散点重叠；灭绝 clade 统一放中线。
  const living = clades.filter((c) => !c.extinct)
    .sort((a, b) => (a.birthTick - b.birthTick) || (a.id - b.id));
  const rowOf = new Map(living.map((c, i) =>
    [c.id, living.length === 1 ? 0.5 : i / (living.length - 1)]));

  const points = clades.map((c) => {
    const niche = nicheOf(c);
    return {
      id: c.id,
      name: c.name,
      niche,
      members: c.aliveCount ?? 0,
      extinct: !!c.extinct,
      color: c.color || '#cfe3d6',
      x: (niche - NICHE_MIN) / (NICHE_MAX - NICHE_MIN),
      y: c.extinct ? 0.5 : (rowOf.get(c.id) ?? 0.5),
    };
  });

  const overlaps = [];
  for (let i = 0; i < living.length; i++) {
    for (let j = i + 1; j < living.length; j++) {
      const d = Math.abs(nicheOf(living[i]) - nicheOf(living[j]));
      if (d < overlapThreshold) {
        overlaps.push({ a: living[i].id, b: living[j].id, dist: d });
      }
    }
  }

  return { version: NICHE_VERSION, currentTick, threshold: overlapThreshold, points, overlaps };
}

// ─── overlay 渲染（需要 DOM）──────────────────────────────────────────────────

// 手动圆角矩形（与 renderer-phylogeny 同款）。
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

const MONO = '10px "SF Mono", ui-monospace, monospace';

export function createNicheOverlay(opts = {}) {
  const width = opts.width ?? 340;
  const height = opts.height ?? 210;
  const right = opts.right ?? 400; // phylogeny 面板 (right:20 width:360) 左侧
  const bottom = opts.bottom ?? 120;
  const PAD = { top: 26, bottom: 40, left: 22, right: 22 };

  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const canvas = document.createElement('canvas');
  canvas.id = 'species-niche-overlay';
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
  let cloud = null;

  const px = (x) => PAD.left + x * (width - PAD.left - PAD.right);
  const py = (y) => PAD.top + y * (height - PAD.top - PAD.bottom);

  function render() {
    if (!visible) return;
    ctx.clearRect(0, 0, width, height);
    ctx.fillStyle = 'rgba(10,16,18,0.65)';
    ctx.strokeStyle = 'rgba(210,232,220,0.16)';
    ctx.lineWidth = 1;
    panelPath(ctx, width, height, 10);
    ctx.fill();
    ctx.stroke();

    ctx.font = 'bold 11px "SF Mono", ui-monospace, monospace';
    ctx.fillStyle = '#e9f2ec';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'alphabetic';
    ctx.fillText('NICHE SPACE', PAD.left, 16);
    ctx.font = MONO;
    ctx.fillStyle = 'rgba(207,227,214,0.55)';
    ctx.textAlign = 'right';
    ctx.fillText(cloud ? `t=${cloud.currentTick}` : 't=—', width - PAD.right, 16);

    if (!cloud || cloud.points.length === 0) {
      ctx.fillStyle = 'rgba(207,227,214,0.5)';
      ctx.textAlign = 'center';
      ctx.fillText('no clades yet', width / 2, height / 2);
      return;
    }

    // 重叠连线（琥珀色，竞争排斥的预警带）。
    const byId = new Map(cloud.points.map((p) => [p.id, p]));
    ctx.strokeStyle = 'rgba(255,179,71,0.7)';
    ctx.lineWidth = 1.5;
    for (const ov of cloud.overlaps) {
      const a = byId.get(ov.a);
      const b = byId.get(ov.b);
      if (!a || !b || a.extinct || b.extinct) continue;
      ctx.setLineDash([4, 3]);
      ctx.beginPath();
      ctx.moveTo(px(a.x), py(a.y));
      ctx.lineTo(px(b.x), py(b.y));
      ctx.stroke();
      ctx.setLineDash([]);
    }

    // 轴线 + 刻度。
    const axisY = py(0.5);
    ctx.strokeStyle = 'rgba(207,227,214,0.35)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(px(0), axisY);
    ctx.lineTo(px(1), axisY);
    ctx.stroke();
    ctx.fillStyle = 'rgba(207,227,214,0.5)';
    ctx.font = MONO;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    for (const [v, label] of [[0, '-1'], [0.5, '0'], [1, '+1']]) {
      ctx.beginPath();
      ctx.moveTo(px(v), axisY - 3);
      ctx.lineTo(px(v), axisY + 3);
      ctx.stroke();
      ctx.fillText(label, px(v), height - PAD.bottom + 8);
    }

    // 散点：半径 ∝ sqrt(成员数)；灭绝 = 灰 + 虚线圈。
    for (const p of cloud.points) {
      const r = 2.5 + Math.sqrt(Math.max(0, p.members)) * 0.9;
      if (p.extinct) {
        ctx.strokeStyle = 'rgba(160,175,168,0.55)';
        ctx.setLineDash([2, 2]);
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.arc(px(p.x), py(p.y), r + 1.5, 0, Math.PI * 2);
        ctx.stroke();
        ctx.setLineDash([]);
        ctx.fillStyle = 'rgba(160,175,168,0.4)';
      } else {
        ctx.fillStyle = p.color;
      }
      ctx.beginPath();
      ctx.arc(px(p.x), py(p.y), r, 0, Math.PI * 2);
      ctx.fill();
      // 少量 clade 时直接标名（灭绝加 †）。
      if (cloud.points.length <= 8) {
        ctx.fillStyle = p.extinct ? 'rgba(160,175,168,0.8)' : 'rgba(233,242,236,0.9)';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'bottom';
        ctx.fillText(`${p.extinct ? '†' : ''}${p.name}`, px(p.x), py(p.y) - r - 3);
      }
    }
  }

  return {
    canvas,
    update(c) { cloud = c; render(); },
    setVisible(v) { visible = v; canvas.style.display = v ? '' : 'none'; if (v) render(); },
    render,
    get visible() { return visible; },
    get cloud() { return cloud; },
  };
}
