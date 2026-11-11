import * as THREE from 'three';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';
import {
  WORLD_SIZE,
  WATER_LEVEL,
  terrainHeight,
  slopeAt,
  plantSuitability,
  clamp,
  mulberry32,
  smoothstep,
} from './world.js';

// ---------- 程序化贴图 ----------
function tileNoise(x, y, period, seed) {
  const xi = Math.floor(x);
  const yi = Math.floor(y);
  const xf = x - xi;
  const yf = y - yi;
  const u = xf * xf * (3 - 2 * xf);
  const v = yf * yf * (3 - 2 * yf);
  const h = (a, b) => {
    let n = Math.imul((a % period + period) % period + 31, 374761393) ^
            Math.imul((b % period + period) % period + 17, 668265263) ^ seed;
    n = Math.imul(n ^ (n >>> 13), 1274126177);
    return ((n ^ (n >>> 16)) >>> 0) / 4294967295;
  };
  const a = h(xi, yi);
  const b = h(xi + 1, yi);
  const c = h(xi, yi + 1);
  const d = h(xi + 1, yi + 1);
  return (a * (1 - u) + b * u) * (1 - v) + (c * (1 - u) + d * u) * v;
}

function makeSurfaceTextures() {
  const size = 512;
  const albedo = document.createElement('canvas');
  albedo.width = albedo.height = size;
  const actx = albedo.getContext('2d');
  const heightCanvas = document.createElement('canvas');
  heightCanvas.width = heightCanvas.height = size;
  const hctx = heightCanvas.getContext('2d');

  actx.fillStyle = '#b9c0a5';
  actx.fillRect(0, 0, size, size);
  hctx.fillStyle = '#808080';
  hctx.fillRect(0, 0, size, size);

  const rng = mulberry32(1177);
  const wrap = (x, y, fn) => {
    const xs = [x];
    const ys = [y];
    if (x < 24) xs.push(x + size);
    if (x > size - 24) xs.push(x - size);
    if (y < 24) ys.push(y + size);
    if (y > size - 24) ys.push(y - size);
    for (const xx of xs) for (const yy of ys) fn(xx, yy);
  };

  // 大尺度草皮/土壤色斑，打散平铺感
  const patchColors = [
    'rgba(94,118,76,0.12)', 'rgba(186,186,148,0.12)',
    'rgba(78,100,64,0.09)', 'rgba(206,202,170,0.09)',
  ];
  for (let i = 0; i < 90; i++) {
    const x = rng() * size;
    const y = rng() * size;
    const r = 24 + rng() * 90;
    const col = patchColors[Math.floor(rng() * patchColors.length)];
    wrap(x, y, (xx, yy) => {
      const g = actx.createRadialGradient(xx, yy, 0, xx, yy, r);
      g.addColorStop(0, col);
      g.addColorStop(1, 'rgba(0,0,0,0)');
      actx.fillStyle = g;
      actx.fillRect(xx - r, yy - r, r * 2, r * 2);
    });
  }

  const blades = [];
  const palette = [
    [120, 140, 92], [148, 164, 112], [178, 182, 140],
    [104, 126, 82], [170, 178, 128], [202, 200, 168],
    [142, 135, 102], [118, 138, 105],
  ];
  for (let i = 0; i < 7200; i++) {
    const c = palette[Math.floor(rng() * palette.length)];
    blades.push({
      x: rng() * size,
      y: rng() * size,
      len: 3 + rng() * 15,
      ang: -Math.PI / 2 + (rng() - 0.5) * 1.1,
      bend: (rng() - 0.5) * 4,
      w: 0.6 + rng() * 1.7,
      a: 0.28 + rng() * 0.5,
      c,
    });
  }
  for (const b of blades) {
    wrap(b.x, b.y, (x, y) => {
      const tx = x + Math.cos(b.ang) * b.len + b.bend;
      const ty = y + Math.sin(b.ang) * b.len;
      actx.strokeStyle = `rgba(${b.c[0]},${b.c[1]},${b.c[2]},${b.a})`;
      actx.lineWidth = b.w;
      actx.beginPath();
      actx.moveTo(x, y);
      actx.quadraticCurveTo(
        x + Math.cos(b.ang) * b.len * 0.55,
        y + Math.sin(b.ang) * b.len * 0.55,
        tx, ty
      );
      actx.stroke();

      const hv = 128 + 48 * b.a;
      hctx.strokeStyle = `rgb(${hv},${hv},${hv})`;
      hctx.lineWidth = Math.max(0.7, b.w);
      hctx.beginPath();
      hctx.moveTo(x, y);
      hctx.quadraticCurveTo(
        x + Math.cos(b.ang) * b.len * 0.55,
        y + Math.sin(b.ang) * b.len * 0.55,
        tx, ty
      );
      hctx.stroke();
    });
  }

  // 小石子
  for (let i = 0; i < 900; i++) {
    const x = rng() * size;
    const y = rng() * size;
    const r = 0.4 + rng() * 1.4;
    const v = 800 + rng() * 60;
    wrap(x, y, (xx, yy) => {
      actx.fillStyle = `rgba(${v * 0.92},${v * 0.9},${v * 0.84},0.5)`;
      actx.beginPath();
      actx.arc(xx, yy, r, 0, Math.PI * 2);
      actx.fill();
    });
  }

  const aimg = actx.getImageData(0, 0, size, size);
  const himg = hctx.getImageData(0, 0, size, size);
  const hdata = new Float32Array(size * size);
  for (let i = 0; i < size * size; i++) {
    hdata[i] = (himg.data[i * 4] + himg.data[i * 4 + 1] + himg.data[i * 4 + 2]) / 765;
  }

  const normal = document.createElement('canvas');
  normal.width = normal.height = size;
  const nctx = normal.getContext('2d');
  const nimg = nctx.createImageData(size, size);
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const i = (y * size + x) * 4;
      const l = hdata[y * size + ((x - 1 + size) % size)];
      const r = hdata[y * size + ((x + 1) % size)];
      const d = hdata[((y - 1 + size) % size) * size + x];
      const u = hdata[((y + 1) % size) * size + x];
      let nx = (l - r) * 4.2;
      let ny = (d - u) * 4.2;
      const len = Math.hypot(nx, ny, 1);
      nx /= len;
      ny /= len;
      nimg.data[i] = Math.round((nx * 0.5 + 0.5) * 255);
      nimg.data[i + 1] = Math.round((ny * 0.5 + 0.5) * 255);
      nimg.data[i + 2] = Math.round((0.5 + 0.5 / len) * 255);
      nimg.data[i + 3] = 255;
    }
  }
  nctx.putImageData(nimg, 0, 0);

  const albedoTex = new THREE.CanvasTexture(albedo);
  albedoTex.colorSpace = THREE.SRGBColorSpace;
  albedoTex.wrapS = albedoTex.wrapT = THREE.RepeatWrapping;
  albedoTex.repeat.set(118, 118);
  albedoTex.anisotropy = 8;
  const normalTex = new THREE.CanvasTexture(normal);
  normalTex.wrapS = normalTex.wrapT = THREE.RepeatWrapping;
  normalTex.repeat.set(168, 168);
  normalTex.anisotropy = 8;
  return { albedo: albedoTex, normal: normalTex };
}

