// renderer-phylogeny.mjs — Phase 15 species phylogeny overlay (cladogram).
//
// 数据流：wasm-viewer 每 60 sim-tick 通过 C-ABI（sim_clade_count / sim_get_clade /
// sim_clade_name / sim_event_count / sim_get_event）批量拉取一次 clade 与
// speciation event 快照 → buildPhylogenyGraph 生成纯数据图（Node 可测）→
// update() 绘制矩形枝型 cladogram。树从左（根 = t=0 的原始 kind）向右（现生种）
// 生长；灭绝分支虚线变灰（化石）；分化节点画红点。
//
// clade 记录布局（与 wasm.rs sim_get_clade 同步，Phase 16 起 40 bytes）：
//   [id u32][parent u32 (u32.MAX=根)][kind_hash u32][birth u32]
//   [death u32 (u32.MAX=存活)][member_count u32][centroid_eff f64][argb u32]
//   [niche f32]  ← Phase 16 尾部追加（member_count 即原 alive_count @20）
// event 记录布局（与 wasm.rs sim_get_event 同步，56 bytes）：
//   [tick u32][parent u32][child_count u32][event_kind u32（原 reserved）]
//   [genetic_distance f64][child0..7 u32]
//   kind：0=speciation / 1=extinction / 2=竞争排斥灭绝（Phase 16）
//
// readPhylogenyState / buildPhylogenyGraph 是纯函数（不触 DOM），Node 冒烟测试
// 直接消费；createPhylogenyOverlay 才依赖 document。

export const PHYLOGENY_VERSION = 1;
export const CLADE_RECORD_SIZE = 40; // Phase 16: +niche f32 tail (offset 36)
export const EVENT_RECORD_SIZE = 56;
export const CLADE_CAP = 64;
export const EVENT_CAP = 64;
export const CLADE_NAME_CAP = 32;
export const MAX_EVENT_CHILDREN = 8;

const ROOT_PARENT = 0xFFFFFFFF;
const ALIVE_TICK = 0xFFFFFFFF;

// ─── C-ABI 批量拉取（纯函数，Node 可测）───────────────────────────────────────

// E = wasm instance exports；cladePtr/eventPtr/namePtr 指向 scratch 内的
// SCRATCH_OFF_CLADES / SCRATCH_OFF_EVENTS / SCRATCH_OFF_CLADE_NAME 区域。
// 返回 { clades, events }：plain object 数组，字段与 Rust 记录一一对应。
export function readPhylogenyState(E, sim, cladePtr, eventPtr, namePtr, nameCap = CLADE_NAME_CAP) {
  const clades = [];
  if (typeof E.sim_clade_count === 'function') {
    const decoder = new TextDecoder();
    const cladeCount = E.sim_clade_count(sim) | 0;
    for (let i = 0; i < cladeCount && i < CLADE_CAP; i++) {
      if (E.sim_get_clade(sim, i, cladePtr) !== 1) break;
      // 每次调用都重建 DataView：memory.grow 会让旧 view 失效。
      const dv = new DataView(E.memory.buffer, cladePtr, CLADE_RECORD_SIZE);
      const id = dv.getUint32(0, true);
      const parentRaw = dv.getUint32(4, true);
      const birthTick = dv.getUint32(12, true);
      const deathRaw = dv.getUint32(16, true);
      const aliveCount = dv.getUint32(20, true);
      const centroidEff = dv.getFloat64(24, true);
      const argb = dv.getUint32(32, true);
      const nicheOffset = dv.getFloat32(36, true); // Phase 16 tail field
      const nameLen = E.sim_clade_name(sim, i, namePtr, nameCap) | 0;
      const name = decoder.decode(new Uint8Array(E.memory.buffer, namePtr, Math.max(0, nameLen)));
      clades.push({
        id,
        parentId: parentRaw === ROOT_PARENT ? null : parentRaw,
        kindHash: dv.getUint32(8, true),
        birthTick,
        deathTick: deathRaw === ALIVE_TICK ? null : deathRaw,
        aliveCount,
        centroidEff,
        nicheOffset,
        color: `#${(argb & 0xffffff).toString(16).padStart(6, '0')}`,
        name,
        extinct: deathRaw !== ALIVE_TICK,
      });
    }
  }
  const events = [];
  if (typeof E.sim_event_count === 'function') {
    const eventCount = E.sim_event_count(sim) | 0;
    for (let i = 0; i < eventCount && i < EVENT_CAP; i++) {
      // Phase 16: 返回值 >= 1 表示记录有效（灭绝事件 0 个子 clade 也返回 1），
      // 0 才是失败。真实 child_count 从记录 offset 8 读取；offset 12 是 kind
      // （0=speciation / 1=extinction / 2=竞争排斥灭绝）。
      const ret = E.sim_get_event(sim, i, eventPtr) | 0;
      if (ret === 0) break; // 0 = 失败
      const dv = new DataView(E.memory.buffer, eventPtr, EVENT_RECORD_SIZE);
      const tick = dv.getUint32(0, true);
      const parentCladeId = dv.getUint32(4, true);
      const kind = dv.getUint32(12, true);
      const n = Math.min(dv.getUint32(8, true), MAX_EVENT_CHILDREN);
      const childCladeIds = [];
      for (let k = 0; k < n; k++) childCladeIds.push(dv.getUint32(24 + k * 4, true));
      events.push({ tick, parentCladeId, childCladeIds, geneticDistance: dv.getFloat64(16, true), kind });
    }
  }
  return { clades, events };
}

