type Map<'a, T> = Deq<(&'a str, T)>;
type Index<'a, T> = Map<'a, Map<'a, T>>;

pub fn parse_compile<I: AsRef<str>, O: io::Write>(
    input: I,
    output: O,
) -> Result<()> {
    let mut obj = Index::new();

    for line in input.as_ref().lines() {
        let mut tokens: Deq<&str> = line.split_whitespace().collect();

        if tokens.is_empty() {
            continue;
        }

        let key = tokens.pop_front().ok_or_else(|| {
            ErrorKind::from(format!("No key? {line:?}", line = line))
        })?;

        if key == "Host" {
            let name = tokens.front().ok_or_else(|| {
                ErrorKind::from(format!("No host name? {line:?}", line = line))
            })?;
            obj.push_back((name, <_>::default()));
        } else {
            let (_host_name, host) = obj.back_mut().ok_or_else(|| {
                ErrorKind::from(format!(
                    "No host before line {line:?}",
                    line = line
                ))
            })?;
            host.push_back((key, tokens));
        }
    }
    cbor::to_writer(output, &obj)?;
    Ok(())
}

use super::*;
