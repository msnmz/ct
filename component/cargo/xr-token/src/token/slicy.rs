use super::Prop;

/// ## The `fmt::Debug` constraint
///
/// This constraint is purely artificial, to ease debugging in all
/// the terminal parsers, where Slicy is used.
///
/// It comes for free because it is completely encapsulated in the
/// simple `: Slicy` constraint that each parser has. At call site,
/// all primitive slicy types implement Debug anyway.
///
/// This note serves to say that if it a source of a problem, it
/// *MUST* be able to be removed. It affects (obviously) only Debugging
/// code.
///
pub trait Slicy: std::fmt::Debug {
    fn len(&self) -> usize;
    fn subslice(&self, s: usize, e: usize) -> &Self;
    fn subopen(&self, s: usize) -> &Self {
        self.subslice(s, self.len())
    }

    type Item;
    fn count_range_of<P: Prop<Self::Item>>(&self, prop: P) -> usize;

    fn peek<P>(&self, prop: P) -> bool
    where
        P: Fn(&Self::Item) -> bool,
    {
        self.count_range_of(|i, t: &_| i == 0 && prop(t)) == 1
    }
}

impl<T: std::fmt::Debug> Slicy for [T] {
    fn len(&self) -> usize {
        self.len()
    }
    fn subslice(&self, s: usize, e: usize) -> &Self {
        &self[s..e]
    }

    type Item = T;
    fn count_range_of<P: Prop<Self::Item>>(&self, prop: P) -> usize {
        count_range_of(self.iter(), |i, t: &&T| prop.check(i, t))
    }
}

impl Slicy for str {
    fn len(&self) -> usize {
        self.len()
    }
    fn subslice(&self, s: usize, e: usize) -> &Self {
        &self[s..e]
    }

    type Item = char;
    fn count_range_of<P: Prop<Self::Item>>(&self, prop: P) -> usize {
        count_range_of(self.chars(), prop)
    }
}

impl<T: std::fmt::Debug> Slicy for T {
    fn len(&self) -> usize {
        1
    }
    fn subslice(&self, s: usize, e: usize) -> &Self {
        if s == 0 && e == 1 {
            self
        } else {
            panic!("{} {}", s, e)
        }
    }

    type Item = T;
    fn count_range_of<P: Prop<Self::Item>>(&self, prop: P) -> usize {
        if prop.check(0, self) {
            1
        } else {
            0
        }
    }
}

fn count_range_of<I: IntoIterator, P: Prop<I::Item>>(
    iter: I,
    prop: P,
) -> usize {
    let mut i = 0;
    for item in iter.into_iter() {
        let t: &I::Item = &item;

        if prop.check(i, t) {
            i += 1;
        } else {
            break;
        }
    }
    i
}
