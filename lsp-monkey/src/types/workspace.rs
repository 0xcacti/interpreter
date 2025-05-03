use crate::types::uri::DocumentUri;
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
