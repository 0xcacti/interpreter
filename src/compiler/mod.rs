pub mod error;
use crate::{code::Instructions, evaluator::object, parser::ast};
use error::CompileError;

pub struct Compiler {
    pub instructions: Instructions,
    pub constants: Vec<object::Object>,
}

pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<object::Object>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            instructions: Instructions::new(),
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
        code::{make, Opcode},
        lexer::Lexer,
        parser::{ast::Node, Parser},
    };

    use super::*;

    fn test_compilation(
        input: &str,
        actual_instructions: Vec<Instructions>,
        actual_constants: Vec<object::Object>,
    ) {
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        let mut compiler = Compiler::new();
        compiler.compile(Node::Program(program)).unwrap();
        let bytecode = compiler.bytecode();
        test_instructions(bytecode.instructions, actual_instructions);
        test_constants(bytecode.constants, actual_constants);
    }

    fn concatenate_instructions(instructions: &Vec<Instructions>) -> Instructions {
        let mut concattenated: Instructions = Instructions::new();
        for instruction in instructions {
            concattenated.extend(instruction.clone());
        }
        concattenated
    }

    fn test_instructions(expected: Instructions, actual: Vec<Instructions>) {
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
            vec![
                make(Opcode::Constant, Instructions::new(vec![0])),
                make(Opcode::Constant, Instructions::new(vec![1])),
            ],
            vec![object::Object::Integer(1), object::Object::Integer(2)],
        );
    }
}
