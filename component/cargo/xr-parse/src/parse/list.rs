use super::*;

pub struct List<L>(pub L);
use self::List as Rcrs;

impl<'s, 'p, S, H, T> Parser<'s, S> for Rcrs<(H, T)>
where
    H: Parser<'s, S>,
    for<'r> Rcrs<&'r T>: Parser<'s, S>,
    Rcrs<T>: Parser<'s, S>,
    for<'r> ParseOut<'s, S, Rcrs<&'r T>>: Into<ParseOut<'s, S, Rcrs<T>>>,
    S: Slicy + ?Sized,
    T: 'p,
{
    type Output = (ParseOut<'s, S, H>, ParseOut<'s, S, Rcrs<T>>);

    fn parse(&self, inp: &'s S) -> Option<Self::Output> {
        let Self((head, tail)) = self;

        head.parse(inp).and_then(|head| {
            let tail_inp = head.span().descope(inp);
            Rcrs(tail).parse(tail_inp).map(|tail| (head, tail.into()))
        })
    }
}
//impl<'s, 'p, S, H, T> Parser<'s, S> for Rcrs<(&'p H, &'p T)>
//where
//    H: Parser<'s, S> + ?Sized,
//    Rcrs<&'p T>: Parser<'s, S>,
//    S: Slicy + ?Sized,
//    T: 'p,
//{
//    type Output = (ParseOut<'s, S, H>, ParseOut<'s, S, Rcrs<&'p T>>);
//
//    fn parse(&self, inp: &'s S) -> Option<Self::Output> {
//        let &Self((head, tail)) = self;
//
//        head.parse(inp).and_then(|head| {
//            let tail_inp = head.span().descope(inp);
//            Rcrs(tail).parse(tail_inp).map(|tail| (head, tail))
//        })
//    }
//}
impl<'s, 'p, S, H, T> Parser<'s, S> for Rcrs<&'p (&'p H, T)>
where
    H: Parser<'s, S> + ?Sized,
    Rcrs<&'p T>: Parser<'s, S>,
    S: Slicy + ?Sized,
{
    type Output = (ParseOut<'s, S, H>, ParseOut<'s, S, Rcrs<&'p T>>);

    fn parse(&self, inp: &'s S) -> Option<Self::Output> {
        let Self((head, tail)) = self;

        head.parse(inp).and_then(|head| {
            let tail_inp = head.span().descope(inp);
            Rcrs(tail).parse(tail_inp).map(|tail| (head, tail))
        })
    }
}
impl<'s, 'p, S, H> Parser<'s, S> for Rcrs<(H,)>
where
    H: Parser<'s, S>,
    S: Slicy + ?Sized,
{
    type Output = (ParseOut<'s, S, H>,);

    fn parse(&self, inp: &'s S) -> Option<Self::Output> {
        let Self((head,)) = self;
        head.parse(inp).map(|head| (head,))
    }
}
impl<'s, 'p, S, H> Parser<'s, S> for Rcrs<&'p (H,)>
where
    H: Parser<'s, S> + ?Sized,
    S: Slicy + ?Sized,
{
    type Output = (ParseOut<'s, S, H>,);

    fn parse(&self, inp: &'s S) -> Option<Self::Output> {
        let Self((head,)) = self;
        head.parse(inp).map(|head| (head,))
    }
}

#[test]
fn test_list() {
    let a = &Token(xr_token::exact(&'a'));
    let b = &Token(xr_token::exact(&'b'));
    let c = &Token(xr_token::exact(&'c'));
    let __ = &(a, (b, (c,)));
    let either = &List(__);

    assert!(either.parse("").is_none());
    assert_eq!(either.parse("a"), None);
    assert_eq!(either.parse("ab"), None);
    assert_eq!(either.parse("abc"), Some(("a", ("b", ("c",)))));
    assert_eq!(either.parse("abcd"), Some(("a", ("b", ("c",)))));
}
