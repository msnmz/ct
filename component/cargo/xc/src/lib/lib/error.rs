macro_rules! impl_from {
    ($n:ident, $t:ty) => {
        impl From<$t> for ErrorKind {
            fn from(err: $t) -> Self {
                Self::$n(err)
            }
        }
    };
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

impl<T> From<T> for Error
where
    T: Into<ErrorKind>,
{
    fn from(err: T) -> Self {
        Self { kind: err.into() }
    }
}
pub type Result<T> = std::result::Result<T, Error>;

// TODO derive impl-from in wind
impl_from!(Io, io::Error);
impl_from!(Ron, ron::Error);
impl_from!(Json, json::Error);
impl_from!(Yaml, yaml::Error);
impl_from!(Utf8, std::str::Utf8Error);
#[cfg(feature = "syn")]
impl_from!(Syn, syn::Error);
impl_from!(Cbor, cbor::Error);
impl_from!(Msg, String);

#[derive(Debug)]
pub enum ErrorKind {
    Io(io::Error),
    Ron(ron::Error),
    Json(json::Error),
    Yaml(yaml::Error),
    Utf8(std::str::Utf8Error),
    #[cfg(feature = "syn")]
    Syn(syn::Error),
    Cbor(cbor::Error),
    Msg(String),
}

use super::*;
