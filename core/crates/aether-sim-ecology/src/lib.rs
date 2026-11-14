pub mod ecology;
#[cfg(feature = "spatial-index")]
pub mod spatial_index; // Phase 31 Part B: uniform-grid C12 prerequisite
pub mod save_state;
pub mod terrain;
pub mod niche; // Phase 16: clade niche axis + competitive exclusion
pub mod cataclysm; // Phase 17: cataclysms + recovery dynamics
pub mod geology; // Phase 18: plate tectonics + eustatic sea level
pub mod biochemistry; // Phase 18: fast/slow nutrient pools + fertility feedback
pub mod culture; // Phase 37: knowledge tokens + niche construction
pub mod region; // Phase 38: multi-region frame tree + shared geo/nutrients
pub mod l0_shadow; // Phase 38: L0 statistical shadows (Lotka-Volterra aggregate)
pub mod wasm;
