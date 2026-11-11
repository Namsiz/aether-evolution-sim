import * as THREE from 'three';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';
import { terrainHeight, mulberry32 } from './world.js';

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

function add(parts, geo, color, variation = 0.05, seed = 1) {
  paint(geo, color, variation, seed);
  parts.push(geo);
}

function box(parts, w, h, d, x, y, z, color, seed, ry = 0) {
  const g = new THREE.BoxGeometry(w, h, d);
  if (ry) g.rotateY(ry);
  g.translate(x, y, z);
  add(parts, g, color, 0.04, seed);
}

function makeCabin() {
  const parts = [];
  let seed = 21;
  box(parts, 8.6, 0.7, 8.6, 0, 0.35, 0, 0x9a9385, seed++);
  box(parts, 7.6, 4.4, 7.6, 0, 2.8, 0, 0xf0e7d4, seed++);
  for (const sx of [-1, 1]) {
    for (const sz of [-1, 1]) {
      box(parts, 0.32, 4.5, 0.32, sx * 3.7, 2.8, sz * 3.7, 0x8a6a48, seed++);
    }
  }
  box(parts, 1.5, 2.4, 0.16, 0, 1.65, 3.86, 0x6b4a2e, seed++);
  box(parts, 1.9, 0.26, 1.2, 0, 0.55, 4.2, 0xb9a887, seed++);
  // 坡屋顶：两块斜板，蓝色，呼应参考作品的蓝色屋顶
  const roofL = new THREE.BoxGeometry(5.4, 0.42, 8.9);
  roofL.rotateZ(0.345);
  roofL.translate(-2.35, 5.5, 0);
  add(parts, roofL, 0x3f6f8f, 0.04, seed++);
  const roofR = new THREE.BoxGeometry(5.4, 0.42, 8.9);
  roofR.rotateZ(-0.345);
  roofR.translate(2.35, 5.5, 0);
  add(parts, roofR, 0x3f6f8f, 0.04, seed++);
  box(parts, 0.5, 0.34, 9.0, 0, 6.28, 0, 0x2f5670, seed++);
  box(parts, 0.9, 2.3, 0.9, 2.3, 6.6, 2.1, 0x9d7d5c, seed++);
  box(parts, 1.0, 0.3, 1.0, 2.3, 7.75, 2.1, 0x6f5942, seed++);
  const merged = mergeGeometries(parts, false);
  const mat = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.86, metalness: 0 });
  const mesh = new THREE.Mesh(merged, mat);
  mesh.castShadow = true;
  mesh.receiveShadow = true;

  const group = new THREE.Group();
  group.add(mesh);
  const glow = new THREE.MeshBasicMaterial({ color: 0xffc46b });
  const winGeo = new THREE.BoxGeometry(1.25, 1.25, 0.09);
  for (const x of [-2.05, 2.05]) {
    const w = new THREE.Mesh(winGeo, glow);
    w.position.set(x, 3.05, 3.86);
    group.add(w);
  }
  const sideWin = new THREE.Mesh(new THREE.BoxGeometry(0.09, 1.2, 1.2), glow);
  sideWin.position.set(3.86, 3.05, -1.4);
  group.add(sideWin);
  return group;
}

function shorelinePoint(angle) {
  const ca = Math.cos(angle);
  const sa = Math.sin(angle);
  let lo = 70;
  let hi = 360;
  for (let k = 0; k < 20; k++) {
    const mid = (lo + hi) / 2;
    if (terrainHeight(ca * mid, sa * mid) > 0.55) lo = mid;
    else hi = mid;
  }
  return { x: ca * hi, z: sa * hi, r: hi, ca, sa };
}

