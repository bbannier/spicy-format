# spicy-format

This provides a formatter for the [Spicy
language](https://docs.zeek.org/projects/spicy/en/latest/).

⚠ The implementation is very much a work in progress at the moment, and should
be considered **ALPHA QUALITY**.

## Development

This formatter is implemented in Rust with
[Topiary](https://github.com/tweag/topiary). It uses
[tree-sitter-spicy](https://github.com/bbannier/tree-sitter-spicy) for parsing
input source code into a tree-sitter CST. Formatting of that CST is done
through [Topiary queries](https://github.com/tweag/topiary#design). The actual
queries are defined in [src/query.scm](src/query.scm). Changing the formatter
should not require any changes to the Rust code, but only to the query file.

Test cases are collected and automatically discovered by the test harness in
the [corpus/](corpus/) directory. To execute the test harness run

```console
cargo t
```

For each `corpus/<INPUT>.spicy` the repository contains a matching expected
result baseline `corpus/<INPUT>.spicy.expected`. To update the baselines
execute the test harness with the environment variable `UPDATE_BASELINE` set.

```console
UPDATE_BASELINE=1 cargo t
```

This will update or if needed create the outdated baselines. When adding new
tests to the corpus, commit the baseline as well.

To run formatting against an external corpus run the test suite with
`SPICY_FORMAT_EXTERNAL_CORPUS` set.

```console
SPICY_FORMAT_EXTERNAL_CORPUS=<PATH TO SPICY CORPUS> cargo t corpus_external
```

The suite automatically filters out of some files in the Spicy test suite with
known unsupported constructs.
