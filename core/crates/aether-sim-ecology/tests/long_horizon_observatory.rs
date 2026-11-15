//! Phase 40 — long-horizon observatory: let the world run 10^5 ticks.
//!
//! No new dynamics. The entire point is to observe what the full Phase 31-39
//! stack actually produces over geological-ish time (100k ticks = 10k sim
//! seconds ≈ 2.8 sim hours ≈ 5.5 seasonal cycles).
//!
//! Run with:
//!   cargo test --release --test long_horizon_observatory -- --ignored --nocapture
//!
//! Outputs one CSV row per sample to /tmp/long_horizon_seed_<seed>.csv and
//! prints event lines (speciation / invention / group births) as they fire.

use aether_sim_ecology::civilization_metrics::measure;
use aether_sim_ecology::ecology::Ecology;

const SEEDS: [u32; 3] = [20261102, 20261105, 42];
const TICKS: u32 = 100_000;
const SAMPLE_EVERY: u32 = 1_000;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * ((t as f64 / 300.0) * std::f64::consts::PI).sin()
}

#[test]
#[ignore = "long-horizon observatory — run manually (≈20 min for 3 seeds)"]
fn long_horizon_observation() {
    for seed in SEEDS {
        let path = format!("/tmp/long_horizon_seed_{}.csv", seed);
        let mut csv = String::from(
            "t,pop,herb,pred,scav,omn,fish,deco,clades,tokens,bearers,groups,members,\
             levels,input,loss,diss,trophic,cog,info,tainter,oee_rate\n",
        );
        let mut eco = Ecology::new(seed);
        let mut last_clade_events = eco.clade_events().len();
        let mut last_tokens = eco.token_registry.len();
        let mut peak_groups = 0usize;
        let mut peak_members = 0usize;
        let mut total_speciations = 0usize;
        let mut total_inventions = 0usize;
        let mut pred_extinct_tick: Option<u32> = None;

        eprintln!("=== seed {} starting ({} ticks) ===", seed, TICKS);
        for t in 1..=TICKS {
            eco.update(0.1, daylight(t));

            // Event tracking (sampled every tick for events, cheap checks).
            let n_events = eco.clade_events().len();
            if n_events > last_clade_events {
                for e in &eco.clade_events()[last_clade_events..n_events] {
                    total_speciations += 1;
                    eprintln!(
                        "  [speciation] seed={} t={} event_tick={} parent_clade={} children={:?} dist={:.3}",
                        seed, t, e.tick, e.parent_clade_id, e.child_clade_ids, e.genetic_distance
                    );
                }
                last_clade_events = n_events;
            }
            let n_tokens = eco.token_registry.len();
            if n_tokens > last_tokens {
                total_inventions += n_tokens - last_tokens;
                eprintln!("  [invention] seed={} t={} tokens={}", seed, t, n_tokens);
                last_tokens = n_tokens;
            }

            if t % SAMPLE_EVERY == 0 {
                let m = measure(&eco);
                let groups = eco.groups.len();
                let members: usize = eco.groups.iter().map(|g| g.members.len()).sum();
                peak_groups = peak_groups.max(groups);
                peak_members = peak_members.max(members);
                let kind = |name: &str| m.kinds.iter().find(|k| k.name == name).map(|k| k.alive).unwrap_or(0);
                let (herb, pred, scav, omn, fish, deco) = (
                    kind("herb"), kind("pred"), kind("scav"), kind("omn"), kind("fish"), kind("deco"),
                );
                if pred == 0 && pred_extinct_tick.is_none() {
                    pred_extinct_tick = Some(t);
                    eprintln!("  [extinction] seed={} t={} PRED population reached zero", seed, t);
                }
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{:.1},{:.1},{:.3},{:.4},{:.4},{:.4},{:.1},{:.3}\n",
                    t, m.information.population, herb, pred, scav, omn, fish, deco,
                    m.information.live_clades, m.information.token_count, m.information.token_bearers,
                    groups, members, m.complexity.selection_levels,
                    m.energy.primary_input, m.energy.total_loss, m.energy.dissipation_ratio,
                    m.energy.trophic_flux_share, m.energy.cognition_share,
                    m.energy.information_share, m.complexity.tainter_slope,
                    m.oee.speciation_rate_per_1k,
                ));
                if t % 10_000 == 0 {
                    eprintln!(
                        "  [sample] seed={} t={} pop={} herb={} pred={} scav={} omn={} fish={} deco={} clades={} tokens={} groups={} members={} levels={} diss={:.2} tainter={:.1}",
                        seed, t, m.information.population, herb, pred, scav, omn, fish, deco,
                        m.information.live_clades, m.information.token_count,
                        groups, members, m.complexity.selection_levels,
                        m.energy.dissipation_ratio, m.complexity.tainter_slope
                    );
                }
            }
        }
        std::fs::write(&path, csv).unwrap();
        eprintln!(
            "=== seed {} done: speciations={} inventions={} peak_groups={} peak_members={} pred_extinct_at={:?} csv={} ===",
            seed, total_speciations, total_inventions, peak_groups, peak_members, pred_extinct_tick, path
        );
    }
    eprintln!("=== all seeds complete ===");
}
