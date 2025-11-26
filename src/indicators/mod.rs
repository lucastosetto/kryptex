pub mod error;
pub mod parser;
pub mod validation;
pub mod registry;

pub mod momentum;
pub mod trend;
pub mod volatility;
pub mod structure;

pub use error::IndicatorError;
pub use parser::*;
pub use validation::*;
pub use registry::*;
