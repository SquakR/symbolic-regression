//! Settings core functionality module.
use super::types::{ConvertOutputData, Converter, ConverterOperation};
use crate::expression_tree::random::Random;
use crate::expression_tree::{ExpressionTree, Function, Node, Operation, Operator};
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct NodeProbability {
    pub operator_node: f64,
    pub function_node: f64,
    pub value_node: f64,
}

pub struct Mutation {
    pub mutation_fn: Box<dyn Fn(&mut ExpressionTree, &mut dyn Random, &Settings) -> bool>,
    pub probability: f64,
}

impl Mutation {
    pub fn mutate<R>(
        &self,
        expression_tree: &mut ExpressionTree,
        random: &mut R,
        settings: &Settings,
    ) -> bool
    where
        R: Random,
    {
        (self.mutation_fn)(expression_tree, random, settings)
    }
}

pub struct Settings {
    pub operators: Vec<Rc<Operator>>,
    pub functions: Vec<Rc<Function>>,
    pub converters: Vec<Converter>,
    pub variable_complexity: u32,
    pub constant_complexity: u32,
    pub complexity_impact: f32,
    pub get_node_probability_fn: Box<dyn Fn(u32) -> NodeProbability>,
    pub mutations: Vec<Mutation>,
}

impl Settings {
    pub fn get_node_probability(&self, tree_complexity: u32) -> NodeProbability {
        (self.get_node_probability_fn)(tree_complexity)
    }
    pub fn find_function_by_name(&self, name: &str) -> Option<Rc<Function>> {
        for function in &self.functions {
            if function.get_name() == name {
                return Some(Rc::clone(function));
            }
        }
        None
    }
    pub fn find_unary_operator_by_name(&self, name: &str) -> Option<Rc<Operator>> {
        for operator in &self.operators {
            if operator.get_name() == name && operator.arguments_number == 1 {
                return Some(Rc::clone(operator));
            }
        }
        None
    }
    pub fn find_binary_operator_by_name(&self, name: &str) -> Option<Rc<Operator>> {
        for operator in &self.operators {
            if operator.get_name() == name && operator.arguments_number == 2 {
                return Some(Rc::clone(operator));
            }
        }
        None
    }
    pub fn find_converters(&self, operation: &ConverterOperation) -> Vec<&Converter> {
        let mut converters = vec![];
        for converter in &self.converters {
            if converter.from == *operation {
                converters.push(converter)
            }
        }
        converters
    }
    pub fn convert(
        &self,
        operation: ConverterOperation,
        arguments: Vec<Node>,
    ) -> ConvertOutputData {
        for converter in self.find_converters(&operation) {
            if converter.is_conversion_possible(&operation, &arguments) {
                return converter.convert(arguments);
            }
        }
        ConvertOutputData {
            operation,
            arguments,
        }
    }
    pub fn mutate<R>(&self, expression_tree: &mut ExpressionTree, random: &mut R)
    where
        R: Random,
    {
        let random_probability = random.gen_float_standard();
        let mut probability = 0.0;
        let mut executed = true;
        let mut performed = false;
        while executed && !performed {
            executed = false;
            for mutation in &self.mutations {
                probability += mutation.probability;
                if random_probability < probability {
                    executed = true;
                    performed = mutation.mutate(expression_tree, random, self);
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression_tree::random::MockRandom;
    use crate::expression_tree::ValueNode;
    use std::f64::consts::E;

    #[test]
    fn test_find_unary_operator_by_name() {
        let settings = Settings::default();
        assert_eq!(
            settings.operators[5],
            settings.find_unary_operator_by_name("+").unwrap()
        );
        assert_eq!(None, settings.find_unary_operator_by_name("&"));
    }

    #[test]
    fn test_find_binary_operator_by_name() {
        let settings = Settings::default();
        assert_eq!(
            settings.operators[0],
            settings.find_binary_operator_by_name("+").unwrap()
        );
        assert_eq!(None, settings.find_binary_operator_by_name("&"));
    }

    #[test]
    fn test_find_function_by_name() {
        let settings = Settings::default();
        assert_eq!(
            settings.functions[0],
            settings.find_function_by_name("abs").unwrap()
        );
        assert_eq!(None, settings.find_function_by_name("fn"));
    }

    #[test]
    fn test_find_converters() {
        let settings = Settings::default();
        assert_eq!(
            vec![&settings.converters[4], &settings.converters[5]],
            settings.find_converters(&ConverterOperation::Operator(
                settings.find_binary_operator_by_name("^").unwrap()
            ))
        );
        let empty_vec: Vec<&Converter> = vec![];
        assert_eq!(
            empty_vec,
            settings.find_converters(&ConverterOperation::Operator(
                settings.find_binary_operator_by_name("+").unwrap()
            ))
        );
    }

    #[test]
    fn test_conversion_is_not_possible() {
        let settings = Settings::default();
        let log = settings.find_function_by_name("log").unwrap();
        let expected_output_data = ConvertOutputData {
            operation: ConverterOperation::Function(Rc::clone(&log)),
            arguments: vec![
                Node::Value(ValueNode::Variable(String::from("x"))),
                Node::Value(ValueNode::Constant(10.0)),
            ],
        };
        let actual_output_data = settings.convert(
            ConverterOperation::Function(Rc::clone(&log)),
            vec![
                Node::Value(ValueNode::Variable(String::from("x"))),
                Node::Value(ValueNode::Constant(10.0)),
            ],
        );
        assert_eq!(expected_output_data, actual_output_data);
    }

    #[test]
    fn test_convert_log_to_ln() {
        let settings = Settings::default();
        let expected_output_data = ConvertOutputData {
            operation: ConverterOperation::Function(settings.find_function_by_name("ln").unwrap()),
            arguments: vec![Node::Value(ValueNode::Variable(String::from("x")))],
        };
        let actual_output_data = settings.convert(
            ConverterOperation::Function(settings.find_function_by_name("log").unwrap()),
            vec![
                Node::Value(ValueNode::Constant(E + 0.0001)),
                Node::Value(ValueNode::Variable(String::from("x"))),
            ],
        );
        assert_eq!(expected_output_data, actual_output_data);
    }

    #[test]
    fn test_convert_pow_to_sqrt() {
        let settings = Settings::default();
        let expected_output_data = ConvertOutputData {
            operation: ConverterOperation::Operator(
                settings.find_binary_operator_by_name("^").unwrap(),
            ),
            arguments: vec![
                Node::Value(ValueNode::Constant(2.0)),
                Node::Value(ValueNode::Constant(0.5)),
            ],
        };
        let actual_output_data = settings.convert(
            ConverterOperation::Function(settings.find_function_by_name("sqrt").unwrap()),
            vec![Node::Value(ValueNode::Constant(2.0))],
        );
        assert_eq!(expected_output_data, actual_output_data);
    }

    #[test]
    #[should_panic(expected = "Mutation number 2.")]
    fn test_mutate() {
        let mut settings = Settings::default();
        let mut expression_tree = ExpressionTree {
            root: Node::Value(ValueNode::Constant(5.0)),
            variables: vec![],
        };
        let expression_tree_clone = expression_tree.clone();
        let mut random = MockRandom::new(vec![], vec![], vec![0.45]);
        settings.mutations = vec![
            Mutation {
                mutation_fn: Box::new(|_, _, _| panic!("Mutation number 1.")),
                probability: 0.2,
            },
            Mutation {
                mutation_fn: Box::new(move |actual_expression_tree, _, _| {
                    assert_eq!(expression_tree_clone, *actual_expression_tree);
                    panic!("Mutation number 2.");
                }),
                probability: 0.3,
            },
            Mutation {
                mutation_fn: Box::new(|_, _, _| panic!("Mutation number 3.")),
                probability: 0.3,
            },
        ];
        settings.mutate(&mut expression_tree, &mut random);
    }
}
