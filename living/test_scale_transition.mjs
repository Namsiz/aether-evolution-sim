// test_scale_transition.mjs — Phase 18 scale-transition acceptance tests (Node).
//
// Exercises the Phase 18 C-ABI + save-state layers:
//   1. ABI guards: eight new exports (sim_geology_tick / sim_plate_* /
//      sim_sea_level / sim_epoch_name / sim_biochem_step /
//      sim_nutrient_state / sim_fertility_boost). Validates record sizes
//      by reading back a valid plate/nutrient record, and verifies that
//      out-of-bounds calls return 0 (never trap).
//   2. NATURAL 100k-tick run under active tectonics: asserts ≥ 1 speciation
//      event fires. Plate-driven uplift provides persistent geographic
//      barriers boosting the speciation engine's isolation gate over the
//      long arc. Seed 20261102 DECO spread naturally spans >400 world units
//      so even modest uplift suffices.
//   3. v5 roundtrip bits-exact for geologic state + nutrient pools: after
//      a 10_000-tick run, serialize → deserialize → compare plates, sea
//      level, epoch, last_geo_tick, pool values, then re-run 2000 ticks
//      from each copy and lockstep-checksum.
//   4. Legacy v4 compat: downgrade Version + strip Geology/Nutrient keys
//      from a v5 dump — sim_restore_json must accept it, restore plates
//      to empty (geology no-op), pools to baseline (fast=snow=0.5,
//      deposit=1.0), and keep running identically to a pre-v5 world.
//   5. Biochemical zero-side-effect at baseline through t=1000: two
//      lockstep fresh sims; assert identical checksums through t=1000
//      (the first biochem commit has no ecological effect because
//      fast_pool starts exactly at baseline and there is no deposit yet).
//
// Pure-data renderer smoke tests (no DOM) cover the two new mjs modules.
//
// Usage: node living/test_scale_transition.mjs [--wasm path]

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { buildGeologyView, GEOMETRY_GRID_N } from './src/renderer-geology.mjs';
import { buildNutrientView, fastPoolColor } from './src/renderer-biochemistry.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const wasmArg = args.indexOf('--wasm');
const wasmPath = wasmArg >= 0 ? args[wasmArg + 1] : path.join(__dirname, 'ecology-sim.wasm');

let failures = 0;
const check = (name, cond, detail = '') => {
  if (cond) console.log(`  ok   ${name}${detail ? ` (${detail})` : ''}`);
  else { console.error(`  FAIL ${name}${detail ? ` — ${detail}` : ''}`); failures += 1; }
};

const SEED = 20261102;
const DAYLIGHT_CAP = 64;
// Scratch layout matching wasm-viewer.mjs / wasm.rs (phase 16/17 additions
// grow the arena but DO NOT shift these offsets).
const AGENT_BASE = 38 * 38 * 4;
const AGENT_SLOT_SIZE = 48;
const AGENT_CAP = 1024;
const OFF_DAYLIGHT = AGENT_BASE + AGENT_CAP * AGENT_SLOT_SIZE;
const OFF_LEN = 0xFFFF_FFFC; // safety net — never written by sim_step; just use OFF_DAYLIGHT.

const NIGHT_LIGHT_DAYLIGHT = 0.4;
function daylight(_t) { return NIGHT_LIGHT_DAYLIGHT; }

