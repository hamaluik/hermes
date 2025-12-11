//! JSON-RPC 2.0 protocol implementation with LSP-style message framing.
//!
//! Messages are framed with HTTP-style headers:
//! ```text
//! Content-Length: <byte_length>\r\n
//! \r\n
//! <JSON-RPC message>
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Standard JSON-RPC and Hermes-specific error codes.
#[allow(dead_code)]
pub mod error_codes {
    // standard JSON-RPC errors
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    // hermes-specific errors (-32000 to -32049)
    pub const GENERAL_ERROR: i32 = -32000;
    pub const NOT_INITIALIZED: i32 = -32001;
    pub const ALREADY_INITIALIZED: i32 = -32002;
    pub const NO_MESSAGE_OPEN: i32 = -32003;
    pub const INVALID_MESSAGE: i32 = -32004;
    pub const INVALID_PATH: i32 = -32005;
    pub const PATH_NOT_FOUND: i32 = -32006;
    pub const INVALID_URL: i32 = -32007;
    pub const WINDOW_ERROR: i32 = -32008;
    pub const COMMAND_NOT_FOUND: i32 = -32009;
    pub const COMMAND_TIMEOUT: i32 = -32010;
    pub const VALIDATION_ERROR: i32 = -32011;
    pub const DIALOG_ERROR: i32 = -32012;
}

/// Errors that can occur during protocol operations.
#[derive(Debug)]
pub enum ProtocolError {
    /// End of stream reached unexpectedly.
    Eof,
    /// Invalid Content-Length header.
    InvalidHeader(String),
    /// I/O error during read/write.
    Io(std::io::Error),
    /// JSON serialization/deserialization error.
    Json(serde_json::Error),
    /// Message exceeds maximum allowed size.
    MessageTooLarge(usize),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolError::Eof => write!(f, "unexpected end of stream"),
            ProtocolError::InvalidHeader(msg) => write!(f, "invalid header: {msg}"),
            ProtocolError::Io(e) => write!(f, "I/O error: {e}"),
            ProtocolError::Json(e) => write!(f, "JSON error: {e}"),
            ProtocolError::MessageTooLarge(size) => {
                write!(f, "message too large: {size} bytes")
            }
        }
    }
}

impl std::error::Error for ProtocolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProtocolError::Io(e) => Some(e),
            ProtocolError::Json(e) => Some(e),
            ProtocolError::Eof
            | ProtocolError::InvalidHeader(_)
            | ProtocolError::MessageTooLarge(_) => None,
        }
    }
}

impl From<std::io::Error> for ProtocolError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            ProtocolError::Eof
        } else {
            ProtocolError::Io(e)
        }
    }
}

impl From<serde_json::Error> for ProtocolError {
    fn from(e: serde_json::Error) -> Self {
        ProtocolError::Json(e)
    }
}

/// JSON-RPC 2.0 request identifier.
///
/// Can be either a number or a string per the JSON-RPC specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestId::Number(n) => write!(f, "{n}"),
            RequestId::String(s) => write!(f, "{s}"),
        }
    }
}

/// JSON-RPC 2.0 request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl Request {
    /// Create a new request with the given method and parameters.
    pub fn new(
        id: RequestId,
        method: impl Into<String>,
        params: Option<serde_json::Value>,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC 2.0 successful response message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: RequestId,
    pub result: serde_json::Value,
}

impl Response {
    /// Create a new successful response.
    pub fn new(id: RequestId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result,
        }
    }
}

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl RpcError {
    /// Create a new error with the given code and message.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create a new error with additional data.
    pub fn with_data(code: i32, message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Create a parse error (-32700).
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(error_codes::PARSE_ERROR, message)
    }

    /// Create an invalid request error (-32600).
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(error_codes::INVALID_REQUEST, message)
    }

    /// Create a method not found error (-32601).
    pub fn method_not_found(method: &str) -> Self {
        Self::new(
            error_codes::METHOD_NOT_FOUND,
            format!("method not found: {method}"),
        )
    }

    /// Create an invalid params error (-32602).
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(error_codes::INVALID_PARAMS, message)
    }

    /// Create an internal error (-32603).
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(error_codes::INTERNAL_ERROR, message)
    }

    /// Create a not initialized error (-32001).
    pub fn not_initialized() -> Self {
        Self::new(error_codes::NOT_INITIALIZED, "extension not initialized")
    }

    /// Create a command not found error (-32009).
    pub fn command_not_found(command: &str) -> Self {
        Self::new(
            error_codes::COMMAND_NOT_FOUND,
            format!("command not found: {command}"),
        )
    }

    /// Create a command timeout error (-32010).
    pub fn command_timeout(command: &str) -> Self {
        Self::new(
            error_codes::COMMAND_TIMEOUT,
            format!("command timed out: {command}"),
        )
    }

    /// Create an invalid message error (-32004).
    pub fn invalid_message(message: impl Into<String>) -> Self {
        Self::new(error_codes::INVALID_MESSAGE, message)
    }

    /// Create an invalid URL error (-32007).
    pub fn invalid_url(message: impl Into<String>) -> Self {
        Self::new(error_codes::INVALID_URL, message)
    }

    /// Create a window error (-32008).
    pub fn window_error(message: impl Into<String>) -> Self {
        Self::new(error_codes::WINDOW_ERROR, message)
    }

    /// Create a dialog error (-32012).
    pub fn dialog_error(message: impl Into<String>) -> Self {
        Self::new(error_codes::DIALOG_ERROR, message)
    }
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for RpcError {}

