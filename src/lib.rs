//! Rust source parsing and the RustLight syntax model.

pub mod rustlight_parser;

pub use rustlight_parser::{parse_rust_source, parse_rust_type_facts, ParseError};
pub use rustlightast_model::*;
