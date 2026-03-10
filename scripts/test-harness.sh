#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_DIR="${ROOT_DIR}/reports/tests"
RUST_LOG="${REPORT_DIR}/cargo-test.log"
EXAMPLES_LOG="${REPORT_DIR}/examples.log"
SUMMARY="${REPORT_DIR}/summary.txt"

mkdir -p "$REPORT_DIR"

if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi

{
  echo "=== Rust Test Harness Summary ==="
  date
  echo "Repo: $ROOT_DIR"
  echo
} > "$SUMMARY"

pushd "$ROOT_DIR" >/dev/null

if command -v cargo >/dev/null 2>&1; then
  echo "Running Rust tests..."
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
  echo "Skipping Rust tests (cargo not found)"
  RUST_STATUS=127
  {
    echo "Rust status: 127"
    echo "Rust summary: cargo not found"
    echo
  } >> "$SUMMARY"
fi

echo "Running example harness validations..." | tee "$EXAMPLES_LOG"
EXAMPLE_STATUS=0
examples=(
  "DecisionTable"
  "DecisionTree"
  "ifThen"
  "Scorecard"
  "Constraint"
  "Validation"
  "ECA"
  "Flow"
)

for example in "${examples[@]}"; do
  script_path="${ROOT_DIR}/${example}/run.sh"
  result_path="${ROOT_DIR}/${example}/result.json"

  if [[ ! -x "$script_path" ]]; then
    echo "[FAIL] ${example}: missing executable run script (${script_path})" | tee -a "$EXAMPLES_LOG"
    EXAMPLE_STATUS=1
    continue
  fi

  set +e
  "$script_path" >> "$EXAMPLES_LOG" 2>&1
  run_status=$?
  set -e

  if [[ $run_status -ne 0 ]]; then
    echo "[FAIL] ${example}: run script exited ${run_status}" | tee -a "$EXAMPLES_LOG"
    EXAMPLE_STATUS=1
    continue
  fi

  if [[ ! -s "$result_path" ]]; then
    echo "[FAIL] ${example}: missing/empty result file (${result_path})" | tee -a "$EXAMPLES_LOG"
    EXAMPLE_STATUS=1
    continue
  fi

  echo "[OK] ${example}" | tee -a "$EXAMPLES_LOG"
done

{
  echo "Examples status: $EXAMPLE_STATUS"
  echo "Examples log: $EXAMPLES_LOG"
  echo
} >> "$SUMMARY"

popd >/dev/null

cat "$SUMMARY"

if [[ ${RUST_STATUS:-1} -ne 0 || ${EXAMPLE_STATUS:-1} -ne 0 ]]; then
  echo "Rust test harness failed. See reports in $REPORT_DIR" >&2
  exit 1
fi

echo "Rust test harness passed. Reports written to $REPORT_DIR"
