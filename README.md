## Rules CLI (Rust, Excel-First)

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
