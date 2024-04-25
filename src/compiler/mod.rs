pub mod error;
pub mod symbol_table;
use crate::{
    code::{self, Instructions, Opcode},
    object::{builtin::Builtin, CompiledFunction, Object},
    parser::ast::{Expression, Literal, Node, Statement},
    token::Token,
};
use error::CompileError;

use std::{cell::RefCell, rc::Rc};

use self::symbol_table::{Scope, SymbolTable};

pub struct Compiler {
    pub constants: Rc<RefCell<Vec<Rc<Object>>>>,
    pub symbol_table: Rc<RefCell<SymbolTable>>,
    pub scopes: Vec<CompilationScope>,
    pub scope_index: usize,
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

#[derive(Clone)]
pub struct CompilationScope {
    pub instructions: Instructions,
    pub last_instruction: EmittedInstruction,
    pub previous_instruction: EmittedInstruction,
}

impl Compiler {
    pub fn new() -> Self {
        let global_table = SymbolTable::new();

        for (i, builtin) in Builtin::variants().iter().enumerate() {
            global_table
                .borrow_mut()
                .define_builtin(i, builtin.to_string());
        }

        let main_scope = CompilationScope {
            instructions: Instructions::new(vec![]),
            last_instruction: EmittedInstruction {
                opcode: Opcode::Constant,
                position: 0,
            },
            previous_instruction: EmittedInstruction {
                opcode: Opcode::Constant,
                position: 0,
            },
        };

        Compiler {
            constants: Rc::new(RefCell::new(vec![])),
            symbol_table: global_table,
            scopes: vec![main_scope],
            scope_index: 0,
        }
    }

    pub fn new_with_state(
        symbol_table: Rc<RefCell<SymbolTable>>,
        constants: Rc<RefCell<Vec<Rc<Object>>>>,
    ) -> Self {
        let global_table = SymbolTable::new();

        for (i, builtin) in Builtin::variants().iter().enumerate() {
            global_table
                .borrow_mut()
                .define_builtin(i, builtin.to_string());
        }

        let main_scope = CompilationScope {
            instructions: Instructions::new(vec![]),
            last_instruction: EmittedInstruction {
                opcode: Opcode::Constant,
                position: 0,
            },
            previous_instruction: EmittedInstruction {
                opcode: Opcode::Constant,
                position: 0,
            },
        };

        Compiler {
            constants,
            symbol_table,
            scopes: vec![main_scope],
            scope_index: 0,
        }
    }

    fn current_instructions(&self) -> &code::Instructions {
        &self.scopes[self.scope_index].instructions
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
                    match symbol.scope {
                        Scope::Global => {
                            self.emit(Opcode::SetGlobal, vec![symbol.index]);
                        }
                        Scope::Local => {
                            self.emit(Opcode::SetLocal, vec![symbol.index]);
                        }
                        Scope::Builtin => {
                            // TODO: is this right
                            return Err(CompileError::new("cannot assign to builtin".to_string()));
                        }
                    }
                }

                Statement::Return(expression) => {
                    self.compile(Node::Expression(expression))?;

                    self.emit(Opcode::ReturnValue, vec![]);
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
                        for element in elements.clone().iter() {
                            self.compile(Node::Expression(element.clone()))?;
                        }
                        self.emit(Opcode::Array, vec![elements.len()]);
                    }

                    Literal::Hash(pairs) => {
                        let mut key_value_pairs = vec![];

                        for k in &pairs {
                            key_value_pairs.push(k);
                        }

                        key_value_pairs.sort_by(|a, b| a.0.cmp(&b.0));

                        for k in key_value_pairs {
                            self.compile(Node::Expression(k.0.clone()))?;
                            self.compile(Node::Expression(k.1.clone()))?;
                        }

                        self.emit(Opcode::Hash, vec![pairs.len() * 2]);
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

                    let after_consequence_position = self.current_instructions().len();
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

                    let after_alternative_position = self.current_instructions().len();
                    self.change_operand(jump_position, after_alternative_position);
                }

