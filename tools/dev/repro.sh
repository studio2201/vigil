#!/usr/bin/env bash
# repro.sh — build, hash, baseline-compare. Honors §16 (no toolchain pin).
set -euo pipefail
FLOOR_MINOR="96"  # rustc ≥ 1.96
command -v rustc >/dev/null 2>&1 || { echo "error: rustc not found (install via rustup)" >&2; exit 2; }
RUSTC_VER="$(rustc --version | awk '{print $2}')"
HOST_TRIPLE="$(rustc -vV | sed -n 's|host: ||p')"
MINOR="${RUSTC_VER#*.}"; MINOR="${MINOR%%.*}"
[ "$MINOR" -ge "$FLOOR_MINOR" ] || { echo "error: rustc $RUSTC_VER below floor 1.$FLOOR_MINOR" >&2; exit 3; }
cargo build --locked --release --offline
BIN="target/release/vigil"
[ -f "$BIN" ] || { echo "error: $BIN not produced" >&2; exit 4; }
SHA="$(sha256sum "$BIN" | awk '{print $1}')"
echo "rustc: $RUSTC_VER"; echo "host: $HOST_TRIPLE"; echo "sha256: $SHA"
BASELINE_DIR="tools/dev/baselines"; mkdir -p "$BASELINE_DIR"
BASELINE_FILE="${BASELINE_DIR}/${HOST_TRIPLE}__rustc-${RUSTC_VER}.sha256"
if [ ! -f "$BASELINE_FILE" ]; then
  echo "$SHA  $BIN" > "$BASELINE_FILE"; echo "baseline recorded: $BASELINE_FILE"; exit 0
fi
EXPECTED="$(awk '{print $1}' "$BASELINE_FILE")"
if [ "$EXPECTED" != "$SHA" ]; then
  echo "MISMATCH: expected=$EXPECTED actual=$SHA" >&2; exit 5
fi
echo "reproducible: OK"
