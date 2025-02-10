use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct CreateDatabaseRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCollectionRequest {
    pub gist_id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateObjectRequest {
    pub gist_id: String,
    pub collection_name: String,
    pub data: Value,
}

#[derive(Debug, Deserialize)]
pub struct GetDatabaseRequest {
    pub _gist_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateObjectRequest {
    pub gist_id: String,
    pub collection_name: String,
    pub object_id: String,
    pub data: Value,
}

#[derive(Debug, Deserialize)]
pub struct DeleteObjectRequest {
    pub gist_id: String,
    pub collection_name: String,
    pub object_id: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteCollectionRequest {
    pub gist_id: String,
    pub collection_name: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteDatabaseRequest {
    pub gist_id: String,
}