/// JSON-RPC 2.0 error response message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
    pub error: RpcError,
}

impl ErrorResponse {
    /// Create a new error response.
    pub fn new(id: Option<RequestId>, error: RpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            error,
        }
    }
}

/// JSON-RPC 2.0 notification message (no id, no response expected).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl Notification {
    /// Create a new notification.
    pub fn new(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }
}

/// Any JSON-RPC 2.0 message type.
///
/// Used for parsing incoming messages when the type isn't known in advance.
#[derive(Debug, Clone)]
pub enum Message {
    Request(Request),
    Response(Response),
    Error(ErrorResponse),
    Notification(Notification),
}

impl Serialize for Message {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Message::Request(r) => r.serialize(serializer),
            Message::Response(r) => r.serialize(serializer),
            Message::Error(e) => e.serialize(serializer),
            Message::Notification(n) => n.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // deserialize as raw JSON first
        let value = serde_json::Value::deserialize(deserializer)?;

        // check for required "jsonrpc" field
        if value.get("jsonrpc").and_then(|v| v.as_str()) != Some("2.0") {
            return Err(serde::de::Error::custom("missing or invalid jsonrpc field"));
        }

        // determine message type based on fields present
        let has_id = value.get("id").is_some();
        let has_method = value.get("method").is_some();
        let has_result = value.get("result").is_some();
        let has_error = value.get("error").is_some();

        if has_error {
            // error response (may or may not have id)
            let error: ErrorResponse =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            Ok(Message::Error(error))
        } else if has_result && has_id {
            // successful response
            let response: Response =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            Ok(Message::Response(response))
        } else if has_method && has_id {
            // request (has both method and id)
            let request: Request =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            Ok(Message::Request(request))
        } else if has_method && !has_id {
            // notification (has method but no id)
            let notification: Notification =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            Ok(Message::Notification(notification))
        } else {
            Err(serde::de::Error::custom(
                "invalid JSON-RPC message: cannot determine type",
            ))
        }
    }
}

// cap at 10 MB to prevent memory exhaustion from malicious/malformed messages
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

/// Read a single JSON-RPC message from a reader with Content-Length framing.
///
/// The message format is:
/// ```text
/// Content-Length: <byte_length>\r\n
/// \r\n
/// <JSON-RPC message>
/// ```
pub async fn read_message<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> Result<Message, ProtocolError> {
    // read headers until we find Content-Length
    let mut content_length: Option<usize> = None;
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            return Err(ProtocolError::Eof);
        }

        let trimmed = line.trim();

        // empty line signals end of headers
        if trimmed.is_empty() {
            break;
        }

        // parse Content-Length header
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            let value = value.trim();
            content_length = Some(value.parse().map_err(|_| {
                ProtocolError::InvalidHeader(format!("invalid Content-Length: {value}"))
            })?);
        }
        // ignore other headers (e.g., Content-Type)
    }

    let content_length = content_length
        .ok_or_else(|| ProtocolError::InvalidHeader("missing Content-Length header".to_string()))?;

    if content_length > MAX_MESSAGE_SIZE {
        return Err(ProtocolError::MessageTooLarge(content_length));
    }

    // read exact number of bytes
    let mut buffer = vec![0u8; content_length];
    reader.read_exact(&mut buffer).await?;

    // parse JSON
    let message: Message = serde_json::from_slice(&buffer)?;
    Ok(message)
}

