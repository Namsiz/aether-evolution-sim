//! ecology-sim — headless driver for the Rust port of living/src/ecology.js.
//!
//! Flags (all hand-parsed, std only):
//!   --seed <u32>        RNG seed (default 20261102)
//!   --ticks <u32>       ecological ticks to run (default 300; dt=0.1 → 1 substep/tick)
//!   --debug             print SNAPSHOT every tick
//!   --dump-state <N>    every N ticks dump all live agents as STATE_DUMP lines (includes t=0 init snapshot)
//!   --diff-mode         per-tick per-field diff against a JS oracle dump (--js-dump <path>),
//!                       reports the first differing agent/field and exits
//!   --bisect            binary-search the first forking tick using --js-dump <path>
//!   --js-dump <path>    JS-side STATE_DUMP file (produced by scripts/dump_state_diff.mjs --js-dump-only)
//!   --ignore <list>     comma-separated STATE_DUMP field names to skip in diff/bisect mode
//!   --dump-grass <t1,t2,...>  emit GRASS_ARR bit-hex dumps at specified ticks for grid-level diagnostics

mod ecology;
mod save_state;
mod terrain;

use ecology::{Agent, Ecology, Genome};
use std::collections::HashMap;
use std::io::Write;

fn arg_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn arg_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}

// Format an f64 the way JS String(v) does, so STATE_DUMP lines are directly
// comparable as text where possible; the diff parser compares numerically anyway.
fn fmt(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    if v == 0.0 {
        return "0".to_string(); // JS prints -0 as "0"
    }
    format!("{}", v)
}

// Per-agent partial of the oracle checksum: FNV-1a 64 over the same field
// sequence checksum() uses for agents (kind code, id, x*100, z*100, energy*100,
// age*10, all floored; only the low 32 bits of each value participate, negative
// floors wrap like BigInt(v) & 0xffffffffn). NaN floors to 0 on both sides.
fn checksum_local(a: &Agent) -> u64 {
    const MOD: u64 = 0x0000_0100_0000_01B3; // FNV-1a 64 prime 0x100000001B3
    let mut h: u64 = 14695981039346656033;
    fn mix(h: &mut u64, v: f64) {
        let f = if v.is_finite() { v.floor() } else { 0.0 };
        let val = (f as i64 as u64) & 0xFFFF_FFFF;
        *h = (*h ^ val).wrapping_mul(MOD);
    }
    mix(&mut h, a.kind.as_bytes()[0] as f64);
    mix(&mut h, a.id as f64);
    mix(&mut h, a.x * 100.0);
    mix(&mut h, a.z * 100.0);
    mix(&mut h, a.energy * 100.0);
    mix(&mut h, a.age * 10.0);
    h
}

fn genome_token(g: &Genome) -> String {
    let mut s = format!(
        "speed={},sense={},size={},metabolism={},lifespan={},hue={},bold={},curious={},social={},helpThr={}",
        fmt(g.speed),
        fmt(g.sense),
        fmt(g.size),
        fmt(g.metabolism),
        fmt(g.lifespan),
        fmt(g.hue),
        fmt(g.bold),
        fmt(g.curious),
        fmt(g.social),
        g.help_thr.map(fmt).unwrap_or_else(|| "undefined".to_string()),
    );
    if let Some(p) = g.pref_sig {
        s.push_str(&format!(",prefSig={}", fmt(p)));
    }
    if let Some(t) = g.tol_sig {
        s.push_str(&format!(",tolSig={}", fmt(t)));
    }
    s
}

fn dump_state(eco: &Ecology, tick: u32, out: &mut String) {
    for a in &eco.agents {
        if !a.alive {
            continue;
        }
        let lineage = a.lineage.unwrap_or("undefined");
        out.push_str(&format!(
            "STATE_DUMP t={} kind={} id={} x={} z={} energy={} age={} bite={} repro={} state={} heading={} lineage={} genome={{{}}} checksum_local={:08x}\n",
            tick,
            a.kind,
            a.id,
            fmt(a.x),
            fmt(a.z),
            fmt(a.energy),
            fmt(a.age),
            fmt(a.bite),
            fmt(a.repro),
            a.state,
            fmt(a.heading),
            lineage,
            genome_token(&a.genome),
            checksum_local(a),
        ));
    }
}

