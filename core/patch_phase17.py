#!/usr/bin/env python3
"""Phase 17 surgical patcher: cataclysm (extinction events) + recovery dynamics.

Applies the Phase 17 diff to five existing files via exact search/replace
blocks. Every block must match exactly `count` times or the script aborts
WITHOUT writing anything (atomic per-file: all edits of a file are applied to
an in-memory string first).

New files (NOT touched by this script, shipped alongside):
  core/crates/aether-sim-ecology/src/cataclysm.rs
  core/crates/aether-sim-ecology/tests/cataclysm_test.rs
  living/src/renderer-cataclysm.mjs
  living/test_cataclysm.mjs

Usage:
    cd ~/workspace/world
    python3 core/patch_phase17.py --check   # dry run: verify all blocks match
    python3 core/patch_phase17.py --apply   # apply
"""
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent  # world/

DRY_RUN = False


def s(text):
    """Dedent-free literal block (keeps the author's exact indentation)."""
    return text


def apply_edits(rel_path, edits):
    path = ROOT / rel_path
    text = path.read_text(encoding="utf-8")
    for i, (search, replace, count) in enumerate(edits):
        n = text.count(search)
        if n == 0 and count == 1 and text.count(replace) >= 1:
            continue  # already applied (idempotent re-run)
        if n != count:
            raise SystemExit(
                f"FATAL: {rel_path} edit#{i}: expected {count} occurrence(s), "
                f"found {n}. Aborting, file NOT modified.\n"
                f"--- search block ---\n{search}\n--------------------")
        text = text.replace(search, replace)
    if not DRY_RUN:
        path.write_text(text, encoding="utf-8")
    print(f"  {'[check] ' if DRY_RUN else ''}patched {rel_path}: "
          f"{len(edits)} edits OK")


# ---------------------------------------------------------------------------
# lib.rs / main.rs — declare the new module in both module trees
# ---------------------------------------------------------------------------

LIB = "core/crates/aether-sim-ecology/src/lib.rs"
LIB_EDITS = []  # already applied (architect rewrote lib.rs directly)

MAIN = "core/crates/aether-sim-ecology/src/main.rs"
MAIN_EDITS = [
    (s('''mod niche; // Phase 16: clade niche axis + competitive exclusion
'''),
     s('''mod niche; // Phase 16: clade niche axis + competitive exclusion
mod cataclysm; // Phase 17: cataclysms + recovery dynamics
'''), 1),
]

# ---------------------------------------------------------------------------
# save_state.rs — v3 → v4
# ---------------------------------------------------------------------------

SAVESTATE = "core/crates/aether-sim-ecology/src/save_state.rs"

SAVESTATE_EDITS = []  # already applied (architect rewrote save_state.rs directly)

ECOLOGY = "core/crates/aether-sim-ecology/src/ecology.rs"

