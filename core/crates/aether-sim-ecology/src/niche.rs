//! niche.rs — Phase 16: clade niche axis + competitive exclusion.
//!
//! All logic here is pure bookkeeping over the Phase 15 clade table: no RNG
//! draws, no HashMap iteration (plain Vec scans in clade-table order), so it
//! cannot perturb the seeded RNG streams or the checksum trajectory.
//! `impl Ecology` lives in this sibling module to keep the Phase 15
//! speciation engine (`ecology.rs`) untouched.
//!
//! Niche model: each clade carries a 1-D `niche_offset` in [-1, 1]. A living
//! daughter clade re-derives its position every detection pass from its
//! parent's position plus the current centroid deco_efficiency gap
//! (NICHE_EFF_SCALE maps the legal eff band [0.5, 1.5] onto [-1, 1]); the
//! root clade is pinned at 0. The value is therefore a pure function of
//! saved clade state + live genomes — no RNG, no extra drift history — so a
//! restored sim re-derives bit-identical niches (save/restore contract).
//!
//! Phase 51 adds the second half of the axis' meaning: `niche_offset` is also
//! a resource-preference coordinate (see `niche_pref_mult`). The multiplier
//! is keyed off a pure cache of the clade table (`niche_by_clade`), so the
//! per-agent-per-tick cost is one bounds-checked Vec read plus two field
//! reads — O(1), no new scan and no RNG.

use crate::ecology::{
    COMPETITION_BURDEN, COMPETITION_BURDEN_MAX, DECO, Ecology, NICHE_EFF_SCALE,
    NICHE_OVERLAP_THRESHOLD, NICHE_PARTITION_BAND, NICHE_PREF_GAIN, SpeciesKind,
};
use crate::save_state;

/// Phase 51: niche resource-partition telemetry — the share of living members
/// whose clade sits below / inside / above the partition band. Rebuilt from
/// the cached member lists on every detection pass.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NichePartitionShare {
    /// Members in clades with `niche_offset < -NICHE_PARTITION_BAND`
    /// (mineral-preferring side of the axis).
    pub mineral: f64,
    /// Members in clades inside the neutral band (Phase 50 behaviour).
    pub neutral: f64,
    /// Members in clades with `niche_offset > +NICHE_PARTITION_BAND`
    /// (grass-preferring side of the axis).
    pub grass: f64,
    /// `mineral + grass` — the share of members that actually carry a
    /// resource preference.
    pub partitioned: f64,
}

impl Ecology {
    /// Niche position of `clade_id` (0.0 for unknown / unassigned agents —
    /// the root and legacy behaviour).
    #[inline]
    pub(crate) fn clade_niche_of(&self, clade_id: u32) -> f64 {
        self.species_clades.iter()
            .find(|c| c.id == clade_id)
            .map_or(0.0, |c| c.niche_offset)
    }

    /// Phase 51: O(1) niche lookup for the foraging hot path — reads the
    /// cache rebuilt by `rebuild_niche_cache` (identical values to
    /// `clade_niche_of`, which is a scan). 0.0 for unassigned/unknown ids.
    #[inline]
    pub(crate) fn clade_niche_cached(&self, clade_id: u32) -> f64 {
        self.niche_by_clade.get(clade_id as usize).copied().unwrap_or(0.0)
    }

    /// Phase 51: rebuild the clade-id → niche cache (index = clade id).
    /// Called at the end of every detection pass and after a restore; a pure
    /// mirror of `species_clades`, so it can never diverge for long.
    pub(crate) fn rebuild_niche_cache(&mut self) {
        let n = self.next_clade_id as usize;
        self.niche_by_clade.clear();
        self.niche_by_clade.resize(n, 0.0);
        for c in self.species_clades.iter() {
            if let Some(slot) = self.niche_by_clade.get_mut(c.id as usize) {
                *slot = c.niche_offset;
            }
        }
    }

