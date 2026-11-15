use liblup::boundary::find_git_root;
use liblup::{lookup, Boundary, Hit, KindFilter, LupError, Query};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
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

/// A temp dir and its physical path. `lookup` needs a physical start, and
/// macOS's temp dir sits under the `/var` -> `/private/var` symlink.
fn physical_tempdir() -> (TempDir, PathBuf) {
    let tmp = TempDir::new().unwrap();
    let root = std::fs::canonicalize(tmp.path()).unwrap();
    (tmp, root)
}

fn bytes(p: &Path) -> &[u8] {
    p.as_os_str().as_bytes()
}

fn path_hit(p: PathBuf) -> Hit {
    Hit::Path(bytes(&p).to_vec())
}

#[test]
fn finds_query_at_start() {
    let (_tmp, root) = physical_tempdir();
    std::fs::write(root.join(".env"), b"x=1\n").unwrap();

    let hits = lookup(&Query::new(b".env"), bytes(&root), Boundary::Root).unwrap();
    assert_eq!(hits, vec![path_hit(root.join(".env"))]);
}

#[test]
fn finds_query_one_level_up() {
    let (_tmp, root) = physical_tempdir();
    let nested = root.join("a").join("b");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(root.join("a").join(".env"), b"x=1\n").unwrap();

    let hits = lookup(&Query::new(b".env"), bytes(&nested), Boundary::Root).unwrap();
    assert_eq!(hits, vec![path_hit(root.join("a").join(".env"))]);
}

#[test]
fn finds_query_at_deep_ancestor() {
    let (_tmp, root) = physical_tempdir();
    let nested = root.join("a").join("b").join("c").join("d");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(root.join(".env"), b"x=1\n").unwrap();

    let hits = lookup(&Query::new(b".env"), bytes(&nested), Boundary::Root).unwrap();
    assert_eq!(hits, vec![path_hit(root.join(".env"))]);
}

#[test]
fn no_match_returns_no_match_error() {
    let (_tmp, root) = physical_tempdir();
    let nested = root.join("a").join("b");
    std::fs::create_dir_all(&nested).unwrap();

    let r = lookup(&Query::new(b".env"), bytes(&nested), Boundary::Root);
    assert!(matches!(r, Err(LupError::NoMatch)));
}

#[test]
fn relative_start_is_a_usage_error() {
    let r = lookup(&Query::new(b".env"), b"a/b", Boundary::Root);
    assert!(matches!(r, Err(LupError::UsageError(_))));
}

#[test]
fn all_hits_closest_first() {
    let (_tmp, root) = physical_tempdir();
    let nested = root.join("a").join("b");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(root.join(".env"), b"outer=1\n").unwrap();
    std::fs::write(root.join("a").join(".env"), b"middle=1\n").unwrap();
    std::fs::write(nested.join(".env"), b"inner=1\n").unwrap();

    let mut q = Query::new(b".env");
    q.all = true;

    let hits = lookup(&q, bytes(&nested), Boundary::Root).unwrap();
    assert_eq!(
        hits,
        vec![
            path_hit(nested.join(".env")),
            path_hit(root.join("a").join(".env")),
            path_hit(root.join(".env")),
        ]
    );
}

#[test]
fn multi_component_query() {
    let (_tmp, root) = physical_tempdir();
    let nested = root.join("project").join("src");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::create_dir(root.join("project").join(".claude")).unwrap();
    std::fs::write(
        root.join("project").join(".claude").join("settings.json"),
        b"{}",
    )
    .unwrap();

    let hits = lookup(
        &Query::new(b".claude/settings.json"),
        bytes(&nested),
        Boundary::Root,
    )
    .unwrap();
    assert_eq!(
        hits,
        vec![path_hit(
            root.join("project").join(".claude").join("settings.json")
        )]
    );
}

#[test]
fn files_only_filter_skips_closer_directory() {
    let (_tmp, root) = physical_tempdir();
    // Top-level: regular file named "target".
    std::fs::write(root.join("target"), b"file-content").unwrap();
    // Mid-level: directory named "target" (would be the closer hit without filter).
    let mid = root.join("a");
    std::fs::create_dir(&mid).unwrap();
    std::fs::create_dir(mid.join("target")).unwrap();
    let nested = mid.join("b");
    std::fs::create_dir(&nested).unwrap();

    let mut q = Query::new(b"target");
    q.kind_filter = Some(KindFilter::Files);

    let hits = lookup(&q, bytes(&nested), Boundary::Root).unwrap();
    assert_eq!(hits, vec![path_hit(root.join("target"))]);
}

#[test]
fn dirs_only_filter_skips_closer_regular_file() {
    let (_tmp, root) = physical_tempdir();
    // Top-level: directory named ".git".
    std::fs::create_dir(root.join(".git")).unwrap();
    // Mid-level: regular file named ".git" (would be the closer hit without filter).
    let mid = root.join("a");
    std::fs::create_dir(&mid).unwrap();
    std::fs::write(mid.join(".git"), b"gitfile").unwrap();
    let nested = mid.join("b");
    std::fs::create_dir(&nested).unwrap();

    let mut q = Query::new(b".git");
    q.kind_filter = Some(KindFilter::Dirs);

    let hits = lookup(&q, bytes(&nested), Boundary::Root).unwrap();
    assert_eq!(hits, vec![path_hit(root.join(".git"))]);
}

