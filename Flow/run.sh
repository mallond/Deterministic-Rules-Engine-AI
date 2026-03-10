#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXAMPLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_FILE="${EXAMPLE_DIR}/result.json"

if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

cd "$ROOT_DIR"

echo "Building release binary..."
cargo build --release

echo "Running Flow example..."
./target/release/rules-cli run \
  --rules "${EXAMPLE_DIR}/rules.json" \
  --facts "${EXAMPLE_DIR}/facts.json" \
  --out "$OUT_FILE"

echo "Done. Result written to: $OUT_FILE"
cat "$OUT_FILE"
