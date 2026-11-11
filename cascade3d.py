#!/usr/bin/env python3
"""
cascade3d.py — 三维坍缩世界

和二维版的区别只有空间本身：
  粗层是 20x20x20 的统计格，观察者是球体视野，粒子有 z 轴，
  渲染是透视投影。

所有规则沿用二维版：催化反应、基因组复制、自然选择。
"""

import math
import random
from dataclasses import dataclass, field

SIZE = 600
CELL = 40
NX = SIZE // CELL          # 20 -> 8000 格
K = 8
SEED = 7

GRAV = 0.035
DIFF = 0.06
INJECT_N = 8
INJECT_E = 4.0
E0 = 1.2

CAT_R = 55.0
REACTION_LOSS = 0.45
E_REQ_BASE = 0.8
MUTATE = 0.03
SPONTANEOUS = 0.02

BOND_R = 65.0
BOND_L = 12.0
SPRING = 0.03
DIFFUSE = 0.04
MAX_DEG = 6
PARTICLE_GRAV = 0.9
REPEL = 1.0
DAMP = 0.97
VMAX = 0.6

GENOME_LEN = 6
ALPHABET = 4
MUTATION_RATE = 0.02
REPLICATE_E = 2.4
REPLICATE_COST = 1.1
REPLICATE_REWARD = 0.4   # 催化别人复制时，催化者分到的能量

TYPE_COLOR = [
    '#6aa8ff', '#5fe0c8', '#ffb45e', '#c98bff',
    '#ff6b8a', '#8dff6b', '#ffd76b', '#6bfff0',
]


@dataclass
class Cell:
    n: float = 0.0
    energy: float = 0.0
    types: list = field(default_factory=lambda: [1.0 / K] * K)
    vx: float = 0.0
    vy: float = 0.0
    vz: float = 0.0
    temp: float = 0.0
    instantiated: bool = False
    version: int = 0


@dataclass
class Particle:
    id: int
    x: float
    y: float
    z: float
    vx: float
    vy: float
    vz: float
    e: float
    t: int
    cell: tuple
    born: int
    last: int = 0
    genome: tuple = ()


def dist(a, b):
    return math.sqrt((a.x - b.x) ** 2 + (a.y - b.y) ** 2 + (a.z - b.z) ** 2)


def poisson(rng, lam):
    if lam <= 0:
        return 0
    if lam > 40:
        return max(0, int(round(rng.gauss(lam, math.sqrt(lam)))))
    L = math.exp(-lam)
    k, p = 0, 1.0
    while True:
        k += 1
        p *= rng.random()
        if p <= L:
            return k - 1


def weighted(rng, dist_):
    r = rng.random()
    acc = 0.0
    for i, w in enumerate(dist_):
        acc += w
        if r <= acc:
            return i
    return len(dist_) - 1


def random_genome(rng):
    return tuple(rng.randrange(ALPHABET) for _ in range(GENOME_LEN))


def mutate(g, rng):
    out = list(g) if g else list(random_genome(rng))
    for i in range(len(out)):
        if rng.random() < MUTATION_RATE:
            out[i] = rng.randrange(ALPHABET)
    return tuple(out)


def genome_type(g):
    return sum(g) % K if g else 0


def is_self_catalytic(g):
    return len(g) == 6 and g[:3] == g[3:]


def match_score(a, b):
    """a 的催化位点与 b 的反应位点匹配几位（0-3）。"""
    if len(a) != 6 or len(b) != 6:
        return 0
    return sum(1 for i in range(3) if a[i] == b[3 + i])


def genome_affinity(a, b):
    """两个基因组有多少位相同（0-6）。相似的就容易粘在一起。"""
    if len(a) != 6 or len(b) != 6:
        return 0
    return sum(1 for i in range(6) if a[i] == b[i])


def catalyses(a, b):
    """3 位里匹配 2 位即可催化。生物识别很少完全精确。"""
    return match_score(a, b) >= 2


def reaction(a, b):
    return (a * b + 1) % K, (a + b) % K


