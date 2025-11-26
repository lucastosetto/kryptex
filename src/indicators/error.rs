use std::fmt;

#[derive(Debug, Clone)]
pub enum IndicatorError {
    ParseError(String),
    ValidationError(String),
    MissingField(String),
    InvalidNumericFormat(String),
    OutOfRange { field: String, value: f64, min: f64, max: f64 },
    InvalidPeriod { field: String, value: u32 },
}

impl fmt::Display for IndicatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndicatorError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            IndicatorError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            IndicatorError::MissingField(field) => write!(f, "Missing required field: {}", field),
            IndicatorError::InvalidNumericFormat(value) => {
                write!(f, "Invalid numeric format: {}", value)
            }
            IndicatorError::OutOfRange { field, value, min, max } => {
                write!(
                    f,
                    "Field '{}' value {} is out of range [{}, {}]",
                    field, value, min, max
                )
            }
            IndicatorError::InvalidPeriod { field, value } => {
                write!(f, "Field '{}' has invalid period: {}", field, value)
            }
        }
    }
}

impl std::error::Error for IndicatorError {}

