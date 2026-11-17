//! Phase 51 acceptance tests — niche resource partition.
//!
//! niche_offset doubles as a resource-preference coordinate: beyond
//! NICHE_PARTITION_BAND a clade's members forage more efficiently in cells
//! whose local resource mix (grass vs mineral relative richness) matches
//! their side of the axis. Coverage:
//!
//! 1. `partition_semantics_double_run_roundtrip_and_bite_gain` — the full
//!    stack on the real first-fork world (seed 20261102, deco-A forks at
//!    t=34680 with |niche| > band):
//!    * multiplier semantics: neutral inside the band / for the root /
//!      unassigned ids, direction follows the local gradient, gain bounded
//!      by NICHE_PREF_GAIN, off switch short-circuits to 1.0;
//!    * telemetry `niche_partition_share` reports the banded population
//!      and the shares sum to 1;
//!    * v18 save/restore roundtrip after the fork continues bit-exactly;
//!    * behavioural self-controlled bite gain: the SAME staged agent bites
//!      the SAME staged carcass with only the cell's mineral load changed
//!      (via save/restore + inject_mineral) — a banded clade member gains
//!      more in its preferred mix, a neutral root member is bit-identical;
//!    * double run of the partition-ON path is bit-identical.
//! 2. `niche_partition_off_is_bit_exact_phase50_anchor` — the A/B arm:
//!    partition OFF reproduces the Phase 50 trajectory bit-exactly (pinned
//!    at the pre-fork t=30000 and post-fork t=60000 anchors captured on the
//!    pre-Phase-51 binary), partition ON diverges after the first banded
//!    fork while staying bit-exact with Phase 50 before it.
//! 3. `v17_archive_loads_with_partition_on_and_continues_identically` — a
//!    pre-v18 archive (no NichePartitionOn key, Version=17) loads with the
//!    default-true flag and continues bit-identically with the same world
//!    saved as v18 (the serde-default v17 compatibility contract).

use aether_sim_ecology::ecology::{
    Ecology, Carcass, DECO, GRID_N, GRID_SPAN, NICHE_PARTITION_BAND, NICHE_PREF_GAIN,
};

const SEED: u32 = 20261102;
/// The first banded clade (deco-A) forks at t=34680 on this seed; 40000 gives
/// it a comfortable survival margin (Phase 50: alive until t=787500).
const FORK_DEPTH: u32 = 40_000;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

/// Centre world coordinates of cell `idx` (mirrors `Ecology::cell_coord`).
fn cell_center(idx: usize) -> (f64, f64) {
    let half = GRID_SPAN / 2.0;
    let cell = GRID_SPAN / GRID_N as f64;
    let i = idx % GRID_N as usize;
    let j = idx / GRID_N as usize;
    (-half + (i as f64 + 0.5) * cell, -half + (j as f64 + 0.5) * cell)
}

/// A grass-dominated cell (mineral loaded to 0 by the caller when needed).
fn grass_rich_cell(eco: &Ecology) -> usize {
    let g = eco.grass();
    let mut best = 0usize;
    for (i, &v) in g.iter().enumerate() {
        if v > g[best] { best = i; }
    }
    assert!(g[best] > 0.3, "no grass-rich cell found (max grass {})", g[best]);
    best
}

/// A bare (mineral-dominated) cell: near-zero grass, so a mineral load makes
/// the local gradient unambiguously negative.
fn bare_cell(eco: &Ecology) -> usize {
    let g = eco.grass();
    let mut best = 0usize;
    for (i, &v) in g.iter().enumerate() {
        if v < g[best] { best = i; }
    }
    assert!(g[best] < 0.1, "no bare cell found (min grass {})", g[best]);
    best
}