// ----- JS dump file parsing -------------------------------------------------

#[derive(Debug, Clone)]
struct Rec {
    kind: String,
    id: u32,
    // ordered (key, value) pairs, genome kept as its own ordered pair list
    fields: Vec<(String, Tok)>,
}

#[derive(Debug, Clone)]
enum Tok {
    Num(f64),
    Str(String),
    Genome(Vec<(String, Tok)>),
}

fn parse_num(s: &str) -> Tok {
    // JS prints Infinity/NaN; Rust f64::from_str accepts "inf"/"NaN" and "1e+21" etc.
    if let Ok(v) = s.parse::<f64>() {
        Tok::Num(v)
    } else {
        Tok::Str(s.to_string())
    }
}

fn parse_genome(inner: &str) -> Vec<(String, Tok)> {
    let mut out = Vec::new();
    for part in inner.split(',') {
        if part.is_empty() {
            continue;
        }
        if let Some(eq) = part.find('=') {
            let k = part[..eq].to_string();
            let v = &part[eq + 1..];
            let tok = if v == "undefined" { Tok::Str("undefined".into()) } else { parse_num(v) };
            out.push((k, tok));
        }
    }
    out
}

fn parse_line(line: &str) -> Option<(u32, Rec)> {
    let rest = line.strip_prefix("STATE_DUMP ")?;
    let mut tick = None;
    let mut kind = String::new();
    let mut id = None;
    let mut fields = Vec::new();
    for tok in rest.split_whitespace() {
        let eq = tok.find('=')?;
        let k = &tok[..eq];
        let v = &tok[eq + 1..];
        match k {
            "t" => tick = v.parse().ok(),
            "kind" => kind = v.to_string(),
            "id" => id = v.parse().ok(),
            "genome" => {
                let inner = v.strip_prefix('{').and_then(|s| s.strip_suffix('}')).unwrap_or(v);
                fields.push(("genome".to_string(), Tok::Genome(parse_genome(inner))));
            }
            _ => {
                // numbers may be NaN/Infinity; keep raw string for reporting.
                // checksum_local is a hex string — parsing it as f64 would
                // overflow to Infinity (e.g. "7874705813e71588").
                let num = if k == "checksum_local" {
                    None
                } else {
                    v.parse::<f64>().ok()
                };
                fields.push((
                    k.to_string(),
                    match num {
                        Some(n) => Tok::Num(n),
                        None => Tok::Str(v.to_string()),
                    },
                ));
            }
        }
    }
    Some((tick?, Rec { kind, id: id?, fields }))
}

fn load_js_dump(path: &str) -> Result<HashMap<u32, Vec<Rec>>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path, e))?;
    let mut map: HashMap<u32, Vec<Rec>> = HashMap::new();
    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        if let Some((t, rec)) = parse_line(line) {
            map.entry(t).or_default().push(rec);
        }
    }
    Ok(map)
}

// Field comparison order mirrors the STATE_DUMP emission order.
// kind/id are compared structurally before this loop, so they are not listed here.
const FIELD_ORDER: [&str; 10] = [
    "x", "z", "energy", "age", "bite", "repro", "state", "heading", "lineage", "checksum_local",
];

fn tok_eq(a: &Tok, b: &Tok) -> bool {
    match (a, b) {
        (Tok::Num(x), Tok::Num(y)) => x.to_bits() == y.to_bits() || (*x == 0.0 && *y == 0.0),
        (Tok::Str(x), Tok::Str(y)) => x == y,
        (Tok::Genome(ga), Tok::Genome(gb)) => {
            if ga.len() != gb.len() {
                return false;
            }
            ga.iter().zip(gb.iter()).all(|((ka, va), (kb, vb))| ka == kb && tok_eq(va, vb))
        }
        _ => false,
    }
}

