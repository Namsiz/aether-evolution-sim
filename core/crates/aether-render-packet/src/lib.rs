//! Render-Packet — RS → JS transport schema for the ecology sim.
//!
//! ## Design Decisions
//!
//! 1. **Field-selection principle.** Only fields a renderer needs are exposed:
//!    - Per-agent: kind/id/x/z/energy/age/genome(linearized)/lineage/state/bite/repro/cargo/hunger/image/help-cd/fear/think.
//!      Internal-only fields (parent-id, dead-reason, trail history, fitness records, speciation event) are omitted — they have zero visual semantics.
//!    - Grass: total grass, per-species grass_by_species (`Vec<Vec<f32>>`), suit/moist/shade.
//!    - StatsAgg: species counts, plant mass, births/deaths/tick-count. Time-series evolution, isolation-history, deposit-summary etc. are research-grade and excluded from the hot serialization path.
//!    - Carcasses: NOT exported here (Chief decision pending — see README/decision doc).
//!      Rationale: SCAV pathing already happens inside the sim; renderer-side SCAV can
//!      approximate by observing agent positions. Adding them would bloat the packet
//!      by ~O(60) entries per tick for marginal visual value. Revisit if Phase 4
//!      needs scavenging visuals.
//!
//! 2. **Precision downgrade policy.**
//!    - x / z : kept as f64 — spatial locating needs sub-cell accuracy.
//!    - energy / age / genome values / bite / repro / cargo / help_cd / fear / think /
//!      grass_* / suit / moist / shade / plant_total : downgraded to f32. ±1 J error
//!      has no visual impact on instanced-mesh sizing/coloring, and halves the payload.
//!    - Optional floats (genome_help_thr / genome_pref_sig / genome_tol_sig / lineage)
//!      serialize as JSON `null` when `None`.
//!
//! 3. **Versioning.** Schema-breaking changes (field removal, type rename,
//!    array-size change, new required field) MUST increment `Meta.version`. Downstream
//!    JS consumer asserts the expected version at boot. `#[serde(default)]` on every
//!    non-essential field lets older payloads deserialize into newer structs
//!    (backward-compat reads).
//!
//!    v1 → v2：StatsAgg 新增 `FishCount`，支持第五物种 FISH。
//!    v2 → v3：WorldState 新增 `MineralField`（76²）与 `MineralFieldResampled`（38²）可选字段，
//!    用于热力图渲染；v2 消费者通过 `#[serde(default)]` 兜底反序列化。
//!
//!    Phase 5：AgentView 追加 4 个预计算视觉派生字段（GenomeSizeVisual / GenomeHueVisual /
//!    GenomeColorHex / FinColorVisual），纯 additive，仍为 v3 —— 旧 payload 缺字段时反序列化为
//!    默认值，渲染端降级为每种族单色；新 payload 对旧消费者只是多几个被忽略的键。
//!
//! 4. **Naming convention.** All public struct fields are `PascalCase` via
//!    `#[serde(rename_all = "PascalCase")]`. JS consumers that prefer camelCase run a
//!    transformer pass during deserialization; the Rust side never emits mixed styles.
//!
//! 5. **Zero borrowing across the boundary.** `Snapshot` contains owned data only; it
//!    never holds `&Ecology` or lifetimes tied to the sim. Permits capture-and-later-
//!    consume patterns (record→log→playback, cross-thread send) without pinning.
//!
//! 6. **Single-direction export.** Snapshot is write-only from the renderer's view:
//!    no setter, no mutable reference, no way to push state back into Ecology. This
//!    enforces the v2.0 architecture §5 "renderer must not write sim state" invariant.

// ─────────────────────────────────────────────────────────────────────────────
// Schema structs (PascalCase)
// ─────────────────────────────────────────────────────────────────────────────

/// Top-level snapshot root. One snapshot per call site.
#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Snapshot {
    #[serde(default)]
    pub meta: Meta,
    #[serde(default)]
    pub world: WorldState,
    #[serde(default)]
    pub agents: Vec<AgentView>,
    #[serde(default)]
    pub stats: StatsAgg,
}

