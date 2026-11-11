import * as THREE from 'three';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';
import { HERB, PRED, SCAV, OMN } from './ecology.js';
import { terrainHeight, clamp, mulberry32 } from './world.js';

function paint(geo, color, variation = 0.06, seed = 1) {
  const n = geo.attributes.position.count;
  const arr = new Float32Array(n * 3);
  const c = new THREE.Color(color);
  const rng = mulberry32(seed);
  for (let i = 0; i < n; i++) {
    const v = 1 + (rng() - 0.5) * variation * 2;
    arr[i * 3] = c.r * v;
    arr[i * 3 + 1] = c.g * v;
    arr[i * 3 + 2] = c.b * v;
  }
  geo.setAttribute('color', new THREE.BufferAttribute(arr, 3));
  return geo;
}

function add(parts, geo, color, variation, seed) {
  paint(geo, color, variation, seed);
  parts.push(geo);
}

const UP = new THREE.Vector3(0, 1, 0);

function limb(parts, p0, p1, r0, r1, color, seed, radial = 5) {
  const dir = new THREE.Vector3().subVectors(p1, p0);
  const len = dir.length();
  if (len < 0.0001) return;
  const geo = new THREE.CylinderGeometry(r1, r0, len, radial, 1, true);
  geo.translate(0, len / 2, 0);
  const q = new THREE.Quaternion().setFromUnitVectors(UP, dir.clone().normalize());
  const m = new THREE.Matrix4().makeRotationFromQuaternion(q);
  m.setPosition(p0);
  geo.applyMatrix4(m);
  add(parts, geo, color, 0.08, seed);
}

function ball(parts, x, y, z, sx, sy, sz, color, seed, ws = 8, hs = 6) {
  const g = new THREE.SphereGeometry(1, ws, hs);
  g.scale(sx, sy, sz);
  g.translate(x, y, z);
  add(parts, g, color, 0.08, seed);
}

function makeHerbivoreGeometry() {
  const parts = [];
  let seed = 301;
  ball(parts, 0, 1.05, -0.05, 0.5, 0.5, 0.9, 0xb18a5f, seed++, 9, 7);
  ball(parts, 0, 1.08, -0.62, 0.44, 0.46, 0.42, 0xa47c53, seed++);
  ball(parts, 0, 1.3, 0.45, 0.3, 0.35, 0.34, 0xb99264, seed++);
  limb(parts, new THREE.Vector3(0, 1.35, 0.54), new THREE.Vector3(0, 1.58, 0.98), 0.2, 0.15, 0xb99264, seed++);
  ball(parts, 0, 1.6, 1.08, 0.26, 0.26, 0.34, 0xc39d6c, seed++, 8, 6);
  ball(parts, 0, 1.5, 1.34, 0.16, 0.16, 0.24, 0x8e6a45, seed++, 7, 5);
  ball(parts, 0.19, 1.58, 1.17, 0.07, 0.07, 0.07, 0x8a5f3a, seed++);
  ball(parts, -0.19, 1.58, 1.17, 0.07, 0.07, 0.07, 0x8a5f3a, seed++);
  ball(parts, 0.1, 1.64, 1.3, 0.035, 0.035, 0.035, 0x221a12, seed++, 6, 4);
  ball(parts, -0.1, 1.64, 1.3, 0.035, 0.035, 0.035, 0x221a12, seed++, 6, 4);

  // 耳朵
  for (const side of [-1, 1]) {
    const ear = new THREE.ConeGeometry(0.11, 0.34, 5, 1);
    ear.rotateZ(side * 0.55);
    ear.translate(side * 0.23, 1.82, 1.02);
    add(parts, ear, 0x9e7850, 0.1, seed++);
  }

  // 四肢
  const legs = [
    [0.3, 0.55], [-0.3, 0.55], [0.3, -0.62], [-0.3, -0.62],
  ];
  for (const [x, z] of legs) {
    limb(parts, new THREE.Vector3(x, 0.98, z), new THREE.Vector3(x, 0.09, z + (z > 0 ? 0.04 : -0.02)), 0.1, 0.06, 0x8c6846, seed++);
    ball(parts, x, 0.06, z, 0.09, 0.07, 0.12, 0x5d4430, seed++, 6, 4);
  }

  // 尾巴
  limb(parts, new THREE.Vector3(0, 1.2, -0.94), new THREE.Vector3(0, 0.92, -1.16), 0.07, 0.02, 0xb0895c, seed++);
  ball(parts, 0, 0.9, -1.17, 0.09, 0.12, 0.09, 0xe3d6bd, seed++);

  // 鹿角
  for (const side of [-1, 1]) {
    const base = new THREE.Vector3(side * 0.12, 1.78, 1.02);
    const mid = new THREE.Vector3(side * 0.3, 2.22, 0.92);
    const tip = new THREE.Vector3(side * 0.42, 2.58, 0.78);
    limb(parts, base, mid, 0.045, 0.03, 0xd8c9a8, seed++, 4);
    limb(parts, mid, tip, 0.03, 0.012, 0xd8c9a8, seed++, 4);
    limb(parts, mid, new THREE.Vector3(side * 0.5, 2.31, 1.12), 0.022, 0.01, 0xd8c9a8, seed++, 4);
  }

  return mergeGeometries(parts, false);
}

