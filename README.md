## The World's Smallest Rules Engine

## Rust CLI Upgrade (Excel-First)

This repo now includes a Rust-based CLI (`rules-cli`) designed for complex, spreadsheet-managed rules and native executable builds.

### What’s new
- Excel-backed rule loading (`.xlsx`) for business-managed rule tables
- JSON-backed fallback for rules-as-code workflows
- CLI-driven execution (`run`) for terminal/CI use
- Scaffold command to generate a starter rule template + sample facts
- Native executable output via `cargo build --release`

### Rule sheet columns
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
- `next_rule` (optional explicit rule-chain jump)
- `next_true` (optional, decision tree true branch)
- `next_false` (optional, decision tree false branch)

### Supported rule models

1. Decision Table
2. Decision Tree
3. If–Then (Production)
4. Scorecard
5. Constraint
6. Validation
7. Event–Condition–Action (use `_event` in facts for event payload)
8. Flow

### Quick start

```bash
# 1) Build CLI
cargo build --release

# 2) Generate template files
cargo run -- scaffold --out-dir ./examples
# Open examples/rules-template.csv in Excel, edit rules, save as .xlsx

# 3) Execute with facts JSON
cargo run -- run --rules ./examples/rules.xlsx --facts ./examples/sample-facts.json
```

### Build executable

```bash
cargo build --release
./target/release/rules-cli run --rules ./examples/rules.xlsx --facts ./examples/sample-facts.json
```

---

## Legacy TypeScript Engine (kept for reference)

A small functional Rule Engine - simple maybe an understatement... 

![image](https://github.com/mallond/rules/assets/993459/b1fa993f-00ec-4d3a-b28e-816e581e2135)


History
- 02/08/16 Original Javascript without AI
- 06/25/24 New Typescript version with AI



## Intro

Simplicity is elegance. This rule engine is NOT forward chaining, backward chaining, nor is this an inference engine. What it is, is a pragmatic reactive engine. Simply stated, when there is a condition, execute an action -- most assuredly in a stateless manner. 

Prime directives, and ables ...
- Stateless
- Simple input, output, and process
- Asynchronous non-blocking invocations
- Rules to be self contained and serializable to a database (persistable)
- Rules to be versioned (versionable)
- Rules to be inventoried and stored in a library (sharable)
- Rules can be modified on-the-fly (adaptable)
- Rules can be unit tested independently (testable)
- Crazy fast

#####

[Martin Fowler on Rule Engines](http://martinfowler.com/bliki/RulesEngine.html)

# AI 06/25/24
Much has changed in the last 9 years this simple example was created. We are in the AI Age and
Prompts can do much of the work. It is the intention of this code to be used with AI prompts to 
generate versionalble rule-sets. 

```
// Rule Abilities 
{ables: [persistable, versionable, shareable, adaptable, testable, chainable, reusable, extensible]}
```

# AI Assisted Code Generation Claude Sonnet 3.5

## Upload Example Code and let AI create executable examples

![image](https://github.com/mallond/rules/assets/993459/360fb2fd-ec77-42c8-8e4b-ad29e291fb67)

## Cloude Sonnet 3.5 Generated New Code

![image](https://github.com/mallond/rules/assets/993459/80cc55a6-5ffc-4fdf-8c12-b9d6edfc7f42)

## Done-Done, like it Worked! 
- Use your imagination
- Upgrade Existing Code
- Use AI

  

## A few Use Cases for Usage

- Validations
- Mappings
- Calculations
- Batch processing
- Automated business rules - decision engine
- Automation of sequential events and process
- Home security, sprinkler systems, and darn maybe your car...

## Benefits

- Repository of rules
- Traceability
- Modify the rule not the code
- Dynamic changeability - conditions change so do the execution
- Easy to understand and modify
- Extensible
- Maintainable
- Reusable
- Chainable  
- Micro foot print and scalable

## Design - IPO (Input Process Output)


<img src="https://lh6.googleusercontent.com/-_xFQNsVja9s/U1XsZBOVi0I/AAAAAAAAG1I/a4Le6ruZDqU/w674-h502-no/rulesEngine.png" style="position:absolute; right:0px;" width="300px" />


![image](https://github.com/mallond/rules/assets/993459/c5b2c6ef-666a-4eb7-8ac6-8d2b879f0dc0)

 




