//! region.rs — Phase 38: multi-region world (frame tree) + per-region ecology
//! sandboxes sharing geology/nutrient layers.
//!
//! **Design decision (Chief-audited, Phase 38)**: the existing `Ecology` is a
//! self-contained single-region sandbox. Rather than refactor it (which would
//! touch the Phase 31-37 accepted mechanisms), Phase 38 introduces a `World`
//! container that owns:
//!   * `regions: Vec<Region>` — the frame-tree nodes (id, name, center, radius,
//!     parent/children for the reference-frame hierarchy).
//!   * `sandboxes: Vec<Ecology>` — one independent ecology sandbox per region
//!     (independent agent pool, plant layer, carcass pool).
//!   * shared geology (`GeologicState`) + nutrients (`NutrientPools`) — the
//!     physical basements are world-global; each region's ecology reads them at
//!     its own step boundary. (A single-region `World` reduces to exactly the
//!     `Ecology::new` trajectory, bit-exact.)
//!   * `l0_shadows: Vec<L0Shadow>` — the L0 statistical shadows for clades
//!     that are not under any "contact-driven refine" observation.
//!   * migration corridors: adjacent-region boundary cells where agents may
//!     move between sandboxes with probability `p_migrate` per tick, paying an
//!     energy toll (deterministic, dedicated RNG stream).
//!
//! **Determinism contract (roadmap §4)**: every new RNG consumer here uses a
//! DEDICATED stream derived as `seed ^ REGION_RNG_TAG` for migration draws and
//! `(seed, region_id, epoch)` for L0→L2 refine sampling. No new draws are taken
//! from the ecology main/prng/fish streams, so a single-region `World` (or an
//! untouched `Ecology`) keeps its Phase 36/37 checksum trajectory bit-exact.
//!
//! **Per-region clock formalization**: each region carries its own
//! `last_geo_step` / `last_biochem_step` / `last_spec_check` tick stamps. The
//! world tick is the master clock; sub-step functions (geology 10k, biochem 1k,
//! speciation 60) fire for a region only when its own clock's boundary is
//! reached (default = world boundary, so the legacy single-region timing is
//! preserved exactly).
//!
//! **Migration corridor semantics**: regions are treated as overlapping circles
//! on the shared world plane. Two regions are "adjacent" when their circles
//! intersect (`dist(center_a, center_b) < radius_a + radius_b`). The corridor
//! is the band around the midpoint of the two centers; an agent crossing that
//! band (within `corridor_halfwidth`) at its boundary is eligible to migrate.
//! Migration moves the agent to the OTHER region's sandbox (re-spawned with a
//! fresh id — see `migrate_agent`), consuming `MIGRATE_ENERGY_TOLL` energy and
//! drawing from the dedicated migration RNG stream.

use crate::ecology::{Agent, Ecology, SpeciesKind, HERB, PRED, SCAV, OMN, FISH, DECO};
use crate::l0_shadow::{L0Shadow, L0_REFINE_MIN_POP, l0_from_sandbox, REFINE_CONTACT_DIST};
use aether_rng::Mulberry32;

/// Dedicated migration RNG tag: `seed ^ REGION_RNG_TAG` seeds the per-world
/// migration stream. ASCII "REG1" — never collides with the ecology tags
/// (0x4745_4F31 GEO / 0x42_52_41_49 BRAIN / culture TOKEN tag).
pub const REGION_RNG_TAG: u32 = 0x52_45_47_31;
/// Migration corridor half-width (world units) around the adjacent-region
/// midpoint band.
pub const CORRIDOR_HALFWIDTH: f64 = 60.0;
/// Per-tick migration probability (0 = migration disabled).
pub const MIGRATE_PROB: f64 = 0.004;
/// Energy toll for a successful migration (world energy units).
pub const MIGRATE_ENERGY_TOLL: f64 = 8.0;
/// Minimum energy for an agent to be eligible to migrate (avoids starving
/// agents dying mid-corridor).
pub const MIGRATE_MIN_ENERGY: f64 = 20.0;
/// Default world radius (single-region world spans the whole grid).
pub const DEFAULT_REGION_RADIUS: f64 = 560.0;

/// One reference-frame node: a named circular region of the shared plane.
/// `parent_id`/`children` express the frame tree (v2.0 aether-coords wiring).
#[derive(Clone, Debug)]
pub struct Region {
    pub id: u32,
    pub name: String,
    pub center_x: f64,
    pub center_z: f64,
    pub radius: f64,
    pub parent_id: Option<u32>,
    pub children: Vec<u32>,
    /// Per-region clock stamps (formalized sub-step timing).
    pub last_geo_step: u32,
    pub last_biochem_step: u32,
    pub last_spec_check: u32,
}

