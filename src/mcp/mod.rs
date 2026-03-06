mod dispatch;
mod protocol;
mod tools;

use protocol::{JsonRpcError, JsonRpcResponse, INVALID_PARAMS, METHOD_NOT_FOUND, PARSE_ERROR};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

/// Run the MCP server: read JSON-RPC from stdin, write responses to stdout.
pub fn run_mcp_server(project_dir: Option<&str>) -> anyhow::Result<()> {
    if let Some(dir) = project_dir {
        std::env::set_current_dir(dir)
            .map_err(|e| anyhow::anyhow!("Failed to set project directory '{}': {}", dir, e))?;
    }

    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    for line in stdin.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break, // EOF or read error
        };

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: protocol::JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(r) => r,
            Err(e) => {
                let err = JsonRpcError::new(None, PARSE_ERROR, format!("Parse error: {}", e));
                write_json(&mut stdout, &err)?;
                continue;
            }
        };

        // Notifications have no id — don't send a response
        if req.id.is_none() {
            // Silently consume notifications (e.g. notifications/initialized)
            continue;
        }

        let response_json = handle_request(&req);
        write_line(&mut stdout, &response_json)?;
    }

    Ok(())
}

fn handle_request(req: &protocol::JsonRpcRequest) -> String {
    match req.method.as_str() {
        "initialize" => {
            let resp = JsonRpcResponse::new(
                req.id.clone(),
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "gdcli",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }),
            );
            serde_json::to_string(&resp).unwrap_or_default()
        }
        "ping" => {
            let resp = JsonRpcResponse::new(req.id.clone(), json!({}));
            serde_json::to_string(&resp).unwrap_or_default()
        }
        "tools/list" => {
            let resp = JsonRpcResponse::new(req.id.clone(), tools::tools_list_json());
            serde_json::to_string(&resp).unwrap_or_default()
        }
        "tools/call" => handle_tools_call(req),
        _ => {
            let err = JsonRpcError::new(
                req.id.clone(),
                METHOD_NOT_FOUND,
                format!("Method not found: {}", req.method),
            );
            serde_json::to_string(&err).unwrap_or_default()
        }
    }
}

fn handle_tools_call(req: &protocol::JsonRpcRequest) -> String {
    let tool_name = req
        .params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if tool_name.is_empty() {
        let err = JsonRpcError::new(
            req.id.clone(),
            INVALID_PARAMS,
            "Missing 'name' in tools/call params",
        );
        return serde_json::to_string(&err).unwrap_or_default();
    }

    let arguments = req
        .params
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Object(serde_json::Map::new()));

    let result = dispatch::call_tool(tool_name, &arguments);

    let resp = JsonRpcResponse::new(
        req.id.clone(),
        json!({
            "content": [{
                "type": "text",
                "text": result.text
            }],
            "isError": result.is_error
        }),
    );
    serde_json::to_string(&resp).unwrap_or_default()
}

fn write_json<T: serde::Serialize>(out: &mut impl Write, val: &T) -> io::Result<()> {
    let s = serde_json::to_string(val).unwrap_or_default();
    writeln!(out, "{}", s)?;
    out.flush()
}

fn write_line(out: &mut impl Write, s: &str) -> io::Result<()> {
    writeln!(out, "{}", s)?;
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_response() {
        let req = protocol::JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "initialize".into(),
            params: json!({}),
        };
        let resp = handle_request(&req);
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["id"], 1);
        assert!(parsed["result"]["capabilities"]["tools"].is_object());
        assert_eq!(parsed["result"]["serverInfo"]["name"], "gdcli");
    }

    #[test]
    fn test_ping_response() {
        let req = protocol::JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(42)),
            method: "ping".into(),
            params: json!({}),
        };
        let resp = handle_request(&req);
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["id"], 42);
        assert!(parsed["result"].is_object());
    }

    #[test]
    fn test_tools_list_has_19_tools() {
        let req = protocol::JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(2)),
            method: "tools/list".into(),
            params: json!({}),
        };
        let resp = handle_request(&req);
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        let tools = parsed["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 19);
    }

    #[test]
    fn test_unknown_method() {
        let req = protocol::JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(3)),
            method: "nonexistent".into(),
            params: json!({}),
        };
        let resp = handle_request(&req);
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["error"]["code"], -32601);
    }

    #[test]
    fn test_tools_call_missing_name() {
        let req = protocol::JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(4)),
            method: "tools/call".into(),
            params: json!({}),
        };
        let resp = handle_request(&req);
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        assert_eq!(parsed["error"]["code"], -32602);
    }

    #[test]
    fn test_tools_call_unknown_tool() {
        let req = protocol::JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(5)),
            method: "tools/call".into(),
            params: json!({"name": "nonexistent_tool", "arguments": {}}),
        };
        let resp = handle_request(&req);
        let parsed: Value = serde_json::from_str(&resp).unwrap();
        let result = &parsed["result"];
        assert_eq!(result["isError"], true);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Unknown tool"));
    }
}
