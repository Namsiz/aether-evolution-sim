//! Snapshot export path: converts an Ecology instance into a PascalCase JSON
//! `Snapshot` packet consumable by the Three.js viewer on the JS side.
//!
//! Three layers of entry points:
//! 1. `snapshot_json(seed, ticks, daylight_series_json) -> String` (native & wasm)
//!    — stateless: builds a fresh Ecology and replays `ticks` from the seed.
//! 2. `#[wasm_bindgen] pub fn snapshot(...)` (wasm-only) — same packet via glue.
//! 3. `#[no_mangle] extern "C" sim_*` family (wasm-only, zero glue) — stateful
//!    `WasmSimCore` handle living across JS frames: `sim_step` advances N ticks,
//!    `sim_agents_flat` / `sim_mineral_resampled` fill JS-provided typed arrays
//!    without any JSON round-trip. This is the hot path the wasm viewer uses.

use aether_render_packet::{AgentView, Meta, Snapshot, StatsAgg, WorldState};
use crate::save_state::SaveState;

// ─────────────────────────────────────────────────────────────────────────────
// Phase 5 visual-genomics：genome → render scale / hue / ARGB 颜色。
// 纯函数，JSON AgentView 路径与 C-ABI flat scratch 路径共用，保证两条路径逐位一致。
//
// Hue 映射：genome.hue 是选型交配基因（初始 [0,1) 全随机，±0.03/代漂移，diff>0.35 禁配），
// 直接作色相会让所有物种变彩虹、物种身份丢失。因此：
//   final_hue = wrap01(anchor[kind] + (hue − 0.5) × 0.30 + shift[kind])
// 锚点定物种色相家族，genome.hue 提供 ±0.15 个体变异（亚群聚类 = 真实谱系可见），
// FISH 额外用 fin_color 基因表达蓝→青→绿漂移。
// ─────────────────────────────────────────────────────────────────────────────

/// genome.size 基因历史区间，映射到 render_scale [VIS_SCALE_MIN, VIS_SCALE_MAX]；
/// mutate 可把 size 推出历史区间（clamp [0.58, 1.7]），故归一化后截断到 [0, 1]。
pub const SIZE_GENE_MIN: f32 = 0.78;
pub const SIZE_GENE_MAX: f32 = 1.28;
pub const VIS_SCALE_MIN: f32 = 0.6;
pub const VIS_SCALE_MAX: f32 = 1.4;

/// genome.hue → 个体色相变异幅度（±0.15）。
const HUE_SPREAD: f64 = 0.30;

/// lineage 成员标记位（scratch SLOT_FLAGS bit0；JSON 路径由 Lineage 字段本身表达）。
pub const FLAG_LINEAGE: u32 = 1 << 0;

#[inline]
fn wrap01(h: f64) -> f64 {
    (h.fract() + 1.0).fract()
}

/// 物种色相锚点：herb 金黄 / pred 红褐 / scav 棕 / omn 绿 / fish 蓝 / deco 紫。
fn hue_anchor(kind: &str) -> f64 {
    match kind {
        crate::ecology::HERB => 0.11,
        crate::ecology::PRED => 0.04,
        crate::ecology::SCAV => 0.08,
        crate::ecology::OMN => 0.30,
        crate::ecology::FISH => 0.56,
        crate::ecology::DECO => 0.75,
        _ => 0.5,
    }
}

/// kind 级色相微移。FISH 的 fin_color 基因是唯一主表达通道（蓝→青→绿）；
/// 其余物种只在锚点基础上做微调（草案中的 DECO/OMN/SCAV/PRED 偏移已折叠进锚点）。
fn hue_shift(kind: &str, fin_color: f64) -> f64 {
    match kind {
        crate::ecology::FISH => fin_color * 0.3,
        crate::ecology::DECO => 0.06,
        crate::ecology::OMN => 0.04,
        crate::ecology::SCAV => -0.04,
        crate::ecology::PRED => -0.02,
        _ => 0.0,
    }
}

/// 每物种饱和度/明度：保持物种可辨识的同时让 hue 变异主导个体差异。
fn hue_sl(kind: &str) -> (f64, f64) {
    match kind {
        crate::ecology::HERB => (0.58, 0.67),
        crate::ecology::PRED => (0.39, 0.38),
        crate::ecology::SCAV => (0.48, 0.28),
        crate::ecology::OMN => (0.46, 0.69),
        crate::ecology::FISH => (0.56, 0.72),
        crate::ecology::DECO => (0.47, 0.70),
        _ => (0.5, 0.6),
    }
}

/// 最终视觉色相，wrap 到 [0, 1)。
#[inline]
pub fn visual_hue(kind: &str, hue: f64, fin_color: f64) -> f64 {
    wrap01(hue_anchor(kind) + (hue - 0.5) * HUE_SPREAD + hue_shift(kind, fin_color))
}

/// Phase 9: DECO 专属色相 —— deco_efficiency 直接驱动（基因组 hue 基因不再参与 DECO 可视色）。
/// efficiency 偏离 1.0 每单位 → shift 0.24，eff ∈ [0.5, 1.5] → shift ∈ [-0.12, +0.12]（clamp）。
/// 视觉语义：efficiency ≈ 0.5 → hue 0.63（深紫偏蓝），1.0 → 0.75（标准紫罗兰），1.5 → 0.87（品红）。
#[inline]
pub fn visual_hue_deco(deco_efficiency: f64) -> f64 {
    let eff_shift = ((deco_efficiency - 1.0) * 0.24).clamp(-0.12, 0.12);
    wrap01(0.75 + eff_shift)
}

/// deco_efficiency ∈ [0.5, 1.5] 线性映射到 u16 [0, 65535]（与 GenomeHueVisual 同量化域，
/// 但语义为效率代理值，非色相）。区间外截断。
#[inline]
pub fn deco_efficiency_to_u16(efficiency: f64) -> u16 {
    let t = ((efficiency - 0.5) / 1.0).clamp(0.0, 1.0);
    (t * 65535.0).round() as u16
}

/// Phase 10: DECO 效率代理值（u16）→ 直方图 bucket 下标，bucket i 覆盖
/// [DECO_HUE_MIN + i × span, DECO_HUE_MIN + (i+1) × span)，span = DECO_HUE_SPAN / buckets。
/// eff u16 端点分别落在首/末桶：65535 → hue 0.871 恰好是 span × buckets 边界，
/// floor 后可能等于 buckets，故 min(buckets − 1) 防止越界（wasm 里 slice 越界是 trap）。
#[inline]
pub fn deco_hue_bucket(eff_u16: u16, buckets: usize) -> usize {
    let hue = DECO_HUE_MIN + (eff_u16 as f64 / 65535.0) * DECO_HUE_SPAN;
    let idx = ((hue - DECO_HUE_MIN) / (DECO_HUE_SPAN / buckets as f64)) as usize;
    idx.min(buckets - 1)
}