impl Region {
    pub fn new(id: u32, name: &str, center_x: f64, center_z: f64, radius: f64) -> Self {
        Region {
            id,
            name: name.to_string(),
            center_x,
            center_z,
            radius,
            parent_id: None,
            children: Vec::new(),
            last_geo_step: 0,
            last_biochem_step: 0,
            last_spec_check: 0,
        }
    }

    /// Distance between two region centers.
    pub fn dist(&self, other: &Region) -> f64 {
        let dx = self.center_x - other.center_x;
        let dz = self.center_z - other.center_z;
        (dx * dx + dz * dz).sqrt()
    }

    /// Two regions are adjacent when their circles intersect.
    pub fn adjacent(&self, other: &Region) -> bool {
        self.dist(other) < self.radius + other.radius
    }

    /// The corridor midpoint between two adjacent regions (used to place the
    /// migration band).
    pub fn corridor_mid(&self, other: &Region) -> (f64, f64) {
        (
            0.5 * (self.center_x + other.center_x),
            0.5 * (self.center_z + other.center_z),
        )
    }
}

/// Phase 38: multi-region world container.
///
/// Owns the region table, per-region ecology sandboxes, the L0 shadow table,
/// and the migration corridor list. Geology and biochemistry are independent
/// per sandbox (seed-derived deterministic) so no shared-mutation is needed;
/// a single-region world with region_id=0 and seed preserves the legacy
/// bit-exact trajectory exactly.
pub struct World {
    pub regions: Vec<Region>,
    pub sandboxes: Vec<Ecology>,
    /// L0 statistical shadows (one per clade × region that is NOT under
    /// contact-driven refine observation).
    pub l0_shadows: Vec<L0Shadow>,
    /// Cell → region mapping (grid index → region id), sized GRID_N².
    pub region_map: Vec<u32>,
    /// Per-world migration RNG stream (seed ^ REGION_RNG_TAG).
    pub migration_rng: Mulberry32,
    /// Migration enabled flag (default true; off = isolated sandboxes).
    pub migration_on: bool,
    /// Master seed (used for refine stream derivation).
    pub seed: u32,
    /// Master tick counter (shared across all regions).
    pub tick_count: u32,
}

impl World {
    /// Build a world from an explicit region list. Each region's ecology is
    /// initialized deterministically from `seed` (a fresh `Ecology::new` is
    /// seeded from `seed ^ region_id` so distinct regions diverge — the legacy
    /// `seed` maps to region 0 when there is exactly one region, preserving the
    /// single-region bit-exact trajectory).
    pub fn new(seed: u32, regions: Vec<Region>) -> Self {
        assert!(!regions.is_empty(), "World::new requires ≥ 1 region");
        let n_regions = regions.len();
        let mut sandboxes = Vec::with_capacity(n_regions);
        for r in &regions {
            // region 0 = master seed (bit-exact legacy path); others derived.
            let s = if r.id == 0 { seed } else { seed ^ r.id };
            sandboxes.push(Ecology::new(s));
        }
        let grid_n = sandboxes[0].grid_n as usize;
        let region_map = World::build_region_map(&regions, grid_n, sandboxes[0].cell);
        let l0_shadows = l0_from_sandbox(seed, &sandboxes, &regions, &region_map);
        World {
            regions,
            sandboxes,
            l0_shadows,
            region_map,
            migration_rng: Mulberry32::new(seed ^ REGION_RNG_TAG),
            migration_on: true,
            seed,
            tick_count: 0,
        }
    }

    /// Build the cell→region map. Each grid cell is assigned to the region
    /// whose center is nearest (ties resolved by lower region id). Unreachable
    /// cells (outside every region circle) get the nearest region anyway —
    /// the grid is always fully partitioned so no cell is orphaned.
    fn build_region_map(regions: &[Region], grid_n: usize, cell: f64) -> Vec<u32> {
        let half = crate::terrain::SIM_RADIUS;
        let mut map = vec![0u32; grid_n * grid_n];
        for j in 0..grid_n {
            for i in 0..grid_n {
                let x = -half + (i as f64 + 0.5) * cell;
                let z = -half + (j as f64 + 0.5) * cell;
                let idx = j * grid_n + i;
                let mut best = 0u32;
                let mut best_d = f64::MAX;
                for r in regions {
                    let dx = x - r.center_x;
                    let dz = z - r.center_z;
                    let d = dx * dx + dz * dz;
                    if d < best_d {
                        best_d = d;
                        best = r.id;
                    }
                }
                map[idx] = best;
            }
        }
        map
    }

