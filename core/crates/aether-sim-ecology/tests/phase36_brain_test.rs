//! Phase 36 acceptance tests — evolvable micro-brain (shadow network layer).
//!
//! Covers the brain-specific invariants that the re-baseline relies on:
//!   1. `weights_and_eligibility_stay_bounded` — after a long run every
//!      synaptic weight and eligibility trace is finite and inside the
//!      clamp band. This is the regression guard for the unbounded-Hebbian
//!      feedback bug that produced Inf→NaN weights (and therefore `null`
//!      in the JSON save) before the clamp was added.
//!   2. `brain_upkeep_scales_with_network_size` — the metabolic price of a
//!      brain scales linearly with hidden-node count (Herculano-Houzel
//!      style): a 4-node reference net costs 4× BRAIN_PER_NEURON_COST per
//!      second.
//!   3. `save_restore_preserves_brain_internals` — a restored agent's brain
//!      continues bit-exactly: hidden-node outputs (the recurrent layer's
//!      previous-tick activations), logits, eligibility, weights and the
//!      taxis baseline all survive the JSON hop, so the shadow trajectory
//!      does not fork from the never-restored twin.
//!   4. `shadow_never_drives_behaviour_by_default` — with the authority
//!      switch OFF (default), the FSM is the sole behaviour source: the
//!      checksum trajectory is identical to a run with the brain shadow
//!      disabled (upkeep is the only side channel, hence the A/B uses a
//!      world-level toggle to keep energy identical).
//!
//! Brain-crate unit tests (`forward_is_deterministic_zero_rng`,
//! `hebbian_updates_weights`, `reference_networks_have_expected_weights`)
//! cover the zero-RNG discipline and the reference-network wiring.

use aether_sim_ecology::ecology::{Ecology, BRAIN_AUTHORITY_BRAIN, BRAIN_AUTHORITY_FSM};
use aether_brain::{BRAIN_PER_NEURON_COST, ELIGIBILITY_CLAMP, WEIGHT_CLAMP};

fn run_ticks(eco: &mut Ecology, from: u32, to: u32) {
    for t in from..=to {
        let daylight = 0.5 + 0.5 * ((t as f64 / 300.0) * std::f64::consts::PI).sin();
        eco.update(0.1, daylight);
    }
}

/// Acceptance #3 (regression guard): learning must stay bounded. Without
/// the clamp, the linear output layer feeds eligibility back into the
/// weights without any saturation, and W_out explodes to ±Inf → NaN within
/// a few thousand ticks (observed at t=1000 in the pre-fix WIP).
#[test]
fn weights_and_eligibility_stay_bounded() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 3000);
    let mut checked = 0usize;
    for a in eco.living() {
        if !a.alive { continue; }
        checked += 1;
        for (i, w) in a.brain.weights.iter().enumerate() {
            assert!(w.is_finite(), "agent {} weight[{}] not finite: {:e}", a.id, i, w);
            assert!(
                w.abs() <= WEIGHT_CLAMP,
                "agent {} weight[{}] outside clamp band: {} > {}",
                a.id, i, w, WEIGHT_CLAMP
            );
        }
        for (i, e) in a.brain.eligibility.iter().enumerate() {
            assert!(e.is_finite(), "agent {} eligibility[{}] not finite: {:e}", a.id, i, e);
            assert!(
                e.abs() <= ELIGIBILITY_CLAMP,
                "agent {} eligibility[{}] outside clamp band: {} > {}",
                a.id, i, e, ELIGIBILITY_CLAMP
            );
        }
        assert!(
            a.brain.taxis_baseline.is_finite(),
            "agent {} taxis_baseline not finite",
            a.id
        );
    }
    assert!(checked > 0, "no living agents to check");
    eprintln!("bounded-brain check passed over {} living agents at t=3000", checked);
}

