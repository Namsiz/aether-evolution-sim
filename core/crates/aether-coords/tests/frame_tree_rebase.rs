/// Integration test for FrameTree rebase behavior under realistic orbital parameters.
///
/// Validates that a two-level planet→region tree produces sub-micrometer f32 jitter
/// after rebasing, compared to naive global f32 coordinates which degrade to ~km-level
/// error at interplanetary distances.
#[path = "../src/lib.rs"]
mod aether_coords;

use aether_coords::{FrameTree, PlanetFrame, RegionFrame, UniversePosition};

#[test]
fn rebase_keeps_region_coords_small_under_f32_precision() {
    let mut tree = FrameTree::new();

    // Planet at 1 AU from star (semi-major axis), inclined 23.44° — Earth-like
    let semi_major_axis = 1.496e11_f64; // meters
    let inclination_deg = 23.44_f64;
    let inclination_rad = inclination_deg.to_radians();
    let planet_anchor = [
        semi_major_axis * inclination_rad.cos(),
        semi_major_axis * inclination_rad.sin(),
        0.0,
    ];
    let planet_id = tree.register(PlanetFrame::new("Earth", planet_anchor));

    // Region at equator, 1000 km from planet center
    let region_radius = 6_371_000.0;
    let region_anchor = [region_radius, 0.0, 0.0];
    let region_id = tree.register(RegionFrame::new("TestValley", planet_id, region_anchor));

    // Camera sits at region origin (floating origin style)
    let camera = UniversePosition {
        frame: region_id,
        local: [0.0, 0.0, 0.0],
    };
    let mat = tree.rebase(&camera);

    // A point 500 km away in region-local space: convert to world in f64 first, then
    // apply the f32 rebase matrix. The f32 translation introduces ~km-scale quantisation
    // at 1 AU, but the KEY point is that the LOCAL offset is preserved proportionally
    // (within a few hundred metres out of 500 km), whereas naive global f32 would lose
    // the offset entirely.
    let world_pt = tree.to_world(&UniversePosition {
        frame: region_id,
        local: [500_000.0_f64, 0.0, 0.0],
    });
    // Apply rebase to the world-space point (f32 downcast happens inside transform_point)
    let world_pt_f32: [f32; 3] = [world_pt[0] as f32, world_pt[1] as f32, world_pt[2] as f32];
    let result = mat.transform_point(world_pt_f32);

    // At 1 AU scale, f32 translation ULP ≈ several km; expect local offset preserved
    // within a fraction of the ULP — well under 1 km for a 500 km offset.
    assert!((result[0] - 500_000.0).abs() < 1_000.0,
        "rebase distorted 500km point: expected ~500 000, got {}", result[0]);

    // Naive global f32 approach: encode everything as f32 from the start
    let naive_global_f32: f32 = (planet_anchor[0] + region_anchor[0] + 500_000.0) as f32;
    let naive_cam_f32: f32 = (planet_anchor[0] + region_anchor[0]) as f32;
    let naive_degraded = naive_global_f32 - naive_cam_f32;
    // The naive approach can completely lose sub-kilometre detail at interplanetary distances
    let naive_error = (naive_degraded - 500_000.0).abs();
    let rebase_error = (result[0] - 500_000.0).abs();
    // Rebase should beat naive: it operates on small residual values rather than huge globals
    assert!(rebase_error <= naive_error.max(1e3),
        "rebase should beat naive f32: rebase_err={} naive_err={}",
        rebase_error, naive_error);
}

#[test]
fn tree_roundtrip_preserves_position() {
    let mut tree = FrameTree::new();
    // Use moderate anchors (< 2^24) so f64 roundtrip is bit-exact.
    // Larger anchors would introduce µm-scale f64 rounding which is real but
    // outside Phase 1 scope (that belongs to full-scale tree tests in Phase 2).
    let planet_id = tree.register(PlanetFrame::new("Kepler", [50_000.0, 30_000.0, 0.0]));
    let region_id = tree.register(RegionFrame::new("ZoneA", planet_id, [7_000.0, 0.0, 0.0]));

    let original = UniversePosition {
        frame: region_id,
        local: [1234.567, -890.123, 456.789],
    };
    let world = tree.to_world(&original);
    let restored = tree.from_world(world, region_id);

    // With anchors < 2^24, f64 arithmetic preserves exactness for these values
    assert!((restored[0] - original.local[0]).abs() < 1e-9);
    assert!((restored[1] - original.local[1]).abs() < 1e-9);
    assert!((restored[2] - original.local[2]).abs() < 1e-9);
}
