pub mod object;

use self::object::Object;
use crate::parser::ast::*;

pub fn evaluate(node: Node) -> Object {
    match node {
        Node::Program(program) => evaluate_statements(&program),
        Node::Statement(statement) => evaluate_statement(&statement),
        Node::Expression(expression) => evaluate_expression(&expression),
    }
}

fn evaluate_statements(statements: &Vec<Statement>) -> Object {
    let mut result = Object::Null;
    for statement in statements {
        result = evaluate_statement(statement);
    }
    result
}

fn evaluate_statement(statement: &Statement) -> Object {
    match statement {
        Statement::Let(name, expression) => {
            let value = evaluate_expression(expression);
            return value;
        }
        Statement::Return(expression) => {
            let value = evaluate_expression(expression);
            return value;
        }
        Statement::Expression(expression) => evaluate_expression(expression),
    }
}

fn evaluate_expression(expression: &Expression) -> Object {
    match expression {
        Expression::Literal(literal) => evaluate_literal(literal),
        _ => Object::Null,
    }
}

fn evaluate_literal(literal: &Literal) -> Object {
    match literal {
        Literal::Integer(integer) => Object::Integer(*integer),
        Literal::Boolean(boolean) => Object::Boolean(*boolean),
        Literal::String(string) => Object::String(string.clone()),
    }
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
        evaluate(Node::Program(program.unwrap()))
    }

    fn test_integer_value(o: Object, expected: i64) {
        match o {
            Object::Integer(i) => assert_eq!(i, expected),
            _ => panic!("Expected Integer, got {:?}", o),
        }
    }

    #[test]
    fn it_evaluates_integer_expressions() {
        let tests = vec![("5", 5), ("10", 10)];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_integer_value(evaluated, expected);
        }
    }
}
