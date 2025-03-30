use crate::types::base::{Integer, LSPAny};
use serde::{Deserialize, Serialize};

/// Base message type as defined by JSON-RPC.
/// The Language Server Protocol always uses "2.0" as the jsonrpc version.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct Message {
    pub jsonrpc: String,
}

/// A unique identifier for a request, can be int or string
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Integer(Integer),
    String(String),
}

/// A request message to describe a request between the client and the server.
/// Every processed request must send a response back to the sender of the request.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct RequestMessage {
    /// The JSON-RPC protocol version, must be set to "2.0".
    pub jsonrpc: String,

    /// The request id.
    pub id: RequestId,

    /// The method to be invoked.
    pub method: String,

    /// The method's params.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<LSPAny>,
}

/// The error object in case a request fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    /// A number indicating the error type that occurred.
    pub code: Integer,

    /// A string providing a short description of the error.
    pub message: String,

    /// A Primitive or Structured value that contains additional information about the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<LSPAny>,
}

/// A Response Message sent as a result of a request.
/// If a request doesn't provide a result value the receiver still needs to
/// return a response message to conform to the JSON-RPC specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    /// The JSON-RPC protocol version, always "2.0".
    pub jsonrpc: String,

    /// The request id.
    pub id: Option<RequestId>,

    /// The result of a request. This member is REQUIRED on success.
    /// This member MUST NOT exist if there was an error invoking the method.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<LSPAny>,

    /// The error object in case a request fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

/// A notification message. A processed notification message must not send a
/// response back. They work like events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    /// The JSON-RPC protocol version, always "2.0".
    pub jsonrpc: String,

    /// The method to be invoked.
    pub method: String,

    /// The notification's params.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<LSPAny>,
}
