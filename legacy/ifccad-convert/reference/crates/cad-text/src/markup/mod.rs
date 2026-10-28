//! Strict, bounded CAD source grammar. Tokens retain scope and source spelling.
//! This leaf module has no CAD runtime or native document dependency.

mod escape;
mod lexer;
mod syntax;

pub use escape::escape_mtext_literal;
pub use lexer::{lex, lex_text};
pub use syntax::*;

#[cfg(test)]
mod tests;
