use crate::parser::ArgParser;
use crate::types::*;

/// Standalone builder for defining a boolean flag argument.
///
/// Construct with [`Flag::new()`], chain modifiers like [`.short()`](Flag::short)
/// and [`.hidden()`](Flag::hidden), then pass to [`ArgBuilder::flag()`].
#[derive(Clone, Debug)]
pub struct Flag {
    long: String,
    short: Option<char>,
    description: String,
    hidden: bool,
}

impl Flag {
    /// Create a new flag with a long name. Description defaults to empty string.
    pub fn new(long: &str) -> Self {
        Self {
            long: long.to_string(),
            short: None,
            description: String::new(),
            hidden: false,
        }
    }

    /// Set the description shown in help text.
    pub fn desc(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Set the optional single-character short form.
    pub fn short(mut self, ch: char) -> Self {
        self.short = Some(ch);
        self
    }

    /// Mark this flag as hidden (excluded from help text).
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }
}

impl From<Flag> for FlagDef {
    fn from(f: Flag) -> FlagDef {
        FlagDef {
            long: f.long,
            short: f.short,
            description: f.description,
            hidden: f.hidden,
        }
    }
}

/// Standalone builder for defining a key-value option argument.
///
/// Construct with [`Opt::new()`], chain modifiers like [`.short()`](Opt::short),
/// [`.required()`](Opt::required), [`.default()`](Opt::default),
/// [`.env()`](Opt::env), [`.multi()`](Opt::multi), and [`.hidden()`](Opt::hidden),
/// then pass to [`ArgBuilder::option()`].
#[derive(Clone, Debug)]
pub struct Opt {
    long: String,
    short: Option<char>,
    placeholder: String,
    description: String,
    required: bool,
    default: Option<String>,
    env_var: Option<String>,
    multi: bool,
    hidden: bool,
}

impl Opt {
    /// Create a new option with only the long name.
    /// Placeholder defaults to the uppercased long name; description defaults to empty.
    pub fn new(long: &str) -> Self {
        let placeholder = long.to_uppercase();
        Self {
            long: long.to_string(),
            short: None,
            placeholder,
            description: String::new(),
            required: false,
            default: None,
            env_var: None,
            multi: false,
            hidden: false,
        }
    }

    /// Set the placeholder shown in help text (e.g. "FILE").
    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }

    /// Set the description shown in help text.
    pub fn desc(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Set the optional single-character short form.
    pub fn short(mut self, ch: char) -> Self {
        self.short = Some(ch);
        self
    }

    /// Mark this option as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set a default value for this option.
    pub fn default(mut self, value: &str) -> Self {
        self.default = Some(value.to_string());
        self
    }

    /// Set an environment variable fallback for this option.
    pub fn env(mut self, var_name: &str) -> Self {
        self.env_var = Some(var_name.to_string());
        self
    }

    /// Mark this option as accepting multiple values.
    pub fn multi(mut self) -> Self {
        self.multi = true;
        self
    }

    /// Mark this option as hidden (excluded from help text).
    pub fn hidden(mut self) -> Self {
        self.hidden = true;
        self
    }
}

impl From<Opt> for OptionDef {
    fn from(o: Opt) -> OptionDef {
        OptionDef {
            long: o.long,
            short: o.short,
            placeholder: o.placeholder,
            description: o.description,
            required: o.required,
            default: o.default,
            env_var: o.env_var,
            multi: o.multi,
            hidden: o.hidden,
        }
    }
}

/// Standalone builder for defining a positional argument.
///
/// Construct with [`Pos::new()`], chain modifiers like [`.desc()`](Pos::desc),
/// [`.required()`](Pos::required), [`.default()`](Pos::default), and
/// [`.multi()`](Pos::multi), then pass to [`ArgBuilder::positional()`].
#[derive(Clone, Debug)]
pub struct Pos {
    name: String,
    description: String,
    required: bool,
    default: Option<String>,
    multi: bool,
}

impl Pos {
    /// Create a new positional with a name and description.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            required: false,
            default: None,
            multi: false,
        }
    }

    /// Set the description shown in help text.
    pub fn desc(mut self, description: &str) -> Self {
        self.description = description.to_string();
        self
    }

    /// Mark this positional as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set a default value for this positional argument.
    pub fn default(mut self, value: &str) -> Self {
        self.default = Some(value.to_string());
        self
    }

    /// Mark this positional as collecting all remaining arguments.
    pub fn multi(mut self) -> Self {
        self.multi = true;
        self
    }
}

impl From<Pos> for PositionalDef {
    fn from(p: Pos) -> PositionalDef {
        PositionalDef {
            name: p.name,
            description: p.description,
            required: p.required,
            default: p.default,
            multi: p.multi,
        }
    }
}

/// Fluent builder for constructing an [`ArgParser`].
///
/// Chain calls to [`flag()`](ArgBuilder::flag), [`option()`](ArgBuilder::option),
/// [`positional()`](ArgBuilder::positional), and [`subcommand()`](ArgBuilder::subcommand)
/// to define the argument schema, then call [`build()`](ArgBuilder::build) to produce
/// the parser. Construct argument definitions using [`Flag`], [`Opt`], and [`Pos`]
/// and pass them directly to the builder methods.
#[must_use = "builder does nothing until .build() is called"]
#[derive(Clone, Debug)]
pub struct ArgBuilder {
    program_name: Option<String>,
    program_desc: Option<String>,
    version: Option<String>,
    flags: Vec<FlagDef>,
    options: Vec<OptionDef>,
    positionals: Vec<PositionalDef>,
    subcommands: Vec<SubcommandDef>,
}

