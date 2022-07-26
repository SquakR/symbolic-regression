/// Expression tree parser module.
/// The parser uses the shunting yard algorithm.
/// https://en.wikipedia.org/wiki/Shunting_yard_algorithm
use crate::expression_tree::{
    BinaryOperation, BinaryOperationKind, ExpressionTree, Node, UnaryOperation, UnaryOperationKind,
    Value,
};
use std::cmp::Ordering;
use std::f64::consts::E;

impl ExpressionTree {
    pub fn parse(expression: &str) -> ExpressionTree {
        ExpressionTree {
            root: Box::new(Node::Value(Value::Constant(0.0))),
            variables: vec![],
        }
    }
}

fn perform_lexical_analysis(expression: &str) -> Vec<Token> {
    let mut tokens = vec![];
    let mut string = String::new();
    for c in expression.chars() {
        if c.is_whitespace() {
            continue;
        }
        if let Some(token) = recognize_symbol(c) {
            if string.len() != 0 {
                tokens.push(recognize_string(&string));
                string = String::new();
            }
            tokens.push(token);
        } else {
            string.push(c);
        }
    }
    if string.len() != 0 {
        tokens.push(recognize_string(&string));
    }
    tokens
}

fn recognize_string(string: &str) -> Token {
    if let Some(function) = Function::try_parse(string) {
        return Token::Function(function);
    }
    match string.parse::<f64>() {
        Ok(constant) => Token::Constant(constant),
        Err(_) => Token::Variable(string.to_owned()),
    }
}

