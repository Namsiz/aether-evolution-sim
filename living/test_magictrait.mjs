// 验证第五波补充：magic trait 生态耦合（体型-草种匹配）
// 关键观测：湿区 H size 均值应 > 旱区 H size 均值（生态耦合信号）
import { Ecology, HERB, PRED } from './src/ecology.js';

const minutes = Number(process.argv[2] || 60);
const seeds = (process.argv[3] || '11,22,33').split(',').map(Number);
const totalTicks = minutes * 600;
const moistThresh = 0.25;

function countKind(eco, kind) {
  let n = 0;
  for (const a of eco.agents) if (a.alive && a.kind === kind) n++;
  return n;
}

function splitSizeByMoist(eco) {
  let wetSizes = [], drySizes = [];
  const half = eco.GRID_SPAN / 2;
  for (const a of eco.agents) {
    if (!a.alive || a.kind !== HERB) continue;
    const i = Math.floor((a.z + half) / eco.CELL);
    const j = Math.floor((a.x + half) / eco.CELL);
    if (i < 0 || j < 0 || i >= eco.GRID_N || j >= eco.GRID_N) continue;
    const idx = j * eco.GRID_N + i;
    const m = eco.moist[idx] ?? 0;
    const arr = m > moistThresh ? wetSizes : drySizes;
    arr.push(a.genome.size);
  }
  return { wetSizes, drySizes };
}

function mean(arr) { return arr.length ? arr.reduce((a, b) => a + b, 0) / arr.length : NaN; }
function variance(arr) {
  if (arr.length < 2) return NaN;
  const m = mean(arr);
  return arr.reduce((s, v) => s + (v - m) ** 2, 0) / arr.length;
}

let allOk = true;
const rows = [];

for (const seed of seeds) {
  const eco = new Ecology(seed);
  let minH = 999, minP = 999, hadNaN = false;
  let preSplit = null; // 初始 size 分布

  for (let i = 1; i <= totalTicks; i++) {
    const daylight = 0.5 + 0.5 * Math.sin((i / 300) * Math.PI);
    eco.update(0.1, daylight);
    if (i % 600 === 0) {
      const h = countKind(eco, HERB);
      const p = countKind(eco, PRED);
      minH = Math.min(minH, h); minP = Math.min(minP, p);
      if (i === 600) preSplit = splitSizeByMoist(eco);
      // 采集 t=end 时的 size 分布
    }
    // NaN 巡检
    for (const a of eco.agents) {
      if (!a.alive) continue;
      if (a.kind === HERB && (!Number.isFinite(a.genome.size) || a.genome.size < 0.3 || a.genome.size > 2.5)) { hadNaN = true; break; }
    }
    if (hadNaN) break;
  }

  const postSplit = splitSizeByMoist(eco);
  const wetMean = mean(preSplit.wetSizes), dryMean = mean(preSplit.drySizes);
  const wetMeanEnd = mean(postSplit.wetSizes), dryMeanEnd = mean(postSplit.drySizes);
  const wetVar = variance(postSplit.wetSizes), dryVar = variance(postSplit.drySizes);
  const h = countKind(eco, HERB);
  const p = countKind(eco, PRED);
  const sizeDiff = wetMeanEnd - dryMeanEnd; // 正值=湿区偏大，符合耦合预期
  const sizeDiffSign = sizeDiff > 0.01 ? '✓湿区大' : sizeDiff < -0.01 ? '✗旱区大' : '≈持平';

  const info = `seed ${seed}: H=${h} P=${p} | wetMean=${wetMeanEnd.toFixed(3)} dryMean=${dryMeanEnd.toFixed(3)} diff=${sizeDiff.toFixed(4)} ${sizeDiffSign} | varW=${wetVar.toFixed(4)} varD=${dryVar.toFixed(4)} | NaN=${hadNaN ? '有' : '无'}`;
  rows.push(info);
  if (hadNaN) allOk = false;
  if (h === 0 || p === 0) allOk = false;
}

console.log(`=== magic trait 生态耦合验证：${seeds.length} seed × ${minutes}min ===`);
for (const r of rows) console.log(r);
console.log(allOk ? '\nPASS' : '\nFAIL');
process.exit(allOk ? 0 : 1);
