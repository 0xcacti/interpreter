use serde::{Deserialize, Serialize};

/// ChangeAnnotationSupport defines capabilities specific to change annotations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeAnnotationSupport {
    /// Whether the client groups edits with equal labels into tree nodes,
    /// for instance all edits labelled with "Changes in Strings" would
    /// be a tree node.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups_on_label: Option<bool>,
}
