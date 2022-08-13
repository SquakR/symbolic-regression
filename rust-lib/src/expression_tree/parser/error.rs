//! Parser error module.
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    MissingCommaOrOpeningBracketError(MissingCommaOrOpeningBracketError),
    MissingCommaError(MissingCommaError),
    InvalidArgumentsNumberError(InvalidArgumentsNumberError),
    EmptyFormulaError,
    MultipleFormulaError,
}

#[derive(Debug, PartialEq)]
pub struct MissingCommaOrOpeningBracketError {
    pub data: ErrorTokenData,
}

#[derive(Debug, PartialEq)]
pub struct MissingCommaError {
    pub data: ErrorTokenData,
}

#[derive(Debug, PartialEq)]
pub struct InvalidArgumentsNumberError {
    pub data: ErrorTokenData,
    pub expected: usize,
    pub actual: usize,
}

#[derive(Debug, PartialEq)]
pub struct ErrorTokenData {
    pub string: String,
    pub position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::MissingCommaOrOpeningBracketError(err) =>
                write!(
                    f,
                    "Missing comma or opening bracket at position {}. The token string is \"{}\".",
                    err.data.position, err.data.string
                ),
            ParseError::MissingCommaError(err) =>
                write!(
                    f,
                    "Missing comma error at position {}. The token string is \"{}\".",
                    err.data.position, err.data.string
                ),
            ParseError::InvalidArgumentsNumberError(err) => write!(
                f,
                "Invalid number of arguments at position {}, expected {}, but actually {}. Token string is \"{}\".",
                err.data.position, err.expected, err.actual, err.data.string
            ),
            ParseError::EmptyFormulaError => write!(f, "The formula is empty."),
            ParseError::MultipleFormulaError => write!(f, "The formula is multiple."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        assert_eq!(
            "Missing comma or opening bracket at position 5. The token string is \"sin\".",
            format!(
                "{}",
                ParseError::MissingCommaOrOpeningBracketError(MissingCommaOrOpeningBracketError {
                    data: ErrorTokenData {
                        string: String::from("sin"),
                        position: 5
                    }
                })
            )
        );
        assert_eq!(
            "Missing comma error at position 5. The token string is \"sin\".",
            format!(
                "{}",
                ParseError::MissingCommaError(MissingCommaError {
                    data: ErrorTokenData {
                        string: String::from("sin"),
                        position: 5
                    }
                })
            )
        );
        assert_eq!(
            "Invalid number of arguments at position 5, expected 2, but actually 1. Token string is \"log\".",
            format!("{}", ParseError::InvalidArgumentsNumberError(InvalidArgumentsNumberError {
                data: ErrorTokenData {
                    string: String::from("log"),
                    position: 5,
                },
                expected: 2,
                actual: 1
            }))
        );
        assert_eq!(
            "The formula is empty.",
            format!("{}", ParseError::EmptyFormulaError)
        );
        assert_eq!(
            "The formula is multiple.",
            format!("{}", ParseError::MultipleFormulaError)
        );
    }
}
