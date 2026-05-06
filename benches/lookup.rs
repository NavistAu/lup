//! In-process benchmarks for `lup::walk::lookup`.
//!
//! These measure the walk loop cost without process-spawn / dyld / Rust-runtime
//! overhead — i.e., the "algorithmic" part of lup. Compare to the wall-clock
//! perf test in `tests/perf.rs` which includes all of those.
//!
//! Run with `cargo bench`. Results land in `target/criterion/`.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lup::walk::{lookup, Boundary, Query};

const DEPTH: usize = 32;

fn bench_walk(c: &mut Criterion) {
    // Build the fixture once; criterion's iter() loop reuses it across samples.
    let tmp = tempfile::TempDir::new().unwrap();
    std::fs::write(tmp.path().join(".env"), b"FIXTURE=1\n").unwrap();
    let mut deep = tmp.path().to_path_buf();
    for i in 1..=DEPTH {
        deep = deep.join(format!("d{}", i));
    }
    std::fs::create_dir_all(&deep).unwrap();

    // Pin HOME outside the fixture so the default boundary doesn't short-circuit.
    std::env::set_var("HOME", "/nonexistent-home-for-bench");
    std::env::set_current_dir(&deep).unwrap();

    let mut group = c.benchmark_group("walk");
    group.bench_function("depth32_hit_at_root", |b| {
        b.iter(|| {
            let q = Query::new(b".env");
            black_box(lookup(&q, Boundary::Root)).unwrap();
        });
    });
    group.bench_function("depth32_no_match", |b| {
        b.iter(|| {
            let q = Query::new(b".no_such_file_here");
            let _ = black_box(lookup(&q, Boundary::Root));
        });
    });
    group.finish();
}

criterion_group!(benches, bench_walk);
criterion_main!(benches);
