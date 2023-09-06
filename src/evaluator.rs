pub mod builtin;
pub mod environment;
pub mod error;
pub mod object;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use self::builtin::Builtin;
use self::environment::{Env, Environment};
use self::error::EvaluatorError;
use self::object::Object;
use crate::{parser::ast::*, token::Token};

pub fn evaluate(node: Node, env: Env) -> Result<Rc<Object>, EvaluatorError> {
    match node {
        Node::Program(program) => evaluate_statements(&program, env),
        Node::Statement(statement) => evaluate_statement(&statement, env),
        Node::Expression(expression) => evaluate_expression(&expression, env),
    }
}

fn evaluate_statements(
    statements: &Vec<Statement>,
    env: Env,
) -> Result<Rc<Object>, EvaluatorError> {
    let mut result = Rc::new(Object::Null);

    for statement in statements {
        let intermediate_value = evaluate_statement(statement, Rc::clone(&env))?;

        match *intermediate_value {
            Object::ReturnValue(_) => return Ok(intermediate_value),
            _ => result = intermediate_value,
        }
    }
    Ok(result)
}

fn evaluate_statement(statement: &Statement, env: Env) -> Result<Rc<Object>, EvaluatorError> {
    match statement {
        Statement::Let(name, expression) => {
            let value = evaluate_expression(expression, Rc::clone(&env))?;
            let object = Rc::clone(&value);
            env.borrow_mut().set(name.to_string(), object);
            return Ok(value);
        }
        Statement::Return(expression) => {
            let value = evaluate_expression(expression, Rc::clone(&env))?;
            return Ok(Rc::new(Object::ReturnValue(value)));
        }
        Statement::Expression(expression) => evaluate_expression(expression, env),
    }
}

fn is_truthy(object: &Object) -> bool {
    match object {
        Object::Null => false,
        Object::Boolean(false) => false,
        _ => true,
    }
}

fn evaluate_expression(expression: &Expression, env: Env) -> Result<Rc<Object>, EvaluatorError> {
    match expression {
        Expression::Identifier(identifier) => evaluate_identifier(identifier, Rc::clone(&env)),
        Expression::Literal(literal) => evaluate_literal(literal, Rc::clone(&env)),
        Expression::Prefix(operator, expression) => {
            let right = evaluate_expression(expression, env)?;
            evaluate_prefix_expression(operator, &right)
        }
        Expression::Infix(left, operator, right) => {
            let left = evaluate_expression(left, Rc::clone(&env))?;
            let right = evaluate_expression(right, Rc::clone(&env))?;
            evaluate_infix_expression(operator, &left, &right)
        }

        Expression::If(condition, consequence, alternative) => {
            let condition = evaluate_expression(condition, Rc::clone(&env))?;
            if is_truthy(&condition) {
                evaluate_block_statement(&consequence, Rc::clone(&env))
            } else if let Some(alternative) = alternative {
                evaluate_block_statement(&alternative, Rc::clone(&env))
            } else {
                Ok(Rc::new(Object::Null))
            }
        }
        Expression::Function(parameters, body) => Ok(Rc::new(Object::Function(
            parameters.clone(),
            body.clone(),
            Rc::clone(&env),
        ))),

        Expression::FunctionCall(function, arguments) => {
            // we need to add checking for function here
            println!("function call");
            println!("function: {:?}", function);

            // TODO - do we need to change this to literally work on vec<Expression> because we
            // don't want to evaluate yet
            if **function == Expression::Identifier("quote".to_string()) {
                return Ok(Rc::new(Object::Quote(arguments[0].clone())));
            }
            let function = evaluate_expression(function, Rc::clone(&env))?;
            let arguments = evaluate_expressions(arguments, Rc::clone(&env))?;
            apply_function(Rc::clone(&function), &arguments)
        }

        Expression::Index(left, index) => {
            let left = evaluate_expression(left, Rc::clone(&env))?;
            let index = evaluate_expression(index, Rc::clone(&env))?;
            evaluate_index_expression(&left, &index)
        }
        _ => Ok(Rc::new(Object::Null)),
    }
}

