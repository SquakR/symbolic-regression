/// Expression tree core functionality module.
use crate::types::{Function, Operation, Operator};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ExpressionTree<'a> {
    pub root: Node<'a>,
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

impl<'a> Computable for ExpressionTree<'a> {
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

impl<'a> ExpressionTree<'a> {
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
    fn subs_node(node: &'a Node, variables: &HashMap<&str, f64>) -> Node<'a> {
        match node {
            Node::Operator(operation_node) => Node::Operator(OperationNode {
                operation: operation_node.operation,
                arguments: operation_node
                    .arguments
                    .iter()
                    .map(|argument_node| ExpressionTree::subs_node(argument_node, variables))
                    .collect::<Vec<Node>>(),
            }),
            Node::Function(operation_node) => Node::Function(OperationNode {
                operation: operation_node.operation,
                arguments: operation_node
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
}

#[derive(Debug, PartialEq)]
pub enum Node<'a> {
    Operator(OperationNode<'a, Operator>),
    Function(OperationNode<'a, Function>),
    Value(ValueNode),
}

impl<'a> Computable for Node<'a> {
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

#[derive(Debug, PartialEq)]
pub struct OperationNode<'a, T: Operation> {
    operation: &'a T,
    arguments: Vec<Node<'a>>,
}

impl<'a, T: Operation> Computable for OperationNode<'a, T> {
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
            let mut computation_result: Option<Result<f64, ComputeError>> = None;
            match argument {
                Node::Operator(operator_node) => computation_result = Some(operator_node.compute()),
                Node::Function(function_node) => computation_result = Some(function_node.compute()),
                _ => {}
            }
            if let Some(result) = computation_result {
                match result {
                    Ok(value) => *argument = Node::Value(ValueNode::Constant(value)),
                    Err(_) => (argument).simplify(),
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionNode<'a> {
    function: Function,
    arguments: Vec<Node<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueNode {
    Variable(String),
    Constant(f64),
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
    use crate::settings::{self, OperationCollection, Settings};

    #[test]
    #[should_panic(expected = "Expression tree does not contain y variable.")]
    fn test_subs_panics_with_wrong_variable() {
        let settings = settings::get_default_settings();
        let tree = create_test_tree_with_variables(&settings);
        tree.subs(&HashMap::from([("y", 2.0)]));
    }

    #[test]
    fn test_subs_x1_variable() {
        let settings = settings::get_default_settings();
        let tree = create_test_tree_with_variables(&settings);
        let expected_tree = ExpressionTree {
            root: Node::Operator(OperationNode {
                operation: settings.operators.find_by_name("+").unwrap(),
                arguments: vec![
                    Node::Operator(OperationNode {
                        operation: settings.operators.find_by_name("-").unwrap(),
                        arguments: vec![
                            Node::Value(ValueNode::Constant(2.0)),
                            Node::Value(ValueNode::Constant(1.0)),
                        ],
                    }),
                    Node::Operator(OperationNode {
                        operation: settings.operators.find_by_name("*").unwrap(),
                        arguments: vec![
                            Node::Function(OperationNode {
                                operation: settings.functions.find_by_name("sin").unwrap(),
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
        let settings = settings::get_default_settings();
        let actual = OperationNode {
            operation: settings.operators.find_by_name("+").unwrap(),
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
        let settings = settings::get_default_settings();
        let actual = Node::Operator(OperationNode {
            operation: settings.operators.find_by_name("+").unwrap(),
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
        let settings = settings::get_default_settings();
        let actual = Node::Function(OperationNode {
            operation: settings.functions.find_by_name("sin").unwrap(),
            arguments: vec![Node::Value(ValueNode::Constant(0.5))],
        })
        .compute()?;
        assert_eq!(0.5_f64.sin(), actual);
        Ok(())
    }

    #[test]
    fn test_tree_with_variables_compute() {
        let settings = settings::get_default_settings();
        if let Ok(_) = create_test_tree_with_variables(&settings).compute() {
            panic!("Computing a tree with a variable must return a `ComputeError`.");
        }
    }

    #[test]
    fn test_tree_without_variables_compute() -> Result<(), ComputeError> {
        let settings = settings::get_default_settings();
        let actual = create_test_tree_without_variables(&settings).compute()?;
        assert_eq!(2.0_f64 * 3.0_f64 + 5.0_f64.sin() * 10.0_f64, actual);
        Ok(())
    }

    #[test]
    fn test_tree_without_variables_simplify() {
        let settings = settings::get_default_settings();
        let mut tree = create_test_tree_without_variables(&settings);
        tree.simplify();
        let expected_tree = ExpressionTree {
            root: Node::Value(ValueNode::Constant(2.0 * 3.0 + 5.0_f64.sin() * 10.0)),
            variables: vec![],
        };
        assert_eq!(expected_tree, tree);
    }

    #[test]
    fn test_tree_to_simplify_simplify() {
        let settings = settings::get_default_settings();
        let mut tree = create_tree_to_simplify(&settings);
        tree.simplify();
        let expected_tree = ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.functions.find_by_name("sin").unwrap(),
                arguments: vec![Node::Operator(OperationNode {
                    operation: settings.operators.find_by_name("+").unwrap(),
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

    fn create_test_tree_with_variables(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Operator(OperationNode {
                operation: settings.operators.find_by_name("+").unwrap(),
                arguments: vec![
                    Node::Operator(OperationNode {
                        operation: settings.operators.find_by_name("-").unwrap(),
                        arguments: vec![
                            Node::Value(ValueNode::Variable(String::from("x1"))),
                            Node::Value(ValueNode::Constant(1.0)),
                        ],
                    }),
                    Node::Operator(OperationNode {
                        operation: settings.operators.find_by_name("*").unwrap(),
                        arguments: vec![
                            Node::Function(OperationNode {
                                operation: settings.functions.find_by_name("sin").unwrap(),
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

    fn create_test_tree_without_variables(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Operator(OperationNode {
                operation: settings.operators.find_by_name("+").unwrap(),
                arguments: vec![
                    Node::Operator(OperationNode {
                        operation: settings.operators.find_by_name("*").unwrap(),
                        arguments: vec![
                            Node::Value(ValueNode::Constant(2.0)),
                            Node::Value(ValueNode::Constant(3.0)),
                        ],
                    }),
                    Node::Operator(OperationNode {
                        operation: settings.operators.find_by_name("*").unwrap(),
                        arguments: vec![
                            Node::Function(OperationNode {
                                operation: settings.functions.find_by_name("sin").unwrap(),
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

    fn create_tree_to_simplify(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.functions.find_by_name("sin").unwrap(),
                arguments: vec![Node::Operator(OperationNode {
                    operation: settings.operators.find_by_name("+").unwrap(),
                    arguments: vec![
                        Node::Value(ValueNode::Variable(String::from("x"))),
                        Node::Operator(OperationNode {
                            operation: settings.operators.find_by_name("*").unwrap(),
                            arguments: vec![
                                Node::Value(ValueNode::Constant(2.0)),
                                Node::Operator(OperationNode {
                                    operation: settings.operators.find_by_name("+").unwrap(),
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
