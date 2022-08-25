//! Module with symbolic regression default model core functionality.
use super::super::crossing;
use super::super::fitness::{Fitness, FitnessError};
use super::super::input_data::InputData;
use super::super::settings::Settings;
use super::generation_size::GenerationSize;
use super::stop_criterion::{StopCriterion, StopReason};
use super::utils::{get_individuals_fitness, sort_individuals, IdGenerator};
use crate::expression_tree::{Computable, DefaultRandom, ExpressionTree, Random};
use rand::rngs::ThreadRng;
use std::rc::Rc;

pub struct Model<R: Random> {
    pub settings: Settings,
    pub input_data: InputData,
    pub stop_criterion: StopCriterion,
    pub generation_size: GenerationSize,
    pub auxiliary_expression_trees: Vec<ExpressionTree>,
    pub callback: Option<Box<dyn Fn(&[Rc<Individual>])>>,
    pub random: R,
    pub id_generator: Box<dyn Iterator<Item = u32>>,
}

impl<R: Random> Model<R> {
    pub fn new(
        settings: Settings,
        input_data: InputData,
        stop_criterion: StopCriterion,
        generation_size: GenerationSize,
        auxiliary_expression_trees: Vec<ExpressionTree>,
        callback: Option<Box<dyn Fn(&[Rc<Individual>])>>,
    ) -> Model<DefaultRandom<ThreadRng>> {
        Model {
            settings,
            input_data,
            stop_criterion,
            generation_size,
            auxiliary_expression_trees,
            callback,
            random: DefaultRandom(rand::thread_rng()),
            id_generator: Box::new(IdGenerator { id: 0 }),
        }
    }
    pub fn run(&mut self) -> Result<ModelResult, FitnessError> {
        let mut generation_number = 0;
        let mut without_improvement_generation_number = 0;
        let mut current_generation = self.create_first_generation()?;
        self.execute_callback(&current_generation);
        let mut error = current_generation[0].fitness.error;
        let mut stop_reason = self.stop_criterion.must_stop(
            error,
            without_improvement_generation_number,
            generation_number,
        );
        while let None = stop_reason {
            generation_number += 1;
            let next_generation =
                self.create_next_generation(&current_generation, generation_number)?;
            if self.is_next_generation_better(&current_generation, &next_generation) {
                without_improvement_generation_number = 0;
            } else {
                without_improvement_generation_number += 1;
            };
            current_generation = next_generation;
            self.execute_callback(&current_generation);
            error = current_generation[0].fitness.error;
            stop_reason = self.stop_criterion.must_stop(
                error,
                without_improvement_generation_number,
                generation_number,
            );
        }
        Ok(ModelResult {
            individual: Rc::clone(&current_generation[0]),
            stop_reason: match stop_reason {
                Some(stop_reason) => stop_reason,
                None => unreachable!(),
            },
        })
    }
    fn create_first_generation(&mut self) -> Result<Vec<Rc<Individual>>, FitnessError> {
        let initial_expression_trees = self.create_initial_expression_trees();
        let mut first_generation = self.create_individuals(initial_expression_trees, 0)?;
        sort_individuals(&mut first_generation);
        Ok(first_generation)
    }
    fn create_initial_expression_trees(&mut self) -> Vec<ExpressionTree> {
        if self.auxiliary_expression_trees.len() >= self.generation_size.generation_len as usize {
            self.auxiliary_expression_trees
                .drain(0..self.generation_size.generation_len as usize)
                .collect()
        } else {
            let mut generation = self
                .auxiliary_expression_trees
                .drain(0..self.auxiliary_expression_trees.len())
                .collect::<Vec<ExpressionTree>>();
            while generation.len() < self.generation_size.generation_len as usize {
                generation.push(ExpressionTree::create_random(
                    &mut self.random,
                    &self.settings,
                    &self.input_data.variables[0..self.input_data.variables.len() - 1],
                ));
            }
            generation
        }
    }
    fn create_next_generation(
        &mut self,
        current_generation: &[Rc<Individual>],
        generation_number: u32,
    ) -> Result<Vec<Rc<Individual>>, FitnessError> {
        let mut individuals = current_generation
            .iter()
            .cloned()
            .collect::<Vec<Rc<Individual>>>();
        let individuals_to_cross = self.select_individuals_to_cross(current_generation);
        let mut expression_trees = self.cross(&individuals_to_cross);
        if self.auxiliary_expression_trees.len() > 0 {
            expression_trees.push(self.auxiliary_expression_trees.remove(0));
        }
        individuals.append(&mut self.create_individuals(expression_trees, generation_number)?);
        sort_individuals(&mut individuals);
        individuals.drain(self.generation_size.generation_len as usize..);
        Ok(individuals)
    }
    fn select_individuals_to_cross<'a>(
        &mut self,
        individuals: &'a [Rc<Individual>],
    ) -> Vec<Rc<Individual>> {
        let adapted_number = self.generation_size.get_adapted_number();
        let unadapted_number = self.generation_size.get_unadapted_number();
        let mut individuals_to_cross = individuals
            .iter()
            .take(adapted_number)
            .cloned()
            .collect::<Vec<Rc<Individual>>>();
        while individuals_to_cross.len() < adapted_number + unadapted_number {
            individuals_to_cross.push(Rc::clone(
                &individuals[self
                    .random
                    .gen_range(adapted_number..self.generation_size.generation_len as usize)],
            ))
        }
        individuals_to_cross
    }
    fn cross(&mut self, individuals: &[Rc<Individual>]) -> Vec<ExpressionTree> {
        let mut expression_trees = vec![];
        while expression_trees.len() != individuals.len() {
            let parent1 = &individuals[self.random.gen_range(0..individuals.len())];
            let mut parent2 = &individuals[self.random.gen_range(0..individuals.len())];
            while parent2 == parent1 {
                parent2 = &individuals[self.random.gen_range(0..individuals.len())];
            }
            let mut expression_tree = crossing::cross(
                &parent1.expression_tree,
                &parent2.expression_tree,
                &mut self.random,
            );
            self.settings.mutate(&mut expression_tree, &mut self.random);
            expression_tree.simplify();
            expression_trees.push(expression_tree);
        }
        expression_trees
    }
    fn create_individuals(
        &mut self,
        expression_trees: Vec<ExpressionTree>,
        generation_number: u32,
    ) -> Result<Vec<Rc<Individual>>, FitnessError> {
        let mut individuals = vec![];
        for expression_tree in expression_trees {
            let fitness = expression_tree.get_fitness(&self.settings, &self.input_data)?;
            let defective = fitness.error.is_nan();
            individuals.push(Rc::new(Individual {
                id: self.id_generator.next().unwrap(),
                generation_number,
                expression_tree,
                fitness,
                defective,
            }));
        }
        Ok(individuals)
    }
    fn is_next_generation_better(
        &self,
        previous_generation: &[Rc<Individual>],
        next_generation: &[Rc<Individual>],
    ) -> bool {
        if let Some(without_improvement) = &self.stop_criterion.without_improvement {
            if get_individuals_fitness(previous_generation)
                - get_individuals_fitness(next_generation)
                > without_improvement.error
            {
                return true;
            }
            let adapted_number = self.generation_size.get_adapted_number();
            if get_individuals_fitness(&previous_generation[0..adapted_number])
                - get_individuals_fitness(&next_generation[0..adapted_number])
                > without_improvement.error
            {
                return true;
            }
            false
        } else {
            true
        }
    }
    fn execute_callback(&self, individuals: &[Rc<Individual>]) {
        if let Some(callback) = &self.callback {
            (callback)(individuals);
        }
    }
}