/// genome.size → render_scale，[0.78, 1.28] → [0.6, 1.4]，区间外截断。
#[inline]
pub fn render_scale_from_size(size: f64) -> f32 {
    let t = ((size as f32 - SIZE_GENE_MIN) / (SIZE_GENE_MAX - SIZE_GENE_MIN)).clamp(0.0, 1.0);
    VIS_SCALE_MIN + t * (VIS_SCALE_MAX - VIS_SCALE_MIN)
}

/// HSL → sRGB，打包为 0xAARRGGBB（alpha = 0xFF）。与 CSS hsl() 语义一致。
pub fn hsl_to_rgb_u32(h: f64, s: f64, l: f64) -> u32 {
    let (r, g, b) = if s <= 0.0 {
        (l, l, l)
    } else {
        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        (hue_to_rgb(p, q, wrap01(h) + 1.0 / 3.0), hue_to_rgb(p, q, wrap01(h)), hue_to_rgb(p, q, wrap01(h) - 1.0 / 3.0))
    };
    let to8 = |v: f64| -> u32 { (v * 255.0).round().clamp(0.0, 255.0) as u32 };
    0xFF00_0000 | (to8(r) << 16) | (to8(g) << 8) | to8(b)
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 0.5 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

/// Genome → (render_scale, final_hue_u16, argb_hex, fin_visual, deco_eff_visual)。两条导出路径共用。
fn visual_fields(a: &crate::ecology::Agent) -> (f32, u16, u32, Option<f32>, Option<u16>) {
    let is_deco = a.kind == crate::ecology::DECO;
    let hue = if is_deco {
        visual_hue_deco(a.genome.deco_efficiency)
    } else {
        visual_hue(a.kind, a.genome.hue, a.genome.fin_color)
    };
    let (s, l) = hue_sl(a.kind);
    (
        render_scale_from_size(a.genome.size),
        (hue * 65535.0).round() as u16,
        hsl_to_rgb_u32(hue, s, l),
        if a.kind == crate::ecology::FISH { Some(a.genome.fin_color as f32) } else { None },
        if is_deco { Some(deco_efficiency_to_u16(a.genome.deco_efficiency)) } else { None },
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// Snapshot construction — owner-based (consumes Ecology) and reference-based
// (borrows, for the stateful wasm path) share one field list.
// ─────────────────────────────────────────────────────────────────────────────

impl From<crate::ecology::Ecology> for Snapshot {
    fn from(eco: crate::ecology::Ecology) -> Self {
        snapshot_from_ref(&eco)
    }
}

/// Borrowed variant of the owner-based conversion: copies agent fields out of
/// `&Agent` so the sim instance stays intact for further `update` calls.
fn agent_view(a: &crate::ecology::Agent) -> AgentView {
    // hunger = energy / max_energy, clamped to [0, 1].
    let max_e = if a.max_energy > 0.0 { a.max_energy } else { 1.0 };
    let hunger = ((a.energy / max_e).min(1.0)) as f32;
    let (vis_scale, hue_u16, color_hex, fin_vis, deco_eff_vis) = visual_fields(a);

    AgentView {
        id: a.id,
        kind: a.kind.to_string(),
        x: a.x,
        z: a.z,
        energy: a.energy as f32,
        age: a.age as f32,
        genome_speed: a.genome.speed as f32,
        genome_sense: a.genome.sense as f32,
        genome_size: a.genome.size as f32,
        genome_metabolism: a.genome.metabolism as f32,
        genome_lifespan: a.genome.lifespan as f32,
        genome_hue: a.genome.hue as f32,
        genome_bold: a.genome.bold as f32,
        genome_curious: a.genome.curious as f32,
        genome_social: a.genome.social as f32,
        genome_help_thr: a.genome.help_thr.map(|v| v as f32),
        genome_pref_sig: a.genome.pref_sig.map(|v| v as f32),
        genome_tol_sig: a.genome.tol_sig.map(|v| v as f32),
        lineage: a.lineage.map(|s| s.to_string()),
        state: a.state.to_string(),
        bite: a.bite as f32,
        repro: a.repro as f32,
        cargo: {
            let mut buf = vec![0.0f32; crate::ecology::GRASS_N];
            let n = a.cargo.len().min(crate::ecology::GRASS_N);
            buf[..n].copy_from_slice(&a.cargo[..n]);
            buf
        },
        hunger,
        image: a.image,
        help_cd: a.help_cd as f32,
        fear: a.fear as f32,
        think: a.think as f32,
        time_since_last_eat: 0.0,
        genome_size_visual: vis_scale,
        genome_hue_visual: hue_u16,
        genome_color_hex: color_hex,
        fin_color_visual: fin_vis,
        deco_eff_visual: deco_eff_vis,
    }
}

/// Reference-based twin of the previous owner-based `From<Ecology>`: keeps the
/// sim alive between snapshots so the wasm viewer can snapshot every frame.
fn snapshot_from_ref(eco: &crate::ecology::Ecology) -> Snapshot {
    // Alive agents sorted by ascending id (deterministic order for renderer).
    let mut agent_views: Vec<AgentView> = eco
        .agents
        .iter()
        .filter(|a| a.alive)
        .map(agent_view)
        .collect();
    agent_views.sort_by_key(|v| v.id);

    // Grass/suit/moist/shade — clone from accessor slices into owned Vecs.
    let grass = eco.grass().to_vec();
    let grass_by_species: Vec<Vec<f32>> =
        (0..crate::ecology::GRASS_N).map(|sp| eco.grass_p(sp).to_vec()).collect();
    let suit = eco.suit().to_vec();
    let moist = eco.moist().to_vec();
    let shade = eco.shade().to_vec();

    // v3 mineral field: full + downsampled (38×38, 4×4 block average).
    let mineral_full: Vec<f32> = eco.mineral_field().to_vec();
    let mineral_resampled = downsample_block_average(&mineral_full, 76, 38);

    // Live counts from the agent list itself (not history) so Snapshot.agents
    // and Snapshot.stats always agree — roundtrip test asserts this.
    let (mut h, mut p, mut sc, mut om, mut f, mut d) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    for a in &agent_views {
        match a.kind.as_str() {
            crate::ecology::HERB => h += 1,
            crate::ecology::PRED => p += 1,
            crate::ecology::SCAV => sc += 1,
            crate::ecology::OMN => om += 1,
            crate::ecology::FISH => f += 1,
            crate::ecology::DECO => d += 1,
            _ => {}
        }
    }
    let s = eco.stats();
    let stats = StatsAgg {
        tick_count: eco.tick_count(),
        season: eco.season,
        herb_count: h,
        pred_count: p,
        scav_count: sc,
        omn_count: om,
        fish_count: f,
        deco_count: d,
        total_agents: h + p + sc + om + f + d,
        plant_total: s.plant as f32,
        plant_by: s.plant_by.iter().map(|&v| v as f32).collect(),
        births: eco.births,
        deaths: eco.deaths,
        kills: eco.kills,
        starved: eco.starved,
        aged: eco.aged,
        // Phase 9: DECO efficiency distribution. Use the live count `d` computed above to
        // disambiguate "no DECO alive" (mean=0.0, std=0.0) from a real population.
        deco_eff_mean: if d == 0 { None } else { Some(s.deco_eff_mean) },
        deco_eff_std: if d < 2 { None } else { Some(s.deco_eff_std) },
    };

    Snapshot {
        meta: Meta::default(),
        world: WorldState {
            tick_count: eco.tick_count(),
            season: eco.season,
            daylight: eco.daylight as f32,
            grid_n: eco.grid_n,
            grass,
            grass_by_species,
            suit,
            moist,
            shade,
            mineral_field: Some(mineral_full),
            mineral_field_resampled: Some(mineral_resampled),
        },
        agents: agent_views,
        stats,
    }
}

/// 4×4 block-average downsample: input is src_len×src_len, output dst_len×dst_len.
/// Each output cell receives the mean of its corresponding input block.
/// Preserves total integral approximately: sum(out) ≈ sum(in) × (dst/src)².
pub fn downsample_block_average(
    src: &[f32],
    src_len: usize,
    dst_len: usize,
) -> Vec<f32> {
    let factor = src_len / dst_len;
    let mut out = vec![0.0f32; dst_len * dst_len];
    for dy in 0..dst_len {
        for dx in 0..dst_len {
            let mut acc = 0.0f64;
            for sy in 0..factor {
                for sx in 0..factor {
                    let src_idx = (dy * factor + sy) * src_len + (dx * factor + sx);
                    if src_idx < src.len() {
                        acc += src[src_idx] as f64;
                    }
                }
            }
            let block = (factor * factor) as f64;
            out[dy * dst_len + dx] = (acc / block) as f32;
        }
    }
    out
}

impl From<crate::ecology::Agent> for AgentView {
    fn from(a: crate::ecology::Agent) -> Self {
        agent_view(&a)
    }
}

/// Run the sim from seed through `ticks`, then emit a single JSON Snapshot string.
/// `daylight_series_json` must be a JSON array of `f64`, length ≥ ticks. Each
/// element drives the daylight value for that substep inside `Ecology::update`.
pub fn snapshot_json(seed: u32, ticks: usize, daylight_series_json: &str) -> String {
    let series: Vec<f64> = serde_json::from_str(daylight_series_json)
        .unwrap_or_else(|_| {
            // Fallback sinusoidal series when caller passes empty/invalid.
            let n = ticks.max(1);
            (0..n)
                .map(|i| 0.5 + 0.5 * aether_libm::sin((i as f64 / 300.0) * std::f64::consts::PI))
                .collect()
        });
    let mut eco = crate::ecology::Ecology::new(seed);
    for (i, &daylight) in series.iter().enumerate().take(ticks) {
        eco.update(0.1, daylight);
    }
    let snap: Snapshot = eco.into();
    serde_json::to_string(&snap).expect("serialize snapshot")
}

/// Returns true when compiled for wasm32 — allows callers to branch behavior.
pub const fn is_wasm_target() -> bool {
    cfg!(target_arch = "wasm32")
}

// Note: the former `#[wasm_bindgen] pub fn snapshot(...)` binding was removed:
// it introduced a `__wbindgen_placeholder__` import that made the module
// un-instantiable without generated glue JS. The stateless path remains
// available natively via `snapshot_json`; the wasm viewer uses the stateless
// C-ABI `sim_*` exports below directly.

// ─────────────────────────────────────────────────────────────────────────────
// Stateful C-ABI exports for the Three.js wasm viewer (no wasm-bindgen glue).
//
// JS usage sketch:
//   const { instance } = await WebAssembly.instantiateStreaming(fetch(url));
//   const sim = instance.exports.sim_new(seed);
//   instance.exports.sim_step(sim, n, daylightF64Array.byteOffset / 8, n);
//   const count = instance.exports.sim_agents_flat(sim, xBuf.byteOffset >> 2, ...);
//   instance.exports.sim_free(sim);
//
// Layout constants (must match the JS viewer):
//   GRID_N = 76, GRID_SPAN = 1080, mineral resampled = 38×38 = 1444 f32.
//   agent kind bytes: h e r b → 'h', pred → 'p', scav → 's', omn → 'o',
//                     fish → 'f', deco → 'd'.
// ─────────────────────────────────────────────────────────────────────────────

pub const GRID_N: u32 = 76;
pub const GRID_SPAN: f64 = 1080.0;
pub const MINERAL_RESAMPLED_LEN: usize = 38 * 38;
pub const MINERAL_RESAMPLED_N: u32 = 38;

/// Scratch layout (byte offsets from `sim_scratch(sim)`, mirrored in the JS
/// viewer). Single reusable arena: mineral resampled block, then SoA agent
/// arrays for up to AGENT_CAP live agents, then a 64-slot f64 daylight buffer
/// the JS side writes before each `sim_step` call.
pub const AGENT_CAP: usize = 1024;
pub const DAYLIGHT_CAP: usize = 64;
pub const SCRATCH_OFF_MINERAL: usize = 0;
pub const MINERAL_STRIDE: usize = 4; // f32 align
pub const MINERAL_BYTES: usize = MINERAL_RESAMPLED_LEN * MINERAL_STRIDE; // 5776
pub const AGENT_BASE: usize = MINERAL_BYTES;
/// Per-agent slot inside the AoS row (offsets are relative to the first
/// agent's X slot, not absolute scratch offsets).
pub const SLOT_X: usize = 0;
pub const SLOT_Z: usize = 4;
pub const SLOT_KIND: usize = 8;
pub const SLOT_ENERGY: usize = 12;
pub const SLOT_HUNGER: usize = 16;
pub const SLOT_SIZE: usize = 20;
pub const SLOT_HUE: usize = 24;
/// Phase 5 视觉基因化新增槽位（颜色/缩放在 RS 侧预计算；ID 供渲染端跨帧追踪个体）。
pub const SLOT_COLOR: usize = 28;
pub const SLOT_VIS_SCALE: usize = 32;
pub const SLOT_FLAGS: usize = 36;
pub const SLOT_ID: usize = 40;
/// Phase 9: DECO 可视效率代理值（u16，非 DECO 时不写，FLAG_DECO_EFF_VALID 位表示有效）。
pub const SLOT_DECO_EFF: usize = 44;
pub const FLAG_DECO_EFF_VALID: u32 = 1 << 1;
/// Phase 10: DECO hue 直方图固定量程 [0.629, 0.871]（Phase 9 定义的效率→hue 全域）。
/// 量程不随实际分布动态变化，不同 tick 的直方图才可直接对比（漂移 = 分布真的动了，
/// 而不是坐标轴在动）。
pub const DECO_HUE_MIN: f64 = 0.629;
pub const DECO_HUE_SPAN: f64 = 0.242;
pub const DECO_HUE_BUCKETS_CAP: usize = 64;
pub const AGENT_SLOT_SIZE: usize = 48; // next slot starts 48 bytes later
/// Absolute scratch offsets.
pub const SCRATCH_OFF_X: usize = AGENT_BASE + SLOT_X;
pub const SCRATCH_OFF_Z: usize = AGENT_BASE + SLOT_Z;
pub const SCRATCH_OFF_KIND: usize = AGENT_BASE + SLOT_KIND;
pub const SCRATCH_OFF_ENERGY: usize = AGENT_BASE + SLOT_ENERGY;
pub const SCRATCH_OFF_HUNGER: usize = AGENT_BASE + SLOT_HUNGER;
pub const SCRATCH_OFF_SIZE: usize = AGENT_BASE + SLOT_SIZE;
pub const SCRATCH_OFF_HUE: usize = AGENT_BASE + SLOT_HUE;
pub const SCRATCH_OFF_DAYLIGHT: usize = AGENT_BASE + AGENT_CAP * AGENT_SLOT_SIZE; // 64 × f64
pub const SCRATCH_OFF_COUNTS: usize = SCRATCH_OFF_DAYLIGHT + DAYLIGHT_CAP * 8; // 6 × u32
pub const SCRATCH_OFF_HIST: usize = SCRATCH_OFF_COUNTS + 6 * 4; // DECO_HUE_BUCKETS_CAP × u32
/// Phase 15: phylogeny readout region appended after the histogram area.
/// Clade record (40 bytes since Phase 16, mirrors sim_get_clade):
///   [id u32][parent u32 (u32::MAX = root)][kind_hash u32][birth u32]
///   [death u32 (u32::MAX = alive)][member_count u32][centroid_eff f64][argb u32]
///   [niche f32]   <- Phase 16 tail field (member_count = alive_count @20)
/// Event record (56 bytes, mirrors sim_get_event):
///   [tick u32][parent u32][child_count u32][event_kind u32 (was reserved)]
///   [genetic_distance f64][child0..7 u32]
pub const CLADE_CAP: usize = 64;
pub const EVENT_CAP: usize = 64;
/// Phase 16: 36 -> 40 bytes (niche f32 appended at offset 36; existing
/// offsets are unchanged so JS readers of the old layout only miss the tail).
pub const CLADE_RECORD_SIZE: usize = 40;
pub const EVENT_RECORD_SIZE: usize = 56;
/// Phase 17: cataclysm record (sim_get_cataclysm, JS-provided out buffer —
/// NOT part of the scratch arena, so SCRATCH_SIZE is unchanged):
///   [id u32][kind u32][tick u32][ends_at u32][remaining u32]
///   [x f32][z f32][radius f32][severity f32][active u32]
pub const CATACLYSM_RECORD_SIZE: usize = 40;
pub const CATACLYSM_CAP: usize = 64;
pub const CLADE_NAME_CAP: usize = 32;
pub const MAX_EVENT_CHILDREN: usize = 8;
pub const SCRATCH_OFF_CLADES: usize = SCRATCH_OFF_HIST + DECO_HUE_BUCKETS_CAP * 4;
pub const SCRATCH_OFF_EVENTS: usize = SCRATCH_OFF_CLADES + CLADE_CAP * CLADE_RECORD_SIZE;
pub const SCRATCH_OFF_CLADE_NAME: usize = SCRATCH_OFF_EVENTS + EVENT_CAP * EVENT_RECORD_SIZE;
pub const SCRATCH_SIZE: usize = SCRATCH_OFF_CLADE_NAME + CLADE_NAME_CAP;

/// FNV-1a 32 over a string (stable cross-session kind hash for clade records).
fn fnv1a_32(s: &str) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in s.as_bytes() {
        h ^= *b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

#[repr(C)]
pub struct WasmSimCore {
    eco: crate::ecology::Ecology,
    scratch: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
mod sim_exports {
    use super::*;

    /// # Safety
    /// Returns an opaque handle; all other sim_* calls take it back unchanged.
    #[no_mangle]
    pub unsafe extern "C" fn sim_new(seed: u32) -> *mut WasmSimCore {
        Box::into_raw(Box::new(WasmSimCore {
            eco: crate::ecology::Ecology::new(seed),
            scratch: vec![0u8; SCRATCH_SIZE],
        }))
    }

    /// # Safety
    /// `sim` must be a handle from `sim_new` that has not been freed yet.
    #[no_mangle]
    pub unsafe extern "C" fn sim_free(sim: *mut WasmSimCore) {
        if sim.is_null() {
            return;
        }
        drop(Box::from_raw(sim));
    }

    /// Advance the sim by `n` substeps of dt=0.1s. `daylight` is `n` f64 values
    /// in [0,1]; when shorter, the last provided value repeats; when absent a
    /// sinusoidal day curve is derived from the current tick count.
    ///
    /// # Safety
    /// `sim` from `sim_new`; `daylight` points to `daylight_len` readable f64s.
    #[no_mangle]
    pub unsafe extern "C" fn sim_step(
        sim: *mut WasmSimCore,
        n: u32,
        daylight: *const f64,
        daylight_len: u32,
    ) {
        let core = &mut *sim;
        for i in 0..n as usize {
            let dl = if !daylight.is_null() && (i as u32) < daylight_len {
                *daylight.add(i)
            } else if !daylight.is_null() && daylight_len > 0 {
                *daylight.add(daylight_len as usize - 1)
            } else {
                let t = (core.eco.tick_count() as f64 * 0.1 / 300.0) * std::f64::consts::PI;
                0.5 + 0.5 * aether_libm::sin(t)
            };
            core.eco.update(0.1, dl);
        }
    }

    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_tick(sim: *mut WasmSimCore) -> u32 {
        (*sim).eco.tick_count()
    }

    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_season(sim: *mut WasmSimCore) -> u32 {
        (*sim).eco.season.max(0) as u32
    }

    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_daylight(sim: *mut WasmSimCore) -> f64 {
        (*sim).eco.daylight
    }

    /// Write live counts for [herb, pred, scav, omn, fish, deco] into the
    /// scratch counts region (6 × u32 at SCRATCH_OFF_COUNTS). Returns 6.
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_counts(sim: *mut WasmSimCore) -> u32 {
        let core = &mut *sim;
        const KINDS: [&str; 6] = [
            crate::ecology::HERB,
            crate::ecology::PRED,
            crate::ecology::SCAV,
            crate::ecology::OMN,
            crate::ecology::FISH,
            crate::ecology::DECO,
        ];
        let out = core.scratch.as_mut_ptr().add(SCRATCH_OFF_COUNTS) as *mut u32;
        for (i, kind) in KINDS.iter().enumerate() {
            let c = core.eco.agents.iter().filter(|a| a.alive && a.kind == *kind).count();
            *out.add(i) = c as u32;
        }
        KINDS.len() as u32
    }

    /// Copy the live agents (ascending id, capped at AGENT_CAP) into the
    /// scratch AoS rows. Returns the number written (≤ cap).
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_agents_flat(sim: *mut WasmSimCore, cap: u32) -> u32 {
        let core = &mut *sim;
        let cap = (cap as usize).min(AGENT_CAP);
        let base = core.scratch.as_mut_ptr() as usize + AGENT_BASE;
        let mut n = 0usize;
        for a in core.eco.agents.iter().filter(|a| a.alive) {
            if n >= cap {
                break;
            }
            let row = (base + n * AGENT_SLOT_SIZE) as *mut u8;
            let max_e = if a.max_energy > 0.0 { a.max_energy } else { 1.0 };
            *(row.add(SLOT_X) as *mut f32) = a.x as f32;
            *(row.add(SLOT_Z) as *mut f32) = a.z as f32;
            *row.add(SLOT_KIND) = match a.kind {
                crate::ecology::HERB => b'h',
                crate::ecology::PRED => b'p',
                crate::ecology::SCAV => b's',
                crate::ecology::OMN => b'o',
                crate::ecology::FISH => b'f',
                crate::ecology::DECO => b'd',
                _ => b'?',
            };
            *(row.add(SLOT_ENERGY) as *mut f32) = a.energy as f32;
            *(row.add(SLOT_HUNGER) as *mut f32) = (a.energy / max_e).min(1.0) as f32;
            *(row.add(SLOT_SIZE) as *mut f32) = a.genome.size as f32;
            *(row.add(SLOT_HUE) as *mut f32) = a.genome.hue as f32;
            let (vis_scale, _hue_u16, color_hex, _fin_vis, deco_eff_vis) = visual_fields(a);
            *(row.add(SLOT_COLOR) as *mut u32) = color_hex;
            *(row.add(SLOT_VIS_SCALE) as *mut f32) = vis_scale;
            let flags = if a.lineage.is_some() { FLAG_LINEAGE } else { 0 }
                | if deco_eff_vis.is_some() { FLAG_DECO_EFF_VALID } else { 0 };
            *(row.add(SLOT_FLAGS) as *mut u32) = flags;
            *(row.add(SLOT_ID) as *mut u32) = a.id;
            if let Some(eff) = deco_eff_vis {
                *(row.add(SLOT_DECO_EFF) as *mut u32) = eff as u32;
            }
            n += 1;
        }
        n as u32
    }

    /// Base pointer of the reusable scratch arena. Layout constants are fixed
    /// (see SCRATCH_OFF_* above and the mirrored table in wasm-viewer.mjs).
    /// JS reads/writes through a view on `memory.buffer` at these offsets.
    ///
    /// # Safety
    /// `sim` from `sim_new`. The pointer stays valid for the sim's lifetime.
    #[no_mangle]
    pub unsafe extern "C" fn sim_scratch(sim: *mut WasmSimCore) -> *mut u8 {
        (*sim).scratch.as_mut_ptr()
    }

    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_scratch_size() -> u32 {
        SCRATCH_SIZE as u32
    }

    /// Write the 38×38 block-averaged mineral field into the scratch arena.
    /// Returns the number of f32s written (1444), or 0 on failure.
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_mineral_resampled(sim: *mut WasmSimCore) -> u32 {
        let core = &mut *sim;
        if core.scratch.len() < SCRATCH_OFF_X {
            return 0;
        }
        let full = core.eco.mineral_field();
        let dst = &mut core.scratch[SCRATCH_OFF_MINERAL..SCRATCH_OFF_X];
        let dst_f32 =
            std::slice::from_raw_parts_mut(dst.as_mut_ptr() as *mut f32, MINERAL_RESAMPLED_LEN);
        let factor = (GRID_N as usize) / MINERAL_RESAMPLED_N as usize;
        let src_len = GRID_N as usize;
        let dst_len = MINERAL_RESAMPLED_N as usize;
        for dy in 0..dst_len {
            for dx in 0..dst_len {
                let mut acc = 0.0f64;
                for sy in 0..factor {
                    for sx in 0..factor {
                        let idx = (dy * factor + sy) * src_len + (dx * factor + sx);
                        if idx < full.len() {
                            acc += full[idx] as f64;
                        }
                    }
                }
                dst_f32[dy * dst_len + dx] = (acc / (factor * factor) as f64) as f32;
            }
        }
        MINERAL_RESAMPLED_LEN as u32
    }

    /// Peak value of the full-resolution mineral field (for HUD).
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_mineral_peak(sim: *mut WasmSimCore) -> f32 {
        (*sim)
            .eco
            .mineral_field()
            .iter()
            .fold(0.0f32, |m, &v| if v > m { v } else { m })
    }

    /// Phase 9: DECO 种群效率均值与标准差（每帧最多 4x/sec HUD 调用）。DECO 不存在时 mean/std = 0.0。
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_deco_eff_mean(sim: *mut WasmSimCore) -> f64 {
        (*sim).eco.stats().deco_eff_mean
    }
    #[no_mangle]
    pub unsafe extern "C" fn sim_deco_eff_std(sim: *mut WasmSimCore) -> f64 {
        (*sim).eco.stats().deco_eff_std
    }

    /// Phase 10: DECO hue 直方图。把每个存活 DECO 的 `deco_eff_visual` 量化到
    /// `buckets` 个固定桶（量程 [0.629, 0.871] 与 visual_hue_deco 一致，跨 tick 可比），
    /// 计数写入 `out_counts`（caller 提供指向长度 ≥ buckets 的 u32 数组）。返回
    /// 实际写入的 bucket 数（buckets 被截断到 DECO_HUE_BUCKETS_CAP = 64）；无存活
    /// DECO 时全桶写 0；入参无效则静默丢弃（不 panic、不 trap）。
    ///
    /// # Safety
    /// `sim` from `sim_new`；`out_counts` 可写至少 DECO_HUE_BUCKETS_CAP 个 u32。
    #[no_mangle]
    pub unsafe extern "C" fn sim_deco_hue_histogram(
        sim: *mut WasmSimCore,
        buckets: u32,
        out_counts: *mut u32,
    ) -> u32 {
        if sim.is_null() || out_counts.is_null() || buckets == 0 {
            return 0;
        }
        let n = (buckets as usize).min(DECO_HUE_BUCKETS_CAP);
        let core = &mut *sim;
        // 先清后累：调用方若用更小桶数复用同一 scratch 区域，不会残留旧计数。
        let out = std::slice::from_raw_parts_mut(out_counts, n);
        out.fill(0);
        for a in core.eco.agents.iter().filter(|a| a.alive && a.kind == crate::ecology::DECO) {
            let eff = deco_efficiency_to_u16(a.genome.deco_efficiency);
            out[deco_hue_bucket(eff, n)] += 1;
        }
        n as u32
    }

    /// Phase 15: total number of species clades ever created (root included,
    /// extinct branches included — the fossil record). 0 on a null handle.
    /// JS pulls this every 60 sim-ticks alongside sim_get_clade (O(N_clades)
    /// batch read, never a per-frame hot path).
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_clade_count(sim: *mut WasmSimCore) -> u32 {
        if sim.is_null() { return 0; }
        (*sim).eco.species_clades().len() as u32
    }

    /// Fill the 36-byte clade record at `out`:
    /// [id u32][parent u32 (u32::MAX = root)][kind_hash u32][birth u32]
    /// [death u32 (u32::MAX = alive)][alive_count u32][centroid_eff f64][argb u32].
    /// `alive_count` mirrors the last detection pass (≤ 60 ticks stale);
    /// `argb` colors the node from centroid_eff via the Phase 9 DECO hue map.
    /// Returns 1 on success, 0 on null/bounds failure (never traps).
    ///
    /// # Safety
    /// `sim` from `sim_new`; `out` writable for CLADE_RECORD_SIZE bytes.
    #[no_mangle]
    pub unsafe extern "C" fn sim_get_clade(sim: *mut WasmSimCore, idx: u32, out: *mut u8) -> u32 {
        if sim.is_null() || out.is_null() { return 0; }
        let core = &mut *sim;
        let Some(c) = core.eco.species_clades().get(idx as usize) else { return 0; };
        let eff = c.centroid_genome.deco_efficiency;
        let argb = hsl_to_rgb_u32(visual_hue_deco(eff), 0.47, 0.70);
        let mut rec = [0u8; CLADE_RECORD_SIZE];
        rec[0..4].copy_from_slice(&c.id.to_le_bytes());
        rec[4..8].copy_from_slice(&c.parent_id.unwrap_or(u32::MAX).to_le_bytes());
        rec[8..12].copy_from_slice(&fnv1a_32(c.kind).to_le_bytes());
        rec[12..16].copy_from_slice(&c.birth_tick.to_le_bytes());
        rec[16..20].copy_from_slice(&c.death_tick.unwrap_or(u32::MAX).to_le_bytes());
        rec[20..24].copy_from_slice(&(c.members.len() as u32).to_le_bytes());
        rec[24..32].copy_from_slice(&eff.to_le_bytes());
        rec[32..36].copy_from_slice(&argb.to_le_bytes());
        // Phase 16: clade niche position (f32 tail field; offset 36 was the
        // record end before Phase 16, so old readers simply never see it).
        rec[36..40].copy_from_slice(&(c.niche_offset as f32).to_le_bytes());
        std::ptr::copy_nonoverlapping(rec.as_ptr(), out, CLADE_RECORD_SIZE);
        1
    }

    /// UTF-8 clade name (e.g. "deco", "deco-A") into `buf`, truncated to
    /// `cap`; returns the byte length written. 0 on failure.
    ///
    /// # Safety
    /// `sim` from `sim_new`; `buf` writable for `cap` bytes.
    #[no_mangle]
    pub unsafe extern "C" fn sim_clade_name(
        sim: *mut WasmSimCore,
        idx: u32,
        buf: *mut u8,
        cap: u32,
    ) -> u32 {
        if sim.is_null() || buf.is_null() || cap == 0 { return 0; }
        let core = &mut *sim;
        let Some(c) = core.eco.species_clades().get(idx as usize) else { return 0; };
        let bytes = c.name.as_bytes();
        let n = bytes.len().min(cap as usize);
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), buf, n);
        n as u32
    }

    /// Phase 15: number of clade speciation events (split log). 0 on null.
    ///
    /// # Safety
    /// `sim` from `sim_new`.
    #[no_mangle]
    pub unsafe extern "C" fn sim_event_count(sim: *mut WasmSimCore) -> u32 {
        if sim.is_null() { return 0; }
        (*sim).eco.clade_events().len() as u32
    }

    /// Fill the 56-byte event record at `out`:
    /// [tick u32][parent_clade_id u32][child_count u32][reserved u32]
    /// [genetic_distance f64][child0..7 u32].
    /// Children beyond MAX_EVENT_CHILDREN (= 8) are truncated (the engine
    /// emits one child per pass in practice). Returns child_count ≥ 1 on
    /// success, 0 on failure.
    ///
    /// # Safety
    /// `sim` from `sim_new`; `out` writable for EVENT_RECORD_SIZE bytes.
    #[no_mangle]
    pub unsafe extern "C" fn sim_get_event(sim: *mut WasmSimCore, idx: u32, out: *mut u8) -> u32 {
        if sim.is_null() || out.is_null() { return 0; }
        let core = &mut *sim;
        let Some(e) = core.eco.clade_events().get(idx as usize) else { return 0; };
        let mut rec = [0u8; EVENT_RECORD_SIZE];
        rec[0..4].copy_from_slice(&e.tick.to_le_bytes());
        rec[4..8].copy_from_slice(&e.parent_clade_id.to_le_bytes());
        let n_children = e.child_clade_ids.len().min(MAX_EVENT_CHILDREN);
        rec[8..12].copy_from_slice(&(n_children as u32).to_le_bytes());
        // Phase 16: the reserved u32 now carries the event kind
        // (0 = speciation, 1 = extinction, 2 = competitive exclusion).
        rec[12..16].copy_from_slice(&(e.event_kind as u32).to_le_bytes());
        rec[16..24].copy_from_slice(&e.genetic_distance.to_le_bytes());
        for (k, cid) in e.child_clade_ids.iter().take(MAX_EVENT_CHILDREN).enumerate() {
            rec[24 + k * 4..28 + k * 4].copy_from_slice(&cid.to_le_bytes());
        }
        std::ptr::copy_nonoverlapping(rec.as_ptr(), out, EVENT_RECORD_SIZE);
        // Return contract (Phase 16): >= 1 on success (children count, min 1
        // so extinction records — 0 children — stay distinguishable from
        // failure), 0 on failure. JS reads the true child count from the
        // record's child_count field at offset 8.
        n_children.max(1) as u32
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
    /// NUL-terminated UTF-8 buffer; write its length to `out_len` when non-null.
    /// Release with `sim_free_json(ptr, len)`.
    ///
    /// # Safety
    /// `sim` from `sim_new`; `out_len` writable when non-null.
    #[no_mangle]
    pub unsafe extern "C" fn sim_snapshot_json(sim: *mut WasmSimCore, out_len: *mut u32) -> *mut u8 {
        let snap = snapshot_from_ref(&(*sim).eco);
        let json = serde_json::to_string(&snap).expect("serialize snapshot");
        // Exact-capacity buffer so sim_free_json's from_raw_parts(len, len) is valid.
        let mut bytes = Vec::with_capacity(json.len() + 1);
        bytes.extend_from_slice(json.as_bytes());
        bytes.push(0);
        let len = bytes.len();
        if !out_len.is_null() {
            *out_len = len as u32;
        }
        let ptr = bytes.as_mut_ptr();
        std::mem::forget(bytes);
        ptr
    }

    /// # Safety
    /// `ptr`/`len` must come from `sim_snapshot_json` and not be freed yet.
    #[no_mangle]
    pub unsafe extern "C" fn sim_free_json(ptr: *mut u8, len: u32) {
        if ptr.is_null() {
            return;
        }
        drop(Vec::from_raw_parts(ptr, len as usize, len as usize));
    }

    /// Serialize the sim's internal full state as `aether-sim-save-state` v1 JSON.
    /// Returns a pointer to a NUL-terminated UTF-8 buffer; write its length to
    /// `out_len` when non-null. Release with `sim_free_json(ptr, len)`.
    /// The original `sim` handle remains valid — saves are side-effect free.
    ///
    /// # Safety
    /// `sim` from `sim_new`; `out_len` writable when non-null.
    #[no_mangle]
    pub unsafe extern "C" fn sim_save_json(sim: *mut WasmSimCore, out_len: *mut u32) -> *mut u8 {
        let state = (*sim).eco.to_save_state();
        let json = serde_json::to_string(&state)
            .expect("serialize save_state into aether-sim-save-state v1 JSON");
        let mut bytes = Vec::with_capacity(json.len() + 1);
        bytes.extend_from_slice(json.as_bytes());
        bytes.push(0);
        let len = bytes.len();
        if !out_len.is_null() {
            *out_len = len as u32;
        }
        let ptr = bytes.as_mut_ptr();
        std::mem::forget(bytes);
        ptr
    }

    /// Restore a sim from a prior `sim_save_json` output (UTF-8 JSON). Builds a
    /// fresh handle; the caller is responsible for freeing it later via
    /// `sim_free`. Returns a null pointer on malformed JSON instead of unwinding
    /// (panic would abort the WASM instance under wasm-release).
    ///
    /// # Safety
    /// `json_ptr` points to `json_len` readable UTF-8 bytes. The buffer is NOT
    /// consumed — the caller keeps ownership.
    #[no_mangle]
    pub unsafe extern "C" fn sim_restore_json(
        json_ptr: *const u8,
        json_len: u32,
    ) -> *mut WasmSimCore {
        if json_ptr.is_null() || json_len == 0 {
            return std::ptr::null_mut();
        }
        let bytes = std::slice::from_raw_parts(json_ptr, json_len as usize);
        match serde_json::from_slice::<SaveState>(bytes) {
            Ok(state) => {
                // Validate schema_name/version now so panic (trap) only happens
                // during restore, not during a future update call.
                assert_eq!(state.schema_name, crate::save_state::SAVE_STATE_SCHEMA_NAME);
                // Phase 16: accept older schema revisions (v1/v2 saves restore
                // with #[serde(default)] niche/burden fields), reject future ones.
                assert!(
                    state.version <= crate::save_state::SAVE_STATE_VERSION,
                    "unsupported future save state version {} (current is {})",
                    state.version, crate::save_state::SAVE_STATE_VERSION
                );
                let core = WasmSimCore {
                    eco: crate::ecology::Ecology::from_save_state(state),
                    scratch: vec![0u8; SCRATCH_SIZE],
                };
                Box::into_raw(Box::new(core))
            }
            Err(_) => std::ptr::null_mut(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn downsample_constant_preserves_value() {
        let src = vec![0.5f32; 16]; // 4×4 constant field
        let out = downsample_block_average(&src, 4, 2);
        assert_eq!(out.len(), 4);
        for v in &out {
            assert!((v - 0.5).abs() < 1e-6, "expected 0.5, got {}", v);
        }
    }

    #[test]
    fn downsample_8_to_4_avg_correct() {
        // 8×8 all-zeros → 4×4 all-zeros
        let src: Vec<f32> = vec![0.0; 64];
        let out = downsample_block_average(&src, 8, 4);
        assert_eq!(out.len(), 16);
        for v in &out {
            assert!(*v == 0.0);
        }
    }

    #[test]
    fn snapshot_from_ref_matches_owner_based_contract() {
        // Both paths must produce identical v3 packets for the same seed/ticks.
        let seed = 424242u32;
        let ticks = 40usize;
        let mut eco = crate::ecology::Ecology::new(seed);
        for i in 0..ticks {
            eco.update(0.1, 0.5 + 0.3 * (i as f64 * 0.01).sin());
        }
        let owner: Snapshot = eco.into();
        let mut eco2 = crate::ecology::Ecology::new(seed);
        for i in 0..ticks {
            eco2.update(0.1, 0.5 + 0.3 * (i as f64 * 0.01).sin());
        }
        let borrowed = snapshot_from_ref(&eco2);
        let a = serde_json::to_string(&owner).unwrap();
        let b = serde_json::to_string(&borrowed).unwrap();
        assert_eq!(a, b, "ref-based snapshot diverged from owner-based one");
    }

    #[test]
    fn hsl_to_rgb_u32_known_colors() {
        // CSS hsl() 语义的锚点色。
        assert_eq!(hsl_to_rgb_u32(0.0, 1.0, 0.5), 0xFF_FF_00_00);
        assert_eq!(hsl_to_rgb_u32(1.0 / 3.0, 1.0, 0.5), 0xFF_00_FF_00);
        assert_eq!(hsl_to_rgb_u32(2.0 / 3.0, 1.0, 0.5), 0xFF_00_00_FF);
        assert_eq!(hsl_to_rgb_u32(0.0, 0.0, 1.0), 0xFF_FF_FF_FF);
        assert_eq!(hsl_to_rgb_u32(0.0, 0.0, 0.0), 0xFF_00_00_00);
        // hue wrap：−0.25 ≡ 0.75。
        assert_eq!(hsl_to_rgb_u32(-0.25, 1.0, 0.5), hsl_to_rgb_u32(0.75, 1.0, 0.5));
        assert_eq!(hsl_to_rgb_u32(1.25, 1.0, 0.5), hsl_to_rgb_u32(0.25, 1.0, 0.5));
    }

    #[test]
    fn render_scale_boundary_and_clamp() {
        // 基因区间边界精确映射。
        assert!((render_scale_from_size(0.78) - 0.6).abs() < 1e-6, "0.78 → 0.6");
        assert!((render_scale_from_size(1.28) - 1.4).abs() < 1e-6, "1.28 → 1.4");
        assert!((render_scale_from_size(1.03) - 1.0).abs() < 1e-6, "midpoint → 1.0");
        // mutate 可把 size 推到 [0.58, 1.7]，超出历史区间时截断。
        assert!((render_scale_from_size(0.58) - 0.6).abs() < 1e-6, "below range clamps low");
        assert!((render_scale_from_size(1.7) - 1.4).abs() < 1e-6, "above range clamps high");
    }

    #[test]
    fn visual_hue_kind_bands() {
        // fish：锚点 0.56 + fin∈[0,0.3] ± 0.15 → [0.41, 1.01]，蓝→青→绿。
        for hue in [0.0f64, 0.25, 0.5, 0.75] {
            for fin in [0.0f64, 0.5, 1.0] {
                let h = visual_hue(crate::ecology::FISH, hue, fin);
                assert!(h >= 0.41 - 1e-9 || h <= 0.011, "fish hue {} (hue {}, fin {})", h, hue, fin);
            }
        }
        // pred：红褐带 wrap([−0.13, 0.17])。
        for hue in [0.0f64, 0.5, 1.0] {
            let h = visual_hue(crate::ecology::PRED, hue, 0.0);
            assert!(h <= 0.17 + 1e-9 || h >= 0.87 - 1e-9, "pred hue {} (hue {})", h, hue);
        }
        // herb：金黄带 wrap([−0.04, 0.26])。
        for hue in [0.0f64, 0.5, 1.0] {
            let h = visual_hue(crate::ecology::HERB, hue, 0.0);
            assert!(h <= 0.26 + 1e-9 || h >= 0.96 - 1e-9, "herb hue {} (hue {})", h, hue);
        }
    }

    #[test]
    fn deco_efficiency_visual_mapping() {
        // Phase 9 spec：efficiency → hue（DECO 可视基因化，锚点 0.75，±0.12）。
        assert!((visual_hue_deco(0.5) - 0.63).abs() < 1e-9, "eff 0.5 → 0.63 (deep purple/blue)");
        assert!((visual_hue_deco(1.0) - 0.75).abs() < 1e-9, "eff 1.0 → 0.75 (standard violet)");
        assert!((visual_hue_deco(1.5) - 0.87).abs() < 1e-9, "eff 1.5 → 0.87 (magenta/pink)");
        // efficiency 超出历史区间时 shift 截断。
        assert!((visual_hue_deco(0.0) - 0.63).abs() < 1e-9);
        assert!((visual_hue_deco(2.0) - 0.87).abs() < 1e-9);
        // u16 映射端点 + 中点。
        assert_eq!(deco_efficiency_to_u16(0.5), 0);
        assert_eq!(deco_efficiency_to_u16(1.0), 32768); // 0.5 × 65535 = 32767.5 → round → 32768
        assert_eq!(deco_efficiency_to_u16(1.5), 65535);
        assert_eq!(deco_efficiency_to_u16(0.0), 0, "below range clamps low");
        assert_eq!(deco_efficiency_to_u16(2.0), 65535, "above range clamps high");
        // Phase 7 主数字：efficiency 1.0444 → 1.3242 的 hue 可视化。
        let h0 = visual_hue_deco(1.0444);
        let h3k = visual_hue_deco(1.3242);
        assert!(h3k > h0, "efficiency gain must shift hue right (toward magenta): {} vs {}", h0, h3k);
        assert!((h0 - 0.760656).abs() < 1e-6, "t=0 hue {}", h0);
        assert!((h3k - 0.827808).abs() < 1e-6, "t=3000 hue {}", h3k);
    }

    #[test]
    fn agent_view_visual_fields() {
        let mut eco = crate::ecology::Ecology::new(7);
        for i in 0..5 {
            eco.update(0.1, 0.6);
        }
        let snap = snapshot_from_ref(&eco);
        assert!(!snap.agents.is_empty(), "warm world should have agents");
        for a in &snap.agents {
            assert!(a.genome_size_visual >= VIS_SCALE_MIN - 1e-6);
            assert!(a.genome_size_visual <= VIS_SCALE_MAX + 1e-6);
            assert_eq!(a.genome_color_hex & 0xFF00_0000, 0xFF00_0000, "alpha must be 0xFF");
        }
        let fish = snap.agents.iter().find(|a| a.kind == crate::ecology::FISH).expect("fish present");
        assert!(fish.fin_color_visual.is_some(), "FISH carries fin gene");
        let non_fish = snap.agents.iter().find(|a| a.kind != crate::ecology::FISH).expect("non-fish present");
        assert!(non_fish.fin_color_visual.is_none(), "only FISH carries fin gene");
        // Phase 9: DECO deco_eff_visual 有值，非 DECO 必为 None。
        let deco = snap.agents.iter().find(|a| a.kind == crate::ecology::DECO).expect("deco present");
        assert!(deco.deco_eff_visual.is_some(), "DECO carries deco_efficiency visual");
        let non_deco = snap.agents.iter().find(|a| a.kind != crate::ecology::DECO).expect("non-deco present");
        assert!(non_deco.deco_eff_visual.is_none(), "only DECO carries deco_eff_visual");
        // Phase 9: DECO hue 必须在效率驱动区间 [0.63, 0.87] 内。
        let deco_hue = deco.genome_hue_visual as f64 / 65535.0;
        assert!(deco_hue >= 0.63 - 1e-3 && deco_hue <= 0.87 + 1e-3, "deco hue {} out of [0.63, 0.87]", deco_hue);
    }

    #[test]
    fn deco_hue_bucket_boundary_and_mono() {
        // u16 端点 → bucket 端点；eff=1.0 落在中间桶附近。
        assert_eq!(deco_hue_bucket(0, 24), 0, "u16=0 falls in first bucket");
        assert_eq!(deco_hue_bucket(65535, 24), 23, "u16=65535 falls in last bucket");
        assert_eq!(deco_hue_bucket(32768, 24), 12, "eff=1.0 lands near bucket center");

        // 退化到单桶：所有输入落桶 0。
        assert_eq!(deco_hue_bucket(0, 1), 0);
        assert_eq!(deco_hue_bucket(65535, 1), 0);

        // 单调性：eff 越大桶号不减，且不超出 [0, buckets-1]。
        for &buckets in &[2usize, 8, 24, 64] {
            let mut last = 0usize;
            for i in (0..=655u16).step_by(100) {
                let b = deco_hue_bucket(i * 100, buckets);
                assert!(b >= last, "monotonicity broken at eff={} buckets={}", i * 100, buckets);
                assert!(b < buckets, "out of range at eff={} buckets={}", i * 100, buckets);
                last = b;
            }
        }
    }

    #[test]
    fn scratch_layout_covers_histogram_region() {
        // 各区域严格递增不重叠；histogram 区之后是 Phase 15 clade/event/name 区。
        assert!(super::SCRATCH_OFF_DAYLIGHT < super::SCRATCH_OFF_COUNTS);
        assert!(super::SCRATCH_OFF_COUNTS < super::SCRATCH_OFF_HIST);
        assert!(super::SCRATCH_OFF_HIST < super::SCRATCH_OFF_CLADES);
        assert!(super::SCRATCH_OFF_CLADES < super::SCRATCH_OFF_EVENTS);
        assert!(super::SCRATCH_OFF_EVENTS < super::SCRATCH_OFF_CLADE_NAME);
        assert_eq!(
            super::SCRATCH_OFF_CLADE_NAME + super::CLADE_NAME_CAP,
            super::SCRATCH_SIZE
        );
    }

    #[test]
    fn fnv1a_32_known_vectors() {
        // 标准 FNV-1a 32 位测试向量。
        assert_eq!(fnv1a_32(""), 0x811c_9dc5);
        assert_eq!(fnv1a_32("a"), 0xe40c_292c);
        assert_eq!(fnv1a_32("deco"), fnv1a_32("deco"), "deterministic");
        assert_ne!(fnv1a_32("deco"), fnv1a_32("deco-A"), "kind vs clade name differ");
    }
}
