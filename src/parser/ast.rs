use std::fmt::{Display, Formatter, Result};

use crate::token::Token;

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(i64),
    Boolean(bool),
    String(String),
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Literal::Integer(i) => write!(f, "{}", i),
            Literal::String(s) => write!(f, "{}", s),
            Literal::Boolean(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Identifier(String),
    Literal(Literal),
    Prefix(Token, Box<Expression>),
    Infix(Box<Expression>, Token, Box<Expression>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Expression::Identifier(name) => write!(f, "{}", name),
            Expression::Literal(value) => write!(f, "{}", value),
            Expression::Prefix(token, value) => write!(f, "({}{})", token, value),
            Expression::Infix(left, token, right) => write!(f, "({} {} {})", left, token, right),
        }
    }
}

#[derive(Debug)]
pub enum Statement {
    Let(String, Expression),
    Return(Expression),
    Expression(Expression),
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Statement::Let(name, value) => write!(f, "let {} = {};", name, value),
            Statement::Return(value) => write!(f, "return {};", value),
            Statement::Expression(value) => write!(f, "{}", value),
        }
    }
}

// define node enum - our parser works on statement and expression nodes
pub enum Node {
    Program(Vec<Statement>),
    Statement(Statement),
    Expression(Expression),
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Node::Program(statements) => {
                for statement in statements {
                    write!(f, "{}", statement)?;
                }
                Ok(())
            }
            Node::Statement(statement) => write!(f, "{}", statement),
            Node::Expression(expression) => write!(f, "{}", expression),
        }
    }
}
