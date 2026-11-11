#!/usr/bin/env python3
"""
export_scene.py — 跑一遍世界，导出可交互的三维场景。

输出 world3d.html：单文件，内置数据 + Canvas 三维渲染器。
鼠标拖拽旋转，滚轮缩放。
"""

import json
import math
import cascade3d as c3


def largest_component(world):
    adj = {}
    for a, b in world.edges:
        adj.setdefault(a, set()).add(b)
        adj.setdefault(b, set()).add(a)
    seen = set()
    best = 0
    for s in list(world.particles):
        if s in seen:
            continue
        stack = [s]
        seen.add(s)
        n = 0
        while stack:
            x = stack.pop()
            n += 1
            for y in adj.get(x, ()):
                if y not in seen:
                    seen.add(y)
                    stack.append(y)
        best = max(best, n)
    return best


def build_world(steps=500):
    w = c3.World()
    for _ in range(120):
        w.coarse_step()
    cx, cy, cz = c3.best_spot(w, 90)
    obs = c3.Observer(w, cx, cy, cz, 90)
    obs.look()
    for _ in range(steps):
        w.coarse_step()
        obs.look()
        c3.fine_step(w)
    return w


def export(world, path):
    ps = list(world.particles.values())
    n = len(ps)
    mean_x = sum(p.x for p in ps) / n
    mean_y = sum(p.y for p in ps) / n
    mean_z = sum(p.z for p in ps) / n
    ds = sorted(math.sqrt((p.x - mean_x) ** 2 + (p.y - mean_y) ** 2 + (p.z - mean_z) ** 2) for p in ps)
    R = ds[int(0.85 * (len(ds) - 1))] if ds else 1.0
    R = max(R, 1.0)

    particles = [
        {"x": round(p.x - mean_x, 2),
         "y": round(p.y - mean_y, 2),
         "z": round(p.z - mean_z, 2),
         "e": round(p.e, 3),
         "t": p.t}
        for p in ps
    ]
    ids = {p.id: i for i, p in enumerate(ps)}
    bonds = [[ids[a], ids[b]] for a, b in world.edges if a in ids and b in ids]

    comp = largest_component(world)
    kinds, _ = c3.genome_report(world)
    common, directed, mutual = c3.catalytic_ecology(world)

    data = {
        "particles": particles,
        "bonds": bonds,
        "meta": {
            "n": n,
            "bonds": len(bonds),
            "R": round(R, 2),
            "genomes": kinds,
            "largest": comp,
            "mutual": mutual,
        },
    }
    payload = json.dumps(data, separators=(",", ":"))

    html = TEMPLATE.replace("__DATA__", payload)
    open(path, "w").write(html)

    print(f"粒子 {n} | 键 {len(bonds)} | 最大团 {comp} | 基因组 {kinds} | 互催化 {mutual}")
    print(f"-> {path}")


