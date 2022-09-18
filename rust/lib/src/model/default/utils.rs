//! Model utils module.
use super::core::Individual;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::rc::Rc;

pub struct IdGenerator {
    pub id: u32,
}

impl Iterator for IdGenerator {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        let id = self.id;
        self.id += 1;
        Some(id)
    }
}

pub fn get_individuals_fitness(individuals: &[Rc<Individual>]) -> f64 {
    let valid_individuals = individuals
        .iter()
        .filter(|individual| !individual.defective);
    valid_individuals
        .clone()
        .map(|individual| individual.fitness.error)
        .sum::<f64>()
        / valid_individuals.count() as f64
}

pub fn sort_individuals(individuals: &mut Vec<Rc<Individual>>, complexity_impact: f32) {
    let mut points = HashMap::new();
    for individual in individuals.iter() {
        points.insert(individual.id, 0.0);
    }
    add_individual_error_points(individuals, &mut points);
    add_individual_complexity_points(individuals, &mut points, complexity_impact);
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
    complexity_impact: f32,
) {
    add_individual_points(
        individuals,
        points,
        |i1, i2| i2.fitness.complexity.cmp(&i1.fitness.complexity),
        1.0 / complexity_impact,
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

#[cfg(test)]
mod tests {
    use super::super::super::fitness::Fitness;
    use super::*;
    use crate::expression_tree::{ExpressionTree, Node, ValueNode};
    use std::f64::NAN;

    #[test]
    fn test_id_generator() {
        let mut id_generator = IdGenerator { id: 0 };
        assert_eq!(Some(0), id_generator.next());
        assert_eq!(Some(1), id_generator.next());
    }

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
        add_individual_complexity_points(&individuals, &mut points, 0.5);
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
        sort_individuals(&mut individuals, 0.5);
        assert_eq!(expected_individuals, individuals);
    }

    #[test]
    fn test_get_individuals_fitness() {
        let individuals = create_test_individuals();
        assert_eq!(0.007, get_individuals_fitness(&individuals));
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
}
