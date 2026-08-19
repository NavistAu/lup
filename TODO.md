# TODO

- **Test flakiness under the default parallel runner.** Three tests in
  `tests/walk.rs` (`finds_query_at_pwd`, `echo_returns_file_contents`,
  `no_match_returns_no_match_error`) mutate the process-wide current directory
  and race each other; `cargo test` fails intermittently while
  `cargo test -- --test-threads=1` is green. Fix by making the tests
  cwd-independent (pass paths instead of chdir) or serializing that module.
  Noticed 2026-08-19 while porting the release pipeline.
