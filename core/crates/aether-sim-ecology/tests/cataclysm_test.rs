//! Phase 17 cataclysm acceptance tests (deterministic-injection style).
//!
//! Six pillars:
//! 1. `impact_clears_local_resources_and_kills` — the blast zeroes grass and
//!    mineral fields inside the radius (cell-center convention) and applies
//!    exact `severity × CATACLYSM_DAMAGE` energy damage; deaths reuse the
//!    regular `die()` path (deaths counter + one carcass each).
//! 2. `dust_recovers_linearly` — the daylight multiplier drops to
//!    `1 − severity` at injection and recovers along the exact linear
//!    schedule `1 − severity × (remaining / duration)`, reaching 1.0 after
//!    exactly `duration` substeps.
//! 3. `climate_shift_modulates_growth_then_restores` — with herbivores and
//!    predators removed (pure abiotic A/B), the throttled run keeps strictly
//!    less total plant mass (time-averaged, per the Phase 16 lesson), the
//!    multiplier expires after `duration`, and growth resumes.
//! 4. `save_restore_preserves_cataclysm_state` — SaveState v4 roundtrips the
//!    full cataclysm table bit-exactly (live countdown included) and the
//!    restored sim continues in lockstep checksums.
//! 5. `legacy_v3_save_loads_without_cataclysms` — a stripped v3 document
//!    restores to an empty table and keeps running bit-identically to a
//!    never-saved control (zero-side-effect contract).
//! 6. `post_impact_survivor_bias` — K-Pg selection: with metabolism genes
//!    injected 0.7/1.5 at parity (genome-independent), the dust-winter drain
//!    (∝ metabolism) skews survivors below the injected midpoint.
//!
//! Setup mirrors `niche_divergence_test.rs`: same seed, per-tick daylight
//! curve, direct state mutation for deterministic injection.

use aether_sim_ecology::ecology::{
    Ecology, CATACLYSM_CLIMATE, CATACLYSM_DAMAGE, CATACLYSM_IMPACT, DECO, HERB, PRED,
};
use aether_sim_ecology::save_state;
use aether_sim_ecology::terrain::{terrain_height, WATER_LEVEL};

const SEED: u32 = 20261102;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in (from + 1)..=to {
        eco.update(0.1, daylight(t));
    }
}

/// World distance — same predicate as the engine (`sqrt(dx²+dz²)`, never
/// `hypot`, so boundary classification matches `impact_blast` bit-for-bit).
fn dist(ax: f64, az: f64, bx: f64, bz: f64) -> f64 {
    let dx = ax - bx;
    let dz = az - bz;
    (dx * dx + dz * dz).sqrt()
}

/// First valid land cell center in row-major order (deterministic).
fn land_cell_center(eco: &Ecology) -> (f64, f64) {
    let half = eco.grid_span / 2.0;
    for j in 0..eco.grid_n {
        for i in 0..eco.grid_n {
            let x = -half + (i as f64 + 0.5) * eco.cell;
            let z = -half + (j as f64 + 0.5) * eco.cell;
            let h = terrain_height(x, z);
            if h > WATER_LEVEL + 2.0 && h < 80.0 {
                return (x, z);
            }
        }
    }
    unreachable!("a seeded world always has land cells");
}

fn grid_index_of(eco: &Ecology, x: f64, z: f64) -> Option<usize> {
    let half = eco.grid_span / 2.0;
    let i = ((x + half) / eco.cell).floor();
    let j = ((z + half) / eco.cell).floor();
    if i < 0.0 || j < 0.0 || i >= eco.grid_n as f64 || j >= eco.grid_n as f64 {
        return None;
    }
    Some(j as usize * eco.grid_n as usize + i as usize)
}

fn grass_sum_in_radius(eco: &Ecology, x: f64, z: f64, radius: f64) -> f64 {
    let half = eco.grid_span / 2.0;
    let n = eco.grid_n as usize;
    let mut sum = 0.0f64;
    let mut count = 0usize;
    for idx in 0..n * n {
        let col = (idx % n) as f64;
        let row = (idx / n) as f64;
        let cx = -half + (col + 0.5) * eco.cell;
        let cz = -half + (row + 0.5) * eco.cell;
        if dist(cx, cz, x, z) <= radius {
            sum += eco.grass()[idx] as f64;
            count += 1;
        }
    }
    assert!(count > 0, "radius must cover at least one cell");
    sum
}