/// Stage `idxs` (banded, control) as feeding DECOs at `cell`: pinned inside
/// the cell (two agents 3 units apart so each owns its own carcass), state
/// "eat", bite timer expired, low starting energy and a small carcass.
///
/// The carcass is deliberately SMALL (0.5 energy): a full DECO bite is
/// 2.0 × 62 ≈ 124 energy against a 100-energy cap, so a full bite always
/// saturates and would swallow the preference multiplier. A small bite is
/// the regime where the ±NICHE_PREF_GAIN swing is actually visible in the
/// energy budget (and it is the regime a hungry decomposer really lives in).
fn stage_bite(eco: &mut Ecology, idxs: [usize; 2], cx: f64, cz: f64) {
    for (k, &i) in idxs.iter().enumerate() {
        let dx = if k == 0 { -1.5 } else { 1.5 };
        let a = &mut eco.agents[i];
        a.x = cx + dx; a.z = cz;
        a.prev_x = cx + dx; a.prev_z = cz;
        a.home_x = cx; a.home_z = cz;
        a.target_x = cx + dx; a.target_z = cz;
        a.state = "eat";
        a.bite = 0.0;
        a.speed = 0.0;
        a.desired_speed = 0.0;
        a.think = 100.0;
        a.repro = 50.0;
        a.energy = 5.0;
    }
    eco.carcasses.clear();
    eco.carcasses.push(Carcass { x: cx - 1.5, z: cz, energy: 1.0, age: 0.0, occupied_by: None });
    eco.carcasses.push(Carcass { x: cx + 1.5, z: cz, energy: 1.0, age: 0.0, occupied_by: None });
}

/// One staged bite from the world state `eco` (unmodified): restore a copy,
/// stage the two agents, set the cell's mineral load to `mineral`, run one
/// tick, and return both agents' energy deltas.
fn staged_bite(eco: &Ecology, idxs: [usize; 2], cell: usize, mineral: f32, t: u32) -> [f64; 2] {
    let mut w = Ecology::from_save_state(eco.to_save_state());
    let (cx, cz) = cell_center(cell);
    stage_bite(&mut w, idxs, cx, cz);
    w.inject_mineral(cell, mineral);
    let e0 = w.agents[idxs[0]].energy;
    let e1 = w.agents[idxs[1]].energy;
    w.update(0.1, daylight(t + 1));
    [w.agents[idxs[0]].energy - e0, w.agents[idxs[1]].energy - e1]
}

