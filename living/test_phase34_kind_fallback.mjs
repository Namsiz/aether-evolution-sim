// Phase 34 Node smoke: derived-kind names flow through the C-ABI and the
// wasm-viewer kind mapping has an unknown-byte fallback.
//
//   1. Seed a DECO divergence, run until the DECO kind forks (a derived
//      KindDef is registered in the save's Kinds table).
//   2. Rewrite one DECO agent's Kind to the derived name, restore, and drive
//      a frame: the JSON snapshot must report the derived kind NAME string
//      (RenderPacket contract) and the flat ABI must write the b'?' byte.
//   3. Static check: wasm-viewer.mjs maps unknown kind bytes to a default
//      species instead of dropping the agent.
//
// Usage: node test_phase34_kind_fallback.mjs [wasm-path]

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

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
const AGENT_CAP = 1024, DAYLIGHT_CAP = 64;
const AGENT_BASE = 38 * 38 * 4;
const AGENT_SLOT_SIZE = 48;
const SLOT_KIND = 8;
const OFF_DAYLIGHT = AGENT_BASE + AGENT_CAP * AGENT_SLOT_SIZE;
const ANCESTORS = new Set(['herb', 'pred', 'scav', 'omn', 'fish', 'deco']);

async function main() {
  const bytes = fs.readFileSync(wasmPath);
  const mod = await WebAssembly.compile(bytes);
  const E = (await WebAssembly.instantiate(mod, {})).exports;

  const sim = E.sim_new(SEED);
  const stepRange = (handle, from, to) => {
    const ptr = E.sim_scratch(handle);
    for (let t = from; t < to;) {
      const n = Math.min(DAYLIGHT_CAP, to - t);
      const dv = new DataView(E.memory.buffer);
      for (let i = 0; i < n; i++) dv.setFloat64(ptr + OFF_DAYLIGHT + i * 8, daylight(t + i), true);
      E.sim_step(handle, n, ptr + OFF_DAYLIGHT, n);
      t += n;
    }
  };
  const readSave = (handle) => {
    const jsonPtr = E.sim_save_json(handle, 0);
    const u8 = new Uint8Array(E.memory.buffer, jsonPtr);
    let len = 0;
    while (u8[len] !== 0) len++;
    const out = JSON.parse(new TextDecoder().decode(u8.slice(0, len)));
    E.sim_free_json(jsonPtr, len + 1);
    return out;
  };
  const writeSave = (save) => {
    const enc = new TextEncoder().encode(JSON.stringify(save));
    const needPages = Math.ceil(enc.length / 65536) + 1;
    const prevPages = E.memory.buffer.byteLength / 65536;
    E.memory.grow(needPages);
    new Uint8Array(E.memory.buffer, prevPages * 65536, enc.length).set(enc);
    return E.sim_restore_json(prevPages * 65536, enc.length);
  };

  console.log('── warmup + seeded DECO divergence ──');
  stepRange(sim, 0, 300);
  let save = readSave(sim);

  // Seed two DECO clusters at the farthest-apart seeded positions with a wide
  // efficiency gap (the same fixture the Phase 15/34 tests use).
  const decos = save.Agents.filter((a) => a.Kind === 'deco');
  check('warmup keeps ≥ 15 deco agents', decos.length >= 15, String(decos.length));
  let bestD = 0, best = [decos[0], decos[1]];
  for (let i = 0; i < decos.length; i++) {
    for (let j = i + 1; j < decos.length; j++) {
      const d = Math.hypot(decos[i].X - decos[j].X, decos[i].Z - decos[j].Z);
      if (d > bestD) { bestD = d; best = [decos[i], decos[j]]; }
    }
  }
  check('seeded deco spread > 400', bestD > 400, `${bestD.toFixed(0)} units`);
  const J = [[0, 0], [6, 2], [-5, 5], [3, -6], [-7, -3], [8, 7], [-2, 8], [5, -8]];
  let k = 0;
  for (const a of save.Agents) {
    if (a.Kind !== 'deco') continue;
    const first = k < 8;
    const anchor = best[first ? 0 : 1];
    const j = J[k % J.length];
    a.X = anchor.X + j[0]; a.Z = anchor.Z + j[1];
    a.PrevX = a.X; a.PrevZ = a.Z; a.HomeX = anchor.X; a.HomeZ = anchor.Z;
    a.Genome.DecoEfficiency = first ? 0.55 : 1.45;
    k += 1;
  }
  for (const anchor of best) {
    save.Carcasses.push({ X: anchor.X + 4, Z: anchor.Z, Energy: 30, Age: 0, OccupiedBy: null });
  }
  save.DecoEffChoiceMode = 0;
  const sim2 = writeSave(save);
  check('restore accepted the seeded save', sim2 !== 0);

  console.log('── run to the DECO fork ──');
  stepRange(sim2, 300, 900);
  save = readSave(sim2);
  const derivedDefs = (save.Kinds || []).filter((d) => d.ParentKind !== null && d.ParentKind !== undefined);
  check('a derived KindDef was registered (Kinds table)', derivedDefs.length >= 1,
    derivedDefs.map((d) => d.Name).join(','));
  const derivedName = derivedDefs.length ? derivedDefs[derivedDefs.length - 1].Name : null;
  check('derived kind has a deterministic parent-name-derived label',
    !!derivedName && derivedName.startsWith('deco-'), String(derivedName));

  console.log('── derived-kind agent through the ABI ──');
  // Rewrite one DECO agent to carry the derived kind (the same data path a
  // birth into the forked clade takes), then restore and drive one frame.
  const target = save.Agents.find((a) => a.Kind === 'deco');
  target.Kind = derivedName;
  const sim3 = writeSave(save);
  check('restore accepted the derived-kind agent', sim3 !== 0);

  stepRange(sim3, 900, 902);
  // JSON snapshot (RenderPacket): the agent's kind must be the derived NAME.
  const jsonPtr = E.sim_snapshot_json(sim3, 0);
  const u8 = new Uint8Array(E.memory.buffer, jsonPtr);
  let len = 0;
  while (u8[len] !== 0) len++;
  const snap = JSON.parse(new TextDecoder().decode(u8.slice(0, len)));
  E.sim_free_json(jsonPtr, len + 1);
  const kindsInSnap = new Set(snap.Agents.map((a) => a.Kind ?? a.kind));
  check('snapshot JSON carries the derived kind NAME', kindsInSnap.has(derivedName),
    [...kindsInSnap].join(','));
  check('the six ancestor names are unchanged', [...ANCESTORS].every((n) => !kindsInSnap.has(`${n}-`)));

  // Flat ABI: derived-kind agents are written as the b'?' unknown byte.
  const nFlat = E.sim_agents_flat(sim3, AGENT_CAP);
  const scratchPtr = E.sim_scratch(sim3);
  const dv = new DataView(E.memory.buffer);
  let unknownByte = 0;
  for (let i = 0; i < nFlat; i++) {
    if (dv.getUint8(scratchPtr + AGENT_BASE + i * AGENT_SLOT_SIZE + SLOT_KIND) === 0x3F) unknownByte += 1;
  }
  check('flat ABI writes the b\'?\' byte for derived-kind agents', unknownByte > 0, String(unknownByte));

  // Static fallback check: the viewer maps unknown bytes to a default species.
  const viewerSrc = fs.readFileSync(path.join(__dirname, 'src', 'wasm-viewer.mjs'), 'utf8');
  check('wasm-viewer defines KIND_BYTE_FALLBACK', /KIND_BYTE_FALLBACK\s*=/.test(viewerSrc));
  const uses = (viewerSrc.match(/\?\?\s*KIND_BYTE_FALLBACK/g) || []).length;
  check('both kind-byte readers fall back instead of dropping the agent', uses >= 2, `${uses} sites`);

  E.sim_free(sim);
  if (sim2) E.sim_free(sim2);
  if (sim3) E.sim_free(sim3);
  console.log(failures === 0 ? '\nPHASE 34 KIND-FALLBACK SMOKE OK' : `\n${failures} CHECK(S) FAILED`);
  process.exit(failures === 0 ? 0 : 1);
}

main().catch((e) => { console.error(e); process.exit(1); });
