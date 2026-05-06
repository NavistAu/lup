use lup::{lookup, Boundary, Hit, KindFilter, LupError, Query};
use lup::boundary::find_git_root;
use std::os::unix::ffi::OsStrExt;
use tempfile::TempDir;
extern crate libc;

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

#[test]
fn multi_component_query() {
    let tmp = TempDir::new().unwrap();
    let nested = tmp.path().join("project").join("src");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::create_dir(tmp.path().join("project").join(".claude")).unwrap();
    std::fs::write(
        tmp.path().join("project").join(".claude").join("settings.json"),
        b"{}",
    ).unwrap();

    let hits = run_lookup_in(&nested, b".claude/settings.json", Boundary::Root).unwrap();
    assert_eq!(hits.len(), 1);
    let canonical_tmp = std::fs::canonicalize(tmp.path()).unwrap();
    let expected = canonical_tmp.join("project").join(".claude").join("settings.json");
    assert_eq!(hits[0], Hit::Path(expected.as_os_str().as_bytes().to_vec()));
}

#[test]
fn files_only_filter_skips_closer_directory() {
    let tmp = TempDir::new().unwrap();
    // Top-level: regular file named "target".
    std::fs::write(tmp.path().join("target"), b"file-content").unwrap();
    // Mid-level: directory named "target" (would be the closer hit without filter).
    let mid = tmp.path().join("a");
    std::fs::create_dir(&mid).unwrap();
    std::fs::create_dir(mid.join("target")).unwrap();
    let nested = mid.join("b");
    std::fs::create_dir(&nested).unwrap();

    let mut q = Query::new(b"target");
    q.kind_filter = Some(KindFilter::Files);

    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&nested).unwrap();
    let hits = lookup(&q, Boundary::Root).unwrap();
    std::env::set_current_dir(prev).unwrap();

    let canonical_tmp = std::fs::canonicalize(tmp.path()).unwrap();
    let expected = canonical_tmp.join("target");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0], Hit::Path(expected.as_os_str().as_bytes().to_vec()));
}

#[test]
fn dirs_only_filter_skips_closer_regular_file() {
    let tmp = TempDir::new().unwrap();
    // Top-level: directory named ".git".
    std::fs::create_dir(tmp.path().join(".git")).unwrap();
    // Mid-level: regular file named ".git" (would be the closer hit without filter).
    let mid = tmp.path().join("a");
    std::fs::create_dir(&mid).unwrap();
    std::fs::write(mid.join(".git"), b"gitfile").unwrap();
    let nested = mid.join("b");
    std::fs::create_dir(&nested).unwrap();

    let mut q = Query::new(b".git");
    q.kind_filter = Some(KindFilter::Dirs);

    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&nested).unwrap();
    let hits = lookup(&q, Boundary::Root).unwrap();
    std::env::set_current_dir(prev).unwrap();

    let canonical_tmp = std::fs::canonicalize(tmp.path()).unwrap();
    let expected = canonical_tmp.join(".git");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0], Hit::Path(expected.as_os_str().as_bytes().to_vec()));
}

#[test]
fn follow_default_treats_dangling_symlink_as_missing() {
    use std::os::unix::fs::symlink;
    let tmp = TempDir::new().unwrap();
    symlink("/this/path/does/not/exist", tmp.path().join(".env")).unwrap();

    // Default follow=true: faccessat follows the dangling link, ENOENT -> no match.
    let r = run_lookup_in(tmp.path(), b".env", Boundary::Root);
    assert!(matches!(r, Err(LupError::NoMatch)));
}

#[test]
fn no_follow_matches_symlink_itself() {
    use std::os::unix::fs::symlink;
    let tmp = TempDir::new().unwrap();
    symlink("/this/path/does/not/exist", tmp.path().join(".env")).unwrap();

    let mut q = Query::new(b".env");
    q.follow = false;

    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    let hits = lookup(&q, Boundary::Root).unwrap();
    std::env::set_current_dir(prev).unwrap();

    let canonical_tmp = std::fs::canonicalize(tmp.path()).unwrap();
    let expected = canonical_tmp.join(".env");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0], Hit::Path(expected.as_os_str().as_bytes().to_vec()));
}

