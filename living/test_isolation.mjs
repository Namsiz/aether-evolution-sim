// 无头验证：生殖隔离（配对壁垒）对生态稳态与物种形成闭环的影响
// 每个 seed 跑两遍对照：barrier=0.5（默认）vs barrier=0（干净对照，行为与改动前一致）
// 用法：node test_isolation.mjs [分钟数=120] [seed列表逗号分隔]
import { Ecology } from './src/ecology.js';

const minutes = Number(process.argv[2] || 120);
const seeds = (process.argv[3] || '11,22,33,44,55,66').split(',').map(Number);
const totalTicks = minutes * 600;

// 单次模拟：返回种群极值、终态与隔离观测
function runSim(seed, barrier) {
  const eco = new Ecology(seed);
  eco.GENE_BARRIER_STRENGTH = barrier; // 实例字段；置 0 时配对壁垒完全关闭（干净对照）
  let minH = 999, minP = 999, minS = 999, minO = 999;
  let hadNaN = false;
  let firstDist = null, lastDist = null;
  const count = () => {
    let h = 0, p = 0, s = 0, o = 0;
    for (const a of eco.agents) {
      if (!a.alive) continue;
      if (!Number.isFinite(a.x) || !Number.isFinite(a.z) || !Number.isFinite(a.energy) ||
          !Number.isFinite(a.genome.speed) || !Number.isFinite(a.genome.sense)) { hadNaN = true; break; }
      if (a.kind === 'herb') h++;
      else if (a.kind === 'pred') p++;
      else if (a.kind === 'scav') s++;
      else o++;
    }
    return { h, p, s, o };
  };
  for (let i = 1; i <= totalTicks; i++) {
    const daylight = 0.5 + 0.5 * Math.sin((i / 300) * Math.PI);
    eco.update(0.1, daylight);
    if (i % 600 === 0) {
      const c = count();
      minH = Math.min(minH, c.h); minP = Math.min(minP, c.p);
      minS = Math.min(minS, c.s); minO = Math.min(minO, c.o);
      const s2 = eco.stats();
      if (s2.isolationHistory.length) {
        const d = s2.isolationHistory[s2.isolationHistory.length - 1].dist;
        if (firstDist === null) firstDist = d;
        lastDist = d;
      }
    }
  }
  const fin = count();
  const s = eco.stats();
  const trend = firstDist !== null && lastDist !== null
    ? `${firstDist.toFixed(3)} → ${lastDist.toFixed(3)} (${lastDist > firstDist ? '↑' : lastDist < firstDist ? '↓' : '＝'}${Math.abs(lastDist - firstDist).toFixed(3)})`
    : '无隔离记录';
  return {
    minH, minP, minS, minO, hadNaN, trend,
    fin, stats: s,
  };
}

function fmt(r, barrier) {
  const s = r.stats;
  return `barrier=${barrier}: 终态 H=${r.fin.h} P=${r.fin.p} S=${r.fin.s} O=${r.fin.o} | 全程最小 H=${r.minH} P=${r.minP} | P存活=${r.fin.p > 0 ? '是' : '否'} NaN=${r.hadNaN ? '有' : '无'} ` +
    `出生=${s.births} 阻断=${s.barrierBlocks} 隔离群=${s.isolatedGroups.map(g=>`${g.name}(${g.count})`).join(',')||'无'} 基因距离=${r.trend}`;
}

let pAliveBarrier = 0, pAliveControl = 0;
let allOk = true;
const rows = [];

for (const seed of seeds) {
  const withB = runSim(seed, 0.5);
  const ctrl = runSim(seed, 0);
  if (withB.hadNaN || ctrl.hadNaN) allOk = false;
  if (withB.fin.p > 0) pAliveBarrier++;
  if (ctrl.fin.p > 0) pAliveControl++;
  rows.push(`seed ${seed}\n  [对照] ${fmt(ctrl, 0)}\n  [实验] ${fmt(withB, 0.5)}`);
}

console.log(`=== 生殖隔离无头验证（双组对照）：${seeds.length} seed × ${minutes}min，barrier 0.5 vs 0 ===`);
for (const r of rows) console.log(r);
console.log(`P 终态存活：实验(0.5) ${pAliveBarrier}/${seeds.length}，对照(0) ${pAliveControl}/${seeds.length}，NaN=${allOk ? '无' : '有'}`);
process.exit(allOk ? 0 : 1);