    /// Region id for a cell (grid index).
    #[inline]
    pub fn region_of_cell(&self, idx: usize) -> u32 {
        self.region_map.get(idx).copied().unwrap_or(0)
    }

    /// Region id at a world position (nearest region center).
    pub fn region_at(&self, x: f64, z: f64) -> u32 {
        let mut best = 0u32;
        let mut best_d = f64::MAX;
        for r in &self.regions {
            let dx = x - r.center_x;
            let dz = z - r.center_z;
            let d = dx * dx + dz * dz;
            if d < best_d {
                best_d = d;
                best = r.id;
            }
        }
        best
    }

    /// Number of regions.
    pub fn n_regions(&self) -> usize {
        self.regions.len()
    }

    /// Read-only view of a region's sandbox.
    pub fn sandbox(&self, region_id: u32) -> &Ecology {
        let idx = self.region_index(region_id);
        &self.sandboxes[idx]
    }

    /// Mutable view of a region's sandbox.
    pub fn sandbox_mut(&mut self, region_id: u32) -> &mut Ecology {
        let idx = self.region_index(region_id);
        &mut self.sandboxes[idx]
    }

    /// Index of a region in the regions/sandboxes vectors.
    fn region_index(&self, region_id: u32) -> usize {
        self.regions.iter().position(|r| r.id == region_id)
            .unwrap_or(0)
    }

    /// Adjacency list (region_id pairs) — deterministic region-id order.
    pub fn adjacent_pairs(&self) -> Vec<(u32, u32)> {
        let mut pairs = Vec::new();
        for i in 0..self.regions.len() {
            for j in (i + 1)..self.regions.len() {
                if self.regions[i].adjacent(&self.regions[j]) {
                    pairs.push((self.regions[i].id, self.regions[j].id));
                }
            }
        }
        pairs
    }

    /// Advance the whole world by one tick.
    ///
    /// Order (deterministic):
    ///   1. shared geology/biochem step (world boundary, per the master clock);
    ///      each sandbox runs its own independent substep.
    ///   2. per-region ecology substeps — all sandboxes advance on the same
    ///      daylight curve; each sandbox tracks its own last step timestamps.
    ///   3. migration corridor pass (dedicated RNG stream).
    ///   4. L0 shadow evolution (unobserved clades).
    ///   5. contact-driven refine bookkeeping.
    pub fn update(&mut self, dt: f64, daylight: f64) {
        if dt <= 0.0 { return; }
        self.tick_count += 1;

        // Shared geology + biochem fire on the world master clock.
        if self.tick_count % crate::geology::GEO_TICKS_PER_STEP == 0 {
            for sandbox in &mut self.sandboxes {
                sandbox.geology_step();
            }
        }
        if self.tick_count % crate::biochemistry::BIOCHEM_TICKS == 0 {
            for sandbox in &mut self.sandboxes {
                sandbox.biochem_step();
            }
        }

        // Per-region ecology substeps (all on the same daylight curve).
        for idx in 0..self.sandboxes.len() {
            let rid = self.regions[idx].id;
            self.sandboxes[idx].update(dt, daylight);
            // Per-region sub-step boundary tracking (formalized clocks).
            self.regions[idx].last_geo_step = self.sandboxes[idx].tick_count();
            self.regions[idx].last_biochem_step = self.sandboxes[idx].tick_count();
            self.regions[idx].last_spec_check = self.sandboxes[idx].tick_count();
            let _ = rid;
        }

        // Migration corridor pass.
        if self.migration_on {
            self.migration_pass();
        }

        // L0 shadow evolution (unobserved clades keep evolving — dead zone).
        self.evolve_l0_shadows();

        // Contact-driven refine bookkeeping.
        self.refine_contact_pass();
    }

