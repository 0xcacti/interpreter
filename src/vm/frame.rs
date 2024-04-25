use crate::{code::Instructions, object::Object};
use std::rc::Rc;

use super::error::VmError;

#[derive(Debug, Clone)]
pub struct Frame {
    pub function: Rc<Object>,
    pub ip: isize,
    pub base_pointer: usize,
}

impl Frame {
    pub fn new(function: Rc<Object>, base_pointer: usize) -> Result<Frame, VmError> {
        match &*function {
            Object::Closure(compiled_function, _) => Ok(Frame {
                function: Rc::new(Object::CompiledFunction(compiled_function.clone())),
                ip: -1,
                base_pointer,
            }),
            _ => Err(VmError::new(format!(
                "Expected CompiledFunction, got {:?}",
                function
            ))),
        }
    }

    pub fn instructions(&self) -> Result<Instructions, VmError> {
        match &*self.function {
            Object::CompiledFunction(compiled_function) => {
                Ok(compiled_function.instructions().clone())
            }
            _ => Err(VmError::new(format!(
                "Expected CompiledFunction, got {:?}",
                self.function
            ))),
        }
    }
}
