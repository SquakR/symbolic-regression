//! Input data module.
use serde::{Deserialize, Serialize};
use serde_json::Error as ServeJsonError;
use std::collections::{BTreeMap, HashMap};
use std::fmt;

/// Input data for error computation.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InputData {
    pub variables: Vec<String>,
    pub rows: Vec<Vec<f64>>,
}

impl InputData {
    /// Returns new InputData with constraint checks.
    pub fn new(variables: Vec<String>, rows: Vec<Vec<f64>>) -> Result<InputData, InputDataError> {
        if variables.len() < 2 {
            return Err(InputDataError {
                message: String::from("The \"InputData\" struct must contain at least two variables, of which the last variable is output one.")
            });
        }
        let mut variables_count = HashMap::new();
        for variable in &variables {
            let entry = variables_count.entry(variable).or_insert(0);
            *entry = *entry + 1
        }
        for (variable, value) in variables_count {
            if value > 1 {
                return Err(InputDataError {
                    message: format!("The variable \"{}\" occurs {} times.", variable, value),
                });
            }
        }
        for (i, row) in rows.iter().enumerate() {
            if row.len() < variables.len() {
                return Err(InputDataError {
                    message: format!(
                        "The row at index {} contains {} values, but must contain {}.",
                        i,
                        row.len(),
                        variables.len()
                    ),
                });
            }
        }
        Ok(InputData { variables, rows })
    }
    /// Crete new InputData from json string.
    pub fn from_json(json: &str) -> Result<InputData, FromJsonError> {
        if let Ok(input_data) = serde_json::from_str::<InputData>(json) {
            match InputData::new(input_data.variables, input_data.rows) {
                Ok(input_data) => return Ok(input_data),
                Err(err) => return Err(FromJsonError::InputDataError(err)),
            }
        };
        match serde_json::from_str::<BTreeMap<String, Vec<f64>>>(json) {
            Ok(input_data) => {
                let max_len = input_data
                    .values()
                    .max_by(|a, b| a.len().cmp(&b.len()))
                    .unwrap()
                    .len();
                for (variable, value) in &input_data {
                    if value.len() < max_len {
                        return Err(FromJsonError::InputDataError(InputDataError {
                            message: format!(
                                "The variable \"{}\" contains {} values, but must contain {}.",
                                variable,
                                value.len(),
                                max_len
                            ),
                        }));
                    }
                }
                match InputData::new(
                    input_data.keys().cloned().collect::<Vec<String>>(),
                    transpose(input_data.into_values().collect()),
                ) {
                    Ok(input_data) => return Ok(input_data),
                    Err(err) => return Err(FromJsonError::InputDataError(err)),
                }
            }
            Err(err) => Err(FromJsonError::ServeJsonError(err)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct InputDataError {
    message: String,
}

impl fmt::Display for InputDataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug)]
pub enum FromJsonError {
    ServeJsonError(ServeJsonError),
    InputDataError(InputDataError),
}

impl fmt::Display for FromJsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FromJsonError::ServeJsonError(err) => write!(f, "{}", err),
            FromJsonError::InputDataError(err) => write!(f, "{}", err),
        }
    }
}

fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>>
where
    T: Clone,
{
    assert!(!v.is_empty());
    (0..v[0].len())
        .map(|i| v.iter().map(|inner| inner[i].clone()).collect::<Vec<T>>())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid() -> Result<(), InputDataError> {
        let actual_input_data = InputData::new(
            vec![String::from("x1"), String::from("x2"), String::from("y")],
            vec![vec![1.0, 2.0, 3.0], vec![1.0, -1.0, 0.0]],
        )?;
        let expected_input_data = InputData {
            variables: vec![String::from("x1"), String::from("x2"), String::from("y")],
            rows: vec![vec![1.0, 2.0, 3.0], vec![1.0, -1.0, 0.0]],
        };
        assert_eq!(expected_input_data, actual_input_data);
        Ok(())
    }

    #[test]
    fn test_new_invalid_not_enough_variables() {
        let expected_error = InputDataError {
            message: String::from("The \"InputData\" struct must contain at least two variables, of which the last variable is output one.")
        };
        match InputData::new(vec![String::from("x")], vec![vec![1.0], vec![2.0]]) {
            Ok(input_data) => panic!(
                "Expected {:?} error, but {:?} was received.",
                expected_error, input_data
            ),
            Err(err) => assert_eq!(expected_error, err),
        };
    }

    #[test]
    fn test_new_invalid_duplicate_variable() {
        let expected_error = InputDataError {
            message: String::from("The variable \"x1\" occurs 2 times."),
        };
        match InputData::new(
            vec![String::from("x1"), String::from("x2"), String::from("x1")],
            vec![
                vec![1.0, 2.0, 3.0],
                vec![1.0, -1.0, 0.0],
                vec![3.0, 3.0, 6.0],
            ],
        ) {
            Ok(input_data) => panic!(
                "Expected {:?} error, but {:?} was received.",
                expected_error, input_data
            ),
            Err(err) => assert_eq!(expected_error, err),
        }
    }

    #[test]
    fn test_new_wrong_row() {
        let expected_error = InputDataError {
            message: String::from("The row at index 1 contains 2 values, but must contain 3."),
        };
        match InputData::new(
            vec![String::from("x1"), String::from("x2"), String::from("y")],
            vec![vec![1.0, 2.0, 3.0], vec![1.0, 0.0], vec![3.0, 3.0, 6.0]],
        ) {
            Ok(input_data) => panic!(
                "Expected {:?} error, but {:?} was received.",
                expected_error, input_data
            ),
            Err(err) => assert_eq!(expected_error, err),
        };
    }

    #[test]
    fn test_from_json_valid() -> Result<(), FromJsonError> {
        for json in [
            r#"{
                "variables": ["x1", "x2", "y"],
                "rows": [
                    [1, 2, 3],
                    [1.0, -1.0, 0]
                ]
            }"#,
            r#"{
                "x1": [1, 1.0],
                "x2": [2, -1.0],
                "y": [3, 0]
            }"#,
        ] {
            let actual_input_data = InputData::from_json(json)?;
            let expected_input_data = InputData {
                variables: vec![String::from("x1"), String::from("x2"), String::from("y")],
                rows: vec![vec![1.0, 2.0, 3.0], vec![1.0, -1.0, 0.0]],
            };
            assert_eq!(expected_input_data, actual_input_data);
        }
        Ok(())
    }

    #[test]
    fn test_from_json_input_data_form_invalid() {
        let expected_error = InputDataError {
            message: String::from("The row at index 1 contains 2 values, but must contain 3."),
        };
        match InputData::from_json(
            r#"{
                "variables": ["x1", "x2", "y"],
                "rows": [
                    [1, 2, 3],
                    [1.0, 0],
                    [3.0, 3.0, 6.0]
                ]
            }"#,
        ) {
            Ok(input_data) => panic!(
                "Expected {:?} error, but {:?} was received.",
                expected_error, input_data
            ),
            Err(err) => {
                if let FromJsonError::InputDataError(err) = &err {
                    assert_eq!(expected_error, *err);
                } else {
                    panic!(
                        "Expected {:?} error, but {:?} was received.",
                        expected_error, err
                    );
                }
            }
        };
    }

    #[test]
    fn test_from_json_variables_map_form_invalid() {
        let expected_error = InputDataError {
            message: String::from("The variable \"x2\" contains 2 values, but must contain 3."),
        };
        match InputData::from_json(
            r#"{
                "x1": [1, 1.0, 3.0],
                "x2": [2, 3.0],
                "y": [3, 0, 6.0]
            }"#,
        ) {
            Ok(input_data) => panic!(
                "Expected {:?} error, but {:?} was received.",
                expected_error, input_data
            ),
            Err(err) => {
                if let FromJsonError::InputDataError(err) = &err {
                    assert_eq!(expected_error, *err);
                } else {
                    panic!(
                        "Expected {:?} error, but {:?} was received.",
                        expected_error, err
                    );
                }
            }
        };
    }

    #[test]
    fn test_from_json_invalid_form() {
        match InputData::from_json(
            r#"{
                "x1": [1, 1.0],
                "x2": [2, -1.0],
                "y": {}
            }"#,
        ) {
            Ok(input_data) => panic!(
                "Expected \"ServeJsonError\" error, but {:?} was received.",
                input_data
            ),
            Err(err) => {
                if let FromJsonError::InputDataError(err) = err {
                    panic!(
                        "Expected \"ServeJsonError\" error, but {:?} was received.",
                        err
                    )
                }
            }
        };
    }

    #[test]
    fn test_transpose() {
        let actual_vec = transpose(vec![vec![1, 2], vec![3, 4], vec![5, 6]]);
        let expected_vec = vec![vec![1, 3, 5], vec![2, 4, 6]];
        assert_eq!(expected_vec, actual_vec);
    }
}
