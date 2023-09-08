use std::fmt::{Display, Formatter, Result};

use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Boolean(bool),
    String(String),
    Array(Vec<Expression>),
    Hash(Vec<(Expression, Expression)>),
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Literal::Integer(i) => write!(f, "{}", *i),
            Literal::String(s) => write!(f, "{}", s),
            Literal::Boolean(s) => write!(f, "{}", s),
            Literal::Array(a) => {
                write!(f, "[")?;
                for (i, e) in a.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, "]")
            }
            Literal::Hash(h) => {
                write!(f, "{{")?;
                for (i, (k, v)) in h.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
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
#[derive(Debug, Clone, PartialEq)]
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

pub fn modify<M>(node: Node, modifier: M) -> Node
where
    M: Fn(Node) -> Node + Clone,
{
    let new_node = match node {
        Node::Program(statements) => {
            let modified_statements: Vec<Statement> = statements
                .into_iter()
                .map(|s| {
                    if let Node::Statement(modified_s) =
                        modify(Node::Statement(s), modifier.clone())
                    {
                        modified_s
                    } else {
                        panic!("Expected a Node::Statement variant!"); // Handle this better based on your requirements
                    }
                })
                .collect();
            Node::Program(modified_statements)
        }
        // Node::Expression(expression) => Node::Expression(expression),
        _ => node,
    };
    modifier(new_node)
}

// type ModifierFunc = fn(Node) -> Node;
// pub fn modify_with_function<M>(node: Node, modifier: ModifierFunc) -> Node
// where
//     M: Fn(Node) -> Node + Clone,
// {
//     let new_node = match node {
//         Node::Program(statements) => {
//             let modified_statements: Vec<Statement> = statements.iter().map(|s| modify(Node::Statement(s), modifier.clone())).collect();
//             Node::Program(modified_statements)
//         }
//         Node::Expression(expression) => {
//             let modified_expression = modify(expression, modifier);
//
//         }
//         _ => node,
//     }
//     modifier(new_node)
//
// }

#[cfg(test)]
mod test {

    use super::*;
    fn unwrap_node_to_expression(node: Node) -> Expression {
        match node {
            Node::Expression(expr) => expr,
            _ => panic!("Expected Node::Expression!"),
        }
    }

    #[test]
    fn it_modifies() {
        let one = || -> Node { Node::Expression(Expression::Literal(Literal::Integer(1))) };
        let two = || -> Node { Node::Expression(Expression::Literal(Literal::Integer(2))) };

        let turn_one_into_two = |expr: Node| -> Node {
            match expr {
                Node::Expression(Expression::Literal(Literal::Integer(1))) => {
                    return Node::Expression(Expression::Literal(Literal::Integer(2)))
                }
                _ => return expr,
            }
        };

        let tests = vec![
            (
                one(),
                Node::Expression(Expression::Literal(Literal::Integer(2))),
            ),
            (
                two(),
                Node::Expression(Expression::Literal(Literal::Integer(2))),
            ),
        ];

        for (input, expected) in tests {
            let modified = modify(input, turn_one_into_two);
            println!("modified: {}", modified);
            println!("expected: {}", expected);
            assert_eq!(modified, expected);
        }
    }

    #[test]
    fn it_modifies_infixs() {
        let one = || -> Node { Node::Expression(Expression::Literal(Literal::Integer(1))) };
        let two = || -> Node { Node::Expression(Expression::Literal(Literal::Integer(2))) };

        let turn_one_into_two = |expr: Node| -> Node {
            match expr {
                Node::Expression(Expression::Literal(Literal::Integer(1))) => {
                    return Node::Expression(Expression::Literal(Literal::Integer(2)))
                }
                _ => return expr,
            }
        };

        let tests = vec![(
            Node::Expression(Expression::Infix(
                Box::new(unwrap_node_to_expression(one())),
                Token::Plus,
                Box::new(unwrap_node_to_expression(two())),
            )),
            (Node::Expression(Expression::Infix(
                Box::new(unwrap_node_to_expression(two())),
                Token::Plus,
                Box::new(unwrap_node_to_expression(two())),
            ))),
        )];

        for (input, expected) in tests {
            let modified = modify(input, turn_one_into_two);
            println!("modified: {}", modified);
            println!("expected: {}", expected);
            assert_eq!(modified, expected);
        }
    }
}
