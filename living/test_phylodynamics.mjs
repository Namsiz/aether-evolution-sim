// test_phylodynamics.mjs — Phase 15 phylodynamics smoke test (Node, C-ABI only).
//
// Exercises the full wasm pipeline the browser viewer uses:
//   1. sim_new(20261102) → warm up 300 ticks (sinusoidal daylight).
//   2. sim_save_json → mutate the DECO seed into two far-apart clusters with a
//      0.9-wide deco_efficiency gap (mirrors the Rust species_diversification
//      fixture) + inject carcasses + legacy mode → sim_restore_json.
//   3. Run to 3000 ticks; every 60 ticks pull clades/events via the Phase 15
//      C-ABI exports (sim_clade_count / sim_get_clade / sim_clade_name /
//      sim_event_count / sim_get_event).
//   4. Assert: clade count ≥ 2 (a split happened), ≥ 1 speciation event,
//      at least one clade name ≠ the root kind ("deco-A"-style), and the
//      phylogeny graph schema (version + nodes + edges) is well-formed.
//   5. Telemetry (no assert): a NATURAL run (unmodified seed) logs whether
//      spontaneous speciation fires under plain selection pressure — data for
//      the Chief's threshold tuning decision.
//
// Usage: node living/test_phylodynamics.mjs [--wasm path]

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { readPhylogenyState, buildPhylogenyGraph, CLADE_CAP, CLADE_RECORD_SIZE, EVENT_CAP, EVENT_RECORD_SIZE, CLADE_NAME_CAP, PHYLOGENY_VERSION } from './src/renderer-phylogeny.mjs';

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

// Scratch layout mirrors (must match wasm.rs / wasm-viewer.mjs).
const AGENT_CAP = 1024, DAYLIGHT_CAP = 64;
const AGENT_BASE = 38 * 38 * 4;
const OFF_DAYLIGHT = AGENT_BASE + AGENT_CAP * 48;
const OFF_COUNTS = OFF_DAYLIGHT + DAYLIGHT_CAP * 8;
const OFF_HIST = OFF_COUNTS + 6 * 4;
const OFF_CLADES = OFF_HIST + 64 * 4;
const OFF_EVENTS = OFF_CLADES + CLADE_CAP * CLADE_RECORD_SIZE;
const OFF_CLADE_NAME = OFF_EVENTS + EVENT_CAP * EVENT_RECORD_SIZE;
const EXPECTED_SCRATCH = OFF_CLADE_NAME + CLADE_NAME_CAP;

