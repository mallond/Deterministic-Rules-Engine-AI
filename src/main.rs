use anyhow::{anyhow, Context, Result};
use calamine::{open_workbook_auto, Data, Reader};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Number, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(name = "rules-cli")]
#[command(about = "Excel-first CLI rule engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Execute rules against a JSON facts payload
    Run {
        /// Path to rules file (.xlsx or .json)
        #[arg(short, long)]
        rules: String,
        /// Path to input facts file (JSON object)
        #[arg(short, long)]
        facts: String,
        /// Optional start rule id (otherwise lowest order enabled rule)
        #[arg(long)]
        start_rule: Option<String>,
        /// Optional output path. If omitted, prints JSON to stdout
        #[arg(short, long)]
        out: Option<String>,
    },
    /// Generate an Excel template and sample facts file
    Scaffold {
        /// Directory to write template files
        #[arg(short, long, default_value = ".")]
        out_dir: String,
    },
}

#[derive(Debug, Clone)]
struct Rule {
    id: String,
    enabled: bool,
    order: i64,
    rule_type: RuleType,
    field: String,
    op: Operator,
    value: Value,
    action: Action,
    score: i64,
    message: Option<String>,
    next_rule: Option<String>,
    next_true: Option<String>,
    next_false: Option<String>,
}

#[derive(Debug, Clone)]
enum RuleType {
    DecisionTable,
    DecisionTree,
    IfThen,
    Scorecard,
    Constraint,
    Validation,
    Eca,
    Flow,
}

#[derive(Debug, Clone)]
enum Operator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    StartsWith,
    EndsWith,
    In,
}

#[derive(Debug, Clone)]
enum Action {
    Approve,
    Reject,
    Review,
    Continue,
}

#[derive(Debug, Serialize, Deserialize)]
struct EngineResult {
    status: String,
    fired_rules: Vec<String>,
    messages: Vec<String>,
    total_score: i64,
    context: HashMap<String, Value>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            rules,
            facts,
            start_rule,
            out,
        } => {
            let rules = load_rules(&rules)?;
            let facts = load_facts(&facts)?;
            let result = execute_rules(rules, facts, start_rule)?;
            let payload = serde_json::to_string_pretty(&result)?;
            if let Some(out_path) = out {
                fs::write(out_path, payload)?;
            } else {
                println!("{payload}");
            }
        }
        Commands::Scaffold { out_dir } => {
            scaffold(&out_dir)?;
            println!("Created rules-template.csv and sample-facts.json in {out_dir}");
            println!("Tip: open rules-template.csv in Excel and save as rules.xlsx");
        }
    }

    Ok(())
}

fn load_rules(path: &str) -> Result<Vec<Rule>> {
    match Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "xlsx" | "xlsm" | "xls" => load_rules_from_excel(path),
        "json" => load_rules_from_json(path),
        other => Err(anyhow!(
            "Unsupported rules format: {other}. Use .xlsx or .json"
        )),
    }
}

fn load_rules_from_json(path: &str) -> Result<Vec<Rule>> {
    #[derive(Debug, Deserialize)]
    struct RuleRow {
        id: String,
        enabled: Option<bool>,
        order: i64,
        rule_type: Option<String>,
        field: String,
        op: String,
        value: Value,
        action: String,
        score: Option<i64>,
        message: Option<String>,
        next_rule: Option<String>,
        next_true: Option<String>,
        next_false: Option<String>,
    }

    let raw = fs::read_to_string(path).with_context(|| format!("reading {path}"))?;
    let rows: Vec<RuleRow> = serde_json::from_str(&raw).context("invalid rules JSON")?;

    rows.into_iter()
        .map(|row| {
            Ok(Rule {
                id: row.id,
                enabled: row.enabled.unwrap_or(true),
                order: row.order,
                rule_type: parse_rule_type(row.rule_type.as_deref().unwrap_or("if_then"))?,
                field: row.field,
                op: parse_operator(&row.op)?,
                value: row.value,
                action: parse_action(&row.action)?,
                score: row.score.unwrap_or(0),
                message: row.message,
                next_rule: row.next_rule,
                next_true: row.next_true,
                next_false: row.next_false,
            })
        })
        .collect()
}