function makeGrassTexture() {
  const c = document.createElement('canvas');
  c.width = 128;
  c.height = 128;
  const ctx = c.getContext('2d');
  ctx.clearRect(0, 0, 128, 128);
  const rng = mulberry32(2024);
  for (let i = 0; i < 26; i++) {
    const x0 = 10 + rng() * 108;
    const height = 52 + rng() * 72;
    const bend = (rng() - 0.5) * 40;
    const w = 1.5 + rng() * 3.2;
    const g = ctx.createLinearGradient(0, 128, 0, 128 - height);
    g.addColorStop(0, 'rgba(150,164,104,0.98)');
    g.addColorStop(0.55, 'rgba(214,224,170,1)');
    g.addColorStop(1, 'rgba(248,250,224,1)');
    ctx.strokeStyle = g;
    ctx.lineWidth = w;
    ctx.lineCap = 'round';
    ctx.beginPath();
    ctx.moveTo(x0, 128);
    ctx.quadraticCurveTo(x0 + bend * 0.2, 128 - height * 0.55, x0 + bend, 128 - height);
    ctx.stroke();
  }
  const tex = new THREE.CanvasTexture(c);
  tex.colorSpace = THREE.SRGBColorSpace;
  tex.anisotropy = 2;
  return tex;
}

