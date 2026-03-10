#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="${ROOT_DIR}/reports/tests"
TS_JSON="${REPORT_DIR}/jest-results.json"
RUST_LOG="${REPORT_DIR}/cargo-test.log"
SUMMARY="${REPORT_DIR}/summary.txt"

mkdir -p "$REPORT_DIR"

if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

export npm_config_cache="${ROOT_DIR}/.npm-cache"

RUST_STATUS=0
TS_STATUS=0

{
  echo "=== Test Harness Summary ==="
  date
  echo "Repo: $ROOT_DIR"
  echo
} > "$SUMMARY"

pushd "$ROOT_DIR" >/dev/null

if command -v cargo >/dev/null 2>&1; then
  echo "[1/2] Running Rust tests..."
  set +e
  cargo test | tee "$RUST_LOG"
  RUST_STATUS=${PIPESTATUS[0]}
  set -e
  {
    echo "Rust status: $RUST_STATUS"
    echo "Rust summary:"
    grep -E "^test result:" "$RUST_LOG" || echo "(no rust summary line found)"
    echo
  } >> "$SUMMARY"
else
  echo "[1/2] Skipping Rust tests (cargo not found)"
  RUST_STATUS=127
  {
    echo "Rust status: 127"
    echo "Rust summary: cargo not found"
    echo
  } >> "$SUMMARY"
fi

if [[ -f package.json ]]; then
  echo "[2/2] Running TypeScript/Jest tests..."
  if [[ ! -d node_modules ]]; then
    echo "Installing Node dependencies (npm ci)..."
    npm ci
  fi

  set +e
  npx jest --runInBand --passWithNoTests --json --outputFile "$TS_JSON"
  TS_STATUS=$?
  set -e

  {
    echo "TypeScript/Jest status: $TS_STATUS"
    if [[ -f "$TS_JSON" ]]; then
      node -e '
        const fs = require("fs");
        const p = process.argv[1];
        const d = JSON.parse(fs.readFileSync(p, "utf8"));
        console.log(`Jest summary: total=${d.numTotalTests}, passed=${d.numPassedTests}, failed=${d.numFailedTests}, suites=${d.numTotalTestSuites}`);
      ' "$TS_JSON"
    else
      echo "Jest summary: no JSON report found"
    fi
    echo
  } >> "$SUMMARY"
else
  echo "[2/2] Skipping TypeScript/Jest tests (no package.json)"
  TS_STATUS=0
fi

popd >/dev/null

cat "$SUMMARY"

overall=0
if [[ $RUST_STATUS -ne 0 ]]; then
  overall=1
fi
if [[ $TS_STATUS -ne 0 ]]; then
  overall=1
fi

if [[ $overall -ne 0 ]]; then
  echo "Test harness failed. See reports in $REPORT_DIR" >&2
  exit 1
fi

echo "Test harness passed. Reports written to $REPORT_DIR"
