#!/usr/bin/env python3
"""
cascade.py — 坍缩世界

世界分两层：
  粗层：一张统计场。永远存在，永远演化。只记录"有多少、多热、往哪走"。
  细层：具体粒子。只在被观察时实例化，观察结束就蒸发回统计量。

观察者不是读者，是世界的成因之一：
视野内的统计态坍缩成粒子，粒子演化、成键、聚变，
离开时再把结果压回统计量。它看过的地方，世界从此不同。
"""

import math
import random
from dataclasses import dataclass, field

SIZE = 2000
CELL = 25
NX = SIZE // CELL
K = 12
SEED = 7

GRAV = 0.03
DIFF = 0.06
INJECT_N = 4
INJECT_E = 3.0
E0 = 1.2

FUSE_E = 4.0
FUSE_LOSS = 0.55
FISSION_E = 2.0
FISSION_LOSS = 0.35
PATCH = 55.0      # 区室边长：粒子基本不跨区室，模拟细胞膜
PATCH_MIGRATE = 0.002  # 偶尔的跨界迁移
BOND_R = 24.0
BOND_L = 11.0
SPRING = 0.03
DIFFUSE = 0.04
MAX_DEG = 7
PARTICLE_GRAV = 0.3
REPEL = 1.0
DAMP = 0.97
VMAX = 0.45

TYPE_COLOR = [
    '#6aa8ff', '#5fe0c8', '#ffb45e', '#c98bff',
    '#ff6b8a', '#8dff6b', '#ffd76b', '#6bfff0',
    '#b06bff', '#ff9a6b', '#6b9aff', '#d0ff6b',
]

CAT_R = 34.0        # 催化剂的作用半径
REACTION_LOSS = 0.45
E_REQ_BASE = 0.8
MUTATE = 0.03       # 反应偶尔出错，产生意外产物
SPONTANEOUS = 0.02  # 没有催化剂时的热涨落

# 遗传层：每个粒子携带一段基因组，可复制、可变异
GENOME_LEN = 6
ALPHABET = 4
MUTATION_RATE = 0.02
REPLICATE_E = 2.4
REPLICATE_COST = 1.1


# ---------------------------------------------------------------
# 数据结构
# ---------------------------------------------------------------


@dataclass
class Cell:
    n: float = 0.0
    energy: float = 0.0
    types: list = field(default_factory=lambda: [1.0 / K] * K)
    vx: float = 0.0
    vy: float = 0.0
    temp: float = 0.0
    instantiated: bool = False
    version: int = 0
    collapsed_at: int = -1


@dataclass
class Particle:
    id: int
    x: float
    y: float
    vx: float
    vy: float
    e: float
    t: int
    cell: tuple
    born: int
    last: int = 0
    genome: tuple = ()


@dataclass
class Event:
    id: int
    tick: int
    level: int
    rule: str
    deps: tuple
    detail: str
    layer: int = 0


