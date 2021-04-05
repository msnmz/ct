#[derive(Debug)]
pub enum Input<'a> {
    Str(&'a str),
    Stdin,
    File(&'a str),
}

impl<'a> Input<'a> {
    pub fn open(&self) -> Result<Box<dyn io::Read + 'a>> {
        use Input::*;
        Ok(match self {
            Str(s) => Box::new(s.as_bytes()),
            File(p) => Box::new(strip_shebang(io::BufReader::new(te!(
                fs::File::open(p),
                format!("Input file: {}", p)
            )))),
            Stdin => Box::new(io::stdin()),
        })
    }
}

pub fn strip_shebang<R: io::BufRead>(inner: R) -> impl io::Read {
    #[derive(Debug, Default)]
    pub struct Mem(Option<usize>, Option<usize>);
    struct SheBangStripper<R: io::Read> {
        inner: R,
        mem: Mem,
    }

    impl<R: io::BufRead> io::Read for SheBangStripper<R> {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let Self { inner, mem } = self;

            loop {
                match mem {
                    Mem(s @ None, b @ None) => {
                        log::debug!("None None");

                        let buf = inner.fill_buf()?;
                        if buf.len() == 0 {
                            *s = Some(0);
                            *b = Some(0);
                            break;
                        }
                        if buf.len() < 2 {
                            continue;
                        }

                        *s = Some(0);
                        *b = if buf[0..2].eq(b"#!") { None } else { Some(0) };
                    }
                    Mem(Some(_a), s @ None) => {
                        log::debug!("Some({}) None", _a);

                        let buf = inner.fill_buf()?;
                        *s = buf.iter().position(|&b| b == b'\n');
                    }
                    Mem(Some(0), Some(0)) => {
                        log::trace!("Some(0) Some(0)");
                        break;
                    }
                    Mem(Some(0), Some(b)) => {
                        log::debug!("Some(0) Some({})", b);
                        inner.consume(*b);
                        *b = 0;
                    }
                    other => panic!("Unhandlable {:?}", other),
                }
            }

            inner.read(buffer)
        }
    }

    SheBangStripper {
        inner,
        mem: <_>::default(),
    }
}

impl<'a> Input<'a> {
    pub fn to_display(&self) -> impl fmt::Display + '_ {
        match self {
            Input::File(path) => path,
            Input::Stdin => "stdin",
            Input::Str(s) => s,
        }
    }
}

use super::*;
