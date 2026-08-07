use crate::{TraceLevel, trace, trace_drain, trace_dropped};

const MAX_LINE: usize = 1024;
const MAX_ARGS: usize = 16;
const MAX_ARG: usize = 127;
const MAX_OUTPUT: usize = 511;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellOutcome {
    Handled(Vec<String>),
    ForHost(Vec<String>),
    Empty,
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("{message}")]
pub struct ShellError {
    message: String,
}

impl ShellError {
    fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        trace(
            TraceLevel::Error,
            "pwsh",
            &message,
            "packwand-platform/src/shell.rs",
            None,
        );
        Self { message }
    }
}

pub fn shell_parse(line: &str) -> Result<Vec<String>, ShellError> {
    let input = line.as_bytes();
    if input.len() > MAX_LINE {
        return Err(ShellError::new("line exceeds the maximum length"));
    }
    let mut words = Vec::new();
    let mut cursor = 0;
    while cursor < input.len() {
        match input[cursor] {
            b' ' | b'\t' | b'\r' => {
                cursor += 1;
                continue;
            }
            b'#' => break,
            b'\n' => {
                if cursor + 1 < input.len() {
                    return Err(ShellError::new("more than one line in a single command"));
                }
                break;
            }
            _ => {}
        }
        if words.len() == MAX_ARGS {
            return Err(ShellError::new("too many words in one command"));
        }
        let mut word = Vec::new();
        let mut quote = None;
        let mut produced = false;
        while cursor < input.len() {
            let byte = input[cursor];
            if quote.is_none() && matches!(byte, b' ' | b'\t' | b'\r' | b'\n' | b'#') {
                break;
            }
            if quote.is_none() && matches!(byte, b'\'' | b'"') {
                quote = Some(byte);
                produced = true;
                cursor += 1;
                continue;
            }
            if quote == Some(byte) {
                quote = None;
                cursor += 1;
                continue;
            }
            let decoded = if quote == Some(b'"') && byte == b'\\' {
                let Some(next) = input.get(cursor + 1).copied() else {
                    return Err(ShellError::new("trailing backslash with nothing to escape"));
                };
                cursor += 2;
                match next {
                    b'n' => b'\n',
                    b't' => b'\t',
                    b'r' => b'\r',
                    b'"' | b'\'' | b'\\' => next,
                    _ => return Err(ShellError::new("unrecognised escape sequence")),
                }
            } else {
                cursor += 1;
                byte
            };
            if word.len() == MAX_ARG {
                return Err(ShellError::new(
                    "word is longer than the maximum argument length",
                ));
            }
            word.push(decoded);
            produced = true;
        }
        if quote.is_some() {
            return Err(ShellError::new("unterminated quote"));
        }
        if !produced {
            return Err(ShellError::new("empty word"));
        }
        words
            .push(String::from_utf8(word).map_err(|_| ShellError::new("word is not valid UTF-8"))?);
    }
    Ok(words)
}

pub fn shell_exec(line: &str) -> Result<ShellOutcome, ShellError> {
    let words = shell_parse(line)?;
    let Some(verb) = words.first().map(String::as_str) else {
        return Ok(ShellOutcome::Empty);
    };
    let lines = match verb {
        "help" => vec![
            "pw4shell built-ins:".into(),
            "  help                     list the built-in commands".into(),
            "  version                  report the Packwand native-core version".into(),
            "  echo <words...>          echo the arguments back".into(),
            "  status <code>            explain a compatibility status code".into(),
            "  trace drain|drops        read the native trace ring".into(),
            "Packwand CLI verbs are handled by the host; try 'packwand --help'.".into(),
        ],
        "version" => vec![format!(
            "Packwand native core {}",
            env!("CARGO_PKG_VERSION")
        )],
        "echo" => {
            let output = words[1..].join(" ");
            if output.len() > MAX_OUTPUT {
                return Err(ShellError::new("echo output exceeds the line limit"));
            }
            vec![output]
        }
        "status" => vec![status_line(&words)?],
        "trace" => trace_lines(&words)?,
        _ => return Ok(ShellOutcome::ForHost(words)),
    };
    Ok(ShellOutcome::Handled(lines))
}

fn status_line(words: &[String]) -> Result<String, ShellError> {
    if words.len() != 2 {
        return Err(ShellError::new("usage: status <code>"));
    }
    let code = words[1]
        .parse::<i32>()
        .map_err(|_| ShellError::new("status: not a number"))?;
    if code.unsigned_abs() > 999_999 {
        return Err(ShellError::new("status: number too large"));
    }
    let (name, description) = status(code);
    Ok(format!("{name} ({code}): {description}"))
}

fn status(code: i32) -> (&'static str, &'static str) {
    match code {
        0 => ("PWC_OK", "success"),
        -1 => ("PWC_EINVAL", "invalid argument"),
        -2 => ("PWC_ENOENT", "no such object"),
        -3 => ("PWC_EPERM", "operation not permitted by handle rights"),
        -4 => ("PWC_EAGAIN", "would block, retry"),
        -5 => ("PWC_ENOMEM", "allocation failed or arena exhausted"),
        -6 => ("PWC_EBADF", "unknown handle"),
        -7 => ("PWC_ESTALE", "handle generation mismatch, object was freed"),
        -8 => ("PWC_ENOSYS", "syscall not implemented on this platform"),
        -9 => ("PWC_EIO", "platform I/O failure"),
        -10 => ("PWC_ETIMEDOUT", "deadline expired"),
        -11 => ("PWC_ECANCELED", "operation cancelled"),
        -12 => ("PWC_EOVERFLOW", "value or buffer too large"),
        _ => ("PWC_EUNKNOWN", "unknown status code"),
    }
}

fn trace_lines(words: &[String]) -> Result<Vec<String>, ShellError> {
    match words.get(1).map(String::as_str) {
        Some("drops") if words.len() == 2 => {
            Ok(vec![format!("{} record(s) dropped", trace_dropped())])
        }
        Some("drain") if words.len() == 2 => {
            let records = trace_drain();
            if records.is_empty() {
                return Ok(vec!["trace is empty".into()]);
            }
            Ok(records
                .into_iter()
                .map(|record| {
                    format!(
                        "[{}] {} {} {}",
                        record.sequence, record.module, record.origin, record.message
                    )
                })
                .collect())
        }
        _ => Err(ShellError::new("usage: trace drain | trace drops")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_preserves_the_pw4shell_grammar() {
        assert_eq!(
            shell_parse("echo one 'two three' \"four\\nlines\" # ignored").unwrap(),
            vec!["echo", "one", "two three", "four\nlines"]
        );
        assert_eq!(shell_parse("  # comment").unwrap(), Vec::<String>::new());
        assert!(shell_parse("echo \"unterminated").is_err());
        assert!(shell_parse("one\ntwo").is_err());
        assert!(shell_parse(&format!("echo {}", "x".repeat(128))).is_err());
    }

    #[test]
    fn unknown_commands_are_for_the_bundled_host_only() {
        assert_eq!(
            shell_exec("packwand --help").unwrap(),
            ShellOutcome::ForHost(vec!["packwand".into(), "--help".into()])
        );
    }
}
