pub fn parse(stdout: Vec<u8>, typ: &ast::Type) -> Result<Value<'static>> {
    use ast::Type::*;

    Ok(match typ {
        VBoxVms => {
            let lines = stdout.split(|&b| b == b'\n');
            let ids = lines.filter_map(|line| {
                if line.is_empty() {
                    None
                } else {
                    Some((|| {
                        let find =
                            |c: u8| line.iter().copied().position(|b| b == c);

                        let start = te!(find(b'{')) + 1;
                        let end = te!(find(b'}'));

                        let id = te!(std::str::from_utf8(&line[start..end]));
                        let value = Value::Str(Cow::from(id.to_owned()));

                        Ok(value)
                    })())
                }
            });

            Value::List(te!(ids.collect::<Result<_>>()))
        }
        Null => Value::Empty,
        String => {
            Value::Str(Cow::from(te!(std::string::String::from_utf8(stdout))))
        }
        Display => Value::Empty,
        Stream => err!("todo"),
    })
}

use super::*;
use engine::Value;
