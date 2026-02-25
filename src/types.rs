use std::fmt;

use crate::parser::ArgParser;

/// Definition of a boolean flag (e.g. `--verbose` / `-v`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlagDef {
    /// Long name used as `--name`.
    pub long: String,
    /// Optional single-character short form used as `-c`.
    pub short: Option<char>,
    /// Human-readable description shown in help text.
    pub description: String,
    /// When `true`, this flag is omitted from help text but still parsed.
    pub hidden: bool,
}

/// Definition of a key-value option (e.g. `--output FILE` / `-o FILE`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionDef {
    /// Long name used as `--name`.
    pub long: String,
    /// Optional single-character short form used as `-c`.
    pub short: Option<char>,
    /// Placeholder shown in help text (e.g. `FILE`, `NUM`).
    pub placeholder: String,
    /// Human-readable description shown in help text.
    pub description: String,
    /// When `true`, parsing fails if this option is not provided.
    pub required: bool,
    /// Default value used when the option is absent from CLI and env.
    pub default: Option<String>,
    /// Environment variable name to check as a fallback.
    pub env_var: Option<String>,
    /// When `true`, repeated occurrences are collected into a `Vec`.
    pub multi: bool,
    /// When `true`, this option is omitted from help text but still parsed.
    pub hidden: bool,
}

/// Definition of a positional argument (e.g. `<input>` or `[extra]`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositionalDef {
    /// Name shown in usage line and help text.
    pub name: String,
    /// Human-readable description shown in help text.
    pub description: String,
    /// When `true`, parsing fails if this positional is not provided.
    pub required: bool,
}

/// Definition of a subcommand: a name, description, and its own [`ArgParser`].
#[derive(Clone, Debug, PartialEq)]
pub struct SubcommandDef {
    /// Subcommand name as typed on the command line.
    pub name: String,
    /// Human-readable description shown in help text.
    pub description: String,
    /// Independent parser that handles arguments after the subcommand name.
    pub parser: ArgParser,
}

/// Errors produced during argument parsing.
///
/// The `HelpRequested` and `VersionRequested` variants carry the formatted
/// text that should be printed to stdout. All other variants represent actual
/// errors and carry a descriptive message suitable for display.
///
/// Implements [`std::fmt::Display`] and [`std::error::Error`]. When the
/// `color` feature is enabled, the `Display` output includes ANSI styling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// A required argument was not provided. Contains the argument name.
    MissingRequired(String),
    /// An option was provided without a value. Contains the option name.
    MissingValue(String),
    /// An unrecognized argument was encountered. Contains the raw token.
    UnknownArgument(String),
    /// A structural error in the parser definition. Contains a description.
    InvalidFormat(String),
    /// The `-h` / `--help` flag was encountered. Contains the formatted help text.
    HelpRequested(String),
    /// The `-V` / `--version` flag was encountered. Contains the formatted version text.
    VersionRequested(String),
    /// A subcommand was expected but none was provided. Contains available names.
    NoSubcommand(String),
    /// An unrecognized subcommand was provided. Contains the unknown name.
    UnknownSubcommand(String),
    /// A non-multi option was provided more than once. Contains the option name.
    DuplicateOption(String),
    /// A command-line argument contained bytes that are not valid UTF-8.
    /// Contains the lossy representation.
    InvalidUtf8(String),
}

// ── Leaf colorization helpers for ParseError ───────────────────────────────

/// Returns the "error: " prefix with bold+red styling when color is enabled,
/// or an empty string when color is disabled (matching the current plain output).
#[cfg(feature = "color")]
fn error_prefix() -> String {
    use nanocolor::Colorize;
    format!("{} ", "error:".bold().red())
}

#[cfg(not(feature = "color"))]
fn error_prefix() -> String {
    String::new()
}

/// Returns the argument string with yellow styling when color is enabled,
/// or the plain string when color is disabled.
#[cfg(feature = "color")]
fn yellow_arg(s: &str) -> String {
    use nanocolor::Colorize;
    s.yellow().to_string()
}

#[cfg(not(feature = "color"))]
fn yellow_arg(s: &str) -> String {
    s.to_string()
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::HelpRequested(text) | ParseError::VersionRequested(text) => {
                write!(f, "{text}")
            }
            ParseError::MissingRequired(name) => {
                write!(f, "{}missing required argument: {}", error_prefix(), yellow_arg(name))
            }
            ParseError::MissingValue(name) => {
                write!(
                    f,
                    "{}missing value for option: {}",
                    error_prefix(),
                    yellow_arg(&format!("--{name}"))
                )
            }
            ParseError::UnknownArgument(token) => {
                write!(f, "{}unknown argument: {}", error_prefix(), yellow_arg(token))
            }
            ParseError::InvalidFormat(msg) => {
                write!(f, "{}invalid format: {msg}", error_prefix())
            }
            ParseError::NoSubcommand(names) => {
                write!(f, "{}no subcommand provided. Available: {names}", error_prefix())
            }
            ParseError::UnknownSubcommand(name) => {
                write!(f, "{}unknown subcommand: {}", error_prefix(), yellow_arg(name))
            }
            ParseError::DuplicateOption(name) => {
                write!(
                    f,
                    "{}option {} was provided more than once (use .multi() to allow repeats)",
                    error_prefix(),
                    yellow_arg(&format!("--{name}"))
                )
            }
            ParseError::InvalidUtf8(lossy) => {
                write!(
                    f,
                    "{}argument is not valid UTF-8: {}",
                    error_prefix(),
                    yellow_arg(lossy)
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}
