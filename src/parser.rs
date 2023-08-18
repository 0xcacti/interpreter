pub mod ast;
pub mod errors;
pub mod precedence;

use crate::lexer::Lexer;
use crate::parser::ast::*;
use crate::parser::errors::*;
use crate::parser::precedence::*;
use crate::token::Token;
use anyhow::Result;

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
    errors: ParserErrors,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Parser {
            lexer,
            current_token,
            peek_token,
            errors: Vec::new(),
        }
    }

    fn next_token(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    pub fn parse_program(&mut self) -> Result<Vec<Statement>> {
        let mut program = Vec::new();

        while !self.current_token_is(&Token::Eof) {
            match self.parse_statement() {
                Ok(statement) => program.push(statement),
                Err(e) => self.errors.push(e),
            }
            self.next_token();
        }

        Ok(program)
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match self.current_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, ParserError> {
        let ident = match &self.peek_token {
            Token::Ident(ref id) => id.clone(),
            t => {
                return Err(ParserError::new(format!(
                    "parse error: expected identifier, got {:?}",
                    t
                )));
            }
        };

        self.next_token();
        self.expect_peek_token(&Token::Assign)?;
        self.next_token();

        // skip expression for now
        while self.current_token != Token::Semicolon {
            self.next_token();
        }

        Ok(Statement::Let(ident, Expression::Literal("".to_string())))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParserError> {
        // we want to kill the return token
        self.next_token();
        while self.current_token != Token::Semicolon {
            self.next_token();
        }
        Ok(Statement::Return(Expression::Literal("".to_string())))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParserError> {
        let expression_statement = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(&Token::Semicolon) {
            self.next_token();
        }
        Ok(expression_statement)
    }

    fn peek_token_is(&self, token: &Token) -> bool {
        self.peek_token == *token
    }

    fn current_token_is(&self, token: &Token) -> bool {
        self.current_token == *token
    }

    fn expect_peek_token(&mut self, token: &Token) -> Result<(), ParserError> {
        if self.peek_token_is(&token) {
            self.next_token();
            Ok(())
        } else {
            Err(ParserError::new(format!(
                "parse error: expected {:?}, got {:?}",
                token, self.peek_token
            )))
        }
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

    #[test]
    fn it_parses_return_statements() {
        let input = r#"
        return 5;
        return 10;
        return 993322;
        "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 3);
        check_return_statement(&program[0]);
        check_return_statement(&program[1]);
        check_return_statement(&program[2]);
    }

    #[test]
    fn it_parses_identifier_expressions() {
        let input = r#"
        foobar;
        "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 1);
        match &program[0] {
            Statement::Expression(Expression::Identifier(ref ident)) => assert_eq!(ident, "foobar"),
            _ => panic!("expected identifier expression"),
        }
    }

    fn check_let_statement(s: &Statement, name: &str) {
        match s {
            Statement::Let(ref ident, _) => assert_eq!(ident, name),
            _ => panic!("expected let statement"),
        }
    }

    fn check_return_statement(s: &Statement) {
        match s {
            Statement::Return(_) => (),
            _ => panic!("expected return statement"),
        }
    }
}
