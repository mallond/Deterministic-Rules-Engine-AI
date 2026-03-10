#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${1:-8787}"

cd "$ROOT_DIR/DecisionTableUI"

echo "Starting DecisionTable UI at http://localhost:${PORT}"
python3 -m http.server "$PORT"
