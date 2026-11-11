#!/usr/bin/env node
// test_winterAgg.mjs — 第六波：冬季吸引聚合（winter aggregation pull）验证
// 对比「聚合关闭（WIN_AGG_THRESHOLD=0）」vs「聚合开启（默认 250）」，其余参数（含 WINTER_HELP_SCALE=0.6）两组一致。
// 时长：9000 sim-秒（完整四季 7200 + 次年春季 1800，用于测「冬季 H 最低点 → 春季 H 回升 400」恢复速度）。
// 用法：
//   node test_winterAgg.mjs                          -- 主控：跑 ON/OFF 两组，输出对比表
//   node test_winterAgg.mjs --run <seed> <minutes>   -- 子进程：跑单次模拟，输出 JSON（env 控制开关）
import { fileURLToPath } from 'node:url';
import { spawnSync } from 'node:child_process';

let Ecology;
const loadEco = async () => { ({ Ecology } = await import('./src/ecology.js')); };

const SEED = 11;
const MINUTES = 150; // 9000 sim-秒

function runSim(seed, minutes) {
  const eco = new Ecology(seed);
  const totalTicks = minutes * 60 * 10;
  const series = [];
  for (let i = 1; i <= totalTicks; i++) {
    const daylight = 0.5 + 0.5 * Math.sin((i / 300) * Math.PI);
    eco.update(0.1, daylight);
    if (i % 300 === 0) { // 每 30 sim-秒
      const st = eco.stats();
      series.push({ t: st.time, season: st.season, h: st.h, p: st.p, scav: st.scav, omn: st.omn, coopShareH: st.coopShareH });
    }
  }
  const finalSt = eco.stats();
  const seg = (lo, hi) => series.filter((s) => s.t >= lo - 1e-9 && s.t < hi - 1e-9);
  const mean = (arr, k) => (arr.length ? +(arr.reduce((s, x) => s + (x[k] || 0), 0) / arr.length).toFixed(2) : null);
  const minOf = (arr, k) => (arr.length ? Math.min(...arr.map((x) => x[k] || 0)) : null);

  // 分季统计：春[0,1800) 夏[1800,3600) 秋[3600,5400) 冬[5400,7200) 次春[7200,9000)
  const seasons = {
    spring1: seg(0, 1800), summer: seg(1800, 3600), autumn: seg(3600, 5400),
    winter: seg(5400, 7200), spring2: seg(7200, 9000),
  };
  const winter = seasons.winter;
  // 冬季 H 最低点 → 春季 H 首次回升到 400 的时间（ sim-秒；未回升则 null）
  let hMinT = null, hMinV = Infinity;
  for (const s of series) {
    if (s.t >= 5400 - 1e-9 && s.h < hMinV) { hMinV = s.h; hMinT = s.t; }
  }
  let recoverT = null;
  for (const s of series) {
    if (hMinT !== null && s.t > hMinT && s.h >= 400) { recoverT = +(s.t - hMinT).toFixed(0); break; }
  }
  // 全程（剔除两组必然相同的春夏秋，仅冬+次春）H 均值偏离
  const winterSpring = [...winter, ...seasons.spring2];
  return {
    seed, minutes,
    final: { h: finalSt.h, p: finalSt.p, scav: finalSt.scav, omn: finalSt.omn },
    perSeason: Object.fromEntries(Object.entries(seasons).map(([k, arr]) => [k, {
      hMean: mean(arr, 'h'), pMean: mean(arr, 'p'),
      hMin: minOf(arr, 'h'), pMin: minOf(arr, 'p'),
      coopMean: mean(arr, 'coopShareH'),
    }])),
    winterHMinSeries: hMinV, winterHMinT: hMinT,
    springRecoverTo400: recoverT,
    winterSpringHMean: mean(winterSpring, 'h'),
    winterSpringPMean: mean(winterSpring, 'p'),
    helpCostShareH: +finalSt.helpCostShareH.toFixed(4),
    helpCount: finalSt.helpCount,
    pExtinct: series.some((s) => (s.p || 0) <= 0),
    hExtinct: series.some((s) => (s.h || 0) <= 0),
  };
}

function childMode() {
  const seed = Number(process.argv[3]);
  const minutes = Number(process.argv[4]) || MINUTES;
  process.stdout.write(JSON.stringify(runSim(seed, minutes)));
}

function mainMode() {
  delete process.env.WIN_AGG_THRESHOLD; // ON 组：默认 250
  const on = runSim(SEED, MINUTES);
  const res = spawnSync(process.execPath, [fileURLToPath(import.meta.url), '--run', String(SEED), String(MINUTES)], {
    encoding: 'utf8',
    env: { ...process.env, WIN_AGG_THRESHOLD: '0' }, // OFF 组：关闭聚合
  });
  if (res.error) throw res.error;
  if (res.status !== 0) throw new Error(`OFF child exit=${res.status}\nstderr: ${String(res.stderr).slice(0, 400)}`);
  const off = JSON.parse(res.stdout);

  const row = (label, r) => {
    const ps = r.perSeason;
    return `${label.padEnd(6)} | 春1 H${ps.spring1.hMean}/P${ps.spring1.pMean}/coop${ps.spring1.coopMean}` +
      ` | 夏 H${ps.summer.hMean}/P${ps.summer.pMean}` +
      ` | 秋 H${ps.autumn.hMean}/P${ps.autumn.pMean}` +
      ` | 冬 H${ps.winter.hMean}/P${ps.winter.pMean}/coop${ps.winter.coopMean}/Hmin${ps.winter.hMin}/Pmin${ps.winter.pMin}` +
      ` | 次春 H${ps.spring2.hMean}/P${ps.spring2.pMean}/coop${ps.spring2.coopMean}`;
  };
  console.log('组   | 春1(H/P/coop) | 夏(H/P) | 秋(H/P) | 冬(H/P/coop/Hmin/Pmin) | 次春(H/P/coop)');
  console.log(row('ON', on));
  console.log(row('OFF', off));
  console.log('');
  console.log(`冬Hmin: ON ${on.winterHMinSeries} @t=${on.winterHMinT} | OFF ${off.winterHMinSeries} @t=${off.winterHMinT}`);
  console.log(`冬→次春 H≥400 恢复耗时: ON ${on.springRecoverTo400 ?? '未回升'} | OFF ${off.springRecoverTo400 ?? '未回升'} (sim-秒)`);
  console.log(`冬+次春 H 均值: ON ${on.winterSpringHMean} | OFF ${off.winterSpringHMean} → 偏离 ${off.winterSpringHMean ? (((on.winterSpringHMean - off.winterSpringHMean) / off.winterSpringHMean) * 100).toFixed(1) : '?'}%`);
  console.log(`helpCostShareH: ON ${on.helpCostShareH} | OFF ${off.helpCostShareH}  (helpCount ON ${on.helpCount} / OFF ${off.helpCount})`);
  console.log(`灭绝: ON P${on.pExtinct ? '是' : '否'}/H${on.hExtinct ? '是' : '否'} | OFF P${off.pExtinct ? '是' : '否'}/H${off.hExtinct ? '是' : '否'}`);
}

if (process.argv[2] === '--run') {
  await loadEco();
  childMode();
} else {
  await loadEco();
  mainMode();
}
