use crate::models::request_models::{
    CreateObjectRequest, DeleteObjectRequest, UpdateObjectRequest,
};
use crate::services::github_service::github_request;
use crate::utils::api_response::api_response;
use serde_json::{json, Map, Value};
use uuid::Uuid;
use worker::{Method, Request, Response, Result, RouteContext};

pub async fn create_object(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let token = match req.headers().get("Authorization")? {
        Some(h) => h.replace("Bearer ", ""),
        None => return api_response(401, None, "", "Authorization header required"),
    };

    let payload: CreateObjectRequest = match req.json().await {
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
    .await
    .map_err(|e| e.to_string())?;

    let file = gist["files"][&filename]
        .as_object()
        .ok_or("Collection not found")?;

    let content = file["content"].as_str().unwrap_or("{}");
    let mut data: Map<String, Value> = serde_json::from_str(content)?;

    let object_id = Uuid::new_v4().to_string();
    data.insert(object_id.clone(), payload.data.clone());

    let description = gist["description"].as_str().unwrap_or("GistDB Database");
    let update_body = json!({
        "description": description,
        "files": { filename: { "content": serde_json::to_string(&data)? } }
    });

    match github_request(
        &token,
        Method::Patch,
        &format!("https://api.github.com/gists/{}", payload.gist_id),
        Some(update_body),
    )
    .await
    {
        Ok(_) => api_response(
            201,
            Some(json!({ "object_id": object_id, "data": payload.data })),
            "Object created",
            "",
        ),
        Err(e) => api_response(500, None, "", &e.to_string()),
    }
}

pub async fn update_object(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let token = match req.headers().get("Authorization")? {
        Some(h) => h.replace("Bearer ", ""),
        None => return api_response(401, None, "", "Authorization header required"),
    };

    let payload: UpdateObjectRequest = match req.json().await {
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
    .await
    .map_err(|e| e.to_string())?;

    let file = gist["files"][&filename]
        .as_object()
        .ok_or("Collection not found")?;

    let content = file["content"].as_str().unwrap_or("{}");
    let mut data: Map<String, Value> = serde_json::from_str(content)?;

    if !data.contains_key(&payload.object_id) {
        return api_response(404, None, "", "Object not found");
    }

    data.insert(payload.object_id.to_string(), payload.data);

    let description = gist["description"].as_str().unwrap_or("GistDB Database");
    let body = json!({
        "description": description,
        "files": { filename: { "content": serde_json::to_string(&data)? } }
    });

    match github_request(
        &token,
        Method::Patch,
        &format!("https://api.github.com/gists/{}", payload.gist_id),
        Some(body),
    )
    .await
    {
        Ok(_) => api_response(
            200,
            Some(json!({ "updated": payload.object_id })),
            "Object updated",
            "",
        ),
        Err(e) => api_response(500, None, "", &e.to_string()),
    }
}

pub async fn delete_object(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let token = match req.headers().get("Authorization")? {
        Some(h) => h.replace("Bearer ", ""),
        None => return api_response(401, None, "", "Authorization header required"),
    };

    let payload: DeleteObjectRequest = match req.json().await {
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

    let file = match gist["files"].get(&filename) {
        Some(f) => f,
        None => return api_response(404, None, "", "Collection not found"),
    };

    let content = file["content"].as_str().unwrap_or("{}");
    let mut data: Map<String, Value> = serde_json::from_str(content)?;

    if !data.contains_key(&payload.object_id) {
        return api_response(404, None, "", "Object not found");
    }

    data.remove(&payload.object_id);

    let description = gist["description"].as_str().unwrap_or("GistDB Database");
    let body = json!({
        "description": description,
        "files": { filename: { "content": serde_json::to_string(&data)? } }
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
        Some(json!({ "deleted": payload.object_id })),
        "Object deleted",
        "",
    )
}
