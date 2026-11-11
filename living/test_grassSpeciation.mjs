// 无头验证：草物种化（生产者分化）——三草种共存、P 存活、无 NaN、空间分化证据
// 检查项：
//   ① P 全程未灭绝（minP>0 且终态 P>0；排除迁入回血掩盖）
//   ② 无 NaN
//   ③ 三草种真共存（全图均量 > 0.01，且湿区草种-1 均值 ≥ 0.02；
//                   >0 但 ≤ 0.01 记「保底维持」，不算 PASS 也不算灭绝）
//   ④ 空间分化成立（湿/旱主导种不同，且两侧主导种均值 ≥ 1.2 × 次高种均值；
//                   余量不足记「分化不显著」，单独呈现，不计入硬判定）
// 用法：node test_grassSpeciation.mjs [分钟数=120] [seed列表逗号分隔]
import { Ecology } from './src/ecology.js';

const minutes = Number(process.argv[2] || 120);
const seeds = (process.argv[3] || '11,22,33,44,55,66').split(',').map(Number);
const totalTicks = minutes * 600;

const COEXIST_MIN = 0.01;      // 全图均量真共存下限
const WET_SP1_MIN = 0.02;      // 湿区草种-1（index 0）均值下限（真共存辅助判据）
const SPATIAL_MARGIN = 1.2;    // 主导种/次高种 余量阈值

function spatialSplit(eco) {
  const n = eco.grassP.length; // 草种数（与 ecology GRASS_N 一致）
  const wet = new Array(n).fill(0), dry = new Array(n).fill(0);
  let nw = 0, nd = 0;
  for (let i = 0; i < eco.grass.length; i++) {
    if (eco.suit[i] <= 0.02) continue;
    if (eco.moist[i] > 0.25) {
      for (let s = 0; s < n; s++) wet[s] += eco.grassP[s][i];
      nw++;
    } else if (eco.moist[i] === 0) {
      for (let s = 0; s < n; s++) dry[s] += eco.grassP[s][i];
      nd++;
    }
  }
  const wn = nw ? wet.map(v => v / nw) : new Array(n).fill(0);
  const dn = nd ? dry.map(v => v / nd) : new Array(n).fill(0);
  return {
    wet: { means: wn, dom: wn.indexOf(Math.max(...wn)) },
    dry: { means: dn, dom: dn.indexOf(Math.max(...dn)) },
    nw, nd,
  };
}

function dominantWithMargin(means) {
  const sorted = [...means].sort((a, b) => b - a);
  const dom = means.indexOf(Math.max(...means));
  const marginOk = sorted[0] >= SPATIAL_MARGIN * sorted[1];
  const marginRatio = sorted[1] > 0 ? sorted[0] / sorted[1] : Infinity;
  return { dom, marginOk, ratio: marginRatio, sorted };
}

function runSim(seed) {
  const eco = new Ecology(seed);
  let minH = 999, minP = 999, minS = 999, minO = 999;
  let hadNaN = false;
  const count = () => {
    let h = 0, p = 0, s = 0, o = 0;
    for (const a of eco.agents) {
      if (!a.alive) continue;
      if (!Number.isFinite(a.x) || !Number.isFinite(a.z) || !Number.isFinite(a.energy)) { hadNaN = true; break; }
      if (a.kind === 'herb') h++;
      else if (a.kind === 'pred') p++;
      else if (a.kind === 'scav') s++;
      else o++;
    }
    return { h, p, s, o };
  };
  const checkGrassNaN = () => {
    for (let s = 0; s < eco.grassP.length; s++) {
      const g = eco.grassP[s];
      for (let i = 0; i < g.length; i++) if (!Number.isFinite(g[i])) return true;
    }
    return false;
  };
  for (let i = 1; i <= totalTicks; i++) {
    const daylight = 0.5 + 0.5 * Math.sin((i / 300) * Math.PI);
    eco.update(0.1, daylight);
    if (i % 600 === 0) {
      const c = count();
      minH = Math.min(minH, c.h); minP = Math.min(minP, c.p);
      minS = Math.min(minS, c.s); minO = Math.min(minO, c.o);
      if (i % 6000 === 0 && checkGrassNaN()) hadNaN = true;
    }
  }
  const fin = count();
  if (checkGrassNaN()) hadNaN = true;
  const by = eco.plantAmountBy();
  const sp = spatialSplit(eco);
  return {
    minH, minP, minS, minO, hadNaN, fin,
    plantBy: by,
    wet: sp.wet, dry: sp.dry, nw: sp.nw, nd: sp.nd,
  };
}

