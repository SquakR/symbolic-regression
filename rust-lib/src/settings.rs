//! Module with model settings.
use crate::types::{Associativity, Converter, ConverterOperation, Function, Operation, Operator};
use std::f64::{consts::E, consts::PI, NAN};
use std::rc::Rc;

pub struct Settings {
    pub operators: Vec<Rc<Operator>>,
    pub functions: Vec<Rc<Function>>,
    pub converters: Vec<Converter>,
}

impl Settings {
    pub fn default() -> Settings {
        let mut settings = Settings {
            operators: vec![],
            functions: vec![],
            converters: vec![],
        };
        settings.set_default_operators();
        settings.set_default_functions();
        settings.set_default_converters();
        settings
    }
    pub fn set_default_operators(&mut self) {
        self.operators = vec![
            Rc::new(Operator {
                name: String::from("+"),
                arguments_number: 2,
                precedence: 1,
                associativity: Associativity::Left,
                complexity: 1,
                compute_fn: |arguments| arguments[0] + arguments[1],
            }),
            Rc::new(Operator {
                name: String::from("-"),
                arguments_number: 2,
                precedence: 1,
                associativity: Associativity::Left,
                complexity: 1,
                compute_fn: |arguments| arguments[0] - arguments[1],
            }),
            Rc::new(Operator {
                name: String::from("*"),
                arguments_number: 2,
                precedence: 2,
                associativity: Associativity::Left,
                complexity: 2,
                compute_fn: |arguments| arguments[0] * arguments[1],
            }),
            Rc::new(Operator {
                name: String::from("/"),
                arguments_number: 2,
                precedence: 2,
                associativity: Associativity::Left,
                complexity: 2,
                compute_fn: |arguments| arguments[0] / arguments[1],
            }),
            Rc::new(Operator {
                name: String::from("^"),
                arguments_number: 2,
                precedence: 3,
                associativity: Associativity::Right,
                complexity: 3,
                compute_fn: |arguments| arguments[0].powf(arguments[1]),
            }),
            Rc::new(Operator {
                name: String::from("+"),
                arguments_number: 1,
                precedence: 4,
                associativity: Associativity::Right,
                complexity: 1,
                compute_fn: |arguments| arguments[0],
            }),
            Rc::new(Operator {
                name: String::from("-"),
                arguments_number: 1,
                precedence: 4,
                associativity: Associativity::Right,
                complexity: 1,
                compute_fn: |arguments| -arguments[0],
            }),
        ];
    }
    pub fn set_default_functions(&mut self) {
        self.functions = vec![
            Rc::new(Function {
                name: String::from("abs"),
                arguments_number: 1,
                complexity: 3,
                io_only: false,
                compute_fn: |arguments| arguments[0].abs(),
            }),
            Rc::new(Function {
                name: String::from("log"),
                arguments_number: 2,
                complexity: 4,
                io_only: false,
                compute_fn: |arguments| arguments[0].log(arguments[1]),
            }),
            Rc::new(Function {
                name: String::from("sin"),
                arguments_number: 1,
                complexity: 4,
                io_only: false,
                compute_fn: |arguments| arguments[0].sin(),
            }),
            Rc::new(Function {
                name: String::from("arcsin"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].asin(),
            }),
            Rc::new(Function {
                name: String::from("cos"),
                arguments_number: 1,
                complexity: 4,
                io_only: false,
                compute_fn: |arguments| arguments[0].cos(),
            }),
            Rc::new(Function {
                name: String::from("arccos"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].acos(),
            }),
            Rc::new(Function {
                name: String::from("tan"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].tan(),
            }),
            Rc::new(Function {
                name: String::from("arctan"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| arguments[0].atan(),
            }),
            Rc::new(Function {
                name: String::from("cot"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| 1.0 / arguments[0].tan(),
            }),
            Rc::new(Function {
                name: String::from("arccot"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| PI / 2.0 - arguments[0].atan(),
            }),
            Rc::new(Function {
                name: String::from("sinh"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].sinh(),
            }),
            Rc::new(Function {
                name: String::from("arsinh"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| arguments[0].asinh(),
            }),
            Rc::new(Function {
                name: String::from("cosh"),
                arguments_number: 1,
                complexity: 5,
                io_only: false,
                compute_fn: |arguments| arguments[0].cosh(),
            }),
            Rc::new(Function {
                name: String::from("arcosh"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| arguments[0].acosh(),
            }),
            Rc::new(Function {
                name: String::from("tanh"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| arguments[0].tanh(),
            }),
            Rc::new(Function {
                name: String::from("artanh"),
                arguments_number: 1,
                complexity: 7,
                io_only: false,
                compute_fn: |arguments| arguments[0].atanh(),
            }),
            Rc::new(Function {
                name: String::from("coth"),
                arguments_number: 1,
                complexity: 6,
                io_only: false,
                compute_fn: |arguments| 1.0 / arguments[0].tanh(),
            }),
            Rc::new(Function {
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
            }),
            Rc::new(Function {
                name: String::from("ln"),
                arguments_number: 1,
                complexity: 4,
                io_only: true,
                compute_fn: |arguments| arguments[0].ln(),
            }),
            Rc::new(Function {
                name: String::from("exp"),
                arguments_number: 1,
                complexity: 3,
                io_only: true,
                compute_fn: |arguments| arguments[0].exp(),
            }),
            Rc::new(Function {
                name: String::from("sqrt"),
                arguments_number: 1,
                complexity: 3,
                io_only: true,
                compute_fn: |arguments| arguments[0].sqrt(),
            }),
        ];
    }
    pub fn set_default_converters(&mut self) {
        let circumflex = self.operators.find_binary_by_name("^").unwrap();
        let log = self.functions.find_by_name("log").unwrap();
        let ln = self.functions.find_by_name("ln").unwrap();
        let exp = self.functions.find_by_name("exp").unwrap();
        let sqrt = self.functions.find_by_name("sqrt").unwrap();
        self.converters = vec![
            Converter {
                from: ConverterOperation::Function(Rc::clone(&ln)),
                to: ConverterOperation::Function(Rc::clone(&log)),
                is_conversion_possible_fn: |_| true,
                convert_fn: |arguments| vec![E, arguments[0]],
            },
            Converter {
                from: ConverterOperation::Function(Rc::clone(&exp)),
                to: ConverterOperation::Operator(Rc::clone(&circumflex)),
                is_conversion_possible_fn: |_| true,
                convert_fn: |arguments| vec![E, arguments[0]],
            },
            Converter {
                from: ConverterOperation::Function(Rc::clone(&sqrt)),
                to: ConverterOperation::Operator(Rc::clone(&circumflex)),
                is_conversion_possible_fn: |_| true,
                convert_fn: |arguments| vec![arguments[0], 0.5],
            },
            Converter {
                from: ConverterOperation::Function(Rc::clone(&log)),
                to: ConverterOperation::Function(Rc::clone(&ln)),
                is_conversion_possible_fn: |arguments| (arguments[0] - E).abs() <= 0.001,
                convert_fn: |arguments| vec![arguments[1]],
            },
            Converter {
                from: ConverterOperation::Operator(Rc::clone(&circumflex)),
                to: ConverterOperation::Function(Rc::clone(&exp)),
                is_conversion_possible_fn: |arguments| (arguments[0] - E).abs() <= 0.001,
                convert_fn: |arguments| vec![E, arguments[1]],
            },
            Converter {
                from: ConverterOperation::Operator(Rc::clone(&circumflex)),
                to: ConverterOperation::Function(Rc::clone(&sqrt)),
                is_conversion_possible_fn: |arguments| (arguments[1] - 0.5).abs() <= 0.001,
                convert_fn: |arguments| vec![arguments[0]],
            },
        ];
    }
}

pub trait FunctionCollection {
    fn find_by_name(&self, name: &str) -> Option<Rc<Function>>;
}

impl FunctionCollection for [Rc<Function>] {
    fn find_by_name(&self, name: &str) -> Option<Rc<Function>> {
        for function in self {
            if function.get_name() == name {
                return Some(Rc::clone(function));
            }
        }
        None
    }
}

pub trait OperatorCollection {
    fn find_unary_by_name(&self, name: &str) -> Option<Rc<Operator>>;
    fn find_binary_by_name(&self, name: &str) -> Option<Rc<Operator>>;
}

impl OperatorCollection for [Rc<Operator>] {
    fn find_unary_by_name(&self, name: &str) -> Option<Rc<Operator>> {
        for operator in self {
            if operator.get_name() == name && operator.arguments_number == 1 {
                return Some(Rc::clone(operator));
            }
        }
        None
    }
    fn find_binary_by_name(&self, name: &str) -> Option<Rc<Operator>> {
        for operator in self {
            if operator.get_name() == name && operator.arguments_number == 2 {
                return Some(Rc::clone(operator));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_unary_operator_by_name() {
        let settings = Settings::default();
        assert_eq!(
            settings.operators[5],
            settings.operators.find_unary_by_name("+").unwrap()
        );
        assert_eq!(None, settings.operators.find_unary_by_name("&"));
    }

    #[test]
    fn test_find_binary_operator_by_name() {
        let settings = Settings::default();
        assert_eq!(
            settings.operators[0],
            settings.operators.find_binary_by_name("+").unwrap()
        );
        assert_eq!(None, settings.operators.find_binary_by_name("&"));
    }

    #[test]
    fn test_find_function_by_name() {
        let settings = Settings::default();
        assert_eq!(
            settings.functions[0],
            settings.functions.find_by_name("abs").unwrap()
        );
        assert_eq!(None, settings.functions.find_by_name("fn"));
    }
}
