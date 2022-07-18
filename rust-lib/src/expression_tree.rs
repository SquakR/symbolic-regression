use std::collections::HashMap;
use std::f64::{consts::PI, NAN};

pub trait Computable {
    /// Compute a node by computing all child nodes and performing a node operation.
    /// Panic if some node contains a variable instead of a constant.
    /// For computation, the idea of ​​the composite pattern is used.
    fn compute(&self) -> f64;
}

#[derive(Debug, Copy, Clone)]
pub enum UnaryOperationKind {
    Inversion,
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

impl Computable for UnaryOperation {
    fn compute(&self) -> f64 {
        let argument_result = self.argument.compute();
        match self.kind {
            UnaryOperationKind::Inversion => -argument_result,
            UnaryOperationKind::Exp => argument_result.exp(),
            UnaryOperationKind::Ln => argument_result.ln(),
            UnaryOperationKind::Sin => argument_result.sin(),
            UnaryOperationKind::Arcsin => argument_result.asin(),
            UnaryOperationKind::Cos => argument_result.cos(),
            UnaryOperationKind::Arccos => argument_result.acos(),
            UnaryOperationKind::Tan => argument_result.tan(),
            UnaryOperationKind::Arctan => argument_result.atan(),
            UnaryOperationKind::Cot => 1.0 / argument_result.tan(),
            UnaryOperationKind::Arccot => PI / 2.0 - argument_result.atan(),
            UnaryOperationKind::Sinh => argument_result.sinh(),
            UnaryOperationKind::Arsinh => argument_result.asinh(),
            UnaryOperationKind::Cosh => argument_result.cosh(),
            UnaryOperationKind::Arcosh => argument_result.acosh(),
            UnaryOperationKind::Tanh => argument_result.tanh(),
            UnaryOperationKind::Artanh => argument_result.atanh(),
            UnaryOperationKind::Coth => 1.0 / argument_result.tanh(),
            UnaryOperationKind::Arcoth => {
                if argument_result < -1.0 || argument_result > 1.0 {
                    ((argument_result + 1.0) / (argument_result - 1.0)).ln() * 0.5
                } else {
                    NAN
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum BinaryOperationKind {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Power,
    Logarithm,
}

pub struct BinaryOperation {
    pub kind: BinaryOperationKind,
    pub first_argument: Box<Node>,
    pub second_argument: Box<Node>,
}

impl Computable for BinaryOperation {
    fn compute(&self) -> f64 {
        let first_argument_result = (*self.first_argument).compute();
        let second_argument_result = (*self.second_argument).compute();
        match self.kind {
            BinaryOperationKind::Addition => first_argument_result + second_argument_result,
            BinaryOperationKind::Subtraction => first_argument_result - second_argument_result,
            BinaryOperationKind::Multiplication => first_argument_result * second_argument_result,
            BinaryOperationKind::Division => first_argument_result / second_argument_result,
            BinaryOperationKind::Power => first_argument_result.powf(second_argument_result),
            BinaryOperationKind::Logarithm => first_argument_result.log(second_argument_result),
        }
    }
}

pub enum Value {
    Variable(String),
    Constant(f64),
}

impl Computable for Value {
    fn compute(&self) -> f64 {
        match self {
            Value::Variable(variable) => panic!("{} variable is not a constant.", variable),
            Value::Constant(constant) => *constant,
        }
    }
}

pub enum Node {
    UnaryOperation(UnaryOperation),
    BinaryOperation(BinaryOperation),
    Value(Value),
}

impl Computable for Node {
    fn compute(&self) -> f64 {
        match self {
            Node::UnaryOperation(unary_operation) => unary_operation.compute(),
            Node::BinaryOperation(binary_operation) => binary_operation.compute(),
            Node::Value(value) => value.compute(),
        }
    }
}

pub struct Tree {
    pub root: Box<Node>,
    pub variables: Vec<String>,
}

impl Computable for Tree {
    fn compute(&self) -> f64 {
        (*self.root).compute()
    }
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
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Expression tree does not contain y variable.")]
    fn test_subs_panics_with_wrong_variable() {
        let tree = create_subs_test_tree();
        tree.subs(&HashMap::from([("y", 2.0)]));
    }

    #[test]
    fn test_subs_x1_variable() {
        let tree = create_subs_test_tree();
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

    #[test]
    #[should_panic(expected = "x variable is not a constant.")]
    fn test_value_variable_compute() {
        Value::Variable(String::from("x")).compute();
    }

    #[test]
    fn test_value_constant_compute() {
        assert_eq!(1.0_f64, Value::Constant(1.0).compute());
    }

    #[test]
    fn test_unary_operation_compute() {
        for (expected, kind) in [
            (-1.0_f64, UnaryOperationKind::Inversion),
            (1.0_f64.exp(), UnaryOperationKind::Exp),
            (1.0_f64.ln(), UnaryOperationKind::Ln),
            (1.0_f64.sin(), UnaryOperationKind::Sin),
            (1.0_f64.asin(), UnaryOperationKind::Arcsin),
            (1.0_f64.cos(), UnaryOperationKind::Cos),
            (1.0_f64.acos(), UnaryOperationKind::Arccos),
            (1.0_f64.tan(), UnaryOperationKind::Tan),
            (1.0_f64.atan(), UnaryOperationKind::Arctan),
            (1.0 / 1.0_f64.tan(), UnaryOperationKind::Cot),
            (PI / 2.0 - 1.0_f64.atan(), UnaryOperationKind::Arccot),
            (1.0_f64.sinh(), UnaryOperationKind::Sinh),
            (1.0_f64.asinh(), UnaryOperationKind::Arsinh),
            (1.0_f64.cosh(), UnaryOperationKind::Cosh),
            (1.0_f64.acosh(), UnaryOperationKind::Arcosh),
            (1.0_f64.tanh(), UnaryOperationKind::Tanh),
            (1.0_f64.atanh(), UnaryOperationKind::Artanh),
            (1.0 / 1.0_f64.tanh(), UnaryOperationKind::Coth),
        ] {
            let actual = UnaryOperation {
                kind,
                argument: Box::new(Node::Value(Value::Constant(1.0))),
            }
            .compute();
            assert_eq!(
                expected, actual,
                "For {:?} kind expected {:.3} but got {:.3}.",
                kind, expected, actual
            )
        }
    }

    #[test]
    fn test_unary_operation_arcoth_less_compute() {
        assert_eq!(
            ((-1.5_f64 + 1.0_f64) / (-1.5_f64 - 1.0_f64)).ln() * 0.5,
            UnaryOperation {
                kind: UnaryOperationKind::Arcoth,
                argument: Box::new(Node::Value(Value::Constant(-1.5))),
            }
            .compute()
        );
    }

    #[test]
    fn test_unary_operation_arcoth_in_wrong_range_compute() {
        assert!(UnaryOperation {
            kind: UnaryOperationKind::Arcoth,
            argument: Box::new(Node::Value(Value::Constant(0.5))),
        }
        .compute()
        .is_nan());
    }

    #[test]
    fn test_unary_operation_arcoth_greater_compute() {
        assert_eq!(
            ((1.5_f64 + 1.0_f64) / (1.5_f64 - 1.0_f64)).ln() * 0.5,
            UnaryOperation {
                kind: UnaryOperationKind::Arcoth,
                argument: Box::new(Node::Value(Value::Constant(1.5))),
            }
            .compute()
        );
    }

    #[test]
    fn text_binary_operation_compute() {
        for (expected, kind) in [
            (5.0_f64, BinaryOperationKind::Addition),
            (-1.0_f64, BinaryOperationKind::Subtraction),
            (6.0_f64, BinaryOperationKind::Multiplication),
            (2.0_f64 / 3.0_f64, BinaryOperationKind::Division),
            (2.0_f64.powf(3.0_f64), BinaryOperationKind::Power),
            (2.0_f64.log(3.0_f64), BinaryOperationKind::Logarithm),
        ] {
            let actual = BinaryOperation {
                kind,
                first_argument: Box::new(Node::Value(Value::Constant(2.0))),
                second_argument: Box::new(Node::Value(Value::Constant(3.0))),
            }
            .compute();
            assert_eq!(
                expected, actual,
                "For {:?} kind expected {:.3} but got {:.3}.",
                kind, expected, actual
            )
        }
    }

    #[test]
    fn test_node_unary_operation_compute() {
        assert_eq!(
            -1.0_f64,
            Node::UnaryOperation(UnaryOperation {
                kind: UnaryOperationKind::Inversion,
                argument: Box::new(Node::Value(Value::Constant(1.0)))
            })
            .compute()
        );
    }

    #[test]
    fn test_node_binary_operation_compute() {
        assert_eq!(
            4.0_f64,
            Node::BinaryOperation(BinaryOperation {
                kind: BinaryOperationKind::Addition,
                first_argument: Box::new(Node::Value(Value::Constant(2.0))),
                second_argument: Box::new(Node::Value(Value::Constant(2.0)))
            })
            .compute()
        );
    }

    #[test]
    fn test_node_value_operation_compute() {
        assert_eq!(1.0_f64, Node::Value(Value::Constant(1.0)).compute());
    }

    #[test]
    fn test_tree_compute() {
        let tree = create_compute_test_tree();
        assert_eq!(2.0_f64 * 3.0_f64 + 5.0_f64.sin() * 10.0_f64, tree.compute());
    }

    fn create_subs_test_tree() -> Tree {
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

    fn create_compute_test_tree() -> Tree {
        Tree {
            root: Box::new(Node::BinaryOperation(BinaryOperation {
                kind: BinaryOperationKind::Addition,
                first_argument: Box::new(Node::BinaryOperation(BinaryOperation {
                    kind: BinaryOperationKind::Multiplication,
                    first_argument: Box::new(Node::Value(Value::Constant(2.0))),
                    second_argument: Box::new(Node::Value(Value::Constant(3.0))),
                })),
                second_argument: Box::new(Node::BinaryOperation(BinaryOperation {
                    kind: BinaryOperationKind::Multiplication,
                    first_argument: Box::new(Node::UnaryOperation(UnaryOperation {
                        kind: UnaryOperationKind::Sin,
                        argument: Box::new(Node::Value(Value::Constant(5.0))),
                    })),
                    second_argument: Box::new(Node::Value(Value::Constant(10.0))),
                })),
            })),
            variables: vec![],
        }
    }
}