                Expression::Identifier(name) => {
                    let symbol = self.symbol_table.borrow_mut().resolve(&name);
                    match symbol {
                        Some(symbol) => match symbol.scope {
                            Scope::Global => {
                                self.emit(Opcode::GetGlobal, vec![symbol.index]);
                            }
                            Scope::Local => {
                                self.emit(Opcode::GetLocal, vec![symbol.index]);
                            }
                            Scope::Builtin => {
                                self.emit(Opcode::GetBuiltin, vec![symbol.index]);
                            }
                        },
                        None => {
                            return Err(CompileError::new(format!("undefined variable: {}", name)));
                        }
                    }
                }

                Expression::Index(indexable, index) => {
                    self.compile(Node::Expression(*indexable))?;
                    self.compile(Node::Expression(*index))?;
                    self.emit(Opcode::Index, vec![]);
                }

                Expression::Function(parameters, body) => {
                    self.enter_scope();

                    let num_params = parameters.len();
                    for parameter in parameters {
                        self.symbol_table.borrow_mut().define(parameter);
                    }

                    self.compile(Node::Program(body))?;

                    if self.last_instruction_is(Opcode::Pop) {
                        self.replace_instruction(
                            self.scopes[self.scope_index].last_instruction.position,
                            code::make(Opcode::ReturnValue, vec![]),
                        );
                        let current_scope = &mut self.scopes[self.scope_index];
                        current_scope.last_instruction.opcode = Opcode::ReturnValue;
                    }

                    if !self.last_instruction_is(Opcode::ReturnValue) {
                        self.emit(Opcode::Return, vec![]);
                    }

                    let num_locals = self.symbol_table.borrow().num_definitions;
                    let fn_instructions = self.leave_scope();
                    let compiled_fn = Rc::new(Object::CompiledFunction(Rc::new(
                        CompiledFunction::new(fn_instructions, num_locals, num_params),
                    )));

                    let constant_index = self.add_constant(compiled_fn);

                    self.emit(Opcode::Closure, vec![constant_index, 0]);
                }

                Expression::FunctionCall(function, arguments) => {
                    self.compile(Node::Expression(*function))?;
                    let len = arguments.len();
                    for argument in arguments {
                        self.compile(Node::Expression(argument))?;
                    }
                    self.emit(Opcode::Call, vec![len]);
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
            instructions: self.current_instructions().clone(),
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
        let current_scope = &mut self.scopes[self.scope_index];
        current_scope.previous_instruction = current_scope.last_instruction.clone();
        current_scope.last_instruction = EmittedInstruction { opcode, position };
    }

    pub fn last_instruction_is(&self, opcode: Opcode) -> bool {
        let current_scope = &self.scopes[self.scope_index];
        current_scope.last_instruction.opcode == opcode
    }

    pub fn remove_last_instruction(&mut self) {
        let current_scope = &mut self.scopes[self.scope_index];
        let last = current_scope.last_instruction.position;
        current_scope.instructions = current_scope.instructions.slice(0, last).into();
        current_scope.last_instruction = current_scope.previous_instruction.clone();
    }

    pub fn add_instructions(&mut self, instructions: Vec<u8>) -> usize {
        let position_new = self.current_instructions().len();
        let current_scope = &mut self.scopes[self.scope_index];
        current_scope
            .instructions
            .extend(Instructions::new(instructions));
        position_new
    }

    fn replace_instruction(&mut self, position: usize, new_instructions: Vec<u8>) {
        let current_scope = &mut self.scopes[self.scope_index];
        for i in 0..new_instructions.len() {
            current_scope.instructions[position + i] = new_instructions[i];
        }
    }

    fn change_operand(&mut self, position: usize, operand: usize) {
        let current_scope = &mut self.scopes[self.scope_index];
        let opcode = current_scope.instructions[position];
        let new_instrution = code::make(opcode.into(), vec![operand]);
        self.replace_instruction(position, new_instrution);
    }

    fn enter_scope(&mut self) {
        self.scopes.push(CompilationScope {
            instructions: Instructions::new(vec![]),
            last_instruction: EmittedInstruction {
                opcode: Opcode::Constant,
                position: 0,
            },
            previous_instruction: EmittedInstruction {
                opcode: Opcode::Constant,
                position: 0,
            },
        });
        self.scope_index += 1;
        let symbol_table = SymbolTable::new_enclosed(self.symbol_table.clone());
        self.symbol_table = symbol_table;
    }

