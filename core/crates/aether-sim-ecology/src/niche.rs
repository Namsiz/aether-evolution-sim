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

use crate::ecology::{
    COMPETITION_BURDEN, COMPETITION_BURDEN_MAX, DECO, Ecology, NICHE_EFF_SCALE,
    NICHE_OVERLAP_THRESHOLD,
};
use crate::save_state;

impl Ecology {
    /// Niche position of `clade_id` (0.0 for unknown / unassigned agents —
    /// the root and legacy behaviour).
    #[inline]
    pub(crate) fn clade_niche_of(&self, clade_id: u32) -> f64 {
        self.species_clades.iter()
            .find(|c| c.id == clade_id)
            .map_or(0.0, |c| c.niche_offset)
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
            let living: Vec<(u32, f64, f64, usize)> = self.species_clades.iter()
                .filter(|c| c.death_tick.is_none() && c.kind == DECO)
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
    }
}
