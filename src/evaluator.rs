pub mod error;
pub mod object;

use self::error::EvaluatorError;
use self::object::Object;
use crate::{parser::ast::*, token::Token};

pub fn evaluate(node: Node) -> Result<Box<Object>, EvaluatorError> {
    match node {
        Node::Program(program) => evaluate_statements(&program),
        Node::Statement(statement) => evaluate_statement(&statement),
        Node::Expression(expression) => evaluate_expression(&expression),
    }
}

fn evaluate_statements(statements: &Vec<Statement>) -> Result<Box<Object>, EvaluatorError> {
    let mut result = Box::new(Object::Null);
    for statement in statements {
        let intermediate_value = evaluate_statement(statement)?;

        match *intermediate_value {
            Object::ReturnValue(_) => return Ok(intermediate_value),
            _ => result = intermediate_value,
        }
    }
    Ok(result)
}

fn evaluate_statement(statement: &Statement) -> Result<Box<Object>, EvaluatorError> {
    match statement {
        Statement::Let(name, expression) => {
            let value = evaluate_expression(expression);
            return value;
        }
        Statement::Return(expression) => {
            let value = evaluate_expression(expression)?;
            return Ok(Box::new(Object::ReturnValue(value)));
        }
        Statement::Expression(expression) => evaluate_expression(expression),
    }
}

fn is_truthy(object: &Object) -> bool {
    match object {
        Object::Null => false,
        Object::Boolean(false) => false,
        _ => true,
    }
}

fn evaluate_expression(expression: &Expression) -> Result<Box<Object>, EvaluatorError> {
    match expression {
        Expression::Literal(literal) => evaluate_literal(literal),
        Expression::Prefix(operator, expression) => {
            let right = evaluate_expression(expression)?;
            evaluate_prefix_expression(operator, &right)
        }
        Expression::Infix(left, operator, right) => {
            let left = evaluate_expression(left)?;
            let right = evaluate_expression(right)?;
            evaluate_infix_expression(operator, &left, &right)
        }

        Expression::If(condition, consequence, alternative) => {
            let condition = evaluate_expression(condition)?;
            if is_truthy(&condition) {
                evaluate_block_statement(&consequence)
            } else if let Some(alternative) = alternative {
                evaluate_block_statement(&alternative)
            } else {
                Ok(Box::new(Object::Null))
            }
        }
        _ => Ok(Box::new(Object::Null)),
    }
}

fn evaluate_block_statement(block: &Vec<Statement>) -> Result<Box<Object>, EvaluatorError> {
    let mut result = Box::new(Object::Null);
    for statement in block {
        let intermediate_value = evaluate_statement(statement)?;
        match *result {
            Object::ReturnValue(_) => return Ok(result),
            _ => result = intermediate_value,
        }
    }
    Ok(result)
}

fn evaluate_literal(literal: &Literal) -> Result<Box<Object>, EvaluatorError> {
    match literal {
        Literal::Integer(integer) => Ok(Box::new(Object::Integer(*integer))),
        Literal::Boolean(boolean) => Ok(Box::new(Object::Boolean(*boolean))),
        Literal::String(string) => Ok(Box::new(Object::String(string.clone()))),
    }
}

fn evaluate_prefix_expression(
    operator: &Token,
    expression: &Object,
) -> Result<Box<Object>, EvaluatorError> {
    match operator {
        Token::Bang => evaluate_bang_prefix_operator(expression),
        Token::Dash => evaluate_dash_prefix_operator(expression),
        _ => Ok(Box::new(Object::Null)),
    }
}

fn evaluate_infix_expression(
    operator: &Token,
    left: &Object,
    right: &Object,
) -> Result<Box<Object>, EvaluatorError> {
    match (left, right) {
        (Object::Integer(left), Object::Integer(right)) => {
            evaluate_integer_infix_operator(operator, *left, *right)
        }
        (Object::Boolean(left), Object::Boolean(right)) => {
            evaluate_boolean_infix_operator(operator, *left, *right)
        }
        (Object::String(left), Object::String(right)) => {
            evaluate_string_infix_operator(operator, left, right)
        }
        _ => Err(EvaluatorError::new(format!(
            "type mismatch between operands: {:?} {:?} {:?}",
            left, operator, right
        ))),
    }
}

fn evaluate_bang_prefix_operator(expression: &Object) -> Result<Box<Object>, EvaluatorError> {
    match expression {
        Object::Boolean(b) => Ok(Box::new(Object::Boolean(!b))),
        Object::Null => Ok(Box::new(Object::Boolean(true))),
        _ => Ok(Box::new(Object::Boolean(false))),
    }
}

fn evaluate_dash_prefix_operator(expression: &Object) -> Result<Box<Object>, EvaluatorError> {
    match expression {
        Object::Integer(i) => Ok(Box::new(Object::Integer(-i))),
        _ => Err(EvaluatorError::new(format!(
            "Unknown operator: -{:?}",
            expression
        ))),
    }
}

fn evaluate_string_infix_operator(
    operator: &Token,
    left: &String,
    right: &String,
) -> Result<Box<Object>, EvaluatorError> {
    match operator {
        Token::Plus => {
            let mut string = left.clone();
            string.push_str(right);
            Ok(Box::new(Object::String(string)))
        }
        Token::Eq => Ok(Box::new(Object::Boolean(left == right))),
        Token::NotEq => Ok(Box::new(Object::Boolean(left != right))),

        _ => Err(EvaluatorError::new(format!(
            "Unknown operator: {:?} {:?} {:?}",
            left, operator, right
        ))),
    }
}

