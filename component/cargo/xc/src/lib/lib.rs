pub mod error;
pub mod ssh_config;
#[cfg(feature = "syn")]
pub mod terrarust;
pub mod vagrant_machine_readable_quote_unquote;
pub mod version;

const RON: &str = "r";
const JSON: &str = "j";
const YAML: &str = "y";
const TOML: &str = "m";
#[cfg(feature = "syn")]
const TERRARUST: &str = "t";
const SSH_CONFIG: &str = "h";
const VAGRANT: &str = "v";
const STRING: &str = "s";
const BYTES: &str = "b";
const NULL_SEPARATED: &str = "0";

pub fn app<I: io::Read, O: io::Write>(
    args: Vec<String>,
    mut stdin: I,
    mut stdout: O,
) -> Result<()> {
    let mode = args.get(1).map(String::as_str);

    let mut buffer = String::new();
    let in_size = io::Read::read_to_string(&mut stdin, &mut buffer)?;
    log::info!("IN: {in_size}", in_size = in_size);

    fn from(mode: &str) -> &str {
        mode.get(0..1).unwrap_or(RON)
    }
    fn to(mode: &str) -> &str {
        mode.get(1..2).unwrap_or(JSON)
    }

    fn do_direct<O: io::Write>(
        mode: &str,
        buffer: &str,
        mut stdout: O,
    ) -> Result<()> {
        match to(mode) {
            BYTES => {
                let mut separator = "";

                write!(stdout, "[")?;
                for byte in buffer.bytes() {
                    write!(stdout, "{}{}", separator, byte)?;
                    separator = ", ";
                }
                write!(stdout, "]")?;

                Ok(())
            }
            NULL_SEPARATED => {
                buffer.lines().fold("", |separator, line| {
                    let _ = write!(stdout, "{}{}", separator, line);
                    "\x00"
                });
                Ok(())
            }
            other => {
                return Err(
                    format!("Unknown how to do direct to {:?}", other).into()
                )
            }
        }
    }
    fn do_trans<'de, D: Deserializer<'de>, S: Serializer>(
        d: D,
        s: S,
    ) -> Result<S::Ok>
    where
        S::Error: Into<ErrorKind>,
    {
        Ok(serde_transcode::transcode(d, s)?)
    }
    fn do_to<'de, D: Deserializer<'de>, O: io::Write>(
        mode: &str,
        d: D,
        stdout: O,
    ) -> Result<()> {
        match to(mode) {
            RON => {
                let opts = Some(ron::ser::PrettyConfig::new());
                let struct_names = true;
                do_trans(
                    d,
                    &mut ron::Serializer::new(stdout, opts, struct_names)?,
                )
            }
            JSON => do_trans(d, &mut json::Serializer::new(stdout)),
            YAML => do_trans(d, &mut yaml::Serializer::new(stdout)),
            other => {
                return Err(
                    format!("Unknown to: {other}", other = other).into()
                )
            }
        }
    }
    fn do_from<O: io::Write>(
        mode: &str,
        buffer: &str,
        stdout: O,
    ) -> Result<()> {
        match from(mode) {
            STRING => do_direct(mode, buffer, stdout),
            RON => do_to(
                mode,
                &mut ron::Deserializer::from_bytes(buffer.as_bytes())?,
                stdout,
            ),
            JSON => do_to(
                mode,
                &mut json::Deserializer::from_slice(buffer.as_bytes()),
                stdout,
            ),
            YAML => do_to(
                mode,
                yaml::Deserializer::from_slice(buffer.as_bytes()),
                stdout,
            ),
            TOML => do_to(mode, &mut toml::Deserializer::new(buffer), stdout),
            #[cfg(feature = "syn")]
            TERRARUST => {
                let mut bytes = Vec::<u8>::new();
                terrarust::parse_compile(buffer, &mut bytes)?;

                let opts = bincode::options();
                let deser =
                    &mut bincode::Deserializer::from_slice(&bytes, opts);

                let _ = deser;
                let deser =
                    &mut serde_cbor::Deserializer::from_mut_slice(&mut bytes);

                do_to(mode, deser, stdout)
            }
            SSH_CONFIG => {
                let mut bytes = Vec::<u8>::new();
                ssh_config::parse_compile(buffer, &mut bytes)?;

                let opts = bincode::options();
                let deser =
                    &mut bincode::Deserializer::from_slice(&bytes, opts);

                let _ = deser;
                let deser =
                    &mut serde_cbor::Deserializer::from_mut_slice(&mut bytes);

                do_to(mode, deser, stdout)
            }
            VAGRANT => {
                let mut bytes = Vec::<u8>::new();
                vagrant_machine_readable_quote_unquote::parse_compile(
                    buffer, &mut bytes,
                )?;

                let opts = bincode::options();
                let deser =
                    &mut bincode::Deserializer::from_slice(&bytes, opts);

                let _ = deser;
                let deser =
                    &mut serde_cbor::Deserializer::from_mut_slice(&mut bytes);

                do_to(mode, deser, stdout)
            }
            other => {
                return Err(
                    format!("Unknown from: {other}", other = other).into()
                )
            }
        }
    }

    do_from(mode.unwrap_or(""), &buffer, &mut stdout)
}

pub use {
    error::{
        Error,
        ErrorKind,
        Result,
    },
    serde::{
        de::Deserializer,
        ser::Serializer,
    },
    serde_cbor as cbor,
    serde_json as json,
    serde_yaml as yaml,
    std::{
        collections::{
            BTreeMap as Map,
            VecDeque as Deq,
        },
        env,
        fmt,
        format_args as f,
        io,
    },
};
