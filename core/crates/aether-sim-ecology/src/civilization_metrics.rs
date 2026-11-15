//! Phase 39.0 — civilization-scale metrics (headless research telemetry).
//!
//! Pure read-only observability: every function here inspects the existing
//! `Ecology` state and the `EnergyLedger` cumulative books. Nothing in this
//! module mutates the simulation, draws RNG, or feeds back into dynamics —
//! the "metrology is a telescope, not a whip" discipline from the roadmap
//! (§2.5) applies in full.
//!
//! Reference frames (see roadmap §Phase 39+):
//! * **Kardashev flux** — energy throughput per second, expressed as a
//!   fraction of the sim's primary-productivity input (light_in). This is
//!   the emergent analog of Kardashev's W-scale, built from our own ledger
//!   rather than fixed astrophysical constants.
//! * **Adami information stock** — the information-theoretic complexity of
//!   the living population: genome loci count × live individuals, summed
//!   per kind (Adami/Ofria/Collier 2000: genome complexity = information
//!   stored about the environment).
//! * **Tainter marginal slope** — the marginal energy return of complexity:
//!   per-clade energy throughput divided by the metabolic investment, as a
//!   running slope. Negative slopes are the Tainter collapse signal (§Q5).
//! * **Selection hierarchy count** — the number of distinct selection
//!   levels currently active: 1 = individual-level (all agents are
//!   independent), >1 would require group/colony entities (Phase 39.1+).
//! * **OEE activity** — Bedau-style rate of new-type appearance: clade
//!   births per 1k ticks, plus token-registry growth. A rate trending to
//!   zero is the plateau signal (roadmap §Q1).
//!
//! All values are deterministic pure functions of the current state.

use crate::ecology::{Ecology, SpeciesKind, DECO, FISH, HERB, OMN, PRED, SCAV};

/// One species' live population + per-kind throughput row.
#[derive(Clone, Debug, Default)]
pub struct KindTelemetry {
    pub kind: SpeciesKind,
    pub name: &'static str,
    pub alive: usize,
    pub mean_genome_loci: f64,
    pub mean_sense: f64,
    pub mean_speed: f64,
}

/// Kardashev-style energy-flux readout, fraction of primary input.
#[derive(Clone, Debug, Default)]
pub struct EnergyFlux {
    /// Cumulative primary production booked by the ledger (light_in).
    pub primary_input: f64,
    /// Σ of all booked losses (the "waste heat" column).
    pub total_loss: f64,
    /// `total_loss / primary_input` — how much of the input has been
    /// dissipated vs. stored in biomass. Approaches 1.0 at steady state.
    pub dissipation_ratio: f64,
    /// `predation_gain / primary_input` — the fraction of primary energy
    /// that reached the second trophic level (the predator-mediated flux).
    pub trophic_flux_share: f64,
    /// `brain_cost / primary_input` — the fraction of primary energy spent
    /// on cognition (the "thinking bill"). The single most direct measure
    /// of how expensive intelligence is in this world.
    pub cognition_share: f64,
    /// `info_cost / primary_input` — the fraction spent on perception /
    /// information processing.
    pub information_share: f64,
}

/// Adami-style information stock: total heritable complexity in the world.
#[derive(Clone, Debug, Default)]
pub struct InformationStock {
    /// Total live individuals across all kinds.
    pub population: usize,
    /// Mean genome loci count (core 17 + ext) over the live population.
    pub mean_loci: f64,
    /// Total loci = Σ per-individual loci (the information stock proxy).
    pub total_loci: usize,
    /// Distinct clades currently alive (a diversity-of-information term).
    pub live_clades: usize,
    /// Distinct knowledge tokens in the registry (cultural information).
    pub token_count: usize,
    /// Number of live agents carrying at least one token.
    pub token_bearers: usize,
}

/// Tainter-style marginal-return slope + selection hierarchy count.
#[derive(Clone, Debug, Default)]
pub struct ComplexityTelemetry {
    /// Number of active selection levels: 1 = individual-level only,
    /// 2 = individual + live group layer (Phase 39.1).
    pub selection_levels: usize,
    /// Per-clade energy throughput / metabolic investment, as a running
    /// slope. Negative = complexity is outrunning its energy base.
    pub tainter_slope: f64,
    /// Number of live clades with a recorded niche offset (organizational
    /// complexity proxy — each clade represents a distinct niche).
    pub niche_occupancy: usize,
}

/// OEE activity dashboard: new-type appearance rates (Bedau-style).
#[derive(Clone, Debug, Default)]
pub struct OeeActivity {
    /// Total clade births recorded in the event log.
    pub clade_births: usize,
    /// Clade births per 1000 ticks.
    pub speciation_rate_per_1k: f64,
    /// Total token registry entries (inventions).
    pub invention_count: usize,
    /// New lineages (distinct `LineageInfo` ids) observed.
    pub lineage_count: usize,
}

/// The full civilization-scale telemetry packet. All fields are derived
/// views; the packet is a pure function of the current `Ecology` state.
#[derive(Clone, Debug, Default)]
pub struct CivilizationMetrics {
    pub tick: u32,
    pub kinds: Vec<KindTelemetry>,
    pub energy: EnergyFlux,
    pub information: InformationStock,
    pub complexity: ComplexityTelemetry,
    pub oee: OeeActivity,
}

