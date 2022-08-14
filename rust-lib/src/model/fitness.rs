//! Expression tree fitness module.
use super::settings::{Operation, Settings};
use crate::expression_tree::{ExpressionTree, Node, OperationNode, ValueNode};

pub struct Fitness {
    /// The sum of squared differences between actual and computed values.
    pub error: f64,
    /// The complexity of an expression tree which is the sum of the complexity of expression tree nodes.
    pub complexity: u32,
}

impl ExpressionTree {
    fn get_complexity(&self, settings: &Settings) -> u32 {
        self.root.get_complexity(settings)
    }
}

impl Node {
    fn get_complexity(&self, settings: &Settings) -> u32 {
        match self {
            Node::Operator(operator_node) => operator_node.get_complexity(settings),
            Node::Function(function_node) => function_node.get_complexity(settings),
            Node::Value(value_node) => value_node.get_complexity(settings),
        }
    }
}

impl<T: Operation> OperationNode<T> {
    fn get_complexity(&self, settings: &Settings) -> u32 {
        self.operation.get_complexity()
            + self
                .arguments
                .iter()
                .map(|argument| argument.get_complexity(settings))
                .sum::<u32>()
    }
}

impl ValueNode {
    fn get_complexity(&self, settings: &Settings) -> u32 {
        match self {
            ValueNode::Variable(_) => settings.variable_complexity,
            ValueNode::Constant(_) => settings.constant_complexity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::settings::Settings;

    #[test]
    fn test_value_node_get_complexity() {
        let settings = create_settings();
        assert_eq!(
            2,
            ValueNode::Variable(String::from("x")).get_complexity(&settings)
        );
        assert_eq!(1, ValueNode::Constant(1.0).get_complexity(&settings));
    }

    #[test]
    fn test_operation_node_get_complexity() {
        let settings = create_settings();
        let operation_node = OperationNode {
            operation: settings.find_binary_operator_by_name("+").unwrap(),
            arguments: vec![
                Node::Value(ValueNode::Variable(String::from("x"))),
                Node::Value(ValueNode::Constant(1.0)),
            ],
        };
        assert_eq!(4, operation_node.get_complexity(&settings));
    }

    #[test]
    fn test_node_get_complexity() {
        let settings = create_settings();
        let node = Node::Function(OperationNode {
            operation: settings.find_function_by_name("log").unwrap(),
            arguments: vec![
                Node::Value(ValueNode::Variable(String::from("x"))),
                Node::Value(ValueNode::Constant(1.0)),
            ],
        });
        assert_eq!(7, node.get_complexity(&settings));
    }

    #[test]
    fn test_expression_tree_get_complexity() {
        let settings = create_settings();
        let expression_tree = ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("log").unwrap(),
                arguments: vec![
                    Node::Value(ValueNode::Variable(String::from("x"))),
                    Node::Value(ValueNode::Constant(1.0)),
                ],
            }),
            variables: vec![String::from("x")],
        };
        assert_eq!(7, expression_tree.get_complexity(&settings));
    }

    fn create_settings() -> Settings {
        let mut settings = Settings::default();
        settings.variable_complexity = 2;
        settings.constant_complexity = 1;
        settings
    }
}
