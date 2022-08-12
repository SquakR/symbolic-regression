//! Module with common types.
use crate::expression_tree::{Node, OperationNode};
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::fmt;
use std::rc::Rc;

pub trait Operation {
    fn compute(&self, arguments: &[f64]) -> f64;
    fn get_name(&self) -> &str;
}

#[derive(Clone)]
pub struct Operator {
    pub name: String,
    pub precedence: u8,
    pub associativity: Associativity,
    pub arguments_number: usize,
    pub complexity: u32,
    pub compute_fn: fn(arguments: &[f64]) -> f64,
}

impl Operation for Operator {
    fn compute(&self, arguments: &[f64]) -> f64 {
        if arguments.len() != self.arguments_number {
            panic!(
                "The operator `{}` expected {} arguments, but received {}.",
                self,
                self.arguments_number,
                arguments.len()
            );
        }
        (self.compute_fn)(arguments)
    }
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

impl fmt::Debug for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Operator")
            .field("name", &self.name)
            .field("arguments_number", &self.arguments_number)
            .field("precedence", &self.precedence)
            .field("associativity", &self.associativity)
            .field("complexity", &self.complexity)
            .finish()
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Operator {
    fn eq(&self, other: &Operator) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl Operator {
    pub fn is_computed_before(&self, other: &Operator) -> bool {
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

#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub arguments_number: usize,
    pub complexity: u32,
    pub compute_fn: fn(arguments: &[f64]) -> f64,
}

impl Operation for Function {
    fn compute(&self, arguments: &[f64]) -> f64 {
        if arguments.len() != self.arguments_number {
            panic!(
                "The function `{}` expected {} arguments, but received {}.",
                self,
                self.arguments_number,
                arguments.len()
            );
        }
        (self.compute_fn)(arguments)
    }
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Function")
            .field("name", &self.name)
            .field("arguments_number", &self.arguments_number)
            .field("complexity", &self.complexity)
            .finish()
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Function) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Associativity {
    Left,
    Right,
}

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
    use super::*;

    mod operator_tests {
        use super::*;

        #[test]
        fn test_debug() {
            let plus_operator = create_plus_operator();
            assert_eq!(
                "Operator { name: \"+\", arguments_number: 2, precedence: 1, associativity: Left, complexity: 1 }",
                format!("{:?}", plus_operator)
            );
        }

        #[test]
        fn test_display() {
            let plus_operator = create_plus_operator();
            assert_eq!("+", format!("{}", plus_operator));
        }

        #[test]
        fn test_eq() {
            let plus_operator1 = create_plus_operator();
            let mut plus_operator2 = create_plus_operator();
            assert!(plus_operator1 == plus_operator2);
            plus_operator2.name = String::from("-");
            assert!(plus_operator1 != plus_operator2);
        }

        #[test]
        fn test_compute() {
            let plus_operator = create_plus_operator();
            assert_eq!(3.0, plus_operator.compute(&[1.0, 2.0]));
        }

        #[test]
        #[should_panic(expected = "The operator `+` expected 2 arguments, but received 1.")]
        fn test_compute_panic() {
            let plus_operator = create_plus_operator();
            plus_operator.compute(&[1.0]);
        }

        #[test]
        fn test_operator_is_computed_before() {
            let plus = create_plus_operator();
            let minus = create_minis_operator();
            let asterisk = create_asterisk_operator();
            let slash = create_slash_operator();
            let circumflex = create_circumflex_operator();
            assert!(plus.is_computed_before(&plus));
            assert!(plus.is_computed_before(&minus));
            assert!(!plus.is_computed_before(&asterisk));
            assert!(circumflex.is_computed_before(&slash));
        }
    }

    mod function_tests {
        use super::*;

        #[test]
        fn test_debug() {
            let test_function = create_test_function();
            assert_eq!(
                "Function { name: \"sin\", arguments_number: 1, complexity: 1 }",
                format!("{:?}", test_function)
            );
        }

        #[test]
        fn test_display() {
            let test_function = create_test_function();
            assert_eq!("sin", format!("{}", test_function));
        }

        #[test]
        fn test_eq() {
            let test_function1 = create_test_function();
            let mut test_function2 = create_test_function();
            assert!(test_function1 == test_function2);
            test_function2.name = String::from("cos");
            assert!(test_function1 != test_function2);
        }

        #[test]
        fn test_compute() {
            let test_function = create_test_function();
            assert_eq!(2.0_f64.sin(), test_function.compute(&[2.0]));
        }

        #[test]
        #[should_panic(expected = "The function `sin` expected 1 arguments, but received 2.")]
        fn test_compute_panic() {
            let test_function = create_test_function();
            test_function.compute(&[1.0, 2.0]);
        }

        fn create_test_function() -> Function {
            Function {
                name: String::from("sin"),
                arguments_number: 1,
                complexity: 1,
                compute_fn: |arguments| arguments[0].sin(),
            }
        }
    }

    mod converter_tests {
        use super::*;
        use crate::expression_tree::ValueNode;
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
            let operator_output_data = ConvertOutputData {
                operation: ConverterOperation::Operator(Rc::new(create_plus_operator())),
                arguments: vec![
                    Node::Value(ValueNode::Variable(String::from("x"))),
                    Node::Value(ValueNode::Constant(1.0)),
                ],
            };
            let expected_node = Node::Operator(OperationNode {
                operation: Rc::new(create_plus_operator()),
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

    fn create_plus_operator() -> Operator {
        Operator {
            name: String::from("+"),
            arguments_number: 2,
            precedence: 1,
            associativity: Associativity::Left,
            complexity: 1,
            compute_fn: |arguments| arguments[0] + arguments[1],
        }
    }

    fn create_minis_operator() -> Operator {
        Operator {
            name: String::from("-"),
            arguments_number: 2,
            precedence: 1,
            associativity: Associativity::Left,
            complexity: 1,
            compute_fn: |arguments| arguments[0] - arguments[1],
        }
    }

    fn create_asterisk_operator() -> Operator {
        Operator {
            name: String::from("*"),
            arguments_number: 2,
            precedence: 2,
            associativity: Associativity::Left,
            complexity: 2,
            compute_fn: |arguments| arguments[0] * arguments[1],
        }
    }

    fn create_slash_operator() -> Operator {
        Operator {
            name: String::from("/"),
            arguments_number: 2,
            precedence: 2,
            associativity: Associativity::Left,
            complexity: 2,
            compute_fn: |arguments| arguments[0] / arguments[1],
        }
    }

    fn create_circumflex_operator() -> Operator {
        Operator {
            name: String::from("^"),
            arguments_number: 2,
            precedence: 3,
            associativity: Associativity::Right,
            complexity: 3,
            compute_fn: |arguments| arguments[0].powf(arguments[1]),
        }
    }
}
