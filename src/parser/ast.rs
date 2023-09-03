use std::fmt::{Display, Formatter, Result};

use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Boolean(bool),
    String(String),
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Literal::Integer(i) => write!(f, "{}", *i),
            Literal::String(s) => write!(f, "{}", s),
            Literal::Boolean(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Identifier(String),
    Literal(Literal),
    Prefix(Token, Box<Expression>),
    Infix(Box<Expression>, Token, Box<Expression>),
    If(Box<Expression>, Vec<Statement>, Option<Vec<Statement>>),
    Function(Vec<String>, Vec<Statement>),
    FunctionCall(Box<Expression>, Vec<Expression>),
    Index(Box<Expression>, Box<Expression>),
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Expression::Identifier(name) => write!(f, "{}", name),
            Expression::Literal(value) => write!(f, "{}", value),
            Expression::Prefix(token, value) => write!(f, "({}{})", token, value),
            Expression::Infix(left, token, right) => write!(f, "{} {} {}", left, token, right),
            Expression::If(condition, consequence, alternative) => {
                write!(f, "if {} {{", condition)?;
                for statement in consequence {
                    write!(f, "{}", statement)?;
                }
                write!(f, "}}")?;
                if let Some(alternative) = alternative {
                    write!(f, " else {{")?;
                    for statement in alternative {
                        write!(f, "{}", statement)?;
                    }
                    write!(f, "}}")?;
                }
                Ok(())
            }
            Expression::Function(parameters, body) => {
                write!(f, "fn(")?;
                for (i, parameter) in parameters.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", parameter)?;
                }
                write!(f, ") {{")?;
                for statement in body {
                    write!(f, "{}", statement)?;
                }
                write!(f, "}}")
            }
            Expression::FunctionCall(function, arguments) => {
                write!(f, "{}(", function)?;
                for (i, argument) in arguments.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", argument)?;
                }
                write!(f, ")")
            }
            Expression::Index(left, index) => write!(f, "({}[{}])", left, index),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
