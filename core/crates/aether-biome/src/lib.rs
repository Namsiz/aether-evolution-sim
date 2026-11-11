//! aether-biome — Phase 4-A spatial biomes framework.
//!
//! A `Biome` describes a geographic region with its own suitability modifier,
//! carrying capacities, and home-range modulation factor. `BiomeRegistry`
//! maintains a list of biomes and provides fast O(1)-ish lookup by world
//! coordinate (circular pre-filter + linear scan; ≤10 biomes expected).
//!
//! Zero external dependencies (`std` only), wasm32 target compatible.
// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single biome region in the simulation world.
#[derive(Debug, Clone)]
pub struct Biome {
    /// Unique opaque identifier.
    pub id: u32,
    /// Human-readable name used in debug output.
    pub name: &'static str,
    /// Center X in world coordinates (f64 matches universe frame).
    pub center_x: f64,
    /// Center Z in world coordinates.
    pub center_z: f64,
    /// Influence radius. Agents outside this circle are not affected.
    pub radius: f64,
    /// Global suitability multiplier applied on top of the terrain suit field.
    /// `0.5` halves effective carrying capacity; `2.0` doubles it.
    pub suit_modifier: f64,
    /// Per-species carrying-capacity override (uses current species counts
    /// against these limits instead of the global MAX_* constants).
    pub max_herb: usize,
    pub max_pred: usize,
    pub max_scav: usize,
    pub max_omn: usize,
    pub max_fish: Option<usize>,
    /// Home-range radius modulation multiplier. Multipled into the base range
    /// before `_clampHome` is invoked. Default `1.0`.
    pub home_range_mod: f64,
}

impl Biome {
    /// Returns true if `(x, z)` lies inside this biome's influence circle.
    #[inline]
    pub fn point_in_biome(&self, x: f64, z: f64) -> bool {
        let dx = x - self.center_x;
        let dz = z - self.center_z;
        dx * dx + dz * dz <= self.radius * self.radius
    }

    /// Maps world coordinates to an index into `suit_by` / `moist_buf`.
    /// Returns `None` if the coordinates fall outside the local grid bounds.
    /// Local grid spans `[-radius, +radius]` in both axes, mapped uniformly
    /// onto `[0, grid_n)` cells.
    #[inline]
    pub fn local_grid_idx(&self, gx: f64, gz: f64, grid_n: usize) -> Option<usize> {
        // Map [-radius, +radius] → [0, grid_n-1]
        let half_cells = (grid_n as f64 - 1.0) * 0.5;
        let i = ((gx - self.center_x) / self.radius * half_cells + half_cells).floor() as isize;
        let j = ((gz - self.center_z) / self.radius * half_cells + half_cells).floor() as isize;
        if i < 0 || j < 0 || i as usize >= grid_n || j as usize >= grid_n {
            return None;
        }
        Some((j as usize) * grid_n + i as usize)
    }
}

/// Additional per-biome suitability/moisture buffers. Allocated lazily when a
/// biome overrides the default full-world behavior.
#[derive(Debug, Default, Clone)]
pub struct BiomeField {
    /// Local grid resolution (e.g. 32 → 32×32 cells covering the biome radius).
    pub grid_n: usize,
    /// Pre-computed `terrain_suit * suit_modifier` per cell.
    pub suit_by: Vec<f32>,
    /// Moisture buffer (mirrors the ecology-level moist field but local).
    pub moist_buf: Vec<f32>,
}

/// Registry holding all defined biomes plus the implicit "full-world" default.
///
/// Lookup strategy: iterate all biomes in order, checking `point_in_biome`
/// first. With ≤10 biomes this is effectively O(1). The full-world biome
/// (id=0, radius=SIM_RADIUS) acts as fallback when no finer biome claims the
/// coordinate.
#[derive(Debug, Clone)]
pub struct BiomeRegistry {
    /// User-defined biomes (sorted by priority; later entries override earlier
    /// ones when centers overlap).
    biomes: Vec<Biome>,
}

