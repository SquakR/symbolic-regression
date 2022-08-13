//! Expression tree types serializers module.
use super::types::{Node, OperationNode, ValueNode};
use crate::model::settings::Operation;
use serde::ser::{Serialize, SerializeMap, Serializer};

impl Serialize for ValueNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ValueNode::Variable(variable) => serializer.serialize_str(variable),
            ValueNode::Constant(constant) => serializer.serialize_f64(*constant),
        }
    }
}

impl<T: Operation> Serialize for OperationNode<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut operation_node = serializer.serialize_map(Some(1))?;
        operation_node.serialize_entry(self.operation.get_name(), &self.arguments)?;
        operation_node.end()
    }
}

impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Node::Value(value_node) => value_node.serialize(serializer),
            Node::Operator(operator_node) => operator_node.serialize(serializer),
            Node::Function(function_node) => function_node.serialize(serializer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::ExpressionTree;
    use super::*;
    use crate::model::settings::Settings;
    use serde_json::{self, Error};

    #[test]
    fn test_serialize_expression_tree_to_json() -> Result<(), Error> {
        let settings = Settings::default();
        let tree = ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("log").unwrap(),
                arguments: vec![
                    Node::Value(ValueNode::Constant(10.0)),
                    Node::Operator(OperationNode {
                        operation: settings.find_binary_operator_by_name("+").unwrap(),
                        arguments: vec![
                            Node::Value(ValueNode::Variable(String::from("x"))),
                            Node::Value(ValueNode::Constant(2.0)),
                        ],
                    }),
                ],
            }),
            variables: vec![String::from("x")],
        };
        let expected_json = "{\"root\":{\"log\":[10.0,{\"+\":[\"x\",2.0]}]},\"variables\":[\"x\"]}";
        let actual_json = serde_json::to_string(&tree)?;
        assert_eq!(expected_json, actual_json);
        Ok(())
    }
}
