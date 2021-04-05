type Opt<'a> = (&'static str, fn(&mut Options<'a>));

pub fn parse_args<'a, A, S>(args: &'a A) -> Result<Options<'a>>
where
    &'a A: IntoIterator<Item = &'a S>,
    S: AsRef<str> + 'a,
{
    let mut i = args.into_iter().skip(1);
    let mut opts = Options {
        input: Input::Stdin,
        eval: true,
        export_ast: false,
        named_args: <_>::default(),
        err_server: <_>::default(),
        err_server_clear: <_>::default(),
        err_server_quit: <_>::default(),
    };
    loop {
        match i.next().map(<_>::as_ref) {
            None => break,
            Some("--error-server") => {
                opts.err_server = true;
            }
            Some("--error-server-clear") => {
                opts.err_server_clear = true;
            }
            Some("--error-server-quit") => {
                opts.err_server_quit = true;
            }
            Some("-f") => {
                let filepath = i.next();
                let filepath =
                    te!(filepath, "Missing argument to -f").as_ref();
                opts.input = Input::File(filepath);
            }
            Some("-c") => {
                let script = te!(i.next(), "Missing argument to -c").as_ref();
                opts.input = Input::Str(script);
            }
            Some("-s") => {
                opts.eval = false;
            }
            Some("-a") => {
                opts.export_ast = true;
            }
            Some("-ah") => {
                let name = i.next();
                let name = te!(name, "Missing name to -ah");
                let sval = i.next();
                let sval = te!(sval, "Missing argument to -ah");

                opts.named_args.insert(name.as_ref(), sval.as_ref());
            }
            Some(other) => err!(f!("Unknown argument: {:#?}", other)),
        }
    }
    Ok(opts)
}

pub struct Options<'a> {
    pub input: Input<'a>,
    pub eval: bool,
    pub export_ast: bool,
    pub named_args: Map<&'a str, &'a str>,
    pub err_server: bool,
    pub err_server_clear: bool,
    pub err_server_quit: bool,
}

use super::*;
use std::format as f;
