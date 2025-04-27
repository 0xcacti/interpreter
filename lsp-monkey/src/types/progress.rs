use crate::types::base::UInteger;
use crate::types::message::ProgressToken;
use serde::{Deserialize, Serialize};

/// A work‐done progress payload, tagged by `"kind"`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum WorkDoneProgress {
    /// Payload type to start a progress report.
    #[serde(rename = "begin")]
    Begin {
        /// Mandatory title of the progress operation, used to briefly inform about the type of
        /// operation being performed.
        ///
        /// Examples: "Indexing" or "Linking dependencies".
        title: String,

        /// Controls if a cancel button should show to allow the user to cancel the
        /// long running operation. Clients that don't support cancellation are
        /// allowed to ignore the setting.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cancellable: Option<bool>,

        /// Optional, more detailed associated progress message. Contains
        /// complementary information to the `title`.
        ///
        /// Examples: "3/25 files", "project/src/module2", "node_modules/some_dep".
        /// If unset, the previous progress message (if any) is still valid.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,

        /// Optional progress percentage to display (value 100 is considered 100%).
        /// If not provided infinite progress is assumed and clients are allowed
        /// to ignore the `percentage` value in subsequent report notifications.
        ///
        /// The value should be steadily rising. Clients are free to ignore values
        /// that are not following this rule. The value range is [0, 100].
        #[serde(default, skip_serializing_if = "Option::is_none")]
        percentage: Option<UInteger>,
    },

    /// Payload type to report intermediate progress.
    #[serde(rename = "report")]
    Report {
        /// Updated cancel-button state.
        ///

        /// Controls enablement state of a cancel button. This property is only valid
        /// if a cancel button got requested in the `WorkDoneProgressBegin` payload.
        ///
        /// Clients that don't support cancellation or don't support control the
        /// button's enablement state are allowed to ignore the setting.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cancellable: Option<bool>,

        /// Optional, more detailed associated progress message. Contains
        /// complementary information to the `title`.
        ///
        /// Examples: "3/25 files", "project/src/module2", "node_modules/some_dep".
        /// If unset, the previous progress message (if any) is still valid.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,

        /// Optional progress percentage to display (value 100 is considered 100%).
        /// If not provided infinite progress is assumed and clients are allowed
        /// to ignore the `percentage` value in subsequent report notifications.
        ///
        /// The value should be steadily rising. Clients are free to ignore values
        /// that are not following this rule. The value range is [0, 100].
        #[serde(default, skip_serializing_if = "Option::is_none")]
        percentage: Option<UInteger>,
    },

    /// Payload type to end a progress report.
    #[serde(rename = "end")]
    End {
        /// Optional, a final message indicating to for example indicate the outcome
        /// of the operation.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
}

/// Mixin you embed in *request* parameter structs when the client wants
/// to receive progress updates for that request.
///
/// If `work_done_token` is `Some`, the client is telling you:
/// “Here’s a token—feel free to send `$\/progress` notifications
/// tagged with this value while you work on my request.”
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkDoneProgressParams {
    /// An optional token that a server can use to report work done progress.
    #[serde(
        rename = "workDoneToken",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    token: Option<ProgressToken>,
}

/// Server capability option signalling that the server supports
/// sending `$\/progress` notifications for this feature.
///
/// When you set this to `Some(true)` in your `InitializeResult.capabilities`
/// (e.g. under `referencesProvider`), clients will know they can
/// attach a `workDoneToken` and expect progress updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkDoneProgressOptions {
    /// If `true`, the server will send `$\/progress` notifications
    /// for client-initiated tokens on this feature.
    #[serde(
        rename = "workDoneProgress",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub work_done_progress: Option<bool>,
}

/// Parameters for the `window/workDoneProgress/create` request.
///
/// Servers call this to get a fresh token they can use *outside* of
/// any particular request (e.g. for background re‐indexing).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkDoneProgressCreateParams {
    /// A new token to be used exactly once: one `Begin`, zero or more `Report`,
    /// and one `End` notification against it.
    pub token: ProgressToken,
}

/// Mixin for *streaming partial results* in requests like `textDocument/references`.
///
/// If `partial_result_token` is `Some`, the server may send
/// `$\/progress` notifications whose `value` payloads are chunks of
/// the final result array.
#[derive(Debug, Clone, Deserialize)]
pub struct PartialResultParams {
    /// An optional token that a server can use to report partial results (e.g.
    /// streaming) to the client.
    #[serde(
        rename = "partialResultToken",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub partial_result_token: Option<ProgressToken>,
}