impl BiomeRegistry {
    /// Creates an empty registry (no biomes). Callers can later push via
    /// `add_biome` or use the full-world fallback directly.
    pub fn new() -> Self {
        Self {
            biomes: Vec::new(),
        }
    }

    /// Adds a biome. Caller owns ordering semantics.
    pub fn add_biome(&mut self, b: Biome) {
        self.biomes.push(b);
    }

    /// Registered biomes in insertion order (Phase 6 save/restore support:
    /// `find_biome` resolves overlaps last-match-wins, so restoring must
    /// re-add in the same order).
    pub fn biomes(&self) -> &[Biome] {
        &self.biomes
    }

    /// Returns the biome that contains `(x, z)`, or `None` if none match.
    /// Iterates in insertion order; last match wins when regions overlap.
    #[inline]
    pub fn find_biome(&self, x: f64, z: f64) -> Option<&Biome> {
        let mut result: Option<&Biome> = None;
        for (i, b) in self.biomes.iter().enumerate() {
            if b.point_in_biome(x, z) {
                result = Some(&self.biomes[i]);
            }
        }
        result
    }

    /// Same as `find_biome` but returns a mutable reference (not typical, kept
    /// for symmetry). Panics if called while any immutable ref exists — Rust
    /// borrow checker handles this automatically at call sites.
    #[inline]
    pub fn find_biome_mut(&mut self, x: f64, z: f64) -> Option<&mut Biome> {
        let idx = self
            .biomes
            .iter()
            .enumerate()
            .filter(|(_, b)| b.point_in_biome(x, z))
            .map(|(i, _)| i)
            .last();
        idx.map(|i| &mut self.biomes[i])
    }

    /// Given an agent at `(x, z)`, returns its species-specific capacity
    /// limit. Falls back to the global const if no biome applies.
    pub fn max_for_kind(&self, kind: &str, x: f64, z: f64) -> usize {
        if let Some(b) = self.find_biome(x, z) {
            match kind {
                "herb" => b.max_herb,
                "pred" => b.max_pred,
                "scav" => b.max_scav,
                "omn" => b.max_omn,
                "fish" => b.max_fish.unwrap_or(usize::MAX),
                _ => unreachable!(),
            }
        } else {
            // Fallback to global caps. The caller must supply these separately;
            // we return usize::MAX here to signal "no override".
            usize::MAX
        }
    }

    /// Returns the home-range modulation factor for an agent at `(x, z)`.
    pub fn home_range_mod(&self, x: f64, z: f64) -> f64 {
        self.find_biome(x, z)
            .map(|b| b.home_range_mod)
            .unwrap_or(1.0)
    }
}

