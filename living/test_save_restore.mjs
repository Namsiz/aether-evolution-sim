// test_save_restore.mjs — Phase 6 save/restore smoke test for the C-ABI wasm.
//
// Exercises the JS-side workflow end-to-end:
//   1. sim_new(20261102) → sim_step ×100 (sinusoidal daylight).
//   2. sim_save_json → UTF-8 JSON (aether-sim-save-state v1).
//   3. sim_restore_json → fresh handle (original sim unaffected).
//   4. Both sims advance 100 more ticks with identical daylight.
//   5. sim_snapshot_json on both → parseSnapshot (v3) → six-species census match.
//
// Memory discipline: all typed-array views referencing E.memory.buffer are
// constructed LAZILY (right before read/write), never cached across calls,
// so a post-call memory.grow cannot invalidate them.
//
// JSON payload placement: sim_restore_json wants a raw byte buffer pointer.
// We grow enough pages upfront (ceil(json_len / 65536)) and write the
// encoded bytes into the freshly committed region at byte offset
// prevPages*65536, which is guaranteed valid for the duration of the call.
//
// Usage: node living/test_save_restore.mjs [--wasm path]

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseSnapshot } from './src/renderer-wasm-viewer.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const wasmArg = args.indexOf('--wasm');
const wasmPath = wasmArg >= 0 ? args[wasmArg + 1] : path.join(__dirname, 'ecology-sim.wasm');

let failures = 0;
const check = (name, cond, detail = '') => {
  if (cond) console.log(`  ok   ${name}${detail ? ` (${detail})` : ''}`);
  else { console.error(`  FAIL ${name}${detail ? ` — ${detail}` : ''}`); failures += 1; }
};

const daylight = t => 0.5 + 0.5 * Math.sin((t / 300) * Math.PI);

// Scratch layout constants (must mirror wasm.rs).
const AGENT_CAP = 1024;
const DAYLIGHT_CAP = 64;
const MINERAL_BYTES = 38 * 38 * 4; // 5776
const AGENT_BASE = MINERAL_BYTES;
const AGENT_SLOT_SIZE = 48;
const OFF_DAYLIGHT = AGENT_BASE + AGENT_CAP * AGENT_SLOT_SIZE;
/// Phase 10: histogram 区位于 counts 之后（DECO_HUE_BUCKETS_CAP × u32）。
const HIST_BYTES = 64 * 4;
/// Phase 15: 系统发育区追加在 hist 之后（clade/event/name，与 wasm.rs 同步）。
const CLADE_BYTES = 64 * 40 + 64 * 56 + 32; // Phase 16: clade record 36→40 (niche f32)
const SCRATCH_SIZE = OFF_DAYLIGHT + DAYLIGHT_CAP * 8 + 6 * 4 + HIST_BYTES + CLADE_BYTES;
const OFF_LEN = SCRATCH_SIZE - 4; // last u32 slot