TEMPLATE = r"""<!DOCTYPE html>
<html lang="zh">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>三维世界</title>
<style>
  html,body{margin:0;height:100%;background:#03050a;overflow:hidden;
    font-family:ui-monospace,Menlo,monospace;color:#7fa8d8}
  canvas{display:block;width:100vw;height:100vh;cursor:grab}
  canvas:active{cursor:grabbing}
  #hud{position:fixed;left:14px;top:12px;font-size:12px;line-height:1.6;
    text-shadow:0 0 6px #000;pointer-events:none}
  #hud b{color:#9fe0ff}
</style>
</head>
<body>
<canvas id="c"></canvas>
<div id="hud"></div>
<script>
const DATA = __DATA__;
const cvs = document.getElementById('c');
const ctx = cvs.getContext('2d');

let W = 0, H = 0, DPR = Math.min(2, window.devicePixelRatio || 1);
function resize(){
  W = cvs.clientWidth; H = cvs.clientHeight;
  cvs.width = W * DPR; cvs.height = H * DPR;
  ctx.setTransform(DPR, 0, 0, DPR, 0, 0);
}
window.addEventListener('resize', resize);

const COLORS = ['#6aa8ff','#5fe0c8','#ffb45e','#c98bff',
                '#ff6b8a','#8dff6b','#ffd76b','#6bfff0'];

const P = DATA.particles, B = DATA.bonds, M = DATA.meta;
const R = M.R || 100;
const DIST = 3.0 * R;

let yaw = 0.6, pitch = 0.35, zoom = 1.0;
let drag = false, lx = 0, ly = 0;

cvs.addEventListener('pointerdown', e => { drag = true; lx = e.clientX; ly = e.clientY; });
window.addEventListener('pointerup', () => { drag = false; });
window.addEventListener('pointermove', e => {
  if (!drag) return;
  yaw += (e.clientX - lx) * 0.008;
  pitch += (e.clientY - ly) * 0.008;
  pitch = Math.max(-1.5, Math.min(1.5, pitch));
  lx = e.clientX; ly = e.clientY;
});
cvs.addEventListener('wheel', e => {
  e.preventDefault();
  zoom *= Math.exp(-e.deltaY * 0.001);
  zoom = Math.max(0.25, Math.min(8, zoom));
}, {passive: false});

function project(x, y, z){
  const cy = Math.cos(yaw), sy = Math.sin(yaw);
  const x1 = x * cy - z * sy;
  const z1 = x * sy + z * cy;
  const cp = Math.cos(pitch), sp = Math.sin(pitch);
  const y1 = y * cp - z1 * sp;
  const z2 = y * sp + z1 * cp;
  const d = z2 + DIST;
  if (d < 1) return null;
  const f = 1.25 * Math.min(W, H) * zoom;
  return {sx: W/2 + x1 * f / d, sy: H/2 + y1 * f / d, d};
}

function frame(){
  ctx.fillStyle = '#03050a';
  ctx.fillRect(0, 0, W, H);

  const pts = new Array(P.length);
  for (let i = 0; i < P.length; i++){
    const p = P[i];
    pts[i] = project(p.x, p.y, p.z);
  }

  // 键：按深度排序后画，远的先画
  const edges = [];
  for (let i = 0; i < B.length; i++){
    const a = pts[B[i][0]], b = pts[B[i][1]];
    if (!a || !b) continue;
    edges.push({a, b, d: (a.d + b.d) / 2});
  }
  edges.sort((u, v) => v.d - u.d);
  ctx.lineWidth = 1;
  for (const e of edges){
    const fade = Math.max(0.03, Math.min(0.32, DIST / e.d * 0.22));
    ctx.strokeStyle = 'rgba(74,122,176,' + fade.toFixed(3) + ')';
    ctx.beginPath();
    ctx.moveTo(e.a.sx, e.a.sy);
    ctx.lineTo(e.b.sx, e.b.sy);
    ctx.stroke();
  }

  // 粒子：按深度排序
  const order = [];
  for (let i = 0; i < P.length; i++) if (pts[i]) order.push(i);
  order.sort((i, j) => pts[j].d - pts[i].d);
  for (const i of order){
    const p = P[i], q = pts[i];
    const r = Math.max(0.7, 2.6 * (0.5 + p.e * 0.22) * DIST / q.d);
    const fade = Math.max(0.22, Math.min(1, DIST * 1.25 / q.d));
    ctx.globalAlpha = 0.30 + 0.65 * fade;
    ctx.fillStyle = COLORS[p.t % COLORS.length];
    ctx.beginPath();
    ctx.arc(q.sx, q.sy, r, 0, 6.28318);
    ctx.fill();
  }
  ctx.globalAlpha = 1;

  document.getElementById('hud').innerHTML =
    '<b>三维坍缩世界</b><br>' +
    '粒子 ' + M.n + ' · 键 ' + M.bonds + '<br>' +
    '最大连通团 ' + M.largest + '<br>' +
    '基因组 ' + M.genomes + ' · 互催化 ' + M.mutual + '<br>' +
    '<span style="opacity:.6">拖拽旋转 · 滚轮缩放</span>';

  requestAnimationFrame(frame);
}

resize();
requestAnimationFrame(frame);
</script>
</body>
</html>
"""


if __name__ == '__main__':
    w = build_world(500)
    export(w, 'world3d.html')