fn load_rules_from_excel(path: &str) -> Result<Vec<Rule>> {
    let mut workbook = open_workbook_auto(path).with_context(|| format!("opening {path}"))?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("Workbook has no sheets"))?;
    let range = workbook.worksheet_range(&sheet_name)?;

    let mut rows = range.rows();
    let headers = rows
        .next()
        .ok_or_else(|| anyhow!("No header row found in sheet {sheet_name}"))?;

    let header_map: HashMap<String, usize> = headers
        .iter()
        .enumerate()
        .map(|(idx, cell)| (cell_to_string(cell).to_ascii_lowercase(), idx))
        .collect();

    let required = ["id", "enabled", "order", "field", "op", "value", "action"];
    for col in required {
        if !header_map.contains_key(col) {
            return Err(anyhow!("Missing required column: {col}"));
        }
    }

    let mut out = Vec::new();
    for row in rows {
        if row.iter().all(cell_is_empty) {
            continue;
        }

        let get = |name: &str| -> Result<&Data> {
            let idx = *header_map
                .get(name)
                .ok_or_else(|| anyhow!("missing header {name}"))?;
            row.get(idx)
                .ok_or_else(|| anyhow!("missing value for column {name}"))
        };

        let id = cell_to_string(get("id")?).trim().to_string();
        let enabled = parse_bool(get("enabled")?)?;
        let order = parse_i64(get("order")?)?;
        let rule_type = header_map
            .get("rule_type")
            .and_then(|idx| row.get(*idx))
            .map(cell_to_string)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "if_then".to_string());
        let field = cell_to_string(get("field")?).trim().to_string();
        let op = parse_operator(cell_to_string(get("op")?).trim())?;
        let value = parse_value(cell_to_string(get("value")?).trim());
        let action = parse_action(cell_to_string(get("action")?).trim())?;
        let score = header_map
            .get("score")
            .and_then(|idx| row.get(*idx))
            .map(parse_i64)
            .transpose()?
            .unwrap_or(0);
        let message = header_map
            .get("message")
            .and_then(|idx| row.get(*idx))
            .map(cell_to_string)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let next_rule = header_map
            .get("next_rule")
            .and_then(|idx| row.get(*idx))
            .map(cell_to_string)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let next_true = header_map
            .get("next_true")
            .and_then(|idx| row.get(*idx))
            .map(cell_to_string)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let next_false = header_map
            .get("next_false")
            .and_then(|idx| row.get(*idx))
            .map(cell_to_string)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        out.push(Rule {
            id,
            enabled,
            order,
            rule_type: parse_rule_type(&rule_type)?,
            field,
            op,
            value,
            action,
            score,
            message,
            next_rule,
            next_true,
            next_false,
        });
    }

    if out.is_empty() {
        return Err(anyhow!("No rules loaded from workbook"));
    }

    Ok(out)
}

