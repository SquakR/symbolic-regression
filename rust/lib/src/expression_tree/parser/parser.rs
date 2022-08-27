//! Parser module.
use super::super::types::{ExpressionTree, Node, ValueNode};
use super::error::{
    InvalidArgumentsNumberError, MissingCommaError, MissingCommaOrOpeningBracketError, ParseError,
};
use super::lexer::{Lexer, Token};
use crate::model::settings::{ConverterOperation, Settings};
use std::collections::VecDeque;
use std::rc::Rc;

impl<'a> ExpressionTree {
    pub fn parse(expression: &str, settings: &'a Settings) -> Result<ExpressionTree, ParseError> {
        Parser::parse(expression, settings)
    }
}

struct Parser<'a> {
    settings: &'a Settings,
    lexer: Lexer<'a>,
    queue: VecDeque<Node>,
    stack: Vec<Rc<Token>>,
    variables: Vec<String>,
}

impl<'a> Parser<'a> {
    fn parse(expression: &str, settings: &'a Settings) -> Result<ExpressionTree, ParseError> {
        if expression.len() == 0 {
            return Err(ParseError::EmptyFormulaError);
        }
        let mut parser = Parser::new(expression, settings);
        parser.lexer.perform_lexical_analysis();
        parser.handle_tokens()?;
        if parser.queue.len() != 1 {
            return Err(ParseError::MultipleFormulaError);
        }
        Ok(ExpressionTree {
            root: parser.queue.pop_front().unwrap(),
            variables: parser.variables.iter().cloned().collect::<Vec<String>>(),
        })
    }
    fn new(expression: &str, settings: &'a Settings) -> Parser<'a> {
        Parser {
            settings,
            lexer: Lexer::new(expression, settings),
            queue: VecDeque::new(),
            stack: vec![],
            variables: vec![],
        }
    }
    fn handle_tokens(&mut self) -> Result<(), ParseError> {
        let tokens_rcs = self
            .lexer
            .tokens
            .iter()
            .cloned()
            .collect::<Vec<Rc<Token>>>();
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
        if !self.variables.contains(&value) {
            self.variables.push(value.to_owned());
        }
        match self.push_token(token) {
            Err(err) => Err(ParseError::InvalidArgumentsNumberError(err)),
            Ok(_) => Ok(()),
        }
    }
    fn handle_function(&mut self, token: Rc<Token>) {
        self.stack.push(token);
    }
    fn handle_comma(&mut self, token: Rc<Token>) -> Result<(), ParseError> {
        self.shift_until_opening_bracket(token)
    }
    fn handle_operator(&mut self, token: Rc<Token>) -> Result<(), ParseError> {
        let value = match &*token {
            Token::Operator(token_value) => token_value.value.to_owned(),
            _ => unreachable!(),
        };
        loop {
            if self.stack.len() == 0 {
                break;
            }
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
        self.stack.push(token);
        Ok(())
    }
    fn handle_opening_bracket(&mut self, token: Rc<Token>) {
        self.stack.push(token);
    }
    fn handle_close_bracket(&mut self, token: Rc<Token>) -> Result<(), ParseError> {
        self.shift_until_opening_bracket(token)?;
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
                .push_back(Node::Value(ValueNode::Constant(token_value.value)))),
            Token::Variable(token_value) => Ok(self.queue.push_back(Node::Value(
                ValueNode::Variable(token_value.value.to_owned()),
            ))),
            Token::Function(token_value) => {
                let node = self.create_operation_node(
                    Rc::clone(&token),
                    ConverterOperation::Function(Rc::clone(&token_value.value)),
                    token_value.value.arguments_number,
                )?;
                Ok(self.queue.push_back(node))
            }
            Token::Operator(token_value) => {
                let node = self.create_operation_node(
                    Rc::clone(&token),
                    ConverterOperation::Operator(Rc::clone(&token_value.value)),
                    token_value.value.arguments_number,
                )?;
                Ok(self.queue.push_back(node))
            }
            _ => Ok(()),
        }
    }
    fn create_operation_node(
        &mut self,
        token: Rc<Token>,
        operation: ConverterOperation,
        arguments_number: usize,
    ) -> Result<Node, InvalidArgumentsNumberError> {
        let arguments = self.extract_arguments(Rc::clone(&token), arguments_number)?;
        Ok(self.settings.convert(operation, arguments).to_node())
    }
    fn extract_arguments(
        &mut self,
        token: Rc<Token>,
        arguments_number: usize,
    ) -> Result<Vec<Node>, InvalidArgumentsNumberError> {
        if self.queue.len() < arguments_number {
            return Err(InvalidArgumentsNumberError {
                data: (&*token).get_error_token_data(),
                expected: arguments_number,
                actual: self.queue.len(),
            });
        }
        let arguments = self
            .queue
            .split_off(self.queue.len() - arguments_number)
            .into_iter()
            .collect::<Vec<Node>>();
        Ok(arguments)
    }
    fn shift_until_opening_bracket(&mut self, token: Rc<Token>) -> Result<(), ParseError> {
        let mut tokens = VecDeque::new();
        loop {
            if self.stack.len() == 0 {
                match *token {
                    Token::Comma(_) => {
                        return Err(ParseError::MissingCommaOrOpeningBracketError(
                            MissingCommaOrOpeningBracketError {
                                data: (&*token).get_error_token_data(),
                            },
                        ))
                    }
                    Token::CloseBracket(_) => {
                        return Err(ParseError::MissingCommaError({
                            MissingCommaError {
                                data: (&*token).get_error_token_data(),
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
                return Err(ParseError::MissingCommaError(MissingCommaError {
                    data: (&*self.stack[self.stack.len() - 1]).get_error_token_data(),
                }));
            }
            let last_token = self.stack.pop().unwrap();
            if let Err(err) = self.push_token(last_token) {
                return Err(ParseError::InvalidArgumentsNumberError(err));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::types::OperationNode;
    use super::super::lexer::TokenValue;
    use super::*;
    use std::f64::consts::E;

    #[test]
    fn test_push_token() -> Result<(), InvalidArgumentsNumberError> {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.push_token(Rc::new(create_one_token()))?;
        let expected_error = InvalidArgumentsNumberError {
            data: create_plus_token(&settings).get_error_token_data(),
            expected: 2,
            actual: 1,
        };
        match parser.push_token(Rc::new(create_plus_token(&settings))) {
            Ok(_) => panic!("Expected {:?}, but Ok(()) was received.", expected_error),
            Err(err) => assert_eq!(expected_error, err),
        }
        parser.push_token(Rc::new(create_x_token()))?;
        parser.push_token(Rc::new(create_plus_token(&settings)))?;
        parser.push_token(Rc::new(create_sin_token(&settings)))?;
        assert_eq!(
            VecDeque::from(vec![Node::Function(OperationNode {
                operation: settings.find_function_by_name("sin").unwrap(),
                arguments: vec![Node::Operator(OperationNode {
                    operation: settings.find_binary_operator_by_name("+").unwrap(),
                    arguments: vec![
                        Node::Value(ValueNode::Constant(1.0)),
                        Node::Value(ValueNode::Variable(String::from("x")))
                    ]
                })]
            }),]),
            parser.queue
        );
        Ok(())
    }

    #[test]
    fn test_extract_arguments() -> Result<(), InvalidArgumentsNumberError> {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.queue = VecDeque::from(vec![
            Node::Value(ValueNode::Constant(1.0)),
            Node::Value(ValueNode::Constant(2.0)),
            Node::Value(ValueNode::Constant(3.0)),
        ]);
        parser.extract_arguments(Rc::new(create_log_token(&settings)), 2)?;
        assert_eq!(
            VecDeque::from(vec![Node::Value(ValueNode::Constant(1.0))]),
            parser.queue
        );
        let expected_error = InvalidArgumentsNumberError {
            data: create_log_token(&settings).get_error_token_data(),
            expected: 2,
            actual: 1,
        };
        match parser.extract_arguments(Rc::new(create_log_token(&settings)), 2) {
            Ok(_) => panic!("Expected {:?}, but Ok(()) was received.", expected_error),
            Err(err) => assert_eq!(expected_error, err),
        };
        Ok(())
    }

    #[test]
    fn test_create_operation_node() -> Result<(), InvalidArgumentsNumberError> {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.queue = VecDeque::from(vec![Node::Value(ValueNode::Variable(String::from("x")))]);
        let expected_node = Node::Function(OperationNode {
            operation: settings.find_function_by_name("log").unwrap(),
            arguments: vec![
                Node::Value(ValueNode::Constant(E)),
                Node::Value(ValueNode::Variable(String::from("x"))),
            ],
        });
        let actual_node = parser.create_operation_node(
            Rc::new(create_ln_token(&settings)),
            ConverterOperation::Function(settings.find_function_by_name("ln").unwrap()),
            1,
        )?;
        assert_eq!(expected_node, actual_node);
        assert_eq!(VecDeque::new(), parser.queue);
        Ok(())
    }

    #[test]
    fn test_shift_until_opening_bracket() -> Result<(), ParseError> {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.queue = VecDeque::from(vec![
            Node::Value(ValueNode::Constant(1.0)),
            Node::Value(ValueNode::Variable(String::from("x"))),
        ]);
        parser.stack = vec![
            Rc::new(create_one_token()),
            Rc::new(create_opening_bracket_token()),
            Rc::new(create_log_token(&settings)),
        ];
        parser.shift_until_opening_bracket(Rc::new(create_close_bracket_token()))?;
        assert_eq!(
            vec![
                Rc::new(create_one_token()),
                Rc::new(create_opening_bracket_token()),
            ],
            parser.stack
        );
        assert_eq!(
            VecDeque::from(vec![Node::Function(OperationNode {
                operation: settings.find_function_by_name("log").unwrap(),
                arguments: vec![
                    Node::Value(ValueNode::Constant(1.0)),
                    Node::Value(ValueNode::Variable(String::from("x"))),
                ]
            }),]),
            parser.queue
        );
        Ok(())
    }

    #[test]
    fn test_shift_until_opening_bracket_comma() {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.stack = vec![
            Rc::new(create_one_token()),
            Rc::new(create_log_token(&settings)),
        ];
        let expected_error =
            ParseError::MissingCommaOrOpeningBracketError(MissingCommaOrOpeningBracketError {
                data: create_comma_token().get_error_token_data(),
            });
        match parser.shift_until_opening_bracket(Rc::new(create_comma_token())) {
            Ok(_) => panic!("Expected {:?}, but Ok(()) was received.", expected_error),
            Err(err) => assert_eq!(expected_error, err),
        }
        let expected_stack: Vec<Rc<Token>> = vec![];
        assert_eq!(expected_stack, parser.stack);
        assert_eq!(VecDeque::new(), parser.queue);
    }

    #[test]
    fn test_shift_until_opening_bracket_close_bracket() {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.stack = vec![
            Rc::new(create_one_token()),
            Rc::new(create_log_token(&settings)),
        ];
        let expected_error = ParseError::MissingCommaError(MissingCommaError {
            data: create_close_bracket_token().get_error_token_data(),
        });
        match parser.shift_until_opening_bracket(Rc::new(create_close_bracket_token())) {
            Ok(_) => panic!("Expected {:?}, but Ok(()) was received.", expected_error),
            Err(err) => assert_eq!(expected_error, err),
        }
        let expected_stack: Vec<Rc<Token>> = vec![];
        assert_eq!(expected_stack, parser.stack);
        assert_eq!(VecDeque::new(), parser.queue);
    }

    #[test]
    fn test_shift_until_opening_bracket_invalid_arguments_number() {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.queue = VecDeque::from(vec![Node::Value(ValueNode::Constant(1.0))]);
        parser.stack = vec![
            Rc::new(create_one_token()),
            Rc::new(create_opening_bracket_token()),
            Rc::new(create_log_token(&settings)),
        ];
        let expected_error = ParseError::InvalidArgumentsNumberError(InvalidArgumentsNumberError {
            data: create_log_token(&settings).get_error_token_data(),
            expected: 2,
            actual: 1,
        });
        match parser.shift_until_opening_bracket(Rc::new(create_close_bracket_token())) {
            Ok(_) => panic!("Expected {:?}, but Ok(()) was received.", expected_error),
            Err(err) => assert_eq!(expected_error, err),
        }
        assert_eq!(
            vec![
                Rc::new(create_one_token()),
                Rc::new(create_opening_bracket_token()),
            ],
            parser.stack
        );
        assert_eq!(
            VecDeque::from(vec![Node::Value(ValueNode::Constant(1.0))]),
            parser.queue
        );
    }

    #[test]
    fn test_shift_all() -> Result<(), ParseError> {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.stack = vec![
            Rc::new(create_plus_token(&settings)),
            Rc::new(create_one_token()),
            Rc::new(create_x_token()),
        ];
        parser.shift_all()?;
        let expected_stack: Vec<Rc<Token>> = vec![];
        assert_eq!(expected_stack, parser.stack);
        assert_eq!(
            VecDeque::from(vec![Node::Operator(OperationNode {
                operation: settings.find_binary_operator_by_name("+").unwrap(),
                arguments: vec![
                    Node::Value(ValueNode::Variable(String::from("x"))),
                    Node::Value(ValueNode::Constant(1.0)),
                ]
            })]),
            parser.queue
        );
        Ok(())
    }

    #[test]
    fn test_shift_all_opening_bracket() {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.stack = vec![
            Rc::new(create_one_token()),
            Rc::new(create_opening_bracket_token()),
            Rc::new(create_x_token()),
        ];
        let expected_error = ParseError::MissingCommaError(MissingCommaError {
            data: create_opening_bracket_token().get_error_token_data(),
        });
        match parser.shift_all() {
            Ok(_) => panic!("Expected {:?}, but Ok(()) was received.", expected_error),
            Err(err) => assert_eq!(expected_error, err),
        }
        assert_eq!(
            vec![
                Rc::new(create_one_token()),
                Rc::new(create_opening_bracket_token()),
            ],
            parser.stack
        );
        assert_eq!(
            VecDeque::from(vec![Node::Value(ValueNode::Variable(String::from("x")))]),
            parser.queue
        );
    }

    #[test]
    fn test_shift_all_invalid_arguments_number() {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.queue = VecDeque::from(vec![Node::Value(ValueNode::Constant(1.0))]);
        parser.stack = vec![Rc::new(create_plus_token(&settings))];
        let expected_error = ParseError::InvalidArgumentsNumberError(InvalidArgumentsNumberError {
            data: create_plus_token(&settings).get_error_token_data(),
            expected: 2,
            actual: 1,
        });
        match parser.shift_all() {
            Ok(_) => panic!("Expected {:?}, but Ok(()) was received.", expected_error),
            Err(err) => assert_eq!(expected_error, err),
        }
        let expected_stack: Vec<Rc<Token>> = vec![];
        assert_eq!(expected_stack, parser.stack);
        assert_eq!(
            VecDeque::from(vec![Node::Value(ValueNode::Constant(1.0))]),
            parser.queue
        );
    }

    #[test]
    fn test_handle_constant() {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        match parser.handle_constant(Rc::new(create_one_token())) {
            Ok(_) => assert_eq!(
                VecDeque::from(vec![Node::Value(ValueNode::Constant(1.0))]),
                parser.queue
            ),
            Err(_) => unreachable!(),
        };
    }

    #[test]
    fn test_handle_variable() {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        match parser.handle_variable(Rc::new(create_x_token())) {
            Ok(_) => {
                assert_eq!(
                    VecDeque::from(vec![Node::Value(ValueNode::Variable(String::from("x")))]),
                    parser.queue
                );
                assert_eq!(vec![String::from("x")], parser.variables);
            }
            Err(_) => unreachable!(),
        }
    }

    #[test]
    fn test_handle_operator_without_computed_before() -> Result<(), ParseError> {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.stack = vec![Rc::new(create_plus_token(&settings))];
        parser.handle_operator(Rc::new(create_asterisk_token(&settings)))?;
        assert_eq!(
            vec![
                Rc::new(create_plus_token(&settings)),
                Rc::new(create_asterisk_token(&settings))
            ],
            parser.stack
        );
        assert_eq!(VecDeque::new(), parser.queue);
        Ok(())
    }

    #[test]
    fn test_handle_operator_with_computed_before() -> Result<(), ParseError> {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.queue = VecDeque::from(vec![
            Node::Value(ValueNode::Constant(1.0)),
            Node::Value(ValueNode::Variable(String::from("x"))),
        ]);
        parser.stack = vec![Rc::new(create_asterisk_token(&settings))];
        parser.handle_operator(Rc::new(create_plus_token(&settings)))?;
        assert_eq!(vec![Rc::new(create_plus_token(&settings)),], parser.stack);
        assert_eq!(
            VecDeque::from(vec![Node::Operator(OperationNode {
                operation: settings.find_binary_operator_by_name("*").unwrap(),
                arguments: vec![
                    Node::Value(ValueNode::Constant(1.0)),
                    Node::Value(ValueNode::Variable(String::from("x"))),
                ]
            })]),
            parser.queue
        );
        Ok(())
    }

    #[test]
    fn test_handle_operator_with_computed_before_invalid_arguments_number() {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.queue = VecDeque::from(vec![Node::Value(ValueNode::Constant(1.0))]);
        parser.stack = vec![Rc::new(create_asterisk_token(&settings))];
        let expected_error = ParseError::InvalidArgumentsNumberError(InvalidArgumentsNumberError {
            data: create_asterisk_token(&settings).get_error_token_data(),
            expected: 2,
            actual: 1,
        });
        match parser.handle_operator(Rc::new(create_plus_token(&settings))) {
            Ok(_) => panic!("Expected {:?}, but Ok(()) was received.", expected_error),
            Err(err) => assert_eq!(expected_error, err),
        };
        let expected_stack: Vec<Rc<Token>> = vec![];
        assert_eq!(expected_stack, parser.stack);
        assert_eq!(
            VecDeque::from(vec![Node::Value(ValueNode::Constant(1.0))]),
            parser.queue
        );
    }

    #[test]
    fn test_handle_close_bracket() -> Result<(), ParseError> {
        let settings = Settings::default();
        let mut parser = Parser::new("", &settings);
        parser.queue = VecDeque::from(vec![
            Node::Value(ValueNode::Constant(1.0)),
            Node::Value(ValueNode::Variable(String::from("x"))),
        ]);
        parser.stack = vec![
            Rc::new(create_one_token()),
            Rc::new(create_log_token(&settings)),
            Rc::new(create_opening_bracket_token()),
        ];
        parser.handle_close_bracket(Rc::new(create_close_bracket_token()))?;
        assert_eq!(vec![Rc::new(create_one_token())], parser.stack);
        assert_eq!(
            VecDeque::from(vec![Node::Function(OperationNode {
                operation: settings.find_function_by_name("log").unwrap(),
                arguments: vec![
                    Node::Value(ValueNode::Constant(1.0)),
                    Node::Value(ValueNode::Variable(String::from("x"))),
                ]
            })]),
            parser.queue
        );
        Ok(())
    }

    #[test]
    fn test_parse_empty_formula_error() {
        let settings = Settings::default();
        let expected_error = ParseError::EmptyFormulaError;
        match Parser::parse("", &settings) {
            Ok(actual_tree) => panic!(
                "Expected {:?}, but {:?} was received.",
                expected_error, actual_tree
            ),
            Err(err) => assert_eq!(expected_error, err),
        }
    }

    #[test]
    fn test_multiple_formula_error() {
        let settings = Settings::default();
        let expected_error = ParseError::MultipleFormulaError;
        match Parser::parse("x + 1 1 + 2", &settings) {
            Ok(actual_tree) => panic!(
                "Expected {:?}, but {:?} was received.",
                expected_error, actual_tree
            ),
            Err(err) => assert_eq!(expected_error, err),
        }
    }

    #[test]
    fn test_parse_without_functions() -> Result<(), ParseError> {
        let settings = Settings::default();
        let expression = String::from("3 + 4 * 2 / ( x - 5 ) ^ -2 ^ 3");
        let plus = settings.find_binary_operator_by_name("+").unwrap();
        let unary_minus = settings.find_unary_operator_by_name("-").unwrap();
        let binary_minus = settings.find_binary_operator_by_name("-").unwrap();
        let slash = settings.find_binary_operator_by_name("/").unwrap();
        let asterisk = settings.find_binary_operator_by_name("*").unwrap();
        let circumflex = settings.find_binary_operator_by_name("^").unwrap();
        let actual_tree = Parser::parse(&expression, &settings)?;
        assert_eq!(
            ExpressionTree {
                root: Node::Operator(OperationNode {
                    operation: Rc::clone(&plus),
                    arguments: vec![
                        Node::Value(ValueNode::Constant(3.0)),
                        Node::Operator(OperationNode {
                            operation: Rc::clone(&slash),
                            arguments: vec![
                                Node::Operator(OperationNode {
                                    operation: Rc::clone(&asterisk),
                                    arguments: vec![
                                        Node::Value(ValueNode::Constant(4.0)),
                                        Node::Value(ValueNode::Constant(2.0)),
                                    ]
                                }),
                                Node::Operator(OperationNode {
                                    operation: Rc::clone(&circumflex),
                                    arguments: vec![
                                        Node::Operator(OperationNode {
                                            operation: Rc::clone(&binary_minus),
                                            arguments: vec![
                                                Node::Value(ValueNode::Variable(String::from("x"))),
                                                Node::Value(ValueNode::Constant(5.0)),
                                            ]
                                        }),
                                        Node::Operator(OperationNode {
                                            operation: Rc::clone(&circumflex),
                                            arguments: vec![
                                                Node::Operator(OperationNode {
                                                    operation: Rc::clone(&unary_minus),
                                                    arguments: vec![Node::Value(
                                                        ValueNode::Constant(2.0)
                                                    ),]
                                                }),
                                                Node::Value(ValueNode::Constant(3.0)),
                                            ]
                                        })
                                    ]
                                })
                            ]
                        })
                    ]
                }),
                variables: vec![String::from("x")]
            },
            actual_tree
        );
        Ok(())
    }

    #[test]
    fn test_parse_with_functions() -> Result<(), ParseError> {
        let settings = Settings::default();
        let expression = String::from("-ln(log(2, 3) / x1 * x2)");
        let unary_minus = settings.find_unary_operator_by_name("-").unwrap();
        let asterisk = settings.find_binary_operator_by_name("*").unwrap();
        let slash = settings.find_binary_operator_by_name("/").unwrap();
        let log = settings.find_function_by_name("log").unwrap();
        let actual_tree = Parser::parse(&expression, &settings)?;
        assert_eq!(
            ExpressionTree {
                root: Node::Operator(OperationNode {
                    operation: Rc::clone(&unary_minus),
                    arguments: vec![Node::Function(OperationNode {
                        operation: Rc::clone(&log),
                        arguments: vec![
                            Node::Value(ValueNode::Constant(E)),
                            Node::Operator(OperationNode {
                                operation: Rc::clone(&asterisk),
                                arguments: vec![
                                    Node::Operator(OperationNode {
                                        operation: Rc::clone(&slash),
                                        arguments: vec![
                                            Node::Function(OperationNode {
                                                operation: Rc::clone(&log),
                                                arguments: vec![
                                                    Node::Value(ValueNode::Constant(2.0)),
                                                    Node::Value(ValueNode::Constant(3.0)),
                                                ]
                                            }),
                                            Node::Value(ValueNode::Variable(String::from("x1")))
                                        ]
                                    }),
                                    Node::Value(ValueNode::Variable(String::from("x2")))
                                ]
                            })
                        ]
                    })]
                }),
                variables: vec![String::from("x1"), String::from("x2")]
            },
            actual_tree
        );
        Ok(())
    }

    fn create_one_token() -> Token {
        Token::Constant(TokenValue {
            value: 1.0,
            string: String::from("1.0"),
            position: 0,
        })
    }

    fn create_x_token() -> Token {
        Token::Variable(TokenValue {
            value: String::from("x"),
            string: String::from("x"),
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

    fn create_asterisk_token(settings: &Settings) -> Token {
        Token::Operator(TokenValue {
            value: settings.find_binary_operator_by_name("*").unwrap(),
            string: String::from("*"),
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

    fn create_log_token<'a>(settings: &Settings) -> Token {
        Token::Function(TokenValue {
            value: settings.find_function_by_name("log").unwrap(),
            string: String::from("log"),
            position: 0,
        })
    }

    fn create_ln_token<'a>(settings: &Settings) -> Token {
        Token::Function(TokenValue {
            value: settings.find_function_by_name("ln").unwrap(),
            string: String::from("ln"),
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

    fn create_close_bracket_token() -> Token {
        Token::CloseBracket(TokenValue {
            value: (),
            string: String::from(")"),
            position: 0,
        })
    }

    fn create_comma_token() -> Token {
        Token::Comma(TokenValue {
            value: (),
            string: String::from(","),
            position: 0,
        })
    }
}
