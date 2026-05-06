//! Boundary resolution.
//!
//! Default rule: if PWD is HOME or under HOME → HOME, else → /.
//! `pwd` and `home` MUST be normalized: absolute, no trailing `/` (except `/`),
//! no `.` / `..` / `//` components.

pub fn resolve_default<'a>(pwd: &'a [u8], home: &'a [u8]) -> &'a [u8] {
    if is_under_or_equal(pwd, home) {
        home
    } else {
        b"/"
    }
}

/// True if `child == parent` or `child` is under `parent` (parent is a path-prefix).
fn is_under_or_equal(child: &[u8], parent: &[u8]) -> bool {
    if child == parent {
        return true;
    }
    if !child.starts_with(parent) {
        return false;
    }
    // Edge: parent == "/", child == "/anything" — starts_with(b"/") is true, and we want
    // "/foo" to be under "/", so just return true.
    if parent == b"/" {
        return true;
    }
    // Otherwise the byte after `parent` in `child` must be a path separator,
    // otherwise this is a similarly-named-but-distinct directory ("/home/alice2" vs
    // "/home/alice").
    child.get(parent.len()) == Some(&b'/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pwd_is_home_returns_home() {
        let b = resolve_default(b"/home/alice", b"/home/alice");
        assert_eq!(b, b"/home/alice");
    }

    #[test]
    fn pwd_under_home_returns_home() {
        let b = resolve_default(b"/home/alice/projects/foo", b"/home/alice");
        assert_eq!(b, b"/home/alice");
    }

    #[test]
    fn pwd_outside_home_returns_root() {
        let b = resolve_default(b"/etc", b"/home/alice");
        assert_eq!(b, b"/");
    }

    #[test]
    fn pwd_with_home_as_prefix_but_not_under_returns_root() {
        let b = resolve_default(b"/home/alice2/x", b"/home/alice");
        assert_eq!(b, b"/");
    }
}
