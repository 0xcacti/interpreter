use thiserror::Error;

#[derive(Debug, Clone, Error, Eq, PartialEq)]
#[error("{msg}")]
pub struct ObjectError {
    pub msg: String,
}

impl ObjectError {
    pub fn new(msg: String) -> Self {
        ObjectError { msg }
    }
}
