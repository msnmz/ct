pub trait Prop<T>
where
    Self: Fn(usize, &T) -> bool,
{
    fn check(&self, i: usize, t: &T) -> bool {
        self(i, t)
    }
}
impl<T, P: Fn(usize, &T) -> bool> Prop<T> for P {}