#[test]
fn home_boundary_stops_at_home_inclusive() {
    let tmp = TempDir::new().unwrap();
    let fake_home = tmp.path().join("home");
    std::fs::create_dir_all(&fake_home).unwrap();
    let nested = fake_home.join("project");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(fake_home.join(".env"), b"home=1\n").unwrap();

    let prev_home = std::env::var_os("HOME");
    std::env::set_var("HOME", &fake_home);

    let hits = run_lookup_in(&nested, b".env", Boundary::Home).unwrap();
    assert_eq!(hits.len(), 1);
    let canonical_home = std::fs::canonicalize(&fake_home).unwrap();
    let expected = canonical_home.join(".env");
    assert_eq!(hits[0], Hit::Path(expected.as_os_str().as_bytes().to_vec()));

    if let Some(h) = prev_home {
        std::env::set_var("HOME", h);
    }
}

#[test]
fn home_boundary_walks_to_root_when_pwd_outside_home() {
    let tmp = TempDir::new().unwrap();
    let fake_home = tmp.path().join("home");
    std::fs::create_dir_all(&fake_home).unwrap();
    let elsewhere = tmp.path().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    std::fs::write(tmp.path().join(".env"), b"x=1\n").unwrap();

    let prev_home = std::env::var_os("HOME");
    std::env::set_var("HOME", &fake_home);

    let hits = run_lookup_in(&elsewhere, b".env", Boundary::Home).unwrap();
    assert!(!hits.is_empty());
    let canonical_tmp = std::fs::canonicalize(tmp.path()).unwrap();
    let expected = canonical_tmp.join(".env");
    assert_eq!(hits[0], Hit::Path(expected.as_os_str().as_bytes().to_vec()));

    if let Some(h) = prev_home {
        std::env::set_var("HOME", h);
    }
}

#[test]
fn git_boundary_errors_when_no_repo() {
    let tmp = TempDir::new().unwrap();
    let r = run_lookup_in(tmp.path(), b".env", Boundary::Git);
    assert!(matches!(r, Err(LupError::NotInGitRepo)));
}

#[test]
fn echo_returns_file_contents() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join(".env"), b"FOO=bar\n").unwrap();

    let mut q = Query::new(b".env");
    q.echo = true;

    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    let hits = lookup(&q, Boundary::Root).unwrap();
    std::env::set_current_dir(prev).unwrap();

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0], Hit::Contents(b"FOO=bar\n".to_vec()));
}

#[test]
fn echo_on_directory_errors() {
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join("target")).unwrap();

    let mut q = Query::new(b"target");
    q.echo = true;

    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    let r = lookup(&q, Boundary::Root);
    std::env::set_current_dir(prev).unwrap();

    match r {
        Err(LupError::Io(e)) => {
            assert!(
                e.raw_os_error() == Some(libc::EISDIR)
                    || e.kind() == std::io::ErrorKind::Other
            );
        }
        other => panic!("expected Io error for dir, got {:?}", other),
    }
}

#[test]
fn echo_with_all_concatenates_in_walk_order() {
    let tmp = TempDir::new().unwrap();
    let nested = tmp.path().join("a").join("b");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(tmp.path().join(".env"), b"OUTER\n").unwrap();
    std::fs::write(nested.join(".env"), b"INNER\n").unwrap();

    let mut q = Query::new(b".env");
    q.echo = true;
    q.all = true;

    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&nested).unwrap();
    let hits = lookup(&q, Boundary::Root).unwrap();
    std::env::set_current_dir(prev).unwrap();

    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0], Hit::Contents(b"INNER\n".to_vec()));
    assert_eq!(hits[1], Hit::Contents(b"OUTER\n".to_vec()));
}
