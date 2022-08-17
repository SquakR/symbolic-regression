//! Module with expression tree traversal algorithms.
use super::types::{ExpressionTree, Node};

impl ExpressionTree {
    /// Pre-order expression tree traversal algorithm.
    pub fn walk_pre_order<'a, C>(&'a self, callback: &mut C)
    where
        C: FnMut(&'a Node) -> (),
    {
        self.root.walk_pre_order(callback);
    }
    /// Return a reference to the node that satisfies the predicate, or None if no such node exists.
    pub fn get_node<C>(&self, predicate: &mut C) -> Option<&Node>
    where
        C: FnMut(&Node) -> bool,
    {
        self.root.get_node(predicate)
    }
    /// Return a mutable reference to the node that satisfies the predicate, or None if no such node exists.
    pub fn get_node_mut<C>(&mut self, predicate: &mut C) -> Option<&mut Node>
    where
        C: FnMut(&Node) -> bool,
    {
        self.root.get_node_mut(predicate)
    }
    /// Return number of nodes.
    pub fn count_nodes(&self) -> usize {
        let mut counter = 0;
        self.walk_pre_order(&mut |_| {
            counter += 1;
        });
        counter
    }
    /// Return operator node indices according to pre-order traversal algorithm.
    pub fn get_operator_node_indices(&self) -> Vec<usize> {
        let mut indices = vec![];
        let mut counter = 0;
        self.walk_pre_order(&mut |node| {
            if let Node::Operator(_) = node {
                indices.push(counter);
            }
            counter += 1;
        });
        indices
    }
    /// Return function node indices according to pre-order traversal algorithm.
    pub fn get_function_node_indices(&self) -> Vec<usize> {
        let mut indices = vec![];
        let mut counter = 0;
        self.walk_pre_order(&mut |node| {
            if let Node::Function(_) = node {
                indices.push(counter);
            }
            counter += 1;
        });
        indices
    }
    /// Return value node indices according to pre-order traversal algorithm.
    pub fn get_value_node_indices(&self) -> Vec<usize> {
        let mut indices = vec![];
        let mut counter = 0;
        self.walk_pre_order(&mut |node| {
            if let Node::Value(_) = node {
                indices.push(counter);
            }
            counter += 1;
        });
        indices
    }
    /// Return a reference to the node at index according to pre-order traversal algorithm.
    pub fn get_node_at(&self, index: usize) -> &Node {
        let mut counter = 0;
        self.get_node(&mut |_| {
            if counter == index {
                return true;
            }
            counter += 1;
            false
        })
        .unwrap()
    }
    /// Return a mutable reference to the node at index according to pre-order traversal algorithm.
    pub fn get_node_at_mut(&mut self, index: usize) -> &mut Node {
        let mut counter = 0;
        self.get_node_mut(&mut |_| {
            if counter == index {
                return true;
            }
            counter += 1;
            false
        })
        .unwrap()
    }
}

