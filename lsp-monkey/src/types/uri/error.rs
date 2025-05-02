use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{msg}")]
pub struct UriError {
    pub msg: String,
}

impl UriError {
    pub fn new(msg: String) -> Self {
        UriError { msg }
    }
}
