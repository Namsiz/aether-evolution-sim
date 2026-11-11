#!/usr/bin/env bash
# diff_rs_vs_js.sh — differential testing driver for JS oracle vs Rust sim
#
# Usage:
#   scripts/diff_rs_vs_js.sh [--seed SEED] [--ticks N] [--compare checksum|counts] [--bisect]
#
# Default mode (legacy, Phase 1/2 checksum diff):
#   Runs the JS ecology oracle and the Rust headless sim for N ticks and diffs
#   their SNAPSHOT lines + final checksum.
#
# --bisect mode (P0 fork locator):
#   1. Node oracle writes a per-tick STATE_DUMP file (scripts/dump_state_diff.mjs --js-dump-only).
#   2. Rust binary runs its in-process binary search (--bisect --js-dump <file>)
#      over ticks [0, N] and reports the first forking tick with the first
#      differing agent id / field / JS-vs-RS values.
#   3. A second bisect pass with `--ignore lineage` skips the known JS-only
#      speciation lineage tags to expose the first behavioral fork.
#   4. Prints init structure alignment (agent counts by kind, both sides) and
#      the final checksum comparison.

set -euo pipefail

# Ensure cargo/rustc are on PATH (needed for the Rust validation step below)
if [[ -z "${PATH//:/}" || ! -x "$(command -v cargo 2>/dev/null)" ]]; then
  export PATH="$HOME/.cargo/bin:$PATH"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LIVING_ROOT="$PROJECT_ROOT/../living"

SEED=20261102
TICKS=3000
COMPARE="checksum"
BISECT=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --seed)    SEED="$2"; shift 2 ;;
    --ticks)   TICKS="$2"; shift 2 ;;
    --compare) COMPARE="$2"; shift 2 ;;
    --bisect)  BISECT=1; shift ;;
    *) echo "Unknown arg: $1"; exit 2 ;;
  esac
done

if [[ ! -d "$LIVING_ROOT" ]]; then
  echo "[ERROR] living/ directory not found at $LIVING_ROOT" >&2
  exit 2
fi

# ---------- --bisect mode (P0 fork locator) ----------
if [[ "$BISECT" == "1" ]]; then
  JS_DUMP=$(mktemp /tmp/state_dump_js_${SEED}.XXXXXX.tsv)
  trap 'rm -f "$JS_DUMP"' EXIT

  echo "===== [1/4] JS oracle STATE_DUMP (seed=${SEED}, ticks=${TICKS}) ====="
  node "$SCRIPT_DIR/dump_state_diff.mjs" --js-dump-only "$JS_DUMP" --seed "$SEED" --ticks "$TICKS"

  cd "$PROJECT_ROOT"
  cargo build -p aether-sim-ecology 2>/dev/null
  BIN="$PROJECT_ROOT/target/debug/ecology-sim"

  echo ""
  echo "===== [2/4] Bisect pass 1: raw (all fields) ====="
  "$BIN" --seed "$SEED" --ticks "$TICKS" --bisect --js-dump "$JS_DUMP"

  echo ""
  echo "===== [3/4] Bisect pass 2: --ignore lineage (skip JS-only speciation tags) ====="
  "$BIN" --seed "$SEED" --ticks "$TICKS" --bisect --js-dump "$JS_DUMP" --ignore lineage || true

  echo ""
  echo "===== [4/4] Final checksum comparison (ticks=${TICKS}) ====="
  JS_CK=$(node --input-type=module -e "
import { Ecology } from '${LIVING_ROOT}/src/ecology.js';
const eco = new Ecology(${SEED});
for (let i = 1; i <= ${TICKS}; i++) {
  const daylight = 0.5 + 0.5 * Math.sin((i / 300) * Math.PI);
  eco.update(0.1, daylight);
}
console.log(eco.checksum());
" 2>/dev/null)
  RS_CK=$("$BIN" --seed "$SEED" --ticks "$TICKS" | awk '/^CHECKSUM_FINAL/ {print $2}')
  echo "JS  checksum: ${JS_CK}"
  echo "RS  checksum: ${RS_CK}"
  if [[ "$JS_CK" == "$RS_CK" ]]; then
    echo "[PASS] checksums identical"
  else
    echo "[FAIL] checksums differ (分叉点之后的数值漂移；分叉 tick 见上方 BISECT 输出)"
  fi
  exit 0
fi

# ---------- legacy checksum/counts diff mode ----------
JS_OUT=$(mktemp /tmp/js_oracle.XXXXXX.tsv)
RS_OUT=$(mktemp /tmp/rs_sim.XXXXXX.tsv)
trap 'rm -f "$JS_OUT" "$RS_OUT"' EXIT

# --- JS oracle run ---
node --input-type=module << JSEOF > "$JS_OUT" 2>/dev/null
import { Ecology } from '${LIVING_ROOT}/src/ecology.js';
const eco = new Ecology(${SEED});
const totalTicks = ${TICKS};
for (let i = 1; i <= totalTicks; i++) {
  const daylight = 0.5 + 0.5 * Math.sin((i / 300) * Math.PI);
  eco.update(0.1, daylight);
  if (i % 50 === 0 || i === ${TICKS}) {
    const s = eco.stats();
    console.log('SNAPSHOT', i, s.h, s.p, eco.checksum());
  }
}
console.log('CHECKSUM_FINAL', eco.checksum());
JSEOF

echo "[INFO] JS oracle ran ${TICKS} ticks (seed=${SEED}), output → ${JS_OUT}"

# --- Rust validation run ---
cd "$PROJECT_ROOT"
cargo test -p aether-rng --release 2>&1 | tail -5 || true
echo "[INFO] aether-rng unit tests complete"

cargo run -p aether-sim-ecology -- --seed ${SEED} --ticks ${TICKS} 2>/dev/null | while read line; do
  case "$line" in
    SNAPSHOT*)
      tick=$(echo "$line" | awk '{print $2}')
      h=$(echo "$line" | awk '{print $3}')
      p=$(echo "$line" | awk '{print $4}')
      ck=$(echo "$line" | awk '{print $5}')
      echo "SNAPSHOT ${tick} ${h} ${p} ${ck}"
      ;;
    CHECKSUM_FINAL*)
      ck=$(echo "$line" | awk '{print $2}')
      echo "CHECKSUM_FINAL ${ck}"
      ;;
  esac
done > "$RS_OUT"

# --- Diff ---
echo ""
echo "========== DIFF JS vs RUST =========="
diff -u "$JS_OUT" "$RS_OUT" || true
echo "======================================"

if diff -q "$JS_OUT" "$RS_OUT" >/dev/null 2>&1; then
  echo "[PASS] JS and Rust outputs are identical (seed=${SEED}, ticks=${TICKS})"
  exit 0
else
  echo "[FAIL] Divergence detected between JS oracle and Rust sim"
  exit 1
fi
