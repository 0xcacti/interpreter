use crate::types::{UInteger, uri::DocumentUri, workspace::WorkspaceFolder};
use serde::{Deserialize, Serialize};

/// A glob pattern is either a raw string or a relative‐pattern object.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GlobPattern {
    /// e.g. `"**/*.rs"`
    Pattern(String),

    /// e.g. `{ baseUri: {...}, pattern: "**/*.rs" }`
    Relative(RelativePattern),
}

/// A helper to construct glob patterns matched relative to a base URI or workspace folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelativePattern {
    /// Either a workspace folder or a raw URI.
    #[serde(rename = "baseUri")]
    pub base: WorkspaceFolderOrUri,

    /// The glob pattern itself.
    pub pattern: String,
}

/// Union of WorkspaceFolder or plain URI
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WorkspaceFolderOrUri {
    Folder(WorkspaceFolder),
    Uri(DocumentUri),
}

/// The kind of file‐system events to watch.  Defaults (1|2|4 = 7) if you omit it.
pub type WatchKind = UInteger;
pub mod watch_kind {
    use super::WatchKind;
    pub const CREATE: WatchKind = 1;
    pub const CHANGE: WatchKind = 2;
    pub const DELETE: WatchKind = 4;
}

/// One individual watcher
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSystemWatcher {
    /// GlobPattern = String | RelativePattern
    pub glob_pattern: GlobPattern,

    /// Watch kinds (create=1, change=2, delete=4).  
    /// If you omit this field, you should treat it as `1|2|4` on the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<WatchKind>,
}

/// An event describing a file change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEvent {
    /// The file’s URI
    pub uri: DocumentUri,

    /// The change type (1 = created, 2 = changed, 3 = deleted)
    #[serde(rename = "type")]
    pub r#type: FileChangeType,
}

/// File‐change type is 1, 2 or 3
pub type FileChangeType = UInteger;
pub mod file_change_type {
    use super::FileChangeType;
    pub const CREATED: FileChangeType = 1;
    pub const CHANGED: FileChangeType = 2;
    pub const DELETED: FileChangeType = 3;
}
