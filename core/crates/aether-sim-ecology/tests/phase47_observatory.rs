//! Phase 47 long-horizon observatory — the phylogeny under the region-island
//! model (200k ticks, default world). Run:
//! cargo test --release --test phase47_observatory -- --ignored --nocapture
//!
//! Outputs /tmp/phase47_observatory.csv (time series) and
//! /tmp/phase47_tree.csv (final tree with depth).

use aether_sim_ecology::ecology::{Ecology, DECO, REGION_COUNT};
use std::collections::HashMap;

const SEED: u32 = 20261102;
const TICKS: u32 = 200_000;
const SAMPLE_EVERY: u32 = 5_000;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * aether_libm::sin((t as f64 / 300.0) * std::f64::consts::PI)
}

fn clade_depth(eco: &Ecology, id: u32) -> u32 {
    let by_id: HashMap<u32, u32> = eco.species_clades().iter()
        .map(|c| (c.id, c.parent_id.unwrap_or(u32::MAX))).collect();
    let mut d = 0;
    let mut cur = id;
    while let Some(&p) = by_id.get(&cur) {
        if p == u32::MAX || d > 64 { break; }
        d += 1;
        cur = p;
    }
    d
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() { f64::NAN } else { xs.iter().sum::<f64>() / xs.len() as f64 }
}

fn region_genotype_spread(eco: &Ecology) -> String {
    let live: Vec<_> = eco.living().iter()
        .filter(|a| a.alive && eco.root_kind(a.kind) == DECO)
        .collect();
    let mut out = String::new();
    for r in 0..REGION_COUNT as usize {
        let rs: Vec<_> = live.iter().filter(|a| a.region as usize == r).collect();
        if rs.is_empty() { continue; }
        out.push_str(&format!(
            "r{}:n={} speed={:.3} sense={:.2} size={:.3} eff={:.3}|",
            r, rs.len(),
            mean(&rs.iter().map(|a| a.genome.speed).collect::<Vec<_>>()),
            mean(&rs.iter().map(|a| a.genome.sense).collect::<Vec<_>>()),
            mean(&rs.iter().map(|a| a.genome.size).collect::<Vec<_>>()),
            mean(&rs.iter().map(|a| a.genome.deco_efficiency).collect::<Vec<_>>()),
        ));
    }
    out
}

#[test]
#[ignore = "200k-tick Phase 47 phylogeny observatory"]
fn phase47_observatory() {
    let mut eco = Ecology::new(SEED);
    let mut csv = String::from(
        "t,pop,deco_family,live_clades,total_clades,spec_events,extinctions,dispersal,max_depth\n",
    );
    let mut last_events = 0usize;
    eprintln!("=== Phase 47 observatory seed={SEED} ticks={TICKS} p={} ===", eco.dispersal_p);
    for t in 1..=TICKS {
        eco.update(0.1, daylight(t));
        let n = eco.clade_events().len();
        if n > last_events {
            for e in &eco.clade_events()[last_events..n] {
                eprintln!(
                    "  [event] t={} parent={} children={:?} dist={:.3} kind={} type={}",
                    t, e.parent_clade_id, e.child_clade_ids, e.genetic_distance,
                    e.kind_id, e.event_kind
                );
            }
            last_events = n;
        }
        if t % SAMPLE_EVERY == 0 {
            let live = eco.living().iter().filter(|a| a.alive).count();
            let live_clades = eco.species_clades().iter().filter(|c| c.death_tick.is_none()).count();
            let spec = eco.clade_events().iter().filter(|e| e.event_kind == 0).count();
            let ext = eco.clade_events().iter().filter(|e| e.event_kind != 0).count();
            let max_depth = eco.species_clades().iter()
                .filter(|c| c.death_tick.is_none())
                .map(|c| clade_depth(&eco, c.id)).max().unwrap_or(0);
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{}\n",
                t, live, eco.count_family(DECO), live_clades,
                eco.species_clades().len(), spec, ext, eco.dispersal_events, max_depth
            ));
            if t % 50_000 == 0 {
                eprintln!(
                    "[obs] t={} pop={} deco_family={} live_clades={} spec={} ext={} dispersal={} depth={} | {}",
                    t, live, eco.count_family(DECO), live_clades, spec, ext,
                    eco.dispersal_events, max_depth, region_genotype_spread(&eco)
                );
            }
        }
    }
    let mut tree = String::from("id,parent,kind,birth,death,members,depth\n");
    for c in eco.species_clades() {
        tree.push_str(&format!(
            "{},{:?},{},{},{},{},{}\n",
            c.id, c.parent_id, c.kind, c.birth_tick,
            c.death_tick.map(|d| d.to_string()).unwrap_or_default(),
            c.members.len(), clade_depth(&eco, c.id)
        ));
    }
    std::fs::write("/tmp/phase47_observatory.csv", csv).unwrap();
    std::fs::write("/tmp/phase47_tree.csv", tree).unwrap();
    eprintln!(
        "=== Phase 47 observatory done: clades={} events={} depth={} ===",
        eco.species_clades().len(), last_events,
        eco.species_clades().iter().filter(|c| c.death_tick.is_none())
            .map(|c| clade_depth(&eco, c.id)).max().unwrap_or(0)
    );
}
