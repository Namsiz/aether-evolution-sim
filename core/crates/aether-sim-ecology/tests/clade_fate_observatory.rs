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
//! unset runs the compiled default), plus `PHASE_DECO_ENV=on|off` (Phase 56:
//! the decomposition-field A/B on the same terms). Output files carry the arm
//! suffix so parallel A/B runs never overwrite each other.
//!
//! Phase 59 adds `PHASE_REPRO_OLD=on|off`: "on" restores the Phase 58 DECO
//! fertility line (`0.96 × (2 − eff)`, sterile below eff 0.9583); unset runs
//! the saturated fix. The per-clade dump also carries `mean_eff` /
//! `deco_ready` / `min_pair` (Phase 58's death diagnostics).
//!
//! Phase 60 adds `PHASE_DECO_REGION_CAP=on|off`: "on" spreads each DECO kind's
//! budget over the four home regions by local carcass-food retention
//! (`deco_env` mean); unset runs the compiled default.
//!
//! Phase 61 adds `PHASE_RENORM=on|off` (candidate a: renormalise the regional
//! share over the occupied regions), `PHASE_CAP_FLOOR=on|off` (candidate b: an
//! occupied region keeps at least `min(budget, speciation_min_side)` slots)
//! and a second per-clade dump: `/tmp/clade_life_*.csv` carries the
//! engine-level Phase 61 counters (births, deaths by cause, birth refusals by
//! gate, mate misses, peak population, extinction tick, mean age/energy at
//! death) for every clade the world ever booked.
//!
//! Phase 63 adds the kind-budget arms (`research/phase63-report.md`):
//! `PHASE_BUDGET_GROWTH=on` (candidate G: a filled daughter earns one slot per
//! 5 000 survival ticks out of the parent's share), `PHASE_FORK_MIN=10|12|16`
//! (candidate M: both fork floors) and `PHASE_FORK_DAUGHTER_MIN=12`
//! (candidate A: only the daughter's grant floor). The final stderr block also
//! prints every kind's budget, its growth ledger and the DECO family total.

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

