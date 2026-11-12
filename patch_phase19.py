#!/usr/bin/env python3
"""Phase 19 — Morphology Pipeline: minimal Rust patch.

Adds two purely-visual f64 fields (hull_ratio, limb_scale) to Genome.
They are derived deterministically from the agent ID via a 16-bit FNV split,
so NO RNG stream is consumed → checksum invariant.

Files touched:
  core/crates/aether-sim-ecology/src/ecology.rs
  core/crates/aether-sim-ecology/src/save_state.rs

Apply: python3 patch_phase19.py
"""

import os, sys, re

ROOT = os.path.dirname(os.path.abspath(__file__))
ECO = os.path.join(ROOT, "core/crates/aether-sim-ecology/src/ecology.rs")
SAVE = os.path.join(ROOT, "core/crates/aether-sim-ecology/src/save_state.rs")


def patch_ecology():
    with open(ECO, "r") as f:
        src = f.read()

    # ── 1. Genome struct: add two fields after deco_efficiency ──
    old = "    pub deco_efficiency: f64,\n}"
    new = "    pub deco_efficiency: f64,\n    pub hull_ratio: f64,\n    pub limb_scale: f64,\n}"
    assert src.count(old) == 1, f"Genome struct anchor not found (count={src.count(old)})"
    src = src.replace(old, new)

    # ── 2. default_deco(): add fields ──
    old = "            fin_color: 0.0, deco_efficiency: 1.0,\n        }\n    }\n}"
    new = "            fin_color: 0.0, deco_efficiency: 1.0,\n            hull_ratio: 1.0, limb_scale: 3.0,\n        }\n    }\n}"
    assert src.count(old) == 1, f"default_deco anchor not found"
    src = src.replace(old, new)

    # ── 3. Add FNV helper just before the random_genome function ──
    old = "    fn random_genome(&mut self, kind: &'static str) -> Genome {"
    new = """    /// Phase 19: deterministic morphology from agent ID (no RNG consumed).
    /// Uses a 16-bit FNV split so JS can replicate from SLOT_ID.
    #[inline]
    fn morpho_pair(id: u32) -> (f64, f64) {
        let h = id.wrapping_mul(0x9E3779B9);
        let hr = 0.6 + ((h & 0xFFFF) as f64 / 65535.0) * 0.8;
        let ls = 2.0 + (((h >> 16) & 0xFFFF) as f64 / 65535.0) * 2.0;
        (hr, ls)
    }

    fn random_genome(&mut self, kind: &'static str) -> Genome {"""
    assert src.count(old) == 1, f"random_genome anchor not found"
    src = src.replace(old, new)

    # ── 4. Inside random_genome: add let-binding at top of HERB branch ──
    # Insert after the first `if kind == HERB {` line, before the Genome literal
    old = """    fn random_genome(&mut self, kind: &'static str) -> Genome {
        if kind == HERB {
            Genome {"""
    new = """    fn random_genome(&mut self, kind: &'static str) -> Genome {
        let (hr, ls) = Self::morpho_pair(self.next_id);
        if kind == HERB {
            Genome {"""
    assert src.count(old) == 1, f"random_genome HERB entry anchor not found"
    src = src.replace(old, new)

    # ── 5. Add fields to all 4 Genome literals in random_genome ──
    # Each ends with "deco_efficiency: ..." followed by "}"
    # Pattern: after the last deco_efficiency line in each branch, add our fields
    for match_start in [m.start() for m in re.finditer(r"deco_efficiency: [^,]+,\n\s*\}", src)]:
        pass  # We'll handle this more precisely below

    # Actually, let's do it line by line: in random_genome, each branch has
    # `deco_efficiency: 1.0,` or `deco_efficiency: 0.5 + self.rng.next_f64(),`
    # followed by a closing `}`. We need to insert `hull_ratio: hr, limb_scale: ls,` before the `}`.
    # There are 4 such occurrences in random_genome.

    # Find the random_genome function body and patch each closing
    rg_start = src.index("    fn random_genome")
    # Find the end of random_genome (next fn after it)
    next_fn = src.index("\n    fn ", rg_start + 10)
    rg_body = src[rg_start:next_fn]

    # Count occurrences of the pattern in random_genome
    count = rg_body.count("deco_efficiency: 1.0,\n            }")
    count += rg_body.count("deco_efficiency: 0.5 + self.rng.next_f64(),\n            }")
    assert count == 4, f"Expected 4 deco_efficiency closings in random_genome, found {count}"

    # Replace in random_genome body only
    rg_new = rg_body.replace(
        "deco_efficiency: 1.0,\n            }",
        "deco_efficiency: 1.0,\n            hull_ratio: hr, limb_scale: ls,\n            }"
    )
    rg_new = rg_new.replace(
        "deco_efficiency: 0.5 + self.rng.next_f64(),\n            }",
        "deco_efficiency: 0.5 + self.rng.next_f64(),\n            hull_ratio: hr, limb_scale: ls,\n            }"
    )
    src = src[:rg_start] + rg_new + src[next_fn:]

    # ── 6. mix_genome: add average of parents ──
    old = "            deco_efficiency: (a.deco_efficiency + b.deco_efficiency) * 0.5,\n        };\n        if let"
    new = "            deco_efficiency: (a.deco_efficiency + b.deco_efficiency) * 0.5,\n            hull_ratio: (a.hull_ratio + b.hull_ratio) * 0.5,\n            limb_scale: (a.limb_scale + b.limb_scale) * 0.5,\n        };\n        if let"
    assert src.count(old) == 1, f"mix_genome anchor not found"
    src = src.replace(old, new)

    # ── 7. mean_genome: add accumulation + division ──
    old = "            fin_color: 0.0, deco_efficiency: 0.0,\n        };\n        let mut opt_sum"
    new = "            fin_color: 0.0, deco_efficiency: 0.0,\n            hull_ratio: 0.0, limb_scale: 0.0,\n        };\n        let mut opt_sum"
    assert src.count(old) == 1, f"mean_genome init anchor not found"
    src = src.replace(old, new)

    old = "            acc.fin_color += g.fin_color;\n            acc.deco_efficiency += g.deco_efficiency;\n            for"
    new = "            acc.fin_color += g.fin_color;\n            acc.deco_efficiency += g.deco_efficiency;\n            acc.hull_ratio += g.hull_ratio;\n            acc.limb_scale += g.limb_scale;\n            for"
    assert src.count(old) == 1, f"mean_genome acc loop anchor not found"
    src = src.replace(old, new)

    old = "acc.deco_efficiency /= n;\n"
    # Find the context to disambiguate
    old_ctx = "acc.social /= n;\n        acc.fin_color /= n;\n        acc.deco_efficiency /= n;\n"
    new_ctx = "acc.social /= n;\n        acc.fin_color /= n;\n        acc.deco_efficiency /= n;\n        acc.hull_ratio /= n;\n        acc.limb_scale /= n;\n"
    assert src.count(old_ctx) == 1, f"mean_genome division anchor not found"
    src = src.replace(old_ctx, new_ctx)

    # ── 8. genome_to_save: add fields ──
    old = "        fin_color: g.fin_color, deco_efficiency: g.deco_efficiency,\n    }\n}\n\nfn genome_from_save"
    new = "        fin_color: g.fin_color, deco_efficiency: g.deco_efficiency,\n        hull_ratio: g.hull_ratio, limb_scale: g.limb_scale,\n    }\n}\n\nfn genome_from_save"
    assert src.count(old) == 1, f"genome_to_save anchor not found"
    src = src.replace(old, new)

    # ── 9. genome_from_save: add fields ──
    old = "        fin_color: g.fin_color, deco_efficiency: g.deco_efficiency,\n    }\n}\n\nfn agent_to_save"
    new = "        fin_color: g.fin_color, deco_efficiency: g.deco_efficiency,\n        hull_ratio: g.hull_ratio, limb_scale: g.limb_scale,\n    }\n}\n\nfn agent_to_save"
    assert src.count(old) == 1, f"genome_from_save anchor not found"
    src = src.replace(old, new)

    with open(ECO, "w") as f:
        f.write(src)
    print(f"  patched {os.path.relpath(ECO, ROOT)}")


def patch_save_state():
    with open(SAVE, "r") as f:
        src = f.read()

    # Add fields to SaveGenome with #[serde(default)]
    old = "    pub fin_color: f64,\n    pub deco_efficiency: f64,\n}"
    new = """    pub fin_color: f64,
    pub deco_efficiency: f64,
    #[serde(default)]
    pub hull_ratio: f64,
    #[serde(default)]
    pub limb_scale: f64,
}"""
    assert src.count(old) == 1, f"SaveGenome anchor not found (count={src.count(old)})"
    src = src.replace(old, new)

    with open(SAVE, "w") as f:
        f.write(src)
    print(f"  patched {os.path.relpath(SAVE, ROOT)}")


if __name__ == "__main__":
    print("Phase 19 — Rust morphology patch:")
    patch_ecology()
    patch_save_state()
    print("Done. Run `cargo build` to verify compilation.")
