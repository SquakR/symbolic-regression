/// Expression tree core functionality module.
use std::collections::HashMap;
use std::f64::{consts::PI, NAN};

pub trait Computable {
    /// Compute a node by computing all child nodes and performing a node operation.
    /// Return `ComputeError` if some node contains a variable instead of a constant.
    /// For computation, the idea of ​​the composite pattern is used.
    fn compute(&self) -> Result<f64, ComputeError>;
    /// Simplify a node by replacing all child nodes that can be computed with values.
    /// For example, sin(x + 2,0 * (3.0 + 2.0)) -> sin(x + 14.0)
    fn simplify(&mut self) -> ();
}

pub struct ExpressionTree {
    pub root: Box<Node>,
    pub variables: Vec<String>,
}

impl Computable for ExpressionTree {
    fn compute(&self) -> Result<f64, ComputeError> {
        (*self.root).compute()
    }
    fn simplify(&mut self) -> () {
        match self.compute() {
            Ok(value) => self.root = Box::new(Node::Value(Value::Constant(value))),
            Err(_) => self.root.simplify(),
        }
    }
}

impl ExpressionTree {
    /// Return a new ExpressionTree where variables have been replaced with values from the `variables` HashMap.
    /// Panic if `variables` HaspMap contains non-existing variables.
    pub fn subs(&self, variables: &HashMap<&str, f64>) -> ExpressionTree {
        for &key in variables.keys() {
            if !self.variables.iter().any(|variable| variable == key) {
                panic!("Expression tree does not contain {} variable.", key);
            }
        }
        ExpressionTree {
            root: ExpressionTree::subs_node(&self.root, variables),
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
                argument: ExpressionTree::subs_node(&operation.argument, variables),
            })),
            Node::BinaryOperation(operation) => Box::new(Node::BinaryOperation(BinaryOperation {
                kind: operation.kind,
                first_argument: ExpressionTree::subs_node(&operation.first_argument, variables),
                second_argument: ExpressionTree::subs_node(&operation.second_argument, variables),
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

pub enum Node {
    UnaryOperation(UnaryOperation),
    BinaryOperation(BinaryOperation),
    Value(Value),
}

impl Computable for Node {
    fn compute(&self) -> Result<f64, ComputeError> {
        match self {
            Node::UnaryOperation(unary_operation) => unary_operation.compute(),
            Node::BinaryOperation(binary_operation) => binary_operation.compute(),
            Node::Value(value) => value.compute(),
        }
    }
    fn simplify(&mut self) -> () {
        match self {
            Node::UnaryOperation(unary_operation) => unary_operation.simplify(),
            Node::BinaryOperation(binary_operation) => binary_operation.simplify(),
            Node::Value(value) => value.simplify(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum UnaryOperationKind {
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
    fn compute(&self) -> Result<f64, ComputeError> {
        let argument_result = self.argument.compute()?;
        match self.kind {
            UnaryOperationKind::Sin => Ok(argument_result.sin()),
            UnaryOperationKind::Arcsin => Ok(argument_result.asin()),
            UnaryOperationKind::Cos => Ok(argument_result.cos()),
            UnaryOperationKind::Arccos => Ok(argument_result.acos()),
            UnaryOperationKind::Tan => Ok(argument_result.tan()),
            UnaryOperationKind::Arctan => Ok(argument_result.atan()),
            UnaryOperationKind::Cot => Ok(1.0 / argument_result.tan()),
            UnaryOperationKind::Arccot => Ok(PI / 2.0 - argument_result.atan()),
            UnaryOperationKind::Sinh => Ok(argument_result.sinh()),
            UnaryOperationKind::Arsinh => Ok(argument_result.asinh()),
            UnaryOperationKind::Cosh => Ok(argument_result.cosh()),
            UnaryOperationKind::Arcosh => Ok(argument_result.acosh()),
            UnaryOperationKind::Tanh => Ok(argument_result.tanh()),
            UnaryOperationKind::Artanh => Ok(argument_result.atanh()),
            UnaryOperationKind::Coth => Ok(1.0 / argument_result.tanh()),
            UnaryOperationKind::Arcoth => {
                if argument_result < -1.0 || argument_result > 1.0 {
                    Ok(((argument_result + 1.0) / (argument_result - 1.0)).ln() * 0.5)
                } else {
                    Ok(NAN)
                }
            }
        }
    }
    fn simplify(&mut self) -> () {
        let mut computation_result: Option<Result<f64, ComputeError>> = None;
        match &*self.argument {
            Node::UnaryOperation(unary_operation) => {
                computation_result = Some(unary_operation.compute())
            }
            Node::BinaryOperation(binary_operation) => {
                computation_result = Some(binary_operation.compute())
            }
            _ => {}
        }
        if let Some(result) = computation_result {
            match result {
                Ok(value) => self.argument = Box::new(Node::Value(Value::Constant(value))),
                Err(_) => (*self.argument).simplify(),
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
    fn compute(&self) -> Result<f64, ComputeError> {
        let first_argument_result = (*self.first_argument).compute()?;
        let second_argument_result = (*self.second_argument).compute()?;
        match self.kind {
            BinaryOperationKind::Addition => Ok(first_argument_result + second_argument_result),
            BinaryOperationKind::Subtraction => Ok(first_argument_result - second_argument_result),
            BinaryOperationKind::Multiplication => {
                Ok(first_argument_result * second_argument_result)
            }
            BinaryOperationKind::Division => Ok(first_argument_result / second_argument_result),
            BinaryOperationKind::Power => Ok(first_argument_result.powf(second_argument_result)),
            BinaryOperationKind::Logarithm => Ok(first_argument_result.log(second_argument_result)),
        }
    }
    fn simplify(&mut self) -> () {
        for argument in [&mut self.first_argument, &mut self.second_argument] {
            let mut computation_result: Option<Result<f64, ComputeError>> = None;
            match &**argument {
                Node::UnaryOperation(unary_operation) => {
                    computation_result = Some(unary_operation.compute())
                }
                Node::BinaryOperation(binary_operation) => {
                    computation_result = Some(binary_operation.compute())
                }
                _ => {}
            }
            if let Some(result) = computation_result {
                match result {
                    Ok(value) => *argument = Box::new(Node::Value(Value::Constant(value))),
                    Err(_) => (argument).simplify(),
                }
            }
        }
    }
}

pub enum Value {
    Variable(String),
    Constant(f64),
}

impl Computable for Value {
    fn compute(&self) -> Result<f64, ComputeError> {
        match self {
            Value::Variable(variable) => Err(ComputeError::new(variable)),
            Value::Constant(constant) => Ok(*constant),
        }
    }
    fn simplify(&mut self) -> () {}
}

#[derive(Debug, Clone)]
pub struct ComputeError {
    pub message: String,
}

impl ComputeError {
    fn new(variable: &str) -> ComputeError {
        ComputeError {
            message: format!("{} variable is not a constant.", variable),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Expression tree does not contain y variable.")]
    fn test_subs_panics_with_wrong_variable() {
        let tree = create_test_tree_with_variables();
        tree.subs(&HashMap::from([("y", 2.0)]));
    }

    #[test]
    fn test_subs_x1_variable() {
        let tree = create_test_tree_with_variables();
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
    fn test_value_variable_compute() {
        if let Ok(_) = Value::Variable(String::from("x")).compute() {
            panic!("Computing a value with a variable must return a `ComputeError`.");
        }
    }

    #[test]
    fn test_value_constant_compute() -> Result<(), ComputeError> {
        let actual = Value::Constant(1.0).compute()?;
        assert_eq!(1.0_f64, actual);
        Ok(())
    }

    #[test]
    fn test_unary_operation_compute() -> Result<(), ComputeError> {
        for (expected, kind) in [
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
            .compute()?;
            assert_eq!(
                expected, actual,
                "For {:?} kind expected {:.3} but got {:.3}.",
                kind, expected, actual
            )
        }
        Ok(())
    }

    #[test]
    fn test_unary_operation_arcoth_less_compute() -> Result<(), ComputeError> {
        let actual = UnaryOperation {
            kind: UnaryOperationKind::Arcoth,
            argument: Box::new(Node::Value(Value::Constant(-1.5))),
        }
        .compute()?;
        assert_eq!(
            ((-1.5_f64 + 1.0_f64) / (-1.5_f64 - 1.0_f64)).ln() * 0.5,
            actual
        );
        Ok(())
    }

    #[test]
    fn test_unary_operation_arcoth_in_wrong_range_compute() -> Result<(), ComputeError> {
        let actual = UnaryOperation {
            kind: UnaryOperationKind::Arcoth,
            argument: Box::new(Node::Value(Value::Constant(0.5))),
        }
        .compute()?;
        assert!(actual.is_nan());
        Ok(())
    }

    #[test]
    fn test_unary_operation_arcoth_greater_compute() -> Result<(), ComputeError> {
        let actual = UnaryOperation {
            kind: UnaryOperationKind::Arcoth,
            argument: Box::new(Node::Value(Value::Constant(1.5))),
        }
        .compute()?;
        assert_eq!(
            ((1.5_f64 + 1.0_f64) / (1.5_f64 - 1.0_f64)).ln() * 0.5,
            actual
        );
        Ok(())
    }

    #[test]
    fn text_binary_operation_compute() -> Result<(), ComputeError> {
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
            .compute()?;
            assert_eq!(
                expected, actual,
                "For {:?} kind expected {:.3} but got {:.3}.",
                kind, expected, actual
            )
        }
        Ok(())
    }

    #[test]
    fn test_node_unary_operation_compute() -> Result<(), ComputeError> {
        let actual = Node::UnaryOperation(UnaryOperation {
            kind: UnaryOperationKind::Sin,
            argument: Box::new(Node::Value(Value::Constant(1.0))),
        })
        .compute()?;
        assert_eq!(1.0_f64.sin(), actual);
        Ok(())
    }

    #[test]
    fn test_node_binary_operation_compute() -> Result<(), ComputeError> {
        let actual = Node::BinaryOperation(BinaryOperation {
            kind: BinaryOperationKind::Addition,
            first_argument: Box::new(Node::Value(Value::Constant(2.0))),
            second_argument: Box::new(Node::Value(Value::Constant(2.0))),
        })
        .compute()?;
        assert_eq!(4.0_f64, actual);
        Ok(())
    }

    #[test]
    fn test_node_value_operation_compute() -> Result<(), ComputeError> {
        let actual = Node::Value(Value::Constant(1.0)).compute()?;
        assert_eq!(1.0_f64, actual);
        Ok(())
    }

    #[test]
    fn test_tree_with_variables_compute() {
        if let Ok(_) = create_test_tree_with_variables().compute() {
            panic!("Computing a tree with a variable must return a `ComputeError`.");
        }
    }

    #[test]
    fn test_tree_without_variables_compute() -> Result<(), ComputeError> {
        let actual = create_test_tree_without_variables().compute()?;
        assert_eq!(2.0_f64 * 3.0_f64 + 5.0_f64.sin() * 10.0_f64, actual);
        Ok(())
    }

    #[test]
    fn test_tree_without_variables_simplify() {
        let mut tree = create_test_tree_without_variables();
        tree.simplify();
        match *tree.root {
            Node::Value(value) => match value {
                Value::Constant(constant) => {
                    assert_eq!(2.0_f64 * 3.0_f64 + 5.0_f64.sin() * 10.0_f64, constant)
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_tree_to_simplify_simplify() {
        let mut tree = create_tree_to_simplify();
        tree.simplify();
        match *tree.root {
            Node::UnaryOperation(unary_operation) => {
                match unary_operation.kind {
                    UnaryOperationKind::Sin => {}
                    _ => unreachable!(),
                }
                match *unary_operation.argument {
                    Node::BinaryOperation(binary_operation) => {
                        match binary_operation.kind {
                            BinaryOperationKind::Addition => {}
                            _ => unreachable!(),
                        }
                        match *binary_operation.first_argument {
                            Node::Value(value) => match value {
                                Value::Variable(variable) => assert_eq!("x", variable),
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        }
                        match *binary_operation.second_argument {
                            Node::Value(value) => match value {
                                Value::Constant(constant) => assert_eq!(14.0_f64, constant),
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

    fn create_test_tree_with_variables() -> ExpressionTree {
        ExpressionTree {
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

    fn create_test_tree_without_variables() -> ExpressionTree {
        ExpressionTree {
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

    fn create_tree_to_simplify() -> ExpressionTree {
        ExpressionTree {
            root: Box::new(Node::UnaryOperation(UnaryOperation {
                kind: UnaryOperationKind::Sin,
                argument: Box::new(Node::BinaryOperation(BinaryOperation {
                    kind: BinaryOperationKind::Addition,
                    first_argument: Box::new(Node::Value(Value::Variable(String::from("x")))),
                    second_argument: Box::new(Node::BinaryOperation(BinaryOperation {
                        kind: BinaryOperationKind::Multiplication,
                        first_argument: Box::new(Node::Value(Value::Constant(2.0))),
                        second_argument: Box::new(Node::BinaryOperation(BinaryOperation {
                            kind: BinaryOperationKind::Addition,
                            first_argument: Box::new(Node::Value(Value::Constant(3.0))),
                            second_argument: Box::new(Node::Value(Value::Constant(4.0))),
                        })),
                    })),
                })),
            })),
            variables: vec![String::from("x")],
        }
    }
}
