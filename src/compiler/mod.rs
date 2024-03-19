pub mod error;
use crate::{
    code::{self, Instructions, Opcode},
    evaluator::object,
    parser::ast::{Expression, Literal, Node, Statement},
};
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
            instructions: Instructions::new(vec![]),
            constants: vec![],
        }
    }

    pub fn compile(&mut self, program: Node) -> Result<(), CompileError> {
        match program {
            Node::Program(program) => {
                for statement in program {
                    self.compile(Node::Statement(statement))?;
                }
            }
            Node::Statement(statement) => match statement {
                Statement::Expression(expression) => {
                    self.compile(Node::Expression(expression))?;
                }
                _ => {
                    panic!("not implemented")
                }
            },

            Node::Expression(expression) => match expression {
                Expression::Infix(left, _, right) => {
                    self.compile(Node::Expression(*left))?;
                    self.compile(Node::Expression(*right))?;
                }
                Expression::Literal(literal) => match literal {
                    Literal::Integer(value) => {
                        let integer = object::Object::Integer(value);
                        let position = self.add_constant(integer);
                        self.emit(Opcode::Constant, vec![position]);
                    }
                    _ => {
                        panic!("not implemented")
                    }
                },
                _ => {
                    panic!("not implemented")
                }
            },
        }
        Ok(())
    }

    pub fn bytecode(&self) -> Bytecode {
        Bytecode {
            instructions: self.instructions.clone(),
            constants: self.constants.clone(),
        }
    }

    pub fn add_constant(&mut self, object: object::Object) -> usize {
        self.constants.push(object);

        self.constants.len() - 1
    }

    pub fn emit(&mut self, opcode: Opcode, operands: Vec<usize>) -> usize {
        let ins = code::make(opcode, operands);
        let pos = self.add_instructions(ins);
        pos
    }

    pub fn add_instructions(&mut self, instructions: Vec<u8>) -> usize {
        let position_new = self.instructions.len();
        self.instructions.extend(Instructions::new(instructions));
        position_new
    }
}

#[cfg(test)]
mod test {
    use crate::{code::make, lexer::Lexer, parser::Parser};

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
        let mut concattenated: Instructions = Instructions::new(vec![]);
        for instruction in instructions {
            concattenated.extend(instruction.clone());
        }
        concattenated
    }

    fn test_instructions(expected: Instructions, actual: Vec<Instructions>) {
        let concattenated = concatenate_instructions(&actual);
        assert_eq!(expected, concattenated);
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
                Instructions::new(make(Opcode::Constant, vec![0])),
                Instructions::new(make(Opcode::Constant, vec![1])),
            ],
            vec![object::Object::Integer(1), object::Object::Integer(2)],
        );
    }
}
