use crate::models::request_models::{CreateCollectionRequest, DeleteCollectionRequest};
use crate::services::github_service::github_request;
use crate::utils::api_response::api_response;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use worker::{Method, Request, Response, Result, RouteContext};

pub async fn create_collection(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let token = match req.headers().get("Authorization")? {
        Some(h) => h.replace("Bearer ", ""),
        None => return api_response(401, None, "", "Authorization header required"),
    };

    let payload: CreateCollectionRequest = match req.json().await {
        Ok(p) => p,
        Err(_) => return api_response(400, None, "", "Invalid request body"),
    };

    let filename = format!("{}.json", payload.name);
    let gist_id = payload.gist_id;

    let existing = github_request(
        &token,
        Method::Get,
        &format!("https://api.github.com/gists/{}", gist_id),
        None,
    )
    .await
    .map_err(|e| e.to_string())?;

    if existing["files"][&filename].is_object() {
        return api_response(409, None, "", "Collection already exists");
    }

    let description = existing["description"]
        .as_str()
        .unwrap_or("GistDB Database");
    let update_body = json!({
        "description": description,
        "files": { filename: { "content": "{}" } }
    });

    match github_request(
        &token,
        Method::Patch,
        &format!("https://api.github.com/gists/{}", gist_id),
        Some(update_body),
    )
    .await
    {
        Ok(_) => api_response(
            201,
            Some(json!({ "collection_name": payload.name })),
            "Collection created",
            "",
        ),
        Err(e) => api_response(500, None, "", &e.to_string()),
    }
}

pub async fn get_collection(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let token = match req.headers().get("Authorization")? {
        Some(h) => h.replace("Bearer ", ""),
        None => return api_response(401, None, "", "Authorization header required"),
    };

    let gist_id = match ctx.param("gistId") {
        Some(id) => id,
        None => return api_response(400, None, "", "Missing gist ID"),
    };

    let url = req.url()?;
    let query_params: HashMap<_, _> = url.query_pairs().into_owned().collect();
    let collection_name = query_params.get("collection_name").map(|s| s.to_string());

    let gist = github_request(
        &token,
        Method::Get,
        &format!("https://api.github.com/gists/{}", gist_id),
        None,
    )
    .await
    .map_err(|e| e.to_string())?;

    if let Some(collection_name) = collection_name {
        let filename = format!("{}.json", collection_name);
        match gist["files"].get(&filename) {
            Some(file) => {
                let content = file["content"].as_str().unwrap_or("{}");
                let data: Value = serde_json::from_str(content)?;
                api_response(200, Some(data), "Collection contents", "")
            }
            None => api_response(404, None, "", "Collection not found"),
        }
    } else {
        let mut result = Map::new();
        for (filename, file) in gist["files"].as_object().unwrap_or(&Map::new()) {
            let content = file["content"].as_str().unwrap_or("{}");
            result.insert(
                filename.clone(),
                serde_json::from_str(content).unwrap_or(Value::Null),
            );
        }
        api_response(200, Some(Value::Object(result)), "Database contents", "")
    }
}

pub async fn delete_collection(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let token = match req.headers().get("Authorization")? {
        Some(h) => h.replace("Bearer ", ""),
        None => return api_response(401, None, "", "Authorization header required"),
    };

    let payload: DeleteCollectionRequest = match req.json().await {
        Ok(p) => p,
        Err(_) => return api_response(400, None, "", "Invalid request body"),
    };

    let filename = format!("{}.json", payload.collection_name);

    let gist = github_request(
        &token,
        Method::Get,
        &format!("https://api.github.com/gists/{}", payload.gist_id),
        None,
    )
    .await?;

    if let Some(files) = gist["files"].as_object() {
        if !files.contains_key(&filename) {
            return api_response(404, None, "", "Collection not found");
        }
    } else {
        return api_response(500, None, "", "Internal Server Error");
    }

    let description = gist["description"].as_str().unwrap_or("GistDB Database");
    let body = json!({
        "description": description,
        "files": { filename: null }
    });

    github_request(
        &token,
        Method::Patch,
        &format!("https://api.github.com/gists/{}", payload.gist_id),
        Some(body),
    )
    .await?;

    api_response(
        200,
        Some(json!({ "deleted_collection": payload.collection_name })),
        "Collection deleted",
        "",
    )
}
