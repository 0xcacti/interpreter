pub mod builtin;
pub mod environment;
pub mod error;

use std::{
    collections::HashMap,
    fmt::{Display, Formatter, Result},
    hash::{Hash, Hasher},
    rc::Rc,
};

use crate::code;
use crate::parser::ast::{Node, Statement};

use environment::Env;

use self::builtin::Builtin;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledFunction {
    pub instructions: code::Instructions,
    pub num_parameters: usize,
    pub num_locals: usize,
}

impl CompiledFunction {
    pub fn new(instructions: code::Instructions, num_parameters: usize, num_locals: usize) -> Self {
        CompiledFunction {
            instructions,
            num_parameters,
            num_locals,
        }
    }

    pub fn instructions(&self) -> &code::Instructions {
        &self.instructions
    }

    pub fn num_locals(&self) -> usize {
        self.num_locals
    }

    pub fn num_parameters(&self) -> usize {
        self.num_parameters
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    String(String),
    Array(Vec<Rc<Object>>),
    Hash(HashMap<Rc<Object>, Rc<Object>>),
    ReturnValue(Rc<Object>),
    Function(Vec<String>, Vec<Statement>, Env),
    CompiledFunction(Rc<CompiledFunction>),
    Builtin(Builtin),
    Macro(Vec<String>, Vec<Statement>, Env),
    Quote(Node),
    Null,
    Closure(Rc<CompiledFunction>, Vec<Rc<Object>>),
}

impl Eq for Object {}

impl Object {
    pub fn is_integer(&self) -> bool {
        match self {
            Object::Integer(_) => true,
            _ => false,
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Object::Integer(i) => write!(f, "{}", i),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::String(s) => write!(f, "{}", s),
            Object::ReturnValue(o) => write!(f, "{}", o),
            Object::Null => write!(f, "null"),
            Object::Function(parameters, _, _) => {
                let params = parameters.join(", ");
                write!(f, "fn({}) {{...}}", params)
            }
            Object::Builtin(b) => write!(f, "{}", b),
            Object::Array(a) => {
                let elements: Vec<String> = a.iter().map(|e| format!("{}", e)).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            Object::Hash(h) => {
                let mut pairs: Vec<String> = Vec::new();
                for (k, v) in h.iter() {
                    pairs.push(format!("{}: {}", k, v));
                }
                write!(f, "{{{}}}", pairs.join(", "))
            }
            Object::Quote(s) => {
                write!(f, "QUOTE({})", s)
            }
            Object::Macro(parameters, _, _) => {
                let params = parameters.join(", ");
                write!(f, "macro({}) {{...}}", params)
            }
            Object::CompiledFunction(compiled_function) => {
                write!(f, "{}", compiled_function.instructions)
            }
            Object::Closure(_, _) => write!(f, "closure | |"),
        }
    }
}

impl From<bool> for Object {
    fn from(b: bool) -> Self {
        Object::Boolean(b)
    }
}

impl From<i64> for Object {
    fn from(i: i64) -> Self {
        Object::Integer(i)
    }
}

impl From<String> for Object {
    fn from(s: String) -> Self {
        Object::String(s)
    }
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Boolean(b) => *b,
            Object::Null => false,
            _ => true,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Object::String(s) => s.is_empty(),
            Object::Array(a) => a.is_empty(),
            _ => false,
        }
    }

    pub fn is_hashable(&self) -> bool {
        match self {
            Object::Integer(_) => true,
            Object::Boolean(_) => true,
            Object::String(_) => true,
            _ => false,
        }
    }
}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Object::Integer(i) => i.hash(state),
            Object::Boolean(b) => b.hash(state),
            Object::String(s) => s.hash(state),
            _ => "".hash(state),
        }
    }
}