#[test]
fn partition_semantics_double_run_roundtrip_and_bite_gain() {
    let mut a = Ecology::new(SEED);
    for t in 1..=FORK_DEPTH {
        a.update(0.1, daylight(t));
    }

    // --- a living banded clade must exist past the first fork ---
    let banded = a.species_clades().iter()
        .filter(|c| c.death_tick.is_none() && c.niche_offset.abs() > NICHE_PARTITION_BAND)
        .max_by(|x, y| x.niche_offset.abs().partial_cmp(&y.niche_offset.abs()).unwrap())
        .expect("expected a living banded clade by t=40000 (deco-A forks at 34680)")
        .clone();
    let root = a.species_clades().iter()
        .find(|c| c.parent_id.is_none() && c.death_tick.is_none())
        .expect("root clade missing at t=40000")
        .clone();
    assert_eq!(root.niche_offset, 0.0, "the root is pinned at niche 0");
    let grass_preferring = banded.niche_offset > NICHE_PARTITION_BAND;
    eprintln!(
        "banded clade {} niche={:+.3} ({}) / root {} niche={:+.3}",
        banded.name, banded.niche_offset,
        if grass_preferring { "grass-preferring" } else { "mineral-preferring" },
        root.name, root.niche_offset,
    );

    // --- telemetry: the share census reports the banded population ---
    let share = a.niche_partition_share;
    let sum = share.mineral + share.neutral + share.grass;
    assert!((sum - 1.0).abs() < 1e-9, "band shares must sum to 1, got {sum}");
    assert!(share.partitioned > 0.0, "telemetry must report a nonzero banded share");
    if grass_preferring {
        assert!(share.grass > 0.0, "grass-preferring clade reported as share.grass");
    } else {
        assert!(share.mineral > 0.0, "mineral-preferring clade reported as share.mineral");
    }

    // --- multiplier semantics: a grass-dense cell vs a bare (mineral-loaded)
    //     cell — the two extremes of the local resource gradient ---
    let grass_cell = grass_rich_cell(&a);
    let bare_cell = bare_cell(&a);
    a.inject_mineral(grass_cell, 0.0);
    a.inject_mineral(bare_cell, 1.0);
    let (g_val, m_val) = (a.grass()[grass_cell] as f64, a.mineral_field()[grass_cell] as f64);
    assert!(g_val > 0.3 && m_val < 0.1, "expected a grass-dominated cell, g={g_val} m={m_val}");
    assert!(a.grass()[bare_cell] < 0.1 && a.mineral_field()[bare_cell] > 0.9,
        "expected a mineral-dominated cell, g={} m={}",
        a.grass()[bare_cell], a.mineral_field()[bare_cell]);
    let m_banded_grass = a.niche_pref_mult(banded.id, grass_cell);
    let m_banded_mineral = a.niche_pref_mult(banded.id, bare_cell);
    // Root / unassigned / off-mechanism ids are neutral.
    assert_eq!(a.niche_pref_mult(root.id, grass_cell), 1.0, "root clade must be neutral everywhere");
    assert_eq!(a.niche_pref_mult(root.id, bare_cell), 1.0, "root clade must be neutral everywhere");
    assert_eq!(a.niche_pref_mult(u32::MAX, grass_cell), 1.0, "unassigned id must be neutral");
    assert_eq!(a.niche_pref_mult(banded.id, a.grass().len()), 1.0, "out-of-range cell must be neutral");
    // Banded clade: boosted in the matching mix, damped in the opposite one.
    let strength = ((banded.niche_offset.abs() - NICHE_PARTITION_BAND)
        / (1.0 - NICHE_PARTITION_BAND)).min(1.0);
    if grass_preferring {
        assert!(m_banded_grass > 1.0, "grass-preferring clade must be boosted in a grass-rich cell (m={m_banded_grass})");
        assert!(m_banded_mineral < 1.0, "grass-preferring clade must be damped in a mineral-rich cell (m={m_banded_mineral})");
    } else {
        assert!(m_banded_grass < 1.0, "mineral-preferring clade must be damped in a grass-rich cell (m={m_banded_grass})");
        assert!(m_banded_mineral > 1.0, "mineral-preferring clade must be boosted in a mineral-rich cell (m={m_banded_mineral})");
    }
    // Both swings respect the strength-scaled bound.
    for m in [m_banded_grass, m_banded_mineral] {
        assert!((m - 1.0).abs() <= NICHE_PREF_GAIN * strength + 1e-9,
            "multiplier {m} exceeds the strength-scaled bound");
    }
    // Off switch: everything collapses to exactly 1.0.
    a.niche_partition_off();
    assert_eq!(a.niche_pref_mult(banded.id, grass_cell), 1.0, "off switch must short-circuit to 1.0");
    assert_eq!(a.niche_pref_at(banded.id, cell_center(grass_cell).0, cell_center(grass_cell).1), 1.0);
    a.niche_partition = true; // restore the A-arm default for the continuation below

    // --- v18 save/restore roundtrip after the fork: bit-exact continuation ---
    let saved = a.to_save_state();
    let mut restored = Ecology::from_save_state(saved);
    assert!(restored.niche_partition, "the partition flag must ride SaveState v18");
    assert_eq!(
        restored.niche_pref_mult(banded.id, grass_cell),
        a.niche_pref_mult(banded.id, grass_cell),
        "a restored world must see its niches immediately (cache rebuilt on load)",
    );
    let lock_to = FORK_DEPTH + 1_500;
    for t in (FORK_DEPTH + 1)..=lock_to {
        a.update(0.1, daylight(t));
        restored.update(0.1, daylight(t));
    }
    assert_eq!(a.checksum(), restored.checksum(),
        "roundtrip continuation diverged at t={lock_to}");

    // --- behavioural self-controlled bite gain (acceptance: preference is
    //     measurable where it matters — the actual foraging energy path) ---
    let banded_agent = a.agents.iter().position(|x| {
        x.alive && x.clade_id == banded.id && a.root_kind(x.kind) == DECO
            && x.brain_authority != aether_sim_ecology::ecology::BRAIN_AUTHORITY_BRAIN
    }).expect("no living banded member eligible for the staged bite");
    let control_agent = a.agents.iter().position(|x| {
        x.alive && x.clade_id == root.id && a.root_kind(x.kind) == DECO
            && x.brain_authority != aether_sim_ecology::ecology::BRAIN_AUTHORITY_BRAIN
    }).expect("no living root member eligible for the staged bite");
    let idxs = [banded_agent, control_agent];
    let t = a.tick_count();
    // Grass-dense cell (mineral 0) vs a bare, mineral-loaded cell: the ONLY
    // difference between the two staged bites is the resource mix.
    let delta_grass = staged_bite(&a, idxs, grass_cell, 0.0, t);
    let delta_mineral = staged_bite(&a, idxs, bare_cell, 1.0, t);
    // Control (root, niche 0): the multiplier is exactly 1.0 in both mixes —
    // the two staged bites must be bit-identical.
    assert_eq!(delta_grass[1], delta_mineral[1],
        "neutral root member: staged bites must be bit-identical ({} vs {})",
        delta_grass[1], delta_mineral[1]);
    // Banded member: gains more in its preferred mix (the swing is
    // 0.5 × 62 × eff × 2 × NICHE_PREF_GAIN × strength ≈ 2-5 energy units;
    // demand > 1).
    if grass_preferring {
        assert!(delta_grass[0] > delta_mineral[0] + 1.0,
            "grass-preferring member must gain more on grass-rich ground ({} vs {})",
            delta_grass[0], delta_mineral[0]);
    } else {
        assert!(delta_mineral[0] > delta_grass[0] + 1.0,
            "mineral-preferring member must gain more on mineral-rich ground ({} vs {})",
            delta_mineral[0], delta_grass[0]);
    }
    eprintln!(
        "staged bite deltas (banded, control): grass-rich {:+.3}/{:+.3} mineral-rich {:+.3}/{:+.3}",
        delta_grass[0], delta_grass[1], delta_mineral[0], delta_mineral[1],
    );

    // --- double run of the partition-ON path: a second independent world
    //     replays the post-fork stretch in lockstep ---
    let mut c = Ecology::from_save_state(a.to_save_state());
    for t in (FORK_DEPTH + 1)..=(FORK_DEPTH + 300) {
        a.update(0.1, daylight(t));
        c.update(0.1, daylight(t));
    }
    assert_eq!(a.checksum(), c.checksum(), "double run diverged on the partition-ON path");
}