fn evaluate_expressions(
    expressions: &Vec<Expression>,
    env: Env,
) -> Result<Vec<Rc<Object>>, EvaluatorError> {
    let mut result = Vec::new();
    for expression in expressions {
        let evaluated = evaluate_expression(expression, Rc::clone(&env))?;
        result.push(evaluated);
    }
    Ok(result)
}

fn apply_function(
    function: Rc<Object>,
    args: &Vec<Rc<Object>>,
) -> Result<Rc<Object>, EvaluatorError> {
    match &*function {
        Object::Function(parameters, body, env) => {
            let mut env = Environment::new_enclosed_environment(Rc::clone(&env));
            if parameters.len() != args.len() {
                return Err(EvaluatorError::new(format!(
                    "wrong number of arguments: got={}, want={}",
                    args.len(),
                    parameters.len()
                )));
            }
            for (i, parameter) in parameters.iter().enumerate() {
                env.set(parameter.to_string(), Rc::clone(&args[i]));
            }
            let executed = evaluate_block_statement(&body, Rc::new(RefCell::new(env)))?;
            match &*executed {
                Object::ReturnValue(value) => Ok(Rc::clone(value)),
                _ => Ok(executed),
            }
        }
        Object::Builtin(builtin) => builtin.apply(args),
        _ => Err(EvaluatorError::new(format!("not a function: {}", function))),
    }
}

fn evaluate_identifier(identifier: &str, env: Env) -> Result<Rc<Object>, EvaluatorError> {
    match env.borrow().get(identifier) {
        Some(object) => Ok(Rc::clone(&object)),
        None => match Builtin::lookup(identifier) {
            Some(builtin) => Ok(Rc::new(builtin)),
            None => Err(EvaluatorError::new(format!(
                "identifier not found: {}",
                identifier
            ))),
        },
    }
}

fn evaluate_block_statement(
    block: &Vec<Statement>,
    env: Env,
) -> Result<Rc<Object>, EvaluatorError> {
    let mut result = Rc::new(Object::Null);
    for statement in block {
        let intermediate_value = evaluate_statement(statement, Rc::clone(&env))?;
        match *result {
            Object::ReturnValue(_) => return Ok(result),
            _ => result = intermediate_value,
        }
    }
    Ok(result)
}

fn evaluate_literal(literal: &Literal, env: Env) -> Result<Rc<Object>, EvaluatorError> {
    match literal {
        Literal::Integer(integer) => Ok(Rc::new(Object::Integer(*integer))),
        Literal::Boolean(boolean) => Ok(Rc::new(Object::Boolean(*boolean))),
        Literal::String(string) => Ok(Rc::new(Object::String(string.clone()))),
        Literal::Array(elements) => {
            let elements = evaluate_expressions(elements, Rc::clone(&env))?;
            Ok(Rc::new(Object::Array(elements)))
        }
        Literal::Hash(pairs) => {
            let mut hash = HashMap::new();
            for (key, value) in pairs {
                let key = evaluate_expression(key, Rc::clone(&env))?;
                let value = evaluate_expression(value, Rc::clone(&env))?;
                hash.insert(key, value);
            }
            Ok(Rc::new(Object::Hash(hash)))
        }
    }
}

fn evaluate_prefix_expression(
    operator: &Token,
    expression: &Object,
) -> Result<Rc<Object>, EvaluatorError> {
    match operator {
        Token::Bang => evaluate_bang_prefix_operator(expression),
        Token::Dash => evaluate_dash_prefix_operator(expression),
        _ => Ok(Rc::new(Object::Null)),
    }
}

fn evaluate_infix_expression(
    operator: &Token,
    left: &Object,
    right: &Object,
) -> Result<Rc<Object>, EvaluatorError> {
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
            "type mismatch between operands: {} {} {}",
            left, operator, right
        ))),
    }
}

fn evaluate_bang_prefix_operator(expression: &Object) -> Result<Rc<Object>, EvaluatorError> {
    match expression {
        Object::Boolean(b) => Ok(Rc::new(Object::Boolean(!b))),
        Object::Null => Ok(Rc::new(Object::Boolean(true))),
        _ => Ok(Rc::new(Object::Boolean(false))),
    }
}