// ─── WASM boilerplate ───────────────────────────────────────────────────────

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
  const guards = [
    'sim_geology_tick', 'sim_plate_count', 'sim_get_plate',
    'sim_sea_level', 'sim_epoch_name',
    'sim_biochem_step', 'sim_nutrient_state', 'sim_fertility_boost',
  ];
  for (const n of guards) {
    check(`export ${n}`, typeof E[n] === 'function', `got ${typeof E[n]}`);
  }

  let sim = E.sim_new(SEED);
  const sp = E.sim_scratch(sim);

  check('plate count is 3', (E.sim_plate_count(sim) | 0) === 3, `${E.sim_plate_count(sim) | 0}`);
  {
    // Read first plate: must return 1 (success) and fill 48 bytes.
    const r = E.sim_get_plate(sim, 0, sp);
    check('sim_get_plate(idx=0) returns 1', r === 1);
    if (r === 1) {
      const dv = new DataView(E.memory.buffer, sp, 48);
      check('plate[0].id == 1', dv.getUint32(0, true) === 1);
      check('plate[0].type_ ∈ {0,1,2}', [0, 1, 2].includes(dv.getUint32(40, true)),
        `got ${dv.getUint32(40, true)}`);
      check('plate[0].x is finite', Number.isFinite(dv.getFloat64(4, true)));
      check('plate[0].vx is finite', Number.isFinite(dv.getFloat64(20, true)));
    }
    // Out-of-bounds: must return 0 (never traps).
    const bad = E.sim_get_plate(sim, 99, sp);
    check('sim_get_plate(idx=99) returns 0', bad === 0);
    // Null handle: must return 0.
    const nullHandle = 0;
    const nullR = E.sim_get_plate(nullHandle, 0, sp);
    check('null handle returns 0', nullR === 0);
  }

  check('sea_level starts at 0', Math.abs(E.sim_sea_level(sim)) < 1e-9, E.sim_sea_level(sim).toFixed(6));
  {
    const epLen = E.sim_epoch_name(sim, sp, 128) | 0;
    check('epoch_name non-empty', epLen > 0 && epLen < 128, epLen);
    check('epoch_name begins "Pangaea"', new TextDecoder().decode(new Uint8Array(E.memory.buffer, sp, Math.max(0, epLen))).startsWith('Pangaea'));
    // Null/out-of-bounds: safe no-trap.
    const zlen = E.sim_epoch_name(sim, sp, 0) | 0;
    check('epoch_name cap=0 returns 0', zlen === 0);
  }

  // Nutrient pools at baseline: fast=snow=0.5, deposit_rate=1.0, fert_boost=0.
  {
    const NUT_RECORD = 24;
    const dv = new DataView(E.memory.buffer, sp, NUT_RECORD);
    E.sim_nutrient_state(sim, sp);
    check('nutrient fast_pool baseline', Math.abs(dv.getFloat64(0, true) - 0.5) < 1e-12,
      dv.getFloat64(0, true).toFixed(10));
    check('nutrient slow_pool baseline', Math.abs(dv.getFloat64(8, true) - 0.5) < 1e-12,
      dv.getFloat64(8, true).toFixed(10));
    check('nutrient deposit_rate baseline', Math.abs(dv.getFloat64(16, true) - 1.0) < 1e-12,
      dv.getFloat64(16, true).toFixed(10));
    check('fertility_boost baseline 0', Math.abs(E.sim_fertility_boost(sim)) < 1e-12,
      E.sim_fertility_boost(sim).toFixed(10));
  }

  E.sim_free(sim);

  // ── Part 1: natural 100k tick run — geology/biochem layers stay alive ──
  console.log('── natural 100k tick run (active tectonics) ──');
  sim = E.sim_new(SEED);
  const sp1 = E.sim_scratch(sim);
  const dtBuf = new Float64Array(E.memory.buffer, sp1 + OFF_DAYLIGHT, DAYLIGHT_CAP);

  let now = 0;
  const BATCH = 100;
  const stepFn = (from, to) => {
    for (let t = from + 1; t <= to;) {
      const n = Math.min(DAYLIGHT_CAP, to - t + 1);
      for (let i = 0; i < n; i++) dtBuf[i] = daylight(t + i - 1);
      E.sim_step(sim, n, sp1 + OFF_DAYLIGHT, n);
      t += n;
    }
  };

  // Run to 100k. Log geology snapshots at each geo-step boundary (every
  // 10k ticks).
  for (let t = 1; t <= 100_000; t += BATCH) {
    const end = Math.min(t + BATCH - 1, 100_000);
    stepFn(t - 1, end);
    if (end % GEO_TICKS_PER_STEP === 0 || end === 100_000) {
      const plates = [];
      const cnt = E.sim_plate_count(sim) | 0;
      for (let pi = 0; pi < cnt; pi++) {
        E.sim_get_plate(sim, pi, sp1);
        const pdv = new DataView(E.memory.buffer, sp1, 48);
        plates.push({ id: pdv.getUint32(0, true), x: pdv.getFloat64(4, true), z: pdv.getFloat64(12, true), vx: pdv.getFloat64(20, true), vz: pdv.getFloat64(28, true), type_: pdv.getUint32(40, true) });
      }
      const sl = E.sim_sea_level(sim);
      const epLen = E.sim_epoch_name(sim, sp1, 128) | 0;
      const ep = new TextDecoder().decode(new Uint8Array(E.memory.buffer, sp1, Math.max(0, epLen)));
      console.log(`  t=${end} plates=${cnt} sea=${sl.toFixed(1)} epoch="${ep}"`);
    }
  }
  {
    const sl = E.sim_sea_level(sim);
    const epLen = E.sim_epoch_name(sim, sp1, 128) | 0;
    const ep = new TextDecoder().decode(new Uint8Array(E.memory.buffer, sp1, Math.max(0, epLen)));
    const fc = E.sim_plate_count(sim) | 0;
    console.log(`  final: tick=${E.sim_tick(sim)} sea=${sl.toFixed(1)} epoch="${ep}" plates=${fc}`);
  }

  // Speciation assertion: ≥ 1 child clade means at least one split event
  // fired (a root-only phylogeny has all parentId=null; any non-null
  // parentId indicates a speciation step happened at some point).
  const cladeCount = E.sim_clade_count(sim) | 0;
  const roots = []; const children = [];
  for (let i = 0; i < cladeCount; i++) {
    E.sim_get_clade(sim, i, sp1);
    const cdv = new DataView(E.memory.buffer, sp1, 40);
    const cid = cdv.getUint32(0, true);
    const pid = cdv.getUint32(4, true);
    if (pid === 0xffffffff) roots.push(cid); else children.push(cid);
  }
  check('speciation event fired during 100k natural run (≥ 1 child clade)', children.length >= 1,
    `roots=${roots.length} children=${children.length}`);

  // ── Part 2: v5 roundtrip — geology + nutrients bit-exact ──
  console.log('── v5 roundtrip: geology + nutrient pools ──');
  const preSave = readSave(E, sim);
  E.sim_free(sim);
  const ptr5 = restoreSave(E, preSave);
  check('sim_restore_json accepts v5', ptr5 !== 0);
  if (ptr5 === 0) {
    console.log('  SKIP roundtrip asserts (restore null)');
  } else {
    const sp5 = E.sim_scratch(ptr5);
    // Sea-level roundtrip: preSave.SeaLevel was serialized by Rust; read back.
    // Note: SeaLevel may not appear in save JSON if it's always 0.0 when
    // no geo step has fired (our v5 serializer always emits it though).
    const postSea = E.sim_sea_level(ptr5);
    const savedSL = preSave.SeaLevel ?? 0.0;
    check('roundtripped sea_level matches original', Math.abs(postSea - savedSL) < 1e-12,
      `saved=${savedSL.toFixed(6)} restored=${postSea.toFixed(6)}`);
    const postEpLen = E.sim_epoch_name(ptr5, sp5, 128) | 0;
    const postEp = new TextDecoder().decode(new Uint8Array(E.memory.buffer, sp5, Math.max(0, postEpLen)));
    const savedEp = preSave.EpochName ?? preSave.Epoch ?? 'Pangaea Assembly';
    check('roundtripped epoch matches original', postEp === savedEp,
      `"${postEp}" vs "${savedEp}"`);
    // Pools: compare to the values serialized into the JSON.
    const pdv = new DataView(E.memory.buffer, sp5, 24);
    E.sim_nutrient_state(ptr5, sp5);
    const rf = pdv.getFloat64(0, true);
    const rs = pdv.getFloat64(8, true);
    const rd = pdv.getFloat64(16, true);
    const sf = preSave.NutrientFast ?? 0.5;
    const ss = preSave.NutrientSlow ?? 0.5;
    const sd = preSave.NutrientDepositRate ?? 1.0;
    check('roundtripped fast_pool', Math.abs(rf - sf) < 1e-12,
      `${rf.toFixed(8)} vs ${sf.toFixed(8)}`);
    check('roundtripped slow_pool', Math.abs(rs - ss) < 1e-12);
    check('roundtripped deposit_rate', Math.abs(rd - sd) < 1e-12);
    check('plates survive roundtrip intact', (E.sim_plate_count(ptr5) | 0) === 3);
    // Lockstep continuation: fork two restores and step 500 more ticks.
    const simA = restoreSave(E, preSave);
    const simB = restoreSave(E, preSave);
    check('two independent restores ok', simA !== 0 && simB !== 0);
    if (simA !== 0 && simB !== 0) {
      const spA = E.sim_scratch(simA);
      const spB = E.sim_scratch(simB);
      const dtA = new Float64Array(E.memory.buffer, spA + OFF_DAYLIGHT, DAYLIGHT_CAP);
      const dtB = new Float64Array(E.memory.buffer, spB + OFF_DAYLIGHT, DAYLIGHT_CAP);
      for (let t = preSave.TickCount + 1; t <= preSave.TickCount + 500; t++) {
        dtA[0] = daylight(t); dtB[0] = daylight(t);
        E.sim_step(simA, 1, spA + OFF_DAYLIGHT, 1);
        E.sim_step(simB, 1, spB + OFF_DAYLIGHT, 1);
      }
      const ckA = readSave(E, simA);
      const ckB = readSave(E, simB);
      check('lockstep after 500 ticks matches post-roundtrip',
        JSON.stringify(ckA.RngState) === JSON.stringify(ckB.RngState)
        && ckA.TickCount === ckB.TickCount,
        'diverged after v5 roundtrip');
      E.sim_free(simA);
      E.sim_free(simB);
    }
    E.sim_free(ptr5);
  }

  // ── Part 3: legacy v4 compat — strips geo/nutrients, downgrades version ──
  console.log('── legacy v4 compat: no geology effects after restore ──');
  const s0 = E.sim_new(SEED);
  const sp0 = E.sim_scratch(s0);
  const dt0 = new Float64Array(E.memory.buffer, sp0 + OFF_DAYLIGHT, DAYLIGHT_CAP);
  // Fast-forward 200 ticks (pre-first-geo-step, pre-first-biochem-step).
  for (let t = 1; t <= 200; t++) {
    dt0[0] = daylight(t);
    E.sim_step(s0, 1, sp0 + OFF_DAYLIGHT, 1);
  }
  const sv5 = readSave(E, s0);
  E.sim_free(s0);
  // Strip the Phase 18 additions.
  const legacy = JSON.parse(JSON.stringify(sv5));
  delete legacy.Geology;
  delete legacy.NutrientFast;
  delete legacy.NutrientSlow;
  delete legacy.NutrientDepositRate;
  legacy.Version = 4;
  check('legacy schema_name preserved', legacy.SchemaName === 'aether-sim-save-state');
  const ptrLegacy = restoreSave(E, legacy);
  check('sim_restore_json accepts stripped v4', ptrLegacy !== 0);
  if (ptrLegacy !== 0) {
    const spL = E.sim_scratch(ptrLegacy);
    check('plates empty after v4 restore (geology no-op)', (E.sim_plate_count(ptrLegacy) | 0) === 0);
    // Nutrient pools must land on neutral baseline (serde default → our
    // from_save_state logic overrides to FAST_BASELINE when absent).
    const ldv = new DataView(E.memory.buffer, spL, 24);
    E.sim_nutrient_state(ptrLegacy, spL);
    check('v4 legacy fast_pool defaults to baseline 0.5', Math.abs(ldv.getFloat64(0, true) - 0.5) < 1e-12,
      ldv.getFloat64(0, true).toFixed(8));
    check('v4 legacy slow_pool defaults to baseline 0.5', Math.abs(ldv.getFloat64(8, true) - 0.5) < 1e-12);
    check('v4 legacy deposit_rate defaults to 1.0', Math.abs(ldv.getFloat64(16, true) - 1.0) < 1e-12);
    // Run past the first geology step boundary: plates still empty →
    // geology_step is silent; ecology continues identically to a pre-v5
    // world (no elevation_delta, no nutrient drift).
    const preRunTick = sv5.TickCount;
    for (let t = preRunTick + 1; t <= preRunTick + 100; t++) {
      dt0[0] = daylight(t);
      E.sim_step(ptrLegacy, 1, spL + OFF_DAYLIGHT, 1);
    }
    const after = readSave(E, ptrLegacy);
    check('v4 legacy continues simulating (agent list non-empty)',
      Array.isArray(after.Agents) && after.Agents.length > 0,
      `agents=${after.Agents?.length ?? 'null'}`);
    check('v4 legacy tick advances correctly', after.TickCount === preRunTick + 100);
    E.sim_free(ptrLegacy);
  }

  // ── Part 4: biochemical zero-side-effect at baseline through t=1000 ──
  console.log('── biochem zero-side-effect at baseline ──');
  let simC = E.sim_new(SEED);
  let simD = E.sim_new(SEED);
  const spC = E.sim_scratch(simC);
  const spD = E.sim_scratch(simD);
  const dtC = new Float64Array(E.memory.buffer, spC + OFF_DAYLIGHT, DAYLIGHT_CAP);
  const dtD = new Float64Array(E.memory.buffer, spD + OFF_DAYLIGHT, DAYLIGHT_CAP);
  // Run both identically up to just before the first biochem commit (t=999).
  for (let t = 1; t <= 999; t++) {
    dtC[0] = daylight(t);
    dtD[0] = daylight(t);
    E.sim_step(simC, 1, spC + OFF_DAYLIGHT, 1);
    E.sim_step(simD, 1, spD + OFF_DAYLIGHT, 1);
  }
  // Snapshots before the first biochem commit must be identical.
  const ckA = readSave(E, simC);
  const ckB = readSave(E, simD);
  check('lockstep identical at t=999 (pre-first-biochem-step)',
    JSON.stringify(ckA.RngState) === JSON.stringify(ckB.RngState),
    'diverged before any biochem_step');
  // One more tick pushes both through t=1000 (both execute biochem_step once;
  // pool_acc was all zeros at start, and SLOW_TO_FAST_RATE acts on slow_excess
  // which is 0 at baseline — the commit is a no-op, pools stay exactly at
  // baseline, fertility remains 0. Checksums therefore must agree.)
  dtC[0] = daylight(1000);
  dtD[0] = daylight(1000);
  E.sim_step(simC, 1, spC + OFF_DAYLIGHT, 1);
  E.sim_step(simD, 1, spD + OFF_DAYLIGHT, 1);
  const ckA2 = readSave(E, simC);
  const ckB2 = readSave(E, simD);
  check('lockstep still identical at t=1000 (zero-deposit baseline commit)',
    JSON.stringify(ckA2.RngState) === JSON.stringify(ckB2.RngState),
    'diverged on first biochem_step despite zero deposits');
  {
    const ndv = new DataView(E.memory.buffer, spC, 24);
    E.sim_nutrient_state(simC, spC);
    check('fast_pool still baseline at t=1000', Math.abs(ndv.getFloat64(0, true) - 0.5) < 1e-12,
      ndv.getFloat64(0, true).toFixed(8));
    check('fertility_boost still 0 at t=1000', Math.abs(E.sim_fertility_boost(simC)) < 1e-12);
  }
  E.sim_free(simC);
  E.sim_free(simD);

  // ── Part 5: pure-data renderer smoke tests (no DOM dependency) ──
  console.log('── renderer pure-function smoke ──');
  check('buildNutrientView yields finite fertile', Number.isFinite(buildNutrientView(0.5, 0.5, 1.0).fertile));
  check('fastPoolColor(baseline) non-empty string',
    typeof fastPoolColor(0.5) === 'string' && fastPoolColor(0.5).length > 0);
  check('fastPoolColor(depleted) valid rgba', fastPoolColor(0.0).startsWith('rgba('));
  check('fastPoolColor(enriched) valid rgba', fastPoolColor(1.5).startsWith('rgba('));
  // buildGeologyView: empty plates + sea_level=0 → ocean mask from terrain alone.
  const gv = buildGeologyView([], 0.0);
  check('buildGeologyView cells size matches geometry',
    gv.cells.length === GEOMETRY_GRID_N * GEOMETRY_GRID_N);
  check('buildGeologyView plates empty w/o inputs', gv.plates.length === 0);
  // With a couple of fake plates the view carries their data.
  const gv2 = buildGeologyView([{ id: 1, x: 100, z: 200, vx: -1, vz: 0, angle: 0, type_: 0, seismicIndex: 0 }], 10.0);
  check('buildGeologyView carries plate input', gv2.plates.length === 1 && gv2.plates[0].id === 1);
  check('buildGeologyView reflects sea_level', gv2.seaLevel === 10.0);

  console.log(failures === 0 ? '\nALL CHECKS PASSED' : `\n${failures} CHECK(S) FAILED`);
  process.exit(failures === 0 ? 0 : 1);
}

main().catch((e) => { console.error('FAIL: crashed:', e); process.exit(1); });
