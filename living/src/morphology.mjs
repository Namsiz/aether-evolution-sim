// morphology.mjs — Phase 19: per-agent visual morphology from genome + agent ID.
//
// Deterministic derivation (matches Rust `Ecology::morpho_pair`):
//   hull_ratio = 0.6 + ((id * 0x9E3779B9 & 0xFFFF) / 65535) * 0.8
//   limb_scale = 2.0 + (((id * 0x9E3779B9) >> 16 & 0xFFFF) / 65535) * 2.0
//
// No RNG is consumed on the Rust side; the same integer hash is replicable
// from the agent ID already passed through the C-ABI (SLOT_ID).

/**
 * Derive morphology parameters from an agent ID.
 * @param {number} agentId — u32, from SLOT_ID
 * @returns {{ hull_ratio: number, limb_scale: number, aPhase: number, aSwayAmp: number }}
 */
export function deriveMorphology(agentId) {
  const h = (agentId * 0x9E3779B9) >>> 0;
  const hull_ratio = 0.6 + ((h & 0xFFFF) / 65535.0) * 0.8;
  const limb_scale = 2.0 + (((h >>> 16) & 0xFFFF) / 65535.0) * 2.0;
  // Per-instance animation attributes (deterministic from ID)
  const aPhase = (agentId * 0x61C88647 >>> 0) / 0x100000000 * Math.PI * 2;
  const aSwayAmp = 0.5 + 0.5 * ((agentId * 0x85EBCA77 >>> 0) / 0x100000000);
  return { hull_ratio, limb_scale, aPhase, aSwayAmp };
}

// ─── LOD thresholds ──────────────────────────────────────────────────────────
export const LOD_DIST_0 = 100;  // < 100: full detail
export const LOD_DIST_1 = 400;  // < 400: medium
// ≥ 400: low (LOD-2)

/**
 * Pick the LOD level index (0/1/2) based on camera-to-agent distance.
 * @param {number} dist — world units
 * @returns {0|1|2}
 */
export function lodLevelForDist(dist) {
  if (dist < LOD_DIST_0) return 0;
  if (dist < LOD_DIST_1) return 1;
  return 2;
}

/**
 * Build a per-agent BufferGeometry for a given species and genome configuration.
 * Used by the V-key "morphology compare mode" to show individual agents.
 *
 * @param {string} kind — one of 'herb','pred','scav','omn','fish','deco'
 * @param {{ hull_ratio?: number, limb_scale?: number, size?: number,
 *           fin_color?: number, deco_efficiency?: number }} genome
 * @param {number} [cladeId=0]
 * @param {boolean} [isFossil=false]
 * @param {import('three')} THREE
 * @returns {{ geometry: import('three').BufferGeometry, material: import('three').Material, dispose: () => void }}
 */
export function buildSpeciesMesh(kind, genome, cladeId, isFossil, THREE) {
  const hr = genome.hull_ratio ?? 1.0;
  const ls = genome.limb_scale ?? 3.0;
  const size = genome.size ?? 1.0;
  const finColor = genome.fin_color ?? 0.5;
  const decoEff = genome.deco_efficiency ?? 1.0;

  // Body section count: high efficiency → compact (3), low → elongated (5)
  const sections = Math.round(3 + (1.5 - Math.min(decoEff, 1.5)) * 1.33);
  const numSections = Math.max(3, Math.min(5, sections));

  // Base body: ellipsoid (scaled sphere) with hull_ratio modulation
  const baseSegW = [8, 6, 4][lodLevelForDist(200)]; // medium detail for individual meshes
  const baseSegH = [6, 5, 3][lodLevelForDist(200)];
  const bodyGeo = new THREE.SphereGeometry(1, baseSegW, baseSegH);
  bodyGeo.scale(size * hr, size, size / Math.sqrt(hr));

  // Fin/limb construction: number = round(limb_scale), size varies by kind
  const numFins = Math.round(ls); // 2, 3, or 4
  const finGeos = [];
  for (let f = 0; f < numFins; f++) {
    const finGeo = new THREE.ConeGeometry(0.15 * size, 0.5 * size, 4);
    // Position fins around the body
    const angle = (f / numFins) * Math.PI * 2;
    const r = 0.8 * size;
    finGeo.translate(
      Math.cos(angle) * r,
      0,
      Math.sin(angle) * r
    );
    finGeo.rotateX(Math.PI / 4);
    finGeos.push(finGeo);
  }

  // Tail (cone pointing backward) with fin_color tint
  const tailGeo = new THREE.ConeGeometry(0.3 * size, 0.8 * size, 6);
  tailGeo.rotateZ(-Math.PI / 2);
  tailGeo.translate(0, 0, -size * hr * 0.6);

  // Merge all into one BufferGeometry
  const merged = mergeGeometries(THREE, [bodyGeo, ...finGeos, tailGeo]);

  // Material: fossil = grey transparent; normal = white (instanceColor drives)
  let material;
  if (isFossil) {
    material = new THREE.MeshLambertMaterial({
      color: 0x888888,
      transparent: true,
      opacity: 0.4,
    });
  } else {
    material = new THREE.MeshLambertMaterial({ color: 0xffffff });
  }

  function dispose() {
    merged.dispose();
    material.dispose();
    for (const g of [bodyGeo, ...finGeos, tailGeo]) g.dispose();
  }

  return { geometry: merged, material, dispose };
}

