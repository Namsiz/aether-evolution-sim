import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { EffectComposer } from 'three/addons/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/addons/postprocessing/RenderPass.js';
import { UnrealBloomPass } from 'three/addons/postprocessing/UnrealBloomPass.js';
import { ShaderPass } from 'three/addons/postprocessing/ShaderPass.js';
import { SMAAPass } from 'three/addons/postprocessing/SMAAPass.js';
import { OutputPass } from 'three/addons/postprocessing/OutputPass.js';
import { createEnvironment } from './sky.js';
import { createTerrain } from './terrain.js';
import { Ecology, HERB, PRED } from './ecology.js';
import { createAgentRenderer } from './agents.js';
import { createLandmarks } from './landmarks.js';
import {
  terrainHeight,
  slopeAt,
  WATER_LEVEL,
  SIM_RADIUS,
  clamp,
  smoothstep,
  mulberry32,
} from './world.js';

const $ = (id) => document.getElementById(id);
const canvas = $('stage');
const dtClamp = (v) => Math.min(0.05, Math.max(0, v));

const ua = navigator.userAgent || '';
const coarse = window.matchMedia?.('(pointer: coarse)')?.matches;
const lowPower = coarse || /Android|iPhone|iPad|Mobile/i.test(ua) || (navigator.hardwareConcurrency || 8) <= 4;
const quality = lowPower
  ? {
      terrainSegments: 312,
      grassCount: 36000,
      treeCount: 192,
      treeCapacity: 260,
      bushCount: 390,
      dpr: 1.5,
      shadowMap: 1536,
      smaa: true,
    }
  : {
      terrainSegments: 448,
      grassCount: 52000,
      treeCount: 290,
      treeCapacity: 380,
      bushCount: 620,
      dpr: 1.75,
      shadowMap: 2048,
      smaa: true,
    };

const state = {
  started: false,
  paused: false,
  speed: 1,
  orbit: true,
  tool: 'look',
  gameHours: 7.5,
  day: 1,
  ecologyAccumulator: 0,
  grassAcc: 0,
  shadeAcc: 0,
  timeScrubbing: false,
  lastStats: 0,
  fps: 0,
  iso: false,
  mode: 'observe',
  walkView: 'third',
  hunger: 100,
  life: 100,
  miles: {},
  weather: 'sun',
};

const player = {
  x: 281, y: 0, z: 258, vy: 0, yaw: 0.6, pitch: -0.25,
  onGround: true, walkPhase: 0, rig: null,
};
const joy = { x: 0, y: 0 };
let walkDrag = null;
let camSaved = null;
let ecoHudT = 0;

let renderer;
let scene;
let camera;
let isoCam = null;
let renderPass;
let controls;
let composer;
let bloom;
let grade;
let env;
let terrain;
let ecology;
let agents;
let landmarks;
let sunLight;
let sunDir;
let isoAz = 3.86;
let isoElv = 0.34;
let isoDist = 620;
let isoHalf = 100;
let isoTarget = new THREE.Vector3(281, 14, 249);
let isoDrag = null;
let selected = null;

const pulses = [];
const clock = new THREE.Clock();
const pointer = new THREE.Vector2(0, 0);
const raycaster = new THREE.Raycaster();
let pointerInside = false;
let pointerDown = null;
let hoverRing = null;
let toastTimer = 0;
let glowTex = null;

function setLoad(text, error = false) {
  const el = $('load-status');
  if (!el) return;
  el.textContent = text;
  el.style.color = error ? '#e8a07a' : '';
}

function nextFrame() {
  return new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
}

function makeGlowTexture() {
  if (glowTex) return glowTex;
  const c = document.createElement('canvas');
  c.width = c.height = 64;
  const ctx = c.getContext('2d');
  const g = ctx.createRadialGradient(32, 32, 0, 32, 32, 32);
  g.addColorStop(0, 'rgba(255,255,255,1)');
  g.addColorStop(0.3, 'rgba(200,230,205,0.45)');
  g.addColorStop(1, 'rgba(140,190,155,0)');
  ctx.fillStyle = g;
  ctx.fillRect(0, 0, 64, 64);
  glowTex = new THREE.CanvasTexture(c);
  glowTex.colorSpace = THREE.SRGBColorSpace;
  return glowTex;
}

function setupRenderer() {
  renderer = new THREE.WebGLRenderer({
    canvas,
    antialias: true,
    powerPreference: 'high-performance',
    stencil: false,
  });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, quality.dpr));
  renderer.setSize(window.innerWidth, window.innerHeight, false);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.0;
  renderer.outputColorSpace = THREE.SRGBColorSpace;

  scene = new THREE.Scene();
  const portrait = window.innerWidth / window.innerHeight < 0.85;
  camera = new THREE.PerspectiveCamera(portrait ? 60 : 52, window.innerWidth / window.innerHeight, 0.4, 22000);
  camera.position.set(270, terrainHeight(270, 340) + 43, 340);

  controls = new OrbitControls(camera, canvas);
  controls.target.set(-45, 10, -110);
  controls.enableDamping = true;
  controls.dampingFactor = 0.055;
  controls.minDistance = 7;
  controls.maxDistance = 980;
  controls.minPolarAngle = 0.16;
  controls.maxPolarAngle = Math.PI * 0.492;
  controls.enablePan = false;
  controls.autoRotate = true;
  controls.autoRotateSpeed = 0.075;
  controls.update();

  isoCam = new THREE.OrthographicCamera(-1, 1, 1, -1, 0.5, 22000);
  updateIsoCamera();
  isoCam.rotation.order = 'YXZ';

  controls.addEventListener('start', () => {
    if (controls.autoRotate) {
      state.orbit = false;
      controls.autoRotate = false;
      const btn = $('orbit-btn');
      if (btn) btn.classList.remove('on');
    }
    hideRing();
  });
}

function setupPost() {
  const w = window.innerWidth;
  const h = window.innerHeight;
  composer = new EffectComposer(renderer);
  composer.setPixelRatio(renderer.getPixelRatio());
  composer.setSize(w, h);
  renderPass = new RenderPass(scene, camera);
  composer.addPass(renderPass);

  bloom = new UnrealBloomPass(new THREE.Vector2(w, h), 0.24, 0.62, 0.88);
  composer.addPass(bloom);

  grade = new ShaderPass({
    uniforms: {
      tDiffuse: { value: null },
      uVignette: { value: 0.3 },
      uSat: { value: 1.045 },
      uTime: { value: 0 },
    },
    vertexShader: /* glsl */ `
      varying vec2 vUv;
      void main() {
        vUv = uv;
        gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
      }`,
    fragmentShader: /* glsl */ `
      uniform sampler2D tDiffuse;
      uniform float uVignette;
      uniform float uSat;
      uniform float uTime;
      varying vec2 vUv;
      void main() {
        vec4 tex = texture2D(tDiffuse, vUv);
        vec3 c = tex.rgb;
        float l = dot(c, vec3(0.2126, 0.7152, 0.0722));
        c = mix(vec3(l), c, uSat);
        float lt = l / (l + 1.0);
        vec3 tint = mix(vec3(0.968, 0.985, 1.018), vec3(1.025, 1.006, 0.974), smoothstep(0.05, 0.95, lt));
        c *= tint;
        vec3 cc = c * (1.0 + (c - 1.0) * 0.10);
        c = mix(c, cc, 0.5);
        float gr = fract(sin(dot(vUv * vec2(1113.0, 1737.0) + uTime * vec2(0.7, 1.3), vec2(12.9898, 78.233))) * 43758.5453);
        c += (gr - 0.5) * 0.006;
        float d = length((vUv - 0.5) * vec2(1.0, 0.92));
        c *= 1.0 - smoothstep(0.28, 0.8, d) * uVignette;
        c *= 1.0 + (vUv.y - 0.5) * 0.028;
        gl_FragColor = vec4(c, tex.a);
      }`,
  });
  composer.addPass(grade);

  if (quality.smaa) {
    const smaa = new SMAAPass(w * renderer.getPixelRatio(), h * renderer.getPixelRatio());
    composer.addPass(smaa);
  }
  composer.addPass(new OutputPass());

  window.addEventListener('resize', onResize);
}

function updateIsoCamera() {
  if (!isoCam) return;
  const halfH = isoHalf;
  const halfW = halfH * (window.innerWidth / window.innerHeight);
  isoCam.left = -halfW;
  isoCam.right = halfW;
  isoCam.top = halfH;
  isoCam.bottom = -halfH;
  isoCam.updateProjectionMatrix();
  const target = isoTarget;
  const cp = Math.cos(isoElv);
  const sp = Math.sin(isoElv);
  isoCam.position.set(
    target.x + isoDist * cp * Math.cos(isoAz),
    target.y + isoDist * sp,
    target.z + isoDist * cp * Math.sin(isoAz)
  );
  isoCam.lookAt(target);
}

function onResize() {
  if (!renderer) return;
  const w = window.innerWidth;
  const h = window.innerHeight;
  camera.aspect = w / h;
  camera.fov = w / h < 0.85 ? 60 : 52;
  camera.updateProjectionMatrix();
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, quality.dpr));
  renderer.setSize(w, h, false);
  composer.setPixelRatio(renderer.getPixelRatio());
  composer.setSize(w, h);
  updateIsoCamera();
}

function setupRing() {
  const geo = new THREE.RingGeometry(1.1, 1.45, 48);
  geo.rotateX(-Math.PI / 2);
  const mat = new THREE.MeshBasicMaterial({
    color: 0xbfe0c4,
    transparent: true,
    opacity: 0.78,
    depthWrite: false,
    side: THREE.DoubleSide,
    blending: THREE.AdditiveBlending,
  });
  hoverRing = new THREE.Mesh(geo, mat);
  hoverRing.visible = false;
  hoverRing.renderOrder = 30;
  scene.add(hoverRing);
}

function inspectAt() {
  if (!agents || !ecology) return;
  const px = pointer.x;
  const py = pointer.y;
  const v = new THREE.Vector3();
  let best = null;
  let bestD = 0.05 * 0.05;
  for (const a of ecology.living()) {
    if (!a.alive) continue;
    v.set(a.x, a.y + a.genome.size * 1.7, a.z).project(camera);
    if (v.z > 1) continue;
    const dx = v.x - px;
    const dy = v.y - py;
    const d2 = dx * dx + dy * dy;
    if (d2 < bestD) {
      bestD = d2;
      best = a;
    }
  }
  if (best) {
    selected = best;
    updateInspectPanel();
    return;
  }
  selected = null;
  $('inspect').classList.remove('show');
}

// B. 动物 detail 增强：仅 selected.id 变化时重算（不在 animate 节流循环重复扫描）
let lastDetailId = 0;
function computeEcoDetail(target) {
  const kindKey = target.kind === HERB ? 'herb' : target.kind === PRED ? 'pred' : target.kind === 'scav' ? 'scav' : 'omn';
  const hist = ecology.evolution && ecology.evolution[kindKey];
  let evoLine = '';
  if (hist && hist.length) {
    const recent = hist.slice(-3); // 最近最多三代祖先
    const avg = (f) => recent.reduce((s, v) => s + (v[f] || 0), 0) / recent.length;
    evoLine =
      `<div class="eco-detail-section" style="margin-top:6px;border-top:1px solid rgba(210,232,220,0.12);padding-top:5px;">` +
      `<span style="color:#93a6b8;font-size:7.5px;">近${recent.length}代均值</span><br>` +
      `速<b>${avg('speed').toFixed(1)}</b> · 感<b>${Math.round(avg('sense'))}</b> · 代<b>${avg('metabolism').toFixed(2)}</b>` +
      (kindKey === 'herb' ? ` · 阈<b>${avg('helpThr').toFixed(2)}</b>` : '') +
      `</div>`;
  }
  // 附近 willing/needy 小字（HelpR 语义：dist<22，饱腹=energy≥0.4max，饥饿<0.4max）
  const HELP_R = 22;
  let willingN = 0, needyN = 0;
  for (const a of ecology.agents) {
    if (!a.alive || a.kind !== target.kind || a.id === target.id) continue;
    if (Math.hypot(a.x - target.x, a.z - target.z) < HELP_R) {
      if (a.energy >= 0.4 * a.maxEnergy) willingN++;
      else needyN++;
    }
  }
  const nearLine = (willingN || needyN)
    ? `<div class="eco-detail-section" style="margin-top:4px;font-size:8px;color:rgba(190,208,199,0.55);">附近同种: willing=${willingN} needy=${needyN} (${HELP_R}内)</div>`
    : '';
  return evoLine + nearLine;
}