// ---------- 地形 ----------
export function createTerrain(scene, quality) {
  const seg = quality.terrainSegments;
  const geo = new THREE.PlaneGeometry(WORLD_SIZE, WORLD_SIZE, seg, seg);
  geo.rotateX(-Math.PI / 2);
  const pos = geo.attributes.position;
  for (let i = 0; i < pos.count; i++) {
    const x = pos.getX(i);
    const z = pos.getZ(i);
    pos.setY(i, terrainHeight(x, z));
  }
  geo.computeVertexNormals();

  const normals = geo.attributes.normal;
  const colors = new Float32Array(pos.count * 3);
  const cGrassLush = new THREE.Color(0x436127);
  const cGrassDark = new THREE.Color(0x2f421d);
  const cGrassDry = new THREE.Color(0x6f703e);
  const cGrassWarm = new THREE.Color(0x5c6b32);
  const cSand = new THREE.Color(0xb6a57e);
  const cSandWet = new THREE.Color(0x827961);
  const cRock = new THREE.Color(0x746e63);
  const cRockDark = new THREE.Color(0x49453e);
  const cSnow = new THREE.Color(0xd9dee1);
  const cScree = new THREE.Color(0x8b857a);
  const tmp = new THREE.Color();
  const tmp2 = new THREE.Color();

  for (let i = 0; i < pos.count; i++) {
    const x = pos.getX(i);
    const z = pos.getZ(i);
    const h = pos.getY(i);
    const ny = Math.max(0.06, normals.getY(i));
    const slope = Math.sqrt(1 - ny * ny) / ny;
    const patch = tileNoise((x + 800) * 0.006, (z + 800) * 0.006, 64, 33);
    const fine = tileNoise((x + 800) * 0.05, (z + 800) * 0.05, 32, 71);

    if (h < WATER_LEVEL + 0.65) {
      tmp.copy(cSandWet).lerp(cSand, smoothstep(WATER_LEVEL, WATER_LEVEL + 1.6, h));
    } else if (slope > 0.66 || (h > 80 && slope > 0.32)) {
      tmp.copy(cRockDark).lerp(cRock, patch);
      if (h > 105 && slope > 0.48) tmp.lerp(cScree, fine * 0.5);
    } else if (h > 128 + patch * 16) {
      tmp.copy(cRock).lerp(cSnow, smoothstep(130, 155, h) * (1 - smoothstep(0.7, 1.25, slope) * 0.6));
    } else {
      const moisture = smoothstep(360, 140, Math.hypot(x, z));
      tmp.copy(cGrassDark).lerp(cGrassLush, moisture * 0.75 + patch * 0.25);
      tmp.lerp(cGrassDry, clamp((1 - moisture) * 0.55 + (patch - 0.5) * 0.45, 0, 0.7));
      tmp.lerp(cGrassWarm, fine * 0.18);
    }

    const shade = 0.82 + fine * 0.3 - smoothstep(0.5, 1.3, slope) * 0.16;
    tmp.multiplyScalar(shade);
    colors[i * 3] = tmp.r;
    colors[i * 3 + 1] = tmp.g;
    colors[i * 3 + 2] = tmp.b;
  }
  geo.setAttribute('color', new THREE.BufferAttribute(colors, 3));

  const surface = makeSurfaceTextures();
  const terrainMat = new THREE.MeshStandardMaterial({
    vertexColors: true,
    map: surface.albedo,
    normalMap: surface.normal,
    normalScale: new THREE.Vector2(0.48, 0.48),
    roughness: 0.94,
    metalness: 0.0,
  });
  const terrain = new THREE.Mesh(geo, terrainMat);
  terrain.receiveShadow = true;
  terrain.castShadow = false;
  terrain.name = 'terrain';
  scene.add(terrain);

  const rng = mulberry32(4242);

  // ---------- 草 ----------
  const grassTex = makeGrassTexture();
  const grassGeo = buildGrassGeometry();
  const grassMat = new THREE.MeshStandardMaterial({
    map: grassTex,
    alphaTest: 0.38,
    side: THREE.DoubleSide,
    roughness: 0.85,
    metalness: 0,
    color: 0xffffff,
    vertexColors: false,
  });
  const wind = { value: 0 };
  grassMat.onBeforeCompile = (shader) => {
    shader.uniforms.uWindTime = wind;
    shader.vertexShader = 'uniform float uWindTime;\n' + shader.vertexShader;
    shader.vertexShader = shader.vertexShader.replace(
      '#include <begin_vertex>',
      `#include <begin_vertex>
      #ifdef USE_INSTANCING
        vec3 ip = vec3(instanceMatrix[3][0], instanceMatrix[3][1], instanceMatrix[3][2]);
        float sway = sin(uWindTime * 1.45 + ip.x * 0.055 + ip.z * 0.032);
        float gust = sin(uWindTime * 0.32 + ip.x * 0.014 - ip.z * 0.01) * 0.5 + 0.5;
        transformed.x += sway * uv.y * (0.12 + gust * 0.12);
        transformed.z += cos(uWindTime * 1.15 + ip.z * 0.06) * uv.y * 0.1;
      #endif`
    );
  };
  const grassCount = quality.grassCount;
  const grass = new THREE.InstancedMesh(grassGeo, grassMat, grassCount);
  grass.receiveShadow = true;
  grass.castShadow = false;
  const grassColors = [0x7d9255, 0x8aa15f, 0x6f8549, 0x97a96c, 0x778c50];
  // 草种-1湿区翠绿 / 草种-2旱区枯黄 / 草种-3过渡带橄榄，按 rMax 加权
  const grassSpeciesColors = [0x2dad2d, 0xc4a030, 0x7a9a5a];
  const grassSpeciesWeight = [1.4, 0.9, 1.1];
  const dummy = new THREE.Object3D();
  const gColor = new THREE.Color();
  const _gM = new THREE.Matrix4();
  const _gP = new THREE.Vector3();
  const _gQ = new THREE.Quaternion();
  const _gE = new THREE.Euler();
  const _gS = new THREE.Vector3();
  const grassX = new Float32Array(grassCount);
  const grassY = new Float32Array(grassCount);
  const grassZ = new Float32Array(grassCount);
  const grassRot = new Float32Array(grassCount);
  const grassSx = new Float32Array(grassCount);
  const grassSy = new Float32Array(grassCount);
  const grassSz = new Float32Array(grassCount);
  let gi = 0;
  let guard = 0;
  const nearX = 232;
  const nearZ = 292;
  while (gi < grassCount && guard < grassCount * 10) {
    guard++;
    let x, z;
    if (gi < grassCount * 0.78) {
      const r = 118 * Math.sqrt(rng());
      const a = rng() * Math.PI * 2;
      x = nearX + Math.cos(a) * r;
      z = nearZ + Math.sin(a) * r;
    } else {
      const r = 730 * Math.sqrt(rng());
      const a = rng() * Math.PI * 2;
      x = Math.cos(a) * r;
      z = Math.sin(a) * r;
    }
    const h = terrainHeight(x, z);
    if (h < WATER_LEVEL + 0.55 || h > 82) continue;
    if (slopeAt(x, z) > 0.7) continue;
    const suit = plantSuitability(x, z);
    if (suit < 0.08 || rng() > 0.3 + suit * 0.9) continue;

    dummy.position.set(x, h - 0.04, z);
    const grot = rng() * Math.PI * 2;
    dummy.rotation.set(0, grot, 0);
    const s = 0.95 + rng() * 1.3;
    const gsx = 0.72 + rng() * 0.8;
    const gsz = 0.72 + rng() * 0.8;
    dummy.scale.set(gsx, s, gsz);
    dummy.updateMatrix();
    grass.setMatrixAt(gi, dummy.matrix);
    gColor.setHex(grassColors[Math.floor(rng() * grassColors.length)]);
    gColor.offsetHSL(0, 0, (rng() - 0.5) * 0.1);
    grass.setColorAt(gi, gColor);
    grassX[gi] = x; grassY[gi] = h - 0.04; grassZ[gi] = z;
    grassRot[gi] = grot; grassSx[gi] = gsx; grassSy[gi] = s; grassSz[gi] = gsz;
    gi++;
  }
  grass.count = gi;
  grass.instanceMatrix.needsUpdate = true;
  if (grass.instanceColor) grass.instanceColor.needsUpdate = true;
  grass.computeBoundingSphere();
  scene.add(grass);

  // 草量视觉动态：草叶 scale/疏密随 ecology 草量网格变化（被吃→变矮变细→再生→恢复）
  // trample 可选 {x,z}：玩家踩踏区域压低草（步行模式下角色可见）
  function updateGrassVisual(grid, gridN, span, trample, grassP) {
    const cell = span / gridN;
    const invCell = 1 / cell;
    const half = span / 2;
    let changed = false;
    for (let k = 0; k < gi; k++) {
      let cx = Math.floor((grassX[k] + half) * invCell);
      let cz = Math.floor((grassZ[k] + half) * invCell);
      if (cx < 0) cx = 0; else if (cx >= gridN) cx = gridN - 1;
      if (cz < 0) cz = 0; else if (cz >= gridN) cz = gridN - 1;
      const idx = cz * gridN + cx;
      const g = grid[idx];
      const f = 0.12 + 0.88 * g;
      let ts = 1;
      let trampleT = 0;
      if (trample) {
        const dx = grassX[k] - trample.x, dz = grassZ[k] - trample.z;
        const d2 = dx * dx + dz * dz;
        if (d2 < 25) {
          const t = Math.min(1, (5 - Math.sqrt(d2)) / 2.4);
          ts = 1 - 0.82 * t;
          trampleT = t;
        }
      }
      const fs = (0.5 + 0.5 * f) * ts;
      _gM.compose(
        _gP.set(grassX[k], grassY[k], grassZ[k]),
        _gQ.setFromEuler(_gE.set(0, grassRot[k], 0)),
        _gS.set(grassSx[k] * fs, grassSy[k] * f * ts, grassSz[k] * fs)
      );
      grass.setMatrixAt(k, _gM);
      if (grassP && g > 0.02) {
        const total = grassP[0][idx] + grassP[1][idx] + grassP[2][idx];
        if (total > 0.02) {
          let cr = 0, cg = 0, cb = 0;
          for (let s = 0; s < 3; s++) {
            const w = grassP[s][idx] * grassSpeciesWeight[s];
            const base = grassSpeciesColors[s];
            cr += (base >> 16 & 0xff) / 255 * w;
            cg += (base >> 8 & 0xff) / 255 * w;
            cb += (base & 0xff) / 255 * w;
          }
          const brightness = (0.25 + 0.75 * Math.min(1, total * 1.5)) * (1 - 0.25 * trampleT);
          const v = 0.92 + rng() * 0.16;
          gColor.setRGB(clamp(cr * brightness * v, 0, 1), clamp(cg * brightness * v, 0, 1), clamp(cb * brightness * v, 0, 1));
          grass.setColorAt(k, gColor);
          changed = true;
        }
      }
    }
    grass.instanceMatrix.needsUpdate = true;
    if (changed && grass.instanceColor) grass.instanceColor.needsUpdate = true;
  }

  // ---------- 树 ----------
  const foliage = makeFoliageTexture();
  const canopyMat = new THREE.MeshStandardMaterial({
    vertexColors: true,
    map: foliage.albedo,
    normalMap: foliage.normal,
    normalScale: new THREE.Vector2(0.4, 0.4),
    alphaTest: 0.34,
    side: THREE.DoubleSide,
    roughness: 0.9,
    metalness: 0,
  });
  const trunkMat = new THREE.MeshStandardMaterial({
    vertexColors: true,
    roughness: 0.92,
    metalness: 0,
  });
  const archetypes = [
    { name: 'broadleaf', ...makeBroadleaf(), capacity: quality.treeCapacity },
    { name: 'conifer', ...makeConifer(), capacity: quality.treeCapacity },
    { name: 'birch', ...makeBirch(), capacity: Math.max(30, quality.treeCapacity >> 1) },
  ];
  const treeMeshes = [];
  const treeData = [];
  const treeRng = mulberry32(808);
  for (const arc of archetypes) {
    const trunk = new THREE.InstancedMesh(arc.trunkGeo, trunkMat, arc.capacity);
    const canopy = new THREE.InstancedMesh(arc.canopyGeo, canopyMat, arc.capacity);
    for (const m of [trunk, canopy]) {
      m.castShadow = true;
      m.receiveShadow = true;
      m.count = 0;
      m.frustumCulled = true;
      scene.add(m);
    }
    trunk.userData.instances = [];
    treeMeshes.push({ trunk, canopy, data: trunk.userData.instances });
  }

  // ---------- 植被演替 ----------
  // stage 0=幼苗 1=幼树 2=成年 3=衰老；scale 随 age 插值（相对成熟基准 s）
  function treeStageScale(life) {
    const s = life.s;
    if (life.age < 40) return s * (0.3 + (life.age / 40) * 0.35);
    if (life.age < 120) return s * (0.65 + ((life.age - 40) / 80) * 0.35);
    if (life.age < 400) return s;
    const over = life.age - 400;
    return s * (1 - Math.min(0.35, over / 1200));
  }
  function nearTree(x, z, minD) {
    const m2 = minD * minD;
    for (const en of treeMeshes) {
      for (const t of en.data) {
        const dx = t.x - x, dz = t.z - z;
        if (dx * dx + dz * dz < m2) return true;
      }
    }
    return false;
  }

  function placeTree(x, z, scale = null, variant = null, age = null) {
    const h = terrainHeight(x, z);
    if (h < WATER_LEVEL + 1.6 || h > 112) return false;
    if (slopeAt(x, z) > 0.62) return false;
    let v = variant;
    if (v === null || v === undefined) {
      const r = treeRng();
      if (h > 58) v = 1;
      else if (h > 38) v = r < 0.72 ? 1 : 0;
      else v = r < 0.74 ? 0 : (r < 0.95 ? 1 : 2);
    }
    const entry = treeMeshes[v];
    if (entry.data.length >= entry.trunk.instanceMatrix.count) return false;
    const s = scale ?? (1.0 + treeRng() * 0.95);
    const vy = 0.92 + treeRng() * 0.18;
    const rot = treeRng() * Math.PI * 2;
    const a = age ?? (treeRng() * 700);
    const life = {
      x, z, s, vy, rot, variant: v,
      age: a,
      maxAge: 520 + treeRng() * 380,
      seedT: 4 + treeRng() * 14,
      stage: a < 40 ? 0 : a < 120 ? 1 : a < 400 ? 2 : 3,
      _lastSt: 0,
    };
    const st = treeStageScale(life);
    life._lastSt = st;
    dummy.position.set(x, h - 0.15, z);
    dummy.rotation.set(0, rot, 0);
    dummy.scale.set(st, st * vy, st);
    dummy.updateMatrix();
    const idx = entry.data.length;
    entry.trunk.setMatrixAt(idx, dummy.matrix);
    entry.canopy.setMatrixAt(idx, dummy.matrix);
    gColor.setHex(v === 1 ? 0xd8e8cc : 0xffffff);
    gColor.offsetHSL(0, 0, (treeRng() - 0.5) * 0.06);
    entry.trunk.setColorAt(idx, gColor);
    entry.canopy.setColorAt(idx, gColor);
    entry.data.push(life);
    entry.trunk.count = entry.data.length;
    entry.canopy.count = entry.data.length;
    for (const m of [entry.trunk, entry.canopy]) {
      m.instanceMatrix.needsUpdate = true;
      if (m.instanceColor) m.instanceColor.needsUpdate = true;
      m.computeBoundingSphere();
    }
    treeData.push({ x, z });
    return true;
  }

  function removeTreeByIndex(entry, i) {
    const last = entry.data.length - 1;
    const lastMat = new THREE.Matrix4();
    const lastCol = new THREE.Color();
    if (i !== last) {
      entry.trunk.getMatrixAt(last, lastMat);
      entry.trunk.getColorAt(last, lastCol);
      entry.trunk.setMatrixAt(i, lastMat);
      entry.trunk.setColorAt(i, lastCol);
      entry.canopy.getMatrixAt(last, lastMat);
      entry.canopy.getColorAt(last, lastCol);
      entry.canopy.setMatrixAt(i, lastMat);
      entry.canopy.setColorAt(i, lastCol);
      entry.data[i] = entry.data[last];
    }
    entry.data.pop();
    entry.trunk.count = entry.data.length;
    entry.canopy.count = entry.data.length;
    for (const m of [entry.trunk, entry.canopy]) {
      m.instanceMatrix.needsUpdate = true;
      if (m.instanceColor) m.instanceColor.needsUpdate = true;
      m.computeBoundingSphere();
    }
  }

  function removeTree(x, z, radius = 6) {
    let best = null;
    let bestD = radius * radius;
    for (const entry of treeMeshes) {
      for (let i = 0; i < entry.data.length; i++) {
        const dx = entry.data[i].x - x;
        const dz = entry.data[i].z - z;
        const d = dx * dx + dz * dz;
        if (d < bestD) {
          bestD = d;
          best = { entry, i };
        }
      }
    }
    if (!best) return false;
    removeTreeByIndex(best.entry, best.i);
    return true;
  }

  function nearestTree(x, z, radius) {
    let best = null;
    let bestD = radius * radius;
    for (const entry of treeMeshes) {
      for (let i = 0; i < entry.data.length; i++) {
        const t = entry.data[i];
        const dx = t.x - x, dz = t.z - z;
        const d = dx * dx + dz * dz;
        if (d < bestD) { bestD = d; best = { entry, i, life: t }; }
      }
    }
    return best;
  }

  // 某位置树冠遮蔽度：近成熟树=阴（竞争光照，抑制下方树苗/草）
  function shadeAt(x, z) {
    let sh = 0;
    for (const entry of treeMeshes) {
      for (const t of entry.data) {
        if (t.stage < 2) continue;
        const dx = t.x - x, dz = t.z - z;
        const d2 = dx * dx + dz * dz;
        const cr = t.s * 3.4;
        if (d2 < cr * cr) sh += (1 - Math.sqrt(d2) / cr) * 0.4;
        if (sh >= 1) return 1;
      }
    }
    return Math.min(1, sh);
  }

  // 种子落点：避开水面/陡坡/低适生/地标/过密
  function trySeedTree(parent) {
    const LANDMARKS = [[281, 249, 26], [298, 229, 20], [262, 270, 22], [231, 258, 18]];
    for (let k = 0; k < 6; k++) {
      const ang = treeRng() * Math.PI * 2;
      const rr = 22 + treeRng() * 70;
      const nx = parent.x + Math.cos(ang) * rr;
      const nz = parent.z + Math.sin(ang) * rr;
      const h = terrainHeight(nx, nz);
      if (h < WATER_LEVEL + 1.6 || h > 112) continue;
      if (slopeAt(nx, nz) > 0.62) continue;
      if (plantSuitability(nx, nz) < 0.15) continue;
      let blocked = false;
      for (const [lx, lz, rad] of LANDMARKS) {
        const dx = nx - lx, dz = nz - lz;
        if (dx * dx + dz * dz < rad * rad) { blocked = true; break; }
      }
      if (blocked) continue;
      if (nearTree(nx, nz, 6 + parent.s * 2.2)) continue;
      if (placeTree(nx, nz, null, null, 0)) return;
    }
  }

  // 每生态步进 dt（模拟秒），推进植被自主演替；树生长速率受环境选择（树荫=光照竞争、适生=土壤）
  function updateTrees(dt) {
    if (dt <= 0) return;
    for (const entry of treeMeshes) {
      const data = entry.data;
      for (let i = 0; i < data.length; i++) {
        const life = data[i];
        const suit = plantSuitability(life.x, life.z);
        const shadeP = 1 - Math.min(1, shadeAt(life.x, life.z) * 0.85);
        life.age += dt * (0.35 + suit * 0.9) * shadeP; // 环境选择：肥沃+向阳快长，树荫下慢
        life.seedT -= dt;
        life.stage = life.age < 40 ? 0 : life.age < 120 ? 1 : life.age < 400 ? 2 : 3;
        if (life.stage >= 2 && life.seedT <= 0) {
          life.seedT = (22 + treeRng() * 22) / (0.5 + suit * 1.2); // 适生地种子更频（选择压力）
          if (treeRng() < 0.10) trySeedTree(life);
        }
        if (life.age > 500) {
          const p = (life.age - 500) / 400;
          if (treeRng() < p * dt * 0.12) { removeTreeByIndex(entry, i); i--; continue; }
        }
        const st = treeStageScale(life);
        if (Math.abs(st - life._lastSt) > 0.012) {
          life._lastSt = st;
          dummy.position.set(life.x, terrainHeight(life.x, life.z) - 0.15, life.z);
          dummy.rotation.set(0, life.rot, 0);
          dummy.scale.set(st, st * life.vy, st);
          dummy.updateMatrix();
          entry.trunk.setMatrixAt(i, dummy.matrix);
          entry.canopy.setMatrixAt(i, dummy.matrix);
          entry.trunk.instanceMatrix.needsUpdate = true;
          entry.canopy.instanceMatrix.needsUpdate = true;
        }
      }
    }
  }

  let placedTrees = 0;
  let tries = 0;
  const targetTrees = quality.treeCount;
  while (placedTrees < targetTrees && tries < targetTrees * 40) {
    tries++;
    const r = 310 + treeRng() * 420;
    const a = treeRng() * Math.PI * 2;
    const x = Math.cos(a) * r;
    const z = Math.sin(a) * r;
    if (plantSuitability(x, z) < 0.08 && treeRng() > 0.25) continue;
    if (placeTree(x, z)) placedTrees++;
  }
  // 远岸林带：成簇生长，避免排成一条线
  for (let c = 0; c < 18; c++) {
    const a = -2.9 + treeRng() * 1.55;
    const r = 360 + treeRng() * 190;
    const cx = Math.cos(a) * r;
    const cz = Math.sin(a) * r;
    const count = 3 + Math.floor(treeRng() * 4);
    for (let j = 0; j < count; j++) {
      const x = cx + (treeRng() - 0.5) * 58;
      const z = cz + (treeRng() - 0.5) * 58;
      if (placeTree(x, z, 1.15 + treeRng() * 0.85, null, 250 + treeRng() * 150)) placedTrees++;
    }
  }
  // 取景前景的树，特意留在相机两侧
  const grove = [
    [185, 278, 1.45, 0], [305, 275, 1.5, 1], [150, 245, 1.3, 2],
    [325, 250, 1.45, 0], [120, 290, 1.25, 0], [340, 240, 1.35, 1],
    [95, 320, 1.2, 2],
  ];
  for (const [x, z, s, v] of grove) {
    if (placeTree(x, z, s, v, 250 + treeRng() * 150)) placedTrees++;
  }

  // ---------- 岩石 ----------
  const rockGeo = makeRockGeometry(31);
  const rockMat = new THREE.MeshStandardMaterial({
    vertexColors: true,
    roughness: 0.96,
    metalness: 0.0,
    flatShading: true,
  });
  const rockCap = 120;
  const rocks = new THREE.InstancedMesh(rockGeo, rockMat, rockCap);
  rocks.castShadow = true;
  rocks.receiveShadow = true;
  const rockRng = mulberry32(515);
  const rockData = [];
  let ri = 0;
  let rockTries = 0;
  while (ri < rockCap && rockTries < rockCap * 12) {
    rockTries++;
    const r = 230 + rockRng() * 560;
    const a = rockRng() * Math.PI * 2;
    const x = Math.cos(a) * r;
    const z = Math.sin(a) * r;
    const h = terrainHeight(x, z);
    const s = slopeAt(x, z);
    if (h < WATER_LEVEL - 0.5 || (s < 0.28 && rockRng() > 0.18)) { continue; }
    const sc = 0.5 + Math.pow(rockRng(), 2.2) * 4.2;
    dummy.position.set(x, h + sc * 0.12, z);
    dummy.rotation.set(rockRng() * 0.6, rockRng() * Math.PI * 2, rockRng() * 0.6);
    dummy.scale.set(sc * (0.8 + rockRng() * 0.5), sc * (0.55 + rockRng() * 0.35), sc * (0.8 + rockRng() * 0.5));
    dummy.updateMatrix();
    rocks.setMatrixAt(ri, dummy.matrix);
    gColor.setHex(rockRng() > 0.5 ? 0xd0c7b7 : 0xafa79a);
    rocks.setColorAt(ri, gColor);
    rockData.push({ x, z });
    ri++;
  }
  rocks.count = ri;
  rocks.instanceMatrix.needsUpdate = true;
  if (rocks.instanceColor) rocks.instanceColor.needsUpdate = true;
  rocks.computeBoundingSphere();
  scene.add(rocks);

  function addRock(x, z) {
    if (ri >= rockCap) return false;
    const h = terrainHeight(x, z);
    if (h < WATER_LEVEL - 0.4 || slopeAt(x, z) > 1.2) return false;
    const sc = 0.5 + Math.pow(rockRng(), 2.2) * 4.2;
    dummy.position.set(x, h + sc * 0.12, z);
    dummy.rotation.set(rockRng() * 0.6, rockRng() * Math.PI * 2, rockRng() * 0.6);
    dummy.scale.set(sc * (0.8 + rockRng() * 0.5), sc * (0.55 + rockRng() * 0.35), sc * (0.8 + rockRng() * 0.5));
    dummy.updateMatrix();
    rocks.setMatrixAt(ri, dummy.matrix);
    gColor.setHex(rockRng() > 0.5 ? 0xd0c7b7 : 0xafa79a);
    rocks.setColorAt(ri, gColor);
    rockData.push({ x, z });
    ri++;
    rocks.count = ri;
    rocks.instanceMatrix.needsUpdate = true;
    if (rocks.instanceColor) rocks.instanceColor.needsUpdate = true;
    rocks.computeBoundingSphere();
    return true;
  }

  // ---------- 灌木 ----------
  const bushGeo = makeBushGeometry();
  const bushMat = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.95 });
  const bushCap = quality.bushCount;
  const bushes = new THREE.InstancedMesh(bushGeo, bushMat, bushCap);
  bushes.castShadow = true;
  bushes.receiveShadow = true;
  const bushRng = mulberry32(616);
  const bushData = [];
  let bi = 0;
  let bushTries = 0;
  while (bi < bushCap - 30 && bushTries < bushCap * 12) {
    bushTries++;
    const r = 170 + bushRng() * 430;
    const a = bushRng() * Math.PI * 2;
    const x = Math.cos(a) * r;
    const z = Math.sin(a) * r;
    const h = terrainHeight(x, z);
    if (h < WATER_LEVEL + 1.5 || h > 72 || slopeAt(x, z) > 0.55) { continue; }
    const s = 0.55 + bushRng() * 1.3;
    dummy.position.set(x, h, z);
    dummy.rotation.set(0, bushRng() * Math.PI * 2, 0);
    dummy.scale.set(s, s * (0.7 + bushRng() * 0.5), s);
    dummy.updateMatrix();
    bushes.setMatrixAt(bi, dummy.matrix);
    gColor.setHex(bushRng() > 0.5 ? 0x617a39 : 0x738b45);
    bushes.setColorAt(bi, gColor);
    bushData.push({ x, z });
    bi++;
  }
  bushes.count = bi;
  bushes.instanceMatrix.needsUpdate = true;
  if (bushes.instanceColor) bushes.instanceColor.needsUpdate = true;
  bushes.computeBoundingSphere();
  scene.add(bushes);

  function removeRockAt(i) {
    const last = ri - 1;
    if (i !== last) {
      rocks.getMatrixAt(last, dummy.matrix);
      rocks.setMatrixAt(i, dummy.matrix);
      if (rocks.instanceColor) {
        rocks.getColorAt(last, gColor);
        rocks.setColorAt(i, gColor);
      }
      rockData[i] = rockData[last];
    }
    rockData.pop();
    ri--;
    rocks.count = ri;
    rocks.instanceMatrix.needsUpdate = true;
    if (rocks.instanceColor) rocks.instanceColor.needsUpdate = true;
    rocks.computeBoundingSphere();
  }

  function addBush(x, z) {
    const h = terrainHeight(x, z);
    if (bi >= bushCap) return false;
    const s = 0.55 + Math.random() * 0.6;
    dummy.position.set(x, h, z);
    dummy.rotation.set(0, Math.random() * Math.PI * 2, 0);
    dummy.scale.set(s, s * 0.9, s);
    dummy.updateMatrix();
    bushes.setMatrixAt(bi, dummy.matrix);
    gColor.setHex(Math.random() > 0.5 ? 0x617a39 : 0x738b45);
    bushes.setColorAt(bi, gColor);
    bushData.push({ x, z });
    bi++;
    bushes.count = bi;
    bushes.instanceMatrix.needsUpdate = true;
    if (bushes.instanceColor) bushes.instanceColor.needsUpdate = true;
    bushes.computeBoundingSphere();
    return true;
  }

  function removeBushAt(i) {
    const last = bi - 1;
    if (i !== last) {
      bushes.getMatrixAt(last, dummy.matrix);
      bushes.setMatrixAt(i, dummy.matrix);
      if (bushes.instanceColor) {
        bushes.getColorAt(last, gColor);
        bushes.setColorAt(i, gColor);
      }
      bushData[i] = bushData[last];
    }
    bushData.pop();
    bi--;
    bushes.count = bi;
    bushes.instanceMatrix.needsUpdate = true;
    if (bushes.instanceColor) bushes.instanceColor.needsUpdate = true;
    bushes.computeBoundingSphere();
  }

  function nearestRock(x, z, radius) {
    let best = -1, bestD = radius * radius;
    for (let i = 0; i < rockData.length; i++) {
      const dx = rockData[i].x - x, dz = rockData[i].z - z;
      const d = dx * dx + dz * dz;
      if (d < bestD) { bestD = d; best = i; }
    }
    return best >= 0 ? { i: best, x: rockData[best].x, z: rockData[best].z } : null;
  }

  function nearestBush(x, z, radius) {
    let best = -1, bestD = radius * radius;
    for (let i = 0; i < bushData.length; i++) {
      const dx = bushData[i].x - x, dz = bushData[i].z - z;
      const d = dx * dx + dz * dz;
      if (d < bestD) { bestD = d; best = i; }
    }
    return best >= 0 ? { i: best, x: bushData[best].x, z: bushData[best].z } : null;
  }

  return {
    terrain,
    grass,
    grassMaterial: grassMat,
    rocks,
    bushes,
    treeMeshes,
    treeData,
    addTree: placeTree,
    removeTree,
    nearestTree,
    shadeAt,
    addRock,
    nearestRock,
    nearestBush,
    addBush,
    removeRockAt,
    removeBushAt,
    updateTrees,
    updateGrassVisual,
    treeCount: () => treeMeshes.reduce((n, t) => n + t.data.length, 0),
    update: (elapsed) => {
      wind.value = elapsed;
    },
  };
}

