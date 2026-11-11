//! Explicit test harness for `aether_biome::Biome` / `BiomeRegistry` lookup
//! semantics, kept outside lib.rs to stay independent of any future internal
//! refactors. Mirrors the public API contract described in Phase 4-A.

use aether_biome::{Biome, BiomeRegistry};

#[test]
fn point_in_biome_circle() {
    let b = Biome {
        id: 1, name: "test",
        center_x: 0.0, center_z: 0.0, radius: 100.0,
        suit_modifier: 1.0, max_herb: 100, max_pred: 30, max_scav: 10, max_omn: 20, max_fish: None,
        home_range_mod: 1.0,
    };
    assert!(b.point_in_biome(0.0, 0.0));
    assert!(b.point_in_biome(99.0, 0.0));
    // On boundary (closed disk): r == radius is included.
    assert!(b.point_in_biome(100.0, 0.0));
    assert!(!b.point_in_biome(100.1, 0.0));
    // Diagonal
    assert!(b.point_in_biome(70.0, 70.0)); // √9800 ≈ 98.99 < 100
    assert!(!b.point_in_biome(71.0, 71.0)); // √10082 ≈ 100.41 > 100
}

#[test]
fn local_grid_idx_maps_center_to_mid_cell() {
    let b = Biome {
        id: 1, name: "g",
        center_x: 0.0, center_z: 0.0, radius: 200.0,
        suit_modifier: 1.0, max_herb: 100, max_pred: 30, max_scav: 10, max_omn: 20, max_fish: None,
        home_range_mod: 1.0,
    };
    // grid_n=32 → indices [0..31]; center maps to floor(15.5)=15 in each dim.
    let idx = b.local_grid_idx(0.0, 0.0, 32).expect("center must map inside");
    assert_eq!(idx, 15 * 32 + 15);
    // Corners
    assert_eq!(b.local_grid_idx(-200.0, -200.0, 32), Some(0));
    assert_eq!(b.local_grid_idx(200.0, 200.0, 32), Some(31 * 32 + 31));
    // Outside
    assert!(b.local_grid_idx(-201.0, 0.0, 32).is_none());
    assert!(b.local_grid_idx(0.0, 250.0, 32).is_none());
}

#[test]
fn registry_last_match_wins_on_overlap() {
    let mut reg = BiomeRegistry::new();
    reg.add_biome(Biome {
        id: 1, name: "outer",
        center_x: 0.0, center_z: 0.0, radius: 200.0,
        suit_modifier: 1.0, max_herb: 500, max_pred: 100, max_scav: 50, max_omn: 80, max_fish: None,
        home_range_mod: 1.0,
    });
    reg.add_biome(Biome {
        id: 2, name: "inner",
        center_x: 0.0, center_z: 0.0, radius: 100.0,
        suit_modifier: 1.5, max_herb: 200, max_pred: 50, max_scav: 20, max_omn: 30, max_fish: None,
        home_range_mod: 1.3,
    });
    // Overlap at origin → inner wins (last-match policy)
    let b = reg.find_biome(0.0, 0.0).expect("should match");
    assert_eq!(b.name, "inner");
    assert_eq!(b.max_herb, 200);
    // Just outside inner but inside outer
    let b2 = reg.find_biome(101.0, 0.0).expect("should match outer");
    assert_eq!(b2.name, "outer");
    assert_eq!(b2.max_herb, 500);
}

#[test]
fn max_for_kind_returns_biome_values_or_max_fallback() {
    let mut reg = BiomeRegistry::new();
    reg.add_biome(Biome {
        id: 1, name: "small",
        center_x: 0.0, center_z: 0.0, radius: 50.0,
        suit_modifier: 0.8, max_herb: 60, max_pred: 12, max_scav: 5, max_omn: 8, max_fish: None,
        home_range_mod: 0.7,
    });
    assert_eq!(reg.max_for_kind("herb", 0.0, 0.0), 60);
    assert_eq!(reg.max_for_kind("pred", 0.0, 0.0), 12);
    assert_eq!(reg.max_for_kind("scav", 0.0, 0.0), 5);
    assert_eq!(reg.max_for_kind("omn", 0.0, 0.0), 8);
    // Far away → no matching biome
    assert_eq!(reg.max_for_kind("herb", 9999.0, 0.0), usize::MAX);
}

#[test]
fn home_range_mod_returns_default_outside_all_biomes() {
    let reg = BiomeRegistry::new();
    assert_eq!(reg.home_range_mod(0.0, 0.0), 1.0);
}
