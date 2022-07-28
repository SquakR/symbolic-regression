//! Expression tree parser module.
//! The parser uses the shunting yard algorithm.
//! https://en.wikipedia.org/wiki/Shunting_yard_algorithm
use crate::expression_tree::{ExpressionTree, Node, ValueNode};
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
        let mut parser = Parser::new(expression, settings);
        Ok(ExpressionTree {
            root: Node::Value(ValueNode::Constant(0.0)),
            variables: vec![],
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
}

#[derive(Debug)]
pub enum ParseError<'a> {
    MissingCommaOrOpeningParenthesisError(MissingCommaOrOpeningParenthesisError<'a>),
    MissionCommaError(MissionCommaError<'a>),
    InvalidArgumentsNumberError(InvalidArgumentsNumberError<'a>),
}

#[derive(Debug)]
pub struct MissingCommaOrOpeningParenthesisError<'a> {
    token: Token<'a>,
}

#[derive(Debug)]
pub struct MissionCommaError<'a> {
    token: Token<'a>,
}

#[derive(Debug)]
pub struct InvalidArgumentsNumberError<'a> {
    token: Token<'a>,
    expected: u8,
    actual: u8,
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

impl Operator {
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
}
