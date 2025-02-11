use crate::models::request_models::SearchRequest;
use crate::services::github_service;
use crate::utils::api_response::api_response;
use crate::utils::{cache, search};
use serde_json::{json, Value};
use worker::Cache;
use worker::*;

fn get_auth_token(req: &Request) -> Result<String> {
    req.headers()
        .get("Authorization")?
        .ok_or_else(|| worker::Error::RustError("Authorization header missing".to_string()))
        .map(|token| token.replace("Bearer ", ""))
}

async fn parse_body(req: &mut Request) -> Result<SearchRequest> {
    req.json()
        .await
        .map_err(|e| worker::Error::RustError(format!("Failed to parse request body: {}", e)))
}

pub async fn search_objects(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let token = get_auth_token(&req)?;
    let payload: SearchRequest = parse_body(&mut req).await?;

    let filename = format!("{}.json", payload.collection_name);
    let cache_key = format!(
        "https://gistdb.com/search/{}/{}",
        payload.gist_id, payload.query
    );

    let cache = Cache::default();

    let results = cache::cached_search(&cache, &cache_key, async {
        let content = github_service::get_gist_file(&token, &payload.gist_id, &filename).await?;
        let data: Value = serde_json::from_str(&content)
            .map_err(|e| worker::Error::RustError(format!("Failed to parse JSON: {}", e)))?;
        let search_results = search::search_json(&data, &payload.query, payload.field.as_deref())?;
        Ok(json!(search_results))
    })
    .await?;

    api_response(200, Some(results), "Search completed", "")
}
