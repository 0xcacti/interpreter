pub mod ast;
use crate::lexer::Lexer;
use crate::parser::ast::*;
use crate::token::Token;
use anyhow::Result;

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Parser {
            lexer,
            current_token,
            peek_token,
        }
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut program = Vec::new();

        while self.current_token != Token::Eof {
            match self.parse_statement() {
                Ok(statement) => program.push(statement),
                Err(e) => panic!("parse error: {}", e),
            }
            self.next_token();
        }

        Ok(program)
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.current_token {
            Token::Let => self.parse_let_statement(),
            _ => Err(anyhow::anyhow!("parse error: expected let statement")),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement> {
        let ident = match &self.peek_token {
            Token::Ident(ref id) => id.clone(),
            t => {
                panic!("parse error: expected identifier, got {:?}", t);
            }
        };

        self.next_token();
        if self.peek_token != Token::Assign {
            panic!("parse error: expected assign, got {:?}", self.peek_token);
        }

        self.next_token();

        while self.current_token != Token::Semicolon {
            self.next_token();
        }

        Ok(Statement::Let(ident, Expression::Literal("".to_string())))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn it_pareses_let_statements() {
        let input = r#"
    let x = 5;
    let y = 10;
    let foobar = 838383;
    "#;

        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 3);
        check_let_statement(&program[0], "x");
        check_let_statement(&program[1], "y");
        check_let_statement(&program[2], "foobar");
    }

    fn check_let_statement(s: &Statement, name: &str) {
        match s {
            Statement::Let(ref ident, _) => assert_eq!(ident, name),
            _ => panic!("expected let statement"),
        }
    }
}