// ---------- 几何构建 ----------
function buildGrassGeometry() {
  const parts = [];
  for (let i = 0; i < 2; i++) {
    const g = new THREE.PlaneGeometry(1, 1, 1, 1);
    g.translate(0, 0.5, 0);
    g.rotateY((i / 2) * Math.PI);
    parts.push(g);
  }
  return mergeGeometries(parts, false);
}

function colorizeGeometry(geo, color, variation = 0.05, seed = 1) {
  const count = geo.attributes.position.count;
  const arr = new Float32Array(count * 3);
  const c = new THREE.Color(color);
  const rng = mulberry32(seed);
  for (let i = 0; i < count; i++) {
    const v = 1 + (rng() - 0.5) * variation * 2;
    arr[i * 3] = c.r * v;
    arr[i * 3 + 1] = c.g * v;
    arr[i * 3 + 2] = c.b * v;
  }
  geo.setAttribute('color', new THREE.BufferAttribute(arr, 3));
  return geo;
}

function addPart(parts, geo, color, variation, seed) {
  colorizeGeometry(geo, color, variation, seed);
  parts.push(geo);
}

function branchMatrix(x, y, z, angleY, tilt, length) {
  const m = new THREE.Matrix4();
  const q = new THREE.Quaternion().setFromEuler(new THREE.Euler(0, angleY, tilt, 'YXZ'));
  const pos = new THREE.Vector3(x, y, z).add(
    new THREE.Vector3(0, length / 2, 0).applyQuaternion(q)
  );
  m.compose(pos, q, new THREE.Vector3(1, 1, 1));
  return m;
}

