//! Argument parser. Pure; consumes a `Vec<OsString>` (the argv tail), returns ParsedArgs.

use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ParsedArgs {
    pub query: Option<Vec<u8>>,
    pub all: bool,
    pub echo: bool,
    pub null_terminate: bool,
    pub relative: bool,
    pub git: bool,
    pub root: bool,
    pub files_only: bool,
    pub dirs_only: bool,
    pub follow_explicit: bool,
    pub no_follow_explicit: bool,
    pub help: bool,
    pub version: bool,
    pub completions_shell: Option<Vec<u8>>,
}

impl ParsedArgs {
    /// Effective follow-symlinks setting after applying defaults.
    /// Default is follow=true; -P forces follow=false.
    pub fn effective_follow(&self) -> bool {
        !self.no_follow_explicit
    }
}

pub fn parse(argv_tail: Vec<OsString>) -> Result<ParsedArgs, String> {
    let mut a = ParsedArgs::default();
    let mut iter = argv_tail.into_iter();
    let mut allow_flags = true;
    while let Some(arg) = iter.next() {
        let bytes = arg.into_vec();
        if allow_flags && bytes == b"--" {
            allow_flags = false;
            continue;
        }
        if allow_flags && bytes.starts_with(b"--") {
            parse_long(&bytes, &mut a, &mut iter)?;
        } else if allow_flags && bytes.starts_with(b"-") && bytes.len() > 1 {
            parse_short(&bytes, &mut a)?;
        } else {
            if a.query.is_some() {
                return Err("multiple positional arguments not allowed".into());
            }
            a.query = Some(bytes);
        }
    }
    validate(&a)?;
    Ok(a)
}

fn parse_long(
    bytes: &[u8],
    a: &mut ParsedArgs,
    iter: &mut impl Iterator<Item = OsString>,
) -> Result<(), String> {
    match bytes {
        b"--all" => a.all = true,
        b"--echo" => a.echo = true,
        b"--null" => a.null_terminate = true,
        b"--relative" => a.relative = true,
        b"--git" => a.git = true,
        b"--root" => a.root = true,
        b"--files" => a.files_only = true,
        b"--dirs" => a.dirs_only = true,
        b"--follow" => a.follow_explicit = true,
        b"--no-follow" => a.no_follow_explicit = true,
        b"--help" => a.help = true,
        b"--version" => a.version = true,
        b"--completions" => {
            let next = iter
                .next()
                .ok_or_else(|| "--completions requires <SHELL>".to_string())?;
            a.completions_shell = Some(next.into_vec());
        }
        _ => return Err(format!("unknown flag: {}", String::from_utf8_lossy(bytes))),
    }
    Ok(())
}

fn parse_short(bytes: &[u8], a: &mut ParsedArgs) -> Result<(), String> {
    for &c in &bytes[1..] {
        match c {
            b'a' => a.all = true,
            b'e' => a.echo = true,
            b'g' => a.git = true,
            b'r' => a.root = true,
            b'f' => a.files_only = true,
            b'd' => a.dirs_only = true,
            b'L' => a.follow_explicit = true,
            b'P' => a.no_follow_explicit = true,
            b'0' => a.null_terminate = true,
            b'h' => a.help = true,
            b'V' => a.version = true,
            other => return Err(format!("unknown short flag: -{}", other as char)),
        }
    }
    Ok(())
}

