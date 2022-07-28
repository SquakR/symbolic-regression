//! Module with model settings.
use crate::types::{Associativity, Function, Operation, Operator};
use std::f64::{consts::PI, NAN};

pub struct Settings {
    pub operators: Vec<Operator>,
    pub functions: Vec<Function>,
}

pub trait OperationCollection<T: Operation + Clone> {
    fn find_by_name(&self, name: &str) -> Option<&T>;
}

impl<T: Operation + Clone> OperationCollection<T> for Vec<T> {
    fn find_by_name(&self, name: &str) -> Option<&T> {
        for operation in self {
            if operation.get_name() == name {
                return Some(operation);
            }
        }
        None
    }
}

pub fn get_default_settings() -> Settings {
    Settings {
        operators: vec![
            Operator {
                name: String::from("+"),
                arguments_number: 2,
                precedence: 1,
                associativity: Associativity::Left,
                complexity: 1,
                io_only: false,
                compute_fn: |arguments| arguments[0] + arguments[1],
            },
            Operator {
                name: String::from("-"),
                arguments_number: 2,
                precedence: 1,
                associativity: Associativity::Left,
                complexity: 1,
                io_only: false,
                compute_fn: |arguments| arguments[0] - arguments[1],
            },
            Operator {
                name: String::from("*"),
                arguments_number: 2,
                precedence: 2,
                associativity: Associativity::Left,
                complexity: 2,
                io_only: false,
                compute_fn: |arguments| arguments[0] * arguments[1],
            },
            Operator {
                name: String::from("/"),
                arguments_number: 2,
                precedence: 2,
                associativity: Associativity::Left,
                complexity: 2,
                io_only: false,
                compute_fn: |arguments| arguments[0] / arguments[1],
            },
            Operator {
                name: String::from("^"),
                arguments_number: 2,
                precedence: 3,
                associativity: Associativity::Right,
                complexity: 3,
                io_only: false,
                compute_fn: |arguments| arguments[0].powf(arguments[1]),
            },
            Operator {
                name: String::from("-"),
                arguments_number: 1,
                precedence: 1,
                associativity: Associativity::Left,
                complexity: 1,
                io_only: true,
                compute_fn: |arguments| -arguments[0],
            },
        ],
        functions: vec![
            Function {
                name: String::from("log"),
                arguments_number: 2,
                complexity: 4,
                io_only: false,
                compute_fn: |arguments| arguments[0].log(arguments[1]),
            },
            Function {
                name: String::from("sin"),
                arguments_number: 1,
                complexity: 4,
                io_only: false,
                compute_fn: |arguments| arguments[0].sin(),
            },
            Function {
                name: String::from("arcsin"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].asin(),
            },
            Function {
                name: String::from("cos"),
                arguments_number: 1,
                complexity: 4,
                io_only: false,
                compute_fn: |arguments| arguments[0].cos(),
            },
            Function {
                name: String::from("arccos"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].acos(),
            },
            Function {
                name: String::from("tan"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].tan(),
            },
            Function {
                name: String::from("arctan"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| arguments[0].atan(),
            },
            Function {
                name: String::from("cot"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| 1.0 / arguments[0].tan(),
            },
            Function {
                name: String::from("arccot"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| PI / 2.0 - arguments[0].atan(),
            },
            Function {
                name: String::from("sinh"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].sinh(),
            },
            Function {
                name: String::from("arsinh"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| arguments[0].asinh(),
            },
            Function {
                name: String::from("cosh"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].cosh(),
            },
            Function {
                name: String::from("arcosh"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| arguments[0].acosh(),
            },
            Function {
                name: String::from("tanh"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| arguments[0].tanh(),
            },
            Function {
                name: String::from("artanh"),
                arguments_number: 1,
                complexity: 7,
                io_only: false,
                compute_fn: |arguments| arguments[0].atanh(),
            },
            Function {
                name: String::from("coth"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| 1.0 / arguments[0].tanh(),
            },
            Function {
                name: String::from("arcoth"),
                arguments_number: 1,
                complexity: 7,
                io_only: false,
                compute_fn: |arguments| {
                    if arguments[0] < -1.0 || arguments[0] > 1.0 {
                        ((arguments[0] + 1.0) / (arguments[0] - 1.0)).ln() * 0.5
                    } else {
                        NAN
                    }
                },
            },
            Function {
                name: String::from("ln"),
                arguments_number: 1,
                complexity: 4,
                io_only: true,
                compute_fn: |arguments| arguments[0].ln(),
            },
            Function {
                name: String::from("exp"),
                arguments_number: 1,
                complexity: 3,
                io_only: true,
                compute_fn: |arguments| arguments[0].exp(),
            },
            Function {
                name: String::from("sqrt"),
                arguments_number: 1,
                complexity: 3,
                io_only: true,
                compute_fn: |arguments| arguments[0].sqrt(),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_operator_by_name() {
        let settings = get_default_settings();
        assert_eq!(
            settings.functions[0],
            *settings.functions.find_by_name("log").unwrap()
        );
        assert_eq!(None, settings.functions.find_by_name("fn"));
    }

    #[test]
    fn test_find_function_by_name() {
        let settings = get_default_settings();
        assert_eq!(
            settings.operators[0],
            *settings.operators.find_by_name("+").unwrap()
        );
        assert_eq!(None, settings.operators.find_by_name("&"));
    }
}
