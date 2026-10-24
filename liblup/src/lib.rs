//! lup library: walks up the directory hierarchy looking for a literal path.
//!
//! `lookup` takes the start directory, so it walks from any directory without
//! `chdir`:
//!
//! ```no_run
//! use liblup::{lookup, Boundary, Hit, Query};
//!
//! let hits = lookup(&Query::new(b".env"), b"/work/app/src", Boundary::Home(Some(b"/work")))?;
//! if let Some(Hit::Path(path)) = hits.first() {
//!     println!("{}", String::from_utf8_lossy(path));
//! }
//! # Ok::<(), liblup::LupError>(())
//! ```
//!
//! The `lup` command is the `lup-cli` crate.

pub mod boundary;
pub mod walk;

pub use walk::{lookup, Boundary, Hit, KindFilter, LupError, Query};
