use std::collections::HashMap;

#[derive(Copy, Clone)]
pub enum UnaryOperationKind {
    Inversion,
    Factorial,
    Exp,
    Ln,
    Sin,
    Arcsin,
    Cos,
    Arccos,
    Tan,
    Arctan,
    Cot,
    Arccot,
    Sinh,
    Arsinh,
    Cosh,
    Arcosh,
    Tanh,
    Artanh,
    Coth,
    Arcoth,
}

pub struct UnaryOperation {
    pub kind: UnaryOperationKind,
    pub argument: Box<Node>,
}

#[derive(Copy, Clone)]
pub enum BinaryOperationKind {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Degree,
    Logarithm,
}

pub struct BinaryOperation {
    pub kind: BinaryOperationKind,
    pub first_argument: Box<Node>,
    pub second_argument: Box<Node>,
}

pub enum Value {
    Variable(String),
    Constant(f64),
}

pub enum Node {
    UnaryOperation(UnaryOperation),
    BinaryOperation(BinaryOperation),
    Value(Value),
}

pub struct Tree {
    pub root: Box<Node>,
    pub variables: Vec<String>,
}

impl Tree {
    /// Return a new Tree where variables have been replaced with values from the `variables` HashMap.
    /// Panic if `variables` HaspMap contains non-existing variables.
    pub fn subs(&self, variables: &HashMap<&str, f64>) -> Tree {
        for &key in variables.keys() {
            if !self.variables.iter().any(|variable| variable == key) {
                panic!("Expression tree does not contain {} variable.", key);
            }
        }
        Tree {
            root: Tree::subs_node(&self.root, variables),
            variables: self
                .variables
                .clone()
                .into_iter()
                .filter(|variable| !variables.keys().any(|key| key == variable))
                .collect(),
        }
    }
    fn subs_node(node: &Box<Node>, variables: &HashMap<&str, f64>) -> Box<Node> {
        match &**node {
            Node::UnaryOperation(operation) => Box::new(Node::UnaryOperation(UnaryOperation {
                kind: operation.kind,
                argument: Tree::subs_node(&operation.argument, variables),
            })),
            Node::BinaryOperation(operation) => Box::new(Node::BinaryOperation(BinaryOperation {
                kind: operation.kind,
                first_argument: Tree::subs_node(&operation.first_argument, variables),
                second_argument: Tree::subs_node(&operation.second_argument, variables),
            })),
            Node::Value(value) => match value {
                Value::Constant(value) => Box::new(Node::Value(Value::Constant(*value))),
                Value::Variable(variable) => match variables.get(variable.as_str()) {
                    Some(constant) => Box::new(Node::Value(Value::Constant(*constant))),
                    None => Box::new(Node::Value(Value::Variable(variable.to_string()))),
                },
            },
        }
    }
}

#[cfg(test)]
mod expression_tree_tests {
    use super::*;

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
