pub mod error;
use crate::{code::Instructions, evaluator::object, parser::ast};
use error::CompileError;

pub struct Compiler {
    pub instructions: Instructions,
    pub constants: Vec<object::Object>,
}

pub struct Bytecode {
    pub instructions: Vec<u8>,
    pub constants: Vec<object::Object>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            instructions: vec![],
            constants: vec![],
        }
    }

    pub fn compile(&mut self, program: ast::Node) -> Result<(), CompileError> {
        Ok(())
    }

    pub fn bytecode(&self) -> Bytecode {
        Bytecode {
            instructions: self.instructions.clone(),
            constants: self.constants.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        lexer::Lexer,
        parser::{ast::Node, Parser},
    };

    use super::*;

    fn test_compilation(input: &str) {
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        let mut compiler = Compiler::new();
        compiler.compile(Node::Program(program)).unwrap();
        let bytecode = compiler.bytecode();
    }

    #[test]
    fn it_compiles_integer_arithmetic() {}
}