/// Fixed-format header identifying schema version and runtime provenance.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Meta {
    /// Breaks when changed — caller must assert this matches their build.
    #[serde(default = "meta_default_version")]
    pub version: u32,
    pub engine: String,
    /// Sim wall-clock seconds at capture time (approximate; driven by `self.time`).
    #[serde(default)]
    pub time_seconds: f32,
}

const fn meta_default_version() -> u32 { 3 }

impl Default for Meta {
    fn default() -> Self {
        Self {
            version: meta_default_version(),
            engine: "aether-sim-ecology-rust".to_string(),
            time_seconds: 0.0,
        }
    }
}



/// Grid-level fields that the renderer reads before touching agents.
/// Terrain height is intentionally omitted — renderer maintains its own terrain
/// mesh baked into geometry; this packet only conveys what moves.
#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WorldState {
    #[serde(default)]
    pub tick_count: u32,
    #[serde(default)]
    pub season: i32,
    #[serde(default)]
    pub daylight: f32,
    #[serde(default)]
    pub grid_n: u32,
    /// Flat array of length GRID_N² representing current grass biomass.
    #[serde(default)]
    pub grass: Vec<f32>,
    /// Per-species grass layer: `grass_by_species[sp][i]`, `sp ∈ [0, 3)`.
    #[serde(default)]
    pub grass_by_species: Vec<Vec<f32>>,
    /// Suitability per cell (0..1). Renderer maps to color/scale.
    #[serde(default)]
    pub suit: Vec<f32>,
    /// Moisture per cell. Drives secondary color tints.
    #[serde(default)]
    pub moist: Vec<f32>,
    /// Shade fraction per cell. Used for alpha blending.
    #[serde(default)]
    pub shade: Vec<f32>,
    /// Full-resolution mineral intensity field (76×76 = 5776 floats). v3+ only.
    /// Serialized when exporter requests full payload; `None` on legacy packets.
    #[serde(default)]
    pub mineral_field: Option<Vec<f32>>,
    /// Downsampled mineral field (38×38 = 1444 floats), 4×4 block average.
    /// Preferred transport for per-tick heat-map rendering.
    #[serde(default)]
    pub mineral_field_resampled: Option<Vec<f32>>,
}

/// Per-agent view. One entry per live agent, ordered by ascending `id`.
/// Genome fields are linearized (no nested object) to keep the JSON flat.
#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AgentView {
    pub id: u32,
    pub kind: String,
    /// World-space position. y (terrain height) omitted — renderer has own height map.
    pub x: f64,
    pub z: f64,
    /// Energy (Joules, f32-downgraded). Renderer uses for health bar tint.
    #[serde(default)]
    pub energy: f32,
    /// Age (f32-downgraded). Renderer uses for fade-on-old.
    #[serde(default)]
    pub age: f32,
    /// Genome linearized into flat f32 fields. Optional genes emit `null` when unset.
    pub genome_speed: f32,
    pub genome_sense: f32,
    pub genome_size: f32,
    pub genome_metabolism: f32,
    pub genome_lifespan: f32,
    pub genome_hue: f32,
    pub genome_bold: f32,
    pub genome_curious: f32,
    pub genome_social: f32,
    #[serde(default)]
    pub genome_help_thr: Option<f32>,
    #[serde(default)]
    pub genome_pref_sig: Option<f32>,
    #[serde(default)]
    pub genome_tol_sig: Option<f32>,
    pub lineage: Option<String>,
    pub state: String,
    #[serde(default)]
    pub bite: f32,
    #[serde(default)]
    pub repro: f32,
    /// Seed cargo — one slot per registered plant (default 3 = the legacy
    /// `GRASS_N`; a registered 4th plant widens it, and JS reads at most its
    /// known slots, so a default-3 world is unaffected).
    #[serde(default = "empty_cargo")]
    pub cargo: Vec<f32>,
    /// Derived: `energy / max_energy`, clamped to [0, 1]. Health-bar fill.
    #[serde(default)]
    pub hunger: f32,
    #[serde(default)]
    pub image: i32,
    #[serde(default)]
    pub help_cd: f32,
    #[serde(default)]
    pub fear: f32,
    #[serde(default)]
    pub think: f32,
    /// Sim does not track eat-recency natively; reserved for future schema, always 0.
    #[serde(default)]
    pub time_since_last_eat: f32,
    // ── Phase 5 visual-genomics：RS 侧预计算，避免 JS 重复 hue→RGB 浮点数学 ──
    /// Render scale derived from genome.size, clamped to [0.6, 1.4].
    #[serde(default)]
    pub genome_size_visual: f32,
    /// Final hue after species anchor + kind shift, quantized u16 on the sRGB circle.
    #[serde(default)]
    pub genome_hue_visual: u16,
    /// Final instance color packed ARGB (alpha = 0xFF)：hsl_to_rgb(final_hue, s_kind, l_kind)。
    #[serde(default)]
    pub genome_color_hex: u32,
    /// FISH-only fin gene ([0,1])；其余物种为 `None`。
    #[serde(default)]
    pub fin_color_visual: Option<f32>,
    /// Phase 9: DECO 专属可视代理 —— deco_efficiency ∈ [0.5,1.5] 线性映射到 u16
    /// （与 GenomeHueVisual 同域）；非 DECO 必为 `None`。
    #[serde(default)]
    pub deco_eff_visual: Option<u16>,
}