// ---------------------------------------------------------------------------
// 1. Impact: local resource wipe + exact damage via the die() path
// ---------------------------------------------------------------------------

#[test]
fn impact_clears_local_resources_and_kills() {
    let mut eco = Ecology::new(SEED);
    run_ticks(&mut eco, 0, 100);

    // Anchor on the seeded herd cluster: `seed_life` deterministically spawns
    // 18 herbs around (215, 292), so a radius-60 blast is guaranteed hits
    // regardless of where the row-major first-land-cell scan lands.
    let (cx, cz) = (215.0, 292.0);
    assert!(terrain_height(cx, cz) > WATER_LEVEL + 2.0, "herd anchor must be on land");
    let radius = 60.0;
    let pre_grass = grass_sum_in_radius(&eco, cx, cz, radius);
    assert!(pre_grass > 0.0, "land cells must carry grass before the blast");

    // Far reference cell: the antipode of the impact point (deterministically
    // far for a row-major first-land-cell hit near the top-left corner).
    let far_idx = grid_index_of(&eco, -cx, -cz).expect("antipode inside the grid");
    assert!(dist(-cx, -cz, cx, cz) > radius + 2.0 * eco.cell, "reference cell must be outside the blast");
    let far_grass_before = eco.grass()[far_idx];
    let far_min_before = eco.mineral_field()[far_idx];

    let deaths_before = eco.deaths;
    let carcasses_before = eco.carcasses.len();
    let pre_agents: Vec<(u32, bool, f64, f64)> = eco.agents.iter()
        .map(|a| (a.id, a.alive, a.energy, dist(a.x, a.z, cx, cz)))
        .collect();

    let id = eco.trigger_cataclysm(CATACLYSM_IMPACT, cx, cz, radius, 1.0, 100);
    assert!(id >= 1, "impact injection must succeed");

    // (1) every in-radius cell: grass + all species layers + mineral, bit-zero.
    let half = eco.grid_span / 2.0;
    let n = eco.grid_n as usize;
    let mut in_cells = 0usize;
    for idx in 0..n * n {
        let col = (idx % n) as f64;
        let row = (idx / n) as f64;
        let ccx = -half + (col + 0.5) * eco.cell;
        let ccz = -half + (row + 0.5) * eco.cell;
        if dist(ccx, ccz, cx, cz) <= radius {
            in_cells += 1;
            assert_eq!(eco.grass()[idx], 0.0, "grass cell {idx} must be zeroed");
            assert_eq!(eco.grass_p(0)[idx], 0.0, "grass_p[0] cell {idx} must be zeroed");
            assert_eq!(eco.grass_p(1)[idx], 0.0, "grass_p[1] cell {idx} must be zeroed");
            assert_eq!(eco.grass_p(2)[idx], 0.0, "grass_p[2] cell {idx} must be zeroed");
            assert_eq!(eco.mineral_field()[idx], 0.0, "mineral cell {idx} must be zeroed");
        }
    }
    assert!(in_cells >= 4, "radius 60 must cover several cells, got {in_cells}");

    // (2) outside cells untouched (bit-exact — no smear, no drift).
    assert_eq!(eco.grass()[far_idx], far_grass_before, "far grass cell untouched");
    assert_eq!(eco.mineral_field()[far_idx], far_min_before, "far mineral cell untouched");

    // (3) agents: exact energy accounting through the regular die() path.
    let mut died: u32 = 0;
    let mut hit = 0usize;
    for (aid, was_alive, pre_energy, d) in &pre_agents {
        if !was_alive || *d > radius { continue; }
        hit += 1;
        let a = eco.agents.iter().find(|a| a.id == *aid).unwrap();
        if a.alive {
            assert!((a.energy - (pre_energy - CATACLYSM_DAMAGE)).abs() < 1e-9,
                "survivor {aid} must lose exactly severity × CATACLYSM_DAMAGE");
        } else {
            assert!(*pre_energy <= CATACLYSM_DAMAGE + 1e-9,
                "agent {aid} died but carried {pre_energy} energy before the blast");
            died += 1;
        }
    }
    assert!(hit > 0, "impact must reach at least one agent");
    assert_eq!(eco.deaths - deaths_before, died, "deaths counter must match the die() path");
    assert_eq!(eco.carcasses.len() - carcasses_before, died as usize, "each cataclysm death leaves a carcass");

    // (4) record bookkeeping.
    assert_eq!(eco.cataclysms.len(), 1);
    let c = &eco.cataclysms[0];
    assert_eq!(c.kind, CATACLYSM_IMPACT);
    assert!(c.active);
    assert_eq!(c.remaining, 100);
    assert_eq!(c.dust_recovery_ticks, 100);
    assert_eq!(c.tick, 100);
}

