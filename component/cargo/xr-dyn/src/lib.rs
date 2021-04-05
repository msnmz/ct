pub const VERSION: &str = "0.0.1";

use std::{
    borrow::Cow,
    collections::BTreeMap as Map,
    marker::PhantomData as __,
};

pub fn load_str<I: AsRef<str>>(source: &I) -> ron::Result<GrammarTable> {
    ron::from_str(source.as_ref())
}

#[cfg(test)]
mod __test {
    use super::*;
    use std::{
        fs,
        io,
        result::Result as Either,
    };
    type Result<T> =
        Either<T, Either<io::Error, Either<ron::Error, Either<(), ()>>>>;
    #[test]
    fn test() -> Result<()> {
        #[allow(non_snake_case)]
        let Io = |err| Ok(err);
        #[allow(non_snake_case)]
        let Ron = |err| Err(Ok(err));
        #[allow(non_snake_case)]
        let Non = |err| Err(Err(Ok(err)));

        let mut file =
            fs::File::open("../xr/src/lib/lib/parse_ruby.ron").map_err(Io)?;
        let mut string = String::new();
        io::Read::read_to_string(&mut file, &mut string).map_err(Io)?;

        load_str(&string)
            .map_err(Ron)
            .and_then(|GrammarTable(productions)| {
                productions
                    .get("Program")
                    .map(|_| ())
                    .ok_or_else(|| Non(()))
            })
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct GrammarTable<'s>(
    #[serde(borrow)] pub Cow<'s, Map<Cow<'s, str>, Cow<'s, Production<'s>>>>,
);

#[derive(serde::Deserialize, Debug, Clone)]
pub enum Production<'s> {
    #[serde(borrow)]
    Token(Property<'s>),
    #[serde(borrow)]
    List(List<'s>),
    #[serde(borrow)]
    Any(Any<'s>),
    #[serde(borrow)]
    Either(Either<'s>),
}
#[derive(serde::Deserialize, Debug, Clone)]
pub enum Property<'s> {
    #[serde(borrow)]
    Exact(Slicy<'s>),
    #[serde(borrow)]
    Class(Predicate<'s>),
}

pub type RuleRef<'s> = Cow<'s, str>;
pub type RulesRefs<'s> = Cow<'s, [RuleRef<'s>]>;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct List<'s>(#[serde(borrow)] pub RulesRefs<'s>);
#[derive(serde::Deserialize, Debug, Clone)]
pub struct Either<'s>(#[serde(borrow)] pub RulesRefs<'s>);
#[derive(serde::Deserialize, Debug, Clone)]
pub struct Any<'s>(#[serde(borrow)] pub RuleRef<'s>);

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Slicy<'s> {
    #[serde(borrow)]
    SliceChar(Cow<'s, [char]>),
    #[serde(borrow)]
    Str(Cow<'s, str>),
    Char(char),
}

#[derive(serde::Deserialize, Debug, Clone)]
pub enum Predicate<'s> {
    Lt(usize),
    Eq(char),
    Prop(CharProp<'s>),
    #[serde(borrow)]
    Not(Box<Predicate<'s>>),
    #[serde(borrow)]
    All(Vec<Predicate<'s>>),
    #[serde(borrow)]
    Either(Vec<Predicate<'s>>),
}
#[derive(serde::Deserialize, Debug, Clone)]
pub enum CharProp<'s> {
    #[serde(rename = "is_alphabetic")]
    IsAlphabetic,
    #[serde(rename = "is_alphanumeric")]
    IsAlphanumeric,
    #[serde(rename = "is_ascii_whitespace")]
    IsAsciiWhitespace,

    __(#[serde(default)] __<&'s ()>),
}
