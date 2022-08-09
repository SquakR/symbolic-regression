//! Module with model settings.
use crate::expression_tree::{Node, ValueNode};
use crate::types::{
    Associativity, ConvertOutputData, Converter, ConverterOperation, Function, Operation, Operator,
};
use std::f64::{consts::E, consts::PI, NAN};
use std::rc::Rc;

pub struct Settings {
    pub operators: Vec<Rc<Operator>>,
    pub functions: Vec<Rc<Function>>,
    pub converters: Vec<Converter>,
}

impl Settings {
    pub fn find_function_by_name(&self, name: &str) -> Option<Rc<Function>> {
        for function in &self.functions {
            if function.get_name() == name {
                return Some(Rc::clone(function));
            }
        }
        None
    }
    pub fn find_unary_operator_by_name(&self, name: &str) -> Option<Rc<Operator>> {
        for operator in &self.operators {
            if operator.get_name() == name && operator.arguments_number == 1 {
                return Some(Rc::clone(operator));
            }
        }
        None
    }
    pub fn find_binary_operator_by_name(&self, name: &str) -> Option<Rc<Operator>> {
        for operator in &self.operators {
            if operator.get_name() == name && operator.arguments_number == 2 {
                return Some(Rc::clone(operator));
            }
        }
        None
    }
    pub fn find_converters(&self, operation: &ConverterOperation) -> Vec<&Converter> {
        let mut converters = vec![];
        for converter in &self.converters {
            if converter.from == *operation {
                converters.push(converter)
            }
        }
        converters
    }
    pub fn convert(
        &self,
        operation: ConverterOperation,
        arguments: Vec<Node>,
    ) -> ConvertOutputData {
        for converter in self.find_converters(&operation) {
            if converter.is_conversion_possible(&operation, &arguments) {
                return converter.convert(arguments);
            }
        }
        ConvertOutputData {
            operation,
            arguments,
        }
    }
    pub fn default() -> Settings {
        let mut settings = Settings {
            operators: vec![],
            functions: vec![],
            converters: vec![],
        };
        settings.set_default_operators();
        settings.set_default_functions();
        settings.set_default_converters();
        settings
    }
    pub fn set_default_operators(&mut self) {
        self.operators = vec![
            Rc::new(Operator {
                name: String::from("+"),
                arguments_number: 2,
                precedence: 1,
                associativity: Associativity::Left,
                complexity: 1,
                compute_fn: |arguments| arguments[0] + arguments[1],
            }),
            Rc::new(Operator {
                name: String::from("-"),
                arguments_number: 2,
                precedence: 1,
                associativity: Associativity::Left,
                complexity: 1,
                compute_fn: |arguments| arguments[0] - arguments[1],
            }),
            Rc::new(Operator {
                name: String::from("*"),
                arguments_number: 2,
                precedence: 2,
                associativity: Associativity::Left,
                complexity: 2,
                compute_fn: |arguments| arguments[0] * arguments[1],
            }),
            Rc::new(Operator {
                name: String::from("/"),
                arguments_number: 2,
                precedence: 2,
                associativity: Associativity::Left,
                complexity: 2,
                compute_fn: |arguments| arguments[0] / arguments[1],
            }),
            Rc::new(Operator {
                name: String::from("^"),
                arguments_number: 2,
                precedence: 3,
                associativity: Associativity::Right,
                complexity: 3,
                compute_fn: |arguments| arguments[0].powf(arguments[1]),
            }),
            Rc::new(Operator {
                name: String::from("+"),
                arguments_number: 1,
                precedence: 4,
                associativity: Associativity::Right,
                complexity: 1,
                compute_fn: |arguments| arguments[0],
            }),
            Rc::new(Operator {
                name: String::from("-"),
                arguments_number: 1,
                precedence: 4,
                associativity: Associativity::Right,
                complexity: 1,
                compute_fn: |arguments| -arguments[0],
            }),
        ];
    }
    pub fn set_default_functions(&mut self) {
        self.functions = vec![
            Rc::new(Function {
                name: String::from("abs"),
                arguments_number: 1,
                complexity: 3,
                compute_fn: |arguments| arguments[0].abs(),
            }),
            Rc::new(Function {
                name: String::from("log"),
                arguments_number: 2,
                complexity: 4,
                compute_fn: |arguments| arguments[0].log(arguments[1]),
            }),
            Rc::new(Function {
                name: String::from("sin"),
                arguments_number: 1,
                complexity: 4,
                compute_fn: |arguments| arguments[0].sin(),
            }),
            Rc::new(Function {
                name: String::from("arcsin"),
                arguments_number: 1,
                complexity: 5,
                compute_fn: |arguments| arguments[0].asin(),
            }),
            Rc::new(Function {
                name: String::from("cos"),
                arguments_number: 1,
                complexity: 4,
                compute_fn: |arguments| arguments[0].cos(),
            }),
            Rc::new(Function {
                name: String::from("arccos"),
                arguments_number: 1,
                complexity: 5,
                compute_fn: |arguments| arguments[0].acos(),
            }),
            Rc::new(Function {
                name: String::from("tan"),
                arguments_number: 1,
                complexity: 5,
                compute_fn: |arguments| arguments[0].tan(),
            }),
            Rc::new(Function {
                name: String::from("arctan"),
                arguments_number: 1,
                complexity: 6,
                compute_fn: |arguments| arguments[0].atan(),
            }),
            Rc::new(Function {
                name: String::from("cot"),
                arguments_number: 1,
                complexity: 5,
                compute_fn: |arguments| 1.0 / arguments[0].tan(),
            }),
            Rc::new(Function {
                name: String::from("arccot"),
                arguments_number: 1,
                complexity: 6,
                compute_fn: |arguments| PI / 2.0 - arguments[0].atan(),
            }),
            Rc::new(Function {
                name: String::from("sinh"),
                arguments_number: 1,
                complexity: 5,
                compute_fn: |arguments| arguments[0].sinh(),
            }),
            Rc::new(Function {
                name: String::from("arsinh"),
                arguments_number: 1,
                complexity: 6,
                compute_fn: |arguments| arguments[0].asinh(),
            }),
            Rc::new(Function {
                name: String::from("cosh"),
                arguments_number: 1,
                complexity: 5,
                compute_fn: |arguments| arguments[0].cosh(),
            }),
            Rc::new(Function {
                name: String::from("arcosh"),
                arguments_number: 1,
                complexity: 6,
                compute_fn: |arguments| arguments[0].acosh(),
            }),
            Rc::new(Function {
                name: String::from("tanh"),
                arguments_number: 1,
                complexity: 6,
                compute_fn: |arguments| arguments[0].tanh(),
            }),
            Rc::new(Function {
                name: String::from("artanh"),
                arguments_number: 1,
                complexity: 7,
                compute_fn: |arguments| arguments[0].atanh(),
            }),
            Rc::new(Function {
                name: String::from("coth"),
                arguments_number: 1,
                complexity: 6,
                compute_fn: |arguments| 1.0 / arguments[0].tanh(),
            }),
            Rc::new(Function {
                name: String::from("arcoth"),
                arguments_number: 1,
                complexity: 7,
                compute_fn: |arguments| {
                    if arguments[0] < -1.0 || arguments[0] > 1.0 {
                        ((arguments[0] + 1.0) / (arguments[0] - 1.0)).ln() * 0.5
                    } else {
                        NAN
                    }
                },
            }),
            Rc::new(Function {
                name: String::from("ln"),
                arguments_number: 1,
                complexity: 4,
                compute_fn: |arguments| arguments[0].ln(),
            }),
            Rc::new(Function {
                name: String::from("exp"),
                arguments_number: 1,
                complexity: 3,
                compute_fn: |arguments| arguments[0].exp(),
            }),
            Rc::new(Function {
                name: String::from("sqrt"),
                arguments_number: 1,
                complexity: 3,
                compute_fn: |arguments| arguments[0].sqrt(),
            }),
        ];
    }
    pub fn set_default_converters(&mut self) {
        let circumflex = self.find_binary_operator_by_name("^").unwrap();
        let log = self.find_function_by_name("log").unwrap();
        let ln = self.find_function_by_name("ln").unwrap();
        let exp = self.find_function_by_name("exp").unwrap();
        let sqrt = self.find_function_by_name("sqrt").unwrap();
        self.converters = vec![
            Converter {
                from: ConverterOperation::Function(Rc::clone(&ln)),
                to: ConverterOperation::Function(Rc::clone(&log)),
                is_conversion_possible_fn: |_| true,
                convert_fn: |mut arguments| {
                    arguments.insert(0, Node::Value(ValueNode::Constant(E)));
                    arguments
                },
            },
            Converter {
                from: ConverterOperation::Function(Rc::clone(&exp)),
                to: ConverterOperation::Operator(Rc::clone(&circumflex)),
                is_conversion_possible_fn: |_| true,
                convert_fn: |mut arguments| {
                    arguments.insert(0, Node::Value(ValueNode::Constant(E)));
                    arguments
                },
            },
            Converter {
                from: ConverterOperation::Function(Rc::clone(&sqrt)),
                to: ConverterOperation::Operator(Rc::clone(&circumflex)),
                is_conversion_possible_fn: |_| true,
                convert_fn: |mut arguments| {
                    arguments.push(Node::Value(ValueNode::Constant(0.5)));
                    arguments
                },
            },
            Converter {
                from: ConverterOperation::Function(Rc::clone(&log)),
                to: ConverterOperation::Function(Rc::clone(&ln)),
                is_conversion_possible_fn: |arguments| {
                    if let Node::Value(ValueNode::Constant(constant)) = arguments[0] {
                        (constant - E).abs() <= 0.001
                    } else {
                        false
                    }
                },
                convert_fn: |mut arguments| {
                    arguments.remove(0);
                    arguments
                },
            },
            Converter {
                from: ConverterOperation::Operator(Rc::clone(&circumflex)),
                to: ConverterOperation::Function(Rc::clone(&exp)),
                is_conversion_possible_fn: |arguments| {
                    if let Node::Value(ValueNode::Constant(constant)) = arguments[0] {
                        (constant - E).abs() <= 0.001
                    } else {
                        false
                    }
                },
                convert_fn: |mut arguments| {
                    arguments.remove(0);
                    arguments
                },
            },
            Converter {
                from: ConverterOperation::Operator(Rc::clone(&circumflex)),
                to: ConverterOperation::Function(Rc::clone(&sqrt)),
                is_conversion_possible_fn: |arguments| {
                    if let Node::Value(ValueNode::Constant(constant)) = arguments[1] {
                        (constant - 0.5).abs() <= 0.001
                    } else {
                        false
                    }
                },
                convert_fn: |mut arguments| {
                    arguments.remove(1);
                    arguments
                },
            },
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_unary_operator_by_name() {
        let settings = Settings::default();
        assert_eq!(
            settings.operators[5],
            settings.find_unary_operator_by_name("+").unwrap()
        );
        assert_eq!(None, settings.find_unary_operator_by_name("&"));
    }

    #[test]
    fn test_find_binary_operator_by_name() {
        let settings = Settings::default();
        assert_eq!(
            settings.operators[0],
            settings.find_binary_operator_by_name("+").unwrap()
        );
        assert_eq!(None, settings.find_binary_operator_by_name("&"));
    }

    #[test]
    fn test_find_function_by_name() {
        let settings = Settings::default();
        assert_eq!(
            settings.functions[0],
            settings.find_function_by_name("abs").unwrap()
        );
        assert_eq!(None, settings.find_function_by_name("fn"));
    }

    #[test]
    fn test_find_converters() {
        let settings = Settings::default();
        assert_eq!(
            vec![&settings.converters[4], &settings.converters[5]],
            settings.find_converters(&ConverterOperation::Operator(
                settings.find_binary_operator_by_name("^").unwrap()
            ))
        );
        let empty_vec: Vec<&Converter> = vec![];
        assert_eq!(
            empty_vec,
            settings.find_converters(&ConverterOperation::Operator(
                settings.find_binary_operator_by_name("+").unwrap()
            ))
        );
    }

    #[test]
    fn test_conversion_is_not_possible() {
        let settings = Settings::default();
        let log = settings.find_function_by_name("log").unwrap();
        let expected_output_data = ConvertOutputData {
            operation: ConverterOperation::Function(Rc::clone(&log)),
            arguments: vec![
                Node::Value(ValueNode::Variable(String::from("x"))),
                Node::Value(ValueNode::Constant(10.0)),
            ],
        };
        let actual_output_data = settings.convert(
            ConverterOperation::Function(Rc::clone(&log)),
            vec![
                Node::Value(ValueNode::Variable(String::from("x"))),
                Node::Value(ValueNode::Constant(10.0)),
            ],
        );
        assert_eq!(expected_output_data, actual_output_data);
    }

    #[test]
    fn test_convert_log_to_ln() {
        let settings = Settings::default();
        let expected_output_data = ConvertOutputData {
            operation: ConverterOperation::Function(settings.find_function_by_name("ln").unwrap()),
            arguments: vec![Node::Value(ValueNode::Variable(String::from("x")))],
        };
        let actual_output_data = settings.convert(
            ConverterOperation::Function(settings.find_function_by_name("log").unwrap()),
            vec![
                Node::Value(ValueNode::Constant(E + 0.0001)),
                Node::Value(ValueNode::Variable(String::from("x"))),
            ],
        );
        assert_eq!(expected_output_data, actual_output_data);
    }

    #[test]
    fn test_convert_pow_to_sqrt() {
        let settings = Settings::default();
        let expected_output_data = ConvertOutputData {
            operation: ConverterOperation::Operator(
                settings.find_binary_operator_by_name("^").unwrap(),
            ),
            arguments: vec![
                Node::Value(ValueNode::Constant(2.0)),
                Node::Value(ValueNode::Constant(0.5)),
            ],
        };
        let actual_output_data = settings.convert(
            ConverterOperation::Function(settings.find_function_by_name("sqrt").unwrap()),
            vec![Node::Value(ValueNode::Constant(2.0))],
        );
        assert_eq!(expected_output_data, actual_output_data);
    }
}
