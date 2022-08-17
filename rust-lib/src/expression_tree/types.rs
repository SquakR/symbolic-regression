//! Expression tree types module.
use serde::Serialize;
use std::cmp::Ordering;
use std::fmt;
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

pub trait Operation {
    fn compute(&self, arguments: &[f64]) -> f64;
    fn get_name(&self) -> &str;
    fn get_complexity(&self) -> u32;
}

#[derive(Clone)]
pub struct Operator {
    pub name: String,
    pub precedence: u8,
    pub associativity: Associativity,
    pub arguments_number: usize,
    pub complexity: u32,
    pub compute_fn: fn(arguments: &[f64]) -> f64,
}

impl Operation for Operator {
    fn compute(&self, arguments: &[f64]) -> f64 {
        if arguments.len() != self.arguments_number {
            panic!(
                "The operator `{}` expected {} arguments, but received {}.",
                self,
                self.arguments_number,
                arguments.len()
            );
        }
        (self.compute_fn)(arguments)
    }
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
    fn get_complexity(&self) -> u32 {
        self.complexity
    }
}

impl fmt::Debug for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Operator")
            .field("name", &self.name)
            .field("arguments_number", &self.arguments_number)
            .field("precedence", &self.precedence)
            .field("associativity", &self.associativity)
            .field("complexity", &self.complexity)
            .finish()
    }
}

impl PartialEq for Operator {
    fn eq(&self, other: &Operator) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl Operator {
    pub fn is_computed_before(&self, other: &Operator) -> bool {
        match self.precedence.cmp(&other.precedence) {
            Ordering::Equal => match other.associativity {
                Associativity::Left => true,
                Associativity::Right => false,
            },
            Ordering::Greater => true,
            Ordering::Less => false,
        }
    }
}

#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub arguments_number: usize,
    pub complexity: u32,
    pub compute_fn: fn(arguments: &[f64]) -> f64,
}

impl Operation for Function {
    fn compute(&self, arguments: &[f64]) -> f64 {
        if arguments.len() != self.arguments_number {
            panic!(
                "The function `{}` expected {} arguments, but received {}.",
                self,
                self.arguments_number,
                arguments.len()
            );
        }
        (self.compute_fn)(arguments)
    }
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
    fn get_complexity(&self) -> u32 {
        self.complexity
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Function")
            .field("name", &self.name)
            .field("arguments_number", &self.arguments_number)
            .field("complexity", &self.complexity)
            .finish()
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Function) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Associativity {
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::settings::Settings;

    mod operator_tests {
        use super::*;

        #[test]
        fn test_debug() {
            let settings = Settings::default();
            let plus_operator = settings.find_binary_operator_by_name("+").unwrap();
            assert_eq!(
                "Operator { name: \"+\", arguments_number: 2, precedence: 1, associativity: Left, complexity: 1 }",
                format!("{:?}", plus_operator)
            );
        }

        #[test]
        fn test_eq() {
            let settings = Settings::default();
            let plus_operator1 = settings.find_binary_operator_by_name("+").unwrap();
            let plus_operator2 = settings.find_binary_operator_by_name("+").unwrap();
            assert!(plus_operator1 == plus_operator2);
            let minus_operator = settings.find_binary_operator_by_name("-").unwrap();
            assert!(plus_operator1 != minus_operator);
        }

        #[test]
        fn test_compute() {
            let settings = Settings::default();
            let plus_operator = settings.find_binary_operator_by_name("+").unwrap();
            assert_eq!(3.0, plus_operator.compute(&[1.0, 2.0]));
        }

        #[test]
        #[should_panic(expected = "The operator `+` expected 2 arguments, but received 1.")]
        fn test_compute_panic() {
            let settings = Settings::default();
            let plus_operator = settings.find_binary_operator_by_name("+").unwrap();
            plus_operator.compute(&[1.0]);
        }

        #[test]
        fn test_operator_is_computed_before() {
            let settings = Settings::default();
            let plus_operator = settings.find_binary_operator_by_name("+").unwrap();
            let minus_operator = settings.find_binary_operator_by_name("-").unwrap();
            let asterisk_operator = settings.find_binary_operator_by_name("*").unwrap();
            let slash_operator = settings.find_binary_operator_by_name("/").unwrap();
            let circumflex_operator = settings.find_binary_operator_by_name("^").unwrap();
            assert!(plus_operator.is_computed_before(&plus_operator));
            assert!(plus_operator.is_computed_before(&minus_operator));
            assert!(!plus_operator.is_computed_before(&asterisk_operator));
            assert!(circumflex_operator.is_computed_before(&slash_operator));
        }
    }

    mod function_tests {
        use super::*;

        #[test]
        fn test_debug() {
            let settings = Settings::default();
            let sin_function = settings.find_function_by_name("sin").unwrap();
            assert_eq!(
                "Function { name: \"sin\", arguments_number: 1, complexity: 4 }",
                format!("{:?}", sin_function)
            );
        }

        #[test]
        fn test_eq() {
            let settings = Settings::default();
            let sin_function1 = settings.find_function_by_name("sin").unwrap();
            let sin_function2 = settings.find_function_by_name("sin").unwrap();
            let cos_function = settings.find_function_by_name("cos").unwrap();
            assert!(sin_function1 == sin_function2);
            assert!(sin_function1 != cos_function);
        }

        #[test]
        fn test_compute() {
            let settings = Settings::default();
            let sin_function = settings.find_function_by_name("sin").unwrap();
            assert_eq!(2.0_f64.sin(), sin_function.compute(&[2.0]));
        }

        #[test]
        #[should_panic(expected = "The function `sin` expected 1 arguments, but received 2.")]
        fn test_compute_panic() {
            let settings = Settings::default();
            let sin_function = settings.find_function_by_name("sin").unwrap();
            sin_function.compute(&[1.0, 2.0]);
        }
    }
}