ECOLOGY_EDITS = [
    # 1. Grid constants become crate-visible (cataclysm.rs computes cell
    #    centers with the same values — no constant duplication).
    (s('''const GRID_N: u32 = 76;
const GRID_SPAN: f64 = 1080.0;
const CELL: f64 = GRID_SPAN / (GRID_N as f64);
'''),
     s('''pub(crate) const GRID_N: u32 = 76;
pub(crate) const GRID_SPAN: f64 = 1080.0;
pub(crate) const CELL: f64 = GRID_SPAN / (GRID_N as f64);
'''), 1),

    # 2. Phase 17 constants + Cataclysm struct after the event-kind constants.
    (s('''pub const EVENT_KIND_SPECIATION: u8 = 0;
pub const EVENT_KIND_EXTINCTION: u8 = 1;
pub const EVENT_KIND_COMPETITIVE_EXCLUSION: u8 = 2;
'''),
     s('''pub const EVENT_KIND_SPECIATION: u8 = 0;
pub const EVENT_KIND_EXTINCTION: u8 = 1;
pub const EVENT_KIND_COMPETITIVE_EXCLUSION: u8 = 2;

// ----- Phase 17: cataclysms + recovery dynamics --------------------------------
//
// Cataclysms are experimenter-injected extinction events (C-ABI only — the
// engine never fires one on its own). All bookkeeping is deterministic: no
// RNG draws anywhere in the cataclysm path, and with no active cataclysm
// every environment multiplier is exactly 1.0 (f64 x*1.0 is bit-exact), so
// the checksum trajectory of cataclysm-free runs is untouched.
/// kind 0: asteroid impact — local resource wipe + mass mortality + dust.
pub const CATACLYSM_IMPACT: u8 = 0;
/// kind 1: climate shift — global grass-growth throttle, no direct mortality.
pub const CATACLYSM_CLIMATE: u8 = 1;
/// Energy struck off every agent inside the impact radius (scaled by
/// severity). Deaths reuse the regular die() path (cause "cataclysm").
pub const CATACLYSM_DAMAGE: f64 = 60.0;
/// Peak additive metabolic-stress factor at full dust cover: drain formulas
/// scale with the genome's metabolism gene, so this yields a measurable
/// K-Pg-style selection against high-maintenance phenotypes.
pub const CATACLYSM_STRESS_SCALE: f64 = 1.5;

/// One cataclysm record: the injection event plus its live effect state.
/// `remaining` counts down once per substep in `update`; `active` flips to
/// false at zero. Persisted in SaveState v4 via the compact JSON string.
#[derive(Clone, Debug)]
pub struct Cataclysm {
    pub id: u32,
    pub kind: u8,
    /// Injection tick (ecology tick_count at trigger time).
    pub tick: u32,
    pub x: f64,
    pub z: f64,
    pub radius: f64,
    pub severity: f64,
    /// Impact only: dust multiplier recovers linearly over this many ticks.
    pub dust_recovery_ticks: u32,
    /// Climate only: additive growth modifier (negative = throttled).
    pub climate_delta: f64,
    /// Last tick the effect can act (tick + duration at injection).
    pub ends_at: u32,
    /// Live countdown; > 0 ⇔ active.
    pub remaining: u32,
    pub active: bool,
}
'''), 1),

    # 3. Ecology struct: grass/resource fields become crate-visible so
    #    cataclysm.rs can apply the blast directly (render accessors stay).
    (s('''    grass: Vec<f32>,
    grass_p: [Vec<f32>; GRASS_N],
'''),
     s('''    // Phase 17: cataclysm.rs mutates the resource fields directly (impact
    // blast); the public accessors remain authoritative for reads.
    pub(crate) grass: Vec<f32>,
    pub(crate) grass_p: [Vec<f32>; GRASS_N],
'''), 1),

    (s('''    /// Decomposers deposit into it while feeding on carcasses; it decays each
    /// tick and multiplies grass logistic growth. Clamped to [0, 1].
    mineral_field: Vec<f32>,
'''),
     s('''    /// Decomposers deposit into it while feeding on carcasses; it decays each
    /// tick and multiplies grass logistic growth. Clamped to [0, 1].
    pub(crate) mineral_field: Vec<f32>,
'''), 1),

    # 4. Ecology struct: cataclysm table + id counter.
    (s('''    pub clade_burdens: Vec<(u32, f64)>,
}
'''),
     s('''    pub clade_burdens: Vec<(u32, f64)>,
    /// Phase 17: cataclysm injection log (history + live effect state), in
    /// trigger order. See `cataclysm.rs`; persisted in SaveState v4.
    pub cataclysms: Vec<Cataclysm>,
    next_cataclysm_id: u32,
}
'''), 1),

    # 5. Both constructors init the new fields (2 occurrences).
    (s('''            clade_burdens: Vec::new(),
'''),
     s('''            clade_burdens: Vec::new(),
            cataclysms: Vec::new(),
            next_cataclysm_id: 1,
'''), 2),

    # 6. die() becomes crate-visible (cataclysm.rs reuses the regular path).
    (s('''    fn die(&mut self, idx: usize, cause: &'static str) {
'''),
     s('''    pub(crate) fn die(&mut self, idx: usize, cause: &'static str) {
'''), 1),

    # 7. update() top: cataclysm env modulation (dust × daylight).
    (s('''    pub fn update(&mut self, dt: f64, daylight: f64) {
        if dt <= 0.0 { return; }
        self.daylight = daylight;
'''),
     s('''    pub fn update(&mut self, dt: f64, daylight: f64) {
        if dt <= 0.0 { return; }
        // Phase 17: cataclysm env modulation. Dust darkens the daylight (also
        // drives the resting/plasticity thresholds downstream); the climate
        // shift throttles grass growth below. With no active cataclysm both
        // multipliers are exactly 1.0 — f64 x*1.0 is bit-exact, so cataclysm-
        // free runs keep their checksum trajectory untouched.
        let (dust_mult, grow_mult) = self.cataclysm_env_mods();
        self.daylight = daylight * dust_mult;
'''), 1),

    # 8. Per-substep cataclysm countdown.
    (s('''            self.time += h;
            self.tick_count += 1;
'''),
     s('''            self.time += h;
            self.tick_count += 1;
            // Phase 17: per-substep cataclysm countdown (pure bookkeeping,
            // zero RNG).
            self.tick_cataclysms();
'''), 1),

    # 9. season_meta: dust metabolic stress (herb/pred/scav/omn drain path).
    (s('''            let season_meta = [1.0f64, 1.15, 0.9, 0.75][self.season as usize];
            self.season_meta = season_meta;
'''),
     s('''            let season_meta = [1.0f64, 1.15, 0.9, 0.75][self.season as usize];
            // Phase 17: impact-dust metabolic stress — the additive factor
            // scales with (1 − dust), so every drain (∝ genome metabolism)
            // rises while the sky is dark (K-Pg-style high-maintenance
            // penalty). Exactly 1.0 without active impacts → bit-exact no-op.
            self.season_meta = season_meta * (1.0 + self.cataclysm_stress_scale());
'''), 1),

    # 10. Grass growth: climate multiplier + dust-darkened daylight.
    (s('''            let grow = 0.0056 * season_grass * (0.18 + 0.82 * daylight) * (1.0 + self.rain * 0.02);
'''),
     s('''            // Phase 17: grow_mult = climate-shift throttle; self.daylight
            // already carries the dust darkening.
            let grow = 0.0056 * season_grass * grow_mult * (0.18 + 0.82 * self.daylight) * (1.0 + self.rain * 0.02);
'''), 1),

    # 11. DECO drain: dust stress (its season table bypasses self.season_meta).
    (s('''        let total_drain = g_metabolism * (0.12 + 0.25 * activity) * crowd_penalty * season_meta_deco * h * eff_metab_mod * (1.0 + niche_burden);
'''),
     s('''        // Phase 17: impact-dust metabolic stress also hits DECO (its drain
        // path uses its own season table, bypassing self.season_meta).
        let cat_stress = 1.0 + self.cataclysm_stress_scale();
        let total_drain = g_metabolism * (0.12 + 0.25 * activity) * crowd_penalty * season_meta_deco * h * eff_metab_mod * (1.0 + niche_burden) * cat_stress;
'''), 1),

    # 12. cataclysm_to_json serializer next to clade_event_to_json.
    (s('''fn clade_event_to_json(e: &CladeSpeciationEvent) -> String {
    let children = e.child_clade_ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(",");
    format!(
        "{{\\"tick\\":{},\\"parentCladeId\\":{},\\"childCladeIds\\":[{}],\\"geneticDistance\\":{},\\"kind\\":{}}}",
        e.tick, e.parent_clade_id, children, e.genetic_distance, e.event_kind,
    )
}
'''),
     s('''fn clade_event_to_json(e: &CladeSpeciationEvent) -> String {
    let children = e.child_clade_ids.iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(",");
    format!(
        "{{\\"tick\\":{},\\"parentCladeId\\":{},\\"childCladeIds\\":[{}],\\"geneticDistance\\":{},\\"kind\\":{}}}",
        e.tick, e.parent_clade_id, children, e.genetic_distance, e.event_kind,
    )
}

/// Compact camelCase JSON view of a Cataclysm for the SaveState v4 string
/// field (same hand-rolled format! trick as clade_to_json — no extra serde
/// visitors in the wasm binary).
fn cataclysm_to_json(c: &Cataclysm) -> String {
    format!(
        "{{\\"id\\":{},\\"kind\\":{},\\"tick\\":{},\\"x\\":{},\\"z\\":{},\\"radius\\":{},\\"severity\\":{},\\"dustRecoveryTicks\\":{},\\"climateDelta\\":{},\\"endsAt\\":{},\\"remaining\\":{},\\"active\\":{}}}",
        c.id, c.kind, c.tick, c.x, c.z, c.radius, c.severity,
        c.dust_recovery_ticks, c.climate_delta, c.ends_at, c.remaining,
        if c.active { 1 } else { 0 },
    )
}
'''), 1),

    # 13. to_save_state: serialize the cataclysm table.
    (s('''            // Phase 16: burden map as compact JSON (see save_state::SavedBurdens).
            CladeNicheBurdens: save_state::SavedBurdens(format!(
                "[{}]",
                self.clade_burdens.iter()
                    .map(|(id, b)| format!("[{},{}]", id, b))
                    .collect::<Vec<String>>()
                    .join(",")
            )),
'''),
     s('''            // Phase 16: burden map as compact JSON (see save_state::SavedBurdens).
            CladeNicheBurdens: save_state::SavedBurdens(format!(
                "[{}]",
                self.clade_burdens.iter()
                    .map(|(id, b)| format!("[{},{}]", id, b))
                    .collect::<Vec<String>>()
                    .join(",")
            )),
            // Phase 17: cataclysm history + live effect state (compact JSON,
            // see save_state::SavedCataclysms).
            Cataclysms: save_state::SavedCataclysms(format!(
                "[{}]",
                self.cataclysms.iter().map(cataclysm_to_json).collect::<Vec<String>>().join(",")
            )),
            NextCataclysmId: self.next_cataclysm_id,
'''), 1),

    # 14. from_save_state: parse the cataclysm table (pre-v4 → empty).
    (s('''        let next_clade_id = state.NextCladeId;
        let last_clade_spec_tick = state.LastCladeSpecTick;
'''),
     s('''        let next_clade_id = state.NextCladeId;
        let last_clade_spec_tick = state.LastCladeSpecTick;

        // Phase 17: cataclysm table. Pre-v4 saves carry the serde default
        // (empty string) → empty history, id counter 1 (pre-cataclysm world,
        // bit-identical continuation).
        let cataclysms: Vec<Cataclysm> =
            serde_json::from_str::<serde_json::Value>(&state.Cataclysms.0)
                .ok()
                .and_then(|v| v.as_array().cloned())
                .map(|arr| {
                    arr.iter().filter_map(|c| {
                        Some(Cataclysm {
                            id: c.get("id")?.as_u64()? as u32,
                            kind: c.get("kind")?.as_u64()? as u8,
                            tick: c.get("tick")?.as_u64()? as u32,
                            x: c.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            z: c.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            radius: c.get("radius").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            severity: c.get("severity").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            dust_recovery_ticks: c.get("dustRecoveryTicks").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                            climate_delta: c.get("climateDelta").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            ends_at: c.get("endsAt").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                            remaining: c.get("remaining").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                            active: c.get("active").and_then(|v| v.as_u64()).unwrap_or(0) == 1,
                        })
                    }).collect::<Vec<Cataclysm>>()
                })
                .unwrap_or_default();
        let next_cataclysm_id = if state.NextCataclysmId > 0 { state.NextCataclysmId } else { 1 };
'''), 1),

    # 15. from_save_state struct literal: wire the cataclysm fields in.
    (s('''            species_clades,
            next_clade_id,
            speciation_events,
            last_clade_spec_tick,
            clade_burdens,
            seed: state.seed,
'''),
     s('''            species_clades,
            next_clade_id,
            speciation_events,
            last_clade_spec_tick,
            clade_burdens,
            cataclysms,
            next_cataclysm_id,
            seed: state.seed,
'''), 1),
]

