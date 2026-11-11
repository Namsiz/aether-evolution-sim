#!/usr/bin/env python3
"""Phase 16 surgical patcher: adaptive radiation + competitive exclusion.

Applies the Phase 16 diff to four existing files via exact search/replace
blocks. Every block must match exactly `count` times or the script aborts
WITHOUT writing anything (atomic per-file: all edits of a file are applied to
an in-memory string first).

It also self-heals `ecology.rs` if the working copy was lost: when the file
is missing or under 1000 bytes it is restored from git HEAD (read-only
`git show`; requires the Phase 15 commit 1cba059 to be present).

Usage:
    cd ~/workspace/world
    python3 core/patch_phase16.py --check   # dry run: verify all blocks match
    python3 core/patch_phase16.py --apply   # apply
"""
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent  # world/

# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------

def s(text):
    """Dedent-free literal block (keeps the author's exact indentation)."""
    return text


def apply_edits(rel_path, edits, restore_from_git=False):
    path = ROOT / rel_path
    if restore_from_git and (not path.exists() or path.stat().st_size < 1000):
        r = subprocess.run(["git", "show", f"HEAD:{rel_path}"],
                           cwd=ROOT, capture_output=True)
        if r.returncode != 0 or len(r.stdout) < 1000:
            raise SystemExit(
                f"FATAL: cannot restore {rel_path} from git HEAD "
                f"(rc={r.returncode}). Is the Phase 15 commit present? "
                f"{r.stderr.decode(errors='replace')[:300]}")
        path.write_bytes(r.stdout)
        print(f"  restored {rel_path} from git HEAD ({len(r.stdout)} bytes)")
    text = path.read_text(encoding="utf-8")
    for i, (search, replace, count) in enumerate(edits):
        n = text.count(search)
        if n != count:
            raise SystemExit(
                f"FATAL: {rel_path} edit#{i}: expected {count} occurrence(s), "
                f"found {n}. Aborting, file NOT modified.\n"
                f"--- search block ---\n{search}\n--------------------")
        text = text.replace(search, replace)
    if not PATHS_DRY_RUN:
        path.write_text(text, encoding="utf-8")
    print(f"  {'[check] ' if PATHS_DRY_RUN else ''}patched {rel_path}: "
          f"{len(edits)} edits OK")


PATHS_DRY_RUN = False

# ---------------------------------------------------------------------------
# ecology.rs
# ---------------------------------------------------------------------------

ECOLOGY = "core/crates/aether-sim-ecology/src/ecology.rs"

