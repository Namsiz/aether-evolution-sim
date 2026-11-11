// renderer-wasm-viewer.mjs — RS → JS transport protocol stub.
//
// This file is a SKELETON for the future Three.js viewer that consumes
// Snapshot packets produced by aether-sim-ecology (Rust). It does NOT render
// anything — it only pins the Sim → Renderer data contract:
//   1. how to deserialize the JSON payload (with PascalCase keys),
//   2. the exact field shapes the renderer may rely on,
//   3. the instancedMesh / grass-update interface signatures (TODO in Phase 4).
//
// The real main.js (living/src/main.js) is FROZEN in v2.0 architecture; this
// module is additive and must not touch it.

// ─────────────────────────────────────────────────────────────────────────────
// 1. Deserialization — PascalCase keys come straight from Rust serde.
//    A future consumer can snake_case→camelCase if desired, but the packet
//    itself is authoritative and should not be silently renamed.
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Parse a raw JSON string (or object) into a validated Snapshot view.
 * Throws if Meta.Version does not match the consumer's expected version.
 *
 * @param {string|object} payload — the RS snapshot JSON (string) or parsed object
 * @param {number} [expectedVersion=1]
 * @returns {{ meta: object, world: object, agents: Array<object>, stats: object }}
 */
export function parseSnapshot(payload, expectedVersion = 1) {
  const snap = typeof payload === 'string' ? JSON.parse(payload) : payload;
  if (!snap || typeof snap !== 'object') throw new Error('snapshot: not an object');
  if (!snap.Meta) throw new Error('snapshot: missing Meta');
  const version = snap.Meta.Version;
  if (version !== expectedVersion) {
    throw new Error(`snapshot: schema Version mismatch (got ${version}, expect ${expectedVersion})`);
  }
  if (!snap.World || !Array.isArray(snap.World.Grass)) {
    throw new Error('snapshot: missing World.Grass array');
  }
  if (!Array.isArray(snap.Agents)) {
    throw new Error('snapshot: missing Agents array');
  }
  return snap;
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. Field contract (what the renderer may rely on — nothing else is guaranteed)
//
//    Meta:     { Version:number, Engine:string, TimeSeconds:number }
//    World:    { TickCount:number, Season:number, Daylight:number, GridN:number,
//                Grass:number[], GrassBySpecies:number[][], Suit:number[],
//                Moist:number[], Shade:number[] }
//    Agent:    { Id:number, Kind:'herb'|'pred'|'scav'|'omn', X:number(f64), Z:number(f64),
//                Energy:number(f32), Age:number(f32),
//                GenomeSpeed/GenomeSense/GenomeSize/GenomeMetabolism/GenomeLifespan:
//                number(f32), GenomeHue/GenomeBold/GenomeCurious/GenomeSocial:number(f32),
//                GenomeHelpThr/GenomePrefSig/GenomeTolSig: number|null(f32),
//                Lineage:string|null, State:string,
//                Bite:number(f32), Repro:number(f32), Cargo:number[3],
//                Hunger:number(f32∈[0,1]), Image:number, HelpCd:number(f32),
//                Fear:number(f32), Think:number(f32), TimeSinceLastEat:number,
//                GenomeSizeVisual:number(f32∈[0.6,1.4]), GenomeHueVisual:number(u16∈[0,65535]),
//                GenomeColorHex:number(0xAARRGGBB), FinColorVisual:number|null(FISH-only) }
//    Stats:    { TickCount, Season, HerbCount, PredCount, ScavCount, OmnCount,
//                TotalAgents, PlantTotal, PlantBy:number[3],
//                Births, Deaths, Kills, Starved, Aged }
//
//    NOT exported (never rely on them here): Carcasses, parent-id, dead-reason,
//    trail history, evolution time-series, isolation history, deposit summary.
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// 3. Instanced-mesh update (Phase 4) — interface stubs, no rendering here.
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Update agent instanced meshes from a Snapshot's Agents array.
 *
 * TODO(Phase 4): wire into a real THREE.InstancedMesh instance.
 *
 * Pseudocode:
 *   const matrix = new THREE.Matrix4();
 *   const color  = new THREE.Color();
 *   agents.forEach((a, i) => {
 *     matrix.setPosition(a.X, terrainHeight(a.X, a.Z), a.Z);
 *     instancedMesh.setMatrixAt(i, matrix);
 *     // hue from genome, energy→saturation, hunger→alpha
 *     color.setHSL(a.GenomeHue, 1 - a.Hunger, 0.5);
 *     instancedMesh.setColorAt(i, color);
 *   });
 *   instancedMesh.instanceMatrix.needsUpdate = true;
 *   instancedMesh.instanceColor.needsUpdate = true;
 *
 * Per-agent fields used by the renderer:
 *   X / Z          — world position (terrain Y sampled locally)
 *   Kind           — selects the instanced mesh bucket (herb/pred/scav/omn)
 *   GenomeHue      — base color hue (0..1)
 *   GenomeSize     — scale factor for the instance matrix
 *   GenomeSpeed    — could tint "fast" individuals brighter (TODO)
 *   Energy / Hunger— health bar fill / color desaturation
 *   Age            — fade-out near GenomeLifespan
 *   State          — state string may map to a shader variant (TODO)
 *   Cargo          — FRUITY seed payload; may render as floating dots (TODO)
 *
 * @param {Array<object>} agents — Snapshot.Agents slice
 * @param {object} [opts] — placeholder for mesh handles; filled in Phase 4
 */
export function updateAgents(agents, opts = {}) {
  // TODO(Phase 4): replace this stub with real InstancedMesh updates.
  return { count: agents.length, updated: false };
}

/**
 * Grass-grid visual update contract.
 *
 * Expected input shapes (all from Snapshot.World):
 *   grass      — GridN² flat array, per-cell total biomass (f32)
 *   grassBySpecies — 3 × GridN² flat arrays, per-species biomass
 *   suit / moist / shade — GridN² flat arrays (0..1 scalars)
 *
 * updateGrassVisual(grass, { grassBySpecies, suit, moist, shade, gridN })
 *   — reads the full grass slice each frame; caller may pass a dirty-region
 *     subset in a future revision (schema change would bump Meta.Version).
 *
 * TODO(Phase 4): wire to a THREE.DataTexture / custom shader; the flat-array
 * shape above is already renderer-agnostic.
 *
 * @param {number[]} grass
 * @param {{ grassBySpecies?: number[][], suit?: number[], moist?: number[], shade?: number[], gridN?: number }} [opts]
 * @returns {{ cells: number, updated: boolean }}
 */
export function updateGrassVisual(grass, opts = {}) {
  // TODO(Phase 4): replace this stub with real texture/shader updates.
  return { cells: grass.length, updated: false };
}
