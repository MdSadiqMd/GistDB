use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use uuid::Uuid;
use worker::*;

#[derive(Deserialize)]
struct RequestPayload {
    github_token: String,
    gist_id: String,
    file_name: String,
    content: Option<Value>,
}

#[derive(Deserialize)]
struct DeleteEntryRequest {
    id: String,
    file_name: String,
}

#[derive(Serialize)]
struct ApiResponse {
    status: String,
    data: Option<Value>,
    message: String,
    error: String,
}

async fn fetch_gist(token: &str, gist_id: &str) -> Result<Value> {
    let url = format!("https://api.github.com/gists/{}", gist_id);
    let mut req = Request::new(&url, Method::Get)?;

    req.headers_mut()?
        .set("Authorization", &format!("token {}", token))?;
    req.headers_mut()?.set("User-Agent", "Cloudflare-Worker")?;
    req.headers_mut()?
        .set("Accept", "application/vnd.github.v3+json")?;

    let mut resp = Fetch::Request(req).send().await?;
    resp.json().await
}

async fn update_gist(token: &str, gist_id: &str, body: Value) -> Result<Value> {
    let url = format!("https://api.github.com/gists/{}", gist_id);

    let mut req = Request::new_with_init(
        &url,
        RequestInit::new()
            .with_method(Method::Patch)
            .with_body(Some(serde_json::to_string(&body)?.into())),
    )?;

    req.headers_mut()?
        .set("Authorization", &format!("token {}", token))?;
    req.headers_mut()?.set("User-Agent", "Cloudflare-Worker")?;
    req.headers_mut()?
        .set("Accept", "application/vnd.github.v3+json")?;
    req.headers_mut()?.set("Content-Type", "application/json")?;

    let mut resp = Fetch::Request(req).send().await?;
    resp.json().await
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/api/read/:gist_id", |req, ctx| async move {
            let auth_header = match req.headers().get("Authorization")? {
                Some(token) => token,
                None => return Response::error("Missing authorization token", 401),
            };
            let token = auth_header.replace("Bearer ", "");

            let gist_id = match ctx.param("gist_id") {
                Some(id) => id,
                None => return Response::error("Missing gist ID", 400),
            };

            match fetch_gist(&token, &gist_id).await {
                Ok(gist_data) => {
                    let files = match gist_data.get("files").and_then(Value::as_object) {
                        Some(files) => files,
                        None => return Response::error("Invalid gist data", 500),
                    };

                    let mut response_data = Map::new();
                    for (file_name, file_info) in files {
                        let content_str = file_info
                            .get("content")
                            .and_then(|c| c.as_str())
                            .unwrap_or("{}");
                        let content_json: Value = match serde_json::from_str(content_str) {
                            Ok(json) => json,
                            Err(_) => Value::String(content_str.to_string()),
                        };
                        response_data.insert(file_name.clone(), content_json);
                    }

                    Response::from_json(&ApiResponse {
                        status: "success".to_string(),
                        data: Some(Value::Object(response_data)),
                        message: "Gist retrieved successfully".to_string(),
                        error: "".to_string(),
                    })
                }
                Err(e) => Response::error(e.to_string(), 500),
            }
        })
        .post_async("/api/create", |mut req, _ctx| async move {
            let payload: RequestPayload = match req.json().await {
                Ok(p) => p,
                Err(_) => return Response::error("Invalid request payload", 400),
            };

            let github_token = &payload.github_token;
            let gist_id = &payload.gist_id;
            let _file_name = &payload.file_name;
            let request_content = match payload.content {
                Some(content) => content,
                None => return Response::error("Content is required", 400),
            };

            let existing_gist = match fetch_gist(github_token, gist_id).await {
                Ok(gist) => gist,
                Err(e) => return Response::error(e.to_string(), 500),
            };

            let existing_files = match existing_gist.get("files").and_then(Value::as_object) {
                Some(files) => files,
                None => return Response::error("Invalid gist data", 500),
            };

            let request_files = match request_content.as_object() {
                Some(files) => files,
                None => return Response::error("Content must be an object", 400),
            };

            let mut merged_files = Map::new();

            for (file_name, file_value) in request_files {
                let new_content = match file_value.get("content") {
                    Some(content) => content,
                    None => {
                        return Response::error(
                            format!("Missing 'content' for file '{}'", file_name),
                            400,
                        )
                    }
                };

                let existing_file_content = existing_files
                    .get(file_name)
                    .and_then(|f| f.get("content"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("{}");

                let mut existing_data: Value = match serde_json::from_str(existing_file_content) {
                    Ok(data) => data,
                    Err(_) => json!({}),
                };

                let new_id = Uuid::new_v4().to_string();

                if let Value::Object(ref mut obj) = existing_data {
                    obj.insert(new_id, new_content.clone());
                } else {
                    let mut new_obj = Map::new();
                    new_obj.insert(new_id, new_content.clone());
                    existing_data = Value::Object(new_obj);
                }

                let merged_content = match serde_json::to_string(&existing_data) {
                    Ok(s) => s,
                    Err(e) => return Response::error(e.to_string(), 500),
                };

                merged_files.insert(
                    file_name.clone(),
                    json!({
                        "content": merged_content
                    }),
                );
            }

            let body = json!({
                "files": merged_files
            });

            match update_gist(github_token, gist_id, body).await {
                Ok(data) => Response::from_json(&ApiResponse {
                    status: "success".to_string(),
                    data: Some(data),
                    message: "Gist updated successfully".to_string(),
                    error: "".to_string(),
                }),
                Err(e) => Response::error(e.to_string(), 500),
            }
        })
        .put_async("/api/update/:gist_id", |mut req, _ctx| async move {
            let payload: RequestPayload = match req.json().await {
                Ok(p) => p,
                Err(e) => return Response::error(format!("Invalid payload: {}", e), 400),
            };

            let existing_gist = match fetch_gist(&payload.github_token, &payload.gist_id).await {
                Ok(gist) => gist,
                Err(e) => return Response::error(e.to_string(), 500),
            };

            let mut merged_files = Map::new();
            if let Some(content) = payload.content {
                if let Some(files) = content.as_object() {
                    for (file_name, file_info) in files {
                        let existing_content = existing_gist
                            .get("files")
                            .and_then(|f| f.get(file_name))
                            .and_then(|f| f.get("content"))
                            .and_then(|c| c.as_str())
                            .unwrap_or("{}");

                        let mut existing_data: Value =
                            serde_json::from_str(existing_content).unwrap_or_else(|_| json!({}));

                        if let Some(update_content) = file_info.get("content") {
                            if let (Some(existing_obj), Some(update_obj)) =
                                (existing_data.as_object_mut(), update_content.as_object())
                            {
                                for (key, value) in update_obj {
                                    if !existing_obj.contains_key(key) {
                                        return Response::error(
                                            format!("ID '{}' does not exist", key),
                                            404,
                                        );
                                    }
                                    existing_obj.insert(key.clone(), value.clone());
                                }
                            }
                        }

                        let merged_content =
                            serde_json::to_string(&existing_data).map_err(|e| e.to_string())?;

                        merged_files
                            .insert(file_name.clone(), json!({ "content": merged_content }));
                    }
                }
            }

            let body = json!({ "files": merged_files });

            match update_gist(&payload.github_token, &payload.gist_id, body).await {
                Ok(data) => Response::from_json(&ApiResponse {
                    status: "success".to_string(),
                    data: Some(data),
                    message: "Gist updated successfully".to_string(),
                    error: "".to_string(),
                }),
                Err(e) => Response::error(e.to_string(), 500),
            }
        })
        .delete_async("/api/delete-file/:gist_id", |req, ctx| async move {
            let auth_header = match req.headers().get("Authorization")? {
                Some(token) => token,
                None => return Response::error("Missing authorization token", 401),
            };
            let token = auth_header.replace("Bearer ", "");

            let gist_id = match ctx.param("gist_id") {
                Some(id) => id,
                None => return Response::error("Missing gist ID", 400),
            };

            let _file_name = match req.url()?.query_pairs().find(|(key, _)| key == "file_name") {
                Some((_, file_name)) => file_name.to_string(),
                None => return Response::error("Missing file name", 400),
            };

            let file_name = match req.url()?.query_pairs().find(|(key, _)| key == "file_name") {
                Some((_, file_name)) => file_name.to_string(),
                None => return Response::error("Missing file name", 400),
            };

            match update_gist(
                &token,
                &gist_id,
                json!({ "files": { file_name.clone(): null } }),
            )
            .await
            {
                Ok(_) => Response::from_json(&ApiResponse {
                    status: "success".to_string(),
                    data: Some(json!({ file_name: true })),
                    message: "File deleted successfully".to_string(),
                    error: "".to_string(),
                }),
                Err(e) => Response::error(e.to_string(), 500),
            }
        })
        .delete_async("/api/delete-entry/:gist_id", |mut req, ctx| async move {
            let auth_header = match req.headers().get("Authorization")? {
                Some(token) => token,
                None => return Response::error("Missing authorization token", 401),
            };
            let token = auth_header.replace("Bearer ", "");

            let gist_id = match ctx.param("gist_id") {
                Some(id) => id,
                None => return Response::error("Missing gist ID", 400),
            };

            let payload: DeleteEntryRequest = match req.json().await {
                Ok(p) => p,
                Err(e) => return Response::error(format!("Invalid payload: {}", e), 400),
            };

            let existing_gist = match fetch_gist(&token, &gist_id).await {
                Ok(gist) => gist,
                Err(e) => return Response::error(e.to_string(), 500),
            };

            let existing_content = existing_gist
                .get("files")
                .and_then(|f| f.get(&payload.file_name))
                .and_then(|f| f.get("content"))
                .and_then(|c| c.as_str())
                .unwrap_or("{}");

            let mut existing_data: Value =
                serde_json::from_str(existing_content).unwrap_or_else(|_| json!({}));

            let deleted_entry = if let Some(obj) = existing_data.as_object_mut() {
                if !obj.contains_key(&payload.id) {
                    return Response::error(format!("ID '{}' does not exist", payload.id), 404);
                }
                obj.remove(&payload.id)
            } else {
                return Response::error("Invalid gist content format", 500);
            };

            let updated_content =
                serde_json::to_string(&existing_data).map_err(|e| e.to_string())?;

            let file_name = payload.file_name.clone();

            match update_gist(
                &token,
                &gist_id,
                json!({ "files": { file_name.clone(): { "content": updated_content } } }),
            )
            .await
            {
                Ok(_) => Response::from_json(&ApiResponse {
                    status: "success".to_string(),
                    data: Some(json!({ file_name: { payload.id: deleted_entry } })),
                    message: "Entry deleted successfully".to_string(),
                    error: "".to_string(),
                }),
                Err(e) => Response::error(e.to_string(), 500),
            }
        })
        .run(req, env)
        .await
}
