use thiserror::Error;

pub type ParserErrors = Vec<ParserError>;

#[derive(Debug, Clone, Error)]
#[error("{msg}")]
pub struct ParserError {
    pub msg: String,
}

impl ParserError {
    pub fn new(msg: String) -> Self {
        ParserError { msg }
    }
}
