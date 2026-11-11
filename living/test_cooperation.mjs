#!/usr/bin/env node
// test_cooperation.mjs — 间接互惠（image scoring，「让食」）验证
// 验收线：≥3 seed × ≥600s；所有随机走 Ecology 内部 this.rng（本文件不引入随机源）。
// 用法：
//   node test_cooperation.mjs                          -- 主控模式：3 seed 合作组（HELP_ENABLED 默认 true）
//                                                         + 基线组（子进程 HELP_ENABLED=0），断言后输出对比表
//   node test_cooperation.mjs --run <seed> <minutes>   -- 子进程模式：跑单次模拟，输出 JSON
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

// N1：动态 import 使 HELP_ENABLED 常量在环境清理之后求值（静态 import 会被提升，delete 来不及）
let Ecology;
const loadEco = async () => { ({ Ecology } = await import('./src/ecology.js')); };

const SEEDS = [11, 22, 33];
const DEFAULT_MINUTES = 10; // 600s

function runSim(seed, minutes) {
  const eco = new Ecology(seed);
  const totalTicks = minutes * 60 * 10;
  const series = []; // 每 30s 一采样点
  let nanFound = false, imgBad = 0, cdBad = 0, negE = 0;
  for (let i = 1; i <= totalTicks; i++) {
    const daylight = 0.5 + 0.5 * Math.sin((i / 300) * Math.PI);
    eco.update(0.1, daylight);
    if (i % 300 === 0) { // 每 30s
      let minE = Infinity;
      for (const a of eco.agents) {
        if (!a.alive) continue;
        if (!Number.isFinite(a.energy) || !Number.isFinite(a.x) || !Number.isFinite(a.image) || !Number.isFinite(a.helpCd)) nanFound = true;
        if (a.image < -5 - 1e-9 || a.image > 5 + 1e-9) imgBad++;
        if (a.helpCd < -0.01 || a.helpCd > 12.01) cdBad++;
        if (a.kind === 'herb') {
          if (a.energy < minE) minE = a.energy;
          if (a.energy < -0.01) negE++;
        }
      }
      const st = eco.stats();
      series.push({
        t: st.time, season: st.season,
        h: st.h, p: st.p, scav: st.scav, omn: st.omn,
        coopShareH: st.coopShareH, helpThrH: st.helpThrH,
        minHerbEnergy: Number.isFinite(minE) ? +minE.toFixed(2) : null,
      });
    }
  }
  const finalSt = eco.stats();
  const stat = (key, f) => f(series.map((s) => s[key] || 0));
  const springH = series.filter((s) => s.season === 0).map((s) => s.h);
  const winterH = series.filter((s) => s.season === 3).map((s) => s.h);
  // coopShareH 统计段：t > 200s
  const post200 = series.filter((s) => s.t >= 210 - 1e-9);
  const coopMeanPost200 = post200.length ? post200.reduce((s, x) => s + (x.coopShareH || 0), 0) / post200.length : 0;
  return {
    seed, minutes,
    final: { h: finalSt.h, p: finalSt.p, scav: finalSt.scav, omn: finalSt.omn },
    minH: stat('h', (v) => Math.min(...v)),
    maxH: stat('h', (v) => Math.max(...v)),
    meanH: +stat('h', (v) => v.reduce((a, b) => a + b, 0) / v.length).toFixed(1),
    min: {
      h: stat('h', (v) => Math.min(...v)), p: stat('p', (v) => Math.min(...v)),
      scav: stat('scav', (v) => Math.min(...v)), omn: stat('omn', (v) => Math.min(...v)),
    },
    finalCounts: { p: finalSt.p, scav: finalSt.scav, omn: finalSt.omn },
    coopShareHMeanPost200: +coopMeanPost200.toFixed(4),
    coopShareHLast: +(series[series.length - 1]?.coopShareH || 0).toFixed(4),
    helpThrFinal: +(finalSt.evHistH?.[finalSt.evHistH.length - 1]?.helpThr ?? 0).toFixed(4),
    helpThrMaxSeries: +stat('helpThrH', (v) => Math.max(...v)).toFixed(4),
    helpCount: finalSt.helpCount,
    rejectCount: finalSt.rejectCount,
    helpCostShareH: +finalSt.helpCostShareH.toFixed(4),
    // 第五波验收：冬季时段（t>=5400s）coopShareH 均值
    winterCoopShareHMeanPost5400: (() => {
      const w = series.filter((s) => s.t >= 5400 - 1e-9 && s.season === 3);
      return w.length ? +(w.reduce((s, x) => s + (x.coopShareH || 0), 0) / w.length).toFixed(4) : null;
    })(),
    winterHMin: (() => { const ws = series.filter((s) => s.season === 3); return ws.length ? Math.min(...ws.map((x) => x.h)) : 0; })(),
    winterPMin: (() => { const ws = series.filter((s) => s.season === 3); return ws.length ? Math.min(...ws.map((x) => x.p)) : 0; })(),
    springHMean: springH.length ? +(springH.reduce((a, b) => a + b, 0) / springH.length).toFixed(1) : null,
    winterHMean: winterH.length ? +(winterH.reduce((a, b) => a + b, 0) / winterH.length).toFixed(1) : null,
    noNegEnergy: negE === 0,
    nanFound,
    imgBad,
    cdBad,
  };
}

