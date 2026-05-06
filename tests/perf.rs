//! Performance floor enforcement.
//!
//! Builds the depth-8 fixture, runs hyperfine 3 times (lup, forky.sh, pure.sh),
//! and asserts:
//!     lup_median * 50 <= forky_median   (forky is at least 50× slower)
//!     lup_median * 10 <= pure_median    (pure is at least 10× slower)
//!
//! Also appends a row to bench/history/main.jsonl.

use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy)]
struct HyperfineResult {
    median: f64,
    mean: f64,
    stddev: f64,
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn require_release_build() -> PathBuf {
    let exe = env!("CARGO_BIN_EXE_lup");
    let path = PathBuf::from(exe);
    let profile = std::env::var("PROFILE").unwrap_or_default();
    if !path.to_string_lossy().contains("/release/") && profile != "release" {
        panic!(
            "tests/perf.rs must be run with `cargo nextest run --release` or \
             `cargo test --release` — got binary at {}",
            path.display()
        );
    }
    path
}

fn build_fixture() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let out = Command::new("bash")
        .arg(project_root().join("bench/fixtures/build_tree.sh"))
        .arg(tmp.path())
        .arg("8")
        .output()
        .expect("build_tree.sh");
    assert!(out.status.success(), "build_tree.sh failed: {:?}", out);
    let deepest = String::from_utf8(out.stdout).unwrap().trim().to_string();
    (tmp, PathBuf::from(deepest))
}

fn run_hyperfine(cwd: &std::path::Path, command: &str) -> HyperfineResult {
    let json_path = std::env::temp_dir().join(format!("lup-bench-{}.json", std::process::id()));
    let status = Command::new("hyperfine")
        .args(["--warmup", "5", "--runs", "21", "--export-json"])
        .arg(&json_path)
        .arg(command)
        .current_dir(cwd)
        .status()
        .expect("hyperfine must be installed (mise install)");
    assert!(status.success(), "hyperfine failed for: {}", command);

    let data: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&json_path).unwrap()).unwrap();
    let r = &data["results"][0];
    HyperfineResult {
        median: r["median"].as_f64().unwrap(),
        mean: r["mean"].as_f64().unwrap(),
        stddev: r["stddev"].as_f64().unwrap_or(0.0),
    }
}

#[test]
fn perf_floors_hold() {
    let lup = require_release_build();
    let (_guard, deepest) = build_fixture();

    std::env::set_var("HOME", "/nonexistent-home-for-test");

    let lup_cmd = format!("{} -r .env", lup.display());
    let forky_cmd = format!(
        "bash {} .env",
        project_root().join("bench/baselines/forky.sh").display()
    );
    let pure_cmd = format!(
        "bash {} .env",
        project_root().join("bench/baselines/pure.sh").display()
    );

    let lup_r = run_hyperfine(&deepest, &lup_cmd);
    let forky_r = run_hyperfine(&deepest, &forky_cmd);
    let pure_r = run_hyperfine(&deepest, &pure_cmd);

    eprintln!(
        "lup  median: {:.6}s  forky median: {:.6}s  pure median: {:.6}s",
        lup_r.median, forky_r.median, pure_r.median
    );
    eprintln!(
        "ratios: forky/lup = {:.1}×  pure/lup = {:.1}×",
        forky_r.median / lup_r.median,
        pure_r.median / lup_r.median
    );

    let row = serde_json::json!({
        "timestamp": epoch_seconds(),
        "version": env!("CARGO_PKG_VERSION"),
        "git_sha": std::env::var("GIT_SHA").unwrap_or_default(),
        "lup_median": lup_r.median,
        "forky_median": forky_r.median,
        "pure_median": pure_r.median,
        "ratio_forky": forky_r.median / lup_r.median,
        "ratio_pure": pure_r.median / lup_r.median,
    });
    let history = project_root().join("bench/history/main.jsonl");
    use std::io::Write as _;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&history)
    {
        let _ = writeln!(f, "{}", row);
    }

    assert!(
        lup_r.median * 50.0 <= forky_r.median,
        "FLOOR BREACH: lup vs forky-bash ratio is {:.1}× (need >=50×)",
        forky_r.median / lup_r.median
    );
    assert!(
        lup_r.median * 10.0 <= pure_r.median,
        "FLOOR BREACH: lup vs pure-bash ratio is {:.1}× (need >=10×)",
        pure_r.median / lup_r.median
    );

    if lup_r.median * 100.0 > forky_r.median {
        eprintln!(
            "WARN: aspirational 100× target missed for forky (got {:.1}×)",
            forky_r.median / lup_r.median
        );
    }
    if lup_r.median * 100.0 > pure_r.median {
        eprintln!(
            "WARN: aspirational 100× target missed for pure (got {:.1}×)",
            pure_r.median / lup_r.median
        );
    }
}

fn epoch_seconds() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