fn evaluate_dash_prefix_operator(expression: &Object) -> Result<Rc<Object>, EvaluatorError> {
    match expression {
        Object::Integer(i) => Ok(Rc::new(Object::Integer(-i))),
        _ => Err(EvaluatorError::new(format!(
            "unknown operator: -{}",
            expression
        ))),
    }
}

fn evaluate_string_infix_operator(
    operator: &Token,
    left: &String,
    right: &String,
) -> Result<Rc<Object>, EvaluatorError> {
    match operator {
        Token::Plus => {
            let mut string = left.clone();
            string.push_str(right);
            Ok(Rc::new(Object::String(string)))
        }
        Token::Eq => Ok(Rc::new(Object::Boolean(left == right))),
        Token::NotEq => Ok(Rc::new(Object::Boolean(left != right))),

        _ => Err(EvaluatorError::new(format!(
            "unknown operator: {} {} {}",
            left, operator, right
        ))),
    }
}

fn evaluate_boolean_infix_operator(
    operator: &Token,
    left: bool,
    right: bool,
) -> Result<Rc<Object>, EvaluatorError> {
    let result = match operator {
        &Token::Eq => Object::Boolean(left == right),
        &Token::NotEq => Object::Boolean(left != right),
        _ => {
            return Err(EvaluatorError::new(format!(
                "unknown operator: {} {} {}",
                left, operator, right
            )))
        }
    };

    Ok(Rc::new(result))
}

fn evaluate_integer_infix_operator(
    operator: &Token,
    left: i64,
    right: i64,
) -> Result<Rc<Object>, EvaluatorError> {
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
                "unknown operator: {} {} {}",
                left, operator, right
            )))
        }
    };

    Ok(Rc::new(result))
}

fn evaluate_index_expression(left: &Object, index: &Object) -> Result<Rc<Object>, EvaluatorError> {
    match (left, index) {
        (Object::Array(elements), Object::Integer(i)) => {
            let i = *i as usize;
            if i >= elements.len() {
                return Ok(Rc::new(Object::Null));
            }
            Ok(Rc::clone(&elements[i]))
        }
        (Object::Hash(hash), index) => {
            let key = index.clone();
            match hash.get(&key) {
                Some(value) => Ok(Rc::clone(value)),
                None => Ok(Rc::new(Object::Null)),
            }
        }
        _ => Err(EvaluatorError::new(format!(
            "index operator not supported: {}",
            left
        ))),
    }
}

#[cfg(test)]
mod test {

    use std::cell::RefCell;

    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn test_eval(input: String) -> Result<Rc<Object>, EvaluatorError> {
        let l = Lexer::new(input.as_ref());
        let mut p = Parser::new(l);
        let program = p.parse_program();
        evaluate(
            Node::Program(program.unwrap()),
            Rc::new(RefCell::new(Environment::new())),
        )
    }

    fn test_object_is_expected(
        outcome: &Result<Rc<Object>, EvaluatorError>,
        expected: &Result<Rc<Object>, EvaluatorError>,
    ) {
        match (outcome, expected) {
            (Ok(object), Ok(expected_object)) => match (&**object, &**expected_object) {
                (Object::Integer(i), Object::Integer(j)) => assert_eq!(i, j),
                (Object::Boolean(b), Object::Boolean(c)) => assert_eq!(b, c),
                (Object::String(s), Object::String(t)) => assert_eq!(s, t),
                (Object::Null, Object::Null) => assert!(true),
                (Object::ReturnValue(v1), Object::ReturnValue(v2)) => {
                    test_object_is_expected(&Ok(v1.clone()), &Ok(v2.clone()));
                }
                (Object::Array(a), Object::Array(b)) => {
                    assert_eq!(a.len(), b.len());
                    for (i, v) in a.iter().enumerate() {
                        test_object_is_expected(&Ok(v.clone()), &Ok(b[i].clone()));
                    }
                }
                (Object::Hash(a), Object::Hash(b)) => {
                    assert_eq!(a.len(), b.len());
                    for (k, v) in a.iter() {
                        test_object_is_expected(&Ok(v.clone()), &Ok(b[k].clone()));
                    }
                }
                (Object::Quote(a), Object::Quote(b)) => {
                    for (i, v) in a.iter().enumerate() {
                        test_object_is_expected(&Ok(v.clone()), &Ok(b[i].clone()));
                    }
                }
                (_, _) => panic!("unexpected types {:?} and {:?}", object, expected_object),
            },
            (Err(e), Err(expected_err)) => assert_eq!(e.msg, expected_err.msg),
            (_, _) => panic!(
                "mismatched Ok and Err types {:?} and {:?}",
                outcome, expected
            ),
        }
    }

