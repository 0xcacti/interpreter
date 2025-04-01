use thiserror::Error;

#[derive(Error, Debug)]
pub enum LspError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Failed to parse content length: {0}")]
    Parse(#[from] std::num::ParseIntError),

    #[error("LSP protocol error: {0}")]
    Protocol(String),
}

impl LspError {
    pub fn new(msg: String) -> Self {
        LspError::Protocol(msg)
    }
}

