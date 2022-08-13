//! Input data module.

/// Input data for error computation.
#[derive(Debug, PartialEq)]
pub struct InputData {
    pub variables: Vec<String>,
    pub rows: Vec<Vec<f64>>,
}

impl InputData {
    /// Returns new InputData with constraint checks.
    pub fn new(variables: Vec<String>, rows: Vec<Vec<f64>>) -> Result<InputData, String> {
        if variables.len() < 2 {
            return Err(String::from("The `InputData` struct must contain at least two variables, of which the last variable is output one."));
        }
        for (i, row) in rows.iter().enumerate() {
            if row.len() < variables.len() {
                return Err(format!(
                    "The row at index {} contains {} values, but must contain {}.",
                    i,
                    row.len(),
                    variables.len()
                ));
            }
        }
        Ok(InputData { variables, rows })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid() -> Result<(), String> {
        let input_data = InputData::new(
            vec![String::from("x1"), String::from("x2"), String::from("y")],
            vec![vec![1.0, 2.0, 3.0], vec![1.0, -1.0, 0.0]],
        )?;
        assert_eq!(
            vec![String::from("x1"), String::from("x2"), String::from("y")],
            input_data.variables
        );
        assert_eq!(
            vec![vec![1.0, 2.0, 3.0], vec![1.0, -1.0, 0.0]],
            input_data.rows
        );
        Ok(())
    }

    #[test]
    fn test_new_invalid_not_enough_variables() {
        let expected_error = "The `InputData` struct must contain at least two variables, of which the last variable is output one.";
        match InputData::new(vec![String::from("x")], vec![vec![1.0], vec![2.0]]) {
            Ok(input_data) => panic!(
                "Expected \"{:?}\" error message, but {:?} was received.",
                expected_error, input_data
            ),
            Err(err) => assert_eq!(expected_error, err),
        };
    }

    #[test]
    fn test_new_wrong_row() {
        let expected_error = "The row at index 1 contains 2 values, but must contain 3.";
        match InputData::new(
            vec![String::from("x1"), String::from("x2"), String::from("y")],
            vec![vec![1.0, 2.0, 3.0], vec![1.0, 0.0], vec![3.0, 3.0, 6.0]],
        ) {
            Ok(input_data) => panic!(
                "Expected \"{:?}\" error message, but {:?} was received.",
                expected_error, input_data
            ),
            Err(err) => assert_eq!(expected_error, err),
        };
    }
}
