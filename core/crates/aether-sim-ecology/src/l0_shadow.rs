//! l0_shadow.rs — Phase 38: L0 statistical shadows (Lotka-Volterra aggregate
//! dynamics for unobserved clades).
//!
//! **Core invariant**: "未观察 / 未接触" does NOT mean "dead zone" — every
//! clade's L0 aggregate continues evolving deterministically each tick even
//! when no live agent in any sandbox carries that clade id. The world stays
//! globally dynamic.
//!
//! **Contact-driven refine**: a clade transitions back to L2 individuals when:
//!   * An agent carrying that clade id comes within `REFINE_CONTACT_DIST` of
//!     a region boundary (cross-region proximity — signal of future migration).
//!   * The L0 aggregate population drops below `L0_REFINE_MIN_POP` — stale
//!     shadows are resurrected before they atrophy.
//!   Upon refine, `sample_n()` L2 agents are spawned inside the shadow's
//!   region using a DEDICATED `(seed, region_id, epoch)` RNG stream so the
//!   sample is bit-exact replayable.
//!
//! **Refine coarsen directionality** (contact-only; no observation-driven):
//!   * Refine: L0 → L2 (populate sandboxes from the shadow distribution).
//!   * Coarsen is NOT implemented here — L2 → L0 happens implicitly at
//!     extinction time (when the last L2 agent dies the remaining L0 stats
//!     become the authoritative record until a contact event re-triggers refine).
//!
//! **Lotka-Volterra base model**: single-species logistic growth per shadow
//! (a simplification that suffices for Phase 38 acceptance tests; full
//! multi-species competition matrix is left for Phase 39+). Each `L0Shadow`
//! carries its own (r, K) parameters, initialized from the first observed
//! sample of the corresponding clade in the sandbox. The deterministic step
//! is Euler integration on dN/dt = r N (1 - N/K).

use crate::ecology::{Ecology, Agent, SpeciesKind};
use crate::region::{Region, World, DEFAULT_REGION_RADIUS, REGION_RNG_TAG};
use aether_rng::Mulberry32;
use std::collections::HashMap;

/// When a clade's L0 shadow population drifts below this value, it is
/// automatically re-refined (prevents permanent shadow collapse to zero).
pub const L0_REFINE_MIN_POP: f64 = 5.0;
/// Distance to a region boundary that counts as "contact" for refine trigger.
pub const REFINE_CONTACT_DIST: f64 = 80.0;
/// Max agents to spawn per refine pass per clade (capped for sanity; the
/// true count is `min(sample_n(), K_l0)` but we bound per-pass to avoid
/// sudden bursts exceeding sandbox caps).
const REFINING_MAX_PER_PASS: usize = 30;

/// One L0 shadow: aggregate (mean/var/LV params) for a single clade inside
/// one region. A clade may have multiple L0 shadows across different regions
/// (each region has its own shadow independently).
#[derive(Clone, Debug)]
pub struct L0Shadow {
    /// Master seed used for derive streams (shared across all shadows in a
    /// world, set at construction).
    pub seed: u32,
    /// Which region this shadow belongs to.
    pub region_id: u32,
    /// Clade id in the sandbox's species_clades table.
    pub clade_id: u32,
    /// Current L0 estimate of mean population (integer-adjacent float for
    /// sampling).
    pub abundance: f64,
    /// Variance in population (used for Gaussian-ish sampling during refine).
    pub variance: f64,
    /// Intrinsic growth rate r (initialized from sandbox clade sample, kept
    /// positive via abs).
    pub r: f64,
    /// Carrying capacity K (initialized from sandbox clade sample, floored
    /// at 10 to avoid degenerate zero-K dynamics).
    pub k: f64,
    /// Dedicated refine stream state: `(seed, region_id, epoch)` fold.
    pub refine_epoch: u32,
    pub refine_stream_state: u32,
    /// How many ticks since last L0 step (bookkeeping for telemetry).
    pub last_step_tick: u32,
}

impl L0Shadow {
    /// Deterministic refine RNG stream for this (region_id, clade_id).
    ///
    /// Formula: fold(seed ^ REGION_RNG_TAG, region_id as u32, epoch, clade_id)
    /// into a Mulberry32 seed. Matches the roadmap §4 RNG discipline:
    /// dedicated stream derived from `(seed, region_id, epoch)`.
    pub fn refine_stream(&mut self) -> Mulberry32 {
        self.refine_epoch += 1;
        let epoch_val = self.refine_epoch;
        let h: u64 = (self.seed as u64)
            .wrapping_mul(0x9E3779B97F4A7C15)
            .wrapping_add(self.region_id as u64)
            .wrapping_add((epoch_val as u64).wrapping_mul(0x6C62272E07BB0142));
        let h = h ^ (h >> 32);
        // Mix in region tag bit-pattern so different worlds don't collide.
        let with_tag = h ^ ((REGION_RNG_TAG as u64) << 16);
        // Fold through clade_id for uniqueness per clade.
        let final_seed = (with_tag as u32) ^ clade_id_fold(self.clade_id);
        self.refine_stream_state = final_seed;
        Mulberry32::new(final_seed)
    }

