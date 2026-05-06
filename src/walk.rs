//! Type definitions for the walk algorithm. Implementation in tasks 5–10.

#[derive(Debug, Clone)]
pub struct Query<'a> {
    pub path: &'a [u8],
    pub all: bool,
    pub echo: bool,
    pub kind_filter: Option<KindFilter>,
    pub follow: bool,
    pub null_terminate: bool,
    pub relative: bool,
}

impl<'a> Query<'a> {
    pub fn new(path: &'a [u8]) -> Self {
        Self {
            path,
            all: false,
            echo: false,
            kind_filter: None,
            follow: true,
            null_terminate: false,
            relative: false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KindFilter {
    Files,
    Dirs,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Boundary {
    Home,
    Git,
    Root,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Hit {
    Path(Vec<u8>),
    Contents(Vec<u8>),
}

#[derive(Debug)]
pub enum LupError {
    NoMatch,
    NotInGitRepo,
    UsageError(String),
    Io(std::io::Error),
}

impl From<std::io::Error> for LupError {
    fn from(e: std::io::Error) -> Self {
        LupError::Io(e)
    }
}

/// Walk up from `$PWD` looking for matches.
/// Stub implementation; replaced in tasks 5–10.
pub fn lookup(_query: &Query, _boundary: Boundary) -> Result<Vec<Hit>, LupError> {
    Err(LupError::NoMatch)
}