ECOLOGY_EDITS = [
    # 1. Phase 16 constants after the Phase 15 speciation constants.
    (s('''/// A split is only declared when both sides keep at least this many members
/// (a one-agent "species" is sampling noise and dies within a pass).
const SPECIATION_MIN_SIDE: usize = 2;
'''),
     s('''/// A split is only declared when both sides keep at least this many members
/// (a one-agent "species" is sampling noise and dies within a pass).
const SPECIATION_MIN_SIDE: usize = 2;

// ----- Phase 16: niche axis + competitive exclusion ---------------------------
//
// Niche model: each clade carries a 1-D `niche_offset` in [-1, 1]. A living
// daughter clade re-derives its position every detection pass from its
// parent's position plus the current centroid deco_efficiency gap
// (NICHE_EFF_SCALE maps the legal eff band [0.5, 1.5] onto [-1, 1]); the root
// clade is pinned at 0. The value is therefore a pure function of saved
// clade state + live genomes — no RNG draws, no extra saved drift history —
// so a restored sim re-derives bit-identical niches (save/restore contract).
/// eff gap -> niche displacement: (eff_child - eff_parent) * 2 spans [-1, 1]
/// for the legal eff band [0.5, 1.5].
pub const NICHE_EFF_SCALE: f64 = 2.0;
/// Two clades whose niche positions are closer than this compete for the
/// same resources (1-D Gause overlap window).
pub const NICHE_OVERLAP_THRESHOLD: f64 = 0.15;
/// Extra metabolic multiplier applied to the fitness-loser of an overlapping
/// clade pair (+15 % drain). Only computed in SELECTIVE mode — in legacy mode
/// deco_efficiency is fitness-neutral by contract, so burdening the
/// lower-efficiency clade would smuggle selection into the neutral control
/// (`legacy_mode_shows_no_selection`).
pub const COMPETITION_BURDEN: f64 = 0.15;
/// Cap on the summed burden of one clade (multi-overlap stacking).
pub const COMPETITION_BURDEN_MAX: f64 = 0.5;
/// Wander-heading bias in radians: a clade at niche +/-1 roams up to
/// +/-0.9 rad off its unbiased random heading, spatially separating clades.
pub const NICHE_HEADING_BIAS: f64 = 0.9;
/// Fraction of a bite's mineral boost re-deposited into the niche-direction
/// neighbor cell (DECO mineral preference direction).
pub const NICHE_DEPOSIT_BIAS: f64 = 0.25;
/// Event log kinds (CladeSpeciationEvent.event_kind / ABI event-record
/// offset 12, formerly the reserved u32).
pub const EVENT_KIND_SPECIATION: u8 = 0;
pub const EVENT_KIND_EXTINCTION: u8 = 1;
pub const EVENT_KIND_COMPETITIVE_EXCLUSION: u8 = 2;
'''), 1),

    # 2. SpeciesClade gains niche_offset.
    (s('''    pub centroid_genome: Genome,
    pub members: Vec<u32>,
}
'''),
     s('''    pub centroid_genome: Genome,
    pub members: Vec<u32>,
    /// Phase 16: 1-D niche position in [-1, 1]. Living daughters re-derive it
    /// every pass (parent position + centroid eff gap); fossils freeze it.
    pub niche_offset: f64,
}
'''), 1),

    # 3. CladeSpeciationEvent gains event_kind.
    (s('''pub struct CladeSpeciationEvent {
    pub tick: u32,
    pub parent_clade_id: u32,
    pub child_clade_ids: Vec<u32>,
    pub genetic_distance: f64,
}
'''),
     s('''pub struct CladeSpeciationEvent {
    pub tick: u32,
    pub parent_clade_id: u32,
    pub child_clade_ids: Vec<u32>,
    pub genetic_distance: f64,
    /// Phase 16: 0 = speciation, 1 = plain extinction, 2 = competitive
    /// exclusion (the losing clade carried a nonzero burden when its members
    /// emptied). Appended at the tail; JSON key "kind" defaults to 0.
    pub event_kind: u8,
}
'''), 1),

    # 4. Ecology struct gains clade_burdens.
    (s('''    pub speciation_events: Vec<CladeSpeciationEvent>,
    last_clade_spec_tick: u32,
}
'''),
     s('''    pub speciation_events: Vec<CladeSpeciationEvent>,
    last_clade_spec_tick: u32,
    /// Phase 16: per-clade metabolic burden from niche competition, rebuilt
    /// every detection pass (clade id -> multiplier >= 0). Vec of pairs in
    /// first-seen order — deterministic iteration, no HashMap in wasm.
    pub clade_burdens: Vec<(u32, f64)>,
}
'''), 1),

    # 5. Both constructors init the new field (2 occurrences).
    (s('''            last_clade_spec_tick: 0,
'''),
     s('''            last_clade_spec_tick: 0,
            clade_burdens: Vec::new(),
'''), 2),

    # 6. check_speciation part (a): extinction events in the unified log.
    (s('''        for clade in self.species_clades.iter_mut() {
            if clade.members.is_empty() && clade.death_tick.is_none() {
                clade.death_tick = Some(now);
            }
        }
        if deco_idx.is_empty() { return; }
'''),
     s('''        // Phase 16: a clade whose member count hits zero freezes its record
        // AND logs an extinction event in the unified log. kind 2 = the clade
        // carried a competitive burden in the previous pass (Gause exclusion),
        // kind 1 = plain death (resource loss / old age).
        let mut extinct: Vec<(u32, u8)> = Vec::new();
        for clade in self.species_clades.iter_mut() {
            if clade.members.is_empty() && clade.death_tick.is_none() {
                clade.death_tick = Some(now);
                let kind = if self.clade_burdens.iter().any(|(id, b)| *id == clade.id && *b > 0.0) {
                    EVENT_KIND_COMPETITIVE_EXCLUSION
                } else {
                    EVENT_KIND_EXTINCTION
                };
                extinct.push((clade.id, kind));
            }
        }
        for (id, kind) in extinct {
            self.speciation_events.push(CladeSpeciationEvent {
                tick: now,
                parent_clade_id: id,
                child_clade_ids: Vec::new(),
                genetic_distance: 0.0,
                event_kind: kind,
            });
        }
        if deco_idx.is_empty() { return; }
'''), 1),

    # 7. Root clade literal: niche pinned at 0.
    (s('''            self.species_clades.push(SpeciesClade {
                id,
                parent_id: None,
                kind: DECO,
                name: DECO.to_string(),
                birth_tick: 0,
                death_tick: None,
                centroid_genome: centroid,
                members: deco_idx.iter().map(|&i| self.agents[i].id).collect(),
            });
'''),
     s('''            self.species_clades.push(SpeciesClade {
                id,
                parent_id: None,
                kind: DECO,
                name: DECO.to_string(),
                birth_tick: 0,
                death_tick: None,
                centroid_genome: centroid,
                members: deco_idx.iter().map(|&i| self.agents[i].id).collect(),
                niche_offset: 0.0,
            });
'''), 1),

    # 8. check_speciation step (e): hook the Phase 16 engine (niche.rs).
    (s('''        // (d) refresh cached members + centroids from live agents.
        self.refresh_clade_stats(&deco_idx);
    }
'''),
     s('''        // (d) refresh cached members + centroids from live agents.
        self.refresh_clade_stats(&deco_idx);

        // (e) Phase 16: niche re-derivation + competitive-exclusion burdens
        // (impl in niche.rs; pure bookkeeping — consumes no RNG draws).
        self.update_niches_and_competition();
    }
'''), 1),

    # 9. detect_splits: speciation events carry kind 0.
    (s('''                event = Some(CladeSpeciationEvent {
                    tick: now,
                    parent_clade_id: parent,
                    child_clade_ids: Vec::new(),
                    genetic_distance: 0.0,
                });
'''),
     s('''                event = Some(CladeSpeciationEvent {
                    tick: now,
                    parent_clade_id: parent,
                    child_clade_ids: Vec::new(),
                    genetic_distance: 0.0,
                    event_kind: EVENT_KIND_SPECIATION,
                });
'''), 1),

    # 10. detect_splits: birth-time niche for the daughter clade.
    (s('''            let kind = self.species_clades.iter().find(|c| c.id == parent)
                .map(|c| c.kind).unwrap_or(DECO);
            let cid = self.next_clade_id;
            self.next_clade_id += 1;
            let refs: Vec<&Agent> = members.iter().map(|&p| &self.agents[deco_idx[p]]).collect();
            let centroid = Self::mean_genome(&refs);
            for &p in members.iter() {
                self.agents[deco_idx[p]].clade_id = cid;
            }
            self.species_clades.push(SpeciesClade {
                id: cid,
                parent_id: Some(parent),
                kind,
                name: format!("{}-{}", kind, clade_letter(cid)),
                birth_tick: now,
                death_tick: None,
                centroid_genome: centroid,
                members: members.iter().map(|&p| self.agents[deco_idx[p]].id).collect(),
            });
'''),
     s('''            let kind = self.species_clades.iter().find(|c| c.id == parent)
                .map(|c| c.kind).unwrap_or(DECO);
            let cid = self.next_clade_id;
            self.next_clade_id += 1;
            let refs: Vec<&Agent> = members.iter().map(|&p| &self.agents[deco_idx[p]]).collect();
            let centroid = Self::mean_genome(&refs);
            // Phase 16: birth-time niche = parent niche + centroid eff gap
            // (update_niches_and_competition re-derives it on the same pass;
            // this initial value keeps the record self-consistent).
            let (parent_niche, parent_eff) = self.species_clades.iter()
                .find(|c| c.id == parent)
                .map(|p| (p.niche_offset, p.centroid_genome.deco_efficiency))
                .unwrap_or((0.0, centroid.deco_efficiency));
            let niche0 = (parent_niche
                + (centroid.deco_efficiency - parent_eff) * NICHE_EFF_SCALE)
                .clamp(-1.0, 1.0);
            for &p in members.iter() {
                self.agents[deco_idx[p]].clade_id = cid;
            }
            self.species_clades.push(SpeciesClade {
                id: cid,
                parent_id: Some(parent),
                kind,
                name: format!("{}-{}", kind, clade_letter(cid)),
                birth_tick: now,
                death_tick: None,
                centroid_genome: centroid,
                members: members.iter().map(|&p| self.agents[deco_idx[p]].id).collect(),
                niche_offset: niche0,
            });
'''), 1),

    # 11. update_deco: read the agent's clade niche once per tick.
    (s('''        let g_metabolism = self.agents[idx].genome.metabolism;
        let deco_eff = self.agents[idx].genome.deco_efficiency;
        self.agents[idx].bite -= h;
'''),
     s('''        let g_metabolism = self.agents[idx].genome.metabolism;
        let deco_eff = self.agents[idx].genome.deco_efficiency;
        // Phase 16: the agent's clade niche modulates foraging direction and
        // mineral-deposit bias below (0.0 for root / unassigned agents).
        let clade_niche = self.clade_niche_of(self.agents[idx].clade_id);
        self.agents[idx].bite -= h;
'''), 1),

    # 12. update_deco wander #1 (search-state branch, anchored on "approach").
    (s('''                self.agents[idx].state = "approach";
            } else {
                self.agents[idx].state = "wander";
                let wa = self.rng.next_f64() * std::f64::consts::PI * 2.0;
'''),
     s('''                self.agents[idx].state = "approach";
            } else {
                self.agents[idx].state = "wander";
                // Phase 16: clade niche biases the wander heading — diverging
                // clades roam in different directions (spatial niche split).
                let wa = self.rng.next_f64() * std::f64::consts::PI * 2.0 + clade_niche * NICHE_HEADING_BIAS;
'''), 1),

    # 13. update_deco wander #2 (approach-abandon branch, anchored on "eat").
    (s('''                        self.agents[idx].state = "eat";
                    } else {
                        self.agents[idx].state = "wander";
                        let wa = self.rng.next_f64() * std::f64::consts::PI * 2.0;
'''),
     s('''                        self.agents[idx].state = "eat";
                    } else {
                        self.agents[idx].state = "wander";
                        // Phase 16: same heading bias as the search branch.
                        let wa = self.rng.next_f64() * std::f64::consts::PI * 2.0 + clade_niche * NICHE_HEADING_BIAS;
'''), 1),

    # 14. update_deco deposit: niche-directed mineral enrichment.
    (s('''                        for (ni, nj) in [(gi_col + 1, gi_row), (gi_col - 1, gi_row), (gi_col, gi_row + 1), (gi_col, gi_row - 1)] {
                            if ni >= 0 && nj >= 0 && (ni as u32) < GRID_N && (nj as u32) < GRID_N {
                                let ng = (nj as usize) * GRID_N as usize + ni as usize;
                                if self.suit[ng] > 0.02 {
                                    self.mineral_field[ng] = (self.mineral_field[ng] as f64 + boost * 0.4).min(1.0) as f32;
                                }
                            }
                        }
'''),
     s('''                        for (ni, nj) in [(gi_col + 1, gi_row), (gi_col - 1, gi_row), (gi_col, gi_row + 1), (gi_col, gi_row - 1)] {
                            if ni >= 0 && nj >= 0 && (ni as u32) < GRID_N && (nj as u32) < GRID_N {
                                let ng = (nj as usize) * GRID_N as usize + ni as usize;
                                if self.suit[ng] > 0.02 {
                                    self.mineral_field[ng] = (self.mineral_field[ng] as f64 + boost * 0.4).min(1.0) as f32;
                                }
                            }
                        }
                        // Phase 16: niche-directed deposit — clades with
                        // diverging niches enrich different directions, which
                        // spatially partitions the soil-mineral resource.
                        if clade_niche.abs() > 0.05 {
                            let theta = clade_niche * std::f64::consts::PI;
                            let (dxn, dzn) = (aether_libm::cos(theta), aether_libm::sin(theta));
                            let cands = [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)];
                            let mut best = cands[0];
                            let mut best_dot = f64::NEG_INFINITY;
                            for &cand in &cands {
                                let d = dxn * cand.0 as f64 + dzn * cand.1 as f64;
                                if d > best_dot { best_dot = d; best = cand; }
                            }
                            let dni = gi_col + best.0;
                            let dnj = gi_row + best.1;
                            if dni >= 0 && dnj >= 0 && (dni as u32) < GRID_N && (dnj as u32) < GRID_N {
                                let ng = (dnj as usize) * GRID_N as usize + dni as usize;
                                if self.suit[ng] > 0.02 {
                                    self.mineral_field[ng] = (self.mineral_field[ng] as f64 + boost * NICHE_DEPOSIT_BIAS).min(1.0) as f32;
                                }
                            }
                        }
'''), 1),

    # 15. update_deco metabolism: competitive burden multiplier.
    (s('''        let total_drain = g_metabolism * (0.12 + 0.25 * activity) * crowd_penalty * season_meta_deco * h * eff_metab_mod;
'''),
     s('''        // Phase 16: competitive-exclusion burden — the loser of an overlapping
        // clade pair drains +15 % faster (burden map rebuilt every detection
        // pass; always empty in legacy mode).
        let niche_burden = self.clade_burden_of(self.agents[idx].clade_id);
        let total_drain = g_metabolism * (0.12 + 0.25 * activity) * crowd_penalty * season_meta_deco * h * eff_metab_mod * (1.0 + niche_burden);
'''), 1),

    # 16. clade_to_json: append the niche key (pre-v3 saves default 0.0).
    (s('''    format!(
        "{{\\"id\\":{},\\"parentId\\":{},\\"kind\\":\\"{}\\",\\"name\\":\\"{}\\",\\"birthTick\\":{},\\"deathTick\\":{},\\"centroidEff\\":{},\\"aliveCount\\":{}}}",
        c.id, parent, c.kind, c.name, c.birth_tick, death,
        c.centroid_genome.deco_efficiency, c.members.len(),
    )
'''),
     s('''    format!(
        "{{\\"id\\":{},\\"parentId\\":{},\\"kind\\":\\"{}\\",\\"name\\":\\"{}\\",\\"birthTick\\":{},\\"deathTick\\":{},\\"centroidEff\\":{},\\"aliveCount\\":{},\\"niche\\":{}}}",
        c.id, parent, c.kind, c.name, c.birth_tick, death,
        c.centroid_genome.deco_efficiency, c.members.len(), c.niche_offset,
    )
'''), 1),

    # 17. clade_event_to_json: append the kind key (default 0).
    (s('''    format!(
        "{{\\"tick\\":{},\\"parentCladeId\\":{},\\"childCladeIds\\":[{}],\\"geneticDistance\\":{}}}",
        e.tick, e.parent_clade_id, children, e.genetic_distance,
    )
'''),
     s('''    format!(
        "{{\\"tick\\":{},\\"parentCladeId\\":{},\\"childCladeIds\\":[{}],\\"geneticDistance\\":{},\\"kind\\":{}}}",
        e.tick, e.parent_clade_id, children, e.genetic_distance, e.event_kind,
    )
'''), 1),

    # 18. to_save_state: serialize the burden map.
    (s('''            LastCladeSpecTick: self.last_clade_spec_tick,
'''),
     s('''            LastCladeSpecTick: self.last_clade_spec_tick,
            // Phase 16: burden map as compact JSON (see save_state::SavedBurdens).
            CladeNicheBurdens: save_state::SavedBurdens(format!(
                "[{}]",
                self.clade_burdens.iter()
                    .map(|(id, b)| format!("[{},{}]", id, b))
                    .collect::<Vec<String>>()
                    .join(",")
            )),
'''), 1),

    # 19. from_save_state: parse niche (default 0.0).
    (s('''            let centroid_eff = c["centroidEff"].as_f64().unwrap_or(1.0);
            SpeciesClade {
                id,
                parent_id,
                kind: save_state::intern_str(&kind),
                name,
                birth_tick,
                death_tick,
                centroid_genome: { let mut g = Genome::default_deco(); g.deco_efficiency = centroid_eff; g },
                members: Vec::new(),
            }
        }).collect();
'''),
     s('''            let centroid_eff = c["centroidEff"].as_f64().unwrap_or(1.0);
            // Phase 16: pre-v3 saves carry no "niche" key -> 0.0; the next
            // detection pass re-derives it from the parent chain.
            let niche = c.get("niche").and_then(|v| v.as_f64()).unwrap_or(0.0);
            SpeciesClade {
                id,
                parent_id,
                kind: save_state::intern_str(&kind),
                name,
                birth_tick,
                death_tick,
                centroid_genome: { let mut g = Genome::default_deco(); g.deco_efficiency = centroid_eff; g },
                members: Vec::new(),
                niche_offset: niche,
            }
        }).collect();
'''), 1),

    # 20. from_save_state: parse event kind (default 0 = speciation).
    (s('''        let speciation_events = json_events.iter().map(|e| CladeSpeciationEvent {
            tick: e["tick"].as_u64().unwrap_or(0) as u32,
            parent_clade_id: e["parentCladeId"].as_u64().unwrap_or(0) as u32,
            child_clade_ids: e.get("childCladeIds")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().map(|v| v.as_u64().unwrap_or(0) as u32).collect())
                .unwrap_or_default(),
            genetic_distance: e["geneticDistance"].as_f64().unwrap_or(0.0),
        }).collect();
'''),
     s('''        let speciation_events = json_events.iter().map(|e| CladeSpeciationEvent {
            tick: e["tick"].as_u64().unwrap_or(0) as u32,
            parent_clade_id: e["parentCladeId"].as_u64().unwrap_or(0) as u32,
            child_clade_ids: e.get("childCladeIds")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().map(|v| v.as_u64().unwrap_or(0) as u32).collect())
                .unwrap_or_default(),
            genetic_distance: e["geneticDistance"].as_f64().unwrap_or(0.0),
            // Phase 16: pre-v3 saves default to speciation (0).
            event_kind: e.get("kind").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
        }).collect();
'''), 1),

    # 21. from_save_state: parse the burden map.
    (s('''        let next_clade_id = state.NextCladeId;
        let last_clade_spec_tick = state.LastCladeSpecTick;
'''),
     s('''        let next_clade_id = state.NextCladeId;
        let last_clade_spec_tick = state.LastCladeSpecTick;

        // Phase 16: competitive-exclusion burden map `[[cladeId,burden],…]`.
        // Pre-v3 saves carry the serde default (empty string) -> no burdens;
        // the next detection pass recomputes them from live clades.
        let clade_burdens: Vec<(u32, f64)> =
            serde_json::from_str::<serde_json::Value>(&state.CladeNicheBurdens.0)
                .ok()
                .and_then(|v| v.as_array().cloned())
                .map(|arr| {
                    arr.iter().filter_map(|pair| {
                        let id = pair.get(0)?.as_u64()? as u32;
                        let b = pair.get(1)?.as_f64()?;
                        Some((id, b))
                    }).collect::<Vec<(u32, f64)>>()
                })
                .unwrap_or_default();
'''), 1),

    # 22. from_save_state struct literal: wire the burdens in.
    (s('''            species_clades,
            next_clade_id,
            speciation_events,
            last_clade_spec_tick,
            seed: state.seed,
'''),
     s('''            species_clades,
            next_clade_id,
            speciation_events,
            last_clade_spec_tick,
            clade_burdens,
            seed: state.seed,
'''), 1),
]