impl Default for BiomeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_in_biome_exact_center() {
        let b = Biome {
            id: 1,
            name: "meadow",
            center_x: 0.0,
            center_z: 0.0,
            radius: 100.0,
            suit_modifier: 1.5,
            max_herb: 200,
            max_pred: 50,
            max_scav: 20,
            max_omn: 30,
            max_fish: None,
            home_range_mod: 1.2,
        };
        assert!(b.point_in_biome(0.0, 0.0));
        assert!(b.point_in_biome(99.0, 0.0));
        assert!(b.point_in_biome(100.0, 0.0)); // on boundary — closed disk
        assert!(!b.point_in_biome(101.0, 0.0));
        assert!(b.point_in_biome(70.0, 70.0)); // diag √(9800)≈99 < 100
        assert!(!b.point_in_biome(71.0, 71.0)); // √(10082)≈100.4 > 100
    }

    #[test]
    fn point_in_biome_offset_center() {
        let b = Biome {
            id: 2,
            name: "valley",
            center_x: 200.0,
            center_z: -150.0,
            radius: 80.0,
            suit_modifier: 0.8,
            max_herb: 120,
            max_pred: 30,
            max_scav: 10,
            max_omn: 20,
            max_fish: None,
            home_range_mod: 0.9,
        };
        assert!(b.point_in_biome(200.0, -150.0));
        assert!(b.point_in_biome(280.0, -150.0)); // on edge of r=80 disc
        assert!(!b.point_in_biome(281.0, -150.0));
        // dz = 80 → on boundary; use dz = 81 for definitely outside
        assert!(!b.point_in_biome(200.0, -69.0));
    }

    #[test]
    fn local_grid_idx_boundary_cases() {
        let b = Biome {
            id: 3,
            name: "test",
            center_x: 0.0,
            center_z: 0.0,
            radius: 200.0,
            suit_modifier: 1.0,
            max_herb: 100,
            max_pred: 20,
            max_scav: 10,
            max_omn: 15,
            max_fish: None,
            home_range_mod: 1.0,
        };
        // Grid is centered on biome, spanning ±radius.
        // corner cells should be valid
        assert!(b.local_grid_idx(-200.0, -200.0, 32).is_some());
        assert!(b.local_grid_idx(200.0, 200.0, 32).is_some());
        // well outside
        assert!(b.local_grid_idx(-201.0, 0.0, 32).is_none());
        assert!(b.local_grid_idx(0.0, 250.0, 32).is_none());
        // center maps to roughly middle cell
        let idx = b.local_grid_idx(0.0, 0.0, 32).unwrap();
        assert_eq!(idx, 15 * 32 + 15);
    }

    #[test]
    fn registry_finds_closest_and_overlapping() {
        let mut reg = BiomeRegistry::new();
        reg.add_biome(Biome {
            id: 1,
            name: "a",
            center_x: -100.0,
            center_z: 0.0,
            radius: 60.0,
            suit_modifier: 1.2,
            max_herb: 100,
            max_pred: 20,
            max_scav: 10,
            max_omn: 15,
            max_fish: None,
            home_range_mod: 1.0,
        });
        reg.add_biome(Biome {
            id: 2,
            name: "b",
            center_x: 100.0,
            center_z: 0.0,
            radius: 60.0,
            suit_modifier: 0.7,
            max_herb: 80,
            max_pred: 15,
            max_scav: 8,
            max_omn: 12,
            max_fish: None,
            home_range_mod: 0.8,
        });
        // Inside biome A
        assert_eq!(reg.find_biome(-100.0, 0.0).unwrap().name, "a");
        // Inside biome B
        assert_eq!(reg.find_biome(100.0, 0.0).unwrap().name, "b");
        // Outside both → None
        assert!(reg.find_biome(0.0, 0.0).is_none());
    }

    #[test]
    fn registry_overlapping_last_wins() {
        let mut reg = BiomeRegistry::new();
        reg.add_biome(Biome {
            id: 1,
            name: "outer",
            center_x: 0.0,
            center_z: 0.0,
            radius: 200.0,
            suit_modifier: 1.0,
            max_herb: 500,
            max_pred: 100,
            max_scav: 50,
            max_omn: 80,
            max_fish: None,
            home_range_mod: 1.0,
        });
        reg.add_biome(Biome {
            id: 2,
            name: "inner",
            center_x: 0.0,
            center_z: 0.0,
            radius: 100.0,
            suit_modifier: 1.5,
            max_herb: 200,
            max_pred: 50,
            max_scav: 20,
            max_omn: 30,
            max_fish: None,
            home_range_mod: 1.3,
        });
        // Inner should win because it was added second (last-match-wins policy)
        let b = reg.find_biome(0.0, 0.0).unwrap();
        assert_eq!(b.name, "inner");
        assert_eq!(b.max_herb, 200);
        assert_eq!(b.home_range_mod, 1.3);
        // Edge case: just outside inner but still inside outer
        let b2 = reg.find_biome(101.0, 0.0).unwrap();
        assert_eq!(b2.name, "outer");
        assert_eq!(b2.max_herb, 500);
    }

    #[test]
    fn max_for_kind_fallback_to_usize_max() {
        let reg = BiomeRegistry::new();
        assert_eq!(reg.max_for_kind("herb", 9999.0, 0.0), usize::MAX);
    }

    #[test]
    fn home_range_mod_returns_default_outside_biomes() {
        let reg = BiomeRegistry::new();
        assert_eq!(reg.home_range_mod(0.0, 0.0), 1.0);
    }
}
