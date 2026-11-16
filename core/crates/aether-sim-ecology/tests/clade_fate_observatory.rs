//! Phase 46.1 — clade fate observatory: the phylogeny under 1M ticks.
//!
//! Phase 45 proved a world can mint its first natural species (t=3780, DECO
//! fork). This run asks what happens to those species over geological time:
//! does the tree keep branching, or does each fork settle into a stable
//! daughter lineage? How long do species live? Which lineages win?
//!
//! Run: cargo test --release --test clade_fate_observatory -- --ignored --nocapture

use aether_sim_ecology::civilization_metrics::{format_line, measure};
use aether_sim_ecology::ecology::Ecology;

const SEED: u32 = 20261102;
const TICKS: u32 = 1_000_000;
const SAMPLE_EVERY: u32 = 5_000;

fn daylight(t: u32) -> f64 {
    0.5 + 0.5 * ((t as f64 / 300.0) * std::f64::consts::PI).sin()
}

#[test]
#[ignore = "million-tick clade-fate run — ~70 minutes"]
fn clade_fate_observation() {
    let csv_path = "/tmp/clade_fate_20261102.csv";
    let tree_path = "/tmp/clade_tree_20261102.csv";
    let mut csv = String::from(
        "t,pop,herb,pred,scav,omn,fish,deco,clades_live,clades_total,events,tokens,bearers,groups,levels,authorities\n",
    );
    let mut eco = Ecology::new(SEED);
    let mut last_events = 0usize;
    eprintln!("=== clade-fate seed {SEED} ({TICKS} ticks) ===");
    for t in 1..=TICKS {
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
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                t, m.information.population, kind("herb"), kind("pred"), kind("scav"),
                kind("omn"), kind("fish"), kind("deco"),
                live_clades, eco.species_clades().len(), last_events,
                m.information.token_count, m.information.token_bearers,
                eco.groups.len(), m.complexity.selection_levels, auth,
            ));
            if t % 50_000 == 0 {
                eprintln!("[fate] t={} {} live_clades={} total={} events={}",
                    t, format_line(&m), live_clades, eco.species_clades().len(), last_events);
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
    std::fs::write(csv_path, csv).unwrap();
    std::fs::write(tree_path, tree).unwrap();
    eprintln!(
        "=== clade-fate done: clades={} events={} csv={csv_path} tree={tree_path} ===",
        eco.species_clades().len(), last_events
    );
}
