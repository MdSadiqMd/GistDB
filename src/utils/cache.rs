use serde_json::Value;
use worker::{Cache, Response};

// const SEARCH_CACHE_TTL: u32 = 60;

pub async fn cached_search(
    cache: &Cache,
    key: &str,
    search_op: impl std::future::Future<Output = Result<Value, worker::Error>>,
) -> Result<Value, worker::Error> {
    if let Some(mut cached) = cache.get(key, true).await? {
        return Ok(cached.json().await?);
    }

    let result = search_op.await?;
    cache.put(key, Response::from_json(&result)?).await?;
    Ok(result)
}
