//! Expression tree parser module.
//! The parser uses the shunting yard algorithm.
//! https://en.wikipedia.org/wiki/Shunting_yard_algorithm
use crate::expression_tree::{ExpressionTree, Node, OperationNode, ValueNode};
use crate::settings::{OperationCollection, Settings};
use crate::types::{Associativity, Function, Operator};
use std::cmp::Ordering;
use std::collections::{HashSet, VecDeque};
use std::rc::Rc;

impl<'a> ExpressionTree<'a> {
    pub fn parse(
        expression: &str,
        settings: &'a Settings,
    ) -> Result<ExpressionTree<'a>, ParseError<'a>> {
        Parser::parse(expression, settings)
    }
}

struct Parser<'a> {
    expression: String,
    settings: &'a Settings,
    tokens: Vec<Rc<Token<'a>>>,
    queue: VecDeque<Node<'a>>,
    stack: Vec<Rc<Token<'a>>>,
    variables: HashSet<String>,
}

impl<'a> Parser<'a> {
    fn parse(
        expression: &str,
        settings: &'a Settings,
    ) -> Result<ExpressionTree<'a>, ParseError<'a>> {
        if expression.len() == 0 {
            return Err(ParseError::EmptyFormulaError(EmptyFormulaError {}));
        }
        let mut parser = Parser::new(expression, settings);
        parser.perform_lexical_analysis();
        parser.handle_tokens()?;
        if parser.queue.len() != 1 {
            return Err(ParseError::MultipleFormulaError(MultipleFormulaError {}));
        }
        Ok(ExpressionTree {
            root: parser.queue.pop_front().unwrap(),
            variables: parser.variables.iter().cloned().collect::<Vec<String>>(),
        })
    }
    fn new(expression: &str, settings: &'a Settings) -> Parser<'a> {
        Parser {
            expression: expression.to_owned(),
            settings,
            tokens: vec![],
            queue: VecDeque::new(),
            stack: vec![],
            variables: HashSet::new(),
        }
    }
    fn perform_lexical_analysis(&mut self) {
        let mut string = String::new();
        let mut p = 0;
        for (i, c) in self.expression.chars().enumerate() {
            if c.is_whitespace() {
                continue;
            }
            for j in 0..string.len() {
                if let Some(token) = self.recognize_string(&string[j..], p + j) {
                    if j != 0 {
                        self.tokens
                            .push(Rc::new(Parser::recognize_value_string(&string[0..j], p)));
                    }
                    self.tokens.push(Rc::new(token));
                    string = String::new();
                    p = i;
                    break;
                }
            }
            string.push(c);
        }
        if string.len() != 0 {
            self.tokens.push(Rc::new(Parser::recognize_value_string(
                &string,
                self.expression.len() - string.len(),
            )));
        }
    }
    fn recognize_value_string(string: &str, position: usize) -> Token<'a> {
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
    fn recognize_string(&self, string: &str, position: usize) -> Option<Token<'a>> {
        if let Some(operation_token) = self.recognize_operation_string(string, position) {
            return Some(operation_token);
        }
        if let Some(function_token) = Parser::recognize_service_string(string, position) {
            return Some(function_token);
        }
        None
    }
    fn recognize_operation_string(&self, string: &str, position: usize) -> Option<Token<'a>> {
        if let Some(operator) = self.settings.operators.find_by_name(string) {
            return Some(Token::Operator(TokenValue {
                value: operator,
                string: string.to_owned(),
                position,
            }));
        };
        if let Some(function) = self.settings.functions.find_by_name(string) {
            return Some(Token::Function(TokenValue {
                value: function,
                string: string.to_owned(),
                position,
            }));
        }
        None
    }
    fn recognize_service_string(string: &str, position: usize) -> Option<Token<'a>> {
        match string {
            "(" => Some(Token::OpeningBracket(TokenValue {
                value: (),
                string: string.to_owned(),
                position,
            })),
            ")" => Some(Token::CloseBracket(TokenValue {
                value: (),
                string: string.to_owned(),
                position,
            })),
            "," => Some(Token::Comma(TokenValue {
                value: (),
                string: string.to_string(),
                position,
            })),
            _ => None,
        }
    }
    fn handle_tokens(&mut self) -> Result<(), ParseError<'a>> {
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
    fn handle_constant(&mut self, token: Rc<Token<'a>>) -> Result<(), ParseError<'a>> {
        match self.push_token(token) {
            Err(err) => Err(ParseError::InvalidArgumentsNumberError(err)),
            Ok(_) => Ok(()),
        }
    }
    fn handle_variable(&mut self, token: Rc<Token<'a>>) -> Result<(), ParseError<'a>> {
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
    fn handle_function(&mut self, token: Rc<Token<'a>>) {
        self.stack.push(token);
    }
    fn handle_comma(&mut self, token: Rc<Token<'a>>) -> Result<(), ParseError<'a>> {
        self.shift_while_opening_bracket(token)
    }
    fn handle_operator(&mut self, token: Rc<Token<'a>>) -> Result<(), ParseError<'a>> {
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
    fn handle_opening_bracket(&mut self, token: Rc<Token<'a>>) {
        self.stack.push(token);
    }
    fn handle_close_bracket(&mut self, token: Rc<Token<'a>>) -> Result<(), ParseError<'a>> {
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
    fn push_token(&mut self, token: Rc<Token<'a>>) -> Result<(), InvalidArgumentsNumberError<'a>> {
        match &*token {
            Token::Constant(token_value) => Ok(self
                .queue
                .push_back(Node::Value(ValueNode::Constant(token_value.value)))),
            Token::Variable(token_value) => Ok(self.queue.push_back(Node::Value(
                ValueNode::Variable(token_value.value.to_owned()),
            ))),
            Token::Function(token_value) => {
                if self.queue.len() != token_value.value.arguments_number {
                    return Err(InvalidArgumentsNumberError {
                        token: (&*token).clone(),
                        expected: token_value.value.arguments_number,
                        actual: self.queue.len(),
                    });
                }
                let arguments = self.queue.split_off(0).into_iter().collect::<Vec<Node>>();
                let node = Node::Function(OperationNode {
                    operation: token_value.value,
                    arguments,
                });
                Ok(self.queue.push_back(node))
            }
            Token::Operator(token_value) => {
                if self.queue.len() != token_value.value.arguments_number {
                    return Err(InvalidArgumentsNumberError {
                        token: (&*token).clone(),
                        expected: token_value.value.arguments_number,
                        actual: self.queue.len(),
                    });
                }
                let arguments = self.queue.split_off(0).into_iter().collect::<Vec<Node>>();
                let node = Node::Operator(OperationNode {
                    operation: token_value.value,
                    arguments,
                });
                Ok(self.queue.push_back(node))
            }
            _ => Ok(()),
        }
    }
    fn shift_while_opening_bracket(&mut self, token: Rc<Token<'a>>) -> Result<(), ParseError<'a>> {
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
    fn shift_all(&mut self) -> Result<(), ParseError<'a>> {
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
pub enum ParseError<'a> {
    MissingCommaOrOpeningParenthesisError(MissingCommaOrOpeningParenthesisError<'a>),
    MissionCommaError(MissionCommaError<'a>),
    EmptyFormulaError(EmptyFormulaError),
    MultipleFormulaError(MultipleFormulaError),
    InvalidArgumentsNumberError(InvalidArgumentsNumberError<'a>),
}

#[derive(Debug, PartialEq)]
pub struct MissingCommaOrOpeningParenthesisError<'a> {
    token: Token<'a>,
}

#[derive(Debug, PartialEq)]
pub struct MissionCommaError<'a> {
    token: Token<'a>,
}

#[derive(Debug, PartialEq)]
pub struct EmptyFormulaError;

#[derive(Debug, PartialEq)]
pub struct MultipleFormulaError;

#[derive(Debug, PartialEq)]
pub struct InvalidArgumentsNumberError<'a> {
    token: Token<'a>,
    expected: usize,
    actual: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum Token<'a> {
    Constant(TokenValue<f64>),
    Variable(TokenValue<String>),
    Function(TokenValue<&'a Function>),
    Operator(TokenValue<&'a Operator>),
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

impl<'a> Operator {
    fn is_computed_before(&self, other: &Operator) -> bool {
        match self.precedence.cmp(&other.precedence) {
            Ordering::Equal => match other.associativity {
                Associativity::Left => true,
                Associativity::Right => false,
            },
            Ordering::Greater => true,
            Ordering::Less => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings;

    #[test]
    fn test_operator_is_computed_before() {
        let settings = settings::get_default_settings();
        let plus = settings.operators.find_by_name("+").unwrap();
        let minus = settings.operators.find_by_name("-").unwrap();
        let asterisk = settings.operators.find_by_name("*").unwrap();
        let slash = settings.operators.find_by_name("/").unwrap();
        let circumflex = settings.operators.find_by_name("^").unwrap();
        assert!(plus.is_computed_before(plus));
        assert!(plus.is_computed_before(minus));
        assert!(!plus.is_computed_before(asterisk));
        assert!(circumflex.is_computed_before(slash));
    }

    #[test]
    fn test_recognize_value_string_constant() {
        let expected_token = Token::Constant(TokenValue {
            value: 1.0,
            string: String::from("1.0"),
            position: 5,
        });
        assert_eq!(expected_token, Parser::recognize_value_string("1.0", 5));
    }

    #[test]
    fn test_recognize_value_string_variable() {
        let expected_token = Token::Variable(TokenValue {
            value: String::from("x1"),
            string: String::from("x1"),
            position: 5,
        });
        assert_eq!(expected_token, Parser::recognize_value_string("x1", 5));
    }

    #[test]
    fn test_recognize_string_operator() {
        let settings = settings::get_default_settings();
        let parser = Parser::new("", &settings);
        let expected_token = Token::Operator(TokenValue {
            value: settings.operators.find_by_name("+").unwrap(),
            string: String::from("+"),
            position: 5,
        });
        match parser.recognize_string("+", 5) {
            Some(actual_token) => assert_eq!(expected_token, actual_token),
            None => panic!("Expected {:?} but got None.", expected_token),
        }
    }

    #[test]
    fn test_recognize_string_function() {
        let settings = settings::get_default_settings();
        let parser = Parser::new("", &settings);
        let expected_token = Token::Function(TokenValue {
            value: settings.functions.find_by_name("sin").unwrap(),
            string: String::from("sin"),
            position: 5,
        });
        match parser.recognize_string("sin", 5) {
            Some(actual_token) => assert_eq!(expected_token, actual_token),
            None => panic!("Expected {:?} but got None.", expected_token),
        }
    }

    #[test]
    fn test_recognize_string_service() {
        let settings = settings::get_default_settings();
        let parser = Parser::new("", &settings);
        let expected_token = Token::OpeningBracket(TokenValue {
            value: (),
            string: String::from("("),
            position: 5,
        });
        match parser.recognize_string("(", 5) {
            Some(actual_token) => assert_eq!(expected_token, actual_token),
            None => panic!("Expected {:?} but got None.", expected_token),
        }
    }

    #[test]
    fn test_recognize_string_none() {
        let settings = settings::get_default_settings();
        let parser = Parser::new("", &settings);
        if let Some(actual_token) = parser.recognize_string("unknown", 5) {
            panic!("Expected None but got {:?}", actual_token)
        }
    }

    #[test]
    fn test_perform_lexical_analysis() {
        let settings = settings::get_default_settings();
        let mut parser = Parser::new("log(2.0, x) + cos(0.0) - x", &settings);
        parser.perform_lexical_analysis();
        assert_eq!(
            vec![
                Rc::new(Token::Function(TokenValue {
                    value: settings.functions.find_by_name("log").unwrap(),
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
                    value: settings.operators.find_by_name("+").unwrap(),
                    string: String::from("+"),
                    position: 12
                })),
                Rc::new(Token::Function(TokenValue {
                    value: settings.functions.find_by_name("cos").unwrap(),
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
                    value: settings.operators.find_by_name("-").unwrap(),
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

    #[test]
    fn test_push_token() {
        let settings = settings::get_default_settings();
        let mut parser = Parser::new("", &settings);
        let one_token = Token::Constant(TokenValue {
            value: 1.0,
            string: String::from("1.0"),
            position: 0,
        });
        let x_token = Token::Variable(TokenValue {
            value: String::from("x"),
            string: String::from("x"),
            position: 0,
        });
        let plus_token = Token::Operator(TokenValue {
            value: settings.operators.find_by_name("+").unwrap(),
            string: String::from("+"),
            position: 0,
        });
        let sin_token = Token::Function(TokenValue {
            value: settings.functions.find_by_name("sin").unwrap(),
            string: String::from("sin"),
            position: 0,
        });
        if let Err(err) = parser.push_token(Rc::new(one_token)) {
            panic!(
                "Expected to push a token with a constant \"1.0\", but an error was received {:?}.",
                err
            );
        }
        match parser.push_token(Rc::new(plus_token.clone())) {
            Ok(_) => panic!("Expected InvalidArgumentsNumberError, but Ok(()) was received."),
            Err(err) => assert_eq!(
                InvalidArgumentsNumberError {
                    token: plus_token.clone(),
                    expected: 2,
                    actual: 1,
                },
                err
            ),
        }
        if let Err(err) = parser.push_token(Rc::new(x_token)) {
            panic!(
                "Expected to push a token with a variable \"x\", but an error was received {:?}.",
                err
            )
        }
        match parser.push_token(Rc::new(sin_token.clone())) {
            Ok(_) => panic!("Expected InvalidArgumentsNumberError, but Ok(()) was received."),
            Err(err) => assert_eq!(
                InvalidArgumentsNumberError {
                    token: sin_token.clone(),
                    expected: 1,
                    actual: 2
                },
                err
            ),
        }
        if let Err(err) = parser.push_token(Rc::new(plus_token.clone())) {
            panic!(
                "Expected to push a token with a operator \"+\", but an error was received {:?}.",
                err
            )
        }
        if let Err(err) = parser.push_token(Rc::new(sin_token.clone())) {
            panic!(
                "Expected to push a token with a function \"sin\", but an error was received {:?}.",
                err
            )
        }
        assert_eq!(parser.queue.len(), 1);
        assert_eq!(
            Node::Function(OperationNode {
                operation: settings.functions.find_by_name("sin").unwrap(),
                arguments: vec![Node::Operator(OperationNode {
                    operation: settings.operators.find_by_name("+").unwrap(),
                    arguments: vec![
                        Node::Value(ValueNode::Constant(1.0)),
                        Node::Value(ValueNode::Variable(String::from("x")))
                    ]
                })]
            }),
            parser.queue.pop_front().unwrap()
        );
    }
}
