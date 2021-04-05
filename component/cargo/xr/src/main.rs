mod lib;
use lib::*;

pub fn main() {
    pretty_env_logger::init();

    match lib::app() {
        Ok(()) => (),
        Err(e) => {
            xerr::v2::show_trace(e, |a| eprintln!("{}", a));
            impl xerr::v2::ErrorShow for ErrorKind {
                fn show(self) -> String {
                    format!("{:?}", self)
                }
            }

            std::process::exit(1);
        }
    }
}