fn load_facts(path: &str) -> Result<HashMap<String, Value>> {
    let raw = fs::read_to_string(path).with_context(|| format!("reading {path}"))?;
    let val: Value = serde_json::from_str(&raw).context("invalid facts JSON")?;
    let obj = val
        .as_object()
        .ok_or_else(|| anyhow!("facts must be a JSON object"))?;
    Ok(obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
}

fn execute_rules(
    mut rules: Vec<Rule>,
    facts: HashMap<String, Value>,
    start_rule: Option<String>,
) -> Result<EngineResult> {
    rules.retain(|r| r.enabled);
    rules.sort_by_key(|r| r.order);

    let by_id: HashMap<String, Rule> = rules.into_iter().map(|r| (r.id.clone(), r)).collect();
    if by_id.is_empty() {
        return Err(anyhow!("No enabled rules to execute"));
    }

    let mut current =
        start_rule.or_else(|| by_id.values().min_by_key(|r| r.order).map(|r| r.id.clone()));

    let mut fired_rules = Vec::new();
    let mut messages = Vec::new();
    let mut total_score = 0_i64;
    let mut status = "NO_MATCH".to_string();
    let context = facts.clone();

    while let Some(rule_id) = current {
        let rule = by_id
            .get(&rule_id)
            .ok_or_else(|| anyhow!("Unknown rule id: {rule_id}"))?;

        let candidate = facts.get(&rule.field).cloned().unwrap_or(Value::Null);
        let matched = eval(&candidate, &rule.op, &rule.value)?;

        if matched {
            fired_rules.push(rule.id.clone());
            if let Some(msg) = &rule.message {
                messages.push(msg.clone());
            }

            if matches!(rule.rule_type, RuleType::Scorecard) {
                total_score += rule.score;
                status = "SCORECARD".to_string();
            }

            if !matches!(rule.action, Action::Continue) {
                status = match rule.action {
                    Action::Approve => "APPROVED",
                    Action::Reject => "REJECTED",
                    Action::Review => "REVIEW",
                    Action::Continue => "CONTINUE",
                }
                .to_string();
            }

            current = next_on_true(rule, &by_id);
            if !matches!(rule.action, Action::Continue) && current.is_none() {
                break;
            }
        } else {
            current = next_on_false(rule, &by_id);
        }
    }

    Ok(EngineResult {
        status,
        fired_rules,
        messages,
        total_score,
        context,
    })
}

fn next_on_true(rule: &Rule, by_id: &HashMap<String, Rule>) -> Option<String> {
    match rule.rule_type {
        RuleType::DecisionTree => rule
            .next_true
            .clone()
            .or_else(|| rule.next_rule.clone())
            .or_else(|| next_by_order(by_id, rule.order)),
        _ => rule
            .next_rule
            .clone()
            .or_else(|| next_by_order(by_id, rule.order)),
    }
}

fn next_on_false(rule: &Rule, by_id: &HashMap<String, Rule>) -> Option<String> {
    match rule.rule_type {
        RuleType::DecisionTree => rule
            .next_false
            .clone()
            .or_else(|| next_by_order(by_id, rule.order)),
        _ => next_by_order(by_id, rule.order),
    }
}

fn next_by_order(by_id: &HashMap<String, Rule>, order: i64) -> Option<String> {
    by_id
        .values()
        .filter(|r| r.order > order)
        .min_by_key(|r| r.order)
        .map(|r| r.id.clone())
}

fn eval(left: &Value, op: &Operator, right: &Value) -> Result<bool> {
    Ok(match op {
        Operator::Eq => left == right,
        Operator::Ne => left != right,
        Operator::Gt => to_f64(left)? > to_f64(right)?,
        Operator::Gte => to_f64(left)? >= to_f64(right)?,
        Operator::Lt => to_f64(left)? < to_f64(right)?,
        Operator::Lte => to_f64(left)? <= to_f64(right)?,
        Operator::Contains => to_str(left)?.contains(to_str(right)?),
        Operator::StartsWith => to_str(left)?.starts_with(to_str(right)?),
        Operator::EndsWith => to_str(left)?.ends_with(to_str(right)?),
        Operator::In => {
            let arr = right
                .as_array()
                .ok_or_else(|| anyhow!("IN expects right side to be JSON array"))?;
            arr.contains(left)
        }
    })
}

fn parse_rule_type(rule_type: &str) -> Result<RuleType> {
    match rule_type.trim().to_ascii_lowercase().as_str() {
        "decision_table" | "decisiontable" | "table" => Ok(RuleType::DecisionTable),
        "decision_tree" | "decisiontree" | "tree" => Ok(RuleType::DecisionTree),
        "if_then" | "ifthen" | "production" => Ok(RuleType::IfThen),
        "scorecard" => Ok(RuleType::Scorecard),
        "constraint" => Ok(RuleType::Constraint),
        "validation" => Ok(RuleType::Validation),
        "eca" | "event_condition_action" => Ok(RuleType::Eca),
        "flow" => Ok(RuleType::Flow),
        _ => Err(anyhow!("Unsupported rule_type: {rule_type}")),
    }
}

fn parse_operator(op: &str) -> Result<Operator> {
    match op.trim().to_ascii_lowercase().as_str() {
        "eq" | "=" | "==" => Ok(Operator::Eq),
        "ne" | "!=" => Ok(Operator::Ne),
        "gt" | ">" => Ok(Operator::Gt),
        "gte" | ">=" => Ok(Operator::Gte),
        "lt" | "<" => Ok(Operator::Lt),
        "lte" | "<=" => Ok(Operator::Lte),
        "contains" => Ok(Operator::Contains),
        "starts_with" | "startswith" => Ok(Operator::StartsWith),
        "ends_with" | "endswith" => Ok(Operator::EndsWith),
        "in" => Ok(Operator::In),
        _ => Err(anyhow!("Unsupported operator: {op}")),
    }
}

fn parse_action(action: &str) -> Result<Action> {
    match action.trim().to_ascii_lowercase().as_str() {
        "approve" => Ok(Action::Approve),
        "reject" => Ok(Action::Reject),
        "review" => Ok(Action::Review),
        "continue" => Ok(Action::Continue),
        _ => Err(anyhow!("Unsupported action: {action}")),
    }
}

fn parse_value(raw: &str) -> Value {
    if raw.is_empty() {
        return Value::Null;
    }
    if let Ok(v) = serde_json::from_str::<Value>(raw) {
        return v;
    }
    if let Ok(i) = raw.parse::<i64>() {
        return Value::Number(Number::from(i));
    }
    if let Ok(f) = raw.parse::<f64>() {
        if let Some(n) = Number::from_f64(f) {
            return Value::Number(n);
        }
    }
    if let Ok(b) = raw.parse::<bool>() {
        return Value::Bool(b);
    }
    Value::String(raw.to_string())
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::Error(e) => format!("ERROR({e:?})"),
        Data::DateTime(dt) => dt.to_string(),
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
    }
}

