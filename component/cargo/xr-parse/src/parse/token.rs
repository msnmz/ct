use super::*;

pub struct Token<T>(pub T);

impl<'s, S, T> Parser<'s, S> for Token<T>
where
    S: Slicy + Spanned + ?Sized + 's,
    T: xr_token::Token<S>,
{
    type Output = &'s S;
    fn parse(&self, inp: &'s S) -> Option<Self::Output> {
        let Self(token) = self;
        token.try_match(inp).non_empty().map(|s| s.scope(inp))
    }
}

#[test]
fn test_token() {
    let a = &Token(xr_token::exact(&'a'));

    assert_eq!(a.parse(""), None);
    assert_eq!(a.parse("b"), None);
    assert_eq!(a.parse("ba"), None);
    assert_eq!(a.parse("a"), Some("a"));

    let b = &Token(xr_token::class(|i, _: &_| i < 2));
    assert_eq!(b.parse(""), None);
    assert_eq!(b.parse("b"), Some("b"));
    assert_eq!(b.parse("ba"), Some("ba"));
    assert_eq!(b.parse("abc"), Some("ab"));

    let c = &Token(xr_token::class(|_, t: &_| t != &b'x'));
    assert_eq!(c.parse("".as_bytes()), None);
    assert_eq!(c.parse("b".as_bytes()), Some(b"b".as_ref()));
    assert_eq!(c.parse("ba".as_bytes()), Some(b"ba".as_ref()));
    assert_eq!(c.parse("abc".as_bytes()), Some(b"abc".as_ref()));
    assert_eq!(c.parse("abcxd".as_bytes()), Some(b"abc".as_ref()));
}