// ─── 图构建（纯函数，Node 可测）───────────────────────────────────────────────
//
// 输入 clades/events 来自 readPhylogenyState（或测试夹具）。输出：
//   { version, rootTick, currentTick, nodes[], edges[], events[] }
//   node: { id,name,parentId,birthTick,deathTick,extinct,aliveCount,
//           centroidEff,color, x,y, tipX, depth }   x/y/tipX ∈ [0,1]
//   edge: { from,to, geneticDistance, extinct, birthTick }
//   event dot: { tick,parentCladeId,geneticDistance,x,y }
export function buildPhylogenyGraph(clades, events, currentTick, opts = {}) {
  const byId = new Map(clades.map((c) => [c.id, c]));
  const childrenOf = new Map();
  for (const c of clades) {
    if (c.parentId !== null && c.parentId !== undefined && byId.has(c.parentId)) {
      if (!childrenOf.has(c.parentId)) childrenOf.set(c.parentId, []);
      childrenOf.get(c.parentId).push(c.id);
    }
  }
  for (const list of childrenOf.values()) {
    list.sort((a, b) => (byId.get(a).birthTick - byId.get(b).birthTick) || (a - b));
  }

  const roots = clades.filter((c) => c.parentId === null || c.parentId === undefined || !byId.has(c.parentId));
  const rootTick = clades.length ? Math.min(...clades.map((c) => c.birthTick)) : 0;
  const span = Math.max(1, currentTick - rootTick);

  // 叶节点均匀分 Y（按出生时间自上而下），内部节点 = 子节点均值（后序）。
  const yOf = new Map();
  const depthOf = new Map();
  const leaves = clades.filter((c) => !childrenOf.has(c.id))
    .sort((a, b) => (a.birthTick - b.birthTick) || (a.id - b.id));
  const nLeaves = Math.max(1, leaves.length);
  leaves.forEach((c, i) => yOf.set(c.id, nLeaves === 1 ? 0.5 : i / (nLeaves - 1)));
  const computeY = (id, depth) => {
    depthOf.set(id, Math.max(depth, depthOf.get(id) ?? 0));
    const kids = childrenOf.get(id);
    if (!kids || !kids.length) return yOf.get(id) ?? 0.5;
    let sum = 0;
    for (const k of kids) sum += computeY(k, depth + 1);
    const y = sum / kids.length;
    yOf.set(id, y);
    return y;
  };
  for (const r of roots) computeY(r.id, 0);

  const nx = (tick) => (tick - rootTick) / span;

  const distOf = new Map();
  for (const ev of events) {
    for (const cid of ev.childCladeIds) distOf.set(cid, ev.geneticDistance);
  }

  const nodes = clades.map((c) => ({
    id: c.id,
    name: c.name,
    parentId: c.parentId ?? null,
    birthTick: c.birthTick,
    deathTick: c.deathTick ?? null,
    extinct: !!c.extinct,
    aliveCount: c.aliveCount ?? 0,
    centroidEff: c.centroidEff ?? 0,
    nicheOffset: c.nicheOffset ?? 0,
    color: c.color || '#cfe3d6',
    x: nx(c.birthTick),
    y: yOf.get(c.id) ?? 0.5,
    tipX: c.extinct ? nx(c.deathTick ?? c.birthTick) : 1,
    depth: depthOf.get(c.id) ?? 0,
    geneticDistance: distOf.get(c.id) ?? null,
  }));

  const edges = [];
  for (const c of clades) {
    if (c.parentId === null || c.parentId === undefined || !byId.has(c.parentId)) continue;
    edges.push({
      from: c.parentId,
      to: c.id,
      geneticDistance: distOf.get(c.id) ?? null,
      extinct: !!c.extinct,
      birthTick: c.birthTick,
    });
  }

  const eventDots = events.map((ev) => {
    const p = byId.get(ev.parentCladeId);
    return {
      tick: ev.tick,
      parentCladeId: ev.parentCladeId,
      geneticDistance: ev.geneticDistance,
      kind: ev.kind ?? 0,
      x: nx(ev.tick),
      y: p ? (yOf.get(p.id) ?? 0.5) : 0.5,
    };
  });

  return {
    version: PHYLOGENY_VERSION,
    rootTick,
    currentTick,
    fossilHighlight: !!opts.fossilHighlight,
    nodes,
    edges,
    events: eventDots,
  };
}

