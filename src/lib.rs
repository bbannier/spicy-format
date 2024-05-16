#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use {
    miette::Diagnostic,
    miette::{Result, SourceOffset, SourceSpan},
    std::string::FromUtf8Error,
    thiserror::Error,
    topiary_core::{FormatterError, Operation, TopiaryQuery},
};

#[derive(Error, Debug, Diagnostic)]
#[error("format error")]
pub enum FormatError {
    #[diagnostic(code(spicy_format::parse_error))]
    #[error("parse error")]
    Parse {
        #[source_code]
        src: String,

        #[label("syntax not understood")]
        err_span: SourceSpan,
    },

    #[error("internal query error")]
    Query(#[help] String),

    #[error("idempotency violated")]
    Idempotency,

    #[error("UTF8 conversion error")]
    UTF8(#[from] FromUtf8Error),

    #[error("unknown error")]
    Unknown,
}

const QUERY: &str = include_str!("query.scm");

/// Format Spicy source code.
///
/// # Arguments
///
/// - `input`: Spicy source code to format
/// - `tolerate_parsing_errors`: whether source code with syntax errors should be accepted or
///   rejected.
/// - `skip_idempotence`: skip check that AST of formatted source is identical to input. This is
///   intended for working around current formatter limitations.
///
/// # Examples
///
/// ```
/// # use spicy_format::format;
/// let source = "global   x  : count=42 ;";
/// assert_eq!(
///     format(source, false, false).unwrap(),
///     "global x: count = 42;"
/// );
/// ```
pub fn format(
    input: &str,
    skip_idempotence: bool,
    tolerate_parsing_errors: bool,
) -> Result<String> {
    let mut output = Vec::new();

    let grammar = topiary_tree_sitter_facade::Language::from(tree_sitter_spicy::language());

    let query = TopiaryQuery::new(&grammar, QUERY).map_err(|e| match e {
        FormatterError::Query(m, e) => FormatError::Query(match e {
            None => m,
            Some(e) => format!("{m}: {e}"),
        }),
        _ => FormatError::Unknown,
    })?;

    let language = {
        topiary_core::Language {
            name: "spicy".to_string(),
            indent: Some("    ".to_string()),
            grammar,
            query,
        }
    };

    if let Err(e) = topiary_core::formatter(
        &mut input.as_bytes(),
        &mut output,
        &language,
        Operation::Format {
            skip_idempotence,
            tolerate_parsing_errors,
        },
    ) {
        Err(match e {
            FormatterError::Query(m, e) => FormatError::Query(match e {
                None => m,
                Some(e) => format!("{m}: {e}"),
            }),
            FormatterError::Idempotence => FormatError::Idempotency,
            FormatterError::Parsing {
                start_line,
                start_column,
                end_line,
                end_column,
            } => {
                let start = SourceOffset::from_location(
                    input,
                    start_line
                        .try_into()
                        .expect("cannot represent u32 as usize"),
                    start_column
                        .try_into()
                        .expect("cannot represent u32 as usize"),
                );
                let end = SourceOffset::from_location(
                    input,
                    end_line.try_into().expect("cannot represent u32 as usize"),
                    end_column
                        .try_into()
                        .expect("cannot represent u32 as usize"),
                );
                FormatError::Parse {
                    src: input.to_string(),
                    err_span: (start.offset(), end.offset() - start.offset()).into(),
                }
            }
            _ => FormatError::Unknown,
        })?;
    };

    let output = String::from_utf8(output).map_err(FormatError::UTF8)?;

    // Final cleanup of result. If we received an input not ending in a newline, also return an
    // output without newline. We do not want to force a newline since we e.g., could be formatting
    // input received from an editor and do not want to insert additional newlines.
    if input.ends_with('\n') {
        Ok(output)
    } else {
        Ok(output.trim_end().into())
    }
}

#[cfg(test)]
mod test {
    use super::format;
    use miette::{miette, Result};
    use rayon::prelude::*;
    use std::{
        collections::HashMap,
        path::{Path, PathBuf},
    };

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
                let input = std::fs::read_to_string(t.path()).ok()?;

                let output = {
                    let path = t.path().to_path_buf();

                    let file_name = format!(
                        "{}.expected",
                        path.file_name()
                            .expect(&format!(
                                "cannot get filename component of {}",
                                path.display()
                            ))
                            .to_string_lossy()
                    );

                    let mut o = path;
                    assert!(o.pop());

                    o.join(file_name)
                };

                let formatted = format(&input, false, true).expect(&format!(
                    "cannot format source file {}",
                    t.path().to_string_lossy()
                ));

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
    fn no_trailing_newline() -> Result<()> {
        assert_eq!(
            format("global   x  : count=42 ;", false, false).unwrap(),
            "global x: count = 42;"
        );
        Ok(())
    }

    #[test]
    fn corpus_external() -> Result<()> {
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

                match super::format(&source, false, false) {
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
}
