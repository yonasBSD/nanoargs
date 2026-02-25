use std::collections::HashMap;
use std::str::FromStr;

/// The result of parsing a set of arguments.
///
/// Provides typed accessors for flags, options, positionals, and subcommands.
/// Constructed by [`ArgParser::parse()`](crate::ArgParser::parse) or [`ParseResultBuilder::build()`].
#[derive(Clone, Debug, PartialEq)]
pub struct ParseResult {
    flags: HashMap<String, bool>,
    option_values: HashMap<String, Vec<String>>,
    positionals: Vec<String>,
    subcommand: Option<String>,
    subcommand_result: Option<Box<ParseResult>>,
    /// Known flag and option names from the schema (empty for schema-free parsing).
    /// Used by `debug_assert!` checks to catch typos during development.
    known_flags: Vec<String>,
    known_options: Vec<String>,
}

/// Builder for constructing a [`ParseResult`] manually.
///
/// Useful for testing code that consumes parse results without running a real
/// parser.
///
/// ```rust
/// use nanoargs::ParseResultBuilder;
///
/// let result = ParseResultBuilder::new()
///     .flag("verbose", true)
///     .option("output", "file.txt")
///     .positional("input.txt")
///     .build();
///
/// assert!(result.get_flag("verbose"));
/// assert_eq!(result.get_option("output"), Some("file.txt"));
/// assert_eq!(result.get_positionals(), &["input.txt"]);
/// ```
#[must_use = "builder does nothing until .build() is called"]
#[derive(Clone, Debug, Default)]
pub struct ParseResultBuilder {
    flags: HashMap<String, bool>,
    option_values: HashMap<String, Vec<String>>,
    positionals: Vec<String>,
    subcommand: Option<String>,
    subcommand_result: Option<Box<ParseResult>>,
}

impl ParseResultBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a flag value.
    pub fn flag(mut self, name: &str, value: bool) -> Self {
        self.flags.insert(name.to_string(), value);
        self
    }

    /// Set an option value (overwrites any previous value).
    pub fn option(mut self, name: &str, value: &str) -> Self {
        self.option_values.insert(name.to_string(), vec![value.to_string()]);
        self
    }

    /// Append a value to a multi-value option.
    pub fn multi_option(mut self, name: &str, value: &str) -> Self {
        self.option_values.entry(name.to_string()).or_default().push(value.to_string());
        self
    }

    /// Add a positional argument.
    pub fn positional(mut self, value: &str) -> Self {
        self.positionals.push(value.to_string());
        self
    }

    /// Set the subcommand name and its parse result.
    pub fn subcommand(mut self, name: &str, result: ParseResult) -> Self {
        self.subcommand = Some(name.to_string());
        self.subcommand_result = Some(Box::new(result));
        self
    }

    /// Build the [`ParseResult`].
    ///
    /// The resulting `ParseResult` will have schema metadata derived from the
    /// builder's flags and options, so `debug_assert!` typo checks work in
    /// test code too.
    #[must_use]
    pub fn build(self) -> ParseResult {
        let known_flags: Vec<String> = self.flags.keys().cloned().collect();
        let known_options: Vec<String> = self.option_values.keys().cloned().collect();
        let mut result = ParseResult::new(
            self.flags,
            self.option_values,
            self.positionals,
            self.subcommand,
            self.subcommand_result,
        );
        result.set_known_names(known_flags, known_options);
        result
    }
}

impl ParseResult {
    /// Internal constructor used by the parser and free-form parse functions.
    pub(crate) fn new(
        flags: HashMap<String, bool>,
        option_values: HashMap<String, Vec<String>>,
        positionals: Vec<String>,
        subcommand: Option<String>,
        subcommand_result: Option<Box<ParseResult>>,
    ) -> Self {
        Self {
            flags,
            option_values,
            positionals,
            subcommand,
            subcommand_result,
            known_flags: Vec::new(),
            known_options: Vec::new(),
        }
    }

    /// Internal: attach known names from the schema so accessors can
    /// `debug_assert!` against typos. Called by the parser after construction.
    pub(crate) fn set_known_names(&mut self, flags: Vec<String>, options: Vec<String>) {
        self.known_flags = flags;
        self.known_options = options;
    }

    /// Returns `true` when schema metadata is present (i.e. the result was
    /// produced by the schema-based parser, not `parse_loose`).
    fn has_schema(&self) -> bool {
        !self.known_flags.is_empty() || !self.known_options.is_empty()
    }

    /// Returns `true` if the flag was provided, `false` otherwise.
    ///
    /// In debug builds, panics if `name` was never registered as a flag.
    /// This catches typos like `get_flag("verbos")` during development.
    pub fn get_flag(&self, name: &str) -> bool {
        debug_assert!(
            !self.has_schema() || self.known_flags.iter().any(|f| f == name),
            "get_flag(\"{name}\"): unknown flag. Known flags: {:?}",
            self.known_flags
        );
        self.flags.get(name).copied().unwrap_or(false)
    }