fn empty_cargo() -> Vec<f32> {
    vec![0.0; 3]
}

/// Slimmed aggregation of global sim counters + species-level population.
/// Historical time-series (evolution, isolation_history, speciation_event) are
/// excluded intentionally — research telemetry, not rendering.
#[derive(Clone, Debug, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StatsAgg {
    #[serde(default)]
    pub tick_count: u32,
    #[serde(default)]
    pub season: i32,
    #[serde(default)]
    pub herb_count: usize,
    #[serde(default)]
    pub pred_count: usize,
    #[serde(default)]
    pub scav_count: usize,
    #[serde(default)]
    pub omn_count: usize,
    #[serde(default)]
    pub fish_count: usize,
    #[serde(default)]
    pub deco_count: usize,
    #[serde(default)]
    pub total_agents: usize,
    #[serde(default)]
    pub plant_total: f32,
    #[serde(default)]
    pub plant_by: Vec<f32>,
    #[serde(default)]
    pub births: u32,
    #[serde(default)]
    pub deaths: u32,
    #[serde(default)]
    pub kills: u32,
    #[serde(default)]
    pub starved: u32,
    #[serde(default)]
    pub aged: u32,
    /// Phase 9: alive DECO 的 deco_efficiency 均值（DECO count == 0 时 None）。
    #[serde(default)]
    pub deco_eff_mean: Option<f64>,
    /// Phase 9: alive DECO 的 deco_efficiency 标准差（DECO count < 2 时 None）。
    #[serde(default)]
    pub deco_eff_std: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pascalcase_roundtrips() {
        let snap = Snapshot {
            meta: Meta { version: 3, engine: "aether-render-packet".into(), time_seconds: 42.0 },
            world: WorldState {
                tick_count: 10, season: 1, daylight: 0.75, grid_n: 76,
                grass: vec![0.1; 16],
                grass_by_species: vec![vec![0.1; 16]; 3],
                suit: vec![0.5; 16],
                moist: vec![0.3; 16],
                shade: vec![0.2; 16],
                mineral_field: Some(vec![0.05; 16]),
                mineral_field_resampled: Some(vec![0.1; 4]),
            },
            agents: vec![AgentView {
                id: 1, kind: "herb".into(), x: 10.0, z: 20.0,
                energy: 80.0, age: 50.0,
                genome_speed: 4.2, genome_sense: 30.0, genome_size: 1.0,
                genome_metabolism: 1.0, genome_lifespan: 300.0,
                genome_hue: 0.42, genome_bold: 0.3, genome_curious: 0.5, genome_social: 0.4,
                genome_help_thr: None, genome_pref_sig: None, genome_tol_sig: None,
                lineage: None, state: "wander".into(), bite: 0.0, repro: 0.0,
                cargo: vec![0.0; 3], hunger: 0.53, image: 0, help_cd: 0.0, fear: 0.0, think: 0.0,
                time_since_last_eat: 0.0,
                // Phase 5 precomputed visuals.
                genome_size_visual: 1.0, genome_hue_visual: 27528, genome_color_hex: 0xFFB4D9A8,
                fin_color_visual: None,
                // Phase 9 DECO visual proxy; herb has None per schema rule but Some for roundtrip test.
                deco_eff_visual: Some(32768),
            }],
            stats: StatsAgg {
                tick_count: 10, season: 1,
                herb_count: 2, pred_count: 1, scav_count: 0, omn_count: 0, fish_count: 0, deco_count: 0,
                total_agents: 3,
                plant_total: 10.5, plant_by: vec![4.0, 3.5, 3.0],
                births: 0, deaths: 0, kills: 0, starved: 0, aged: 0,
                deco_eff_mean: Some(1.32), deco_eff_std: Some(0.18),
            },
        };
        let json = serde_json::to_string(&snap).expect("serialize");
        let parsed: Snapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(snap, parsed);
        assert!(json.contains("\"TickCount\""));
        assert!(json.contains("\"HerbCount\""));
        assert!(json.contains("\"GrassBySpecies\""));
        assert!(json.contains("\"FishCount\""));
        assert!(json.contains("\"MineralField\""));
        assert!(json.contains("\"MineralFieldResampled\""));
        // Phase 5 additive fields roundtrip correctly.
        assert!(json.contains("\"GenomeSizeVisual\""));
        assert!(json.contains("\"GenomeHueVisual\""));
        assert!(json.contains("\"GenomeColorHex\""));
        assert!(json.contains("\"FinColorVisual\""));
        // Phase 9 additive fields roundtrip correctly.
        assert!(json.contains("\"DecoEffVisual\""));
        assert!(json.contains("\"DecoEffMean\""));
        assert!(json.contains("\"DecoEffStd\""));
        let a = &parsed.agents[0];
        assert!((a.genome_size_visual - 1.0).abs() < 1e-6);
        assert_eq!(a.genome_hue_visual, 27528);
        assert_eq!(a.genome_color_hex, 0xFFB4D9A8);
        assert_eq!(a.fin_color_visual, None);
        assert_eq!(a.deco_eff_visual, Some(32768));
        assert_eq!(parsed.stats.deco_eff_mean, Some(1.32));
        assert_eq!(parsed.stats.deco_eff_std, Some(0.18));
    }

    #[test]
    fn backwards_compat_missing_fields() {
        // Old-format payload missing optional/optional-object fields still deserializes
        // thanks to #[serde(default)].
        let compact = serde_json::json!({
            "Meta": {"Version": 1, "Engine": "old-engine"},
            "World": {
                "TickCount": 5, "Season": 0, "Daylight": 0.5, "GridN": 76,
                "Grass": []
            },
            "Agents": [{
                "Id": 7, "Kind": "pred", "X": 12.0, "Z": 33.0,
                "Energy": 80.0, "Age": 45.0,
                "GenomeSpeed": 4.0, "GenomeSense": 30.0, "GenomeSize": 1.0,
                "GenomeMetabolism": 1.0, "GenomeLifespan": 300.0,
                "GenomeHue": 0.42, "GenomeBold": 0.3, "GenomeCurious": 0.5, "GenomeSocial": 0.4,
                "Lineage": null, "State": "wander",
                "Cargo": [0.0, 0.0, 0.0], "Hunger": 0.53,
                "Image": 0, "HelpCd": 0.0, "Fear": 0.0, "Think": 0.0
            }],
            "Stats": {}
        });
        // v1 格式的 compact JSON 缺失 FishCount / GenomeSizeVisual 等新字段，由 #[serde(default)]
        // 兜底为零值 / None；保证旧 payload 仍可反序列化进 v3 结构。
        let snap: Snapshot = serde_json::from_value(compact).expect("missing optional fields OK");
        assert_eq!(snap.meta.version, 1);
        assert_eq!(snap.stats.fish_count, 0);
        assert_eq!(snap.stats.herb_count, 0);
        assert!(snap.world.suit.is_empty());
        // Phase 5 fields default for legacy payloads.
        let a = &snap.agents[0];
        assert!((a.genome_size_visual - 0.0).abs() < 1e-6);
        assert_eq!(a.genome_hue_visual, 0);
        assert_eq!(a.genome_color_hex, 0);
        assert_eq!(a.fin_color_visual, None);
    }
}