fn recognize_symbol(c: char) -> Option<Token> {
    if let Some(operator) = Operator::try_parse(c) {
        return Some(Token::Operator(operator));
    }
    match c {
        '(' => Some(Token::OpeningBracket),
        ')' => Some(Token::CloseBracket),
        ',' => Some(Token::Comma),
        _ => None,
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Constant(f64),
    Variable(String),
    Function(Function),
    Operator(Operator),
    OpeningBracket,
    CloseBracket,
    Comma,
}

#[derive(Debug, PartialEq)]
enum Function {
    Sin,
    Arcsin,
    Cos,
    Arccos,
    Tan,
    Arctan,
    Cot,
    Arccot,
    Sinh,
    Arsinh,
    Cosh,
    Arcosh,
    Tanh,
    Artanh,
    Coth,
    Arcoth,
    Log,
    Ln,
    Exp,
    Sqrt,
}

impl Function {
    fn try_parse(string: &str) -> Option<Function> {
        match string.to_lowercase().as_str() {
            "sin" => Some(Function::Sin),
            "arcsin" => Some(Function::Arcsin),
            "cos" => Some(Function::Cos),
            "arccos" => Some(Function::Arccos),
            "tan" => Some(Function::Tan),
            "arctan" => Some(Function::Arctan),
            "cot" => Some(Function::Cot),
            "arccot" => Some(Function::Arccot),
            "sinh" => Some(Function::Sinh),
            "arsinh" => Some(Function::Arsinh),
            "cosh" => Some(Function::Cosh),
            "arcosh" => Some(Function::Arcosh),
            "tanh" => Some(Function::Tanh),
            "artanh" => Some(Function::Artanh),
            "coth" => Some(Function::Coth),
            "arcoth" => Some(Function::Arcoth),
            "log" => Some(Function::Log),
            "ln" => Some(Function::Ln),
            "exp" => Some(Function::Exp),
            "sqrt" => Some(Function::Sqrt),
            _ => None,
        }
    }
    fn to_node(&self, mut arguments: Vec<Node>) -> Result<Node, InvalidArgumentsNumberError> {
        if let Some(unary_operation_kind) = match self {
            Function::Sin => Some(UnaryOperationKind::Sin),
            Function::Arcsin => Some(UnaryOperationKind::Arcsin),
            Function::Cos => Some(UnaryOperationKind::Cos),
            Function::Arccos => Some(UnaryOperationKind::Arccos),
            Function::Tan => Some(UnaryOperationKind::Tan),
            Function::Arctan => Some(UnaryOperationKind::Arctan),
            Function::Cot => Some(UnaryOperationKind::Cot),
            Function::Arccot => Some(UnaryOperationKind::Arccot),
            Function::Sinh => Some(UnaryOperationKind::Sinh),
            Function::Arsinh => Some(UnaryOperationKind::Arsinh),
            Function::Cosh => Some(UnaryOperationKind::Cosh),
            Function::Arcosh => Some(UnaryOperationKind::Arcosh),
            Function::Tanh => Some(UnaryOperationKind::Tanh),
            Function::Artanh => Some(UnaryOperationKind::Artanh),
            Function::Coth => Some(UnaryOperationKind::Coth),
            Function::Arcoth => Some(UnaryOperationKind::Arcoth),
            _ => None,
        } {
            if arguments.len() != 1 {
                return Err(InvalidArgumentsNumberError {
                    expected: 1,
                    actual: arguments.len() as u8,
                });
            }
            return Ok(Node::UnaryOperation(UnaryOperation {
                kind: unary_operation_kind,
                argument: Box::new(arguments.remove(0)),
            }));
        };
        if let Some(binary_operation_kind) = match self {
            Function::Ln => Some(BinaryOperationKind::Logarithm),
            Function::Exp => Some(BinaryOperationKind::Power),
            _ => None,
        } {
            if arguments.len() != 1 {
                return Err(InvalidArgumentsNumberError {
                    expected: 1,
                    actual: arguments.len() as u8,
                });
            }
            return Ok(Node::BinaryOperation(BinaryOperation {
                kind: binary_operation_kind,
                first_argument: Box::new(Node::Value(Value::Constant(E))),
                second_argument: Box::new(arguments.remove(0)),
            }));
        }
        match self {
            Function::Log => {
                if arguments.len() != 2 {
                    return Err(InvalidArgumentsNumberError {
                        expected: 2,
                        actual: arguments.len() as u8,
                    });
                }
                return Ok(Node::BinaryOperation(BinaryOperation {
                    kind: BinaryOperationKind::Logarithm,
                    first_argument: Box::new(arguments.remove(0)),
                    second_argument: Box::new(arguments.remove(0)),
                }));
            }
            Function::Sqrt => {
                if arguments.len() != 1 {
                    return Err(InvalidArgumentsNumberError {
                        expected: 1,
                        actual: arguments.len() as u8,
                    });
                }
                return Ok(Node::BinaryOperation(BinaryOperation {
                    kind: BinaryOperationKind::Power,
                    first_argument: Box::new(arguments.remove(0)),
                    second_argument: Box::new(Node::Value(Value::Constant(0.5_f64))),
                }));
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Operator {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Circumflex,
}

impl Operator {
    fn get_precedence(&self) -> u8 {
        match self {
            Operator::Plus | Operator::Minus => 1,
            Operator::Asterisk | Operator::Slash => 2,
            Operator::Circumflex => 3,
        }
    }
    fn try_parse(c: char) -> Option<Operator> {
        match c {
            '+' => Some(Operator::Plus),
            '-' => Some(Operator::Minus),
            '*' => Some(Operator::Asterisk),
            '/' => Some(Operator::Slash),
            '^' => Some(Operator::Circumflex),
            _ => None,
        }
    }
    fn to_node(&self, first_argument: Node, second_argument: Node) -> Node {
        let binary_operation_kind = match self {
            Operator::Plus => BinaryOperationKind::Addition,
            Operator::Minus => BinaryOperationKind::Subtraction,
            Operator::Asterisk => BinaryOperationKind::Multiplication,
            Operator::Slash => BinaryOperationKind::Division,
            Operator::Circumflex => BinaryOperationKind::Power,
        };
        Node::BinaryOperation(BinaryOperation {
            kind: binary_operation_kind,
            first_argument: Box::new(first_argument),
            second_argument: Box::new(second_argument),
        })
    }
}

impl Ord for Operator {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_precedence().cmp(&other.get_precedence())
    }
}

impl PartialOrd for Operator {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct InvalidArgumentsNumberError {
    expected: u8,
    actual: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_ordering() {
        assert!(Operator::Plus <= Operator::Minus);
        assert!(Operator::Minus <= Operator::Plus);
        assert!(Operator::Asterisk <= Operator::Slash);
        assert!(Operator::Slash <= Operator::Asterisk);
        assert!(Operator::Plus < Operator::Asterisk);
        assert!(Operator::Asterisk < Operator::Circumflex);
    }

    #[test]
    fn test_operator_try_parse() {
        for (c, expected_operator) in [
            ('+', Operator::Plus),
            ('-', Operator::Minus),
            ('*', Operator::Asterisk),
            ('/', Operator::Slash),
            ('^', Operator::Circumflex),
        ] {
            match Operator::try_parse(c) {
                Some(actual_operator) => assert_eq!(expected_operator, actual_operator),
                None => panic!(
                    "The character '{}' must be {:?}, but the actual value returned is None.",
                    c, expected_operator
                ),
            }
        }
        if let Some(operator) = Operator::try_parse('w') {
            panic!(
                "The character 'w' is not an operator, but the actual value returned is {:?}.",
                operator
            );
        }
    }

    #[test]
    fn test_operator_to_node() {
        for (operator, expected_binary_operation_kind) in [
            (Operator::Plus, BinaryOperationKind::Addition),
            (Operator::Minus, BinaryOperationKind::Subtraction),
            (Operator::Asterisk, BinaryOperationKind::Multiplication),
            (Operator::Slash, BinaryOperationKind::Division),
            (Operator::Circumflex, BinaryOperationKind::Power),
        ] {
            let expected_node = Node::BinaryOperation(BinaryOperation {
                kind: expected_binary_operation_kind,
                first_argument: Box::new(Node::Value(Value::Constant(1.0))),
                second_argument: Box::new(Node::Value(Value::Constant(2.0))),
            });
            let actual_node = operator.to_node(
                Node::Value(Value::Constant(1.0)),
                Node::Value(Value::Constant(2.0)),
            );
            assert_eq!(expected_node, actual_node);
        }
    }

    #[test]
    fn test_function_try_parse() {
        for (string, expected_function) in [
            ("sin", Function::Sin),
            ("arcsin", Function::Arcsin),
            ("cos", Function::Cos),
            ("arccos", Function::Arccos),
            ("tan", Function::Tan),
            ("arctan", Function::Arctan),
            ("cot", Function::Cot),
            ("arccot", Function::Arccot),
            ("sinh", Function::Sinh),
            ("arsinh", Function::Arsinh),
            ("cosh", Function::Cosh),
            ("arcosh", Function::Arcosh),
            ("tanh", Function::Tanh),
            ("artanh", Function::Artanh),
            ("coth", Function::Coth),
            ("arcoth", Function::Arcoth),
            ("log", Function::Log),
            ("ln", Function::Ln),
            ("exp", Function::Exp),
            ("sqrt", Function::Sqrt),
        ] {
            match Function::try_parse(string) {
                Some(actual_function) => assert_eq!(expected_function, actual_function),
                None => panic!(
                    "The string \"{}\" must be {:?}, but the actual value returned is None.",
                    string, expected_function
                ),
            }
        }
        match Function::try_parse("SiN") {
            Some(actual_function) => assert_eq!(Function::Sin, actual_function),
            None => panic!(
                "The string \"SiN\" must be {:?}, but the actual value returned is None.",
                Function::Sin
            ),
        }
        if let Some(function) = Function::try_parse("fn") {
            panic!(
                "The string \"fn\" is not a function, but the actual value returned is {:?}.",
                function
            );
        }
    }

    #[test]
    fn test_function_to_node_main() {
        for (function, expected_unary_operation_kind) in [
            (Function::Sin, UnaryOperationKind::Sin),
            (Function::Arcsin, UnaryOperationKind::Arcsin),
            (Function::Cos, UnaryOperationKind::Cos),
            (Function::Arccos, UnaryOperationKind::Arccos),
            (Function::Tan, UnaryOperationKind::Tan),
            (Function::Arctan, UnaryOperationKind::Arctan),
            (Function::Cot, UnaryOperationKind::Cot),
            (Function::Arccot, UnaryOperationKind::Arccot),
            (Function::Sinh, UnaryOperationKind::Sinh),
            (Function::Arsinh, UnaryOperationKind::Arsinh),
            (Function::Cosh, UnaryOperationKind::Cosh),
            (Function::Arcosh, UnaryOperationKind::Arcosh),
            (Function::Tanh, UnaryOperationKind::Tanh),
            (Function::Artanh, UnaryOperationKind::Artanh),
            (Function::Coth, UnaryOperationKind::Coth),
            (Function::Arcoth, UnaryOperationKind::Arcoth),
        ] {
            let expected_node = Node::UnaryOperation(UnaryOperation {
                kind: expected_unary_operation_kind,
                argument: Box::new(Node::Value(Value::Constant(1.0))),
            });
            match function.to_node(vec![Node::Value(Value::Constant(1.0))]) {
                Ok(node) => assert_eq!(expected_node, node),
                Err(err) => panic!(
                    "The node was expected for the {:?} function and one argument, but the actual value returned is {:?}.",
                    function, err
                ),
            }
            match function.to_node(vec![Node::Value(Value::Constant(1.0)), Node::Value(Value::Constant(2.0))]) {
                Ok(node) => panic!(
                    "The error expected for the {:?} function and two arguments, but the actual value returned is {:?}.",
                    function, node
                ),
                Err(err) => assert_eq!(
                    InvalidArgumentsNumberError {
                        expected: 1,
                        actual: 2
                    },
                    err
                ),
            }
        }
    }

    #[test]
    fn test_function_to_node_ln_exp() {
        for (function, expected_binary_operation_kind) in [
            (Function::Ln, BinaryOperationKind::Logarithm),
            (Function::Exp, BinaryOperationKind::Power),
        ] {
            let expected_node = Node::BinaryOperation(BinaryOperation {
                kind: expected_binary_operation_kind,
                first_argument: Box::new(Node::Value(Value::Constant(E))),
                second_argument: Box::new(Node::Value(Value::Constant(1.0))),
            });
            match function.to_node(vec![Node::Value(Value::Constant(1.0))]) {
                Ok(node) => assert_eq!(expected_node, node),
                Err(err) => panic!(
                    "The node was expected for the {:?} function and one argument, but the actual value returned is {:?}.",
                    function, err
                )
            }
            match function.to_node(vec![Node::Value(Value::Constant(1.0)), Node::Value(Value::Constant(2.0))]) {
                Ok(node) => panic!(
                    "The error expected for the {:?} function and two arguments, but the actual value returned is {:?}.",
                    function, node
                ),
                Err(err) => assert_eq!(
                    InvalidArgumentsNumberError {
                        expected: 1,
                        actual: 2
                    },
                    err
                ),
            }
        }
    }

    #[test]
    fn test_function_to_node_log() {
        let expected_node = Node::BinaryOperation(BinaryOperation {
            kind: BinaryOperationKind::Logarithm,
            first_argument: Box::new(Node::Value(Value::Constant(1.0))),
            second_argument: Box::new(Node::Value(Value::Constant(2.0))),
        });
        match Function::Log.to_node(vec![Node::Value(Value::Constant(1.0)), Node::Value(Value::Constant(2.0))]) {
            Ok(node) => assert_eq!(expected_node, node),
            Err(err) =>  panic!(
                "The node was expected for the {:?} function and two arguments, but the actual value returned is {:?}.",
                Function::Log, err
            )
        }
        match Function::Log.to_node(vec![Node::Value(Value::Constant(1.0))]) {
            Ok(node) => panic!(
                "The error expected for the {:?} function and one argument, but the actual value returned is {:?}.",
                Function::Log, node
            ),
            Err(err) => assert_eq!(
                InvalidArgumentsNumberError {
                    expected: 2,
                    actual: 1
                },
                err
            ),
        }
    }

    #[test]
    fn test_function_to_node_sqrt() {
        let expected_node = Node::BinaryOperation(BinaryOperation {
            kind: BinaryOperationKind::Power,
            first_argument: Box::new(Node::Value(Value::Constant(2.0))),
            second_argument: Box::new(Node::Value(Value::Constant(0.5))),
        });
        match Function::Sqrt.to_node(vec![Node::Value(Value::Constant(2.0))]) {
            Ok(node) => assert_eq!(expected_node, node),
            Err(err) => panic!(
                "The node was expected for the {:?} function and one argument, but the actual value returned is {:?}.",
                Function::Sqrt, err
            )
        }
        match Function::Sqrt.to_node(vec![Node::Value(Value::Constant(1.0)), Node::Value(Value::Constant(2.0))]) {
            Ok(node) => panic!(
                "The error expected for the {:?} function and two arguments, but the actual value returned is {:?}.",
                Function::Sqrt, node
            ),
            Err(err) => assert_eq!(
                InvalidArgumentsNumberError {
                    expected: 1,
                    actual: 2
                },
                err
            ),
        }
    }

    #[test]
    fn test_recognize_symbol() {
        match recognize_symbol('+') {
            Some(token) => assert_eq!(Token::Operator(Operator::Plus), token),
            None => panic!(
                "The character '+' must bu {:?}, but the actual value returned is None.",
                Token::Operator(Operator::Plus)
            ),
        }
        for (c, expected_token) in [
            ('(', Token::OpeningBracket),
            (')', Token::CloseBracket),
            (',', Token::Comma),
        ] {
            match recognize_symbol(c) {
                Some(actual_token) => assert_eq!(expected_token, actual_token),
                None => panic!(
                    "The character '{}' must be {:?}, but the actual value returned is None.",
                    c, expected_token
                ),
            }
        }
        if let Some(token) = recognize_symbol('w') {
            panic!(
                "The character 'w' is not an token, but the actual value returned is {:?}.",
                token
            );
        }
    }

    #[test]
    fn test_recognize_string() {
        assert_eq!(Token::Function(Function::Sin), recognize_string("sin"));
        assert_eq!(Token::Constant(1.0), recognize_string("1.0"));
        assert_eq!(Token::Variable(String::from("x1")), recognize_string("x1"));
    }

    #[test]
    fn test_perform_lexical_analysis() {
        assert_eq!(
            vec![
                Token::Function(Function::Log),
                Token::OpeningBracket,
                Token::Constant(2.0),
                Token::Comma,
                Token::Variable(String::from("x")),
                Token::CloseBracket,
                Token::Operator(Operator::Plus),
                Token::Function(Function::Cos),
                Token::OpeningBracket,
                Token::Constant(0.0),
                Token::CloseBracket,
                Token::Operator(Operator::Minus),
                Token::Variable(String::from("x")),
            ],
            perform_lexical_analysis("log(2.0, x) + cos(0.0) - x")
        );
    }

    #[test]
    fn test_parse() {
        assert_eq!(0.0, 0.0);
    }
}
