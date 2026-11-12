// test_cataclysm.mjs — Phase 17 cataclysm smoke test (Node, C-ABI only).
//
// Exercises the Phase 17 wasm pipeline the browser viewer uses:
//   1. ABI guards: sim_trigger_cataclysm / sim_cataclysm_count /
//      sim_get_cataclysm / sim_env_mods.
//   2. Impact injection: in-radius grass+mineral cells zeroed in the save
//      JSON, outside cells bit-untouched; in-radius agents either died
//      (exactly the energy <= severity×60 accounting, one carcass each) or
//      lost exactly severity×60 energy.
//   3. Dust recovery curve: sim_env_mods returns the exact linear schedule
//      1 − severity × (remaining/duration) tick by tick, then snaps to 1.0.
//   4. SaveState v4 roundtrip: active cataclysms (live countdown included)
//      survive the JSON hop; restored sim continues in bit-exact lockstep.
//   5. Legacy v3 compat: strip Cataclysms/NextCataclysmId, Version → 3 →
//      restore ok, empty table, and bit-exact match with a never-injected
//      control run.
//   6. Pure renderer helpers (cataclysmMarkPositions / formatCataclysmHud).
//
// Usage: node living/test_cataclysm.mjs [--wasm path]

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { cataclysmMarkPositions, formatCataclysmHud, CATACLYSM_IMPACT, CATACLYSM_CLIMATE } from './src/renderer-cataclysm.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const wasmArg = args.indexOf('--wasm');
const wasmPath = wasmArg >= 0 ? args[wasmArg + 1] : path.join(__dirname, 'ecology-sim.wasm');

let failures = 0;
const check = (name, cond, detail = '') => {
  if (cond) console.log(`  ok   ${name}${detail ? ` (${detail})` : ''}`);
  else { console.error(`  FAIL ${name}${detail ? ` — ${detail}` : ''}`); failures += 1; }
};

const daylight = (t) => 0.5 + 0.5 * Math.sin((t / 300) * Math.PI);
const SEED = 20261102;
const GRID_N = 76, HALF = 540, CELL = 1080 / GRID_N;
// Scratch layout mirror (must match wasm.rs / wasm-viewer.mjs).
const OFF_DAYLIGHT = 1444 * 4 + 1024 * 48;

function readSave(E, sim) {
  const jsonPtr = E.sim_save_json(sim, 0);
  const u8 = new Uint8Array(E.memory.buffer, jsonPtr);
  let len = 0;
  while (u8[len] !== 0) len++;
  const bytes = u8.slice(0, len);
  E.sim_free_json(jsonPtr, len + 1);
  return JSON.parse(new TextDecoder().decode(bytes));
}

function restoreSave(E, saveState) {
  const enc = new TextEncoder().encode(JSON.stringify(saveState));
  const prevPages = E.memory.buffer.byteLength / 65536;
  E.memory.grow(Math.ceil(enc.length / 65536) + 1);
  new Uint8Array(E.memory.buffer, prevPages * 65536, enc.length).set(enc);
  return E.sim_restore_json(prevPages * 65536, enc.length);
}