async function main() {
  const bytes = fs.readFileSync(wasmPath);
  const mod = await WebAssembly.compile(bytes);
  const instance = await WebAssembly.instantiate(mod, {});
  const E = instance.exports;

  console.log('── ABI guards ──');
  for (const n of ['sim_clade_count', 'sim_get_clade', 'sim_clade_name', 'sim_event_count', 'sim_get_event']) {
    check(`export ${n}`, typeof E[n] === 'function');
  }
  check('scratch layout includes phylogeny region', E.sim_scratch_size() === EXPECTED_SCRATCH,
    `${E.sim_scratch_size()} vs ${EXPECTED_SCRATCH}`);

  const sim = E.sim_new(SEED);
  const scratchPtr = E.sim_scratch(sim);

  const step = (handle, ptr, from, to) => {
    for (let t = from + 1; t <= to;) {
      const n = Math.min(DAYLIGHT_CAP, to - t + 1);
      const dv = new DataView(E.memory.buffer);
      for (let i = 0; i < n; i++) dv.setFloat64(ptr + OFF_DAYLIGHT + i * 8, daylight(t + i - 1), true);
      E.sim_step(handle, n, ptr + OFF_DAYLIGHT, n);
      t += n;
    }
  };

  console.log('── warmup 300 ticks ──');
  step(sim, scratchPtr, 0, 300);

  // Seeded-divergence fixture: split the 15 DECOs into two clusters (eff 0.55
  // / 1.45) around the two farthest-apart seeded positions, legacy mode.
  // sim_save_json with a null out_len still NUL-terminates the payload.
  const jsonPtr = E.sim_save_json(sim, 0);
  const saveBytes = (() => {
    const u8 = new Uint8Array(E.memory.buffer, jsonPtr);
    let len = 0;
    while (u8[len] !== 0) len++;
    return u8.slice(0, len);
  })();
  E.sim_free_json(jsonPtr, saveBytes.length + 1);
  const saveState = JSON.parse(new TextDecoder().decode(saveBytes));

  const decos = saveState.Agents.filter((a) => a.Kind === 'deco');
  check('warmup keeps ≥ 15 deco agents alive', decos.length >= 15, String(decos.length));
  let bestD = 0, bestPair = [decos[0], decos[1]];
  for (let i = 0; i < decos.length; i++) {
    for (let j = i + 1; j < decos.length; j++) {
      const d = Math.hypot(decos[i].X - decos[j].X, decos[i].Z - decos[j].Z);
      if (d > bestD) { bestD = d; bestPair = [decos[i], decos[j]]; }
    }
  }
  check('seeded deco spread > 400', bestD > 400, `${bestD.toFixed(0)} units`);
  const anchors = [bestPair[0], bestPair[1]].map((a) => [a.X, a.Z]);
  const JITTER = [[0, 0], [6, 2], [-5, 5], [3, -6], [-7, -3], [8, 7], [-2, 8], [5, -8]];
  let k = 0;
  for (const a of saveState.Agents) {
    if (a.Kind !== 'deco') continue;
    const firstHalf = k < 8;
    const anchor = anchors[firstHalf ? 0 : 1];
    const j = JITTER[k % JITTER.length];
    a.X = anchor[0] + j[0]; a.Z = anchor[1] + j[1];
    a.PrevX = a.X; a.PrevZ = a.Z;
    a.HomeX = anchor[0]; a.HomeZ = anchor[1];
    a.Genome.DecoEfficiency = firstHalf ? 0.55 : 1.45;
    k += 1;
  }
  for (const [ax, az] of anchors) {
    saveState.Carcasses.push({ X: ax + 4, Z: az, Energy: 30, Age: 0, OccupiedBy: null });
    saveState.Carcasses.push({ X: ax - 4, Z: az, Energy: 30, Age: 0, OccupiedBy: null });
  }
  saveState.DecoEffChoiceMode = 0; // legacy: efficiency is fitness-neutral

  const modifiedJson = JSON.stringify(saveState);
  const enc = new TextEncoder().encode(modifiedJson);
  // Write the payload into a fresh wasm-memory page region.
  const needPages = Math.ceil(enc.length / 65536) + 1;
  const prevPages = E.memory.buffer.byteLength / 65536;
  E.memory.grow(needPages);
  new Uint8Array(E.memory.buffer, prevPages * 65536, enc.length).set(enc);
  const restoredPtr = E.sim_restore_json(prevPages * 65536, enc.length);
  check('sim_restore_json accepts modified v2 save', restoredPtr !== 0);
  E.sim_free(sim);
  const sim2 = restoredPtr;
  const sp2 = E.sim_scratch(sim2);

  console.log('── phylodynamic run: 300 → 3000 ticks ──');
  const samples = [];
  const pull = (tick) => {
    const st = readPhylogenyState(E, sim2, sp2 + OFF_CLADES, sp2 + OFF_EVENTS, sp2 + OFF_CLADE_NAME, CLADE_NAME_CAP);
    samples.push({ tick, clades: st.clades, events: st.events });
    return st;
  };
  let last = 0;
  for (let t = 301; t <= 3000; t += 60) {
    step(sim2, sp2, last, t);
    last = t;
    pull(t);
  }

  const finalState = samples[samples.length - 1];
  const cladeCounts = samples.map((s) => s.clades.length);
  const eventCounts = samples.map((s) => s.events.length);
  console.log('  clade count timeline:', cladeCounts.filter((_, i) => i % 5 === 0).join(' '));
  console.log('  final clades:', finalState.clades.map((c) =>
    `${c.name}${c.extinct ? `†(t=${c.deathTick})` : ''}`).join(' · '));
  console.log('  final events:', finalState.events.map((e) =>
    `t=${e.tick} parent=${e.parentCladeId} kids=[${e.childCladeIds}] d=${e.geneticDistance.toFixed(2)}`).join(' ; '));

  console.log('── assertions ──');
  check('clade count ≥ 2 (speciation fired under seeded divergence)', Math.max(...cladeCounts) >= 2,
    `peak ${Math.max(...cladeCounts)}`);
  check('≥ 1 speciation event recorded', Math.max(...eventCounts) >= 1, `${Math.max(...eventCounts)} events`);
  const childNames = finalState.clades.filter((c) => c.name !== 'deco').map((c) => c.name);
  check('a clade name differs from the root kind', childNames.length >= 1, childNames.join(','));
  check('child names follow the deco-X convention', childNames.every((n) => n.startsWith('deco-')),
    childNames.join(','));
  check('root clade present with parent=null', finalState.clades.some((c) => c.name === 'deco' && c.parentId === null));

  // Phylogeny graph schema (the exact object the overlay consumes).
  const graph = buildPhylogenyGraph(finalState.clades, finalState.events, 3000, {});
  check('graph version field', graph.version === PHYLOGENY_VERSION, String(graph.version));
  check('graph.nodes is an array with one entry per clade', Array.isArray(graph.nodes) && graph.nodes.length === finalState.clades.length);
  check('graph.edges is an array', Array.isArray(graph.edges));
  check('edges form a tree (n-1 edges)', graph.edges.length === Math.max(0, graph.nodes.length - 1),
    `${graph.edges.length} edges / ${graph.nodes.length} nodes`);
  const ids = new Set(graph.nodes.map((n) => n.id));
  check('edge endpoints resolve to nodes', graph.edges.every((e) => ids.has(e.from) && ids.has(e.to)));
  check('node schema complete', graph.nodes.every((n) =>
    typeof n.id === 'number' && typeof n.name === 'string' && typeof n.birthTick === 'number'
    && Number.isFinite(n.x) && Number.isFinite(n.y) && n.x >= 0 && n.x <= 1 && n.y >= 0 && n.y <= 1));
  check('event dots carried into graph', graph.events.length === finalState.events.length);

  console.log('── natural-run telemetry (no assert; Chief decision data) ──');
  const simN = E.sim_new(SEED);
  const spN = E.sim_scratch(simN);
  let naturalPeak = 0;
  let naturalEvents = 0;
  let naturalSamples = [];
  for (let t = 61; t <= 3000; t += 60) {
    step(simN, spN, t - 60, t);
    const st = readPhylogenyState(E, simN, spN + OFF_CLADES, spN + OFF_EVENTS, spN + OFF_CLADE_NAME, CLADE_NAME_CAP);
    naturalPeak = Math.max(naturalPeak, st.clades.length);
    naturalEvents = st.events.length;
    naturalSamples.push(`t=${t}:${st.clades.length}/${st.events.length}`);
  }
  console.log('  natural clades/events every 300:', naturalSamples.filter((_, i) => i % 5 === 0).join(' '));
  console.log(`  natural peak clades=${naturalPeak} events=${naturalEvents}`);
  check('natural run stays alive with a root clade', naturalPeak >= 1);
  E.sim_free(simN);
  E.sim_free(sim2);

  console.log(failures === 0 ? '\nALL CHECKS PASSED' : `\n${failures} CHECK(S) FAILED`);
  process.exit(failures === 0 ? 0 : 1);
}

main().catch((e) => { console.error('FAIL: crashed:', e); process.exit(1); });
