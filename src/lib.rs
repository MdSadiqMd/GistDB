use crate::routes::{collection_routes, database_routes, health_routes, object_routes};
use serde_json::json;
use worker::{event, Env, Request, Response, Result, Router};
mod models;
mod routes;
mod services;
mod utils;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    let router = Router::new();

    router
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
        .get_async("/health", health_routes::health_check)
        .post_async("/api/databases", database_routes::create_database)
        .delete_async("/api/databases", database_routes::delete_database)
        .post_async("/api/collections", collection_routes::create_collection)
        .get_async("/api/:gistId", collection_routes::get_collection)
        .delete_async("/api/collections", collection_routes::delete_collection)
        .post_async("/api/objects", object_routes::create_object)
        .put_async("/api/objects", object_routes::update_object)
        .delete_async("/api/objects", object_routes::delete_object)
        .run(req, env)
        .await
}