/// Write a JSON-RPC message to a writer with Content-Length framing.
pub async fn write_message<W: AsyncWrite + Unpin>(
    writer: &mut W,
    msg: &Message,
) -> Result<(), ProtocolError> {
    let json = serde_json::to_string(msg)?;
    let header = format!("Content-Length: {}\r\n\r\n", json.len());

    writer.write_all(header.as_bytes()).await?;
    writer.write_all(json.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use tokio::io::BufReader;

    #[test]
    fn test_request_id_number_serialization() {
        let id = RequestId::Number(42);
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "42");
    }

    #[test]
    fn test_request_id_string_serialization() {
        let id = RequestId::String("abc-123".to_string());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"abc-123\"");
    }

    #[test]
    fn test_request_serialization() {
        let request = Request::new(
            RequestId::Number(1),
            "initialize",
            Some(serde_json::json!({"version": "1.0"})),
        );
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"method\":\"initialize\""));
    }

    #[test]
    fn test_response_serialization() {
        let response = Response::new(RequestId::Number(1), serde_json::json!({"status": "ok"}));
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"result\""));
    }

    #[test]
    fn test_error_response_serialization() {
        let error = ErrorResponse::new(
            Some(RequestId::Number(1)),
            RpcError::method_not_found("unknown"),
        );
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"error\""));
        assert!(json.contains(&error_codes::METHOD_NOT_FOUND.to_string()));
    }

    #[test]
    fn test_notification_serialization() {
        let notification = Notification::new(
            "window/closed",
            Some(serde_json::json!({"windowId": "123"})),
        );
        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"window/closed\""));
        assert!(!json.contains("\"id\"")); // notifications have no id
    }

    #[test]
    fn test_message_deserialize_request() {
        let json = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, Message::Request(_)));
    }

    #[test]
    fn test_message_deserialize_response() {
        let json = r#"{"jsonrpc":"2.0","id":1,"result":{"status":"ok"}}"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, Message::Response(_)));
    }

    #[test]
    fn test_message_deserialize_error() {
        let json =
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"method not found"}}"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, Message::Error(_)));
    }

    #[test]
    fn test_message_deserialize_notification() {
        let json = r#"{"jsonrpc":"2.0","method":"window/closed","params":{"windowId":"123"}}"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert!(matches!(msg, Message::Notification(_)));
    }

    #[test]
    fn test_message_deserialize_invalid_jsonrpc() {
        let json = r#"{"jsonrpc":"1.0","id":1,"method":"test"}"#;
        let result: Result<Message, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_read_message() {
        let content = r#"{"jsonrpc":"2.0","id":1,"method":"test"}"#;
        let framed = format!("Content-Length: {}\r\n\r\n{}", content.len(), content);
        let cursor = Cursor::new(framed.as_bytes().to_vec());
        let mut reader = BufReader::new(cursor);

        let msg = read_message(&mut reader).await.unwrap();
        assert!(matches!(msg, Message::Request(_)));
    }

    #[tokio::test]
    async fn test_read_message_with_extra_headers() {
        let content = r#"{"jsonrpc":"2.0","id":1,"method":"test"}"#;
        let framed = format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            content.len(),
            content
        );
        let cursor = Cursor::new(framed.as_bytes().to_vec());
        let mut reader = BufReader::new(cursor);

        let msg = read_message(&mut reader).await.unwrap();
        assert!(matches!(msg, Message::Request(_)));
    }

    #[tokio::test]
    async fn test_read_message_missing_content_length() {
        let framed = "Content-Type: application/json\r\n\r\n{}";
        let cursor = Cursor::new(framed.as_bytes().to_vec());
        let mut reader = BufReader::new(cursor);

        let result = read_message(&mut reader).await;
        assert!(matches!(result, Err(ProtocolError::InvalidHeader(_))));
    }

    #[tokio::test]
    async fn test_write_message() {
        let request = Request::new(RequestId::Number(1), "test", None);
        let msg = Message::Request(request);

        let mut buffer = Vec::new();
        write_message(&mut buffer, &msg).await.unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.starts_with("Content-Length: "));
        assert!(output.contains("\r\n\r\n"));
        assert!(output.contains("\"jsonrpc\":\"2.0\""));
    }

    #[tokio::test]
    async fn test_roundtrip() {
        let original = Request::new(
            RequestId::String("test-123".to_string()),
            "editor/getMessage",
            Some(serde_json::json!({"format": "hl7"})),
        );
        let msg = Message::Request(original.clone());

        // write to buffer
        let mut buffer = Vec::new();
        write_message(&mut buffer, &msg).await.unwrap();

        // read back
        let cursor = Cursor::new(buffer);
        let mut reader = BufReader::new(cursor);
        let read_msg = read_message(&mut reader).await.unwrap();

        if let Message::Request(req) = read_msg {
            assert_eq!(req.id, original.id);
            assert_eq!(req.method, original.method);
            assert_eq!(req.params, original.params);
        } else {
            panic!("expected Request message");
        }
    }

    #[test]
    fn test_rpc_error_display() {
        let error = RpcError::method_not_found("test");
        let display = format!("{error}");
        assert!(display.contains("-32601"));
        assert!(display.contains("method not found"));
    }

    #[test]
    fn test_protocol_error_display() {
        let error = ProtocolError::Eof;
        assert_eq!(format!("{error}"), "unexpected end of stream");

        let error = ProtocolError::InvalidHeader("bad header".to_string());
        assert!(format!("{error}").contains("bad header"));

        let error = ProtocolError::MessageTooLarge(100);
        assert!(format!("{error}").contains("100"));
    }
}
