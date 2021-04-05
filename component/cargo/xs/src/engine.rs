#[derive(Debug, Default)]
pub struct Engine<'d> {
    pub r0: Value<'d>,
    pub variables: Map<String, Value<'d>>,
    pub aliases: Map<String, ast::Stmt<'d>>,
    pub each: Value<'d>,

    pub h0_proc: Option<ProcessHandleState>,
    park: Park,
}
impl<'d> Engine<'d> {
    pub fn r0_load_each(mut self) -> Self {
        mem::swap(&mut self.r0, &mut self.each);
        self
    }

    pub fn set_r0<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Park, Value<'d>) -> Value<'d>,
    {
        self.r0 = f(&mut self.park, self.r0);
        self
    }

    pub fn set_var<N, F>(mut self, name: N, f: F) -> Self
    where
        N: FnOnce(&mut Park, String) -> String,
        F: FnOnce(&mut Park, Value<'d>) -> Value<'d>,
    {
        let Self {
            mut park,
            mut variables,
            r0,
            ..
        } = self;

        let name = {
            let buf = park.unpark_string();
            name(&mut park, buf)
        };

        let value = variables.entry(name.clone()).or_default();
        *value = f(&mut park, r0);

        log::trace!("{}={:?}", name, value);
        park.strings.push(name);

        self.variables = variables;
        self.park = park;
        self.r0 = <_>::default();
        self
    }

    pub fn exec<'e>(
        &'e mut self,
        cmd: process::Command,
    ) -> ProcessHandle<'e, 'd>
    where
        'd: 'e,
    {
        ProcessHandle {
            ng: self,
            state: ProcessHandleState::Init(cmd),
        }
    }

    pub fn assume_child<'e>(
        &'e mut self,
        child: process::Child,
    ) -> ProcessHandle<'e, 'd>
    where
        'd: 'e,
    {
        ProcessHandle {
            ng: self,
            state: ProcessHandleState::Running(child),
        }
    }
}

#[derive(Debug, Default)]
pub struct Park {
    strings: Vec<String>,
}
impl Park {
    pub fn drain(&mut self, _value: Value) {}
    pub fn unpark<R>(source: &mut Vec<R>) -> R
    where
        R: clear::Clear + Default,
    {
        clearing(source.pop().unwrap_or_default())
    }
    pub fn unpark_string(&mut self) -> String {
        Self::unpark(&mut self.strings)
    }
}

#[derive(Debug)]
pub struct ProcessHandle<'e, 'd> {
    ng: &'e mut Engine<'d>,
    state: ProcessHandleState,
}
#[derive(Debug, wind::EnumDefault)]
pub enum ProcessHandleState {
    Init(process::Command),
    Running(process::Child),
    Pipeline(Vec<process::Child>),
    Done(process::Output),
    #[default]
    Dummy,
}

impl<'e, 'd> ProcessHandle<'e, 'd> {
    pub fn start(&mut self) -> Result<()> {
        use ProcessHandleState::*;

        Ok(match &mut self.state {
            Init(cmd) => {
                self.state =
                    Running(te!(cmd.spawn(), format!("Spawning {:?}", cmd)))
            }
            other => {
                log::warn!(
                    "start() request ignored for initialized process: {:#?}",
                    other
                );
            }
        })
    }
    pub fn wait(&mut self) -> Result<()> {
        use ProcessHandleState::*;

        let mut state = mem::take(&mut self.state);

        state = match state {
            Running(child) => Done(te!(child.wait_with_output())),
            other => {
                log::warn!(
                    "wait() request ignored for not running process: {:#?}",
                    other
                );
                other
            }
        };

        Ok(self.state = state)
    }
    pub fn complete(self) -> Result<process::Output> {
        use ProcessHandleState::*;

        Ok(match self.state {
            Done(output) => output,
            other => err!(f!("Process not complete: {:#?}", other)),
        })
    }
    pub fn complete_success(self) -> Result<process::Output> {
        let output = te!(self.complete());

        if !output.status.success() {
            return Err(xerr!(
                "Sub-process did not exit successfully: {:?}\n\n=== OUTPUT ===\n{}",
                output.status,
                String::from_utf8(output.stdout).unwrap_or_else(|_| "<not utf8 text>".to_owned())
            ));
        }

        // Use ng
        let _ = self.ng;
        Ok(output)
    }
    pub fn return_to_h0(self) -> Result<&'e mut Engine<'d>> {
        let Self { ng, state } = self;
        use ProcessHandleState::*;
        match (&mut ng.h0_proc, state) {
            (s @ None, Running(child)) => {
                *s = Some(Pipeline(vec![child]));
            }
            (Some(Pipeline(line)), Running(child)) => {
                line.push(child);
            }
            other => err!(format!("How do I {:?} ?", other)),
        }

        Ok(ng)
    }
}

