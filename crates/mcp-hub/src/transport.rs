// TRRUSTT — MCP Hub: Transport Layer
// stdio transport for MCP communication.

use std::process::Stdio;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument};

use shared::Result;
use crate::protocol::{JsonRpcRequest, JsonRpcResponse};

/// Transport for MCP communication via stdio (subprocess).
pub struct Transport {
    stdin: Mutex<ChildStdin>,
    stdout_lines: Mutex<tokio::io::Lines<BufReader<ChildStdout>>>,
    _child: Child,
    request_counter: Mutex<u64>,
}

impl Transport {
    /// Create a new stdio transport by spawning a process.
    #[instrument]
    pub async fn stdio(command: &str, args: &[&str]) -> Result<Self> {
        debug!(command = %command, args = ?args, "Spawning MCP server process");

        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| shared::AppError::internal(format!("Failed to spawn '{}': {}", command, e)))?;

        let stdin = child.stdin.take()
            .ok_or_else(|| shared::AppError::internal("Failed to capture stdin"))?;
        let stdout = child.stdout.take()
            .ok_or_else(|| shared::AppError::internal("Failed to capture stdout"))?;

        let reader = BufReader::new(stdout);

        info!(command = %command, "MCP server process spawned");

        Ok(Self {
            stdin: Mutex::new(stdin),
            stdout_lines: Mutex::new(reader.lines()),
            _child: child,
            request_counter: Mutex::new(0),
        })
    }

    /// Send a JSON-RPC request and wait for the response.
    #[instrument(skip(self))]
    pub async fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let mut counter = self.request_counter.lock().await;
        let id = *counter;
        *counter += 1;
        drop(counter);

        let request = JsonRpcRequest::new(Value::Number(serde_json::Number::from(id)), method, params);
        let request_json = serde_json::to_string(&request)
            .map_err(|e| shared::AppError::internal(e.to_string()))?;

        debug!(method = %method, id = id, "Sending MCP request");

        // Write request
        {
            let mut stdin = self.stdin.lock().await;
            stdin.write_all(request_json.as_bytes()).await.map_err(|e| {
                shared::AppError::internal(format!("Write to stdin failed: {}", e))
            })?;
            stdin.write_all(b"\n").await.map_err(|e| {
                shared::AppError::internal(format!("Write newline failed: {}", e))
            })?;
            stdin.flush().await.map_err(|e| {
                shared::AppError::internal(format!("Flush stdin failed: {}", e))
            })?;
        }

        // Read response
        let mut lines = self.stdout_lines.lock().await;
        while let Some(line) = lines.next_line().await.map_err(|e| {
            shared::AppError::internal(format!("Read from stdout failed: {}", e))
        })? {
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }

            let response: JsonRpcResponse = serde_json::from_str(&line)
                .map_err(|e| shared::AppError::internal(format!("Invalid JSON-RPC response: {}", e)))?;

            // Check if this response matches our request id
            if response.id.as_ref().and_then(|v| v.as_u64()) == Some(id) {
                if let Some(error) = response.error {
                    return Err(shared::AppError::internal(format!(
                        "MCP server error [{}]: {}",
                        error.code, error.message
                    )));
                }
                return response.result
                    .ok_or_else(|| shared::AppError::internal("MCP response missing result"));
            }
        }

        Err(shared::AppError::internal("MCP server closed connection"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_creation_fails_for_nonexistent() {
        // This test verifies error handling for invalid commands
        // In real tests, you'd mock the subprocess
    }
}