# ---------------------------------------------------------------------------
# wasm.rs
# ---------------------------------------------------------------------------

WASM = "core/crates/aether-sim-ecology/src/wasm.rs"

WASM_EDITS = [
    (s('''pub const CLADE_RECORD_SIZE: usize = 36;
'''),
     s('''/// Phase 16: 36 -> 40 bytes (niche f32 appended at offset 36; existing
/// offsets are unchanged so JS readers of the old layout only miss the tail).
pub const CLADE_RECORD_SIZE: usize = 40;
'''), 1),

    (s('''/// Phase 15: phylogeny readout region appended after the histogram area.
/// Clade record (36 bytes, mirrors sim_get_clade):
///   [id u32][parent u32 (u32::MAX = root)][kind_hash u32][birth u32]
///   [death u32 (u32::MAX = alive)][alive_count u32][centroid_eff f64][argb u32]
/// Event record (56 bytes, mirrors sim_get_event):
///   [tick u32][parent u32][child_count u32][reserved u32]
///   [genetic_distance f64][child0..7 u32]
'''),
     s('''/// Phase 15: phylogeny readout region appended after the histogram area.
/// Clade record (40 bytes since Phase 16, mirrors sim_get_clade):
///   [id u32][parent u32 (u32::MAX = root)][kind_hash u32][birth u32]
///   [death u32 (u32::MAX = alive)][member_count u32][centroid_eff f64][argb u32]
///   [niche f32]   <- Phase 16 tail field (member_count = alive_count @20)
/// Event record (56 bytes, mirrors sim_get_event):
///   [tick u32][parent u32][child_count u32][event_kind u32 (was reserved)]
///   [genetic_distance f64][child0..7 u32]
'''), 1),

    (s('''        rec[32..36].copy_from_slice(&argb.to_le_bytes());
        std::ptr::copy_nonoverlapping(rec.as_ptr(), out, CLADE_RECORD_SIZE);
        1
    }
'''),
     s('''        rec[32..36].copy_from_slice(&argb.to_le_bytes());
        // Phase 16: clade niche position (f32 tail field; offset 36 was the
        // record end before Phase 16, so old readers simply never see it).
        rec[36..40].copy_from_slice(&(c.niche_offset as f32).to_le_bytes());
        std::ptr::copy_nonoverlapping(rec.as_ptr(), out, CLADE_RECORD_SIZE);
        1
    }
'''), 1),

    (s('''        let n_children = e.child_clade_ids.len().min(MAX_EVENT_CHILDREN);
        rec[8..12].copy_from_slice(&(n_children as u32).to_le_bytes());
        rec[16..24].copy_from_slice(&e.genetic_distance.to_le_bytes());
'''),
     s('''        let n_children = e.child_clade_ids.len().min(MAX_EVENT_CHILDREN);
        rec[8..12].copy_from_slice(&(n_children as u32).to_le_bytes());
        // Phase 16: the reserved u32 now carries the event kind
        // (0 = speciation, 1 = extinction, 2 = competitive exclusion).
        rec[12..16].copy_from_slice(&(e.event_kind as u32).to_le_bytes());
        rec[16..24].copy_from_slice(&e.genetic_distance.to_le_bytes());
'''), 1),

    (s('''        std::ptr::copy_nonoverlapping(rec.as_ptr(), out, EVENT_RECORD_SIZE);
        n_children as u32
    }
'''),
     s('''        std::ptr::copy_nonoverlapping(rec.as_ptr(), out, EVENT_RECORD_SIZE);
        // Return contract (Phase 16): >= 1 on success (children count, min 1
        // so extinction records — 0 children — stay distinguishable from
        // failure), 0 on failure. JS reads the true child count from the
        // record's child_count field at offset 8.
        n_children.max(1) as u32
    }
'''), 1),

    (s('''                assert_eq!(state.schema_name, crate::save_state::SAVE_STATE_SCHEMA_NAME);
                assert_eq!(state.version, crate::save_state::SAVE_STATE_VERSION);
'''),
     s('''                assert_eq!(state.schema_name, crate::save_state::SAVE_STATE_SCHEMA_NAME);
                // Phase 16: accept older schema revisions (v1/v2 saves restore
                // with #[serde(default)] niche/burden fields), reject future ones.
                assert!(
                    state.version <= crate::save_state::SAVE_STATE_VERSION,
                    "unsupported future save state version {} (current is {})",
                    state.version, crate::save_state::SAVE_STATE_VERSION
                );
'''), 1),
]

