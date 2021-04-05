mod error;
pub mod parse_ruby;
mod ruby_sources;
mod version;

pub fn app() -> Result<()> {
    {
        let (a, b) = version::BUILD_ID.split_at(8);
        log::info!(
            concat!(
                "[<<:: xr ::>>] -:",
                "\x1b[38;5;111m[\x1b[38;5;112;1m{SHORT_SLUG}\x1b[m\x1b[38;5;111m]",
                "\x1b[38;5;240m{REST_SLUG}\x1b[m"
            ),
            SHORT_SLUG = a,
            REST_SLUG = b
        );
    }

    use parse_ruby::RubyEngine;
    let mut rb = parse_ruby::new();

    te!(ruby_sources::find_rubies_in_repo(
        ruby_sources::gh::airbrake::ID,
        |path, content| rb.load_as_file(path, content)
    ));

    Ok(())
}

#[macro_export]
macro_rules! stub {
    ($($f:tt)*) => {
        eprintln!("[WARN] {}:{} :: {}",
            file!(),
            line!(),
            format_args!($($f)*)
        )
    }
}

pub use {
    error::*,
    std::{
        fmt,
        fs,
        io,
        path::Path,
        rc::Rc,
        result,
        string,
    },
};
