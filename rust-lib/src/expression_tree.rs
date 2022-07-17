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

pub struct UnaryOperation<'a> {
    pub kind: UnaryOperationKind,
    pub argument: Box<Node<'a>>,
}

pub enum BinaryOperationKind {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Degree,
    Logarithm,
}

pub struct BinaryOperation<'a> {
    pub kind: BinaryOperationKind,
    pub first_argument: Box<Node<'a>>,
    pub second_argument: Box<Node<'a>>,
}

pub struct Variable {
    pub name: String,
    pub value: Option<f64>,
}

pub enum Value<'a> {
    Variable(&'a Variable),
    Value(f64),
}

pub enum Node<'a> {
    UnaryOperation(UnaryOperation<'a>),
    BinaryOperation(BinaryOperation<'a>),
    Value(Value<'a>),
}

pub struct Tree<'a> {
    root: Node<'a>,
    variables: Vec<Variable>,
}

impl<'a> Tree<'a> {}