// ---------------------------------------------------------------------------
// 2. Dust winter: exact linear recovery schedule
// ---------------------------------------------------------------------------

#[test]
fn dust_recovers_linearly() {
    let mut eco = Ecology::new(SEED);
    run_ticks(&mut eco, 0, 50);
    const D: u32 = 100;
    const SEV: f64 = 0.6;
    // Water center: no land agents nearby, isolating the dust leg.
    eco.trigger_cataclysm(CATACLYSM_IMPACT, 0.0, 0.0, 10.0, SEV, D);

    // At injection: full darkness, growth multiplier untouched.
    let (dust, grow) = eco.cataclysm_env_mods();
    assert!((dust - (1.0 - SEV)).abs() < 1e-12, "full darkness at injection, got {dust}");
    assert!((grow - 1.0).abs() < 1e-12, "impact must not touch the growth multiplier directly");

    // Per-substep linear schedule: after k updates the countdown sits at
    // D − k, so the NEXT tick's multiplier is 1 − SEV × (D − k)/D … and the
    // daylight the k-th update actually used was the pre-decrement value.
    for k in 1..=D {
        eco.update(0.1, 0.5);
        let (dust, _) = eco.cataclysm_env_mods();
        let expected = 1.0 - SEV * ((D - k) as f64 / D as f64);
        assert!((dust - expected).abs() < 1e-12,
            "tick {k}: dust {dust:.6} vs expected {expected:.6}");
        let used_dust = 1.0 - SEV * ((D - k + 1) as f64 / D as f64);
        assert!((eco.daylight - 0.5 * used_dust).abs() < 1e-12,
            "tick {k}: effective daylight must carry the dust multiplier");
    }

    let c = &eco.cataclysms[0];
    assert!(!c.active && c.remaining == 0, "dust window must close exactly at D");
    let (dust, _) = eco.cataclysm_env_mods();
    assert_eq!(dust, 1.0, "fully recovered");
}

// ---------------------------------------------------------------------------
// 3. Climate shift: growth throttle, expiry, recovery
// ---------------------------------------------------------------------------