function makePier(angle) {
  const parts = [];
  let seed = 61;
  const length = 22;
  const halfW = 1.7;
  for (let i = 0; i < 13; i++) {
    box(parts, 3.5, 0.16, 1.25, 0, 0.56, 1.0 + i * 1.62, i % 2 ? 0x97744f : 0x8a6847, seed++);
  }
  box(parts, 0.34, 0.34, length, -1.35, 0.42, length / 2, 0x6d4f34, seed++);
  box(parts, 0.34, 0.34, length, 1.35, 0.42, length / 2, 0x6d4f34, seed++);
  for (const z of [3, 9, 15, 21]) {
    for (const x of [-1.45, 1.45]) {
      box(parts, 0.3, 2.9, 0.3, x, -0.9, z, 0x5e4530, seed++);
    }
  }
  const merged = mergeGeometries(parts, false);
  const mat = new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.92, metalness: 0 });
  const pier = new THREE.Mesh(merged, mat);
  pier.castShadow = true;
  pier.receiveShadow = true;

  const group = new THREE.Group();
  group.add(pier);

  // 小船
  const boatParts = [];
  let bs = 71;
  box(boatParts, 1.5, 0.55, 3.8, 0, 0.35, 0, 0x7a4f30, bs++);
  box(boatParts, 1.85, 0.22, 4.1, 0, 0.66, 0, 0xb0865a, bs++);
  box(boatParts, 1.1, 0.12, 1.2, 0, 0.62, 0.9, 0x5e3d25, bs++);
  box(boatParts, 1.1, 0.12, 1.2, 0, 0.62, -0.9, 0x5e3d25, bs++);
  const boat = new THREE.Mesh(
    mergeGeometries(boatParts, false),
    new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.88, metalness: 0 })
  );
  boat.castShadow = true;
  boat.position.set(3.1, 0.05, 12);
  boat.rotation.y = 0.08;
  group.add(boat);

  const shore = shorelinePoint(angle);
  const yaw = Math.atan2(-shore.ca, -shore.sa);
  group.position.set(shore.x - shore.ca * 2.0, 0, shore.z - shore.sa * 2.0);
  group.rotation.y = yaw;
  return group;
}

function makeSteppingStones(from, to, count = 12) {
  const parts = [];
  let seed = 91;
  const rng = mulberry32(seed);
  for (let i = 0; i < count; i++) {
    const t = (i + 0.5) / count;
    const x = from.x + (to.x - from.x) * t + (rng() - 0.5) * 2.2;
    const z = from.z + (to.z - from.z) * t + (rng() - 0.5) * 2.2;
    const h = terrainHeight(x, z);
    if (h < 0.5) continue;
    const g = new THREE.CylinderGeometry(0.7 + rng() * 0.35, 0.8 + rng() * 0.3, 0.18, 7);
    g.translate(x, h + 0.08, z);
    add(parts, g, rng() > 0.5 ? 0xb3ac9b : 0x9a9384, 0.08, seed++);
  }
  if (!parts.length) return null;
  const mesh = new THREE.Mesh(
    mergeGeometries(parts, false),
    new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.95 })
  );
  mesh.receiveShadow = true;
  return mesh;
}

function makeClouds(scene) {
  const rng = mulberry32(2026);
  const mat = new THREE.MeshStandardMaterial({
    color: 0xffffff,
    roughness: 1,
    metalness: 0,
    flatShading: true,
    emissive: 0x8fa4bb,
    emissiveIntensity: 0.18,
    fog: false,
  });
  const clouds = [];
  for (let i = 0; i < 14; i++) {
    const parts = [];
    const blobs = 3 + Math.floor(rng() * 3);
    for (let j = 0; j < blobs; j++) {
      const g = new THREE.SphereGeometry(1, 7, 5);
      const sx = 3.5 + rng() * 4;
      const sy = 2.5 + rng() * 2;
      const sz = 3 + rng() * 4;
      g.scale(sx, sy, sz);
      g.translate((rng() - 0.5) * 18, (rng() - 0.5) * 4, (rng() - 0.5) * 14);
      parts.push(g);
    }
    const geo = mergeGeometries(parts, false);
    const mesh = new THREE.Mesh(geo, mat);
    const a = -0.8 + rng() * 2.4;
    const r = 480 + rng() * 320;
    mesh.position.set(Math.cos(a) * r, 260 + rng() * 110, Math.sin(a) * r);
    mesh.rotation.y = rng() * Math.PI;
    mesh.castShadow = false;
    mesh.receiveShadow = false;
    mesh.renderOrder = 4;
    scene.add(mesh);
    clouds.push({ mesh, speed: 0.5 + rng() * 0.8, phase: rng() * Math.PI * 2 });
  }
  return clouds;
}

