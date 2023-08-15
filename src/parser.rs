pub mod ast;

use crate::lexer::Lexer;
use crate::parser::ast::*;
use crate::token::Token;

pub struct Parser {
    lexer: Lexer,
    cur_token: Option<Token>,
    peek_token: Option<Token>,
}

impl Parser {
    pub fn new(l: Lexer) -> Self {
        let mut parser = Parser {
            lexer: l,
            cur_token: None,
            peek_token: None,
        };

        parser.next_token();
        parser.next_token();
        parser
    }

    pub fn cur_token(&self) -> Option<&Token> {
        self.cur_token.as_ref()
    }

    pub fn peek_token(&self) -> Option<&Token> {
        self.peek_token.as_ref()
    }

    pub fn next_token(&mut self) {
        self.cur_token = self.peek_token.take();
        self.peek_token = Some(self.lexer.next_token());
        // exist
    }

    pub fn parse_program(&mut self) -> Option<Program> {
        let mut program = Program::new();
        while self.cur_token != Some(Token::Eof) {
            if let Some(stmt) = self.parse_statement() {
                program.push(stmt);
            }
            self.next_token();
        }
        Some(program)
    }

    pub fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token {
            Some(Token::Let) => self.parse_let_statement(),
            _ => None,
        }
    }

    pub fn parse_let_statement(&mut self) -> Option<Statement> {
        if let Token::Ident(_) = self.peek_token().unwrap() {
            self.next_token();
            self.expect_peek(Token::Assign)?;
            self.next_token();

            let statement = Statement::LetStatement(
                Identifier::Identifier("=".to_string()),
                self.parse_expression()?,
            );
            if self.peek_token == Some(Token::Semicolon) {
                self.next_token();
            }
            Some(statement)
        } else {
            None
        }
    }

    pub fn expect_peek(&mut self, t: Token) -> Option<()> {
        if self.peek_token == Some(t) {
            self.next_token();
            Some(())
        } else {
            None
        }
    }

    pub fn parse_expression(&mut self) -> Option<Expression> {
        None
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
            Statement::LetStatement(ref ident, _) => match ident {
                Identifier::Identifier(ref ident) => assert_eq!(ident, name),
            },
            _ => panic!("expected let statement"),
        }
    }
}
