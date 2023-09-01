use anyhow::Result;
use interpreter::lexer::Lexer;
use interpreter::parser::ast::Node;
use interpreter::parser::Parser;
use std::io::{self, Write}; // <-- Add this import for flushing stdout
use users::get_current_username;

const PROMPT: &str = ">> ";

fn main() -> Result<()> {
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
        let program = parser.parse_program();

        match program {
            Ok(program) => {
                println!("{}", Node::Program(program));
            }
            Err(err) => {
                println!("Woops! We ran into some monkey business here!");
                println!("parser errors:");
                for e in err {
                    eprintln!("\t{}", e);
                }
            }
        }
    }
}
