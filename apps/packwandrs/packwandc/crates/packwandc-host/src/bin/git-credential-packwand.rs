//! Git credential-helper frontend for the native Packwand credential store.

use std::io::{self, BufRead, Write};

fn read_password() -> io::Result<Option<String>> {
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.is_empty() {
            break;
        }
        if let Some(password) = line.strip_prefix("password=") {
            return Ok(Some(password.to_owned()));
        }
    }
    Ok(None)
}

fn native_error(error: packwandc::Error) -> io::Error {
    io::Error::other(error.to_string())
}

fn main() -> io::Result<()> {
    let action = std::env::args().nth(1).unwrap_or_default();
    match action.as_str() {
        "get" => {
            if let Some(secret) = packwandc::KeyStore.load().map_err(native_error)? {
                let password = String::from_utf8(secret)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                let mut stdout = io::stdout().lock();
                writeln!(stdout, "password={password}")?;
                writeln!(stdout)?;
            }
        }
        "store" => {
            if let Some(password) = read_password()? {
                packwandc::KeyStore
                    .save(password.as_bytes())
                    .map_err(native_error)?;
            }
        }
        "erase" => packwandc::KeyStore.clear().map_err(native_error)?,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected get, store, or erase",
            ));
        }
    }
    Ok(())
}