#[test]
fn follow_default_treats_dangling_symlink_as_missing() {
    use std::os::unix::fs::symlink;
    let (_tmp, root) = physical_tempdir();
    symlink("/this/path/does/not/exist", root.join(".env")).unwrap();

    // Default follow=true: faccessat follows the dangling link, ENOENT -> no match.
    let r = lookup(&Query::new(b".env"), bytes(&root), Boundary::Root);
    assert!(matches!(r, Err(LupError::NoMatch)));
}

#[test]
fn no_follow_matches_symlink_itself() {
    use std::os::unix::fs::symlink;
    let (_tmp, root) = physical_tempdir();
    symlink("/this/path/does/not/exist", root.join(".env")).unwrap();

    let mut q = Query::new(b".env");
    q.follow = false;

    let hits = lookup(&q, bytes(&root), Boundary::Root).unwrap();
    assert_eq!(hits, vec![path_hit(root.join(".env"))]);
}

#[test]
fn home_boundary_stops_at_home_inclusive() {
    let (_tmp, root) = physical_tempdir();
    let home = root.join("home");
    let nested = home.join("project");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(home.join(".env"), b"home=1\n").unwrap();

    let boundary = Boundary::Home(Some(bytes(&home)));
    let hits = lookup(&Query::new(b".env"), bytes(&nested), boundary).unwrap();
    assert_eq!(hits, vec![path_hit(home.join(".env"))]);
}

#[test]
fn home_boundary_does_not_walk_past_home() {
    let (_tmp, root) = physical_tempdir();
    let home = root.join("home");
    let nested = home.join("project");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(root.join(".env"), b"x=1\n").unwrap();

    let boundary = Boundary::Home(Some(bytes(&home)));
    let r = lookup(&Query::new(b".env"), bytes(&nested), boundary);
    assert!(matches!(r, Err(LupError::NoMatch)));
}

#[test]
fn home_boundary_walks_to_root_when_start_outside_home() {
    let (_tmp, root) = physical_tempdir();
    let home = root.join("home");
    std::fs::create_dir_all(&home).unwrap();
    let elsewhere = root.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    std::fs::write(root.join(".env"), b"x=1\n").unwrap();

    let boundary = Boundary::Home(Some(bytes(&home)));
    let hits = lookup(&Query::new(b".env"), bytes(&elsewhere), boundary).unwrap();
    assert_eq!(hits, vec![path_hit(root.join(".env"))]);
}

#[test]
fn home_boundary_without_a_home_walks_to_root() {
    let (_tmp, root) = physical_tempdir();
    let nested = root.join("home").join("project");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(root.join(".env"), b"x=1\n").unwrap();

    let hits = lookup(&Query::new(b".env"), bytes(&nested), Boundary::Home(None)).unwrap();
    assert_eq!(hits, vec![path_hit(root.join(".env"))]);
}

#[test]
fn git_boundary_errors_when_no_repo() {
    let (_tmp, root) = physical_tempdir();
    let r = lookup(&Query::new(b".env"), bytes(&root), Boundary::Git);
    assert!(matches!(r, Err(LupError::NotInGitRepo)));
}

#[test]
fn echo_returns_file_contents() {
    let (_tmp, root) = physical_tempdir();
    std::fs::write(root.join(".env"), b"FOO=bar\n").unwrap();

    let mut q = Query::new(b".env");
    q.echo = true;

    let hits = lookup(&q, bytes(&root), Boundary::Root).unwrap();
    assert_eq!(hits, vec![Hit::Contents(b"FOO=bar\n".to_vec())]);
}

#[test]
fn echo_on_directory_errors() {
    let (_tmp, root) = physical_tempdir();
    std::fs::create_dir(root.join("target")).unwrap();

    let mut q = Query::new(b"target");
    q.echo = true;

    match lookup(&q, bytes(&root), Boundary::Root) {
        Err(LupError::Io(e)) => {
            assert!(
                e.raw_os_error() == Some(libc::EISDIR) || e.kind() == std::io::ErrorKind::Other
            );
        }
        other => panic!("expected Io error for dir, got {:?}", other),
    }
}

#[test]
fn echo_with_all_concatenates_in_walk_order() {
    let (_tmp, root) = physical_tempdir();
    let nested = root.join("a").join("b");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(root.join(".env"), b"OUTER\n").unwrap();
    std::fs::write(nested.join(".env"), b"INNER\n").unwrap();

    let mut q = Query::new(b".env");
    q.echo = true;
    q.all = true;

    let hits = lookup(&q, bytes(&nested), Boundary::Root).unwrap();
    assert_eq!(
        hits,
        vec![
            Hit::Contents(b"INNER\n".to_vec()),
            Hit::Contents(b"OUTER\n".to_vec()),
        ]
    );
}

/// With a type filter the caller said what they want, so a directory that
/// cannot be searched is an error, not a silent "no hit": skipping it could
/// hide the very file the caller is after.
#[test]
fn type_filter_in_an_unsearchable_directory_is_an_io_error() {
    use std::os::unix::fs::PermissionsExt;
    if unsafe { libc::geteuid() } == 0 {
        return; // root ignores directory permissions
    }
    let (_tmp, root) = physical_tempdir();
    let locked = root.join("locked");
    std::fs::create_dir(&locked).unwrap();
    // readable (so it can be opened) but not searchable (so it cannot be probed)
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o600)).unwrap();

    let mut q = Query::new(b".env");
    q.kind_filter = Some(KindFilter::Files);
    let got = lookup(&q, bytes(&locked), Boundary::Root);
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o700)).unwrap();

    match got {
        Err(LupError::Io(e)) => assert_eq!(e.raw_os_error(), Some(libc::EACCES)),
        other => panic!("expected an EACCES io error, got {other:?}"),
    }
}
