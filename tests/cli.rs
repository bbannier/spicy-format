use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

use assert_cmd::cargo::CommandCargoExt;
use tempfile::NamedTempFile;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn trailing_newline_stdin() -> Result<()> {
    let mut cmd = Command::cargo_bin("spicy-format")?;
    let cmd = cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    let mut child = cmd.spawn()?;

    let mut stdin = child.stdin.as_ref().unwrap();
    stdin.write("1;\n".as_bytes())?;

    child.wait()?;
    let mut stdout = String::new();
    child.stdout.unwrap().read_to_string(&mut stdout)?;
    assert_eq!(stdout, "1;\n");
    Ok(())
}

#[test]
fn trailing_newline_file() -> Result<()> {
    let mut input = NamedTempFile::new()?;
    input.write("1;\n".as_bytes())?;

    let mut cmd = Command::cargo_bin("spicy-format")?;
    let cmd = cmd.arg(input.path()).stdout(Stdio::piped()).output()?;

    let stdout = String::from_utf8(cmd.stdout)?;

    assert_eq!(stdout, "1;\n");
    Ok(())
}
