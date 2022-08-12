//! Expression tree core functionality module.
use crate::settings::Settings;
use crate::types::{ConverterOperation, Function, Operation, Operator};
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, PartialEq, Serialize)]
pub struct ExpressionTree {
    pub root: Node,
    pub variables: Vec<String>,
}

pub trait Computable {
    /// Compute a node by computing all child nodes and performing a node operation.
    /// Return `ComputeError` if some node contains a variable instead of a constant.
    /// For computation, the idea of ​​the composite pattern is used.
    fn compute(&self) -> Result<f64, ComputeError>;
    /// Simplify a node by replacing all child nodes that can be computed with values.
    /// For example, sin(x + 2,0 * (3.0 + 2.0)) -> sin(x + 14.0)
    fn simplify(&mut self) -> ();
}

#[derive(Debug, Clone)]
pub struct ComputeError {
    pub message: String,
}

impl ComputeError {
    fn new(variable: &str) -> ComputeError {
        ComputeError {
            message: format!("{} variable is not a constant.", variable),
        }
    }
}

impl Computable for ExpressionTree {
    fn compute(&self) -> Result<f64, ComputeError> {
        self.root.compute()
    }
    fn simplify(&mut self) -> () {
        match self.compute() {
            Ok(value) => self.root = Node::Value(ValueNode::Constant(value)),
            Err(_) => self.root.simplify(),
        }
    }
}

impl fmt::Display for ExpressionTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.root)
    }
}

impl ExpressionTree {
    /// Return a new ExpressionTree where variables have been replaced with values from the `variables` HashMap.
    /// Panic if `variables` HaspMap contains non-existing variables.
    pub fn subs(&self, variables: &HashMap<&str, f64>) -> ExpressionTree {
        for &key in variables.keys() {
            if !self.variables.iter().any(|variable| variable == key) {
                panic!("Expression tree does not contain {} variable.", key);
            }
        }
        ExpressionTree {
            root: ExpressionTree::subs_node(&self.root, variables),
            variables: self
                .variables
                .clone()
                .into_iter()
                .filter(|variable| !variables.keys().any(|key| key == variable))
                .collect(),
        }
    }
    /// Return a new ExpressionTree where internal functions and operations have been replaced with output ones.
    pub fn create_output(&self, settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: ExpressionTree::create_output_node(&self.root, settings),
            variables: self.variables.clone(),
        }
    }
    fn subs_node(node: &Node, variables: &HashMap<&str, f64>) -> Node {
        match node {
            Node::Operator(operator_node) => Node::Operator(OperationNode {
                operation: Rc::clone(&operator_node.operation),
                arguments: operator_node
                    .arguments
                    .iter()
                    .map(|argument_node| ExpressionTree::subs_node(argument_node, variables))
                    .collect::<Vec<Node>>(),
            }),
            Node::Function(function_node) => Node::Function(OperationNode {
                operation: Rc::clone(&function_node.operation),
                arguments: function_node
                    .arguments
                    .iter()
                    .map(|argument_node| ExpressionTree::subs_node(argument_node, variables))
                    .collect::<Vec<Node>>(),
            }),
            Node::Value(value_node) => match value_node {
                ValueNode::Constant(value) => Node::Value(ValueNode::Constant(*value)),
                ValueNode::Variable(variable) => match variables.get(variable.as_str()) {
                    Some(constant) => Node::Value(ValueNode::Constant(*constant)),
                    None => Node::Value(ValueNode::Variable(variable.to_string())),
                },
            },
        }
    }
    fn create_output_node(node: &Node, settings: &Settings) -> Node {
        match node {
            Node::Operator(operator_node) => {
                let mut output_data = settings.convert(
                    ConverterOperation::Operator(Rc::clone(&operator_node.operation)),
                    operator_node.arguments.clone(),
                );
                output_data.arguments = output_data
                    .arguments
                    .iter()
                    .map(|argument_node| {
                        ExpressionTree::create_output_node(argument_node, settings)
                    })
                    .collect::<Vec<Node>>();
                output_data.to_node()
            }
            Node::Function(function_node) => {
                let mut output_data = settings.convert(
                    ConverterOperation::Function(Rc::clone(&function_node.operation)),
                    function_node.arguments.clone(),
                );
                output_data.arguments = output_data
                    .arguments
                    .iter()
                    .map(|argument_node| {
                        ExpressionTree::create_output_node(argument_node, settings)
                    })
                    .collect::<Vec<Node>>();
                output_data.to_node()
            }
            Node::Value(value_node) => Node::Value(value_node.clone()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    Operator(OperationNode<Operator>),
    Function(OperationNode<Function>),
    Value(ValueNode),
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Operator(operator_node) => {
                if operator_node.operation.arguments_number == 2 {
                    f.write_str(
                        &operator_node
                            .arguments
                            .iter()
                            .map(|argument| {
                                if let Node::Operator(argument_node) = argument {
                                    if !argument_node
                                        .operation
                                        .is_computed_before(&operator_node.operation)
                                    {
                                        return format!("({})", argument);
                                    }
                                }
                                format!("{}", argument)
                            })
                            .collect::<Vec<String>>()
                            .join(&format!(" {} ", operator_node.operation.name)),
                    )
                } else {
                    write!(f, "{}", operator_node)
                }
            }
            Node::Function(function_node) => write!(f, "{}", function_node),
            Node::Value(value_node) => write!(f, "{}", value_node),
        }
    }
}