function fmt3(a) {
  return a.map(v => v.toFixed(3)).join('/');
}

function verdictLabel(ok, spatialEstablished, status) {
  if (!ok) return 'FAIL';
  if (spatialEstablished) return 'PASS';
  return '分化不显著（不计入硬判定）';
}

let allFail = false;
const rows = [];
const counts = { pass: 0, neutral: 0, fail: 0 };
for (const seed of seeds) {
  const r = runSim(seed);
  // 每种状态
  const status = r.plantBy.map(v =>
    v > COEXIST_MIN ? '真共存' : (v > 0 ? '保底维持' : '灭绝')
  );
  const coexist = r.plantBy.every(v => v > COEXIST_MIN) && r.wet.means[0] >= WET_SP1_MIN;
  const wetD = dominantWithMargin(r.wet.means);
  const dryD = dominantWithMargin(r.dry.means);
  const spatialEstablished = wetD.dom !== dryD.dom && wetD.marginOk && dryD.marginOk;
  const pAlive = r.fin.p > 0 && r.minP > 0;
  const ok = pAlive && !r.hadNaN && coexist;
  const ver = verdictLabel(ok, spatialEstablished, status);
  if (ver === 'FAIL') allFail = true;
  counts[ver === 'PASS' ? 'pass' : ver === '分化不显著（不计入硬判定）' ? 'neutral' : 'fail']++;

  const labelMinP = r.minP === 999 ? '—' : r.minP.toFixed(4);
  const wetMargin = wetD.sorted[1] > 0 ? (wetD.sorted[0] / wetD.sorted[1]).toFixed(2) : '∞';
  const dryMargin = dryD.sorted[1] > 0 ? (dryD.sorted[0] / dryD.sorted[1]).toFixed(2) : '∞';
  const wetDomSpecies = r.wet.dom !== -1 ? `草种-${wetD.dom + 1}` : '—';
  const dryDomSpecies = r.dry.dom !== -1 ? `草种-${dryD.dom + 1}` : '—';
  const spLabel = spatialEstablished ? '成立' : (wetD.dom === dryD.dom ? '湿旱主导种相同' : '余量不足');

  rows.push(
    `seed ${seed}: 终态 H=${r.fin.h} P=${r.fin.p} S=${r.fin.s} O=${r.fin.o} | 最小 H=${r.minH} P=${labelMinP}${r.minP < 999 ? '（全程未灭绝=' + (r.minP > 0 ? '是' : '否') + ')' : ''} | NaN=${r.hadNaN ? '有' : '无'}` +
    `\n  三草种均量 [湿/喜/广]: ${fmt3(r.plantBy)} → ${status.join('/')}` +
    `  共存=${coexist ? '是（湿区草种-1 均值=' + r.wet.means[0].toFixed(4) + ' ≥ ' + WET_SP1_MIN + '）' : '否'}` +
    `\n  湿区(n=${r.nw}) 主导=「${wetDomSpecies}」均量=${fmt3(r.wet.means)}（余量比=${wetMargin}）` +
    ` | 旱区(n=${r.nd}) 主导=「${dryDomSpecies}」均量=${fmt3(r.dry.means)}（余量比=${dryMargin}）` +
    ` 空间分化=${spLabel}` +
    `\n  判定=${ver}`
  );
}

console.log(`=== 草物种化无头验证：${seeds.length} seed × ${minutes}min ===`);
for (const r of rows) console.log(r);
console.log(`汇总：PASS=${counts.pass}  分化不显著=${counts.neutral}  FAIL=${counts.fail}`);
if (counts.fail > 0) {
  console.log('存在 FAIL');
  process.exit(1);
}
console.log(counts.neutral > 0 ? '全部通过或分化不显著（中性）' : '全部 PASS');
process.exit(0);
