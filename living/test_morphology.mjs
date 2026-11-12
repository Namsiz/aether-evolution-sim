// test_morphology.mjs — Node smoke test for Phase 19 morphology pipeline.
//
// Usage: node living/test_morphology.mjs
// No DOM required — tests pure derivation + Three.js geometry creation in Node.

import * as THREE from 'three';
import {
  deriveMorphology,
  buildSpeciesMesh,
  buildSpeciesBaseGeometry,
  lodLevelForDist,
  applyHullRatioToScale,
  createMorphologyCompareMode,
} from './src/morphology.mjs';
import {
  createMorphologyMaterial,
  updateMorphoTime,
  getLODLevel,
  setLODLevel,
  LOD_THRESHOLDS,
  MESH_ANIM_UNIFORMS,
} from './src/morphology-shader.mjs';

let failures = 0;
const check = (name, cond, detail = '') => {
  if (cond) console.log(`  ok   ${name}${detail ? ` — ${detail}` : ''}`);
  else { console.error(`  FAIL ${name}${detail ? ` — ${detail}` : ''}`); failures += 1; }
};

console.log('Phase 19 morphology pipeline:\n');

// ─── 1. deriveMorphology: range + determinism ───────────────────────────────
{
  const m = deriveMorphology(42);
  check('hull_ratio in [0.6, 1.4]', m.hull_ratio >= 0.6 && m.hull_ratio <= 1.4,
    `got ${m.hull_ratio.toFixed(4)}`);
  check('limb_scale in [2.0, 4.0]', m.limb_scale >= 2.0 && m.limb_scale <= 4.0,
    `got ${m.limb_scale.toFixed(4)}`);
  check('aPhase in [0, 2π)', m.aPhase >= 0 && m.aPhase < Math.PI * 2,
    `got ${m.aPhase.toFixed(4)}`);
  check('aSwayAmp in [0.5, 1.0]', m.aSwayAmp >= 0.5 && m.aSwayAmp <= 1.0,
    `got ${m.aSwayAmp.toFixed(4)}`);

  // Determinism: same ID → same values
  const m2 = deriveMorphology(42);
  check('determinism: same ID → same hull_ratio', m.hull_ratio === m2.hull_ratio);
  check('determinism: same ID → same aPhase', m.aPhase === m2.aPhase);

  // Different IDs give different values
  const m3 = deriveMorphology(43);
  check('different ID → different hull_ratio', m.hull_ratio !== m3.hull_ratio,
    `id=42: ${m.hull_ratio.toFixed(6)}, id=43: ${m3.hull_ratio.toFixed(6)}`);
}

// ─── 2. hull_ratio=1.0 sanity: symmetric scale ──────────────────────────────
{
  const scale = new THREE.Vector3();
  applyHullRatioToScale(scale, 1.0, 1.0);
  check('hull_ratio=1.0 → x=1, y=1, z=1', scale.x === 1 && scale.y === 1 && scale.z === 1,
    `got (${scale.x}, ${scale.y}, ${scale.z})`);

  applyHullRatioToScale(scale, 1.0, 1.4);
  check('hull_ratio=1.4 → x > z (elongated)', scale.x > scale.z,
    `x=${scale.x.toFixed(3)}, z=${scale.z.toFixed(3)}`);

  applyHullRatioToScale(scale, 1.0, 0.6);
  check('hull_ratio=0.6 → x < z (compact)', scale.x < scale.z,
    `x=${scale.x.toFixed(3)}, z=${scale.z.toFixed(3)}`);

  // Volume preservation: |x| × |z| ≈ 1 for base scale 1
  applyHullRatioToScale(scale, 1.0, 1.2);
  const vol = Math.abs(scale.x * scale.z);
  check('volume ≈ preserved (±5%)', Math.abs(vol - 1.0) < 0.05,
    `x*z = ${vol.toFixed(4)}`);
}

// ─── 3. limb_scale → fin count via buildSpeciesMesh ─────────────────────────
{
  // Build meshes with different limb_scale values
  const geo2 = buildSpeciesMesh('herb', { limb_scale: 2.0, size: 1.0, hull_ratio: 1.0 }, 0, false, THREE);
  const geo3 = buildSpeciesMesh('herb', { limb_scale: 3.0, size: 1.0, hull_ratio: 1.0 }, 0, false, THREE);
  const geo4 = buildSpeciesMesh('herb', { limb_scale: 4.0, size: 1.0, hull_ratio: 1.0 }, 0, false, THREE);

  const v2 = geo2.geometry.attributes.position.count;
  const v3 = geo3.geometry.attributes.position.count;
  const v4 = geo4.geometry.attributes.position.count;
  check('limb_scale=2: vertex count < limb_scale=3', v2 < v3, `${v2} < ${v3}`);
  check('limb_scale=3: vertex count < limb_scale=4', v3 < v4, `${v3} < ${v4}`);
  check('buildSpeciesMesh returns geometry', !!geo2.geometry);
  check('buildSpeciesMesh returns dispose fn', typeof geo2.dispose === 'function');

  geo2.dispose(); geo3.dispose(); geo4.dispose();
}

