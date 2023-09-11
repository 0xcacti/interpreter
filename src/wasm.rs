use crate::evaluator::environment::Environment;
use crate::evaluator::{define_macros, evaluate, expand_macros};
use wasm_bindgen::prelude::*;

use crate::lexer::Lexer;
use crate::parser::ast::Node;
use crate::parser::Parser;
use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
}; // <-- Add this import for flushing stdout
use users::get_current_username;

const PROMPT: &str = ">> ";

#[wasm_bindgen]
pub fn interpret(input: &str) -> String {
    let env = Rc::new(RefCell::new(Environment::new()));
    let macro_env = Rc::new(RefCell::new(Environment::new()));

    let lexer = Lexer::new(&input);
    let mut parser = Parser::new(lexer.into());
    let program = parser.parse_program();

    if let Ok(mut program) = program {
        define_macros(&mut program, Rc::clone(&macro_env));
        let expanded = expand_macros(Node::Program(program.clone()), Rc::clone(&macro_env));
        println!("{}", evaluate(expanded, Rc::clone(&env))?);
    } else if let Err(err) = &program {
        println!("Woops! We ran into some monkey business here!");
        println!("parser errors:");
        for e in err {
            eprintln!("\t{}", e);
        }
    }
}
