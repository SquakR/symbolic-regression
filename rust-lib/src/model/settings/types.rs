//! Module with settings types.
use crate::expression_tree::types::{Function, Node, OperationNode, Operator};
use std::cmp::PartialEq;
use std::fmt;
use std::rc::Rc;

pub struct Converter {
    pub from: ConverterOperation,
    pub to: ConverterOperation,
    pub is_conversion_possible_fn: fn(arguments: &[Node]) -> bool,
    pub convert_fn: fn(arguments: Vec<Node>) -> Vec<Node>,
}

impl fmt::Debug for Converter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Converter")
            .field("from", &self.from)
            .field("to", &self.to)
            .finish()
    }
}

impl PartialEq for Converter {
    fn eq(&self, other: &Converter) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl Converter {
    pub fn is_conversion_possible(
        &self,
        operation: &ConverterOperation,
        arguments: &[Node],
    ) -> bool {
        *operation == self.from && (self.is_conversion_possible_fn)(arguments)
    }
    pub fn convert(&self, arguments: Vec<Node>) -> ConvertOutputData {
        ConvertOutputData {
            operation: self.to.clone(),
            arguments: (self.convert_fn)(arguments),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ConvertOutputData {
    pub operation: ConverterOperation,
    pub arguments: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConverterOperation {
    Operator(Rc<Operator>),
    Function(Rc<Function>),
}

impl ConvertOutputData {
    pub fn to_node(self) -> Node {
        match self.operation {
            ConverterOperation::Function(function) => Node::Function(OperationNode {
                operation: Rc::clone(&function),
                arguments: self.arguments,
            }),
            ConverterOperation::Operator(operator) => Node::Operator(OperationNode {
                operation: Rc::clone(&operator),
                arguments: self.arguments,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::core::Settings;
    use super::*;

    mod converter_tests {
        use super::*;
        use crate::expression_tree::types::ValueNode;
        use std::f64::consts::E;

        #[test]
        fn test_debug() {
            let test_converter = create_log_to_ln_converter();
            assert_eq!(
                format!(
                    "Converter {{ from: {:?}, to: {:?} }}",
                    ConverterOperation::Function(Rc::new(create_log_function())),
                    ConverterOperation::Function(Rc::new(create_ln_function())),
                ),
                format!("{:?}", test_converter)
            );
        }
        #[test]
        fn test_eq() {
            let test_converter1 = create_log_to_ln_converter();
            let mut test_converter2 = create_log_to_ln_converter();
            assert!(test_converter1 == test_converter2);
            test_converter2.from = ConverterOperation::Function(Rc::new(create_ln_function()));
            assert!(test_converter1 != test_converter2);
        }

        #[test]
        fn test_is_conversion_possible() {
            let log_function = Rc::new(create_log_function());
            let ln_function = Rc::new(create_ln_function());
            let converter = create_log_to_ln_converter();
            let possible_arguments = vec![
                Node::Value(ValueNode::Constant(E + 0.0001)),
                Node::Value(ValueNode::Constant(10.0)),
            ];
            let not_possible_arguments = vec![
                Node::Value(ValueNode::Constant(E + 0.01)),
                Node::Value(ValueNode::Constant(10.0)),
            ];
            assert!(converter.is_conversion_possible(
                &ConverterOperation::Function(Rc::clone(&log_function)),
                &possible_arguments
            ));
            assert!(!converter.is_conversion_possible(
                &ConverterOperation::Function(Rc::clone(&ln_function)),
                &not_possible_arguments
            ));
        }

        #[test]
        fn test_convert() {
            let ln_function = Rc::new(create_ln_function());
            let converter = create_log_to_ln_converter();
            let arguments = vec![
                Node::Value(ValueNode::Constant(E + 0.0001)),
                Node::Value(ValueNode::Constant(10.0)),
            ];
            assert_eq!(
                ConvertOutputData {
                    operation: ConverterOperation::Function(Rc::clone(&ln_function)),
                    arguments: vec![Node::Value(ValueNode::Constant(10.0))]
                },
                converter.convert(arguments)
            );
        }

        #[test]
        fn test_convert_output_data_to_node_operator() {
            let settings = Settings::default();
            let operator_output_data = ConvertOutputData {
                operation: ConverterOperation::Operator(
                    settings.find_binary_operator_by_name("+").unwrap(),
                ),
                arguments: vec![
                    Node::Value(ValueNode::Variable(String::from("x"))),
                    Node::Value(ValueNode::Constant(1.0)),
                ],
            };
            let expected_node = Node::Operator(OperationNode {
                operation: settings.find_binary_operator_by_name("+").unwrap(),
                arguments: vec![
                    Node::Value(ValueNode::Variable(String::from("x"))),
                    Node::Value(ValueNode::Constant(1.0)),
                ],
            });
            assert_eq!(expected_node, operator_output_data.to_node());
        }

        #[test]
        fn test_convert_output_data_to_node_function() {
            let function_output_data = ConvertOutputData {
                operation: ConverterOperation::Function(Rc::new(create_log_function())),
                arguments: vec![
                    Node::Value(ValueNode::Variable(String::from("x"))),
                    Node::Value(ValueNode::Constant(1.0)),
                ],
            };
            let expected_node = Node::Function(OperationNode {
                operation: Rc::new(create_log_function()),
                arguments: vec![
                    Node::Value(ValueNode::Variable(String::from("x"))),
                    Node::Value(ValueNode::Constant(1.0)),
                ],
            });
            assert_eq!(expected_node, function_output_data.to_node());
        }

        fn create_log_function() -> Function {
            Function {
                name: String::from("log"),
                arguments_number: 2,
                complexity: 1,
                compute_fn: |arguments| arguments[0].log(arguments[1]),
            }
        }

        fn create_ln_function() -> Function {
            Function {
                name: String::from("ln"),
                arguments_number: 1,
                complexity: 1,
                compute_fn: |arguments| arguments[0].ln(),
            }
        }

        fn create_log_to_ln_converter() -> Converter {
            Converter {
                from: ConverterOperation::Function(Rc::new(create_log_function())),
                to: ConverterOperation::Function(Rc::new(create_ln_function())),
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
            }
        }
    }
}
