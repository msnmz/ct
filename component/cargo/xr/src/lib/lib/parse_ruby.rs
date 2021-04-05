pub type Stmt<_Fsl, _Nl0, _Nl1> =
    Either<List![StmtExpr<_Fsl, _Nl0>, StmtEmpty<_Nl1>]>;
pub fn stmt<S>(
) -> Stmt<impl tkn::Token<S>, impl tkn::Token<S>, impl tkn::Token<S>>
where
    S: Slicy + ?Sized + 'static,
    S::Item: PartialEq<char>,
{
    let __ = list![stmt_expr(), stmt_empty()];
    Either(__)
}
pub type StmtEmpty<_Nl> = List<List![Nl<_Nl>]>;
pub fn stmt_empty<S>() -> StmtEmpty<impl tkn::Token<S>>
where
    S: Slicy + ?Sized + 'static,
    S::Item: PartialEq<char>,
{
    let __ = list![nl()];
    List(__)
}
pub type StmtExpr<Fsl, _Nl> = List<List![StmtInner<Fsl>, Nl<_Nl>]>;
pub fn stmt_expr<S>() -> StmtExpr<impl tkn::Token<S>, impl tkn::Token<S>>
where
    S: Slicy + ?Sized + 'static,
    S::Item: PartialEq<char>,
{
    let __ = list![stmt_inner(), nl()];
    List(__)
}
pub type StmtInner<_Fsl> = Either<List![Fsl<_Fsl>]>;
pub fn stmt_inner<S>() -> StmtInner<impl tkn::Token<S>>
where
    S: Slicy + ?Sized + 'static,
    S::Item: PartialEq<char>,
{
    let __ = list![fsl()];
    Either(__)
}
pub type Nl<T> = Token<T>;
pub fn nl<S>() -> Nl<impl tkn::Token<S>>
where
    S: ?Sized + Slicy + 'static,
    S::Item: PartialEq<char>,
{
    Token(exact(&'\n'))
}
pub type Fsl<T> = Token<T>;
pub fn fsl<S>() -> Fsl<impl tkn::Token<S>>
where
    S: ?Sized + Slicy + 'static,
    S::Item: PartialEq<char>,
{
    Token(exact("# frozen_string_literal: true"))
}
//pub fn surrounded<'s, S, T, P>(token: T, parser: P) -> impl Parser<Output =
//ParseOut<'s, S, List<List![T,P,T]>>
//>
//{
//}

use {
    wind_std_list::{
        list,
        List,
    },
    //},
    xr_parse::{
        //either::either::{
        //    A,
        //    B,
        //},
        Any,
        Either,
        List,
        Parser,
        Token,
    },
    xr_token::{
        self as tkn,
        class,
        //eq,
        exact,
        //not,
        //Prop,
        Slicy,
        Spanned,
    },
};

use super::{
    xerr,
    Error,
    Result,
};

pub fn new() -> impl RubyEngine {}

pub trait RubyEngine {
    fn load_as_file(&mut self, path: &str, content: &[u8]) -> Result<()>;
}

impl RubyEngine for () {
    fn load_as_file(&mut self, path: &str, content: &[u8]) -> Result<()> {
        let _ = (path,);
        log::info!("Parsing: {}", path);

        let id = &Token(class(|i, t: &char| {
            i == 0 && t.is_alphabetic() || t.is_alphanumeric()
        }));
        //let QT = &Token(exact(&'\''));
        let stmt = &stmt();

        let __ = &list![stmt, stmt, id, &Any(Token(exact(" ")))];
        let __ = &List(__);

        let rb_program = __;
        let content = std::str::from_utf8(content).unwrap();
        let r = Parser::parse(&rb_program, content);
        log::debug!(
            "Result: {:#?} {:#?}",
            r, //.map(|p| p.span().scope(content)),
            r.as_ref().map(|p| &p.span().descope(content)[0..50])
        );

        Err(xerr!("undone"))
    }
}
