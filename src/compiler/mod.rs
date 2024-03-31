pub mod error;
use crate::{
    code::{self, Instructions, Opcode},
    evaluator::object::Object,
    parser::ast::{Expression, Literal, Node, Statement},
    token::Token,
};
use error::CompileError;

use std::rc::Rc;

pub struct Compiler {
    pub instructions: Instructions,
    pub constants: Vec<Rc<Object>>,
}

pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Rc<Object>>,
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
                    self.emit(Opcode::Pop, vec![]);
                }
                _ => {
                    panic!("not implemented")
                }
            },

            Node::Expression(expression) => match expression {
                Expression::Infix(left, operator, right) => {
                    self.compile(Node::Expression(*left))?;
                    self.compile(Node::Expression(*right))?;
                    match operator {
                        Token::Plus => {
                            self.emit(Opcode::Add, vec![]);
                        }
                        Token::Dash => {
                            self.emit(Opcode::Sub, vec![]);
                        }
                        Token::Asterisk => {
                            self.emit(Opcode::Mul, vec![]);
                        }
                        Token::Slash => {
                            self.emit(Opcode::Div, vec![]);
                        }
                        _ => {
                            panic!("not implemented")
                        }
                    }
                }
                Expression::Literal(literal) => match literal {
                    Literal::Integer(value) => {
                        let integer = Rc::new(Object::Integer(value));
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

    pub fn add_constant(&mut self, object: Rc<Object>) -> usize {
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
        actual_constants: Vec<Rc<Object>>,
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

    fn test_constants(expected: Vec<Rc<Object>>, actual: Vec<Rc<Object>>) {
        assert_eq!(expected.len(), actual.len());
        for (i, constant) in expected.iter().enumerate() {
            match &**constant {
                Object::Integer(expected) => match &*actual[i] {
                    Object::Integer(actual) => assert_eq!(expected, actual),
                    _ => panic!("constant not integer"),
                },
                _ => panic!("constant not integer"),
            }
        }
    }

    #[test]
    fn it_pops_expressions() {
        test_compilation(
            "1; 2",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Pop, vec![]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(1)), Rc::new(Object::Integer(2))],
        );
    }

    #[test]
    fn it_compiles_integer_arithmetic() {
        test_compilation(
            "1 + 2",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Add, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(1)), Rc::new(Object::Integer(2))],
        );

        test_compilation(
            "1 - 2",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Sub, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(1)), Rc::new(Object::Integer(2))],
        );

        test_compilation(
            "1 * 2",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Mul, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(1)), Rc::new(Object::Integer(2))],
        );

        test_compilation(
            "2 / 1",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Div, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(2)), Rc::new(Object::Integer(1))],
        );
    }
}
