use std::collections::HashMap;

use crate::result::ParseResult;
use crate::types::ParseError;

/// Schema-free parse of `std::env::args()` for quick-and-dirty scripts.
///
/// **This is a convenience escape hatch, not the recommended way to parse
/// arguments.** It exists for throwaway scripts where defining a full schema
/// via [`ArgBuilder`](crate::ArgBuilder) feels like overkill. For anything
/// user-facing or longer-lived, prefer the schema-based API — it gives you
/// validation, required arguments, defaults, env-var fallback, help text, and
/// none of the footguns listed below.
///
/// # Heuristic behavior
///
/// Because there is no schema, the parser guesses whether a `--key` token is
/// a flag or an option: if the next token exists and does **not** start with
/// `-`, it is consumed as the value. This means:
///
/// - `--output file.txt` → option `output = "file.txt"`
/// - `--verbose --debug` → two flags (`verbose`, `debug`)
/// - `--output -v` → `--output` becomes a **flag** (not an option), because
///   `-v` starts with `-` and is not consumed. `-v` is then parsed separately.
///   This is the main footgun: if you meant `--output` to take a value that
///   happens to look like a flag, it silently gets the wrong answer.
///
/// # When to use
///
/// Only for quick, disposable scripts where you control all the inputs and
/// don't need help text, required-argument checks, or typed parsing. If any
/// of your options could plausibly receive a value that starts with `-`, use
/// [`ArgBuilder`](crate::ArgBuilder) instead.
pub fn parse_loose() -> Result<ParseResult, ParseError> {
    let mut args = Vec::new();
    for os_arg in std::env::args_os().skip(1) {
        match os_arg.into_string() {
            Ok(s) => args.push(s),
            Err(bad) => return Err(ParseError::InvalidUtf8(bad.to_string_lossy().into_owned())),
        }
    }
    parse_loose_from(args)
}

/// Internal implementation for schema-free parsing of an argument list.
///
/// Uses a heuristic: `--key` is treated as an option if the next token exists
/// and does not start with `-`; otherwise it is treated as a flag. This can
/// misclassify edge cases (e.g. `--output -v` treats `--output` as a flag
/// and `-v` as a separate token). Callers who need exact semantics should use
/// [`ArgBuilder`](crate::ArgBuilder).
pub(crate) fn parse_loose_from(args: Vec<String>) -> Result<ParseResult, ParseError> {
    let mut flag_values: HashMap<String, bool> = HashMap::new();
    let mut option_values: HashMap<String, Option<String>> = HashMap::new();
    let mut positional_values: Vec<String> = Vec::new();
    let mut rest_are_positional = false;
    let mut i = 0;

    while i < args.len() {
        let token = &args[i];

        if rest_are_positional {
            positional_values.push(token.clone());
            i += 1;
            continue;
        }

        if token == "--" {
            rest_are_positional = true;
            i += 1;
            continue;
        }

        if token == "--help" || token == "-h" {
            return Err(ParseError::HelpRequested(String::from(
                "Usage: [FLAGS] [OPTIONS] [ARGS]\n",
            )));
        }

        if let Some(after) = token.strip_prefix("--") {
            if let Some(eq_pos) = after.find('=') {
                let key = &after[..eq_pos];
                let value = &after[eq_pos + 1..];
                option_values.insert(key.to_string(), Some(value.to_string()));
            } else {
                let key = after;
                // Peek ahead: if next token exists and doesn't start with '-', treat as option value
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    option_values.insert(key.to_string(), Some(args[i].clone()));
                } else {
                    // No value follows — treat as flag
                    flag_values.insert(key.to_string(), true);
                }
            }
        } else if token.starts_with('-') && token.len() > 1 {
            let after = &token[1..];

            if let Some(eq_pos) = after.find('=') {
                let key = &after[..eq_pos];
                let value = &after[eq_pos + 1..];
                option_values.insert(key.to_string(), Some(value.to_string()));
            } else {
                let key = after;
                if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                    i += 1;
                    option_values.insert(key.to_string(), Some(args[i].clone()));
                } else {
                    flag_values.insert(key.to_string(), true);
                }
            }
        } else {
            positional_values.push(token.clone());
        }

        i += 1;
    }

    Ok(ParseResult::new(
        flag_values,
        option_values.into_iter().filter_map(|(k, v)| v.map(|val| (k, vec![val]))).collect(),
        positional_values,
        None,
        None,
    ))
}
