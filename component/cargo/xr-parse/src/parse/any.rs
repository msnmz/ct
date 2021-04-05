// TODO: gvz (also parse token)
use super::*;

pub struct Any<P>(pub P);

impl<'s, P, S> Parser<'s, S> for Any<P>
where
    P: Parser<'s, S>,
    S: Slicy + ?Sized,
{
    type Output = Vec<ParseOut<'s, S, P>>;
    fn parse(&self, inp: &'s S) -> Option<Self::Output> {
        let Self(parser) = self;

        if let Some(p) = parser.parse(inp) {
            let mut inp = p.span().descope(inp);
            let mut result = vec![p];

            while let Some(p) = parser.parse(inp) {
                inp = p.span().descope(inp);
                result.push(p);
            }

            return Some(result);
        }

        None
    }
}

#[test]
fn test_any_either() {
    let a = &Token(xr_token::exact(&'a'));
    let any = &Any(a);

    assert!(any.parse("").is_none());
    assert_eq!(any.parse("a"), Some(vec!["a"]));
    assert_eq!(any.parse("aa"), Some(vec!["a", "a"]));
    assert_eq!(any.parse("aaabaa"), Some(vec!["a", "a", "a"]));
}