function makeWindmill() {
  const parts = [];
  let seed = 111;
  const tower = new THREE.CylinderGeometry(1.05, 2.3, 13, 8);
  tower.translate(0, 6.5, 0);
  add(parts, tower, 0xf0ead9, 0.05, seed++);
  const base = new THREE.CylinderGeometry(2.6, 2.85, 1.0, 8);
  base.translate(0, 0.5, 0);
  add(parts, base, 0xc9bfa8, 0.05, seed++);
  const cap = new THREE.ConeGeometry(1.55, 2.4, 8);
  cap.translate(0, 14.2, 0);
  add(parts, cap, 0xb14a3a, 0.05, seed++);
  const mesh = new THREE.Mesh(
    mergeGeometries(parts, false),
    new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.85, metalness: 0 })
  );
  mesh.castShadow = true;
  mesh.receiveShadow = true;
  const group = new THREE.Group();
  group.add(mesh);
  // 四片风叶，可整体绕 z 轴旋转（面向镜头）
  const blades = new THREE.Group();
  const bladeMat = new THREE.MeshStandardMaterial({ color: 0xf2ead8, roughness: 0.8 });
  for (let i = 0; i < 4; i++) {
    const b = new THREE.Mesh(new THREE.BoxGeometry(0.55, 5.4, 0.2), bladeMat);
    b.position.y = 3.0;
    const holder = new THREE.Group();
    holder.add(b);
    holder.rotation.z = (i * Math.PI) / 2;
    blades.add(holder);
  }
  blades.position.set(0, 12.9, 1.15);
  group.add(blades);
  return { group, blades };
}

function makePlayground() {
  const parts = [];
  let seed = 141;
  const pad = new THREE.CylinderGeometry(6.5, 6.5, 0.35, 18);
  pad.translate(0, 0.17, 0);
  add(parts, pad, 0xe0c98f, 0.06, seed++);
  // 秋千架：两根立柱 + 红色横杆 + 两把蓝色吊座
  for (const sx of [-2.6, 2.6]) {
    box(parts, 0.32, 3.4, 0.32, sx, 1.9, -2.0, 0x8a6a48, seed++);
  }
  box(parts, 6.0, 0.3, 0.3, 0, 3.55, -2.0, 0xc65b4a, seed++);
  for (const sx of [-1.6, 1.6]) {
    box(parts, 0.5, 0.22, 0.5, sx, 1.5, -2.0, 0x4a7fb5, seed++);
  }
  // 滑梯：红色平台 + 四腿 + 黄色斜坡 + 侧梯
  box(parts, 1.7, 0.24, 1.7, 3.4, 1.85, 0.6, 0xb14a3a, seed++);
  for (const px of [2.7, 4.1]) {
    for (const pz of [-0.1, 1.3]) {
      box(parts, 0.22, 1.9, 0.22, px, 0.95, pz, 0x8a6a48, seed++);
    }
  }
  const ramp = new THREE.BoxGeometry(1.5, 0.2, 4.4);
  ramp.rotateX(0.62);
  ramp.translate(3.4, 0.95, 2.9);
  add(parts, ramp, 0xd9a83f, 0.05, seed++);
  box(parts, 0.24, 0.24, 2.2, 3.0, 0.7, 1.5, 0x8a6a48, seed++);
  const mesh = new THREE.Mesh(
    mergeGeometries(parts, false),
    new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.85, metalness: 0 })
  );
  mesh.castShadow = true;
  mesh.receiveShadow = true;
  return mesh;
}

