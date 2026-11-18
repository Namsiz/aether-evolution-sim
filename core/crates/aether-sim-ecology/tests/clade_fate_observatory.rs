//! Phase 46.1 — clade fate observatory: the phylogeny under 1M ticks.
//!
//! Phase 45 proved a world can mint its first natural species (t=3780, DECO
//! fork). This run asks what happens to those species over geological time:
//! does the tree keep branching, or does each fork settle into a stable
//! daughter lineage? How long do species live? Which lineages win?
//!
//! Run: cargo test --release --test clade_fate_observatory -- --ignored --nocapture
//!
//! Arms are env-parameterised so several instances can run in parallel (one
//! process per seed/arm): `PHASE_SEED=42`, `PHASE_TICKS=80000`, and
//! `PHASE_STRUCTURE_ENV=on|off` (Phase 55: the structure-env A/B — "on" flips
//! the net-zero dipole on before the first tick, "off" freezes it at all-1.0;
//! unset runs the compiled default). Output files carry the arm suffix so
//! parallel A/B runs never overwrite each other.

use aether_sim_ecology::civilization_metrics::{format_line, measure};
use aether_sim_ecology::ecology::Ecology;

fn env_u32(name: &str, default: u32) -> u32 {
    std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn env_str(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.is_empty())
}

const SAMPLE_EVERY: u32 = 5_000;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * ((t as f64 / 300.0) * std::f64::consts::PI).sin()
}

/// Phase 47 region sectors (polar angle, four quadrants) — mirrored from
/// `Ecology::region_of` so the observatory can bucket agents and structures
/// without exposing the private helper.
fn region_of(x: f64, z: f64) -> u8 {
    let two_pi = std::f64::consts::PI * 2.0;
    let a = aether_libm::atan2(z, x) + std::f64::consts::PI;
    let sector = (a / two_pi * 4.0).floor() as i32;
    sector.clamp(0, 3) as u8
}

/// Spatial variance of the standing grass field over live cells — the
/// heterogeneity metric of the Phase 55 A/B (suit<=0.02 cells never grow and
/// would add a constant floor to the variance).
fn grass_stats(eco: &Ecology) -> (f64, f64) {
    let mut n = 0.0f64;
    let mut sum = 0.0f64;
    for i in 0..eco.grass_field().len() {
        if eco.suit_field()[i] <= 0.02 { continue; }
        n += 1.0;
        sum += eco.grass_field()[i] as f64;
    }
    let mean = sum / n;
    let mut var = 0.0f64;
    for i in 0..eco.grass_field().len() {
        if eco.suit_field()[i] <= 0.02 { continue; }
        var += (eco.grass_field()[i] as f64 - mean) * (eco.grass_field()[i] as f64 - mean);
    }
    (mean, var / n)
}

/// Mean and variance of the structure regrowth multiplier (0 when the field
/// is all-1.0 — the OFF arm's heterogeneity is exactly zero by construction).
fn env_stats(eco: &Ecology) -> (f64, f64) {
    let n = eco.structure_env.len() as f64;
    let mean = eco.structure_env.iter().map(|&v| v as f64).sum::<f64>() / n;
    let var = eco.structure_env.iter().map(|&v| {
        let d = v as f64 - mean; d * d
    }).sum::<f64>() / n;
    (mean, var)
}