fn parse_bool(cell: &Data) -> Result<bool> {
    let s = cell_to_string(cell);
    let l = s.trim().to_ascii_lowercase();
    match l.as_str() {
        "true" | "1" | "yes" | "y" => Ok(true),
        "false" | "0" | "no" | "n" => Ok(false),
        _ => Err(anyhow!("Invalid boolean value: {s}")),
    }
}

fn parse_i64(cell: &Data) -> Result<i64> {
    cell_to_string(cell)
        .trim()
        .parse::<i64>()
        .with_context(|| format!("Invalid integer: {}", cell_to_string(cell)))
}

fn cell_is_empty(cell: &Data) -> bool {
    matches!(cell, Data::Empty) || cell_to_string(cell).trim().is_empty()
}

fn to_f64(v: &Value) -> Result<f64> {
    match v {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| anyhow!("number not f64 compatible")),
        Value::String(s) => s.parse::<f64>().map_err(|_| anyhow!("not numeric: {s}")),
        _ => Err(anyhow!("not numeric: {v}")),
    }
}

fn to_str(v: &Value) -> Result<&str> {
    v.as_str().ok_or_else(|| anyhow!("not a string: {v}"))
}

fn scaffold(out_dir: &str) -> Result<()> {
    let template = "id,enabled,order,rule_type,field,op,value,action,score,message,next_rule,next_true,next_false
income_high,true,10,decision_table,annual_income,gt,100000,continue,0,Income is high,credit_good,,
credit_good,true,20,if_then,credit_score,gte,700,approve,0,Strong profile,,,
credit_low,true,30,validation,credit_score,lt,620,reject,0,Credit score too low,,,
fallback_review,true,40,flow,requested_amount,gt,250000,review,0,Large amount - send to manual review,,,
";

    let sample_facts = json!({
      "annual_income": 120000,
      "credit_score": 735,
      "requested_amount": 180000,
      "loan_type": "mortgage",
      "_event": "application_submitted"
    });

    fs::create_dir_all(out_dir)?;
    fs::write(Path::new(out_dir).join("rules-template.csv"), template)?;
    fs::write(
        Path::new(out_dir).join("sample-facts.json"),
        serde_json::to_string_pretty(&sample_facts)?,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(input: Value) -> HashMap<String, Value> {
        input
            .as_object()
            .expect("facts object")
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    #[test]
    fn supports_if_then_and_decision_table_flow() {
        let rules = vec![
            Rule {
                id: "income_high".into(),
                enabled: true,
                order: 10,
                rule_type: RuleType::DecisionTable,
                field: "annual_income".into(),
                op: Operator::Gt,
                value: json!(100000),
                action: Action::Continue,
                score: 0,
                message: Some("income ok".into()),
                next_rule: Some("credit_good".into()),
                next_true: None,
                next_false: None,
            },
            Rule {
                id: "credit_good".into(),
                enabled: true,
                order: 20,
                rule_type: RuleType::IfThen,
                field: "credit_score".into(),
                op: Operator::Gte,
                value: json!(700),
                action: Action::Approve,
                score: 0,
                message: Some("credit ok".into()),
                next_rule: None,
                next_true: None,
                next_false: None,
            },
        ];

        let result = execute_rules(
            rules,
            facts(json!({"annual_income": 120000, "credit_score": 730})),
            None,
        )
        .expect("execution should pass");

        assert_eq!(result.status, "APPROVED");
        assert_eq!(result.fired_rules, vec!["income_high", "credit_good"]);
    }

    #[test]
    fn supports_decision_tree_branching() {
        let rules = vec![
            Rule {
                id: "root".into(),
                enabled: true,
                order: 10,
                rule_type: RuleType::DecisionTree,
                field: "credit_score".into(),
                op: Operator::Gte,
                value: json!(700),
                action: Action::Continue,
                score: 0,
                message: None,
                next_rule: None,
                next_true: Some("approve".into()),
                next_false: Some("reject".into()),
            },
            Rule {
                id: "approve".into(),
                enabled: true,
                order: 20,
                rule_type: RuleType::Flow,
                field: "credit_score".into(),
                op: Operator::Gte,
                value: json!(700),
                action: Action::Approve,
                score: 0,
                message: None,
                next_rule: None,
                next_true: None,
                next_false: None,
            },
            Rule {
                id: "reject".into(),
                enabled: true,
                order: 30,
                rule_type: RuleType::Flow,
                field: "credit_score".into(),
                op: Operator::Lt,
                value: json!(700),
                action: Action::Reject,
                score: 0,
                message: None,
                next_rule: None,
                next_true: None,
                next_false: None,
            },
        ];

        let result = execute_rules(rules, facts(json!({"credit_score": 640})), None)
            .expect("execution should pass");
        assert_eq!(result.status, "REJECTED");
        assert_eq!(result.fired_rules, vec!["reject"]);
    }

    #[test]
    fn supports_scorecard_rules() {
        let rules = vec![
            Rule {
                id: "stable_income".into(),
                enabled: true,
                order: 10,
                rule_type: RuleType::Scorecard,
                field: "years_employed".into(),
                op: Operator::Gte,
                value: json!(3),
                action: Action::Continue,
                score: 30,
                message: None,
                next_rule: None,
                next_true: None,
                next_false: None,
            },
            Rule {
                id: "low_dti".into(),
                enabled: true,
                order: 20,
                rule_type: RuleType::Scorecard,
                field: "dti".into(),
                op: Operator::Lt,
                value: json!(0.4),
                action: Action::Continue,
                score: 40,
                message: None,
                next_rule: None,
                next_true: None,
                next_false: None,
            },
        ];

        let result = execute_rules(
            rules,
            facts(json!({"years_employed": 5, "dti": 0.32})),
            None,
        )
        .expect("execution should pass");

        assert_eq!(result.total_score, 70);
        assert_eq!(result.status, "SCORECARD");
    }

    #[test]
    fn supports_constraint_rules() {
        let rules = vec![Rule {
            id: "max_amount".into(),
            enabled: true,
            order: 10,
            rule_type: RuleType::Constraint,
            field: "requested_amount".into(),
            op: Operator::Gt,
            value: json!(500000),
            action: Action::Reject,
            score: 0,
            message: Some("Amount exceeds policy limit".into()),
            next_rule: None,
            next_true: None,
            next_false: None,
        }];

        let result = execute_rules(rules, facts(json!({"requested_amount": 600000})), None)
            .expect("execution should pass");

        assert_eq!(result.status, "REJECTED");
        assert_eq!(result.fired_rules, vec!["max_amount"]);
    }

    #[test]
    fn supports_validation_rules() {
        let rules = vec![Rule {
            id: "email_required".into(),
            enabled: true,
            order: 10,
            rule_type: RuleType::Validation,
            field: "email".into(),
            op: Operator::Eq,
            value: json!(null),
            action: Action::Reject,
            score: 0,
            message: Some("Email is required".into()),
            next_rule: None,
            next_true: None,
            next_false: None,
        }];

        let result = execute_rules(rules, facts(json!({"email": null})), None)
            .expect("execution should pass");

        assert_eq!(result.status, "REJECTED");
        assert_eq!(result.messages, vec!["Email is required"]);
    }

    #[test]
    fn supports_eca_rules() {
        let rules = vec![Rule {
            id: "on_submit".into(),
            enabled: true,
            order: 10,
            rule_type: RuleType::Eca,
            field: "_event".into(),
            op: Operator::Eq,
            value: json!("application_submitted"),
            action: Action::Review,
            score: 0,
            message: Some("Submitted event captured".into()),
            next_rule: None,
            next_true: None,
            next_false: None,
        }];

        let result = execute_rules(
            rules,
            facts(json!({"_event": "application_submitted"})),
            None,
        )
        .expect("execution should pass");

        assert_eq!(result.status, "REVIEW");
        assert_eq!(result.fired_rules, vec!["on_submit"]);
    }

    #[test]
    fn supports_flow_rules() {
        let rules = vec![
            Rule {
                id: "route_high".into(),
                enabled: true,
                order: 10,
                rule_type: RuleType::Flow,
                field: "requested_amount".into(),
                op: Operator::Gt,
                value: json!(250000),
                action: Action::Continue,
                score: 0,
                message: None,
                next_rule: Some("manual_review".into()),
                next_true: None,
                next_false: None,
            },
            Rule {
                id: "manual_review".into(),
                enabled: true,
                order: 20,
                rule_type: RuleType::Flow,
                field: "requested_amount".into(),
                op: Operator::Gt,
                value: json!(250000),
                action: Action::Review,
                score: 0,
                message: Some("Routed to manual review".into()),
                next_rule: None,
                next_true: None,
                next_false: None,
            },
        ];

        let result = execute_rules(rules, facts(json!({"requested_amount": 300000})), None)
            .expect("execution should pass");

        assert_eq!(result.status, "REVIEW");
        assert_eq!(result.fired_rules, vec!["route_high", "manual_review"]);
    }
}
