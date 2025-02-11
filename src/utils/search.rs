use crate::models::request_models::SearchRequest;
use serde_json::Value;
use worker::Result;
use worker::*;

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

pub fn get_auth_token(req: &Request) -> Result<String> {
    req.headers()
        .get("Authorization")?
        .ok_or_else(|| worker::Error::RustError("Authorization header missing".to_string()))
        .map(|token| token.replace("Bearer ", ""))
}

pub async fn parse_body(req: &mut Request) -> Result<SearchRequest> {
    req.json()
        .await
        .map_err(|e| worker::Error::RustError(format!("Failed to parse request body: {}", e)))
}
