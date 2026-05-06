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

struct OwnedFd(libc::c_int);

impl OwnedFd {
    fn raw(&self) -> libc::c_int {
        self.0
    }
}

impl Drop for OwnedFd {
    fn drop(&mut self) {
        if self.0 >= 0 {
            unsafe {
                libc::close(self.0);
            }
        }
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

    let dir_fd_raw = unsafe {
        let dot = b".\0";
        libc::openat(
            libc::AT_FDCWD,
            dot.as_ptr() as *const libc::c_char,
            libc::O_DIRECTORY | libc::O_RDONLY,
        )
    };
    if dir_fd_raw < 0 {
        return Err(LupError::Io(std::io::Error::last_os_error()));
    }
    let mut dir = OwnedFd(dir_fd_raw);

    loop {
        let matched = probe(dir.raw(), &query_cstr, query.kind_filter, query.follow)?;
        if matched {
            if query.echo {
                let contents = read_file_contents(dir.raw(), &query_cstr)?;
                hits.push(Hit::Contents(contents));
            } else {
                let mut hit_path = current.clone();
                push_component(&mut hit_path, query.path);
                hits.push(Hit::Path(hit_path));
            }
            if !query.all {
                return Ok(hits);
            }
        }

        if current == boundary_bytes {
            break;
        }

        let parent_fd_raw = unsafe {
            let dotdot = b"..\0";
            libc::openat(
                dir.raw(),
                dotdot.as_ptr() as *const libc::c_char,
                libc::O_DIRECTORY | libc::O_RDONLY,
            )
        };
        if parent_fd_raw < 0 {
            return Err(LupError::Io(std::io::Error::last_os_error()));
        }
        dir = OwnedFd(parent_fd_raw); // old dir's Drop closes it

        if !pop_component(&mut current) {
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

fn probe(
    dir_fd: libc::c_int,
    query: &CString,
    kind: Option<KindFilter>,
    follow: bool,
) -> Result<bool, LupError> {
    if kind.is_none() && follow {
        // Cheap path: just existence with symlink-following. faccessat returns
        // 0 on success and -1 on any failure (ENOENT, EACCES, ENOTDIR, etc.);
        // we treat all failures as "not a hit" and continue the walk. This
        // matches typical shell semantics where an inaccessible ancestor is
        // walked past, not an error.
        let r = unsafe { libc::faccessat(dir_fd, query.as_ptr(), libc::F_OK, 0) };
        return Ok(r == 0);
    }
    // Need stat info or no-follow semantics. Note the asymmetry with the
    // faccessat path: only ENOENT is treated as "not a hit"; other errors
    // (EACCES, EIO, etc.) bubble up as LupError::Io. The reason is that with
    // a type filter the caller has expressed intent ("only files" or "only
    // dirs"), and silently skipping a permission-denied ancestor could mask
    // the answer the user is looking for. With plain existence (above) the
    // walk-past behavior is the principle of least surprise.
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    let flags = if follow { 0 } else { libc::AT_SYMLINK_NOFOLLOW };
    let r = unsafe { libc::fstatat(dir_fd, query.as_ptr(), &mut st as *mut libc::stat, flags) };
    if r != 0 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::ENOENT) {
            return Ok(false);
        }
        return Err(LupError::Io(err));
    }
    let mode = st.st_mode;
    let matched = match kind {
        None => true,
        Some(KindFilter::Files) => is_reg(mode),
        Some(KindFilter::Dirs) => is_dir(mode),
    };
    Ok(matched)
}

fn is_reg(mode: libc::mode_t) -> bool {
    (mode & libc::S_IFMT) == libc::S_IFREG
}

fn is_dir(mode: libc::mode_t) -> bool {
    (mode & libc::S_IFMT) == libc::S_IFDIR
}

fn read_file_contents(dir_fd: libc::c_int, name: &CString) -> Result<Vec<u8>, LupError> {
    let raw_fd = unsafe { libc::openat(dir_fd, name.as_ptr(), libc::O_RDONLY) };
    if raw_fd < 0 {
        return Err(LupError::Io(std::io::Error::last_os_error()));
    }
    let file = OwnedFd(raw_fd); // closed on drop, including panic-unwind paths

    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstat(file.raw(), &mut st as *mut libc::stat) } != 0 {
        return Err(LupError::Io(std::io::Error::last_os_error()));
    }
    if (st.st_mode & libc::S_IFMT) != libc::S_IFREG {
        return Err(LupError::Io(std::io::Error::from_raw_os_error(
            libc::EISDIR,
        )));
    }

    let size = st.st_size as usize;
    let mut buf: Vec<u8> = Vec::with_capacity(size);
    unsafe {
        let mut total: usize = 0;
        while total < size {
            let n = libc::read(
                file.raw(),
                buf.as_mut_ptr().add(total) as *mut libc::c_void,
                size - total,
            );
            if n < 0 {
                return Err(LupError::Io(std::io::Error::last_os_error()));
            }
            if n == 0 {
                break;
            }
            total += n as usize;
        }
        buf.set_len(total);
    }
    Ok(buf)
}
