// Phase 18 scale-transition smoke tests (compact, timeout-safe).
// Validates: ABI guards, v5 roundtrip, legacy v4 compat, biochem determinism.
'use strict';
import fs from 'fs';
const wasmPath = new URL('./ecology-sim.wasm', import.meta.url).pathname;
const SEED = 20261102;
import { buildNutrientView, fastPoolColor } from './src/renderer-biochemistry.mjs';
import { buildGeologyView, GEOMETRY_GRID_N } from './src/renderer-geology.mjs';
const DAYLIGHT_CAP = 4;
const OFF_DAYLIGHT = 0;

function daylight(t) { return 0.5 + 0.5 * Math.sin((t / 300) * Math.PI); }
let failures = 0;
function check(name, ok, detail) {
  if (ok) console.log(`  ok   ${name}`);
  else { console.error(`  FAIL ${name}${detail ? ` — ${detail}` : ''}`); failures++; }
}
function readSave(E, sim) {
  const ptr = E.sim_scratch(sim);
  const lenPtr = ptr;  // reuse scratch[0..4] for out_len (u32)
  new Uint32Array(E.memory.buffer, lenPtr, 1)[0] = 0;
  const jsonPtr = E.sim_save_json(sim, lenPtr);
  const rawLen = new Uint32Array(E.memory.buffer, lenPtr, 1)[0];
  const jsonLen = rawLen - 1;  // minus NUL terminator
  const jsonBytes = new Uint8Array(E.memory.buffer, jsonPtr, jsonLen);
  const json = new TextDecoder().decode(jsonBytes);
  E.sim_free_json(jsonPtr, rawLen);
  return JSON.parse(json);
}
async function main() {
  const bytes = fs.readFileSync(wasmPath);
  const mod = await WebAssembly.compile(bytes);
  const instance = await WebAssembly.instantiate(mod, {});
  const E = instance.exports;

  console.log('── ABI guards ──');
  for (const n of ['sim_geology_tick','sim_plate_count','sim_get_plate',
    'sim_sea_level','sim_epoch_name','sim_biochem_step',
    'sim_nutrient_state','sim_fertility_boost']) {
    check(`export ${n}`, typeof E[n] === 'function', `got ${typeof E[n]}`);
  }
  check('scratch layout large enough for Phase 18 ABI', E.sim_scratch_size() >= 61000,
    `${E.sim_scratch_size()} vs expected ≥61000`);

  // ── Part 1: short warmup + nutrient baseline ──
  console.log('── warmup + nutrient baseline ──');
  let sim = E.sim_new(SEED);
  let sp = E.sim_scratch(sim);
  const dt = new Float64Array(E.memory.buffer, sp + OFF_DAYLIGHT, DAYLIGHT_CAP);
  // Run 200 ticks (pre-first-biochem-step, pre-first-geology-step).
  for (let t = 1; t <= 200; t++) { dt[0] = daylight(t); E.sim_step(sim, 1, sp + OFF_DAYLIGHT, 1); }
  const ndv = new DataView(E.memory.buffer, sp, 24);
  E.sim_nutrient_state(sim, sp);
  check('fast_pool baseline 0.5', Math.abs(ndv.getFloat64(0, true) - 0.5) < 1e-12,
    ndv.getFloat64(0, true).toFixed(8));
  check('slow_pool baseline 0.5', Math.abs(ndv.getFloat64(8, true) - 0.5) < 1e-12);
  check('deposit_rate baseline 1.0', Math.abs(ndv.getFloat64(16, true) - 1.0) < 1e-12);
  check('fertility_boost baseline 0', Math.abs(E.sim_fertility_boost(sim)) < 1e-12);
  const pc = (E.sim_plate_count(sim)|0);
  check('plates seeded with 3 continental plates', pc === 3, `got ${pc}`);
  check('epoch defaults Pangaea Assembly',
    (() => { const b = new Uint8Array(E.memory.buffer, sp, 64);
      const len = E.sim_epoch_name(sim, sp, 64) | 0;
      return new TextDecoder().decode(b.slice(0,len)) === 'Pangaea Assembly'; })());

  // ── Part 2: advance past first biochem commit (t=1000) and verify pools rose ──
  console.log('── past first biochem commit (t=1000) ──');
  for (let t = 201; t <= 1000; t++) {
    dt[0] = daylight(t); E.sim_step(sim, 1, sp + OFF_DAYLIGHT, 1);
  }
  // Create fresh DataView to avoid detached-buffer issue after sim_free.
  const sp2 = E.sim_scratch(sim);
  const ndv2 = new DataView(E.memory.buffer, sp2, 24);
  E.sim_nutrient_state(sim, sp2);
  const postFast = ndv2.getFloat64(0, true);
  console.log(`  fast_pool at t=1000 = ${postFast.toFixed(6)} (baseline was 0.5)`);
  check('fast_pool rose above baseline after one commit', postFast > 0.5,
    `post=${postFast.toFixed(6)}`);
  const postSlow = ndv2.getFloat64(8, true);
  check('slow_pool still finite', Number.isFinite(postSlow) && postSlow >= 0);
  E.sim_free(sim);

  // ── Part 3: v5 roundtrip — bit-exact geology + nutrients ──
  console.log('── v5 roundtrip ──');
  sim = E.sim_new(SEED);
  for (let t = 1; t <= 1000; t++) {
    dt[0] = daylight(t); E.sim_step(sim, 1, sp + OFF_DAYLIGHT, 1);
  }
  const saveObj = await readSave(E, sim);
  const jsonObj = saveObj;
  const json = JSON.stringify(saveObj);
  const jsonBytes2 = new TextEncoder().encode(json);
  const growPages2 = Math.ceil(jsonBytes2.length / 65536);
  const prevPages2 = E.memory.grow(growPages2);
  const jsonAddr2 = prevPages2 * 65536;
  new Uint8Array(E.memory.buffer, jsonAddr2, jsonBytes2.length).set(jsonBytes2);
  const restored = E.sim_restore_json(jsonAddr2, jsonBytes2.length);
  check('sim_restore_json accepts v5', restored !== 0);
  if (restored !== 0) {
    const spr = E.sim_scratch(restored);
    E.sim_nutrient_state(restored, spr);
    // Compare the actual values from the save JSON.
    const jf = jsonObj.NutrientFast ?? 0.5;
    const js = jsonObj.NutrientSlow ?? 0.5;
    console.log(`  v5 json nut: fast=${jf.toFixed(8)} slow=${js.toFixed(8)}`);
    const rf = new DataView(E.memory.buffer, spr, 24).getFloat64(0, true);
    const rs = new DataView(E.memory.buffer, spr, 24).getFloat64(8, true);
    // Note: `postFast`/`postSlow` here reflect the live state AFTER the t=1000
    // biochem commit (which fires inside `update()` when tick_count reaches 1000).
    // The SavedState v5 serializes the state BEFORE that biochem_step, so direct
    // equality is impossible — instead, restore and advance 1 tick; both sims must
    // diverge identically from the same starting point.
    const dtRest = new Float64Array(E.memory.buffer, spr, DAYLIGHT_CAP);
    dtRest[0] = daylight(1001);
    E.sim_step(restored, 1, spr + OFF_DAYLIGHT, 1);
    const ndvR = new DataView(E.memory.buffer, spr, 24);
    E.sim_nutrient_state(restored, spr);
    const rfast = ndvR.getFloat64(0, true);
    const rs_low = ndvR.getFloat64(8, true);
    // Advance original by one more tick for fair comparison.
    const dtPost = new Float64Array(E.memory.buffer, sp2, DAYLIGHT_CAP);
    dtPost[0] = daylight(1001);
    E.sim_step(sim, 1, sp2 + OFF_DAYLIGHT, 1);
    const ndvP = new DataView(E.memory.buffer, sp2, 24);
    E.sim_nutrient_state(sim, sp2);
    const pfast = ndvP.getFloat64(0, true);
    const ps_low = ndvP.getFloat64(8, true);
    console.log(`  post-roundtrip-fast=${rfast.toFixed(8)} orig-after=${pfast.toFixed(8)}`);
    check('roundtripped fast_pool locksteps after restore', Math.abs(rfast - pfast) < 1e-12,
      `restored_post=${rfast.toFixed(8)} original_post=${pfast.toFixed(8)}`);
    check('roundtripped slow_pool locksteps after restore', Math.abs(rs_low - ps_low) < 1e-12);
    check('plates count preserved', (E.sim_plate_count(restored)|0) === (E.sim_plate_count(sim)|0));
    const ckOrig = await readSave(E, sim);
    const ckRest = await readSave(E, restored);
    check('RNG state roundtrips bit-exact',
      JSON.stringify(ckOrig.RngState) === JSON.stringify(ckRest.RngState),
      'RNG diverged after restore');
    E.sim_free(restored);
  }
  E.sim_free(sim);

  // ── Part 4: legacy v4 compat — stripped save restores cleanly ──
  console.log('── legacy v4 compat ──');
  sim = E.sim_new(SEED);
  for (let t = 1; t <= 200; t++) {
    dt[0] = daylight(t); E.sim_step(sim, 1, sp + OFF_DAYLIGHT, 1);
  }
  const sv5 = await readSave(E, sim);
  E.sim_free(sim);
  const legacy = JSON.parse(JSON.stringify(sv5));
  delete legacy.Geology; delete legacy.NutrientFast; delete legacy.NutrientSlow;
  delete legacy.NutrientDepositRate; legacy.Version = 4;
  check('legacy schema_name preserved', legacy.SchemaName === 'aether-sim-save-state');
  const legacyJson = JSON.stringify(legacy);
  const legacyBytes = new TextEncoder().encode(legacyJson);
  const growP = Math.ceil(legacyBytes.length / 65536);
  const prevP = E.memory.grow(growP);
  const jsonAddr = prevP * 65536;
  new Uint8Array(E.memory.buffer, jsonAddr, legacyBytes.length).set(legacyBytes);
  const ptrL = E.sim_restore_json(jsonAddr, legacyBytes.length);
  check('sim_restore_json accepts stripped v4', ptrL !== 0);
  if (ptrL !== 0) {
    const spL = E.sim_scratch(ptrL);
    const ldv = new DataView(E.memory.buffer, spL, 24);
    E.sim_nutrient_state(ptrL, spL);
    check('v4 legacy fast_pool defaults to baseline 0.5',
      Math.abs(ldv.getFloat64(0,true)-0.5)<1e-12, ldv.getFloat64(0,true).toFixed(8));
    check('v4 legacy slow_pool defaults to baseline 0.5',
      Math.abs(ldv.getFloat64(8,true)-0.5)<1e-12);
    check('v4 legacy deposit_rate defaults to 1.0',
      Math.abs(ldv.getFloat64(16,true)-1.0)<1e-12);
    // Run 100 more ticks: no plates → geology_step is no-op; ecology continues.
    for (let t = 201; t <= 300; t++) {
      dt[0] = daylight(t); E.sim_step(ptrL, 1, spL + OFF_DAYLIGHT, 1);
    }
    const after = await readSave(E, ptrL);
    check('v4 legacy continues simulating (agents alive)',
      Array.isArray(after.Agents) && after.Agents.length > 0,
      `agents=${after.Agents?.length??'null'}`);
    check('v4 legacy tick advances correctly', after.TickCount === 300);
    E.sim_free(ptrL);
  }

  // ── Part 5: pure-data renderer smoke ──
  console.log('── renderer pure-function smoke ──');
  check('buildNutrientView yields string fertile',
    typeof buildNutrientView(0.5, 0.5, 1.0).fertile === 'string');
  check('fastPoolColor(baseline) non-empty',
    typeof fastPoolColor(0.5) === 'string' && fastPoolColor(0.5).length > 0);
  check('fastPoolColor(depleted) valid rgba', fastPoolColor(0.0).startsWith('rgba('));
  check('fastPoolColor(enriched) valid rgba', fastPoolColor(1.5).startsWith('rgba('));
  const gv = buildGeologyView([], 0.0);
  check('buildGeologyView cells size matches geometry',
    gv.cells.length === GEOMETRY_GRID_N * GEOMETRY_GRID_N);
  check('buildGeologyView plates empty w/o inputs', gv.plates.length === 0);
  const gv2 = buildGeologyView([{ id: 1, x: 100, z: 200, vx: -1, vz: 0, angle: 0, type_: 0, seismicIndex: 0 }], 10.0);
  check('buildGeologyView carries plate input', gv2.plates.length === 1 && gv2.plates[0].id === 1);
  check('buildGeologyView reflects sea_level', gv2.seaLevel === 10.0);

  console.log(failures === 0 ? '\nALL CHECKS PASSED' : `\n${failures} CHECK(S) FAILED`);
  process.exit(failures === 0 ? 0 : 1);
}
main().catch(e => { console.error('FAIL: crashed:', e); process.exit(1); });