pub struct ModelResult {
    pub individual: Rc<Individual>,
    pub stop_reason: StopReason,
}

#[derive(Debug)]
pub struct Individual {
    pub id: u32,
    pub generation_number: u32,
    pub expression_tree: ExpressionTree,
    pub fitness: Fitness,
    pub defective: bool,
}

impl PartialEq for Individual {
    fn eq(&self, other: &Individual) -> bool {
        self.id == other.id
    }
}

#[cfg(test)]
mod tests {
    use super::super::stop_criterion::WithoutImprovement;
    use super::*;
    use crate::expression_tree::{MockRandom, Node, OperationNode, ValueNode};
    use calamine::{DataType, Range, Reader, Xlsx};
    use std::cmp::Ordering;
    use std::f64::NAN;
    use std::path::PathBuf;

    #[test]
    #[should_panic(expected = "Panic.")]
    fn test_execute_callback() {
        let individuals = create_test_individuals();
        let model = create_model(
            10,
            0,
            None,
            Some(Box::new(|actual_individuals| {
                assert_eq!(create_test_individuals(), actual_individuals);
                panic!("Panic.")
            })),
        );
        model.execute_callback(&individuals);
    }

    #[test]
    fn test_is_next_generation_better() {
        let model = create_model(10, 0, None, None);
        let previous_generation = create_test_individuals();
        let mut next_generation = create_test_individuals();
        assert!(!model.is_next_generation_better(&previous_generation, &next_generation));
        let mut id_generator = IdGenerator { id: 1 };
        next_generation[1] = create_individual(
            Fitness {
                error: 0.006,
                complexity: 3,
            },
            &mut id_generator,
        );
        assert!(model.is_next_generation_better(&previous_generation, &next_generation));
    }

