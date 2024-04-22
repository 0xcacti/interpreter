use crate::{code::Instructions, evaluator::object::Object};
use std::rc::Rc;

use super::error::VmError;

#[derive(Debug, Clone)]
pub struct Frame {
    pub function: Rc<Object>,
    pub ip: isize,
}

impl Frame {
    pub fn new(function: Rc<Object>) -> Result<Frame, VmError> {
        match &*function {
            Object::CompiledFunction(_) => Ok(Frame { function, ip: -1 }),
            _ => Err(VmError::new(format!(
                "Expected CompiledFunction, got {:?}",
                function
            ))),
        }
    }

    pub fn instructions(&self) -> Result<Instructions, VmError> {
        match &*self.function {
            Object::CompiledFunction(function) => Ok(function.clone()),
            _ => Err(VmError::new(format!(
                "Expected CompiledFunction, got {:?}",
                self.function
            ))),
        }
    }
}
