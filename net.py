"""把反应网络画成图：节点=类型，边=催化关系，SCC高亮。"""
import math, sys
import cascade

def render_network(world, comps, path, S=680):
    R = S * 0.36
    cx = cy = S / 2
    pos = {}
    for i in range(cascade.K):
        a = -math.tau * i / cascade.K + math.pi / 2
        pos[i] = (cx + R * math.cos(a), cy + R * math.sin(a))

    in_cycle = set()
    for c in comps:
        in_cycle |= set(c)

    adj = {}
    for a, b, k, c in world.reactions:
        adj.setdefault(k, set()).add(c)

    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{S}" height="{S}" viewBox="0 0 {S} {S}">',
        f'<rect width="{S}" height="{S}" fill="#04060c"/>',
    ]

    for k, cs in adj.items():
        for c in cs:
            x1, y1 = pos[k]
            x2, y2 = pos[c]
            if k == c:
                continue
            dx, dy = x2 - x1, y2 - y1
            d = math.hypot(dx, dy)
            ux, uy = dx / d, dy / d
            x1 += ux * 20; y1 += uy * 20
            x2 -= ux * 20; y2 -= uy * 20
            live = k in in_cycle and c in in_cycle
            col = '#6fe8b0' if live else '#2b3a5c'
            op = 0.75 if live else 0.35
            wd = 1.6 if live else 0.7
            out.append(f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" stroke="{col}" stroke-width="{wd}" opacity="{op}"/>')

    for i in range(cascade.K):
        x, y = pos[i]
        live = i in in_cycle
        r = 17 if live else 13
        fill = cascade.TYPE_COLOR[i]
        op = 0.95 if live else 0.35
        out.append(f'<circle cx="{x:.1f}" cy="{y:.1f}" r="{r}" fill="{fill}" opacity="{op}"/>')
        out.append(f'<text x="{x:.1f}" y="{y+5:.1f}" font-family="monospace" font-size="13" fill="#04060c" text-anchor="middle" opacity="{op}">{i}</text>')

    out.append(f'<text x="20" y="32" font-family="monospace" font-size="14" fill="#7fa8d8">{len(world.reactions)} 种反应 / 自催化集 {[len(c) for c in comps]}</text>')
    out.append('</svg>')
    open(path, 'w').write('\n'.join(out))

w = cascade.World()
for _ in range(200):
    w.coarse_step()
ox, oy = cascade.best_spot(w, 170)
obs = cascade.Observer(w, ox, oy, 170)
obs.look()
for _ in range(500):
    w.coarse_step(); obs.look(); cascade.fine_step(w)
comps = cascade.raf_report(w)
render_network(w, comps, 'network.svg')
print('反应', len(w.reactions), '自催化集', [sorted(c) for c in comps])
