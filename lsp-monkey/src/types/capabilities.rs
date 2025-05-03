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
pub struct ClientCapabilities {
    /// workspace specific client capabilities
    workspace: Option<WorkspaceClientCapabilities>, // TODO: WorkspaceClientCapabilities
}

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

/// ChangeAnnotationSupport defines capabilities specific to change annotations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeAnnotationSupport {
    /// Whether the client groups edits with equal labels into tree nodes,
    /// for instance all edits labelled with "Changes in Strings" would
    /// be a tree node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub groups_on_label: Option<bool>,
}
