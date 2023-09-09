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

        Node::Expression(expression) => match expression {
            Expression::Infix(left, token, right) => {
                let modified_left = modify(Node::Expression(*left), modifier.clone());
                let modified_right = modify(Node::Expression(*right), modifier.clone());
                Node::Expression(Expression::Infix(
                    Box::new(unwrap_node_to_expression(modified_left)),
                    token,
                    Box::new(unwrap_node_to_expression(modified_right)),
                ))
            }

            Expression::Prefix(left, expression) => {
                let modified_expression = modify(Node::Expression(*expression), modifier.clone());
                Node::Expression(Expression::Prefix(
                    left,
                    Box::new(unwrap_node_to_expression(modified_expression)),
                ))
            }

            Expression::Index(left, index) => {
                let modified_left = modify(Node::Expression(*left), modifier.clone());
                let modified_index = modify(Node::Expression(*index), modifier.clone());
                Node::Expression(Expression::Index(
                    Box::new(unwrap_node_to_expression(modified_left)),
                    Box::new(unwrap_node_to_expression(modified_index)),
                ))
            }

            Expression::If(condition, consequence, alternative) => {
                let modified_condition = modify(Node::Expression(*condition), modifier.clone());

                let modified_consequence: Vec<Statement> =
                    unwrap_node_to_statements(modify(Node::Program(consequence), modifier.clone()));

                let modified_alternative: Option<Vec<Statement>> = match alternative {
                    Some(alternative) => Some(unwrap_node_to_statements(modify(
                        Node::Program(alternative),
                        modifier.clone(),
                    ))),
                    None => None,
                };
                Node::Expression(Expression::If(
                    Box::new(unwrap_node_to_expression(modified_condition)),
                    modified_consequence,
                    modified_alternative,
                ))
            }

            Expression::Function(arguments, body) => {
                let modified_arguments: Vec<String> = arguments
                    .iter()
                    .map(|argument| {
                        let modified_argument = modify(
                            Node::Expression(Expression::Identifier(argument.clone())),
                            modifier.clone(),
                        );
                        let modified_expression = unwrap_node_to_expression(modified_argument);
                        let modified_identifier = match modified_expression {
                            Expression::Identifier(identifier) => identifier,
                            _ => panic!("Expected Expression::Identifier!"),
                        };
                        modified_identifier
                    })
                    .collect();

                let modified_body: Vec<Statement> =
                    unwrap_node_to_statements(modify(Node::Program(body), modifier.clone()));
                Node::Expression(Expression::Function(modified_arguments, modified_body))
            }

            Expression::Literal(literal) => {
                let modified_literal = match literal {
                    Literal::Array(expressions) => {
                        let modified_expressions: Vec<Expression> = expressions
                            .iter()
                            .map(|expression| {
                                let modified_expression =
                                    modify(Node::Expression(expression.clone()), modifier.clone());
                                unwrap_node_to_expression(modified_expression)
                            })
                            .collect();
                        Literal::Array(modified_expressions)
                    }
                    Literal::Hash(pairs) => {
                        let modified_pairs: Vec<(Expression, Expression)> = pairs
                            .iter()
                            .map(|(key, value)| {
                                let modified_key =
                                    modify(Node::Expression(key.clone()), modifier.clone());
                                let modified_value =
                                    modify(Node::Expression(value.clone()), modifier.clone());
                                (
                                    unwrap_node_to_expression(modified_key),
                                    unwrap_node_to_expression(modified_value),
                                )
                            })
                            .collect();
                        Literal::Hash(modified_pairs)
                    }
                    _ => literal.clone(),
                };
                Node::Expression(Expression::Literal(modified_literal))
            }

            _ => Node::Expression(expression),
        },
        Node::Statement(statement) => match statement {
            Statement::Expression(expression) => {
                let modified_expression = modify(Node::Expression(expression), modifier.clone());
                Node::Statement(Statement::Expression(unwrap_node_to_expression(
                    modified_expression,
                )))
            }
            Statement::Return(expression) => {
                let modified_expression = modify(Node::Expression(expression), modifier.clone());
                Node::Statement(Statement::Return(unwrap_node_to_expression(
                    modified_expression,
                )))
            }
            Statement::Let(name, expression) => {
                let modified_expression = modify(Node::Expression(expression), modifier.clone());
                Node::Statement(Statement::Let(
                    name,
                    unwrap_node_to_expression(modified_expression),
                ))
            }
        },
        _ => node,
    };
    modifier(new_node)
}

fn unwrap_node_to_expression(node: Node) -> Expression {
    match node {
        Node::Expression(expr) => expr,
        _ => panic!("Expected Node::Expression!"),
    }
}

fn unwrap_node_to_statements(node: Node) -> Vec<Statement> {
    match node {
        Node::Program(statements) => statements,
        _ => panic!("Expected Node::Program!"),
    }
}

