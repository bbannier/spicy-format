use {
    clap::Parser,
    miette::{ensure, Context, Diagnostic, Result},
    rayon::iter::{IntoParallelRefIterator, ParallelIterator},
    spicy_format::format,
    std::{io::Read, path::PathBuf},
    thiserror::Error,
};

#[derive(Parser)]
#[clap(version = version())]
struct Args {
    #[clap(
        help = "input files to operate on",
        long_help = "if not provided read input from stdin"
    )]
    input_files: Vec<PathBuf>,

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

    let format = |code: &str, source: &str| {
        format(code, args.skip_idempotence, !args.reject_parse_errors)
            .wrap_err(format!("while formatting '{source}'"))
    };

    if args.input_files.is_empty() {
        let stdin = std::io::stdin();
        let mut buf = String::new();
        stdin
            .lock()
            .read_to_string(&mut buf)
            .map_err(Error::Io)
            .wrap_err("while reading input from stdin")?;
        println!("{}", format(&buf, "<stdin>")?);
    } else {
        let failed = args
            .input_files
            .par_iter()
            .filter_map(|input_file| {
                let source = match std::fs::read_to_string(input_file)
                    .map_err(Error::Io)
                    .wrap_err(format!("while reading input file {}", input_file.display()))
                {
                    Err(e) => {
                        eprintln!("{e:#}");
                        return Some(input_file);
                    }
                    Ok(s) => s,
                };

                let formatted = match format(&source, &input_file.display().to_string()) {
                    Ok(f) => f,
                    Err(e) => {
                        eprintln!("{e:#}");
                        return Some(input_file);
                    }
                };

                if args.inplace {
                    if let Err(e) = std::fs::write(input_file, formatted)
                        .map_err(Error::Io)
                        .wrap_err(format!(
                            "while writing output file {}",
                            input_file.display()
                        ))
                    {
                        eprintln!("{e:#}");
                        return Some(input_file);
                    }
                } else {
                    println!("{formatted}");
                    return Some(input_file);
                }

                None
            })
            .count();

        ensure!(failed == 0, "could not format {failed} file(s)");
    }

    Ok(())
}

fn version() -> &'static str {
    option_env!("VERGEN_GIT_DESCRIBE").unwrap_or_else(|| env!("CARGO_PKG_VERSION"))
}