async function main() {
  const bytes = fs.readFileSync(wasmPath);
  const mod = await WebAssembly.compile(bytes);
  const instance = await WebAssembly.instantiate(mod, {});
  const E = instance.exports;

  // Helpers: each builds a fresh view on the CURRENT ArrayBuffer so memory
  // growth between calls cannot corrupt our accesses.
  const bufU8  = (addr, len)  => new Uint8Array (E.memory.buffer, addr, len);
  const bufU32 = (addr)       => new Uint32Array(E.memory.buffer, addr, 1);
  const bufF64 = (addr)       => new Float64Array(E.memory.buffer, addr, 1);

  // ABI regression guards.
  for (const n of ['sim_new','sim_free','sim_step','sim_tick','sim_snapshot_json',
    'sim_free_json','sim_save_json','sim_restore_json']) {
    check(`export sim_*:${n}`, typeof E[n] === 'function');
  }
  check('scratch_size matches layout', E.sim_scratch_size() === SCRATCH_SIZE,
    `${E.sim_scratch_size()} vs expected ${SCRATCH_SIZE}`);

  const SEED = 20261102;
  const SAVE_AT = 100;
  const TOTAL = 200;

  const simA = E.sim_new(SEED);
  const scratchA = E.sim_scratch(simA);

  // ── 1. Advance simA to SAVE_AT ──────────────────────────────────────────
  for (let t = 1; t <= SAVE_AT; t++) {
    bufF64(scratchA + OFF_DAYLIGHT)[0] = daylight(t);
    E.sim_step(simA, 1, scratchA + OFF_DAYLIGHT, 1);
  }
  check(`simA tick == ${SAVE_AT}`, E.sim_tick(simA) === SAVE_AT, `${E.sim_tick(simA)}`);

  // ── 2. Save state (sim_save_json) ────────────────────────────────────────
  let ptrSave = 0, jsonLen = 0, saveJson = '';
  try {
    bufU32(scratchA + OFF_LEN)[0] = 0;
    ptrSave = E.sim_save_json(simA, scratchA + OFF_LEN);
    jsonLen = bufU32(scratchA + OFF_LEN)[0] - 1; // out_len includes NUL terminator
    check('sim_save_json returned non-null ptr', ptrSave !== 0);
    check('json length plausible', jsonLen > 10_000 && jsonLen < 8_000_000,
      `${(jsonLen / 1024).toFixed(1)} KB`);
    saveJson = new TextDecoder().decode(bufU8(ptrSave, jsonLen));
  } catch (e) {
    check('sim_save_json', false, e.message);
    E.sim_free(simA);
    return;
  }
  const saveSizeKB = (saveJson.length / 1024).toFixed(1);
  E.sim_free_json(ptrSave, jsonLen);

  // Structural sanity checks on the saved document.
  try {
    const s = JSON.parse(saveJson);
    check('save JSON parses', true);
    // Phase 7 bumped SaveState to v2 (DecoEffChoiceMode); Phase 8/15 extended
    // it without a version bump (serde defaults). Accept v2+.
    check('Version>=2', s.Version >= 2, `${s.Version}`);
    check('SchemaName correct', s.SchemaName === 'aether-sim-save-state');
    check('TickCount==SAVE_AT', s.TickCount === SAVE_AT, `${s.TickCount}`);
    check('Agents present', Array.isArray(s.Agents) && s.Agents.length > 0,
      `${s.Agents?.length} agents`);
    check('Grass grid 76×76=5776', Array.isArray(s.Grass) && s.Grass.length === 76 * 76,
      `${s.Grass?.length ?? '?'}`);
    check('GrassBySpecies 3 layers', Array.isArray(s.GrassBySpecies) && s.GrassBySpecies.length === 3);
    check('MineralField 76×76', Array.isArray(s.MineralField) && s.MineralField.length === 76 * 76);
    check('Carcasses array present', Array.isArray(s.Carcasses));
    check('IsolatedGroups array present', Array.isArray(s.IsolatedGroups));
check('RNG states numeric',
      typeof s.RngState?.State === 'number' && typeof s.PrngState?.State === 'number'
      && typeof s.FishRngState?.State === 'number');
  } catch (e) {
    check('save structural', false, e.message);
  }

  // ── 3. Restore ───────────────────────────────────────────────────────────
  // NOTE: exact-length bytes only — serde_json::from_slice errors on any
  // trailing NUL, so do NOT append a terminator here.
  const enc = new TextEncoder().encode(saveJson);
  const growPages = Math.ceil(enc.length / 65536);
  const prevPages = E.memory.grow(growPages);
  const jsonAddr = prevPages * 65536;
  bufU8(jsonAddr, enc.length).set(enc);
  let simB = 0;
  try {
    simB = E.sim_restore_json(jsonAddr, enc.length);
  } catch (e) {
    check('sim_restore_json call', false, e.message);
    E.sim_free(simA);
    return;
  }
  check('restore returned non-null handle', simB !== 0);
  if (!simB) {
    console.error('\nFATAL: restore returned null — aborting.');
    E.sim_free(simA);
    process.exit(1);
  }
  check('original sim unchanged (still tick 100)', E.sim_tick(simA) === SAVE_AT);

  // ── 4. Advance both sims to TOTAL ───────────────────────────────────────
  for (let t = SAVE_AT + 1; t <= TOTAL; t++) {
    bufF64(scratchA + OFF_DAYLIGHT)[0] = daylight(t);
    E.sim_step(simA, 1, scratchA + OFF_DAYLIGHT, 1);
    E.sim_step(simB, 1, scratchA + OFF_DAYLIGHT, 1);
  }
  check(`simA tick == ${TOTAL}`, E.sim_tick(simA) === TOTAL, `${E.sim_tick(simA)}`);
  check(`simB tick == ${TOTAL}`, E.sim_tick(simB) === TOTAL, `${E.sim_tick(simB)}`);

  // ── 5. Compare rendered snapshots ────────────────────────────────────────
  const snapOf = (sim) => {
    bufU32(scratchA + OFF_LEN)[0] = 0;
    const p = E.sim_snapshot_json(sim, scratchA + OFF_LEN);
    const n = bufU32(scratchA + OFF_LEN)[0] - 1; // exclude NUL terminator
    const text = new TextDecoder().decode(bufU8(p, n));
    E.sim_free_json(p, n);
    return parseSnapshot(text, 3);
  };
  let snapA, snapB;
  try {
    snapA = snapOf(simA);
    snapB = snapOf(simB);
    check('snapA Meta.Version===3', snapA.Meta.Version === 3);
    check('snapB Meta.Version===3', snapB.Meta.Version === 3);
  } catch (e) {
    check('snapshot parse', false, e.message);
    E.sim_free(simA); E.sim_free(simB);
    return;
  }

  const ka = [snapA.Stats.HerbCount, snapA.Stats.PredCount, snapA.Stats.ScavCount,
              snapA.Stats.OmnCount, snapA.Stats.FishCount, snapA.Stats.DecoCount];
  const kb = [snapB.Stats.HerbCount, snapB.Stats.PredCount, snapB.Stats.ScavCount,
              snapB.Stats.OmnCount, snapB.Stats.FishCount, snapB.Stats.DecoCount];
  ['herb','pred','scav','omn','fish','deco'].forEach((label, i) =>
    check(`census ${label} equal`, ka[i] === kb[i], `${ka[i]} vs ${kb[i]}`));
  check('total_agents equal',
    snapA.Stats.TotalAgents === snapB.Stats.TotalAgents,
    `${snapA.Stats.TotalAgents} vs ${snapB.Stats.TotalAgents}`);
  check('grass_total close (<0.1%)',
    Math.abs(snapA.Stats.PlantTotal - snapB.Stats.PlantTotal) / (snapA.Stats.PlantTotal || 1) < 0.001,
    `${snapA.Stats.PlantTotal.toFixed(4)} vs ${snapB.Stats.PlantTotal.toFixed(4)}`);

  E.sim_free(simA); E.sim_free(simB);

  console.log('\nSummary:');
  console.log(`  save JSON size   : ${saveSizeKB} KB (t=${SAVE_AT}, schema=aether-sim-save-state v1)`);
  console.log(`  census A @ t=200 : H=${ka[0]} P=${ka[1]} S=${ka[2]} O=${ka[3]} F=${ka[4]} D=${ka[5]}`);
  console.log(`  census B @ t=200 : H=${kb[0]} P=${kb[1]} S=${kb[2]} O=${kb[3]} F=${kb[4]} D=${kb[5]}`);

  if (failures === 0) console.log('\nAll checks passed.');
  else console.log(`\n${failures} check(s) failed.`);
  process.exit(failures > 0 ? 1 : 0);
}

main().catch(e => { console.error(e); process.exit(2); });