impl Computable for Node {
    fn compute(&self) -> Result<f64, ComputeError> {
        match self {
            Node::Operator(operator_node) => operator_node.compute(),
            Node::Function(function_node) => function_node.compute(),
            Node::Value(value) => value.compute(),
        }
    }
    fn simplify(&mut self) -> () {
        match self {
            Node::Operator(operator_node) => operator_node.simplify(),
            Node::Function(function_node) => function_node.simplify(),
            Node::Value(value) => value.simplify(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct OperationNode<T: Operation> {
    pub operation: Rc<T>,
    pub arguments: Vec<Node>,
}

impl fmt::Display for OperationNode<Operator> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.operation.arguments_number {
            1 => write!(f, "{}{}", self.operation.get_name(), self.arguments[0]),
            2 => write!(
                f,
                "{} {} {}",
                self.arguments[0],
                self.operation.get_name(),
                self.arguments[1]
            ),
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for OperationNode<Function> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.operation.get_name(),
            self.arguments
                .iter()
                .map(|argument| argument.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl<T: Operation> Computable for OperationNode<T> {
    fn compute(&self) -> Result<f64, ComputeError> {
        let mut arguments_result = vec![];
        for argument in &self.arguments {
            let argument_result = argument.compute()?;
            arguments_result.push(argument_result);
        }
        Ok(self.operation.compute(&arguments_result))
    }
    fn simplify(&mut self) -> () {
        for argument in self.arguments.iter_mut() {
            let computation_result = match argument {
                Node::Operator(operator_node) => Some(operator_node.compute()),
                Node::Function(function_node) => Some(function_node.compute()),
                _ => None,
            };
            if let Some(result) = computation_result {
                match result {
                    Ok(value) => *argument = Node::Value(ValueNode::Constant(value)),
                    Err(_) => (argument).simplify(),
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueNode {
    Variable(String),
    Constant(f64),
}

impl fmt::Display for ValueNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueNode::Variable(variable) => write!(f, "{}", variable),
            ValueNode::Constant(constant) => write!(f, "{}", constant),
        }
    }
}

impl Computable for ValueNode {
    fn compute(&self) -> Result<f64, ComputeError> {
        match self {
            ValueNode::Variable(variable) => Err(ComputeError::new(variable)),
            ValueNode::Constant(constant) => Ok(*constant),
        }
    }
    fn simplify(&mut self) -> () {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::E;

    #[test]
    #[should_panic(expected = "Expression tree does not contain y variable.")]
    fn test_subs_panics_with_wrong_variable() {
        let settings = Settings::default();
        let tree = create_tree_with_variables(&settings);
        tree.subs(&HashMap::from([("y", 2.0)]));
    }

    #[test]
    fn test_subs_x1_variable() {
        let settings = Settings::default();
        let tree = create_tree_with_variables(&settings);
        let expected_tree = ExpressionTree {
            root: Node::Operator(OperationNode {
                operation: settings.find_binary_operator_by_name("+").unwrap(),
                arguments: vec![
                    Node::Operator(OperationNode {
                        operation: settings.find_binary_operator_by_name("-").unwrap(),
                        arguments: vec![
                            Node::Value(ValueNode::Constant(2.0)),
                            Node::Value(ValueNode::Constant(1.0)),
                        ],
                    }),
                    Node::Operator(OperationNode {
                        operation: settings.find_binary_operator_by_name("*").unwrap(),
                        arguments: vec![
                            Node::Function(OperationNode {
                                operation: settings.find_function_by_name("sin").unwrap(),
                                arguments: vec![Node::Value(ValueNode::Constant(2.0))],
                            }),
                            Node::Value(ValueNode::Variable(String::from("x2"))),
                        ],
                    }),
                ],
            }),
            variables: vec![String::from("x2")],
        };
        let actual_tree = tree.subs(&HashMap::from([("x1", 2.0)]));
        assert_eq!(expected_tree, actual_tree);
    }

    #[test]
    fn test_create_output() {
        let settings = Settings::default();
        let tree = create_tree_to_output(&settings);
        let expected_tree = ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("sin").unwrap(),
                arguments: vec![Node::Function(OperationNode {
                    operation: settings.find_function_by_name("exp").unwrap(),
                    arguments: vec![Node::Function(OperationNode {
                        operation: settings.find_function_by_name("ln").unwrap(),
                        arguments: vec![Node::Value(ValueNode::Variable(String::from("x")))],
                    })],
                })],
            }),
            variables: vec![String::from("x")],
        };
        let actual_tree = tree.create_output(&settings);
        assert_eq!(expected_tree, actual_tree)
    }

    #[test]
    fn test_display_value_node() {
        let variable_node = ValueNode::Variable(String::from("x"));
        assert_eq!("x", variable_node.to_string());
        let constant_node = ValueNode::Constant(1.0);
        assert_eq!("1", constant_node.to_string());
    }

    #[test]
    fn test_display_operation_node_operator() {
        let settings = Settings::default();
        let one_argument_node = OperationNode {
            operation: settings.find_unary_operator_by_name("-").unwrap(),
            arguments: vec![Node::Value(ValueNode::Constant(1.0))],
        };
        assert_eq!("-1", one_argument_node.to_string());
        let two_arguments_node = OperationNode {
            operation: settings.find_binary_operator_by_name("+").unwrap(),
            arguments: vec![
                Node::Value(ValueNode::Variable(String::from("x"))),
                Node::Value(ValueNode::Constant(1.0)),
            ],
        };
        assert_eq!("x + 1", two_arguments_node.to_string());
    }

    #[test]
    fn test_display_operation_node_function() {
        let settings = Settings::default();
        let sin_node = OperationNode {
            operation: settings.find_function_by_name("sin").unwrap(),
            arguments: vec![Node::Value(ValueNode::Constant(1.0))],
        };
        assert_eq!("sin(1)", sin_node.to_string());
        let log_node = OperationNode {
            operation: settings.find_function_by_name("log").unwrap(),
            arguments: vec![
                Node::Value(ValueNode::Constant(2.0)),
                Node::Value(ValueNode::Variable(String::from("x"))),
            ],
        };
        assert_eq!("log(2, x)", log_node.to_string());
    }

    #[test]
    fn test_display_expression_tree() {
        let settings = Settings::default();
        let tree = create_tree_to_display(&settings);
        assert_eq!(
            "(log(0.5, 1) * x1 / 1.5) ^ (2 ^ 2.5 * (3 + x2))",
            tree.to_string()
        );
    }

    #[test]
    fn test_value_variable_compute() {
        if let Ok(_) = ValueNode::Variable(String::from("x")).compute() {
            panic!("Computing a value with a variable must return a `ComputeError`.");
        }
    }

    #[test]
    fn test_value_constant_compute() -> Result<(), ComputeError> {
        let actual = ValueNode::Constant(1.0).compute()?;
        assert_eq!(1.0_f64, actual);
        Ok(())
    }

    #[test]
    fn test_operation_node_compute() -> Result<(), ComputeError> {
        let settings = Settings::default();
        let actual = OperationNode {
            operation: settings.find_binary_operator_by_name("+").unwrap(),
            arguments: vec![
                Node::Value(ValueNode::Constant(1.0)),
                Node::Value(ValueNode::Constant(2.0)),
            ],
        }
        .compute()?;
        assert_eq!(3.0, actual);
        Ok(())
    }

    #[test]
    fn test_node_value_node_compute() -> Result<(), ComputeError> {
        let actual = Node::Value(ValueNode::Constant(1.0)).compute()?;
        assert_eq!(1.0_f64, actual);
        Ok(())
    }

    #[test]
    fn test_node_operator_compute() -> Result<(), ComputeError> {
        let settings = Settings::default();
        let actual = Node::Operator(OperationNode {
            operation: settings.find_binary_operator_by_name("+").unwrap(),
            arguments: vec![
                Node::Value(ValueNode::Constant(1.0)),
                Node::Value(ValueNode::Constant(2.0)),
            ],
        })
        .compute()?;
        assert_eq!(1.0 + 2.0, actual);
        Ok(())
    }

    #[test]
    fn test_node_function_compute() -> Result<(), ComputeError> {
        let settings = Settings::default();
        let actual = Node::Function(OperationNode {
            operation: settings.find_function_by_name("sin").unwrap(),
            arguments: vec![Node::Value(ValueNode::Constant(0.5))],
        })
        .compute()?;
        assert_eq!(0.5_f64.sin(), actual);
        Ok(())
    }

    #[test]
    fn test_tree_with_variables_compute() {
        let settings = Settings::default();
        if let Ok(_) = create_tree_with_variables(&settings).compute() {
            panic!("Computing a tree with a variable must return a `ComputeError`.");
        }
    }

    #[test]
    fn test_tree_without_variables_compute() -> Result<(), ComputeError> {
        let settings = Settings::default();
        let actual = create_tree_without_variables(&settings).compute()?;
        assert_eq!(2.0_f64 * 3.0_f64 + 5.0_f64.sin() * 10.0_f64, actual);
        Ok(())
    }

    #[test]
    fn test_tree_without_variables_simplify() {
        let settings = Settings::default();
        let mut tree = create_tree_without_variables(&settings);
        tree.simplify();
        let expected_tree = ExpressionTree {
            root: Node::Value(ValueNode::Constant(2.0 * 3.0 + 5.0_f64.sin() * 10.0)),
            variables: vec![],
        };
        assert_eq!(expected_tree, tree);
    }

    #[test]
    fn test_tree_to_simplify_simplify() {
        let settings = Settings::default();
        let mut tree = create_tree_to_simplify(&settings);
        tree.simplify();
        let expected_tree = ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("sin").unwrap(),
                arguments: vec![Node::Operator(OperationNode {
                    operation: settings.find_binary_operator_by_name("+").unwrap(),
                    arguments: vec![
                        Node::Value(ValueNode::Variable(String::from("x"))),
                        Node::Value(ValueNode::Constant(14.0)),
                    ],
                })],
            }),
            variables: vec![String::from("x")],
        };
        assert_eq!(expected_tree, tree);
    }

    fn create_tree_with_variables(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Operator(OperationNode {
                operation: settings.find_binary_operator_by_name("+").unwrap(),
                arguments: vec![
                    Node::Operator(OperationNode {
                        operation: settings.find_binary_operator_by_name("-").unwrap(),
                        arguments: vec![
                            Node::Value(ValueNode::Variable(String::from("x1"))),
                            Node::Value(ValueNode::Constant(1.0)),
                        ],
                    }),
                    Node::Operator(OperationNode {
                        operation: settings.find_binary_operator_by_name("*").unwrap(),
                        arguments: vec![
                            Node::Function(OperationNode {
                                operation: settings.find_function_by_name("sin").unwrap(),
                                arguments: vec![Node::Value(ValueNode::Variable(String::from(
                                    "x1",
                                )))],
                            }),
                            Node::Value(ValueNode::Variable(String::from("x2"))),
                        ],
                    }),
                ],
            }),
            variables: vec![String::from("x1"), String::from("x2")],
        }
    }

    fn create_tree_without_variables(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Operator(OperationNode {
                operation: settings.find_binary_operator_by_name("+").unwrap(),
                arguments: vec![
                    Node::Operator(OperationNode {
                        operation: settings.find_binary_operator_by_name("*").unwrap(),
                        arguments: vec![
                            Node::Value(ValueNode::Constant(2.0)),
                            Node::Value(ValueNode::Constant(3.0)),
                        ],
                    }),
                    Node::Operator(OperationNode {
                        operation: settings.find_binary_operator_by_name("*").unwrap(),
                        arguments: vec![
                            Node::Function(OperationNode {
                                operation: settings.find_function_by_name("sin").unwrap(),
                                arguments: vec![Node::Value(ValueNode::Constant(5.0))],
                            }),
                            Node::Value(ValueNode::Constant(10.0)),
                        ],
                    }),
                ],
            }),
            variables: vec![],
        }
    }

    fn create_tree_to_display(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Operator(OperationNode {
                operation: settings.find_binary_operator_by_name("^").unwrap(),
                arguments: vec![
                    Node::Operator(OperationNode {
                        operation: settings.find_binary_operator_by_name("*").unwrap(),
                        arguments: vec![
                            Node::Function(OperationNode {
                                operation: settings.find_function_by_name("log").unwrap(),
                                arguments: vec![
                                    Node::Value(ValueNode::Constant(0.5)),
                                    Node::Value(ValueNode::Constant(1.0)),
                                ],
                            }),
                            Node::Operator(OperationNode {
                                operation: settings.find_binary_operator_by_name("/").unwrap(),
                                arguments: vec![
                                    Node::Value(ValueNode::Variable(String::from("x1"))),
                                    Node::Value(ValueNode::Constant(1.5)),
                                ],
                            }),
                        ],
                    }),
                    Node::Operator(OperationNode {
                        operation: settings.find_binary_operator_by_name("*").unwrap(),
                        arguments: vec![
                            Node::Operator(OperationNode {
                                operation: settings.find_binary_operator_by_name("^").unwrap(),
                                arguments: vec![
                                    Node::Value(ValueNode::Constant(2.0)),
                                    Node::Value(ValueNode::Constant(2.5)),
                                ],
                            }),
                            Node::Operator(OperationNode {
                                operation: settings.find_binary_operator_by_name("+").unwrap(),
                                arguments: vec![
                                    Node::Value(ValueNode::Constant(3.0)),
                                    Node::Value(ValueNode::Variable(String::from("x2"))),
                                ],
                            }),
                        ],
                    }),
                ],
            }),
            variables: vec![String::from("x1"), String::from("x2")],
        }
    }