    /// Sample how many L2 agents to spawn this refine pass (bounded by K and
    /// REFINING_MAX_PER_PASS).
    pub fn sample_n(&self) -> usize {
        let cap = self.k as usize;
        cap.min(REFINING_MAX_PER_PASS).max(1)
    }

    /// Initialize an L0 shadow from the first observation of the clade in a
    /// sandbox. Called once during World::new (no RNG — pure statistics).
    pub fn from_sandbox(clade_id: u32, region_id: u32, sandbox: &Ecology) -> Self {
        let alive: Vec<&Agent> = sandbox.agents.iter()
            .filter(|a| a.alive && a.clade_id == clade_id)
            .collect();
        let n = alive.len();
        let abundance = if n > 0 {
            let avg_speed: f64 = alive.iter().map(|a| a.genome.speed).sum::<f64>() / n as f64;
            let avg_size: f64 = alive.iter().map(|a| a.genome.size).sum::<f64>() / n as f64;
            // Heuristic init: start abundance near current count, K near
            // biome-cap-adjusted carrying capacity scaled by clade size.
            let speed_norm = (avg_speed / 5.0).clamp(0.2, 3.0);
            let size_norm = (avg_size / 1.0).clamp(0.3, 2.5);
            let est_k = (DEFAULT_REGION_RADIUS / 50.0) * speed_norm * size_norm;
            let est_r = 0.05 * speed_norm;
            L0Shadow {
                seed: sandbox.seed,
                region_id,
                clade_id,
                abundance: n as f64,
                variance: (n as f64 * 0.2).max(1.0),
                r: est_r.max(0.01),
                k: est_k.max(10.0),
                refine_epoch: 0,
                refine_stream_state: 0,
                last_step_tick: sandbox.tick_count(),
            }
        } else {
            // No current agents — cold-start with modest numbers.
            L0Shadow {
                seed: sandbox.seed,
                region_id,
                clade_id,
                abundance: 20.0,
                variance: 10.0,
                r: 0.05,
                k: 100.0,
                refine_epoch: 0,
                refine_stream_state: 0,
                last_step_tick: sandbox.tick_count(),
            }
        };
        abundance
    }

    /// Step this shadow's LV dynamics deterministically (Euler, dt = 0.1).
    pub fn step(&mut self, dt: f64) {
        // Logistic growth: dN/dt = r N (1 - N/K)
        let growth = self.r * self.abundance * (1.0 - self.abundance / self.k);
        let noise_scale = (self.variance / self.k.sqrt()).abs().min(0.1) * dt;
        // Zero-RNG step: deterministic only — the "noise" here is bookkeeping
        // variance tracking, not RNG draws (RNG discipline).
        self.abundance += growth * dt;
        if self.abundance < 0.0 { self.abundance = 0.0; }
        // Variance drifts toward K (carrying capacity uncertainty grows as N→K).
        let target_var = self.abundance * (1.0 - self.abundance / self.k) * self.k;
        self.variance += (target_var - self.variance) * 0.01;
        if self.variance < 0.5 { self.variance = 0.5; }
        self.last_step_tick += 1;
    }
}

fn clade_id_fold(id: u32) -> u32 {
    // Simple folding: xor with a prime rotation to mix bits.
    ((id as u64).wrapping_mul(0x9E3779B97F4A7C15) ^ ((id as u64) << 17)) as u32
}

/// Build the initial L0 shadow table from a list of sandboxes + regions + map.
/// Called once during World::new. Each (clade_id, region_id) pair gets exactly
/// one L0Shadow entry. Iteration order is deterministic (region id asc, then
/// clade id asc within each region). Uses zero RNG — pure statistics.
pub fn l0_from_sandbox(
    _seed: u32,
    sandboxes: &[Ecology],
    regions: &[Region],
    _region_map: &[u32],
) -> Vec<L0Shadow> {
    // Collect unique (clade_id, region_id) pairs. Region 0 first (deterministic).
    let mut seen: Vec<(u32, u32)> = Vec::new();
    for ridx in 0..regions.len() {
        let region_id = regions[ridx].id;
        for c in &sandboxes[ridx].species_clades {
            if !seen.iter().any(|&(cid, rid)| cid == c.id && rid == region_id) {
                seen.push((c.id, region_id));
            }
        }
    }
    seen.sort_by_key(|&(cid, rid)| (rid, cid)); // region-id primary, clade-id secondary

    let mut shadows = Vec::with_capacity(seen.len());
    for (clade_id, region_id) in &seen {
        let ridx = regions.iter().position(|r| r.id == *region_id).unwrap_or(0);
        let shadow = L0Shadow::from_sandbox(*clade_id, *region_id, &sandboxes[ridx]);
        shadows.push(shadow);
    }
    shadows
}


