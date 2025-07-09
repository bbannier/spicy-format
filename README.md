# spicy-format

This provides an opinionated source code formatter for the [Spicy
language](https://docs.zeek.org/projects/spicy/en/latest/).

## Installation

### Prebuilt binaries

Navigate to the [latest
release](https://github.com/bbannier/spicy-format/releases/latest) and run the
mentioned installation script, e.g.,

```console
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/bbannier/spicy-format/releases/download/v0.25.0/spicy-format-installer.sh | sh
```

Follow the printed instructions. This will install both the formatter
`spicy-format` itself as well as an updater script `spicy-format-update` which
can be used to update to newer releases.

Alternatively you can also download prebuilt binaries and updaters on the
release page.

### Building from source

If no binaries are available for you platform you need to install from source.
This requires a Rust installation, use e.g., [rustup](https://rustup.rs/) for
setting one up.

You can then install the project with

```console
cargo install --locked --git https://github.com/bbannier/spicy-format
```

It is **strongly suggested** to use `--locked` to use dependency versions locked
in the project.

## Development

This formatter is implemented in Rust with
[Topiary](https://github.com/tweag/topiary). It uses
[tree-sitter-spicy](https://github.com/bbannier/tree-sitter-spicy) for parsing
input source code into a tree-sitter CST. Formatting of that CST is done
through [Topiary queries](https://github.com/tweag/topiary#design). The actual
queries are defined in [`src/query.scm`](src/query.scm). Most of the time
changing the formatter should not require any changes to the Rust code, but
only to the query file.

### Working with the Spicy tree-sitter grammar

#### General process

To change formatting of certain nodes it might often be sufficient to inspect
the [Spicy tree-sitter
grammar](https://github.com/bbannier/tree-sitter-spicy/blob/main/grammar.js)
and change formatting of the nodes in [`src/query.scm`](src/query.scm).

To instead inspect the full parse trees of code samples check out
[tree-sitter-spicy](https://github.com/bbannier/tree-sitter-spicy) and use it
to parse the code, e.g.,

```console
# Get Spicy tree-sitter grammar.
git clone https://github.com/bbannier/tree-sitter-spicy
$ cd tree-sitter-spicy

# If needed install the tree-sitter CLI.
$ npm install tree-sitter

# Use tree-sitter to parse a source file.
$ tree-sitter parse hello.spicy
(module [0, 0] - [3, 0]
  entities: (module_decl [0, 0] - [0, 13]
    name: (ident [0, 7] - [0, 12]
      (name [0, 7] - [0, 12])))
  entities: (statement [2, 0] - [2, 18]
    (print [2, 0] - [2, 18]
      (expression [2, 6] - [2, 17]
        (string [2, 6] - [2, 17])))))
```

Use this information to write dedicated formatter queries.

#### Making adjustments to the grammar

If the grammar does not have support for constructs or incorrectly parses
them the grammar needs to be updated and bumped. The used version of the
grammar is specified in
[`Cargo.toml`](https://github.com/bbannier/spicy-format/blob/main/Cargo.toml),

```toml
[dependencies]
tree-sitter-spicy = { git = "https://github.com/bbannier/tree-sitter-spicy" }
...
```

You can specify a different repository here, or enforce a specific revision, e.g.,

```toml
tree-sitter-spicy = { git = "https://github.com/MY_FORK/tree-sitter-spicy", rev = "b9958baeda3dc77fa94af2ca2cd84723bd532d08" }
```

In order to use a unpublished local version you can replace the dependency with
a [path
dependency](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-path-dependencies),
e.g.,

```toml
tree-sitter-spicy = { path = "path/to/local/version" }
```

### Tests

Test cases are collected and automatically discovered by the test harness in
the [`corpus`/](corpus/) directory. To execute the test harness run

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