fn unwrap_node_to_statement(node: Node) -> Statement {
    match node {
        Node::Statement(statement) => statement,
        _ => panic!("Expected Node::Statement!"),
    }
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
    fn get_closures() -> (
        Box<dyn Fn() -> Node>,
        Box<dyn Fn() -> Node>,
        Box<dyn Fn(Node) -> Node>,
    ) {
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
        (Box::new(one), Box::new(two), Box::new(turn_one_into_two))
    }

    #[test]
    fn it_modifies() {
        let (one, two, turn_one_into_two) = get_closures();

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
            let modified = modify(input, &turn_one_into_two);
            println!("modified: {}", modified);
            println!("expected: {}", expected);
            assert_eq!(modified, expected);
        }
    }

    #[test]
    fn it_modifies_infixs() {
        let (one, two, turn_one_into_two) = get_closures();

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
            let modified = modify(input, &turn_one_into_two);
            println!("modified: {}", modified);
            println!("expected: {}", expected);
            assert_eq!(modified, expected);
        }
    }

    #[test]
    fn it_modifies_prefixs() {
        let (one, two, turn_one_into_two) = get_closures();
        let tests = vec![(
            Node::Expression(Expression::Prefix(
                Token::Bang,
                Box::new(unwrap_node_to_expression(one())),
            )),
            Node::Expression(Expression::Prefix(
                Token::Bang,
                Box::new(unwrap_node_to_expression(two())),
            )),
        )];

        for (input, expected) in tests {
            let modified = modify(input, &turn_one_into_two);
            println!("modified: {}", modified);
            println!("expected: {}", expected);
            assert_eq!(modified, expected);
        }
    }

    #[test]
    fn it_modifies_indexs() {
        let (one, two, turn_one_into_two) = get_closures();
        let tests = vec![(
            Node::Expression(Expression::Index(
                Box::new(unwrap_node_to_expression(one())),
                Box::new(unwrap_node_to_expression(one())),
            )),
            Node::Expression(Expression::Index(
                Box::new(unwrap_node_to_expression(two())),
                Box::new(unwrap_node_to_expression(two())),
            )),
        )];

        for (input, expected) in tests {
            let modified = modify(input, &turn_one_into_two);
            println!("modified: {}", modified);
            println!("expected: {}", expected);
            assert_eq!(modified, expected);
        }
    }

    #[test]
    fn it_modifies_if_expressions() {
        let (one, two, turn_one_into_two) = get_closures();
        let tests = vec![(
            Node::Expression(Expression::If(
                Box::new(unwrap_node_to_expression(one())),
                vec![Statement::Expression(unwrap_node_to_expression(one()))],
                None,
            )),
            Node::Expression(Expression::If(
                Box::new(unwrap_node_to_expression(two())),
                vec![Statement::Expression(unwrap_node_to_expression(two()))],
                None,
            )),
        )];

        for (input, expected) in tests {
            let modified = modify(input, &turn_one_into_two);
            println!("modified: {}", modified);
            println!("expected: {}", expected);
            assert_eq!(modified, expected);
        }
    }

    #[test]
    fn it_modifies_return_statements() {
        let (one, two, turn_one_into_two) = get_closures();
        let tests = vec![(
            Node::Statement(Statement::Return(unwrap_node_to_expression(one()))),
            Node::Statement(Statement::Return(unwrap_node_to_expression(two()))),
        )];

        for (input, expected) in tests {
            let modified = modify(input, &turn_one_into_two);
            println!("modified: {}", modified);
            println!("expected: {}", expected);
            assert_eq!(modified, expected);
        }
    }

    #[test]
    fn it_modifies_let_statements() {
        let (one, two, turn_one_into_two) = get_closures();
        let tests = vec![(
            Node::Statement(Statement::Let(
                "a".to_string(),
                unwrap_node_to_expression(one()),
            )),
            Node::Statement(Statement::Let(
                "a".to_string(),
                unwrap_node_to_expression(two()),
            )),
        )];

        for (input, expected) in tests {
            let modified = modify(input, &turn_one_into_two);
            println!("modified: {}", modified);
            println!("expected: {}", expected);
            assert_eq!(modified, expected);
        }
    }

    #[test]
    fn it_modifies_function_literals() {
        let (one, two, turn_one_into_two) = get_closures();
        let tests = vec![(
            Node::Expression(Expression::Function(
                vec!["a".to_string()],
                vec![Statement::Expression(unwrap_node_to_expression(one()))],
            )),
            Node::Expression(Expression::Function(
                vec!["a".to_string()],
                vec![Statement::Expression(unwrap_node_to_expression(two()))],
            )),
        )];

        for (input, expected) in tests {
            let modified = modify(input, &turn_one_into_two);
            println!("modified: {}", modified);
            println!("expected: {}", expected);
            assert_eq!(modified, expected);
        }
    }

    #[test]
    fn it_modifies_array_literals() {
        let (one, two, turn_one_into_two) = get_closures();
        let tests = vec![(
            Node::Expression(Expression::Literal(Literal::Array(vec![
                unwrap_node_to_expression(one()),
                unwrap_node_to_expression(one()),
            ]))),
            Node::Expression(Expression::Literal(Literal::Array(vec![
                unwrap_node_to_expression(two()),
                unwrap_node_to_expression(two()),
            ]))),
        )];

        for (input, expected) in tests {
            let modified = modify(input, &turn_one_into_two);
            println!("modified: {}", modified);
            println!("expected: {}", expected);
            assert_eq!(modified, expected);
        }
    }

    #[test]
    fn it_modifies_hash_literals() {
        let (one, two, turn_one_into_two) = get_closures();
        let tests = vec![(
            Node::Expression(Expression::Literal(Literal::Hash(vec![(
                unwrap_node_to_expression(one()),
                unwrap_node_to_expression(one()),
            )]))),
            Node::Expression(Expression::Literal(Literal::Hash(vec![(
                unwrap_node_to_expression(two()),
                unwrap_node_to_expression(two()),
            )]))),
        )];

        for (input, expected) in tests {
            let modified = modify(input, &turn_one_into_two);
            println!("modified: {}", modified);
            println!("expected: {}", expected);
            assert_eq!(modified, expected);
        }
    }
}