function makeLighthouse() {
  const parts = [];
  let seed = 201;
  // 石质基座
  const base = new THREE.CylinderGeometry(2.4, 3.0, 1.2, 10);
  base.translate(0, 0.6, 0);
  add(parts, base, 0xb3ac9b, 0.05, seed++);
  // 塔身：6 段红白交替，逐渐收窄
  const nSeg = 6;
  const segH = 1.6;
  const r0 = 1.95, r1 = 0.8;
  for (let i = 0; i < nSeg; i++) {
    const t0 = i / nSeg, t1 = (i + 1) / nSeg;
    const g = new THREE.CylinderGeometry(r0 + (r1 - r0) * t1, r0 + (r1 - r0) * t0, segH, 10);
    g.translate(0, 1.2 + segH * (i + 0.5), 0);
    add(parts, g, i % 2 ? 0xf0ead9 : 0xc14e3c, 0.04, seed++);
  }
  // 灯室平台
  const deck = new THREE.CylinderGeometry(1.25, 1.45, 0.5, 10);
  deck.translate(0, 1.2 + segH * nSeg + 0.25, 0);
  add(parts, deck, 0x8a6a48, 0.05, seed++);
  // 灯室玻璃罩 + 红顶
  const gallery = new THREE.CylinderGeometry(0.8, 0.9, 1.25, 10);
  gallery.translate(0, 1.2 + segH * nSeg + 0.5 + 0.625, 0);
  add(parts, gallery, 0xf5efe0, 0.03, seed++);
  const roof = new THREE.ConeGeometry(1.0, 1.0, 10);
  roof.translate(0, 1.2 + segH * nSeg + 0.5 + 1.25 + 0.5, 0);
  add(parts, roof, 0xb14a3a, 0.04, seed++);
  const mesh = new THREE.Mesh(
    mergeGeometries(parts, false),
    new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.85, metalness: 0 })
  );
  mesh.castShadow = true;
  mesh.receiveShadow = true;
  const group = new THREE.Group();
  group.add(mesh);
  // 灯室发光
  const lamp = new THREE.Mesh(
    new THREE.CylinderGeometry(0.66, 0.66, 0.85, 10),
    new THREE.MeshBasicMaterial({ color: 0xffe9a0 })
  );
  lamp.position.set(0, 1.2 + segH * nSeg + 0.5 + 0.625, 0);
  group.add(lamp);
  return group;
}

function makeBoats() {
  const spots = [
    [134, 129, 0.6],
    [145, 101, 2.2],
    [106, 123, 4.0],
  ];
  const group = new THREE.Group();
  for (const [bx, bz, yaw] of spots) {
    const parts = [];
    let seed = 71;
    box(parts, 1.5, 0.55, 3.8, 0, 0.35, 0, 0x7a4f30, seed++);
    box(parts, 1.85, 0.22, 4.1, 0, 0.66, 0, 0xb0865a, seed++);
    box(parts, 1.1, 0.12, 1.2, 0, 0.62, 0.9, 0x5e3d25, seed++);
    box(parts, 1.1, 0.12, 1.2, 0, 0.62, -0.9, 0x5e3d25, seed++);
    const boat = new THREE.Mesh(
      mergeGeometries(parts, false),
      new THREE.MeshStandardMaterial({ vertexColors: true, roughness: 0.88, metalness: 0 })
    );
    boat.castShadow = true;
    boat.receiveShadow = true;
    const holder = new THREE.Group();
    holder.add(boat);
    holder.position.set(bx, 0.02, bz);
    holder.rotation.y = yaw;
    holder.scale.setScalar(2.0);
    group.add(holder);
  }
  return group;
}

