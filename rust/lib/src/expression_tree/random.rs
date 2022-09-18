//! Module with random operations on the expression tree.
use super::types::{ExpressionTree, Node, Operation, OperationNode, ValueNode};
use crate::model::settings::Settings;
use rand::rngs::ThreadRng;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use std::ops::Range;
use std::rc::Rc;

pub trait Random {
    fn gen_float(&mut self) -> f64;
    fn gen_float_standard(&mut self) -> f64;
    fn gen_range(&mut self, range: Range<usize>) -> usize;
}

pub struct DefaultRandom<G: Rng, D: Distribution<f64>> {
    rng: G,
    float_distribution: D,
}

impl Default for DefaultRandom<ThreadRng, Normal<f64>> {
    fn default() -> DefaultRandom<ThreadRng, Normal<f64>> {
        DefaultRandom {
            rng: rand::thread_rng(),
            float_distribution: Normal::new(0.0, 100.0).unwrap(),
        }
    }
}

impl<G: Rng, D: Distribution<f64>> Random for DefaultRandom<G, D> {
    fn gen_float(&mut self) -> f64 {
        self.float_distribution.sample(&mut self.rng)
    }
    fn gen_float_standard(&mut self) -> f64 {
        self.rng.gen()
    }
    fn gen_range(&mut self, range: Range<usize>) -> usize {
        if range.is_empty() {
            return range.start;
        }
        self.rng.gen_range(range)
    }
}

impl ExpressionTree {
    pub fn create_random<R>(
        random: &mut R,
        settings: &Settings,
        variables: &[String],
    ) -> ExpressionTree
    where
        R: Random + ?Sized,
    {
        let CreateRandomNodeResult { node, .. } =
            Node::create_random(random, settings, variables, 0);
        ExpressionTree {
            root: node,
            variables: variables.iter().cloned().collect::<Vec<String>>(),
        }
    }
    pub fn get_random_node<R>(&self, random: &mut R) -> &Node
    where
        R: Random + ?Sized,
    {
        self.get_node_at(random.gen_range(0..self.count_nodes()))
    }
    pub fn get_random_node_mut<R>(&mut self, random: &mut R) -> &mut Node
    where
        R: Random + ?Sized,
    {
        self.get_node_at_mut(random.gen_range(0..self.count_nodes()))
    }
    pub fn get_random_value_node<R>(&self, random: &mut R) -> &Node
    where
        R: Random + ?Sized,
    {
        let value_node_indices = self.get_value_node_indices();
        self.get_node_at(value_node_indices[random.gen_range(0..value_node_indices.len())])
    }
    pub fn get_random_value_node_mut<R>(&mut self, random: &mut R) -> &mut Node
    where
        R: Random + ?Sized,
    {
        let value_node_indices = self.get_value_node_indices();
        self.get_node_at_mut(value_node_indices[random.gen_range(0..value_node_indices.len())])
    }
    pub fn find_random_operator_node<R>(&self, random: &mut R) -> Option<&Node>
    where
        R: Random + ?Sized,
    {
        let operator_node_indices = self.get_operator_node_indices();
        if operator_node_indices.len() == 0 {
            return None;
        }
        Some(
            self.get_node_at(
                operator_node_indices[random.gen_range(0..operator_node_indices.len())],
            ),
        )
    }
    pub fn find_random_operator_node_mut<R>(&mut self, random: &mut R) -> Option<&mut Node>
    where
        R: Random + ?Sized,
    {
        let operator_node_indices = self.get_operator_node_indices();
        if operator_node_indices.len() == 0 {
            return None;
        }
        Some(self.get_node_at_mut(
            operator_node_indices[random.gen_range(0..operator_node_indices.len())],
        ))
    }
    pub fn find_random_function_node<R>(&self, random: &mut R) -> Option<&Node>
    where
        R: Random + ?Sized,
    {
        let function_node_indices = self.get_function_node_indices();
        if function_node_indices.len() == 0 {
            return None;
        }
        Some(
            self.get_node_at(
                function_node_indices[random.gen_range(0..function_node_indices.len())],
            ),
        )
    }
    pub fn find_random_function_node_mut<R>(&mut self, random: &mut R) -> Option<&mut Node>
    where
        R: Random + ?Sized,
    {
        let function_node_indices = self.get_function_node_indices();
        if function_node_indices.len() == 0 {
            return None;
        }
        Some(self.get_node_at_mut(
            function_node_indices[random.gen_range(0..function_node_indices.len())],
        ))
    }
    pub fn find_random_operation_node<R>(&self, random: &mut R) -> Option<&Node>
    where
        R: Random + ?Sized,
    {
        let mut indices = self.get_operator_node_indices();
        indices.append(&mut self.get_function_node_indices());
        if indices.len() == 0 {
            return None;
        }
        Some(self.get_node_at(indices[random.gen_range(0..indices.len())]))
    }
    pub fn find_random_operation_node_mut<R>(&mut self, random: &mut R) -> Option<&mut Node>
    where
        R: Random + ?Sized,
    {
        let mut indices = self.get_operator_node_indices();
        indices.append(&mut self.get_function_node_indices());
        if indices.len() == 0 {
            return None;
        }
        Some(self.get_node_at_mut(indices[random.gen_range(0..indices.len())]))
    }
}