// 计算并注入 burst rate（形态变化率）：每单位时间内的遗传距离。
// burst_rate > 阈值（默认 0.02/tick，即 100 tick 累计 2.0）视为「活跃演化」分支，
// 反之视为停滞；用于渲染着色（间断平衡可视化，Phase 20）。
const BURST_RATE_THRESHOLD = 0.02; // d/tick — empirically calibrated against test data
export function annotatePunctuatedEquilibrium(g) {
  if (!g || !g.edges.length) return g;
  const currentTick = g.currentTick ?? 0;
  // Per-edge burst rating: an edge is "active" when its child carries a non-trivial
  // genetic distance and hasn't been extinct for too long (>500 ticks ago).
  // This captures the punctuated-equilibrium idea that bursts happen around
  // recent speciation events rather than over the whole lineage lifetime.
  const STALE_EXPIRY = 500; // ticks after death before edge is considered "dead history"
  for (const e of g.edges) {
    const kid = g.nodes.find(n => n.id === e.to);
    if (e.geneticDistance == null) {
      e.burstRate = 0;
      e.isBurst = false;
      continue;
    }
    const isStale = kid?.extinct && (kid.deathTick != null && (currentTick - kid.deathTick) > STALE_EXPIRY);
    e.isBurst = e.geneticDistance >= 0.3 && !isStale;
    e.burstRate = e.geneticDistance; // keep raw distance for tooltip clarity
  }
  // Compute summary stats for HUD.
  const totalDuration = g.edges.reduce((s, e) => s + Math.max(1, currentTick - e.birthTick), 0);
  const burstDuration = g.edges.filter(e => e.isBurst).reduce((s, e) => s + Math.max(1, currentTick - e.birthTick), 0);
  g.activitySummary = {
    pctBurst: totalDuration ? (burstDuration / totalDuration * 100).toFixed(0) : '—',
    burstEdges: g.edges.filter(e => e.isBurst).length,
    totalEdges: g.edges.length,
    recentBursts: g.events.filter(ev => ev.kind === 0 && (currentTick - ev.tick) < 500).length,
  };
  return g;
}

// ─── overlay 渲染（需要 DOM）──────────────────────────────────────────────────

// 手动圆角矩形（与 renderer-evolution-histogram 同款，避免依赖较新的 roundRect）。
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
const EXTINCT_DIM = 'rgba(160,175,168,0.45)';
const EXTINCT_BRIGHT = '#9fb3a8';
// Phase 20: punctuated equilibrium rendering.
// Stasis edges (burstRate below threshold) render dimmed; active-burst edges
// keep their original clade color at full opacity. A faint "PE" badge appears
// in the header when the simulation shows >60% burst-time across current
// lineages, signaling that morphological change clusters around speciation.
const BURST_ALPHA_MIN = 0.18;
const BURST_ALPHA_MAX = 1.0;

// Quick hex -> RGB helpers for overlay alpha compositing (Phase 20 PE rendering).
function _hexC(h, fn) { return fn(parseInt(h.slice(1), 16)); }
function hexR(c) { return _hexC(c, v => (v >> 16) & 0xff); }
function hexG(c) { return _hexC(c, v => (v >>  8) & 0xff); }
function hexB(c) { return _hexC(c, v => (v      ) & 0xff); }