#[test]
#[ignore = "million-tick clade-fate run — ~70 minutes"]
fn clade_fate_observation() {
    let seed = env_u32("PHASE_SEED", 20261102);
    let ticks = env_u32("PHASE_TICKS", 1_000_000);
    let arm = match env_str("PHASE_STRUCTURE_ENV").as_deref() {
        Some("on") => "envon",
        Some("off") => "envoff",
        _ => "default",
    };
    // Calibration-sweep tag (Phase 55): parallel arms of the same switch must
    // never overwrite each other's output files.
    let tag = env_str("PHASE_ARM_TAG").unwrap_or_default();
    let suffix = if tag.is_empty() { String::new() } else { format!("_{tag}") };
    let csv_path = format!("/tmp/clade_fate_{seed}_{arm}{suffix}.csv");
    let tree_path = format!("/tmp/clade_tree_{seed}_{arm}{suffix}.csv");
    let culture_path = format!("/tmp/clade_culture_{seed}_{arm}{suffix}.csv");
    let mut csv = String::from(
        "t,pop,herb,pred,scav,omn,fish,deco,clades_live,clades_total,events,tokens,bearers,groups,levels,authorities,grass_mean,grass_var,env_mean,env_var,structs,deco_r0,deco_r1,deco_r2,deco_r3,structs_r0,structs_r1,structs_r2,structs_r3\n",
    );
    let mut eco = Ecology::new(seed);
    if arm == "envon" { eco.structure_env_on = true; }
    if arm == "envoff" { eco.structure_env_off(); }
    let mut last_events = 0usize;
    eprintln!("=== clade-fate seed {seed} ({ticks} ticks, structure_env={arm}) ===");
    for t in 1..=ticks {
        eco.update(0.1, daylight(t));
        let n = eco.clade_events().len();
        if n > last_events {
            for e in &eco.clade_events()[last_events..n] {
                eprintln!(
                    "  [split] t={} parent={} children={:?} dist={:.3} kind={} event_kind={}",
                    t, e.parent_clade_id, e.child_clade_ids, e.genetic_distance, e.kind_id, e.event_kind
                );
            }
            last_events = n;
        }
        if t % SAMPLE_EVERY == 0 {
            let m = measure(&eco);
            let kind = |name: &str| m.kinds.iter().find(|k| k.name == name).map(|k| k.alive).unwrap_or(0);
            let live_clades = eco.species_clades().iter().filter(|c| c.death_tick.is_none()).count();
            let auth = eco.living().iter()
                .filter(|a| a.brain_authority == aether_sim_ecology::ecology::BRAIN_AUTHORITY_BRAIN)
                .count();
            let (grass_mean, grass_var) = grass_stats(&eco);
            let (env_mean, env_var) = env_stats(&eco);
            // Phase 55: spatial bookkeeping — do the DECO stay spread over
            // the four regions, and where do the structures pile up? A
            // culture that builds everywhere is not a differentiated culture.
            let mut deco_by_region = [0usize; 4];
            for a in eco.living() {
                if a.kind != aether_sim_ecology::ecology::DECO { continue; }
                let r = region_of(a.x, a.z) as usize;
                if r < 4 { deco_by_region[r] += 1; }
            }
            let mut structs_by_region = [0usize; 4];
            for s in &eco.structure_registry.structures {
                let r = region_of(s.x, s.z) as usize;
                if r < 4 { structs_by_region[r] += 1; }
            }
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.5},{:.5},{:.5},{:.5},{},{},{},{},{},{},{},{}\n",
                t, m.information.population, kind("herb"), kind("pred"), kind("scav"),
                kind("omn"), kind("fish"), kind("deco"),
                live_clades, eco.species_clades().len(), last_events,
                m.information.token_count, m.information.token_bearers,
                eco.groups.len(), m.complexity.selection_levels, auth,
                grass_mean, grass_var, env_mean, env_var, eco.structure_registry.len(),
                deco_by_region[0], deco_by_region[1], deco_by_region[2], deco_by_region[3],
                structs_by_region[0], structs_by_region[1], structs_by_region[2], structs_by_region[3],
            ));
            if t % 50_000 == 0 {
                eprintln!("[fate] t={} {} live_clades={} total={} events={} grass_var={:.5} env_var={:.5} structs={}",
                    t, format_line(&m), live_clades, eco.species_clades().len(), last_events,
                    grass_var, env_var, eco.structure_registry.len());
            }
        }
    }
    // Final tree dump: every clade with birth/death/members/kind.
    let mut tree = String::from("id,parent,kind,kind_id,derived_kind,birth,death,name,members,axes\n");
    for c in eco.species_clades() {
        tree.push_str(&format!(
            "{},{:?},{},{},{:?},{},{},{},{},{:?}\n",
            c.id, c.parent_id, c.kind, c.kind_id, c.derived_kind,
            c.birth_tick, c.death_tick.map(|d| d.to_string()).unwrap_or_default(),
            c.name, c.members.len(), c.axes,
        ));
    }
    // Phase 55: per-clade culture dump — do live clades carry different
    // structural signatures? Group the token bearers by (kind, clade) — a
    // clade id alone mixes every species that never forked (they all sit in
    // clade 0) — then attribute the standing structures to the clades whose
    // repertoire contains their token.
    let mut culture = String::from(
        "kind,clade_id,clade_name,members_alive,mean_token_grass_mod,frac_grass_positive,structs_with_own_token,mean_x,mean_z\n",
    );
    let clade_name: std::collections::HashMap<u32, String> = eco.species_clades().iter()
        .map(|c| (c.id, c.name.clone())).collect();
    // (kind, clade) → (members, grass_mod sum, positive count, token count)
    let mut by_clade: std::collections::HashMap<(u16, u32), (usize, f64, usize, usize)> =
        std::collections::HashMap::new();
    let mut clade_tokens: std::collections::HashMap<(u16, u32), std::collections::HashSet<u32>> =
        std::collections::HashMap::new();
    for a in eco.living() {
        if a.known_tokens.is_empty() { continue; }
        let e = by_clade.entry((a.kind, a.clade_id)).or_insert((0, 0.0, 0, 0));
        e.0 += 1;
        e.3 += a.known_tokens.len();
        for &tid in &a.known_tokens {
            clade_tokens.entry((a.kind, a.clade_id)).or_default().insert(tid);
            if let Some(tok) = eco.token_registry.get(tid) {
                let (gm, _) = aether_sim_ecology::culture::token_to_env_mod(tok);
                e.1 += gm;
                if gm > 0.0 { e.2 += 1; }
            }
        }
    }
    let mut clades: Vec<_> = by_clade.into_iter().collect();
    clades.sort_by_key(|((kind, id), _)| (*kind, *id));
    for ((kind, id), (members, gm_sum, pos, token_n)) in clades {
        // structures whose token this clade holds
        let mut n_struct = 0usize;
        let mut sx = 0.0f64;
        let mut sz = 0.0f64;
        if let Some(tokens) = clade_tokens.get(&(kind, id)) {
            for s in &eco.structure_registry.structures {
                if tokens.contains(&s.token_id) {
                    n_struct += 1;
                    sx += s.x;
                    sz += s.z;
                }
            }
        }
        culture.push_str(&format!(
            "{},{},{},{},{:.5},{:.3},{},{:.1},{:.1}\n",
            kind,
            id,
            clade_name.get(&id).map(|s| s.as_str()).unwrap_or("?"),
            members,
            if token_n > 0 { gm_sum / token_n as f64 } else { 0.0 },
            if token_n > 0 { pos as f64 / token_n as f64 } else { 0.0 },
            n_struct,
            if n_struct > 0 { sx / n_struct as f64 } else { 0.0 },
            if n_struct > 0 { sz / n_struct as f64 } else { 0.0 },
        ));
    }
    // Standing structures by grass sign: where do positive vs negative
    // tokens build? (x, z) means per sign class.
    let mut pos_n = 0usize; let mut pos_x = 0.0f64; let mut pos_z = 0.0f64;
    let mut neg_n = 0usize; let mut neg_x = 0.0f64; let mut neg_z = 0.0f64;
    for s in &eco.structure_registry.structures {
        let gm = eco.token_registry.get(s.token_id)
            .map(|tok| aether_sim_ecology::culture::token_to_env_mod(tok).0)
            .unwrap_or(0.0);
        if gm > 0.0 { pos_n += 1; pos_x += s.x; pos_z += s.z; }
        else if gm < 0.0 { neg_n += 1; neg_x += s.x; neg_z += s.z; }
    }
    culture.push_str(&format!(
        "# structures: pos={} mean=({:.1},{:.1}) neg={} mean=({:.1},{:.1})\n",
        pos_n, if pos_n > 0 { pos_x / pos_n as f64 } else { 0.0 },
        if pos_n > 0 { pos_z / pos_n as f64 } else { 0.0 },
        neg_n, if neg_n > 0 { neg_x / neg_n as f64 } else { 0.0 },
        if neg_n > 0 { neg_z / neg_n as f64 } else { 0.0 },
    ));
    std::fs::write(&csv_path, csv).unwrap();
    std::fs::write(&tree_path, tree).unwrap();
    std::fs::write(&culture_path, culture).unwrap();
    eprintln!(
        "=== clade-fate done: clades={} events={} csv={csv_path} tree={tree_path} culture={culture_path} ===",
        eco.species_clades().len(), last_events
    );
}