pub fn validate(a: &ParsedArgs) -> Result<(), String> {
    let short_circuit = a.help || a.version || a.completions_shell.is_some();
    if !short_circuit && a.query.is_none() {
        return Err("missing positional query argument".into());
    }
    if a.git && a.root {
        return Err("flags -g/--git and -r/--root are mutually exclusive".into());
    }
    if a.files_only && a.dirs_only {
        return Err("flags -f/--files and -d/--dirs are mutually exclusive".into());
    }
    if a.follow_explicit && a.no_follow_explicit {
        return Err("flags -L/--follow and -P/--no-follow are mutually exclusive".into());
    }
    Ok(())
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const HELP: &str = "\
lup — walk up the directory hierarchy looking for a path

USAGE:
    lup [OPTIONS] <QUERY>
    lup --completions <SHELL>
    lup -h | --help | -V | --version

ARGUMENTS:
    <QUERY>           Literal path to look for at each ancestor (e.g. .env, .claude/settings.json)

OPTIONS:
    -a, --all         Print every hit, closest-first
    -e, --echo        Print file contents instead of paths
    -g, --git         Stop at git root; error if not in a git repo
    -r, --root        Walk all the way to filesystem root
    -f, --files       Only match regular files
    -d, --dirs        Only match directories
    -L, --follow      Follow symlinks (default)
    -P, --no-follow   Don't follow symlinks
        --relative    Print paths relative to PWD instead of absolute
    -0, --null        Terminate each output record with NUL instead of newline
        --completions <SHELL>
                      Print completion script for bash, zsh, or fish
    -h, --help        Print help and exit
    -V, --version     Print version and exit

EXIT CODES:
    0  match(es) found
    1  no match within boundary
    2  usage error
    3  -g specified but not in a git repository
    4  I/O error during -e read

EXAMPLES:
    lup .git
    lup .claude/settings.json
    lup -g .env
    lup -a .env
    source $(lup .env)
    eval $(lup -e .env)
";

#[cfg(test)]
mod tests {
    use super::*;

    fn os(bytes: &[u8]) -> OsString {
        OsString::from_vec(bytes.to_vec())
    }
    fn parse_strs(bs: &[&[u8]]) -> Result<ParsedArgs, String> {
        parse(bs.iter().map(|b| os(b)).collect())
    }

    #[test]
    fn parses_query_only() {
        let a = parse_strs(&[b".env"]).unwrap();
        assert_eq!(a.query.as_deref(), Some(b".env".as_ref()));
        assert!(!a.all);
    }

    #[test]
    fn parses_short_flags() {
        let a = parse_strs(&[b"-a", b".env"]).unwrap();
        assert!(a.all);
        assert_eq!(a.query.as_deref(), Some(b".env".as_ref()));
    }

    #[test]
    fn parses_stacked_short_flags() {
        let a = parse_strs(&[b"-ae", b".env"]).unwrap();
        assert!(a.all);
        assert!(a.echo);
    }

    #[test]
    fn parses_long_flags() {
        let a = parse_strs(&[b"--all", b"--null", b".env"]).unwrap();
        assert!(a.all);
        assert!(a.null_terminate);
    }

    #[test]
    fn rejects_unknown_short_flag() {
        let r = parse_strs(&[b"-x", b".env"]);
        assert!(r.is_err());
    }

    #[test]
    fn rejects_unknown_long_flag() {
        let r = parse_strs(&[b"--bogus", b".env"]);
        assert!(r.is_err());
    }

    #[test]
    fn double_dash_treats_remainder_as_positional() {
        let a = parse_strs(&[b"--", b"-x"]).unwrap();
        assert_eq!(a.query.as_deref(), Some(b"-x".as_ref()));
    }

    #[test]
    fn completions_consumes_shell_argument() {
        let a = parse_strs(&[b"--completions", b"zsh"]).unwrap();
        assert_eq!(a.completions_shell.as_deref(), Some(b"zsh".as_ref()));
    }

    #[test]
    fn mutex_violation_g_r() {
        let r = parse_strs(&[b"-g", b"-r", b".env"]);
        assert!(r.is_err());
        let err = r.unwrap_err();
        assert!(err.contains("-g") || err.contains("--git"));
    }

    #[test]
    fn mutex_violation_f_d() {
        let r = parse_strs(&[b"-f", b"-d", b".env"]);
        assert!(r.is_err());
    }

    #[test]
    fn mutex_violation_l_p() {
        let r = parse_strs(&[b"-L", b"-P", b".env"]);
        assert!(r.is_err());
    }

    #[test]
    fn missing_query_is_error() {
        let r = parse_strs(&[b"-a"]);
        assert!(r.is_err());
    }

    #[test]
    fn help_short_circuits_missing_query() {
        let a = parse_strs(&[b"-h"]).unwrap();
        assert!(a.help);
        assert!(a.query.is_none());
    }

    #[test]
    fn version_short_circuits_missing_query() {
        let a = parse_strs(&[b"-V"]).unwrap();
        assert!(a.version);
    }

    #[test]
    fn completions_short_circuits_missing_query() {
        let a = parse_strs(&[b"--completions", b"bash"]).unwrap();
        assert_eq!(a.completions_shell.as_deref(), Some(b"bash".as_ref()));
        assert!(a.query.is_none());
    }

    #[test]
    fn help_text_mentions_examples() {
        assert!(HELP.contains("source $(lup .env)"));
        assert!(HELP.contains("eval $(lup -e .env)"));
    }

    #[test]
    fn version_constant_is_set() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }
}
