use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Deserialize)]
struct Rule {
    id: String,
    enabled: Option<bool>,
    order: i64,
    field: String,
    op: String,
    value: Value,
    action: String,
    message: Option<String>,
    next_rule: Option<String>,
}

#[derive(Debug, Serialize)]
struct EngineResult {
    status: String,
    fired_rules: Vec<String>,
    messages: Vec<String>,
    context: HashMap<String, Value>,
}

#[wasm_bindgen]
pub fn run_rules_json(rules_json: &str, facts_json: &str) -> Result<String, JsValue> {
    let rules: Vec<Rule> = serde_json::from_str(rules_json)
        .map_err(|e| JsValue::from_str(&format!("invalid rules json: {e}")))?;
    let facts_value: Value = serde_json::from_str(facts_json)
        .map_err(|e| JsValue::from_str(&format!("invalid facts json: {e}")))?;

    let facts_obj = facts_value
        .as_object()
        .ok_or_else(|| JsValue::from_str("facts must be a JSON object"))?;
    let facts: HashMap<String, Value> = facts_obj
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let result = run_rules(rules, facts).map_err(|e| JsValue::from_str(&e))?;
    serde_json::to_string_pretty(&result)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize result: {e}")))
}

fn run_rules(mut rules: Vec<Rule>, facts: HashMap<String, Value>) -> Result<EngineResult, String> {
    rules.retain(|r| r.enabled.unwrap_or(true));
    rules.sort_by_key(|r| r.order);

    if rules.is_empty() {
        return Err("no enabled rules".to_string());
    }

    let by_id: HashMap<String, Rule> = rules.into_iter().map(|r| (r.id.clone(), r)).collect();
    let mut current = by_id
        .values()
        .min_by_key(|r| r.order)
        .map(|r| r.id.clone());

    let mut status = "NO_MATCH".to_string();
    let mut fired_rules = Vec::new();
    let mut messages = Vec::new();

    while let Some(rule_id) = current {
        let rule = by_id
            .get(&rule_id)
            .ok_or_else(|| format!("unknown rule id: {rule_id}"))?;

        let left = facts.get(&rule.field).cloned().unwrap_or(Value::Null);
        if eval(&left, &rule.op, &rule.value)? {
            fired_rules.push(rule.id.clone());
            if let Some(m) = &rule.message {
                messages.push(m.clone());
            }
            let action = rule.action.to_ascii_lowercase();
            if action != "continue" {
                status = action.to_ascii_uppercase();
                break;
            }
            current = rule
                .next_rule
                .clone()
                .or_else(|| next_by_order(&by_id, rule.order));
        } else {
            current = next_by_order(&by_id, rule.order);
        }
    }

    Ok(EngineResult {
        status,
        fired_rules,
        messages,
        context: facts,
    })
}

fn next_by_order(by_id: &HashMap<String, Rule>, order: i64) -> Option<String> {
    by_id
        .values()
        .filter(|r| r.order > order)
        .min_by_key(|r| r.order)
        .map(|r| r.id.clone())
}

fn eval(left: &Value, op: &str, right: &Value) -> Result<bool, String> {
    match op.trim().to_ascii_lowercase().as_str() {
        "eq" | "=" | "==" => Ok(left == right),
        "ne" | "!=" => Ok(left != right),
        "gt" | ">" => Ok(to_f64(left)? > to_f64(right)?),
        "gte" | ">=" => Ok(to_f64(left)? >= to_f64(right)?),
        "lt" | "<" => Ok(to_f64(left)? < to_f64(right)?),
        "lte" | "<=" => Ok(to_f64(left)? <= to_f64(right)?),
        "contains" => Ok(to_str(left)?.contains(to_str(right)?)),
        "starts_with" | "startswith" => Ok(to_str(left)?.starts_with(to_str(right)?)),
        "ends_with" | "endswith" => Ok(to_str(left)?.ends_with(to_str(right)?)),
        _ => Err(format!("unsupported operator: {op}")),
    }
}

fn to_f64(v: &Value) -> Result<f64, String> {
    match v {
        Value::Number(n) => n.as_f64().ok_or_else(|| "number conversion failed".to_string()),
        Value::String(s) => s.parse::<f64>().map_err(|_| format!("not numeric: {s}")),
        _ => Err(format!("not numeric: {v}")),
    }
}

fn to_str(v: &Value) -> Result<&str, String> {
    v.as_str().ok_or_else(|| format!("not string: {v}"))
}
