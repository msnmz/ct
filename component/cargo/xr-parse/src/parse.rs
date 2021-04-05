pub use {
    any::Any,
    either::Either,
    list::List,
    token::Token,
};

mod any;
pub mod either;
mod list;
mod token;
use xr_token::{
    Slicy,
    Spanned,
};

pub type ParseOut<'s, S, T> = <T as Parser<'s, S>>::Output;

pub trait Parser<'s, S: ?Sized> {
    type Output: Spanned;
    fn parse(&self, inp: &'s S) -> Option<Self::Output>;
}

pub trait Parse<'s, S: ?Sized>:
    Parser<'s, S, Output = Self> + Sized + Spanned
{
}

pub trait ParseOwned<S: ?Sized>: for<'r> Parse<'r, S> {}

impl<'a, 's, A, S> Parser<'s, S> for &'a A
where
    A: Parser<'s, S>,
    S: ?Sized,
{
    type Output = ParseOut<'s, S, A>;
    fn parse(&self, inp: &'s S) -> Option<Self::Output> {
        A::parse(*self, inp)
    }
}

impl<A, S: ?Sized> ParseOwned<S> for A where
    Self: for<'r> Parse<'r, S, Output = A>
{
}

impl<'s, A, S: ?Sized> Parse<'s, S> for A where
    Self: Parser<'s, S, Output = Self> + Sized + Spanned
{
}