    /// Phase 51: foraging-efficiency multiplier for a clade member eating in
    /// cell `ci`. Public so the A/B acceptance tests can pin the semantics
    /// directly (same contract as `Ecology::fork_budget_grant`).
    ///
    /// The cell's LOCAL resource gradient — `(grass − mineral) / (grass +
    /// mineral)` — decides which side of the niche axis the cell favours; the
    /// clade's side of the band decides whether that is a bonus or a penalty.
    /// Grass-preferring clades (`niche > +band`) gain up to +NICHE_PREF_GAIN
    /// in grass-dominated cells and lose as much in mineral-dominated ones;
    /// mineral-preferring clades (`niche < −band`) get the mirror image. The
    /// strength fades in continuously from the band edge to the axis end, so
    /// there is no hard on/off threshold (no absorbing state).
    ///
    /// Pure function of (niche_offset, cell grass, cell mineral): O(1), no
    /// RNG draw, no state write. Exactly 1.0 whenever the mechanism is off,
    /// the clade is unassigned/root, or the clade sits inside the band.
    ///
    /// SELECTIVE-mode gate (same reasoning as the Phase 16 competition
    /// burden): the niche axis is derived from the deco_efficiency gap, so a
    /// fitness-relevant preference on it would smuggle efficiency-driven
    /// selection into the legacy neutral control. In legacy mode the axis
    /// stays pure bookkeeping.
    #[inline]
    pub fn niche_pref_mult(&self, clade_id: u32, ci: usize) -> f64 {
        if !self.niche_partition { return 1.0; }
        if self.deco_eff_choice_mode != save_state::DECO_CHOICE_SELECTIVE { return 1.0; }
        let niche = self.clade_niche_cached(clade_id);
        if niche.abs() <= NICHE_PARTITION_BAND { return 1.0; }
        if ci >= self.grass.len() { return 1.0; }
        let g = self.grass[ci] as f64;
        let m = self.mineral_field[ci] as f64;
        let rel = (g - m) / (g + m + 1e-9);
        let strength = ((niche.abs() - NICHE_PARTITION_BAND) / (1.0 - NICHE_PARTITION_BAND)).min(1.0);
        let aligned = if niche > 0.0 { rel } else { -rel };
        1.0 + NICHE_PREF_GAIN * strength * aligned
    }

    /// Phase 51: `niche_pref_mult` at a world position (off-grid → 1.0).
    #[inline]
    pub fn niche_pref_at(&self, clade_id: u32, x: f64, z: f64) -> f64 {
        if !self.niche_partition { return 1.0; }
        let ci = self.cell_index(x, z);
        if ci < 0 { return 1.0; }
        self.niche_pref_mult(clade_id, ci as usize)
    }

    /// Current competitive metabolic burden of `clade_id` (0.0 when none).
    #[inline]
    pub(crate) fn clade_burden_of(&self, clade_id: u32) -> f64 {
        self.clade_burdens.iter()
            .find(|(id, _)| *id == clade_id)
            .map_or(0.0, |(_, b)| *b)
    }

