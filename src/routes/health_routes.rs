use serde_json::json;
use worker::{Request, Response, Result, RouteContext};

pub async fn health_check(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
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
}
