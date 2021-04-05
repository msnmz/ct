#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Script<'d>(#[serde(borrow)] pub Deq<Stmt<'d>>);

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub enum Stmt<'d> {
    Exec {
        cmd: &'d str,
        #[serde(default)]
        args: Deq<Expr<'d>>,
        #[serde(default)]
        cwd: Option<&'d str>,
        #[serde(default)]
        env: Map<&'d str, Expr<'d>>,
        #[serde(default)]
        output: Type,
        #[serde(default)]
        allow_failure: bool,
        #[serde(default)]
        stdin: Io<'d>,
    },

    ForEach(&'d str, Box<Stmt<'d>>), // TODO: Expr
    Loop(Box<Stmt<'d>>),             // TODO: Expr
    Let(&'d str, Box<Stmt<'d>>),     // TODO: Expr
    Alias(&'d str, Box<Stmt<'d>>),
    // Call the alias
    AliasStmt(&'d str),
    // TODO obsolete
    Clone(&'d str),
    Expr(Expr<'d>),
    List(Vec<Stmt<'d>>),
    WriteFile(Expr<'d>, Expr<'d>),
    WriteValue(Expr<'d>, Expr<'d>),
}

#[derive(
    Debug, Clone, serde::Deserialize, serde::Serialize, PartialEq, Eq,
)]
#[serde(untagged)]
pub enum Expr<'d> {
    Str(&'d str),
    Str2 { s: &'d str },
    Bs { bs: Cow<'d, [u8]> },
    Each,
    Var { var: &'d str },
    ReadSource { source: &'d str },
    List(Vec<Expr<'d>>),
}

#[derive(
    Debug, Clone, serde::Deserialize, serde::Serialize, wind::EnumDefault,
)]
pub enum Type {
    #[default]
    Null,
    VBoxVms,
    String,
    Display,
    Stream,
}

#[derive(
    Debug,
    Clone,
    serde::Deserialize,
    serde::Serialize,
    wind::EnumDefault,
    PartialEq,
    Eq,
)]
pub enum Io<'d> {
    #[default]
    Default,
    File(#[serde(borrow)] Expr<'d>),
    Tty,
    Source(&'d str),
}

use super::*;
