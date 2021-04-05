type __ =
    super::__<(Error, Result<()>, Entry<'static, (), ()>, Cow<'static, ()>)>;

fn __<T>() -> Result<T> {
    te!(Err(xerr!("")));
    err!("");
}

use super::*;
