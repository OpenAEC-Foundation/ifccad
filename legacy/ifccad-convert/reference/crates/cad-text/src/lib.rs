//! CAD-specific text preparation for independent drawing model adapters.
//! This library owns no drawing identities, ownership, file IO or loss policy.

mod content;
mod diagnostics;
mod emit;
mod layout;
pub mod markup;
mod paragraph;
mod placement;
mod styles;

pub use content::{parse_mtext, parse_text, MarkupColor, ParsedMTextContent};
pub use diagnostics::*;
pub use emit::{emit_mtext, emit_text, CadTextProfile, MTextMarkupInput};
pub use placement::*;
pub use styles::*;
