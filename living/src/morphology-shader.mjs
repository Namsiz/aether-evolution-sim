// morphology-shader.mjs — Phase 19: vertex animation shader (swim bob + tail sway).
//
// Injected into MeshLambertMaterial via onBeforeCompile so the main render path
// keeps Three.js built-in lighting + instanceColor, with per-instance animation
// added on top (zero CPU cost: all in GPU vertex shader).
//
// Per-instance attributes (set on geometry):
//   aPhase   — float, 0..2π, seeded from agent ID
//   aSwayAmp — float, 0..1, modulated by deco_efficiency
//
// Uniform (updated by JS each frame):
//   uTime    — elapsed seconds

/**
 * GLSL vertex shader injection (prepended to the Lambert vertex main).
 * After the standard `#include <begin_vertex>` block, we add:
 *   1. bob: world Y offset = sin(uTime*2 + aPhase) * 0.30 * aSwayAmp
 *      (doubled from 0.15 for more visible breathing motion)
 *   2. tail sway: for vertices with z < -0.3 * bodySize, X offset =
 *      sin(uTime*1.5 + aPhase + PI/3) * 0.18 * aSwayAmp
 *      (doubled from 0.08 for more visible undulation)
 */
export const VERTEX_SHADER_INJECTION = /* glsl */ `
  #ifdef USE_INSTANCING
    attribute float aPhase;
    attribute float aSwayAmp;
    uniform float uTime;
  #endif

  #include <begin_vertex>

  #ifdef USE_INSTANCING
    // Swim bob — doubled amplitude for visibility
    transformed.y += sin(uTime * 2.0 + aPhase) * 0.30 * aSwayAmp;
    // Tail sway — doubled amplitude for visibility
    if (position.z < -0.3) {
      transformed.x += sin(uTime * 1.5 + aPhase + 3.14159265 * 0.333) * 0.18 * aSwayAmp;
    }
  #endif
`;

/**
 * Create a Lambert material with the morphology vertex animation injected.
 * @param {{ transparent?: boolean, opacity?: number, color?: number }} opts
 * @param {import('three')} THREE
 * @returns {import('three').Material} — a MeshLambertMaterial with onBeforeCompile patch
 */
export function createMorphologyMaterial(THREE, opts = {}) {
  const mat = new THREE.MeshLambertMaterial({
    color: opts.color ?? 0xffffff,
    transparent: opts.transparent ?? false,
    opacity: opts.opacity ?? 1.0,
  });

  // Uniform holder so we can update uTime from JS without recompiling
  const uniforms = { uTime: { value: 0.0 } };
  mat.userData.morphoUniforms = uniforms;

  mat.onBeforeCompile = (shader) => {
    shader.uniforms.uTime = uniforms.uTime;
    // Inject after the standard includes at the top of the vertex shader
    shader.vertexShader = shader.vertexShader
      .replace('#include <common>', `#include <common>
        attribute float aPhase;
        attribute float aSwayAmp;
        uniform float uTime;`)
      .replace('#include <begin_vertex>', /* glsl */ `#include <begin_vertex>
        // Phase 19 morphology: swim bob + tail sway (per-instance, GPU-only)
        transformed.y += sin(uTime * 2.0 + aPhase) * 0.15 * aSwayAmp;
        if (position.z < -0.3) {
          transformed.x += sin(uTime * 1.5 + aPhase + 1.047) * 0.08 * aSwayAmp;
        }`);
  };

  return mat;
}

/**
 * Update the uTime uniform on a material created by createMorphologyMaterial.
 * Call once per frame in the render loop.
 * @param {import('three').Material} material
 * @param {number} elapsed — seconds since world start
 */
export function updateMorphoTime(material, elapsed) {
  if (material.userData.morphoUniforms) {
    material.userData.morphoUniforms.uTime.value = elapsed;
  }
}

// ─── LOD switch (per-species, driven by camera distance) ─────────────────────

/**
 * LOD thresholds in world units.
 */
export const LOD_THRESHOLDS = {
  lod0: 100, // < 100: full detail
  lod1: 400, // < 400: medium; ≥ 400: low
};

/**
 * Determine LOD level for a given camera-to-origin distance.
 * @param {number} dist
 * @returns {0|1|2}
 */
export function getLODLevel(dist) {
  if (dist < LOD_THRESHOLDS.lod0) return 0;
  if (dist < LOD_THRESHOLDS.lod1) return 1;
  return 2;
}

/**
 * Apply LOD to a species' InstancedMesh by swapping its geometry.
 * @param {import('three').InstancedMesh} mesh — must have userData.lodGeos = [geo0, geo1, geo2]
 * @param {0|1|2} level
 * @param {import('three')} THREE
 */
export function setLODLevel(mesh, level, THREE) {
  const geos = mesh.userData.lodGeos;
  if (!geos || !geos[level]) return;
  const oldGeo = mesh.geometry;
  // Preserve custom attributes on the new geometry
  mesh.geometry = geos[level];
  // Re-attach per-instance attributes to the new geometry
  if (oldGeo.attributes.aPhase) {
    mesh.geometry.setAttribute('aPhase', oldGeo.attributes.aPhase);
    mesh.geometry.setAttribute('aSwayAmp', oldGeo.attributes.aSwayAmp);
  }
}

/**
 * MESH_ANIM_UNIFORMS — exported for external uniform bookkeeping.
 */
export const MESH_ANIM_UNIFORMS = {
  uTime: { value: 0.0 },
};
