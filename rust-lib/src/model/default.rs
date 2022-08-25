//! Module with default symbolic regression model.
pub mod core;
pub mod stop_criterion;
mod utils;

pub use self::core::*;
pub use stop_criterion::*;
