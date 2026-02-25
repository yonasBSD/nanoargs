//! # nanoargs
//!
//! A minimal, zero-dependency argument parser for Rust CLI applications.
//! Part of the [nano crate family](https://crates.io/search?q=nano).
//!
//! nanoargs gives you flags, options, positional arguments, subcommands,
//! environment variable fallbacks, typed parsing, auto-generated help text,
//! version handling, and optional colored help — all without pulling in a single
//! transitive dependency.
//!
//! ## Quick Start
//!
//! ```rust
//! use nanoargs::{ArgBuilder, Flag, Opt, Pos};
//!
//! let parser = ArgBuilder::new()
//!     .name("myapp")
//!     .description("A sample CLI tool")
//!     .flag(Flag::new("verbose").desc("Enable verbose output").short('v'))
//!     .option(Opt::new("output").placeholder("FILE").desc("Output file path").short('o'))
//!     .positional(Pos::new("input").desc("Input file").required())
//!     .build()
//!     .unwrap();
//!
//! // Parse from a Vec<String>:
//! // let result = parser.parse(args)?;
//!
//! // Or parse from std::env::args():
//! // let result = parser.parse_env()?;
//! ```
//!
//! ## Features
//!
//! | Feature | Description |
//! |---------|-------------|
//! | Flags | Boolean switches like `--verbose` / `-v` |
//! | Options | Key-value pairs like `--output file.txt` or `-o=file.txt` |
//! | Positionals | Ordered arguments like `<input>` (required) or `[extra]` (optional) |
//! | Subcommands | Nested command trees with independent argument sets |
//! | Short flag clusters | Combined flags like `-abc` and `-aboval` |
//! | Typed parsing | `get_option_or_default::<T>()`, `get_option_required::<T>()`, and more |
//! | Env var fallback | `.env("MY_VAR")` on options, with CLI > env > default precedence |
//! | Multi-value options | `.multi()` to collect repeated `--tag a --tag b` into a `Vec` |
//! | Hidden arguments | `.hidden()` to keep flags/options out of help text |
//! | Default values | `.default("value")` for optional options |
//! | Required arguments | `.required()` on options and positionals |
//! | Auto help | Built-in `-h` / `--help` with column-aligned output |
//! | Version flag | `.version("1.0.0")` enables `--version` / `-V` |
//! | Colored output | Opt-in `color` feature for ANSI-styled help and errors via [`nanocolor`](https://crates.io/crates/nanocolor) |
//!
//! ## Builder API
//!
//! All argument definitions flow through [`ArgBuilder`] using a chainable builder pattern:
//!
//! ```rust
//! # use nanoargs::{ArgBuilder, Flag, Opt, Pos};
//! let parser = ArgBuilder::new()
//!     .name("greet")
//!     .version("1.0.0")
//!     .flag(Flag::new("loud").desc("Shout the greeting").short('l'))
//!     .option(Opt::new("name").placeholder("NAME").desc("Who to greet").short('n').required())
//!     .option(Opt::new("times").placeholder("N").desc("Repeat count").default("1"))
//!     .positional(Pos::new("extra").desc("Extra words"))
//!     .build()
//!     .unwrap();
//! ```
//!
//! Construct argument definitions using [`Flag`], [`Opt`], and [`Pos`], chain
//! modifiers like `.required()`, `.default()`, `.env()`, `.multi()`, `.hidden()`,
//! then pass them directly to the builder methods.
//!
//! ## Parsing and Results
//!
//! ```rust,no_run
//! # use nanoargs::ArgBuilder;
//! # let parser = ArgBuilder::new().build().unwrap();
//! let result = parser.parse_env().unwrap();
//!
//! // Flags return bool
//! let verbose = result.get_flag("verbose");
//!
//! // Options return Option<&str>
//! let output = result.get_option("output");
//!
//! // Typed parsing with default (returns Result — Err on bad parse)
//! let count: u32 = result.get_option_or_default("times", 1).unwrap();
//!
//! // Or require it (returns Result for ? operator)
//! // let count: u32 = result.get_option_required("times")?;
//!
//! // Lazy default via closure
//! // let count: u32 = result.get_option_or("times", || expensive_default())?;
//!
//! // Low-level typed parse: Option<Result<T, Err>>
//! let parsed: Option<Result<u32, _>> = result.get_option_parsed("times");
//!
//! // Multi-value options return &[String]
//! let tags = result.get_option_values("tags");
//!
//! // Typed multi-values with fallback
//! // let tags: Vec<String> = result.get_option_values_or_default("tags", vec![])?;
//!
//! // Positionals in order
//! let positionals = result.get_positionals();
//!
//! // Subcommand access
//! if let Some(name) = result.subcommand() {
//!     let sub = result.subcommand_result().unwrap();
//! }
//! ```
//!
//! ## Subcommands
//!
//! ```rust
//! # use nanoargs::{ArgBuilder, Flag, Pos};
//! let sub = ArgBuilder::new()
//!     .positional(Pos::new("file").desc("File to add").required())
//!     .build().unwrap();
//!
//! let parser = ArgBuilder::new()
//!     .name("git-lite")
//!     .flag(Flag::new("verbose").desc("Verbose output").short('v'))
//!     .subcommand("add", "Stage files", sub)
//!     .build().unwrap();
//! ```
//!
//! Global flags are parsed before the subcommand name. Everything after the
//! subcommand name is delegated to the subcommand's parser.
//!
//! ## Error Handling
//!
//! Parsing returns `Result<ParseResult, ParseError>`. The [`ParseError`] variants
//! cover missing required arguments, missing option values, unknown arguments,
//! invalid formats, unknown subcommands, and the special `HelpRequested` /
//! `VersionRequested` cases (which carry the formatted text).
//!
//! ## Colored Output
//!
//! Enable the `color` Cargo feature for ANSI-styled help text and error messages:
//!
//! ```toml
//! [dependencies]
//! nanoargs = { version = "0.1", features = ["color"] }
//! ```
//!
//! Colors are applied automatically and suppressed when `NO_COLOR` is set or
//! stdout is not a TTY (handled by `nanocolor`).
//!
//! ## Schema-Free Parsing (Escape Hatch)
//!
//! [`parse_loose`] is a convenience escape hatch for throwaway scripts where
//! defining a full schema is overkill. **It is not the recommended way to
//! parse arguments** — prefer [`ArgBuilder`] for anything user-facing.
//!
//! ```rust,no_run
//! let result = nanoargs::parse_loose().unwrap();
//! let verbose = result.get_flag("verbose");
//! let output = result.get_option("output");
//! let positionals = result.get_positionals();
//! ```
//!
//! `parse_loose` uses a heuristic (if the next token doesn't start with `-`,
//! it's consumed as a value), which means `--output -v` silently treats
//! `--output` as a flag. It also provides no help text, no required-argument
//! validation, and no typed parsing. For anything beyond a quick script, use
//! [`ArgBuilder`].

mod builders;
mod free;
mod help;
mod macros;
mod parser;
mod result;
mod types;

pub use builders::{ArgBuilder, Flag, Opt, Pos};
pub use free::parse_loose;
pub use parser::ArgParser;
pub use result::{OptionError, ParseResult, ParseResultBuilder};
pub use types::{FlagDef, OptionDef, ParseError, PositionalDef, SubcommandDef};

#[cfg(test)]
mod tests;
