#!/usr/bin/env node
// test_sig.mjs — 涌现式生殖隔离（sig/pref/tol 三基因）验证
// 验收线：≥3 seed × ≥600s；所有随机走 Ecology 内部 this.rng（本文件不引入随机源）。
// 用法：
//   node test_sig.mjs                          -- 主控模式：3 seed sig 组（SIG_ENABLED 默认 true）
//                                                 + 基线组（子进程 SIG_ENABLED=0），断言后输出对比表
//   node test_sig.mjs --run <seed> <minutes>   -- 子进程模式：跑单次模拟，输出 JSON（含逐样本序列，供复现性 diff）
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

// 动态 import 使 SIG_ENABLED 常量在环境清理之后求值（静态 import 会被提升，delete 来不及）
let Ecology;
const loadEco = async () => { ({ Ecology } = await import('./src/ecology.js')); };

const SEEDS = [11, 22, 33];
const DEFAULT_MINUTES = 10; // 600s
const SAMPLE_EVERY = 300;   // 每 30s 采样一次（tick = 0.1s）

function runSim(seed, minutes) {
  const eco = new Ecology(seed);
  const totalTicks = minutes * 60 * 10;
  const series = []; // 每 30s 一采样点
  let nanFound = false, negE = 0, domainBad = 0;
  let lowHStreak = 0, lowPStreak = 0, stopLoss = null; // 止损：H<60 持续 600s / P<5 持续 300s
  for (let i = 1; i <= totalTicks; i++) {
    const daylight = 0.5 + 0.5 * Math.sin((i / 300) * Math.PI);
    eco.update(0.1, daylight);
    if (i % SAMPLE_EVERY === 0) {
      let h = 0, p = 0;
      for (const a of eco.agents) {
        if (!a.alive) continue;
        if (!Number.isFinite(a.energy) || !Number.isFinite(a.x) || !Number.isFinite(a.z)) nanFound = true;
        if (a.kind === 'herb') {
          h++;
          if (a.energy < -0.01) negE++;
          const g = a.genome;
          if (g.prefSig === undefined || g.prefSig < 0.58 - 1e-9 || g.prefSig > 1.7 + 1e-9 ||
              g.tolSig === undefined || g.tolSig < 0.05 - 1e-9 || g.tolSig > 0.25 + 1e-9) domainBad++;
        } else if (a.kind === 'pred') {
          p++;
        }
      }
      const st = eco.stats();
      series.push({
        t: +st.time.toFixed(1), season: st.season, h, p,
        scav: st.scav, omn: st.omn,
        sigMeanH: st.sigMeanH, sigVarH: st.sigVarH,
        prefMeanH: st.prefMeanH, tolMeanH: st.tolMeanH,
        sigPR: st.sigPairFailRate, att: st.sigPairAttempts, fail: st.sigPairFails,
      });
      // 止损窗口（30s 采样粒度）：H<60 连续 20 样本 = 600s；P<5 连续 10 样本 = 300s
      lowHStreak = h < 60 ? lowHStreak + 1 : 0;
      lowPStreak = p < 5 ? lowPStreak + 1 : 0;
      if (!stopLoss && lowHStreak >= 20) stopLoss = `H<60 持续 600s (t=${st.time.toFixed(0)}s)`;
      if (!stopLoss && lowPStreak >= 10) stopLoss = `P<5 持续 300s (t=${st.time.toFixed(0)}s)`;
      if (stopLoss) break;
    }
  }
  const finalSt = eco.stats();
  const meanOf = (key) => series.length ? series.reduce((s, x) => s + (x[key] || 0), 0) / series.length : 0;
  const minOf = (key) => series.length ? Math.min(...series.map((x) => x[key] || 0)) : 0;
  const win = (a, b) => {
    const dAtt = (series[b].att || 0) - (series[a].att || 0);
    const dFail = (series[b].fail || 0) - (series[a].fail || 0);
    return dAtt > 0 ? dFail / dAtt : 0;
  };
  // t>200s 子序列（剔除初始随机基因组播种瞬态：t<180s d 分布极宽，与隔离涌现无关）
  const post200Idx = series.map((s, i) => (s.t >= 210 - 1e-9 ? i : -1)).filter((i) => i >= 0);
  const post200 = post200Idx.map((i) => series[i]);
  let tolTrend = null;
  let sigPRPost200 = null;
  if (post200.length >= 4) {
    const k = Math.max(1, Math.floor(post200.length / 3));
    const head = post200.slice(0, k).reduce((s, x) => s + x.tolMeanH, 0) / k;
    const tail = post200.slice(-k).reduce((s, x) => s + x.tolMeanH, 0) / k;
    tolTrend = { head: +head.toFixed(4), tail: +tail.toFixed(4), dir: tail < head - 1e-4 ? 'down' : tail > head + 1e-4 ? 'up' : 'flat' };
    // sigPR 同样取 t>200s 首/尾 1/3 窗口（涌现趋势：尾 > 首；首段应 < 0.65，即同质化基线阻断不高）
    const e0 = post200Idx[0], e1 = post200Idx[k - 1], l0 = post200Idx[post200Idx.length - k], l1 = post200Idx[post200Idx.length - 1];
    sigPRPost200 = { early: +win(e0, e1).toFixed(4), late: +win(l0, l1).toFixed(4) };
  }
  return {
    seed, minutes,
    final: { h: finalSt.h, p: finalSt.p, scav: finalSt.scav, omn: finalSt.omn },
    min: { h: minOf('h'), p: minOf('p'), scav: minOf('scav'), omn: minOf('omn') },
    meanH: +meanOf('h').toFixed(1),
    tolMeanFinal: +(series[series.length - 1]?.tolMeanH || 0).toFixed(4),
    prefMeanFinal: +(series[series.length - 1]?.prefMeanH || 0).toFixed(4),
    sigVarFinal: +(series[series.length - 1]?.sigVarH || 0).toFixed(4),
    sigPRFinal: finalSt.sigPairFailRate,
    sigPRPost200,
    sigAttTotal: finalSt.sigPairAttempts,
    sigFailTotal: finalSt.sigPairFails,
    tolTrend,
    stopLoss,
    nanFound, negE, domainBad,
    series,
  };
}

