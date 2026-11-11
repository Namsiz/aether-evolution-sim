// 共享世界函数：地形高度、坡度、生物群系与确定性随机数。
// 渲染和生态模拟必须使用同一套高度场，否则生物会浮空或沉入地面。

export const WORLD_SIZE = 1600;
export const WORLD_HALF = WORLD_SIZE / 2;
export const WATER_LEVEL = 0;
export const SIM_RADIUS = 560;

export function smoothstep(a, b, x) {
  const t = Math.min(1, Math.max(0, (x - a) / (b - a)));
  return t * t * (3 - 2 * t);
}

export function clamp(v, a, b) {
  return v < a ? a : v > b ? b : v;
}

export function lerp(a, b, t) {
  return a + (b - a) * t;
}

export function mulberry32(seed) {
  let a = seed >>> 0;
  return function () {
    a |= 0;
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

// ---- 2D Perlin ----
const PERM = new Uint8Array(512);
(function buildPerm() {
  const rng = mulberry32(1337);
  const p = new Uint8Array(256);
  for (let i = 0; i < 256; i++) p[i] = i;
  for (let i = 255; i > 0; i--) {
    const j = Math.floor(rng() * (i + 1));
    const t = p[i];
    p[i] = p[j];
    p[j] = t;
  }
  for (let i = 0; i < 512; i++) PERM[i] = p[i & 255];
})();

function fade(t) {
  return t * t * t * (t * (t * 6 - 15) + 10);
}

function grad(hash, x, y) {
  switch (hash & 7) {
    case 0: return x + y;
    case 1: return -x + y;
    case 2: return x - y;
    case 3: return -x - y;
    case 4: return x;
    case 5: return -x;
    case 6: return y;
    default: return -y;
  }
}

export function noise2(x, y) {
  const xi = Math.floor(x) & 255;
  const yi = Math.floor(y) & 255;
  const xf = x - Math.floor(x);
  const yf = y - Math.floor(y);
  const u = fade(xf);
  const v = fade(yf);
  const aa = PERM[PERM[xi] + yi];
  const ab = PERM[PERM[xi] + yi + 1];
  const ba = PERM[PERM[xi + 1] + yi];
  const bb = PERM[PERM[xi + 1] + yi + 1];
  const x1 = lerp(grad(aa, xf, yf), grad(ba, xf - 1, yf), u);
  const x2 = lerp(grad(ab, xf, yf - 1), grad(bb, xf - 1, yf - 1), u);
  return lerp(x1, x2, v) * 0.70710678;
}

export function fbm2(x, y, octaves = 5, lacunarity = 2.0, gain = 0.5) {
  let amp = 0.5;
  let freq = 1.0;
  let sum = 0;
  for (let i = 0; i < octaves; i++) {
    sum += noise2(x * freq, y * freq) * amp;
    freq *= lacunarity;
    amp *= gain;
  }
  return sum; // 约 -0.5..0.5
}

export function ridgedFbm(x, y, octaves = 5) {
  let amp = 0.55;
  let freq = 1.0;
  let sum = 0;
  let norm = 0;
  for (let i = 0; i < octaves; i++) {
    const n = 1 - Math.abs(noise2(x * freq, y * freq) * 2);
    sum += n * n * amp;
    norm += amp;
    freq *= 2.03;
    amp *= 0.5;
  }
  return sum / norm;
}

// 一座湖盆，四周是山；一侧山势较低，留出远景缺口。
export function terrainHeight(x, z) {
  const d = Math.hypot(x, z);
  const ang = Math.atan2(z, x);
  const shore = smoothstep(105, 240, d);

  let h = -9.5 + 12.5 * shore;

  const roll = fbm2(x * 0.0055 + 13.1, z * 0.0055 - 4.7, 4) * 2;
  h += roll * 5.5 * smoothstep(120, 255, d);

  const m = smoothstep(330, 730, d);
  const ridge = ridgedFbm(x * 0.0028 + 2.2, z * 0.0028 - 1.7, 5);
  const dir = 0.48 + 0.52 * (0.5 + 0.5 * Math.cos(ang - 2.15));
  h += Math.pow(m, 1.35) * (26 + 165 * ridge) * dir;

  // 高处增加破碎的岩脊，低处保持草甸平滑
  const crag = smoothstep(55, 135, h) * (fbm2(x * 0.021, z * 0.021, 3) * 2) * 13;
  h += crag;

  h += smoothstep(650, 900, d) * 24;
  return h;
}

export function slopeAt(x, z) {
  const e = 3.0;
  const dx = terrainHeight(x + e, z) - terrainHeight(x - e, z);
  const dz = terrainHeight(x, z + e) - terrainHeight(x, z - e);
  return Math.hypot(dx, dz) / (2 * e);
}

export function terrainNormal(x, z, out) {
  const e = 1.6;
  const hx = terrainHeight(x + e, z) - terrainHeight(x - e, z);
  const hz = terrainHeight(x, z + e) - terrainHeight(x, z - e);
  out.set(-hx, 2 * e, -hz).normalize();
  return out;
}

// 生物群系与草量：供着色和生态栅格共用
export function groundKind(x, z, h = terrainHeight(x, z), slope = slopeAt(x, z)) {
  if (h < WATER_LEVEL + 1.1) return 'sand';
  if (slope > 0.68) return 'rock';
  if (h > 132) return 'snow';
  if (h > 95 && slope > 0.34) return 'rock';
  return 'grass';
}

export function plantSuitability(x, z) {
  const h = terrainHeight(x, z);
  if (h < WATER_LEVEL + 1.2 || h > 78) return 0;
  const s = slopeAt(x, z);
  const wet = smoothstep(230, 120, Math.hypot(x, z)) * 0.35 + 0.65;
  return clamp(
    smoothstep(WATER_LEVEL + 1.2, WATER_LEVEL + 7, h) *
      (1 - smoothstep(0.42, 0.85, s)) *
      (1 - smoothstep(58, 88, h)) * wet,
    0, 1
  );
}