impl ArgBuilder {
    /// Create a new builder with no arguments defined.
    pub fn new() -> Self {
        Self {
            program_name: None,
            program_desc: None,
            version: None,
            flags: Vec::new(),
            options: Vec::new(),
            positionals: Vec::new(),
            subcommands: Vec::new(),
        }
    }

    /// Set the program name shown in usage and version text.
    pub fn name(mut self, name: &str) -> Self {
        self.program_name = Some(name.to_string());
        self
    }

    /// Set the program description shown at the top of help text.
    pub fn description(mut self, desc: &str) -> Self {
        self.program_desc = Some(desc.to_string());
        self
    }

    /// Set the version string. Enables `--version` / `-V` flags.
    pub fn version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    /// Add a flag definition to the builder.
    pub fn flag(mut self, flag: Flag) -> Self {
        self.flags.push(FlagDef::from(flag));
        self
    }

    /// Add an option definition to the builder.
    pub fn option(mut self, opt: Opt) -> Self {
        self.options.push(OptionDef::from(opt));
        self
    }

    /// Add a positional argument definition to the builder.
    pub fn positional(mut self, pos: Pos) -> Self {
        self.positionals.push(PositionalDef::from(pos));
        self
    }

    /// Register a subcommand with a name, description, and its own pre-built ArgParser.
    pub fn subcommand(mut self, name: &str, desc: &str, parser: ArgParser) -> Self {
        if let Some(existing) = self.subcommands.iter_mut().find(|s| s.name == name) {
            existing.description = desc.to_string();
            existing.parser = parser;
        } else {
            self.subcommands.push(SubcommandDef {
                name: name.to_string(),
                description: desc.to_string(),
                parser,
            });
        }
        self
    }

    /// Validate the schema and produce an [`ArgParser`].
    ///
    /// Returns [`ParseError::InvalidFormat`] if there are duplicate long names,
    /// duplicate short characters, or a `-V` conflict when a version is set.
    #[must_use = "returns the built ArgParser; did you forget to assign it?"]
    pub fn build(self) -> Result<ArgParser, ParseError> {
        // Validate no duplicate long names across flags and options
        let mut seen_longs = std::collections::HashSet::new();
        for flag in &self.flags {
            if !seen_longs.insert(&flag.long) {
                return Err(ParseError::InvalidFormat(format!(
                    "duplicate long argument name: --{}",
                    flag.long
                )));
            }
        }
        for opt in &self.options {
            if !seen_longs.insert(&opt.long) {
                return Err(ParseError::InvalidFormat(format!(
                    "duplicate long argument name: --{}",
                    opt.long
                )));
            }
        }

        // Validate no duplicate short chars across flags and options
        let mut seen_shorts = std::collections::HashSet::new();
        for flag in &self.flags {
            if let Some(ch) = flag.short {
                if !seen_shorts.insert(ch) {
                    return Err(ParseError::InvalidFormat(format!("duplicate short argument: -{}", ch)));
                }
            }
        }
        for opt in &self.options {
            if let Some(ch) = opt.short {
                if !seen_shorts.insert(ch) {
                    return Err(ParseError::InvalidFormat(format!("duplicate short argument: -{}", ch)));
                }
            }
        }

        // Validate no -V conflict when version is configured
        if self.version.is_some() {
            for flag in &self.flags {
                if flag.short == Some('V') {
                    return Err(ParseError::InvalidFormat(
                        "duplicate short argument: -V (reserved for --version)".to_string(),
                    ));
                }
            }
            for opt in &self.options {
                if opt.short == Some('V') {
                    return Err(ParseError::InvalidFormat(
                        "duplicate short argument: -V (reserved for --version)".to_string(),
                    ));
                }
            }
        }

        // Validate positional configurations
        for pos in &self.positionals {
            if pos.required && pos.default.is_some() {
                return Err(ParseError::InvalidFormat(format!(
                    "positional '{}' cannot be both required and have a default",
                    pos.name
                )));
            }
            if pos.required && pos.multi {
                return Err(ParseError::InvalidFormat(format!(
                    "positional '{}' cannot be both required and multi",
                    pos.name
                )));
            }
        }
        if let Some(pos) = self
            .positionals
            .iter()
            .enumerate()
            .find(|(i, p)| p.multi && *i < self.positionals.len() - 1)
            .map(|(_, p)| p)
        {
            return Err(ParseError::InvalidFormat(format!(
                "multi positional '{}' must be the last positional",
                pos.name
            )));
        }

        Ok(ArgParser {
            program_name: self.program_name,
            program_desc: self.program_desc,
            version: self.version,
            flags: self.flags,
            options: self.options,
            positionals: self.positionals,
            subcommands: self.subcommands,
        })
    }
}

impl Default for ArgBuilder {
    fn default() -> Self {
        Self::new()
    }
}
