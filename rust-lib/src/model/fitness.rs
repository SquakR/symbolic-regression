//! Expression tree fitness module.
use super::input_data::InputData;
use super::settings::Settings;
use crate::expression_tree::{
    Computable, ComputeError, ExpressionTree, Node, Operation, OperationNode, SubsError, ValueNode,
};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Fitness {
    /// The sum of squared differences between actual and computed values.
    pub error: f64,
    /// The complexity of an expression tree which is the sum of the complexity of expression tree nodes.
    pub complexity: u32,
}

impl ExpressionTree {
    pub fn get_fitness(
        &self,
        settings: &Settings,
        input_data: &InputData,
    ) -> Result<Fitness, FitnessError> {
        Ok(Fitness {
            error: self.get_error(input_data)?,
            complexity: self.get_complexity(settings),
        })
    }
    pub fn get_error(&self, input_data: &InputData) -> Result<f64, FitnessError> {
        let mut error = 0.0;
        for row in &input_data.rows {
            let mut variables = HashMap::new();
            for (variable, value) in input_data
                .variables
                .iter()
                .map(|variable| variable.as_str())
                .zip(row.iter().cloned())
                .take(input_data.variables.len() - 1)
            {
                variables.insert(variable, value);
            }
            let subs_tree = match self.subs(&variables) {
                Ok(subs_tree) => subs_tree,
                Err(err) => return Err(FitnessError::SubsError(err)),
            };
            let compute_result = match subs_tree.compute() {
                Ok(compute_result) => compute_result,
                Err(err) => return Err(FitnessError::ComputeError(err)),
            };
            error += (compute_result - row[row.len() - 1]).powf(2.0);
        }
        Ok(error)
    }
    pub fn get_complexity(&self, settings: &Settings) -> u32 {
        self.root.get_complexity(settings)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FitnessError {
    ComputeError(ComputeError),
    SubsError(SubsError),
}

impl Node {
    pub fn get_complexity(&self, settings: &Settings) -> u32 {
        match self {
            Node::Operator(operator_node) => operator_node.get_complexity(settings),
            Node::Function(function_node) => function_node.get_complexity(settings),
            Node::Value(value_node) => value_node.get_complexity(settings),
        }
    }
}

impl<T: Operation> OperationNode<T> {
    pub fn get_complexity(&self, settings: &Settings) -> u32 {
        self.operation.get_complexity()
            + self
                .arguments
                .iter()
                .map(|argument| argument.get_complexity(settings))
                .sum::<u32>()
    }
}

impl ValueNode {
    pub fn get_complexity(&self, settings: &Settings) -> u32 {
        match self {
            ValueNode::Variable(_) => settings.variable_complexity,
            ValueNode::Constant(_) => settings.constant_complexity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::settings::Settings;

    #[test]
    fn test_value_node_get_complexity() {
        let settings = create_settings();
        assert_eq!(
            2,
            ValueNode::Variable(String::from("x")).get_complexity(&settings)
        );
        assert_eq!(1, ValueNode::Constant(1.0).get_complexity(&settings));
    }

    #[test]
    fn test_operation_node_get_complexity() {
        let settings = create_settings();
        let operation_node = OperationNode {
            operation: settings.find_binary_operator_by_name("+").unwrap(),
            arguments: vec![
                Node::Value(ValueNode::Variable(String::from("x"))),
                Node::Value(ValueNode::Constant(1.0)),
            ],
        };
        assert_eq!(4, operation_node.get_complexity(&settings));
    }

    #[test]
    fn test_node_get_complexity() {
        let settings = create_settings();
        let node = Node::Function(OperationNode {
            operation: settings.find_function_by_name("log").unwrap(),
            arguments: vec![
                Node::Value(ValueNode::Variable(String::from("x"))),
                Node::Value(ValueNode::Constant(1.0)),
            ],
        });
        assert_eq!(7, node.get_complexity(&settings));
    }

    #[test]
    fn test_expression_tree_get_complexity() {
        let settings = create_settings();
        let expression_tree = ExpressionTree {
            root: Node::Function(OperationNode {
                operation: settings.find_function_by_name("log").unwrap(),
                arguments: vec![
                    Node::Value(ValueNode::Variable(String::from("x"))),
                    Node::Value(ValueNode::Constant(1.0)),
                ],
            }),
            variables: vec![String::from("x")],
        };
        assert_eq!(7, expression_tree.get_complexity(&settings));
    }

    #[test]
    fn test_expression_tree_get_error() -> Result<(), FitnessError> {
        let settings = create_settings();
        let expression_tree = create_expression_tree_to_get_fitness(&settings);
        let input_data = create_input_data_to_get_fitness();
        let expected_error = 0.75;
        let actual_error = expression_tree.get_error(&input_data)?;
        assert_eq!(expected_error, actual_error);
        Ok(())
    }

    #[test]
    fn test_expression_tree_get_error_subs_error() {
        let settings = create_settings();
        let expression_tree = create_expression_tree_to_get_fitness(&settings);
        let mut input_data = create_input_data_to_get_fitness();
        input_data.variables[0] = String::from("z");
        let expected_error = FitnessError::SubsError(SubsError {
            message: String::from(r#"Expression tree does not contain "z" variable."#),
        });
        match expression_tree.get_fitness(&settings, &input_data) {
            Ok(fitness) => panic!(
                "Expected {:?}, but {:?} was received.",
                expected_error, fitness
            ),
            Err(err) => assert_eq!(expected_error, err),
        };
    }

    #[test]
    fn test_expression_tree_get_error_compute_error() {
        let settings = create_settings();
        let expression_tree = create_expression_tree_to_get_fitness(&settings);
        let mut input_data = create_input_data_to_get_fitness();
        input_data.variables.remove(0);
        for row in &mut input_data.rows {
            row.remove(0);
        }
        let expected_error = FitnessError::ComputeError(ComputeError {
            message: String::from(r#"The "x1" variable is not a constant."#),
        });
        match expression_tree.get_fitness(&settings, &input_data) {
            Ok(fitness) => panic!(
                "Expected {:?}, but {:?} was received.",
                expected_error, fitness
            ),
            Err(err) => assert_eq!(expected_error, err),
        };
    }

    #[test]
    fn test_expression_tree_get_fitness() -> Result<(), FitnessError> {
        let settings = create_settings();
        let expression_tree = create_expression_tree_to_get_fitness(&settings);
        let input_data = create_input_data_to_get_fitness();
        let expected_fitness = Fitness {
            error: 0.75,
            complexity: 7,
        };
        let actual_fitness = expression_tree.get_fitness(&settings, &input_data)?;
        assert_eq!(expected_fitness, actual_fitness);
        Ok(())
    }

    fn create_settings() -> Settings {
        let mut settings = Settings::default();
        settings.variable_complexity = 2;
        settings.constant_complexity = 1;
        settings
    }

    fn create_expression_tree_to_get_fitness(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Operator(OperationNode {
                operation: settings.find_binary_operator_by_name("+").unwrap(),
                arguments: vec![
                    Node::Value(ValueNode::Constant(0.5)),
                    Node::Operator(OperationNode {
                        operation: settings.find_binary_operator_by_name("+").unwrap(),
                        arguments: vec![
                            Node::Value(ValueNode::Variable(String::from("x1"))),
                            Node::Value(ValueNode::Variable(String::from("x2"))),
                        ],
                    }),
                ],
            }),
            variables: vec![String::from("x1"), String::from("x2")],
        }
    }

    fn create_input_data_to_get_fitness() -> InputData {
        InputData::build(
            vec![String::from("x1"), String::from("x2"), String::from("y")],
            vec![
                vec![1.0, 2.0, 3.0],
                vec![1.0, -1.0, 0.0],
                vec![3.0, 3.0, 6.0],
            ],
        )
        .unwrap()
    }
}
