//! Expression tree parser module.
//! The parser uses the shunting yard algorithm.
//! https://en.wikipedia.org/wiki/Shunting_yard_algorithm
mod error;
mod lexer;
pub mod parser;

pub use parser::*;