#[derive(
    Debug, Clone, wind::EnumDefault, serde::Serialize, serde::Deserialize,
)]
#[serde(into = "JVal", try_from = "JVal")]
pub enum Value<'d> {
    #[default]
    Empty,
    Str(Cow<'d, str>),
    List(Vec<Value<'d>>),
    Bytes(Cow<'d, [u8]>),
}

#[derive(
    Debug, Clone, wind::EnumDefault, serde::Serialize, serde::Deserialize,
)]
#[serde(untagged)]
pub enum JVal<'d> {
    #[default]
    Null,
    Str(Cow<'d, str>),
    Arr(Vec<JVal<'d>>),
}

impl<'d> From<Value<'d>> for JVal<'d> {
    fn from(val: Value<'d>) -> Self {
        match val {
            Value::Empty => JVal::Null,
            Value::Str(s) => JVal::Str(s),
            Value::List(v) => {
                JVal::Arr(v.into_iter().map(<_>::into).collect())
            }
            Value::Bytes(_) => todo!(),
        }
    }
}
use std::result::Result as Rsl;
impl<'d> TryFrom<JVal<'d>> for Value<'d> {
    type Error = String;
    fn try_from(jval: JVal<'d>) -> Rsl<Self, String> {
        Ok(match jval {
            JVal::Null => Value::Empty,
            JVal::Str(s) => Value::Str(s),
            JVal::Arr(v) => Value::List(
                v.into_iter().map(<_>::try_into).collect::<Rsl<_, _>>()?,
            ),
        })
    }
}

fn parse_value<'d>(s: Cow<'d, str>) -> Value<'d> {
    // json::from_str(s.as_ref()).unwrap_or_else(|e| {
    //     eprintln!("Falling back to arg as string bcz {:?}", e);
    Value::Str(s)
    // })
}

impl<'d> From<&'d str> for Value<'d> {
    fn from(s: &'d str) -> Self {
        parse_value(Cow::from(s))
    }
}
impl<'d> From<String> for Value<'d> {
    fn from(s: String) -> Self {
        parse_value(Cow::from(s))
    }
}
impl<'d> From<Vec<Self>> for Value<'d> {
    fn from(v: Vec<Self>) -> Self {
        Self::List(v)
    }
}
impl<'d> From<Cow<'d, [u8]>> for Value<'d> {
    fn from(v: Cow<'d, [u8]>) -> Self {
        Self::Bytes(v)
    }
}

pub struct Strval<B>(pub B);
impl<B> Strval<B>
where
    B: BorrowMut<String> + Into<String> + From<String>,
{
    pub fn as_argument(self, val: &Value) -> Result<B> {
        let Self(mut buf) = self;
        Ok(match val {
            Value::Empty => {
                log::warn!("as_argument() for Empty");
                buf
            }
            Value::Str(s) => {
                buf.borrow_mut().push_str(s);
                buf
            }
            v @ Value::List(_) => {
                te!(Strval(buf).as_json(v))
            }
            other => {
                err!(format!("How to argument? {:?}", other))
            }
        })
    }
    pub fn as_json(self, val: &Value) -> Result<B> {
        let Self(buf) = self;
        let buf: String = buf.into();
        let mut buf: Vec<u8> = buf.into_bytes();
        use io::Write;

        let return_ = |buf| Ok(B::from(te!(String::from_utf8(buf))));

        Ok(match val {
            Value::Empty => {
                log::warn!("as_json() for Empty");
                te!(return_(buf))
            }
            Value::Str(s) => {
                te!(json::to_writer(&mut buf, s));
                te!(return_(buf))
            }
            Value::List(l) => {
                let mut sep = "";

                write!(buf, "[")?;
                for val in l {
                    write!(buf, "{}", sep)?;
                    sep = ",";
                    buf = te!(Strval(te!(return_(buf))).as_json(val))
                        .into()
                        .into_bytes();
                }
                write!(buf, "]")?;
                te!(return_(buf))
            }
            Value::Bytes(bs) => {
                te!(json::to_writer(&mut buf, bs));
                te!(return_(buf))
            }
        })
    }
}

impl<'a, 'd> TryFrom<&'a Value<'d>> for &'a str {
    type Error = Error;
    fn try_from(val: &'a Value<'d>) -> Result<&'a str> {
        match val {
            Value::Str(s) => Ok(s.as_ref()),
            other => Err(xerr!(f!("Value not a string: {:#?}", other))),
        }
    }
}
impl<'a, 'd> TryFrom<&'a Value<'d>> for &'a [Value<'d>] {
    type Error = Error;
    fn try_from(val: &'a Value<'d>) -> Result<&'a [Value<'d>]> {
        match val {
            Value::List(s) => Ok(s.as_ref()),
            other => Err(xerr!(f!("Value not a list: {:#?}", other))),
        }
    }
}
impl<'d> TryFrom<Value<'d>> for Vec<Value<'d>> {
    type Error = Error;
    fn try_from(val: Value<'d>) -> Result<Vec<Value<'d>>> {
        match val {
            Value::List(s) => Ok(s),
            other => Err(xerr!(f!("Value not a list: {:#?}", other))),
        }
    }
}

pub fn with_name(s: &str) -> impl FnOnce(&mut Park, String) -> String + '_ {
    move |_, mut st| {
        st.push_str(s);
        st
    }
}
pub fn with_val_id<'d>() -> impl FnOnce(&mut Park, Value<'d>) -> Value<'d> {
    |_, v| v
}

use super::*;
use std::format as f;
