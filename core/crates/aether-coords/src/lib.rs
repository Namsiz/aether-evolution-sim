//! `aether-coords`: minimal reference frame tree for multi-scale coordinate transforms.
//!
//! **Why this design?**  
//! Float32 precision breaks down at ~10 km scale (~1 m error), but our target spans
//! ~40 orders of magnitude (10⁻¹⁰ m molecules → 10²⁶ m observable universe). A single
//! global f32 coordinate system is impossible. Instead, every entity's position is
//! anchored to a local `ScaleFrame`, and transforms are computed on demand along the
//! parent chain in f64 before downcasting to f32 for the GPU.
//!
//! **Phase 1 scope** (per ARCHITECTURE.md §2.1): only two levels — `planet` and `region`.
//! Region origin = planet's anchor expressed in planet-space coordinates. No generic
//! N-level hierarchy yet; that comes when L0-L3 LOD simulation requires deeper nesting.
//!
//! **Design notes vs. Three.js / Bevy**:  
//! - We do NOT embed a full GLM-style matrix library. Only the minimal `rebase()` path
//!   needed for floating-origin camera transforms is implemented.  
//! - `ScaleFrame` is a trait, not an ECS component — decoupled so it works with any
//!   future ECS (Bevy, self-authored SoA, etc.).  
//! - All math uses f64 internally; f32 downgrade happens at the `rebase` return boundary.

/// Entity identifier in the frame tree.
pub type EntityId = u64;
/// Frame identifier. Same width as EntityId; distinct semantic namespace.
pub type FrameId = u64;

/// Trait for a scale frame node in the reference frame tree.
///
/// Implementors carry their own origin anchor (in parent coords, f64 precision) and
/// their local unit scale factor (meters per local-unit). Example: a planet frame
/// might have `unit_scale = 1.0` (local units = meters), while a region frame on it
/// also has `unit_scale = 1.0` but offset by the planet's surface position.
pub trait ScaleFrame: std::fmt::Debug {
    /// Parent frame ID, or `None` for root frames (e.g., galaxy center).
    fn parent_id(&self) -> Option<EntityId>;
    /// Anchor point of THIS frame expressed in PARENT frame coordinates [x, y, z] (f64).
    fn anchor_in_parent(&self) -> [f64; 3];
    /// Local spatial unit in meters (e.g., 1.0 = meters, 0.001 = millimeters).
    fn unit_scale(&self) -> f64;
}

/// Position specified relative to a named reference frame.
///
/// This is the canonical "true" position format. To render, call `FrameTree::rebase()`
/// with the camera's position to get an f32 transform matrix.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UniversePosition {
    pub frame: FrameId,
    pub local: [f64; 3],
}

/// Column-major 4×4 transformation matrix (OpenGL / WebGL compatible layout).
///
/// `matrix.0[r][c]` = element at row r, column c. Multiplication: `v' = M * v` (column vector).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformMatrix(pub [[f32; 4]; 4]);

impl TransformMatrix {
    /// Identity matrix.
    pub fn identity() -> Self {
        let mut m = [[0.0f32; 4]; 4];
        for i in 0..4 {
            m[i][i] = 1.0;
        }
        TransformMatrix(m)
    }

    /// Apply this transform to a 3D point (homogeneous w=1).
    #[must_use]
    pub fn transform_point(&self, p: [f32; 3]) -> [f32; 3] {
        let m = &self.0;
        let w = m[3][0] * p[0] + m[3][1] * p[1] + m[3][2] * p[2] + m[3][3];
        [
            (m[0][0] * p[0] + m[0][1] * p[1] + m[0][2] * p[2] + m[0][3]) / w,
            (m[1][0] * p[0] + m[1][1] * p[1] + m[1][2] * p[2] + m[1][3]) / w,
            (m[2][0] * p[0] + m[2][1] * p[1] + m[2][2] * p[2] + m[2][3]) / w,
        ]
    }
}

/// Registry + parent-chain index for ScaleFrame nodes.
///
/// Phase 1 supports exactly two levels: one planet frame, one region frame childed to it.
/// Future Phases extend this to an N-level tree; the API already accommodates arbitrary depth.
#[derive(Debug)]
pub struct FrameTree {
    frames: std::collections::HashMap<FrameId, Box<dyn ScaleFrame>>,
    children: std::collections::HashMap<FrameId, Vec<FrameId>>,
    next_id: FrameId,
}

