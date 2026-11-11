#!/usr/bin/env node
// dump_state_diff.mjs — JS-side STATE_DUMP generator + JS/RS first-fork locator.
//
// Usage:
//   node scripts/dump_state_diff.mjs [--seed S] [--ticks N]
//       Runs the JS oracle, writes /tmp/state_dump_js_<seed>.tsv, runs the Rust
//       binary (cargo run -p aether-sim-ecology -- --dump-state 1) for the same
//       window, then reports the first diverging agent/field.
//   node scripts/dump_state_diff.mjs --js-dump-only <path> [--seed S] [--ticks N]
//       Only writes the JS STATE_DUMP file (used by diff_rs_vs_js.sh --bisect).

import { Ecology } from '../../living/src/ecology.js';
import { execFileSync } from 'node:child_process';
import { writeFileSync, readFileSync, existsSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const argv = process.argv.slice(2);
function argVal(name, dflt) {
  const i = argv.indexOf(name);
  return i >= 0 && argv[i + 1] !== undefined ? argv[i + 1] : dflt;
}
function argStr(name) { return argv.includes(name); }

const SEED = parseInt(argVal('--seed', '20261102'), 10);
const TICKS = parseInt(argVal('--ticks', '60'), 10);
const JS_OUT = argVal('--js-out', join(tmpdir(), `state_dump_js_${SEED}.tsv`));
const RS_OUT = argVal('--rs-out', join(tmpdir(), `state_dump_rs_${SEED}.tsv`));
const DUMP_ONLY_PATH = argVal('--js-dump-only', null);
const GRASS_DUMP_PATH = argVal('--dump-grass', null);
const GRASS_TICKS = (argVal('--grass-ticks', '0,40,80,120,160,200,240,280,300') || '')
  .split(',').map((s) => parseInt(s, 10)).filter(Number.isFinite);
const IGNORE = (argVal('--ignore', '') || '')
  .split(',').map((s) => s.trim()).filter(Boolean);

// Format like JS String(v) (shortest round-trip; Infinity/NaN spelled out).
function fmt(v) {
  if (!Number.isFinite(v)) return String(v); // "NaN" / "Infinity" / "-Infinity"
  return String(v); // JS already drops trailing .0 and -0 prints as 0
}

// Per-agent partial of ecology.checksum(): FNV-1a 64 over
// [kindCode, id, floor(x*100), floor(z*100), floor(energy*100), floor(age*10)],
// low 32 bits of each value, matching BigInt(v) & 0xffffffffn semantics.
const FNV_MOD = 0x100000001b3n;
function checksumLocal(a) {
  let h = 14695981039346656033n;
  const mix = (v) => {
    const f = Number.isFinite(v) ? Math.floor(v) : 0;
    h ^= BigInt(f) & 0xffffffffn;
    h = (h * FNV_MOD) & 0xffffffffffffffffn;
  };
  mix(a.kind.charCodeAt(0));
  mix(a.id);
  mix(a.x * 100);
  mix(a.z * 100);
  mix(a.energy * 100);
  mix(a.age * 10);
  return h.toString(16).padStart(8, '0');
}

// Full-grid f32 bit-hex dump, mirroring the RS binary's --dump-grass format.
function grassHex(arr) {
  const u32 = new Uint32Array(arr.buffer, arr.byteOffset, arr.length);
  let s = '';
  for (let i = 0; i < u32.length; i++) s += u32[i].toString(16).padStart(8, '0');
  return s;
}

function grassDumpLines(eco, tick) {
  const lines = [];
  const emit = (name, arr) => lines.push(`GRASS_ARR t=${tick} arr=${name} ${grassHex(arr)}`);
  emit('moist', eco.moist);
  emit('suit', eco.suit);
  emit('shade', eco.shade);
  for (let sp = 0; sp < 3; sp++) {
    emit(`suit_eff_by_${sp}`, eco.suitEffBy[sp]);
    emit(`grass_p_${sp}`, eco.grassP[sp]);
  }
  emit('grass', eco.grass);
  return lines;
}

function runJsGrassDump(eco, ticks) {
  const lines = [];
  if (ticks.includes(0)) lines.push(...grassDumpLines(eco, 0));
  for (let i = 1; i <= TICKS; i++) {
    const daylight = 0.5 + 0.5 * Math.sin((i / 300) * Math.PI);
    eco.update(0.1, daylight);
    if (ticks.includes(i)) lines.push(...grassDumpLines(eco, i));
  }
  return lines.join('\n') + '\n';
}

function genomeToken(g) {
  const parts = [
    `speed=${fmt(g.speed)}`, `sense=${fmt(g.sense)}`, `size=${fmt(g.size)}`,
    `metabolism=${fmt(g.metabolism)}`, `lifespan=${fmt(g.lifespan)}`, `hue=${fmt(g.hue)}`,
    `bold=${fmt(g.bold)}`, `curious=${fmt(g.curious)}`, `social=${fmt(g.social)}`,
    `helpThr=${g.helpThr !== undefined ? fmt(g.helpThr) : 'undefined'}`,
  ];
  if (g.prefSig !== undefined) parts.push(`prefSig=${fmt(g.prefSig)}`);
  if (g.tolSig !== undefined) parts.push(`tolSig=${fmt(g.tolSig)}`);
  return parts.join(',');
}

function dumpAgent(tick, a) {
  return [
    `STATE_DUMP t=${tick}`,
    `kind=${a.kind}`,
    `id=${a.id}`,
    `x=${fmt(a.x)}`, `z=${fmt(a.z)}`,
    `energy=${fmt(a.energy)}`, `age=${fmt(a.age)}`,
    `bite=${fmt(a.bite)}`, `repro=${fmt(a.repro)}`,
    `state=${a.state}`, `heading=${fmt(a.heading)}`,
    `lineage=${a.lineage !== undefined && a.lineage !== null ? a.lineage : 'undefined'}`,
    `genome={${genomeToken(a.genome)}}`,
    `checksum_local=${checksumLocal(a)}`,
  ].join(' ');
}

function runJsDump() {
  const eco = new Ecology(SEED);
  const lines = [];
  for (const a of eco.agents) if (a.alive) lines.push(dumpAgent(0, a));
  for (let i = 1; i <= TICKS; i++) {
    const daylight = 0.5 + 0.5 * Math.sin((i / 300) * Math.PI);
    eco.update(0.1, daylight);
    for (const a of eco.agents) if (a.alive) lines.push(dumpAgent(i, a));
  }
  return { text: lines.join('\n') + '\n', eco };
}

function structSummary(rows) {
  const c = { herb: 0, pred: 0, scav: 0, omn: 0, fish: 0 };
  for (const r of rows) c[r.kind] = (c[r.kind] || 0) + 1;
  return `herb=${c.herb} pred=${c.pred} scav=${c.scav} omn=${c.omn} fish=${c.fish} total=${rows.length}`;
}

// Parse a STATE_DUMP file into Map<tick, Rec[]> with ordered field tokens.
function parseDump(text) {
  const map = new Map();
  for (const line of text.split('\n')) {
    if (!line.startsWith('STATE_DUMP ')) continue;
    const rec = { kind: '', id: 0, fields: [] };
    let tick = -1;
    for (const tok of line.slice('STATE_DUMP '.length).split(' ')) {
      const eq = tok.indexOf('=');
      if (eq < 0) continue;
      const k = tok.slice(0, eq), v = tok.slice(eq + 1);
      if (k === 't') { tick = parseInt(v, 10); continue; }
      if (k === 'kind') { rec.kind = v; continue; }
      if (k === 'id') { rec.id = parseInt(v, 10); continue; }
      if (k === 'genome') {
        const inner = v.startsWith('{') ? v.slice(1, -1) : v;
        rec.fields.push(['genome', inner.split(',').map((p) => {
          const e = p.indexOf('=');
          return [p.slice(0, e), p.slice(e + 1)];
        })]);
        continue;
      }
      rec.fields.push([k, v]);
    }
    if (tick >= 0) {
      if (!map.has(tick)) map.set(tick, []);
      map.get(tick).push(rec);
    }
  }
  return map;
}

const numEq = (a, b) => (Number.isNaN(a) && Number.isNaN(b)) || a === b || (a === 0 && b === 0);
function valEq(jv, rv) {
  if (jv === undefined || rv === undefined) return false;
  if (jv === rv) return true;
  const jn = Number(jv), rn = Number(rv);
  if (!Number.isNaN(jn) && !Number.isNaN(rn)) return numEq(jn, rn);
  return false;
}

const FLAT_FIELDS = ['x', 'z', 'energy', 'age', 'bite', 'repro', 'state', 'heading', 'lineage', 'checksum_local'];

function firstDiff(jsMap, rsMap, maxTick) {
  for (let t = 0; t <= maxTick; t++) {
    const js = jsMap.get(t) || [];
    const rs = rsMap.get(t) || [];
    const n = Math.max(js.length, rs.length);
    for (let i = 0; i < n; i++) {
      const j = js[i], r = rs[i];
      if (!j) return { tick: t, id: r.id, kind: r.kind, field: 'missing_in_js', val_js: 'none', val_rs: `id=${r.id}` };
      if (!r) return { tick: t, id: j.id, kind: j.kind, field: 'missing_in_rs', val_js: `id=${j.id}`, val_rs: 'none' };
      if (j.id !== r.id) return { tick: t, id: j.id, kind: j.kind, field: 'id', val_js: String(j.id), val_rs: String(r.id) };
      if (j.kind !== r.kind) return { tick: t, id: j.id, kind: j.kind, field: 'kind', val_js: j.kind, val_rs: r.kind };
      for (const f of FLAT_FIELDS) {
        if (IGNORE.includes(f)) continue;
        const jv = j.fields.find((x) => x[0] === f)?.[1];
        const rv = r.fields.find((x) => x[0] === f)?.[1];
        if (!valEq(jv, rv)) {
          return { tick: t, id: j.id, kind: j.kind, field: f, val_js: jv ?? '<absent>', val_rs: rv ?? '<absent>' };
        }
      }
      const jg = j.fields.find((x) => x[0] === 'genome')?.[1] || [];
      const rg = r.fields.find((x) => x[0] === 'genome')?.[1] || [];
      const m = Math.max(jg.length, rg.length);
      for (let k = 0; k < m; k++) {
        const [jk, jv2] = jg[k] || ['<absent>', undefined];
        const [rk, rv2] = rg[k] || ['<absent>', undefined];
        if (IGNORE.includes(`genome.${jk}`)) continue;
        if (jk !== rk) {
          return { tick: t, id: j.id, kind: j.kind, field: `genome[${k}].key`, val_js: jk, val_rs: rk };
        }
        if (!valEq(jv2, rv2)) {
          return { tick: t, id: j.id, kind: j.kind, field: `genome.${jk}`, val_js: jv2 ?? '<absent>', val_rs: rv2 ?? '<absent>' };
        }
      }
    }
  }
  return null;
}

function findCargo() {
  const candidates = [process.env.CARGO, `${process.env.HOME}/.cargo/bin/cargo`, 'cargo'].filter(Boolean);
  for (const c of candidates) {
    try { execFileSync(c, ['--version'], { stdio: 'pipe' }); return c; } catch { /* next */ }
  }
  return null;
}

function runRustDump(cargo) {
  const coreRoot = new URL('..', import.meta.url).pathname;
  execFileSync(cargo, ['build', '-p', 'aether-sim-ecology'], { cwd: coreRoot, stdio: 'pipe' });
  const bin = join(coreRoot, 'target/debug/ecology-sim');
  const out = execFileSync(bin, ['--seed', String(SEED), '--ticks', String(TICKS), '--dump-state', '1'], {
    encoding: 'utf8', maxBuffer: 60 * 1024 * 1024,
  });
  return out;
}

// ---- main ----
if (GRASS_DUMP_PATH) {
  // Grass bit-level diagnostic mode: fresh Ecology, emit JS-side GRASS_ARR
  // lines, run the RS binary with matching --dump-grass, then diff and report
  // the first differing (tick, field, index).
  const ecoGrass = new Ecology(SEED);
  const jsGrass = runJsGrassDump(ecoGrass, GRASS_TICKS);
  writeFileSync(GRASS_DUMP_PATH, jsGrass);
  console.log(`[INFO] JS grass dump (ticks=${GRASS_TICKS.join(',')}) → ${GRASS_DUMP_PATH}`);
  const cargo = findCargo();
  if (!cargo) { console.error('[ERROR] cargo not found'); process.exit(2); }
  const coreRoot = new URL('..', import.meta.url).pathname;
  execFileSync(cargo, ['build', '-p', 'aether-sim-ecology'], { cwd: coreRoot, stdio: 'pipe' });
  const bin = join(coreRoot, 'target/debug/ecology-sim');
  const RS_GRASS_OUT = join(tmpdir(), `grass_rs_${SEED}.txt`);
  const rsRaw = execFileSync(bin, ['--seed', String(SEED), '--ticks', String(TICKS), '--dump-grass', GRASS_TICKS.join(',')], { encoding: 'utf8', maxBuffer: 60 * 1024 * 1024 });
  const rsGrass = rsRaw.split('\n').filter((l) => l.startsWith('GRASS_ARR')).join('\n') + '\n';
  writeFileSync(RS_GRASS_OUT, rsGrass);
  console.log(`[INFO] RS grass dump → ${RS_GRASS_OUT}`);
  const jsLines = jsGrass.split('\n').filter(Boolean);
  const rsLines = rsGrass.split('\n').filter(Boolean);
  let found = false;
  for (let li = 0; li < Math.min(jsLines.length, rsLines.length) && !found; li++) {
    const j = jsLines[li], r = rsLines[li];
    if (j === r) continue;
    const [, jt, ja, jhex] = j.match(/^GRASS_ARR t=(\d+) arr=(\S+) (.+)$/);
    const [, rt, ra, rhex] = r.match(/^GRASS_ARR t=(\d+) arr=(\S+) (.+)$/);
    if (jt !== rt || ja !== ra) {
      console.log(`GRASS_DIFF line=${li} label_mismatch js=(t=${jt},arr=${ja}) rs=(t=${rt},arr=${ra})`);
      found = true; break;
    }
    for (let ci = 0; ci < jhex.length; ci += 8) {
      const jv = jhex.slice(ci, ci + 8), rv = rhex.slice(ci, ci + 8);
      if (jv !== rv) {
        const jf = new Float32Array(new Uint32Array([parseInt(jv, 16)]).buffer)[0];
        const rf = new Float32Array(new Uint32Array([parseInt(rv, 16)]).buffer)[0];
        const cell = ci / 8;
        const cellX = cell % 76, cellY = Math.floor(cell / 76);
        const rel = Math.abs(rf - jf) / (Math.abs(jf) || 1);
        console.log(`GRASS_FIRST_DIFF t=${jt} arr=${ja} cell=${cell} (x=${cellX},y=${cellY}) bits js=0x${jv} rs=0x${rv} f32 js=${jf.toExponential(17)} rs=${rf.toExponential(17)} rel_diff=${rel.toExponential(3)}`);
        found = true; break;
      }
    }
  }
  if (!found) console.log(`GRASS_NO_DIFF: all dumped ticks bit-identical (${jsLines.length} lines each)`);
  process.exit(found ? 1 : 0);
}

const { text: jsText, eco } = runJsDump();

if (DUMP_ONLY_PATH) {
  writeFileSync(DUMP_ONLY_PATH, jsText);
  console.log(`[INFO] JS oracle dump (seed=${SEED}, ticks=${TICKS}) → ${DUMP_ONLY_PATH} (${jsText.split('\n').filter((l) => l.startsWith('STATE_DUMP')).length} rows)`);
  console.log(`[INFO] JS init structure: ${structSummary(parseDump(jsText).get(0) || [])}`);
  console.log(`[INFO] JS final checksum: ${eco.checksum()}`);
  process.exit(0);
}

writeFileSync(JS_OUT, jsText);
console.log(`[INFO] JS oracle dump → ${JS_OUT} (${jsText.split('\n').filter((l) => l.startsWith('STATE_DUMP')).length} rows)`);
console.log(`[INFO] JS init structure: ${structSummary(parseDump(jsText).get(0) || [])}`);

const cargo = findCargo();
if (!cargo) {
  console.error('[ERROR] cargo not found on PATH; cannot run Rust side');
  process.exit(2);
}
const rsText = runRustDump(cargo);
writeFileSync(RS_OUT, rsText);
console.log(`[INFO] Rust sim dump → ${RS_OUT}`);
console.log(`[INFO] RS init structure: ${structSummary(parseDump(rsText).get(0) || [])}`);

const jsMap = parseDump(jsText);
const rsMap = parseDump(rsText);
const maxTick = Math.min(Math.max(...jsMap.keys()), Math.max(...rsMap.keys()));
const diff = firstDiff(jsMap, rsMap, maxTick);

if (!diff) {
  console.log(`NO_DIFF up_to_tick=${maxTick}: JS and RS state dumps are field-identical (seed=${SEED})`);
  console.log(`[INFO] JS final checksum: ${eco.checksum()}`);
  process.exit(0);
}

const IGNORE_NOTE = IGNORE.length ? ` (ignored: ${IGNORE.join(',')})` : '';
console.log(
  `FIRST_DIFF tick=${diff.tick} id=${diff.id} kind=${diff.kind} field=${diff.field} val_js=${diff.val_js} val_rs=${diff.val_rs}${IGNORE_NOTE}`
);
console.log(
  `分叉定位：第 ${diff.tick} 个 tick，ID=${diff.id}（${diff.kind}）的 ${diff.field} 字段首次出现偏差：JS=${diff.val_js} vs RS=${diff.val_rs}`
);
if (Number(diff.val_js) && Number(diff.val_rs) && diff.field !== 'checksum_local') {
  const d = Number(diff.val_rs) - Number(diff.val_js);
  console.log(`[INFO] 差值 delta(rs-js)=${d}`);
}
