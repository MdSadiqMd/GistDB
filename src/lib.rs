use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use uuid::Uuid;
use worker::*;

#[derive(Debug, Deserialize)]
struct CreateDatabaseRequest {
    name: String,
}

#[derive(Debug, Deserialize)]
struct CreateCollectionRequest {
    gist_id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct CreateObjectRequest {
    gist_id: String,
    collection_name: String,
    data: Value,
}

#[derive(Debug, Deserialize)]
struct GetDatabaseRequest {
    _gist_id: String,
}

#[derive(Debug, Deserialize)]
struct GetCollectionRequest {
    gist_id: String,
    collection_name: String,
}

#[derive(Debug, Deserialize)]
struct UpdateObjectRequest {
    gist_id: String,
    collection_name: String,
    object_id: String,
    data: Value,
}

#[derive(Debug, Deserialize)]
struct DeleteObjectRequest {
    gist_id: String,
    collection_name: String,
    object_id: String,
}

#[derive(Debug, Deserialize)]
struct DeleteCollectionRequest {
    gist_id: String,
    collection_name: String,
}

#[derive(Debug, Deserialize)]
struct DeleteDatabaseRequest {
    gist_id: String,
}

#[derive(Debug, Serialize)]
struct ApiResponse {
    status: u16,
    data: Option<Value>,
    message: String,
    error: String,
}

fn api_response(status: u16, data: Option<Value>, message: &str, error: &str) -> Result<Response> {
    Response::from_json(&ApiResponse {
        status,
        data,
        message: message.to_string(),
        error: error.to_string(),
    })
}

async fn github_request(
    token: &str,
    method: Method,
    url: &str,
    body: Option<Value>,
) -> Result<Value> {
    let mut init = RequestInit::new();
    let mut init = init.with_method(method);
    if let Some(body) = body {
        let body_str = serde_json::to_string(&body)?;
        init = init.with_body(Some(body_str.into()));
    }

    let mut req = Request::new_with_init(url, &init)?;
    req.headers_mut()?
        .set("Authorization", &format!("Bearer {}", token))?;
    req.headers_mut()?.set("User-Agent", "GistDB-API")?;
    req.headers_mut()?
        .set("Accept", "application/vnd.github.v3+json")?;
    req.headers_mut()?.set("Content-Type", "application/json")?;

    Fetch::Request(req).send().await?.json().await
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        // Root route
        .get_async("/", |_, _| async move {
            Response::from_json(&json!({
                "name": "GistDB",
                "version": "1.0.0",
                "description": "A document database powered by GitHub Gists",
                "author": {
                    "name": "MdSadiqMd",
                    "github": "https://github.com/MdSadiqMd/GistDB",
                    "X":"https://x.com/Md_Sadiq_Md"
                },
                "features": [
                    "Uses GitHub Gists as storage backend",
                    "Multiple collections per database",
                    "JSON document storage",
                    "Full CRUD operations",
                    "GitHub token authentication"
                ],
                "endpoints": {
                    "root": {
                        "GET /": "Get API information and documentation"
                    },
                    "health": {
                        "GET /health": "Check API health status"
                    },
                    "databases": {
                        "POST /api/databases": "Create a new database",
                        "GET /api/:gistId": "Get entire database contents",
                        "DELETE /api/databases": "Delete a database"
                    },
                    "collections": {
                        "POST /api/collections": "Create a new collection",
                        "POST /api/collections/get": "Get collection contents",
                        "DELETE /api/collections": "Delete a collection"
                    },
                    "objects": {
                        "POST /api/objects": "Create a new object",
                        "PUT /api/objects": "Update an existing object",
                        "DELETE /api/objects": "Delete an object"
                    }
                },
                "documentation": "https://github.com/MdSadiqMd/GistDB"
            }))
        })
        // Health check
        .get_async("/health", |_, ctx| async move {
            let environment = ctx
                .var("ENVIRONMENT")
                .map(|env| env.to_string())
                .unwrap_or_else(|_| "production".to_string());

            Response::from_json(&json!({
                "status": {
                    "overall": "healthy",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                },
                "version": {
                    "api": "1.0.0",
                    "environment": environment,
                },
                "dependencies": {
                    "github_api": {
                        "status": "configured",
                        "endpoint": "https://api.github.com"
                    }
                },
                "worker_info": {
                    "datacenter": ctx.var("CF_WORKER_DATACENTER")
                        .map(|d| d.to_string())
                        .unwrap_or_else(|_| "unknown".to_string()),
                    "runtime": "workers",
                }
            }))
        })
        // Initialize new database (gist)
        .post_async("/api/databases", |mut req, _ctx| async move {
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
        })
        // Create new collection
        .post_async("/api/collections", |mut req, _ctx| async move {
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
        })
        // Create new object
        .post_async("/api/objects", |mut req, _ctx| async move {
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
        })
        // Get entire database or specific collection
        .get_async("/api/:gistId", |req, ctx| async move {
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
        })
        // Get collection
        .post_async("/api/collections/get", |mut req, _ctx| async move {
            let token = match req.headers().get("Authorization")? {
                Some(h) => h.replace("Bearer ", ""),
                None => return api_response(401, None, "", "Authorization header required"),
            };

            let payload: GetCollectionRequest = match req.json().await {
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

            match gist["files"].get(&filename) {
                Some(file) => {
                    let content = file["content"].as_str().unwrap_or("{}");
                    let data: Value = serde_json::from_str(content)?;
                    api_response(200, Some(data), "Collection contents", "")
                }
                None => api_response(404, None, "", "Collection not found"),
            }
        })
        // Update object
        .put_async("/api/objects", |mut req, _ctx| async move {
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
        })
        // Delete object
        .delete_async("/api/objects", |mut req, _ctx| async move {
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
        })
        // Delete collection
        .delete_async("/api/collections", |mut req, _ctx| async move {
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
        })
        // Delete database
        .delete_async("/api/databases", |mut req, _ctx| async move {
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
        })
        .run(req, env)
        .await
}