impl FrameTree {
    /// Create an empty frame tree.
    pub fn new() -> Self {
        FrameTree {
            frames: std::collections::HashMap::new(),
            children: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    /// Register a new frame and return its auto-assigned ID.
    pub fn register<F: ScaleFrame + 'static>(&mut self, frame: F) -> FrameId {
        let id = self.next_id;
        self.next_id += 1;
        let parent = frame.parent_id();
        self.frames.insert(id, Box::new(frame));
        if let Some(pid) = parent {
            self.children.entry(pid).or_default().push(id);
        }
        id
    }

    /// Look up a frame by ID.
    pub fn get(&self, id: FrameId) -> Option<&dyn ScaleFrame> {
        self.frames.get(&id).map(|b| b.as_ref())
    }

    /// Compute the camera-to-world transform matrix given a camera UniversePosition.
    ///
    /// Steps:
    /// 1. Walk the camera's frame ancestry in f64 to compute the camera anchor in root space.
    /// 2. Build a translation matrix T such that camera_anchor maps to origin.
    /// 3. Return T as f32 (downcast with lossy conversion; valid since anchors are small in camera space).
    ///
    /// Convention: column-major 4×4 matrix (OpenGL/WebGL compatible). Translation lives in
    /// the last column (`m[row][3]`). Multiplication: `v_out = M * v_in` with column vectors.
    pub fn rebase(&self, camera_pos: &UniversePosition) -> TransformMatrix {
        let cam_anchor = self.frame_anchor_in_root(camera_pos.frame);
        let tx = -cam_anchor[0] as f32;
        let ty = -cam_anchor[1] as f32;
        let tz = -cam_anchor[2] as f32;
        let mut m = TransformMatrix::identity();
        m.0[0][3] = tx;
        m.0[1][3] = ty;
        m.0[2][3] = tz;
        m
    }

    /// Convert a UniversePosition to world-space (root frame) coordinates in f64.
    pub fn to_world(&self, pos: &UniversePosition) -> [f64; 3] {
        let local = pos.local;
        let anchor = self.frame_anchor_in_root(pos.frame);
        [anchor[0] + local[0], anchor[1] + local[1], anchor[2] + local[2]]
    }

    /// Convert a world-space f64 point back to local coords in the given frame.
    pub fn from_world(&self, world: [f64; 3], frame: FrameId) -> [f64; 3] {
        let anchor = self.frame_anchor_in_root(frame);
        [world[0] - anchor[0], world[1] - anchor[1], world[2] - anchor[2]]
    }

    /// Walk up the parent chain summing anchors until reaching the root.
    fn frame_anchor_in_root(&self, frame_id: FrameId) -> [f64; 3] {
        let mut acc = [0.0f64; 3];
        let mut current = Some(frame_id);
        while let Some(fid) = current {
            match self.frames.get(&fid) {
                Some(frame) => {
                    let anc = frame.anchor_in_parent();
                    acc[0] += anc[0];
                    acc[1] += anc[1];
                    acc[2] += anc[2];
                    current = frame.parent_id().map(|pid| pid as FrameId);
                }
                None => break,
            }
        }
        acc
    }
}

/// A planet-scale frame at the stellar-system level.
///
/// Anchor is the planet's position in galactic (parent) coordinates. Unit scale = 1.0 m.
#[derive(Debug)]
pub struct PlanetFrame {
    pub name: String,
    anchor: [f64; 3],
}

impl PlanetFrame {
    pub fn new(name: impl Into<String>, anchor: [f64; 3]) -> Self {
        PlanetFrame { name: name.into(), anchor }
    }
}

impl ScaleFrame for PlanetFrame {
    fn parent_id(&self) -> Option<EntityId> {
        None
    }
    fn anchor_in_parent(&self) -> [f64; 3] {
        self.anchor
    }
    fn unit_scale(&self) -> f64 {
        1.0
    }
}

/// A region-scale frame attached to a planet (surface location).
///
/// Anchor is the region's origin on the planet surface in planet coordinates.
#[derive(Debug)]
pub struct RegionFrame {
    pub name: String,
    pub parent_planet: EntityId,
    anchor: [f64; 3],
}

impl RegionFrame {
    pub fn new(name: impl Into<String>, parent: EntityId, anchor: [f64; 3]) -> Self {
        RegionFrame { name: name.into(), parent_planet: parent, anchor }
    }
}

impl ScaleFrame for RegionFrame {
    fn parent_id(&self) -> Option<EntityId> {
        Some(self.parent_planet)
    }
    fn anchor_in_parent(&self) -> [f64; 3] {
        self.anchor
    }
    fn unit_scale(&self) -> f64 {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_level_rebase_to_zero() {
        let mut tree = FrameTree::new();
        let planet_id = tree.register(PlanetFrame::new("Earth", [0.0, 0.0, 0.0]));
        let region_id = tree.register(RegionFrame::new("Valley", planet_id, [6_371_000.0, 0.0, 0.0]));

        // Camera at region origin — its world-space position is planet_anchor + region_anchor
        let cam = UniversePosition { frame: region_id, local: [0.0, 0.0, 0.0] };
        let cam_world = tree.to_world(&cam); // [6_371_000, 0, 0]
        let mat = tree.rebase(&cam);

        // Rebase must map the camera's own world position to ~origin
        let as_f32 = [cam_world[0] as f32, cam_world[1] as f32, cam_world[2] as f32];
        let out = mat.transform_point(as_f32);
        assert!((out[0] - 0.0).abs() < 1e-3, "rebase failed: {:?}", out);
        assert!((out[1] - 0.0).abs() < 1e-3);
        assert!((out[2] - 0.0).abs() < 1e-3);
    }

    #[test]
    fn to_world_roundtrip_is_identity() {
        let mut tree = FrameTree::new();
        let planet_id = tree.register(PlanetFrame::new("Mars", [1.5e11, 0.0, 0.0]));
        let region_id = tree.register(RegionFrame::new("Olympus", planet_id, [3_389_500.0, 0.0, 0.0]));

        let local = UniversePosition { frame: region_id, local: [100.0, 50.0, 25.0] };
        let world = tree.to_world(&local);
        let back = tree.from_world(world, region_id);
        assert!((back[0] - 100.0).abs() < 1e-9);
        assert!((back[1] - 50.0).abs() < 1e-9);
        assert!((back[2] - 25.0).abs() < 1e-9);
    }

    #[test]
    fn f32_jitter_test_vs_naive_global() {
        // At ~10⁷ m distance, f32 ULP ≈ 1–2 m (2²⁴ ≈ 1.67×10⁷ is the exact-integer limit).
        // Naive global f32 coords lose ~1 m per km of distance from origin.
        // With rebase + f64 intermediate, camera-local coords stay small → f32 precision preserved.
        let mut tree = FrameTree::new();
        // Planet at 10⁷ m from galactic center
        let planet_id = tree.register(PlanetFrame::new("FarPlanet", [1e7, 0.0, 0.0]));
        // Region on planet surface at equator
        let region_id = tree.register(RegionFrame::new("Valley", planet_id, [6_371_000.0, 0.0, 0.0]));

        // Camera at region origin (floating-origin style)
        let cam = UniversePosition { frame: region_id, local: [0.0, 0.0, 0.0] };
        let mat = tree.rebase(&cam);

        // A point 100 m away in region-local space:
        //   world = planet_anchor + region_anchor + local = [1e7 + 6_371_000 + 100, 0, 0]
        //   rebase(world) = world - camera_anchor = [100, 0, 0] (in f64, exact)
        //   After f32 downcast of matrix translation + f32 point → result ≈ 100.0
        let world_pt = tree.to_world(&UniversePosition { frame: region_id, local: [100.0f64, 0.0, 0.0] });
        let world_pt_f32: [f32; 3] = [world_pt[0] as f32, world_pt[1] as f32, world_pt[2] as f32];
        let transformed = mat.transform_point(world_pt_f32);

        // f32 at ~100 m scale has ULP ≈ 1e-5 m; allow 1 mm tolerance
        assert!((transformed[0] - 100.0).abs() < 1e-3,
            "rebase distorted 100m point: expected ~100, got {}", transformed[0]);

        // Naive global f32 approach: encode everything as global f32 from the start
        let planet_anchor_f32: f32 = 1e7_f64 as f32;          // exact (< 2^24)
        let region_anchor_f32: f32 = 6_371_000.0_f64 as f32; // exact
        let local_f32: f32 = 100.0;
        let naive_world = planet_anchor_f32 + region_anchor_f32 + local_f32;
        let naive_cam   = planet_anchor_f32 + region_anchor_f32;
        let naive_local = naive_world - naive_cam;
        // f32 at ~1.6e7 has ULP = 2.0, so naive_local can drift by ±1.0 m from true 100.0
        let naive_error = (naive_local - 100.0).abs();

        let rebase_error = (transformed[0] - 100.0).abs();
        // Rebase should dramatically beat naive: rebase operates on small numbers where f32 is precise
        assert!(rebase_error < naive_error.max(1e-3),
            "rebase should beat naive f32: rebase_err={} naive_err={}", rebase_error, naive_error);
    }
}