    #[test]
    fn test_create_individuals() -> Result<(), FitnessError> {
        let settings = Settings::default();
        let mut model = create_model(10, 0, None, None);
        let mut expression_trees = create_auxiliary_expression_trees(&settings);
        expression_trees.push(create_defective_expression_tree(&settings));
        let mut expected_individuals = vec![];
        for (i, t) in [
            (
                Fitness {
                    error: 5586.5071943121839,
                    complexity: 1,
                },
                false,
            ),
            (
                Fitness {
                    error: 36.34733200046427,
                    complexity: 5,
                },
                false,
            ),
            (
                Fitness {
                    error: NAN,
                    complexity: 4,
                },
                true,
            ),
        ]
        .iter()
        .enumerate()
        {
            expected_individuals.push(Rc::new(Individual {
                id: i as u32,
                generation_number: 0,
                expression_tree: expression_trees[i].clone(),
                fitness: t.0.clone(),
                defective: t.1,
            }));
        }
        let actual_individuals = model.create_individuals(expression_trees, 0)?;
        for i in 0..3 {
            assert_eq!(expected_individuals[i].id, actual_individuals[i].id);
            assert_eq!(
                expected_individuals[i].generation_number,
                actual_individuals[i].generation_number
            );
            assert_eq!(
                expected_individuals[i].expression_tree,
                actual_individuals[i].expression_tree
            );
            assert_eq!(
                expected_individuals[i].fitness.complexity,
                actual_individuals[i].fitness.complexity
            );
            assert!([None, Some(Ordering::Equal)].contains(
                &expected_individuals[i]
                    .fitness
                    .error
                    .partial_cmp(&actual_individuals[i].fitness.error)
            ));
            assert_eq!(
                expected_individuals[i].defective,
                actual_individuals[i].defective
            );
        }
        Ok(())
    }

    #[test]
    fn test_cross() {
        let settings = Settings::default();
        let mut model = create_model(10, 0, Some(create_auxiliary_individuals_random()), None);
        let individuals = create_auxiliary_individuals(&settings, &model.input_data, 0);
        let expected_expression_trees = create_auxiliary_individuals_descendants(&settings);
        let actual_expression_trees = model.cross(&individuals);
        assert_eq!(expected_expression_trees, actual_expression_trees);
    }

