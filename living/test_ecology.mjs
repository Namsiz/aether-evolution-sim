import { Ecology } from './src/ecology.js';

const eco = new Ecology(20261102);
const minutes = Number(process.argv[2] || 10);
const totalTicks = minutes * 60 * 10;
for (let i = 1; i <= totalTicks; i++) {
  const daylight = 0.5 + 0.5 * Math.sin((i / 300) * Math.PI);
  eco.update(0.1, daylight);
  if (i % 300 === 0) {
    const s = eco.stats();
    console.log(
      `${(i / 600).toFixed(1)}min  H ${String(s.h).padStart(3)}  P ${String(s.p).padStart(2)}` +
      `  草 ${s.plant.toFixed(2)}  速度H ${s.speedH.toFixed(2)} 感知H ${s.senseH.toFixed(1)}` +
      `  速度P ${s.speedP.toFixed(2)}  出生 ${s.births} 死亡 ${s.deaths} 捕杀 ${s.kills} 饿死 ${s.starved}`
    );
  }
}
