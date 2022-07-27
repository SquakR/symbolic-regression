/// Module with common types.
use std::cmp::PartialEq;
use std::fmt;

#[derive(Clone)]
pub struct Operator {
    pub string: String,
    pub precedence: u8,
    pub associativity: Associativity,
    pub arguments_number: usize,
    pub complexity: u32,
    pub io_only: bool,
    pub compute_fn: fn(&[f64]) -> f64,
}

impl Operator {
    pub fn compute(&self, arguments: &[f64]) -> f64 {
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
}

impl fmt::Debug for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Operator")
            .field("string", &self.string)
            .field("arguments_number", &self.arguments_number)
            .field("precedence", &self.precedence)
            .field("associativity", &self.associativity)
            .field("complexity", &self.complexity)
            .field("io_only", &self.io_only)
            .finish()
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl PartialEq for Operator {
    fn eq(&self, other: &Operator) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl PartialEq<String> for Operator {
    fn eq(&self, other: &String) -> bool {
        return self.string.to_lowercase() == other.to_lowercase();
    }
}

#[derive(Clone)]
pub struct Function {
    pub string: String,
    pub arguments_number: usize,
    pub complexity: u32,
    pub io_only: bool,
    pub compute_fn: fn(&[f64]) -> f64,
}

impl Function {
    pub fn compute(&self, arguments: &[f64]) -> f64 {
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
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Function")
            .field("string", &self.string)
            .field("arguments_number", &self.arguments_number)
            .field("complexity", &self.complexity)
            .field("io_only", &self.io_only)
            .finish()
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Function) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl PartialEq<String> for Function {
    fn eq(&self, other: &String) -> bool {
        return self.string.to_lowercase() == other.to_lowercase();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Variable(String),
    Constant(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Associativity {
    Left,
    Right,
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
                "Operator { string: \"+\", arguments_number: 2, precedence: 1, associativity: Left, complexity: 1, io_only: false }",
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
            test_operator2.string = String::from("-");
            assert!(test_operator1 != test_operator2);
        }

        #[test]
        fn test_str_eq() {
            let test_operator = create_test_operator();
            assert!(test_operator == String::from("+"));
            assert!(test_operator != String::from("-"));
        }

        #[test]
        fn test_compute() {
            let test_operator = create_test_operator();
            assert_eq!(3.0, test_operator.compute(&[1.0, 2.0]));
        }

        fn create_test_operator() -> Operator {
            Operator {
                string: String::from("+"),
                arguments_number: 2,
                precedence: 1,
                associativity: Associativity::Left,
                complexity: 1,
                io_only: false,
                compute_fn: |arguments| arguments[0] + arguments[1],
            }
        }
    }

    mod function_tests {
        use super::*;

        #[test]
        fn test_debug() {
            let test_function = create_test_function();
            assert_eq!(
                "Function { string: \"sin\", arguments_number: 1, complexity: 1, io_only: false }",
                format!("{:?}", test_function)
            );
        }

        #[test]
        fn test_display() {
            let test_function = create_test_function();
            assert_eq!("sin", format!("{}", test_function));
        }

        #[test]
        fn test_eq() {
            let test_function1 = create_test_function();
            let mut test_function2 = create_test_function();
            assert!(test_function1 == test_function2);
            test_function2.string = String::from("cos");
            assert!(test_function1 != test_function2);
        }

        #[test]
        fn test_str_eq() {
            let test_function = create_test_function();
            assert!(test_function == String::from("sin"));
            assert!(test_function == String::from("SiN"));
            assert!(test_function != String::from("cos"));
        }

        #[test]
        fn test_compute() {
            let test_function = create_test_function();
            assert_eq!(2.0_f64.sin(), test_function.compute(&[2.0]));
        }

        #[test]
        #[should_panic(expected = "The function `sin` expected 1 arguments but received 2.")]
        fn test_compute_panic() {
            let test_function = create_test_function();
            assert_eq!(2.0_f64.sin(), test_function.compute(&[1.0, 2.0]));
        }

        fn create_test_function() -> Function {
            Function {
                string: String::from("sin"),
                arguments_number: 1,
                complexity: 1,
                io_only: false,
                compute_fn: |arguments| arguments[0].sin(),
            }
        }
    }
}
