use crate::types::{base::LSPAny, workspace::WorkspaceClientCapabilities};

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
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    /// workspace specific client capabilities TODO: WorkspaceClientCapabilities
    workspace: Option<WorkspaceClientCapabilities>,

    /// Text document specific client capabilities TODO: TextDocumentClientCapabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    text_document: Option<TextDocumentClientCapabilities>,
}

/// TextDocumentClientCapabilities define capabilities the
/// editor / tool provides on text documents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentClientCapabilities {
    /// Synchronization capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synchronization: Option<SynchronizationClientCapabilities>,

    /// Capabilities specific to the `textDocument/completion` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<CompletionClientCapabilities>,

    /// Capabilities specific to the `textDocument/hover` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover: Option<HoverClientCapabilities>,

    /// Capabilities specific to the `textDocument/signatureHelp` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_help: Option<SignatureHelpClientCapabilities>,

    /// Capabilities specific to the `textDocument/declaration` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration: Option<DeclarationClientCapabilities>,

    /// Capabilities specific to the `textDocument/definition` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<DefinitionClientCapabilities>,

    /// Capabilities specific to the `textDocument/typeDefinition` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_definition: Option<TypeDefinitionClientCapabilities>,

    /// Capabilities specific to the `textDocument/implementation` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation: Option<ImplementationClientCapabilities>,

    /// Capabilities specific to the `textDocument/references` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<ReferencesClientCapabilities>,

    /// Capabilities specific to the `textDocument/documentHighlight` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_highlight: Option<DocumentHighlightClientCapabilities>,

    /// Capabilities specific to the `textDocument/documentSymbol` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_symbol: Option<DocumentSymbolClientCapabilities>,

    /// Capabilities specific to the `textDocument/codeAction` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_action: Option<CodeActionClientCapabilities>,

    /// Capabilities specific to the `textDocument/codeLens` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_lens: Option<CodeLensClientCapabilities>,

    /// Capabilities specific to the `textDocument/documentLink` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_link: Option<DocumentLinkClientCapabilities>,

    /// Capabilities specific to the `textDocument/documentColor` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_provider: Option<DocumentColorClientCapabilities>,

    /// Capabilities specific to the `textDocument/formatting` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatting: Option<DocumentFormattingClientCapabilities>,

    /// Capabilities specific to the `textDocument/rangeFormatting` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_formatting: Option<DocumentRangeFormattingClientCapabilities>,

    /// Capabilities specific to the `textDocument/onTypeFormatting` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_type_formatting: Option<DocumentOnTypeFormattingClientCapabilities>,

    /// Capabilities specific to the `textDocument/rename` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<RenameClientCapabilities>,

    /// Capabilities specific to the `textDocument/publishDiagnostics` notification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_diagnostics: Option<PublishDiagnosticsClientCapabilities>,

    /// Capabilities specific to the `textDocument/foldingRange` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folding_range: Option<FoldingRangeClientCapabilities>,

    /// Capabilities specific to the `textDocument/selectionRange` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_range: Option<SelectionRangeClientCapabilities>,

    /// Capabilities specific to the `textDocument/linkedEditingRange` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_editing_range: Option<LinkedEditingRangeClientCapabilities>,

    /// Capabilitiesspecific to the various call hierarchy requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_hierarchy: Option<CallHierarchyClientCapabilities>,

    /// Capabilities specific to the various semantic token requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_tokens: Option<SemanticTokensClientCapabilities>,

    /// Capabilities specific to the `textDocument/moniker` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub moniker: Option<MonikerClientCapabilities>,

    /// Capabilities specific to the various type hierarchy requests
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_hierarchy: Option<TypeHierarchyClientCapabilities>,

    /// Capabilities specific to the `textDocument/inlineValue` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_value: Option<InlineValueClientCapabilities>,

    /// Capabilities specific to the `textDocument/inlayHint` request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inlay_hint: Option<InlayHintClientCapabilities>,

    /// Capabilities specific to the diagnostic pull model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<DiagnosticClientCapabilities>,
}

/// The client has support for file requests/notifications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileOperationsClientCapabilities {
    /// Whether the client supports dynamic registration for file
    /// requests/notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watchers: Option<FileSystemWatcherClientCapabilities>,

    /// Capabilities specific to the `FileOperationRegistrationOptions`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<FileOperationRegistrationOptions>,
}