fn evaluate_boolean_infix_operator(
    operator: &Token,
    left: bool,
    right: bool,
) -> Result<Box<Object>, EvaluatorError> {
    let result = match operator {
        &Token::Eq => Object::Boolean(left == right),
        &Token::NotEq => Object::Boolean(left != right),
        _ => {
            return Err(EvaluatorError::new(format!(
                "Unknown operator: {:?} {:?} {:?}",
                left, operator, right
            )))
        }
    };

    Ok(Box::new(result))
}

fn evaluate_integer_infix_operator(
    operator: &Token,
    left: i64,
    right: i64,
) -> Result<Box<Object>, EvaluatorError> {
    let result = match operator {
        &Token::Plus => Object::Integer(left + right),
        &Token::Dash => Object::Integer(left - right),
        &Token::Asterisk => Object::Integer(left * right),
        &Token::Slash => {
            if right == 0 {
                return Err(EvaluatorError::new("Division by zero".to_string()));
            }
            Object::Integer(left / right)
        }
        &Token::Lt => Object::Boolean(left < right),
        &Token::Gt => Object::Boolean(left > right),
        &Token::Eq => Object::Boolean(left == right),
        &Token::NotEq => Object::Boolean(left != right),
        _ => {
            return Err(EvaluatorError::new(format!(
                "Unknown operator: {:?} {:?} {:?}",
                left, operator, right
            )))
        }
    };

    Ok(Box::new(result))
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn test_eval(input: String) -> Object {
        let l = Lexer::new(input.as_ref());
        let mut p = Parser::new(l);
        let program = p.parse_program();
        *evaluate(Node::Program(program.unwrap())).unwrap()
    }

    fn test_object_is_expected(o: &Object, expected: &Object) {
        println!("o: {:?}, expected: {:?}", o, expected);
        match (o, expected) {
            (Object::Integer(i), Object::Integer(j)) => assert_eq!(i, j),
            (Object::Boolean(b), Object::Boolean(c)) => assert_eq!(b, c),
            (Object::Null, Object::Null) => assert!(true),
            (Object::ReturnValue(v1), Object::ReturnValue(v2)) => {
                println!("v1: {:?}, v2: {:?}", v1, v2);
                test_object_is_expected(v1, v2);
            }
            (_, _) => panic!("unexpected types {} and {}", o, expected),
        }
    }

    #[test]
    fn it_evaluates_integer_expressions() {
        let tests = vec![("5", 5), ("10", 10)];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Object::Integer(expected));
        }
    }

    #[test]
    fn it_evaluates_boolean_expressions() {
        let tests = vec![("true", true), ("false", false)];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Object::Boolean(expected));
        }
    }

    #[test]
    fn it_evaluates_bang_operator() {
        let tests = vec![
            ("!true", false),
            ("!false", true),
            ("!5", false),
            ("!!true", true),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Object::Boolean(expected));
        }
    }

    #[test]
    fn it_evaluates_dash_operator() {
        let tests = vec![("-5", -5), ("5", 5), ("-10", -10), ("10", 10)];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Object::Integer(expected));
        }
    }
    #[test]
    fn it_evaluates_integer_infix_expressions() {
        let tests = vec![
            ("5 + 5", 10.into()),
            ("5 - 5", 0.into()),
            ("5 * 5", 25.into()),
            ("5 / 5", 1.into()),
            ("5 > 5", false.into()),
            ("5 < 5", false.into()),
            ("5 == 5", true.into()),
            ("5 != 5", false.into()),
            ("5 == 6", false.into()),
            ("5 != 6", true.into()),
        ];
        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &expected);
        }
    }

    #[test]
    fn it_evalutaes_boolean_infix_expressions() {
        let tests = vec![
            ("true == true", true),
            ("true == false", false),
            ("true != true", false),
            ("true != false", true),
            ("false == false", true),
            ("false == true", false),
            ("false != false", false),
            ("false != true", true),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Object::Boolean(expected));
        }
    }

    // #[test]
    // fn it_evaluates_string_infix_expressions() {
    //     let tests = vec![
    //         (
    //             r#""Hello" + " " + "World""#,
    //             "Hello World".to_string().into(),
    //         ),
    //         (r#""Hello" == "Hello""#, true.into()),
    //         (r#""Hello" != "Hello""#, false.into()),
    //         (r#""Hello" == "World""#, false.into()),
    //         (r#""Hello" != "World""#, true.into()),
    //     ];

    //     for (input, expected) in tests {
    //         let evaluated = test_eval(input.to_string());
    //         test_object_is_expected(&evaluated, &expected);
    //     }
    // }
    //

    #[test]
    fn it_evaluates_if_else_expressions() {
        let tests = vec![
            ("if (true) { 10 }", 10.into()),
            ("if (false) { 10 }", Object::Null),
            ("if (1) { 10 }", 10.into()),
            ("if (1 < 2) { 10 }", 10.into()),
            ("if (1 > 2) { 10 }", Object::Null),
            ("if (1 > 2) { 10 } else { 20 }", 20.into()),
            ("if (1 < 2) { 10 } else { 20 }", 10.into()),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &expected);
        }
    }

    #[test]
    fn it_evaluates_return_statements() {
        let tests = vec![
            ("return 10;", Box::new(10.into())),
            // ("return 10; 9;", 10.into()),
            // ("return 2 * 5; 9;", 10.into()),
            // ("9; return 2 * 5; 9;", 10.into()),
            // (
            //     r#"
            // if (10 > 1) {
            //     if (10 > 1) {
            //         return 10;
            //     }
            //     return 1;
            //     }
            // "#,
            //     10.into(),
            // ),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Object::ReturnValue(expected));
        }
    }
}