function updateInspectPanel() {
  const el = $('inspect');
  if (!el || !selected || !selected.alive) return;
  const g = selected.genome;
  const stateName = {
    graze: '吃草', wander: '游荡', flee: '逃跑', hunt: '捕猎',
    scavenge: '觅食', patrol: '巡逻', feed: '进食', drink: '饮水',
  }[selected.state] || selected.state;
  const kindName = selected.kind === HERB ? '牧食者' : '掠食者';
  el.classList.add('show');
  $('insp-title').textContent = kindName;
  $('insp-title').style.color = selected.kind === HERB ? '#d3a670' : '#93a6b8';
  $('insp-state').textContent = stateName;
  $('insp-body').innerHTML =
    `能量 <b>${Math.round(selected.energy)}</b> · 年龄 <b>${Math.round(selected.age)}s</b><br>` +
    `速度 <b>${g.speed.toFixed(1)}</b> · 感知 <b>${Math.round(g.sense)}</b> · 体型 <b>${g.size.toFixed(2)}</b><br>` +
    `世代 <b>${selected.generation + 1}</b> · 代谢 <b>${g.metabolism.toFixed(2)}</b>`;
  // 附加段：selected.id 变化才重算（性能约束：不在 animate 节流循环重复扫描）
  if (selected.id !== lastDetailId) {
    lastDetailId = selected.id;
    lastDetailHtml = computeEcoDetail(selected);
  }
  const stale = el.querySelectorAll('.eco-detail-section');
  for (const n of stale) n.remove(); // 先清除上次追加的段落，避免重复堆叠
  if (lastDetailHtml) el.insertAdjacentHTML('beforeend', lastDetailHtml);
}
let lastDetailHtml = '';

function ringColorForTool(tool) {
  if (tool === 'tree') return 0x9ad18b;
  if (tool === 'rock') return 0xbdb3a3;
  if (tool === 'herb') return 0xe0b77a;
  if (tool === 'pred') return 0x9fb8d8;
  if (tool === 'clear') return 0xdd9a78;
  return 0xbfe0c4;
}

function hideRing() {
  if (hoverRing) hoverRing.visible = false;
}

function updatePointer(e) {
  const rect = canvas.getBoundingClientRect();
  pointer.x = ((e.clientX - rect.left) / rect.width) * 2 - 1;
  pointer.y = -((e.clientY - rect.top) / rect.height) * 2 + 1;
  pointerInside = true;
}

function groundHit() {
  raycaster.setFromCamera(pointer, camera);
  const hits = raycaster.intersectObject(terrain.terrain, false);
  return hits.length ? hits[0] : null;
}

function updateHover() {
  if (!hoverRing || !terrain) return;
  if (selected) return;
  if (!pointerInside || state.tool === 'look' || pointerDown) {
    hoverRing.visible = false;
    return;
  }
  const hit = groundHit();
  if (!hit) {
    hoverRing.visible = false;
    return;
  }
  hoverRing.position.set(hit.point.x, hit.point.y + 0.16, hit.point.z);
  hoverRing.material.color.setHex(ringColorForTool(state.tool));
  hoverRing.visible = true;
}

function setupPointer() {
  setupRing();
  canvas.addEventListener('pointermove', (e) => {
    updatePointer(e);
    if (e.pointerType === 'mouse') updateHover();
  });
  canvas.addEventListener('pointerleave', () => {
    pointerInside = false;
    hideRing();
  });
  canvas.addEventListener('pointerdown', (e) => {
    updatePointer(e);
    pointerDown = { x: e.clientX, y: e.clientY, t: performance.now() };
    if (state.mode === 'walk') {
      walkDrag = { x: e.clientX, y: e.clientY };
      return;
    }
    if (state.iso) isoDrag = { x: e.clientX, y: e.clientY };
    hideRing();
  });
  canvas.addEventListener('pointermove', (e) => {
    updatePointer(e);
    if (walkDrag) {
      player.yaw += (e.clientX - walkDrag.x) * 0.005;
      player.pitch = clamp(player.pitch - (e.clientY - walkDrag.y) * 0.004, -1.1, 0.5);
      walkDrag.x = e.clientX;
      walkDrag.y = e.clientY;
      return;
    }
    if (isoDrag) {
      isoAz += (e.clientX - isoDrag.x) * 0.0055;
      isoDrag.x = e.clientX;
      isoDrag.y = e.clientY;
      return;
    }
    if (state.iso) return;
    if (e.pointerType === 'mouse') updateHover();
  });
  canvas.addEventListener('pointerup', (e) => {
    isoDrag = null;
    walkDrag = null;
    if (state.mode === 'walk') return;
    if (!pointerDown) return;
    const dist = Math.hypot(e.clientX - pointerDown.x, e.clientY - pointerDown.y);
    const elapsed = performance.now() - pointerDown.t;
    pointerDown = null;
    if (dist < 8 && elapsed < 700 && (state.tool === 'look' || !state.iso)) {
      updatePointer(e);
      tryPlace();
    }
    if (e.pointerType === 'mouse') updateHover();
  });
  canvas.addEventListener('pointercancel', () => { pointerDown = null; isoDrag = null; });
  canvas.addEventListener('wheel', (e) => {
    if (!state.iso) return;
    e.preventDefault();
    isoHalf = clamp(isoHalf * Math.exp(e.deltaY * 0.0015), 90, 900);
    updateIsoCamera();
  }, { passive: false });
}

function toast(text) {
  const el = $('toast');
  if (!el) return;
  el.textContent = text;
  el.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => el.classList.remove('show'), 2200);
}

function makePulse(x, y, z, color) {
  const geo = new THREE.RingGeometry(0.7, 0.95, 40);
  geo.rotateX(-Math.PI / 2);
  const mat = new THREE.MeshBasicMaterial({
    color,
    transparent: true,
    opacity: 0.9,
    depthWrite: false,
    side: THREE.DoubleSide,
    blending: THREE.AdditiveBlending,
  });
  const mesh = new THREE.Mesh(geo, mat);
  mesh.position.set(x, y + 0.2, z);
  mesh.renderOrder = 31;
  scene.add(mesh);
  pulses.push({ mesh, age: 0 });
  if (pulses.length > 10) {
    const old = pulses.shift();
    scene.remove(old.mesh);
    old.mesh.geometry.dispose();
    old.mesh.material.dispose();
  }
}

function tryPlace() {
  if (!terrain || !ecology) return;
  if (state.tool === 'look') {
    inspectAt();
    return;
  }
  const hit = groundHit();
  if (!hit) {
    toast('请点击地面');
    return;
  }
  const { x, z } = hit.point;
  if (Math.hypot(x, z) > SIM_RADIUS - 20) {
    toast('已到达生态边界，往里一点');
    return;
  }
  if (state.tool === 'tree') {
    if (terrain.addTree(x, z)) {
      makePulse(x, hit.point.y, z, 0x9ad18b);
      toast('一棵树开始生长');
    } else {
      toast('这里太陡或太靠近水，树无法扎根');
    }
    return;
  }
  if (state.tool === 'rock') {
    if (terrain.addRock(x, z)) {
      makePulse(x, hit.point.y, z, 0xbdb3a3);
      toast('一块岩石落定');
    } else {
      toast('这里放不下岩石');
    }
    return;
  }
  if (state.tool === 'clear') {
    if (terrain.removeTree(x, z, 7)) {
      makePulse(x, hit.point.y, z, 0xdd9a78);
      toast('树木被移除了');
    } else {
      toast('附近没有可移除的树木');
    }
    return;
  }
  if (state.tool === 'herb' || state.tool === 'pred') {
    if (hit.point.y < WATER_LEVEL + 1.0) {
      toast('生命需要干燥的陆地');
      return;
    }
    if (slopeAt(x, z) > 0.8) {
      toast('坡太陡，动物无法立足');
      return;
    }
    const kind = state.tool === 'herb' ? HERB : PRED;
    const a = ecology.spawn(kind, x, z, null, 0, 0);
    if (!a) {
      toast(kind === HERB ? '牧食者已达上限' : '掠食者已达上限');
      hideRing();
      return;
    }
    a.energy = a.maxEnergy * 0.72;
    a.age = 24;
    makePulse(x, hit.point.y, z, kind === HERB ? 0xe0b77a : 0x9fb8d8);
    toast(kind === HERB ? '一只牧食者进入世界' : '一只掠食者进入世界');
  }
}

