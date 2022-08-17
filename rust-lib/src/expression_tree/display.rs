//! `Display` trait implementation for expression tree types.
use super::types::{ExpressionTree, Function, Node, Operation, OperationNode, Operator, ValueNode};
use std::fmt;

impl fmt::Display for ExpressionTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.root)
    }
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

impl fmt::Display for ValueNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValueNode::Variable(variable) => write!(f, "{}", variable),
            ValueNode::Constant(constant) => write!(f, "{}", constant),
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::settings::Settings;

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
    fn test_display_operator() {
        let settings = Settings::default();
        let plus_operator = settings.find_binary_operator_by_name("+").unwrap();
        assert_eq!("+", format!("{}", plus_operator));
    }

    #[test]
    fn test_display_function() {
        let settings = Settings::default();
        let sin_function = settings.find_function_by_name("sin").unwrap();
        assert_eq!("sin", format!("{}", sin_function));
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
}