    #[test]
    fn test_select_individuals_to_cross() {
        let settings = Settings::default();
        let mut model = create_model(10, 0, Some(MockRandom::new_int(vec![3])), None);
        let mut individuals = create_auxiliary_individuals(&settings, &model.input_data, 0);
        individuals.append(&mut create_auxiliary_individuals(
            &settings,
            &model.input_data,
            2,
        ));
        let expected_individuals_to_cross = vec![
            individuals[0].clone(),
            individuals[1].clone(),
            individuals[3].clone(),
        ];
        let actual_individuals_to_cross = model.select_individuals_to_cross(&individuals);
        assert_eq!(expected_individuals_to_cross, actual_individuals_to_cross);
    }

    #[test]
    fn test_create_next_generation() -> Result<(), FitnessError> {
        let settings = Settings::default();
        let mut model = create_model(10, 6, Some(create_auxiliary_individuals_random()), None);
        let mut current_generation = vec![];
        for i in 0..3 {
            current_generation.append(&mut create_auxiliary_individuals(
                &settings,
                &model.input_data,
                i * 2,
            ));
        }
        let auxiliary_individuals_descendants = create_auxiliary_individuals_descendants(&settings);
        let actual_next_generation = model.create_next_generation(&current_generation, 1)?;
        assert_eq!(10, actual_next_generation.len());
        assert_eq!(
            auxiliary_individuals_descendants[0],
            actual_next_generation[0].expression_tree
        );
        Ok(())
    }

    #[test]
    fn test_create_initial_expression_trees_not_random() {
        let settings = Settings::default();
        let mut model = create_model(2, 0, None, None);
        let expected_expression_trees = create_auxiliary_expression_trees(&settings);
        let actual_expression_trees = model.create_initial_expression_trees();
        assert_eq!(expected_expression_trees, actual_expression_trees);
    }

    #[test]
    fn test_create_initial_expression_trees_random() {
        let settings = Settings::default();
        let mut model = create_model(
            3,
            2,
            Some(MockRandom::new(
                vec![0, 0],
                vec![10.0],
                vec![0.1, 0.9, 0.4, 0.9, 0.6],
            )),
            None,
        );
        let mut expected_expression_trees = create_auxiliary_expression_trees(&settings);
        expected_expression_trees.push(ExpressionTree {
            root: Node::Operator(OperationNode {
                operation: settings.find_binary_operator_by_name("+").unwrap(),
                arguments: vec![
                    Node::Value(ValueNode::Variable(String::from("x"))),
                    Node::Value(ValueNode::Constant(10.0)),
                ],
            }),
            variables: vec![String::from("x")],
        });
        let actual_expression_trees = model.create_initial_expression_trees();
        assert_eq!(expected_expression_trees, actual_expression_trees);
    }

    #[test]
    fn test_create_first_generation() -> Result<(), FitnessError> {
        let settings = Settings::default();
        let mut model = create_model(2, 0, None, None);
        let expected_first_generation =
            create_auxiliary_individuals(&settings, &model.input_data, 0)
                .into_iter()
                .rev()
                .collect::<Vec<Rc<Individual>>>();
        let actual_first_generation = model.create_first_generation()?;
        assert_eq!(expected_first_generation, actual_first_generation);
        Ok(())
    }

    fn create_model(
        generation_len: u32,
        id: u32,
        random: Option<MockRandom>,
        callback: Option<Box<dyn Fn(&[Rc<Individual>])>>,
    ) -> Model<MockRandom> {
        let settings = Settings::default();
        let auxiliary_expression_trees = create_auxiliary_expression_trees(&settings);
        Model {
            settings,
            input_data: InputData::from_worksheet_range(get_worksheet(
                "resources/input_data_sin.xlsx",
            ))
            .unwrap(),
            stop_criterion: create_stop_criterion(),
            generation_size: GenerationSize {
                generation_len,
                adapted_percent: 0.2,
                unadapted_percent: 0.1,
            },
            auxiliary_expression_trees,
            callback,
            random: if let Some(random) = random {
                random
            } else {
                MockRandom::new_int(vec![])
            },
            id_generator: Box::new(IdGenerator { id }),
        }
    }

