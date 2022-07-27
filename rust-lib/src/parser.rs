/// Expression tree parser module.
/// The parser uses the shunting yard algorithm.
/// https://en.wikipedia.org/wiki/Shunting_yard_algorithm
use crate::expression_tree::{
    BinaryOperation, BinaryOperationKind, ExpressionTree, Node, UnaryOperation, UnaryOperationKind,
    Value,
};
use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};
use std::f64::consts::E;
use std::rc::Rc;

impl ExpressionTree {
    pub fn parse(expression: &str) -> Result<ExpressionTree, ParseError> {
        Parser::parse(expression)
    }
}

struct Parser {
    expression: String,
    tokens: Vec<Rc<Token>>,
    queue: VecDeque<Node>,
    stack: Vec<Rc<Token>>,
    variables: HashSet<String>,
}

impl Parser {
    fn parse(expression: &str) -> Result<ExpressionTree, ParseError> {
        let mut parser = Parser::new(expression);
        parser.perform_lexical_analysis();
        parser.handle_tokens()?;
        Ok(ExpressionTree {
            root: Box::new(Node::Value(Value::Constant(0.0))),
            variables: vec![],
        })
    }
    fn new(expression: &str) -> Parser {
        Parser {
            expression: expression.to_owned(),
            tokens: vec![],
            queue: VecDeque::new(),
            stack: vec![],
            variables: HashSet::new(),
        }
    }
    fn perform_lexical_analysis(&mut self) {
        let mut string = String::new();
        for (i, c) in self.expression.chars().enumerate() {
            if c.is_whitespace() {
                continue;
            }
            if let Some(token) = Parser::recognize_symbol(c, i) {
                if string.len() != 0 {
                    self.tokens
                        .push(Rc::new(Parser::recognize_string(&string, i - string.len())));
                    string = String::new();
                }
                self.tokens.push(Rc::new(token));
            } else {
                string.push(c);
            }
        }
        if string.len() != 0 {
            self.tokens.push(Rc::new(Parser::recognize_string(
                &string,
                self.expression.len() - string.len(),
            )));
        }
    }
    fn recognize_string(string: &str, position: usize) -> Token {
        if let Some(function) = Function::try_parse(string) {
            return Token::Function(TokenValue {
                value: function,
                string: string.to_owned(),
                position,
            });
        }
        match string.parse::<f64>() {
            Ok(constant) => Token::Constant(TokenValue {
                value: constant,
                string: string.to_owned(),
                position,
            }),
            Err(_) => Token::Variable(TokenValue {
                value: string.to_owned(),
                string: string.to_owned(),
                position,
            }),
        }
    }
    fn recognize_symbol(c: char, position: usize) -> Option<Token> {
        if let Some(operator) = Operator::try_parse(c) {
            return Some(Token::Operator(TokenValue {
                value: operator,
                string: c.to_string(),
                position,
            }));
        }
        match c {
            '(' => Some(Token::OpeningBracket(TokenValue {
                value: (),
                string: c.to_string(),
                position,
            })),
            ')' => Some(Token::CloseBracket(TokenValue {
                value: (),
                string: c.to_string(),
                position,
            })),
            ',' => Some(Token::Comma(TokenValue {
                value: (),
                string: c.to_string(),
                position,
            })),
            _ => None,
        }
    }
    fn handle_tokens(&mut self) -> Result<(), ParseError> {
        let tokens_rcs = self.tokens.iter().cloned().collect::<Vec<Rc<Token>>>();
        for token in tokens_rcs {
            match &*token {
                Token::Constant(_) => {
                    self.handle_constant(token)?;
                }
                Token::Variable(_) => {
                    self.handle_variable(token)?;
                }
                Token::Function(_) => {
                    self.handle_function(token);
                }
                Token::Comma(_) => {
                    self.handle_comma(token)?;
                }
                Token::Operator(_) => {
                    self.handle_operator(token)?;
                }
                Token::OpeningBracket(_) => {
                    self.handle_opening_bracket(token);
                }
                Token::CloseBracket(_) => {
                    self.handle_close_bracket(token)?;
                }
            }
        }
        self.shift_all()?;
        Ok(())
    }
    fn handle_constant(&mut self, token: Rc<Token>) -> Result<(), ParseError> {
        match self.push_token(token) {
            Err(err) => Err(ParseError::InvalidArgumentsNumberError(err)),
            Ok(_) => Ok(()),
        }
    }
    fn handle_variable(&mut self, token: Rc<Token>) -> Result<(), ParseError> {
        let value = match &*token {
            Token::Variable(token_value) => token_value.value.to_owned(),
            _ => unreachable!(),
        };
        self.variables.insert(value.to_owned());
        match self.push_token(token) {
            Err(err) => Err(ParseError::InvalidArgumentsNumberError(err)),
            Ok(_) => Ok(()),
        }
    }
    fn handle_function(&mut self, token: Rc<Token>) {
        self.stack.push(token);
    }
    fn handle_comma(&mut self, token: Rc<Token>) -> Result<(), ParseError> {
        self.shift_while_opening_bracket(token)
    }
    fn handle_operator(&mut self, token: Rc<Token>) -> Result<(), ParseError> {
        let value = match &*token {
            Token::Operator(token_value) => token_value.value.to_owned(),
            _ => unreachable!(),
        };
        if self.stack.len() > 0 {
            loop {
                if let Token::Operator(token_value_o2) = &*self.stack[self.stack.len() - 1] {
                    if token_value_o2.value.is_computed_before(&value) {
                        let last_token = self.stack.pop().unwrap();
                        if let Err(err) = self.push_token(last_token) {
                            return Err(ParseError::InvalidArgumentsNumberError(err));
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        self.stack.push(token);
        Ok(())
    }
    fn handle_opening_bracket(&mut self, token: Rc<Token>) {
        self.stack.push(token);
    }
    fn handle_close_bracket(&mut self, token: Rc<Token>) -> Result<(), ParseError> {
        self.shift_while_opening_bracket(token)?;
        self.stack.pop();
        if self.stack.len() > 0 {
            if let Token::Function(_) = &*self.stack[self.stack.len() - 1] {
                let last_token = self.stack.pop().unwrap();
                if let Err(err) = self.push_token(last_token) {
                    return Err(ParseError::InvalidArgumentsNumberError(err));
                }
            }
        }
        Ok(())
    }
    fn push_token(&mut self, token: Rc<Token>) -> Result<(), InvalidArgumentsNumberError> {
        match &*token {
            Token::Constant(token_value) => Ok(self
                .queue
                .push_front(Node::Value(Value::Constant(token_value.value)))),
            Token::Variable(token_value) => Ok(self
                .queue
                .push_front(Node::Value(Value::Variable(token_value.value.to_owned())))),
            Token::Function(token_value) => {
                let arguments = self.queue.split_off(0).into_iter().collect::<Vec<Node>>();
                match token_value.value.to_node(arguments) {
                    Ok(node) => Ok(self.queue.push_front(node)),
                    Err(err) => Err(InvalidArgumentsNumberError {
                        token: (&*token).clone(),
                        expected: err.expected,
                        actual: err.actual,
                    }),
                }
            }
            Token::Operator(token_value) => {
                let mut arguments = self.queue.split_off(0).into_iter().collect::<Vec<Node>>();
                if arguments.len() != 2 {
                    return Err(InvalidArgumentsNumberError {
                        token: (&*token).clone(),
                        expected: 2,
                        actual: arguments.len() as u8,
                    });
                }
                let node = token_value
                    .value
                    .to_node(arguments.remove(0), arguments.remove(0));
                Ok(self.queue.push_front(node))
            }
            _ => Ok(()),
        }
    }
    fn shift_while_opening_bracket(&mut self, token: Rc<Token>) -> Result<(), ParseError> {
        let mut tokens = VecDeque::new();
        loop {
            if self.stack.len() == 0 {
                match *token {
                    Token::Comma(_) => {
                        return Err(ParseError::MissingCommaOrOpeningParenthesisError(
                            MissingCommaOrOpeningParenthesisError {
                                token: (&*token).clone(),
                            },
                        ))
                    }
                    Token::CloseBracket(_) => {
                        return Err(ParseError::MissionCommaError({
                            MissionCommaError {
                                token: (&*token).clone(),
                            }
                        }))
                    }
                    _ => unreachable!(),
                }
            }
            if let Token::OpeningBracket(_) = *self.stack[self.stack.len() - 1] {
                break;
            }
            tokens.push_back(self.stack.pop().unwrap());
        }
        for token in tokens {
            if let Err(err) = self.push_token(token) {
                return Err(ParseError::InvalidArgumentsNumberError(err));
            }
        }
        Ok(())
    }
    fn shift_all(&mut self) -> Result<(), ParseError> {
        loop {
            if self.stack.len() == 0 {
                return Ok(());
            }
            if let Token::OpeningBracket(_) = *self.stack[self.stack.len() - 1] {
                return Err(ParseError::MissionCommaError(MissionCommaError {
                    token: (&*self.stack[self.stack.len() - 1]).clone(),
                }));
            }
            let last_token = self.stack.pop().unwrap();
            if let Err(err) = self.push_token(last_token) {
                return Err(ParseError::InvalidArgumentsNumberError(err));
            }
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    MissingCommaOrOpeningParenthesisError(MissingCommaOrOpeningParenthesisError),
    MissionCommaError(MissionCommaError),
    InvalidArgumentsNumberError(InvalidArgumentsNumberError),
}

#[derive(Debug)]
pub struct MissingCommaOrOpeningParenthesisError {
    token: Token,
}

#[derive(Debug)]
pub struct MissionCommaError {
    token: Token,
}

#[derive(Debug)]
pub struct InvalidArgumentsNumberError {
    token: Token,
    expected: u8,
    actual: u8,
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Constant(TokenValue<f64>),
    Variable(TokenValue<String>),
    Function(TokenValue<Function>),
    Operator(TokenValue<Operator>),
    OpeningBracket(TokenValue<()>),
    CloseBracket(TokenValue<()>),
    Comma(TokenValue<()>),
}

#[derive(Debug, Clone, PartialEq)]
struct TokenValue<T> {
    value: T,
    string: String,
    position: usize,
}

#[derive(Debug, Clone, PartialEq)]
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
    fn to_node(
        &self,
        mut arguments: Vec<Node>,
    ) -> Result<Node, InternalInvalidArgumentsNumberError> {
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
                return Err(InternalInvalidArgumentsNumberError {
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
                return Err(InternalInvalidArgumentsNumberError {
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
                    return Err(InternalInvalidArgumentsNumberError {
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
                    return Err(InternalInvalidArgumentsNumberError {
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

#[derive(Debug, Clone, Eq, PartialEq)]
enum Operator {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Circumflex,
}

enum Associativity {
    Left,
    Right,
}

impl Operator {
    fn get_precedence(&self) -> u8 {
        match self {
            Operator::Plus | Operator::Minus => 1,
            Operator::Asterisk | Operator::Slash => 2,
            Operator::Circumflex => 3,
        }
    }
    fn get_associativity(&self) -> Associativity {
        match self {
            Operator::Plus | Operator::Minus | Operator::Asterisk | Operator::Slash => {
                Associativity::Left
            }
            Operator::Circumflex => Associativity::Right,
        }
    }
    fn is_computed_before(&self, other: &Operator) -> bool {
        match self.cmp(other) {
            Ordering::Equal => match other.get_associativity() {
                Associativity::Left => true,
                Associativity::Right => false,
            },
            Ordering::Greater => true,
            Ordering::Less => false,
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
struct InternalInvalidArgumentsNumberError {
    expected: u8,
    actual: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod operator_tests {
        use super::*;

        #[test]
        fn test_ordering() {
            assert!(Operator::Plus <= Operator::Minus);
            assert!(Operator::Minus <= Operator::Plus);
            assert!(Operator::Asterisk <= Operator::Slash);
            assert!(Operator::Slash <= Operator::Asterisk);
            assert!(Operator::Plus < Operator::Asterisk);
            assert!(Operator::Asterisk < Operator::Circumflex);
        }

        #[test]
        fn test_is_computed_before() {
            assert!(Operator::Plus.is_computed_before(&Operator::Plus));
            assert!(Operator::Plus.is_computed_before(&Operator::Minus));
            assert!(!Operator::Plus.is_computed_before(&Operator::Asterisk));
            assert!(Operator::Circumflex.is_computed_before(&Operator::Slash));
        }

        #[test]
        fn test_try_parse() {
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
        fn test_to_node() {
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
    }

    mod function_tests {
        use super::*;

        #[test]
        fn test_try_parse() {
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
        fn test_to_node_main() {
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
                    InternalInvalidArgumentsNumberError {
                        expected: 1,
                        actual: 2
                    },
                    err
                ),
            }
            }
        }

        #[test]
        fn test_to_node_ln_exp() {
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
                    InternalInvalidArgumentsNumberError {
                        expected: 1,
                        actual: 2
                    },
                    err
                ),
            }
            }
        }

        #[test]
        fn test_to_node_log() {
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
                InternalInvalidArgumentsNumberError {
                    expected: 2,
                    actual: 1
                },
                err
            ),
        }
        }

        #[test]
        fn test_to_node_sqrt() {
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
                InternalInvalidArgumentsNumberError {
                    expected: 1,
                    actual: 2
                },
                err
            ),
        }
        }
    }

    mod parser_tests {
        use super::*;

        #[test]
        fn test_recognize_symbol() {
            let expected_plus = Token::Operator(TokenValue {
                value: Operator::Plus,
                string: String::from("+"),
                position: 5,
            });
            match Parser::recognize_symbol('+', 5) {
                Some(token) => assert_eq!(expected_plus, token),
                None => panic!(
                    "The character '+' must be {:?}, but the actual value returned is None.",
                    expected_plus
                ),
            }
            for (c, token_factory) in [
                ('(', Token::OpeningBracket as fn(TokenValue<()>) -> Token),
                (')', Token::CloseBracket as fn(TokenValue<()>) -> Token),
                (',', Token::Comma as fn(TokenValue<()>) -> Token),
            ] {
                let expected_token = token_factory(TokenValue {
                    value: (),
                    string: c.to_string(),
                    position: 5,
                });
                match Parser::recognize_symbol(c, 5) {
                    Some(actual_token) => assert_eq!(expected_token, actual_token),
                    None => panic!(
                        "The character '{}' must be {:?}, but the actual value returned is None.",
                        c, expected_token
                    ),
                }
            }
            if let Some(token) = Parser::recognize_symbol('w', 5) {
                panic!(
                    "The character 'w' is not an token, but the actual value returned is {:?}.",
                    token
                );
            }
        }

        #[test]
        fn test_recognize_string() {
            assert_eq!(
                Token::Function(TokenValue {
                    value: Function::Sin,
                    string: String::from("sin"),
                    position: 5
                }),
                Parser::recognize_string("sin", 5)
            );
            assert_eq!(
                Token::Constant(TokenValue {
                    value: 1.0,
                    string: String::from("1.0"),
                    position: 5
                }),
                Parser::recognize_string("1.0", 5)
            );
            assert_eq!(
                Token::Variable(TokenValue {
                    value: String::from("x1"),
                    string: String::from("x1"),
                    position: 5
                }),
                Parser::recognize_string("x1", 5)
            );
        }

        #[test]
        fn test_parser_perform_lexical_analysis() {
            let mut parser = Parser::new("log(2.0, x) + cos(0.0) - x");
            parser.perform_lexical_analysis();
            assert_eq!(
                vec![
                    Rc::new(Token::Function(TokenValue {
                        value: Function::Log,
                        string: String::from("log"),
                        position: 0
                    })),
                    Rc::new(Token::OpeningBracket(TokenValue {
                        value: (),
                        string: String::from("("),
                        position: 3
                    })),
                    Rc::new(Token::Constant(TokenValue {
                        value: 2.0,
                        string: String::from("2.0"),
                        position: 4
                    })),
                    Rc::new(Token::Comma(TokenValue {
                        value: (),
                        string: String::from(","),
                        position: 7
                    })),
                    Rc::new(Token::Variable(TokenValue {
                        value: String::from("x"),
                        string: String::from("x"),
                        position: 9
                    })),
                    Rc::new(Token::CloseBracket(TokenValue {
                        value: (),
                        string: String::from(")"),
                        position: 10
                    })),
                    Rc::new(Token::Operator(TokenValue {
                        value: Operator::Plus,
                        string: String::from("+"),
                        position: 12
                    })),
                    Rc::new(Token::Function(TokenValue {
                        value: Function::Cos,
                        string: String::from("cos"),
                        position: 14
                    })),
                    Rc::new(Token::OpeningBracket(TokenValue {
                        value: (),
                        string: String::from("("),
                        position: 17
                    })),
                    Rc::new(Token::Constant(TokenValue {
                        value: 0.0,
                        string: String::from("0.0"),
                        position: 18
                    })),
                    Rc::new(Token::CloseBracket(TokenValue {
                        value: (),
                        string: String::from(")"),
                        position: 21
                    })),
                    Rc::new(Token::Operator(TokenValue {
                        value: Operator::Minus,
                        string: String::from("-"),
                        position: 23
                    })),
                    Rc::new(Token::Variable(TokenValue {
                        value: String::from("x"),
                        string: String::from("x"),
                        position: 25
                    })),
                ],
                parser.tokens
            );
        }
    }

    #[test]
    fn test_parse() {
        assert_eq!(0.0, 0.0);
    }
}