/**
 * Minimal BufferGeometry merge (position + normal + uv), no index merge.
 * For simple diagnostic meshes this is sufficient.
 */
function mergeGeometries(THREE, geos) {
  let totalVerts = 0;
  for (const g of geos) totalVerts += g.attributes.position.count;

  const positions = new Float32Array(totalVerts * 3);
  const normals = new Float32Array(totalVerts * 3);
  const uvs = new Float32Array(totalVerts * 2);
  let offset = 0;

  for (const g of geos) {
    const pos = g.attributes.position;
    const nor = g.attributes.normal;
    const uv = g.attributes.uv;
    for (let i = 0; i < pos.count; i++) {
      positions[(offset + i) * 3] = pos.getX(i);
      positions[(offset + i) * 3 + 1] = pos.getY(i);
      positions[(offset + i) * 3 + 2] = pos.getZ(i);
      if (nor) {
        normals[(offset + i) * 3] = nor.getX(i);
        normals[(offset + i) * 3 + 1] = nor.getY(i);
        normals[(offset + i) * 3 + 2] = nor.getZ(i);
      }
      if (uv) {
        uvs[(offset + i) * 2] = uv.getX(i);
        uvs[(offset + i) * 2 + 1] = uv.getY(i);
      }
    }
    offset += pos.count;
  }

  const geo = new THREE.BufferGeometry();
  geo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  geo.setAttribute('normal', new THREE.BufferAttribute(normals, 3));
  geo.setAttribute('uv', new THREE.BufferAttribute(uvs, 2));
  return geo;
}

/**
 * Build the base geometry for a species InstancedMesh at a given LOD level.
 * @param {string} kind
 * @param {0|1|2} lod
 * @param {import('three')} THREE
 * @returns {import('three').BufferGeometry}
 */
export function buildSpeciesBaseGeometry(kind, lod, THREE) {
  // LOD-0: full, LOD-1: medium, LOD-2: low polygon count
  const segs = [
    [10, 8], // LOD-0
    [8, 6],  // LOD-1
    [6, 4],  // LOD-2
  ][lod];

  switch (kind) {
    case 'herb': return new THREE.SphereGeometry(1.0, segs[0], segs[1]);
    case 'pred': { const g = new THREE.ConeGeometry(0.9, 2.6, segs[0]); g.rotateX(Math.PI / 2); return g; }
    case 'scav': { const g = new THREE.ConeGeometry(0.7, 2.0, Math.max(4, segs[0] - 2)); g.rotateX(Math.PI / 2); return g; }
    case 'omn':  return new THREE.SphereGeometry(0.85, segs[0], segs[1]);
    case 'fish': { const g = new THREE.ConeGeometry(0.45, 2.2, Math.max(4, segs[0] - 2)); g.rotateX(Math.PI / 2); return g; }
    case 'deco': return new THREE.OctahedronGeometry(0.7, lod === 0 ? 1 : 0);
    default:     return new THREE.SphereGeometry(1, segs[0], segs[1]);
  }
}

