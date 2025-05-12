use crate::types::{
    annotations::ChangeAnnotationSupport,
    base::LSPAny,
    filesystem::FileSystemWatcher,
    kinds::{FailureHandlingKind, ResourceOperationKind},
    progress::{PartialResultParams, WorkDoneProgressParams},
    results::PreviousResultId,
    symbols::{ResolveSupport, SymbolKindCapabilities, TagSupport},
    uri::DocumentUri,
};
use serde::{Deserialize, Serialize};

use super::TagSupport;

/// A workspace folder as returned in the initialize request
/// or workspace/didChangeWorkspaceFolders notification.  
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    /// The associated URI for this workspace folder.
    pub uri: DocumentUri,

    /// The name of the workspace folder.
    /// Used to refer to this workspace folder in the user interface.
    pub name: String,
}
/// A list of `WorkspaceFolder`s, as used in InitializeParams.workspace_folders`
pub type WorkspaceFolders = Vec<WorkspaceFolder>;

/// The parameters of the workspace/didChangeClientCapabilities notification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeConfigurationClientCapabilities {
    /// Did change configuration notification supports dynamic registration.
    dynamic_registration: Option<bool>,
}

/// Notification for workspace/didChangeConfiguration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeConfigurationParams {
    /// The actual changed settings
    pub settings: LSPAny,
}

/// Did change watched files notification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeWatchedFilesRegistrationOptions {
    /// The watchers to register
    pub watchers: Vec<FileSystemWatcher>,
}

/// Workspace‐specific client capabilities as defined by the Language Server Protocol.
///
/// This struct corresponds to the optional `workspace` property on the top‐level
/// [`ClientCapabilities`](https://microsoft.github.io/language-server-protocol/specifications/specification-current/#client_capabilities)
/// object.  It allows a client to declare which workspace features it supports:
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceClientCapabilities {
    ///  The client supports applying batch edits to
    ///  the workspace by supporting the request 'workspace/applyEdit'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_edit: Option<bool>,

    /// Capabilities specific to `WorkspaceEdit`s
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_edit: Option<WorkspaceEditClientCapabilities>,

    /// Capabilities specific to the `workspace/didChangeConfiguration` notification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_change_configuration: Option<DidChangeConfigurationClientCapabilities>,

    // Capabilities specific to the `workspace/didChangeWatchedFiles` notification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_change_watched_files: Option<DidChangeWatchedFilesRegistrationOptions>,

    /// Capabilities specific to the `workspace/symbol` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<WorkspaceSymbolClientCapabilities>,

    /// Capabilities specific to the `workspace/executeCommand` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execute_command: Option<ExecuteCommandClientCapabilities>,

    /// The client has support for workspace folders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<bool>,

    /// the client supports workspace/configuration requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<bool>,

    /// Capabilities specific to the semantic token requests scoped to the workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_tokens: Option<SemanticTokensWorkspaceClientCapabilities>,

    /// Capabilities specific to the code lens requests scoped to the workspace
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_lens: Option<CodeLensWorkspaceClientCapabilities>,
}

/// WorkspaceEditClientCapabilities defines capabilities specific to workspace edits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceEditClientCapabilities {
    /// The client supports versioned document changes in `WorkspaceEdit`s

    #[serde(rename = "documentChanges", skip_serializing_if = "Option::is_none")]
    pub document_changes: Option<bool>,

    /// The resource operations the client supports. Clients should at least
    /// support 'create', 'rename' and 'delete' files and folders.
    #[serde(rename = "resourceOperations", skip_serializing_if = "Option::is_none")]
    pub resource_operations: Option<Vec<ResourceOperationKind>>,

    /// The failure handling strategy of a client if applying the workspace edit
    /// fails.
    #[serde(rename = "failureHandling", skip_serializing_if = "Option::is_none")]
    pub failure_handling: Option<FailureHandlingKind>,

    /// Whether the client normalizes line endings to the client specific setting.
    /// If set to `true` the client will normalize line ending characters
    /// in a workspace edit to the client specific new line character(s).
    #[serde(
        rename = "normalizesLineEndings",
        skip_serializing_if = "Option::is_none"
    )]
    pub normalizes_line_endings: Option<bool>,

    /// Whether the client in general supports change annotations on text edits,
    /// create file, rename file and delete file changes.
    #[serde(
        rename = "changeAnnotationSupport",
        skip_serializing_if = "Option::is_none"
    )]
    pub change_annotation_support: Option<ChangeAnnotationSupport>,
}

/// Parameters of the workspace diagnostic request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDiagnosticParams {
    /// WorkDoneProgress token
    #[serde(flatten)]
    pub work_done: WorkDoneProgressParams,

    /// Partial result token
    #[serde(flatten)]
    pub partial_result: PartialResultParams,

    /// The additional identifier provided during registration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,

    /// The currently known diagnostic reports with their prevous result ids
    prevous_result_ids: Vec<PreviousResultId>,
}

/// Parameters of the workspace symbols request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSymbolClientCapabilities {
    /// Symbol request supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// Specific capabilities for the `SymbolKind` in the `workspace/symbol` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_kind: Option<SymbolKindCapabilities>,

    /// The client supports tags on `SymbolInformation` and `WorkspaceSymbol`.
    /// Clients supporting tags have to handle unknown tags gracefully.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag_support: Option<TagSupport>,

    /// The client support partial workspace symbols. The client will send the
    /// request `workspaceSymbol/resolve` to the server to resolve additional
    /// properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_support: Option<ResolveSupport>,
}
