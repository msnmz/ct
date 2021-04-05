use super::Slicy;

#[derive(Debug)]
pub struct Span(pub usize, pub usize);
pub trait Spanned {
    fn span(&self) -> Span;
}

impl<'s, S> Spanned for &'s S
where
    S: Spanned + ?Sized,
{
    fn span(&self) -> Span {
        S::span(*self)
    }
}
impl<T> Spanned for [T] {
    fn span(&self) -> Span {
        Span(0, self.len())
    }
}
impl Spanned for str {
    fn span(&self) -> Span {
        Span(0, self.len())
    }
}
impl<A: Spanned> Spanned for (A,) {
    fn span(&self) -> Span {
        let (a,) = self;
        a.span()
    }
}
impl<A: Spanned, B: Spanned> Spanned for (A, B) {
    fn span(&self) -> Span {
        let (a, b) = self;
        let a = a.span();
        let b = b.span().place_after(&a);
        Span(a.0, b.1)
    }
}
impl<T: Spanned> Spanned for Vec<T> {
    fn span(&self) -> Span {
        self.iter().fold(Span(0, 0), |s, t| s.include(t))
    }
}
impl<A: Spanned, B: Spanned> Spanned for Result<A, B> {
    fn span(&self) -> Span {
        self.as_ref().map_or_else(|b| b.span(), |a| a.span())
    }
}

impl Span {
    pub fn scope<'s, S>(&self, s: &'s S) -> &'s S
    where
        S: Slicy + ?Sized,
    {
        let &Self(start, end) = self;
        s.subslice(start, end)
    }
    pub fn descope<'s, S>(&self, s: &'s S) -> &'s S
    where
        S: Slicy + ?Sized,
    {
        let &Self(_, end) = self;
        s.subopen(end)
    }

    pub fn place_after(&self, other: &Self) -> Self {
        let &Self(_, d) = other;
        self.fold(|a, b| Span(a + d, b + d))
    }
    pub fn include<N: Spanned>(&self, other: N) -> Self {
        self.fold(|a, b| Span(a, b + other.span().len()))
    }

    pub fn non_empty(&self) -> Option<&Self> {
        self.filter(|s| s.len() > 0)
    }

    pub fn len(&self) -> usize {
        self.fold(|a, b| b - a)
    }

    pub fn filter<P>(&self, pred: P) -> Option<&Self>
    where
        P: FnOnce(&Self) -> bool,
    {
        if pred(self) {
            Some(self)
        } else {
            None
        }
    }

    pub fn fold<F, Z>(&self, f: F) -> Z
    where
        F: FnOnce(usize, usize) -> Z,
    {
        let &Self(a, b) = self;
        f(a, b)
    }
}