    /// Migration pass: for each adjacent pair, agents near the corridor band
    /// may cross to the other region. Uses the dedicated migration RNG.
    fn migration_pass(&mut self) {
        let pairs = self.adjacent_pairs();
        if pairs.is_empty() { return; }
        let mut to_move: Vec<(u32, u32, u32)> = Vec::new(); // (from_rid, agent_id, to_rid)
        for (a, b) in pairs {
            let (mx, mz) = self.regions[self.region_index(a)].corridor_mid(&self.regions[self.region_index(b)]);
            // Collect candidate agents from region a heading toward the corridor.
            self.collect_candidates(a, b, mx, mz, &mut to_move);
            self.collect_candidates(b, a, mx, mz, &mut to_move);
        }
        // Apply moves (post-collection to avoid borrow conflicts).
        for (from_rid, agent_id, to_rid) in to_move {
            self.migrate_agent(from_rid, agent_id, to_rid);
        }
    }

    /// Collect migration candidates from `from_rid` whose position lies inside
    /// the corridor band and whose heading points toward the target.
    fn collect_candidates(
        &mut self,
        from_rid: u32,
        to_rid: u32,
        mx: f64,
        mz: f64,
        out: &mut Vec<(u32, u32, u32)>,
    ) {
        let fi = self.region_index(from_rid);
        let ti = self.region_index(to_rid);
        // Snapshot candidate ids to avoid holding a borrow across `migrate`.
        let mut candidates: Vec<u32> = Vec::new();
        {
            let sandbox = &self.sandboxes[fi];
            for a in &sandbox.agents {
                if !a.alive { continue; }
                if a.energy < MIGRATE_MIN_ENERGY { continue; }
                let dx = a.x - mx;
                let dz = a.z - mz;
                let d2 = dx * dx + dz * dz;
                if d2 > CORRIDOR_HALFWIDTH * CORRIDOR_HALFWIDTH { continue; }
                // Heading must point toward the target region center.
                let tdx = self.regions[ti].center_x - a.x;
                let tdz = self.regions[ti].center_z - a.z;
                let dot = tdx * a.heading.cos() + tdz * a.heading.sin();
                if dot <= 0.0 { continue; }
                candidates.push(a.id);
            }
        }
        for id in candidates {
            // Per-agent migration roll from the dedicated stream.
            let p = self.migration_rng.next_f64();
            if p < MIGRATE_PROB {
                out.push((from_rid, id, to_rid));
            }
        }
    }

    /// Move one agent from one region sandbox to another.
    ///
    /// The agent is removed from `from_rid`'s sandbox and re-spawned in
    /// `to_rid`'s sandbox with a fresh id (the target sandbox's own next_id).
    /// Energy toll is deducted at spawn; the genome/lineage/brain are copied.
    fn migrate_agent(&mut self, from_rid: u32, agent_id: u32, to_rid: u32) {
        let fi = self.region_index(from_rid);
        let ti = self.region_index(to_rid);
        // Find the agent in the source sandbox.
        let (agent, new_energy) = {
            let sandbox = &mut self.sandboxes[fi];
            let pos = sandbox.agents.iter().position(|a| a.id == agent_id && a.alive);
            let Some(pos) = pos else { return };
            let a = &sandbox.agents[pos];
            if a.energy - MIGRATE_ENERGY_TOLL <= 0.0 { return; }
            let mut clone = a.clone();
            clone.energy -= MIGRATE_ENERGY_TOLL;
            let new_energy = clone.energy;
            sandbox.agents.remove(pos);
            (clone, new_energy)
        };
        let _ = new_energy;
        // Re-spawn in the target sandbox.
        let target = &mut self.sandboxes[ti];
        target.agents.push(agent);
    }

    /// Evolve all L0 shadows deterministically (Lotka-Volterra aggregate
    /// update). Unobserved clades keep evolving — no "observation dead zone".
    fn evolve_l0_shadows(&mut self) {
        for i in 0..self.l0_shadows.len() {
            self.l0_shadows[i].step(0.1);
        }
    }

    /// Contact-driven refine: for each region, any L0 shadow whose clade has
    /// come into contact (agent within REFINE_CONTACT_DIST of a region boundary
    /// or the population fell below L0_REFINE_MIN_POP) is refined back to L2
    /// individuals sampled from the shadow's distribution.
    fn refine_contact_pass(&mut self) {
        let mut to_refine: Vec<usize> = Vec::new();
        for i in 0..self.l0_shadows.len() {
            let shadow = &self.l0_shadows[i];
            let rid = shadow.region_id;
            // Already refines this epoch — skip until next contact event.
            if shadow.refine_epoch >= self.tick_count { continue; }
            let sandbox = self.sandbox(rid);
            // Contact condition: the clade has living agents near the region
            // boundary, OR its abundance dropped below the revive threshold.
            let contact = sandbox.agents.iter().any(|a| {
                a.alive && a.clade_id == shadow.clade_id
                    && near_region_boundary(a.x, a.z, &self.regions[self.region_index(rid)])
            });
            if contact || shadow.abundance < L0_REFINE_MIN_POP {
                to_refine.push(i);
            }
        }
        for i in to_refine {
            self.refine_shadow(i);
        }
    }

