use {
    miette::Diagnostic,
    miette::{Result, SourceOffset, SourceSpan},
    std::string::FromUtf8Error,
    thiserror::Error,
    topiary::{FormatterError, TopiaryQuery},
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

    #[error("query error")]
    Query(String),

    #[error("idempotency violated")]
    Idempotency,

    #[error("UTF8 conversion error")]
    UTF8(#[from] FromUtf8Error),

    #[error("unknown error")]
    Unknown,
}

const QUERY: &str = include_str!("query.scm");

pub fn format(
    input: &str,
    skip_idempotence: bool,
    tolerate_parsing_errors: bool,
) -> Result<String> {
    let mut output = Vec::new();
    let language = {
        topiary::Language {
            name: "spicy".to_string(),
            extensions: vec!["spicy".to_string()].into_iter().collect(),
            indent: Some("    ".to_string()),
        }
    };

    let grammar = tree_sitter_facade::Language::from(tree_sitter_spicy::language_spicy());

    let query = TopiaryQuery::new(&grammar, QUERY).unwrap();

    if let Err(e) = topiary::formatter(
        &mut input.as_bytes(),
        &mut output,
        &query,
        &language,
        &grammar,
        topiary::Operation::Format {
            skip_idempotence,
            tolerate_parsing_errors,
        },
    ) {
        Err(match e {
            FormatterError::Query(message, error) => {
                if let Some(err) = error {
                    dbg!(err);
                }
                FormatError::Query(message)
            }
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

    Ok(String::from_utf8(output).map_err(FormatError::UTF8)?)
}

#[cfg(test)]
mod test {
    use super::format;
    use miette::Result;
    use std::path::PathBuf;

    #[test]
    fn corpus() -> Result<()> {
        use pretty_assertions::assert_eq;

        let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus");

        let update_baseline = std::env::var("UPDATE_BASELINE").is_ok();

        for test in std::fs::read_dir(corpus).expect("cannot enumerate corpus") {
            let test = test.expect("cannot list corpus entry {:?}");

            // Exclude baselines.
            if test.path().extension().and_then(|ext| ext.to_str()) == Some("expected") {
                continue;
            }

            eprintln!("parsing {}", test.path().display());

            let input = std::fs::read_to_string(test.path())
                .expect(&format!("cannot read file {}", test.path().display()));

            let output = {
                let path = test.path();

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

            let formatted = format(&input, false, true).expect("cannot format source file");

            if !update_baseline {
                let expected = std::fs::read_to_string(output).expect("cannot read baseline");
                assert_eq!(expected, formatted);
            } else {
                std::fs::write(output, formatted).expect("cannot update baseline");
            }
        }

        Ok(())
    }
}