export function createPhylogenyOverlay(opts = {}) {
  const width = opts.width ?? 360;
  const height = opts.height ?? 280;
  const right = opts.right ?? 20;
  const bottom = opts.bottom ?? 120;
  const PAD = { top: 26, bottom: 20, left: 84, right: 30 };

  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const canvas = document.createElement('canvas');
  canvas.id = 'species-phylogeny-overlay';
  canvas.width = Math.round(width * dpr);
  canvas.height = Math.round(height * dpr);
  // pointer-events:auto —— hover tooltip 需要；代价是该角落不参与相机拖拽。
  canvas.style.cssText = [
    'position:fixed', `right:${right}px`, `bottom:${bottom}px`,
    `width:${width}px`, `height:${height}px`,
    'z-index:10', 'pointer-events:auto', 'display:none',
  ].join(';');
  document.body.appendChild(canvas);
  const ctx = canvas.getContext('2d');
  ctx.scale(dpr, dpr);

  let visible = false;
  let graph = null;
  let hover = null; // { mx, my }
  let fossil = false;

  const setVisible = (v) => {
    visible = v;
    canvas.style.display = v ? '' : 'none';
  };

  const px = (x) => PAD.left + x * (width - PAD.left - PAD.right);
  const py = (y) => PAD.top + y * (height - PAD.top - PAD.bottom);

  function drawTooltip(g) {
    if (!hover || !g) return;
    // 命中测试：尖端点或出生点 12px 内最近的节点。
    let best = null;
    let bestD = 144; // 12px²
    for (const n of g.nodes) {
      for (const [hx, hy] of [[px(n.tipX), py(n.y)], [px(n.x), py(n.y)]]) {
        const d = (hx - hover.mx) ** 2 + (hy - hover.my) ** 2;
        if (d < bestD) { bestD = d; best = n; }
      }
    }
    if (!best) return;
    const lines = [
      `${best.extinct ? '†' : ''}${best.name}`,
      `born t=${best.birthTick}`,
      best.extinct ? `extinct t=${best.deathTick}` : `alive ×${best.aliveCount}`,
      best.geneticDistance != null ? `d=${best.geneticDistance.toFixed(2)} vs parent` : null,
      `eff=${best.centroidEff.toFixed(2)}`,
      `niche=${(best.nicheOffset ?? 0).toFixed(2)}`,
    ].filter(Boolean);
    ctx.font = MONO;
    const w = Math.max(...lines.map((s) => ctx.measureText(s).width)) + 14;
    const h = lines.length * 13 + 8;
    let bx = hover.mx + 12;
    let by = hover.my - h - 6;
    if (bx + w > width - 4) bx = hover.mx - w - 12;
    if (by < 4) by = hover.my + 12;
    ctx.fillStyle = 'rgba(8,14,16,0.92)';
    ctx.strokeStyle = 'rgba(210,232,220,0.3)';
    ctx.lineWidth = 1;
    panelPath(ctx, 0, 0, 0); // no-op path reset
    ctx.beginPath();
    ctx.rect(bx, by, w, h);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = '#e9f2ec';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'alphabetic';
    lines.forEach((s, i) => {
      ctx.fillStyle = i === 0 ? (best.extinct ? EXTINCT_BRIGHT : best.color) : 'rgba(207,227,214,0.8)';
      ctx.fillText(s, bx + 7, by + 15 + i * 13);
    });
  }

  function render() {
    if (!visible) return;
    const g = graph;
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
    let title = 'SPECIES PHYLOGENY';
    // Phase 20: punctuated equilibrium signal — badge when >50% of lineage-time
    // sits in active-burst mode across current clades.
    if (g.activitySummary && +g.activitySummary.pctBurst >= 50) {
      title += ' · PE';
      ctx.fillStyle = '#ffb347';
    }
    ctx.fillText(title, PAD.left - 60, 16);
    ctx.font = MONO;
    ctx.fillStyle = 'rgba(207,227,214,0.55)';
    ctx.textAlign = 'right';
    const summaryText = g.activitySummary
      ? `burst ${g.activitySummary.pctBurst}% · b=${g.activitySummary.burstEdges}/${g.activitySummary.totalEdges}`
      : '';
    ctx.fillText(`${g ? `t=${g.currentTick}` : 't=—'}${summaryText ? '  ' + summaryText : ''}`, width - 10, 16);

    if (!g || g.nodes.length === 0) {
      ctx.fillStyle = 'rgba(207,227,214,0.5)';
      ctx.textAlign = 'center';
      ctx.fillText('no clades yet', width / 2, height / 2);
      return;
    }

    const nodeById = new Map(g.nodes.map((n) => [n.id, n]));

    // 分支：矩形枝型（竖段在父 x，横段在子 y）。灭绝分支虚线灰。
    // Phase 20: 按 burstRate 调节不透明度 — 活跃演化分支明亮，停滞分支沉暗
    // （间断平衡可视化）。
    for (const e of g.edges) {
      const p = nodeById.get(e.from);
      const c = nodeById.get(e.to);
      if (!p || !c) continue;
      const x0 = px(p.x);
      const yP = py(p.y);
      const yC = py(c.y);
      const x1 = px(c.tipX);
      let branchColor = e.extinct
        ? (g.fossilHighlight ? EXTINCT_BRIGHT : EXTINCT_DIM)
        : c.color;
      let alpha = 1.0;
      if (!e.extinct && typeof e.isBurst === 'boolean' && !e.isBurst) {
        // Stasis branch: dim.
        alpha = BURST_ALPHA_MIN;
        branchColor = c.color;
      }
      ctx.strokeStyle = e.extinct ? branchColor
        : `rgba(${hexR(branchColor)},${hexG(branchColor)},${hexB(branchColor)},${alpha.toFixed(2)})`;
      ctx.lineWidth = 1;
      ctx.setLineDash(e.extinct ? [3, 3] : []);
      ctx.beginPath();
      ctx.moveTo(x0 + 0.5, yP);
      ctx.lineTo(x0 + 0.5, yC + 0.5);
      ctx.lineTo(x1, yC + 0.5);
      ctx.stroke();
      ctx.setLineDash([]);
      // 化石高亮：死亡端画小空心方块（⚰ 语义的 canvas 等价物）。
      if (e.extinct && g.fossilHighlight) {
        ctx.strokeStyle = EXTINCT_BRIGHT;
        ctx.strokeRect(x1 - 2.5, yC - 2.5, 5, 5);
      }
    }

    // 事件点：speciation 红 / extinction 灰 / 竞争排斥琥珀（Phase 16）。
    for (const ev of g.events) {
      ctx.fillStyle = ev.kind === 2 ? '#ffb347' : ev.kind === 1 ? '#8a9a90' : '#ff5a5a';
      ctx.beginPath();
      ctx.arc(px(ev.x), py(ev.y), 2.5, 0, Math.PI * 2);
      ctx.fill();
    }

    // 叶节点：色点 + 左侧标签（灭绝加 † 前缀、变灰）。
    ctx.font = MONO;
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';
    for (const n of g.nodes) {
      const isLeaf = !g.edges.some((e) => e.from === n.id);
      const tipX = px(n.tipX);
      const y = py(n.y);
      ctx.fillStyle = n.extinct
        ? (g.fossilHighlight ? EXTINCT_BRIGHT : EXTINCT_DIM)
        : n.color;
      ctx.beginPath();
      ctx.arc(tipX, y, 2.5, 0, Math.PI * 2);
      ctx.fill();
      if (isLeaf) {
        ctx.fillStyle = n.extinct
          ? (g.fossilHighlight ? EXTINCT_BRIGHT : 'rgba(160,175,168,0.7)')
          : '#e9f2ec';
        ctx.fillText(`${n.extinct ? '†' : ''}${n.name}`, PAD.left - 8, y);
      }
    }

    // 底部时间轴：rootTick … currentTick。
    ctx.fillStyle = 'rgba(207,227,214,0.45)';
    ctx.textBaseline = 'alphabetic';
    ctx.textAlign = 'left';
    ctx.fillText(`t=${g.rootTick}`, PAD.left, height - 6);
    ctx.textAlign = 'right';
    ctx.fillText(`t=${g.currentTick}`, width - PAD.right, height - 6);

    drawTooltip(g);
  }

  canvas.addEventListener('mousemove', (e) => {
    const r = canvas.getBoundingClientRect();
    hover = { mx: e.clientX - r.left, my: e.clientY - r.top };
    render();
  });
  canvas.addEventListener('mouseleave', () => {
    hover = null;
    render();
  });

  return {
    canvas,
    update(g) { graph = g; fossil = !!g?.fossilHighlight; render(); },
    setVisible,
    render,
    get visible() { return visible; },
    get graph() { return graph; },
  };
}