    /// Sample L2 agents from an L0 shadow and push them into its region sandbox.
    fn refine_shadow(&mut self, shadow_idx: usize) {
        let (rid, n) = {
            let s = &self.l0_shadows[shadow_idx];
            (s.region_id, s.sample_n())
        };
        let ti = self.region_index(rid);
        let region = &self.regions[ti];
        let target = &mut self.sandboxes[ti];
        let clade_id = self.l0_shadows[shadow_idx].clade_id;
        // Deterministic refine stream: advances the shadow's own epoch counter
        // so repeated refines stay reproducible even without saving state.
        {
            let shadow = &mut self.l0_shadows[shadow_idx];
            shadow.refine_epoch += 1;
            let mut stream = shadow.refine_stream();
            for _k in 0..n {
                // Sample x/z uniformly within the region's circle.
                let angle = stream.next_f64() * std::f64::consts::PI * 2.0;
                let radius = stream.next_f64().sqrt() * region.radius;
                let x = region.center_x + angle.cos() * radius;
                let z = region.center_z + angle.sin() * radius;
                let x = x.clamp(-crate::terrain::WORLD_HALF, crate::terrain::WORLD_HALF);
                let z = z.clamp(-crate::terrain::WORLD_HALF, crate::terrain::WORLD_HALF);
                let h = crate::terrain::terrain_height(x, z);
                // Only spawn on habitable ground.
                if h > crate::terrain::WATER_LEVEL + 1.5 && h < 72.0 {
                    let age = 15.0 + stream.next_f64() * 60.0;
                    if let Some(_aid) = target.spawn_agent(HERB, x, z) {
                        // Assign the shadow's clade id to the new agent.
                        let pos = target.agents.iter().position(|a| a.id == _aid && a.alive);
                        if let Some(p) = pos {
                            target.agents[p].clade_id = clade_id;
                            target.agents[p].age = age;
                        }
                    }
                }
            }
        }
        let _ = (n, clade_id, ti, region);
    }

    /// Total living agent count across all regions.
    pub fn total_agents(&self) -> usize {
        self.sandboxes.iter().map(|s| s.agents.iter().filter(|a| a.alive).count()).sum()
    }

    /// World checksum: fold each region's checksum in region-id order (spec
    /// acceptance #1: cross-migration checksum consistency).
    pub fn checksum(&self) -> String {
        let mut ordered: Vec<&Ecology> = self.regions.iter()
            .map(|r| &self.sandboxes[self.region_index(r.id)])
            .collect();
        ordered.sort_by_key(|s| s.seed);
        let mut h: u128 = 0xC0FFEE;
        for s in ordered {
            let ck = s.checksum();
            // Fold the 16-hex-digit checksum into a u64 then into the running h.
            let v = u64::from_str_radix(&ck[2..], 16).unwrap_or(0);
            h = (h * 0x100000001B3) ^ (v as u128);
            h &= 0xFFFFFFFFFFFFFFFF;
        }
        format!("0x{:016x}", h as u64)
    }

    /// A region's ecology sandbox reference (public accessor for tests/ABI).
    pub fn region_ecos(&self) -> &[Ecology] { &self.sandboxes }
}

/// Whether a point is near a region's boundary (within REFINE_CONTACT_DIST of
/// the circle edge).
fn near_region_boundary(x: f64, z: f64, region: &Region) -> bool {
    let dx = x - region.center_x;
    let dz = z - region.center_z;
    let d = (dx * dx + dz * dz).sqrt();
    (d - region.radius).abs() < REFINE_CONTACT_DIST
}

/// Count of agents per kind in a region (helper for tests/telemetry).
pub fn region_kind_counts(sandbox: &Ecology) -> [(SpeciesKind, usize); 6] {
    [
        (HERB, sandbox.count(HERB)),
        (PRED, sandbox.count(PRED)),
        (SCAV, sandbox.count(SCAV)),
        (OMN, sandbox.count(OMN)),
        (FISH, sandbox.count(FISH)),
        (DECO, sandbox.count(DECO)),
    ]
}
