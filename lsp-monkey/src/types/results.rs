use crate::types::uri::DocumentUri;
use serde::{Deserialize, Serialize};

/// A previous result id in a workspace pull request
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviousResultId {
    /// The URI for which the client knows a result ID
    pub uri: DocumentUri,
    /// The value of the previous result Id
    pub value: String,
}
