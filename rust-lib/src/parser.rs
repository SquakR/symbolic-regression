/// Expression tree parser module.
/// The parser uses the shunting yard algorithm.
/// https://en.wikipedia.org/wiki/Shunting_yard_algorithm
use crate::expression_tree::{ExpressionTree, Node, Value};
use std::cmp::Ordering;

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
    Ln,
    Exp,
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
    Sqrt,
    Log,
}

impl Function {
    fn try_parse(string: &str) -> Option<Function> {
        match string.to_lowercase().as_str() {
            "ln" => Some(Function::Ln),
            "exp" => Some(Function::Exp),
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
            "sqrt" => Some(Function::Sqrt),
            "log" => Some(Function::Log),
            _ => None,
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
            Operator::Plus => 1,
            Operator::Minus => 1,
            Operator::Asterisk => 2,
            Operator::Slash => 2,
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
    fn test_try_parse_operator() {
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
    fn test_try_parse_function() {
        for (string, expected_function) in [
            ("ln", Function::Ln),
            ("exp", Function::Exp),
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
            ("sqrt", Function::Sqrt),
            ("log", Function::Log),
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