#[derive(Debug, PartialEq)]
pub struct CreateRandomNodeResult {
    pub node: Node,
    pub complexity: u32,
}

impl Node {
    pub fn create_random<R>(
        random: &mut R,
        settings: &Settings,
        variables: &[String],
        tree_complexity: u32,
    ) -> CreateRandomNodeResult
    where
        R: Random + ?Sized,
    {
        let node_probability = settings.get_node_probability(tree_complexity);
        let operator_node = node_probability.operator_node;
        let function_node = operator_node + node_probability.function_node;
        let float_standard = random.gen_float_standard();
        if float_standard >= 0.0 && float_standard < operator_node {
            Node::create_random_operator(random, settings, variables, tree_complexity)
        } else if float_standard >= operator_node && float_standard < function_node {
            Node::create_random_function(random, settings, variables, tree_complexity)
        } else {
            Node::create_random_value(random, settings, variables)
        }
    }
    pub fn create_random_operator<R>(
        random: &mut R,
        settings: &Settings,
        variables: &[String],
        tree_complexity: u32,
    ) -> CreateRandomNodeResult
    where
        R: Random + ?Sized,
    {
        let operator =
            Rc::clone(&settings.operators[random.gen_range(0..settings.operators.len())]);
        let mut node_complexity = operator.get_complexity();
        let mut tree_complexity = tree_complexity + node_complexity;
        let arguments = (0..operator.arguments_number)
            .map(|_| {
                let CreateRandomNodeResult { node, complexity } =
                    Node::create_random(random, settings, variables, tree_complexity);
                node_complexity += complexity;
                tree_complexity += complexity;
                node
            })
            .collect::<Vec<Node>>();
        CreateRandomNodeResult {
            node: Node::Operator(OperationNode {
                operation: operator,
                arguments,
            }),
            complexity: node_complexity,
        }
    }
    pub fn create_random_function<R>(
        random: &mut R,
        settings: &Settings,
        variables: &[String],
        tree_complexity: u32,
    ) -> CreateRandomNodeResult
    where
        R: Random + ?Sized,
    {
        let function =
            Rc::clone(&settings.functions[random.gen_range(0..settings.functions.len())]);
        let mut node_complexity = function.get_complexity();
        let mut tree_complexity = tree_complexity + node_complexity;
        let arguments = (0..function.arguments_number)
            .map(|_| {
                let CreateRandomNodeResult { node, complexity } =
                    Node::create_random(random, settings, variables, tree_complexity);
                node_complexity += complexity;
                tree_complexity += complexity;
                node
            })
            .collect::<Vec<Node>>();
        CreateRandomNodeResult {
            node: Node::Function(OperationNode {
                operation: function,
                arguments,
            }),
            complexity: node_complexity,
        }
    }
    pub fn create_random_value<R>(
        random: &mut R,
        settings: &Settings,
        variables: &[String],
    ) -> CreateRandomNodeResult
    where
        R: Random + ?Sized,
    {
        if random.gen_float_standard() < 0.5 {
            CreateRandomNodeResult {
                node: Node::create_random_variable(random, variables),
                complexity: settings.variable_complexity,
            }
        } else {
            CreateRandomNodeResult {
                node: Node::create_random_constant(random),
                complexity: settings.constant_complexity,
            }
        }
    }
    pub fn create_random_variable<R>(random: &mut R, variables: &[String]) -> Node
    where
        R: Random + ?Sized,
    {
        Node::Value(ValueNode::Variable(
            variables[random.gen_range(0..variables.len())].to_owned(),
        ))
    }
    pub fn create_random_constant<R>(random: &mut R) -> Node
    where
        R: Random + ?Sized,
    {
        Node::Value(ValueNode::Constant(random.gen_float()))
    }
}