function makeFoliageTexture() {
  const size = 256;
  const rng = mulberry32(909);
  const ops = [];
  for (let i = 0; i < 64; i++) {
    const cx = rng() * size;
    const cy = rng() * size;
    const n = 2 + Math.floor(rng() * 5);
    for (let j = 0; j < n; j++) {
      ops.push({
        x: cx + (rng() - 0.5) * 24,
        y: cy + (rng() - 0.5) * 18,
        rw: 3 + rng() * 9,
        rh: 2.5 + rng() * 7,
        ang: rng() * Math.PI,
        shade: 0.5 + rng() * 0.5,
      });
    }
  }
  const albedo = document.createElement('canvas');
  albedo.width = albedo.height = size;
  const actx = albedo.getContext('2d');
  const heightC = document.createElement('canvas');
  heightC.width = heightC.height = size;
  const hctx = heightC.getContext('2d');
  actx.clearRect(0, 0, size, size);
  hctx.fillStyle = '#000';
  hctx.fillRect(0, 0, size, size);
  for (const o of ops) {
    const g = Math.min(255, Math.round(96 * o.shade + 46));
    const r = Math.min(255, Math.round(g * 1.06));
    const b = Math.min(255, Math.round(g * 0.7));
    const xs = [o.x];
    const ys = [o.y];
    if (o.x < 20) xs.push(o.x + size);
    if (o.x > size - 20) xs.push(o.x - size);
    if (o.y < 20) ys.push(o.y + size);
    if (o.y > size - 20) ys.push(o.y - size);
    for (const xx of xs) {
      for (const yy of ys) {
        actx.save();
        actx.translate(xx, yy);
        actx.rotate(o.ang);
        actx.fillStyle = `rgba(${r},${g},${b},0.97)`;
        actx.beginPath();
        actx.ellipse(0, 0, o.rw, o.rh, 0, 0, Math.PI * 2);
        actx.fill();
        actx.restore();
        hctx.save();
        hctx.translate(xx, yy);
        hctx.rotate(o.ang);
        hctx.fillStyle = 'rgba(190,190,190,0.9)';
        hctx.beginPath();
        hctx.ellipse(0, 0, o.rw, o.rh, 0, 0, Math.PI * 2);
        hctx.fill();
        hctx.restore();
      }
    }
  }
  const himg = hctx.getImageData(0, 0, size, size);
  const hd = new Float32Array(size * size);
  for (let i = 0; i < size * size; i++) {
    hd[i] = (himg.data[i * 4] + himg.data[i * 4 + 1] + himg.data[i * 4 + 2]) / 765;
  }
  const normal = document.createElement('canvas');
  normal.width = normal.height = size;
  const nctx = normal.getContext('2d');
  const nimg = nctx.createImageData(size, size);
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const i = (y * size + x) * 4;
      const l = hd[y * size + ((x - 1 + size) % size)];
      const r = hd[y * size + ((x + 1) % size)];
      const d = hd[((y - 1 + size) % size) * size + x];
      const u = hd[((y + 1) % size) * size + x];
      let nx = (l - r) * 5.0;
      let ny = (d - u) * 5.0;
      const len = Math.hypot(nx, ny, 1);
      nx /= len;
      ny /= len;
      nimg.data[i] = Math.round((nx * 0.5 + 0.5) * 255);
      nimg.data[i + 1] = Math.round((ny * 0.5 + 0.5) * 255);
      nimg.data[i + 2] = Math.round((0.5 + 0.5 / len) * 255);
      nimg.data[i + 3] = 255;
    }
  }
  nctx.putImageData(nimg, 0, 0);
  const albedoTex = new THREE.CanvasTexture(albedo);
  albedoTex.colorSpace = THREE.SRGBColorSpace;
  albedoTex.wrapS = albedoTex.wrapT = THREE.RepeatWrapping;
  const normalTex = new THREE.CanvasTexture(normal);
  normalTex.wrapS = normalTex.wrapT = THREE.RepeatWrapping;
  return { albedo: albedoTex, normal: normalTex };
}

