mod prop;
mod slicy;
mod span;

pub use {
    prop::Prop,
    slicy::Slicy,
    span::{
        Span,
        Spanned,
    },
};

pub trait Token<S>
where
    Self: Prop<S::Item>,
    S: Slicy + ?Sized,
{
    fn try_match(&self, source: &S) -> Span {
        Span(0, source.count_range_of(self))
    }
}

impl<Self_, S> Token<S> for Self_
where
    Self: Prop<S::Item>,
    S: Slicy + ?Sized,
{
}

pub fn class<S, P>(prop: P) -> impl Token<S>
where
    S: Slicy + ?Sized,
    P: Prop<S::Item>,
{
    prop
}

pub fn exact<'u, S, U>(exact: &'u U) -> impl Token<S> + 'u
where
    S: Slicy + ?Sized + 'u,
    U: Slicy + ?Sized,
    S::Item: PartialEq<U::Item>,
{
    class(move |i, t: &_| i < exact.len() && exact.subopen(i).peek(|s| t == s))
}

pub fn eq<T>(s: &T) -> impl Prop<T> + '_
where
    T: Eq,
{
    move |_, t: &_| t == s
}

pub fn not<P, T>(p: P) -> impl Prop<T>
where
    P: Prop<T>,
{
    move |i, t: &_| !p(i, t)
}
pub fn lt<T>(n: &T) -> impl Prop<T> + '_
where
    T: PartialOrd<usize>,
{
    move |i, _: &_| *n < i
}
