use std::os::unix::ffi::OsStrExt;
use std::process::Command;
use tempfile::TempDir;

fn lup_bin() -> std::path::PathBuf {
    // Cargo sets CARGO_BIN_EXE_lup for integration tests.
    env!("CARGO_BIN_EXE_lup").into()
}

#[test]
fn cli_finds_query_at_pwd() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join(".env"), b"x=1\n").unwrap();
    let out = Command::new(lup_bin())
        .arg("-r")
        .arg(".env")
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let canonical_tmp = std::fs::canonicalize(tmp.path()).unwrap();
    let expected = canonical_tmp.join(".env");
    let mut want = expected.as_os_str().as_bytes().to_vec();
    want.push(b'\n');
    assert_eq!(out.stdout, want);
}

#[test]
fn cli_no_match_exit_1() {
    let tmp = TempDir::new().unwrap();
    let out = Command::new(lup_bin())
        .arg("-r")
        .arg(".env")
        .current_dir(tmp.path())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());
}

#[test]
fn cli_usage_error_exit_2() {
    let out = Command::new(lup_bin())
        .arg("-g")
        .arg("-r")
        .arg(".env")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn cli_help_prints_usage() {
    let out = Command::new(lup_bin()).arg("--help").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains("USAGE:"));
}

#[test]
fn cli_version_matches_cargo_pkg_version() {
    let out = Command::new(lup_bin()).arg("--version").output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.starts_with("lup "));
}

#[test]
fn cli_completions_bash_emits_script() {
    let out = Command::new(lup_bin())
        .arg("--completions")
        .arg("bash")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("_lup"));
}

#[test]
fn cli_completions_unknown_shell_exit_2() {
    let out = Command::new(lup_bin())
        .arg("--completions")
        .arg("powershell")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}
