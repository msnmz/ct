pub fn parse_compile<I: AsRef<str>, O: io::Write>(
    input: I,
    output: O,
) -> Result<()> {
    let ast: Ast = syn::parse_str(input.as_ref())?;
    cbor::to_writer(output, &ast)?;
    Ok(())
}

#[derive(serde::Serialize)]
struct Ast(pub mdl_terraform::Document);

struct P {
    pub tf: syn::Result<mdl_terraform::Document>,
}

use syn::visit::Visit;

impl<'a> Visit<'a> for P {
    fn visit_local(&mut self, local: &syn::Local) {
        let tf = match &mut self.tf {
            Ok(tf) => tf,
            _ => return,
        };

        let name = match &local.pat {
            syn::Pat::Ident(syn::PatIdent { ident, .. }) => ident,
            other => {
                return self.tf = err(
                    other,
                    f!("Unknown pattern in let: {other:?}", other = other),
                )
            }
        };

        let (typ, fields) =
            match &local.init.as_ref().map(|(_, bx)| bx.as_ref()) {
                Some(syn::Expr::Struct(syn::ExprStruct {
                    path: syn::Path { segments, .. },
                    fields,
                    ..
                })) => (segments, fields),
                other => {
                    return self.tf = err(
                        other,
                        f!("Unknown expr in let: {other:?}", other = other),
                    )
                }
            };

        let typ = match typ.first() {
            Some(first) => &first.ident,
            other => {
                return self.tf =
                    err(other, f!("Weird type: {other:?}", other = other))
            }
        };

        let fields = fields
            .iter()
            .map(|syn::FieldValue { expr, member, .. }| match member {
                syn::Member::Named(id) => match expr {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit),
                        ..
                    }) => Ok((id, lit)),
                    other => err(
                        other,
                        f!("Invalid expr: {other:?}", other = other),
                    ),
                },
                other => {
                    err(other, f!("Invalid field: {other:?}", other = other))
                }
            })
            .collect::<syn::Result<Vec<_>>>();
        let fields = match fields {
            Ok(fields) => fields,
            Err(e) => return self.tf = Err(e),
        };

        let name_str = name.to_string();
        let typ_str = typ.to_string();

        let value = tf.input.entry(name_str).or_insert_with(<_>::default);

        value.insert("type".to_owned(), typ_str.into());
        for (k, v) in fields {
            if value.insert(k.to_string(), v.value().into()).is_some() {
                return self.tf =
                    err(v, f!("Reinserted: {k:?} {v:?}", k = k, v = v));
            }
        }
    }
}

impl syn::parse::Parse for Ast {
    fn parse(input: &syn::parse::ParseBuffer) -> syn::Result<Self> {
        let root: syn::ItemFn = input.parse()?;
        let mut p = P {
            tf: Ok(<_>::default()),
        };
        p.visit_block(&root.block);
        let tf: mdl_terraform::Document = p.tf?;
        let ast = Ast(tf);
        Ok(ast)
    }
}

pub fn err<T, S: syn::spanned::Spanned, M: fmt::Display>(
    span: S,
    msg: M,
) -> syn::Result<T> {
    Err(error(span, msg))
}
pub fn error<S: syn::spanned::Spanned, M: fmt::Display>(
    span: S,
    msg: M,
) -> syn::Error {
    syn::Error::new(span.span(), msg)
}

use super::*;
