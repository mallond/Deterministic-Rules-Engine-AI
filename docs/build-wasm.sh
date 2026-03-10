#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

rustup target add wasm32-unknown-unknown >/dev/null
cargo build --manifest-path "$ROOT_DIR/wasm-engine/Cargo.toml" --target wasm32-unknown-unknown --release
wasm-bindgen "$ROOT_DIR/wasm-engine/target/wasm32-unknown-unknown/release/wasm_engine.wasm" \
  --out-dir "$ROOT_DIR/docs/pkg" \
  --target web

echo "WASM build complete: docs/pkg"
