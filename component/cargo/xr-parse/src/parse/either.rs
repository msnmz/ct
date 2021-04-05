// TODO: gvz (also parse token)
use super::*;

pub struct Either<L>(pub L);
use self::Either as Rcrs;

// TODO: move to own crate
pub mod either {
    pub use std::result::Result::{
        self as Disj,
        Err as B,
        Ok as A,
    };
}
use either::{
    Disj,
    A,
    B,
};

impl<'s, S, H, T> Parser<'s, S> for Rcrs<(H, T)>
where
    H: Parser<'s, S>,
    for<'r> Rcrs<&'r T>: Parser<'s, S>,
    Rcrs<T>: Parser<'s, S>,
    for<'r> ParseOut<'s, S, Rcrs<&'r T>>: Into<ParseOut<'s, S, Rcrs<T>>>,
    S: Slicy + ?Sized,
{
    type Output = Disj<ParseOut<'s, S, H>, ParseOut<'s, S, Rcrs<T>>>;

    fn parse(&self, inp: &'s S) -> Option<Self::Output> {
        let Self((head, tail)) = self;

        head.parse(inp)
            .map(A)
            .or_else(|| Rcrs(tail).parse(inp).map(<_>::into).map(B))
    }
}
impl<'s, 'p, S, H, T> Parser<'s, S> for Rcrs<&'p (&'p H, T)>
where
    H: Parser<'s, S> + ?Sized,
    Rcrs<&'p T>: Parser<'s, S>,
    S: Slicy + ?Sized,
{
    type Output = Disj<ParseOut<'s, S, H>, ParseOut<'s, S, Rcrs<&'p T>>>;

    fn parse(&self, inp: &'s S) -> Option<Self::Output> {
        let Self((head, tail)) = self;

        head.parse(inp)
            .map(A)
            .or_else(|| Rcrs(tail).parse(inp).map(B))
    }
}
impl<'s, 'p, S, H> Parser<'s, S> for Rcrs<(H,)>
where
    H: Parser<'s, S>,
    S: Slicy + ?Sized,
{
    type Output = ParseOut<'s, S, H>;

    fn parse(&self, inp: &'s S) -> Option<Self::Output> {
        let Self((head,)) = self;
        head.parse(inp)
    }
}
impl<'s, 'p, S, H> Parser<'s, S> for Rcrs<&'p (H,)>
where
    H: Parser<'s, S>,
    S: Slicy + ?Sized,
{
    type Output = ParseOut<'s, S, H>;

    fn parse(&self, inp: &'s S) -> Option<Self::Output> {
        let Self((head,)) = self;
        head.parse(inp)
    }
}

#[test]
fn test_either() {
    let a = &Token(xr_token::exact(&'a'));
    let b = &Token(xr_token::exact(&'b'));
    let c = &Token(xr_token::exact(&'c'));
    let __ = &(a, (b, (c,)));
    let either = &Either(__);

    assert!(either.parse("").is_none());
    assert_eq!(either.parse("a"), Some(A("a")));
    assert_eq!(either.parse("b"), Some(B(A("b"))));
    assert_eq!(either.parse("c"), Some(B(B("c"))));
}
#[test]
fn test_either_any_list() {
    let a = &Token(xr_token::exact(&'a'));
    let b = &Token(xr_token::exact(&'b'));
    let __ = &(a, (b,));
    let __ = &List(__);
    let __ = &Any(__);
    let x = __;

    let __ = &(b, (a,));
    let __ = &List(__);
    let __ = &Any(__);
    let y = __;

    let __ = &(x,);
    let either = &Either(__);

    assert!(either.parse("").is_none());
    assert!(either.parse("a").is_none());
    assert_eq!(either.parse("ab"), Some(vec![("a", ("b",))]));

    let __ = &(x, (y,));
    let either = &Either(__);

    assert!(either.parse("").is_none());
    assert!(either.parse("a").is_none());
    assert_eq!(either.parse("ab"), Some(A(vec![("a", ("b",))])));
    assert_eq!(either.parse("ba"), Some(B(vec![("b", ("a",))])));
    assert_eq!(either.parse("aba"), Some(A(vec![("a", ("b",))])));
    assert_eq!(either.parse("abba"), Some(A(vec![("a", ("b",))])));

    // Switch any and either
    let __ = &(a, (b,));
    let x = &List(__);
    let __ = &(b, (a,));
    let y = &List(__);

    let __ = &(x, (y,));
    let either = &Either(__);
    let any = &Any(either);

    assert!(any.parse("").is_none());
    assert!(any.parse("a").is_none());
    assert!(any.parse("b").is_none());
    assert_eq!(any.parse("ab"), Some(vec![A(("a", ("b",)))]));
    assert_eq!(any.parse("ba"), Some(vec![B(("b", ("a",)))]));
    assert_eq!(
        any.parse("abbaabbaba"),
        Some(vec![
            A(("a", ("b",))),
            B(("b", ("a",))),
            A(("a", ("b",))),
            B(("b", ("a",))),
            B(("b", ("a",))),
        ])
    );
}
