//! Lexer module.
use super::error::ErrorTokenData;
use crate::model::settings::{Function, Operator, Settings};
use std::rc::Rc;

pub struct Lexer<'a> {
    pub expression: String,
    pub settings: &'a Settings,
    pub tokens: Vec<Rc<Token>>,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Constant(TokenValue<f64>),
    Variable(TokenValue<String>),
    Function(TokenValue<Rc<Function>>),
    Operator(TokenValue<Rc<Operator>>),
    OpeningBracket(TokenValue<()>),
    CloseBracket(TokenValue<()>),
    Comma(TokenValue<()>),
}

impl Token {
    pub fn get_error_token_data(&self) -> ErrorTokenData {
        match self {
            Token::Constant(tv) => tv.get_error_token_data(),
            Token::Variable(tv) => tv.get_error_token_data(),
            Token::Function(tv) => tv.get_error_token_data(),
            Token::Operator(tv) => tv.get_error_token_data(),
            Token::OpeningBracket(tv) => tv.get_error_token_data(),
            Token::CloseBracket(tv) => tv.get_error_token_data(),
            Token::Comma(tv) => tv.get_error_token_data(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenValue<T> {
    pub value: T,
    pub string: String,
    pub position: usize,
}

impl<T> TokenValue<T> {
    fn get_error_token_data(&self) -> ErrorTokenData {
        ErrorTokenData {
            string: self.string.to_owned(),
            position: self.position,
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(expression: &str, settings: &'a Settings) -> Lexer<'a> {
        Lexer {
            expression: expression.to_owned(),
            settings,
            tokens: vec![],
        }
    }
    pub fn perform_lexical_analysis(&mut self) {
        let mut string = String::new();
        let mut p = 0;
        for (i, c) in self.expression.chars().enumerate() {
            if c.is_whitespace() {
                if string.len() != 0 {
                    self.tokens.push(Rc::new(Lexer::recognize_value_string(
                        &string,
                        self.expression.len() - string.len(),
                    )));
                    string = String::new();
                }
                p = i + 1;
                continue;
            }
            string.push(c);
            for j in 0..string.len() {
                if let Some(token) = self.recognize_string(
                    &string[j..],
                    p + j,
                    j == 0 && self.is_next_operator_unary(),
                ) {
                    if j != 0 {
                        self.tokens
                            .push(Rc::new(Lexer::recognize_value_string(&string[0..j], p)));
                    }
                    self.tokens.push(Rc::new(token));
                    string = String::new();
                    p = i + 1;
                    break;
                }
            }
        }
        if string.len() != 0 {
            self.tokens.push(Rc::new(Lexer::recognize_value_string(
                &string,
                self.expression.len() - string.len(),
            )));
        }
    }
    fn recognize_value_string(string: &str, position: usize) -> Token {
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
    fn recognize_string(
        &self,
        string: &str,
        position: usize,
        is_next_operator_unary: bool,
    ) -> Option<Token> {
        self.recognize_operation_string(string, position, is_next_operator_unary)
            .or(Lexer::recognize_service_string(string, position))
    }
    fn recognize_operation_string(
        &self,
        string: &str,
        position: usize,
        is_next_operator_unary: bool,
    ) -> Option<Token> {
        let operator_option = if is_next_operator_unary {
            self.settings.find_unary_operator_by_name(string)
        } else {
            self.settings.find_binary_operator_by_name(string)
        };
        if let Some(operator) = operator_option {
            return Some(Token::Operator(TokenValue {
                value: operator,
                string: string.to_owned(),
                position,
            }));
        };
        if let Some(function) = self.settings.find_function_by_name(string) {
            return Some(Token::Function(TokenValue {
                value: function,
                string: string.to_owned(),
                position,
            }));
        }
        None
    }
    fn is_next_operator_unary(&self) -> bool {
        if self.tokens.len() == 0 {
            return true;
        }
        match &*self.tokens[self.tokens.len() - 1] {
            Token::Operator(_) => true,
            Token::OpeningBracket(_) => true,
            _ => false,
        }
    }
    fn recognize_service_string(string: &str, position: usize) -> Option<Token> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recognize_value_string_constant() {
        let expected_token = Token::Constant(TokenValue {
            value: 1.0,
            string: String::from("1.0"),
            position: 5,
        });
        assert_eq!(expected_token, Lexer::recognize_value_string("1.0", 5));
    }

    #[test]
    fn test_recognize_value_string_variable() {
        let expected_token = Token::Variable(TokenValue {
            value: String::from("x1"),
            string: String::from("x1"),
            position: 5,
        });
        assert_eq!(expected_token, Lexer::recognize_value_string("x1", 5));
    }

    #[test]
    fn test_is_next_operator_unary() {
        let settings = Settings::default();
        let mut lexer = Lexer::new("", &settings);
        assert!(lexer.is_next_operator_unary());
        lexer.tokens = vec![Rc::new(create_plus_token(&settings))];
        assert!(lexer.is_next_operator_unary());
        lexer.tokens = vec![Rc::new(create_opening_bracket_token())];
        assert!(lexer.is_next_operator_unary());
        lexer.tokens = vec![Rc::new(create_one_token())];
        assert!(!lexer.is_next_operator_unary());
    }

    #[test]
    fn test_recognize_string_unary_operator() {
        let settings = Settings::default();
        let lexer = Lexer::new("", &settings);
        let expected_token = Token::Operator(TokenValue {
            value: settings.find_unary_operator_by_name("-").unwrap(),
            string: String::from("-"),
            position: 5,
        });
        match lexer.recognize_string("-", 5, true) {
            Some(actual_token) => assert_eq!(expected_token, actual_token),
            None => panic!("Expected {:?}, but got None.", expected_token),
        }
    }

    #[test]
    fn test_recognize_string_binary_operator() {
        let settings = Settings::default();
        let lexer = Lexer::new("", &settings);
        let expected_token = Token::Operator(TokenValue {
            value: settings.find_binary_operator_by_name("-").unwrap(),
            string: String::from("-"),
            position: 5,
        });
        match lexer.recognize_string("-", 5, false) {
            Some(actual_token) => assert_eq!(expected_token, actual_token),
            None => panic!("Expected {:?}, but got None.", expected_token),
        }
    }

    #[test]
    fn test_recognize_string_function() {
        let settings = Settings::default();
        let lexer = Lexer::new("", &settings);
        let expected_token = Token::Function(TokenValue {
            value: settings.find_function_by_name("sin").unwrap(),
            string: String::from("sin"),
            position: 5,
        });
        match lexer.recognize_string("sin", 5, false) {
            Some(actual_token) => assert_eq!(expected_token, actual_token),
            None => panic!("Expected {:?}, but got None.", expected_token),
        }
    }

    #[test]
    fn test_recognize_string_service() {
        let settings = Settings::default();
        let lexer = Lexer::new("", &settings);
        let expected_token = Token::OpeningBracket(TokenValue {
            value: (),
            string: String::from("("),
            position: 5,
        });
        match lexer.recognize_string("(", 5, false) {
            Some(actual_token) => assert_eq!(expected_token, actual_token),
            None => panic!("Expected {:?}, but got None.", expected_token),
        }
    }

    #[test]
    fn test_recognize_string_none() {
        let settings = Settings::default();
        let lexer = Lexer::new("", &settings);
        if let Some(actual_token) = lexer.recognize_string("unknown", 5, false) {
            panic!("Expected None, but got {:?}", actual_token)
        }
    }

    #[test]
    fn test_perform_lexical_analysis() {
        let settings = Settings::default();
        let mut lexer = Lexer::new("log(2.0, x) + cos(-1.0) - x", &settings);
        lexer.perform_lexical_analysis();
        assert_eq!(
            vec![
                Rc::new(Token::Function(TokenValue {
                    value: settings.find_function_by_name("log").unwrap(),
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
                    value: settings.find_binary_operator_by_name("+").unwrap(),
                    string: String::from("+"),
                    position: 12
                })),
                Rc::new(Token::Function(TokenValue {
                    value: settings.find_function_by_name("cos").unwrap(),
                    string: String::from("cos"),
                    position: 14
                })),
                Rc::new(Token::OpeningBracket(TokenValue {
                    value: (),
                    string: String::from("("),
                    position: 17
                })),
                Rc::new(Token::Operator(TokenValue {
                    value: settings.find_unary_operator_by_name("-").unwrap(),
                    string: String::from("-"),
                    position: 18
                })),
                Rc::new(Token::Constant(TokenValue {
                    value: 1.0,
                    string: String::from("1.0"),
                    position: 19
                })),
                Rc::new(Token::CloseBracket(TokenValue {
                    value: (),
                    string: String::from(")"),
                    position: 22
                })),
                Rc::new(Token::Operator(TokenValue {
                    value: settings.find_binary_operator_by_name("-").unwrap(),
                    string: String::from("-"),
                    position: 24
                })),
                Rc::new(Token::Variable(TokenValue {
                    value: String::from("x"),
                    string: String::from("x"),
                    position: 26
                })),
            ],
            lexer.tokens
        );
    }

    #[test]
    fn test_get_error_token_data() {
        let settings = Settings::default();
        let token = create_sin_token(&settings);
        assert_eq!(
            ErrorTokenData {
                string: String::from("sin"),
                position: 0
            },
            token.get_error_token_data()
        );
    }

    fn create_one_token() -> Token {
        Token::Constant(TokenValue {
            value: 1.0,
            string: String::from("1.0"),
            position: 0,
        })
    }

    fn create_plus_token(settings: &Settings) -> Token {
        Token::Operator(TokenValue {
            value: settings.find_binary_operator_by_name("+").unwrap(),
            string: String::from("+"),
            position: 0,
        })
    }

    fn create_sin_token<'a>(settings: &Settings) -> Token {
        Token::Function(TokenValue {
            value: settings.find_function_by_name("sin").unwrap(),
            string: String::from("sin"),
            position: 0,
        })
    }

    fn create_opening_bracket_token() -> Token {
        Token::OpeningBracket(TokenValue {
            value: (),
            string: String::from("("),
            position: 0,
        })
    }
}
