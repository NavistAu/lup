// Pure-startup baseline binary. `fn main() {}` exits as fast as a Rust program can.
// Used to isolate process-startup cost (fork, exec, dyld, Rust runtime init) from
// the actual work `lup` does. Compare hyperfine timings of noop vs lup invocations.

fn main() {}