function childMode() {
  const seed = Number(process.argv[3]);
  const minutes = Number(process.argv[4]) || DEFAULT_MINUTES;
  process.stdout.write(JSON.stringify(runSim(seed, minutes)));
}

async function mainMode() {
  // 先清外部 shell 污染再加载模块，保证主进程 SIG_ENABLED 按净环境（默认开）求值
  delete process.env.SIG_ENABLED;
  await loadEco();
  const results = {};
  for (const seed of SEEDS) {
    results[seed] = {};
    // sig 组：本进程 SIG_ENABLED 默认 true
    results[seed].sig = runSim(seed, DEFAULT_MINUTES);
    // 基线对照：SIG_ENABLED=0（模块常量在子进程加载时由环境变量解析）
    const res = spawnSync(process.execPath, [fileURLToPath(import.meta.url), '--run', String(seed), String(DEFAULT_MINUTES)], {
      encoding: 'utf8',
      env: { ...process.env, SIG_ENABLED: '0' },
    });
    if (res.error) throw res.error;
    if (res.status !== 0) throw new Error(`baseline child exit=${res.status}\nstderr: ${String(res.stderr).slice(0, 400)}`);
    results[seed].baseline = JSON.parse(res.stdout);
  }

  console.log('seed  组       H终  Hmin  Hmean  P终/Scav终/Omn终  sigPR(终/t>200首/t>200尾)  组合通过率(sig层)  tolMeanH终  prefMeanH终  sigVarH终  tol趋势(t>200)');
  for (const seed of SEEDS) {
    for (const g of ['sig', 'baseline']) {
      const r = results[seed][g];
      const trend = r.tolTrend ? `${r.tolTrend.dir} (${r.tolTrend.head}→${r.tolTrend.tail})` : '-';
      const sigCombo = g === 'sig' ? (1 - r.sigPRFinal).toFixed(4) : '1.0000';
      console.log(
        `${seed}  ${g.padEnd(8)} ${String(r.final.h).padStart(4)} ${String(r.min.h).padStart(5)} ${String(r.meanH).padStart(6)}` +
        `  ${r.final.p}/${r.final.scav}/${r.final.omn}` +
        `  ${g === 'sig' ? `${r.sigPRFinal}/${r.sigPRPost200 ? r.sigPRPost200.early + '/' + r.sigPRPost200.late : '-'}` : '0/-/-'}` +
        `  ${sigCombo.padStart(17)}  ${String(r.tolMeanFinal).padStart(10)}  ${String(r.prefMeanFinal).padStart(11)}  ${String(r.sigVarFinal).padStart(9)}  ${trend}`
      );
    }
  }

  const fails = [];
  const warns = [];
  const assert = (cond, msg) => { if (!cond) fails.push(msg); };
  const warn = (cond, msg) => { if (!cond) warns.push(msg); };
  for (const seed of SEEDS) {
    const c = results[seed].sig, b = results[seed].baseline;
    // ⑥ env 生效断言——基线组 SIG_ENABLED=0 必须 sig 层零尝试零阻断（直接证明开关生效）
    if (b.sigPRFinal !== 0 || b.sigAttTotal !== 0 || b.sigFailTotal !== 0) {
      fails.push(`seed ${seed} 基线组 env 未生效: sigPR=${b.sigPRFinal} attempts=${b.sigAttTotal} fails=${b.sigFailTotal}`);
    }
    // 止损（任一触发即停）
    if (c.stopLoss) fails.push(`seed ${seed} sig 组触发止损: ${c.stopLoss}`);
    if (b.stopLoss) warns.push(`seed ${seed} 基线组触发止损: ${b.stopLoss}（对照减弱）`);
    // ① 四物种全程存活
    for (const sp of ['h', 'p', 'scav', 'omn']) {
      if (c.min[sp] <= 0) fails.push(`seed ${seed} sig 组 ${sp} 中途灭绝 (min=${c.min[sp]})`);
      if (b.min[sp] <= 0) warns.push(`seed ${seed} 基线组 ${sp} 中途灭绝 (min=${b.min[sp]})，对照减弱`);
    }
    // ② H 数量在基线 85-115%（全程均值口径，sig 层真实生效允许稍偏离）
    if (b.meanH <= 0) { fails.push(`seed ${seed} 基线 H 全程均值为 0，无法对照`); continue; }
    const ratio = c.meanH / b.meanH;
    if (ratio < 0.85 || ratio > 1.15) fails.push(`seed ${seed} H 均值偏离基线 ${((ratio - 1) * 100).toFixed(1)}%（超出 85-115%）`);
    // ③ tolMeanH t>200s 后趋势：下降是涌现标志（tol 收紧 → 隔离群分化，越界配对失败率上升）
    if (c.tolTrend) {
      if (c.tolTrend.dir !== 'down') fails.push(`seed ${seed} tolMeanH t>200s 趋势=${c.tolTrend.dir}（${c.tolTrend.head}→${c.tolTrend.tail}），应为下降`);
    } else fails.push(`seed ${seed} 采样不足，tol 趋势未计算`);
    // ④ sigPairFailRate：t>200s 首/尾 1/3 窗口——首段<0.65（任务预期「初始 sigMatch 得分 0.4-0.6」+种子裕量；
    //   原饱和区 0.96-0.997，远离即达标）且尾段>首段（隔离涌现趋势）
    if (!(c.sigPRFinal > 0)) fails.push(`seed ${seed} sigPairFailRate 终值=${c.sigPRFinal}，应为正`);
    if (!c.sigPRPost200) fails.push(`seed ${seed} 采样不足，sigPR 趋势未计算`);
    else {
      if (!(c.sigPRPost200.early < 0.65)) fails.push(`seed ${seed} sigPR t>200 首1/3=${c.sigPRPost200.early}，应<0.65（窗口过紧则初始即饱和）`);
      if (!(c.sigPRPost200.early > 0.3)) fails.push(`seed ${seed} sigPR t>200 首1/3=${c.sigPRPost200.early}，应>0.3（sig 层确实在阻断，排除 sig 失效+噪声假阳性）`);
      if (!(c.sigPRPost200.late > c.sigPRPost200.early)) fails.push(`seed ${seed} sigPR 尾1/3(${c.sigPRPost200.late}) ≤ 首1/3(${c.sigPRPost200.early})，隔离未涌现`);
    }
    // ⑤ 数值健全
    if (c.nanFound) fails.push(`seed ${seed} 检出 NaN`);
    if (c.negE > 0) fails.push(`seed ${seed} H 出现负能量 ${c.negE} 次`);
    if (c.domainBad > 0) fails.push(`seed ${seed} prefSig/tolSig 越域 ${c.domainBad} 次`);
    if (b.nanFound) fails.push(`seed ${seed} 基线组检出 NaN`);
    // ⑦ P 最低值安全线：120min 数据 P min=3 逼近危险线，需持续监控
    if (c.min.p < 8) fails.push(`seed ${seed} sig 组 P 最低值=${c.min.p}，应≥8（逼近危险线）`);
    if (b.min.p < 8) warns.push(`seed ${seed} 基线组 P 最低值=${b.min.p}，应≥8（逼近危险线，对照减弱）`);
  }

  if (warns.length) warns.forEach((w) => console.log('WARN: ' + w));
  if (fails.length) {
    console.error('\nFAIL:');
    fails.forEach((f) => console.error('  - ' + f));
    console.error('\n诊断数据：\n' + JSON.stringify(results, null, 2));
    process.exit(1);
  }
  console.log('\nPASS: 全部断言通过（' + SEEDS.length + ' seed × ' + DEFAULT_MINUTES + 'min，sig 组 vs SIG_ENABLED=0 基线）');
}

if (process.argv[2] === '--run') {
  await loadEco(); // 子进程：保留父进程传入的 env（SIG_ENABLED=0），在 import 前不清理
  childMode();
} else {
  await mainMode();
}