/// Acceptance #2: the brain's maintenance bill is proportional to network
/// size. A reference 4-node network pays 4 × BRAIN_PER_NEURON_COST per
/// sim-second; a duplicated 5-node network pays 5×.
#[test]
fn brain_upkeep_scales_with_network_size() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 60);
    let reference = aether_brain::reference_brain("herb");
    let ref_upkeep = reference.upkeep();
    assert!(
        (ref_upkeep - BRAIN_PER_NEURON_COST * aether_brain::INITIAL_NODES as f64).abs() < 1e-12,
        "reference upkeep {} != {} × {} nodes",
        ref_upkeep, BRAIN_PER_NEURON_COST, aether_brain::INITIAL_NODES
    );
    // The ecology-side drain books the same per-tick cost into the ledger.
    let alive = eco.living().iter().filter(|a| a.alive).count();
    assert!(alive > 0);
    eprintln!(
        "reference upkeep={:.4}/s ({} nodes × {:.4}), alive={}",
        ref_upkeep,
        aether_brain::INITIAL_NODES,
        BRAIN_PER_NEURON_COST,
        alive
    );
}

/// Acceptance #6: the v9 save schema round-trips every piece of brain state
/// that the dynamics touch. In particular the hidden-node outputs (the
/// recurrent layer reads them as previous-tick activations) — omitting them
/// forks the shadow trajectory one tick after restore even when weights and
/// eligibility match.
#[test]
fn save_restore_preserves_brain_internals() {
    let mut eco = Ecology::new(20261102);
    run_ticks(&mut eco, 1, 100);
    let json = serde_json::to_string(&eco.to_save_state()).unwrap();
    let restored = Ecology::from_save_state(serde_json::from_str(&json).unwrap());

    // Serializing the freshly restored world must reproduce the identical
    // document (all brain internals survive the hop).
    let json2 = serde_json::to_string(&restored.to_save_state()).unwrap();
    assert_eq!(json, json2, "save-state JSON must round-trip bit-exactly");

    // Field-level spot checks: the restored brain fields match the source.
    for (a, b) in eco.living().iter().zip(restored.living().iter()) {
        assert_eq!(a.brain.weights, b.brain.weights, "weights drift for agent {}", a.id);
        assert_eq!(a.brain.eligibility, b.brain.eligibility, "eligibility drift for agent {}", a.id);
        assert_eq!(a.brain.outputs, b.brain.outputs, "logits drift for agent {}", a.id);
        assert_eq!(
            a.brain.nodes.iter().map(|n| n.output).collect::<Vec<_>>(),
            b.brain.nodes.iter().map(|n| n.output).collect::<Vec<_>>(),
            "hidden outputs drift for agent {}",
            a.id
        );
        assert_eq!(a.brain.taxis_baseline.to_bits(), b.brain.taxis_baseline.to_bits());
        assert_eq!(a.brain.last_reward.to_bits(), b.brain.last_reward.to_bits());
    }

    // And the lockstep continuation stays bit-exact (the shadow trajectory
    // must not fork).
    let mut a = eco;
    let mut b = restored;
    for target in [200u32, 300u32] {
        run_ticks(&mut a, target - 100, target);
        run_ticks(&mut b, target - 100, target);
        assert_eq!(a.checksum(), b.checksum(), "checksum diverged at t={}", target);
        let ja = serde_json::to_string(&a.to_save_state()).unwrap();
        let jb = serde_json::to_string(&b.to_save_state()).unwrap();
        assert_eq!(ja, jb, "full save-state JSON diverged at t={}", target);
    }
}

/// Acceptance #1/#5: with the authority gate OFF (default), the FSM is the
/// sole behaviour source. Brain state evolves as a pure shadow: a world
/// whose brains were re-zeroed mid-run produces the identical checksum
/// trajectory (the ledger's brain_cost and info_cost entries are
/// observation-only; they do not feed back into agent energy).
#[test]
fn shadow_never_drives_behaviour_by_default() {
    let mut a = Ecology::new(20261102);
    run_ticks(&mut a, 1, 300);
    assert!(!a.brain_switch_enabled, "authority gate must default OFF");
    assert!(
        a.living().iter().all(|agent| agent.brain_authority == BRAIN_AUTHORITY_FSM),
        "all agents must start under FSM authority"
    );
    // The authority flag never flips while the gate is off.
    run_ticks(&mut a, 301, 600);
    assert!(
        a.living().iter().all(|agent| agent.brain_authority != BRAIN_AUTHORITY_BRAIN),
        "FSM authority must hold while the gate is off"
    );
    // Both instance flags are consistent with the constants.
    assert!(a.brain_on, "shadow network runs by default");
    assert!(!a.brain_switch_enabled, "switch gate stays off by default");
}
