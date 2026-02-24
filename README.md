# 📎 nanoargs

[![Crates.io](https://img.shields.io/crates/v/nanoargs)](https://crates.io/crates/nanoargs)
[![Docs.rs](https://docs.rs/nanoargs/badge.svg)](https://docs.rs/nanoargs/latest/nanoargs/)
[![Build Status](https://github.com/anthonysgro/nanoargs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/anthonysgro/nanoargs/actions)
[![Coverage Status](https://coveralls.io/repos/github/anthonysgro/nanoargs/badge.svg?branch=main)](https://coveralls.io/github/anthonysgro/nanoargs?branch=main)
[![License](https://img.shields.io/crates/l/nanoargs)](https://crates.io/crates/nanoargs)

A lightweight, zero-dependency argument parser for Rust.

<p align="center">
  <img src="demo.gif" alt="nanoargs help output" width="700" />
</p>

Part of the [nano](https://github.com/anthonysgro/nano) crate family — minimal, zero-dependency building blocks for Rust.

Everything you'd expect from a CLI parser — flags, options, subcommands, help generation, env fallback, typed parsing — with zero dependencies.

## Why nanoargs?

Choosing a CLI parser in Rust usually feels like a compromise:

- `clap` is the gold standard, but it's a heavy lift. It pulls in 10+ transitive dependencies, deep customization and vast api reference sheets.
- `pico-args` / `lexopt` are zero-dep, but they leave the hard work to you. You'll end up hand-coding your own --help strings, ENV fallbacks, and subcommand logic.
- `nanoargs` is the middle ground. You get the professional features you actually use like subcommands, help generation, and env fallbacks, with **zero** dependencies.


| Feature | `nanoargs` | `clap` | `bpaf` | `pico-args` | `lexopt` |
|---------|:----------:|:------:|:------:|:-----------:|:--------:|
| Dependencies (transitive) | 0 | ~12\* | 5\*\* | 0 | 0 |
| Auto help text | ✓ | ✓ | ✓ | ✗ | ✗ |
| Version flag (`--version`) | ✓ | ✓ | ✓ | ✗ | ✗ |
| Env var fallback | ✓ | ✓ | ✓ | ✗ | ✗ |
| Multi-value options | ✓ | ✓ | ✓ | ✗ | ✗ |
| Subcommands | ✓ | ✓ | ✓ | ✗† | ✗† |
| Combined short flags (`-abc`) | ✓ | ✓ | ✓ | ✓§ | ✓ |
| Default values | ✓ | ✓ | ✓ | ✗ | ✗ |
| Required args | ✓ | ✓ | ✓ | ✗ | ✗ |
| Hidden args | ✓ | ✓ | ✓ | — | — |
| Colored help | ✓§ | ✓ | ✓§ | ✗ | ✗ |
| Derive macros | ✗ | ✓ | ✓ | ✗ | ✗ |
| Shell completions | ✗ | ✓ | ✓§ | ✗ | ✗ |
| Other advanced features | ✗ | ✓ | ✓ | ✗ | ✗ |

\* `clap` with default features. With derive, ~17 total.
\*\* `bpaf` combinatoric API has 0 deps. With derive, 5 total (`bpaf_derive` + `syn` tree).
† No built-in support. Achievable manually by matching on positional tokens.
§ Via opt-in cargo features.

Which one should I use?

- `clap` / `bpaf`: Your CLI is complex and needs deep customization and advanced support.
- `pico-args` / `lexopt`: You’re building something tiny where most features aren't a priority.
- `nanoargs`: You want a clean, intuitive API that supports 90% of use cases without taking on any dependencies.

## Quick Start ([full demo](examples/full_demo.rs))

```sh
cargo add nanoargs
```

```rust
use nanoargs::{ArgBuilder, Flag, Opt, Pos, ParseError};

fn main() {
    let parser = ArgBuilder::new()
        .name("myapp")
        .description("A sample CLI tool")
        .version("1.0.0")
        .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
        .option(Opt::new("output").placeholder("FILE").desc("Output file path").short('o'))
        .positional(Pos::new("input").desc("Input file").required())
        .build()
        .unwrap();

    match parser.parse_env() {
        Ok(result) => {
            println!("verbose: {}", result.get_flag("verbose"));
            println!("output:  {:?}", result.get_option("output"));
            println!("input:   {:?}", result.get_positionals());
        }
        Err(ParseError::HelpRequested(text)) => print!("{text}"),
        Err(ParseError::VersionRequested(text)) => println!("{text}"),
        Err(e) => eprintln!("error: {e}"),
    }
}
```

See [Parsing and Results](#parsing-and-results) and [Error Handling](#error-handling) for more details.

## Defining Arguments

### Flags ([example](examples/flags.rs))

Boolean switches toggled by presence.

```rust
let parser = ArgBuilder::new()
    .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
    .flag(Flag::new("dry-run").desc("Simulate without side effects"))
    .build();
```

```sh
myapp --verbose --dry-run
myapp -v
```

### Options ([example](examples/options.rs))

Key-value arguments with fluent modifiers. Construct an `Opt` with `Opt::new()`, chain `.placeholder()`, `.desc()`, `.short()`, `.required()`, `.default()`, `.env()`, `.multi()`, or `.hidden()` as needed, then pass it to `.option()`.

```rust
let parser = ArgBuilder::new()
    .option(Opt::new("format").placeholder("FMT").desc("Output format").short('f'))
    .option(Opt::new("output").placeholder("FILE").desc("Output file path").short('o').required())
    .option(Opt::new("jobs").placeholder("NUM").desc("Parallel jobs").short('j').default("4"))
    .option(Opt::new("include").placeholder("DIR").desc("Directories to include").short('i').multi())
    .build();
```

```sh
myapp --output result.txt --jobs 8 --include src --include tests
myapp -o=result.txt -j 8
```

### Positionals ([example](examples/positionals.rs))

Unnamed arguments collected in order. Chain `.required()` on the `Pos` builder to make a positional mandatory.

```rust
let parser = ArgBuilder::new()
    .positional(Pos::new("input").desc("Input file").required())
    .positional(Pos::new("extra").desc("Additional arguments"))
    .build();
```

```sh
myapp input.txt extra1 extra2
```

### Environment Variable Fallback ([example](examples/env_fallback.rs))

Options can fall back to environment variables when not provided on the command line. Chain `.env()` on the `Opt` builder. The resolution order is: CLI value → env var → default → error (if required).

```rust
let parser = ArgBuilder::new()
    .option(Opt::new("log-level").placeholder("LEVEL").desc("Log level").short('l').env("MYAPP_LOG_LEVEL"))
    .option(Opt::new("output").placeholder("FILE").desc("Output file").short('o').env("MYAPP_OUTPUT").required())
    .option(Opt::new("format").placeholder("FMT").desc("Output format").short('f').env("MYAPP_FORMAT").default("text"))
    .build();
```

```sh
# CLI value takes priority
myapp --output result.txt

# Falls back to env var when CLI option is omitted
MYAPP_OUTPUT=from_env.txt myapp

# Falls back to default when both CLI and env var are absent
myapp --output result.txt   # format resolves to "text"
```

Help text automatically shows the associated env var:

```
Options:
  -l, --log-level <LEVEL>  Log level [env: MYAPP_LOG_LEVEL]
  -o, --output <FILE>      Output file (required) [env: MYAPP_OUTPUT]
  -f, --format <FMT>       Output format [default: text] [env: MYAPP_FORMAT]
```

### Hidden Arguments

Flags and options can be marked as hidden — they parse normally but are excluded from `--help` output. Useful for internal, debug, or deprecated arguments.

```rust
let parser = ArgBuilder::new()
    .flag(Flag::new("debug").desc("Enable debug mode").short('d').hidden())
    .option(Opt::new("trace-id").placeholder("ID").desc("Internal trace ID").hidden())
    .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
    .build();
```

```sh
# Hidden arguments work on the command line
myapp --debug --trace-id=abc123 --verbose

# But --help only shows --verbose
myapp --help
```

The `.hidden()` modifier is available on both `Flag` and `Opt`, and can be called in any order relative to other modifiers.

### Combined Short Flags ([example](examples/short_flags.rs))

Combine multiple short flags into a single token. The parser walks characters left-to-right against the registered schema.

```rust
let parser = ArgBuilder::new()
    .flag(Flag::new("all").desc("Show all").short('a'))
    .flag(Flag::new("brief").desc("Brief output").short('b'))
    .flag(Flag::new("color").desc("Enable color").short('c'))
    .option(Opt::new("width").placeholder("NUM").desc("Column width").short('w'))
    .build();
```

```sh
# Combined flags
myapp -abc              # sets all, brief, color

# Attached option value
myapp -w10              # sets width to "10"

# Flags + option in one token
myapp -abcw10           # sets all, brief, color + width="10"
myapp -abcw 10          # same — value from next token
```

When the parser encounters an option character during the walk, it claims all remaining characters as the value. If none remain, it consumes the next argument token.

### Subcommands ([example](examples/subcommands.rs))

Git-style subcommands, each with their own flags, options, and positionals. Global flags are parsed before the subcommand token.

```rust
let build_parser = ArgBuilder::new()
    .name("build")
    .description("Compile the project")
    .flag(Flag::new("release").desc("Build in release mode").short('r'))
    .build();

let test_parser = ArgBuilder::new()
    .name("test")
    .description("Run the test suite")
    .flag(Flag::new("verbose").desc("Show detailed output").short('v'))
    .build();

let parser = ArgBuilder::new()
    .name("myapp")
    .description("A demo CLI")
    .flag(Flag::new("quiet").desc("Suppress output").short('q'))
    .subcommand("build", "Compile the project", build_parser)
    .subcommand("test", "Run the test suite", test_parser)
    .build();
```

```sh
myapp build --release
myapp -q test --verbose
myapp --help              # lists available subcommands
myapp build --help        # subcommand-specific help
```

> **Note:** When subcommands are registered, the first bare (non-flag/option) token is always treated as the subcommand name. Parent-level positional arguments are not supported alongside subcommands — this matches git-style CLI conventions.
>
> ```sh
> # Supported — global flags before the subcommand:
> myapp -q build --release
>
> # NOT supported — positionals before the subcommand:
> myapp file.txt build    # "file.txt" is treated as an unknown subcommand
> ```

### Version Flag ([example](examples/version_flag.rs))

Built-in `--version` / `-V` support. Set a version string on the builder and the parser handles the rest.

```rust
let parser = ArgBuilder::new()
    .name("myapp")
    .version(env!("CARGO_PKG_VERSION"))
    .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
    .build()
    .unwrap();
```

```sh
$ myapp --version
myapp 0.1.0

$ myapp -V
myapp 0.1.0
```

The `-V` short flag is reserved when a version is configured — the builder will reject any user-registered flag or option that uses `'V'` as its short form. When no version is set, `--version` and `-V` are treated as unknown arguments, and `'V'` is available for user flags.

When both `--help` and `--version` appear, whichever comes first wins. After `--`, both are treated as positionals.

## Parsing and Results

### Accessors

`parse_env()` reads from `std::env::args()` and returns a `Result<ParseResult, ParseError>`:

```rust
let result = parser.parse_env()?;

// Flags return bool
let verbose = result.get_flag("verbose");

// Options return Option<&str>
let output = result.get_option("output");

// Multi-value options return &[String]
let tags = result.get_option_values("tags");

// Positionals in order
let positionals = result.get_positionals();

// Subcommand access
if let Some(name) = result.subcommand() {
    let sub = result.subcommand_result().unwrap();
}
```

Accessors like `get_flag` and `get_option` use string keys, so a typo like `get_flag("verbos")` would silently return `false`. To catch these during development, nanoargs includes `debug_assert!` checks that panic if you access a name that was never registered. These checks run automatically in debug builds (`cargo test`, `cargo run`) and are stripped in release builds with zero overhead.

You can also pass your own args with `parser.parse(args)` — see [Error Handling](#error-handling) for the full match pattern.

### Typed Parsing

Parse option values into any type implementing `FromStr`. Convenience helpers collapse the common three-way match into a single call:

```rust
// With a default fallback — returns the parsed value, or the default if absent/unparseable
let jobs: u32 = result.get_option_or_default("jobs", 4);

// With a lazy default — closure only runs if needed
let jobs: u32 = result.get_option_or("jobs", || num_cpus());

// Required with Result — use the ? operator
let jobs: u32 = result.get_option_required("jobs")?;
```

For fine-grained control over parse errors, the original accessor is still available:

```rust
match result.get_option_parsed::<u32>("jobs") {
    Some(Ok(n)) => println!("jobs: {}", n),
    Some(Err(e)) => eprintln!("invalid jobs value: {}", e),
    None => println!("jobs not set"),
}
```

### Error Handling ([example](examples/error_handling.rs))

```rust
match parser.parse(args) {
    Ok(result) => { /* use result */ }
    Err(ParseError::HelpRequested(text)) => print!("{}", text),
    Err(ParseError::VersionRequested(text)) => println!("{}", text),
    Err(ParseError::MissingRequired(name)) => eprintln!("missing: {}", name),
    Err(ParseError::MissingValue(name)) => eprintln!("no value for: --{}", name),
    Err(ParseError::UnknownArgument(token)) => eprintln!("unknown: {}", token),
    Err(ParseError::NoSubcommand(msg)) => eprintln!("{}", msg),
    Err(ParseError::UnknownSubcommand(name)) => eprintln!("unknown subcommand: {}", name),
    Err(ParseError::DuplicateOption(name)) => eprintln!("duplicate: --{}", name),
    Err(ParseError::InvalidFormat(msg)) => eprintln!("bad format: {}", msg),
}
```

## Help and Output

### Help Text ([example](examples/help_text.rs))

Auto-generated from your schema. Triggered by `--help` or `-h`.

```sh
$ myapp --help
Usage: myapp [OPTIONS] <input> [extra]

A sample CLI tool

Options:
  -v, --verbose          Enable verbose output
      --dry-run          Simulate without side effects
  -o, --output <FILE>    Output file path (required)
  -j, --jobs <NUM>       Parallel jobs [default: 4]
  -h, --help             Print help
```

### Colored Help (opt-in)

Enable the `color` feature to get ANSI-colored help text and error messages via [nanocolor](https://github.com/anthonysgro/nanocolor):

```toml
[dependencies]
nanoargs = { version = "0.1", features = ["color"] }
```

```sh
cargo run --example help_text --features color -- --help
```

When enabled, section headers are bold yellow, flag/option names are green, placeholders are cyan, and metadata like `[default: ...]` is dim. Error messages get a bold red `error:` prefix. Color is automatically suppressed when `NO_COLOR` is set or output is not a TTY (handled by nanocolor). Without the feature, the crate remains zero-dependency and output is unchanged.

### Double-Dash Separator

Everything after `--` is treated as a positional, even if it looks like a flag or option.

```sh
myapp -- --not-a-flag -abc
# positionals: ["--not-a-flag", "-abc"]
```

## Schema-Free Parsing for Quick Scripts

`parse_loose()` skips the schema entirely — useful for throwaway scripts where defining flags and options feels like overkill.

```rust
fn main() {
    let result = nanoargs::parse_loose().unwrap();
    let verbose = result.get_flag("verbose");
    let output = result.get_option("output");
    let positionals = result.get_positionals();
}
```

It uses a heuristic to guess whether `--key` is a flag or an option: if the next token doesn't start with `-`, it's consumed as the value.

**When it works well:** simple scripts with clear flag/option boundaries (`--verbose --output file.txt`).

**When it doesn't:** `--output -v` silently treats `--output` as a flag (not an option), because `-v` starts with `-`. If your CLI has options that could receive flag-like values, use `ArgBuilder` instead.

## API Reference

See the [full API docs on docs.rs](https://docs.rs/nanoargs/latest/nanoargs/).

## Examples

<details>
<summary>Click to expand all examples</summary>

| Example | Description | Run |
|---------|-------------|-----|
| [flags](examples/flags.rs) | Boolean flags | `cargo run --example flags -- -v --dry-run` |
| [options](examples/options.rs) | Options with defaults and required | `cargo run --example options -- -o=out.txt -j 8` |
| [positionals](examples/positionals.rs) | Positional arguments | `cargo run --example positionals -- file.txt extra` |
| [short_flags](examples/short_flags.rs) | Combined short flags and attached values | `cargo run --example short_flags -- -abcw10` |
| [help_text](examples/help_text.rs) | Auto-generated help | `cargo run --example help_text -- --help` |
| [error_handling](examples/error_handling.rs) | Error handling patterns | `cargo run --example error_handling` |
| [version_flag](examples/version_flag.rs) | Built-in version flag | `cargo run --example version_flag -- --version` |
| [env_fallback](examples/env_fallback.rs) | Environment variable fallback | `cargo run --example env_fallback -- --output out.txt` |
| [subcommands](examples/subcommands.rs) | Git-style subcommands | `cargo run --example subcommands -- build --release` |
| [full_demo](examples/full_demo.rs) | All features together | `cargo run --example full_demo -- -vj8 -o=result.txt input.txt` |

</details>

## Contributing

Contributions are welcome. To get started:

1. Fork the repository
2. Create a feature branch (`git checkout -b my-feature`)
3. Make your changes
4. Run the tests: `cargo test`
5. Submit a pull request

Please keep changes minimal and focused. This crate's goal is to stay small and dependency-free.

## License

This project is licensed under the [MIT License](LICENSE).
