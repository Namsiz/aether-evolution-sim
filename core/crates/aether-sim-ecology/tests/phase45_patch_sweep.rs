//! Phase 45 observatories — patch-frequency sweep + 200k long-run.
//!
//! Run (release, both are long):
//!   cargo test -p aether-sim-ecology --release --test phase45_patch_sweep -- --ignored --nocapture
//!
//! `patch_frequency_sweep` runs three disturbance frequencies (low / mid /
//! high) for 100k ticks each and records the speciation counts — the
//! intermediate-disturbance (Connell 1978) test: is the disturbance-diversity
//! curve non-monotonic?
//!
//! `longrun_200k` runs the default calibration (PATCH_P) for 200k ticks with
//! the full stack and samples speciation, species counts, isolation and the
//! cultural-health metrics (value-weighted bearers, top-token value
//! distribution).

use aether_sim_ecology::civilization_metrics::measure;
use aether_sim_ecology::ecology::Ecology;

const SEED: u32 = 20261102;
const SWEEP_TICKS: u32 = 100_000;
const SWEEP_LEVELS: [f64; 3] = [0.0002, 0.001, 0.005];
const LONGRUN_TICKS: u32 = 200_000;
const SAMPLE_EVERY: u32 = 5_000;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * ((t as f64 / 300.0) * std::f64::consts::PI).sin()
}

fn speciation_count(eco: &Ecology) -> usize {
    eco.clade_events().iter().filter(|e| e.event_kind == 0).count()
}

fn top_token_values(eco: &Ecology, k: usize) -> Vec<(u32, f64)> {
    let mut v: Vec<(u32, f64)> = eco.token_registry.tokens.iter()
        .map(|t| (t.id, t.use_gain))
        .collect();
    v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then(a.0.cmp(&b.0)));
    v.truncate(k);
    v
}

fn run(kind: &str, patch_p: f64, ticks: u32) -> Ecology {
    let mut eco = Ecology::new(SEED);
    eco.patch_p = patch_p;
    let mut last_spec = 0usize;
    eprintln!("=== {kind}: seed {SEED} ticks {ticks} patch_p {patch_p} ===");
    for t in 1..=ticks {
        eco.update(0.1, daylight(t));
        let n = eco.clade_events().len();
        if n > last_spec {
            for e in &eco.clade_events()[last_spec..n] {
                eprintln!("  [{kind} spec] t={t} kind={} parent={} children={:?} dist={:.3} ek={}",
                    e.kind_id, e.parent_clade_id, e.child_clade_ids, e.genetic_distance, e.event_kind);
            }
            last_spec = n;
        }
    }
    eco
}

#[test]
#[ignore = "3 x 100k-tick disturbance sweep — minutes in release"]
fn patch_frequency_sweep() {
    let path = "/tmp/phase45_sweep.csv";
    let mut csv = String::from(
        "patch_p,pop,herb,deco,fish,speciations,extinctions,clades_live,max_clade_kinds,tokens,vwb,patches\n",
    );
    let mut results: Vec<(f64, usize)> = Vec::new();
    for &p in &SWEEP_LEVELS {
        let eco = run("sweep", p, SWEEP_TICKS);
        let m = measure(&eco);
        let kind_count = |k: &str| m.kinds.iter().find(|x| x.name == k).map(|x| x.alive).unwrap_or(0);
        let specs = speciation_count(&eco);
        let exts = eco.clade_events().iter().filter(|e| e.event_kind != 0).count();
        let mut kinds_in_tree: Vec<u32> = eco.species_clades().iter().map(|c| c.kind_id).collect();
        kinds_in_tree.sort_unstable();
        kinds_in_tree.dedup();
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{:.2},{}\n",
            p, m.information.population, kind_count("herb"), kind_count("deco"), kind_count("fish"),
            specs, exts, m.information.live_clades, kinds_in_tree.len(),
            m.information.token_count, eco.culture_stats.value_weighted_bearers, eco.patch_events,
        ));
        results.push((p, specs));
        eprintln!("--- sweep patch_p={p}: speciations={specs} extinctions={exts} pop={} clades={}",
            m.information.population, m.information.live_clades);
    }
    std::fs::write(path, csv).unwrap();
    eprintln!("--- sweep csv: {path}");
    eprintln!("--- sweep curve (patch_p → speciations): {:?}", results);
    assert!(results.iter().any(|&(_, s)| s > 0),
        "at least one disturbance level must break the stability curse (got {results:?})");
}

