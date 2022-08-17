//! Module with random operations on the expression tree.
use super::types::{ExpressionTree, Node};
use rand::distributions::uniform::SampleRange;
use rand::Rng;

pub trait Random {
    fn gen_range<R>(&mut self, range: R) -> usize
    where
        R: SampleRange<usize>;
}

pub struct DefaultRandom<G: Rng>(G);

impl<G: Rng> Random for DefaultRandom<G> {
    fn gen_range<R>(&mut self, range: R) -> usize
    where
        R: SampleRange<usize>,
    {
        self.0.gen_range(range)
    }
}

impl ExpressionTree {
    pub fn get_random_node<R>(&self, rng: &mut R) -> &Node
    where
        R: Random,
    {
        self.get_node_at(rng.gen_range(0..self.count_nodes()))
    }
    pub fn get_random_node_mut<R>(&mut self, rng: &mut R) -> &mut Node
    where
        R: Random,
    {
        self.get_node_at_mut(rng.gen_range(0..self.count_nodes()))
    }
    pub fn get_random_operator_node<R>(&self, rng: &mut R) -> &Node
    where
        R: Random,
    {
        let operator_node_indices = self.get_operator_node_indices();
        self.get_node_at(operator_node_indices[rng.gen_range(0..operator_node_indices.len())])
    }
    pub fn get_random_operator_node_mut<R>(&mut self, rng: &mut R) -> &mut Node
    where
        R: Random,
    {
        let operator_node_indices = self.get_operator_node_indices();
        self.get_node_at_mut(operator_node_indices[rng.gen_range(0..operator_node_indices.len())])
    }
    pub fn get_random_function_node<R>(&self, rng: &mut R) -> &Node
    where
        R: Random,
    {
        let function_node_indices = self.get_function_node_indices();
        self.get_node_at(function_node_indices[rng.gen_range(0..function_node_indices.len())])
    }
    pub fn get_random_function_node_mut<R>(&mut self, rng: &mut R) -> &mut Node
    where
        R: Random,
    {
        let function_node_indices = self.get_function_node_indices();
        self.get_node_at_mut(function_node_indices[rng.gen_range(0..function_node_indices.len())])
    }
    pub fn get_random_value_node<R>(&self, rng: &mut R) -> &Node
    where
        R: Random,
    {
        let value_node_indices = self.get_value_node_indices();
        self.get_node_at(value_node_indices[rng.gen_range(0..value_node_indices.len())])
    }
    pub fn get_random_value_node_mut<R>(&mut self, rng: &mut R) -> &mut Node
    where
        R: Random,
    {
        let value_node_indices = self.get_value_node_indices();
        self.get_node_at_mut(value_node_indices[rng.gen_range(0..value_node_indices.len())])
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{OperationNode, ValueNode};
    use super::*;
    use crate::model::settings::Settings;
    use rand::rngs::mock::StepRng;

    struct MockRandom(StepRng);

    impl Random for MockRandom {
        fn gen_range<R>(&mut self, _: R) -> usize
        where
            R: SampleRange<usize>,
        {
            self.0.gen()
        }
    }

    #[test]
    fn test_get_random_node() {
        let settings = Settings::default();
        let expression_tree = create_test_expression_tree(&settings);
        let expected_node = Node::Value(ValueNode::Constant(2.0));
        let actual_node = expression_tree.get_random_node(&mut MockRandom(StepRng::new(3, 1)));
        assert_eq!(&expected_node, actual_node);
    }

    #[test]
    fn test_get_random_node_mut() {
        let settings = Settings::default();
        let mut expression_tree = create_test_expression_tree(&settings);
        let mut expected_node = Node::Value(ValueNode::Variable(String::from("x1")));
        let actual_node = expression_tree.get_random_node_mut(&mut MockRandom(StepRng::new(6, 1)));
        assert_eq!(&mut expected_node, actual_node);
    }

    #[test]
    fn test_get_random_operator_node() {
        let settings = Settings::default();
        let expression_tree = create_test_expression_tree(&settings);
        let expected_node = Node::Operator(OperationNode {
            operation: settings.find_unary_operator_by_name("-").unwrap(),
            arguments: vec![Node::Value(ValueNode::Constant(2.0))],
        });
        let actual_node =
            expression_tree.get_random_operator_node(&mut MockRandom(StepRng::new(1, 1)));
        assert_eq!(&expected_node, actual_node);
    }

    #[test]
    fn test_get_random_operator_node_mut() {
        let settings = Settings::default();
        let mut expression_tree = create_test_expression_tree(&settings);
        let mut expected_node = Node::Operator(OperationNode {
            operation: settings.find_binary_operator_by_name("+").unwrap(),
            arguments: vec![
                Node::Value(ValueNode::Variable(String::from("x1"))),
                Node::Value(ValueNode::Variable(String::from("x2"))),
            ],
        });
        let actual_node =
            expression_tree.get_random_operator_node_mut(&mut MockRandom(StepRng::new(2, 1)));
        assert_eq!(&mut expected_node, actual_node);
    }

    #[test]
    fn test_get_random_function_node() {
        let settings = Settings::default();
        let expression_tree = create_test_expression_tree(&settings);
        let expected_node = &expression_tree.root;
        let actual_node =
            expression_tree.get_random_function_node(&mut MockRandom(StepRng::new(0, 1)));
        assert_eq!(expected_node, actual_node);
    }

    #[test]
    fn test_get_random_function_node_mut() {
        let settings = Settings::default();
        let mut expression_tree = create_test_expression_tree(&settings);
        let mut expected_node = Node::Function(OperationNode {
            operation: settings.find_function_by_name("cos").unwrap(),
            arguments: vec![Node::Operator(OperationNode {
                operation: settings.find_binary_operator_by_name("+").unwrap(),
                arguments: vec![
                    Node::Value(ValueNode::Variable(String::from("x1"))),
                    Node::Value(ValueNode::Variable(String::from("x2"))),
                ],
            })],
        });
        let actual_node =
            expression_tree.get_random_function_node_mut(&mut MockRandom(StepRng::new(1, 1)));
        assert_eq!(&mut expected_node, actual_node);
    }

    #[test]
    fn test_get_random_value_node() {
        let settings = Settings::default();
        let expression_tree = create_test_expression_tree(&settings);
        let expected_node = Node::Value(ValueNode::Constant(2.0));
        let actual_node =
            expression_tree.get_random_value_node(&mut MockRandom(StepRng::new(0, 1)));
        assert_eq!(&expected_node, actual_node);
    }

    #[test]
    fn test_get_random_value_node_mut() {
        let settings = Settings::default();
        let mut expression_tree = create_test_expression_tree(&settings);
        let mut expected_node = Node::Value(ValueNode::Variable(String::from("x2")));
        let actual_node =
            expression_tree.get_random_value_node_mut(&mut MockRandom(StepRng::new(2, 1)));
        assert_eq!(&mut expected_node, actual_node);
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
