use serde::{Deserialize, Serialize};

/// How a client wants the server to behave if applying a WorkspaceEdit fails.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FailureHandlingKind {
    /// Applying the workspace change is simply aborted if one of the changes
    /// provided fails. All operations executed before the failing operation
    /// stay executed.
    Abort,

    /// All operations are executed transactionally. That means they either all
    /// succeed or no changes at all are applied to the workspace.
    Transactional,

    /// If the workspace edit contains only textual file changes they are
    /// executed transactionally. If resource changes (create, rename or delete
    /// file) are part of the change the failure handling strategy is abort.
    TextOnlyTransactional,

    /// The client tries to undo the operations already executed. But there is no
    /// guarantee this will succeed.
    Undo,
}

/// The kind of resource operations supported by the client
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResourceOperationKind {
    /// Supports create operations for files and folders
    Create,

    /// Supports rename operations for files and folders
    Rename,

    /// Supports delete operations for files and folders
    Delete,
}
