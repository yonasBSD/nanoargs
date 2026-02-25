use std::collections::HashMap;

use crate::result::ParseResult;
use crate::types::*;

/// Internal enum returned by `parse_tokens` to indicate what the caller should handle.
enum TokenAction {
    /// All tokens consumed successfully, parsing complete.
    Done,
    /// Encountered a bare token at position `index` with value `token`.
    Positional { index: usize, token: String },
    /// Encountered `--` at position `index`; remaining tokens start at `index + 1`.
    RestPositional { index: usize },
}

/// The argument parser. Holds the schema (flags, options, positionals,
/// subcommands) and provides methods to parse arguments, generate help text,
/// and serialize/deserialize the schema.
///
/// Constructed via [`ArgBuilder::build()`](crate::ArgBuilder::build).
#[derive(Clone, Debug, PartialEq)]
pub struct ArgParser {
    pub(crate) program_name: Option<String>,
    pub(crate) program_desc: Option<String>,
    pub(crate) version: Option<String>,
    pub(crate) flags: Vec<FlagDef>,
    pub(crate) options: Vec<OptionDef>,
    pub(crate) positionals: Vec<PositionalDef>,
    pub(crate) subcommands: Vec<SubcommandDef>,
}

impl ArgParser {
    /// Returns the program name, if set.
    pub fn program_name(&self) -> Option<&str> {
        self.program_name.as_deref()
    }

    /// Returns the program description, if set.
    pub fn program_desc(&self) -> Option<&str> {
        self.program_desc.as_deref()
    }

    /// Returns the registered flag definitions.
    pub fn flags(&self) -> &[FlagDef] {
        &self.flags
    }

    /// Returns the registered option definitions.
    pub fn options(&self) -> &[OptionDef] {
        &self.options
    }

    /// Returns the registered positional definitions.
    pub fn positionals(&self) -> &[PositionalDef] {
        &self.positionals
    }

    /// Returns the registered subcommand definitions.
    pub fn subcommands(&self) -> &[SubcommandDef] {
        &self.subcommands
    }

    /// Returns formatted version text, or None if no version is configured.
    pub fn version_text(&self) -> Option<String> {
        self.version.as_ref().map(|v| match &self.program_name {
            Some(name) => format!("{name} {v}"),
            None => v.clone(),
        })
    }

    /// Parse a list of argument strings.
    ///
    /// Returns `Err(ParseError::HelpRequested(_))` or
    /// `Err(ParseError::VersionRequested(_))` when `-h`/`--help` or
    /// `-V`/`--version` are encountered — these are not errors per se,
    /// but signal that the caller should print the contained text and exit.
    pub fn parse(&self, args: Vec<String>) -> Result<ParseResult, ParseError> {
        let mut result = if self.subcommands.is_empty() {
            self.parse_no_subcommands(args)?
        } else {
            self.parse_with_subcommands(args)?
        };
        result.set_known_names(
            self.flags.iter().map(|f| f.long.clone()).collect(),
            self.options.iter().map(|o| o.long.clone()).collect(),
        );
        Ok(result)
    }

    /// Validate a value against an optional validator, mapping errors to `ParseError::ValidationFailed`.
    fn validate_value(
        name: &str,
        value: &str,
        validator: &Option<crate::validators::Validator>,
    ) -> Result<(), ParseError> {
        if let Some(v) = validator {
            v.validate(value).map_err(|msg| ParseError::ValidationFailed {
                name: name.to_string(),
                message: msg,
            })
        } else {
            Ok(())
        }
    }

    /// Store an option value into the unified map. For multi-value options,
    /// values are appended; for single-value options, errors if already set.
    fn store_option_value(
        values: &mut HashMap<String, Vec<String>>,
        key: &str,
        value: String,
        multi: bool,
        validator: &Option<crate::validators::Validator>,
    ) -> Result<(), ParseError> {
        Self::validate_value(key, &value, validator)?;
        if multi {
            values.entry(key.to_string()).or_default().push(value);
        } else if values.contains_key(key) {
            return Err(ParseError::DuplicateOption(key.to_string()));
        } else {
            values.insert(key.to_string(), vec![value]);
        }
        Ok(())
    }

