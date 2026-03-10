## Rules CLI (Rust)
 <img width="1536" height="1024" alt="ChatGPT Image Mar 10, 2026, 02_32_15 PM" src="https://github.com/user-attachments/assets/945a032c-741b-4579-925c-18396ec7ce3c" />


A Rust-based rules engine focused on spreadsheet-managed business rules and compiled native executables.

## Features

- Excel-backed rule loading (`.xlsx`, `.xlsm`, `.xls`)
- JSON rule loading (`.json`)
- CLI execution against JSON facts
- Native executable builds (`cargo build --release`)
- Multi-model rule support:
  1. Decision Table
  2. Decision Tree
  3. If–Then (Production)
  4. Scorecard
  5. Constraint
  6. Validation
  7. Event–Condition–Action (ECA)
  8. Flow

## Rule Sheet Columns

Use the first worksheet with these headers:

- `id` (string, unique)
- `enabled` (true/false)
- `order` (number, execution priority)
- `rule_type` (`decision_table`, `decision_tree`, `if_then`, `scorecard`, `constraint`, `validation`, `eca`, `flow`)
- `field` (facts key)
- `op` (`eq`, `ne`, `gt`, `gte`, `lt`, `lte`, `contains`, `starts_with`, `ends_with`, `in`)
- `value` (literal value or JSON array for `in`)
- `action` (`continue`, `approve`, `reject`, `review`)
- `score` (optional integer, used by scorecard rules)
- `message` (optional)
- `next_rule` (optional explicit chain jump)
- `next_true` (optional decision-tree true branch)
- `next_false` (optional decision-tree false branch)

## Quick Start

```bash
# Build binary
cargo build --release

# Generate template + sample facts
cargo run -- scaffold --out-dir ./examples
# Open examples/rules-template.csv in Excel and save as rules.xlsx

# Run rules
cargo run -- run --rules ./examples/rules.xlsx --facts ./examples/sample-facts.json
```

## Executable

```bash
./target/release/rules-cli run --rules ./examples/rules.xlsx --facts ./examples/sample-facts.json
```

## Test Harness

```bash
./scripts/test-harness.sh
```

Reports are written to `reports/tests/`:
- `summary.txt`
- `cargo-test.log`
- `examples.log`

The harness also executes every example `run.sh` and validates that each produces a non-empty `result.json`.

## Example Harness: DecisionTable

A runnable example is included in `DecisionTable/`:

- `DecisionTable/rules.json`
- `DecisionTable/facts.json`
- `DecisionTable/run.sh`

Run it:

```bash
./DecisionTable/run.sh
```

Result JSON is written to `DecisionTable/result.json`.

## Example Harness: DecisionTree

A runnable decision-tree example is included in `DecisionTree/`:

- `DecisionTree/rules.json`
- `DecisionTree/facts.json`
- `DecisionTree/run.sh`

Run it:

```bash
./DecisionTree/run.sh
```

Result JSON is written to `DecisionTree/result.json`.

## Example Harness: ifThen

A runnable If–Then example is included in `ifThen/`:

- `ifThen/rules.json`
- `ifThen/facts.json`
- `ifThen/run.sh`

Run it:

```bash
./ifThen/run.sh
```

Result JSON is written to `ifThen/result.json`.

## Example Harness: Scorecard

A runnable scorecard example is included in `Scorecard/`:

- `Scorecard/rules.json`
- `Scorecard/facts.json`
- `Scorecard/run.sh`

Run it:

```bash
./Scorecard/run.sh
```

Result JSON is written to `Scorecard/result.json`.

## Example Harness: Constraint

A runnable constraint-rules example is included in `Constraint/`:

- `Constraint/rules.json`
- `Constraint/facts.json`
- `Constraint/run.sh`

Run it:

```bash
./Constraint/run.sh
```

Result JSON is written to `Constraint/result.json`.

## Example Harness: Validation

A runnable validation-rules example is included in `Validation/`:

- `Validation/rules.json`
- `Validation/facts.json`
- `Validation/run.sh`

Run it:

```bash
./Validation/run.sh
```

Result JSON is written to `Validation/result.json`.

## Example Harness: ECA

A runnable Event–Condition–Action example is included in `ECA/`:

- `ECA/rules.json`
- `ECA/facts.json`
- `ECA/run.sh`

Run it:

```bash
./ECA/run.sh
```

Result JSON is written to `ECA/result.json`.

## Example Harness: Flow

A runnable flow-rules example is included in `Flow/`:

- `Flow/rules.json`
- `Flow/facts.json`
- `Flow/run.sh`

Run it:

```bash
./Flow/run.sh
```

Result JSON is written to `Flow/result.json`.

## Docs UI: Decision Table (Rust → WASM)

A lightweight browser UI concept is included in `docs/` and published via GitHub Pages.
The rules execution is powered by a Rust WASM module in `wasm-engine/`.

Live URL: http://mallond.github.io/Deterministic-Rules-Engine-AI/

Build/update the WASM bundle:

```bash
./docs/build-wasm.sh
```

Run locally:

```bash
./docs/run-ui.sh
```

Then open `http://localhost:8787`.

Use the form inputs and click **Run Decision Table** to execute the sample decision table and see output instantly.
