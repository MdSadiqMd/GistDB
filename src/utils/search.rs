use serde_json::Value;
use worker::Result;

pub fn search_json(data: &Value, query: &str, field: Option<&str>) -> Result<Vec<String>> {
    let mut results = Vec::new();
    if let Some(obj) = data.as_object() {
        for (id, value) in obj {
            if matches_query(value, query, field) {
                results.push(id.clone());
            }
        }
    }

    Ok(results)
}

fn matches_query(value: &Value, query: &str, field: Option<&str>) -> bool {
    match field {
        Some(f) => value
            .get(f)
            .and_then(|v| v.as_str())
            .map(|s| s.contains(query))
            .unwrap_or(false),
        None => json_contains(value, query),
    }
}

fn json_contains(value: &Value, query: &str) -> bool {
    match value {
        Value::String(s) => s.contains(query),
        Value::Array(a) => a.iter().any(|v| json_contains(v, query)),
        Value::Object(o) => o.values().any(|v| json_contains(v, query)),
        _ => false,
    }
}
