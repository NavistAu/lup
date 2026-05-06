use std::ffi::OsString;
use std::io::Write;
use std::os::unix::ffi::OsStringExt;
use std::process::ExitCode;

use lup::cli::{self, HELP, VERSION};
use lup::completions;
use lup::output;
use lup::walk::{lookup, Boundary, KindFilter, LupError, Query};

fn main() -> ExitCode {
    let argv: Vec<OsString> = std::env::args_os().skip(1).collect();
    let parsed = match cli::parse(argv) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("lup: {}", e);
            eprintln!("Try 'lup --help' for usage.");
            return ExitCode::from(2);
        }
    };

    if parsed.help {
        let _ = std::io::stdout().write_all(HELP.as_bytes());
        return ExitCode::SUCCESS;
    }
    if parsed.version {
        println!("lup {}", VERSION);
        return ExitCode::SUCCESS;
    }
    if let Some(shell) = parsed.completions_shell.as_deref() {
        match completions::completion_text(shell) {
            Some(text) => {
                let _ = std::io::stdout().write_all(text.as_bytes());
                return ExitCode::SUCCESS;
            }
            None => {
                eprintln!(
                    "lup: unknown shell for --completions: {}",
                    String::from_utf8_lossy(shell)
                );
                return ExitCode::from(2);
            }
        }
    }

    let query_bytes = parsed
        .query
        .as_deref()
        .expect("validated to be present");
    let kind_filter = match (parsed.files_only, parsed.dirs_only) {
        (true, false) => Some(KindFilter::Files),
        (false, true) => Some(KindFilter::Dirs),
        _ => None,
    };
    let mut q = Query::new(query_bytes);
    q.all = parsed.all;
    q.echo = parsed.echo;
    q.kind_filter = kind_filter;
    q.follow = parsed.effective_follow();
    q.null_terminate = parsed.null_terminate;
    q.relative = parsed.relative;

    let boundary = if parsed.git {
        Boundary::Git
    } else if parsed.root {
        Boundary::Root
    } else {
        Boundary::Home
    };

    let pwd = std::env::current_dir()
        .map(|p| p.into_os_string().into_vec())
        .unwrap_or_default();

    match lookup(&q, boundary) {
        Ok(hits) => {
            let stdout = std::io::stdout();
            let mut h = stdout.lock();
            if let Err(e) = output::write_hits(&mut h, &hits, &q, &pwd) {
                eprintln!("lup: write error: {}", e);
                return ExitCode::from(4);
            }
            ExitCode::SUCCESS
        }
        Err(LupError::NoMatch) => {
            eprintln!(
                "lup: no match for '{}'",
                String::from_utf8_lossy(query_bytes)
            );
            ExitCode::from(1)
        }
        Err(LupError::NotInGitRepo) => {
            eprintln!("lup: -g requested but not inside a git repository");
            ExitCode::from(3)
        }
        Err(LupError::UsageError(m)) => {
            eprintln!("lup: {}", m);
            ExitCode::from(2)
        }
        Err(LupError::Io(e)) => {
            eprintln!("lup: I/O error: {}", e);
            ExitCode::from(4)
        }
    }
}
