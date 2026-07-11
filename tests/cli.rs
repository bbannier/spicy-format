use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use assert_cmd::cargo;
use filetime::FileTime;
use miette::{NamedSource, miette};
use rayon::prelude::*;
use serde::Deserialize;
use spicy_format::{LANGUAGE, QUERY};
use tempfile::NamedTempFile;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn do_not_touch_unmodified() -> Result<()> {
    let input = NamedTempFile::new()?;
    filetime::set_file_mtime(input.path(), FileTime::zero())?;

    let Output { status, .. } = Command::new(cargo::cargo_bin!())
        .arg(input.path())
        .arg("-i")
        .output()?;
    assert!(status.success());

    let metadata = std::fs::metadata(input.path())?;
    let mtime = FileTime::from_last_modification_time(&metadata);

    assert_eq!(mtime, FileTime::zero());

    Ok(())
}

#[test]
fn coverage() -> miette::Result<()> {
    #[derive(Deserialize)]
    struct Cmd {
        stdout: Option<String>,
    }

    let sources: Vec<_> = std::fs::read_dir("tests/cmd/")
        .expect("could not read test baselines")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if !path.is_file() {
                return None;
            }

            let code = std::fs::read_to_string(&path).expect("unreabable file");

            // The stdout of tests should either be empty or contain valid, parseable code.
            toml::from_str::<Cmd>(&code)
                .expect("unparsable input")
                .stdout
        })
        .collect();

    let source = sources.join("\n");

    check_coverage(&source)?;

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
            // Unsupported legacy syntax.
            "types/vector/legacy-syntax-fail.spicy",
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

            match format(f) {
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

#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .case("tests/cmd/*.toml")
        .case("README.md")
        .default_bin_name("spicy-format");
}

/// Helper to format an input file via the CLI.
fn format<P>(path: P) -> Result<String>
where
    P: Into<PathBuf>,
{
    let path = path.into();
    let output = Command::new(cargo::cargo_bin!())
        .arg("--reject-parse-errors")
        .arg(&path)
        .stdout(Stdio::piped())
        .output()?;

    assert!(
        output.status.success(),
        "could not format {}",
        path.display()
    );
    let output = String::from_utf8(output.stdout)?;

    Ok(output)
}

/// Check coverage of the builtin query against the provided source code.
pub fn check_coverage(input: &str) -> miette::Result<()> {
    let cov = topiary_core::check_query_coverage(input, &LANGUAGE.query, &LANGUAGE.grammar)
        .expect("could not collect coverage");

    let query = NamedSource::new("query.scm", QUERY).with_language("scheme");

    if cov.missing_patterns.is_empty() {
        Ok(())
    } else {
        Err(miette::Report::new(cov).with_source_code(query))
    }
}