#[test]
#[ignore = "200k-tick long-run observatory — minutes in release"]
fn longrun_200k() {
    let path = "/tmp/phase45_longrun.csv";
    let mut csv = String::from(
        "t,pop,herb,pred,scav,omn,fish,deco,pop_family_deco,clades_live,speciations,extinctions,\
         tokens,bearers,vwb,learns,taught,patches,mineral,grass,iso_deco,iso_herb\n",
    );
    let mut eco = Ecology::new(SEED);
    let mut last_spec = 0usize;
    let mut total_spec = 0usize;
    eprintln!("=== longrun 200k: seed {SEED} patch_p {} ===", eco.patch_p);
    for t in 1..=LONGRUN_TICKS {
        eco.update(0.1, daylight(t));
        let n = eco.clade_events().len();
        if n > last_spec {
            total_spec += n - last_spec;
            for e in &eco.clade_events()[last_spec..n] {
                eprintln!("  [spec] t={t} kind={} parent={} children={:?} dist={:.3} ek={}",
                    e.kind_id, e.parent_clade_id, e.child_clade_ids, e.genetic_distance, e.event_kind);
            }
            last_spec = n;
        }
        if t % SAMPLE_EVERY == 0 {
            let m = measure(&eco);
            let kind = |name: &str| m.kinds.iter().find(|k| k.name == name).map(|k| k.alive).unwrap_or(0);
            let fam_deco = eco.living().iter()
                .filter(|a| a.alive && eco.root_kind(a.kind) == 5).count();
            let specs = speciation_count(&eco);
            let exts = eco.clade_events().iter().filter(|e| e.event_kind != 0).count();
            let iso = |kid: u32| eco.isolation_by_kind.iter()
                .find(|(k, _)| *k == kid)
                .and_then(|(_, pts)| pts.last().map(|p| p.dist))
                .unwrap_or(0.0);
            let mineral: f64 = eco.mineral_field().iter().map(|&v| v as f64).sum();
            let grass: f64 = eco.grass().iter().map(|&v| v as f64).sum();
            let bearers = eco.living().iter().filter(|a| a.alive && !a.known_tokens.is_empty()).count();
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.2},{},{},{},{:.0},{:.0},{:.3},{:.3}\n",
                t, m.information.population, kind("herb"), kind("pred"), kind("scav"), kind("omn"),
                kind("fish"), kind("deco"), fam_deco, m.information.live_clades, specs, exts,
                m.information.token_count, bearers, eco.culture_stats.value_weighted_bearers,
                eco.culture_stats.learn_count, eco.culture_stats.taught_count,
                eco.patch_events, mineral, grass, iso(5), iso(0),
            ));
            if t % 25_000 == 0 {
                let tops = top_token_values(&eco, 5);
                eprintln!("t={t} pop={} clades={} specs={} tokens={} bearers={} vwb={:.3} patches={} tops={:?}",
                    m.information.population, m.information.live_clades, specs,
                    m.information.token_count, bearers, eco.culture_stats.value_weighted_bearers,
                    eco.patch_events, tops);
            }
        }
    }
    std::fs::write(path, csv).unwrap();
    let m = measure(&eco);
    eprintln!("=== longrun done: pop={} speciations={total_spec} clades_live={} tokens={} csv={path}",
        m.information.population, m.information.live_clades, m.information.token_count);
}