function makeBroadleaf() {
  const trunkParts = [];
  const canopyParts = [];
  let seed = 11;
  const trunk = new THREE.CylinderGeometry(0.32, 0.55, 7.2, 7, 1);
  trunk.translate(0, 3.6, 0);
  addPart(trunkParts, trunk, 0x5a4631, 0.13, seed++);

  for (let i = 0; i < 4; i++) {
    const b = new THREE.CylinderGeometry(0.09, 0.2, 3.4, 5, 1);
    b.applyMatrix4(branchMatrix(0, 3.5 + i * 0.6, 0, i * 1.7, 0.55 + (i % 2) * 0.2, 3.4));
    addPart(trunkParts, b, 0x523f2b, 0.1, seed++);
  }

  const leafColors = [0x3a5026, 0x46602c, 0x506b32, 0x31481f, 0x5a7438];
  const blobs = [
    [0, 7.6, 0, 3.8, 2.5, 3.8],
    [2.6, 7.3, 0.8, 2.8, 2.1, 2.8],
    [-2.4, 7.5, -0.9, 2.9, 2.0, 2.7],
    [0.9, 9.2, -2.0, 2.5, 2.0, 2.6],
    [-1.5, 9.0, 1.8, 2.4, 1.9, 2.5],
    [1.8, 9.1, 2.1, 2.1, 1.8, 2.2],
    [-0.4, 10.6, 0.3, 2.0, 1.7, 2.0],
    [2.9, 8.2, -1.5, 1.8, 1.6, 1.8],
    [-2.9, 8.4, 1.1, 1.7, 1.5, 1.7],
    [0.0, 6.7, -2.4, 2.2, 1.7, 2.1],
  ];
  for (let i = 0; i < blobs.length; i++) {
    const [x, y, z, sx, sy, sz] = blobs[i];
    const g = new THREE.SphereGeometry(1, 8, 6);
    g.scale(sx, sy, sz);
    g.translate(x, y, z);
    addPart(canopyParts, g, leafColors[i % leafColors.length], 0.16, seed++);
  }
  return {
    trunkGeo: mergeGeometries(trunkParts, false),
    canopyGeo: mergeGeometries(canopyParts, false),
  };
}

