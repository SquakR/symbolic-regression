mod expression_tree;

#[cfg(test)]
mod expression_tree_tests {
    use super::*;
    use expression_tree::{
        BinaryOperation, BinaryOperationKind, Node, Tree, UnaryOperation, UnaryOperationKind, Value,
    };
    use std::collections::HashMap;

    fn create_test_tree() -> Tree {
        Tree {
            root: Box::new(Node::BinaryOperation(BinaryOperation {
                kind: BinaryOperationKind::Addition,
                first_argument: Box::new(Node::BinaryOperation(BinaryOperation {
                    kind: BinaryOperationKind::Multiplication,
                    first_argument: Box::new(Node::Value(Value::Variable(String::from("x1")))),
                    second_argument: Box::new(Node::Value(Value::Constant(1.0))),
                })),
                second_argument: Box::new(Node::BinaryOperation(BinaryOperation {
                    kind: BinaryOperationKind::Multiplication,
                    first_argument: Box::new(Node::UnaryOperation(UnaryOperation {
                        kind: UnaryOperationKind::Sin,
                        argument: Box::new(Node::Value(Value::Variable(String::from("x1")))),
                    })),
                    second_argument: Box::new(Node::Value(Value::Variable(String::from("x2")))),
                })),
            })),
            variables: vec![String::from("x1"), String::from("x2")],
        }
    }

    #[test]
    #[should_panic(expected = "Expression tree does not contain y variable.")]
    fn test_subs_panics_with_wrong_variable() {
        let tree = create_test_tree();
        tree.subs(&HashMap::from([("y", 2.0)]));
    }

    #[test]
    fn test_subs_x1_variable() {
        let tree = create_test_tree();
        let new_tree = tree.subs(&HashMap::from([("x1", 2.0)]));
        match *new_tree.root {
            Node::BinaryOperation(root_operation) => {
                match *root_operation.first_argument {
                    Node::BinaryOperation(root_first_argument_operation) => {
                        match *root_first_argument_operation.first_argument {
                            Node::Value(value) => match value {
                                Value::Constant(constant) => assert_eq!(2.0, constant),
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        }
                        match *root_first_argument_operation.second_argument {
                            Node::Value(value) => match value {
                                Value::Constant(constant) => assert_eq!(1.0, constant),
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
                match *root_operation.second_argument {
                    Node::BinaryOperation(root_second_argument_operation) => {
                        match *root_second_argument_operation.first_argument {
                            Node::UnaryOperation(operation) => match *operation.argument {
                                Node::Value(value) => match value {
                                    Value::Constant(constant) => assert_eq!(2.0, constant),
                                    _ => unreachable!(),
                                },
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        }
                        match *root_second_argument_operation.second_argument {
                            Node::Value(value) => match value {
                                Value::Variable(variable) => assert_eq!("x2", variable),
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }
}