fn tok_raw(t: &Tok) -> String {
    match t {
        Tok::Num(v) => {
            if v.is_nan() {
                "NaN".into()
            } else if v.is_infinite() {
                if *v > 0.0 { "Infinity".into() } else { "-Infinity".into() }
            } else {
                format!("{}", v)
            }
        }
        Tok::Str(s) => s.clone(),
        Tok::Genome(_) => "<genome>".into(),
    }
}

struct DiffReport {
    tick: u32,
    line: String,
}

fn compare_tick(tick: u32, js: &[Rec], eco: &Ecology, ignore: &[String]) -> Option<DiffReport> {
    let ignored = |f: &str| ignore.iter().any(|ig| ig == f);
    let rs: Vec<&Agent> = eco.agents.iter().filter(|a| a.alive).collect();
    let n = js.len().max(rs.len());
    for i in 0..n {
        let (j, r) = (js.get(i), rs.get(i));
        match (j, r) {
            (None, None) => continue,
            (None, Some(a)) => {
                return Some(DiffReport {
                    tick,
                    line: format!(
                        "FIRST_DIFF tick={} id={} kind={} field=missing_in_js val_js=none val_rs=id{}",
                        tick, a.id, a.kind, a.id
                    ),
                });
            }
            (Some(rec), None) => {
                return Some(DiffReport {
                    tick,
                    line: format!(
                        "FIRST_DIFF tick={} id={} kind={} field=missing_in_rs val_js=id{} val_rs=none",
                        tick, rec.id, rec.kind, rec.id
                    ),
                });
            }
            (Some(rec), Some(a)) => {
                if rec.id != a.id {
                    return Some(DiffReport {
                        tick,
                        line: format!(
                            "FIRST_DIFF tick={} field=id val_js={} val_rs={} (agent order/index {})",
                            tick, rec.id, a.id, i
                        ),
                    });
                }
                if rec.kind != a.kind {
                    return Some(DiffReport {
                        tick,
                        line: format!(
                            "FIRST_DIFF tick={} id={} field=kind val_js={} val_rs={}",
                            tick, a.id, rec.kind, a.kind
                        ),
                    });
                }
                // flat fields in canonical order
                for fname in FIELD_ORDER {
                    if ignored(fname) {
                        continue;
                    }
                    let jv = rec.fields.iter().find(|(k, _)| k == fname).map(|(_, v)| v);
                    let rv: Option<Tok> = match fname {
                        "x" => Some(Tok::Num(a.x)),
                        "z" => Some(Tok::Num(a.z)),
                        "energy" => Some(Tok::Num(a.energy)),
                        "age" => Some(Tok::Num(a.age)),
                        "bite" => Some(Tok::Num(a.bite)),
                        "repro" => Some(Tok::Num(a.repro)),
                        "state" => Some(Tok::Str(a.state.to_string())),
                        "heading" => Some(Tok::Num(a.heading)),
                        "lineage" => Some(Tok::Str(a.lineage.unwrap_or("undefined").to_string())),
                        "kind" => Some(Tok::Str(a.kind.to_string())),
                        "checksum_local" => Some(Tok::Str(format!("{:08x}", checksum_local(a)))),
                        _ => None,
                    };
                    // checksum_local compare: recompute the JS token numerically
                    if fname == "checksum_local" {
                        if let Some(Tok::Str(js_hex)) = jv {
                            let js_val = u64::from_str_radix(js_hex, 16).unwrap_or(u64::MAX);
                            if js_val != checksum_local(a) {
                                return Some(DiffReport {
                                    tick,
                                    line: format!(
                                        "FIRST_DIFF tick={} id={} kind={} field=checksum_local val_js=0x{} val_rs=0x{:08x}",
                                        tick, a.id, a.kind, js_hex, checksum_local(a)
                                    ),
                                });
                            }
                            continue;
                        }
                    }
                    match (jv, rv) {
                        (None, None) => {}
                        (Some(jt), None) => {
                            return Some(DiffReport {
                                tick,
                                line: format!(
                                    "FIRST_DIFF tick={} id={} kind={} field={} val_js={} val_rs=<absent>",
                                    tick, a.id, a.kind, fname, tok_raw(&jt),
                                ),
                            });
                        }
                        (None, Some(rt)) => {
                            return Some(DiffReport {
                                tick,
                                line: format!(
                                    "FIRST_DIFF tick={} id={} kind={} field={} val_js=<absent> val_rs={}",
                                    tick, a.id, a.kind, fname, tok_raw(&rt),
                                ),
                            });
                        }
                        (Some(jt), Some(rt)) => {
                            if !tok_eq(&jt, &rt) {
                                return Some(DiffReport {
                                    tick,
                                    line: format!(
                                        "FIRST_DIFF tick={} id={} kind={} field={} val_js={} val_rs={}",
                                        tick, a.id, a.kind, fname, tok_raw(&jt), tok_raw(&rt),
                                    ),
                                });
                            }
                        }
                    }
                }
                // genome fields
                if let Some(Tok::Genome(gj)) = rec.fields.iter().find(|(k, _)| k == "genome").map(|(_, v)| v) {
                    let gr: Vec<(String, Tok)> = {
                        let g = &a.genome;
                        let mut v = vec![
                            ("speed".to_string(), Tok::Num(g.speed)),
                            ("sense".to_string(), Tok::Num(g.sense)),
                            ("size".to_string(), Tok::Num(g.size)),
                            ("metabolism".to_string(), Tok::Num(g.metabolism)),
                            ("lifespan".to_string(), Tok::Num(g.lifespan)),
                            ("hue".to_string(), Tok::Num(g.hue)),
                            ("bold".to_string(), Tok::Num(g.bold)),
                            ("curious".to_string(), Tok::Num(g.curious)),
                            ("social".to_string(), Tok::Num(g.social)),
                            (
                                "helpThr".to_string(),
                                g.help_thr.map(Tok::Num).unwrap_or(Tok::Str("undefined".into())),
                            ),
                        ];
                        if let Some(p) = g.pref_sig {
                            v.push(("prefSig".to_string(), Tok::Num(p)));
                        }
                        if let Some(t) = g.tol_sig {
                            v.push(("tolSig".to_string(), Tok::Num(t)));
                        }
                        v
                    };
                    let m = gj.len().max(gr.len());
                    for k in 0..m {
                        let (jk, jv2) = gj.get(k).map(|(a, b)| (a.clone(), Some(b))).unwrap_or((String::new(), None));
                        let (rk, rv2) = gr.get(k).map(|(a, b)| (a.clone(), Some(b))).unwrap_or((String::new(), None));
                        if !jk.is_empty() && ignored(&format!("genome.{}", jk)) {
                            continue;
                        }
                        if jk != rk {
                            return Some(DiffReport {
                                tick,
                                line: format!(
                                    "FIRST_DIFF tick={} id={} kind={} field=genome[{}].key val_js={} val_rs={}",
                                    tick, a.id, a.kind, k,
                                    if jk.is_empty() { "<absent>".to_string() } else { jk },
                                    if rk.is_empty() { "<absent>".to_string() } else { rk },
                                ),
                            });
                        }
                        if let (Some(jv3), Some(rv3)) = (jv2, rv2) {
                            if !tok_eq(jv3, rv3) {
                                return Some(DiffReport {
                                    tick,
                                    line: format!(
                                        "FIRST_DIFF tick={} id={} kind={} field=genome.{} val_js={} val_rs={}",
                                        tick, a.id, a.kind, jk, tok_raw(jv3), tok_raw(rv3),
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

// Run the sim from scratch for `up_to` ticks, diffing every tick (including t=0)
// against the JS oracle dump. Returns the first divergence, if any.
fn run_diff(seed: u32, up_to: u32, js: &HashMap<u32, Vec<Rec>>, ignore: &[String]) -> Option<DiffReport> {
    let mut eco = Ecology::new(seed);
    if let Some(js_recs) = js.get(&0) {
        if let Some(rep) = compare_tick(0, js_recs, &eco, ignore) {
            return Some(rep);
        }
    } else {
        return Some(DiffReport {
            tick: 0,
            line: "FIRST_DIFF tick=0 field=js_dump_missing_t0 val_js=none val_rs=snapshot".to_string(),
        });
    }
    for i in 1..=up_to {
        let daylight = 0.5 + 0.5 * aether_libm::sin((i as f64 / 300.0) * std::f64::consts::PI);
        eco.update(0.1, daylight);
        match js.get(&i) {
            Some(js_recs) => {
                if let Some(rep) = compare_tick(i, js_recs, &eco, ignore) {
                    return Some(rep);
                }
            }
            None => {
                return Some(DiffReport {
                    tick: i,
                    line: format!("FIRST_DIFF tick={} field=js_dump_missing_tick val_js=none val_rs=snapshot", i),
                });
            }
        }
    }
    None
}

// Emit full-grid f32 bit-hex arrays for grass diagnostics.
// Format: GRASS_ARR t=<tick> arr=<name> <hex_per_cell>
fn emit_grass_dump<W: std::io::Write>(eco: &Ecology, tick: u32, w: &mut W) -> std::io::Result<()> {
    macro_rules! field {
        ($name:expr, $slice:expr) => {{
            let s = $slice;
            let hex: String = s.iter().map(|&v| format!("{:08x}", v.to_bits())).collect();
            writeln!(w, "GRASS_ARR t={} arr={} {}", tick, $name, hex)?;
        }};
    }
    // At t=0: suit/moist/shade set in _initGrass; suit_eff_by+grass_p computed there too.
    // After ticks: grass/suit_eff_by still present (moist is constant post-init).
    field!("moist", eco.moist());
    field!("suit", eco.suit());
    field!("shade", eco.shade());
    for sp in 0..ecology::GRASS_N {
        field!(&format!("suit_eff_by_{}", sp), eco.suit_eff_by(sp));
        field!(&format!("grass_p_{}", sp), eco.grass_p(sp));
    }
    field!("grass", eco.grass());
    Ok(())
}

fn struct_summary(eco: &Ecology) -> String {
    let (mut h, mut p, mut scav, mut omn) = (0usize, 0usize, 0usize, 0usize);
    for a in &eco.agents {
        if !a.alive {
            continue;
        }
        match a.kind {
            ecology::HERB => h += 1,
            ecology::PRED => p += 1,
            ecology::SCAV => scav += 1,
            _ => omn += 1,
        }
    }
    format!("herb={} pred={} scav={} omn={} total={}", h, p, scav, omn, h + p + scav + omn)
}

fn main() {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let args: Vec<String> = std::env::args().collect();
    let seed: u32 = arg_value(&args, "--seed").and_then(|s| s.parse().ok()).unwrap_or(20261102);
    let ticks: u32 = arg_value(&args, "--ticks").and_then(|s| s.parse().ok()).unwrap_or(300);
    let debug = arg_flag(&args, "--debug");
    let dump_every: Option<u32> = arg_value(&args, "--dump-state").and_then(|s| s.parse().ok());
    let diff_mode = arg_flag(&args, "--diff-mode");
    let bisect = arg_flag(&args, "--bisect");
    let js_dump_path = arg_value(&args, "--js-dump");
    let ignore: Vec<String> = arg_value(&args, "--ignore")
        .map(|s| s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect())
        .unwrap_or_default();
    let dump_grass_ticks: Option<Vec<u32>> = arg_value(&args, "--dump-grass")
        .map(|s| s.split(',').filter_map(|p| p.trim().parse::<u32>().ok()).collect())
        .or_else(|| arg_value(&args, "-dgt").map(|s| s.split(',').filter_map(|p| p.trim().parse::<u32>().ok()).collect()));
    let wasm_test = arg_flag(&args, "--wasm-test");

    if wasm_test {
        // Emit a single Snapshot JSON for the given seed/ticks to stdout. Used by
        // living/test_wasm_consumer.mjs for the RS→JS roundtrip validation.
        let daylight_series: Vec<f64> = (0..ticks as usize)
            .map(|i| 0.5 + 0.5 * aether_libm::sin((i as f64 / 300.0) * std::f64::consts::PI))
            .collect();
        let json = aether_sim_ecology::wasm::snapshot_json(
            seed,
            ticks as usize,
            &serde_json::to_string(&daylight_series).unwrap(),
        );
        let _ = writeln!(out, "{}", json);
        return;
    }

    if diff_mode || bisect {
        let path = match js_dump_path {
            Some(p) => p,
            None => {
                eprintln!("ERROR: --diff-mode/--bisect require --js-dump <path>");
                std::process::exit(2);
            }
        };
        let js = match load_js_dump(&path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("ERROR: {}", e);
                std::process::exit(2);
            }
        };
        let max_js_tick = js.keys().copied().max().unwrap_or(0);
        let _ = writeln!(out, "STRUCT_INIT_RS {}", {
            let eco = Ecology::new(seed);
            struct_summary(&eco)
        });
        if let Some(js0) = js.get(&0) {
            let (mut h, mut p, mut scav, mut omn) = (0usize, 0usize, 0usize, 0usize);
            for r in js0 {
                match r.kind.as_str() {
                    ecology::HERB => h += 1,
                    ecology::PRED => p += 1,
                    ecology::SCAV => scav += 1,
                    _ => omn += 1,
                }
            }
            let _ = writeln!(
                out,
                "STRUCT_INIT_JS herb={} pred={} scav={} omn={} total={}",
                h, p, scav, omn, js0.len()
            );
        }

        if bisect {
            // Binary search the minimal forking tick in [0, min(ticks, max_js_tick)].
            let hi = ticks.min(max_js_tick);
            if let Some(rep) = run_diff(seed, hi, &js, &ignore) {
                let _ = writeln!(out, "BISECT fork_found within_tick<={}", hi);
                let mut lo = 0u32;
                let mut high = hi;
                while lo < high {
                    let mid = lo + (high - lo) / 2;
                    if run_diff(seed, mid, &js, &ignore).is_some() {
                        high = mid;
                    } else {
                        lo = mid + 1;
                    }
                }
                // lo is the minimal forking tick; get its detailed report
                let detail = run_diff(seed, lo, &js, &ignore).expect("fork must exist at lo");
                let _ = writeln!(out, "BISECT first_fork_tick={}", lo);
                let _ = writeln!(out, "{}", detail.line);
            } else {
                let _ = writeln!(out, "BISECT no_fork up_to_tick={} checksum_aligned_within_window", hi);
            }
        } else {
            match run_diff(seed, ticks.min(max_js_tick), &js, &ignore) {
                Some(rep) => {
                    let _ = writeln!(out, "DIFF first_fork_tick={}", rep.tick);
                    let _ = writeln!(out, "{}", rep.line);
                    std::process::exit(1);
                }
                None => {
                    let _ = writeln!(out, "NO_DIFF up_to_tick={} (structurally and numerically aligned)", ticks.min(max_js_tick));
                }
            }
        }
        return;
    }

    let mut eco = Ecology::new(seed);
    let mut buf = String::new();
    if let Some(ref gt) = dump_grass_ticks {
        if gt.contains(&0) { let _ = emit_grass_dump(&eco, 0, &mut out); }
    }
    for i in 1..=ticks {
        let daylight = 0.5 + 0.5 * aether_libm::sin((i as f64 / 300.0) * std::f64::consts::PI);
        eco.update(0.1, daylight);
        if let Some(ref gt) = dump_grass_ticks {
            if gt.contains(&i) { let _ = emit_grass_dump(&eco, i, &mut out); }
        }
        if let Some(_n) = dump_every {
            if i % _n == 0 {
                dump_state(&eco, i, &mut buf);
            }
        }
        if debug || i % 300 == 0 {
            let s = eco.stats();
            let _ = writeln!(out, "SNAPSHOT {} {} {} {}", i, s.h, s.p, eco.checksum());
        }
    }
    if !buf.is_empty() {
        let _ = out.write_all(buf.as_bytes());
    }
    let _ = writeln!(out, "CHECKSUM_FINAL {}", eco.checksum());
}