    /// Shared token-parsing helper. Dispatches each token to the appropriate
    /// handler based on its shape. Returns a `TokenAction` when it encounters
    /// a bare token or `--` separator, or `Done` when all tokens are consumed.
    fn parse_tokens(
        &self,
        args: &[String],
        flag_values: &mut HashMap<String, bool>,
        option_values: &mut HashMap<String, Vec<String>>,
    ) -> Result<TokenAction, ParseError> {
        let mut i = 0;
        while i < args.len() {
            let token = &args[i];

            if token == "--" {
                return Ok(TokenAction::RestPositional { index: i });
            }

            if let Some(result) = self.handle_builtin(token) {
                return result.map(|_| TokenAction::Done);
            }

            if let Some(after) = token.strip_prefix("--") {
                self.handle_long_arg(after, token, args, &mut i, flag_values, option_values)?;
            } else if token.starts_with('-') && token.len() > 1 {
                self.handle_short_arg(token, args, &mut i, flag_values, option_values)?;
            } else {
                return Ok(TokenAction::Positional {
                    index: i,
                    token: token.clone(),
                });
            }

            i += 1;
        }

        Ok(TokenAction::Done)
    }

    /// Check for built-in `--help`, `-h`, and `--version` tokens.
    /// Returns `Some(Err(...))` if the token is a built-in, `None` otherwise.
    fn handle_builtin(&self, token: &str) -> Option<Result<(), ParseError>> {
        if token == "--help" || token == "-h" {
            return Some(Err(ParseError::HelpRequested(self.help_text())));
        }
        if token == "--version" {
            return Some(Err(if let Some(text) = self.version_text() {
                ParseError::VersionRequested(text)
            } else {
                ParseError::UnknownArgument(token.to_string())
            }));
        }
        None
    }

    /// Handle a `--long` or `--long=value` token.
    fn handle_long_arg(
        &self,
        after: &str,
        full_token: &str,
        args: &[String],
        i: &mut usize,
        flag_values: &mut HashMap<String, bool>,
        option_values: &mut HashMap<String, Vec<String>>,
    ) -> Result<(), ParseError> {
        if let Some(eq_pos) = after.find('=') {
            let key = &after[..eq_pos];
            let value = &after[eq_pos + 1..];

            if let Some(opt) = self.options.iter().find(|o| o.long == key) {
                Self::store_option_value(option_values, key, value.to_string(), opt.multi, &opt.validator.clone())?;
            } else {
                return Err(ParseError::UnknownArgument(full_token.to_string()));
            }
        } else {
            let key = after;

            if self.flags.iter().any(|f| f.long == key) {
                flag_values.insert(key.to_string(), true);
            } else if let Some(opt) = self.options.iter().find(|o| o.long == key) {
                if *i + 1 >= args.len() {
                    return Err(ParseError::MissingValue(key.to_string()));
                }
                *i += 1;
                Self::store_option_value(option_values, key, args[*i].clone(), opt.multi, &opt.validator.clone())?;
            } else {
                return Err(ParseError::UnknownArgument(full_token.to_string()));
            }
        }
        Ok(())
    }

    /// Handle a short-form token: `-x`, `-x=val`, or combined `-abc` / `-oValue`.
    fn handle_short_arg(
        &self,
        token: &str,
        args: &[String],
        i: &mut usize,
        flag_values: &mut HashMap<String, bool>,
        option_values: &mut HashMap<String, Vec<String>>,
    ) -> Result<(), ParseError> {
        let after = &token[1..];

        if let Some(eq_pos) = after.find('=') {
            self.handle_short_eq(after, eq_pos, token, flag_values, option_values)
        } else if after.len() == 1 {
            self.handle_single_short(after, token, args, i, flag_values, option_values)
        } else {
            self.handle_combined_short_flags(after, token, args, i, flag_values, option_values)
        }
    }

    /// Handle `-x=value` syntax.
    fn handle_short_eq(
        &self,
        after: &str,
        eq_pos: usize,
        full_token: &str,
        flag_values: &mut HashMap<String, bool>,
        option_values: &mut HashMap<String, Vec<String>>,
    ) -> Result<(), ParseError> {
        let key_str = &after[..eq_pos];
        let value = &after[eq_pos + 1..];
        let chars: Vec<char> = key_str.chars().collect();

        // Iterate through all characters except the last: must be registered flags.
        for &ch in &chars[..chars.len() - 1] {
            if let Some(flag) = self.flags.iter().find(|f| f.short == Some(ch)) {
                flag_values.insert(flag.long.clone(), true);
            } else {
                return Err(ParseError::UnknownArgument(full_token.to_string()));
            }
        }

        // Last character must be a registered option.
        let last = *chars.last().unwrap();
        if let Some(opt) = self.options.iter().find(|o| o.short == Some(last)) {
            Self::store_option_value(
                option_values,
                &opt.long,
                value.to_string(),
                opt.multi,
                &opt.validator.clone(),
            )?;
        } else {
            return Err(ParseError::UnknownArgument(full_token.to_string()));
        }
        Ok(())
    }