async function main() {
  const bytes = fs.readFileSync(wasmPath);
  const mod = await WebAssembly.compile(bytes);
  const instance = await WebAssembly.instantiate(mod, {});
  const E = instance.exports;

  console.log('── ABI guards ──');
  for (const n of ['sim_trigger_cataclysm', 'sim_cataclysm_count', 'sim_get_cataclysm', 'sim_env_mods']) {
    check(`export ${n}`, typeof E[n] === 'function');
  }
  // One extra page as the ABI out-buffer (mirrors the viewer boot path).
  const pagesBefore = E.memory.buffer.byteLength / 65536;
  E.memory.grow(1);
  const catBuf = pagesBefore * 65536;

  const step = (sim, from, to) => {
    const sp = E.sim_scratch(sim);
    for (let t = from + 1; t <= to; t++) {
      new DataView(E.memory.buffer).setFloat64(sp + OFF_DAYLIGHT, daylight(t), true);
      E.sim_step(sim, 1, sp + OFF_DAYLIGHT, 1);
    }
  };
  const readDust = (sim) => {
    E.sim_env_mods(sim, catBuf);
    return new DataView(E.memory.buffer, catBuf, 16).getFloat64(0, true);
  };
  const readRec = (sim, i) => {
    if (E.sim_get_cataclysm(sim, i, catBuf) === 0) return null;
    const dv = new DataView(E.memory.buffer, catBuf, 40);
    return {
      id: dv.getUint32(0, true), kind: dv.getUint32(4, true),
      tick: dv.getUint32(8, true), endsAt: dv.getUint32(12, true),
      remaining: dv.getUint32(16, true),
      x: dv.getFloat32(20, true), z: dv.getFloat32(24, true),
      radius: dv.getFloat32(28, true), severity: dv.getFloat32(32, true),
      active: dv.getUint32(36, true) === 1,
    };
  };

  // ── Part 1: impact clears resources + exact damage accounting ────────────
  console.log('── impact: resource wipe + mass mortality ──');
  let sim = E.sim_new(SEED);
  step(sim, 0, 100);
  const s0 = readSave(E, sim);
  // Impact center = center of the cell with the most grass (deterministic).
  let bestIdx = 0, bestV = -1;
  s0.Grass.forEach((v, i) => { if (v > bestV) { bestV = v; bestIdx = i; } });
  const col = bestIdx % GRID_N, row = Math.floor(bestIdx / GRID_N);
  const cx = -HALF + (col + 0.5) * CELL, cz = -HALF + (row + 0.5) * CELL;
  const R = 40;
  const carc0 = s0.Carcasses.length;
  const alive0 = s0.Agents.filter((a) => a.Alive).length;
  const id = E.sim_trigger_cataclysm(sim, CATACLYSM_IMPACT, cx, cz, R, 1.0, 100);
  check('impact injected (id ≥ 1)', id >= 1, `id=${id}`);
  check('cataclysm count == 1', E.sim_cataclysm_count(sim) === 1);
  const s1 = readSave(E, sim);
  let inCells = 0, cleared = 0;
  for (let j = 0; j < GRID_N; j++) {
    for (let i = 0; i < GRID_N; i++) {
      const px = -HALF + (i + 0.5) * CELL, pz = -HALF + (j + 0.5) * CELL;
      if (Math.hypot(px - cx, pz - cz) <= R) {
        inCells++;
        const idx = j * GRID_N + i;
        if (s1.Grass[idx] === 0 && s1.MineralField[idx] === 0) cleared++;
      }
    }
  }
  check('all in-radius cells cleared (grass + mineral == 0)', cleared === inCells && inCells > 0,
    `${cleared}/${inCells}`);
  let outsideUntouched = true;
  outer:
  for (let j = 0; j < GRID_N; j++) {
    for (let i = 0; i < GRID_N; i++) {
      const px = -HALF + (i + 0.5) * CELL, pz = -HALF + (j + 0.5) * CELL;
      if (Math.hypot(px - cx, pz - cz) <= R + CELL) continue;
      const idx = j * GRID_N + i;
      if (s1.Grass[idx] !== s0.Grass[idx] || s1.MineralField[idx] !== s0.MineralField[idx]) {
        outsideUntouched = false;
        break outer;
      }
    }
  }
  check('outside cells bit-untouched', outsideUntouched);
  // Agent accounting: survivors lost exactly severity×60; deaths only when
  // pre-energy ≤ 60; each death leaves exactly one carcass.
  let accountingOk = true;
  for (const a of s1.Agents) {
    const d = Math.hypot(a.X - cx, a.Z - cz);
    if (d > R) continue;
    const pre = s0.Agents.find((p) => p.Id === a.Id && p.Alive);
    if (!pre) continue;
    if (!a.Alive) {
      if (pre.Energy > 60 + 1e-9) accountingOk = false;
    } else if (Math.abs(a.Energy - (pre.Energy - 60)) > 1e-9) {
      accountingOk = false;
    }
  }
  const died = s1.Carcasses.length - carc0;
  const aliveNow = s1.Agents.filter((a) => a.Alive).length;
  check('in-radius agents: exact energy accounting (die() path)', accountingOk, `died=${died}`);
  check('deaths accounted (alive count drop == carcass gain)', alive0 - aliveNow === died,
    `alive ${alive0}→${aliveNow}, carcasses +${died}`);
  const rec0 = readRec(sim, 0);
  check('record layout (kind/tick/active/remaining)',
    rec0 !== null && rec0.kind === CATACLYSM_IMPACT && rec0.tick === 100
      && rec0.active === true && rec0.remaining === 100,
    rec0 ? `remaining=${rec0.remaining}` : 'missing');
  E.sim_free(sim);

  // ── Part 2: dust recovery curve ──────────────────────────────────────────
  console.log('── dust recovery curve ──');
  const sim3 = E.sim_new(SEED);
  step(sim3, 0, 50);
  const SEV = 0.6, D = 100;
  E.sim_trigger_cataclysm(sim3, CATACLYSM_IMPACT, 0, 0, 10, SEV, D);
  check('full darkness at injection', Math.abs(readDust(sim3) - (1 - SEV)) < 1e-12,
    readDust(sim3).toFixed(4));
  let curveOk = true, firstBad = -1;
  for (let k = 1; k <= D; k++) {
    step(sim3, 49 + k, 50 + k);
    const dust = readDust(sim3);
    const expected = 1 - SEV * ((D - k) / D);
    if (Math.abs(dust - expected) > 1e-12) { curveOk = false; firstBad = k; break; }
  }
  check('dust recovers linearly over D ticks', curveOk,
    firstBad < 0 ? 'all 100 ticks exact' : `first mismatch @tick ${firstBad}`);
  check('fully recovered after D ticks', Math.abs(readDust(sim3) - 1) < 1e-12);
  const recAfter = readRec(sim3, 0);
  check('record inactive after window (active=0, remaining=0)',
    recAfter !== null && recAfter.active === false && recAfter.remaining === 0);
  E.sim_free(sim3);

  // ── Part 3: v4 roundtrip + lockstep ──────────────────────────────────────
  console.log('── save/restore v4 roundtrip ──');
  const simA = E.sim_new(SEED);
  step(simA, 0, 100);
  E.sim_trigger_cataclysm(simA, CATACLYSM_IMPACT, 120, -80, 90, 0.8, 300);
  E.sim_trigger_cataclysm(simA, CATACLYSM_CLIMATE, 0, 0, 0, 0.5, 200);
  step(simA, 100, 140);
  const save4 = readSave(E, simA);
  const catList = JSON.parse(save4.Cataclysms);
  check('Version >= 4', save4.Version >= 4, `v${save4.Version}`);
  check('v4 save carries 2 cataclysms + id counter', Array.isArray(catList)
    && catList.length === 2 && save4.NextCataclysmId === 3);
  check('climate record kind = 1 with climateDelta = -0.5',
    catList[1].kind === CATACLYSM_CLIMATE && Math.abs(catList[1].climateDelta + 0.5) < 1e-12);
  const simB = restoreSave(E, save4);
  check('restore ok', simB !== 0);
  if (simB !== 0) {
    check('restored cataclysm count == 2', E.sim_cataclysm_count(simB) === 2);
    check('restored dust multiplier matches', Math.abs(readDust(simB) - readDust(simA)) < 1e-12);
    step(simA, 140, 340);
    step(simB, 140, 340);
    const jA = JSON.stringify(readSave(E, simA));
    const jB = JSON.stringify(readSave(E, simB));
    check('lockstep continuation bit-exact (full save JSON equal)', jA === jB);
    E.sim_free(simB);
  }
  E.sim_free(simA);

  // ── Part 4: legacy v3 compat ─────────────────────────────────────────────
  console.log('── legacy v3 save (no cataclysm fields) ──');
  const simC = E.sim_new(SEED);
  step(simC, 0, 100);
  const save3 = readSave(E, simC);
  E.sim_free(simC);
  const legacy = JSON.parse(JSON.stringify(save3));
  delete legacy.Cataclysms;
  delete legacy.NextCataclysmId;
  legacy.Version = 3;
  const ptr = restoreSave(E, legacy);
  check('v3 restore accepted', ptr !== 0);
  if (ptr !== 0) {
    check('cataclysm table empty after v3 restore', E.sim_cataclysm_count(ptr) === 0);
    step(ptr, 100, 200);
    const simD = E.sim_new(SEED);
    step(simD, 0, 200);
    const jLegacy = JSON.stringify(readSave(E, ptr));
    const jControl = JSON.stringify(readSave(E, simD));
    check('v3-restored run matches the never-injected control (bit-exact)', jLegacy === jControl);
    E.sim_free(ptr);
    E.sim_free(simD);
  }

  // ── Part 5: pure renderer helpers ────────────────────────────────────────
  console.log('── renderer pure functions ──');
  const pos = cataclysmMarkPositions(
    [{ id: 1, kind: CATACLYSM_IMPACT, tick: 600 }, { id: 2, kind: CATACLYSM_CLIMATE, tick: 5000 }],
    { rootTick: 300, currentTick: 900, newestTick: 900 });
  check('mark positions: in-range phyloX, old mark in spectrogram window, future mark excluded',
    pos.marks.length === 2
      && pos.marks[0].phyloX > 0 && pos.marks[0].phyloX < 1 && pos.marks[0].inSpectrogram
      && pos.marks[1].inSpectrogram === false);
  const hud = formatCataclysmHud(
    [{ id: 1, kind: CATACLYSM_IMPACT, tick: 600, remaining: 100, active: true }], 0.42);
  check('HUD badge carries dust readout + ☄ glyph', hud.includes('dust=0.42') && hud.includes('☄'));
  const hudHist = formatCataclysmHud([
    { id: 1, kind: CATACLYSM_IMPACT, tick: 600, remaining: 0, active: false },
    { id: 2, kind: CATACLYSM_CLIMATE, tick: 900, remaining: 5, active: true },
  ], 1);
  check('HUD history line lists recent entries with 🌡', hudHist.includes('灾变') && hudHist.includes('🌡'));

  console.log(failures === 0 ? '\nALL CHECKS PASSED' : `\n${failures} CHECK(S) FAILED`);
  process.exit(failures === 0 ? 0 : 1);
}

main().catch((e) => { console.error('FAIL: crashed:', e); process.exit(1); });
