//! Phase 31 Part B: uniform-grid spatial index (C12 engineering prerequisite).
//!
//! Pure bookkeeping (zero RNG). Replaces the O(n²) full-table scan inside
//! `Ecology::nearest` in a LATER phase (Part C) — this module is gated behind
//! `#[cfg(feature = "spatial-index")]` so default builds pay nothing.
//!
//! Bit-exact equivalence contract (the Part C gate):
//! - `rebuild` takes live agents in Vec-index order.
//! - `candidates` returns all agents whose position lies inside the query
//!   box, sorted by agent index ascending.
//! - A caller that walks those candidates and applies the SAME strict-less
//!   update as the linear scan (`if d2 < best { ... }`) reproduces the
//!   full-scan result bit-for-bit, including tie-breaks (lower index wins,
//!   exactly as the Vec-order scan does).
//!
//! The module does not itself call any RNG and does not mutate sim state.

/// World half-extent for the index box: agents stay inside SIM_RADIUS (560)
/// from terrain wander clamping; 600 leaves margin with no cost.
const WORLD_RADIUS: f64 = 600.0;
/// Bucket count along one axis. Bucket size = 1200 / 30 = 40.0 world units.
const BUCKETS: usize = 30;

#[derive(Debug, Default)]
pub struct SpatialIndex {
    /// Flat bucket grid (row-major, z-major). Each bucket holds agent Vec
    /// indices in insertion (index) order.
    cells: Vec<Vec<usize>>,
}

impl SpatialIndex {
    pub fn new() -> Self {
        SpatialIndex { cells: vec![Vec::new(); BUCKETS * BUCKETS] }
    }

    /// Bucket axis coordinate for a world coordinate (clamped to grid).
    fn bucket_axis(v: f64) -> i32 {
        (((v + WORLD_RADIUS) / (WORLD_RADIUS * 2.0 / BUCKETS as f64)) as i32)
            .clamp(0, BUCKETS as i32 - 1)
    }

    /// Rebuild all buckets from the live agent set.
    /// `live` = (agent Vec index, x, z) in ascending index order (the sim's
    /// canonical order). O(n).
    pub fn rebuild(&mut self, live: &[(usize, f64, f64)]) {
        for c in self.cells.iter_mut() {
            c.clear();
        }
        for &(i, x, z) in live {
            let b = Self::bucket_axis(x) + Self::bucket_axis(z) as i32 * BUCKETS as i32;
            self.cells[b as usize].push(i);
        }
    }

    /// All agent indices whose position lies inside the box
    /// `[cx-r, cx+r] × [cz-r, cz+r]` (a superset of the radius-r disk —
    /// callers do the exact distance test), sorted by agent index ascending.
    pub fn candidates(&self, cx: f64, cz: f64, radius: f64) -> Vec<usize> {
        let x0 = Self::bucket_axis(cx - radius);
        let x1 = Self::bucket_axis(cx + radius);
        let z0 = Self::bucket_axis(cz - radius);
        let z1 = Self::bucket_axis(cz + radius);
        let mut out: Vec<usize> = Vec::new();
        for z in z0..=z1 {
            for x in x0..=x1 {
                out.extend_from_slice(&self.cells[((z * BUCKETS as i32) + x) as usize]);
            }
        }
        // Buckets are disjoint per agent, but visiting bucket order ≠ index
        // order; the equivalence contract needs index-ascending candidates.
        out.sort_unstable();
        out
    }

