use crate::models::request_models::{CreateDatabaseRequest, DeleteDatabaseRequest};
use crate::services::github_service::github_request;
use crate::utils::api_response::api_response;
use serde_json::json;
use worker::{Method, Request, Response, Result, RouteContext};

pub async fn create_database(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let headers = req.headers();
    let token = headers
        .get("Authorization")?
        .unwrap_or_default()
        .replace("Bearer ", "");

    let payload: CreateDatabaseRequest = match req.json().await {
        Ok(p) => p,
        Err(_) => return api_response(400, None, "", "Invalid request body"),
    };

    let filename = format!("{}.json", payload.name);
    let body = json!({
        "description": payload.name,
        "public": false,
        "files": {
            filename: {
                "content": "{}"
            }
        }
    });

    match github_request(
        &token,
        Method::Post,
        "https://api.github.com/gists",
        Some(body),
    )
    .await
    {
        Ok(res) => api_response(
            201,
            Some(json!({ "gist_id": res["id"], "collection_name": payload.name })),
            "Database initialized",
            "",
        ),
        Err(e) => api_response(500, None, "", &e.to_string()),
    }
}

pub async fn delete_database(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let token = match req.headers().get("Authorization")? {
        Some(h) => h.replace("Bearer ", ""),
        None => return api_response(401, None, "", "Authorization header required"),
    };

    let payload: DeleteDatabaseRequest = match req.json().await {
        Ok(p) => p,
        Err(_) => return api_response(400, None, "", "Invalid request body"),
    };

    github_request(
        &token,
        Method::Delete,
        &format!("https://api.github.com/gists/{}", payload.gist_id),
        None,
    )
    .await?;

    api_response(
        200,
        Some(json!({ "deleted_gist": payload.gist_id })),
        "Database deleted",
        "",
    )
}
