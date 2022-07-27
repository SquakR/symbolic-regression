/// Module with model settings.
use crate::types::{Associativity, Function, Operator};
use std::f64::{consts::PI, NAN};

pub struct Settings {
    pub operators: Vec<Operator>,
    pub functions: Vec<Function>,
}

pub fn get_default_settings() -> Settings {
    Settings {
        operators: vec![
            Operator {
                string: String::from("+"),
                arguments_number: 2,
                precedence: 1,
                associativity: Associativity::Left,
                complexity: 1,
                io_only: false,
                compute_fn: |arguments| arguments[0] + arguments[1],
            },
            Operator {
                string: String::from("-"),
                arguments_number: 2,
                precedence: 1,
                associativity: Associativity::Left,
                complexity: 1,
                io_only: false,
                compute_fn: |arguments| arguments[0] - arguments[1],
            },
            Operator {
                string: String::from("*"),
                arguments_number: 2,
                precedence: 2,
                associativity: Associativity::Left,
                complexity: 2,
                io_only: false,
                compute_fn: |arguments| arguments[0] * arguments[1],
            },
            Operator {
                string: String::from("/"),
                arguments_number: 2,
                precedence: 2,
                associativity: Associativity::Left,
                complexity: 2,
                io_only: false,
                compute_fn: |arguments| arguments[0] / arguments[1],
            },
            Operator {
                string: String::from("^"),
                arguments_number: 2,
                precedence: 3,
                associativity: Associativity::Right,
                complexity: 3,
                io_only: false,
                compute_fn: |arguments| arguments[0].powf(arguments[1]),
            },
            Operator {
                string: String::from("-"),
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
                string: String::from("log"),
                arguments_number: 2,
                complexity: 4,
                io_only: false,
                compute_fn: |arguments| arguments[0].log(arguments[1]),
            },
            Function {
                string: String::from("sin"),
                arguments_number: 1,
                complexity: 4,
                io_only: false,
                compute_fn: |arguments| arguments[0].sin(),
            },
            Function {
                string: String::from("arcsin"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].asin(),
            },
            Function {
                string: String::from("cos"),
                arguments_number: 1,
                complexity: 4,
                io_only: false,
                compute_fn: |arguments| arguments[0].cos(),
            },
            Function {
                string: String::from("arccos"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].acos(),
            },
            Function {
                string: String::from("tan"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].tan(),
            },
            Function {
                string: String::from("arctan"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| arguments[0].atan(),
            },
            Function {
                string: String::from("cot"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| 1.0 / arguments[0].tan(),
            },
            Function {
                string: String::from("arccot"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| PI / 2.0 - arguments[0].atan(),
            },
            Function {
                string: String::from("sinh"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].sinh(),
            },
            Function {
                string: String::from("arsinh"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| arguments[0].asinh(),
            },
            Function {
                string: String::from("cosh"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].cosh(),
            },
            Function {
                string: String::from("arcosh"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| arguments[0].acosh(),
            },
            Function {
                string: String::from("tanh"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| arguments[0].tanh(),
            },
            Function {
                string: String::from("artanh"),
                arguments_number: 1,
                complexity: 7,
                io_only: false,
                compute_fn: |arguments| arguments[0].atanh(),
            },
            Function {
                string: String::from("coth"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| 1.0 / arguments[0].tanh(),
            },
            Function {
                string: String::from("arcoth"),
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
                string: String::from("ln"),
                arguments_number: 1,
                complexity: 4,
                io_only: true,
                compute_fn: |arguments| arguments[0].ln(),
            },
            Function {
                string: String::from("exp"),
                arguments_number: 1,
                complexity: 3,
                io_only: true,
                compute_fn: |arguments| arguments[0].exp(),
            },
            Function {
                string: String::from("sqrt"),
                arguments_number: 1,
                complexity: 3,
                io_only: true,
                compute_fn: |arguments| arguments[0].sqrt(),
            },
        ],
    }
}
