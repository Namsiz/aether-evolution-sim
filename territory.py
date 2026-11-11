"""把空间领地画出来：每个网格填主导类型的颜色。"""
import math
import cascade

def render_territory(world, obs, path, bins=10, S=760):
    sp = cascade.spatial_report(world, obs, bins)
    L, T, bw = sp['origin']
    k = (S * 0.86) / (obs.r * 2)
    ox = S * 0.07 - L * k
    oy = S * 0.07 - T * k

    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{S}" height="{S}" viewBox="0 0 {S} {S}">',
        f'<rect width="{S}" height="{S}" fill="#04060c"/>',
    ]

    for (i, j), d in sp['dist'].items():
        top = max(range(cascade.K), key=lambda t: d[t])
        strength = max(d)
        x = L + i * bw
        y = T + j * bw
        out.append(
            f'<rect x="{x*k+ox:.1f}" y="{y*k+oy:.1f}" width="{bw*k+0.6:.1f}" '
            f'height="{bw*k+0.6:.1f}" fill="{cascade.TYPE_COLOR[top]}" '
            f'opacity="{0.12+0.72*strength:.2f}"/>'
        )

    # 粒子作为细点，垫在底下
    for p in world.particles.values():
        out.append(
            f'<circle cx="{p.x*k+ox:.1f}" cy="{p.y*k+oy:.1f}" r="1.1" '
            f'fill="{cascade.TYPE_COLOR[p.t % cascade.K]}" opacity="0.5"/>'
        )

    out.append(
        f'<text x="14" y="28" font-family="monospace" font-size="13" fill="#8fb5e0">'
        f'{bins}x{bins} 领地 · 边界强度 {sp["border"]:.2f} · 主导类型 {len(sp["dominant"])} 种</text>'
    )
    out.append('</svg>')
    open(path, 'w').write('\n'.join(out))
    return sp

w = cascade.World()
for _ in range(200):
    w.coarse_step()
ox, oy = cascade.best_spot(w, 150)
obs = cascade.Observer(w, ox, oy, 150)
obs.look()
for _ in range(180):
    w.coarse_step(); obs.look(); cascade.fine_step(w)
sp = render_territory(w, obs, 'territory.svg')
print('粒子', len(w.particles), '边界', round(sp['border'], 3))
print('主导类型', dict(sorted(sp['dominant'].items())))
print('自催化环', [sorted(c) for c in cascade.raf_report(w)])
