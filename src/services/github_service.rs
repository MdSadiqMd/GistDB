use serde_json::{json, Value};
use worker::{Fetch, Method, Request, RequestInit, Result};

pub async fn github_request(
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

pub async fn get_gist_file(token: &str, gist_id: &str, filename: &str) -> Result<String> {
    let url = format!("https://api.github.com/gists/{}", gist_id);
    let response = github_request(token, Method::Get, &url, None).await?;
    if let Some(files) = response["files"].as_object() {
        if let Some(file) = files.get(filename) {
            if let Some(content) = file["content"].as_str() {
                return Ok(content.to_string());
            }
        }
    }

    Err(worker::Error::RustError(format!(
        "File '{}' not found in Gist '{}'",
        filename, gist_id
    )))
}

#[allow(dead_code)]
pub async fn update_gist_file(
    token: &str,
    gist_id: &str,
    filename: &str,
    content: &str,
) -> Result<()> {
    let url = format!("https://api.github.com/gists/{}", gist_id);
    let body = json!({
        "files": {
            filename: {
                "content": content
            }
        }
    });
    github_request(token, Method::Patch, &url, Some(body)).await?;
    Ok(())
}

#[allow(dead_code)]
pub async fn get_gist_file_chunked(
    token: &str,
    gist_id: &str,
    filename: &str,
    page: usize,
    chunk_size: usize,
) -> Result<String> {
    let url = format!("https://api.github.com/gists/{}", gist_id);
    let response = github_request(token, Method::Get, &url, None).await?;
    if let Some(files) = response["files"].as_object() {
        if let Some(file) = files.get(filename) {
            if let Some(content) = file["content"].as_str() {
                let start = (page - 1) * chunk_size;
                let end = start + chunk_size;
                let chunk = content
                    .chars()
                    .skip(start)
                    .take(end - start)
                    .collect::<String>();
                return Ok(chunk);
            }
        }
    }

    Err(worker::Error::RustError(format!(
        "File '{}' not found in Gist '{}'",
        filename, gist_id
    )))
}
