import cascade

w = cascade.World()
for _ in range(100):
    w.coarse_step()
ox, oy = cascade.best_spot(w, 140)
obs = cascade.Observer(w, ox, oy, 140)
obs.look()

hist = []
for step in range(900):
    w.coarse_step(); obs.look(); cascade.fine_step(w)
    if step % 50 == 49:
        ps = list(w.particles.values())
        sc = sum(1 for p in ps if cascade.is_self_catalytic(p.genome))
        kinds, _ = cascade.genome_report(w)
        hist.append((step + 1, len(ps), kinds, sc, sc / max(1, len(ps))))

for h in hist:
    print(f'{h[0]:4d} {h[1]:5d} {h[2]:5d} {h[3]:4d} {h[4]*100:5.1f}%')

# 画折线图
W, H = 760, 420
pad = 60
n = len(hist)
maxp = max(h[4] for h in hist) or 1
maxk = max(h[2] for h in hist) or 1
pts_p, pts_k = [], []
for i, h in enumerate(hist):
    x = pad + (W - 2 * pad) * i / max(1, n - 1)
    pts_p.append(f'{x:.1f},{H-pad-(H-2*pad)*h[4]/maxp:.1f}')
    pts_k.append(f'{x:.1f},{H-pad-(H-2*pad)*h[2]/maxk:.1f}')

svg = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}">',
       f'<rect width="{W}" height="{H}" fill="#05070d"/>',
       f'<line x1="{pad}" y1="{H-pad}" x2="{W-pad}" y2="{H-pad}" stroke="#2a3550"/>',
       f'<line x1="{pad}" y1="{pad}" x2="{pad}" y2="{H-pad}" stroke="#2a3550"/>',
       f'<polyline points="{" ".join(pts_p)}" fill="none" stroke="#6fe8b0" stroke-width="2.4"/>',
       f'<polyline points="{" ".join(pts_k)}" fill="none" stroke="#6aa8ff" stroke-width="2"/>',
       f'<text x="{pad}" y="28" font-family="monospace" font-size="13" fill="#6fe8b0">自我催化基因组占比 (0-{maxp*100:.0f}%)</text>',
       f'<text x="{pad+300}" y="28" font-family="monospace" font-size="13" fill="#6aa8ff">基因组多样性</text>',
       f'<text x="{pad}" y="{H-pad+24}" font-family="monospace" font-size="12" fill="#7fa8d8">0</text>',
       f'<text x="{W-pad-30}" y="{H-pad+24}" font-family="monospace" font-size="12" fill="#7fa8d8">900 步</text>',
       '</svg>']
open('evolve.svg', 'w').write('\n'.join(svg))
print('svg -> evolve.svg')
