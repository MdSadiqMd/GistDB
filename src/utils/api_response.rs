use crate::models::response_models::ApiResponse;
use serde_json::Value;
use std::result::Result as StdResult;
use worker::Response;

pub fn api_response(
    status: u16,
    data: Option<Value>,
    message: &str,
    error: &str,
) -> StdResult<Response, worker::Error> {
    Response::from_json(&ApiResponse {
        status,
        data,
        message: message.to_string(),
        error: error.to_string(),
    })
}
