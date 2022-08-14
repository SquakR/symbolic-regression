//! Module for computing an expression tree.
use super::types::{ExpressionTree, Node, OperationNode, ValueNode};
use crate::model::settings::Operation;

pub trait Computable {
    /// Compute a node by computing all child nodes and performing a node operation.
    /// Return `ComputeError` if some node contains a variable instead of a constant.
    /// For computation, the idea of ​​the composite pattern is used.
    fn compute(&self) -> Result<f64, ComputeError>;
    /// Simplify a node by replacing all child nodes that can be computed with values.
    /// For example, sin(x + 2,0 * (3.0 + 2.0)) -> sin(x + 14.0)
    fn simplify(&mut self) -> ();
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputeError {
    pub message: String,
}

impl ComputeError {
    fn new(variable: &str) -> ComputeError {
        ComputeError {
            message: format!(r#"The "{}" variable is not a constant."#, variable),
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
    use crate::model::settings::Settings;

    #[test]
    fn test_value_variable_compute() {
        let expected_error = ComputeError {
            message: String::from(r#"The "x" variable is not a constant."#),
        };
        match ValueNode::Variable(String::from("x")).compute() {
            Ok(_) => panic!("Expected {:?}, but Ok(()) was received.", expected_error),
            Err(actual_error) => assert_eq!(expected_error, actual_error),
        };
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
