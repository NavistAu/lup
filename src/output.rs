//! Stdout writers.

use crate::walk::{Hit, Query};
use std::io::Write;

pub fn write_hits(
    out: &mut impl Write,
    hits: &[Hit],
    query: &Query,
    _pwd: &[u8],
) -> std::io::Result<()> {
    let term: u8 = if query.null_terminate { 0 } else { b'\n' };
    for hit in hits {
        match hit {
            Hit::Path(p) => {
                out.write_all(p)?;
                out.write_all(&[term])?;
            }
            Hit::Contents(_) => unreachable!(
                "contents written by write_hits-echo path; not yet implemented"
            ),
        }
    }
    Ok(())
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
}