#[test]
fn niche_partition_off_is_bit_exact_phase50_anchor() {
    // B arm (Phase 50 behaviour): pre-fork and post-fork anchors were captured
    // on the pre-Phase-51 binary (research/phase51-report.md, baseline run).
    let mut off = Ecology::new(SEED);
    off.niche_partition_off();
    for t in 1..=60_000u32 {
        off.update(0.1, daylight(t));
        if t == 30_000 {
            assert_eq!(off.checksum(), "0x516d99450971ff76",
                "pre-fork anchor: partition OFF must be bit-exact with Phase 50 at t=30000");
        }
    }
    assert_eq!(off.checksum(), "0x0ab6c03f2c29eec5",
        "post-fork deep anchor: partition OFF must be bit-exact with Phase 50 at t=60000");

    // A arm (default ON): a bit-exact no-op BEFORE the first banded fork,
    // a diverging trajectory after it (deco-A forks at t=34680 banded; the
    // first visible energy difference lands between t=40000 and t=60000 —
    // full DECO bites saturate against the 100-energy cap, so only the
    // partial/low-efficiency bites carry the preference).
    let mut on = Ecology::new(SEED);
    for t in 1..=60_000u32 {
        on.update(0.1, daylight(t));
        if t == 30_000 {
            assert_eq!(on.checksum(), "0x516d99450971ff76",
                "pre-fork: partition ON must be bit-exact with Phase 50 before any banded clade exists");
        }
    }
    assert_ne!(on.checksum(), "0x0ab6c03f2c29eec5",
        "post-fork: partition ON must diverge from the Phase 50 trajectory by t=60000");
}

/// Phase 51: a pre-v18 archive (no `NichePartitionOn` key, Version=17) loads
/// with the default-true partition flag and continues bit-identically with
/// the same world saved as v18 — the serde-default v17 compatibility
/// contract.
#[test]
fn v17_archive_loads_with_partition_on_and_continues_identically() {
    let mut eco = Ecology::new(SEED);
    for t in 1..=600u32 {
        eco.update(0.1, daylight(t));
    }
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let obj = v.as_object_mut().unwrap();
    obj.remove("NichePartitionOn");
    obj.insert("Version".into(), serde_json::json!(17));
    let v17: aether_sim_ecology::save_state::SaveState =
        serde_json::from_value(v).expect("v17 archive must load via serde defaults");
    assert_eq!(v17.version, 17);
    let mut restored_v17 = Ecology::from_save_state(v17);
    assert!(restored_v17.niche_partition, "pre-v18 archives land on partition ON");
    let mut restored_v18 = Ecology::from_save_state(
        serde_json::from_str::<aether_sim_ecology::save_state::SaveState>(
            &serde_json::to_string(&eco.to_save_state()).unwrap()).unwrap());
    for t in 601..=900u32 {
        restored_v17.update(0.1, daylight(t));
        restored_v18.update(0.1, daylight(t));
    }
    assert_eq!(restored_v17.checksum(), restored_v18.checksum(),
        "v17 and v18 restores of the same world must continue identically");
}