impl Node {
    /// Pre-order traversal algorithm.
    pub fn walk_pre_order<'a, C>(&'a self, callback: &mut C)
    where
        C: FnMut(&'a Self) -> (),
    {
        callback(self);
        if let Node::Operator(operator_node) = self {
            for argument in &operator_node.arguments {
                argument.walk_pre_order(callback);
            }
        }
        if let Node::Function(function_node) = self {
            for argument in &function_node.arguments {
                argument.walk_pre_order(callback);
            }
        }
    }
    /// Return a reference to the node that satisfies the predicate, or None if no such node exists.
    pub fn get_node<C>(&self, predicate: &mut C) -> Option<&Node>
    where
        C: FnMut(&Self) -> bool,
    {
        if predicate(self) {
            return Some(self);
        }
        match self {
            Node::Operator(operator_node) => {
                for argument in &operator_node.arguments {
                    if let Some(node) = argument.get_node(predicate) {
                        return Some(node);
                    }
                }
                None
            }
            Node::Function(function_node) => {
                for argument in &function_node.arguments {
                    if let Some(node) = argument.get_node(predicate) {
                        return Some(node);
                    }
                }
                None
            }
            Node::Value(_) => None,
        }
    }
    /// Return a mutable reference to the node that satisfies the predicate, or None if no such node exists.
    pub fn get_node_mut<C>(&mut self, predicate: &mut C) -> Option<&mut Node>
    where
        C: FnMut(&Self) -> bool,
    {
        if predicate(self) {
            return Some(self);
        }
        match self {
            Node::Operator(operator_node) => {
                for argument in &mut operator_node.arguments {
                    if let Some(node) = argument.get_node_mut(predicate) {
                        return Some(node);
                    }
                }
                None
            }
            Node::Function(function_node) => {
                for argument in &mut function_node.arguments {
                    if let Some(node) = argument.get_node_mut(predicate) {
                        return Some(node);
                    }
                }
                None
            }
            Node::Value(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{Operation, OperationNode, ValueNode};
    use super::*;
    use crate::model::settings::Settings;

    #[test]
    fn test_walk_pre_order() {
        let settings = Settings::default();
        let expression_tree = create_test_expression_tree(&settings);
        let mut test_string = String::new();
        expression_tree.walk_pre_order(&mut |node| match node {
            Node::Operator(operation_node) => {
                test_string.push_str(operation_node.operation.get_name())
            }
            Node::Function(function_node) => {
                test_string.push_str(function_node.operation.get_name())
            }
            Node::Value(value_node) => test_string.push_str(value_node.to_string().as_str()),
        });
        assert_eq!("sin*-2cos+x1x2", test_string);
    }

    #[test]
    fn test_get_node() {
        let settings = Settings::default();
        let expression_tree = create_test_expression_tree(&settings);
        let node = expression_tree.get_node(&mut |node| match node {
            Node::Value(ValueNode::Constant(_)) => true,
            _ => false,
        });
        assert_eq!(Some(&Node::Value(ValueNode::Constant(2.0))), node);
    }

    #[test]
    fn test_get_node_mut() {
        let settings = Settings::default();
        let mut expression_tree = create_test_expression_tree(&settings);
        let node = expression_tree.get_node_mut(&mut |node| match node {
            Node::Value(ValueNode::Variable(_)) => true,
            _ => false,
        });
        assert_eq!(
            Some(&mut Node::Value(ValueNode::Variable(String::from("x1")))),
            node
        );
    }

    #[test]
    fn test_count_nodes() {
        let settings = Settings::default();
        let expression_tree = create_test_expression_tree(&settings);
        let count = expression_tree.count_nodes();
        assert_eq!(8, count);
    }

    #[test]
    fn test_get_operator_node_indices() {
        let settings = Settings::default();
        let expression_tree = create_test_expression_tree(&settings);
        let indices = expression_tree.get_operator_node_indices();
        assert_eq!(vec![1, 2, 5], indices);
    }

    #[test]
    fn test_get_function_node_indices() {
        let settings = Settings::default();
        let expression_tree = create_test_expression_tree(&settings);
        let indices = expression_tree.get_function_node_indices();
        assert_eq!(vec![0, 4], indices);
    }

    #[test]
    fn test_get_value_node_indices() {
        let settings = Settings::default();
        let expression_tree = create_test_expression_tree(&settings);
        let indices = expression_tree.get_value_node_indices();
        assert_eq!(vec![3, 6, 7], indices);
    }

    #[test]
    fn test_get_node_at() {
        let settings = Settings::default();
        let expression_tree = create_test_expression_tree(&settings);
        let node = expression_tree.get_node_at(3);
        assert_eq!(&Node::Value(ValueNode::Constant(2.0)), node);
    }

    #[test]
    fn test_get_node_mut_at() {
        let settings = Settings::default();
        let mut expression_tree = create_test_expression_tree(&settings);
        let node = expression_tree.get_node_at_mut(6);
        assert_eq!(
            &mut Node::Value(ValueNode::Variable(String::from("x1"))),
            node
        );
    }

    fn create_test_expression_tree(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("sin").unwrap(),
                arguments: vec![Node::Operator(OperationNode {
                    operation: settings.find_binary_operator_by_name("*").unwrap(),
                    arguments: vec![
                        Node::Operator(OperationNode {
                            operation: settings.find_unary_operator_by_name("-").unwrap(),
                            arguments: vec![Node::Value(ValueNode::Constant(2.0))],
                        }),
                        Node::Function(OperationNode {
                            operation: settings.find_function_by_name("cos").unwrap(),
                            arguments: vec![Node::Operator(OperationNode {
                                operation: settings.find_binary_operator_by_name("+").unwrap(),
                                arguments: vec![
                                    Node::Value(ValueNode::Variable(String::from("x1"))),
                                    Node::Value(ValueNode::Variable(String::from("x2"))),
                                ],
                            })],
                        }),
                    ],
                })],
            }),
            variables: vec![String::from("x1"), String::from("x2")],
        }
    }
}
