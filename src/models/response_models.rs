use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub status: u16,
    pub data: Option<Value>,
    pub message: String,
    pub error: String,
}
