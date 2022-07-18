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
