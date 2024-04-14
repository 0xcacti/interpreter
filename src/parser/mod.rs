pub mod ast;
pub mod errors;
pub mod precedence;

use self::ast::*;
use self::errors::*;
use self::precedence::*;

use crate::lexer::Lexer;
use crate::token::Token;

use anyhow::Result;
use std::rc::Rc;

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

    pub fn parse_program(&mut self) -> Result<Vec<Statement>, ParserErrors> {
        let mut program = Vec::new();

        while !self.current_token_is(&Token::Eof) {
            match self.parse_statement() {
                Ok(statement) => program.push(statement),
                Err(e) => self.errors.push(e),
            }
            self.next_token();
        }

        if !self.errors.is_empty() {
            Err(self.errors.clone())
        } else {
            Ok(program)
        }
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

        let exp = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(&Token::Semicolon) {
            self.next_token()
        }

        Ok(Statement::Let(ident, exp))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParserError> {
        self.next_token();
        let exp = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token_is(&Token::Semicolon) {
            self.next_token()
        }
        Ok(Statement::Return(exp))
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
            Token::Int(i) => Expression::Literal(Literal::Integer(i)),
            Token::True => Expression::Literal(Literal::Boolean(true)),
            Token::False => Expression::Literal(Literal::Boolean(false)),
            Token::Bang | Token::Dash => self.parse_prefix_expression()?, // is there a better way
            Token::Lparen => {
                self.next_token();
                let exp = self.parse_expression(Precedence::Lowest)?;
                self.expect_peek_token(&Token::Rparen)?;
                exp
            }
            Token::If => self.parse_if_expression()?,
            Token::Function => self.parse_function_expression()?,
            Token::Macro => self.parse_macro_expression()?,
            Token::LBracket => self.parse_array_literal()?,
            Token::Lbrace => self.parse_hash_literal()?,
            Token::String(ref s) => Expression::Literal(Literal::String(s.clone())),
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
                Token::Lparen => {
                    self.next_token();
                    exp = self.parse_function_call_expression(exp)?;
                }
                Token::LBracket => {
                    self.next_token();
                    exp = self.parse_index_expression(exp)?;
                }
                _ => break,
            }
        }

        Ok(exp)
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression, ParserError> {
        let prefix = self.current_token.clone();
        self.next_token();
        let exp = self.parse_expression(Precedence::Prefix)?;
        Ok(Expression::Prefix(prefix, Box::new(exp)))
    }

    fn parse_infix_expression(&mut self, left_exp: Expression) -> Result<Expression, ParserError> {
        let infix = self.current_token.clone();
        let precedence = token_precedence(&self.current_token);
        self.next_token();
        let right_exp = self.parse_expression(precedence)?;
        Ok(Expression::Infix(
            Box::new(left_exp),
            infix,
            Box::new(right_exp),
        ))
    }
    fn parse_if_expression(&mut self) -> Result<Expression, ParserError> {
        self.expect_peek_token(&Token::Lparen)?;
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;
        self.expect_peek_token(&Token::Rparen)?;
        self.expect_peek_token(&Token::Lbrace)?;
        let if_block = self.parse_block_statement()?;
        let else_block = if self.peek_token_is(&Token::Else) {
            self.next_token();
            self.expect_peek_token(&Token::Lbrace)?;
            Some(self.parse_block_statement()?)
        } else {
            None
        };
        Ok(Expression::If(Box::new(condition), if_block, else_block))
    }

    fn parse_macro_expression(&mut self) -> Result<Expression, ParserError> {
        self.expect_peek_token(&Token::Lparen)?;
        let parameters = self.parse_function_parameters()?;
        self.expect_peek_token(&Token::Lbrace)?;
        let body = self.parse_block_statement()?;
        Ok(Expression::Macro(parameters, body))
    }

    fn parse_function_expression(&mut self) -> Result<Expression, ParserError> {
        self.expect_peek_token(&Token::Lparen)?;
        let parameters = self.parse_function_parameters()?;
        self.expect_peek_token(&Token::Lbrace)?;
        let body = self.parse_block_statement()?;
        Ok(Expression::Function(parameters, body))
    }

    fn parse_array_literal(&mut self) -> Result<Expression, ParserError> {
        let elements = self.parse_expression_list(&Token::RBracket)?;
        Ok(Expression::Literal(Literal::Array(Rc::new(elements))))
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<String>, ParserError> {
        let mut identifiers = Vec::new();
        if self.peek_token_is(&Token::Rparen) {
            self.next_token();
            return Ok(identifiers);
        }
        self.next_token();

        match &self.current_token {
            Token::Ident(ident) => identifiers.push(ident.clone()),
            _ => {
                return Err(ParserError::new(format!(
                    "parse error: expected identifier, got {}",
                    self.current_token
                )))
            }
        }

        while self.peek_token_is(&Token::Comma) {
            self.next_token();
            self.next_token();
            match &self.current_token {
                Token::Ident(ident) => identifiers.push(ident.clone()),
                _ => {
                    return Err(ParserError::new(format!(
                        "parse error: expected identifier, got {}",
                        self.current_token
                    )))
                }
            }
        }
        self.expect_peek_token(&Token::Rparen)?;
        Ok(identifiers)
    }

    fn parse_block_statement(&mut self) -> Result<Vec<Statement>, ParserError> {
        let mut statements = Vec::new();
        self.next_token();
        while !self.current_token_is(&Token::Rbrace) && !self.current_token_is(&Token::Eof) {
            if let Ok(statement) = self.parse_statement() {
                statements.push(statement);
            }
            self.next_token();
        }
        Ok(statements)
    }

    fn parse_function_call_expression(
        &mut self,
        exp: Expression,
    ) -> Result<Expression, ParserError> {
        let arguments = self.parse_expression_list(&Token::Rparen)?;
        Ok(Expression::FunctionCall(Box::new(exp), arguments))
    }

    fn parse_expression_list(
        &mut self,
        ending_token: &Token,
    ) -> Result<Vec<Expression>, ParserError> {
        let mut arguments = Vec::new();
        if self.peek_token_is(ending_token) {
            self.next_token();
            return Ok(arguments);
        }

        self.next_token();

        arguments.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token_is(&Token::Comma) {
            self.next_token();
            self.next_token();
            arguments.push(self.parse_expression(Precedence::Lowest)?);
        }

        self.expect_peek_token(ending_token)?;

        Ok(arguments)
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

    fn parse_hash_literal(&mut self) -> Result<Expression, ParserError> {
        let mut map = Vec::new();
        while !self.peek_token_is(&Token::Rbrace) {
            self.next_token();
            let key = self.parse_expression(Precedence::Lowest)?;
            self.expect_peek_token(&Token::Colon)?;
            self.next_token();
            let value = self.parse_expression(Precedence::Lowest)?;
            map.push((key, value));
            if !self.peek_token_is(&Token::Rbrace) {
                self.expect_peek_token(&Token::Comma)?;
            }
        }
        self.expect_peek_token(&Token::Rbrace)?;
        Ok(Expression::Literal(Literal::Hash(map)))
    }

    fn parse_index_expression(&mut self, left: Expression) -> Result<Expression, ParserError> {
        self.next_token();
        let index = self.parse_expression(Precedence::Lowest)?;
        self.expect_peek_token(&Token::RBracket)?;
        Ok(Expression::Index(Box::new(left), Box::new(index)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
        check_let_statement(&program[0], "x", &Expression::Literal(Literal::Integer(5)));
        check_let_statement(&program[1], "y", &Expression::Literal(Literal::Integer(10)));
        check_let_statement(
            &program[2],
            "foobar",
            &Expression::Literal(Literal::Integer(838383)),
        );
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
        check_return_statement(&program[0], &Expression::Literal(Literal::Integer(5)));
        check_return_statement(&program[1], &Expression::Literal(Literal::Integer(10)));
        check_return_statement(&program[2], &Expression::Literal(Literal::Integer(993322)));
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
            3 + 4;
           -5 * 5;
           -a * b;
           !-a;
           a + b + c;
           a + b - c;
           a * b * c;
           a * b / c;
           a + b / c;
           a + b * c + d / e - f;
           5 > 4 == 3 < 4;
           5 < 4 != 3 > 4;
           3 + 4 * 5 == 3 * 1 + 4 * 5;
           a * [1, 2, 3, 4][b * c] * d;
           add(a * b[2], b[1], 2 * [1, 2][1]);
        "#;
        let with_parens = r#"
            (3 + 4);
            ((-5) * 5);
            ((-a) * b);
            (!(-a));
            ((a + b) + c);
            ((a + b) - c);
            ((a * b) * c);
            ((a * b) / c);
            (a + (b / c));
            (((a + (b * c)) + (d / e)) - f);
            ((5 > 4) == (3 < 4));
            ((5 < 4) != (3 > 4));
            ((3 + (4 * 5)) == ((3 * 1) + (4 * 5)));
            ((a * ([1, 2, 3, 4][(b * c)])) * d);
            add((a * (b[2])), (b[1]), (2 * ([1, 2][1])));
            "#;
        // "#;

        let without_parens_lexer = Lexer::new(without_parens.into());
        let mut without_parens_parser = Parser::new(without_parens_lexer);
        let without_parens_program = without_parens_parser.parse_program().unwrap();

        let with_parens_lexer = Lexer::new(with_parens.into());
        let mut with_parens_parser = Parser::new(with_parens_lexer);
        let with_parens_program = with_parens_parser.parse_program().unwrap();

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

    #[test]
    fn it_parses_operator_precedence_with_grouped_expressions() {
        let without_parens = r#"
            1 + (2 + 3) + 4;
            2 / (5 + 5);
            (5 + 5) * 2;
            -(5 + 5);
            !(true == true);
            "#;
        let with_parens = r#"
            ((1 + (2 + 3)) + 4);
            (2 / (5 + 5));
            ((5 + 5) * 2);
            (-(5 + 5));
            (!(true == true));
            "#;
        let without_parens_lexer = Lexer::new(without_parens.into());
        let mut without_parens_parser = Parser::new(without_parens_lexer);
        let without_parens_program = without_parens_parser.parse_program().unwrap();
        let with_parens_lexer = Lexer::new(with_parens.into());
        let mut with_parens_parser = Parser::new(with_parens_lexer);
        let with_parens_program = with_parens_parser.parse_program().unwrap();
        for (without_parens_statement, with_parens_statement) in without_parens_program
            .iter()
            .zip(with_parens_program.iter())
        {
            assert_eq!(without_parens_statement, with_parens_statement);
        }
    }

    #[test]
    fn it_parses_if_expressions() {
        let input = r#"
                if (x < y) { x }
                "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 1);
        check_expression_statement(
            &program[0],
            &Expression::If(
                Box::new(Expression::Infix(
                    Box::new(Expression::Identifier("x".into())),
                    Token::Lt,
                    Box::new(Expression::Identifier("y".into())),
                )),
                vec![Statement::Expression(Expression::Identifier("x".into()))],
                None,
            ),
        )
    }

    #[test]
    fn it_parses_if_else_expression() {
        let input = r#"
                if (x < y) { x } else { y }
                "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 1);
        check_expression_statement(
            &program[0],
            &Expression::If(
                Box::new(Expression::Infix(
                    Box::new(Expression::Identifier("x".into())),
                    Token::Lt,
                    Box::new(Expression::Identifier("y".into())),
                )),
                vec![Statement::Expression(Expression::Identifier("x".into()))],
                Some(vec![Statement::Expression(Expression::Identifier(
                    "y".into(),
                ))]),
            ),
        );
    }

    #[test]
    fn it_parses_function_literal_expressions() {
        let input = r#"
                fn(x, y) { x + y; }
                "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 1);
        check_expression_statement(
            &program[0],
            &Expression::Function(
                vec!["x".into(), "y".into()],
                vec![Statement::Expression(Expression::Infix(
                    Box::new(Expression::Identifier("x".into())),
                    Token::Plus,
                    Box::new(Expression::Identifier("y".into())),
                ))],
            ),
        );
    }

    #[test]
    fn it_parses_function_parameters() {
        let input = r#"
                fn() {};
                fn(x) {};
                fn(x, y, z) {};
                "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 3);
        check_expression_statement(&program[0], &Expression::Function(vec![], vec![]));
        check_expression_statement(&program[1], &Expression::Function(vec!["x".into()], vec![]));
        check_expression_statement(
            &program[2],
            &Expression::Function(vec!["x".into(), "y".into(), "z".into()], vec![]),
        );
    }

    #[test]
    fn it_parses_function_call_expressions() {
        let input = r#"
                add(1, 2 * 3, 4 + 5);
                "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 1);
        check_expression_statement(
            &program[0],
            &Expression::FunctionCall(
                Box::new(Expression::Identifier("add".into())),
                vec![
                    Expression::Literal(Literal::Integer(1)),
                    Expression::Infix(
                        Box::new(Expression::Literal(Literal::Integer(2))),
                        Token::Asterisk,
                        Box::new(Expression::Literal(Literal::Integer(3))),
                    ),
                    Expression::Infix(
                        Box::new(Expression::Literal(Literal::Integer(4))),
                        Token::Plus,
                        Box::new(Expression::Literal(Literal::Integer(5))),
                    ),
                ],
            ),
        );
    }

    #[test]
    fn it_parses_function_call_operator_precedence() {
        let input = r#"
                a + add(b * c) + d;
                add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))
                add(a + b + c * d / f + g)
                "#;
        let expected = r#" 
            ((a + add(b * c)) + d);
            add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)));
            add((((a + b) + ((c * d) / f)) + g))
            "#;

        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        let expected_lexer = Lexer::new(expected.into());
        let mut expected_parser = Parser::new(expected_lexer);
        let expected_program = expected_parser.parse_program().unwrap();
        assert_eq!(program.len(), expected_program.len());
        for (statement, expected_statement) in program.iter().zip(expected_program.iter()) {
            assert_eq!(statement, expected_statement);
        }
    }

    #[test]
    fn it_parses_string_literal_expressions() {
        let input = r#"
                "hello world";
                "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 1);
        check_expression_statement(
            &program[0],
            &Expression::Literal(Literal::String("hello world".into())),
        );
    }

    #[test]
    fn it_parses_array_index_expressions() {
        let input = r#"
                myArray[1 + 1];
                "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 1);
        check_expression_statement(
            &program[0],
            &Expression::Index(
                Box::new(Expression::Identifier("myArray".into())),
                Box::new(Expression::Infix(
                    Box::new(Expression::Literal(Literal::Integer(1))),
                    Token::Plus,
                    Box::new(Expression::Literal(Literal::Integer(1))),
                )),
            ),
        );
    }

    #[test]
    fn it_parses_array_literal_expressions() {
        let input = r#"
                [1, 2 * 2, 3 + 3];
                [1, 2, 3];
                [];
                "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 3);
        check_expression_statement(
            &program[0],
            &Expression::Literal(Literal::Array(Rc::new(vec![
                Expression::Literal(Literal::Integer(1)),
                Expression::Infix(
                    Box::new(Expression::Literal(Literal::Integer(2))),
                    Token::Asterisk,
                    Box::new(Expression::Literal(Literal::Integer(2))),
                ),
                Expression::Infix(
                    Box::new(Expression::Literal(Literal::Integer(3))),
                    Token::Plus,
                    Box::new(Expression::Literal(Literal::Integer(3))),
                ),
            ]))),
        );
        check_expression_statement(
            &program[1],
            &Expression::Literal(Literal::Array(Rc::new(vec![
                Expression::Literal(Literal::Integer(1)),
                Expression::Literal(Literal::Integer(2)),
                Expression::Literal(Literal::Integer(3)),
            ]))),
        );
        check_expression_statement(
            &program[2],
            &Expression::Literal(Literal::Array(Rc::new(vec![]))),
        );
    }

    #[test]
    fn it_parses_hash_literal_expressions() {
        let input = r#"
                {"one": 1, "two": 2, "three": 3};
                {"one": 0 + 1, "two": 10 - 8, "three": 15 / 5};
                {"one": 0 + 1, "two": 10 - 8, "three": 15 / 5};
                {};"#;

        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        assert_eq!(program.len(), 4);
        check_expression_statement(
            &program[0],
            &Expression::Literal(Literal::Hash(vec![
                (
                    Expression::Literal(Literal::String("one".into())),
                    Expression::Literal(Literal::Integer(1)),
                ),
                (
                    Expression::Literal(Literal::String("two".into())),
                    Expression::Literal(Literal::Integer(2)),
                ),
                (
                    Expression::Literal(Literal::String("three".into())),
                    Expression::Literal(Literal::Integer(3)),
                ),
            ])),
        );

        check_expression_statement(
            &program[1],
            &Expression::Literal(Literal::Hash(vec![
                (
                    Expression::Literal(Literal::String("one".into())),
                    Expression::Infix(
                        Box::new(Expression::Literal(Literal::Integer(0))),
                        Token::Plus,
                        Box::new(Expression::Literal(Literal::Integer(1))),
                    ),
                ),
                (
                    Expression::Literal(Literal::String("two".into())),
                    Expression::Infix(
                        Box::new(Expression::Literal(Literal::Integer(10))),
                        Token::Dash,
                        Box::new(Expression::Literal(Literal::Integer(8))),
                    ),
                ),
                (
                    Expression::Literal(Literal::String("three".into())),
                    Expression::Infix(
                        Box::new(Expression::Literal(Literal::Integer(15))),
                        Token::Slash,
                        Box::new(Expression::Literal(Literal::Integer(5))),
                    ),
                ),
            ])),
        );
        check_expression_statement(
            &program[2],
            &Expression::Literal(Literal::Hash(vec![
                (
                    Expression::Literal(Literal::String("one".into())),
                    Expression::Infix(
                        Box::new(Expression::Literal(Literal::Integer(0))),
                        Token::Plus,
                        Box::new(Expression::Literal(Literal::Integer(1))),
                    ),
                ),
                (
                    Expression::Literal(Literal::String("two".into())),
                    Expression::Infix(
                        Box::new(Expression::Literal(Literal::Integer(10))),
                        Token::Dash,
                        Box::new(Expression::Literal(Literal::Integer(8))),
                    ),
                ),
                (
                    Expression::Literal(Literal::String("three".into())),
                    Expression::Infix(
                        Box::new(Expression::Literal(Literal::Integer(15))),
                        Token::Slash,
                        Box::new(Expression::Literal(Literal::Integer(5))),
                    ),
                ),
            ])),
        );

        check_expression_statement(&program[3], &Expression::Literal(Literal::Hash(vec![])));
    }

    #[test]
    fn it_parses_macro_literals() {
        let input = r#"
                macro(x, y) { x + y; };
                "#;
        let lexer = Lexer::new(input.into());
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        check_expression_statement(
            &program[0],
            &Expression::Macro(
                vec!["x".into(), "y".into()],
                vec![Statement::Expression(Expression::Infix(
                    Box::new(Expression::Identifier("x".into())),
                    Token::Plus,
                    Box::new(Expression::Identifier("y".into())),
                ))],
            ),
        );
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
                    (Literal::Array(a), Literal::Array(expected_a)) => {
                        assert_eq!(a.len(), expected_a.len());
                        for (expr, expected_expr) in a.iter().zip(expected_a.iter()) {
                            check_expression(expr, expected_expr);
                        }
                    }
                    (Literal::Hash(h), Literal::Hash(expected_h)) => {
                        assert_eq!(h.len(), expected_h.len());
                        for i in 0..h.len() {
                            let (key, value) = &h[i];
                            let (expected_key, expected_value) = &expected_h[i];
                            check_expression(key, expected_key);
                            check_expression(value, expected_value);
                        }
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
            (
                Expression::If(condition, consequence, alternative),
                Expression::If(expected_condition, expected_consequence, expected_alternative),
            ) => {
                check_expression(&**condition, &**expected_condition);
                for (statement, expected_statement) in
                    consequence.iter().zip(expected_consequence.iter())
                {
                    assert_eq!(statement, expected_statement);
                }
                for (statement, expected_statement) in
                    alternative.iter().zip(expected_alternative.iter())
                {
                    assert_eq!(statement, expected_statement);
                }
            }
            (
                Expression::Function(params, body),
                Expression::Function(expected_params, expected_body),
            ) => {
                assert_eq!(params, expected_params);
                for (statement, expected_statement) in body.iter().zip(expected_body.iter()) {
                    assert_eq!(statement, expected_statement);
                }
            }

            (
                Expression::FunctionCall(function, arguments),
                Expression::FunctionCall(expected_function, expected_arguments),
            ) => {
                check_expression(&**function, &**expected_function);
                for (argument, expected_argument) in arguments.iter().zip(expected_arguments.iter())
                {
                    check_expression(argument, expected_argument);
                }
            }
            (
                Expression::Index(left_expr, index_expr),
                Expression::Index(expected_left_expr, expected_index_expr),
            ) => {
                check_expression(&**left_expr, &**expected_left_expr);
                check_expression(&**index_expr, &**expected_index_expr);
            }
            (
                Expression::Macro(params, body),
                Expression::Macro(expected_params, expected_body),
            ) => {
                assert_eq!(params, expected_params);
                for (statement, expected_statement) in body.iter().zip(expected_body.iter()) {
                    assert_eq!(statement, expected_statement);
                }
            }
            // ... other expression variants can be added as necessary ...
            _ => panic!("Expression type mismatch"),
        }
    }

    fn check_let_statement(s: &Statement, name: &str, expected_exp: &Expression) {
        match s {
            Statement::Let(ref ident, ref exp) => {
                assert_eq!(ident, name);
                assert_eq!(exp, expected_exp);
            }
            _ => panic!("expected let statement"),
        }
    }

    fn check_return_statement(s: &Statement, expected_exp: &Expression) {
        match s {
            Statement::Return(exp) => assert_eq!(exp, expected_exp),
            _ => panic!("expected return statement"),
        }
    }
}
