//! Module with generation size.

#[derive(Debug, PartialEq)]
pub struct GenerationSize {
    pub generation_len: u32,
    pub adapted_percent: f32,
    pub unadapted_percent: f32,
}

impl GenerationSize {
    pub fn build(
        generation_len: u32,
        adapted_percent: f32,
        unadapted_percent: f32,
    ) -> Result<GenerationSize, String> {
        if adapted_percent < 0.0 || adapted_percent > 1.0 {
            return Err(String::from(format!(
                r#""adapted_percent" must be between 0.0 and 1.0, but {} was received."#,
                adapted_percent
            )));
        }
        if unadapted_percent < 0.0 || unadapted_percent > 1.0 {
            return Err(String::from(format!(
                r#""unadapted_percent" must be between 0.0 and 1.0, but {} was received."#,
                unadapted_percent
            )));
        }
        let generation_size = GenerationSize {
            generation_len,
            adapted_percent,
            unadapted_percent,
        };
        let total_number =
            generation_size.get_adapted_number() + generation_size.get_unadapted_number();
        if total_number < 2 {
            return Err(String::from(format!(
                "The number of adapted and unadapted individuals must exceed 2, but {} was received.",
                total_number
            )));
        }
        Ok(generation_size)
    }
    pub fn get_adapted_number(&self) -> usize {
        (self.adapted_percent * self.generation_len as f32) as usize
    }
    pub fn get_unadapted_number(&self) -> usize {
        (self.unadapted_percent * self.generation_len as f32) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build() -> Result<(), String> {
        let expected_generation_size = create_test_generation_size();
        let actual_generation_size = GenerationSize::build(10, 0.2, 0.1)?;
        assert_eq!(expected_generation_size, actual_generation_size);
        Ok(())
    }

    #[test]
    fn test_build_wrong_adapted_percent() {
        let expected_error = String::from(
            r#""adapted_percent" must be between 0.0 and 1.0, but -0.5 was received."#,
        );
        match GenerationSize::build(5, -0.5, 0.1) {
            Ok(_) => panic!(
                r#"Expected "{:?}" error, but Ok(()) was received."#,
                expected_error
            ),
            Err(actual_error) => assert_eq!(expected_error, actual_error),
        }
    }

    #[test]
    fn test_build_wrong_unadapted_percent() {
        let expected_error = String::from(
            r#""unadapted_percent" must be between 0.0 and 1.0, but 1.5 was received."#,
        );
        match GenerationSize::build(5, 0.1, 1.5) {
            Ok(_) => panic!(
                r#"Expected "{:?}" error, but Ok(()) was received."#,
                expected_error
            ),
            Err(actual_error) => assert_eq!(expected_error, actual_error),
        }
    }

    #[test]
    fn test_build_wrong_total_number() {
        let expected_error = String::from(
            "The number of adapted and unadapted individuals must exceed 2, but 1 was received.",
        );
        match GenerationSize::build(5, 0.2, 0.1) {
            Ok(_) => panic!(
                r#"Expected "{:?}" error, but Ok(()) was received."#,
                expected_error
            ),
            Err(actual_error) => assert_eq!(expected_error, actual_error),
        }
    }

    #[test]
    fn test_get_adapted_number() {
        let generation_size = create_test_generation_size();
        assert_eq!(2, generation_size.get_adapted_number());
    }

    #[test]
    fn test_get_unadapted_number() {
        let generation_size = create_test_generation_size();
        assert_eq!(1, generation_size.get_unadapted_number());
    }

    fn create_test_generation_size() -> GenerationSize {
        GenerationSize {
            generation_len: 10,
            adapted_percent: 0.2,
            unadapted_percent: 0.1,
        }
    }
}