function makePredatorGeometry() {
  const parts = [];
  let seed = 501;
  ball(parts, 0, 0.8, -0.08, 0.39, 0.4, 0.88, 0x59524b, seed++, 9, 7);
  ball(parts, 0, 0.86, 0.5, 0.38, 0.42, 0.36, 0x625a52, seed++);
  ball(parts, 0, 0.98, -0.55, 0.33, 0.36, 0.32, 0x4e4842, seed++);
  limb(parts, new THREE.Vector3(0, 0.98, 0.58), new THREE.Vector3(0, 1.14, 0.98), 0.18, 0.14, 0x625a52, seed++);
  ball(parts, 0, 1.14, 1.08, 0.26, 0.25, 0.33, 0x6b6258, seed++, 8, 6);
  ball(parts, 0, 1.06, 1.34, 0.17, 0.15, 0.26, 0x3f3a35, seed++, 7, 5);
  ball(parts, 0.27, 1.21, 1.06, 0.07, 0.07, 0.07, 0x6d655c, seed++);
  ball(parts, -0.27, 1.21, 1.06, 0.07, 0.07, 0.07, 0x6d655c, seed++);
  ball(parts, 0.09, 1.16, 1.3, 0.035, 0.035, 0.035, 0x16120e, seed++, 6, 4);
  ball(parts, -0.09, 1.16, 1.3, 0.035, 0.035, 0.035, 0x16120e, seed++, 6, 4);

  for (const side of [-1, 1]) {
    const ear = new THREE.ConeGeometry(0.1, 0.28, 5, 1);
    ear.rotateZ(side * 0.35);
    ear.translate(side * 0.2, 1.38, 1.02);
    add(parts, ear, 0x514a43, 0.1, seed++);
  }
  const legs = [
    [0.25, 0.52], [-0.25, 0.52], [0.25, -0.62], [-0.25, -0.62],
  ];
  for (const [x, z] of legs) {
    limb(parts, new THREE.Vector3(x, 0.75, z), new THREE.Vector3(x, 0.08, z + (z > 0 ? 0.03 : -0.03)), 0.085, 0.05, 0x4c453f, seed++);
    ball(parts, x, 0.055, z, 0.075, 0.06, 0.1, 0x2f2a25, seed++, 6, 4);
  }
  limb(parts, new THREE.Vector3(0, 0.92, -0.9), new THREE.Vector3(0, 0.72, -1.58), 0.075, 0.025, 0x514a43, seed++);
  ball(parts, 0, 0.7, -1.6, 0.08, 0.1, 0.08, 0x3b3630, seed++);

  return mergeGeometries(parts, false);
}

function shortestAngle(a, b) {
  let d = b - a;
  while (d > Math.PI) d -= Math.PI * 2;
  while (d < -Math.PI) d += Math.PI * 2;
  return d;
}

