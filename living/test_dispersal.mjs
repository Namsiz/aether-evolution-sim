// 无头验证：内动传播（第三波）——果实能量→肠道携带→deposit 落种全链路
// 检查项：
//   ① cargoTotal > 0（有果实被采食、种子入肠道携带）
//   ② deposit 事件发生且 >1 格占比 > 50%（搬运距离中位数 > 1 格，stats().deposit.p50 与内部分位采样双重核对）
//   ③ 水域不变量：suit≤0.02 的格 grassP 全为 0（deposit 与蔓延都不向不可适生格落种）
//   ④ 果实能量占比 < 15%（stats().fruitEnergyShareH，果实只是能量小项，不喧宾夺主）
// 用法：node test_dispersal.mjs [分钟数=30] [seed列表逗号分隔=11,22]
import { Ecology } from './src/ecology.js';

const minutes = Number(process.argv[2] || 30);
const seeds = (process.argv[3] || '11,22').split(',').map(Number);
const totalTicks = minutes * 600;

function runSim(seed) {
  const eco = new Ecology(seed);
  for (let i = 1; i <= totalTicks; i++) {
    const daylight = 0.5 + 0.5 * Math.sin((i / 300) * Math.PI);
    eco.update(0.1, daylight);
  }
  const st = eco.stats();

  // ② >1 格占比：从内部分位采样精确计算（每 4096 次 deposit 采一距今距离，环形覆盖）
  const samples = eco.depositStats.samples;
  let over1 = 0;
  for (const d of samples) if (d > 1) over1++;
  const over1Frac = samples.length ? over1 / samples.length : 0;

  // ③ 水域不变量：suit≤0.02 格的分种草量必须全为 0
  let waterViol = 0;
  for (let i = 0; i < eco.grass.length; i++) {
    if (eco.suit[i] > 0.02) continue;
    for (let s = 0; s < eco.grassP.length; s++) {
      if (eco.grassP[s][i] > 0) { waterViol++; break; }
    }
  }

  return { st, over1Frac, waterViol };
}

const rows = [];
let failed = false;
for (const seed of seeds) {
  const { st, over1Frac, waterViol } = runSim(seed);
  const dep = st.deposit;
  const checks = [
    ['cargoTotal > 0', st.cargoTotal > 0, `cargoTotal=${st.cargoTotal}`],
    ['deposit 发生且 >1 格占比 > 50%',
      !!dep && dep.count > 0 && dep.p50 > 1 && over1Frac > 0.5,
      dep ? `count=${dep.count} p50=${dep.p50} p90=${dep.p90} mean=${dep.meanDistCells} >1格占比=${(over1Frac * 100).toFixed(1)}%` : 'deposit=none'],
    ['水域不变量（suit≤0.02 格 grassP=0）', waterViol === 0, `viols=${waterViol}`],
    ['果实能量占比 < 15%', st.fruitEnergyShareH < 0.15, `fruitEnergyShareH=${st.fruitEnergyShareH}`],
  ];
  const bad = checks.filter(([, ok]) => !ok);
  if (bad.length) failed = true;
  rows.push(
    `seed ${seed}: H=${st.h} P=${st.p} t=${st.time.toFixed(0)}s\n` +
    checks.map(([name, ok, detail]) => `  [${ok ? 'PASS' : 'FAIL'}] ${name} — ${detail}`).join('\n')
  );
}

console.log(`=== 内动传播无头验证：${seeds.length} seed × ${minutes}min ===`);
for (const r of rows) console.log(r);
console.log(failed ? '存在 FAIL' : '全部 PASS');
process.exit(failed ? 1 : 0);
