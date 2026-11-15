//! Phase 43 long-run observatory (ignored probe — run with
//! `cargo test --release -p aether-sim-ecology --test phase43_longrun_observatory -- --ignored --nocapture`).
//!
//! 100k ticks × 3 seeds, sampling the cultural ratchet (bearer curve, taught
//! tokens) and abiogenesis (events, re-origination after extinction). Unlike
//! the Phase 42 probe, an extinct world is NOT abandoned: the whole point of
//! abiogenesis is what happens after the last death, so the run continues and
//! every extinction / re-origination transition is logged.
//!
//! Plus one A/B arm: seed 20261102 with `teach_on = false` (no inheritance)
//! for the bearer-curve comparison.

use aether_sim_ecology::ecology::Ecology;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn sample(eco: &Ecology, seed: u32, label: &str, t: u32) {
    let pop = eco.agents.iter().filter(|a| a.alive).count();
    let bearers = eco.agents.iter().filter(|a| a.alive && !a.known_tokens.is_empty()).count();
    eprintln!(
        "{} seed {} t={} pop={} bearers={} tokens={} learn={} taught={} teach_cost={:.1} \
         invent={} extinct_tok={} abio_ev={} abio_t={} births={} deaths={}",
        label, seed, t, pop, bearers, eco.token_registry.len(), eco.culture_stats.learn_count,
        eco.culture_stats.taught_count, eco.culture_stats.teach_cost_total,
        eco.culture_stats.innovation_count, eco.culture_stats.extinct_token_count,
        eco.abiogenesis_events, eco.abiogenesis_ticks, eco.births, eco.deaths,
    );
}

fn run_world(seed: u32, teach_on: bool, label: &str) {
    let mut eco = Ecology::new(seed);
    eco.teach_on = teach_on;
    let mut extinctions = 0u64;
    let mut was_alive = true;
    for t in 1..=100_000u32 {
        eco.update(0.1, daylight(t));
        let alive = eco.agents.iter().filter(|a| a.alive).count();
        if was_alive && alive == 0 {
            extinctions += 1;
            eprintln!(
                "{} seed {} EXTINCTION #{} at t={} (abio_ev={} abio_t={} tokens={} bearers_were_tracked)",
                label, seed, extinctions, t, eco.abiogenesis_events, eco.abiogenesis_ticks,
                eco.token_registry.len(),
            );
        }
        if !was_alive && alive > 0 {
            eprintln!(
                "{} seed {} RE-ORIGIN at t={} pop={} (abio_ev={} chemo_in={:.1})",
                label, seed, t, alive, eco.abiogenesis_events, eco.ledger.chemo_in,
            );
        }
        was_alive = alive > 0;
        if t % 2500 == 0 {
            sample(&eco, seed, label, t);
        }
    }
    let pop = eco.agents.iter().filter(|a| a.alive).count();
    eprintln!(
        "{} seed {} FATE={} final_pop={} extinctions={} abio_ev={} abio_t={} chemo_in={:.1} \
         tokens={} learn={} taught={} teach_cost={:.1} bearers={} births={} deaths={}",
        label, seed, if pop == 0 { "dead" } else { "alive" }, pop, extinctions,
        eco.abiogenesis_events, eco.abiogenesis_ticks, eco.ledger.chemo_in,
        eco.token_registry.len(), eco.culture_stats.learn_count,
        eco.culture_stats.taught_count, eco.culture_stats.teach_cost_total,
        eco.agents.iter().filter(|a| a.alive && !a.known_tokens.is_empty()).count(),
        eco.births, eco.deaths,
    );
}

#[test]
#[ignore]
fn phase43_longrun_observatory() {
    // PHASE43_ONLY=<0..3> runs a single arm (parallel-observability aid);
    // unset → all four arms sequentially.
    let only: Option<usize> = std::env::var("PHASE43_ONLY").ok()
        .and_then(|v| v.parse().ok());
    let arms: [(u32, bool, &str); 4] = [
        (20261102, true, "ON"),
        (20261105, true, "ON"),
        (42, true, "ON"),
        (20261102, false, "OFF"),
    ];
    for (i, &(seed, teach_on, label)) in arms.iter().enumerate() {
        if only.is_some() && only != Some(i) { continue; }
        run_world(seed, teach_on, label);
    }
}