    #[test]
    fn it_evaluates_integer_expressions() {
        let tests = vec![("5", 5), ("10", 10)];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Ok(Rc::new(Object::Integer(expected))));
        }
    }

    #[test]
    fn it_evaluates_boolean_expressions() {
        let tests = vec![("true", true), ("false", false)];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Ok(Rc::new(Object::Boolean(expected))));
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
            test_object_is_expected(&evaluated, &Ok(Rc::new(Object::Boolean(expected))));
        }
    }

    #[test]
    fn it_evaluates_dash_operator() {
        let tests = vec![("-5", -5), ("5", 5), ("-10", -10), ("10", 10)];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Ok(Rc::new(Object::Integer(expected))));
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
            test_object_is_expected(&evaluated, &Ok(Rc::new(expected)));
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
            test_object_is_expected(&evaluated, &Ok(Rc::new(Object::Boolean(expected))));
        }
    }

    #[test]
    fn it_evaluates_string_infix_expressions() {
        let tests = vec![
            (
                r#""Hello" + " " + "World""#,
                "Hello World".to_string().into(),
            ),
            (r#""Hello" == "Hello""#, true.into()),
            (r#""Hello" != "Hello""#, false.into()),
            (r#""Hello" == "World""#, false.into()),
            (r#""Hello" != "World""#, true.into()),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Ok(Rc::new(expected)));
        }
    }

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
            test_object_is_expected(&evaluated, &Ok(Rc::new(expected)));
        }
    }

    #[test]
    fn it_evaluates_return_statements() {
        let tests = vec![
            ("return 10;", Rc::new(10.into())),
            ("return 10; 9;", Rc::new(10.into())),
            ("return 2 * 5; 9;", Rc::new(10.into())),
            ("9; return 2 * 5; 9;", Rc::new(10.into())),
            (
                r#"
             if (10 > 1) {
                 if (10 > 1) {
                     return 10;
                 }
                 return 1;
                 }
             "#,
                Rc::new(10.into()),
            ),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Ok(Rc::new(Object::ReturnValue(expected))));
        }
    }

    #[test]
    fn it_handles_errors_correctly() {
        let tests = vec![
            ("5 + true", "type mismatch between operands: 5 + true"),
            ("5 + true; 5;", "type mismatch between operands: 5 + true"),
            ("-true", "unknown operator: -true"),
            ("true + false", "unknown operator: true + false"),
            ("5; true + false; 5", "unknown operator: true + false"),
            (
                "if (10 > 1) { true + false; }",
                "unknown operator: true + false",
            ),
            (
                r#"
            if (10 > 1) {
                if (10 > 1) {
                    return true + false;
                }
                return 1;
                }
            "#,
                "unknown operator: true + false",
            ),
            ("foobar", "identifier not found: foobar"),
            (r#"len(1)"#, "argument to `len` not supported, got 1"),
            (
                r#"len("one", "two")"#,
                "wrong number of arguments. expected=1, got=2",
            ),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Err(EvaluatorError::new(expected.to_string())));
        }
    }

    #[test]
    fn it_evaluates_let_statement() {
        let tests = vec![
            ("let a = 5; a;", 5.into()),
            ("let a = 5 * 5; a;", 25.into()),
            ("let a = 5; let b = a; b;", 5.into()),
            ("let a = 5; let b = a; let c = a + b + 5; c;", 15.into()),
        ];
        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Ok(Rc::new(expected)));
        }
    }

    #[test]
    fn it_evaluates_functions() {
        let tests = vec![
            ("let identity = fn(x) { x; }; identity(5);", 5.into()),
            ("let identity = fn(x) { return x; }; identity(5);", 5.into()),
            ("let double = fn(x) { x * 2; }; double(5);", 10.into()),
            ("let add = fn(x, y) { x + y; }; add(5, 5);", 10.into()),
            (
                "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
                20.into(),
            ),
            ("fn(x) { x; }(5)", 5.into()),
        ];
        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Ok(Rc::new(expected)));
        }
    }

    #[test]
    fn it_evaluates_closures() {
        let tests = vec![
            (
                r#"
                let intSeq = fn() {
                    let i = 0;
                    return fn() { i = i + 1; };
                };

                let seq = intSeq();
                seq();
                "#,
                1.into(),
            ),
            (
                r#"
                   
               let newAdder = fn(x) {
                 fn(y) { x + y };
                };
               let addTwo = newAdder(2);
               addTwo(2);
               "#,
                4.into(),
            ),
        ];
        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Ok(Rc::new(expected)));
        }
    }

    #[test]
    fn it_evaluates_builtin_len() {
        let test = vec![
            (r#"len("")"#, 0.into()),
            (r#"len("four")"#, 4.into()),
            (r#"len("hello world")"#, 11.into()),
        ];
        for (input, expected) in test {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Ok(Rc::new(expected)));
        }
    }

    #[test]
    fn it_evaluates_array_literal_expressions() {
        let tests = vec![
            ("[1, 2 * 2, 3 + 3]", vec![1, 4, 6]),
            ("[]", vec![]),
            ("[1 + 2, 3 * 4, 5 + 6]", vec![3, 12, 11]),
        ];
        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            let expected_objects = expected
                .into_iter()
                .map(|i| Rc::new(Object::Integer(i)))
                .collect();
            test_object_is_expected(&evaluated, &Ok(Rc::new(Object::Array(expected_objects))));
        }
    }

    #[test]
    fn it_evaluates_array_index_expressions() {
        let tests = vec![
            ("[1, 2, 3][0]", 1),
            ("[1, 2, 3][1]", 2),
            ("[1, 2, 3][2]", 3),
            ("let i = 0; [1][i];", 1),
            ("[1, 2, 3][1 + 1];", 3),
            ("let myArray = [1, 2, 3]; myArray[2];", 3),
            (
                "let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];",
                6,
            ),
            ("let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i]", 2),
            //  ("[1, 2, 3][3]", 0),
            //  ("[1, 2, 3][-1]", 0),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            if expected == 0 {
                test_object_is_expected(&evaluated, &Ok(Rc::new(Object::Null)));
            } else {
                test_object_is_expected(&evaluated, &Ok(Rc::new(Object::Integer(expected))));
            }
        }
    }

    #[test]
    fn it_evaluates_builtin_rest() {
        let tests = vec![
            ("rest([1, 2, 3])", vec![2, 3]),
            ("rest([1])", vec![]),
            ("rest([])", vec![]),
        ];

        test_object_is_expected(
            &test_eval(tests[0].0.to_string()),
            &Ok(Rc::new(Object::Array(vec![
                Rc::new(Object::Integer(2)),
                Rc::new(Object::Integer(3)),
            ]))),
        );
        test_object_is_expected(
            &test_eval(tests[1].0.to_string()),
            &Ok(Rc::new(Object::Array(vec![]))),
        );
        test_object_is_expected(
            &test_eval(tests[2].0.to_string()),
            &Ok(Rc::new(Object::Null)),
        );
    }

    #[test]
    fn it_evaluates_builtin_first() {
        let tests = vec![("first([1, 2, 3])", 1), ("first([1])", 1), ("first([])", 0)];

        test_object_is_expected(
            &test_eval(tests[0].0.to_string()),
            &Ok(Rc::new(Object::Integer(1))),
        );
        test_object_is_expected(
            &test_eval(tests[1].0.to_string()),
            &Ok(Rc::new(Object::Integer(1))),
        );
        test_object_is_expected(
            &test_eval(tests[2].0.to_string()),
            &Ok(Rc::new(Object::Null)),
        );
    }

    #[test]
    fn it_evaluates_builtin_last() {
        let tests = vec![("last([1, 2, 3])", 3), ("last([1])", 1), ("last([])", 0)];

        test_object_is_expected(
            &test_eval(tests[0].0.to_string()),
            &Ok(Rc::new(Object::Integer(3))),
        );
        test_object_is_expected(
            &test_eval(tests[1].0.to_string()),
            &Ok(Rc::new(Object::Integer(1))),
        );
        test_object_is_expected(
            &test_eval(tests[2].0.to_string()),
            &Ok(Rc::new(Object::Null)),
        );
    }

    #[test]
    fn it_evaluates_builtin_push() {
        let tests = vec![
            ("push([], 1)", vec![1]),
            ("push([1], 2)", vec![1, 2]),
            ("push([1, 2], 3)", vec![1, 2, 3]),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input.to_string());
            let expected_objects = expected
                .into_iter()
                .map(|i| Rc::new(Object::Integer(i)))
                .collect();
            test_object_is_expected(&evaluated, &Ok(Rc::new(Object::Array(expected_objects))));
        }
    }

    #[test]
    fn it_evaluates_hash_literals() {
        let tests = vec![(
            r#"let two = "two"
                {
                    "one": 10 - 9,
                    two: 1 + 1,
                    "thr" + "ee": 6 / 2,
                    4: 4,
                    true: 5,
                    false: 6
                }"#,
            vec![
                (Object::String("one".to_string()), Object::Integer(1)),
                (Object::String("two".to_string()), Object::Integer(2)),
                (Object::String("three".to_string()), Object::Integer(3)),
                (Object::Integer(4), Object::Integer(4)),
                (Object::Boolean(true), Object::Integer(5)),
                (Object::Boolean(false), Object::Integer(6)),
            ],
        )];

        for (input, expected) in &tests {
            let evaluated = test_eval(input.to_string());
            let expected_hash: HashMap<Rc<Object>, Rc<Object>> = expected
                .iter()
                .map(|(k, v)| (Rc::new(k.clone()), Rc::new(v.clone())))
                .collect();
            test_object_is_expected(&evaluated, &Ok(Rc::new(Object::Hash(expected_hash))));
        }
    }

    #[test]
    fn it_evaluates_hash_index_expressions() {
        let tests = vec![
            (r#"{"foo": 5}["foo"]"#, Object::Integer(5)),
            (r#"{"foo": 5}["bar"]"#, Object::Null),
            (r#"let key = "foo"; {"foo": 5}[key]"#, Object::Integer(5)),
            (r#"{}["foo"]"#, Object::Null),
            (r#"{5: 5}[5]"#, Object::Integer(5)),
            (r#"{true: 5}[true]"#, Object::Integer(5)),
            (r#"{false: 5}[false]"#, Object::Integer(5)),
        ];

        for (input, expected) in &tests {
            let evaluated = test_eval(input.to_string());
            test_object_is_expected(&evaluated, &Ok(Rc::new(expected.clone())));
        }
    }

    #[test]
    fn it_evaluates_quotes() {
        let tests = vec![
            (r#"quote(5)"#, vec![Rc::new(Object::Integer(5))]),
            (
                r#"quote(5 + 8)"#,
                vec![Rc::new(Object::Quote(vec![
                    Rc::new(Object::Integer(5)),
                    Rc::new(Object::Integer(8)),
                ]))],
            ),
            (
                r#"quote(foobar)"#,
                vec![Rc::new(Object::String("foobar".to_string()))],
            ),
            (
                r#"quote(foobar + barfoo)"#,
                vec![Rc::new(Object::Quote(vec![
                    Rc::new(Object::String("foobar".to_string())),
                    Rc::new(Object::String("barfoo".to_string())),
                ]))],
            ),
        ];

        for (input, expected) in &tests {
            let evaluated = test_eval(input.to_string());

            test_object_is_expected(&evaluated, &Ok(Rc::new(Object::Quote(expected.clone()))));
        }
    }
}
