use lup::{lookup, Boundary, LupError, Query};

#[test]
fn stub_lookup_returns_no_match() {
    let q = Query::new(b".env");
    let r = lookup(&q, Boundary::Root);
    assert!(matches!(r, Err(LupError::NoMatch)));
}
