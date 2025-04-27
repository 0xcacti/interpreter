use crate::types::base::UInteger;
use serde::{Deserialize, Serialize};

/// Position in a text document expressed as a tuple of line and character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// The zero-based line number
    pub line: UInteger,

    /// The zero-based character offset
    pub character: UInteger,
}

/// Parameters for progress notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoverParams {
    /// The text documenst URI in string form
    pub text_document: String,

    /// The position in the text document
    pub position: Position,
}

/// Result of a hover request
pub struct HoverResult {
    pub value: String,
}
