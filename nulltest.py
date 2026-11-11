"""零假设：真实边界强度 vs 位置打乱。"""
import random
import cascade

def border_of(points, obs, bins):
    L, T = obs.x - obs.r, obs.y - obs.r
    bw = obs.r * 2 / bins
    acc = {}
    for x, y, t in points:
        i = int((x - L) / bw); j = int((y - T) / bw)
        if 0 <= i < bins and 0 <= j < bins:
            acc.setdefault((i, j), [0] * cascade.K)[t] += 1
    dist = {}
    for k, c in acc.items():
        n = sum(c)
        if n >= 3:
            dist[k] = [v / n for v in c]
    e = []
    for (i, j), a in dist.items():
        for di, dj in ((1, 0), (0, 1)):
            b = dist.get((i + di, j + dj))
            if b:
                e.append(sum(abs(a[t] - b[t]) for t in range(cascade.K)) / 2)
    return (sum(e) / len(e) if e else 0.0), len(dist)

w = cascade.World()
for _ in range(140):
    w.coarse_step()
ox, oy = cascade.best_spot(w, 180)
obs = cascade.Observer(w, ox, oy, 180)
obs.look()
for _ in range(140):
    w.coarse_step(); obs.look(); cascade.fine_step(w)

pts = [(p.x, p.y, p.t) for p in w.particles.values()]
xs0 = [p[0] for p in pts]; ys0 = [p[1] for p in pts]
print(f'粒子 {len(pts)}')

for bins in (5, 6, 8, 10):
    actual, cells = border_of(pts, obs, bins)
    rng = random.Random(1)
    nulls = []
    for _ in range(80):
        xs = xs0[:]; ys = ys0[:]
        rng.shuffle(xs); rng.shuffle(ys)
        nulls.append(border_of([(xs[i], ys[i], pts[i][2]) for i in range(len(pts))], obs, bins)[0])
    nulls.sort()
    mean = sum(nulls) / len(nulls)
    p95 = nulls[int(0.95 * len(nulls))]
    sig = '显著' if actual > p95 else '不显著'
    print(f'bins={bins:2d} 有效格={cells:3d} | 实际={actual:.3f} 零均值={mean:.3f} 95%={p95:.3f} -> {sig}')