# ---------------------------------------------------------------------------
# wasm.rs — C-ABI exports + record layout
# ---------------------------------------------------------------------------

WASM = "core/crates/aether-sim-ecology/src/wasm.rs"

WASM_EDITS = [
    # 1. Record constants after the clade/event record sizes.
    (s('''pub const CLADE_RECORD_SIZE: usize = 40;
pub const EVENT_RECORD_SIZE: usize = 56;
'''),
     s('''pub const CLADE_RECORD_SIZE: usize = 40;
pub const EVENT_RECORD_SIZE: usize = 56;
/// Phase 17: cataclysm record (sim_get_cataclysm, JS-provided out buffer —
/// NOT part of the scratch arena, so SCRATCH_SIZE is unchanged):
///   [id u32][kind u32][tick u32][ends_at u32][remaining u32]
///   [x f32][z f32][radius f32][severity f32][active u32]
pub const CATACLYSM_RECORD_SIZE: usize = 40;
pub const CATACLYSM_CAP: usize = 64;
'''), 1),

    # 2. Four sim_* exports after sim_get_event.
    (s('''        n_children.max(1) as u32
    }

    /// Serialize the current state as Snapshot v3 JSON. Returns a pointer to a
'''),
     s('''        n_children.max(1) as u32
    }

    // ─── Phase 17: cataclysm injection & env readout ─────────────────────────

    /// Inject a cataclysm. kind: 0 = impact (x/z/radius/severity used),
    /// 1 = climate shift (severity doubles as the negative growth delta).
    /// Returns the cataclysm id (≥ 1), or 0 on null handle / invalid params.
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_trigger_cataclysm(
        sim: *mut WasmSimCore,
        kind: u32,
        x: f64,
        z: f64,
        radius: f64,
        severity: f64,
        duration: u32,
    ) -> u32 {
        if sim.is_null() { return 0; }
        (*sim).eco.trigger_cataclysm(kind as u8, x, z, radius, severity, duration)
    }

    /// Total number of cataclysms ever injected (history + live). JS reads
    /// records with sim_get_cataclysm, capping at CATACLYSM_CAP = 64.
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_cataclysm_count(sim: *mut WasmSimCore) -> u32 {
        if sim.is_null() { return 0; }
        (*sim).eco.cataclysm_count()
    }

    /// Fill the 40-byte cataclysm record at `out`:
    ///   [id u32][kind u32][tick u32][ends_at u32][remaining u32]
    ///   [x f32][z f32][radius f32][severity f32][active u32]
    /// Returns 1 on success, 0 on null/bounds failure (never traps).
    ///
    /// # Safety
    /// `sim` from `sim_new`; `out` writable for CATACLYSM_RECORD_SIZE bytes.
    #[no_mangle]
    pub unsafe extern "C" fn sim_get_cataclysm(sim: *mut WasmSimCore, idx: u32, out: *mut u8) -> u32 {
        if sim.is_null() || out.is_null() { return 0; }
        let core = &mut *sim;
        let Some(c) = core.eco.cataclysms.get(idx as usize) else { return 0; };
        let mut rec = [0u8; CATACLYSM_RECORD_SIZE];
        rec[0..4].copy_from_slice(&c.id.to_le_bytes());
        rec[4..8].copy_from_slice(&(c.kind as u32).to_le_bytes());
        rec[8..12].copy_from_slice(&c.tick.to_le_bytes());
        rec[12..16].copy_from_slice(&c.ends_at.to_le_bytes());
        rec[16..20].copy_from_slice(&c.remaining.to_le_bytes());
        rec[20..24].copy_from_slice(&(c.x as f32).to_le_bytes());
        rec[24..28].copy_from_slice(&(c.z as f32).to_le_bytes());
        rec[28..32].copy_from_slice(&(c.radius as f32).to_le_bytes());
        rec[32..36].copy_from_slice(&(c.severity as f32).to_le_bytes());
        rec[36..40].copy_from_slice(&(c.active as u32).to_le_bytes());
        std::ptr::copy_nonoverlapping(rec.as_ptr(), out, CATACLYSM_RECORD_SIZE);
        1
    }

    /// Current environment multipliers: writes dust (f64) then growth (f64)
    /// to `out`; returns 2. JS reads these every frame for the dust overlay.
    ///
    /// # Safety
    /// `sim` from `sim_new`; `out` writable for 2 × f64.
    #[no_mangle]
    pub unsafe extern "C" fn sim_env_mods(sim: *mut WasmSimCore, out: *mut f64) -> u32 {
        if sim.is_null() || out.is_null() { return 0; }
        let (dust, grow) = (*sim).eco.cataclysm_env_mods();
        *out.add(0) = dust;
        *out.add(1) = grow;
        2
    }

    /// Serialize the current state as Snapshot v3 JSON. Returns a pointer to a
'''), 1),
]

