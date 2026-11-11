// render-utils.mjs — shared Three.js viewer scaffolding.
//
// Extracted from the v2.0 main.js boot path so multiple viewers (the JS-ecology
// one and the wasm snapshot one) can share the same renderer/scene/camera/
// environment/terrain/landmarks setup without duplicating terrain logic.
// Post-processing (bloom/SMAA/grade) stays main.js-only: the wasm viewer runs
// bare renders to keep the per-frame budget tight.

import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { createEnvironment } from './sky.js';
import { createTerrain } from './terrain.js';
import { createLandmarks } from './landmarks.js';
import { terrainHeight } from './world.js';

export const VIEWER_QUALITY = {
  terrainSegments: 312,
  grassCount: 36000,
  treeCount: 192,
  treeCapacity: 260,
  bushCount: 390,
  dpr: 1.5,
};

export function createViewerRenderer(canvas) {
  const renderer = new THREE.WebGLRenderer({
    canvas,
    antialias: true,
    powerPreference: 'high-performance',
    stencil: false,
  });
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, VIEWER_QUALITY.dpr));
  renderer.setSize(window.innerWidth, window.innerHeight, false);
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.0;
  renderer.outputColorSpace = THREE.SRGBColorSpace;

  const scene = new THREE.Scene();
  scene.fog = new THREE.FogExp2(0x0b0f13, 0.00062);

  const portrait = window.innerWidth / window.innerHeight < 0.85;
  const camera = new THREE.PerspectiveCamera(
    portrait ? 60 : 52,
    window.innerWidth / window.innerHeight,
    0.4,
    22000,
  );
  camera.position.set(270, terrainHeight(270, 340) + 43, 340);

  const controls = new OrbitControls(camera, canvas);
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

  window.addEventListener('resize', () => {
    const w = window.innerWidth;
    const h = window.innerHeight;
    camera.aspect = w / h;
    camera.fov = w / h < 0.85 ? 60 : 52;
    camera.updateProjectionMatrix();
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, VIEWER_QUALITY.dpr));
    renderer.setSize(w, h, false);
  });

  return { renderer, scene, camera, controls };
}

// env (sky/lights/water), terrain, landmarks — the shared static world.
export function createViewerWorld(scene, renderer) {
  const env = createEnvironment(scene, renderer);
  const terrain = createTerrain(scene, VIEWER_QUALITY);
  const landmarks = createLandmarks(scene);
  return { env, terrain, landmarks };
}

// Keep the directional sun light centered on the orbit target so shadows
// follow the inspected region (mirrors main.js updateSunFocus).
export function updateSunFocus(env, target) {
  env.sunLight.target.position.set(target.x, 0, target.z);
  env.sunLight.position.copy(env.sunDir).multiplyScalar(620);
  env.sunLight.position.x += target.x;
  env.sunLight.position.z += target.z;
  env.sunLight.position.y = Math.max(env.sunLight.position.y, 15);
  env.sunLight.target.updateMatrixWorld();
}
