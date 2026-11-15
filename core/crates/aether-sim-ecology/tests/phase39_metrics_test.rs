//! Phase 39.0 acceptance — civilization-scale telemetry is a pure telescope.
//!
//! The whole point of this layer is that it observes without perturbing:
//!   1. `telemetry_is_read_only` — building the metrics packet before and
//!      after a run leaves the checksum bit-identical (no RNG, no state
//!      mutation, no ledger writes).
//!   2. `kardashev_flux_reflects_ledger` — the flux shares are exactly the
//!      ledger ratios (dissipation + trophic + cognition + information
//!      columns), so the numbers can be traced back to the books.
//!   3. `information_stock_counts_loci_and_tokens` — population × loci and
//!      token bearer counts come straight from the live state.
//!   4. `oee_activity_tracks_clade_events` — speciation rate = clade births
//!      per 1k ticks; the format line renders without panicking.

use aether_sim_ecology::civilization_metrics::{format_line, measure};
use aether_sim_ecology::ecology::Ecology;

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in from..=to {
        let daylight = 0.5 + 0.5 * ((t as f64 / 300.0) * std::f64::consts::PI).sin();
        eco.update(0.1, daylight);
    }
}

#[test]
fn telemetry_is_read_only() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 300);
    let before = eco.checksum();
    let m1 = measure(&eco);
    let m2 = measure(&eco);
    let after = eco.checksum();
    assert_eq!(before, after, "measuring must not perturb the world");
    assert_eq!(m1.tick, m2.tick, "tick readout deterministic");
    assert_eq!(m1.information.population, m2.information.population);
    assert_eq!(
        format!("{:?}", m1.energy),
        format!("{:?}", m2.energy),
        "double measure must be identical"
    );
}

#[test]
fn kardashev_flux_reflects_ledger() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 500);
    let m = measure(&eco);
    let l = &eco.ledger;
    let primary = l.light_in.max(1e-12);
    assert!(m.energy.primary_input > 0.0, "primary production booked");
    assert!(
        (m.energy.cognition_share - l.brain_cost / primary).abs() < 1e-12,
        "cognition share must equal brain_cost / light_in"
    );
    assert!(
        (m.energy.information_share - l.info_cost / primary).abs() < 1e-12,
        "information share must equal info_cost / light_in"
    );
    assert!(
        (m.energy.trophic_flux_share - l.predation_gain / primary).abs() < 1e-12,
        "trophic flux must equal predation_gain / light_in"
    );
    // Dissipation is a ratio of positives; it must be non-negative.
    assert!(m.energy.dissipation_ratio >= 0.0);
    eprintln!(
        "flux: primary={:.0} loss={:.0} diss={:.2} trophic={:.4} cog={:.4} info={:.4}",
        m.energy.primary_input, m.energy.total_loss, m.energy.dissipation_ratio,
        m.energy.trophic_flux_share, m.energy.cognition_share, m.energy.information_share
    );
}

#[test]
fn information_stock_counts_loci_and_tokens() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 200);
    let m = measure(&eco);
    let live: Vec<_> = eco.living().iter().filter(|a| a.alive).collect();
    assert_eq!(m.information.population, live.len(), "population readout");
    let expected_loci: usize = live.iter().map(|a| 17 + a.genome.ext.len()).sum();
    assert_eq!(m.information.total_loci, expected_loci, "loci stock");
    let expected_bearers = live.iter().filter(|a| !a.known_tokens.is_empty()).count();
    assert_eq!(m.information.token_bearers, expected_bearers, "token bearers");
    assert_eq!(m.information.token_count, eco.token_registry.len(), "token registry");
    for k in &m.kinds {
        assert!(k.alive <= live.len());
        assert!(k.mean_genome_loci >= 17.0 - 1e-9);
    }
}

#[test]
fn oee_activity_tracks_clade_events() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 3000);
    let m = measure(&eco);
    assert_eq!(m.oee.clade_births, eco.clade_events().len(), "clade births");
    let expect_rate = eco.clade_events().len() as f64 * 1000.0 / eco.tick_count().max(1) as f64;
    assert!(
        (m.oee.speciation_rate_per_1k - expect_rate).abs() < 1e-9,
        "speciation rate per 1k ticks"
    );
    let line = format_line(&m);
    assert!(line.starts_with("CIV t="), "format line starts with CIV: {}", line);
    eprintln!("{}", line);
}
