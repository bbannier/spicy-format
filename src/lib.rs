// With 1.92 this triggers on the fields of `FormatError::Parse` below.
#![allow(clippy::used_underscore_binding)]

use {
    miette::{Diagnostic, Result, SourceOffset, SourceSpan},
    std::{string::FromUtf8Error, sync::LazyLock},
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
        content: String,

        #[label("syntax not understood")]
        span: SourceSpan,
    },

    #[error("idempotency violated")]
    Idempotency,

    #[error("UTF8 conversion error")]
    UTF8(#[from] FromUtf8Error),

    #[error("unknown error")]
    Unknown,
}

impl From<FormatterError> for FormatError {
    fn from(value: FormatterError) -> Self {
        match value {
            FormatterError::Idempotence => FormatError::Idempotency,
            FormatterError::Parsing(span) => {
                let input = span.content.as_ref().map_or_else(|| "", String::as_str);

                let start = SourceOffset::from_location(
                    input,
                    span.start_point()
                        .row()
                        .try_into()
                        .expect("cannot represent u32 as usize"),
                    span.start_point()
                        .column()
                        .try_into()
                        .expect("cannot represent u32 as usize"),
                );
                let end = SourceOffset::from_location(
                    input,
                    span.end_point()
                        .row()
                        .try_into()
                        .expect("cannot represent u32 as usize"),
                    span.end_point()
                        .column()
                        .try_into()
                        .expect("cannot represent u32 as usize"),
                );
                FormatError::Parse {
                    content: input.to_string(),
                    span: (start.offset(), end.offset() - start.offset()).into(),
                }
            }
            _ => FormatError::Unknown,
        }
    }
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
    static LANGUAGE: LazyLock<topiary_core::Language> = LazyLock::new(|| {
        let grammar = topiary_tree_sitter_facade::Language::from(tree_sitter_spicy::LANGUAGE);

        topiary_core::Language {
            name: "spicy".to_string(),
            indent: Some("    ".to_string()),
            query: TopiaryQuery::new(&grammar, QUERY)
                .map_err(FormatError::from)
                .expect("invalid grammar"),
            grammar,
        }
    });

    let mut output = Vec::new();
    topiary_core::formatter_str(
        input,
        &mut output,
        &LANGUAGE,
        Operation::Format {
            skip_idempotence,
            tolerate_parsing_errors,
        },
    )
    .map_err(FormatError::from)?;

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
