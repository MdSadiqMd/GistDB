use serde_json::Value;
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
