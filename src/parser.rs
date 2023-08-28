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

        Ok(Statement::Let(
            ident,
            Expression::Literal(Literal::String("".to_string())),
        ))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParserError> {
        // we want to kill the return token
        self.next_token();
        while self.current_token != Token::Semicolon {
            self.next_token();
        }
        Ok(Statement::Return(Expression::Literal(Literal::String(
            "".to_string(),
        ))))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParserError> {
        let expression_statement = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(&Token::Semicolon) {
            self.next_token();
        }
        Ok(Statement::Expression(expression_statement))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, ParserError> {
        let mut exp = match self.current_token {
            Token::Ident(ref ident) => Expression::Identifier(ident.clone()),
            Token::Int(i) => {
                println!("parse_expression: {:?}", i);
                Expression::Literal(Literal::Integer(i))
            }
            Token::True => Expression::Literal(Literal::Boolean(true)),
            Token::False => Expression::Literal(Literal::Boolean(false)),
            Token::Bang | Token::Dash => self.parse_prefix_expression()?, // is there a better way
            Token::Lparen => {
                self.next_token();
                let exp = self.parse_expression(Precedence::Lowest)?;
                self.expect_peek_token(&Token::Rparen)?;
                exp
            }
            _ => {
                return Err(ParserError::new(format!(
                    "parse error: no prefix parse function for {} found",
                    self.current_token
                )))
            }
        };

        while !self.peek_token_is(&Token::Semicolon) && precedence < self.peek_precedence() {
            match self.peek_token {
                Token::Plus
                | Token::Dash
                | Token::Slash
                | Token::Asterisk
                | Token::Eq
                | Token::NotEq
                | Token::Lt
                | Token::Gt => {
                    self.next_token();
                    exp = self.parse_infix_expression(exp)?;
                }
                _ => break,
            }
        }

        Ok(exp)
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression, ParserError> {
        let prefix = self.current_token.clone();
        println!("parse_prefix_expression: {:?}", prefix);
        self.next_token();
        let exp = self.parse_expression(Precedence::Prefix)?;
        Ok(Expression::Prefix(prefix, Box::new(exp)))
    }

    fn parse_infix_expression(&mut self, left_exp: Expression) -> Result<Expression, ParserError> {
        let infix = self.current_token.clone();
        let precedence = token_precedence(&infix);
        self.next_token();
        let right_exp = self.parse_expression(precedence)?;
        Ok(Expression::Infix(
            Box::new(left_exp),
            infix,
            Box::new(right_exp),
        ))
    }

    fn peek_token_is(&self, token: &Token) -> bool {
        self.peek_token == *token
    }

    fn current_token_is(&self, token: &Token) -> bool {
        self.current_token == *token
    }

    fn peek_precedence(&self) -> Precedence {
        token_precedence(&self.peek_token)
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
        check_expression_statement(&program[0], &Expression::Identifier("foobar".to_string()));
    }

    #[test]
    fn it_parses_integer_literal_expressions() {
        let input = "5;";
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 1);
        check_expression_statement(&program[0], &Expression::Literal(Literal::Integer(5)));
    }

    #[test]
    fn it_parses_prefix_expressions() {
        let input = r#"
            -5;
            !foobar;
            !5;
            -foobar;
            !true;
            "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        println!("{:?}", program);
        assert_eq!(program.len(), 5);
        check_expression_statement(
            &program[0],
            &Expression::Prefix(
                Token::Dash,
                Box::new(Expression::Literal(Literal::Integer(5))),
            ),
        );
        check_expression_statement(
            &program[1],
            &Expression::Prefix(
                Token::Bang,
                Box::new(Expression::Identifier("foobar".to_string())),
            ),
        );
        check_expression_statement(
            &program[2],
            &Expression::Prefix(
                Token::Bang,
                Box::new(Expression::Literal(Literal::Integer(5))),
            ),
        );
        check_expression_statement(
            &program[3],
            &Expression::Prefix(
                Token::Dash,
                Box::new(Expression::Identifier("foobar".to_string())),
            ),
        );
        check_expression_statement(
            &program[4],
            &Expression::Prefix(
                Token::Bang,
                Box::new(Expression::Literal(Literal::Boolean(true))),
            ),
        );
    }

    #[test]
    fn it_parses_infix_expressions() {
        let input = r#"
            5 + 5;
            5 - 5;
            5 * 5;
            5 / 5;
            5 > 5;
            5 < 5;
            5 == 5;
            5 != 5;
            true == true;
            true != false;
            false == false;
            "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 11);
        check_expression_statement(
            &program[0],
            &Expression::Infix(
                Box::new(Expression::Literal(Literal::Integer(5))),
                Token::Plus,
                Box::new(Expression::Literal(Literal::Integer(5))),
            ),
        );
        check_expression_statement(
            &program[1],
            &Expression::Infix(
                Box::new(Expression::Literal(Literal::Integer(5))),
                Token::Dash,
                Box::new(Expression::Literal(Literal::Integer(5))),
            ),
        );
        check_expression_statement(
            &program[2],
            &Expression::Infix(
                Box::new(Expression::Literal(Literal::Integer(5))),
                Token::Asterisk,
                Box::new(Expression::Literal(Literal::Integer(5))),
            ),
        );
        check_expression_statement(
            &program[3],
            &Expression::Infix(
                Box::new(Expression::Literal(Literal::Integer(5))),
                Token::Slash,
                Box::new(Expression::Literal(Literal::Integer(5))),
            ),
        );
        check_expression_statement(
            &program[4],
            &Expression::Infix(
                Box::new(Expression::Literal(Literal::Integer(5))),
                Token::Gt,
                Box::new(Expression::Literal(Literal::Integer(5))),
            ),
        );
        check_expression_statement(
            &program[5],
            &Expression::Infix(
                Box::new(Expression::Literal(Literal::Integer(5))),
                Token::Lt,
                Box::new(Expression::Literal(Literal::Integer(5))),
            ),
        );
        check_expression_statement(
            &program[6],
            &Expression::Infix(
                Box::new(Expression::Literal(Literal::Integer(5))),
                Token::Eq,
                Box::new(Expression::Literal(Literal::Integer(5))),
            ),
        );
        check_expression_statement(
            &program[7],
            &Expression::Infix(
                Box::new(Expression::Literal(Literal::Integer(5))),
                Token::NotEq,
                Box::new(Expression::Literal(Literal::Integer(5))),
            ),
        );
        check_expression_statement(
            &program[8],
            &Expression::Infix(
                Box::new(Expression::Literal(Literal::Boolean(true))),
                Token::Eq,
                Box::new(Expression::Literal(Literal::Boolean(true))),
            ),
        );
        check_expression_statement(
            &program[9],
            &Expression::Infix(
                Box::new(Expression::Literal(Literal::Boolean(true))),
                Token::NotEq,
                Box::new(Expression::Literal(Literal::Boolean(false))),
            ),
        );
        check_expression_statement(
            &program[10],
            &Expression::Infix(
                Box::new(Expression::Literal(Literal::Boolean(false))),
                Token::Eq,
                Box::new(Expression::Literal(Literal::Boolean(false))),
            ),
        );
    }

    #[test]
    fn it_parses_operator_precedence() {
        let without_parens = r#"
            -a * b
            !-a
            a + b + c
            a + b - c
            a * b * c
            a * b / c
            a + b / c
            a + b * c + d / e - f
            3 + 4; -5 * 5
            5 > 4 == 3 < 4
            5 < 4 != 3 > 4
            3 + 4 * 5 == 3 * 1 + 4 * 5
            "#;
        let with_parens = r#"
            ((-a) * b)
            (!(-a))
            ((a + b) + c)
            ((a + b) - c)
            ((a * b) * c)
            ((a * b) / c)   
            (a + (b / c))
            (((a + (b * c)) + (d / e)) - f)
            (3 + 4)((-5) * 5)
            ((5 > 4) == (3 < 4)))
            ((5 < 4) != (3 > 4))
            ((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))
            "#;
        let without_parens_lexer = Lexer::new(without_parens.into());
        let mut without_parens_parser = Parser::new(without_parens_lexer);
        let without_parens_program = without_parens_parser.parse_program().unwrap();
        assert_eq!(without_parens_program.len(), 13);
        let with_parens_lexer = Lexer::new(with_parens.into());
        let mut with_parens_parser = Parser::new(with_parens_lexer);
        let with_parens_program = with_parens_parser.parse_program().unwrap();
        assert_eq!(with_parens_program.len(), 13);
        assert_eq!(without_parens_program, with_parens_program);
    }

    #[test]
    fn it_parses_boolean_literal_expressions() {
        let input = r#"
                true;
                false;
                "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 2);
        check_expression_statement(&program[0], &Expression::Literal(Literal::Boolean(true)));
        check_expression_statement(&program[1], &Expression::Literal(Literal::Boolean(false)));
    }

    fn check_expression_statement(statement: &Statement, expected_value: &Expression) {
        match statement {
            Statement::Expression(expression) => check_expression(expression, expected_value),
            _ => panic!("Expected expression statement"),
        }
    }

    fn check_expression(expr: &Expression, expected_expr: &Expression) {
        match (expr, expected_expr) {
            (Expression::Literal(literal), Expression::Literal(expected_literal)) => {
                match (literal, expected_literal) {
                    (Literal::String(s), Literal::String(expected_s)) => {
                        assert_eq!(s, expected_s);
                    }
                    (Literal::Integer(i), Literal::Integer(expected_i)) => {
                        assert_eq!(i, expected_i);
                    }
                    (Literal::Boolean(b), Literal::Boolean(expected_b)) => {
                        assert_eq!(b, expected_b);
                    }
                    _ => panic!("Literal type mismatch"),
                }
            }
            (Expression::Identifier(ident), Expression::Identifier(expected_ident)) => {
                assert_eq!(ident, expected_ident);
            }
            (
                Expression::Prefix(token, inner_expr),
                Expression::Prefix(expected_token, expected_inner_expr),
            ) => {
                assert_eq!(token, expected_token);
                check_expression(&**inner_expr, &**expected_inner_expr);
            }
            (
                Expression::Infix(left_expr, token, right_expr),
                Expression::Infix(expected_left_expr, expected_token, expected_right_expr),
            ) => {
                assert_eq!(token, expected_token);
                check_expression(&**left_expr, &**expected_left_expr);
                check_expression(&**right_expr, &**expected_right_expr);
            }
            // ... other expression variants can be added as necessary ...
            _ => panic!("Expression type mismatch"),
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
