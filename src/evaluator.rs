pub mod object;

use crate::ast::*;

fn eval(node: Node) -> Object {
    match node {
        Node::Program(program) => eval_statements(program.statements),
        Node::Statement(statement) => eval_statement(statement),
        Node::Expression(expression) => eval_expression(expression),
    }
}

fn eval_statements(statements: Vec<Statement>) -> Object {
    let mut result = Object::Null;
    for statement in statements {
        result = eval_statement(statement);
    }
    result
}

fn eval_statement(statement: &Statement) -> Object {
    match statement {
        Statement::ExpressionStatement(expression) => eval_expression(*expression),
        Statement::ReturnStatement(return_statement) => {
            let val = eval_expression(return_statement.return_value);
            Object::ReturnValue(Box::new(val))
        }
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
        eval(program)
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