/// Emergent A/B for the cultural bottleneck (acceptance #4).
///
/// The free-running world cannot answer this cleanly: use_gain is an
/// exposure-weighted EMA, and the tokens that get placed are exactly the
/// oldest, most widely shared ones — so "harmful" and "popular" correlate
/// by construction (the 200k long-run shows the effect). The controlled
/// experiment injects two tagged tokens — TU (vec[0]=+0.8, positive
/// structure feedback) and TH (vec[0]=-0.8, negative feedback) — into two
/// alternating cohorts of adult HERB carriers, then re-runs the SAME seed
/// with `culture_value_on` on vs off. Values still come purely from the
/// energy feedback path; only the token vectors are seeded.
///
/// Acceptance #4: TU must carry more bearers than TH, and the TU:TH bearer
/// ratio must be strictly larger in the value-sorted arm than in the
/// insertion-order arm.
#[test]
#[ignore = "2 x 12k-tick injection A/B — ~2 min in release"]
fn culture_value_ab() {
    const WARMUP: u32 = 4_000;
    const AB_TICKS: u32 = 8_000;
    fn run_arm(value_on: bool) -> (usize, usize) {
        let mut eco = Ecology::new(SEED);
        eco.culture_value_on = value_on;
        for t in 1..=WARMUP {
            eco.update(0.1, daylight(t));
        }
        let mut v_u = vec![0.05; aether_sim_ecology::culture::TOKEN_DIM];
        v_u[0] = 0.8;
        let mut v_h = vec![0.05; aether_sim_ecology::culture::TOKEN_DIM];
        v_h[0] = -0.8;
        let tu = eco.token_registry.register(
            aether_sim_ecology::culture::KnowledgeToken::new(eco.token_registry.next_token_id, v_u));
        let th = eco.token_registry.register(
            aether_sim_ecology::culture::KnowledgeToken::new(eco.token_registry.next_token_id, v_h));
        let mut i = 0usize;
        for a in eco.agents.iter_mut() {
            if a.alive && a.kind == 0 && a.age > 60.0 {
                a.known_tokens = vec![if i % 2 == 0 { tu } else { th }];
                i += 1;
            }
        }
        eprintln!("  arm value_on={value_on}: injected TU={tu} TH={th} into {i} herb carriers");
        for t in (WARMUP + 1)..=(WARMUP + AB_TICKS) {
            eco.update(0.1, daylight(t));
        }
        let (mut bu, mut bh) = (0usize, 0usize);
        for a in eco.living().iter().filter(|a| a.alive) {
            if a.known_tokens.contains(&tu) { bu += 1; }
            if a.known_tokens.contains(&th) { bh += 1; }
        }
        eprintln!("  arm value_on={value_on}: bearers TU={bu} TH={bh} | value(TU)={:.4} value(TH)={:.4}",
            eco.token_registry.value_of(tu), eco.token_registry.value_of(th));
        (bu, bh)
    }
    let (tu_v, th_v) = run_arm(true);
    let (tu_r, th_r) = run_arm(false);
    eprintln!("=== culture A/B (injection) ===");
    eprintln!("  value arm: TU={tu_v} TH={th_v}");
    eprintln!("  random arm: TU={tu_r} TH={th_r}");
    assert!(tu_v + th_v > 4 && tu_r + th_r > 4, "both tokens must spread in both arms");
    assert!(tu_v > th_v,
        "value-sorted arm: the positive-value token must carry more bearers (TU={tu_v} vs TH={th_v})");
    let ratio_v = tu_v as f64 / (th_v + 1) as f64;
    let ratio_r = tu_r as f64 / (th_r + 1) as f64;
    assert!(ratio_v > ratio_r,
        "the value bias must sharpen the TU:TH ratio ({ratio_v:.2} vs {ratio_r:.2})");
}