/// Build the full telemetry packet from the current world state.
pub fn measure(eco: &Ecology) -> CivilizationMetrics {
    let tick = eco.tick_count();
    let kinds = measure_kinds(eco);
    let energy = measure_energy(eco);
    let information = measure_information(eco, &kinds);
    let complexity = measure_complexity(eco);
    let oee = measure_oee(eco);
    CivilizationMetrics { tick, kinds, energy, information, complexity, oee }
}

fn measure_kinds(eco: &Ecology) -> Vec<KindTelemetry> {
    let kind_ids = [HERB, PRED, SCAV, OMN, FISH, DECO];
    let mut out = Vec::with_capacity(kind_ids.len());
    for &kind in &kind_ids {
        let live: Vec<_> = eco.living().iter().filter(|a| a.alive && a.kind == kind).collect();
        let n = live.len();
        let (mut loci, mut sense, mut speed) = (0usize, 0.0f64, 0.0f64);
        for a in &live {
            loci += 17 + a.genome.ext.len();
            sense += a.genome.sense;
            speed += a.genome.speed;
        }
        let f = n.max(1) as f64;
        out.push(KindTelemetry {
            kind,
            name: eco.kind_name(kind),
            alive: n,
            mean_genome_loci: loci as f64 / f,
            mean_sense: sense / f,
            mean_speed: speed / f,
        });
    }
    out
}

fn measure_energy(eco: &Ecology) -> EnergyFlux {
    let l = &eco.ledger;
    let primary = l.light_in.max(1e-12);
    let total_loss = l.metabolism_loss + l.evaporation + l.repro_loss
        + l.mineralized + l.offspring_invest + l.info_cost + l.brain_cost;
    EnergyFlux {
        primary_input: l.light_in,
        total_loss,
        dissipation_ratio: total_loss / primary,
        trophic_flux_share: l.predation_gain / primary,
        cognition_share: l.brain_cost / primary,
        information_share: l.info_cost / primary,
    }
}

fn measure_information(eco: &Ecology, kinds: &[KindTelemetry]) -> InformationStock {
    let mut population = 0usize;
    let mut total_loci = 0usize;
    let mut token_bearers = 0usize;
    for a in eco.living() {
        if !a.alive { continue; }
        population += 1;
        total_loci += 17 + a.genome.ext.len();
        if !a.known_tokens.is_empty() { token_bearers += 1; }
    }
    let live_clades = eco.species_clades().iter().filter(|c| c.death_tick.is_none()).count();
    let token_count = eco.token_registry.len();
    let mean_loci = if population == 0 { 0.0 } else { total_loci as f64 / population as f64 };
    InformationStock {
        population,
        mean_loci,
        total_loci,
        live_clades,
        token_count,
        token_bearers,
    }
}

fn measure_complexity(eco: &Ecology) -> ComplexityTelemetry {
    let l = &eco.ledger;
    // The "Tainter slope": per-clade energy throughput minus per-clade
    // maintenance cost, normalized by the number of live clades. When the
    // cost side outgrows the throughput side the slope goes negative,
    // which is the emergent collapse signal (roadmap §Q5).
    let live: Vec<_> = eco.species_clades().iter()
        .filter(|c| c.death_tick.is_none()).collect();
    let clades = live.len() as f64;
    let throughput = l.grazed + l.predation_gain + l.scavenged + l.filter_in;
    let maintenance = l.metabolism_loss + l.brain_cost + l.info_cost + l.repro_loss;
    let slope = if clades <= 0.0 { 0.0 } else { (throughput - maintenance) / clades };
    let niche_occupancy = live.iter().filter(|c| c.niche_offset.abs() > 1e-9).count();
    ComplexityTelemetry {
        // Selection levels: the individual layer is always present (1); a live
        // group layer adds the second level (Phase 39.1: group entities with
        // their own energy pool, reproduction and death). Levels are counted
        // from live state, never unlocked by a flag.
        selection_levels: crate::group::selection_levels(eco),
        tainter_slope: slope,
        niche_occupancy,
    }
}

fn measure_oee(eco: &Ecology) -> OeeActivity {
    let clade_births = eco.clade_events().len();
    let ticks = eco.tick_count().max(1) as f64;
    let mut lineages = std::collections::HashSet::new();
    for i in 0..clade_births {
        if let Some(ev) = eco.clade_events().get(i) {
            lineages.insert(ev.parent_clade_id);
        }
    }
    OeeActivity {
        clade_births,
        speciation_rate_per_1k: clade_births as f64 * 1000.0 / ticks,
        invention_count: eco.token_registry.len(),
        lineage_count: lineages.len(),
    }
}

/// Render the packet as a compact human-readable line for the research CLI.
pub fn format_line(m: &CivilizationMetrics) -> String {
    let pop: usize = m.kinds.iter().map(|k| k.alive).sum();
    format!(
        "CIV t={} pop={} clades={} tokens={} bearers={} \
         input={:.1} loss={:.1} diss={:.2} trophic={:.3} cog={:.3} info={:.3} \
         tainter={:.1} levels={} oee_rate={:.2}",
        m.tick,
        pop,
        m.information.live_clades,
        m.information.token_count,
        m.information.token_bearers,
        m.energy.primary_input,
        m.energy.total_loss,
        m.energy.dissipation_ratio,
        m.energy.trophic_flux_share,
        m.energy.cognition_share,
        m.energy.information_share,
        m.complexity.tainter_slope,
        m.complexity.selection_levels,
        m.oee.speciation_rate_per_1k,
    )
}
