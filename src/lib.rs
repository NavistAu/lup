//! lup library: walks up the directory hierarchy looking for a literal path.

pub mod boundary;
pub mod cli;
pub mod completions;
pub mod output;
pub mod walk;

pub use walk::{lookup, Boundary, Hit, KindFilter, LupError, Query};