# ---------------------------------------------------------------------------
# living/src/wasm-viewer.mjs — injection keys, dust overlay, marks, HUD
# ---------------------------------------------------------------------------

VIEWER = "living/src/wasm-viewer.mjs"

VIEWER_EDITS = [
    # 1. histogram import gains the spectrogram geometry constants.
    (s('''import { createEvolutionHistogramOverlay, createSpectrogramOverlay, DECO_HUE_MIN, DECO_HUE_SPAN, DECO_HUE_BUCKETS, HIST_FRAME_TICKS, getTickFromFrame } from './renderer-evolution-histogram.mjs';
'''),
     s('''import { createEvolutionHistogramOverlay, createSpectrogramOverlay, DECO_HUE_MIN, DECO_HUE_SPAN, DECO_HUE_BUCKETS, HIST_FRAME_TICKS, getTickFromFrame,
  SPECTROGRAM_HISTORY_FRAMES, SPECTROGRAM_WIDTH } from './renderer-evolution-histogram.mjs';
'''), 1),

    # 2. cataclysm renderer import after the niche import.
    (s('''import { createNicheOverlay, buildNicheCloud } from './renderer-niche.mjs';
'''),
     s('''import { createNicheOverlay, buildNicheCloud } from './renderer-niche.mjs';
import { createDustOverlay, drawCataclysmVline, drawCataclysmHline, drawCataclysmGlyph,
  formatCataclysmHud, CATACLYSM_IMPACT, CATACLYSM_CLIMATE,
} from './renderer-cataclysm.mjs';
'''), 1),

    # 3. sim handle gains the four Phase 17 ABI wrappers.
    (s('''    readPhylogeny() {
      return readPhylogenyState(E, sim,
        scratchPtr + OFF_CLADES, scratchPtr + OFF_EVENTS, scratchPtr + OFF_CLADE_NAME, CLADE_NAME_CAP);
    },
'''),
     s('''    readPhylogeny() {
      return readPhylogenyState(E, sim,
        scratchPtr + OFF_CLADES, scratchPtr + OFF_EVENTS, scratchPtr + OFF_CLADE_NAME, CLADE_NAME_CAP);
    },
    // Phase 17: cataclysm 注入与读出（旧 wasm 无导出时为 undefined，调用方需先判
    // catSupported）。out 指针由调用方提供（boot 期追加的 1 页内存）。
    triggerCataclysm(kind, x, z, radius, severity, duration) {
      return E.sim_trigger_cataclysm(sim, kind, x, z, radius, severity, duration) | 0;
    },
    cataclysmCount() { return E.sim_cataclysm_count(sim) | 0; },
    getCataclysm(i, outPtr) { return E.sim_get_cataclysm(sim, i, outPtr) | 0; },
    envMods(outPtr) { return E.sim_env_mods(sim, outPtr) | 0; },
'''), 1),

    # 4. boot: grow one page for the ABI out-buffer (before any views exist).
    (s('''  setStatus('加载 WASM 生态内核…');
  const instance = await loadWasmModule();
  const handle = createSimHandle(instance);
'''),
     s('''  setStatus('加载 WASM 生态内核…');
  const instance = await loadWasmModule();
  // Phase 17: cataclysm ABI 出参缓冲 —— 追加 1 页 wasm 内存（旧 wasm 无这些导出
  // 时跳过；memory.grow 会使旧视图失效，ensureViews() 按 byteLength 自动重建）。
  const catSupported = typeof instance.exports.sim_trigger_cataclysm === 'function';
  let catBufPtr = 0;
  if (catSupported) {
    const pagesBefore = instance.exports.memory.buffer.byteLength / 65536;
    instance.exports.memory.grow(1);
    catBufPtr = pagesBefore * 65536;
  }
  const handle = createSimHandle(instance);
'''), 1),

    # 5. cataclysm state + pull/mark functions after the niche overlay block.
    (s('''  // Phase 16: 生态位空间 overlay（U 键切换；复用 phylogeny 的 60-tick 拉取节奏）。
  const nicheOverlay = phyloSupported ? createNicheOverlay() : null;
  let nicheOn = true;
'''),
     s('''  // Phase 16: 生态位空间 overlay（U 键切换；复用 phylogeny 的 60-tick 拉取节奏）。
  const nicheOverlay = phyloSupported ? createNicheOverlay() : null;
  let nicheOn = true;

  // Phase 17: 灾变系统 —— 注入（C/X 键）、烟尘 overlay、spectrogram/phylogeny
  // 时间标记与 HUD 历史（旧 wasm 无导出时整体降级隐藏）。
  const dustOverlay = catSupported ? createDustOverlay() : null;
  let catHistory = [];      // sim_get_cataclysm 拉取快照（旧 → 新）
  let lastCatPull = -Infinity;

  function pullCataclysms(force = false) {
    if (!catSupported) return;
    const t = handle.tick();
    if (!force && t - lastCatPull < 60) return;
    lastCatPull = t;
    const count = Math.min(handle.cataclysmCount(), 64);
    const list = [];
    for (let i = 0; i < count; i++) {
      if (handle.getCataclysm(i, catBufPtr) === 0) break;
      const dv = new DataView(instance.exports.memory.buffer, catBufPtr, 40);
      list.push({
        id: dv.getUint32(0, true),
        kind: dv.getUint32(4, true),
        tick: dv.getUint32(8, true),
        endsAt: dv.getUint32(12, true),
        remaining: dv.getUint32(16, true),
        x: dv.getFloat32(20, true),
        z: dv.getFloat32(24, true),
        radius: dv.getFloat32(28, true),
        severity: dv.getFloat32(32, true),
        active: dv.getUint32(36, true) === 1,
      });
    }
    catHistory = list;
  }

  // 灾变时间标记：phylogeny（时间轴横向 → 竖线 + 图标）与 spectrogram（时间轴
  // 纵向 → 横线 + 图标）。overlay 自身的 render 会整面重绘并抹掉标记，因此这里
  // 每帧在其后补画（几条虚线 + 小图标，开销可忽略）。
  function drawCataclysmMarks() {
    if (!catSupported || catHistory.length === 0) return;
    if (phyloOverlay && phyloOverlay.visible && phyloOverlay.graph) {
      const g = phyloOverlay.graph;
      const w = 360, h = 280; // 与 createPhylogenyOverlay 默认几何同步
      const PAD = { top: 26, bottom: 20, left: 84, right: 30 };
      const ctx = phyloOverlay.canvas.getContext('2d');
      const span = Math.max(1, g.currentTick - g.rootTick);
      for (const c of catHistory) {
        if (c.tick < g.rootTick) continue;
        const x = PAD.left + ((c.tick - g.rootTick) / span) * (w - PAD.left - PAD.right);
        drawCataclysmVline(ctx, c.kind, x, PAD.top, h - PAD.bottom);
        drawCataclysmGlyph(ctx, c.kind, x, PAD.top + 12);
      }
    }
    if (specOverlay && specOverlay.visible && specNewestTick > 0) {
      const ctx = specOverlay.canvas.getContext('2d');
      for (const c of catHistory) {
        const back = (specNewestTick - c.tick) / HIST_FRAME_TICKS;
        if (back < 0 || back >= SPECTROGRAM_HISTORY_FRAMES) continue;
        const y = Math.round(back);
        drawCataclysmHline(ctx, c.kind, y, 12, SPECTROGRAM_WIDTH - 12);
        drawCataclysmGlyph(ctx, c.kind, SPECTROGRAM_WIDTH - 22, Math.max(9, y - 7));
      }
    }
  }
'''), 1),

    # 6. Boot-time baseline pull after the niche overlay warmup.
    (s('''  if (nicheOverlay) {
    nicheOverlay.setVisible(true);
    renderNiche();
  }
'''),
     s('''  if (nicheOverlay) {
    nicheOverlay.setVisible(true);
    renderNiche();
  }
  if (catSupported) {
    pullCataclysms(true); // 预热阶段无灾变，仅建立空基线
    dustOverlay.setDust(1);
  }
'''), 1),

    # 7. C / X injection keys after the F (fossil) branch.
    (s('''    } else if (e.code === 'KeyF' && phyloOverlay) {
      fossilOn = !fossilOn;
      renderPhylo();
      toast(`fossil highlight ${fossilOn ? 'ON' : 'OFF'}（F 切换）`);
'''),
     s('''    } else if (e.code === 'KeyF' && phyloOverlay) {
      fossilOn = !fossilOn;
      renderPhylo();
      toast(`fossil highlight ${fossilOn ? 'ON' : 'OFF'}（F 切换）`);
    } else if (e.code === 'KeyC' && catSupported) {
      // Phase 17: 演示小行星 —— 固定参数（地图中心偏移），实验者手动注入。
      const id = handle.triggerCataclysm(CATACLYSM_IMPACT, 140, -90, 170, 0.9, 600);
      if (id > 0) {
        pullCataclysms(true);
        toast(`☄ 小行星撞击已注入 (#${id}) · 尘幕 600t`);
      } else {
        toast('注入失败（参数无效）');
      }
    } else if (e.code === 'KeyX' && catSupported) {
      // Phase 17: 演示气候突变 —— 全图草地生长 ×0.5，持续 1200t。
      const id = handle.triggerCataclysm(CATACLYSM_CLIMATE, 0, 0, 0, 0.5, 1200);
      if (id > 0) {
        pullCataclysms(true);
        toast(`🌡 气候突变已注入 (#${id}) · 生长 ×0.5 · 1200t`);
      } else {
        toast('注入失败（参数无效）');
      }
'''), 1),

    # 8. Per-frame: cataclysm pull (60-tick gated) + dust overlay concentration.
    (s('''      if (histOn && histFrame % HIST_EVERY === 0) drawHist();
      histFrame += 1;
      // Phase 15: 每 60 sim-tick 拉一次 clade/event（内部有节奏门控）。
      if (phyloOverlay) pullPhylogeny();
    }
'''),
     s('''      if (histOn && histFrame % HIST_EVERY === 0) drawHist();
      histFrame += 1;
      // Phase 15: 每 60 sim-tick 拉一次 clade/event（内部有节奏门控）。
      if (phyloOverlay) pullPhylogeny();
      // Phase 17: 灾变拉取（60-tick 门控）+ 每帧烟尘浓度读数。
      if (catSupported) {
        pullCataclysms();
        handle.envMods(catBufPtr);
        dustOverlay.setDust(new DataView(instance.exports.memory.buffer, catBufPtr, 16).getFloat64(0, true));
      }
    }
'''), 1),

    # 9. Draw the time marks after the ghost pass (overlays self-redraw and
    #    would wipe them otherwise).
    (s('''    updateGhosts(clock.elapsedTime);
    renderer.render(scene, camera);
'''),
     s('''    updateGhosts(clock.elapsedTime);
    if (catSupported) drawCataclysmMarks(); // overlay 自绘之后补画灾变标记
    renderer.render(scene, camera);
'''), 1),

    # 10. HUD: active badge + recent-5 history line.
    (s('''      // Phase 15: 物种谱系行 —— clade 名单（≤5 个直接列名，否则计数）+ 事件数。
      let cladeLine = '';
      if (phyloOverlay && phyloClades.length > 0) {
        const alive = phyloClades.filter((c) => !c.extinct).length;
        const names = phyloClades
          .map((c) => (c.extinct ? `†${c.name}` : c.name));
        const list = names.length <= 5 ? names.join(' · ') : `${alive} alive / ${names.length} total`;
        cladeLine = `<span style="color:#c9a0ff">clades: ${list} · ${phyloEvents.length} event${phyloEvents.length === 1 ? '' : 's'}</span><br>`;
      }
'''),
     s('''      // Phase 15: 物种谱系行 —— clade 名单（≤5 个直接列名，否则计数）+ 事件数。
      let cladeLine = '';
      if (phyloOverlay && phyloClades.length > 0) {
        const alive = phyloClades.filter((c) => !c.extinct).length;
        const names = phyloClades
          .map((c) => (c.extinct ? `†${c.name}` : c.name));
        const list = names.length <= 5 ? names.join(' · ') : `${alive} alive / ${names.length} total`;
        cladeLine = `<span style="color:#c9a0ff">clades: ${list} · ${phyloEvents.length} event${phyloEvents.length === 1 ? '' : 's'}</span><br>`;
      }
      // Phase 17: 活跃灾变徽标 + 最近 5 条历史。
      let catLine = '';
      if (catSupported && catHistory.length > 0) {
        handle.envMods(catBufPtr);
        const dust = new DataView(instance.exports.memory.buffer, catBufPtr, 16).getFloat64(0, true);
        catLine = formatCataclysmHud(catHistory, dust);
      }
'''), 1),

    # 11. Wire catLine into the HUD html.
    (s('''        decoLine +
        cladeLine +
'''),
     s('''        decoLine +
        cladeLine +
        catLine +
'''), 1),
]

# ---------------------------------------------------------------------------

ALL = [
    (LIB, LIB_EDITS),
    (MAIN, MAIN_EDITS),
    (SAVESTATE, SAVESTATE_EDITS),
    (ECOLOGY, ECOLOGY_EDITS),
    (WASM, WASM_EDITS),
    (VIEWER, VIEWER_EDITS),
]


def main():
    global DRY_RUN
    mode = sys.argv[1] if len(sys.argv) > 1 else "--check"
    if mode == "--check":
        DRY_RUN = True
    elif mode != "--apply":
        print(__doc__)
        return 2
    for rel, edits in ALL:
        apply_edits(rel, edits)
    print("ALL PATCHES " + ("VERIFIED (dry run)" if DRY_RUN else "APPLIED"))
    return 0


if __name__ == "__main__":
    sys.exit(main())
