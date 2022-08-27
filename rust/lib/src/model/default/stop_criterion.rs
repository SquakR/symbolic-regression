//! Module with model stop criterion.
use serde::Deserialize;

#[derive(Debug, PartialEq)]
pub enum StopReason {
    Error(f64),
    WithoutImprovement(WithoutImprovement),
    GenerationNumber(u32),
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct WithoutImprovement {
    pub error: f64,
    pub generation_number: u32,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct StopCriterion {
    pub error: Option<f64>,
    pub without_improvement: Option<WithoutImprovement>,
    pub generation_number: Option<u32>,
}

impl StopCriterion {
    pub fn new(
        error: Option<f64>,
        without_improvement: Option<WithoutImprovement>,
        generation_number: Option<u32>,
    ) -> StopCriterion {
        assert!(
            generation_number.is_some() || without_improvement.is_some() || error.is_some(),
            "At least one stop criterion must be set."
        );
        StopCriterion {
            generation_number,
            without_improvement,
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
        if let Some(without_improvement) = &self.without_improvement {
            if without_improvement_generation_number >= without_improvement.generation_number {
                return Some(StopReason::WithoutImprovement(WithoutImprovement {
                    error,
                    generation_number: without_improvement_generation_number,
                }));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "At least one stop criterion must be set.")]
    fn test_new_panic() {
        StopCriterion::new(None, None, None);
    }

    #[test]
    fn test_new() {
        let expected_stop_criterion = StopCriterion {
            generation_number: Some(100),
            without_improvement: Some(WithoutImprovement {
                error: 0.001,
                generation_number: 3,
            }),
            error: Some(0.001),
        };
        let actual_stop_criterion = create_test_stop_criterion();
        assert_eq!(expected_stop_criterion, actual_stop_criterion);
    }

    #[test]
    fn test_must_stop_none() {
        let stop_criterion = create_test_stop_criterion();
        let expected_stop_reason = None;
        assert_eq!(expected_stop_reason, stop_criterion.must_stop(0.01, 2, 99));
    }

    #[test]
    fn test_must_stop_error() {
        let stop_criterion = create_test_stop_criterion();
        let expected_stop_reason = Some(StopReason::Error(0.0005));
        assert_eq!(
            expected_stop_reason,
            stop_criterion.must_stop(0.0005, 2, 99)
        );
    }

    #[test]
    fn test_must_stop_without_improvement() {
        let stop_criterion = create_test_stop_criterion();
        let expected_stop_reason = Some(StopReason::WithoutImprovement(WithoutImprovement {
            error: 0.01,
            generation_number: 3,
        }));
        assert_eq!(expected_stop_reason, stop_criterion.must_stop(0.01, 3, 99))
    }

    #[test]
    fn test_must_stop_generation_number() {
        let stop_criterion = create_test_stop_criterion();
        let expected_stop_reason = Some(StopReason::GenerationNumber(100));
        assert_eq!(expected_stop_reason, stop_criterion.must_stop(0.01, 2, 100));
    }

    #[test]
    fn test_must_stop_all() {
        let stop_criterion = create_test_stop_criterion();
        let expected_stop_reason = Some(StopReason::Error(0.0005));
        assert_eq!(
            expected_stop_reason,
            stop_criterion.must_stop(0.0005, 3, 100)
        )
    }

    fn create_test_stop_criterion() -> StopCriterion {
        StopCriterion::new(
            Some(0.001),
            Some(WithoutImprovement {
                error: 0.001,
                generation_number: 3,
            }),
            Some(100),
        )
    }
}
