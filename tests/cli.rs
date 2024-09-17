use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use assert_cmd::cargo::CommandCargoExt;
use miette::miette;
use rayon::prelude::*;
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

/// Helper to format an input file via the CLI.
fn format<P>(input: P) -> Result<String>
where
    P: Into<PathBuf>,
{
    let output = Command::cargo_bin("spicy-format")?
        .arg("--reject-parse-errors")
        .arg(input.into())
        .stdout(Stdio::piped())
        .output()?;

    let output = String::from_utf8(output.stdout)?;

    Ok(output)
}

#[test]
fn corpus() -> Result<()> {
    use pretty_assertions::assert_eq;

    let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus");

    let update_baseline = std::env::var("UPDATE_BASELINE").is_ok();

    let files = walkdir::WalkDir::new(&corpus)
        .into_iter()
        .filter_map(|e| {
            let e = e.ok()?;

            if e.file_type().is_file()
                && e.path().extension().and_then(|ext| ext.to_str()) == Some("spicy")
            {
                Some(e)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    files
        .par_iter()
        .filter_map(|t| {
            let input = t.path();

            let output = {
                let path = input.to_path_buf();

                let file_name = format!(
                    "{}.expected",
                    path.file_name()
                        .unwrap_or_else(|| panic!(
                            "cannot get filename component of {}",
                            path.display()
                        ))
                        .to_string_lossy()
                );

                let mut o = path;
                assert!(o.pop());

                o.join(file_name)
            };

            let formatted = format(&input).unwrap_or_else(|_| {
                panic!("cannot format source file {}", t.path().to_string_lossy())
            });

            if !update_baseline {
                let expected = std::fs::read_to_string(output).expect("cannot read baseline");
                assert_eq!(
                    expected,
                    formatted,
                    "while formatting {}",
                    t.path().display()
                );
            } else {
                std::fs::write(output, formatted).expect("cannot update baseline");
            }

            Some(1)
        })
        .collect::<Vec<_>>();

    Ok(())
}

#[test]
fn corpus_external() -> miette::Result<()> {
    let Ok(corpus) = std::env::var("SPICY_FORMAT_EXTERNAL_CORPUS") else {
        return Ok(());
    };

    let is_filtered = |p: &Path| -> bool {
        let deny_list = [
            "tools/preprocessor.spicy",
            "types/unit/hooks-fail.spicy",
            // Fails due to parser ambiguity due to https://github.com/zeek/spicy/issues/1566.
            "types/unit/switch-attributes-fail.spicy",
        ];

        deny_list.iter().any(|b| p.ends_with(b))
    };

    // Compute a vector of file names so we can process them below in parallel.
    let files = walkdir::WalkDir::new(&corpus)
        .into_iter()
        .filter_map(|e| {
            let e = e.ok()?;

            if e.file_type().is_file()
                && e.path().extension().and_then(|ext| ext.to_str()) == Some("spicy")
                && !is_filtered(e.path())
            {
                Some(e)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let results = files
        .par_iter()
        .filter_map(|f| {
            let f = f.path().to_str()?;

            let source = std::fs::read_to_string(f).ok()?;

            // Ignore inputs with multiple parts.
            if source.contains("@TEST-START-FILE") {
                return None;
            }

            match format(&source) {
                Err(_) => Some((f.to_string(), false)),
                Ok(_) => Some((f.to_string(), true)),
            }
        })
        .collect::<HashMap<_, _>>();

    let num_tests = results.len();

    let failures = results
        .into_iter()
        .filter_map(|(f, success)| if success { None } else { Some(f) })
        .collect::<Vec<_>>();

    if failures.is_empty() {
        Ok(())
    } else {
        Err(miette!(
            "{} out of {num_tests} inputs failed:\n{}",
            failures.len(),
            failures.join("\n")
        ))
    }
}