# ---------------------------------------------------------------------------
# living/src/renderer-phylogeny.mjs
# ---------------------------------------------------------------------------

PHYLO = "living/src/renderer-phylogeny.mjs"

PHYLO_EDITS = [
    (s('''// clade 记录布局（与 wasm.rs sim_get_clade 同步，36 bytes）：
//   [id u32][parent u32 (u32.MAX=根)][kind_hash u32][birth u32]
//   [death u32 (u32.MAX=存活)][alive_count u32][centroid_eff f64][argb u32]
// event 记录布局（与 wasm.rs sim_get_event 同步，56 bytes）：
//   [tick u32][parent u32][child_count u32][reserved u32]
//   [genetic_distance f64][child0..7 u32]
'''),
     s('''// clade 记录布局（与 wasm.rs sim_get_clade 同步，Phase 16 起 40 bytes）：
//   [id u32][parent u32 (u32.MAX=根)][kind_hash u32][birth u32]
//   [death u32 (u32.MAX=存活)][member_count u32][centroid_eff f64][argb u32]
//   [niche f32]  ← Phase 16 尾部追加（member_count 即原 alive_count @20）
// event 记录布局（与 wasm.rs sim_get_event 同步，56 bytes）：
//   [tick u32][parent u32][child_count u32][event_kind u32（原 reserved）]
//   [genetic_distance f64][child0..7 u32]
//   kind：0=speciation / 1=extinction / 2=竞争排斥灭绝（Phase 16）
'''), 1),

    (s('''export const CLADE_RECORD_SIZE = 36;
'''),
     s('''export const CLADE_RECORD_SIZE = 40; // Phase 16: +niche f32 tail (offset 36)
'''), 1),

    (s('''      const argb = dv.getUint32(32, true);
      const nameLen = E.sim_clade_name(sim, i, namePtr, nameCap) | 0;
'''),
     s('''      const argb = dv.getUint32(32, true);
      const nicheOffset = dv.getFloat32(36, true); // Phase 16 tail field
      const nameLen = E.sim_clade_name(sim, i, namePtr, nameCap) | 0;
'''), 1),

    (s('''        aliveCount,
        centroidEff,
        color: `#${(argb & 0xffffff).toString(16).padStart(6, '0')}`,
'''),
     s('''        aliveCount,
        centroidEff,
        nicheOffset,
        color: `#${(argb & 0xffffff).toString(16).padStart(6, '0')}`,
'''), 1),

    (s('''    for (let i = 0; i < eventCount && i < EVENT_CAP; i++) {
      const childCount = E.sim_get_event(sim, i, eventPtr) | 0;
      if (childCount === 0) break; // 0 = 失败（真实 event 至少 1 个子 clade）
      const dv = new DataView(E.memory.buffer, eventPtr, EVENT_RECORD_SIZE);
      const tick = dv.getUint32(0, true);
      const parentCladeId = dv.getUint32(4, true);
      const n = Math.min(childCount, MAX_EVENT_CHILDREN);
      const childCladeIds = [];
      for (let k = 0; k < n; k++) childCladeIds.push(dv.getUint32(24 + k * 4, true));
      events.push({ tick, parentCladeId, childCladeIds, geneticDistance: dv.getFloat64(16, true) });
    }
'''),
     s('''    for (let i = 0; i < eventCount && i < EVENT_CAP; i++) {
      // Phase 16: 返回值 >= 1 表示记录有效（灭绝事件 0 个子 clade 也返回 1），
      // 0 才是失败。真实 child_count 从记录 offset 8 读取；offset 12 是 kind
      // （0=speciation / 1=extinction / 2=竞争排斥灭绝）。
      const ret = E.sim_get_event(sim, i, eventPtr) | 0;
      if (ret === 0) break; // 0 = 失败
      const dv = new DataView(E.memory.buffer, eventPtr, EVENT_RECORD_SIZE);
      const tick = dv.getUint32(0, true);
      const parentCladeId = dv.getUint32(4, true);
      const kind = dv.getUint32(12, true);
      const n = Math.min(dv.getUint32(8, true), MAX_EVENT_CHILDREN);
      const childCladeIds = [];
      for (let k = 0; k < n; k++) childCladeIds.push(dv.getUint32(24 + k * 4, true));
      events.push({ tick, parentCladeId, childCladeIds, geneticDistance: dv.getFloat64(16, true), kind });
    }
'''), 1),

    (s('''    aliveCount: c.aliveCount ?? 0,
    centroidEff: c.centroidEff ?? 0,
    color: c.color || '#cfe3d6',
'''),
     s('''    aliveCount: c.aliveCount ?? 0,
    centroidEff: c.centroidEff ?? 0,
    nicheOffset: c.nicheOffset ?? 0,
    color: c.color || '#cfe3d6',
'''), 1),

    (s('''  const eventDots = events.map((ev) => {
    const p = byId.get(ev.parentCladeId);
    return {
      tick: ev.tick,
      parentCladeId: ev.parentCladeId,
      geneticDistance: ev.geneticDistance,
      x: nx(ev.tick),
      y: p ? (yOf.get(p.id) ?? 0.5) : 0.5,
    };
  });
'''),
     s('''  const eventDots = events.map((ev) => {
    const p = byId.get(ev.parentCladeId);
    return {
      tick: ev.tick,
      parentCladeId: ev.parentCladeId,
      geneticDistance: ev.geneticDistance,
      kind: ev.kind ?? 0,
      x: nx(ev.tick),
      y: p ? (yOf.get(p.id) ?? 0.5) : 0.5,
    };
  });
'''), 1),

    (s('''    // 分化事件：红点。
    for (const ev of g.events) {
      ctx.fillStyle = '#ff5a5a';
'''),
     s('''    // 事件点：speciation 红 / extinction 灰 / 竞争排斥琥珀（Phase 16）。
    for (const ev of g.events) {
      ctx.fillStyle = ev.kind === 2 ? '#ffb347' : ev.kind === 1 ? '#8a9a90' : '#ff5a5a';
'''), 1),

    (s('''      best.geneticDistance != null ? `d=${best.geneticDistance.toFixed(2)} vs parent` : null,
      `eff=${best.centroidEff.toFixed(2)}`,
'''),
     s('''      best.geneticDistance != null ? `d=${best.geneticDistance.toFixed(2)} vs parent` : null,
      `eff=${best.centroidEff.toFixed(2)}`,
      `niche=${(best.nicheOffset ?? 0).toFixed(2)}`,
'''), 1),
]