export function createAgentRenderer(scene, ecology, maxHerb = 480, maxPred = 140) {
  const herbGeo = makeHerbivoreGeometry();
  const predGeo = makePredatorGeometry();
  const herbMat = new THREE.MeshStandardMaterial({
    vertexColors: true,
    roughness: 0.78,
    metalness: 0,
  });
  const predMat = new THREE.MeshStandardMaterial({
    vertexColors: true,
    roughness: 0.72,
    metalness: 0,
  });
  const herbs = new THREE.InstancedMesh(herbGeo, herbMat, maxHerb);
  const preds = new THREE.InstancedMesh(predGeo, predMat, maxPred);
  for (const m of [herbs, preds]) {
    m.castShadow = true;
    m.receiveShadow = true;
    m.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
    m.frustumCulled = false;
    m.count = 0;
    scene.add(m);
  }

  const dummy = new THREE.Object3D();
  const pos = new THREE.Vector3();
  const scale = new THREE.Vector3();
  const quat = new THREE.Quaternion();
  const euler = new THREE.Euler(0, 0, 0, 'YXZ');
  const color = new THREE.Color();

  const maps = { herb: [], pred: [], scav: [], omn: [] };
  let herbCount = 0;
  let predCount = 0;

  function update(alpha, elapsed) {
    const agents = ecology.living();
    herbCount = 0;
    predCount = 0;
    maps.herb.length = 0;
    maps.pred.length = 0;
    maps.scav.length = 0;
    maps.omn.length = 0;
    for (let i = 0; i < agents.length; i++) {
      const a = agents[i];
      if (!a.alive) continue;
      const t = clamp(alpha, 0, 1);
      const x = a.prevX + (a.x - a.prevX) * t;
      const z = a.prevZ + (a.z - a.prevZ) * t;
      const heading = a.prevHeading + shortestAngle(a.prevHeading, a.heading) * t;
      const y = terrainHeight(x, z);
      const g = a.genome;
      const s = g.size;
      const moving = Math.min(1, a.activity);
      const bob = Math.sin(elapsed * (5.5 + g.speed * 0.7) + a.phase) * 0.055 * s * moving;
      const pitch = 0;
      dummy.position.set(x, y + bob, z);
      dummy.rotation.set(pitch, heading, Math.sin(a.phase + elapsed * 2.0) * 0.012 * moving, 'YXZ');
      dummy.scale.set(s, s, s);
      dummy.updateMatrix();
      if (a.kind === HERB) {
        herbs.setMatrixAt(herbCount, dummy.matrix);
        const tint = 0.94 + (g.hue - 0.5) * 0.12;
        color.setRGB(tint, tint * 0.96, tint * 0.9);
        herbs.setColorAt(herbCount, color);
        maps.herb.push(a);
        herbCount++;
      } else {
        // P/SCAV/OMN 共用 pred 几何，色调区分物种
        preds.setMatrixAt(predCount, dummy.matrix);
        let r, gg, b2;
        if (a.kind === PRED) { r = 0.90 + (g.hue - 0.5) * 0.08; gg = r * 0.97; b2 = r * 0.93; }
        else if (a.kind === SCAV) { r = 0.72 + (g.hue - 0.5) * 0.06; gg = 0.68 + (g.hue - 0.5) * 0.06; b2 = 0.58; } // 暗褐
        else { r = 0.78; gg = 0.86 + (g.hue - 0.5) * 0.06; b2 = 0.72; } // 淡绿（杂食）
        color.setRGB(r, gg, b2);
        preds.setColorAt(predCount, color);
        if (a.kind === PRED) maps.pred.push(a);
        else if (a.kind === SCAV) maps.scav.push(a);
        else maps.omn.push(a);
        predCount++;
      }
    }
    herbs.count = herbCount;
    preds.count = predCount;
    herbs.instanceMatrix.needsUpdate = true;
    preds.instanceMatrix.needsUpdate = true;
    if (herbs.instanceColor) herbs.instanceColor.needsUpdate = true;
    if (preds.instanceColor) preds.instanceColor.needsUpdate = true;
  }

  return {
    update,
    counts: () => ({ h: herbCount, p: predCount }),
    meshes: { herbs, preds },
    maps,
  };
}
