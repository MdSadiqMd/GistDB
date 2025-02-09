use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use worker::*;

#[derive(Deserialize)]
struct RequestPayload {
    github_token: String,
    gist_id: String,
    content: Option<Value>,
}

#[derive(Serialize)]
struct ApiResponse {
    status: String,
    data: Option<Value>,
    message: String,
}

async fn fetch_gist(token: &str, gist_id: &str) -> Result<Value> {
    let url = format!("https://api.github.com/gists/{}", gist_id);
    let mut req = Request::new(&url, Method::Get)?;

    let headers_mut = req.headers_mut()?;
    headers_mut.set("Authorization", &format!("token {}", token))?;
    headers_mut.set("User-Agent", "Cloudflare-Worker")?;
    headers_mut.set("Accept", "application/vnd.github.v3+json")?;

    let mut resp = Fetch::Request(req).send().await?;
    let json: Value = resp.json().await?;
    Ok(json)
}

async fn update_gist(token: &str, gist_id: &str, content: Value) -> Result<Value> {
    let url = format!("https://api.github.com/gists/{}", gist_id);
    let body = json!({
        "files": {
            "database.json": {
                "content": content.to_string()
            }
        }
    });

    let mut req = Request::new_with_init(
        &url,
        RequestInit::new()
            .with_method(Method::Patch)
            .with_body(Some(serde_json::to_string(&body)?.into())),
    )?;

    let headers_mut = req.headers_mut()?;
    headers_mut.set("Authorization", &format!("token {}", token))?;
    headers_mut.set("User-Agent", "Cloudflare-Worker")?;
    headers_mut.set("Accept", "application/vnd.github.v3+json")?;
    headers_mut.set("Content-Type", "application/json")?;

    let mut resp = Fetch::Request(req).send().await?;
    let json: Value = resp.json().await?;
    Ok(json)
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
                Ok(data) => Response::from_json(&ApiResponse {
                    status: "success".to_string(),
                    data: Some(data),
                    message: "Gist retrieved successfully".to_string(),
                }),
                Err(e) => Response::error(e.to_string(), 500),
            }
        })
        .post_async("/api/create", |mut req, _ctx| async move {
            let payload: RequestPayload = match req.json().await {
                Ok(p) => p,
                Err(_) => return Response::error("Invalid request payload", 400),
            };

            if payload.content.is_none() {
                return Response::error("Content is required", 400);
            }

            match update_gist(
                &payload.github_token,
                &payload.gist_id,
                payload.content.unwrap(),
            )
            .await
            {
                Ok(data) => Response::from_json(&ApiResponse {
                    status: "success".to_string(),
                    data: Some(data),
                    message: "Gist created successfully".to_string(),
                }),
                Err(e) => Response::error(e.to_string(), 500),
            }
        })
        .put_async("/api/update/:gist_id", |mut req, _ctx| async move {
            let payload: RequestPayload = match req.json().await {
                Ok(p) => p,
                Err(_) => return Response::error("Invalid request payload", 400),
            };

            if payload.content.is_none() {
                return Response::error("Content is required", 400);
            }

            match update_gist(
                &payload.github_token,
                &payload.gist_id,
                payload.content.unwrap(),
            )
            .await
            {
                Ok(data) => Response::from_json(&ApiResponse {
                    status: "success".to_string(),
                    data: Some(data),
                    message: "Gist updated successfully".to_string(),
                }),
                Err(e) => Response::error(e.to_string(), 500),
            }
        })
        .delete_async("/api/delete/:gist_id", |req, ctx| async move {
            let auth_header = match req.headers().get("Authorization")? {
                Some(token) => token,
                None => return Response::error("Missing authorization token", 401),
            };
            let token = auth_header.replace("Bearer ", "");

            let gist_id = match ctx.param("gist_id") {
                Some(id) => id,
                None => return Response::error("Missing gist ID", 400),
            };

            match update_gist(&token, &gist_id, json!({})).await {
                Ok(_) => Response::from_json(&ApiResponse {
                    status: "success".to_string(),
                    data: None,
                    message: "Gist deleted successfully".to_string(),
                }),
                Err(e) => Response::error(e.to_string(), 500),
            }
        })
        .run(req, env)
        .await
}
