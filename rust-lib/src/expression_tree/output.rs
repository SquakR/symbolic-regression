//! Module for preparing the expression tree for output.
use super::types::{ExpressionTree, Node};
use crate::model::settings::{ConverterOperation, Settings};
use std::rc::Rc;

impl ExpressionTree {
    /// Return a new ExpressionTree where internal functions and operations have been replaced with output ones.
    pub fn create_output(&self, settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: ExpressionTree::create_output_node(&self.root, settings),
            variables: self.variables.clone(),
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

#[cfg(test)]
mod tests {
    use super::super::types::{OperationNode, ValueNode};
    use super::*;
    use std::f64::consts::E;

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
}