    fn create_tree_to_output(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("sin").unwrap(),
                arguments: vec![Node::Operator(OperationNode {
                    operation: settings.find_binary_operator_by_name("^").unwrap(),
                    arguments: vec![
                        Node::Value(ValueNode::Constant(E + 0.0001)),
                        Node::Function(OperationNode {
                            operation: settings.find_function_by_name("log").unwrap(),
                            arguments: vec![
                                Node::Value(ValueNode::Constant(E - 0.0001)),
                                Node::Value(ValueNode::Variable(String::from("x"))),
                            ],
                        }),
                    ],
                })],
            }),
            variables: vec![String::from("x")],
        }
    }

    fn create_tree_to_simplify(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("sin").unwrap(),
                arguments: vec![Node::Operator(OperationNode {
                    operation: settings.find_binary_operator_by_name("+").unwrap(),
                    arguments: vec![
                        Node::Value(ValueNode::Variable(String::from("x"))),
                        Node::Operator(OperationNode {
                            operation: settings.find_binary_operator_by_name("*").unwrap(),
                            arguments: vec![
                                Node::Value(ValueNode::Constant(2.0)),
                                Node::Operator(OperationNode {
                                    operation: settings.find_binary_operator_by_name("+").unwrap(),
                                    arguments: vec![
                                        Node::Value(ValueNode::Constant(3.0)),
                                        Node::Value(ValueNode::Constant(4.0)),
                                    ],
                                }),
                            ],
                        }),
                    ],
                })],
            }),
            variables: vec![String::from("x")],
        }
    }
}
