//! Module with crossing two expression trees.
use crate::expression_tree::{ExpressionTree, Random};

pub fn cross<R>(
    expression_tree1: &ExpressionTree,
    expression_tree2: &ExpressionTree,
    random: &mut R,
) -> ExpressionTree
where
    R: Random,
{
    assert_eq!(
        expression_tree1.variables, expression_tree2.variables,
        "Expression trees must contain the same variables, but the first expression tree contains {:?} and the second contains {:?}.",
        expression_tree1.variables, expression_tree2.variables
    );
    let mut result_tree = expression_tree1.clone();
    *result_tree.get_random_node_mut(random) = expression_tree2.get_random_node(random).clone();
    result_tree
}

#[cfg(test)]
mod tests {
    use super::super::settings::Settings;
    use super::*;
    use crate::expression_tree::{MockRandom, Node, OperationNode, ValueNode};

    #[test]
    fn test_cross() {
        let settings = Settings::default();
        let expression_tree1 = create_expression_tree1(&settings);
        let expression_tree2 = create_expression_tree2(&settings);
        let expected_expression_tree = ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("sin").unwrap(),
                arguments: vec![Node::Operator(OperationNode {
                    operation: settings.find_binary_operator_by_name("+").unwrap(),
                    arguments: vec![
                        Node::Value(ValueNode::Variable(String::from("x1"))),
                        Node::Operator(OperationNode {
                            operation: settings.find_binary_operator_by_name("-").unwrap(),
                            arguments: vec![
                                Node::Value(ValueNode::Variable(String::from("x2"))),
                                Node::Value(ValueNode::Constant(5.0)),
                            ],
                        }),
                    ],
                })],
            }),
            variables: vec![String::from("x1"), String::from("x2")],
        };
        let actual_expression_tree = cross(
            &expression_tree1,
            &expression_tree2,
            &mut MockRandom::new_int(vec![1, 3]),
        );
        assert_eq!(expected_expression_tree, actual_expression_tree);
    }

    #[test]
    #[should_panic(
        expected = r#"Expression trees must contain the same variables, but the first expression tree contains ["x1"] and the second contains ["x1", "x2"]."#
    )]
    fn test_cross_with_different_variables() {
        let settings = Settings::default();
        let mut expression_tree1 = create_expression_tree1(&settings);
        expression_tree1.variables = vec![String::from("x1")];
        let expression_tree2 = create_expression_tree2(&settings);
        cross(
            &expression_tree1,
            &expression_tree2,
            &mut MockRandom::new_int(vec![0]),
        );
    }

    fn create_expression_tree1(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("sin").unwrap(),
                arguments: vec![Node::Operator(OperationNode {
                    operation: settings.find_binary_operator_by_name("+").unwrap(),
                    arguments: vec![
                        Node::Value(ValueNode::Variable(String::from("x1"))),
                        Node::Value(ValueNode::Variable(String::from("x2"))),
                    ],
                })],
            }),
            variables: vec![String::from("x1"), String::from("x2")],
        }
    }

    fn create_expression_tree2(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("cos").unwrap(),
                arguments: vec![Node::Operator(OperationNode {
                    operation: settings.find_binary_operator_by_name("-").unwrap(),
                    arguments: vec![
                        Node::Value(ValueNode::Variable(String::from("x2"))),
                        Node::Value(ValueNode::Constant(5.0)),
                    ],
                })],
            }),
            variables: vec![String::from("x1"), String::from("x2")],
        }
    }
}