function setupUI() {
  const toolButtons = [...document.querySelectorAll('.tool')];
  function setTool(tool) {
    state.tool = tool;
    selected = null;
    $('inspect').classList.remove('show');
    for (const b of toolButtons) b.classList.toggle('active', b.dataset.tool === tool);
    hideRing();
    if (tool !== 'look') {
      state.orbit = false;
      controls.autoRotate = false;
      $('orbit-btn')?.classList.remove('on');
    }
    const hints = {
      look: '拖拽环视 · 滚轮缩放 · WASD 飞行 · 点击动物查看个体',
      tree: '点击草地种下一棵树 · 可继续点击',
      rock: '点击地面放置一块岩石',
      clear: '点击树干移除附近的一棵树',
      herb: '点击草地放入一只牧食者',
      pred: '点击草地放入一只掠食者',
    };
    $('hint').textContent = hints[tool];
  }
  for (const b of toolButtons) b.addEventListener('click', () => setTool(b.dataset.tool));

  $('orbit-btn').addEventListener('click', () => {
    if (state.iso) return;
    state.orbit = !state.orbit;
    controls.autoRotate = state.orbit;
    $('orbit-btn').classList.toggle('on', state.orbit);
    if (state.orbit) setTool('look');
  });

  $('iso-btn').addEventListener('click', () => {
    state.iso = !state.iso;
    $('iso-btn').classList.toggle('on', state.iso);
    controls.enabled = !state.iso;
    controls.autoRotate = false;
    $('orbit-btn').classList.remove('on');
    state.orbit = false;
    hideRing();
    if (state.iso) {
      setTool('look');
      $('hint').textContent = '拖拽旋转 · 滚轮缩放 · 等距观赏';
    } else {
      $('hint').textContent = '拖拽环视 · 滚轮缩放 · WASD 飞行 · 点击动物查看个体';
    }
  });

  $('speed-btn').addEventListener('click', () => {
    state.speed = state.speed === 1 ? 4 : state.speed === 4 ? 12 : 1;
    $('speed-btn').querySelector('b').textContent = state.speed + '×';
    toast('生态速度 ' + state.speed + '×');
  });

  $('pause-btn').addEventListener('click', () => {
    state.paused = !state.paused;
    const btn = $('pause-btn');
    btn.classList.toggle('paused', state.paused);
    btn.querySelector('span').textContent = state.paused ? '继续' : '暂停';
    btn.querySelector('svg').innerHTML = state.paused
      ? '<path d="M8 5l11 7-11 7z"/>'
      : '<path d="M9 6v12M15 6v12"/>';
    toast(state.paused ? '生态时间已暂停' : '生态继续演化');
  });

  const slider = $('time-slider');
  const startScrub = () => { state.timeScrubbing = true; };
  const endScrub = () => { state.timeScrubbing = false; };
  slider.addEventListener('pointerdown', startScrub);
  slider.addEventListener('pointerup', endScrub);
  slider.addEventListener('touchend', endScrub);
  slider.addEventListener('input', () => {
    const v = parseFloat(slider.value);
    state.gameHours = Math.floor(state.gameHours / 24) * 24 + v;
    if (v < 6 && (state.gameHours % 24) > 12) state.gameHours -= 24;
    env.setTime(v);
    state.day = Math.floor(state.gameHours / 24) + 1;
    updateClock();
  });

  $('enter').addEventListener('click', () => {
    if ($('enter').disabled) return;
    state.started = true;
    state.paused = false;
    const overlay = $('loading');
    overlay.classList.add('hidden');
    setTimeout(() => overlay.remove(), 1400);
    controls.autoRotate = state.orbit;
    toast('世界已苏醒 · 每一次出生与死亡都在真实发生');
  });

  $('walk-btn').addEventListener('click', () => {
    setMode(state.mode === 'walk' ? 'observe' : 'walk');
  });
  $('view-btn').addEventListener('click', () => toggleWalkView());

  const joyEl = $('joystick');
  const stickEl = $('stick');
  let joyId = null;
  const joyMove = (e) => {
    if (e.pointerId !== joyId) return;
    const r = joyEl.getBoundingClientRect();
    const cx = r.left + r.width / 2;
    const cy = r.top + r.height / 2;
    let dx = e.clientX - cx;
    let dy = e.clientY - cy;
    const max = r.width * 0.34;
    const d = Math.hypot(dx, dy);
    if (d > max) { dx = dx / d * max; dy = dy / d * max; }
    joy.x = dx / max;
    joy.y = dy / max;
    stickEl.style.transform = 'translate(' + dx + 'px,' + dy + 'px)';
  };
  joyEl.addEventListener('pointerdown', (e) => {
    joyId = e.pointerId;
    joyEl.setPointerCapture(e.pointerId);
    stickEl.classList.add('active');
    joyMove(e);
  });
  joyEl.addEventListener('pointermove', joyMove);
  const joyEnd = (e) => {
    if (e.pointerId !== joyId) return;
    joyId = null;
    joy.x = 0;
    joy.y = 0;
    stickEl.style.transform = '';
    stickEl.classList.remove('active');
  };
  joyEl.addEventListener('pointerup', joyEnd);
  joyEl.addEventListener('pointercancel', joyEnd);

  window.addEventListener('keydown', (e) => {
    if (e.target && /input|textarea/i.test(e.target.tagName)) return;
    if (e.code === 'Space') {
      e.preventDefault();
      if (state.mode === 'walk') {
        if (player.onGround) player.vy = 8.6;
      } else {
        $('pause-btn').click();
      }
    }
    if (e.code === 'KeyV' && state.mode === 'walk') toggleWalkView();
    if (e.code === 'KeyE' && state.mode === 'walk') walkInteract();
    if (e.code === 'KeyR' && e.shiftKey) toggleReplay(); // Shift+R 切换生态轨迹回放（原 R 单击不再触发，避免与 walk 快捷键冲突）
  });

  // C. 生态轨迹回放控件：关闭按钮 + 窗口切换
  $('replay-close')?.addEventListener('click', () => {
    replayOpen = false;
    $('eco-replay-panel')?.classList.add('hidden');
  });
  $('replay-window')?.addEventListener('change', () => {
    if (replayOpen) drawReplay();
  });
}

function updateClock() {
  const h = ((state.gameHours % 24) + 24) % 24;
  const hh = Math.floor(h);
  const mm = Math.floor((h - hh) * 60);
  $('clock').textContent = String(hh).padStart(2, '0') + ':' + String(mm).padStart(2, '0');
  $('day-label').textContent = '第 ' + state.day + ' 天';
  if (!state.timeScrubbing) {
    const slider = $('time-slider');
    const cur = parseFloat(slider.value);
    if (Math.abs(cur - h) > 0.03) slider.value = h.toFixed(2);
  }
}

