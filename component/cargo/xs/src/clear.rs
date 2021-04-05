pub trait Clear {
    fn clear(&mut self);
}

pub fn clearing<C: Clear>(mut c: C) -> C {
    c.clear();
    c
}

impl Clear for String {
    fn clear(&mut self) {
        String::clear(self)
    }
}
impl<T> Clear for Vec<T> {
    fn clear(&mut self) {
        Vec::clear(self)
    }
}
