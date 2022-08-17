//! Module for getting default settings.
use super::settings::{NodeProbability, Settings};
use super::types::{Converter, ConverterOperation};
use crate::expression_tree::{Associativity, Function, Node, Operator, ValueNode};
use std::f64::{consts::E, consts::PI, NAN};
use std::rc::Rc;

impl Settings {
    pub fn default() -> Settings {
        let mut settings = Settings {
            operators: Settings::get_default_operators(),
            functions: Settings::get_default_functions(),
            converters: vec![],
            variable_complexity: 1,
            constant_complexity: 1,
            get_node_probability_fn: |tree_complexity| {
                let operation_node_probability = 2.0 / tree_complexity as f64;
                NodeProbability {
                    operator_node: operation_node_probability,
                    function_node: operation_node_probability,
                    value_node: 1.0 - operation_node_probability * 2.0,
                }
            },
        };
        settings.converters = settings.get_default_converters();
        settings
    }
    pub fn get_default_operators() -> Vec<Rc<Operator>> {
        vec![
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
        ]
    }
    pub fn get_default_functions() -> Vec<Rc<Function>> {
        vec![
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
        ]
    }
    pub fn get_default_converters(&self) -> Vec<Converter> {
        let circumflex = self.find_binary_operator_by_name("^").unwrap();
        let log = self.find_function_by_name("log").unwrap();
        let ln = self.find_function_by_name("ln").unwrap();
        let exp = self.find_function_by_name("exp").unwrap();
        let sqrt = self.find_function_by_name("sqrt").unwrap();
        vec![
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
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_node_probability_fn() {
        let settings = Settings::default();
        let expected_node_probability = NodeProbability {
            operator_node: 0.2,
            function_node: 0.2,
            value_node: 0.6,
        };
        let actual_node_probability = settings.get_node_probability(10);
        assert_eq!(expected_node_probability, actual_node_probability);
    }
}