function makeConifer() {
  const trunkParts = [];
  const canopyParts = [];
  let seed = 41;
  const trunk = new THREE.CylinderGeometry(0.25, 0.48, 12.5, 7, 1);
  trunk.translate(0, 6.25, 0);
  addPart(trunkParts, trunk, 0x4a3a29, 0.12, seed++);
  const layers = [
    [3.8, 4.4, 4.1],
    [3.2, 4.0, 6.1],
    [2.5, 3.7, 8.0],
    [1.8, 3.4, 9.8],
    [1.1, 3.0, 11.4],
  ];
  const cols = [0x21381e, 0x294224, 0x304d29];
  for (let i = 0; i < layers.length; i++) {
    const [r, h, y] = layers[i];
    const g = new THREE.ConeGeometry(r, h, 9, 1);
    g.translate(0, y, 0);
    addPart(canopyParts, g, cols[i % 3], 0.15, seed++);
  }
  return {
    trunkGeo: mergeGeometries(trunkParts, false),
    canopyGeo: mergeGeometries(canopyParts, false),
  };
}

function makeBirch() {
  const trunkParts = [];
  const canopyParts = [];
  let seed = 71;
  const trunk = new THREE.CylinderGeometry(0.14, 0.28, 7.8, 6, 1);
  trunk.translate(0, 3.9, 0);
  addPart(trunkParts, trunk, 0x948b76, 0.1, seed++);
  for (let i = 0; i < 3; i++) {
    const b = new THREE.CylinderGeometry(0.05, 0.11, 2.8, 4, 1);
    b.applyMatrix4(branchMatrix(0, 4.2 + i * 0.6, 0, i * 2.1, 0.62, 2.8));
    addPart(trunkParts, b, 0x8a8270, 0.1, seed++);
  }
  const blobs = [
    [0, 7.4, 0, 2.4, 2.0, 2.3],
    [1.5, 7.1, 0.5, 1.9, 1.5, 1.8],
    [-1.4, 7.2, -0.4, 1.8, 1.4, 1.7],
    [0.3, 8.7, -0.4, 1.7, 1.5, 1.6],
    [-0.4, 8.8, 0.9, 1.5, 1.2, 1.4],
  ];
  const cols = [0x667c36, 0x738a3f, 0x58702d];
  for (let i = 0; i < blobs.length; i++) {
    const [x, y, z, sx, sy, sz] = blobs[i];
    const g = new THREE.SphereGeometry(1, 8, 6);
    g.scale(sx, sy, sz);
    g.translate(x, y, z);
    addPart(canopyParts, g, cols[i % 3], 0.18, seed++);
  }
  return {
    trunkGeo: mergeGeometries(trunkParts, false),
    canopyGeo: mergeGeometries(canopyParts, false),
  };
}

