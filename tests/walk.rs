use lup::{lookup, Boundary, Hit, LupError, Query};
use lup::boundary::find_git_root;
use std::os::unix::ffi::OsStrExt;
use tempfile::TempDir;

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

fn run_lookup_in(
    dir: &std::path::Path,
    query: &[u8],
    boundary: Boundary,
) -> Result<Vec<Hit>, LupError> {
    // Helper: chdir + lookup. Each #[test] runs in its own process under nextest,
    // so the global cwd mutation is safe.
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(dir).unwrap();
    let r = lookup(&Query::new(query), boundary);
    std::env::set_current_dir(prev).unwrap();
    r
}

#[test]
fn finds_query_at_pwd() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join(".env"), b"x=1\n").unwrap();

    let hits = run_lookup_in(tmp.path(), b".env", Boundary::Root).unwrap();
    assert_eq!(hits.len(), 1);
    // Canonicalize so that macOS /tmp -> /private/tmp symlinks don't cause a mismatch
    // (current_dir() returns the resolved path, so expected must match).
    let canonical_dir = std::fs::canonicalize(tmp.path()).unwrap();
    let expected = canonical_dir.join(".env");
    let expected_bytes: Vec<u8> = expected.as_os_str().as_bytes().to_vec();
    assert_eq!(hits[0], Hit::Path(expected_bytes));
}

#[test]
fn finds_query_one_level_up() {
    let tmp = TempDir::new().unwrap();
    let nested = tmp.path().join("a").join("b");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(tmp.path().join("a").join(".env"), b"x=1\n").unwrap();

    let hits = run_lookup_in(&nested, b".env", Boundary::Root).unwrap();
    assert_eq!(hits.len(), 1);
    let canonical_tmp = std::fs::canonicalize(tmp.path()).unwrap();
    let expected = canonical_tmp.join("a").join(".env");
    assert_eq!(hits[0], Hit::Path(expected.as_os_str().as_bytes().to_vec()));
}

#[test]
fn finds_query_at_deep_ancestor() {
    let tmp = TempDir::new().unwrap();
    let nested = tmp.path().join("a").join("b").join("c").join("d");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(tmp.path().join(".env"), b"x=1\n").unwrap();

    let hits = run_lookup_in(&nested, b".env", Boundary::Root).unwrap();
    assert_eq!(hits.len(), 1);
    let canonical_tmp = std::fs::canonicalize(tmp.path()).unwrap();
    let expected = canonical_tmp.join(".env");
    assert_eq!(hits[0], Hit::Path(expected.as_os_str().as_bytes().to_vec()));
}

#[test]
fn no_match_returns_no_match_error() {
    let tmp = TempDir::new().unwrap();
    let nested = tmp.path().join("a").join("b");
    std::fs::create_dir_all(&nested).unwrap();

    let r = run_lookup_in(&nested, b".env", Boundary::Root);
    assert!(matches!(r, Err(LupError::NoMatch)));
}

#[test]
fn all_hits_closest_first() {
    let tmp = TempDir::new().unwrap();
    let nested = tmp.path().join("a").join("b");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(tmp.path().join(".env"), b"outer=1\n").unwrap();
    std::fs::write(tmp.path().join("a").join(".env"), b"middle=1\n").unwrap();
    std::fs::write(nested.join(".env"), b"inner=1\n").unwrap();

    let mut q = Query::new(b".env");
    q.all = true;

    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&nested).unwrap();
    let hits = lookup(&q, Boundary::Root).unwrap();
    std::env::set_current_dir(prev).unwrap();

    let canonical_tmp = std::fs::canonicalize(tmp.path()).unwrap();
    let expected_inner = canonical_tmp.join("a").join("b").join(".env");
    let expected_middle = canonical_tmp.join("a").join(".env");
    let expected_outer = canonical_tmp.join(".env");

    assert_eq!(hits.len(), 3);
    let paths: Vec<&[u8]> = hits.iter().map(|h| match h {
        Hit::Path(p) => p.as_slice(),
        _ => panic!("unexpected"),
    }).collect();
    assert_eq!(paths[0], expected_inner.as_os_str().as_bytes());
    assert_eq!(paths[1], expected_middle.as_os_str().as_bytes());
    assert_eq!(paths[2], expected_outer.as_os_str().as_bytes());
}
