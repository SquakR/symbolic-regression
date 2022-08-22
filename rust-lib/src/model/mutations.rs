//! Module with expression tree mutations.
use super::settings::Settings;
use crate::expression_tree::{ExpressionTree, Node, OperationNode, Random};
use std::cmp::max;
use std::rc::Rc;

pub fn replace_subtree_mutation<R>(
    expression_tree: &mut ExpressionTree,
    random: &mut R,
    settings: &Settings,
) where
    R: Random + ?Sized,
{
    let variables = expression_tree.variables.clone();
    let tree_complexity = expression_tree.get_complexity(settings);
    let complexity_random = 10 - random.gen_range(0..21) as i32;
    let node = expression_tree.get_random_node_mut(random);
    let complexity = max(
        tree_complexity as i32 - node.get_complexity(settings) as i32 + complexity_random,
        0,
    ) as u32;
    *node = Node::create_random(random, settings, &variables, complexity).node;
}

pub fn replace_leaf_mutation<R>(
    expression_tree: &mut ExpressionTree,
    random: &mut R,
    settings: &Settings,
) where
    R: Random + ?Sized,
{
    *expression_tree.get_random_value_node_mut(random) =
        Node::create_random_value(random, settings, &expression_tree.variables).node;
}

pub fn replace_operation_mutation<R>(
    expression_tree: &mut ExpressionTree,
    random: &mut R,
    settings: &Settings,
) where
    R: Random + ?Sized,
{
    let variables = expression_tree.variables.clone();
    let tree_complexity = expression_tree.get_complexity(settings);
    let complexity_random = 10 - random.gen_range(0..21) as i32;
    let mut complexity = max(tree_complexity as i32 + complexity_random, 0) as u32;
    let node = expression_tree.get_random_operation_node_mut(random);
    let (mut arguments, operation_complexity) = match node {
        Node::Operator(operator_node) => (
            operator_node.take_arguments(),
            operator_node.operation.complexity,
        ),
        Node::Function(function_node) => (
            function_node.take_arguments(),
            function_node.operation.complexity,
        ),
        _ => unreachable!(),
    };
    complexity -= operation_complexity;
    let index = random.gen_range(0..settings.operators.len() + settings.functions.len());
    *node = if index < settings.operators.len() {
        let operator = Rc::clone(&settings.operators[index]);
        prepare_arguments(
            random,
            settings,
            &variables,
            &mut arguments,
            operator.arguments_number,
            complexity + operator.complexity,
        );
        Node::Operator(OperationNode {
            operation: operator,
            arguments,
        })
    } else {
        let function = Rc::clone(&settings.functions[index - settings.operators.len()]);
        prepare_arguments(
            random,
            settings,
            &variables,
            &mut arguments,
            function.arguments_number,
            complexity + function.complexity,
        );
        Node::Function(OperationNode {
            operation: function,
            arguments,
        })
    }
}

pub fn remove_operation_mutation<R>(
    expression_tree: &mut ExpressionTree,
    random: &mut R,
    _: &Settings,
) where
    R: Random + ?Sized,
{
    let node = expression_tree.get_random_operation_node_mut(random);
    let mut arguments = match node {
        Node::Operator(operator_node) => operator_node.take_arguments(),
        Node::Function(function_node) => function_node.take_arguments(),
        _ => unreachable!(),
    };
    *node = arguments.remove(random.gen_range(0..arguments.len()));
}

