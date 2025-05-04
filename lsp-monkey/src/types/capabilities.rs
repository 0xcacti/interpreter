use crate::types::{base::LSPAny, workspace::WorkspaceClientCapabilities};

use serde::{Deserialize, Serialize};

/// Represents the level of verbosity with which the server systematically
/// reports its execution trace using $/logTrace notifications.
/// The initial trace value is set by the client at initialization and can be
/// modified later using the $/setTrace notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TraceValue {
    /// No tracing
    Off,
    /// Tracing is enabled
    Messages,
    /// Tracing is enabled and verbose
    Verbose,
}

/// ClientCapabilities define capabilities for dynamic registration,
/// workspace and text document features the client supports.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    /// workspace specific client capabilities TODO: WorkspaceClientCapabilities
    workspace: Option<WorkspaceClientCapabilities>,

    /// Text document specific client capabilities TODO: TextDocumentClientCapabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    text_document: Option<TextDocumentClientCapabilities>,
}