function childMode() {
  const seed = Number(process.argv[3]);
  const minutes = Number(process.argv[4]) || DEFAULT_MINUTES;
  process.stdout.write(JSON.stringify(runSim(seed, minutes)));
}

async function mainMode() {
  // N1：先清外部 shell 污染再加载模块，保证主进程 HELP_ENABLED 按净环境（默认开）求值
  delete process.env.HELP_ENABLED;
  await loadEco();
  const results = {};
  for (const seed of SEEDS) {
    results[seed] = {};
    // 合作组：本进程 HELP_ENABLED 默认 true
    results[seed].coop = runSim(seed, DEFAULT_MINUTES);
    // 基线对照：HELP_ENABLED=0（模块常量在子进程加载时由环境变量解析）
    const res = spawnSync(process.execPath, [fileURLToPath(import.meta.url), '--run', String(seed), String(DEFAULT_MINUTES)], {
      encoding: 'utf8',
      env: { ...process.env, HELP_ENABLED: '0' },
    });
    if (res.error) throw res.error;
    if (res.status !== 0) throw new Error(`baseline child exit=${res.status}\nstderr: ${String(res.stderr).slice(0, 400)}`);
    results[seed].baseline = JSON.parse(res.stdout);
  }

  console.log('seed  组       H终  Hmin  Hmean  P终/Scav终/Omn终  coopShareH(t>200均值)  helpThr终  helpCount/reject  helpCostShareH  冬CoopShare  冬Hmin  冬Pmin');
  for (const seed of SEEDS) {
    for (const g of ['coop', 'baseline']) {
      const r = results[seed][g];
      console.log(
        `${seed}  ${g.padEnd(8)} ${String(r.final.h).padStart(4)} ${String(r.minH).padStart(5)} ${String(r.meanH).padStart(6)}` +
        `  ${r.final.p}/${r.final.scav}/${r.final.omn}` +
        `  ${String(g === 'coop' ? r.coopShareHMeanPost200 : '-').padStart(10)}` +
        `  ${String(g === 'coop' ? r.helpThrFinal : '-').padStart(6)}` +
        `  ${g === 'coop' ? `${r.helpCount}/${r.rejectCount}` : '-'}` +
        `  ${g === 'coop' ? r.helpCostShareH : '-'}` +
        `  ${g === 'coop' ? String(r.winterCoopShareHMeanPost5400 ?? '-').padStart(6) : '-'.padStart(6)}` +
        `  ${g === 'coop' ? String(r.winterHMin ?? '-').padStart(4) : '-'.padStart(4)}` +
        `  ${g === 'coop' ? String(r.winterPMin ?? '-').padStart(4) : '-'.padStart(4)}`
      );
    }
  }

  const fails = [];
  const warns = [];
  const assert = (cond, msg) => { if (!cond) fails.push(msg); };
  for (const seed of SEEDS) {
    const c = results[seed].coop, b = results[seed].baseline;
    // M3：env 生效断言——基线组 HELP_ENABLED=0 必须零帮助/零拒绝/零成本（直接证明开关生效，而非靠 H 数量差异推断）
    if (b.helpCount !== 0 || b.rejectCount !== 0 || b.helpCostShareH !== 0) {
      fails.push(`seed ${seed} 基线组 env 未生效: helpCount=${b.helpCount} rejectCount=${b.rejectCount} helpCostShareH=${b.helpCostShareH}`);
    }
    // 1. H 全程存活 + 四物种全程存活（基线自身灭绝记警号不判合作失败）
    for (const sp of ['h', 'p', 'scav', 'omn']) {
      if (c.min[sp] <= 0) fails.push(`seed ${seed} 合作组 ${sp} 中途灭绝 (min=${c.min[sp]})`);
      if (b.min[sp] <= 0) warns.push(`seed ${seed} 基线组 ${sp} 中途灭绝 (min=${b.min[sp]})，对照减弱`);
    }
    // H 数量偏离基线 ±25%（按全程均值口径；逐样本最差偏离另报）
    if (b.meanH <= 0) { fails.push(`seed ${seed} 基线 H 全程均值为 0，无法对照`); continue; }
    const ratio = c.meanH / b.meanH;
    if (ratio < 0.75 || ratio > 1.25) fails.push(`seed ${seed} H 均值偏离基线 ${(ratio * 100).toFixed(1)}% (>±25%) → 止损条件 (b)`);
    // 2. 合作出现：t>200s 后 coopShareH 均值 > 0.05
    if (c.coopShareHMeanPost200 <= 0.05) {
      if (c.coopShareHMeanPost200 < 0.005 && c.helpThrFinal > 3) {
        fails.push(`seed ${seed} coopShareH=${c.coopShareHMeanPost200}≈0 且 mean helpThr=${c.helpThrFinal}>3 → 疑似空间相遇不足（春 H均值=${c.springHMean} 冬 H均值=${c.winterHMean}）`);
      } else {
        fails.push(`seed ${seed} coopShareH 均值(t>200)=${c.coopShareHMeanPost200} ≤ 0.05（合作未出现）`);
      }
    }
    // 3. helpThr 无单调冲顶（>4 警号）
    if (c.helpThrFinal > 4) warns.push(`seed ${seed} helpThr 终值=${c.helpThrFinal} > 4（警号）`);
    if (c.helpThrMaxSeries > 4.5) warns.push(`seed ${seed} helpThr 过程峰值=${c.helpThrMaxSeries} > 4.5（警号）`);
    // 4. 能量占比 < 5%
    if (c.helpCostShareH >= 0.05) fails.push(`seed ${seed} helpCostShareH=${c.helpCostShareH} ≥ 5% → 止损条件（能量预算失衡）`);
    // 5. 第五波验收：冬季时段（t>=5400s）coopShareH 均值 > 0.02（600s 春季回归场景无冬样本，自动跳过）
    if (c.winterCoopShareHMeanPost5400 !== null && c.winterCoopShareHMeanPost5400 <= 0.02) {
      fails.push(`seed ${seed} 冬季 coopShareH(t>=5400)=${c.winterCoopShareHMeanPost5400} ≤ 0.02（冬季合作未提升）`);
    }
    // 6. 数值健全
    if (c.nanFound) fails.push(`seed ${seed} 检出 NaN`);
    if (!c.noNegEnergy) fails.push(`seed ${seed} H 出现负能量`);
    if (c.imgBad > 0) fails.push(`seed ${seed} image 越界 ${c.imgBad} 次`);
    if (c.cdBad > 0) fails.push(`seed ${seed} helpCd 越界 ${c.cdBad} 次`);
  }

  if (warns.length) warns.forEach((w) => console.log('WARN: ' + w));
  if (fails.length) {
    console.error('\nFAIL:');
    fails.forEach((f) => console.error('  - ' + f));
    console.error('\n诊断数据：\n' + JSON.stringify(results, null, 2));
    process.exit(1);
  }
  console.log('\nPASS: 全部断言通过（' + SEEDS.length + ' seed × ' + DEFAULT_MINUTES + 'min，合作组 vs HELP_ENABLED=0 基线）');
  console.log('\n季节密度段（H 均值）：');
  for (const seed of SEEDS) {
    const c = results[seed].coop;
    console.log(`  seed ${seed}: 春 ${c.springHMean} / 冬 ${c.winterHMean}`);
  }
}

if (process.argv[2] === '--run') {
  await loadEco(); // 子进程：保留父进程传入的 env（HELP_ENABLED=0），在 import 前不清理
  childMode();
} else {
  await mainMode();
}
