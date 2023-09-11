use anyhow::Result;

use interpreter::evaluator::environment::Environment;
use interpreter::evaluator::{define_macros, evaluate, expand_macros};
use wasm_bindgen::prelude::*;

use interpreter::lexer::Lexer;
use interpreter::parser::ast::Node;
use interpreter::parser::Parser;
use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
}; // <-- Add this import for flushing stdout
use users::get_current_username;

const PROMPT: &str = ">> ";

fn main() -> Result<()> {
    let env = Rc::new(RefCell::new(Environment::new()));
    let macro_env = Rc::new(RefCell::new(Environment::new()));
    println!(
        "Dear {}, Welcome to the Mokey Programming Language REPL!",
        get_current_username().unwrap().to_string_lossy()
    );

    loop {
        print!("{}", PROMPT);
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        if line.trim() == "exit" {
            std::process::exit(0);
        }

        let lexer = Lexer::new(&line);
        let mut parser = Parser::new(lexer.into());
        let mut program = parser.parse_program();

        if let Ok(mut program) = program {
            define_macros(&mut program, Rc::clone(&macro_env));
            let expanded =
                expand_macros(Node::Program(program.clone()), Rc::clone(&macro_env)).unwrap();
            println!("{}", evaluate(expanded, Rc::clone(&env))?);
        } else if let Err(err) = &program {
            println!("Woops! We ran into some monkey business here!");
            println!("parser errors:");
            for e in err {
                eprintln!("\t{}", e);
            }
        }
    }
}
