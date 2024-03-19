use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{msg}")]
pub struct VmError<'a> {
    pub msg: &'a str,
}

impl<'a> VmError<'a> {
    pub fn new(msg: &str) -> VmError {
        VmError { msg }
    }
}