pub struct MockRandom {
    pub int: Option<Box<dyn Iterator<Item = usize>>>,
    pub float: Option<Box<dyn Iterator<Item = f64>>>,
    pub float_standard: Option<Box<dyn Iterator<Item = f64>>>,
}

impl MockRandom {
    pub fn new(int: Vec<usize>, float: Vec<f64>, float_standard: Vec<f64>) -> MockRandom {
        MockRandom {
            int: Some(Box::new(int.into_iter().cycle())),
            float: Some(Box::new(float.into_iter().cycle())),
            float_standard: Some(Box::new(float_standard.into_iter().cycle())),
        }
    }
    pub fn new_int(int: Vec<usize>) -> MockRandom {
        MockRandom {
            int: Some(Box::new(int.into_iter().cycle())),
            float: None,
            float_standard: None,
        }
    }
}

impl Default for MockRandom {
    fn default() -> MockRandom {
        MockRandom::new_int(vec![])
    }
}

impl Random for MockRandom {
    fn gen_float(&mut self) -> f64 {
        match &mut self.float {
            Some(float) => float.next().unwrap(),
            None => unreachable!(),
        }
    }
    fn gen_float_standard(&mut self) -> f64 {
        match &mut self.float_standard {
            Some(float_standard) => float_standard.next().unwrap(),
            None => unreachable!(),
        }
    }
    fn gen_range(&mut self, _: Range<usize>) -> usize {
        match &mut self.int {
            Some(int) => int.next().unwrap(),
            None => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::{OperationNode, ValueNode};
    use super::*;
    use crate::model::settings::Settings;

    #[test]
    fn test_get_random_node() {
        let settings = Settings::default();
        let expression_tree = create_expression_tree(&settings);
        let expected_node = Node::Value(ValueNode::Constant(2.0));
        let actual_node = expression_tree.get_random_node(&mut MockRandom::new_int(vec![3]));
        assert_eq!(&expected_node, actual_node);
    }

    #[test]
    fn test_get_random_node_mut() {
        let settings = Settings::default();
        let mut expression_tree = create_expression_tree(&settings);
        let mut expected_node = Node::Value(ValueNode::Variable(String::from("x1")));
        let actual_node = expression_tree.get_random_node_mut(&mut MockRandom::new_int(vec![6]));
        assert_eq!(&mut expected_node, actual_node);
    }
    #[test]
    fn test_get_random_value_node() {
        let settings = Settings::default();
        let expression_tree = create_expression_tree(&settings);
        let expected_node = Node::Value(ValueNode::Constant(2.0));
        let actual_node = expression_tree.get_random_value_node(&mut MockRandom::new_int(vec![0]));
        assert_eq!(&expected_node, actual_node);
    }

    #[test]
    fn test_get_random_value_node_mut() {
        let settings = Settings::default();
        let mut expression_tree = create_expression_tree(&settings);
        let mut expected_node = Node::Value(ValueNode::Variable(String::from("x2")));
        let actual_node =
            expression_tree.get_random_value_node_mut(&mut MockRandom::new_int(vec![2]));
        assert_eq!(&mut expected_node, actual_node);
    }

    #[test]
    fn test_find_random_operator_node() {
        let settings = Settings::default();
        let expression_tree = create_expression_tree(&settings);
        let expected_node = Node::Operator(OperationNode {
            operation: settings.find_unary_operator_by_name("-").unwrap(),
            arguments: vec![Node::Value(ValueNode::Constant(2.0))],
        });
        let actual_node =
            expression_tree.find_random_operator_node(&mut MockRandom::new_int(vec![1]));
        assert_eq!(Some(&expected_node), actual_node);
    }

    #[test]
    fn test_find_random_operator_node_none() {
        let expression_tree = create_value_expression_tree();
        let actual_node =
            expression_tree.find_random_operator_node(&mut MockRandom::new_int(vec![0]));
        assert_eq!(None, actual_node);
    }

    #[test]
    fn test_find_random_operator_node_mut() {
        let settings = Settings::default();
        let mut expression_tree = create_expression_tree(&settings);
        let mut expected_node = create_plus_node(&settings);
        let actual_node =
            expression_tree.find_random_operator_node_mut(&mut MockRandom::new_int(vec![2]));
        assert_eq!(Some(&mut expected_node), actual_node);
    }

    #[test]
    fn test_find_random_operator_node_mut_none() {
        let expression_tree = create_value_expression_tree();
        let actual_node =
            expression_tree.find_random_operator_node(&mut MockRandom::new_int(vec![0]));
        assert_eq!(None, actual_node);
    }

    #[test]
    fn test_find_random_function_node() {
        let settings = Settings::default();
        let expression_tree = create_expression_tree(&settings);
        let expected_node = &expression_tree.root;
        let actual_node =
            expression_tree.find_random_function_node(&mut MockRandom::new_int(vec![0]));
        assert_eq!(Some(expected_node), actual_node);
    }

    #[test]
    fn test_find_random_function_node_none() {
        let expression_tree = create_value_expression_tree();
        let actual_node =
            expression_tree.find_random_function_node(&mut MockRandom::new_int(vec![0]));
        assert_eq!(None, actual_node);
    }

    #[test]
    fn test_find_random_function_node_mut() {
        let settings = Settings::default();
        let mut expression_tree = create_expression_tree(&settings);
        let mut expected_node = create_cos_node(&settings);
        let actual_node =
            expression_tree.find_random_function_node_mut(&mut MockRandom::new_int(vec![1]));
        assert_eq!(Some(&mut expected_node), actual_node);
    }

    #[test]
    fn test_find_random_function_node_mut_none() {
        let mut expression_tree = create_value_expression_tree();
        let actual_node =
            expression_tree.find_random_function_node_mut(&mut MockRandom::new_int(vec![0]));
        assert_eq!(None, actual_node);
    }

    #[test]
    fn test_find_random_operation_node() {
        let settings = Settings::default();
        let expression_tree = create_expression_tree(&settings);
        let expected_node = create_plus_node(&settings);
        let actual_node =
            expression_tree.find_random_operation_node(&mut MockRandom::new_int(vec![2]));
        assert_eq!(Some(&expected_node), actual_node);
    }

    #[test]
    fn test_find_random_operation_node_none() {
        let expression_tree = create_value_expression_tree();
        let actual_node =
            expression_tree.find_random_operation_node(&mut MockRandom::new_int(vec![0]));
        assert_eq!(None, actual_node);
    }

    #[test]
    fn test_find_random_operation_node_mut() {
        let settings = Settings::default();
        let mut expression_tree = create_expression_tree(&settings);
        let mut expected_node = create_cos_node(&settings);
        let actual_node =
            expression_tree.find_random_operation_node_mut(&mut MockRandom::new_int(vec![4]));
        assert_eq!(Some(&mut expected_node), actual_node);
    }

    #[test]
    fn test_find_random_operation_node_mut_none() {
        let mut expression_tree = create_value_expression_tree();
        let actual_node =
            expression_tree.find_random_operation_node_mut(&mut MockRandom::new_int(vec![0]));
        assert_eq!(None, actual_node);
    }

    #[test]
    fn test_create_random_variable_node() {
        let settings = Settings::default();
        let expected_result = CreateRandomNodeResult {
            node: Node::Value(ValueNode::Variable(String::from("x2"))),
            complexity: 1,
        };
        let actual_result = Node::create_random_value(
            &mut MockRandom::new(vec![1], vec![], vec![0.25]),
            &settings,
            &vec![String::from("x1"), String::from("x2"), String::from("x3")],
        );
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn test_create_random_constant_node() {
        let settings = Settings::default();
        let expected_result = CreateRandomNodeResult {
            node: Node::Value(ValueNode::Constant(2.0)),
            complexity: 1,
        };
        let actual_result = Node::create_random_value(
            &mut MockRandom::new(vec![], vec![2.0], vec![0.75]),
            &settings,
            &vec![String::from("x")],
        );
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn test_create_random_operator_node() {
        let settings = Settings::default();
        let expected_result = CreateRandomNodeResult {
            node: Node::Operator(OperationNode {
                operation: settings.find_binary_operator_by_name("-").unwrap(),
                arguments: vec![
                    Node::Value(ValueNode::Variable(String::from("x3"))),
                    Node::Value(ValueNode::Constant(5.0)),
                ],
            }),
            complexity: 3,
        };
        let actual_result = Node::create_random_operator(
            &mut MockRandom::new(vec![1, 2], vec![5.0], vec![0.9, 0.4, 0.9, 0.8]),
            &settings,
            &vec![String::from("x1"), String::from("x2"), String::from("x3")],
            0,
        );
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn test_create_random_function_node() {
        let settings = Settings::default();
        let expected_result = CreateRandomNodeResult {
            node: Node::Function(OperationNode {
                operation: settings.find_function_by_name("cos").unwrap(),
                arguments: vec![Node::Value(ValueNode::Constant(5.0))],
            }),
            complexity: 5,
        };
        let actual_result = Node::create_random_function(
            &mut MockRandom::new(vec![4], vec![5.0], vec![0.65, 0.6]),
            &settings,
            &vec![String::from("x")],
            0,
        );
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn test_create_random_expression_tree() {
        let settings = Settings::default();
        let expected_expression_tree = create_expression_tree(&settings);
        let actual_expression_tree = ExpressionTree::create_random(
            &mut MockRandom::new(
                vec![2, 2, 6, 4, 0, 0, 1],
                vec![2.0],
                vec![
                    0.5, 0.3, 0.25, 0.51, 0.6, 0.24, 0.18, 0.64, 0.49, 0.66, 0.41,
                ],
            ),
            &settings,
            &vec![String::from("x1"), String::from("x2")],
        );
        assert_eq!(expected_expression_tree, actual_expression_tree);
    }

    fn create_expression_tree(settings: &Settings) -> ExpressionTree {
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
                        create_cos_node(settings),
                    ],
                })],
            }),
            variables: vec![String::from("x1"), String::from("x2")],
        }
    }

    fn create_value_expression_tree() -> ExpressionTree {
        ExpressionTree {
            root: Node::Value(ValueNode::Variable(String::from("x1"))),
            variables: vec![String::from("x1"), String::from("x2")],
        }
    }

    fn create_plus_node(settings: &Settings) -> Node {
        Node::Operator(OperationNode {
            operation: settings.find_binary_operator_by_name("+").unwrap(),
            arguments: vec![
                Node::Value(ValueNode::Variable(String::from("x1"))),
                Node::Value(ValueNode::Variable(String::from("x2"))),
            ],
        })
    }

    fn create_cos_node(settings: &Settings) -> Node {
        Node::Function(OperationNode {
            operation: settings.find_function_by_name("cos").unwrap(),
            arguments: vec![create_plus_node(settings)],
        })
    }
}
