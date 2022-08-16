//! Expression tree types module.
use crate::model::settings::{Function, Operation, Operator};
use serde::Serialize;
use std::rc::Rc;

#[derive(Debug, PartialEq, Serialize)]
pub struct ExpressionTree {
    pub root: Node,
    pub variables: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Operator(OperationNode<Operator>),
    Function(OperationNode<Function>),
    Value(ValueNode),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationNode<T: Operation> {
    pub operation: Rc<T>,
    pub arguments: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueNode {
    Variable(String),
    Constant(f64),
}
