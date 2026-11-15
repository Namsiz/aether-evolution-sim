//! Phase 44.2 — million-tick observatory: geological time for the full stack.
//!
//! 1,000,000 ticks = 100,000 sim-seconds ≈ 55 seasonal cycles. The complete
//! Phase 31-44.1 stack (triune brains with behaviour authority, spatial
//! culture, group entities, abiogenesis) runs without intervention.
//!
//! Run: cargo test --release --test longmill_observatory -- --ignored --nocapture

use aether_sim_ecology::civilization_metrics::{format_line, measure};
use aether_sim_ecology::ecology::Ecology;

const SEED: u32 = 20261102;
const TICKS: u32 = 1_000_000;
const SAMPLE_EVERY: u32 = 5_000;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * ((t as f64 / 300.0) * std::f64::consts::PI).sin()
}

#[test]
#[ignore = "million-tick geological run — ~70 minutes"]
fn longmill_observation() {
    let path = "/tmp/longmill_20261102.csv";
    let mut csv = String::from(
        "t,pop,herb,pred,scav,omn,fish,deco,clades,tokens,bearers,groups,members,levels,\
         authorities,tainter,oee_rate,inventions,learns,taught,abio_events\n",
    );
    let mut eco = Ecology::new(SEED);
    let mut last_clades = eco.clade_events().len();
    let mut total_spec = 0usize;
    eprintln!("=== longmill seed {SEED} ({TICKS} ticks) ===");
    for t in 1..=TICKS {
        eco.update(0.1, daylight(t));
        let n = eco.clade_events().len();
        if n > last_clades {
            total_spec += n - last_clades;
            for e in &eco.clade_events()[last_clades..n] {
                eprintln!("  [speciation] t={} parent={} children={:?} dist={:.3}",
                    t, e.parent_clade_id, e.child_clade_ids, e.genetic_distance);
            }
            last_clades = n;
        }
        if t % SAMPLE_EVERY == 0 {
            let m = measure(&eco);
            let kind = |name: &str| m.kinds.iter().find(|k| k.name == name).map(|k| k.alive).unwrap_or(0);
            let auth = eco.living().iter()
                .filter(|a| a.brain_authority == aether_sim_ecology::ecology::BRAIN_AUTHORITY_BRAIN)
                .count();
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.1},{:.3},{},{},{},{}\n",
                t, m.information.population, kind("herb"), kind("pred"), kind("scav"),
                kind("omn"), kind("fish"), kind("deco"),
                m.information.live_clades, m.information.token_count, m.information.token_bearers,
                eco.groups.len(), eco.groups.iter().map(|g| g.members.len()).sum::<usize>(),
                m.complexity.selection_levels, auth, m.complexity.tainter_slope,
                m.oee.speciation_rate_per_1k,
                eco.culture_stats.innovation_count, eco.culture_stats.learn_count,
                eco.culture_stats.taught_count, eco.abiogenesis_events,
            ));
            if t % 25_000 == 0 {
                eprintln!(
                    "[mill] t={} {} authorities={}",
                    t, format_line(&m), auth
                );
            }
        }
    }
    std::fs::write(path, csv).unwrap();
    eprintln!("=== longmill done: speciations={total_spec} inventions={} csv={path} ===",
        eco.culture_stats.innovation_count);
}
