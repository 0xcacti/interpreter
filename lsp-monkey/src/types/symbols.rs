use crate::types::UInteger;
use serde::{Deserialize, Serialize};

pub type SymbolKind = UInteger;
pub mod symbol_kind {
    use super::SymbolKind;
    pub const FILE: SymbolKind = 1;
    pub const MODULE: SymbolKind = 2;
    pub const NAMESPACE: SymbolKind = 3;
    pub const PACKAGE: SymbolKind = 4;
    pub const CLASS: SymbolKind = 5;
    pub const METHOD: SymbolKind = 6;
    pub const PROPERTY: SymbolKind = 7;
    pub const FIELD: SymbolKind = 8;
    pub const CONSTRUCTOR: SymbolKind = 9;
    pub const ENUM: SymbolKind = 10;
    pub const INTERFACE: SymbolKind = 11;
    pub const FUNCTION: SymbolKind = 12;
    pub const VARIABLE: SymbolKind = 13;
    pub const CONSTANT: SymbolKind = 14;
    pub const STRING: SymbolKind = 15;
    pub const NUMBER: SymbolKind = 16;
    pub const BOOLEAN: SymbolKind = 17;
    pub const ARRAY: SymbolKind = 18;
    pub const OBJECT: SymbolKind = 19;
    pub const KEY: SymbolKind = 20;
    pub const NULL: SymbolKind = 21;
    pub const ENUM_MEMBER: SymbolKind = 22;
    pub const STRUCT: SymbolKind = 23;
    pub const EVENT: SymbolKind = 24;
    pub const OPERATOR: SymbolKind = 25;
    pub const TYPE_PARAMETER: SymbolKind = 26;
}

pub type SymbolTag = UInteger;
pub mod symbol_tag {
    use super::SymbolTag;
    /// Render a symbol as obsolete, usually using a strike-out.
    pub const DEPRECATED: SymbolTag = 1;
}

/// Struct for represeting `SymbolKind` within other structs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolKindCapabilities {
    /// The symbol kind values the client supports. When this
    /// property exists the client also guarantees that it will
    /// handle values outside its set gracefully and falls back
    /// to a default value when unknown.
    ///
    /// If this property is not present the client only supports
    /// the symbol kinds from `File` to `Array` as defined in
    /// the initial version of the protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_set: Option<Vec<SymbolKind>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagSupport {
    /// The tags supported by the client.
    pub value_set: Vec<SymbolTag>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveSupport {
    /// The properties the client can resolve lazily. Usually location.range
    pub properties: Vec<String>,
}
