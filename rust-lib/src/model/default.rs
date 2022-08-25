//! Module with default symbolic regression model.
pub mod core;
pub mod generation_size;
pub mod stop_criterion;
mod utils;

pub use self::core::*;
pub use generation_size::*;
pub use stop_criterion::*;