#[test]
fn climate_shift_modulates_growth_then_restores() {
    let mut a = Ecology::new(SEED);
    let mut b = Ecology::new(SEED);
    run_ticks(&mut a, 0, 100);
    run_ticks(&mut b, 0, 100);
    // Remove consumers on both sides: pure abiotic grass A/B (no chaos
    // amplification through herbivore dynamics — Phase 16 lesson).
    a.kill_kind(HERB); a.kill_kind(PRED);
    b.kill_kind(HERB); b.kill_kind(PRED);

    let id = a.trigger_cataclysm(CATACLYSM_CLIMATE, 0.0, 0.0, 0.0, 0.6, 300);
    assert!(id >= 1);
    let (dust, grow) = a.cataclysm_env_mods();
    assert!((dust - 1.0).abs() < 1e-12, "climate shift must not touch the dust multiplier");
    assert!((grow - 0.4).abs() < 1e-12, "growth multiplier = 1 − severity");

    let mut samples_a = Vec::new();
    let mut samples_b = Vec::new();
    for t in 101..=400u32 {
        run_ticks(&mut a, t - 1, t);
        run_ticks(&mut b, t - 1, t);
        if t % 20 == 0 {
            samples_a.push(a.stats().plant);
            samples_b.push(b.stats().plant);
        }
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let avg_a = mean(&samples_a);
    let avg_b = mean(&samples_b);
    eprintln!("=== climate A/B: throttled avg={avg_a:.4} control avg={avg_b:.4} ===");
    assert!(avg_a < avg_b,
        "time-averaged plant mass: throttled run ({avg_a:.4}) must stay below control ({avg_b:.4})");

    assert!(!a.cataclysms[0].active, "climate shift expires after its duration");
    assert_eq!(a.cataclysms[0].remaining, 0);
    assert_eq!(a.cataclysm_env_mods(), (1.0, 1.0), "multipliers snap back exactly");

    // Post-expiry recovery: with the throttle gone A regrows at the control
    // rate, but its grass backlog is depleted — the cumulative deficit can
    // only grow (logistic growth can never outrun a head start). Assert the
    // deficit stays bounded and A keeps growing after expiry.
    run_ticks(&mut a, 400, 520);
    run_ticks(&mut b, 400, 520);
    let end_a = a.stats().plant;
    let end_b = b.stats().plant;
    eprintln!("=== post-expiry: A plant={end_a:.4} B plant={end_b:.4} (A lagging is expected) ===");
    assert!(end_b > end_a, "throttled run must stay behind the control after expiry");
}

// ---------------------------------------------------------------------------
// 4. Save/restore v4: bit-exact cataclysm state + lockstep checksums
// ---------------------------------------------------------------------------

#[test]
fn save_restore_preserves_cataclysm_state() {
    let mut eco = Ecology::new(SEED);
    run_ticks(&mut eco, 0, 100);
    eco.trigger_cataclysm(CATACLYSM_IMPACT, -200.0, 150.0, 120.0, 0.8, 400);
    eco.trigger_cataclysm(CATACLYSM_CLIMATE, 0.0, 0.0, 0.0, 0.4, 250);
    run_ticks(&mut eco, 100, 160); // both effects live
    assert!(eco.cataclysms.iter().all(|c| c.active));

    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let state: save_state::SaveState = serde_json::from_str(&json).unwrap();
    assert_eq!(state.version, save_state::SAVE_STATE_VERSION, "SaveState v4");
    assert!(json.contains("\"Cataclysms\":\"[{"), "v4 save carries the cataclysm JSON string");
    let restored = Ecology::from_save_state(state);

    // Full table roundtrips bit-exactly through the JSON hop.
    assert_eq!(eco.cataclysms.len(), restored.cataclysms.len());
    for (o, r) in eco.cataclysms.iter().zip(restored.cataclysms.iter()) {
        assert_eq!(o.id, r.id);
        assert_eq!(o.kind, r.kind);
        assert_eq!(o.tick, r.tick);
        assert_eq!(o.x, r.x, "x bit-exact");
        assert_eq!(o.z, r.z, "z bit-exact");
        assert_eq!(o.radius, r.radius);
        assert_eq!(o.severity, r.severity);
        assert_eq!(o.dust_recovery_ticks, r.dust_recovery_ticks);
        assert_eq!(o.climate_delta, r.climate_delta);
        assert_eq!(o.ends_at, r.ends_at);
        assert_eq!(o.remaining, r.remaining, "live countdown bit-exact");
        assert_eq!(o.active, r.active);
    }
    assert_eq!(eco.cataclysm_env_mods(), restored.cataclysm_env_mods());

    // Lockstep continuation: identical checksums after restore.
    let mut a_sim = eco;
    let mut b_sim = restored;
    for t in 161..=400u32 {
        run_ticks(&mut a_sim, t - 1, t);
        run_ticks(&mut b_sim, t - 1, t);
        if t % 60 == 0 {
            assert_eq!(a_sim.checksum(), b_sim.checksum(), "lockstep diverged at t={t}");
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Legacy v3 save: empty cataclysm table, zero-side-effect continuation
// ---------------------------------------------------------------------------

#[test]
fn legacy_v3_save_loads_without_cataclysms() {
    // Build a v4 save from a cataclysm-free world, strip the Phase 17 keys
    // and downgrade the version — a faithful v3 document.
    let mut eco = Ecology::new(SEED);
    run_ticks(&mut eco, 0, 200);
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&json).unwrap();
    {
        let obj = v.as_object_mut().unwrap();
        assert!(obj.remove("Cataclysms").is_some(), "v4 save carries the cataclysm key");
        assert!(obj.remove("NextCataclysmId").is_some(), "v4 save carries the id counter");
        obj["Version"] = serde_json::Value::Number(serde_json::Number::from(3u32));
    }
    let legacy: save_state::SaveState = serde_json::from_value(v).unwrap();
    let restored = Ecology::from_save_state(legacy);
    assert!(restored.cataclysms.is_empty(), "v3 restore → empty cataclysm table");
    assert_eq!(restored.cataclysm_env_mods(), (1.0, 1.0), "no phantom multipliers");

    // And it must keep running with checksums identical to a never-saved
    // control — the cataclysm code path is a zero-side-effect no-op here.
    let mut a_sim = restored;
    let mut b_sim = Ecology::new(SEED);
    run_ticks(&mut b_sim, 0, 200);
    for t in 201..=340u32 {
        run_ticks(&mut a_sim, t - 1, t);
        run_ticks(&mut b_sim, t - 1, t);
        if t % 70 == 0 {
            assert_eq!(a_sim.checksum(), b_sim.checksum(),
                "v3-restore control diverged at t={t}");
        }
    }
}

// ---------------------------------------------------------------------------
// 6. K-Pg survivor bias: high-metabolism phenotypes lose energy faster
// ---------------------------------------------------------------------------

#[test]
fn post_impact_survivor_bias() {
    // Mechanism-level A/B: both sims identical (same seed, PRED removed,
    // cohort pinned to alternating 0.7/1.5 metabolism at equal starting
    // energy). B then takes a severe dust winter whose stress multiplier
    // scales every drain with genome metabolism — so B's cohort must end
    // with strictly less total energy than A's.
    let mut a_sim = Ecology::new(SEED);
    let mut b_sim = Ecology::new(SEED);
    run_ticks(&mut a_sim, 0, 100);
    run_ticks(&mut b_sim, 0, 100);
    a_sim.kill_kind(PRED);
    b_sim.kill_kind(PRED);

    let cohort_a: Vec<u32> = a_sim.agents.iter()
        .filter(|a| a.alive && a.kind == HERB
                   && dist(a.x, a.z, 215.0, 292.0) < 60.0)
        .map(|a| a.id).take(18).collect();
    let cohort_b = cohort_a.clone();
    assert!(cohort_a.len() >= 10, "herd cluster has enough herbs, got {}", cohort_a.len());
    for eco in [&mut a_sim, &mut b_sim] {
        let mut k = 0usize;
        for a in eco.agents.iter_mut() {
            if a.alive && a.kind == HERB && cohort_a.contains(&a.id) {
                a.genome.metabolism = if k % 2 == 0 { 0.7 } else { 1.5 };
                a.energy = a.max_energy * 0.8;
                k += 1;
            }
        }
    }
    for t in 101..=300u32 {
        run_ticks(&mut a_sim, t - 1, t);
        run_ticks(&mut b_sim, t - 1, t);
    }
    b_sim.trigger_cataclysm(CATACLYSM_IMPACT, 215.0, 292.0, 60.0, 0.95, 500);
    for t in 301..=900u32 {
        run_ticks(&mut a_sim, t - 1, t);
        run_ticks(&mut b_sim, t - 1, t);
    }
    let sum_e = |eco: &Ecology, ids: &[u32]| -> f64 {
        ids.iter().map(|id| eco.agents.iter().find(|a| a.id == *id).map(|a| a.energy).unwrap_or(0.0)).sum()
    };
    let ea = sum_e(&a_sim, &cohort_a);
    let eb = sum_e(&b_sim, &cohort_b);
    eprintln!("=== survivor bias: A(no-dust)={ea:.2} B(dust)={eb:.2} ===");
    assert!(eb < ea, "dust-winter cohort total energy ({}) must fall below no-dust ({})", eb, ea);
}