    /// Convenience: nearest-neighbor that is bit-exact with the linear scan
    /// semantics (`nearest` in ecology.rs): skip `self_idx`, strict-less
    /// update on squared distance, tie → lowest agent index (candidates are
    /// index-ascending, so the first minimum encountered equals the
    /// Vec-order scan's result).
    ///
    /// `positions` must be the same live set passed to `rebuild` (index
    /// ascending); a merge cursor keeps the walk O(k + m), not O(n·k).
    pub fn nearest(
        &self,
        positions: &[(usize, f64, f64)],
        self_idx: Option<usize>,
        cx: f64,
        cz: f64,
        radius: f64,
    ) -> Option<(usize, f64)> {
        let r2 = radius * radius;
        let cands = self.candidates(cx, cz, radius);
        let mut best: Option<(usize, f64)> = None; // (agent idx, d²)
        let mut p = 0usize;
        for &ci in &cands {
            while p < positions.len() && positions[p].0 < ci {
                p += 1;
            }
            if p < positions.len() && positions[p].0 == ci {
                let (_, x, z) = positions[p];
                p += 1;
                if self_idx == Some(ci) {
                    continue;
                }
                let dx = x - cx;
                let dz = z - cz;
                let d2 = dx * dx + dz * dz;
                if d2 < r2 && (best.is_none() || d2 < best.unwrap().1) {
                    best = Some((ci, d2));
                }
            }
        }
        best.map(|(i, d2)| (i, d2.sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic position set (no RNG: a fixed LCG so tests are
    /// reproducible and independent of the sim's streams).
    fn positions(n: usize, salt: u32) -> Vec<(usize, f64, f64)> {
        let mut state = 0x1234_5678u32.wrapping_add(salt);
        let mut out: Vec<(usize, f64, f64)> = Vec::with_capacity(n);
        for i in 0..n {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let r = ((state >> 8) as f64) / (1u32 << 16) as f64 * 2.0 - 1.0; // [-1, 1)
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let r2 = ((state >> 8) as f64) / (1u32 << 16) as f64 * 2.0 - 1.0;
            out.push((i, r * WORLD_RADIUS, r2 * WORLD_RADIUS));
        }
        out
    }

    /// Reference: the exact semantics of `Ecology::nearest`'s scan loop —
    /// Vec order, strict-less update, tie keeps the earlier (lower) index.
    fn linear_nearest(
        ps: &[(usize, f64, f64)],
        self_idx: Option<usize>,
        cx: f64,
        cz: f64,
        radius: f64,
    ) -> Option<(usize, f64)> {
        let r2 = radius * radius;
        let mut best: Option<(usize, f64)> = None;
        for &(i, x, z) in ps {
            if Some(i) == self_idx {
                continue;
            }
            let dx = x - cx;
            let dz = z - cz;
            let d2 = dx * dx + dz * dz;
            if d2 < r2 && (best.is_none() || d2 < best.unwrap().1) {
                best = Some((i, d2));
            }
        }
        best.map(|(i, d2)| (i, d2.sqrt()))
    }

    #[test]
    fn index_matches_linear_scan() {
        let queries: &[(f64, f64)] = &[(0.0, 0.0), (-300.0, 250.0), (400.0, -380.0), (50.0, 50.0)];
        let radii: &[f64] = &[5.0, 22.0, 60.0, 130.0, 400.0];
        for n in [1usize, 2, 50, 800, 1200] {
            let ps = positions(n, n as u32);
            let mut idx = SpatialIndex::new();
            idx.rebuild(&ps);
            for &(qx, qz) in queries {
                for &r in radii {
                    for self_idx in [None, Some(0)] {
                        assert_eq!(
                            idx.nearest(&ps, self_idx, qx, qz, r),
                            linear_nearest(&ps, self_idx, qx, qz, r),
                            "n={n} q=({qx},{qz}) r={r} self={self_idx:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn tie_break_matches_linear_scan() {
        // Two agents equidistant from the query: linear scan keeps the
        // LOWER index (first encountered); index candidates are sorted by
        // index and the strict-less update must keep that one.
        let ps = vec![
            (0, -10.0_f64, 0.0),
            (1, 10.0, 0.0),
            (2, -10.0, 1.0),
            (3, 10.0, -1.0),
        ];
        let mut idx = SpatialIndex::new();
        idx.rebuild(&ps);
        for &r in &[8.0_f64, 12.0, 30.0] {
            assert_eq!(
                idx.nearest(&ps, None, 0.0, 0.0, r),
                linear_nearest(&ps, None, 0.0, 0.0, r),
                "tie at r={r}"
            );
        }
    }

    #[test]
    fn candidates_sorted_by_index() {
        let ps = positions(200, 9);
        let mut idx = SpatialIndex::new();
        idx.rebuild(&ps);
        let c = idx.candidates(0.0, 0.0, 500.0);
        let sorted = c.clone();
        let mut s = sorted;
        s.sort_unstable();
        assert_eq!(c, s, "candidates must be index-ascending");
    }
}