class World:
    def __init__(self, seed=SEED):
        self.rng = random.Random(seed)
        self.seed = seed
        self.nx = NX
        self.cell = CELL
        self.size = SIZE
        self.cells = [[Cell() for _ in range(NX)] for _ in range(NX)]
        self.particles = {}
        self.edges = set()
        self.reactions = set()
        self.mutations = 0
        self.next_id = 0
        self.events = []
        self.tick = 0
        self.collapses = 0
        self.evaporations = 0
        self._init_field()

    # ---------------------------------------------------------
    # 创世：分形密度场
    # ---------------------------------------------------------
    def _init_field(self):
        nx = self.nx
        f = [[0.0] * nx for _ in range(nx)]
        rng = self.rng
        for scale in (nx * 0.55, nx * 0.28, nx * 0.14):
            for _ in range(9):
                cx = rng.uniform(0, nx)
                cy = rng.uniform(0, nx)
                amp = rng.uniform(0.5, 1.0) * scale
                s2 = 2.0 * scale * scale
                for i in range(nx):
                    for j in range(nx):
                        d2 = (i - cx) ** 2 + (j - cy) ** 2
                        f[i][j] += amp * math.exp(-d2 / s2)

        total = sum(sum(row) for row in f)
        target = nx * nx * 4.0
        k = target / total if total > 0 else 0
        for i in range(nx):
            for j in range(nx):
                c = self.cells[i][j]
                c.n = f[i][j] * k
                c.energy = c.n * E0
                # 创世时只有少数几种元素，其余类型必须由反应自己造出来
                c.types = [1.0 / 6 if i2 < 6 else 0.0 for i2 in range(K)]

    # ---------------------------------------------------------
    # 粗层演化
    # ---------------------------------------------------------
    def coarse_step(self, dt=1.0):
        nx = self.nx
        cells = self.cells
        G = GRAV * dt
        D = DIFF * dt
        dn = [[0.0] * nx for _ in range(nx)]
        de = [[0.0] * nx for _ in range(nx)]

        for i in range(nx):
            row = cells[i]
            dnrow = dn[i]
            derow = de[i]
            for j in range(nx):
                c = row[j]
                if c.instantiated:
                    continue
                cn = c.n
                ce = c.energy
                for di, dj in ((1, 0), (0, 1)):
                    ni = (i + di) % nx
                    nj = (j + dj) % nx
                    o = cells[ni][nj]
                    if o.instantiated:
                        continue      # 被观察的格及其边界冻结，交给细层
                    diff = cn - o.n
                    flow = -diff * G if diff < 0 else diff * D
                    if flow <= 0:
                        continue
                    if flow > cn * 0.25:
                        flow = cn * 0.25
                    dnrow[j] -= flow
                    dn[ni][nj] += flow
                    if cn > 1e-9:
                        ef = flow * ce / cn
                        derow[j] -= ef
                        de[ni][nj] += ef

        for i in range(nx):
            for j in range(nx):
                c = cells[i][j]
                if c.instantiated:
                    continue
                c.n = max(0.0, c.n + dn[i][j])
                c.energy = max(0.0, c.energy + de[i][j])

        for _ in range(INJECT_N):
            i = self.rng.randrange(nx)
            j = self.rng.randrange(nx)
            if not cells[i][j].instantiated:
                cells[i][j].energy += INJECT_E

        self.tick += 1

    # ---------------------------------------------------------
    # 实例化：统计态 -> 粒子
    # 种子由 (世界种子, 格坐标, 版本号) 决定。版本号只在粗化时 +1。
    # 所以同一个没被扰动过的格，无论谁、何时观察，坍缩结果都一样。
    # ---------------------------------------------------------
    def instantiate(self, cx, cy):
        cell = self.cells[cx][cy]
        if cell.instantiated:
            return 0

        seed = (
            self.seed * 1000003
            ^ (cx * 7919 + cy * 104729) * 31
            ^ (cell.version + 1) * 31337
        )
        rng = random.Random(seed)

        count = poisson(rng, cell.n)
        if count == 0:
            return 0

        e_each = cell.energy / count
        if e_each < 0.05:
            return 0

        cell.instantiated = True
        cell.collapsed_at = self.tick
        self.collapses += 1

        for _ in range(count):
            x = (cx + rng.random()) * self.cell
            y = (cy + rng.random()) * self.cell
            t = weighted(rng, cell.types)
            g = random_genome(rng)
            vx = cell.vx + rng.gauss(0, cell.temp + 0.05)
            vy = cell.vy + rng.gauss(0, cell.temp + 0.05)
            pid = self.next_id
            self.next_id += 1
            self.particles[pid] = Particle(
                id=pid, x=x, y=y, vx=vx, vy=vy,
                e=e_each, t=t, cell=(cx, cy),
                born=self.tick, last=0, genome=g,
            )
        return count

    # ---------------------------------------------------------
    # 粗化：粒子 -> 统计态
    # ---------------------------------------------------------
    def coarsen(self, cx, cy):
        cell = self.cells[cx][cy]
        if not cell.instantiated:
            return
        ps = [p for p in self.particles.values() if p.cell == (cx, cy)]
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
            cell.temp = math.sqrt(
                sum((p.vx - cell.vx) ** 2 + (p.vy - cell.vy) ** 2 for p in ps) / n
            )
        else:
            cell.n = 0.0
            cell.energy = 0.0
        for p in ps:
            self.particles.pop(p.id, None)
        cell.instantiated = False
        cell.version += 1
        self.evaporations += 1

    # ---------------------------------------------------------
    # 粒子出口：离开已实例化的区域就蒸发
    # ---------------------------------------------------------
    def relocate(self, p):
        cx = int(p.x // self.cell)
        cy = int(p.y // self.cell)
        cx = min(max(cx, 0), self.nx - 1)
        cy = min(max(cy, 0), self.nx - 1)
        if (cx, cy) == p.cell:
            return True
        if self.cells[cx][cy].instantiated:
            p.cell = (cx, cy)
            return True
        c = self.cells[cx][cy]
        old = c.n
        c.n += 1.0
        c.energy += p.e
        if old > 0:
            c.vx = (c.vx * old + p.vx) / (old + 1)
            c.vy = (c.vy * old + p.vy) / (old + 1)
        self.particles.pop(p.id, None)
        return False

    # ---------------------------------------------------------
    # 事件
    # ---------------------------------------------------------
    def emit(self, level, rule, deps, detail):
        eid = len(self.events)
        layer = 0
        for d in deps:
            if d is not None and 0 <= d < len(self.events):
                layer = max(layer, self.events[d].layer + 1)
        self.events.append(
            Event(id=eid, tick=self.tick, level=level, rule=rule,
                  deps=tuple(deps), detail=detail, layer=layer)
        )
        return eid


# ---------------------------------------------------------------
# 工具
# ---------------------------------------------------------------


def poisson(rng, lam):
    if lam <= 0:
        return 0
    if lam > 40:
        return max(0, int(round(rng.gauss(lam, math.sqrt(lam)))))
    L = math.exp(-lam)
    k = 0
    p = 1.0
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
    """前三位与后三位相同，意味着它能催化自己的复制。"""
    return len(g) == 6 and g[:3] == g[3:]


def catalyses(a, b):
    """a 的催化位点（前三位）是否匹配 b 的反应位点（后三位）。
    这是“另一个基因组能帮我复制”的判定。"""
    return len(a) == 6 and len(b) == 6 and a[:3] == b[3:]


def dist(a, b):
    return math.hypot(a.x - b.x, a.y - b.y)


def reaction(a, b):
    """A + B 在催化剂 K 附近 -> C。K 不被消耗。
    这个映射在 K=12 时有 4 个非平凡自催化环，环之间可以竞争。"""
    return (a * b + 1) % K, (a + b) % K


def total_energy(world):
    e = 0.0
    for i in range(world.nx):
        for j in range(world.nx):
            if not world.cells[i][j].instantiated:
                e += world.cells[i][j].energy
    e += sum(p.e for p in world.particles.values())
    return e


def scc(nodes, adj):
    """Tarjan 强连通分量：催化剂图上的环就是代谢闭环的候选。"""
    idx, low, on, stack, comps, counter = {}, {}, {}, [], [], [0]

    def visit(v):
        idx[v] = low[v] = counter[0]
        counter[0] += 1
        stack.append(v)
        on[v] = True
        for w in adj.get(v, ()):
            if w not in idx:
                visit(w)
                low[v] = min(low[v], low[w])
            elif on.get(w):
                low[v] = min(low[v], idx[w])
        if low[v] == idx[v]:
            comp = []
            while True:
                w = stack.pop()
                on[w] = False
                comp.append(w)
                if w == v:
                    break
            comps.append(comp)

    for v in nodes:
        if v not in idx:
            visit(v)
    return [c for c in comps if len(c) > 1]


def raf_report(world):
    """从实际发生过的反应里抽出催化剂图，找自催化集。"""
    adj = {}
    for a, b, k, c in world.reactions:
        adj.setdefault(k, set()).add(c)
    comps = scc(range(K), adj)
    comps.sort(key=len, reverse=True)
    return comps


def genome_report(world, top=5):
    """基因组多样性：有多少种序列，最常见的是哪几个。"""
    from collections import Counter
    c = Counter(p.genome for p in world.particles.values())
    return len(c), c.most_common(top)


def catalytic_ecology(world, min_count=2):
    """统计常见基因组之间的催化关系：有向边和互催化对。
    互催化对是“合作”的起源；只单向的则是“寄生”。"""
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


def spatial_report(world, obs, bins=8):
    L, T = obs.x - obs.r, obs.y - obs.r
    span = obs.r * 2
    bw = span / bins

    acc = {}
    for p in world.particles.values():
        i = int((p.x - L) / bw)
        j = int((p.y - T) / bw)
        if 0 <= i < bins and 0 <= j < bins:
            acc.setdefault((i, j), [0] * K)[p.t] += 1

    dist = {}
    for k, counts in acc.items():
        n = sum(counts)
        if n >= 3:
            dist[k] = [c / n for c in counts]

    edges = []
    for (i, j), a in dist.items():
        for di, dj in ((1, 0), (0, 1)):
            b = dist.get((i + di, j + dj))
            if b is None:
                continue
            edges.append(sum(abs(a[t] - b[t]) for t in range(K)) / 2.0)

    dominant = {}
    for d in dist.values():
        top = max(range(K), key=lambda t: d[t])
        dominant[top] = dominant.get(top, 0) + 1

    return {
        'cells': len(dist),
        'border': sum(edges) / len(edges) if edges else 0.0,
        'dominant': dominant,
        'dist': dist,
        'bins': bins,
        'origin': (L, T, bw),
    }


def add_edge(world, a, b):
    if a != b:
        world.edges.add((a, b) if a < b else (b, a))


def del_edge(world, a, b):
    world.edges.discard((a, b) if a < b else (b, a))


def kill(world, pid):
    world.particles.pop(pid, None)
    for e in [e for e in world.edges if pid in e]:
        world.edges.discard(e)


# ---------------------------------------------------------------
# 细层
# ---------------------------------------------------------------


def fine_step(world):
    ps = list(world.particles.values())
    if not ps:
        return
    rng = world.rng
    budget = {'bond': 8, 'break': 2, 'fuse': 1, 'fission': 2, 'radiate': 1, 'decay': 2, 'replicate': 3}
    by_id = {p.id: p for p in ps}

    adj = {p.id: set() for p in ps}
    live = set()
    for a, b in world.edges:
        if a in by_id and b in by_id:
            adj[a].add(b)
            adj[b].add(a)
            live.add((a, b))
    world.edges = live

    for a, b in live:
        pa, pb = by_id[a], by_id[b]
        d = dist(pa, pb) + 1e-6
        de = (pa.e - pb.e) * DIFFUSE
        pa.e = max(0.0, pa.e - de)
        pb.e += de
        f = (d - BOND_L) * SPRING
        ux, uy = (pb.x - pa.x) / d, (pb.y - pa.y) / d
        pa.vx += f * ux
        pa.vy += f * uy
        pb.vx -= f * ux
        pb.vy -= f * uy

    gc = 90.0
    grid = {}
    for p in ps:
        grid.setdefault((int(p.x // gc), int(p.y // gc)), []).append(p)
    for p in ps:
        cx, cy = int(p.x // gc), int(p.y // gc)
        for dx in (-1, 0, 1):
            for dy in (-1, 0, 1):
                for q in grid.get((cx + dx, cy + dy), ()):
                    if q.id == p.id:
                        continue
                    ddx, ddy = q.x - p.x, q.y - p.y
                    raw2 = ddx * ddx + ddy * ddy
                    raw = math.sqrt(raw2) + 1e-6
                    soft2 = raw2 + 120.0
                    f = PARTICLE_GRAV * q.e / soft2
                    # 类型 0 扮演表面活性剂：自己人相吸，异类相斥。
                    # 这是全系统唯一一处用“物理”而不是“化学”去制造边界的地方。
                    if p.t == 0 and q.t == 0:
                        aff = 2.4
                    elif p.t == 0 or q.t == 0:
                        aff = -1.6
                    else:
                        aff = 1.0
                    f *= aff
                    if raw < 16.0 and aff > 0:
                        f -= REPEL * q.e / soft2      # 短程排斥：没有它，世界会塌成一个点
                    p.vx += f * ddx / raw
                    p.vy += f * ddy / raw

        for p in ps:
            p.vx *= DAMP
            p.vy *= DAMP
            sp = math.hypot(p.vx, p.vy)
            # 重的跑得慢，轻的跑得快：反应网络才能长成斑图而不是一锅汤
            vmax = VMAX / (1.0 + p.e * 0.55)
            if sp > vmax:
                p.vx *= vmax / sp
                p.vy *= vmax / sp
        # 区室化：粒子被限制在 PATCH 格子里，偶尔才能跨界
        if rng.random() > PATCH_MIGRATE:
            px = int(p.x // PATCH)
            py = int(p.y // PATCH)
            lo_x, hi_x = px * PATCH + 2, (px + 1) * PATCH - 2
            lo_y, hi_y = py * PATCH + 2, (py + 1) * PATCH - 2
            p.x = min(max(p.x, lo_x), hi_x)
            p.y = min(max(p.y, lo_y), hi_y)
        else:
            p.x = min(max(p.x + p.vx, 0.0), world.size)
            p.y = min(max(p.y + p.vy, 0.0), world.size)

    # 细层的背景能量：没有它，粒子会在一轮聚变后全部 froze
    for _ in range(3):
        if ps:
            rng.choice(ps).e += 0.4

    used = set()

    # 成键
    bg = {}
    for p in ps:
        bg.setdefault((int(p.x // BOND_R), int(p.y // BOND_R)), []).append(p)
    for p in ps:
        if budget['bond'] <= 0:
            break
        if p.id in used or len(adj[p.id]) >= MAX_DEG:
            continue
        cx, cy = int(p.x // BOND_R), int(p.y // BOND_R)
        best, bd = None, 1e9
        for dx in (-1, 0, 1):
            for dy in (-1, 0, 1):
                for q in bg.get((cx + dx, cy + dy), ()):
                    if q.id == p.id or q.id in adj[p.id] or q.id in used:
                        continue
                    if len(adj[q.id]) >= MAX_DEG:
                        continue
                    if (p.t + q.t) % K != 1:
                        continue
                    d = dist(p, q)
                    if d <= BOND_R and d < bd:
                        bd, best = d, q
        if best is not None:
            eid = world.emit(0, 'bond', (p.last, best.last), f'{p.id}~{best.id}')
            add_edge(world, p.id, best.id)
            adj[p.id].add(best.id)
            adj[best.id].add(p.id)
            p.last = best.last = eid
            used.add(p.id)
            used.add(best.id)
            budget['bond'] -= 1

    # 断键
    for a, b in list(world.edges):
        if budget['break'] <= 0:
            break
        pa, pb = by_id.get(a), by_id.get(b)
        if pa and pb and dist(pa, pb) > BOND_R * 1.8:
            eid = world.emit(0, 'break', (pa.last, pb.last), f'{a}/{b}')
            del_edge(world, a, b)
            pa.last = pb.last = eid
            budget['break'] -= 1

    # 催化反应：A + B --[K]--> C
    # 催化剂 K 必须在附近，但不被消耗。这是死物质与活代谢之间的分界线。
    if budget['fuse'] > 0:
        tg = {}
        for p in ps:
            tg.setdefault((int(p.x // CAT_R), int(p.y // CAT_R)), []).append(p)
        for a, b in list(world.edges):
            if budget['fuse'] <= 0:
                break
            pa, pb = by_id.get(a), by_id.get(b)
            if not pa or not pb:
                continue

            # 类型 0 是惰性的表面活性剂，不参与反应，只负责造边界
            if pa.t == 0 or pb.t == 0:
                continue

            c_type, k_type = reaction(pa.t, pb.t)
            e_req = E_REQ_BASE + ((pa.t + pb.t) % 3) * 0.3
            if pa.e + pb.e < e_req:
                continue

            cat = None
            cx = int(pa.x // CAT_R)
            cy = int(pa.y // CAT_R)
            for dx in (-1, 0, 1):
                for dy in (-1, 0, 1):
                    for q in tg.get((cx + dx, cy + dy), ()):
                        if q.t != k_type or q.id in (a, b):
                            continue
                        if dist(pa, q) <= CAT_R:
                            cat = q
                            break
                    if cat:
                        break
                if cat:
                    break
            if cat is None:
                if rng.random() > SPONTANEOUS:
                    continue
                # 无催化剂的自发反应：世界的热涨落，也是新类型的唯一出口

            mutated = rng.random() < MUTATE
            if mutated:
                c_type = rng.randrange(K)
                world.mutations += 1

            world.reactions.add((pa.t, pb.t, k_type, c_type))
            eid = world.emit(
                0, 'react',
                (pa.last, pb.last, cat.last if cat else -1),
                f'{pa.t}+{pb.t}--[{k_type if cat else "~"}]-->{"?" if mutated else c_type}',
            )
            ne = pa.e + pb.e - REACTION_LOSS
            # 产物出现在催化剂所在处，而不是中点。
            # 这让“催化”获得空间意义：产物聚集在催化剂周围，而不是被搅匀。
            if cat is not None:
                nx_, ny_ = cat.x, cat.y
            else:
                nx_, ny_ = (pa.x + pb.x) / 2, (pa.y + pb.y) / 2
            nvx, nvy = (pa.vx + pb.vx) / 2, (pa.vy + pb.vy) / 2
            keep = (adj[a] | adj[b]) - {a, b}
            kill(world, a)
            kill(world, b)
            for k in (a, b):
                by_id.pop(k, None)
                adj.pop(k, None)
            pid = world.next_id
            world.next_id += 1
            # 反应产物的基因组从两个反应物之一继承，偶尔重缎
            child_g = pa.genome if rng.random() < 0.5 else pb.genome
            np = Particle(pid, nx_, ny_, nvx, nvy, ne, c_type, (0, 0), world.tick, eid, child_g)
            world.particles[pid] = np
            by_id[pid] = np
            adj[pid] = set()
            for k in keep:
                if k in by_id:
                    add_edge(world, pid, k)
                    adj[pid].add(k)
                    adj[k].add(pid)
            used.add(pid)
            if cat is not None:
                cat.last = eid
            budget['fuse'] -= 1

    # 裂变
    for p in list(world.particles.values()):
        if budget['fission'] <= 0:
            break
        if p.e < FISSION_E or rng.random() > 0.05:
            continue
        eid = world.emit(0, 'fission', (p.last,), f'{p.id}')
        home = p.cell
        nbrs = list(adj.get(p.id, ()))
        kill(world, p.id)
        by_id.pop(p.id, None)
        adj.pop(p.id, None)
        for i in range(2):
            pid = world.next_id
            world.next_id += 1
            np = Particle(
                pid,
                p.x + (rng.random() - 0.5) * 40,
                p.y + (rng.random() - 0.5) * 40,
                p.vx + rng.uniform(-0.6, 0.6),
                p.vy + rng.uniform(-0.6, 0.6),
                p.e / 2 - FISSION_LOSS,
                (p.t + i) % K,
                home, world.tick, eid,
                mutate(p.genome, rng),
            )
            world.particles[pid] = np
            by_id[pid] = np
            adj[pid] = set()
            for k in nbrs[i::2]:
                if k in by_id:
                    add_edge(world, pid, k)
                    adj[pid].add(k)
                    adj[k].add(pid)
        budget['fission'] -= 1

    # 辐射
    for p in list(world.particles.values()):
        if budget['radiate'] <= 0:
            break
        if p.e < 9.0 or rng.random() > 0.25:
            continue
        eid = world.emit(0, 'radiate', (p.last,), f'*{p.id}')
        p.e -= 3.0
        p.last = eid
        ang = rng.random() * math.tau
        pid = world.next_id
        world.next_id += 1
        np = Particle(
            pid,
            p.x + math.cos(ang) * 30,
            p.y + math.sin(ang) * 30,
            p.vx + math.cos(ang) * 1.1,
            p.vy + math.sin(ang) * 1.1,
            1.0, rng.randrange(K), p.cell, world.tick, eid, p.genome,
        )
        world.particles[pid] = np
        by_id[pid] = np
        adj[pid] = set()
        if rng.random() < 0.35:
            add_edge(world, pid, p.id)
            adj[pid].add(p.id)
            adj[p.id].add(pid)
        budget['radiate'] -= 1

    # 复制：基因组第一次真正起作用。子代继承序列，可能变异。
    # 催化分三档：自催化 0.45 / 被邻居催化 0.65 / 无人催化 1.0。
    gg = {}
    if budget['replicate'] > 0:
        for q in ps:
            gg.setdefault((int(q.x // CAT_R), int(q.y // CAT_R)), []).append(q)

    for p in list(world.particles.values()):
        if budget['replicate'] <= 0:
            break

        boost = 1.0
        if is_self_catalytic(p.genome):
            boost = 0.45
        elif gg and p.genome:
            cx = int(p.x // CAT_R)
            cy = int(p.y // CAT_R)
            for dx in (-1, 0, 1):
                for dy in (-1, 0, 1):
                    for q in gg.get((cx + dx, cy + dy), ()):
                        if q.id != p.id and catalyses(q.genome, p.genome):
                            boost = 0.65
                            break
                    if boost < 1.0:
                        break
                if boost < 1.0:
                    break

        if p.e < REPLICATE_E * boost:
            continue
        g = mutate(p.genome, rng)
        nt = genome_type(g)
        eid = world.emit(0, 'replicate', (p.last,), f'{p.id}->{sum(g)}')
        p.e -= REPLICATE_COST
        p.last = eid
        ang = rng.random() * math.tau
        pid = world.next_id
        world.next_id += 1
        np = Particle(
            pid,
            p.x + math.cos(ang) * 16,
            p.y + math.sin(ang) * 16,
            p.vx + math.cos(ang) * 0.4,
            p.vy + math.sin(ang) * 0.4,
            REPLICATE_COST * 0.82,
            nt, p.cell, world.tick, eid, g,
        )
        world.particles[pid] = np
        budget['replicate'] -= 1

    # 衰变
    for p in list(world.particles.values()):
        if budget['decay'] <= 0:
            break
        if p.e < 0.12 and not adj.get(p.id):
            world.emit(0, 'decay', (p.last,), f'~{p.id}')
            kill(world, p.id)
            budget['decay'] -= 1

    for p in list(world.particles.values()):
        world.relocate(p)


# ---------------------------------------------------------------
# 观察者
# ---------------------------------------------------------------


class Observer:
    def __init__(self, world, x, y, r):
        self.world = world
        self.x, self.y, self.r = x, y, r
        self.active = set()
        self.seen = {}
        self.path = [(x, y)]

    def cells_in_view(self):
        cs = self.world.cell
        out = []
        i0 = max(0, int((self.x - self.r) // cs))
        i1 = min(self.world.nx - 1, int((self.x + self.r) // cs))
        j0 = max(0, int((self.y - self.r) // cs))
        j1 = min(self.world.nx - 1, int((self.y + self.r) // cs))
        for i in range(i0, i1 + 1):
            for j in range(j0, j1 + 1):
                cx = (i + 0.5) * cs
                cy = (j + 0.5) * cs
                if math.hypot(cx - self.x, cy - self.y) <= self.r + cs * 0.7:
                    out.append((i, j))
        return out

    def look(self):
        view = set(self.cells_in_view())
        for c in view:
            self.world.instantiate(*c)
        for c in list(self.active - view):
            self.world.coarsen(*c)
        self.active = view
        for c in view:
            cell = self.world.cells[c[0]][c[1]]
            self.seen[c] = (cell.n, cell.energy)

    def move_to(self, x, y):
        self.x, self.y = x, y
        self.path.append((x, y))


# ---------------------------------------------------------------
# 渲染
# ---------------------------------------------------------------


def render(world, obs, path, S=820):
    k = S / world.size
    cs = world.cell * k
    out = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{S}" height="{S}" '
        f'viewBox="0 0 {S} {S}">',
        f'<rect width="{S}" height="{S}" fill="#03050a"/>',
    ]

    # 粗层：一张星点场。每个点是一个统计格，只有密度，没有身份。
    for i in range(world.nx):
        for j in range(world.nx):
            c = world.cells[i][j]
            v = min(1.0, c.n / 16.0)
            if v < 0.02:
                continue
            rr = cs * (0.22 + 0.55 * v)
            r = int(18 + 80 * v)
            g = int(30 + 115 * v)
            b = int(60 + 175 * v)
            op = 0.08 + 0.80 * v
            if c.instantiated:
                op *= 0.45
            out.append(
                f'<circle cx="{(i+0.5)*cs:.1f}" cy="{(j+0.5)*cs:.1f}" r="{rr:.1f}" '
                f'fill="rgb({r},{g},{b})" opacity="{op:.2f}"/>'
            )

    # 细层：被观察处的具体粒子
    for p in world.particles.values():
        rr = 2.2 + (p.e ** 0.6) * 2.6
        out.append(
            f'<circle cx="{p.x*k:.1f}" cy="{p.y*k:.1f}" r="{rr:.1f}" '
            f'fill="{TYPE_COLOR[p.t % K]}" opacity="0.92"/>'
        )

    out.append(
        f'<circle cx="{obs.x*k:.1f}" cy="{obs.y*k:.1f}" r="{obs.r*k:.1f}" '
        f'fill="none" stroke="#9fe0ff" stroke-width="1.2" opacity="0.6"/>'
    )
    if len(obs.path) > 1:
        pts = ' '.join(f'{x*k:.1f},{y*k:.1f}' for x, y in obs.path)
        out.append(
            f'<polyline points="{pts}" fill="none" stroke="#9fe0ff" '
            f'stroke-width="0.9" stroke-dasharray="5 7" opacity="0.35"/>'
        )

    out.append('</svg>')
    with open(path, 'w') as f:
        f.write('\n'.join(out))


# ---------------------------------------------------------------
# 实验
# ---------------------------------------------------------------


def best_spot(world, rad):
    """找一个密度最高的观察点，别让观察者站在虚空里。"""
    best, bs = (world.size / 2, world.size / 2), -1.0
    span = max(1, int(rad // world.cell))
    for i in range(world.nx):
        for j in range(world.nx):
            s = 0.0
            for di in range(-span, span + 1):
                for dj in range(-span, span + 1):
                    s += world.cells[(i + di) % world.nx][(j + dj) % world.nx].n
            if s > bs:
                bs = s
                best = ((i + 0.5) * world.cell, (j + 0.5) * world.cell)
    return best


def selftest():
    print('--- 自检 ---')
    w = World()
    for _ in range(50):
        w.coarse_step()

    e0 = total_energy(w)
    n1 = w.instantiate(40, 40)
    e1 = total_energy(w)
    a = sorted(
        (round(p.x, 9), round(p.y, 9), p.t)
        for p in w.particles.values()
    )

    # 第二个观察者看同一个格：不粗化，直接把粒子抹掉重来
    w.particles.clear()
    w.cells[40][40].instantiated = False
    n2 = w.instantiate(40, 40)
    b = sorted(
        (round(p.x, 9), round(p.y, 9), p.t)
        for p in w.particles.values()
    )

    print(f'能量守恒: {e0:.6f} -> {e1:.6f}  差 {e1-e0:+.9f}')
    print(f'两次独立观察同一格: {n1} vs {n2} 个, 坐标/类型完全一致: {a == b}')
    print()


def main():
    selftest()
    w = World()
    cells = w.nx * w.nx
    avg = sum(c.n for row in w.cells for c in row) / cells
    print(f'创世: {w.nx}x{w.nx} 格, 每格平均 {avg:.2f} 个存在物')
    print(f'总能量 {total_energy(w):.1f}')

    for _ in range(400):
        w.coarse_step()

    # 打印密度分布，看世界到底长成什么形状
    vals = sorted(c.n for row in w.cells for c in row)
    q = lambda p: vals[int(p * (len(vals) - 1))]
    print(f'粗层 400 步: 密度分位 10%={q(0.1):.2f} 50%={q(0.5):.2f} '
          f'90%={q(0.9):.2f} 99%={q(0.99):.2f} 峰={vals[-1]:.2f}')
    print(f'总能量 {total_energy(w):.1f}')

    ox, oy = best_spot(w, 170)
    print(f'观察点选在最密处 ({ox:.0f},{oy:.0f})')
    obs = Observer(w, ox, oy, 170)
    obs.look()
    print(f'第一次坍缩: {len(w.particles)} 个粒子, 总能量 {total_energy(w):.1f}')

    for _ in range(200):
        w.coarse_step()
        obs.look()
        fine_step(w)
    print(f'观察 200 步后: {len(w.particles)} 个粒子, 总能量 {total_energy(w):.1f}')

    render(w, obs, 'collapse_before.svg')

    obs.move_to(min(ox + 220, w.size - 60), max(oy - 160, 60))
    obs.look()
    print(f'移动视野后: {len(w.particles)} 个粒子, 总能量 {total_energy(w):.1f}')

    for _ in range(150):
        w.coarse_step()
        obs.look()
        fine_step(w)

    render(w, obs, 'collapse.svg')

    print(f'坍缩 {w.collapses} 次, 蒸发 {w.evaporations} 次')
    print(f'事件 {len(w.events)} 个, 其中细层 {sum(1 for e in w.events if e.level == 0)} 个')
    print(f'最深因果链 {max((e.layer for e in w.events), default=0)} 层')

    comps = raf_report(w)
    print(f'反应种类 {len(w.reactions)}, 意外变异 {w.mutations}')
    print(f'自催化环: {[len(c) for c in comps]}')
    for c in comps[:4]:
        print('   ', sorted(c))

    sp = spatial_report(w, obs, 8)
    print(f'空间: {sp["cells"]} 个有效格, 边界强度 {sp["border"]:.3f}, '
          f'主导类型分布 {dict(sorted(sp["dominant"].items()))}')

    print(f'最终: {len(w.particles)} 个粒子, 总能量 {total_energy(w):.1f}')
    print('snapshot -> collapse.svg')


if __name__ == '__main__':
    main()
