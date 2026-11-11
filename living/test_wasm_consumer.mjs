// test_wasm_consumer.mjs — roundtrip test for the RS → JS Snapshot protocol.
//
// Runs `ecology-sim --wasm-test <seed> <ticks>` natively, captures the JSON
// packet from stdout, parses it with the same code path the future Three.js
// viewer will use (living/src/renderer-wasm-viewer.mjs), and asserts:
//   1. Meta.Version === 2 (v1→v2: +FishCount)
//   2. Agent count in Agents[] matches StatsAgg.TotalAgents
//   3. Per-kind counts (herb/pred/scav/omn/fish) match StatsAgg
//   4. Grass total is finite and non-negative
//
// Usage: node living/test_wasm_consumer.mjs [--seed 20261102] [--ticks 10]

import { spawnSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseSnapshot } from './src/renderer-wasm-viewer.mjs';

const KINDS = ['herb', 'pred', 'scav', 'omn', 'fish', 'deco'];
const EXPECTED_VERSION = 3;

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const args = process.argv.slice(2);
const seedArg = args.indexOf('--seed');
const ticksArg = args.indexOf('--ticks');
const seed = seedArg >= 0 ? args[seedArg + 1] : '20261102';
const ticks = ticksArg >= 0 ? args[ticksArg + 1] : '10';

const simPath = path.join(__dirname, '..', 'core', 'target', 'debug', 'ecology-sim');
const proc = spawnSync(simPath, ['--seed', seed, '--ticks', ticks, '--wasm-test'], {
  encoding: 'utf8',
  stdio: ['ignore', 'pipe', 'pipe'],
});

if (proc.error) {
  console.error(`FAIL: cannot spawn ${simPath}: ${proc.error.message}`);
  console.error('Build first: cd world/core && cargo build --bin ecology-sim');
  process.exit(1);
}
if (proc.status !== 0) {
  console.error(`FAIL: ecology-sim exited ${proc.status}`);
  console.error(proc.stderr);
  process.exit(1);
}

let snap;
try {
  snap = parseSnapshot(proc.stdout, EXPECTED_VERSION);
} catch (e) {
  console.error(`FAIL: parseSnapshot threw: ${e.message}`);
  process.exit(1);
}

const { Meta, World, Agents, Stats } = snap;
let failures = 0;
const check = (name, cond, detail = '') => {
  if (cond) {
    console.log(`  ok   ${name}${detail ? ` — ${detail}` : ''}`);
  } else {
    console.error(`  FAIL ${name}${detail ? ` — ${detail}` : ''}`);
    failures += 1;
  }
};

console.log(`snapshot seed=${seed} ticks=${ticks}:`);
check(`Meta.Version === ${EXPECTED_VERSION}`, Meta.Version === EXPECTED_VERSION, `got ${Meta.Version}`);
check('World.GridN === 76', World.GridN === 76, `got ${World.GridN}`);
check(
  'Agents.length === Stats.TotalAgents',
  Agents.length === Stats.TotalAgents,
  `agents=${Agents.length} stats=${Stats.TotalAgents}`,
);
check(
  'Agents.length === HerbCount + PredCount + ScavCount + OmnCount + FishCount + DecoCount',
  Agents.length === Stats.HerbCount + Stats.PredCount + Stats.ScavCount + Stats.OmnCount + Stats.FishCount + (Stats.DecoCount ?? 0),
);

// Per-kind census straight from the Agents slice.
const census = Object.fromEntries(KINDS.map((k) => [k, 0]));
for (const a of Agents) {
  if (!(a.Kind in census)) {
    check(`agent Kind is one of ${KINDS.join('/')}`, false, `got ${JSON.stringify(a.Kind)}`);
    failures += 1;
    continue;
  }
  census[a.Kind] += 1;
}
check('herb census === Stats.HerbCount', census.herb === Stats.HerbCount, `${census.herb} vs ${Stats.HerbCount}`);
check('pred census === Stats.PredCount', census.pred === Stats.PredCount, `${census.pred} vs ${Stats.PredCount}`);
check('scav census === Stats.ScavCount', census.scav === Stats.ScavCount, `${census.scav} vs ${Stats.ScavCount}`);
check('omn census === Stats.OmnCount', census.omn === Stats.OmnCount, `${census.omn} vs ${Stats.OmnCount}`);
check('fish census === Stats.FishCount', census.fish === Stats.FishCount, `${census.fish} vs ${Stats.FishCount}`);
check('deco census === Stats.DecoCount', census.deco === (Stats.DecoCount ?? 0), `${census.deco} vs ${Stats.DecoCount}`);

// Mineral field (v3+): full + resampled sanity checks.
if (World.MineralFieldResampled != null) {
  check('World.MineralFieldResampled.length === 1444 (38²)',
    World.MineralFieldResampled.length === 1444, `got ${World.MineralFieldResampled.length}`);
  const resampledSum = World.MineralFieldResampled.reduce((a, v) => a + v, 0);
  check('mineralResampledSum >= 0 and finite',
    Number.isFinite(resampledSum) && resampledSum >= 0, `sum=${resampledSum.toFixed(4)}`);
}
if (World.MineralField != null) {
  check('World.MineralField.length === 5776 (76²)',
    World.MineralField.length === 5776, `got ${World.MineralField.length}`);
}

// Grass sanity: finite, non-negative, correct length.
const grassTotal = World.Grass.reduce((acc, v) => acc + v, 0);
check('World.Grass.length === GridN*GridN', World.Grass.length === World.GridN * World.GridN, `${World.Grass.length}`);
check('grassTotal is finite and >= 0', Number.isFinite(grassTotal) && grassTotal >= 0, `sum=${grassTotal.toFixed(2)}`);

// Agent ids are unique and ascending (deterministic export order).
let idsAscending = true;
for (let i = 1; i < Agents.length; i++) {
  if (Agents[i].Id <= Agents[i - 1].Id) { idsAscending = false; break; }
}
check('agent Ids unique + ascending', idsAscending);

// Sample output for the report.
const firstTwo = Agents.slice(0, 2).map((a) => ({
  Id: a.Id, Kind: a.Kind, X: +a.X.toFixed(2), Z: +a.Z.toFixed(2),
  Energy: +a.Energy.toFixed(2), State: a.State, Hunger: +a.Hunger.toFixed(3),
}));

if (failures > 0) {
  console.error(`\nROUNDTRIP FAILED: ${failures} check(s) failed`);
  process.exit(1);
}
console.log('\nROUNDTRIP OK');
console.log('first 2 agents:', JSON.stringify(firstTwo, null, 2));
console.log('grass total:', grassTotal.toFixed(2));