# ---------------------------------------------------------------------------
# living/src/wasm-viewer.mjs
# ---------------------------------------------------------------------------

VIEWER = "living/src/wasm-viewer.mjs"

VIEWER_EDITS = [
    (s('''import { createPhylogenyOverlay, readPhylogenyState, buildPhylogenyGraph,
  CLADE_CAP, EVENT_CAP, CLADE_RECORD_SIZE, EVENT_RECORD_SIZE, CLADE_NAME_CAP,
} from './renderer-phylogeny.mjs';
'''),
     s('''import { createPhylogenyOverlay, readPhylogenyState, buildPhylogenyGraph,
  CLADE_CAP, EVENT_CAP, CLADE_RECORD_SIZE, EVENT_RECORD_SIZE, CLADE_NAME_CAP,
} from './renderer-phylogeny.mjs';
import { createNicheOverlay, buildNicheCloud } from './renderer-niche.mjs';
'''), 1),

    (s('''  const seenCladeIds = new Set();
  const seenDeadIds = new Set();
  let lastPhyloPull = -Infinity;
'''),
     s('''  const seenCladeIds = new Set();
  const seenDeadIds = new Set();
  let lastPhyloPull = -Infinity;

  // Phase 16: 生态位空间 overlay（U 键切换；复用 phylogeny 的 60-tick 拉取节奏）。
  const nicheOverlay = phyloSupported ? createNicheOverlay() : null;
  let nicheOn = true;
'''), 1),

    (s('''  function renderPhylo() {
    if (!phyloOverlay) return;
    phyloOverlay.update(buildPhylogenyGraph(phyloClades, phyloEvents, handle.tick(), { fossilHighlight: fossilOn }));
  }
'''),
     s('''  function renderPhylo() {
    if (!phyloOverlay) return;
    phyloOverlay.update(buildPhylogenyGraph(phyloClades, phyloEvents, handle.tick(), { fossilHighlight: fossilOn }));
  }

  // Phase 16: niche 云图渲染（与 cladogram 共享同一份 clade 快照）。
  function renderNiche() {
    if (!nicheOverlay) return;
    nicheOverlay.update(buildNicheCloud(phyloClades, handle.tick()));
  }
'''), 1),

    (s('''      if (c.extinct && !seenDeadIds.has(c.id)) {
        seenDeadIds.add(c.id);
        if (!initial) toast(`⚰ ${c.name} extinct @ t=${c.deathTick}`);
      }
'''),
     s('''      if (c.extinct && !seenDeadIds.has(c.id)) {
        seenDeadIds.add(c.id);
        if (!initial) {
          // Phase 16: 竞争排斥灭绝（event kind 2）用 ⚔ 标记，区分普通死亡 ⚰。
          const xev = state.events.find((e) => e.parentCladeId === c.id && (e.kind ?? 0) >= 1);
          toast(xev && xev.kind === 2
            ? `⚔ ${c.name} 竞争排斥灭绝 @ t=${c.deathTick}`
            : `⚰ ${c.name} extinct @ t=${c.deathTick}`);
        }
      }
'''), 1),

    (s('''    phyloClades = state.clades;
    phyloEvents = state.events;
    renderPhylo();
  }
'''),
     s('''    phyloClades = state.clades;
    phyloEvents = state.events;
    renderPhylo();
    renderNiche();
  }
'''), 1),

    (s('''  if (phyloOverlay) {
    phyloOverlay.setVisible(true);
    pullPhylogeny(true); // 预热后首次拉取（root clade 已在 t≈60 诞生）
  }
'''),
     s('''  if (phyloOverlay) {
    phyloOverlay.setVisible(true);
    pullPhylogeny(true); // 预热后首次拉取（root clade 已在 t≈60 诞生）
  }
  if (nicheOverlay) {
    nicheOverlay.setVisible(true);
    renderNiche();
  }
'''), 1),

    (s('''    } else if (e.code === 'KeyT' && phyloOverlay) {
'''),
     s('''    } else if (e.code === 'KeyU' && nicheOverlay) {
      // Phase 16: 生态位空间开关。
      nicheOn = !nicheOn;
      nicheOverlay.setVisible(nicheOn);
      if (nicheOn) renderNiche();
      toast(`niche space ${nicheOn ? 'ON' : 'OFF'}（U 切换）`);
    } else if (e.code === 'KeyT' && phyloOverlay) {
'''), 1),

    (s('''        for (const ov of [histOverlay, specOverlay, phyloOverlay]) {
'''),
     s('''        for (const ov of [histOverlay, specOverlay, phyloOverlay, nicheOverlay]) {
'''), 1),
]