// ─── 4. Fossil rendering: grey + transparent ────────────────────────────────
{
  const fossil = buildSpeciesMesh('pred', { size: 1.0, hull_ratio: 1.0 }, 0, true, THREE);
  check('fossil material color = 0x888888', fossil.material.color.getHex() === 0x888888,
    `got 0x${fossil.material.color.getHex().toString(16)}`);
  check('fossil material transparent', fossil.material.transparent === true);
  check('fossil material opacity = 0.4', Math.abs(fossil.material.opacity - 0.4) < 0.001,
    `got ${fossil.material.opacity}`);
  fossil.dispose();

  // Non-fossil: white opaque
  const alive = buildSpeciesMesh('pred', { size: 1.0, hull_ratio: 1.0 }, 0, false, THREE);
  check('alive material color = 0xffffff', alive.material.color.getHex() === 0xffffff);
  check('alive material not transparent', alive.material.transparent === false);
  alive.dispose();
}

// ─── 5. buildSpeciesBaseGeometry (LOD) ──────────────────────────────────────
{
  for (const kind of ['herb', 'pred', 'scav', 'omn', 'fish', 'deco']) {
    for (const lod of [0, 1, 2]) {
      const geo = buildSpeciesBaseGeometry(kind, lod, THREE);
      check(`LOD${lod} ${kind}: geometry has position attr`, !!geo.attributes.position);
      geo.dispose();
    }
  }
  // LOD-0 has more vertices than LOD-2
  const herb0 = buildSpeciesBaseGeometry('herb', 0, THREE);
  const herb2 = buildSpeciesBaseGeometry('herb', 2, THREE);
  check('LOD-0 > LOD-2 vertex count (herb)',
    herb0.attributes.position.count > herb2.attributes.position.count,
    `${herb0.attributes.position.count} > ${herb2.attributes.position.count}`);
  herb0.dispose(); herb2.dispose();
}

// ─── 6. LOD threshold functions ─────────────────────────────────────────────
{
  check('lodLevelForDist(50) = 0', lodLevelForDist(50) === 0);
  check('lodLevelForDist(200) = 1', lodLevelForDist(200) === 1);
  check('lodLevelForDist(500) = 2', lodLevelForDist(500) === 2);
  check('getLODLevel(50) = 0', getLODLevel(50) === 0);
  check('getLODLevel(200) = 1', getLODLevel(200) === 1);
  check('getLODLevel(500) = 2', getLODLevel(500) === 2);
  check('LOD_THRESHOLDS.lod0 = 100', LOD_THRESHOLDS.lod0 === 100);
  check('LOD_THRESHOLDS.lod1 = 400', LOD_THRESHOLDS.lod1 === 400);
}

// ─── 7. createMorphologyMaterial + updateMorphoTime ────────────────────────
{
  const mat = createMorphologyMaterial(THREE, { color: 0x00ff00 });
  check('material is MeshLambertMaterial', mat.isMeshLambertMaterial === true);
  check('material has morphoUniforms', !!mat.userData.morphoUniforms);
  updateMorphoTime(mat, 12.5);
  check('uTime updated to 12.5', mat.userData.morphoUniforms.uTime.value === 12.5);
  check('material color = 0x00ff00', mat.color.getHex() === 0x00ff00);
  mat.dispose();
}

// ─── 8. MESH_ANIM_UNIFORMS exported ─────────────────────────────────────────
{
  check('MESH_ANIM_UNIFORMS.uTime exists', 'uTime' in MESH_ANIM_UNIFORMS);
}

// ─── 9. createMorphologyCompareMode (mock scene) ────────────────────────────
{
  // Minimal mock scene
  const mockScene = {
    children: [],
    add(...objs) { for (const o of objs) this.children.push(o); },
    remove(obj) { const i = this.children.indexOf(obj); if (i >= 0) this.children.splice(i, 1); },
  };
  const cmp = createMorphologyCompareMode(mockScene, THREE);
  check('compare mode: initially not visible', cmp.visible === false);
  const nowVisible = cmp.toggle();
  check('compare mode: toggle → visible', nowVisible === true);
  cmp.toggle();
  check('compare mode: toggle back → hidden', cmp.visible === false);
  cmp.dispose();
}

// ─── 10. setLODLevel on a mock mesh ─────────────────────────────────────────
{
  const geo0 = new THREE.SphereGeometry(1, 10, 8);
  const geo1 = new THREE.SphereGeometry(1, 8, 6);
  const geo2 = new THREE.SphereGeometry(1, 6, 4);
  const mat = new THREE.MeshLambertMaterial();
  const mockMesh = new THREE.InstancedMesh(geo0, mat, 10);
  mockMesh.geometry = geo0;
  mockMesh.userData.lodGeos = [geo0, geo1, geo2];

  setLODLevel(mockMesh, 2, THREE);
  check('setLODLevel(2) swaps geometry', mockMesh.geometry === geo2);

  setLODLevel(mockMesh, 0, THREE);
  check('setLODLevel(0) swaps geometry back', mockMesh.geometry === geo0);

  geo0.dispose(); geo1.dispose(); geo2.dispose(); mat.dispose();
}

// ─── Summary ────────────────────────────────────────────────────────────────
console.log(`\n${failures === 0 ? 'PASS' : 'FAIL'}: ${failures === 0 ? 'all' : `${failures}`} of ${failures === 0 ? 'all' : 'some'} checks ${failures === 0 ? 'passed' : 'failed'}`);
process.exit(failures > 0 ? 1 : 0);
