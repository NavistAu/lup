//! Stdout writers.

use crate::walk::{Hit, Query};
use std::io::Write;

pub fn write_hits(
    out: &mut impl Write,
    hits: &[Hit],
    query: &Query,
    pwd: &[u8],
) -> std::io::Result<()> {
    let term: u8 = if query.null_terminate { 0 } else { b'\n' };
    for hit in hits {
        match hit {
            Hit::Path(p) => {
                if query.relative {
                    let rel = relativize(p, pwd);
                    out.write_all(&rel)?;
                } else {
                    out.write_all(p)?;
                }
                out.write_all(&[term])?;
            }
            Hit::Contents(c) => {
                out.write_all(c)?;
                if query.null_terminate {
                    out.write_all(&[0u8])?;
                }
                // No newline injection in default echo mode — see spec §3.
            }
        }
    }
    Ok(())
}

fn relativize(abs_path: &[u8], pwd: &[u8]) -> Vec<u8> {
    // abs_path: full hit path (e.g. /a/.env)
    // pwd:       current directory (e.g. /a/b/c)
    // result:    ../../.env (or .env when the hit is in pwd itself)
    let abs_dir_end = abs_path.iter().rposition(|&b| b == b'/').unwrap_or(0);
    let abs_dir = &abs_path[..abs_dir_end];
    let abs_file = &abs_path[abs_dir_end + 1..];

    let abs_components: Vec<&[u8]> = split_components(abs_dir);
    let pwd_components: Vec<&[u8]> = split_components(pwd);

    let common = abs_components
        .iter()
        .zip(pwd_components.iter())
        .take_while(|(a, p)| a == p)
        .count();

    let mut out = Vec::new();
    for _ in common..pwd_components.len() {
        if !out.is_empty() {
            out.push(b'/');
        }
        out.extend_from_slice(b"..");
    }
    for c in &abs_components[common..] {
        if !out.is_empty() {
            out.push(b'/');
        }
        out.extend_from_slice(c);
    }
    if !out.is_empty() {
        out.push(b'/');
    }
    out.extend_from_slice(abs_file);
    if out.is_empty() {
        out.push(b'.');
    }
    out
}

fn split_components(path: &[u8]) -> Vec<&[u8]> {
    path.split(|&b| b == b'/')
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_writes_newline_terminated_paths() {
        let hits = vec![Hit::Path(b"/a/b/.env".to_vec())];
        let q = Query::new(b".env");
        let mut buf = Vec::new();
        write_hits(&mut buf, &hits, &q, b"/a/b/c").unwrap();
        assert_eq!(buf, b"/a/b/.env\n");
    }

    #[test]
    fn default_mode_multiple_hits() {
        let hits = vec![
            Hit::Path(b"/a/b/c/.env".to_vec()),
            Hit::Path(b"/a/.env".to_vec()),
        ];
        let q = Query::new(b".env");
        let mut buf = Vec::new();
        write_hits(&mut buf, &hits, &q, b"/a/b/c").unwrap();
        assert_eq!(buf, b"/a/b/c/.env\n/a/.env\n");
    }

    #[test]
    fn null_terminate_replaces_newline() {
        let hits = vec![Hit::Path(b"/a/.env".to_vec()), Hit::Path(b"/.env".to_vec())];
        let mut q = Query::new(b".env");
        q.null_terminate = true;
        let mut buf = Vec::new();
        write_hits(&mut buf, &hits, &q, b"/a").unwrap();
        assert_eq!(buf, b"/a/.env\0/.env\0");
    }

    #[test]
    fn relative_mode_for_path_in_pwd() {
        let hits = vec![Hit::Path(b"/a/b/c/.env".to_vec())];
        let mut q = Query::new(b".env");
        q.relative = true;
        let mut buf = Vec::new();
        write_hits(&mut buf, &hits, &q, b"/a/b/c").unwrap();
        assert_eq!(buf, b".env\n");
    }

    #[test]
    fn relative_mode_for_ancestor() {
        let hits = vec![Hit::Path(b"/a/.env".to_vec())];
        let mut q = Query::new(b".env");
        q.relative = true;
        let mut buf = Vec::new();
        write_hits(&mut buf, &hits, &q, b"/a/b/c").unwrap();
        assert_eq!(buf, b"../../.env\n");
    }

    #[test]
    fn relative_mode_for_two_levels_up() {
        let hits = vec![Hit::Path(b"/a/.env".to_vec())];
        let mut q = Query::new(b".env");
        q.relative = true;
        let mut buf = Vec::new();
        write_hits(&mut buf, &hits, &q, b"/a/b").unwrap();
        assert_eq!(buf, b"../.env\n");
    }

    #[test]
    fn echo_writes_contents_verbatim() {
        let hits = vec![Hit::Contents(b"FOO=bar\n".to_vec())];
        let mut q = Query::new(b".env");
        q.echo = true;
        let mut buf = Vec::new();
        write_hits(&mut buf, &hits, &q, b"/").unwrap();
        assert_eq!(buf, b"FOO=bar\n");
    }

    #[test]
    fn echo_with_all_concatenates_no_separator() {
        let hits = vec![
            Hit::Contents(b"INNER\n".to_vec()),
            Hit::Contents(b"OUTER\n".to_vec()),
        ];
        let mut q = Query::new(b".env");
        q.echo = true;
        q.all = true;
        let mut buf = Vec::new();
        write_hits(&mut buf, &hits, &q, b"/").unwrap();
        assert_eq!(buf, b"INNER\nOUTER\n");
    }

    #[test]
    fn echo_with_all_and_null_term_separates_contents() {
        let hits = vec![
            Hit::Contents(b"INNER".to_vec()),
            Hit::Contents(b"OUTER".to_vec()),
        ];
        let mut q = Query::new(b".env");
        q.echo = true;
        q.all = true;
        q.null_terminate = true;
        let mut buf = Vec::new();
        write_hits(&mut buf, &hits, &q, b"/").unwrap();
        assert_eq!(buf, b"INNER\0OUTER\0");
    }
}
