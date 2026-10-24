use {
    clap::Parser,
    miette::{Context, Diagnostic, Result, ensure},
    rayon::iter::{IntoParallelRefIterator, ParallelIterator},
    spicy_format::format,
    thiserror::Error,
};

#[derive(Parser)]
#[clap(version = version())]
struct Args {
    #[clap(
        help = "input files to operate on",
        default_values_t = ["/dev/stdin".to_string()],
    )]
    input_files: Vec<String>,

    #[clap(short, long, help = "skip idempotency check")]
    skip_idempotence: bool,

    #[clap(short, long, help = "reject inputs with parse errors")]
    reject_parse_errors: bool,

    #[clap(long, short, help = "format file in place")]
    inplace: bool,
}

#[derive(Error, Debug, Diagnostic)]
enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

fn main() -> Result<()> {
    let args = Args::parse();

    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .tab_width(4)
                .context_lines(3)
                .with_cause_chain()
                .build(),
        )
    }))?;

    let failed = args
        .input_files
        .par_iter()
        .map(|input_file| -> Result<()> {
            let source = std::fs::read_to_string(input_file)
                .map_err(Error::Io)
                .wrap_err(format!("while reading input file {input_file}"))?;

            let formatted = format(&source, args.skip_idempotence, !args.reject_parse_errors)
                .wrap_err(format!("while formatting '{input_file}'"))?;

            if args.inplace {
                // Only write output file if we made any changes.
                if source != formatted {
                    std::fs::write(input_file, formatted)
                        .map_err(Error::Io)
                        .wrap_err(format!("while writing output file {input_file}"))?;
                }
            } else {
                // When printing the result do not insert extra newlines.
                print!("{formatted}");
            }

            Ok(())
        })
        .filter(|f| {
            if let Err(e) = f {
                eprintln!("{e:?}");
                true
            } else {
                false
            }
        })
        .count();

    ensure!(failed == 0, "could not format {failed} file(s)");

    Ok(())
}

fn version() -> &'static str {
    option_env!("VERGEN_GIT_DESCRIBE").unwrap_or_else(|| env!("CARGO_PKG_VERSION"))
}