fn prepare_arguments<R>(
    random: &mut R,
    settings: &Settings,
    variables: &[String],
    arguments: &mut Vec<Node>,
    arguments_number: usize,
    complexity: u32,
) where
    R: Random + ?Sized,
{
    while arguments.len() < arguments_number {
        arguments.push(Node::create_random(random, settings, &variables, complexity).node)
    }
    while arguments.len() > arguments_number {
        arguments.remove(random.gen_range(0..arguments.len()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression_tree::{MockRandom, ValueNode};

    #[test]
    fn test_replace_subtree_mutation() {
        let settings = Settings::default();
        let mut expression_tree = create_test_expression_tree(&settings);
        replace_subtree_mutation(
            &mut expression_tree,
            &mut MockRandom::new(vec![10, 1, 0], vec![], vec![0.62, 0.45]),
            &settings,
        );
        let expected_expression_tree = ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("sin").unwrap(),
                arguments: vec![Node::Value(ValueNode::Variable(String::from("x")))],
            }),
            variables: vec![String::from("x")],
        };
        assert_eq!(expected_expression_tree, expression_tree);
    }

    #[test]
    fn test_replace_leaf_mutation() {
        let settings = Settings::default();
        let mut expression_tree = create_test_expression_tree(&settings);
        replace_leaf_mutation(
            &mut expression_tree,
            &mut MockRandom::new(vec![0], vec![], vec![0.45]),
            &settings,
        );
        let expected_expression_tree = ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("sin").unwrap(),
                arguments: vec![Node::Function(OperationNode {
                    operation: settings.find_function_by_name("log").unwrap(),
                    arguments: vec![
                        Node::Value(ValueNode::Variable(String::from("x"))),
                        Node::Operator(OperationNode {
                            operation: settings.find_binary_operator_by_name("+").unwrap(),
                            arguments: vec![
                                Node::Value(ValueNode::Variable(String::from("x"))),
                                Node::Value(ValueNode::Constant(10.0)),
                            ],
                        }),
                    ],
                })],
            }),
            variables: vec![String::from("x")],
        };
        assert_eq!(expected_expression_tree, expression_tree);
    }

    #[test]
    fn test_replace_operation_mutation_operator_to_function() {
        let settings = Settings::default();
        let mut expression_tree = create_test_expression_tree(&settings);
        replace_operation_mutation(
            &mut expression_tree,
            &mut MockRandom::new_int(vec![10, 0, 11, 0]),
            &settings,
        );
        let expected_expression_tree = ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("sin").unwrap(),
                arguments: vec![Node::Function(OperationNode {
                    operation: settings.find_function_by_name("log").unwrap(),
                    arguments: vec![
                        Node::Value(ValueNode::Constant(5.0)),
                        Node::Function(OperationNode {
                            operation: settings.find_function_by_name("cos").unwrap(),
                            arguments: vec![Node::Value(ValueNode::Constant(10.0))],
                        }),
                    ],
                })],
            }),
            variables: vec![String::from("x")],
        };
        assert_eq!(expected_expression_tree, expression_tree);
    }

    #[test]
    fn test_replace_operation_mutation_function_to_operator() {
        let settings = Settings::default();
        let mut expression_tree = create_test_expression_tree(&settings);
        replace_operation_mutation(
            &mut expression_tree,
            &mut MockRandom::new(vec![10, 1, 2], vec![-5.0], vec![0.43, 0.55]),
            &settings,
        );
        let expected_expression_tree = ExpressionTree {
            root: Node::Operator(OperationNode {
                operation: settings.find_binary_operator_by_name("*").unwrap(),
                arguments: vec![
                    Node::Function(OperationNode {
                        operation: settings.find_function_by_name("log").unwrap(),
                        arguments: vec![
                            Node::Value(ValueNode::Constant(5.0)),
                            Node::Operator(OperationNode {
                                operation: settings.find_binary_operator_by_name("+").unwrap(),
                                arguments: vec![
                                    Node::Value(ValueNode::Variable(String::from("x"))),
                                    Node::Value(ValueNode::Constant(10.0)),
                                ],
                            }),
                        ],
                    }),
                    Node::Value(ValueNode::Constant(-5.0)),
                ],
            }),
            variables: vec![String::from("x")],
        };
        assert_eq!(expected_expression_tree, expression_tree);
    }

    #[test]
    fn test_remove_operation_mutation() {
        let settings = Settings::default();
        let mut expression_tree = create_test_expression_tree(&settings);
        remove_operation_mutation(
            &mut expression_tree,
            &mut MockRandom::new_int(vec![2, 0]),
            &settings,
        );
        let expected_expression_tree = ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("sin").unwrap(),
                arguments: vec![Node::Value(ValueNode::Constant(5.0))],
            }),
            variables: vec![String::from("x")],
        };
        assert_eq!(expected_expression_tree, expression_tree);
    }

    fn create_test_expression_tree(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("sin").unwrap(),
                arguments: vec![Node::Function(OperationNode {
                    operation: settings.find_function_by_name("log").unwrap(),
                    arguments: vec![
                        Node::Value(ValueNode::Constant(5.0)),
                        Node::Operator(OperationNode {
                            operation: settings.find_binary_operator_by_name("+").unwrap(),
                            arguments: vec![
                                Node::Value(ValueNode::Variable(String::from("x"))),
                                Node::Value(ValueNode::Constant(10.0)),
                            ],
                        }),
                    ],
                })],
            }),
            variables: vec![String::from("x")],
        }
    }
}
