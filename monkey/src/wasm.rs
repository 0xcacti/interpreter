use crate::evaluator::{define_macros, evaluate, expand_macros};
use crate::object::environment::Environment;
use wasm_bindgen::prelude::*;

use crate::lexer::Lexer;
use crate::parser::ast::Node;
use crate::parser::Parser;
use std::cell::RefCell;
use std::rc::Rc;

#[wasm_bindgen]
pub fn interpret(input: &str) -> String {
    let env = Rc::new(RefCell::new(Environment::new()));
    let macro_env = Rc::new(RefCell::new(Environment::new()));

    let lexer = Lexer::new(&input);
    let mut parser = Parser::new(lexer.into());
    let program = parser.parse_program();

    match program {
        Ok(mut program) => {
            define_macros(&mut program, Rc::clone(&macro_env));
            let expanded = expand_macros(Node::Program(program.clone()), Rc::clone(&macro_env));

            match expanded {
                Ok(expanded) => {
                    // Note: You may want to return the result of evaluation. Assuming `evaluate` returns a Result<String, SomeError>:
                    match evaluate(expanded, Rc::clone(&env)) {
                        Ok(result) => return result.to_string(),
                        Err(err) => return format!("Evaluation error: {:?}", err),
                    }
                }
                Err(err) => return format!("macro expansion error: {:?}", err),
            }
        }
        Err(err) => {
            let mut error_msg =
                String::from("Woops! We ran into some monkey business here!\nparser errors:\n");
            for e in err {
                error_msg.push_str(&format!("\t{}\n", e));
            }
            error_msg
        }
    }
}
