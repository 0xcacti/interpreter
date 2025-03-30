/// Error code indicating an issue occurred while parsing a JSON-RPC message.
/// This is defined by the JSON-RPC specification.
pub const PARSE_ERROR: i32 = -32700;

/// Error code indicating the JSON-RPC request was malformed.
pub const INVALID_REQUEST: i32 = -32600;

/// Error code indicating the method specified in the request doesn't exist.
pub const METHOD_NOT_FOUND: i32 = -32601;

/// Error code indicating the method parameters are invalid.
pub const INVALID_PARAMS: i32 = -32602;

/// Error code indicating an internal error occurred in the server.
pub const INTERNAL_ERROR: i32 = -32603;

/// This is the start range of JSON-RPC reserved error codes.
/// It doesn't denote a real error code.
pub const JSON_RPC_RESERVED_ERROR_RANGE_START: i32 = -32099;

/// @deprecated Use JSON_RPC_RESERVED_ERROR_RANGE_START instead.
#[deprecated(note = "use JSON_RPC_RESERVED_ERROR_RANGE_START")]
pub const SERVER_ERROR_START: i32 = JSON_RPC_RESERVED_ERROR_RANGE_START;

/// Error code indicating that a server received a notification or
/// request before the server has received the `initialize` request.
pub const SERVER_NOT_INITIALIZED: i32 = -32002;

/// Generic error code for unexpected server errors.
pub const UNKNOWN_ERROR_CODE: i32 = -32001;

/// This is the end range of JSON-RPC reserved error codes.
/// It doesn't denote a real error code.
pub const JSON_RPC_RESERVED_ERROR_RANGE_END: i32 = -32000;

/// @deprecated Use JSON_RPC_RESERVED_ERROR_RANGE_END instead.
#[deprecated(note = "use JSON_RPC_RESERVED_ERROR_RANGE_END")]
pub const SERVER_ERROR_END: i32 = JSON_RPC_RESERVED_ERROR_RANGE_END;

/// This is the start range of LSP reserved error codes.
/// It doesn't denote a real error code.
pub const LSP_RESERVED_ERROR_RANGE_START: i32 = -32899;

/// A request failed but it was syntactically correct, e.g the
/// method name was known and the parameters were valid. The error
/// message should contain human readable information about why
/// the request failed.
pub const REQUEST_FAILED: i32 = -32803;

/// The server cancelled the request. This error code should
/// only be used for requests that explicitly support being
/// server cancellable.
pub const SERVER_CANCELLED: i32 = -32802;

/// The server detected that the content of a document got
/// modified outside normal conditions. A server should
/// NOT send this error code if it detects a content change
/// in it unprocessed messages.
pub const CONTENT_MODIFIED: i32 = -32801;

/// The client has canceled a request and a server has detected
/// the cancel.
pub const REQUEST_CANCELLED: i32 = -32800;

/// This is the end range of LSP reserved error codes.
/// It doesn't denote a real error code.
pub const LSP_RESERVED_ERROR_RANGE_END: i32 = -32800;
