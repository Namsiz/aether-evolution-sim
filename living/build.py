#!/usr/bin/env python3
"""把 living 源码打包成两个可离线打开的单文件 HTML：
- aether.html       JS 生态路径（原有，零行为变化）
- wasm-viewer.html  WASM 快照路径（新增：内联 app-wasm.js + base64 wasm）
"""

import base64
import os
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parent
APP = ROOT.parent / "aether.html"
WASM_VIEWER_SRC = ROOT / "src" / "wasm-viewer.template.html"
WASM_VIEWER = ROOT / "wasm-viewer.html"
WASM = ROOT / "ecology-sim.wasm"

ENV = {**os.environ, "PATH": os.path.expanduser("~/.cargo/bin") + os.pathsep + os.environ.get("PATH", "")}


def run(cmd, **kw):
    subprocess.run(cmd, cwd=ROOT, check=True, **kw)


def build_wasm():
    """Compile the Rust sim to wasm32 and copy it next to the HTML entries.

    Uses the workspace's purpose-built `wasm-release` profile (opt-level=z +
    fat LTO + panic=abort): ~480 KB vs ~640 KB under plain --release. The
    Phase 15 size budget (≤ 515 KB) is measured against this profile.
    """
    core = ROOT.parent / "core"
    target = core / "target" / "wasm32-unknown-unknown" / "wasm-release" / "aether_sim_ecology.wasm"
    try:
        subprocess.run(
            ["cargo", "build", "--profile", "wasm-release", "--target", "wasm32-unknown-unknown",
             "-p", "aether-sim-ecology"],
            cwd=core, check=True, env=ENV,
        )
    except (OSError, subprocess.CalledProcessError) as e:
        if WASM.exists():
            print(f"! cargo build failed ({e}); reusing existing {WASM.name}")
            return
        raise SystemExit("cargo build failed and no prebuilt ecology-sim.wasm present") from e
    WASM.write_bytes(target.read_bytes())
    # wasm-viewer.html fetches the versioned filename (cache-bust on rename);
    # keep the unversioned copy in sync for direct consumers/tests.
    (ROOT / "ecology-sim-v2.wasm").write_bytes(target.read_bytes())


def inline(html: str, js: str, css: str) -> str:
    html = html.replace(
        '<link rel="stylesheet" href="style.css">',
        "<style>\n" + css + "\n</style>",
    )
    return html.replace(
        f'<script src="{js}"></script>',
        "<script>\n" + (ROOT / js).read_text(encoding="utf-8") + "\n</script>",
    )


def main():
    run(["npx", "esbuild", "src/main.js", "--bundle", "--format=iife",
         "--minify", "--outfile=app.js"])
    run(["npx", "esbuild", "src/wasm-viewer.mjs", "--bundle", "--format=iife",
         "--minify", "--outfile=app-wasm.js"])

    build_wasm()

    css = (ROOT / "style.css").read_text(encoding="utf-8")

    # 1) JS-ecology single-file build (unchanged behaviour).
    html = inline((ROOT / "index.html").read_text(encoding="utf-8"), "app.js", css)
    APP.write_text(html, encoding="utf-8")
    print(f"-> {APP}  {APP.stat().st_size / 1024:.0f} KB")

    # 2) WASM single-file build: inline app-wasm.js.
    #    The wasm binary must be served alongside the HTML (same origin, same dir)
    #    so fetch('./ecology-sim.wasm') works — use `python3 -m http.server` or
    #    a real web server for production. file:// URLs cannot fetch wasm.
    wv = inline(WASM_VIEWER_SRC.read_text(encoding="utf-8"), "app-wasm.js", css)
    out = WASM_VIEWER
    out.write_text(wv, encoding="utf-8")
    print(f"-> {out}  {out.stat().st_size / 1024:.0f} KB (wasm external)")


if __name__ == "__main__":
    main()
