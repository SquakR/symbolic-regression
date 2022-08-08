//! Module with common types.
use std::cmp::PartialEq;
use std::fmt;

pub trait Operation {
    fn compute(&self, arguments: &[f64]) -> f64;
    fn get_name(&self) -> &str;
}

#[derive(Clone)]
pub struct Operator {
    pub name: String,
    pub precedence: u8,
    pub associativity: Associativity,
    pub arguments_number: usize,
    pub complexity: u32,
    pub compute_fn: fn(&[f64]) -> f64,
}

impl Operation for Operator {
    fn compute(&self, arguments: &[f64]) -> f64 {
        if arguments.len() != self.arguments_number {
            panic!(
                "The function `{}` expected {} arguments but received {}.",
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

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq for Operator {
    fn eq(&self, other: &Operator) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

#[derive(Clone)]
pub struct Function {
    pub name: String,
    pub arguments_number: usize,
    pub complexity: u32,
    pub io_only: bool,
    pub compute_fn: fn(&[f64]) -> f64,
}

impl Operation for Function {
    fn compute(&self, arguments: &[f64]) -> f64 {
        if arguments.len() != self.arguments_number {
            panic!(
                "The function `{}` expected {} arguments but received {}.",
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
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Function")
            .field("name", &self.name)
            .field("arguments_number", &self.arguments_number)
            .field("complexity", &self.complexity)
            .field("io_only", &self.io_only)
            .finish()
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
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

pub struct FunctionConverter<'a> {
    pub from: &'a Function,
    pub to: &'a Function,
    pub is_conversion_possible_fn: fn(&'a [f64]) -> bool,
    pub convert_fn: fn(&'a [f64]) -> Vec<f64>,
}

impl<'a> FunctionConverter<'a> {
    fn is_conversion_possible(&self, function: &'a Function, arguments: &'a [f64]) -> bool {
        function == self.from && (self.is_conversion_possible_fn)(arguments)
    }
    fn convert(&self, arguments: &'a [f64]) -> ConvertOutputData<'a> {
        ConvertOutputData {
            function: self.to,
            arguments: (self.convert_fn)(arguments),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ConvertOutputData<'a> {
    pub function: &'a Function,
    pub arguments: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod operator_tests {
        use super::*;

        #[test]
        fn test_debug() {
            let test_operator = create_test_operator();
            assert_eq!(
                "Operator { name: \"+\", arguments_number: 2, precedence: 1, associativity: Left, complexity: 1 }",
                format!("{:?}", test_operator)
            );
        }

        #[test]
        fn test_display() {
            let test_operator = create_test_operator();
            assert_eq!("+", format!("{}", test_operator));
        }

        #[test]
        fn test_eq() {
            let test_operator1 = create_test_operator();
            let mut test_operator2 = create_test_operator();
            assert!(test_operator1 == test_operator2);
            test_operator2.name = String::from("-");
            assert!(test_operator1 != test_operator2);
        }

        #[test]
        fn test_compute() {
            let test_operator = create_test_operator();
            assert_eq!(3.0, test_operator.compute(&[1.0, 2.0]));
        }

        fn create_test_operator() -> Operator {
            Operator {
                name: String::from("+"),
                arguments_number: 2,
                precedence: 1,
                associativity: Associativity::Left,
                complexity: 1,
                compute_fn: |arguments| arguments[0] + arguments[1],
            }
        }
    }

    mod function_tests {
        use super::*;
        use std::f64::consts::E;

        #[test]
        fn test_debug() {
            let test_function = create_sin_function();
            assert_eq!(
                "Function { name: \"sin\", arguments_number: 1, complexity: 1, io_only: false }",
                format!("{:?}", test_function)
            );
        }

        #[test]
        fn test_display() {
            let test_function = create_sin_function();
            assert_eq!("sin", format!("{}", test_function));
        }

        #[test]
        fn test_eq() {
            let test_function1 = create_sin_function();
            let mut test_function2 = create_sin_function();
            assert!(test_function1 == test_function2);
            test_function2.name = String::from("cos");
            assert!(test_function1 != test_function2);
        }

        #[test]
        fn test_compute() {
            let test_function = create_sin_function();
            assert_eq!(2.0_f64.sin(), test_function.compute(&[2.0]));
        }

        #[test]
        #[should_panic(expected = "The function `sin` expected 1 arguments but received 2.")]
        fn test_compute_panic() {
            let test_function = create_sin_function();
            assert_eq!(2.0_f64.sin(), test_function.compute(&[1.0, 2.0]));
        }

        #[test]
        fn test_is_conversion_possible() {
            let log_function = create_log_function();
            let ln_function = create_ln_function();
            let converter = FunctionConverter {
                from: &log_function,
                to: &ln_function,
                is_conversion_possible_fn: |arguments| (arguments[0] - E).abs() <= 0.001,
                convert_fn: |arguments| vec![arguments[1]],
            };
            let possible_arguments = vec![E + 0.0001, 10.0];
            let not_possible_arguments = vec![E + 0.01, 10.0];
            assert!(converter.is_conversion_possible(&log_function, &possible_arguments));
            assert!(!converter.is_conversion_possible(&ln_function, &not_possible_arguments));
        }

        #[test]
        fn test_convert() {
            let log_function = create_log_function();
            let ln_function = create_ln_function();
            let converter = FunctionConverter {
                from: &log_function,
                to: &ln_function,
                is_conversion_possible_fn: |arguments| (arguments[0] - E).abs() <= 0.001,
                convert_fn: |arguments| vec![arguments[1]],
            };
            let arguments = vec![E + 0.0001, 10.0];
            assert_eq!(
                ConvertOutputData {
                    function: &ln_function,
                    arguments: vec![10.0]
                },
                converter.convert(&arguments)
            );
        }

        fn create_sin_function() -> Function {
            Function {
                name: String::from("sin"),
                arguments_number: 1,
                complexity: 1,
                io_only: false,
                compute_fn: |arguments| arguments[0].sin(),
            }
        }

        fn create_log_function() -> Function {
            Function {
                name: String::from("log"),
                arguments_number: 2,
                complexity: 1,
                io_only: false,
                compute_fn: |arguments| arguments[0].log(arguments[1]),
            }
        }

        fn create_ln_function() -> Function {
            Function {
                name: String::from("ln"),
                arguments_number: 1,
                complexity: 1,
                io_only: true,
                compute_fn: |arguments| arguments[0].ln(),
            }
        }
    }
}
