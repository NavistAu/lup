use lup::{lookup, Boundary, LupError, Query};
use lup::boundary::find_git_root;
use std::os::unix::ffi::OsStrExt;
use tempfile::TempDir;

#[test]
fn stub_lookup_returns_no_match() {
    let q = Query::new(b".env");
    let r = lookup(&q, Boundary::Root);
    assert!(matches!(r, Err(LupError::NoMatch)));
}

#[test]
fn git_root_found_when_dot_git_exists() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let nested = root.join("a").join("b").join("c");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::create_dir(root.join(".git")).unwrap();

    let got = find_git_root(nested.as_os_str().as_bytes()).unwrap();
    assert_eq!(got, root.as_os_str().as_bytes());
}

#[test]
fn git_root_finds_dot_git_file_for_worktrees() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    std::fs::write(root.join(".git"), "gitdir: /elsewhere\n").unwrap();
    let got = find_git_root(root.as_os_str().as_bytes()).unwrap();
    assert_eq!(got, root.as_os_str().as_bytes());
}

#[test]
fn git_root_not_found_returns_none() {
    let tmp = TempDir::new().unwrap();
    let nested = tmp.path().join("a").join("b");
    std::fs::create_dir_all(&nested).unwrap();
    assert!(find_git_root(nested.as_os_str().as_bytes()).is_none());
}
