#!/usr/bin/env node
// gen_libm_gold.mjs — generate golden samples of Node's Math.sin/cos/exp/atan2/pow
// (V8 12.4 fdlibm path) for the aether-libm bit-exact port tests.
//
// Inputs are drawn from the same mulberry32(20261102) PRNG the Rust test
// reproduces via aether-rng, so only the f64 outputs need to be stored.
//
// Output format (little-endian f64):
//   sin/cos/exp : N pairs  [x, out]
//   atan2       : N triples [y, x, out]
//   pow         : N triples [base, exp, out]
// First K_SPECIAL entries of each file are fixed special inputs.
//
// Files are written to /tmp/libm_gold_sample_<fn>.bin (full 1e6 samples, per
// P0.5 spec) and a 1e4-sample prefix to crates/aether-libm/tests/gold/ so the
// committed regression test stays green without /tmp.

import { writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const N = 1_000_000;
const N_COMMIT = 10_000;
const SEED = 20261102;

function mulberry32(seed) {
  let a = seed >>> 0;
  return function () {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const TWO_PI = Math.PI * 2;

const SPECIAL_SIN = [
  0.0, -0.0, 1e-320, 1e-300, 1e-15, Math.pow(2, -27), Math.pow(2, -26),
  0.3, 0.30000000000000004, 0.5, Math.PI / 4, 0.7, 0.78125, 0.7813,
  Math.PI / 2, 1.0, 1.5, 2.0, 3.0, Math.PI, TWO_PI, 57.29577951308232,
  1e10, 1e15, 2e19 * Math.PI / 2, 1e300, -1e300, Infinity, -Infinity, NaN,
];

const SPECIAL_EXP = [
  0.0, -0.0, 1e-320, 1e-300, 1e-15, Math.pow(2, -28), Math.pow(2, -27),
  0.1, 0.34657359027997264, 0.3465735902799727, 0.5, 0.6931471805599453,
  0.6931471805599454, 1.0, 1.0431471805599453, 1.5, 2.0, 3.0, 10.0,
  700.0, 709.0, 709.782712893384, 709.782712893385, -700.0, -745.0,
  -745.1332191019411, -745.1332191019412, 1e300, Infinity, -Infinity, NaN,
];

const SPECIAL_ATAN2 = [
  [0.0, 0.0], [-0.0, 0.0], [0.0, -0.0], [-0.0, -0.0],
  [0.0, -1.0], [-0.0, -1.0], [1.0, 0.0], [-1.0, 0.0],
  [Infinity, Infinity], [-Infinity, Infinity],
  [Infinity, -Infinity], [-Infinity, -Infinity],
  [0.0, Infinity], [-0.0, Infinity], [1.0, Infinity], [-1.0, Infinity],
  [Infinity, 0.0], [Infinity, -0.0], [Infinity, -1.0],
  [0.0, -Infinity], [1.0, -Infinity], [-1.0, -Infinity],
  [NaN, 1.0], [1.0, NaN], [NaN, NaN],
  [1.0, 1.0], [-1.0, 1.0], [1.0, -1.0], [-1.0, -1.0],
  [1e-300, -1e-300], [1e300, 1e-300], [1e-300, 1e300],
];

const SPECIAL_POW = [
  [0.0, 0.0], [0.0, 1.35], [0.0, 0.5], [0.0, -1.0], [1.0, 1.35],
  [1.0, 0.0], [1.0, Infinity], [1.0, -Infinity], [2.0, 0.0],
  [2.0, 0.5], [2.0, 2.0], [2.0, -0.5], [2.0, 1024.0], [0.5, 1.35],
  [0.7, 1.35], [0.25, 0.5], [8.0, 1.0 / 3.0], [-2.0, 2.0], [-2.0, 3.0],
  [-2.0, 0.5], [Infinity, 2.0], [Infinity, -2.0], [-Infinity, 2.0],
  [NaN, 1.0], [1.0, NaN], [1e-300, 1e300], [0.9999999999999999, 1.35],
];

function writePairs(path, special, f, n, gen) {
  const buf = Buffer.alloc((special.length + n) * 16);
  let off = 0;
  for (const x of special) {
    buf.writeDoubleLE(x, off);
    buf.writeDoubleLE(f(x), off + 8);
    off += 16;
  }
  for (let i = 0; i < n; i++) {
    const x = gen();
    buf.writeDoubleLE(x, off);
    buf.writeDoubleLE(f(x), off + 8);
    off += 16;
  }
  writeFileSync(path, buf);
  console.log(`${path}: ${special.length + n} samples, ${(buf.length / 1e6).toFixed(1)} MB`);
}

function writeTriples(path, special, f, n, genA, genB) {
  const buf = Buffer.alloc((special.length + n) * 24);
  let off = 0;
  for (const [a, b] of special) {
    buf.writeDoubleLE(a, off);
    buf.writeDoubleLE(b, off + 8);
    buf.writeDoubleLE(f(a, b), off + 16);
    off += 24;
  }
  for (let i = 0; i < n; i++) {
    const a = genA();
    const b = genB();
    buf.writeDoubleLE(a, off);
    buf.writeDoubleLE(b, off + 8);
    buf.writeDoubleLE(f(a, b), off + 16);
    off += 24;
  }
  writeFileSync(path, buf);
  console.log(`${path}: ${special.length + n} samples, ${(buf.length / 1e6).toFixed(1)} MB`);
}

const here = dirname(fileURLToPath(import.meta.url));
const goldDir = join(here, '..', 'crates', 'aether-libm', 'tests', 'gold');
mkdirSync(goldDir, { recursive: true });
mkdirSync('/tmp', { recursive: true });

// Per-function independent PRNG streams (seed identical, order identical in Rust).
for (const [name, special, f, mapper] of [
  ['sin', SPECIAL_SIN, Math.sin, x => x],
  ['cos', SPECIAL_SIN, Math.cos, x => x],
  ['exp', SPECIAL_EXP, Math.exp, x => x],
]) {
  const gen = (() => { const rng = mulberry32(SEED); return () => rng() * TWO_PI; })();
  writePairs(`/tmp/libm_gold_sample_${name}.bin`, special, f, N, gen);
  writePairs(join(goldDir, `libm_gold_${name}.bin`), special, f, N_COMMIT, gen);
}

{
  const genY = (() => { const rng = mulberry32(SEED); return () => (rng() * 2 - 1) * TWO_PI; })();
  const genX = (() => { const rng = mulberry32(SEED); return () => (rng() * 2 - 1) * TWO_PI; })();
  writeTriples('/tmp/libm_gold_sample_atan2.bin', SPECIAL_ATAN2, Math.atan2, N, genY, genX);
  writeTriples(join(goldDir, 'libm_gold_atan2.bin'), SPECIAL_ATAN2, Math.atan2, N_COMMIT, genY, genX);
}

{
  const genB = (() => { const rng = mulberry32(SEED); return () => rng(); })();
  const genE = (() => { const rng = mulberry32(SEED); return () => 1.0 + rng() * 0.7; })();
  writeTriples('/tmp/libm_gold_sample_pow.bin', SPECIAL_POW, Math.pow, N, genB, genE);
  writeTriples(join(goldDir, 'libm_gold_pow.bin'), SPECIAL_POW, Math.pow, N_COMMIT, genB, genE);
}

console.log('golden sample generation complete');
