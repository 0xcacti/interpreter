use crate::types::{
    annotations::ChangeAnnotationSupport,
    kinds::{FailureHandlingKind, ResourceOperationKind},
    uri::DocumentUri,
};
use serde::{Deserialize, Serialize};

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

/// Workspace‐specific client capabilities as defined by the Language Server Protocol.
///
/// This struct corresponds to the optional `workspace` property on the top‐level
/// [`ClientCapabilities`](https://microsoft.github.io/language-server-protocol/specifications/specification-current/#client_capabilities)
/// object.  It allows a client to declare which workspace features it supports:
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceClientCapabilities {
    ///  The client supports applying batch edits to
    ///  the workspace by supporting the request 'workspace/applyEdit'
    #[serde(default, rename = "applyEdit", skip_serializing_if = "Option::is_none")]
    pub apply_edit: Option<bool>,

    /// Capabilities specific to `WorkspaceEdit`s
    #[serde(
        default,
        rename = "workspaceEdit",
        skip_serializing_if = "Option::is_none"
    )]
    pub workspace_edit: Option<WorkspaceEditClientCapabilities>,
}

/// WorkspaceEditClientCapabilities defines capabilities specific to workspace edits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceEditClientCapabilities {
    /// The client supports versioned document changes in `WorkspaceEdit`s

    #[serde(
        default,
        rename = "documentChanges",
        skip_serializing_if = "Option::is_none"
    )]
    pub document_changes: Option<bool>,

    /// The resource operations the client supports. Clients should at least
    /// support 'create', 'rename' and 'delete' files and folders.
    #[serde(
        default,
        rename = "resourceOperations",
        skip_serializing_if = "Option::is_none"
    )]
    resource_operations: Option<Vec<ResourceOperationKind>>,

    /// The failure handling strategy of a client if applying the workspace edit
    /// fails.
    #[serde(
        default,
        rename = "failureHandling",
        skip_serializing_if = "Option::is_none"
    )]
    failure_handling: Option<FailureHandlingKind>,

    /// Whether the client normalizes line endings to the client specific setting.
    /// If set to `true` the client will normalize line ending characters
    /// in a workspace edit to the client specific new line character(s).
    #[serde(
        default,
        rename = "normalizesLineEndings",
        skip_serializing_if = "Option::is_none"
    )]
    normalizes_line_endings: Option<bool>,

    /// Whether the client in general supports change annotations on text edits,
    /// create file, rename file and delete file changes.
    #[serde(
        default,
        rename = "changeAnnotationSupport",
        skip_serializing_if = "Option::is_none"
    )]
    pub change_annotation_support: Option<ChangeAnnotationSupport>,
}
