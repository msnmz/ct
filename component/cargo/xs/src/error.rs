xError! {
    [Debug]
    Io = io::Error
    Ron = ron::Error
    Utf8 = std::str::Utf8Error
    Utf8_ = std::string::FromUtf8Error
    Json = json::Error
    Xc = xc::lib::lib::Error
    Var = std::env::VarError
}

const ERR_SERVER_ADDR_VAR_NAME: &str = "XERR_TCP";
const CMD_CLEAR: &[u8] = b"@__XERR_CMD__CLEAR";
const CMD_QUIT: &[u8] = b"@__XERR_CMD__QUIT";

pub fn main(main: fn() -> Result<()>) -> i32 {
    match main() {
        Ok(()) => 0,
        Err(e) => {
            let mut sock = loop {
                if let Ok(addr) = env::var(ERR_SERVER_ADDR_VAR_NAME) {
                    if let Ok(sock) = net::TcpStream::connect(addr) {
                        break Ok(sock);
                    }
                }
                break Err(());
            };

            use io::Write;
            xerr::v2::show_trace(e, |a| {
                let msg = format!("{}\n", a);
                let bytes = msg.as_bytes();

                sock.as_mut()
                    .map_err(|_| ())
                    .and_then(|sock| sock.write_all(bytes).map_err(|_| ()))
                    .unwrap_or_else(|_| io::stderr().write_all(bytes).unwrap())
            });

            1
        }
    }
}

#[derive(Debug)]
pub struct ErrorServer {
    addr: String,
    buf: Vec<u8>,
    stderr: io::Stderr,
}
impl ErrorServer {
    pub fn init_from_env() -> Result<Self> {
        let addr = te!(
            env::var(ERR_SERVER_ADDR_VAR_NAME),
            format!("Missing ENV: {}", ERR_SERVER_ADDR_VAR_NAME)
        );
        log::trace!(
            "Address detected from {}: {}",
            ERR_SERVER_ADDR_VAR_NAME,
            addr
        );

        let buf = Vec::new();
        let stderr = io::stderr();

        Ok(Self { addr, buf, stderr })
    }

    // TODO: make a send_cmd()
    pub fn quit(&mut self) -> Result<()> {
        let Self { addr, .. } = self;
        let mut conn = te!(
            net::TcpStream::connect(addr.clone()),
            format!("Failed to connect to {:?}", addr)
        );
        te!(io::Write::write_all(&mut conn, CMD_QUIT), "");
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        let Self { addr, .. } = self;
        let mut conn = te!(
            net::TcpStream::connect(addr.clone()),
            format!("Failed to connect to {:?}", addr)
        );
        te!(
            io::Write::write_all(&mut conn, CMD_CLEAR),
            format!("Writing clear command")
        );
        Ok(())
    }

    pub fn serve(self) -> Result<()> {
        log::info!("--- Welcome to log server ---");

        let Self {
            addr,
            mut buf,
            mut stderr,
        } = self;

        let listen = te!(
            net::TcpListener::bind(&addr),
            format!("Failed to bind to {}", addr)
        );
        log::trace!("Listening to {}", addr);

        loop {
            log::trace!("Accepting on {:?}", addr);
            let (mut client, client_addr) = te!(
                listen.accept(),
                format!("Accepting client on {:?}", listen)
            );
            log::debug!("Accept: {:?}", client_addr);

            buf.clear();
            log::trace!("Reading");
            let read = te!(
                io::Read::read_to_end(&mut client, &mut buf),
                format!("Reading client data: {:?}", client_addr)
            );
            log::debug!("Read {} from {:?}", read, client_addr);

            if buf == CMD_CLEAR {
                // TODO how??? (without another process call)
                te!(std::process::Command::new("clear").status());
            } else if buf == CMD_QUIT {
                break;
            } else {
                te!(
                    io::Write::write_all(&mut stderr, &buf),
                    format!("Writing to stderr")
                );
            }

            if buf.is_empty() {
                break;
            }
        }
        Ok(())
    }
}

pub fn run_error_server() -> Result<()> {
    te!(ErrorServer::init_from_env()).serve()
}
pub fn clear_error_server() -> Result<()> {
    if let Ok(mut server) = ErrorServer::init_from_env() {
        te!(
            server.clear(),
            format!("Failed to send clear to {:?}", server)
        );
    } else {
        log::warn!("Did not connect to an xerr log server");
    }
    Ok(())
}
pub fn quit() -> Result<()> {
    if let Ok(mut server) = ErrorServer::init_from_env() {
        te!(
            server.quit(),
            format!("Failed to send clear to {:?}", server)
        );
    } else {
        log::warn!("Did not connect to an xerr log server");
    }
    Ok(())
}

impl xerr::v2::ErrorShow for ErrorKind {
    fn show(self) -> String {
        match self {
            ErrorKind::Msg(s) => s,
            other => format!("{:?}", other),
        }
    }
}

pub use xerr::{
    err2 as err,
    te2 as te,
    xError2 as xError,
    xerr2 as xerr,
};

use super::*;