/**
 * Add per-instance animation attributes (aPhase, aSwayAmp) to an InstancedMesh.
 * Must be called after the mesh is created and before the first render.
 * @param {import('three').InstancedMesh} mesh
 * @param {number} cap — max instances
 * @param {import('three')} THREE
 */
export function addAnimAttributes(mesh, cap, THREE) {
  const phaseArr = new Float32Array(cap);
  const swayArr = new Float32Array(cap);
  const aPhase = new THREE.InstancedBufferAttribute(phaseArr, 1);
  const aSwayAmp = new THREE.InstancedBufferAttribute(swayArr, 1);
  aPhase.setUsage(THREE.DynamicDrawUsage);
  aSwayAmp.setUsage(THREE.DynamicDrawUsage);
  // Attach to the geometry (per-instance attributes live on geometry)
  mesh.geometry.setAttribute('aPhase', aPhase);
  mesh.geometry.setAttribute('aSwayAmp', aSwayAmp);
  mesh.userData.animAttrs = { aPhase, aSwayAmp };
}

/**
 * Update per-instance animation attributes for agents.
 * @param {import('three').InstancedMesh} mesh
 * @param {number[]} agentIds — the IDs in instance order
 * @param {import('three')} THREE
 */
export function updateAnimAttributes(mesh, agentIds, THREE) {
  const { aPhase, aSwayAmp } = mesh.userData.animAttrs;
  if (!aPhase) return;
  for (let i = 0; i < agentIds.length; i++) {
    const m = deriveMorphology(agentIds[i]);
    aPhase.array[i] = m.aPhase;
    aSwayAmp.array[i] = m.aSwayAmp;
  }
  aPhase.needsUpdate = true;
  aSwayAmp.needsUpdate = true;
}

/**
 * Apply hull_ratio to the instance matrix (area-preserving anisotropic scale).
 * Elliptical cross-section: x ∝ √hr, z ∝ 1/√hr → x·z = base².
 * @param {import('three').Vector3} scale — will be modified
 * @param {number} baseScale — from genome.size mapping
 * @param {number} hull_ratio — 0.6–1.4
 */
export function applyHullRatioToScale(scale, baseScale, hull_ratio) {
  const s = Math.sqrt(hull_ratio);
  scale.x = baseScale * s;
  scale.y = baseScale;
  scale.z = baseScale / s;
}

// ─── Morphology compare mode (V key) ─────────────────────────────────────────

/**
 * Create the "morphology compare" overlay: shows two agents from the same clade
 * side by side with their individual geometries.
 * @param {import('three').Scene} scene
 * @param {import('three')} THREE
 * @returns {{ toggle: () => void, update: (agents: Map<number, {kind: string, id: number}>) => void, dispose: () => void }}
 */
export function createMorphologyCompareMode(scene, THREE) {
  let visible = false;
  let meshA = null, meshB = null;
  let group = null;

  function disposeMesh(m) {
    if (m) {
      m.geometry.dispose();
      m.material.dispose();
    }
  }

  function show(agentA, agentB) {
    disposeMesh(meshA); disposeMesh(meshB);
    const specA = buildSpeciesMesh(agentA.kind, deriveMorphology(agentA.id), 0, false, THREE);
    const specB = buildSpeciesMesh(agentB.kind, deriveMorphology(agentB.id), 0, false, THREE);
    meshA = new THREE.Mesh(specA.geometry, specA.material);
    meshA.position.set(-3, 1.5, 0);
    meshB = new THREE.Mesh(specB.geometry, specB.material);
    meshB.position.set(3, 1.5, 0);
    if (!group) {
      group = new THREE.Group();
      group.name = 'morphoCompare';
      scene.add(group);
    }
    group.visible = visible;
    group.add(meshA, meshB);
  }

  return {
    toggle() {
      visible = !visible;
      if (group) group.visible = visible;
      return visible;
    },
    show,
    get visible() { return visible; },
    dispose() {
      disposeMesh(meshA); disposeMesh(meshB);
      if (group) { scene.remove(group); group = null; }
    },
  };
}
