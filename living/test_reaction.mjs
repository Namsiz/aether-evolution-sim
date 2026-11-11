// P2d 无头验证 v4：大块砍伐 → 草食动物迁移（质心偏移）。
// 稳定期后砍伐一个 30x30 格大适生块；指标：全体草食到块中心均距、块内占比、草量。
import { Ecology, HERB } from './src/ecology.js';

const N = 76, SPAN = 1080, CELL = SPAN / N, HALF = SPAN / 2;
const SEEDS = [20261102, 20261103, 20261104];
const FOREST = 0.5;
const BASE_S = 720, TEST_S = 480;
const W = 30;

function daylight(tick) { return 0.5 + 0.5 * Math.sin((tick / 300) * Math.PI); }
function run(eco, seconds) {
  for (let i = 0; i < seconds * 10; i++) eco.update(0.1, daylight(eco.tickCount + 1));
}
function cellOf(x, z) { return [Math.floor((x + HALF) / CELL), Math.floor((z + HALF) / CELL)]; }

function pickBlock(eco) {
  let best = -1, bi = 0, bj = 0;
  for (let j0 = 0; j0 + W <= N; j0++) {
    for (let i0 = 0; i0 + W <= N; i0++) {
      let g = 0, w = 0;
      for (let j = j0; j < j0 + W; j++)
        for (let i = i0; i < i0 + W; i++) {
          const idx = j * N + i;
          if (eco.suit[idx] > 0.02) { g += eco.grass[idx] * eco.suit[idx]; w++; }
        }
      const score = w ? g / w : 0;
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

function trial(seed, clearAtEnd) {
  const eco = new Ecology(seed);
  eco.shade.fill(FOREST);
  run(eco, BASE_S);
  const p = pickBlock(eco);
  const t0 = measure(eco, p);
  if (clearAtEnd) {
    for (let j = p.j0; j < p.j0 + W; j++)
      for (let i = p.i0; i < p.i0 + W; i++) eco.shade[j * N + i] = 0;
  }
  run(eco, TEST_S);
  return { p, t0, t1: measure(eco, p) };
}

const acc = { dg: [], dd: [], ds: [] };
console.log('seed     块中心        草量t0→t1(Δ)         均距t0→t1(Δ)        占比t0→t1(Δ)');
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
    `${(E.t0.share * 100).toFixed(1)}→${(E.t1.share * 100).toFixed(1)} (${(ds * 100) >= 0 ? '+' : ''}${(ds * 100).toFixed(1)}%)`
  );
}
const mean = (a) => a.reduce((x, y) => x + y, 0) / a.length;
console.log('\n--- 平均（砍伐-对照） ---');
console.log(`草量Δ ${mean(acc.dg) >= 0 ? '+' : ''}${mean(acc.dg).toFixed(3)}   均距Δ ${mean(acc.dd) >= 0 ? '+' : ''}${mean(acc.dd).toFixed(0)}   占比Δ ${(mean(acc.ds) * 100) >= 0 ? '+' : ''}${(mean(acc.ds) * 100).toFixed(1)}%`);