function makeRockGeometry(seed) {
  const geo = new THREE.IcosahedronGeometry(1, 1);
  const pos = geo.attributes.position;
  const rng = mulberry32(seed);
  const cache = new Map();
  for (let i = 0; i < pos.count; i++) {
    const x = pos.getX(i), y = pos.getY(i), z = pos.getZ(i);
    const key = `${x.toFixed(3)},${y.toFixed(3)},${z.toFixed(3)}`;
    let r = cache.get(key);
    if (r === undefined) {
      r = 0.78 + rng() * 0.4;
      cache.set(key, r);
    }
    pos.setXYZ(i, x * r, y * r * 0.72, z * r);
  }
  geo.computeVertexNormals();
  const count = pos.count;
  const arr = new Float32Array(count * 3);
  const c = new THREE.Color(0xffffff);
  const rng2 = mulberry32(seed + 1);
  for (let i = 0; i < count; i++) {
    const v = 0.62 + rng2() * 0.35;
    arr[i * 3] = c.r * v;
    arr[i * 3 + 1] = c.g * v;
    arr[i * 3 + 2] = c.b * v;
  }
  geo.setAttribute('color', new THREE.BufferAttribute(arr, 3));
  return geo;
}

function makeBushGeometry() {
  const parts = [];
  let seed = 91;
  const blobs = [
    [0, 0.75, 0, 1.1, 0.85, 1.0],
    [0.75, 0.6, 0.2, 0.8, 0.68, 0.75],
    [-0.65, 0.55, -0.2, 0.85, 0.6, 0.8],
    [0.1, 1.35, 0.1, 0.7, 0.6, 0.7],
  ];
  const cols = [0x3d5222, 0x4b6129, 0x596f31];
  for (let i = 0; i < blobs.length; i++) {
    const [x, y, z, sx, sy, sz] = blobs[i];
    const g = new THREE.SphereGeometry(1, 6, 4);
    g.scale(sx, sy, sz);
    g.translate(x, y, z);
    addPart(parts, g, cols[i % 3], 0.18, seed++);
  }
  return mergeGeometries(parts, false);
}
