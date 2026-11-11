// P2e 无头验证：真实树荫（局部）下的砍伐→草量→动物迁移。
// 从 trees.json 读真实树分布，用 refreshShade 逻辑算局部 shade；砍伐=移除树密集区内所有树。
import { Ecology, HERB } from './src/ecology.js';
import { readFileSync } from 'node:fs';

const N = 76, SPAN = 1080, CELL = SPAN / N, HALF = SPAN / 2;
const SEEDS = [20261102, 20261103, 20261104];
const BASE_S = 720, TEST_S = 480;
const W = 12;
const trees = JSON.parse(readFileSync(new URL('./trees.json', import.meta.url)));

function daylight(tick) { return 0.5 + 0.5 * Math.sin((tick / 300) * Math.PI); }
function run(eco, seconds) {
  for (let i = 0; i < seconds * 10; i++) eco.update(0.1, daylight(eco.tickCount + 1));
}

// 复制 main.js refreshShade 的局部树荫计算
function computeShade(list) {
  const grid = new Float32Array(N * N);
  for (const [x, z, s] of list) {
    const rad = Math.max(4, s * 3.4);
    const ci = Math.floor((x + HALF) / CELL);
    const cj = Math.floor((z + HALF) / CELL);
    const ri = Math.ceil(rad / CELL);
    const r2 = rad * rad;
    for (let j = cj - ri; j <= cj + ri; j++) {
      if (j < 0 || j >= N) continue;
      for (let i = ci - ri; i <= ci + ri; i++) {
        if (i < 0 || i >= N) continue;
        const dx = (i + 0.5) * CELL - HALF - x;
        const dz = (j + 0.5) * CELL - HALF - z;
        if (dx * dx + dz * dz < r2) {
          const idx = j * N + i;
          grid[idx] = Math.min(1, grid[idx] + 0.5);
        }
      }
    }
  }
  return grid;
}

// 砍伐区：树最密的 12x12 适生块
function pickCut(eco) {
  let best = -1, bi = 0, bj = 0;
  for (let j0 = 0; j0 + W <= N; j0++) {
    for (let i0 = 0; i0 + W <= N; i0++) {
      let cnt = 0, su = 0, w = 0;
      for (let j = j0; j < j0 + W; j++)
        for (let i = i0; i < i0 + W; i++) {
          const idx = j * N + i;
          if (eco.suit[idx] > 0.02) { su += eco.suit[idx]; w++; }
          for (const [x, z] of trees) {
            const ci = Math.floor((x + HALF) / CELL), cj = Math.floor((z + HALF) / CELL);
            if (ci === i && cj === j) cnt++;
          }
        }
      const score = w ? cnt + su / w : 0;
      if (score > best) { best = score; bi = i0; bj = j0; }
    }
  }
  const x0 = -HALF + bi * CELL, z0 = -HALF + bj * CELL;
  const x1 = x0 + W * CELL, z1 = z0 + W * CELL;
  return { i0: bi, j0: bj, x0, z0, x1, z1, cx: (x0 + x1) / 2, cz: (z0 + z1) / 2 };
}

function measure(eco, p) {
  let g = 0, w = 0;
  for (let j = p.j0; j < p.j0 + W; j++)
    for (let i = p.i0; i < p.i0 + W; i++) {
      const idx = j * N + i;
      if (eco.suit[idx] > 0.02) { g += eco.grass[idx]; w++; }
    }
  let hp = 0, ht = 0, ds = 0;
  for (const a of eco.agents) {
    if (!a.alive || a.kind !== HERB) continue;
    ht++;
    ds += Math.hypot(a.x - p.cx, a.z - p.cz);
    if (a.x >= p.x0 && a.x < p.x1 && a.z >= p.z0 && a.z < p.z1) hp++;
  }
  const st = eco.stats();
  return { grass: w ? g / w : 0, dist: ht ? ds / ht : 0, share: ht ? hp / ht : 0, H: st.h, P: st.p };
}

function cutTrees(p) {
  return trees.filter(([x, z]) => !(x >= p.x0 && x < p.x1 && z >= p.z0 && z < p.z1));
}

function trial(seed, doCut) {
  const eco = new Ecology(seed);
  eco.shade.set(computeShade(trees));
  run(eco, BASE_S);
  const p = pickCut(eco);
  const t0 = measure(eco, p);
  if (doCut) eco.shade.set(computeShade(cutTrees(p)));
  run(eco, TEST_S);
  return { p, t0, t1: measure(eco, p) };
}

const acc = { dg: [], dd: [], ds: [] };
const probe = new Ecology(SEEDS[0]);
probe.shade.set(computeShade(trees));
run(probe, BASE_S);
const probeP = pickCut(probe);
console.log(`砍伐区((${probeP.cx.toFixed(0)},${probeP.cz.toFixed(0)})) 将移除树数: ${trees.length - cutTrees(probeP).length}`);
console.log('seed     区中心        草量t0→t1(Δ)         均距t0→t1(Δ)        占比t0→t1(Δ)      P末');
for (const seed of SEEDS) {
  const C = trial(seed, false);
  const E = trial(seed, true);
  const dg = E.t1.grass - E.t0.grass - (C.t1.grass - C.t0.grass);
  const dd = E.t1.dist - E.t0.dist - (C.t1.dist - C.t0.dist);
  const ds = E.t1.share - E.t0.share - (C.t1.share - C.t0.share);
  acc.dg.push(dg); acc.dd.push(dd); acc.ds.push(ds);
  console.log(
    `${seed}  (${E.p.cx.toFixed(0)},${E.p.cz.toFixed(0)})  ` +
    `${E.t0.grass.toFixed(3)}→${E.t1.grass.toFixed(3)} (${dg >= 0 ? '+' : ''}${dg.toFixed(3)})  ` +
    `${E.t0.dist.toFixed(0)}→${E.t1.dist.toFixed(0)} (${dd >= 0 ? '+' : ''}${dd.toFixed(0)})  ` +
    `${(E.t0.share * 100).toFixed(1)}→${(E.t1.share * 100).toFixed(1)} (${(ds * 100) >= 0 ? '+' : ''}${(ds * 100).toFixed(1)}%)  ` +
    `${E.t1.P}`
  );
}
const mean = (a) => a.reduce((x, y) => x + y, 0) / a.length;
console.log('\n--- 平均（砍伐-对照） ---');
console.log(`草量Δ ${mean(acc.dg) >= 0 ? '+' : ''}${mean(acc.dg).toFixed(3)}   均距Δ ${mean(acc.dd) >= 0 ? '+' : ''}${mean(acc.dd).toFixed(0)}   占比Δ ${(mean(acc.ds) * 100) >= 0 ? '+' : ''}${(mean(acc.ds) * 100).toFixed(1)}%`);