    fn create_stop_criterion() -> StopCriterion {
        StopCriterion::new(
            Some(0.001),
            Some(WithoutImprovement {
                error: 0.001,
                generation_number: 3,
            }),
            Some(100),
        )
    }

    fn create_defective_expression_tree(settings: &Settings) -> ExpressionTree {
        ExpressionTree {
            root: Node::Operator(OperationNode {
                operation: settings.find_binary_operator_by_name("/").unwrap(),
                arguments: vec![
                    Node::Value(ValueNode::Variable(String::from("x"))),
                    Node::Value(ValueNode::Constant(0.0)),
                ],
            }),
            variables: vec![String::from("x")],
        }
    }

    fn create_auxiliary_expression_trees(settings: &Settings) -> Vec<ExpressionTree> {
        vec![
            ExpressionTree {
                root: Node::Value(ValueNode::Variable(String::from("x"))),
                variables: vec![String::from("x")],
            },
            ExpressionTree {
                root: Node::Function(OperationNode {
                    operation: settings.find_function_by_name("sin").unwrap(),
                    arguments: vec![Node::Value(ValueNode::Constant(5.0))],
                }),
                variables: vec![String::from("x")],
            },
        ]
    }

    fn create_auxiliary_individuals(
        settings: &Settings,
        input_data: &InputData,
        id: u32,
    ) -> Vec<Rc<Individual>> {
        let expression_trees = create_auxiliary_expression_trees(settings);
        expression_trees
            .into_iter()
            .enumerate()
            .map(|(i, expression_tree)| {
                let fitness = expression_tree.get_fitness(settings, input_data).unwrap();
                Rc::new(Individual {
                    id: id + i as u32,
                    generation_number: 0,
                    expression_tree,
                    fitness,
                    defective: false,
                })
            })
            .collect()
    }

    fn create_auxiliary_individuals_descendants(settings: &Settings) -> Vec<ExpressionTree> {
        vec![
            ExpressionTree {
                root: Node::Function(OperationNode {
                    operation: settings.find_function_by_name("sin").unwrap(),
                    arguments: vec![Node::Value(ValueNode::Variable(String::from("x")))],
                }),
                variables: vec![String::from("x")],
            },
            ExpressionTree {
                root: Node::Value(ValueNode::Variable(String::from("x"))),
                variables: vec![String::from("x")],
            },
        ]
    }

    fn create_auxiliary_individuals_random() -> MockRandom {
        MockRandom::new(
            vec![1, 1, 0, 0, 1, 1, 0, 0, 1, 0, 0],
            vec![],
            vec![0.95, 0.85],
        )
    }

    fn create_individual(fitness: Fitness, id_generator: &mut IdGenerator) -> Rc<Individual> {
        let defective = fitness.error.is_nan();
        Rc::new(Individual {
            id: id_generator.next().unwrap(),
            generation_number: 0,
            expression_tree: ExpressionTree {
                root: Node::Value(ValueNode::Constant(1.0)),
                variables: vec![],
            },
            fitness,
            defective,
        })
    }

    fn create_test_individuals() -> Vec<Rc<Individual>> {
        let mut id_generator = IdGenerator { id: 0 };
        vec![
            create_individual(
                Fitness {
                    error: NAN,
                    complexity: 2,
                },
                &mut id_generator,
            ),
            create_individual(
                Fitness {
                    error: 0.01,
                    complexity: 3,
                },
                &mut id_generator,
            ),
            create_individual(
                Fitness {
                    error: 0.01,
                    complexity: 2,
                },
                &mut id_generator,
            ),
            create_individual(
                Fitness {
                    error: 0.001,
                    complexity: 3,
                },
                &mut id_generator,
            ),
        ]
    }

    fn get_worksheet(path: &str) -> Range<DataType> {
        let mut path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_buf.push(path);
        let mut workbook: Xlsx<_> = calamine::open_workbook(path_buf).unwrap();
        workbook.worksheet_range("Sheet1").unwrap().unwrap()
    }
}