/// Phase 56: mean and variance of the carcass decay multiplier (all-1.0 → 0).
fn deco_env_stats(eco: &Ecology) -> (f64, f64) {
    let n = eco.deco_env.len() as f64;
    let mean = eco.deco_env.iter().map(|&v| v as f64).sum::<f64>() / n;
    let var = eco.deco_env.iter().map(|&v| {
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
    let deco_arm = match env_str("PHASE_DECO_ENV").as_deref() {
        Some("on") => "decoon",
        Some("off") => "decooff",
        _ => "decodflt",
    };
    // Phase 57: token-value sign A/B — "neutral" enables the
    // `gain = |grass_mod| × 10` arm; unset runs the compiled default
    // (signed reward = the Phase 56 world).
    let sign_arm = match env_str("PHASE_TOKEN_SIGN").as_deref() {
        Some("signed") => "signed",
        Some("neutral") => "neutral",
        _ => "signdflt",
    };
    // Phase 58: decomposition-clamp sweep arm (`PHASE_DECO_CLAMP=0.6,1.4`)
    // and the design-b placement arm (`PHASE_PLACE_VALUE=on`).
    let clamp_env = env_str("PHASE_DECO_CLAMP");
    let clamp_arm = clamp_env.clone().unwrap_or_else(|| "clampdflt".to_string());
    let place_arm = match env_str("PHASE_PLACE_VALUE").as_deref() {
        Some("on") => "bvalon",
        Some("off") => "bvaloff",
        _ => "bvaldflt",
    };
    // Phase 59: fertility-mapping arm — `PHASE_REPRO_OLD=on` restores the
    // Phase 58 line (sterile below eff 0.9583); unset runs the saturated fix.
    let repro_arm = match env_str("PHASE_REPRO_OLD").as_deref() {
        Some("on") => "reproold",
        Some("off") => "reprodfix",
        _ => "reprodflt",
    };
    // Phase 60: regional DECO capacity arm.
    let regcap_arm = match env_str("PHASE_DECO_REGION_CAP").as_deref() {
        Some("on") => "regcapon",
        Some("off") => "regcapoff",
        _ => "regcapdflt",
    };
    // Phase 61: occupied-region renormalisation arm.
    let renorm_arm = match env_str("PHASE_RENORM").as_deref() {
        Some("on") => "renormon",
        Some("off") => "renormoff",
        Some("daughters") => "renormdau",
        _ => "renormdflt",
    };
    // Phase 61 candidate (b): occupied-region minimum retention share; Phase 62
    // adds "small" (floor only for kinds with budget < 2 × speciation_min_side).
    let floor_arm = match env_str("PHASE_CAP_FLOOR").as_deref() {
        Some("on") => "flooron",
        Some("off") => "flooroff",
        Some("small") => "floorsmall",
        _ => "floordflt",
    };
    // Phase 63 kind-budget arms: G (growth), M (both fork floors),
    // A (daughter grant floor only).
    let growth_arm = match env_str("PHASE_BUDGET_GROWTH").as_deref() {
        Some("on") => "growthon",
        Some("off") => "growthoff",
        _ => "growthdflt",
    };
    let fork_min_env = env_str("PHASE_FORK_MIN");
    let fork_min_arm = fork_min_env.clone().unwrap_or_else(|| "forkmindflt".to_string());
    let fork_dau_env = env_str("PHASE_FORK_DAUGHTER_MIN");
    let fork_dau_arm = fork_dau_env.clone().unwrap_or_else(|| "forkdaudflt".to_string());
    // Phase 65 W1: fork grace (births skip the kind-budget gate for one window).
    let grace_arm = match env_str("PHASE_FORK_GRACE").as_deref() {
        Some("on") => "graceon",
        Some("off") => "graceoff",
        _ => "gracedflt",
    };
    let grace_ticks_env = env_str("PHASE_FORK_GRACE_TICKS");
    // Phase 66 R1/R2 + Phase 67 R3: derived-clade regional mate-radius floor.
    let mate_arm = match env_str("PHASE_MATE_FLOOR").as_deref() {
        Some("off") => "mateoff",
        Some("r1") => "mater1",
        Some("r2") => "mater2",
        Some("r3") => "mater3",
        _ => "matedflt",
    };
    // Calibration-sweep tag (Phase 55): parallel arms of the same switch must
    // never overwrite each other's output files.
    let tag = env_str("PHASE_ARM_TAG").unwrap_or_default();
    let suffix = if tag.is_empty() { String::new() } else { format!("_{tag}") };
    let csv_path = format!("/tmp/clade_fate_{seed}_{arm}_{deco_arm}_{sign_arm}_{clamp_arm}_{place_arm}_{repro_arm}_{regcap_arm}_{renorm_arm}_{growth_arm}_{fork_min_arm}_{fork_dau_arm}_{grace_arm}_{mate_arm}{suffix}.csv");
    let tree_path = format!("/tmp/clade_tree_{seed}_{arm}_{deco_arm}_{sign_arm}_{clamp_arm}_{place_arm}_{repro_arm}_{regcap_arm}_{renorm_arm}_{growth_arm}_{fork_min_arm}_{fork_dau_arm}_{grace_arm}_{mate_arm}{suffix}.csv");
    let culture_path = format!("/tmp/clade_culture_{seed}_{arm}_{deco_arm}_{sign_arm}_{clamp_arm}_{place_arm}_{repro_arm}_{regcap_arm}_{renorm_arm}_{growth_arm}_{fork_min_arm}_{fork_dau_arm}_{grace_arm}_{mate_arm}{suffix}.csv");
    let life_path = format!("/tmp/clade_life_{seed}_{arm}_{deco_arm}_{sign_arm}_{clamp_arm}_{place_arm}_{repro_arm}_{regcap_arm}_{renorm_arm}_{growth_arm}_{fork_min_arm}_{fork_dau_arm}_{grace_arm}_{mate_arm}{suffix}.csv");
    let mut csv = String::from(
        "t,pop,herb,pred,scav,omn,fish,deco,clades_live,clades_total,events,tokens,bearers,groups,levels,authorities,grass_mean,grass_var,env_mean,env_var,structs,deco_r0,deco_r1,deco_r2,deco_r3,structs_r0,structs_r1,structs_r2,structs_r3,denv_mean,denv_var\n",
    );
    let mut eco = Ecology::new(seed);
    if arm == "envon" { eco.structure_env_on = true; }
    if arm == "envoff" { eco.structure_env_off(); }
    if deco_arm == "decoon" { eco.deco_env_on = true; }
    if deco_arm == "decooff" { eco.deco_resource_env_off(); }
    if sign_arm == "neutral" { eco.token_value_sign_neutral_on(); }
    if let Some(spec) = &clamp_env {
        let parts: Vec<f32> = spec.split(',').filter_map(|v| v.trim().parse().ok()).collect();
        if parts.len() == 2 { eco.deco_env_clamp_on(parts[0], parts[1]); }
    }
    if place_arm == "bvalon" { eco.token_placement_by_value_on(); }
    if repro_arm == "reproold" { eco.repro_threshold_old_off(); }
    if regcap_arm == "regcapon" { eco.deco_region_cap_on(); }
    if regcap_arm == "regcapoff" { eco.deco_region_cap_off(); }
    if renorm_arm == "renormon" { eco.deco_region_cap_renorm_on(); }
    if renorm_arm == "renormoff" { eco.deco_region_cap_renorm_off(); }
    if renorm_arm == "renormdau" { eco.deco_region_cap_renorm_daughters_on(); }
    if floor_arm == "flooron" { eco.deco_region_cap_floor_on(); }
    if floor_arm == "floorsmall" { eco.deco_region_cap_floor_small_on(); }
    if floor_arm == "flooroff" { eco.deco_region_cap_floor_off(); }
    if growth_arm == "growthon" { eco.fork_budget_growth_on(); }
    if growth_arm == "growthoff" { eco.fork_budget_growth_off(); }
    if let Some(v) = &fork_min_env {
        if let Ok(m) = v.trim().parse::<usize>() { eco.fork_budget_min_set(m); }
    }
    if let Some(v) = &fork_dau_env {
        if let Ok(m) = v.trim().parse::<usize>() { eco.fork_budget_daughter_min_set(m); }
    }
    if grace_arm == "graceon" { eco.fork_grace_on(); }
    if grace_arm == "graceoff" { eco.fork_grace_off(); }
    if let Some(v) = &grace_ticks_env {
        if let Ok(t) = v.trim().parse::<u32>() { eco.fork_grace_ticks_set(t); }
    }
    if mate_arm == "mater1" { eco.mate_floor_r1(); }
    if mate_arm == "mater2" { eco.mate_floor_r2(); }
    if mate_arm == "mater3" { eco.mate_floor_r3(); }
    if mate_arm == "mateoff" { eco.mate_floor_off(); }
    let mut last_events = 0usize;
    eprintln!("=== clade-fate seed {seed} ({ticks} ticks, structure_env={arm}, deco_env={deco_arm}, token_sign={sign_arm}, clamp={clamp_arm}, place={place_arm}, repro={repro_arm}, grace={grace_arm}, mate_floor={mate_arm}) ===");
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
            let (denv_mean, denv_var) = deco_env_stats(&eco);
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
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.5},{:.5},{:.5},{:.5},{},{},{},{},{},{},{},{},{:.5},{:.5}\n",
                t, m.information.population, kind("herb"), kind("pred"), kind("scav"),
                kind("omn"), kind("fish"), kind("deco"),
                live_clades, eco.species_clades().len(), last_events,
                m.information.token_count, m.information.token_bearers,
                eco.groups.len(), m.complexity.selection_levels, auth,
                grass_mean, grass_var, env_mean, env_var, eco.structure_registry.len(),
                deco_by_region[0], deco_by_region[1], deco_by_region[2], deco_by_region[3],
                structs_by_region[0], structs_by_region[1], structs_by_region[2], structs_by_region[3],
                denv_mean, denv_var,
            ));
            if t % 50_000 == 0 {
                eprintln!("[fate] t={} {} live_clades={} total={} events={} grass_var={:.5} env_var={:.5} denv_var={:.5} structs={}",
                    t, format_line(&m), live_clades, eco.species_clades().len(), last_events,
                    grass_var, env_var, denv_var, eco.structure_registry.len());
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
    //
    // Phase 59: rows now cover every live (kind, clade), token-less clades
    // included, and carry the Phase 58 death-diagnostic columns: `mean_eff`,
    // `deco_ready` (DECO family: past maturity, off cooldown, energy over the
    // individual threshold — derived kinds included, they carry the same
    // fertility axis) and `min_pair` (nearest same-row neighbour distance, -1
    // below two members). Token columns stay 0 for a clade that never bore a
    // token.
    struct CladeRow {
        members: usize,
        gm_sum: f64,
        pos: usize,
        token_n: usize,
        eff_sum: f64,
        ready: usize,
        xs: Vec<(f64, f64)>,
    }
    let mut culture = String::from(
        "kind,clade_id,clade_name,members_alive,mean_token_grass_mod,frac_grass_positive,structs_with_own_token,mean_x,mean_z,mean_eff,deco_ready,min_pair\n",
    );
    let clade_name: std::collections::HashMap<u32, String> = eco.species_clades().iter()
        .map(|c| (c.id, c.name.clone())).collect();
    // Phase 61 life dump: a clade id alone is ambiguous while the unassigned
    // sentinel 0 is shared by every family, so the id→name map is only used
    // when the record's family matches the row's.
    let clade_family: std::collections::HashMap<u32, u32> = eco.species_clades().iter()
        .map(|c| (c.id, c.kind_id)).collect();
    let mut by_clade: std::collections::HashMap<(u16, u32), CladeRow> =
        std::collections::HashMap::new();
    let mut clade_tokens: std::collections::HashMap<(u16, u32), std::collections::HashSet<u32>> =
        std::collections::HashMap::new();
    for a in eco.living() {
        let e = by_clade.entry((a.kind, a.clade_id)).or_insert(CladeRow {
            members: 0, gm_sum: 0.0, pos: 0, token_n: 0, eff_sum: 0.0, ready: 0, xs: Vec::new(),
        });
        e.members += 1;
        e.eff_sum += a.genome.deco_efficiency;
        e.xs.push((a.x, a.z));
        if eco.root_kind(a.kind) == aether_sim_ecology::ecology::DECO {
            let thr = eco.deco_repro_threshold(a.genome.deco_efficiency);
            if a.age > eco.kind_def(a.kind).mature_age && a.repro <= 0.0
                && a.energy >= a.max_energy * thr {
                e.ready += 1;
            }
        }
        e.token_n += a.known_tokens.len();
        for &tid in &a.known_tokens {
            clade_tokens.entry((a.kind, a.clade_id)).or_default().insert(tid);
            if let Some(tok) = eco.token_registry.get(tid) {
                let (gm, _) = aether_sim_ecology::culture::token_to_env_mod(tok);
                e.gm_sum += gm;
                if gm > 0.0 { e.pos += 1; }
            }
        }
    }
    let mut clades: Vec<_> = by_clade.into_iter().collect();
    clades.sort_by_key(|((kind, id), _)| (*kind, *id));
    for ((kind, id), row) in clades {
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
        let mut min_pair = -1.0f64;
        for i in 0..row.xs.len() {
            for j in (i + 1)..row.xs.len() {
                let dx = row.xs[i].0 - row.xs[j].0;
                let dz = row.xs[i].1 - row.xs[j].1;
                let d = (dx * dx + dz * dz).sqrt();
                if min_pair < 0.0 || d < min_pair { min_pair = d; }
            }
        }
        culture.push_str(&format!(
            "{},{},{},{},{:.5},{:.3},{},{:.1},{:.1},{:.5},{},{:.1}\n",
            kind,
            id,
            clade_name.get(&id).map(|s| s.as_str()).unwrap_or("?"),
            row.members,
            if row.token_n > 0 { row.gm_sum / row.token_n as f64 } else { 0.0 },
            if row.token_n > 0 { row.pos as f64 / row.token_n as f64 } else { 0.0 },
            n_struct,
            if n_struct > 0 { sx / n_struct as f64 } else { 0.0 },
            if n_struct > 0 { sz / n_struct as f64 } else { 0.0 },
            row.eff_sum / row.members as f64,
            row.ready,
            min_pair,
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
    // Phase 61: engine-level per-clade life counters (births / deaths by
    // cause / refusals by gate / mate misses / peak / extinction tick).
    let mut life = String::from(
        "clade_id,clade_name,root_kind,births,deaths,deaths_starved,deaths_aged,deaths_predation,deaths_other,region_cap_blocks,kind_budget_blocks,mate_misses,mate_miss_no_kin,mate_miss_other_region,mate_miss_beyond_radius,mate_miss_filtered,mate_miss_other,mate_miss_young,mate_miss_young_no_kin,mate_miss_young_other_region,mate_miss_young_beyond_radius,mate_miss_young_filtered,mate_miss_young_other,mate_miss_kind_pop_sum,mate_miss_region_pop_sum,mate_miss_ready_dist_sum,mate_miss_ready_dist_min,dispersal_out,deaths_off_region,first_tick,first_birth_tick,last_birth_tick,peak_pop,peak_tick,region_peak_r0,region_peak_r1,region_peak_r2,region_peak_r3,extinction_tick,mean_death_age,mean_death_energy,last_death_energy,life_ticks\n",
    );
    let mut life_rows: Vec<_> = eco.clade_life_stats().to_vec();
    life_rows.sort_by_key(|s| (s.root_kind, s.clade_id));
    for s in &life_rows {
        let fam_name = eco.kind_name(s.root_kind);
        let clade_label = match clade_family.get(&s.clade_id) {
            Some(&kid) if kid != u32::MAX && eco.root_kind(kid as aether_sim_ecology::ecology::SpeciesKind) == s.root_kind =>
                clade_name.get(&s.clade_id).cloned().unwrap_or_else(|| fam_name.to_string()),
            _ => fam_name.to_string(),
        };
        life.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.2},{:.2},{},{},{},{},{},{},{},{},{},{},{},{},{:.2},{:.3},{:.3},{}\n",
            s.clade_id,
            clade_label,
            fam_name,
            s.births, s.deaths, s.deaths_starved, s.deaths_aged, s.deaths_predation,
            s.deaths_other, s.region_cap_blocks, s.kind_budget_blocks, s.mate_misses,
            s.mate_miss_no_kin, s.mate_miss_other_region, s.mate_miss_beyond_radius,
            s.mate_miss_filtered, s.mate_miss_other,
            s.mate_miss_young, s.mate_miss_young_no_kin, s.mate_miss_young_other_region,
            s.mate_miss_young_beyond_radius, s.mate_miss_young_filtered, s.mate_miss_young_other,
            s.mate_miss_kind_pop_sum, s.mate_miss_region_pop_sum,
            s.mate_miss_ready_dist_sum, s.mate_miss_ready_dist_min,
            s.dispersal_out, s.deaths_off_region,
            s.first_tick,
            s.first_birth_tick.map(|v| v.to_string()).unwrap_or_default(),
            s.last_birth_tick.map(|v| v.to_string()).unwrap_or_default(),
            s.peak_pop, s.peak_tick,
            s.region_peak[0], s.region_peak[1], s.region_peak[2], s.region_peak[3],
            s.extinction_tick.map(|v| v.to_string()).unwrap_or_default(),
            if s.deaths > 0 { s.death_age_sum / s.deaths as f64 } else { 0.0 },
            if s.deaths > 0 { s.death_energy_sum / s.deaths as f64 } else { 0.0 },
            s.last_death_energy,
            s.life_ticks(ticks),
        ));
    }
    std::fs::write(&csv_path, csv).unwrap();
    std::fs::write(&tree_path, tree).unwrap();
    std::fs::write(&culture_path, culture).unwrap();
    std::fs::write(&life_path, life).unwrap();
    for s in &life_rows {
        if s.extinction_tick.is_some() {
            eprintln!(
                "  [extinct] clade={} kind={} births={} deaths={} (starved={} aged={} pred={} other={}) region_blocks={} budget_blocks={} mate_misses={} peak={} first_birth={:?} last_birth={:?} death_tick={:?} mean_age={:.1} mean_e={:.2} last_e={:.2}",
                s.clade_id, s.root_kind, s.births, s.deaths, s.deaths_starved, s.deaths_aged,
                s.deaths_predation, s.deaths_other, s.region_cap_blocks, s.kind_budget_blocks,
                s.mate_misses, s.peak_pop, s.first_birth_tick, s.last_birth_tick,
                s.extinction_tick,
                if s.deaths > 0 { s.death_age_sum / s.deaths as f64 } else { 0.0 },
                if s.deaths > 0 { s.death_energy_sum / s.deaths as f64 } else { 0.0 },
                s.last_death_energy,
            );
            eprintln!(
                "  [extinct] mate_miss_split clade={} no_kin={} other_region={} beyond_radius={} filtered={} other={} young={} young_no_kin={} young_other_region={} young_beyond_radius={} young_filtered={} mean_kind_pop={:.1} mean_region_pop={:.1} mean_ready_dist={:.1} min_ready_dist={:.1} dispersal_out={} deaths_off_region={}",
                s.clade_id, s.mate_miss_no_kin, s.mate_miss_other_region,
                s.mate_miss_beyond_radius, s.mate_miss_filtered, s.mate_miss_other,
                s.mate_miss_young, s.mate_miss_young_no_kin, s.mate_miss_young_other_region,
                s.mate_miss_young_beyond_radius, s.mate_miss_young_filtered,
                s.mate_miss_kind_pop_sum as f64 / s.mate_misses.max(1) as f64,
                s.mate_miss_region_pop_sum as f64 / s.mate_misses.max(1) as f64,
                s.mate_miss_ready_dist_sum / s.mate_miss_beyond_radius.max(1) as f64,
                s.mate_miss_ready_dist_min,
                s.dispersal_out, s.deaths_off_region,
            );
        }
    }
    eprintln!(
        "=== clade-fate done: clades={} events={} csv={csv_path} tree={tree_path} culture={culture_path} life={life_path} ===",
        eco.species_clades().len(), last_events
    );
    // Phase 63: end-state kind shares (the growth arm moves slots
    // parent→child, so the DECO family total must stay 80).
    for k in 0..eco.kinds.len() {
        let name = eco.kind_name(k as aether_sim_ecology::ecology::SpeciesKind);
        eprintln!(
            "  [budget] kind={k} name={name} budget={} growth={} parent={:?} family_total={}",
            eco.kind_budget(k as aether_sim_ecology::ecology::SpeciesKind),
            eco.kind_growth_granted(k as aether_sim_ecology::ecology::SpeciesKind),
            eco.kind_def(k as aether_sim_ecology::ecology::SpeciesKind).parent_kind,
            eco.family_budget_total(aether_sim_ecology::ecology::DECO),
        );
    }
    // Phase 65 W1: fork-grace ledger. `skips` counts births the window
    // admitted that the legacy kind-budget gate would have refused; `settled`
    // is the parent→daughter transfer at window close; `unpaid` is the
    // overdraft the parent could not pay (recorded, never minted).
    let mut grace_skips = 0u64;
    let mut grace_windows = 0usize;
    for k in 0..eco.kinds.len() {
        let kind = k as aether_sim_ecology::ecology::SpeciesKind;
        let skips = eco.kind_grace_skips(kind);
        grace_skips += skips;
        if eco.kind_grace_until(kind) != 0 { grace_windows += 1; }
        if skips > 0 || eco.kind_grace_peak_over(kind) > 0
            || eco.kind_grace_settled(kind) > 0 || eco.kind_grace_unpaid(kind) > 0
        {
            eprintln!(
                "  [grace] kind={k} name={} budget={} until={} skips={} peak_over={} settled={} unpaid={}",
                eco.kind_name(kind), eco.kind_budget(kind), eco.kind_grace_until(kind),
                skips, eco.kind_grace_peak_over(kind),
                eco.kind_grace_settled(kind), eco.kind_grace_unpaid(kind),
            );
        }
    }
    eprintln!("  [grace] total_skips={grace_skips} windows_pending={grace_windows}");
    // Phase 66: mate-miss composition (all clades).
    let mut tot = [0u64; 5];
    let mut young = [0u64; 5];
    let mut kin_pop = 0u64;
    let mut reg_pop = 0u64;
    for s in &life_rows {
        let row = [s.mate_miss_no_kin, s.mate_miss_other_region, s.mate_miss_beyond_radius, s.mate_miss_filtered, s.mate_miss_other];
        for (i, &v) in row.iter().enumerate() { tot[i] += v; }
        let yrow = [s.mate_miss_young_no_kin, s.mate_miss_young_other_region, s.mate_miss_young_beyond_radius, s.mate_miss_young_filtered, s.mate_miss_young_other];
        for (i, &v) in yrow.iter().enumerate() { young[i] += v; }
        kin_pop += s.mate_miss_kind_pop_sum;
        reg_pop += s.mate_miss_region_pop_sum;
    }
    let n = tot.iter().sum::<u64>();
    let ny = young.iter().sum::<u64>();
    // Phase 66 P3a / Phase 67 P3: the arm's scope is derived DECO KINDS.
    // `clade_id != 0` is not a valid proxy: on the Phase 67 pool the DECO root
    // clade materialized as id 1 (the pred root took 0), so the proxy booked
    // the root's legacy-radius misses into the "daughter" total. A clade is in
    // scope iff its kind has a parent kind; the root's misses stay out.
    let is_derived_deco = |s: &aether_sim_ecology::ecology::CladeLifeStats| {
        s.root_kind == aether_sim_ecology::ecology::DECO
            && eco.species_clades().iter().find(|c| c.id == s.clade_id)
                .map(|c| c.kind_id != u32::MAX
                    && eco.kind_def(c.kind_id as aether_sim_ecology::ecology::SpeciesKind)
                        .parent_kind.is_some())
                .unwrap_or(false)
    };
    let dau_deco: u64 = life_rows.iter()
        .filter(|s| is_derived_deco(s))
        .map(|s| s.mate_misses)
        .sum();
    let dau_deco_young: u64 = life_rows.iter()
        .filter(|s| is_derived_deco(s))
        .map(|s| s.mate_miss_young)
        .sum();
    eprintln!(
        "  [mate] total={} no_kin={} other_region={} beyond_radius={} filtered={} other={} mean_kind_pop={:.1} mean_region_pop={:.1}",
        n, tot[0], tot[1], tot[2], tot[3], tot[4],
        kin_pop as f64 / n.max(1) as f64,
        reg_pop as f64 / n.max(1) as f64,
    );
    eprintln!(
        "  [mate] young={} no_kin={} other_region={} beyond_radius={} filtered={} other={}",
        ny, young[0], young[1], young[2], young[3], young[4],
    );
    eprintln!(
        "  [mate] daughter_deco={} daughter_deco_young={}",
        dau_deco, dau_deco_young,
    );
    // Phase 67 P3: miss/clade-tick (the Phase 66 §11.1 fix). The denominator
    // is the same population the numerator comes from: derived DECO clades.
    let dau_rows: Vec<&aether_sim_ecology::ecology::CladeLifeStats> = life_rows.iter()
        .filter(|s| is_derived_deco(s))
        .collect();
    let dau_ticks: u64 = dau_rows.iter().map(|s| s.life_ticks(ticks)).sum();
    let dau_births: u64 = dau_rows.iter().map(|s| s.births).sum();
    let dau_buckets: [u64; 5] = [
        dau_rows.iter().map(|s| s.mate_miss_no_kin).sum(),
        dau_rows.iter().map(|s| s.mate_miss_other_region).sum(),
        dau_rows.iter().map(|s| s.mate_miss_beyond_radius).sum(),
        dau_rows.iter().map(|s| s.mate_miss_filtered).sum(),
        dau_rows.iter().map(|s| s.mate_miss_other).sum(),
    ];
    let dau_rate = |n: u64| if dau_ticks > 0 { n as f64 * 1000.0 / dau_ticks as f64 } else { 0.0 };
    eprintln!(
        "  [mate_rate] daughter_deco misses={} clade_ticks={} misses_per_ktick={:.1} births={} miss_per_birth={:.3}",
        dau_deco, dau_ticks, dau_rate(dau_deco), dau_births,
        if dau_births > 0 { dau_deco as f64 / dau_births as f64 } else { 0.0 },
    );
    eprintln!(
        "  [mate_rate] daughter_deco buckets_per_ktick no_kin={:.2} other_region={:.1} beyond_radius={:.1} filtered={:.1} other={:.1}",
        dau_rate(dau_buckets[0]), dau_rate(dau_buckets[1]), dau_rate(dau_buckets[2]),
        dau_rate(dau_buckets[3]), dau_rate(dau_buckets[4]),
    );
    eprintln!(
        "  [radius] arm={} boost_searches={} only_finds={} only_find_dist_mean={:.2}",
        eco.mate_radius_arm(), eco.radius_boost_searches(), eco.radius_only_finds(),
        eco.radius_only_find_dist_mean(),
    );
}