    /// Handle a single short flag or option: `-x` or `-x value`.
    fn handle_single_short(
        &self,
        after: &str,
        full_token: &str,
        args: &[String],
        i: &mut usize,
        flag_values: &mut HashMap<String, bool>,
        option_values: &mut HashMap<String, Vec<String>>,
    ) -> Result<(), ParseError> {
        let ch = after.chars().next().unwrap();

        if ch == 'V' && self.version.is_some() {
            return Err(ParseError::VersionRequested(self.version_text().unwrap()));
        }

        if let Some(flag) = self.flags.iter().find(|f| f.short == Some(ch)) {
            flag_values.insert(flag.long.clone(), true);
        } else if let Some(opt) = self.options.iter().find(|o| o.short == Some(ch)) {
            if *i + 1 >= args.len() {
                return Err(ParseError::MissingValue(opt.long.clone()));
            }
            *i += 1;
            Self::store_option_value(
                option_values,
                &opt.long,
                args[*i].clone(),
                opt.multi,
                &opt.validator.clone(),
            )?;
        } else {
            return Err(ParseError::UnknownArgument(full_token.to_string()));
        }
        Ok(())
    }

    /// Handle combined short flags and attached values: `-abc`, `-oValue`.
    /// Walks each character, treating flags as combinable and options as
    /// consuming the remaining characters (or the next argument token).
    fn handle_combined_short_flags(
        &self,
        after: &str,
        full_token: &str,
        args: &[String],
        i: &mut usize,
        flag_values: &mut HashMap<String, bool>,
        option_values: &mut HashMap<String, Vec<String>>,
    ) -> Result<(), ParseError> {
        let chars: Vec<char> = after.chars().collect();
        let mut j = 0;
        while j < chars.len() {
            let ch = chars[j];

            // Built-in -h help when 'h' is not a user-registered flag
            if ch == 'h' && self.flags.iter().all(|f| f.short != Some('h')) {
                return Err(ParseError::HelpRequested(self.help_text()));
            }
            // Built-in -V version when version is configured
            if ch == 'V' && self.version.is_some() {
                return Err(ParseError::VersionRequested(self.version_text().unwrap()));
            }

            if let Some(flag) = self.flags.iter().find(|f| f.short == Some(ch)) {
                flag_values.insert(flag.long.clone(), true);
                j += 1;
            } else if let Some(opt) = self.options.iter().find(|o| o.short == Some(ch)) {
                if j + 1 < chars.len() {
                    let value: String = chars[j + 1..].iter().collect();
                    Self::store_option_value(option_values, &opt.long, value, opt.multi, &opt.validator.clone())?;
                } else {
                    if *i + 1 >= args.len() {
                        return Err(ParseError::MissingValue(opt.long.clone()));
                    }
                    *i += 1;
                    Self::store_option_value(
                        option_values,
                        &opt.long,
                        args[*i].clone(),
                        opt.multi,
                        &opt.validator.clone(),
                    )?;
                }
                break;
            } else {
                return Err(ParseError::UnknownArgument(format!(
                    "unrecognized '{}' in '{}'",
                    ch, full_token
                )));
            }
        }
        Ok(())
    }

    /// Apply env var fallback, defaults, and required checks for all options.
    fn apply_option_fallbacks(
        options: &[OptionDef],
        values: &mut HashMap<String, Vec<String>>,
    ) -> Result<(), ParseError> {
        for opt in options {
            let has_values = values.get(&opt.long).is_some_and(|v| !v.is_empty());
            if !has_values {
                // Try env var fallback
                if let Some(ref var_name) = opt.env_var {
                    if let Ok(val) = std::env::var(var_name) {
                        if !val.is_empty() {
                            let resolved = if opt.multi {
                                val.split(',').filter(|s| !s.is_empty()).map(String::from).collect()
                            } else {
                                vec![val]
                            };
                            if !resolved.is_empty() {
                                for v in &resolved {
                                    Self::validate_value(&opt.long, v, &opt.validator)?;
                                }
                                values.insert(opt.long.clone(), resolved);
                                continue;
                            }
                        }
                    }
                }
                // Try default
                if let Some(ref default) = opt.default {
                    Self::validate_value(&opt.long, default, &opt.validator)?;
                    values.insert(opt.long.clone(), vec![default.clone()]);
                } else if opt.required {
                    return Err(ParseError::MissingRequired(opt.long.clone()));
                }
            }
        }
        Ok(())
    }

