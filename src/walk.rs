//! Walk algorithm and core types.

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::ffi::OsStringExt;

use crate::boundary;

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

pub fn lookup(query: &Query, boundary: Boundary) -> Result<Vec<Hit>, LupError> {
    let pwd_os = std::env::current_dir()?.into_os_string();
    let pwd_bytes = pwd_os.as_bytes().to_vec();

    let home_bytes = home_dir_bytes();
    let boundary_bytes: Vec<u8> = match boundary {
        Boundary::Home => match home_bytes.as_deref() {
            Some(h) => boundary::resolve_default(&pwd_bytes, h).to_vec(),
            None => b"/".to_vec(),
        },
        Boundary::Root => b"/".to_vec(),
        Boundary::Git => boundary::find_git_root(&pwd_bytes).ok_or(LupError::NotInGitRepo)?,
    };

    let query_cstr = CString::new(query.path)
        .map_err(|_| LupError::UsageError("query contains NUL byte".into()))?;
    let mut hits: Vec<Hit> = Vec::new();
    let mut current = pwd_bytes;

    let mut dir_fd = unsafe {
        let dot = b".\0";
        libc::openat(
            libc::AT_FDCWD,
            dot.as_ptr().cast::<libc::c_char>(),
            libc::O_DIRECTORY | libc::O_RDONLY,
        )
    };
    if dir_fd < 0 {
        return Err(LupError::Io(std::io::Error::last_os_error()));
    }

    loop {
        let access = unsafe { libc::faccessat(dir_fd, query_cstr.as_ptr(), libc::F_OK, 0) };
        if access == 0 {
            let mut hit_path = current.clone();
            push_component(&mut hit_path, query.path);
            hits.push(Hit::Path(hit_path));
            if !query.all {
                unsafe {
                    libc::close(dir_fd);
                }
                return Ok(hits);
            }
        }

        if current == boundary_bytes {
            unsafe {
                libc::close(dir_fd);
            }
            break;
        }

        let parent_fd = unsafe {
            let dotdot = b"..\0";
            libc::openat(
                dir_fd,
                dotdot.as_ptr().cast::<libc::c_char>(),
                libc::O_DIRECTORY | libc::O_RDONLY,
            )
        };
        unsafe {
            libc::close(dir_fd);
        }
        if parent_fd < 0 {
            return Err(LupError::Io(std::io::Error::last_os_error()));
        }
        dir_fd = parent_fd;

        if !pop_component(&mut current) {
            unsafe {
                libc::close(dir_fd);
            }
            break;
        }
    }

    if hits.is_empty() {
        Err(LupError::NoMatch)
    } else {
        Ok(hits)
    }
}

fn home_dir_bytes() -> Option<Vec<u8>> {
    std::env::var_os("HOME").map(|s| s.into_vec())
}

/// `dst = dst + "/" + component`, with leading-slash dedup so we don't produce `//x`.
fn push_component(dst: &mut Vec<u8>, component: &[u8]) {
    if !dst.is_empty() && !dst.ends_with(b"/") {
        dst.push(b'/');
    }
    dst.extend_from_slice(component);
}

/// Remove the last `/component` from `dst`. Returns false if `dst` was already `/` or empty.
fn pop_component(dst: &mut Vec<u8>) -> bool {
    if dst == b"/" || dst.is_empty() {
        return false;
    }
    if let Some(pos) = dst.iter().rposition(|&b| b == b'/') {
        if pos == 0 {
            dst.truncate(1);
        } else {
            dst.truncate(pos);
        }
        true
    } else {
        false
    }
}
