//! Phase 40.2 acceptance — de-novo invention activates the culture layer.
//!
//! Phase 37 shipped the culture machinery but left it dormant: `recombine`
//! needs TWO known tokens, and the world starts with none — a deadlock the
//! 100k-tick observatory confirmed (zero inventions in 300k simulated ticks).
//! This phase adds the missing first-token pathway (guided variation) and
//! un-gates the culture loop from the empty-registry condition.
//!
//! Acceptance:
//!   1. `culture_activates_and_tokens_emerge` — after 30k ticks the registry
//!      is non-empty and at least one de-novo invention has fired.
//!   2. `learning_and_structures_emerge` — imitation and niche construction
//!      follow once tokens exist.
//!   3. `invented_tokens_stay_in_bounds` — invented vectors live in [-1, 1].
//!   4. `culture_off_is_inert` — with the layer disabled the registry stays
//!      empty and no innovations/learns/structures are recorded, and the
//!      trajectory is deterministic on its own (A/B discipline).

use aether_sim_ecology::ecology::Ecology;

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in from..=to {
        let daylight = 0.5 + 0.5 * ((t as f64 / 300.0) * std::f64::consts::PI).sin();
        eco.update(0.1, daylight);
    }
}

#[test]
fn culture_activates_and_tokens_emerge() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 30_000);
    assert!(eco.token_registry.len() > 0,
        "de-novo invention must populate the token registry (got 0)");
    assert!(eco.culture_stats.innovation_count > 0,
        "innovation counter must record the de-novo events");
    eprintln!(
        "culture active: registry={} innovations={} learns={} structures={}",
        eco.token_registry.len(),
        eco.culture_stats.innovation_count,
        eco.culture_stats.learn_count,
        eco.culture_stats.structure_place_count,
    );
}

#[test]
fn learning_and_structures_emerge() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 30_000);
    assert!(eco.culture_stats.learn_count > 0,
        "imitation must follow once tokens exist (teach-and-copy loop)");
    assert!(eco.culture_stats.structure_place_count > 0,
        "niche construction (structure placement) must fire once tokens are held");
}

#[test]
fn invented_tokens_stay_in_bounds() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 20_000);
    for tok in &eco.token_registry.tokens {
        assert_eq!(tok.vec.len(), aether_sim_ecology::culture::TOKEN_DIM);
        for &v in &tok.vec {
            assert!(v.is_finite(), "token {} contains non-finite value", tok.id);
            assert!((-1.0..=1.0).contains(&v), "token {} value {} out of [-1,1]", tok.id, v);
        }
    }
}

#[test]
fn culture_off_is_inert() {
    let mut a = Ecology::new(20261102);
    a.culture_on = false;
    run_ticks(&mut a, 1, 5_000);
    assert!(a.token_registry.is_empty(), "culture off: no tokens may appear");
    assert_eq!(a.culture_stats.innovation_count, 0);
    assert_eq!(a.culture_stats.learn_count, 0);
    assert_eq!(a.culture_stats.structure_place_count, 0);
    // Determinism of the culture-off world on its own.
    let mut b = Ecology::new(20261102);
    b.culture_on = false;
    run_ticks(&mut b, 1, 5_000);
    assert_eq!(a.checksum(), b.checksum(), "culture-off double run diverged");
}
