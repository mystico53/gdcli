use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 request (MCP uses this over stdio).
#[derive(Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

/// JSON-RPC 2.0 success response.
#[derive(Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    pub result: Value,
}

/// JSON-RPC 2.0 error response.
#[derive(Serialize)]
pub struct JsonRpcError {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    pub error: JsonRpcErrorBody,
}

#[derive(Serialize)]
pub struct JsonRpcErrorBody {
    pub code: i64,
    pub message: String,
}

// Standard JSON-RPC error codes
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const PARSE_ERROR: i64 = -32700;

impl JsonRpcResponse {
    pub fn new(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result,
        }
    }
}

impl JsonRpcError {
    pub fn new(id: Option<Value>, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            error: JsonRpcErrorBody {
                code,
                message: message.into(),
            },
        }
    }
}