export function createLandmarks(scene) {
  const group = new THREE.Group();
  const cabin = makeCabin();
  const cabinX = 281;
  const cabinZ = 249;
  const cabinH = terrainHeight(cabinX, cabinZ);
  cabin.position.set(cabinX, cabinH - 0.1, cabinZ);
  cabin.rotation.y = Math.atan2(-cabinX, -cabinZ);
  cabin.scale.setScalar(1.5);
  group.add(cabin);

  const pierAngle = Math.atan2(cabinZ, cabinX);
  const pier = makePier(pierAngle);
  pier.scale.setScalar(1.4);
  group.add(pier);

  const shore = shorelinePoint(pierAngle);
  const stones = makeSteppingStones(
    { x: cabinX, z: cabinZ + 2 },
    { x: shore.x + shore.ca * 4, z: shore.z + shore.sa * 4 },
    13
  );
  if (stones) group.add(stones);

  const windmill = makeWindmill();
  const wmX = 262, wmZ = 270;
  windmill.group.position.set(wmX, terrainHeight(wmX, wmZ) - 0.15, wmZ);
  windmill.group.rotation.y = Math.atan2(-159 - wmX, -136 - wmZ);
  windmill.group.scale.setScalar(1.3);
  group.add(windmill.group);

  const playground = makePlayground();
  const plX = 298, plZ = 229;
  playground.position.set(plX, terrainHeight(plX, plZ) - 0.1, plZ);
  playground.rotation.y = Math.atan2(-159 - plX, -136 - plZ);
  playground.scale.setScalar(1.3);
  group.add(playground);

  const lighthouse = makeLighthouse();
  const lhX = 231, lhZ = 258;
  lighthouse.position.set(lhX, terrainHeight(lhX, lhZ) - 0.12, lhZ);
  lighthouse.rotation.y = Math.atan2(-159 - lhX, -136 - lhZ);
  lighthouse.scale.setScalar(1.3);
  group.add(lighthouse);

  const boats = makeBoats();
  group.add(boats);

  scene.add(group);
  const clouds = makeClouds(scene);
  const rng = mulberry32(777);

  // diorama clouds: fixed spots behind the cabin along the iso view axis so
  // they appear floating above the hill in isometric mode (perspective keeps
  // the high drifting clouds).
  const vdx = Math.cos(3.86), vdz = Math.sin(3.86);
  const latx = vdz, latz = -vdx;
  const spot = (t, side, y) =>
    new THREE.Vector3(281 + vdx * t + latx * side, y, 249 + vdz * t + latz * side);
  const isoSpots = [
    spot(-70, -90, 48),
    spot(-80, 25, 54),
    spot(-90, 95, 60),
    spot(-100, -45, 66),
    spot(-110, 60, 50),
    spot(-120, -105, 56),
    spot(-130, 15, 62),
    spot(-140, 80, 68),
    spot(-150, -70, 52),
    spot(-160, 40, 58),
    spot(-170, -25, 64),
    spot(-180, 100, 60),
    spot(-190, -85, 54),
    spot(-200, 55, 70),
  ];
  let iso = false;

  return {
    group,
    clouds,
    setIso(on) {
      iso = on;
    },
    update(dt, elapsed) {
      windmill.blades.rotation.z = elapsed * 1.0;
      for (let i = 0; i < clouds.length; i++) {
        const c = clouds[i];
        const p = c.mesh.position;
        if (iso) {
          const s = isoSpots[i % isoSpots.length];
          p.x = s.x;
          p.y = s.y + Math.sin(elapsed * 0.5 + c.phase) * 3;
          p.z = s.z;
          c.mesh.rotation.y = elapsed * 0.02;
        } else {
          p.x += dt * c.speed * 1.5;
          p.z += dt * c.speed * 0.35;
          const r = Math.hypot(p.x, p.z);
          if (r > 820) {
            const a = Math.PI + rng() * Math.PI;
            p.x = Math.cos(a) * 780;
            p.z = Math.sin(a) * 780;
          }
          p.y += Math.sin(elapsed * 0.15 + c.phase) * dt * 0.5;
        }
      }
    },
  };
}