function drawChart() {
  const cvs = $('chart');
  if (!cvs) return;
  const ctx = cvs.getContext('2d');
  const w = cvs.width;
  const h = cvs.height;
  ctx.clearRect(0, 0, w, h);
  const hist = ecology.history;
  const n = Math.min(160, hist.length);
  if (n < 2) return;
  const start = hist.length - n;

  ctx.strokeStyle = 'rgba(200,225,210,0.08)';
  ctx.lineWidth = 1;
  for (let i = 0; i < 3; i++) {
    const y = 8 + (h - 16) * (i / 2);
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(w, y);
    ctx.stroke();
  }

  let maxPop = 20;
  for (let i = start; i < hist.length; i++) {
    maxPop = Math.max(maxPop, hist[i].h, hist[i].p);
  }

  const xAt = (i) => ((i - start) / (n - 1)) * w;
  const yPlant = (v) => h - 6 - clamp(v, 0, 1) * (h - 14);
  const yPop = (v) => h - 5 - (v / maxPop) * (h - 14);

  ctx.beginPath();
  ctx.moveTo(0, h);
  for (let i = start; i < hist.length; i++) {
    ctx.lineTo(xAt(i), yPlant(hist[i].plant));
  }
  ctx.lineTo(w, h);
  ctx.closePath();
  const grad = ctx.createLinearGradient(0, 0, 0, h);
  grad.addColorStop(0, 'rgba(121,150,90,0.28)');
  grad.addColorStop(1, 'rgba(121,150,90,0.01)');
  ctx.fillStyle = grad;
  ctx.fill();

  const line = (key, color) => {
    ctx.beginPath();
    for (let i = start; i < hist.length; i++) {
      const x = xAt(i);
      const y = yPop(hist[i][key]);
      if (i === start) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.strokeStyle = color;
    ctx.lineWidth = 1.7;
    ctx.stroke();
  };
  line('h', 'rgba(211,166,112,0.9)');
  line('p', 'rgba(147,166,184,0.9)');
}

// 演化速率曲线：四物种速度随世代变化（达尔文可视化）
function drawEvoChart() {
  const cvs = $('evo-chart');
  if (!cvs) return;
  const ctx = cvs.getContext('2d');
  const w = cvs.width;
  const h = cvs.height;
  ctx.clearRect(0, 0, w, h);
  const s = ecology.stats();
  const series = [
    { data: s.evHistH, color: 'rgba(211,166,112,0.85)', label: '牧' },
    { data: s.evHistP, color: 'rgba(147,166,184,0.85)', label: '掠' },
    { data: s.evHistScav, color: 'rgba(160,130,90,0.85)', label: '腐' },
    { data: s.evHistOmn, color: 'rgba(120,160,120,0.85)', label: '杂' },
  ];
  let minV = Infinity, maxV = -Infinity;
  for (const sr of series) {
    if (!sr.data || sr.data.length < 2) continue;
    for (const d of sr.data) { minV = Math.min(minV, d.speed); maxV = Math.max(maxV, d.speed); }
  }
  if (minV === Infinity || minV === maxV) { minV = 0; maxV = 10; }
  const range = maxV - minV || 1;
  ctx.strokeStyle = 'rgba(200,225,210,0.06)';
  ctx.lineWidth = 1;
  for (let i = 0; i < 2; i++) {
    const y = 4 + (h - 8) * (i / 2);
    ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke();
  }
  for (const sr of series) {
    if (!sr.data || sr.data.length < 2) continue;
    const n = sr.data.length;
    ctx.beginPath();
    for (let i = 0; i < n; i++) {
      const x = (i / (n - 1)) * w;
      const y = h - 4 - ((sr.data[i].speed - minV) / range) * (h - 8);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.strokeStyle = sr.color;
    ctx.lineWidth = 1.5;
    ctx.stroke();
  }
  // 标注 min/max 速度
  ctx.fillStyle = 'rgba(180,200,180,0.5)';
  ctx.font = '9px sans-serif';
  ctx.fillText(maxV.toFixed(1), 2, 10);
  ctx.fillText(minV.toFixed(1), 2, h - 4);
}

function updateStats() {
  if (!ecology) return;
  const s = ecology.stats();
  $('stat-h').textContent = s.h;
  $('stat-p').textContent = s.p;
  $('stat-plant').textContent = Math.round(s.plant * 100) + '%';
  $('stat-gen').textContent = s.gen + 1;
  let g0s = 0, g0n = 0, g2s = 0, g2n = 0;
  for (const ag of ecology.agents) {
    if (!ag.alive || ag.kind !== 'herb') continue;
    if (ag.generation === 0) { g0s += ag.genome.speed; g0n++; }
    else if (ag.generation >= 2) { g2s += ag.genome.speed; g2n++; }
  }
  const gDiff = (g0n && g2n) ? (g2s / g2n - g0s / g0n) : 0;
  $('stat-gen').textContent = (s.gen + 1) + (gDiff ? (gDiff > 0.3 ? ' ▲' : gDiff < -0.3 ? ' ▼' : ' ＝') : '');
  if (!updateStats._last) updateStats._last = {};
  const ls = updateStats._last;
  let trend = '';
  if (ls.plant != null) trend = s.plant > ls.plant + 0.002 ? '▲' : s.plant < ls.plant - 0.002 ? '▼' : '＝';
  ls.plant = s.plant;
  const ratio = s.h > 0 ? (s.h / Math.max(s.p, 1)).toFixed(1) : '?';
  const scavC = (ecology.agents || []).filter(a => a.alive && a.kind === 'scav').length;
  const omnC = (ecology.agents || []).filter(a => a.alive && a.kind === 'omn').length;
  $('stat-food').textContent = `${s.p}:${s.h} (1:${ratio}) S${scavC} O${omnC}${trend ? ' 草' + trend : ''}`;
  // 达尔文观察：适应度 + 选择压力 + 演化速率
  const sp = s.selectionPressure || {};
  const crowd = sp.crowding != null ? (sp.crowding > 0.8 ? '拥挤' : sp.crowding > 0.5 ? '正常' : '稀疏') : '—';
  const predP = sp.predRatio != null ? (sp.predRatio > 0.2 ? '捕食强' : sp.predRatio > 0.1 ? '捕食中' : '捕食弱') : '—';
  const plantP = sp.plant != null ? (sp.plant < 0.3 ? '匮乏' : sp.plant < 0.5 ? '适中' : '丰饶') : '—';
  const fitDir = s.curFitH != null && s.curFitP != null
    ? (s.curFitH > s.curFitP ? '牧食者更适应' : '掠食者更适应') : '—';
  // 协同进化：H/P 速度趋势对比（军备竞赛）
  const trendH = s.trendH, trendP = s.trendP;
  let coEvol = '—';
  if (trendH > 0.3 && trendP > 0.3) coEvol = '⚔️ 军备竞赛';
  else if (trendH > 0.3) coEvol = '牧食者领先';
  else if (trendP > 0.3) coEvol = '掠食者追赶';
  else if (trendH < -0.3 && trendP < -0.3) coEvol = '双向退化';
  else coEvol = '演化平衡';

  // 物种形成检测：同种基因距离双峰分布→分化预警
  function speciationCheck(speeds, label) {
    if (speeds.length < 20) return '';
    speeds.sort((a, b) => a - b);
    const n = speeds.length;
    const q25 = speeds[Math.floor(n * 0.25)], q75 = speeds[Math.floor(n * 0.75)];
    const gap = q75 - q25;
    const totalRange = speeds[n - 1] - speeds[0];
    // 双峰检测：Q1和Q3间隔内的密度低谷
    const mid = speeds[Math.floor(n / 2)];
    const lowCount = speeds.filter(v => v > q25 - 0.1 && v < q75 + 0.1).length;
    const densityGap = lowCount / n; // 中间密度
    if (totalRange > 2.0 && densityGap < 0.15) return label + '分化中';
    if (gap < 0.5) return ''; // 太集中，无分化
    return '';
  }

  // 物种形成检查（简化版：用 std 和 range）
  function speciationCheckV2(speeds, label) {
    if (speeds.length < 30) return '';
    const avg = speeds.reduce((s, v) => s + v, 0) / speeds.length;
    const std = Math.sqrt(speeds.reduce((s, v) => s + (v - avg) ** 2, 0) / speeds.length);
    const range = Math.max(...speeds) - Math.min(...speeds);
    // 高变异 + 大跨度 → 可能分化
    if (range > 2.5 && std > 0.6) return '🔬' + label + '潜在分化';
    return '';
  }

  const hSpeeds = ecology.agents.filter(a => a.alive && a.kind === 'herb').map(a => a.genome.speed);
  const pSpeeds = ecology.agents.filter(a => a.alive && a.kind === 'pred').map(a => a.genome.speed);
  const oSpeeds = ecology.agents.filter(a => a.alive && a.kind === 'omn').map(a => a.genome.speed);
  const specH = speciationCheckV2(hSpeeds, '牧食');
  const specP = speciationCheckV2(pSpeeds, '掠食');
  const specO = speciationCheckV2(oSpeeds, '杂食');
  const specLabel = [specH, specP, specO].filter(Boolean).join(' · ') || '无分化';

  $('stat-darwin').textContent = `适应 ${s.curFitH != null ? s.curFitH.toFixed(2) : '—'} vs ${s.curFitP != null ? s.curFitP.toFixed(2) : '—'} · ${fitDir} · 草${plantP} ${predP} ${crowd} · ${coEvol}`;
  // 演化速率：四物种性状变化方向（速度/代谢）
  const evTrend = (hist, field, scale) => {
    if (!hist || hist.length < 2) return '＝';
    const first = hist[0][field] || 0, last = hist[hist.length - 1][field] || 0;
    const diff = last - first;
    const norm = scale || 1;
    return diff > 0.08 * norm ? '↑' : diff < -0.08 * norm ? '↓' : '≈';
  };
  const eh = s.evHistH, ep = s.evHistP, es = s.evHistScav, eo = s.evHistOmn;
  const evH = eh && eh.length >= 2 ? `速${evTrend(eh,'speed')} 代${eh[eh.length-1].gen}` : '—';
  const evP = ep && ep.length >= 2 ? `速${evTrend(ep,'speed')} 代${ep[ep.length-1].gen}` : '—';
  const evS = es && es.length >= 2 ? `速${evTrend(es,'speed')} 代${es[es.length-1].gen}` : '—';
  const evO = eo && eo.length >= 2 ? `速${evTrend(eo,'speed')} 代${eo[eo.length-1].gen}` : '—';
  $('stat-evolution')?.textContent != null && ($('stat-evolution').textContent = `牧食${evH} 掠食${evP} 腐食${evS} 杂食${evO} · ${specLabel}`);
  // 物种形成事件显示（谱系命名）
  const se = s.speciationEvent;
  const seEl = $('stat-speciation');
  if (seEl) {
    if (se && se.lineages) {
      const l = se.lineages;
      seEl.textContent = `🐛 ${se.parentKind === 'herb' ? '牧食者' : '?'}→ ${l[0].name}(${l[0].count}) · ${l[1].name}(${l[1].count}) CV=${se.cvSpeed}`;
    } else {
      seEl.textContent = '—';
    }
  }
  let hMin = 99, hMax = 0, pMin = 99, pMax = 0, hRMin = 9999, hRMax = 0, pRMin = 9999, pRMax = 0;
  for (const ag of ecology.agents) {
    if (!ag.alive) continue;
    const sp = ag.genome.speed;
    if (ag.kind === 'herb') {
      if (sp < hMin) hMin = sp; if (sp > hMax) hMax = sp;
      if (ag.range < hRMin) hRMin = ag.range; if (ag.range > hRMax) hRMax = ag.range;
    } else {
      if (sp < pMin) pMin = sp; if (sp > pMax) pMax = sp;
      if (ag.range < pRMin) pRMin = ag.range; if (ag.range > pRMax) pRMax = ag.range;
    }
  }
  const tH = s.trendH > 0.3 ? '▲' : s.trendH < -0.3 ? '▼' : '＝';
  const tP = s.trendP > 0.3 ? '▲' : s.trendP < -0.3 ? '▼' : '＝';
  $('gene-h').textContent = `速度 ${s.speedH.toFixed(1)}（${hMin.toFixed(1)}-${hMax.toFixed(1)}）· 感知 ${Math.round(s.senseH)} · 家域 ${Math.round(hRMin)}-${Math.round(hRMax)} · 演化${tH}`;
  $('gene-p').textContent = `速度 ${s.speedP.toFixed(1)}（${pMin.toFixed(1)}-${pMax.toFixed(1)}）· 感知 ${Math.round(s.senseP)} · 家域 ${Math.round(pRMin)}-${Math.round(pRMax)} · 演化${tP}`;
  drawChart();
  drawEvoChart();
  drawGeneDist();
  drawFitnessLandscape();
  drawArmsRace();
}

// 军备竞赛：gapHP 趋势（正=牧食者领先/负=掠食者追赶）
function drawArmsRace() {
  const cvs = $('arms-race');
  if (!cvs) return;
  const ctx = cvs.getContext('2d');
  const w = cvs.width, h = cvs.height;
  ctx.clearRect(0, 0, w, h);
  const hist = ecology.history;
  const n = Math.min(160, hist.length);
  if (n < 2) return;
  const start = hist.length - n;
  const midY = h / 2;
  // 零线
  ctx.strokeStyle = 'rgba(200,225,210,0.1)';
  ctx.lineWidth = 1;
  ctx.beginPath(); ctx.moveTo(0, midY); ctx.lineTo(w, midY); ctx.stroke();
  // gap 曲线
  let minG = 0, maxG = 0;
  for (let i = start; i < hist.length; i++) {
    const g = hist[i].gapHP || 0;
    minG = Math.min(minG, g); maxG = Math.max(maxG, g);
  }
  const range = maxG - minG || 1;
  ctx.beginPath();
  for (let i = start; i < hist.length; i++) {
    const x = ((i - start) / (n - 1)) * w;
    const y = midY - ((hist[i].gapHP || 0) / range) * (h / 2 - 6);
    if (i === start) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  }
  // 正区域绿（牧领先），负区域红（掠追赶）
  const grad = ctx.createLinearGradient(0, 0, 0, h);
  grad.addColorStop(0, 'rgba(211,166,112,0.8)');
  grad.addColorStop(0.5, 'rgba(180,200,180,0.5)');
  grad.addColorStop(1, 'rgba(147,166,184,0.8)');
  ctx.strokeStyle = grad;
  ctx.lineWidth = 1.5;
  ctx.stroke();
  // 标注
  ctx.fillStyle = 'rgba(180,200,180,0.5)';
  ctx.font = '8px sans-serif';
  ctx.fillText(maxG.toFixed(2), 2, 10);
  ctx.fillText(minG.toFixed(2), 2, h - 4);
}

// 适应度景观：速度(横)×感知(纵)→适应度亮度（达尔文 3D fitness landscape 2D 投影）
function drawFitnessLandscape() {
  const cvs = $('fit-landscape');
  if (!cvs) return;
  const ctx = cvs.getContext('2d');
  const w = cvs.width, h = cvs.height;
  ctx.fillStyle = 'rgba(0,0,0,0.1)';
  ctx.fillRect(0, 0, w, h);
  // 收集活体：速度、感知、适应度
  const cells = {};
  for (const a of ecology.agents) {
    if (!a.alive) continue;
    const sp = Math.round(a.genome.speed);
    const se = Math.round(a.genome.sense / 10);
    const key = `${sp},${se}`;
    if (!cells[key]) cells[key] = { n: 0, fit: 0 };
    const off = a.offspring || 0;
    cells[key].n++;
    cells[key].fit += (a.age / a.genome.lifespan) * (1 + off) / a.genome.metabolism;
  }
  let maxFit = 0;
  for (const c of Object.values(cells)) { c.fit /= c.n; maxFit = Math.max(maxFit, c.fit); }
  // 画格子（亮度 = 适应度）
  const spMin = 2, spMax = 9, seMin = 1, seMax = 13;
  for (const [key, c] of Object.entries(cells)) {
    const [sp, se] = key.split(',').map(Number);
    const x = ((sp - spMin) / (spMax - spMin)) * w;
    const y = h - 4 - ((se - seMin) / (seMax - seMin)) * (h - 10);
    const brightness = c.fit / (maxFit || 1);
    ctx.fillStyle = `rgba(180,200,180,${0.1 + brightness * 0.8})`;
    const cw = w / (spMax - spMin) - 2;
    const ch = (h - 10) / (seMax - seMin) - 2;
    ctx.fillRect(x, y, Math.max(4, cw), Math.max(4, ch));
  }
  ctx.fillStyle = 'rgba(180,200,180,0.4)';
  ctx.font = '8px sans-serif';
  ctx.fillText('速度→', w - 36, h - 4);
  ctx.fillText('感知↑', 2, 10);
}

// 基因频率分布：四物种速度基因直方图（达尔文变异分布可视化——多峰=分化，单峰=同质化）
function drawGeneDist() {
  const cvs = $('gene-dist');
  if (!cvs) return;
  const ctx = cvs.getContext('2d');
  const w = cvs.width, h = cvs.height;
  ctx.clearRect(0, 0, w, h);
  // 收集四物种速度数据
  const data = { herb: [], pred: [], scav: [], omn: [] };
  for (const a of ecology.agents) {
    if (!a.alive) continue;
    const key = a.kind === 'herb' ? 'herb' : a.kind === 'pred' ? 'pred' : a.kind === 'scav' ? 'scav' : 'omn';
    data[key].push(a.genome.speed);
  }
  // 各找 bin 范围
  let minV = 10, maxV = 0;
  for (const arr of Object.values(data)) { if (arr.length) { minV = Math.min(minV, ...arr); maxV = Math.max(maxV, ...arr); } }
  if (maxV <= minV) { maxV = minV + 5; }
  const bins = 20;
  const bw = (maxV - minV) / bins;
  const colors = { herb: 'rgba(211,166,112,0.7)', pred: 'rgba(147,166,184,0.7)', scav: 'rgba(160,130,90,0.7)', omn: 'rgba(120,160,120,0.7)' };
  for (const [kind, arr] of Object.entries(data)) {
    if (arr.length < 2) continue;
    const counts = new Array(bins).fill(0);
    for (const v of arr) { const bi = Math.min(bins - 1, Math.floor((v - minV) / bw)); counts[bi]++; }
    const maxC = Math.max(...counts, 1);
    ctx.beginPath();
    for (let i = 0; i < bins; i++) {
      const x0 = (i / bins) * w;
      const x1 = ((i + 1) / bins) * w;
      const y0 = h - 3 - (counts[i] / maxC) * (h - 10);
      const y1 = h - 3;
      ctx.fillStyle = colors[kind];
      ctx.fillRect(x0 + 1, y0, Math.max(1, x1 - x0 - 2), y1 - y0);
    }
  }
  // 标注 min/max
  ctx.fillStyle = 'rgba(180,200,180,0.4)';
  ctx.font = '8px sans-serif';
  ctx.fillText(minV.toFixed(1), 2, h - 4);
  ctx.fillText(maxV.toFixed(1), w - 24, h - 4);
}

function refreshShade() {
  const N = ecology.GRID_N;
  const span = ecology.GRID_SPAN;
  const cell = span / N;
  const half = span / 2;
  const grid = ecology.shade;
  grid.fill(0);
  for (const en of terrain.treeMeshes) {
    for (const t of en.data) {
      const rad = Math.max(4, t.s * 3.4);
      const ci = Math.floor((t.x + half) / cell);
      const cj = Math.floor((t.z + half) / cell);
      const ri = Math.ceil(rad / cell);
      const r2 = rad * rad;
      for (let j = cj - ri; j <= cj + ri; j++) {
        if (j < 0 || j >= N) continue;
        for (let i = ci - ri; i <= ci + ri; i++) {
          if (i < 0 || i >= N) continue;
          const dx = (i + 0.5) * cell - half - t.x;
          const dz = (j + 0.5) * cell - half - t.z;
          if (dx * dx + dz * dz < r2) {
            const idx = j * N + i;
            grid[idx] = Math.min(1, grid[idx] + 0.5);
          }
        }
      }
    }
  }
}

function createPlayerRig() {
  const g = new THREE.Group();
  const bodyMat = new THREE.MeshStandardMaterial({ color: 0x50708e, roughness: 0.9, metalness: 0, emissive: 0x0d1f33, emissiveIntensity: 0.7 });
  const skinMat = new THREE.MeshStandardMaterial({ color: 0xd8b48a, roughness: 0.92, metalness: 0 });
  const body = new THREE.Mesh(new THREE.CapsuleGeometry(0.34, 0.62, 4, 8), bodyMat);
  body.position.y = 1.02;
  const head = new THREE.Mesh(new THREE.SphereGeometry(0.27, 10, 8), skinMat);
  head.position.y = 1.6;
  const legL = new THREE.Mesh(new THREE.CapsuleGeometry(0.12, 0.5, 3, 6), bodyMat);
  legL.position.set(-0.15, 0.4, 0);
  const legR = legL.clone();
  legR.position.x = 0.15;
  const armL = new THREE.Mesh(new THREE.CapsuleGeometry(0.095, 0.46, 3, 6), skinMat);
  armL.position.set(-0.45, 1.12, 0);
  const armR = armL.clone();
  armR.position.x = 0.45;
  g.add(body, head, legL, legR, armL, armR);
  g.scale.set(1.35, 1.35, 1.35);
  g.traverse((o) => { if (o.isMesh) { o.castShadow = true; o.receiveShadow = true; } });
  const halo = new THREE.Mesh(new THREE.CircleGeometry(0.72, 28), new THREE.MeshBasicMaterial({ color: 0xffd24a, transparent: true, opacity: 0.30, depthWrite: false, side: THREE.DoubleSide }));
  halo.rotation.x = -Math.PI / 2;
  halo.position.y = 0.05;
  halo.castShadow = false;
  halo.receiveShadow = false;
  g.add(halo);
  g.visible = false;
  scene.add(g);
  return { group: g, legL, legR, armL, armR };
}

function setMode(mode) {
  const walk = mode === 'walk';
  if (walk && !state.started) return;
  state.mode = mode;
  if (walk) {
    state.iso = false;
    if (!player.rig) player.rig = createPlayerRig();
    camSaved = { pos: camera.position.clone(), target: controls.target.clone() };
    controls.enabled = false;
    player.x = 295;
    player.z = 265;
    player.y = terrainHeight(player.x, player.z);
    player.vy = 0;
    player.yaw = -1.42;
    player.pitch = -0.25;
    player.onGround = true;
    player.rig.group.visible = state.walkView === 'third';
    $('walk-btn').querySelector('span').textContent = '离开';
    $('view-btn').classList.remove('hidden');
    $('joystick').classList.remove('hidden');
    $('invbar').classList.remove('hidden');
    updateInvUI();
    updateBuildBtn();
    toast('WASD 行走 · 空格跳跃 · E 交互 · F 放置 · B 选型 · V 视角');
  } else {
    if (player.rig) player.rig.group.visible = false;
    $('walk-btn').querySelector('span').textContent = '进入';
    $('view-btn').classList.add('hidden');
    $('joystick').classList.add('hidden');
    $('invbar').classList.add('hidden');
    $('prompt').classList.add('hidden');
    controls.enabled = true;
    if (camSaved) {
      camera.position.copy(camSaved.pos);
      controls.target.copy(camSaved.target);
      controls.update();
    }
    toast('已回到观察视角');
  }
  updateEnvPanels(); // 面板在 observe + walk 模式都显示（仅未开始时隐藏）
}

function toggleWalkView() {
  if (state.mode !== 'walk') return;
  state.walkView = state.walkView === 'third' ? 'first' : 'third';
  if (player.rig) player.rig.group.visible = state.walkView === 'third';
  toast(state.walkView === 'first' ? '第一人称' : '第三人称');
}
function inOBB(c, x, z, pad = 0) {
  const dx = x - c.x;
  const dz = z - c.z;
  const cos = Math.cos(c.rot);
  const sin = Math.sin(c.rot);
  const lx = dx * cos - dz * sin;
  const lz = dx * sin + dz * cos;
  return Math.abs(lx) <= c.hw + pad && Math.abs(lz) <= c.hd + pad;
}

// 玩家可站立平台：返回 (x,z) 处、且玩家脚在 deck 附近时最高的 deck 高度；无则 -Infinity
function deckAt(x, z, curY) {
  let h = -Infinity;
  for (const d of walkDecks) {
    if (inOBB(d, x, z) && curY >= d.y - 0.7) h = Math.max(h, d.y);
  }
  return h;
}

// 地面高度 = max(地形, 可站平台)
function groundAt(x, z, curY) {
  const g = terrainHeight(x, z);
  const d = deckAt(x, z, curY);
  return d > g ? d : g;
}

// 水平阻挡：玩家 (x,z) 是否在某 collider（屋墙等）内（按玩家半径膨胀，防薄墙隧穿）
function collideBlock(x, z) {
  const pad = 0.34;
  for (const c of walkColliders) {
    if (inOBB(c, x, z, pad)) return true;
  }
  return false;
}

function updatePlayer(dt) {
  if (state.mode !== 'walk' || !player.rig) return;
  let ix = 0;
  let iz = 0;
  if (keys.has('KeyW') || keys.has('ArrowUp')) iz -= 1;
  if (keys.has('KeyS') || keys.has('ArrowDown')) iz += 1;
  if (keys.has('KeyA') || keys.has('ArrowLeft')) ix -= 1;
  if (keys.has('KeyD') || keys.has('ArrowRight')) ix += 1;
  ix += joy.x;
  iz += joy.y;
  const mag = Math.min(1, Math.hypot(ix, iz));
  const running = keys.has('ShiftLeft') || keys.has('ShiftRight');
  let speed = (running ? 10.5 : 6.2) * mag;
  if (state.hunger < 10) speed *= 0.6;
  let moving = false;
  if (mag > 0.02) {
    const fwdX = Math.sin(player.yaw);
    const fwdZ = Math.cos(player.yaw);
    const rightX = Math.cos(player.yaw);
    const rightZ = -Math.sin(player.yaw);
    let mx = fwdX * -iz + rightX * ix;
    let mz = fwdZ * -iz + rightZ * ix;
    const ml = Math.hypot(mx, mz);
    if (ml > 0.0001) {
      mx /= ml;
      mz /= ml;
      const nx = player.x + mx * speed * dt;
      const nz = player.z + mz * speed * dt;
      const nhTer = terrainHeight(nx, nz);
      const nhDeck = deckAt(nx, nz, player.y);
      const nh = nhDeck > nhTer ? nhDeck : nhTer;
      const ch = groundAt(player.x, player.z, player.y);
      const steep = (nh - ch) / Math.max(0.001, Math.hypot(nx - player.x, nz - player.z));
      const deepWater = !(nhDeck > nhTer) && nhTer < WATER_LEVEL - 0.35;
      if (!collideBlock(nx, nz) && !deepWater && steep < 1.5) {
        player.x = nx;
        player.z = nz;
        moving = true;
      }
    }
  }
  player.vy -= 30 * dt;
  player.y += player.vy * dt;
  const ground = groundAt(player.x, player.z, player.y);
  if (player.y <= ground) {
    player.y = ground;
    player.vy = 0;
    player.onGround = true;
  } else {
    player.onGround = false;
  }
  const rig = player.rig;
  rig.group.position.set(player.x, player.y, player.z);
  rig.group.rotation.y = player.yaw;
  player.walkPhase += dt * (moving ? speed * 1.55 : 0);
  const swing = Math.sin(player.walkPhase) * 0.55 * (moving ? 1 : 0);
  rig.legL.rotation.x = swing;
  rig.legR.rotation.x = -swing;
  rig.armL.rotation.x = -swing * 0.75;
  rig.armR.rotation.x = swing * 0.75;
  const bob = moving ? Math.abs(Math.sin(player.walkPhase)) * 0.05 : 0;
  rig.group.position.y = player.y + bob;
}

function nearestTarget() {
  if (state.mode !== 'walk') return null;
  let best = null;
  const t = terrain.nearestTree(player.x, player.z, 3.0);
  if (t) {
    best = { type: 'tree', entry: t.entry, i: t.i, life: t.life, d: Math.hypot(t.life.x - player.x, t.life.z - player.z) };
  }
  for (const kind of ['herb', 'pred', 'scav', 'omn']) {
    const arr = agents.maps[kind];
    if (!arr) continue;
    for (let i = 0; i < arr.length; i++) {
      const a = arr[i];
      if (!a.alive) continue;
      const d = Math.hypot(a.x - player.x, a.z - player.z);
      if (d < 2.8 && (!best || d < best.d)) best = { type: kind, obj: a, d };
    }
  }
  const rk = terrain.nearestRock(player.x, player.z, 2.5);
  if (rk) {
    const d = Math.hypot(rk.x - player.x, rk.z - player.z);
    if (!best || d < best.d) best = { type: 'rock', i: rk.i, x: rk.x, z: rk.z, d };
  }
  const bs = terrain.nearestBush(player.x, player.z, 2.5);
  if (bs) {
    const d = Math.hypot(bs.x - player.x, bs.z - player.z);
    if (!best || d < best.d) best = { type: 'bush', i: bs.i, x: bs.x, z: bs.z, d };
  }
  for (const obj of structures) {
    const st = obj.userData && obj.userData.storage;
    if (!st) continue;
    const d = Math.hypot(st.x - player.x, st.z - player.z);
    if (d < 2.5 && (!best || d < best.d)) best = { type: 'storage', x: st.x, z: st.z, d };
  }
  for (const obj of structures) {
    const bd = obj.userData && obj.userData.bed;
    if (!bd) continue;
    const d = Math.hypot(bd.x - player.x, bd.z - player.z);
    if (d < 2.5 && (!best || d < best.d)) best = { type: 'bed', x: bd.x, z: bd.z, d };
  }
  return best;
}

// 观察者模式，无攻击
let attackCD = 0;

function updateHunger(dt) {
  if (state.mode !== 'walk') return;
  const running = keys.has('ShiftLeft') || keys.has('ShiftRight');
  const drain = running ? 0.6 : 0.35;
  const before = state.hunger;
  state.hunger = Math.max(0, state.hunger - drain * dt);
  if (before >= 10 && state.hunger < 10) toast('饥饿中（观察世界，无需进食）');
}

function updateThreat(dt) {
  if (state.mode !== 'walk') return;
  const h = state.gameHours;
  const night = h > 20 || h < 5;
  // 安全区：篝火半径 6，或站在屋地板上
  let safe = false;
  for (const o of structures) {
    if (o.userData && o.userData.fire) {
      const f = o.userData.fire;
      if (Math.hypot(f.x - player.x, f.z - player.z) < 6) { safe = true; break; }
    }
  }
  if (!safe) {
    for (const d of walkDecks) {
      if (d.kind === 'house' && inOBB(d, player.x, player.z) && player.y >= d.y - 0.7) { safe = true; break; }
    }
  }
  if (!night || safe) {
    if (state.life < 100) state.life = Math.min(100, state.life + 2 * dt);
    return;
  }
  let nd = 99;
  for (const a of ecology.agents) {
    if (!a.alive || a.kind !== 'pred') continue;
    const d = Math.hypot(a.x - player.x, a.z - player.z);
    if (d < nd) nd = d;
  }
  if (nd < 2.5) {
    // 玩家为观察者，无伤害
  }
}

// 观察者模式，无死亡/重生
function respawn() {
  state.life = 100;
  state.hunger = 80;
  player.x = 295; player.z = 265;
  player.y = terrainHeight(player.x, player.z);
  player.vy = 0;
}

function walkInteract() {
  if (state.mode !== 'walk') return;
  const t = nearestTarget();
  if (!t) { toast('附近没有可观察的对象'); return; }
  if (t.type === 'herb' || t.type === 'pred') {
    const g = t.obj.genome;
    const nm = t.type === 'herb' ? '牧食者' : t.type === 'pred' ? '掠食者' : '腐食者';
    toast(`${nm}·第${t.obj.generation}代 速${g.speed.toFixed(1)} 感${Math.round(g.sense)}`);
  } else if (t.type === 'tree') {
    toast('一棵树——树荫遮蔽，草在地上竞争生长');
  } else if (t.type === 'rock') {
    toast('一块岩石——不动的见证者');
  } else if (t.type === 'bush') {
    toast('灌木——果子是牧食者的口粮');
  }
}

function updatePrompt() {
  const el = $('prompt');
  if (!el) return;
  if (state.mode !== 'walk') { el.classList.add('hidden'); return; }
  const t = nearestTarget();
  if (!t) { el.classList.add('hidden'); return; }
  if (t.type === 'herb' || t.type === 'pred' || t.type === 'scav' || t.type === 'omn') {
    const g = t.obj.genome;
    const nm = t.type === 'herb' ? '牧食者' : t.type === 'pred' ? '掠食者' : t.type === 'scav' ? '腐食者' : '杂食者';
    const st = { graze: '吃草', wander: '游荡', flee: '逃跑', hunt: '捕猎', avoid: '避开', wary: '警惕', scavenge: '腐食' }[t.obj.state] || t.obj.state;
    const baseRange = t.type === 'herb' ? 170 : t.type === 'pred' ? 280 : t.type === 'scav' ? 200 : 185;
    const rf = t.obj.range / baseRange;
    const rtag = rf > 1.15 ? '探索' : rf < 0.95 ? '守家' : '定居';
    const baseSense = t.type === 'herb' ? 46 : t.type === 'pred' ? 57 : t.type === 'scav' ? 57 : 46;
    const sf = g.sense / baseSense;
    const stag = sf > 1.2 ? '敏锐' : sf < 0.8 ? '迟钝' : '均衡';
    const btag = g.size > 1.25 ? '大型' : g.size < 0.85 ? '小型' : '中型';
    const spf = g.speed / 6.0;
    const stg = spf > 1.15 ? '快速' : spf < 0.85 ? '迟缓' : '均衡';
    const meta = g.metabolism > 1.1 ? '高代谢' : g.metabolism < 0.95 ? '低代谢' : '中代谢';
    const soc = g.social > 0.8 ? '聚群' : g.social < 0.4 ? '独居' : '社交';
    const bold = g.bold > 0.65 ? '大胆' : g.bold < 0.35 ? '谨慎' : '均衡';
    const fit = (t.obj.age / g.lifespan) * (1 + (t.obj.offspring || 0)) / g.metabolism;
    const lineage = t.obj.lineage ? `【${t.obj.lineage}】` : '';
    el.textContent = `${nm}·第${t.obj.generation}代 ${lineage}${st} ${rtag}·${stag}·${btag}·${stg}·${meta}·${soc}·${bold} 适${fit.toFixed(2)}`;
  } else {
    el.textContent = '观察它';
  }
  el.classList.remove('hidden');
}

let weatherT = 0;
// Weather RNG：从 ecology.seed 派生（+1001 offset），保证同 seed 下天气序列与生态步长严格对齐。
// 原实现用 Math.random() 污染了 headless 复现路径，见 v0.2 R5 / v2.0 Phase 0-p3。
// 注意：ecology 是异步赋值的（boot 后），故每帧从 ecology?.seed 读取当前值重新派生 rng state
// 不够优雅但无副作用；更干净的方案是把 weatherRng 挂到 ecology 实例上，此处保持最小改动。
function makeWeatherRng(ecology) {
  return mulberry32((ecology?.seed ?? 20261102) + 1001);
}
let _weatherRng = makeWeatherRng(ecology);
function updateWeather(dt) {
  weatherT -= dt;
  if (weatherT <= 0) {
    weatherT = 60 + _weatherRng() * 60;
    state.weather = _weatherRng() < 0.3 ? 'rain' : 'sun';
    ecology.rain = state.weather === 'rain' ? 1 : 0;
  }
}

function updateEcoHud() {
  const el = $('inv-world');
  if (!el || state.mode !== 'walk') return;
  const s = ecology.stats();
  // 演化趋势 EMA（草食速度平滑），供演化里程碑
  const sp = s.speedH;
  state.miles.evolEma = state.miles.evolEma == null ? sp : state.miles.evolEma * 0.995 + sp * 0.005;
  if (state.miles.evolBase == null) state.miles.evolBase = state.miles.evolEma;
  const hh = Math.floor(state.gameHours), mm = Math.floor((state.gameHours - hh) * 60);
  const seasons = ['春', '夏', '秋', '冬'];
  el.textContent = `D${state.day} ${seasons[s.season || 0]} ${String(hh).padStart(2, '0')}:${String(mm).padStart(2, '0')} 草${s.plant.toFixed(2)} H${s.h} P${s.p} S${s.scav || 0} O${s.omn || 0}${state.weather === 'rain' ? ' · 雨' : ''}`;
}

// A. 环境信息面板更新（每 0.5s，started 后即显示，walk/observe 都可用）
const ECO_MAX = { h: 420, p: 110, s: 40, o: 80 };
function updateEnvPanels() {
  const el = $('eco-panels');
  if (!el) return;
  // 未开始时隐藏；started 后始终显示（walk/observe 模式均可读生态数据）
  el.classList.toggle('hidden', !state.started);
  if (!state.started) return;
  const s = ecology.stats();
  const seasons = ['春', '夏', '秋', '冬'];
  // 季节进度：gameHours 内的当天小时/24
  const dayFrac = ((state.gameHours % 24) + 24) % 24 / 24;
  $('ep-season').textContent = seasons[s.season ?? 0] || '-';
  $('ep-season-bar').style.width = Math.round(dayFrac * 100) + '%';
  $('ep-season-label').textContent = '第 ' + (Math.floor(state.gameHours / 24) % 4 + 1) + '/4 季';
  $('ep-weather').textContent = state.weather === 'rain' ? '🌧' : '☀';
  // 四物种条形（不 import ecology MAX_*，手动常量）
  const counts = [s.h || 0, s.p || 0, s.scav || 0, s.omn || 0];
  const vs = el.querySelectorAll('.ep-v');
  for (let i = 0; i < 4; i++) vs[i].style.width = ECO_MAX[['h','p','s','o'][i]] > 0 ? Math.min(100, Math.round(counts[i] / ECO_MAX[['h','p','s','o'][i]] * 100)) + '%' : '0%';
  const cs = el.querySelectorAll('.ep-counts span');
  for (let i = 0; i < 4; i++) cs[i].textContent = counts[i];
  // 时间（#eco-clock）+ 日出日落倒计时（#eco-dusk-countdown）
  const hh = Math.floor(state.gameHours), mm = Math.floor((state.gameHours - hh) * 60);
  $('eco-clock').textContent = String(hh).padStart(2, '0') + ':' + String(mm).padStart(2, '0');
  const curHour = ((state.gameHours % 24) + 24) % 24;
  let phaseText, countdownText;
  if (curHour >= 6 && curHour < 18) {
    phaseText = '白天';
    const duskLeft = 18 - curHour;
    const dh = Math.floor(duskLeft), dm = Math.round((duskLeft - dh) * 60);
    countdownText = `离日落 ${dh}h${dm.toString().padStart(2,'0')}m`;
  } else {
    phaseText = '夜间';
    const nextSunrise = (curHour >= 18 ? 24 + 6 : 6) - curHour;
    const dh = Math.floor(nextSunrise), dm = Math.round((nextSunrise - dh) * 60);
    countdownText = `离日出 ${dh}h${dm.toString().padStart(2,'0')}m`;
  }
  $('ep-phase').textContent = phaseText;
  $('ep-dusk-countdown').textContent = countdownText;
  // 关键指标：草量 + 三草种占比 + coopShareH
  $('ep-grass').textContent = '草量 ' + s.plant.toFixed(2);
  const by = s.plantBy || [];
  const tr = by.map((v, i) => v ? `${i}: ${v.toFixed(2)}` : '');
  $('ep-species-ratios').textContent = tr.filter(Boolean).join(' · ') || '--';
  $('ep-coop-share').textContent = 'coopShareH: ' + (s.coopShareH != null ? s.coopShareH.toFixed(3) : '--');
}

// C. 生态轨迹回放：Shift+R 切换
let replayOpen = false;
function toggleReplay() {
  if (!ecology || !ecology.history) return;
  replayOpen = !replayOpen;
  const panel = $('eco-replay-panel');
  if (!panel) return;
  if (replayOpen) {
    if (state.paused) {
      panel.classList.remove('hidden');
      drawReplay();
    } else {
      toast('回放仅可在暂停时打开');
      replayOpen = false;
    }
  } else {
    panel.classList.add('hidden');
  }
}
function drawReplay() {
  const cvs = $('replay-canvas');
  if (!cvs || !ecology) return;
  const ctx = cvs.getContext('2d');
  const w = cvs.width, h = cvs.height;
  ctx.clearRect(0, 0, w, h);
  const hist = ecology.history;
  if (!hist.length) return;
  const minutes = Number($('replay-window')?.value || 10);
  const totalSec = minutes * 60;
  // 取 history 中最近 minutes 分钟的数据（history 采样间隔 ~2s）
  const step = 2;
  const nPoints = Math.min(hist.length, Math.ceil(totalSec / step));
  const startIdx = hist.length - nPoints;
  // Y 轴：gapHP（零线居中）
  let minG = 0, maxG = 0;
  for (let i = startIdx; i < hist.length; i++) {
    const g = hist[i].gapHP || 0;
    minG = Math.min(minG, g); maxG = Math.max(maxG, g);
  }
  const range = maxG - minG || 1;
  const midY = h / 2;
  ctx.strokeStyle = 'rgba(200,225,210,0.1)'; ctx.lineWidth = 1;
  ctx.beginPath(); ctx.moveTo(0, midY); ctx.lineTo(w, midY); ctx.stroke();
  ctx.beginPath();
  for (let i = startIdx; i < hist.length; i++) {
    const x = ((i - startIdx) / Math.max(1, nPoints - 1)) * w;
    const y = midY - ((hist[i].gapHP || 0) / range) * (midY - 6);
    i === startIdx ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
  }
  const grad = ctx.createLinearGradient(0, 0, 0, h);
  grad.addColorStop(0, 'rgba(211,166,112,0.85)');
  grad.addColorStop(0.5, 'rgba(180,200,180,0.35)');
  grad.addColorStop(1, 'rgba(147,166,184,0.85)');
  ctx.strokeStyle = grad; ctx.lineWidth = 1.6; ctx.stroke();
  // 标注时间范围
  const firstT = hist[startIdx]?.t ?? 0, lastT = hist[hist.length - 1]?.t ?? 0;
  $('replay-range').textContent = `t=${firstT.toFixed(0)}–${lastT.toFixed(0)}s（${minutes}min窗口）· gapHP`;
}

function updateFarmRegrow(dt) {
  for (const o of structures) {
    const fm = o.userData && o.userData.farm;
    if (!fm || fm.timer < 0) continue;
    fm.timer -= dt;
    if (fm.timer <= 0) {
      if (terrain.addBush(fm.x, fm.z)) fm.timer = -1;
      else fm.timer = 0;
    }
  }
}

function updateMiles() {
  const m = state.miles;
  if (!m.night && (state.gameHours > 20 || state.gameHours < 5)) {
    m.night = true;
    toast('夜色降临——你听见远处嚎叫，掠食者在狩猎（篝火/火把可避险）');
  }
  if (!m.pred && ecology.agents.some(a => a.alive && a.kind === 'pred' && Math.hypot(a.x - player.x, a.z - player.z) < 30)) {
    m.pred = true;
    toast('你看到掠食者——这片世界的猎手');
  }
  if (!m.house && walkDecks.some(d => d.kind === 'house')) {
    m.house = true;
    toast('你的家立起来了——生态仍在你周围流转');
  }
  if (!m.fire && ecology.fires.length) {
    m.fire = true;
    toast('篝火燃起——夜晚的野兽将远离这团光');
  }
}

function updateInvUI() {
  const h = $('inv-hunger'), l = $('inv-life');
  if (h) { h.textContent = Math.round(state.hunger); h.style.color = state.hunger < 25 ? '#e07a5a' : ''; }
  if (l) { l.textContent = Math.round(state.life); l.style.color = state.life < 30 ? '#e05a5a' : ''; }
}

const INV_MAX = 20;
let buildSel = 0;
const structures = [];
// 可站立平台（桥面/地板）：OBB {x,z,hw,hd,rot,y}
const walkDecks = [];
// 阻挡玩家水平移动（屋墙等）：OBB {x,z,hw,hd,rot}
const walkColliders = [];
const buildStoneMat = new THREE.MeshStandardMaterial({ color: 0x8a8578, roughness: 0.9 });
const buildWoodMat = new THREE.MeshStandardMaterial({ color: 0x9a6b3f, roughness: 0.85 });

function makeCampfire(x, z, y) {
  const g = new THREE.Group();
  for (let i = 0; i < 7; i++) {
    const a = i / 7 * Math.PI * 2;
    const st = new THREE.Mesh(new THREE.DodecahedronGeometry(0.16, 0), buildStoneMat);
    st.position.set(Math.cos(a) * 0.42, 0.1, Math.sin(a) * 0.42);
    st.castShadow = true;
    g.add(st);
  }
  const fire = new THREE.Mesh(
    new THREE.ConeGeometry(0.28, 0.62, 8),
    new THREE.MeshBasicMaterial({ color: 0xff8a3c, transparent: true, opacity: 0.92 })
  );
  fire.position.y = 0.44;
  g.add(fire);
  g.position.set(x, y, z);
  scene.add(g);
  g.userData.fire = { x, z };
  return g;
}

function makeFence(x, z, yaw, y) {
  const g = new THREE.Group();
  const rail = new THREE.Mesh(new THREE.BoxGeometry(2.2, 0.14, 0.12), buildWoodMat);
  rail.position.y = 0.75;
  const rail2 = rail.clone();
  rail2.position.y = 0.42;
  const p1 = new THREE.Mesh(new THREE.BoxGeometry(0.14, 0.95, 0.14), buildWoodMat);
  p1.position.set(-1.0, 0.47, 0);
  const p2 = p1.clone();
  p2.position.x = 1.0;
  for (const m of [rail, rail2, p1, p2]) m.castShadow = true;
  g.add(rail, rail2, p1, p2);
  g.position.set(x, y, z);
  g.rotation.y = yaw;
  scene.add(g);
  walkColliders.push({ x, z, hw: 1.1, hd: 0.06, rot: yaw });
  return g;
}
function makeBridge(x, z, yaw, L = 9) {
  const Wd = 2.2;
  const fx = Math.sin(yaw), fz = Math.cos(yaw);
  const y0 = terrainHeight(x + fx * (L / 2), z + fz * (L / 2));
  const y1 = terrainHeight(x - fx * (L / 2), z - fz * (L / 2));
  const y = Math.max(WATER_LEVEL + 0.9, y0, y1) + 0.12;
  const g = new THREE.Group();
  // 长轴沿局部 z（=玩家面向），宽沿局部 x
  const deck = new THREE.Mesh(new THREE.BoxGeometry(Wd, 0.14, L), buildWoodMat);
  deck.position.y = 0.07;
  deck.castShadow = true;
  deck.receiveShadow = true;
  g.add(deck);
  for (const side of [-1, 1]) {
    const rail = new THREE.Mesh(new THREE.BoxGeometry(0.1, 0.1, L), buildWoodMat);
    rail.position.set(side * (Wd / 2 - 0.05), 0.85, 0);
    rail.castShadow = true;
    g.add(rail);
    for (let i = -1; i <= 1; i++) {
      const post = new THREE.Mesh(new THREE.BoxGeometry(0.12, 0.9, 0.12), buildWoodMat);
      post.position.set(side * (Wd / 2 - 0.05), 0.45, i * (L / 2 - 0.3));
      post.castShadow = true;
      g.add(post);
    }
  }
  // 护栏碰撞（两侧），玩家在桥面中线 0.66 内活动，两端开口上下
  for (const side of [-1, 1]) {
    const rx = side * (Wd / 2 - 0.05);
    walkColliders.push({ x: x + rx * Math.cos(yaw), z: z - rx * Math.sin(yaw), hw: 0.05, hd: L / 2, rot: yaw });
  }
  const botY = WATER_LEVEL + 0.4;
  const pierH = y - botY;
  if (pierH > 0.3) {
    for (const d of [-L / 2 + 0.5, L / 2 - 0.5]) {
      const pier = new THREE.Mesh(new THREE.BoxGeometry(0.3, pierH, 0.3), buildStoneMat);
      pier.position.set(0, -pierH / 2 - 0.02, d);
      pier.castShadow = true;
      g.add(pier);
    }
  }
  g.position.set(x, y, z);
  g.rotation.y = yaw;
  scene.add(g);
  // 桥面可站立（OBB），长轴局部 z → hd，宽局部 x → hw
  walkDecks.push({ x, z, hw: Wd / 2, hd: L / 2, rot: yaw, y: y + 0.14, kind: 'bridge' });
  return g;
}
function makeHouse(x, z, yaw, y) {
  const W = 4, D = 4, H = 2.7, wallT = 0.2, doorW = 1.3;
  const g = new THREE.Group();
  const floor = new THREE.Mesh(new THREE.BoxGeometry(W, 0.12, D), buildWoodMat);
  floor.position.y = 0.06;
  floor.receiveShadow = true;
  g.add(floor);
  const back = new THREE.Mesh(new THREE.BoxGeometry(W, H, wallT), buildWoodMat);
  back.position.set(0, H / 2, -D / 2);
  g.add(back);
  const left = new THREE.Mesh(new THREE.BoxGeometry(wallT, H, D), buildWoodMat);
  left.position.set(-W / 2, H / 2, 0);
  g.add(left);
  const right = left.clone();
  right.position.x = W / 2;
  g.add(right);
  const segW = (W - doorW) / 2;
  const segOff = doorW / 2 + segW / 2;
  const frontL = new THREE.Mesh(new THREE.BoxGeometry(segW, H, wallT), buildWoodMat);
  frontL.position.set(-segOff, H / 2, D / 2);
  g.add(frontL);
  const frontR = frontL.clone();
  frontR.position.x = segOff;
  g.add(frontR);
  const roof = new THREE.Mesh(new THREE.BoxGeometry(W + 0.4, 0.16, D + 0.4), buildWoodMat);
  roof.position.y = H + 0.08;
  roof.castShadow = true;
  g.add(roof);
  for (const m of [floor, back, left, right, frontL, frontR]) m.castShadow = true;
  g.position.set(x, y, z);
  g.rotation.y = yaw;
  scene.add(g);
  // 碰撞（墙）与可站立（地板），OBB 用世界坐标
  const c = Math.cos(yaw), s = Math.sin(yaw);
  const w2 = (lx, lz) => ({ x: x + lx * c + lz * s, z: z - lx * s + lz * c });
  // 储物箱（屋角），靠近按 E 存取
  const chest = new THREE.Mesh(new THREE.BoxGeometry(0.7, 0.5, 0.5), buildStoneMat);
  chest.position.set(W / 2 - 0.5, 0.31, -D / 2 + 0.5);
  chest.castShadow = true;
  g.add(chest);
  const chestLid = new THREE.Mesh(new THREE.BoxGeometry(0.72, 0.12, 0.52), buildWoodMat);
  chestLid.position.set(W / 2 - 0.5, 0.62, -D / 2 + 0.5);
  g.add(chestLid);
  const cs = w2(W / 2 - 0.5, -D / 2 + 0.5);
  g.userData.storage = { x: cs.x, z: cs.z };
  // 床（左侧贴墙），靠近按 E 睡觉
  const bedFrame = new THREE.Mesh(new THREE.BoxGeometry(1.0, 0.35, 2.0), buildWoodMat);
  bedFrame.position.set(-W / 2 + 0.6, 0.24, 0);
  bedFrame.castShadow = true;
  g.add(bedFrame);
  const bedMat = new THREE.MeshStandardMaterial({ color: 0x9db3c8, roughness: 0.9 });
  const mattress = new THREE.Mesh(new THREE.BoxGeometry(0.9, 0.16, 1.8), bedMat);
  mattress.position.set(-W / 2 + 0.6, 0.5, 0);
  g.add(mattress);
  const pillow = new THREE.Mesh(new THREE.BoxGeometry(0.7, 0.14, 0.4), bedMat);
  pillow.position.set(-W / 2 + 0.6, 0.62, 0.75);
  g.add(pillow);
  const bd = w2(-W / 2 + 0.6, 0);
  g.userData.bed = { x: bd.x, z: bd.z };
  const t2 = wallT / 2 + 0.05;
  const wallDefs = [
    { lx: 0, lz: -D / 2, hw: W / 2, hd: t2 },
    { lx: -W / 2, lz: 0, hw: t2, hd: D / 2 },
    { lx: W / 2, lz: 0, hw: t2, hd: D / 2 },
    { lx: -segOff, lz: D / 2, hw: segW / 2, hd: t2 },
    { lx: segOff, lz: D / 2, hw: segW / 2, hd: t2 },
  ];
  for (const wd of wallDefs) {
    const p = w2(wd.lx, wd.lz);
    walkColliders.push({ x: p.x, z: p.z, hw: wd.hw, hd: wd.hd, rot: yaw });
  }
  walkDecks.push({ x, z, hw: W / 2, hd: D / 2, rot: yaw, y: y + 0.12, kind: 'house' });
  return g;
}

function makeFarm(x, z, y) {
  const g = new THREE.Group();
  const soil = new THREE.Mesh(new THREE.BoxGeometry(1.7, 0.1, 1.7), new THREE.MeshStandardMaterial({ color: 0x6b5a3e, roughness: 1 }));
  soil.position.y = 0.05;
  soil.receiveShadow = true;
  g.add(soil);
  g.position.set(x, y, z);
  scene.add(g);
  g.userData.farm = { x, z, timer: -1 };
  return g;
}

function placeBuild() { return; }
function cycleBuild() { return; }

function updateBuildBtn() {
  const b = null;
  if (b) b.textContent = '';
}

function updateWalkCamera() {
  const eyeY = player.y + 1.55;
  if (state.walkView === 'first') {
    camera.position.set(player.x, eyeY + 0.05, player.z);
    const dx = Math.sin(player.yaw) * Math.cos(player.pitch);
    const dy = Math.sin(player.pitch);
    const dz = Math.cos(player.yaw) * Math.cos(player.pitch);
    camera.lookAt(player.x + dx, eyeY + 0.05 + dy, player.z + dz);
  } else {
    const baseDist = 6.5;
    const cp = Math.cos(player.pitch);
    const sp = Math.sin(player.pitch);
    let px = player.x;
    let py = eyeY + 0.5;
    let pz = player.z;
    for (let k = 0; k < 9; k++) {
      const d = baseDist * (1 - k * 0.1);
      const cx = player.x - Math.sin(player.yaw) * cp * d;
      const cz = player.z - Math.cos(player.yaw) * cp * d;
      const cy = eyeY - sp * d + 0.5;
      const gh = terrainHeight(cx, cz) + 0.8;
      const wantY = Math.max(cy, eyeY);
      if (wantY >= gh) {
        px = cx; py = wantY; pz = cz;
        break;
      }
      px = cx; py = gh; pz = cz;
    }
    camera.position.set(px, py, pz);
    camera.lookAt(player.x, eyeY - 0.2, player.z);
  }
}

function updateSunFocus() {
  if (!sunLight) return;
  const tx = controls.target.x;
  const tz = controls.target.z;
  sunLight.target.position.set(tx, 0, tz);
  sunLight.position.copy(sunDir).multiplyScalar(620);
  sunLight.position.x += tx;
  sunLight.position.z += tz;
  sunLight.position.y = Math.max(sunLight.position.y, 15);
  sunLight.target.updateMatrixWorld();
}

function animate() {
  requestAnimationFrame(animate);
  const raw = dtClamp(clock.getDelta());
  const elapsed = clock.elapsedTime;

  if (state.mode === 'walk') {
    attackCD = Math.max(0, attackCD - raw);
    updatePlayer(raw);
    updateFarmRegrow(raw);
    updateWeather(raw);
    updateHunger(raw);
    updateThreat(raw);
    ecology.playerX = player.x;
    ecology.playerZ = player.z;
    ecology.torch = null; // 观察者模式，无火把
    if (!ecoHudT || clock.elapsedTime - ecoHudT > 0.5) { ecoHudT = clock.elapsedTime; updateEcoHud(); updateInvUI(); updateMiles(); updateEnvPanels(); }
    updateWalkCamera();
    updatePrompt();
    renderPass.camera = camera;
  } else if (state.iso) {
    updateIsoCamera();
    if (!isoDrag && state.started) isoAz += raw * 0.045;
    renderPass.camera = isoCam;
  } else {
    renderPass.camera = camera;
  }

  if (state.started) {
    state.gameHours += raw / 48.0;
    state.day = Math.floor(state.gameHours / 24) + 1;
    const wrapped = ((state.gameHours % 24) + 24) % 24;
    env.setTime(wrapped);
    if (!state.timeScrubbing) $('time-slider').value = wrapped.toFixed(2);
    updateClock();
  }

  updateSunFocus();
  env.update(raw, elapsed);
  renderer.toneMappingExposure *= state.iso ? 1.18 : 1;
  landmarks.setIso(state.iso);
  landmarks.update(raw, elapsed);
  grade.uniforms.uSat.value = state.iso ? 1.25 : 1.045;
  grade.uniforms.uTime.value = elapsed;
  bloom.strength = (state.iso ? 0.5 : 0.2) + (1 - smoothstep(-0.06, 0.22, env.sunElevation)) * 0.13;
  grade.uniforms.uVignette.value = 0.24 + (1 - smoothstep(-0.04, 0.25, env.sunElevation)) * 0.08;

  if (state.started && !state.paused) {
    const sim = raw * state.speed;
    state.ecologyAccumulator += sim;
    state.grassAcc += raw;
    if (state.grassAcc >= (state.mode === 'walk' ? 0.4 : 1.5)) {
      state.grassAcc = 0;
      terrain.updateGrassVisual(ecology.grass, ecology.GRID_N, ecology.GRID_SPAN, state.mode === 'walk' ? { x: player.x, z: player.z } : null, ecology.grassP);
    }
    state.shadeAcc += raw;
    if (state.shadeAcc >= 20) {
      state.shadeAcc = 0;
      refreshShade();
    }
    const step = 0.1;
    let guard = 0;
    const daylight = clamp((env.sunElevation + 0.03) / 0.27, 0, 1);
    while (state.ecologyAccumulator >= step && guard < 26) {
      ecology.update(step, daylight);
      terrain.updateTrees(step);
      state.ecologyAccumulator -= step;
      guard++;
    }
    if (guard >= 26) state.ecologyAccumulator = 0;
  }
  const alpha = state.paused || !state.started ? 1 : clamp(state.ecologyAccumulator / 0.1, 0, 1);
  agents.update(alpha, elapsed);

  if (selected) {
    if (!selected.alive) {
      selected = null;
      $('inspect').classList.remove('show');
    } else if (hoverRing) {
      const sy = selected.y + selected.genome.size * 2.1;
      hoverRing.position.set(selected.x, sy, selected.z);
      hoverRing.material.color.setHex(0xf2c879);
      hoverRing.scale.setScalar(2.6);
      hoverRing.visible = true;
    }
  }

  for (let i = pulses.length - 1; i >= 0; i--) {
    const p = pulses[i];
    p.age += raw;
    const k = p.age / 1.35;
    if (k >= 1) {
      scene.remove(p.mesh);
      p.mesh.geometry.dispose();
      p.mesh.material.dispose();
      pulses.splice(i, 1);
    } else {
      const s = 1 + k * 7;
      p.mesh.scale.set(s, s, s);
      p.mesh.material.opacity = 0.9 * (1 - k) * (1 - k);
    }
  }

  moveCamera(raw);
  if (state.mode !== 'walk') controls.update();

  if (state.started && performance.now() - state.lastStats > 380) {
    state.lastStats = performance.now();
    updateStats();
    updateInspectPanel();
  }

  composer.render();
}

const keys = new Set();
window.addEventListener('keydown', (e) => {
  if (e.target && /input|textarea/i.test(e.target.tagName)) return;
  keys.add(e.code);
});
window.addEventListener('keyup', (e) => keys.delete(e.code));
window.addEventListener('blur', () => keys.clear());

function moveCamera(dt) {
  if (!state.started || state.iso || state.mode === 'walk') return;
  const w = keys.has('KeyW') || keys.has('ArrowUp');
  const s = keys.has('KeyS') || keys.has('ArrowDown');
  const a = keys.has('KeyA') || keys.has('ArrowLeft');
  const d = keys.has('KeyD') || keys.has('ArrowRight');
  const up = keys.has('KeyE');
  const down = keys.has('KeyQ');
  if (!w && !s && !a && !d && !up && !down) return;

  const speed = (keys.has('ShiftLeft') || keys.has('ShiftRight') ? 120 : 34) * dt;
  const forward = new THREE.Vector3();
  camera.getWorldDirection(forward);
  forward.y = 0;
  forward.normalize();
  const right = new THREE.Vector3().crossVectors(forward, camera.up).normalize();
  const move = new THREE.Vector3();
  if (w) move.add(forward);
  if (s) move.sub(forward);
  if (d) move.add(right);
  if (a) move.sub(right);
  if (up) move.y += 1;
  if (down) move.y -= 1;
  if (move.lengthSq() > 0) {
    move.normalize().multiplyScalar(speed);
    camera.position.add(move);
    controls.target.add(move);
    const minY = terrainHeight(camera.position.x, camera.position.z) + 2.2;
    if (camera.position.y < minY) camera.position.y = minY;
    if (controls.target.y < 1) controls.target.y = 1;
    state.orbit = false;
    controls.autoRotate = false;
    $('orbit-btn')?.classList.remove('on');
  }
}

async function boot() {
  try {
    setLoad('正在计算高度场与湖盆…');
    await nextFrame();

    setupRenderer();
    setupPost();

    env = createEnvironment(scene, renderer, state.gameHours % 24);
    sunLight = env.sunLight;
    sunDir = env.sunDir;
    sunLight.shadow.mapSize.set(quality.shadowMap, quality.shadowMap);

    setLoad('正在生长山脉、森林与草地…');
    await nextFrame();
    terrain = createTerrain(scene, quality);

    setLoad('正在唤醒第一代生命…');
    await nextFrame();
    landmarks = createLandmarks(scene);
    ecology = new Ecology(20261102);
    agents = createAgentRenderer(scene, ecology);

    setupUI();
    setupPointer();
    updateStats();
    updateClock();

    setLoad('正在编译光线与大气着色器…');
    await nextFrame();
    renderer.compile(scene, camera);

    updateSunFocus();
    composer.render();
    setLoad('世界已就绪 · 点击进入');
    const enter = $('enter');
    enter.disabled = false;
    $('enter-label').textContent = '进入世界';

    window.__aether = {
      scene, camera, isoCam, renderer, controls, ecology, terrain, agents, landmarks,
      env, quality, state, composer, bloom, grade,
      inspectAt, tryPlace, refreshShade, setMode, toggleWalkView, player, terrainHeight, updateWalkCamera,
      updatePlayer, keys, joy,
      walkInteract, nearestTarget, updateInvUI, updatePrompt,
      placeBuild, cycleBuild, structures,
      walkDecks, walkColliders, groundAt,
      updateHunger, updateThreat, updateEcoHud, updateStats, updateFarmRegrow, updateWeather,
      updateEnvPanels, toggleReplay, drawReplay,
      updateGrass: (tr) => terrain.updateGrassVisual(ecology.grass, ecology.GRID_N, ecology.GRID_SPAN, tr, ecology.grassP),
      isoSet: (az, elv, half, tx, ty, tz) => {
        if (az !== undefined) isoAz = az;
        if (elv !== undefined) isoElv = elv;
        if (half !== undefined) isoHalf = half;
        if (tx !== undefined) isoTarget.set(tx, ty, tz);
        updateIsoCamera();
      },
      info: () => ({
        calls: renderer.info.render.calls,
        triangles: renderer.info.render.triangles,
        fps: state.fps,
        herb: ecology.stats().h,
        pred: ecology.stats().p,
      }),
    };

    requestAnimationFrame(animate);
    setTimeout(() => {
      console.info(
        '[aether] ready',
        'trees=' + terrain.treeCount(),
        'agents=' + (ecology.stats().h + ecology.stats().p),
        'quality=' + JSON.stringify(quality)
      );
    }, 500);
  } catch (err) {
    console.error(err);
    setLoad('加载失败：' + (err && err.message ? err.message : String(err)), true);
    const btn = $('enter');
    if (btn) btn.disabled = true;
  }
}

window.addEventListener('error', (e) => {
  setLoad('运行时错误：' + (e.message || '未知错误'), true);
});

boot();