class World:
    def __init__(self, seed=SEED):
        self.rng = random.Random(seed)
        self.seed = seed
        self.nx = NX
        self.cell = CELL
        self.size = SIZE
        self.cells = [[[Cell() for _ in range(NX)] for _ in range(NX)] for _ in range(NX)]
        self.particles = {}
        self.edges = set()
        self.next_id = 0
        self.tick = 0
        self.collapses = 0
        self.evaporations = 0
        self._init_field()

    def _init_field(self):
        nx = self.nx
        rng = self.rng
        f = [[[0.0] * nx for _ in range(nx)] for _ in range(nx)]
        for scale in (nx * 0.55, nx * 0.28, nx * 0.14):
            for _ in range(8):
                cx, cy, cz = rng.uniform(0, nx), rng.uniform(0, nx), rng.uniform(0, nx)
                amp = rng.uniform(0.5, 1.0) * scale
                s2 = 2.0 * scale * scale
                for i in range(nx):
                    for j in range(nx):
                        for k in range(nx):
                            d2 = (i - cx) ** 2 + (j - cy) ** 2 + (k - cz) ** 2
                            f[i][j][k] += amp * math.exp(-d2 / s2)

        total = sum(sum(sum(row) for row in plane) for plane in f)
        target = nx ** 3 * 0.35
        k = target / total if total > 0 else 0
        for i in range(nx):
            for j in range(nx):
                for k2 in range(nx):
                    c = self.cells[i][j][k2]
                    c.n = f[i][j][k2] * k
                    c.energy = c.n * E0

    def coarse_step(self, dt=1.0):
        nx = self.nx
        cells = self.cells
        G = GRAV * dt
        D = DIFF * dt
        dn = [[[0.0] * nx for _ in range(nx)] for _ in range(nx)]

        for i in range(nx):
            for j in range(nx):
                for k in range(nx):
                    c = cells[i][j][k]
                    if c.instantiated:
                        continue
                    for di, dj, dk in ((1, 0, 0), (0, 1, 0), (0, 0, 1)):
                        ni, nj, nk = (i + di) % nx, (j + dj) % nx, (k + dk) % nx
                        o = cells[ni][nj][nk]
                        if o.instantiated:
                            continue
                        diff = c.n - o.n
                        flow = -diff * G if diff < 0 else diff * D
                        if flow <= 0:
                            continue
                        if flow > c.n * 0.25:
                            flow = c.n * 0.25
                        dn[i][j][k] -= flow
                        dn[ni][nj][nk] += flow

        for i in range(nx):
            for j in range(nx):
                for k in range(nx):
                    c = cells[i][j][k]
                    if c.instantiated:
                        continue
                    c.n = max(0.0, c.n + dn[i][j][k])

        for _ in range(INJECT_N):
            i, j, k = self.rng.randrange(nx), self.rng.randrange(nx), self.rng.randrange(nx)
            if not cells[i][j][k].instantiated:
                cells[i][j][k].energy += INJECT_E

        self.tick += 1

    def instantiate(self, i, j, k):
        cell = self.cells[i][j][k]
        if cell.instantiated:
            return 0

        seed = (self.seed * 1000003
                ^ (i * 7919 + j * 104729 + k * 1299709) * 31
                ^ (cell.version + 1) * 31337)
        rng = random.Random(seed)

        count = poisson(rng, cell.n)
        if count == 0:
            return 0
        e_each = cell.energy / count
        if e_each < 0.05:
            return 0

        cell.instantiated = True
        self.collapses += 1

        for _ in range(count):
            x = (i + rng.random()) * self.cell
            y = (j + rng.random()) * self.cell
            z = (k + rng.random()) * self.cell
            g = random_genome(rng)
            pid = self.next_id
            self.next_id += 1
            self.particles[pid] = Particle(
                pid, x, y, z,
                cell.vx + rng.gauss(0, cell.temp + 0.05),
                cell.vy + rng.gauss(0, cell.temp + 0.05),
                cell.vz + rng.gauss(0, cell.temp + 0.05),
                e_each, genome_type(g), (i, j, k), self.tick, 0, g,
            )
        return count

    def coarsen(self, i, j, k):
        cell = self.cells[i][j][k]
        if not cell.instantiated:
            return
        ps = [p for p in self.particles.values() if p.cell == (i, j, k)]
        if ps:
            n = len(ps)
            cell.n = float(n)
            cell.energy = sum(p.e for p in ps)
            hist = [0] * K
            for p in ps:
                hist[p.t] += 1
            cell.types = [h / n for h in hist]
            cell.vx = sum(p.vx for p in ps) / n
            cell.vy = sum(p.vy for p in ps) / n
            cell.vz = sum(p.vz for p in ps) / n
            cell.temp = math.sqrt(sum(
                (p.vx - cell.vx) ** 2 + (p.vy - cell.vy) ** 2 + (p.vz - cell.vz) ** 2
                for p in ps) / n)
        else:
            cell.n = 0.0
            cell.energy = 0.0
        for p in ps:
            self.particles.pop(p.id, None)
        cell.instantiated = False
        cell.version += 1
        self.evaporations += 1

    def relocate(self, p):
        i = min(max(int(p.x // self.cell), 0), self.nx - 1)
        j = min(max(int(p.y // self.cell), 0), self.nx - 1)
        k = min(max(int(p.z // self.cell), 0), self.nx - 1)
        if (i, j, k) == p.cell:
            return True
        if self.cells[i][j][k].instantiated:
            p.cell = (i, j, k)
            return True
        c = self.cells[i][j][k]
        old = c.n
        c.n += 1.0
        c.energy += p.e
        if old > 0:
            c.vx = (c.vx * old + p.vx) / (old + 1)
            c.vy = (c.vy * old + p.vy) / (old + 1)
            c.vz = (c.vz * old + p.vz) / (old + 1)
        self.particles.pop(p.id, None)
        return False


def total_energy(world):
    e = 0.0
    for i in range(world.nx):
        for j in range(world.nx):
            for k in range(world.nx):
                c = world.cells[i][j][k]
                if not c.instantiated:
                    e += c.energy
    e += sum(p.e for p in world.particles.values())
    return e


def add_edge(world, a, b):
    if a != b:
        world.edges.add((a, b) if a < b else (b, a))


def kill(world, pid):
    world.particles.pop(pid, None)
    for e in [e for e in world.edges if pid in e]:
        world.edges.discard(e)


def fine_step(world):
    ps = list(world.particles.values())
    if not ps:
        return
    rng = world.rng
    budget = {'bond': 6, 'react': 2, 'replicate': 2, 'decay': 2}

    by_id = {p.id: p for p in ps}
    adj = {p.id: set() for p in ps}
    live = set()
    for a, b in world.edges:
        if a in by_id and b in by_id:
            adj[a].add(b)
            adj[b].add(a)
            live.add((a, b))
    world.edges = live

    # 弹簧 + 传能
    for a, b in live:
        pa, pb = by_id[a], by_id[b]
        d = dist(pa, pb) + 1e-6
        de = (pa.e - pb.e) * DIFFUSE
        pa.e = max(0.0, pa.e - de)
        pb.e += de
        f = (d - BOND_L) * SPRING
        ux, uy, uz = (pb.x - pa.x) / d, (pb.y - pa.y) / d, (pb.z - pa.z) / d
        pa.vx += f * ux; pa.vy += f * uy; pa.vz += f * uz
        pb.vx -= f * ux; pb.vy -= f * uy; pb.vz -= f * uz

    # 三维局部引力
    gc = 90.0
    grid = {}
    for p in ps:
        grid.setdefault((int(p.x // gc), int(p.y // gc), int(p.z // gc)), []).append(p)
    for p in ps:
        cx, cy, cz = int(p.x // gc), int(p.y // gc), int(p.z // gc)
        for dx in (-1, 0, 1):
            for dy in (-1, 0, 1):
                for dz in (-1, 0, 1):
                    for q in grid.get((cx + dx, cy + dy, cz + dz), ()):
                        if q.id == p.id:
                            continue
                        ddx, ddy, ddz = q.x - p.x, q.y - p.y, q.z - p.z
                        raw2 = ddx * ddx + ddy * ddy + ddz * ddz
                        raw = math.sqrt(raw2) + 1e-6
                        soft2 = raw2 + 200.0
                        f = PARTICLE_GRAV * q.e / soft2
                        if raw < 18.0:
                            f -= REPEL * q.e / soft2
                        p.vx += f * ddx / raw
                        p.vy += f * ddy / raw
                        p.vz += f * ddz / raw

    for p in ps:
        p.vx *= DAMP; p.vy *= DAMP; p.vz *= DAMP
        sp = math.sqrt(p.vx ** 2 + p.vy ** 2 + p.vz ** 2)
        if sp > VMAX:
            p.vx *= VMAX / sp; p.vy *= VMAX / sp; p.vz *= VMAX / sp
        p.x = min(max(p.x + p.vx, 0.0), world.size)
        p.y = min(max(p.y + p.vy, 0.0), world.size)
        p.z = min(max(p.z + p.vz, 0.0), world.size)

    # 背景能量
    for _ in range(2):
        rng.choice(ps).e += 0.3

    used = set()

    # 成键（三维网格）
    bg = {}
    for p in ps:
        bg.setdefault((int(p.x // BOND_R), int(p.y // BOND_R), int(p.z // BOND_R)), []).append(p)
    for p in ps:
        if budget['bond'] <= 0:
            break
        if p.id in used or len(adj[p.id]) >= MAX_DEG:
            continue
        cx, cy, cz = int(p.x // BOND_R), int(p.y // BOND_R), int(p.z // BOND_R)
        best, bd = None, 1e9
        for dx in (-1, 0, 1):
            for dy in (-1, 0, 1):
                for dz in (-1, 0, 1):
                    for q in bg.get((cx + dx, cy + dy, cz + dz), ()):
                        if q.id == p.id or q.id in adj[p.id] or q.id in used:
                            continue
                        if len(adj[q.id]) >= MAX_DEG:
                            continue
                        if genome_affinity(p.genome, q.genome) < 4:
                            continue
                        d = dist(p, q)
                        if d <= BOND_R and d < bd:
                            bd, best = d, q
        if best is not None:
            add_edge(world, p.id, best.id)
            adj[p.id].add(best.id)
            adj[best.id].add(p.id)
            used.add(p.id); used.add(best.id)
            budget['bond'] -= 1

    # 催化反应
    if budget['react'] > 0:
        tg = {}
        for p in ps:
            tg.setdefault((int(p.x // CAT_R), int(p.y // CAT_R), int(p.z // CAT_R)), []).append(p)
        for a, b in list(world.edges):
            if budget['react'] <= 0:
                break
            pa, pb = by_id.get(a), by_id.get(b)
            if not pa or not pb:
                continue
            c_type, k_type = reaction(pa.t, pb.t)
            e_req = E_REQ_BASE + ((pa.t + pb.t) % 3) * 0.3
            if pa.e + pb.e < e_req:
                continue
            cat = None
            cx, cy, cz = int(pa.x // CAT_R), int(pa.y // CAT_R), int(pa.z // CAT_R)
            for dx in (-1, 0, 1):
                for dy in (-1, 0, 1):
                    for dz in (-1, 0, 1):
                        for q in tg.get((cx + dx, cy + dy, cz + dz), ()):
                            if q.t != k_type or q.id in (a, b):
                                continue
                            if dist(pa, q) <= CAT_R:
                                cat = q
                                break
                        if cat:
                            break
                    if cat:
                        break
                if cat:
                    break
            if cat is None:
                if rng.random() > SPONTANEOUS:
                    continue
            child_g = pa.genome if rng.random() < 0.5 else pb.genome
            ne = pa.e + pb.e - REACTION_LOSS
            nx_, ny_, nz_ = (pa.x + pb.x) / 2, (pa.y + pb.y) / 2, (pa.z + pb.z) / 2
            nvx, nvy, nvz = (pa.vx + pb.vx) / 2, (pa.vy + pb.vy) / 2, (pa.vz + pb.vz) / 2
            keep = (adj[a] | adj[b]) - {a, b}
            kill(world, a); kill(world, b)
            for kk in (a, b):
                by_id.pop(kk, None); adj.pop(kk, None)
            pid = world.next_id
            world.next_id += 1
            np = Particle(pid, nx_, ny_, nz_, nvx, nvy, nvz, ne, c_type,
                          (0, 0, 0), world.tick, 0, child_g)
            world.particles[pid] = np
            by_id[pid] = np
            adj[pid] = set()
            for kk in keep:
                if kk in by_id:
                    add_edge(world, pid, kk)
                    adj[pid].add(kk)
                    adj[kk].add(pid)
            used.add(pid)
            budget['react'] -= 1

    # 复制（基因组起作用的地方）
    gg = {}
    if budget['replicate'] > 0:
        for q in ps:
            gg.setdefault((int(q.x // CAT_R), int(q.y // CAT_R), int(q.z // CAT_R)), []).append(q)

    for p in list(world.particles.values()):
        if budget['replicate'] <= 0:
            break
        boost = 1.0
        cat = None
        if is_self_catalytic(p.genome):
            boost = 0.45
        elif gg and p.genome:
            cx, cy, cz = int(p.x // CAT_R), int(p.y // CAT_R), int(p.z // CAT_R)
            for dx in (-1, 0, 1):
                for dy in (-1, 0, 1):
                    for dz in (-1, 0, 1):
                        for q in gg.get((cx + dx, cy + dy, cz + dz), ()):
                            if q.id != p.id and catalyses(q.genome, p.genome):
                                cat = q
                                boost = 0.65
                                break
                        if cat is not None:
                            break
                    if cat is not None:
                        break
                if cat is not None:
                    break
        if p.e < REPLICATE_E * boost:
            continue
        g = mutate(p.genome, rng)
        p.e -= REPLICATE_COST
        ang1 = rng.random() * math.tau
        ang2 = math.acos(2 * rng.random() - 1)
        pid = world.next_id
        world.next_id += 1
        np = Particle(
            pid,
            p.x + math.sin(ang2) * math.cos(ang1) * 16,
            p.y + math.sin(ang2) * math.sin(ang1) * 16,
            p.z + math.cos(ang2) * 16,
            p.vx, p.vy, p.vz,
            REPLICATE_COST * 0.82, genome_type(g), p.cell, world.tick, 0, g,
        )
        world.particles[pid] = np
        if cat is not None and cat.id in world.particles:
            cat.e += REPLICATE_REWARD
        budget['replicate'] -= 1

    # 衰变
    for p in list(world.particles.values()):
        if budget['decay'] <= 0:
            break
        if p.e < 0.12 and not adj.get(p.id):
            kill(world, p.id)
            budget['decay'] -= 1

    for p in list(world.particles.values()):
        world.relocate(p)


def genome_report(world, top=5):
    from collections import Counter
    c = Counter(p.genome for p in world.particles.values())
    return len(c), c.most_common(top)


def catalytic_ecology(world, min_count=2):
    from collections import Counter
    cnt = Counter(p.genome for p in world.particles.values())
    common = [g for g, c in cnt.items() if c >= min_count and len(g) == 6]
    directed = 0
    mutual = 0
    for a in common:
        for b in common:
            if a == b:
                continue
            if catalyses(a, b):
                directed += 1
                if catalyses(b, a):
                    mutual += 1
    return len(common), directed // 2, mutual // 2


class Observer:
    def __init__(self, world, cx, cy, cz, r):
        self.world = world
        self.cx, self.cy, self.cz = cx, cy, cz
        self.r = r
        self.active = set()

    def cells_in_view(self):
        cs = self.world.cell
        out = []
        span = int(self.r // cs) + 2
        ci = int(self.cx // cs)
        cj = int(self.cy // cs)
        ck = int(self.cz // cs)
        for di in range(-span, span + 1):
            for dj in range(-span, span + 1):
                for dk in range(-span, span + 1):
                    i = (ci + di) % self.world.nx
                    j = (cj + dj) % self.world.nx
                    k = (ck + dk) % self.world.nx
                    ccx = (i + 0.5) * cs
                    ccy = (j + 0.5) * cs
                    ccz = (k + 0.5) * cs
                    d = math.sqrt((ccx - self.cx) ** 2 + (ccy - self.cy) ** 2 + (ccz - self.cz) ** 2)
                    if d <= self.r + cs * 0.8:
                        out.append((i, j, k))
        return out

    def look(self):
        view = set(self.cells_in_view())
        for c in view:
            self.world.instantiate(*c)
        for c in list(self.active - view):
            self.world.coarsen(*c)
        self.active = view

    def move_to(self, cx, cy, cz):
        self.cx, self.cy, self.cz = cx, cy, cz


def best_spot(world, rad):
    best, bs = (world.size / 2,) * 3, -1.0
    span = max(1, int(rad // world.cell))
    for i in range(world.nx):
        for j in range(world.nx):
            for k in range(world.nx):
                s = 0.0
                for di in range(-span, span + 1):
                    for dj in range(-span, span + 1):
                        for dk in range(-span, span + 1):
                            s += world.cells[(i + di) % world.nx][(j + dj) % world.nx][(k + dk) % world.nx].n
                if s > bs:
                    bs = s
                    best = ((i + 0.5) * world.cell, (j + 0.5) * world.cell, (k + 0.5) * world.cell)
    return best


def render(world, obs, path, S=820):
    """透视投影：按粒子云包围球自动取景，保证整团都在画面里。"""
    ps = list(world.particles.values())
    if ps:
        cx = sum(p.x for p in ps) / len(ps)
        cy = sum(p.y for p in ps) / len(ps)
        cz = sum(p.z for p in ps) / len(ps)
        R = max(math.sqrt((p.x - cx) ** 2 + (p.y - cy) ** 2 + (p.z - cz) ** 2) for p in ps)
        R = max(R, 20.0)
    else:
        cx = cy = cz = world.size / 2
        R = world.size / 2

    D = 3.0 * R
    cam = (cx, cy, cz - D)
    focal = S * 1.25

    def project(p):
        dz = p.z - cam[2]
        if dz <= 1.0:
            return None
        sx = S / 2 + (p.x - cam[0]) * focal / dz
        sy = S / 2 + (p.y - cam[1]) * focal / dz
        return sx, sy, dz

    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{S}" height="{S}" viewBox="0 0 {S} {S}">',
        f'<rect width="{S}" height="{S}" fill="#03050a"/>',
    ]

    # 粗层：远处的统计点
    for i in range(world.nx):
        for j in range(world.nx):
            for k in range(world.nx):
                c = world.cells[i][j][k]
                v = min(1.0, c.n / 16.0)
                if v < 0.03:
                    continue
                pr = project(type('P', (), {
                    'x': (i + 0.5) * world.cell,
                    'y': (j + 0.5) * world.cell,
                    'z': (k + 0.5) * world.cell,
                })())
                if pr is None:
                    continue
                sx, sy, dz = pr
                if not (0 <= sx <= S and 0 <= sy <= S):
                    continue
                rr = max(0.6, 1600 * v / dz)
                op = 0.05 + 0.5 * v
                if c.instantiated:
                    op *= 0.25
                out.append(
                    f'<circle cx="{sx:.1f}" cy="{sy:.1f}" r="{rr:.1f}" '
                    f'fill="rgb({int(30+70*v)},{int(50+110*v)},{int(90+150*v)})" opacity="{op:.2f}"/>'
                )

    # 细层：按深度排序的粒子
    pts = []
    for p in world.particles.values():
        pr = project(p)
        if pr is None:
            continue
        sx, sy, dz = pr
        if not (0 <= sx <= S and 0 <= sy <= S):
            continue
        pts.append((dz, sx, sy, p))

    pts.sort(key=lambda t: -t[0])

    # 键：先画，带深度衰减，让三维骨架显出来
    pos = {p.id: (sx, sy, dz) for dz, sx, sy, p in pts}
    for a, b in world.edges:
        if a not in pos or b not in pos:
            continue
        x1, y1, z1 = pos[a]
        x2, y2, z2 = pos[b]
        depth = (z1 + z2) / 2
        op = max(0.05, min(0.45, 260.0 / depth))
        out.append(
            f'<line x1="{x1:.1f}" y1="{y1:.1f}" x2="{x2:.1f}" y2="{y2:.1f}" '
            f'stroke="#4a7ab0" stroke-width="{max(0.4, 90.0/depth):.1f}" opacity="{op:.2f}"/>'
        )

    for dz, sx, sy, p in pts:
        rr = max(0.8, 2600 * (0.5 + p.e * 0.18) / dz)
        fade = max(0.25, min(1.0, 340.0 / dz))
        out.append(
            f'<circle cx="{sx:.1f}" cy="{sy:.1f}" r="{rr:.1f}" '
            f'fill="{TYPE_COLOR[p.t % K]}" opacity="{0.35+0.6*fade:.2f}"/>'
        )

    out.append(
        f'<text x="14" y="26" font-family="monospace" font-size="13" fill="#7fa8d8">'
        f'三维坍缩世界 · 粒子 {len(pts)} · 格 {world.nx}^3</text>'
    )
    out.append('</svg>')
    open(path, 'w').write('\n'.join(out))


def main():
    w = World()
    print(f'创世: {w.nx}^3 格, 每格平均 {sum(c.n for p in w.cells for r in p for c in r)/(w.nx**3):.2f}')

    for _ in range(80):
        w.coarse_step()

    cx, cy, cz = best_spot(w, 170)
    obs = Observer(w, cx, cy, cz, 170)
    obs.look()
    print(f'坍缩: {len(w.particles)} 个粒子, 能量 {total_energy(w):.1f}')

    for step in range(300):
        w.coarse_step(); obs.look(); fine_step(w)
        if step % 100 == 99:
            ps = list(w.particles.values())
            sc = sum(1 for p in ps if is_self_catalytic(p.genome))
            n, _ = genome_report(w)
            print(f'{step+1:3d} | 粒子 {len(ps):4d} | 基因组 {n:4d} | 自催化 {sc:3d} ({sc/max(1,len(ps))*100:.1f}%)')

    render(w, obs, 'world3d.svg')
    print(f'最终: {len(w.particles)} 粒子, 能量 {total_energy(w):.1f}')
    print('snapshot -> world3d.svg')


if __name__ == '__main__':
    main()
