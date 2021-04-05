pub fn parse_compile<I: AsRef<str>, O: io::Write>(
    input: I,
    output: O,
) -> Result<()> {
    let mut obj = Map::new();
    let mut bx: Option<&mut Map<&str, &str>> = None;

    for line in input.as_ref().lines() {
        // see etc/sample_vagrant_machine_readable_output.txt
        let mut tokens: Deq<&str> = line.split(',').skip(2).collect();

        match (tokens.pop_front(), tokens.pop_front(), tokens.pop_front()) {
            (Some("ui"), _, _)
            | (None, None, None)
            | (Some("box-info"), _, _) => {} // ignore

            (Some("box-name"), Some(box_name), None) => {
                bx = Some(obj.entry(box_name).or_default());
            }
            (Some(key), Some(val), None) => {
                bx.as_mut()
                    .ok_or_else(|| ErrorKind::from(format!("No box?")))?
                    .insert(key, val);
            }
            other => {
                return Err(format!("Undone: {other:?}", other = other).into())
            }
        }
    }

    cbor::to_writer(output, &obj)?;
    Ok(())
}

use super::*;
