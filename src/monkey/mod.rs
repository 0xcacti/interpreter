use anyhow::Result;

use crate::evaluator::environment::Environment;
use crate::evaluator::{define_macros, evaluate, expand_macros};
use crate::utils;
use wasm_bindgen::prelude::*;

use crate::lexer::Lexer;
use crate::parser::ast::Node;
use crate::parser::Parser;
use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
}; // <-- Add this import for flushing stdout
   // use users::get_current_username;

const PROMPT: &str = ">> ";

pub fn repl(path: Option<String>) -> Result<()> {
    let env = Rc::new(RefCell::new(Environment::new()));
    let macro_env = Rc::new(RefCell::new(Environment::new()));
    println!("Welcome to the Mokey Programming Language REPL!",);
    if let Some(path) = path {
        let contents = utils::load_monkey(path)?;
        interpret_chunk(contents, Some(Rc::clone(&env)), Some(Rc::clone(&macro_env)))?;
    }

    loop {
        print!("{}", PROMPT);
        io::stdout().flush()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        if line.trim() == "exit" {
            std::process::exit(0);
        }

        interpret_chunk(line, Some(Rc::clone(&env)), Some(Rc::clone(&macro_env)))?;
    }
}

pub fn interpret_chunk(
    contents: String,
    env: Option<Rc<RefCell<Environment>>>,
    macro_env: Option<Rc<RefCell<Environment>>>,
) -> Result<()> {
    let env = env.unwrap_or_else(|| Rc::new(RefCell::new(Environment::new())));
    let macro_env = macro_env.unwrap_or_else(|| Rc::new(RefCell::new(Environment::new())));

    let lexer = Lexer::new(&contents);
    let mut parser = Parser::new(lexer.into());
    let program = parser.parse_program();
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
    Ok(())
}
