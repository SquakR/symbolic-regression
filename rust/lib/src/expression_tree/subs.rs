//! Module for replacing variables with values.
use super::types::{ExpressionTree, Node, OperationNode, ValueNode};
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

impl ExpressionTree {
    /// Return a new ExpressionTree where variables have been replaced with values from the `variables` HashMap.
    /// Panic if `variables` HaspMap contains non-existing variables.
    pub fn subs(&self, variables: &HashMap<&str, f64>) -> Result<ExpressionTree, SubsError> {
        for &key in variables.keys() {
            if !self.variables.iter().any(|variable| variable == key) {
                return Err(SubsError::new(key));
            }
        }
        Ok(ExpressionTree {
            root: ExpressionTree::subs_node(&self.root, variables),
            variables: self
                .variables
                .clone()
                .into_iter()
                .filter(|variable| !variables.keys().any(|key| key == variable))
                .collect(),
        })
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubsError {
    pub message: String,
}

impl SubsError {
    fn new(variable: &str) -> SubsError {
        SubsError {
            message: format!(
                r#"Expression tree does not contain "{}" variable."#,
                variable
            ),
        }
    }
}

impl fmt::Display for SubsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::settings::Settings;

    #[test]
    fn test_subs_panics_with_wrong_variable() {
        let settings = Settings::default();
        let tree = create_tree_to_subs(&settings);
        let expected_error = SubsError {
            message: String::from(r#"Expression tree does not contain "y" variable."#),
        };
        match tree.subs(&HashMap::from([("y", 2.0)])) {
            Ok(_) => panic!("Expected {:?}, but Ok(()) was received.", expected_error),
            Err(actual_error) => assert_eq!(expected_error, actual_error),
        }
    }

    #[test]
    fn test_subs_x1_variable() -> Result<(), SubsError> {
        let settings = Settings::default();
        let tree = create_tree_to_subs(&settings);
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
                    Node::Value(ValueNode::Variable(String::from("x2"))),
                ],
            }),
            variables: vec![String::from("x2")],
        };
        let actual_tree = tree.subs(&HashMap::from([("x1", 2.0)]))?;
        assert_eq!(expected_tree, actual_tree);
        Ok(())
    }

    fn create_tree_to_subs(settings: &Settings) -> ExpressionTree {
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
                    Node::Value(ValueNode::Variable(String::from("x2"))),
                ],
            }),
            variables: vec![String::from("x1"), String::from("x2")],
        }
    }
}
