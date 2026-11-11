// renderer-mineral-mat.mjs — Three.js heatmap material for the mineral field.
// Visualizes where decomposer (DECO) activity has fertilized the soil.
//
// Data contract (Snapshot v3, PascalCase):
//   World.MineralFieldResampled: Float32Array length 38×38 = 1444, values in [0, 1]
//   World.GridN: 76 (source resolution; resampled at 38)
//
// Usage:
//   import { createMineralHeatmapMaterial, updateMineralTexture } from './renderer-mineral-mat.mjs';
//   const { material, texture } = createMineralHeatmapMaterial(38);
//   // add a plane mesh to the scene with this material, then per tick:
//   updateMineralTexture(texture, snapshot.World.MineralFieldResampled);

import * as THREE from 'three';

// ─── Color ramp ──────────────────────────────────────────────────────────────
// v ∈ [0, 0.25]  冷蓝 → 深蓝   (unfertilized / background)
// v ∈ [0.25, 0.5] 深蓝 → 暗黄  (emerging hotspot)
// v ∈ [0.5, 0.75] 暗黄 → 橙红  (active decomposition)
// v ∈ [0.75, 1.0] 橙红 → 亮白红 (peak mineral concentration)

const FRAG = `
  uniform sampler2D tMineral;
  uniform float uOpacity;
  varying vec2 vUv;
  void main() {
    float v = texture2D(tMineral, vUv).r;
    // piecewise color ramp
    vec3 cold = vec3(0.10, 0.18, 0.55);   // deep blue
    vec3 mid  = vec3(0.85, 0.72, 0.20);   // amber
    vec3 hot  = vec3(0.92, 0.20, 0.10);   // red-orange
    vec3 color = mix(cold, mid, smoothstep(0.0, 0.5, v));
    color = mix(color, hot, smoothstep(0.5, 1.0, v));
    float alpha = uOpacity * v;            // transparent where no mineral
    gl_FragColor = vec4(color, alpha);
  }
`;

const VERT = `
  varying vec2 vUv;
  void main() {
    vUv = uv;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
  }
`;

export function createMineralHeatmapMaterial(gridN = 38) {
  const data = new Float32Array(gridN * gridN);
  const texture = new THREE.DataTexture(data, gridN, gridN, THREE.RedFormat, THREE.FloatType);
  texture.magFilter = THREE.LinearFilter;
  texture.minFilter = THREE.LinearFilter;
  texture.wrapS = THREE.ClampToEdgeWrapping;
  texture.wrapT = THREE.ClampToEdgeWrapping;
  texture.needsUpdate = true;

  const material = new THREE.ShaderMaterial({
    uniforms: {
      tMineral: { value: texture },
      uOpacity: { value: 0.65 },
    },
    vertexShader: VERT,
    fragmentShader: FRAG,
    transparent: true,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
  });

  return { material, texture };
}

export function updateMineralTexture(texture, resampledField) {
  if (!resampledField || resampledField.length === 0) return false;
  const data = texture.image.data;
  if (data.length !== resampledField.length) {
    console.warn('[mineral-mat] field length mismatch', data.length, resampledField.length);
    return false;
  }
  for (let i = 0; i < data.length; i++) data[i] = resampledField[i];
  texture.needsUpdate = true;
  return true;
}

export function createMineralHeatmapPlane(worldSpan = 1080, gridN = 38) {
  const { material, texture } = createMineralHeatmapMaterial(gridN);
  const geometry = new THREE.PlaneGeometry(worldSpan, worldSpan);
  const mesh = new THREE.Mesh(geometry, material);
  mesh.rotation.x = -Math.PI / 2;
  mesh.position.y = 0.02; // just above terrain to avoid z-fighting
  mesh.renderOrder = 1;   // draw after terrain
  mesh.frustumCulled = false;
  return { mesh, material, texture };
}