    /// Runs right after `refresh_clade_stats` in every detection pass.
    ///
    /// 1. Niche re-derivation: every LIVING daughter clade recomputes
    ///    `niche = parent.niche + (centroid_eff − parent_centroid_eff) × NICHE_EFF_SCALE`,
    ///    clamped to [-1, 1]. Fossils keep their last frozen position; the
    ///    root is pinned at 0. Because the formula only reads saved clade
    ///    state and live centroids, a restored sim re-derives identical
    ///    values (bit-exact save/restore).
    /// 2. Competitive exclusion (Gause): every pair of living same-kind
    ///    clades whose niche distance is < NICHE_OVERLAP_THRESHOLD puts a
    ///    COMPETITION_BURDEN metabolic multiplier on the lower-fitness side
    ///    (lower centroid deco_efficiency; ties → smaller population; further
    ///    ties → the younger clade, higher id). Overlaps stack, capped at
    ///    COMPETITION_BURDEN_MAX. In legacy mode deco_efficiency is
    ///    fitness-neutral by contract, so no burdens are computed at all —
    ///    burdening the lower-efficiency clade there would smuggle selection
    ///    into the neutral control (`legacy_mode_shows_no_selection`).
    #[inline(never)]
    pub(crate) fn update_niches_and_competition(&mut self) {
        // --- (e1) niche re-derivation (living clades only) ---
        // Snapshot (id, niche, eff) first: parent lookups must read the
        // pre-update table, and clade-table order keeps everything
        // deterministic.
        let snapshot: Vec<(u32, f64, f64)> = self.species_clades.iter()
            .map(|c| (c.id, c.niche_offset, c.centroid_genome.deco_efficiency))
            .collect();
        let mut updates: Vec<(u32, f64)> = Vec::new();
        for c in self.species_clades.iter() {
            if c.death_tick.is_some() { continue; } // fossils freeze
            let Some(pid) = c.parent_id else { continue }; // root pinned at 0
            let Some(&(_, pn, pe)) = snapshot.iter().find(|(id, _, _)| *id == pid) else { continue };
            let niche = (pn + (c.centroid_genome.deco_efficiency - pe) * NICHE_EFF_SCALE)
                .clamp(-1.0, 1.0);
            updates.push((c.id, niche));
        }
        for (id, niche) in updates {
            if let Some(c) = self.species_clades.iter_mut().find(|c| c.id == id) {
                c.niche_offset = niche;
            }
        }

        // --- (e2) competitive exclusion ---
        // Rebuilt from scratch each pass: stale entries (extinct clades,
        // separated pairs) disappear automatically. Deterministic double loop
        // in clade-table (creation) order; n ≤ CLADE_CAP = 64 → O(n²) ≈ 2k
        // comparisons per pass, negligible.
        let mut burdens: Vec<(u32, f64)> = Vec::new();
        if self.deco_eff_choice_mode == save_state::DECO_CHOICE_SELECTIVE {
            // Phase 34: the competitive-exclusion window is the DECO FAMILY
            // (the DECO root plus any derived DECO forks), not just the exact
            // `"deco"` name — forked DECO clades carry a derived kind name
            // but still live on the deco_efficiency niche axis.
            let living: Vec<(u32, f64, f64, usize)> = self.species_clades.iter()
                .filter(|c| {
                    c.death_tick.is_none() && {
                        if c.kind_id == u32::MAX {
                            c.kind == self.kind_name(DECO)
                        } else {
                            self.root_kind(c.kind_id as SpeciesKind) == DECO
                        }
                    }
                })
                .map(|c| (c.id, c.niche_offset, c.centroid_genome.deco_efficiency, c.members.len()))
                .collect();
            for i in 0..living.len() {
                for j in (i + 1)..living.len() {
                    let (ida, na, ea, ma) = living[i];
                    let (idb, nb, eb, mb) = living[j];
                    if (na - nb).abs() >= NICHE_OVERLAP_THRESHOLD { continue; }
                    // Loser: lower centroid efficiency; tie → smaller
                    // population; further tie → the younger clade yields.
                    let loser = if (ea - eb).abs() < 1e-12 {
                        if ma != mb { if ma < mb { ida } else { idb } }
                        else if ida > idb { ida } else { idb }
                    } else if ea < eb { ida } else { idb };
                    match burdens.iter_mut().find(|(id, _)| *id == loser) {
                        Some((_, b)) => { *b = (*b + COMPETITION_BURDEN).min(COMPETITION_BURDEN_MAX); }
                        None => burdens.push((loser, COMPETITION_BURDEN)),
                    }
                }
            }
        }
        self.clade_burdens = burdens;

        // --- (e3) Phase 51: niche cache + partition telemetry ---
        // The cache mirrors the final niche table for the O(1) foraging
        // lookup; the census reports how much of the living population sits
        // in each band (mineral-preferring / neutral / grass-preferring).
        // Both are pure bookkeeping: no RNG, no trajectory effect.
        self.rebuild_niche_cache();
        let (mut mineral, mut neutral, mut grass) = (0usize, 0usize, 0usize);
        for c in self.species_clades.iter() {
            if c.death_tick.is_some() { continue; }
            let n = c.members.len();
            if c.niche_offset < -NICHE_PARTITION_BAND {
                mineral += n;
            } else if c.niche_offset > NICHE_PARTITION_BAND {
                grass += n;
            } else {
                neutral += n;
            }
        }
        let total = (mineral + neutral + grass) as f64;
        self.niche_partition_share = if total > 0.0 {
            NichePartitionShare {
                mineral: mineral as f64 / total,
                neutral: neutral as f64 / total,
                grass: grass as f64 / total,
                partitioned: (mineral + grass) as f64 / total,
            }
        } else {
            NichePartitionShare::default()
        };
    }
}
