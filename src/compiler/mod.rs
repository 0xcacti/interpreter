pub mod error;
pub mod symbol_table;
use crate::{
    code::{self, Instructions, Opcode},
    evaluator::object::Object,
    parser::ast::{Expression, Literal, Node, Statement},
    token::Token,
};
use error::CompileError;

use std::{cell::RefCell, rc::Rc};

use self::symbol_table::SymbolTable;

pub struct Compiler {
    pub instructions: Instructions,
    pub constants: Rc<RefCell<Vec<Rc<Object>>>>,
    pub last_instruction: EmittedInstruction,
    pub previous_instruction: EmittedInstruction,
    pub symbol_table: Rc<RefCell<SymbolTable>>,
}

pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Rc<RefCell<Vec<Rc<Object>>>>,
}

#[derive(Clone)]
pub struct EmittedInstruction {
    pub opcode: Opcode,
    pub position: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            instructions: Instructions::new(vec![]),
            constants: Rc::new(RefCell::new(vec![])),
            last_instruction: EmittedInstruction {
                opcode: Opcode::Constant,
                position: 0,
            },
            previous_instruction: EmittedInstruction {
                opcode: Opcode::Constant,
                position: 0,
            },
            symbol_table: Rc::new(RefCell::new(SymbolTable::new())),
        }
    }

    pub fn new_with_state(
        symbol_table: Rc<RefCell<SymbolTable>>,
        constants: Rc<RefCell<Vec<Rc<Object>>>>,
    ) -> Self {
        Compiler {
            instructions: Instructions::new(vec![]),
            constants,
            last_instruction: EmittedInstruction {
                opcode: Opcode::Constant,
                position: 0,
            },
            previous_instruction: EmittedInstruction {
                opcode: Opcode::Constant,
                position: 0,
            },
            symbol_table,
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

                Statement::Let(name, expression) => {
                    self.compile(Node::Expression(expression))?;
                    let symbol = self.symbol_table.borrow_mut().define(name);
                    self.emit(Opcode::SetGlobal, vec![symbol.index]);
                }
                _ => {
                    panic!("not implemented")
                }
            },

            Node::Expression(expression) => match expression {
                Expression::Infix(left, operator, right) => {
                    if operator == Token::Lt {
                        self.compile(Node::Expression(*right))?;
                        self.compile(Node::Expression(*left))?;
                        self.emit(Opcode::GreaterThan, vec![]);
                        return Ok(());
                    }

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

                        Token::Gt | Token::Eq | Token::NotEq => {
                            self.emit(
                                match operator {
                                    Token::Lt => Opcode::GreaterThan,
                                    Token::Gt => Opcode::GreaterThan,
                                    Token::Eq => Opcode::Equal,
                                    Token::NotEq => Opcode::NotEqual,
                                    _ => {
                                        panic!("not implemented")
                                    }
                                },
                                vec![],
                            );
                        }

                        _ => {
                            panic!("not implemented")
                        }
                    }
                }

                Expression::Prefix(operator, expression) => {
                    self.compile(Node::Expression(*expression))?;
                    match operator {
                        Token::Bang => {
                            self.emit(Opcode::Bang, vec![]);
                        }
                        Token::Dash => {
                            self.emit(Opcode::Minus, vec![]);
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

                    Literal::Boolean(value) => {
                        if value {
                            self.emit(Opcode::True, vec![]);
                        } else {
                            self.emit(Opcode::False, vec![]);
                        }
                    }

                    Literal::String(value) => {
                        let string = Rc::new(Object::String(value));
                        let position = self.add_constant(string);
                        _ = self.emit(Opcode::Constant, vec![position]);
                    }

                    Literal::Array(elements) => {
                        for element in elements {
                            self.compile(Node::Expression(element))?;
                        }
                        self.emit(Opcode::Array, vec![elements.len()]);
                    }
                    _ => {
                        panic!("not implemented")
                    }
                },

                Expression::If(condition, consequence, alternative) => {
                    self.compile(Node::Expression(*condition))?;

                    let jump_not_truthy_position = self.emit(Opcode::JumpNotTruthy, vec![9999]);

                    self.compile(Node::Program(consequence))?;

                    // leave last element of consequence on the stack
                    if self.last_instruction_is(Opcode::Pop) {
                        self.remove_last_instruction();
                    }

                    let jump_position = self.emit(Opcode::Jump, vec![9999]);

                    let after_consequence_position = self.instructions.len();
                    self.change_operand(jump_not_truthy_position, after_consequence_position);

                    match alternative {
                        Some(alternative) => {
                            self.compile(Node::Program(alternative))?;

                            if self.last_instruction_is(Opcode::Pop) {
                                self.remove_last_instruction();
                            }
                        }
                        None => {
                            self.emit(Opcode::Null, vec![]);
                        }
                    }

                    let after_alternative_position = self.instructions.len();
                    self.change_operand(jump_position, after_alternative_position);
                }

                Expression::Identifier(name) => {
                    let symbol = self.symbol_table.borrow_mut().resolve(&name);
                    match symbol {
                        Some(symbol) => {
                            self.emit(Opcode::GetGlobal, vec![symbol.index]);
                        }
                        None => {
                            return Err(CompileError::new(format!("undefined variable: {}", name)));
                        }
                    }
                }

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
        self.constants.borrow_mut().push(object);

        self.constants.borrow_mut().len() - 1
    }

    pub fn emit(&mut self, opcode: Opcode, operands: Vec<usize>) -> usize {
        let ins = code::make(opcode, operands);
        let pos = self.add_instructions(ins);
        self.set_last_instruction(opcode, pos);
        pos
    }

    pub fn set_last_instruction(&mut self, opcode: Opcode, position: usize) {
        self.previous_instruction = self.last_instruction.clone();
        self.last_instruction = EmittedInstruction { opcode, position };
    }

    pub fn last_instruction_is(&self, opcode: Opcode) -> bool {
        self.last_instruction.opcode == opcode
    }

    pub fn remove_last_instruction(&mut self) {
        let last = self.last_instruction.position;
        self.instructions = self.instructions.slice(0, last).into();
        self.last_instruction = self.previous_instruction.clone();
    }

    pub fn add_instructions(&mut self, instructions: Vec<u8>) -> usize {
        let position_new = self.instructions.len();
        self.instructions.extend(Instructions::new(instructions));
        position_new
    }

    fn replace_instruction(&mut self, position: usize, new_instructions: Vec<u8>) {
        for i in 0..new_instructions.len() {
            self.instructions[position + i] = new_instructions[i];
        }
    }

    fn change_operand(&mut self, position: usize, operand: usize) {
        let opcode = self.instructions[position];
        let new_instrution = code::make(opcode.into(), vec![operand]);
        self.replace_instruction(position, new_instrution);
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

    fn test_constants(expected: Rc<RefCell<Vec<Rc<Object>>>>, actual: Vec<Rc<Object>>) {
        assert_eq!(expected.borrow().len(), actual.len());
        for (i, constant) in expected.borrow().iter().enumerate() {
            match &**constant {
                Object::Integer(expected) => match &*actual[i] {
                    Object::Integer(actual) => assert_eq!(expected, actual),
                    _ => panic!("constant not integer"),
                },
                Object::String(expected) => match &*actual[i] {
                    Object::String(actual) => assert_eq!(expected, actual),
                    _ => panic!("constant not string"),
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

    #[test]
    fn it_compiles_boolean_expressions() {
        test_compilation(
            "true",
            vec![
                make(Opcode::True, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![],
        );

        test_compilation(
            "false",
            vec![
                make(Opcode::False, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![],
        );
    }

    #[test]
    fn it_compiles_comparison_operations() {
        test_compilation(
            "1 == 1",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Equal, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(1)), Rc::new(Object::Integer(1))],
        );

        test_compilation(
            "1 != 2",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::NotEqual, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(1)), Rc::new(Object::Integer(2))],
        );

        test_compilation(
            "1 > 2",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::GreaterThan, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(1)), Rc::new(Object::Integer(2))],
        );

        test_compilation(
            "1 < 2",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::GreaterThan, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(2)), Rc::new(Object::Integer(1))],
        );

        test_compilation(
            "true == false",
            vec![
                make(Opcode::True, vec![]).into(),
                make(Opcode::False, vec![]).into(),
                make(Opcode::Equal, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![],
        );

        test_compilation(
            "true != false",
            vec![
                make(Opcode::True, vec![]).into(),
                make(Opcode::False, vec![]).into(),
                make(Opcode::NotEqual, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![],
        );
    }

    #[test]
    fn it_compiles_prefix_operators() {
        test_compilation(
            "!true",
            vec![
                make(Opcode::True, vec![]).into(),
                make(Opcode::Bang, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![],
        );

        test_compilation(
            "!false",
            vec![
                make(Opcode::False, vec![]).into(),
                make(Opcode::Bang, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![],
        );

        test_compilation(
            "-1",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Minus, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(1))],
        );
    }

    #[test]
    fn it_compiles_conditionals() {
        test_compilation(
            "if (true) { 10 }; 3333;",
            vec![
                make(Opcode::True, vec![]).into(),
                make(Opcode::JumpNotTruthy, vec![10]).into(),
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Jump, vec![11]).into(),
                make(Opcode::Null, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(10)), Rc::new(Object::Integer(3333))],
        );

        test_compilation(
            "if (true) { 10 } else { 20 }; 3333;",
            vec![
                make(Opcode::True, vec![]).into(),
                make(Opcode::JumpNotTruthy, vec![10]).into(),
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Jump, vec![13]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Pop, vec![]).into(),
                make(Opcode::Constant, vec![2]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::Integer(10)),
                Rc::new(Object::Integer(20)),
                Rc::new(Object::Integer(3333)),
            ],
        );
    }

    #[test]
    fn it_compiles_global_let_statements() {
        test_compilation(
            "let one = 1; let two = 2;",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::SetGlobal, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::SetGlobal, vec![1]).into(),
            ],
            vec![Rc::new(Object::Integer(1)), Rc::new(Object::Integer(2))],
        );

        test_compilation(
            "let one = 1; one;",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::SetGlobal, vec![0]).into(),
                make(Opcode::GetGlobal, vec![0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(1))],
        );

        test_compilation(
            "let one = 1; let two = one; two;",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::SetGlobal, vec![0]).into(),
                make(Opcode::GetGlobal, vec![0]).into(),
                make(Opcode::SetGlobal, vec![1]).into(),
                make(Opcode::GetGlobal, vec![1]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(1))],
        );
    }

    #[test]
    fn it_compiles_string_expressions() {
        test_compilation(
            r#""monkey""#,
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::String("monkey".to_string()))],
        );

        test_compilation(
            r#""mon" + "key""#,
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Add, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::String("mon".to_string())),
                Rc::new(Object::String("key".to_string())),
            ],
        );
    }

    #[test]
    fn it_compiles_arrays() {
        test_compilation(
            "[]",
            vec![
                make(Opcode::Array, vec![0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![],
        );

        test_compilation(
            "[1, 2, 3]",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Constant, vec![2]).into(),
                make(Opcode::Array, vec![3]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::Integer(1)),
                Rc::new(Object::Integer(2)),
                Rc::new(Object::Integer(3)),
            ],
        );

        test_compilation(
            "[1 + 2, 3 - 4, 5 * 6]",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Add, vec![]).into(),
                make(Opcode::Constant, vec![2]).into(),
                make(Opcode::Constant, vec![3]).into(),
                make(Opcode::Sub, vec![]).into(),
                make(Opcode::Constant, vec![4]).into(),
                make(Opcode::Constant, vec![5]).into(),
                make(Opcode::Mul, vec![]).into(),
                make(Opcode::Array, vec![3]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::Integer(1)),
                Rc::new(Object::Integer(2)),
                Rc::new(Object::Integer(3)),
                Rc::new(Object::Integer(4)),
                Rc::new(Object::Integer(5)),
                Rc::new(Object::Integer(6)),
            ],
        );
    }
}