    /// Returns the last value for an option, or `None` if absent.
    ///
    /// In debug builds, panics if `name` was never registered as an option.
    /// This catches typos like `get_option("outpt")` during development.
    pub fn get_option(&self, name: &str) -> Option<&str> {
        debug_assert!(
            !self.has_schema() || self.known_options.iter().any(|o| o == name),
            "get_option(\"{name}\"): unknown option. Known options: {:?}",
            self.known_options
        );
        self.option_values.get(name)?.last().map(|s| s.as_str())
    }

    /// Returns all collected values for an option. For single-value options this
    /// is a one-element slice; for multi-value options it contains every collected
    /// value in order; for absent options it returns an empty slice.
    ///
    /// In debug builds, panics if `name` was never registered as an option.
    pub fn get_option_values(&self, name: &str) -> &[String] {
        debug_assert!(
            !self.has_schema() || self.known_options.iter().any(|o| o == name),
            "get_option_values(\"{name}\"): unknown option. Known options: {:?}",
            self.known_options
        );
        self.option_values.get(name).map_or(&[], |v| v.as_slice())
    }

    /// Parse the option value into a typed result via [`FromStr`].
    ///
    /// Returns `None` if the option was absent, `Some(Ok(T))` on success,
    /// or `Some(Err(_))` if the value couldn't be parsed.
    pub fn get_option_parsed<T: FromStr>(&self, name: &str) -> Option<Result<T, T::Err>> {
        self.get_option(name).map(|v| v.parse::<T>())
    }

    /// Parses each collected value for a multi-value option into the target type.
    pub fn get_option_values_parsed<T: FromStr>(&self, name: &str) -> Vec<Result<T, T::Err>> {
        self.get_option_values(name).iter().map(|v| v.parse::<T>()).collect()
    }

    /// Returns the positional arguments in the order they were provided.
    pub fn get_positionals(&self) -> &[String] {
        &self.positionals
    }

    /// Returns the matched subcommand name, if any.
    pub fn subcommand(&self) -> Option<&str> {
        self.subcommand.as_deref()
    }

    /// Returns the parse result for the matched subcommand, if any.
    pub fn subcommand_result(&self) -> Option<&ParseResult> {
        self.subcommand_result.as_deref()
    }

    /// Returns the parsed value, or `default` if the option is absent.
    ///
    /// Returns `Err(OptionError::ParseFailed)` when the option is present
    /// but its value cannot be parsed into `T`. The default is only used
    /// when the option was not provided at all.
    pub fn get_option_or_default<T: FromStr>(&self, name: &str, default: T) -> Result<T, OptionError>
    where
        T::Err: std::fmt::Display,
    {
        match self.get_option(name) {
            Some(v) => v.parse::<T>().map_err(|e| OptionError::ParseFailed {
                option: name.to_string(),
                message: e.to_string(),
            }),
            None => Ok(default),
        }
    }

    /// Returns the parsed value, or calls `f` to produce a fallback if the
    /// option is absent.
    ///
    /// Returns `Err(OptionError::ParseFailed)` when the option is present
    /// but its value cannot be parsed into `T`. The closure `f` is only
    /// called when the option was not provided at all.
    pub fn get_option_or<T: FromStr, F: FnOnce() -> T>(&self, name: &str, f: F) -> Result<T, OptionError>
    where
        T::Err: std::fmt::Display,
    {
        match self.get_option(name) {
            Some(v) => v.parse::<T>().map_err(|e| OptionError::ParseFailed {
                option: name.to_string(),
                message: e.to_string(),
            }),
            None => Ok(f()),
        }
    }

    /// Returns the parsed value, or an error if the option is absent or
    /// its value fails to parse.
    pub fn get_option_required<T: FromStr>(&self, name: &str) -> Result<T, OptionError>
    where
        T::Err: std::fmt::Display,
    {
        match self.get_option(name) {
            Some(v) => v.parse::<T>().map_err(|e| OptionError::ParseFailed {
                option: name.to_string(),
                message: e.to_string(),
            }),
            None => Err(OptionError::Missing {
                option: name.to_string(),
            }),
        }
    }

    /// Returns all values parsed into `T`, or `default` if the option is
    /// absent.
    ///
    /// Returns `Err(OptionError::ParseFailed)` when any single value
    /// cannot be parsed into `T`. The default is only used when the option
    /// was not provided at all.
    pub fn get_option_values_or_default<T: FromStr>(&self, name: &str, default: Vec<T>) -> Result<Vec<T>, OptionError>
    where
        T::Err: std::fmt::Display,
    {
        let raw = self.get_option_values(name);
        if raw.is_empty() {
            return Ok(default);
        }
        raw.iter().map(|v| v.parse::<T>()).collect::<Result<Vec<T>, _>>().map_err(|e| OptionError::ParseFailed {
            option: name.to_string(),
            message: e.to_string(),
        })
    }
}

/// Error returned by `get_option_required` when the option is missing or
/// its value cannot be parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptionError {
    /// The option was not provided on the command line.
    Missing { option: String },
    /// The option value could not be parsed into the target type.
    ParseFailed { option: String, message: String },
}

impl std::fmt::Display for OptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionError::Missing { option } => {
                write!(f, "option --{option} is required but was not provided")
            }
            OptionError::ParseFailed { option, message } => {
                write!(f, "option --{option}: {message}")
            }
        }
    }
}

impl std::error::Error for OptionError {}