    /// Original parse logic for parsers without subcommands.
    fn parse_no_subcommands(&self, args: Vec<String>) -> Result<ParseResult, ParseError> {
        let mut flag_values: HashMap<String, bool> = HashMap::new();
        let mut option_values: HashMap<String, Vec<String>> = HashMap::new();
        let mut positional_values: Vec<String> = Vec::new();

        // Initialize all flags to false
        for flag in &self.flags {
            flag_values.insert(flag.long.clone(), false);
        }

        let mut offset = 0;

        loop {
            match self.parse_tokens(&args[offset..], &mut flag_values, &mut option_values)? {
                TokenAction::Done => break,
                TokenAction::Positional { index, token } => {
                    positional_values.push(token);
                    offset += index + 1;
                }
                TokenAction::RestPositional { index } => {
                    positional_values.extend(args[offset + index + 1..].iter().cloned());
                    break;
                }
            }
        }

        Self::apply_option_fallbacks(&self.options, &mut option_values)?;

        // Apply defaults for missing positionals
        for (idx, pos) in self.positionals.iter().enumerate() {
            if idx >= positional_values.len() {
                if let Some(ref default) = pos.default {
                    while positional_values.len() < idx {
                        positional_values.push(String::new());
                    }
                    positional_values.push(default.clone());
                }
            }
        }

        // Post-parse validation: required positionals
        for (idx, pos) in self.positionals.iter().enumerate() {
            if pos.required && idx >= positional_values.len() {
                return Err(ParseError::MissingRequired(pos.name.clone()));
            }
        }

        // Validate positional values
        for (idx, pos) in self.positionals.iter().enumerate() {
            if idx < positional_values.len() {
                Self::validate_value(&pos.name, &positional_values[idx], &pos.validator)?;
            }
        }

        Ok(ParseResult::new(
            flag_values,
            option_values,
            positional_values,
            None,
            None,
        ))
    }

    /// Subcommand-aware parsing: parse global flags/options, then dispatch to subcommand parser.
    ///
    /// The first bare token (not a flag or option) is treated as the subcommand name.
    /// Parent-level positional arguments are not supported when subcommands are
    /// registered — this follows git-style conventions where the subcommand must
    /// appear before any subcommand-specific arguments.
    fn parse_with_subcommands(&self, args: Vec<String>) -> Result<ParseResult, ParseError> {
        let mut global_flags: HashMap<String, bool> = HashMap::new();
        let mut global_options: HashMap<String, Vec<String>> = HashMap::new();

        match self.parse_tokens(&args, &mut global_flags, &mut global_options)? {
            TokenAction::Positional { index, token } => {
                // First bare token: subcommand lookup
                if let Some(subcmd) = self.subcommands.iter().find(|s| s.name == token) {
                    let remaining: Vec<String> = args[index + 1..].to_vec();
                    let sub_result = subcmd.parser.parse(remaining)?;

                    Self::apply_option_fallbacks(&self.options, &mut global_options)?;

                    Ok(ParseResult::new(
                        global_flags,
                        global_options,
                        vec![],
                        Some(subcmd.name.clone()),
                        Some(Box::new(sub_result)),
                    ))
                } else {
                    Err(ParseError::UnknownSubcommand(token))
                }
            }
            TokenAction::Done | TokenAction::RestPositional { .. } => {
                // Reached end without finding a subcommand token
                let names: Vec<&str> = self.subcommands.iter().map(|s| s.name.as_str()).collect();
                Err(ParseError::NoSubcommand(names.join(", ")))
            }
        }
    }

    /// Parse arguments directly from `std::env::args()`, skipping the
    /// program name (argv[0]).
    ///
    /// This is the recommended entry point for real CLI applications.
    /// Returns `Err(ParseError::InvalidUtf8(_))` if any argument contains
    /// bytes that are not valid UTF-8.
    pub fn parse_env(&self) -> Result<ParseResult, ParseError> {
        let mut args = Vec::new();
        for os_arg in std::env::args_os().skip(1) {
            match os_arg.into_string() {
                Ok(s) => args.push(s),
                Err(bad) => return Err(ParseError::InvalidUtf8(bad.to_string_lossy().into_owned())),
            }
        }
        self.parse(args)
    }
}
