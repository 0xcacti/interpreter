use crate::types::{message::ProgressToken, progress::WorkDoneProgressParams};
use serde::{Deserialize, Serialize};

/// Payload for the very first request sent from the client to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// From `WorkDoneProgressParams`: optional token to report progress.
    #[serde(flatten)]
    pub progress: WorkDoneProgressParams,

    /// The process Id of the parent process that started the server. Is null if
    /// the process has not been started by another process. If the parent
    /// process is not alive then the server should exit (see exit notification)
    /// its process.
    #[serde(rename = "processId")]
    pub process_id: Option<u32>,

    /// Information about the client
    #[serde(
        rename = "clientInfo",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub client_info: Option<ClientInfo>,
}

/// Subobject for clientInfo in InitializeParams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// The name of the client as defined by the client
    pub name: String,

    /// The client's version as defined by the client
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