    fn leave_scope(&mut self) -> Instructions {
        let instructions = self.current_instructions().to_owned();
        self.scopes.pop();
        self.scope_index -= 1;
        let temp_symbol_table = std::mem::replace(&mut self.symbol_table, SymbolTable::new());

        let outer_symbol_table = temp_symbol_table.borrow().outer.clone();

        match outer_symbol_table {
            Some(outer) => {
                self.symbol_table = outer.clone();
            }
            None => {
                panic!("tried to leave scope without outer symbol table")
            }
        }

        instructions
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
                Object::CompiledFunction(compiled_function) => match &*actual[i] {
                    Object::CompiledFunction(actual_compiled_function) => {
                        assert_eq!(compiled_function, actual_compiled_function)
                    }
                    _ => panic!("constant not a compiled function"),
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

    #[test]
    fn it_compiles_hash_expressions() {
        test_compilation(
            "{}",
            vec![
                make(Opcode::Hash, vec![0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![],
        );

        test_compilation(
            "{1: 2, 3: 4, 5: 6}",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Constant, vec![2]).into(),
                make(Opcode::Constant, vec![3]).into(),
                make(Opcode::Constant, vec![4]).into(),
                make(Opcode::Constant, vec![5]).into(),
                make(Opcode::Hash, vec![6]).into(),
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

        test_compilation(
            "{1: 2 + 3, 4: 5 * 6}",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Constant, vec![2]).into(),
                make(Opcode::Add, vec![]).into(),
                make(Opcode::Constant, vec![3]).into(),
                make(Opcode::Constant, vec![4]).into(),
                make(Opcode::Constant, vec![5]).into(),
                make(Opcode::Mul, vec![]).into(),
                make(Opcode::Hash, vec![4]).into(),
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

    #[test]
    fn it_compiles_indexing_operations() {
        test_compilation(
            "[1, 2, 3][1 + 1]",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Constant, vec![2]).into(),
                make(Opcode::Array, vec![3]).into(),
                make(Opcode::Constant, vec![3]).into(),
                make(Opcode::Constant, vec![4]).into(),
                make(Opcode::Add, vec![]).into(),
                make(Opcode::Index, vec![]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::Integer(1)),
                Rc::new(Object::Integer(2)),
                Rc::new(Object::Integer(3)),
                Rc::new(Object::Integer(1)),
                Rc::new(Object::Integer(1)),
            ],
        );
    }

    #[test]
    fn it_compiles_function_literals() {
        test_compilation(
            "fn() { return 5 + 10 }",
            vec![
                make(Opcode::Closure, vec![2, 0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::Integer(5)),
                Rc::new(Object::Integer(10)),
                Rc::new(Object::CompiledFunction(Rc::new(CompiledFunction::new(
                    concatenate_instructions(&vec![
                        make(Opcode::Constant, vec![0]).into(),
                        make(Opcode::Constant, vec![1]).into(),
                        make(Opcode::Add, vec![]).into(),
                        make(Opcode::ReturnValue, vec![]).into(),
                    ]),
                    0,
                    0,
                )))),
            ],
        );

        test_compilation(
            "fn() { 1; 2 }",
            vec![
                make(Opcode::Closure, vec![2, 0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::Integer(1)),
                Rc::new(Object::Integer(2)),
                Rc::new(Object::CompiledFunction(Rc::new(CompiledFunction::new(
                    concatenate_instructions(&vec![
                        make(Opcode::Constant, vec![0]).into(),
                        make(Opcode::Pop, vec![]).into(),
                        make(Opcode::Constant, vec![1]).into(),
                        make(Opcode::ReturnValue, vec![]).into(),
                    ]),
                    0,
                    0,
                )))),
            ],
        );

        test_compilation(
            "fn() { 5 + 10 }",
            vec![
                make(Opcode::Closure, vec![2, 0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::Integer(5)),
                Rc::new(Object::Integer(10)),
                Rc::new(Object::CompiledFunction(Rc::new(CompiledFunction::new(
                    concatenate_instructions(&vec![
                        make(Opcode::Constant, vec![0]).into(),
                        make(Opcode::Constant, vec![1]).into(),
                        make(Opcode::Add, vec![]).into(),
                        make(Opcode::ReturnValue, vec![]).into(),
                    ]),
                    0,
                    0,
                )))),
            ],
        );

        test_compilation(
            "fn() { }",
            vec![
                make(Opcode::Closure, vec![0, 0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::CompiledFunction(Rc::new(
                CompiledFunction::new(
                    concatenate_instructions(&vec![make(Opcode::Return, vec![]).into()]),
                    0,
                    0,
                ),
            )))],
        );
    }

    #[test]
    fn it_compiles_function_calls() {
        test_compilation(
            "fn() { 24 }();",
            vec![
                make(Opcode::Closure, vec![1, 0]).into(),
                make(Opcode::Call, vec![0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::Integer(24)),
                Rc::new(Object::CompiledFunction(Rc::new(CompiledFunction::new(
                    concatenate_instructions(&vec![
                        make(Opcode::Constant, vec![0]).into(),
                        make(Opcode::ReturnValue, vec![]).into(),
                    ]),
                    0,
                    0,
                )))),
            ],
        );

        test_compilation(
            "let noArg = fn() { 24 }; noArg();",
            vec![
                make(Opcode::Closure, vec![1, 0]).into(),
                make(Opcode::SetGlobal, vec![0]).into(),
                make(Opcode::GetGlobal, vec![0]).into(),
                make(Opcode::Call, vec![0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::Integer(24)),
                Rc::new(Object::CompiledFunction(Rc::new(CompiledFunction::new(
                    concatenate_instructions(&vec![
                        make(Opcode::Constant, vec![0]).into(),
                        make(Opcode::ReturnValue, vec![]).into(),
                    ]),
                    0,
                    0,
                )))),
            ],
        );

        test_compilation(
            "let oneArg = fn(a) { a }; oneArg(24);",
            vec![
                make(Opcode::Closure, vec![0, 0]).into(),
                make(Opcode::SetGlobal, vec![0]).into(),
                make(Opcode::GetGlobal, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Call, vec![1]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::CompiledFunction(Rc::new(CompiledFunction::new(
                    concatenate_instructions(&vec![
                        make(Opcode::GetLocal, vec![0]).into(),
                        make(Opcode::ReturnValue, vec![]).into(),
                    ]),
                    1,
                    1,
                )))),
                Rc::new(Object::Integer(24)),
            ],
        );

        test_compilation(
            "let manyArg = fn(a, b, c) { a; b; c }; manyArg(24, 25, 26);",
            vec![
                make(Opcode::Closure, vec![0, 0]).into(),
                make(Opcode::SetGlobal, vec![0]).into(),
                make(Opcode::GetGlobal, vec![0]).into(),
                make(Opcode::Constant, vec![1]).into(),
                make(Opcode::Constant, vec![2]).into(),
                make(Opcode::Constant, vec![3]).into(),
                make(Opcode::Call, vec![3]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::CompiledFunction(Rc::new(CompiledFunction::new(
                    concatenate_instructions(&vec![
                        make(Opcode::GetLocal, vec![0]).into(),
                        make(Opcode::Pop, vec![]).into(),
                        make(Opcode::GetLocal, vec![1]).into(),
                        make(Opcode::Pop, vec![]).into(),
                        make(Opcode::GetLocal, vec![2]).into(),
                        make(Opcode::ReturnValue, vec![]).into(),
                    ]),
                    3,
                    3,
                )))),
                Rc::new(Object::Integer(24)),
                Rc::new(Object::Integer(25)),
                Rc::new(Object::Integer(26)),
            ],
        );
    }

    #[test]
    fn it_handles_scopes_correctly() {
        let mut compiler = Compiler::new();
        assert_eq!(compiler.scope_index, 0);

        let global_symbol_table = compiler.symbol_table.clone();
        compiler.emit(Opcode::Mul, vec![]);

        compiler.enter_scope();

        assert_eq!(compiler.scope_index, 1);

        compiler.emit(Opcode::Sub, vec![]);

        assert_eq!(compiler.scopes[compiler.scope_index].instructions.len(), 1);

        let last = compiler.scopes[compiler.scope_index]
            .last_instruction
            .clone();
        assert_eq!(last.opcode, Opcode::Sub);

        assert_eq!(
            compiler.symbol_table.borrow().outer,
            Some(global_symbol_table.clone())
        );

        compiler.leave_scope();

        assert_eq!(compiler.scope_index, 0);

        assert_eq!(
            *compiler.symbol_table.borrow(),
            *global_symbol_table.borrow()
        );
        assert_eq!(compiler.symbol_table.borrow().outer, None);

        compiler.emit(Opcode::Add, vec![]);

        assert_eq!(compiler.scopes[compiler.scope_index].instructions.len(), 2);

        let last = compiler.scopes[compiler.scope_index]
            .last_instruction
            .clone();

        assert_eq!(last.opcode, Opcode::Add);

        let previous = compiler.scopes[compiler.scope_index]
            .previous_instruction
            .clone();

        assert_eq!(previous.opcode, Opcode::Mul);
    }

    #[test]
    fn it_compiles_with_scope() {
        test_compilation(
            "let num = 55; fn() { num }",
            vec![
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::SetGlobal, vec![0]).into(),
                make(Opcode::Closure, vec![1, 0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::Integer(55)),
                Rc::new(Object::CompiledFunction(Rc::new(CompiledFunction::new(
                    concatenate_instructions(&vec![
                        make(Opcode::GetGlobal, vec![0]).into(),
                        make(Opcode::ReturnValue, vec![]).into(),
                    ]),
                    0,
                    0,
                )))),
            ],
        );

        test_compilation(
            "fn() { let num = 55; num }",
            vec![
                make(Opcode::Closure, vec![1, 0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::Integer(55)),
                Rc::new(Object::CompiledFunction(Rc::new(CompiledFunction::new(
                    concatenate_instructions(&vec![
                        make(Opcode::Constant, vec![0]).into(),
                        make(Opcode::SetLocal, vec![0]).into(),
                        make(Opcode::GetLocal, vec![0]).into(),
                        make(Opcode::ReturnValue, vec![]).into(),
                    ]),
                    1,
                    0,
                )))),
            ],
        );

        test_compilation(
            "fn() { let a = 55; let b = 77; a + b }",
            vec![
                make(Opcode::Closure, vec![2, 0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![
                Rc::new(Object::Integer(55)),
                Rc::new(Object::Integer(77)),
                Rc::new(Object::CompiledFunction(Rc::new(CompiledFunction::new(
                    concatenate_instructions(&vec![
                        make(Opcode::Constant, vec![0]).into(),
                        make(Opcode::SetLocal, vec![0]).into(),
                        make(Opcode::Constant, vec![1]).into(),
                        make(Opcode::SetLocal, vec![1]).into(),
                        make(Opcode::GetLocal, vec![0]).into(),
                        make(Opcode::GetLocal, vec![1]).into(),
                        make(Opcode::Add, vec![]).into(),
                        make(Opcode::ReturnValue, vec![]).into(),
                    ]),
                    2,
                    0,
                )))),
            ],
        );
    }

    #[test]
    fn it_compiles_builtins() {
        test_compilation(
            "len([]); push([], 1);",
            vec![
                make(Opcode::GetBuiltin, vec![0]).into(),
                make(Opcode::Array, vec![0]).into(),
                make(Opcode::Call, vec![1]).into(),
                make(Opcode::Pop, vec![]).into(),
                make(Opcode::GetBuiltin, vec![4]).into(),
                make(Opcode::Array, vec![0]).into(),
                make(Opcode::Constant, vec![0]).into(),
                make(Opcode::Call, vec![2]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::Integer(1))],
        );

        test_compilation(
            "fn() { len([]) }",
            vec![
                make(Opcode::Closure, vec![0, 0]).into(),
                make(Opcode::Pop, vec![]).into(),
            ],
            vec![Rc::new(Object::CompiledFunction(Rc::new(
                CompiledFunction::new(
                    concatenate_instructions(&vec![
                        make(Opcode::GetBuiltin, vec![0]).into(),
                        make(Opcode::Array, vec![0]).into(),
                        make(Opcode::Call, vec![1]).into(),
                        make(Opcode::ReturnValue, vec![]).into(),
                    ]),
                    0,
                    0,
                ),
            )))],
        );
    }
}
