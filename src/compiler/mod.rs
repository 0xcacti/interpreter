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
        code::make,
        lexer::Lexer,
        parser::{ast::Node, Parser},
    };

    use super::*;

    fn test_compilation(
        input: &str,
        expected_instructions: Vec<Instructions>,
        expected_constants: Vec<object::Object>,
    ) {
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        let mut compiler = Compiler::new();
        compiler.compile(Node::Program(program)).unwrap();
        let bytecode = compiler.bytecode();
        test_instructions(expected_instructions, bytecode.instructions);
        test_constants(expected_constants, bytecode.constants);
    }

    fn concatenate_instructions(instructions: &Vec<Instructions>) -> Instructions {
        let mut concattenated = vec![];
        for instruction in instructions {
            concattenated.extend(instruction);
        }
        concattenated
    }

    fn test_instructions(actual: Vec<Instructions>, expected: Instructions) {
        let concattenated = concatenate_instructions(&actual);

        assert_eq!(concattenated.len(), actual.len());
        for (i, instruction) in concattenated.iter().enumerate() {
            assert_eq!(expected[i], *instruction);
        }
    }

    fn test_constants(expected: Vec<object::Object>, actual: Vec<object::Object>) {
        assert_eq!(expected.len(), actual.len());
        for (i, constant) in expected.iter().enumerate() {
            match constant {
                object::Object::Integer(expected) => match actual[i] {
                    object::Object::Integer(actual) => assert_eq!(expected, &actual),
                    _ => panic!("constant not integer"),
                },
                _ => panic!("constant not integer"),
            }
        }
    }

    #[test]
    fn it_compiles_integer_arithmetic() {
        test_compilation(
            "1 + 2",
            vec![make(0, vec![0]), make(0, vec![1])],
            vec![object::Object::Integer(1), object::Object::Integer(2)],
        );
    }
}
