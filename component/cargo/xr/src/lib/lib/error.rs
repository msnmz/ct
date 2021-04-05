pub use xerr::{
    err2 as err,
    te2 as te,
    xError2 as xError,
    xerr2 as xerr,
};

#[cfg(not(feature = "git2"))]
mod git2 {
    pub struct Error;
}
xError! {
    [Debug]

    Git = git2::Error
    Io = io::Error
    Utf8 = std::str::Utf8Error
    Utf8_ = string::FromUtf8Error
}

use super::{
    io,
    string,
};