# ---------------------------------------------------------------------------
# living/test_save_restore.mjs (scratch-size constant: clade record 36 -> 40)
# ---------------------------------------------------------------------------

SAVERESTORE = "living/test_save_restore.mjs"

SAVERESTORE_EDITS = [
    (s('''const CLADE_BYTES = 64 * 36 + 64 * 56 + 32;
'''),
     s('''const CLADE_BYTES = 64 * 40 + 64 * 56 + 32; // Phase 16: clade record 36→40 (niche f32)
'''), 1),
]

# ---------------------------------------------------------------------------

ALL = [
    (ECOLOGY, ECOLOGY_EDITS, True),
    (WASM, WASM_EDITS, False),
    (PHYLO, PHYLO_EDITS, False),
    (VIEWER, VIEWER_EDITS, False),
    (SAVERESTORE, SAVERESTORE_EDITS, False),
]


def main():
    global PATHS_DRY_RUN
    mode = sys.argv[1] if len(sys.argv) > 1 else "--check"
    if mode == "--check":
        PATHS_DRY_RUN = True
    elif mode != "--apply":
        print(__doc__)
        return 2
    for rel, edits, restore in ALL:
        apply_edits(rel, edits, restore_from_git=restore)
    print("ALL PATCHES " + ("VERIFIED (dry run)" if PATHS_DRY_RUN else "APPLIED"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
