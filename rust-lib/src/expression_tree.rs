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
    Value(f64),
}

pub enum Node {
    UnaryOperation(UnaryOperation),
    BinaryOperation(BinaryOperation),
    Value(Value),
}

pub struct Tree {
    root: Node,
    variables: Vec<String>,
}

impl Tree {}
