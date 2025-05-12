use crate::types::{
    base::{Integer, LSPAny},
    capabilities::{ClientCapabilities, TraceValue},
    progress::WorkDoneProgressParams,
    uri::DocumentUri,
    workspace::WorkspaceFolders,
};
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
    pub process_id: Option<Integer>,

    /// Information about the client
    #[serde(rename = "clientInfo", skip_serializing_if = "Option::is_none")]
    pub client_info: Option<ClientInfo>,

    /// The locale the client is currently showing the user interface
    /// in. This must not necessarily be the locale of the operating
    /// system.
    ///
    /// Uses IETF language tags as the value's syntax
    /// (See https://en.wikipedia.org/wiki/IETF_language_tag)
    ///
    /// @since 3.16.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    /// @deprecated in favour of `rootUri`
    #[serde(rename = "rootPath", skip_serializing_if = "Option::is_none")]
    pub root_path: Option<String>,

    /// The rootUri of the workspace. Is null if no
    /// folder is open. If both `rootPath` and `rootUri` are set
    /// `rootUri` wins.
    ///
    /// @deprecated in favour of `workspaceFolders`
    #[serde(rename = "rootUri")]
    #[deprecated(note = "Use `workspace_folders` instead")]
    pub root_uri: Option<DocumentUri>,

    /// User provided initialization options
    #[serde(
        rename = "initializationOptions",
        skip_serializing_if = "Option::is_none"
    )]
    pub initialization_options: Option<LSPAny>,

    /// The capabilities provided by the client (editor or tool)
    pub capabilities: ClientCapabilities,

    /// The initial trace setting. If omitted trace is disabled ('off').
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<TraceValue>,

    /// The workspace folders configured in the client when the server starts.
    /// This property is only available if the client supports workspace folders.
    /// It can be `null` if the client supports workspace folders but none are
    /// configured.
    ///
    /// @since 3.6.0
    #[serde(rename = "workspaceFolders", skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<WorkspaceFolders>,
}

/// Subobject for clientInfo in InitializeParams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// The name of the client as defined by the client
    pub name: String,

    /// The client's version as defined by the client
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
