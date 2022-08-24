//! Module with symbolic regression model.
use super::crossing;
use super::fitness::{Fitness, FitnessError};
use super::input_data::InputData;
use super::settings::Settings;
use crate::expression_tree::{DefaultRandom, ExpressionTree, Random};
use rand::rngs::ThreadRng;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Model<R: Random, C: Fn(&[Rc<Individual>])> {
    pub settings: Settings,
    pub input_data: InputData,
    pub stop_criterion: StopCriterion,
    pub generation_len: u32,
    pub auxiliary_expression_trees: Vec<ExpressionTree>,
    pub callback: C,
    pub random: R,
    pub id_generator: Box<dyn Iterator<Item = u32>>,
}

impl<R: Random, C: Fn(&[Rc<Individual>])> Model<R, C> {
    pub fn new(
        settings: Settings,
        input_data: InputData,
        stop_criterion: StopCriterion,
        generation_len: u32,
        auxiliary_expression_trees: Vec<ExpressionTree>,
        callback: C,
    ) -> Model<DefaultRandom<ThreadRng>, C> {
        Model {
            settings,
            input_data,
            stop_criterion,
            generation_len,
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
        (self.callback)(&current_generation);
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
            (self.callback)(&current_generation);
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
        if self.auxiliary_expression_trees.len() >= self.generation_len as usize {
            self.auxiliary_expression_trees
                .drain(0..self.generation_len as usize)
                .collect()
        } else {
            let mut generation = self
                .auxiliary_expression_trees
                .drain(0..self.auxiliary_expression_trees.len())
                .collect::<Vec<ExpressionTree>>();
            while generation.len() < self.generation_len as usize {
                generation.push(ExpressionTree::create_random(
                    &mut self.random,
                    &self.settings,
                    &self.input_data.variables,
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
        individuals.drain(self.generation_len as usize..);
        Ok(individuals)
    }
    fn select_individuals_to_cross<'a>(
        &mut self,
        individuals: &'a [Rc<Individual>],
    ) -> Vec<Rc<Individual>> {
        let adapted_number = self.get_adapted_number();
        let unadapted_number = self.get_unadapted_number();
        let mut individuals_to_cross = individuals
            .iter()
            .take(adapted_number)
            .cloned()
            .collect::<Vec<Rc<Individual>>>();
        while individuals_to_cross.len() < adapted_number + unadapted_number {
            individuals_to_cross.push(Rc::clone(
                &individuals[self
                    .random
                    .gen_range(adapted_number..self.generation_len as usize)],
            ))
        }
        individuals_to_cross
    }
    fn cross(&mut self, individuals: &[Rc<Individual>]) -> Vec<ExpressionTree> {
        let mut expression_trees = vec![];
        while expression_trees.len() != individuals.len() {
            let parent1 = &individuals[self.random.gen_range(0..individuals.len())];
            let parent2 = &individuals[self.random.gen_range(0..individuals.len())];
            let mut expression_tree = crossing::cross(
                &parent1.expression_tree,
                &parent2.expression_tree,
                &mut self.random,
            );
            self.settings.mutate(&mut expression_tree, &mut self.random);
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
        if get_individuals_fitness(next_generation) < get_individuals_fitness(previous_generation) {
            return true;
        }
        let adapted_number = self.get_adapted_number();
        if get_individuals_fitness(&next_generation[0..adapted_number])
            < get_individuals_fitness(&previous_generation[0..adapted_number])
        {
            return true;
        }
        false
    }
    fn get_adapted_number(&self) -> usize {
        (0.2 * self.generation_len as f32) as usize
    }
    fn get_unadapted_number(&self) -> usize {
        (0.1 * self.generation_len as f32) as usize
    }
}

fn get_individuals_fitness(individuals: &[Rc<Individual>]) -> f64 {
    let valid_individuals = individuals
        .iter()
        .filter(|individual| !individual.defective);
    valid_individuals
        .clone()
        .map(|individual| individual.fitness.error)
        .sum::<f64>()
        / valid_individuals.count() as f64
}

fn sort_individuals(individuals: &mut Vec<Rc<Individual>>) {
    let mut points = HashMap::new();
    for individual in individuals.iter() {
        points.insert(individual.id, 0.0);
    }
    add_individual_error_points(individuals, &mut points);
    add_individual_complexity_points(individuals, &mut points);
    individuals.sort_by(|i1, i2| points[&i2.id].partial_cmp(&points[&i1.id]).unwrap())
}

fn add_individual_error_points(individuals: &[Rc<Individual>], points: &mut HashMap<u32, f32>) {
    add_individual_points(
        individuals,
        points,
        |i1, i2| i2.fitness.error.partial_cmp(&i1.fitness.error).unwrap(),
        1.0,
    );
}

fn add_individual_complexity_points(
    individuals: &[Rc<Individual>],
    points: &mut HashMap<u32, f32>,
) {
    add_individual_points(
        individuals,
        points,
        |i1, i2| i2.fitness.complexity.cmp(&i1.fitness.complexity),
        2.0,
    );
}

fn add_individual_points<F>(
    individuals: &[Rc<Individual>],
    points: &mut HashMap<u32, f32>,
    sort: F,
    coefficient: f32,
) where
    F: Fn(&Rc<Individual>, &Rc<Individual>) -> Ordering,
{
    let mut auxiliary_individuals = individuals.iter().cloned().collect::<Vec<Rc<Individual>>>();
    auxiliary_individuals.sort_by(|i1, i2| {
        if i1.defective && i2.defective {
            return Ordering::Equal;
        }
        if i1.defective {
            return Ordering::Less;
        }
        if i2.defective {
            return Ordering::Greater;
        }
        sort(i1, i2)
    });
    for (i, individual) in auxiliary_individuals.iter().enumerate() {
        if !individual.defective {
            *points.get_mut(&individual.id).unwrap() += i as f32 / coefficient;
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum StopReason {
    Error(f64),
    WithoutImprovementGenerationNumber(u32),
    GenerationNumber(u32),
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
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, PartialEq)]
pub struct StopCriterion {
    pub error: Option<f64>,
    pub without_improvement_generation_number: Option<u32>,
    pub generation_number: Option<u32>,
}

impl StopCriterion {
    pub fn new(
        error: Option<f64>,
        without_improvement_generation_number: Option<u32>,
        generation_number: Option<u32>,
    ) -> StopCriterion {
        assert!(
            generation_number.is_some()
                || without_improvement_generation_number.is_some()
                || error.is_some(),
            "At least one stop criterion must be set."
        );
        StopCriterion {
            generation_number,
            without_improvement_generation_number,
            error,
        }
    }
    pub fn must_stop(
        &self,
        error: f64,
        without_improvement_generation_number: u32,
        generation_number: u32,
    ) -> Option<StopReason> {
        if let Some(err) = self.error {
            if error < err {
                return Some(StopReason::Error(error));
            }
        }
        if let Some(number) = self.without_improvement_generation_number {
            if without_improvement_generation_number >= number {
                return Some(StopReason::WithoutImprovementGenerationNumber(
                    without_improvement_generation_number,
                ));
            }
        }
        if let Some(number) = self.generation_number {
            if generation_number >= number {
                return Some(StopReason::GenerationNumber(generation_number));
            }
        }
        return None;
    }
}

struct IdGenerator {
    id: u32,
}

impl Iterator for IdGenerator {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        let id = self.id;
        self.id += 1;
        Some(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression_tree::{Node, ValueNode};
    use std::f64::NAN;

    #[test]
    fn test_id_generator() {
        let mut id_generator = IdGenerator { id: 0 };
        assert_eq!(Some(0), id_generator.next());
        assert_eq!(Some(1), id_generator.next());
    }

    mod stop_criterion_tests {
        use super::*;

        #[test]
        #[should_panic(expected = "At least one stop criterion must be set.")]
        fn test_new_error() {
            StopCriterion::new(None, None, None);
        }

        #[test]
        fn test_new() {
            let expected_stop_criterion = StopCriterion {
                generation_number: Some(100),
                without_improvement_generation_number: Some(3),
                error: Some(0.001),
            };
            let actual_stop_criterion = create_stop_criterion();
            assert_eq!(expected_stop_criterion, actual_stop_criterion);
        }

        #[test]
        fn test_must_stop_none() {
            let stop_criterion = create_stop_criterion();
            let expected_stop_reason = None;
            assert_eq!(expected_stop_reason, stop_criterion.must_stop(0.01, 2, 99));
        }

        #[test]
        fn test_must_stop_error() {
            let stop_criterion = create_stop_criterion();
            let expected_stop_reason = Some(StopReason::Error(0.0005));
            assert_eq!(
                expected_stop_reason,
                stop_criterion.must_stop(0.0005, 2, 99)
            );
        }

        #[test]
        fn test_must_stop_without_improvement_generation_number() {
            let stop_criterion = create_stop_criterion();
            let expected_stop_reason = Some(StopReason::WithoutImprovementGenerationNumber(3));
            assert_eq!(expected_stop_reason, stop_criterion.must_stop(0.01, 3, 99))
        }

        #[test]
        fn test_must_stop_generation_number() {
            let stop_criterion = create_stop_criterion();
            let expected_stop_reason = Some(StopReason::GenerationNumber(100));
            assert_eq!(expected_stop_reason, stop_criterion.must_stop(0.01, 2, 100));
        }

        #[test]
        fn test_must_stop_all() {
            let stop_criterion = create_stop_criterion();
            let expected_stop_reason = Some(StopReason::Error(0.0005));
            assert_eq!(
                expected_stop_reason,
                stop_criterion.must_stop(0.0005, 3, 100)
            )
        }
    }

    mod model_tests {
        use super::*;

        #[test]
        fn test_add_individual_error_points() {
            let individuals = create_test_individuals();
            let mut points = HashMap::new();
            for individual in individuals.iter() {
                points.insert(individual.id, 0.0);
            }
            add_individual_error_points(&individuals, &mut points);
            let expected_points = HashMap::from([(0, 0.0), (1, 1.0), (2, 2.0), (3, 3.0)]);
            assert_eq!(expected_points, points);
        }

        #[test]
        fn test_add_individual_complexity_points() {
            let individuals = create_test_individuals();
            let mut points = HashMap::new();
            for individual in individuals.iter() {
                points.insert(individual.id, 0.0);
            }
            add_individual_complexity_points(&individuals, &mut points);
            let expected_points = HashMap::from([(0, 0.0), (1, 0.5), (2, 1.5), (3, 1.0)]);
            assert_eq!(expected_points, points);
        }

        #[test]
        fn test_sort_individuals() {
            let mut individuals = create_test_individuals();
            let expected_individuals = individuals
                .iter()
                .cloned()
                .rev()
                .collect::<Vec<Rc<Individual>>>();
            sort_individuals(&mut individuals);
            assert_eq!(expected_individuals, individuals);
        }

        #[test]
        fn test_get_individuals_fitness() {
            let individuals = create_test_individuals();
            assert_eq!(0.007, get_individuals_fitness(&individuals));
        }
    }

    fn create_stop_criterion() -> StopCriterion {
        StopCriterion::new(Some(0.001), Some(3), Some(100))
    }

    fn create_stub_expression_tree() -> ExpressionTree {
        ExpressionTree {
            root: Node::Value(ValueNode::Constant(1.0)),
            variables: vec![],
        }
    }

    fn create_individual(fitness: Fitness, id_generator: &mut IdGenerator) -> Rc<Individual> {
        let defective = fitness.error.is_nan();
        Rc::new(Individual {
            id: id_generator.next().unwrap(),
            generation_number: 0,
            expression_tree: create_stub_expression_tree(),
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
}
